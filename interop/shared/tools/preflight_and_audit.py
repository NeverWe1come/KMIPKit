#!/usr/bin/env python3
"""Pre-flight connection diagnosis and capability profile auditor for KMIPKit interop."""

from __future__ import annotations

import argparse
import json
import socket
import ssl
import sys
from pathlib import Path


def validate_connection_config(config_path: Path, check_network: bool = False, timeout_s: float = 3.0) -> dict:
    resolved_path = config_path.resolve()
    if not resolved_path.exists():
        return {"valid": False, "error": f"Connection configuration not found at {resolved_path}"}

    try:
        with open(resolved_path, "r", encoding="utf-8") as f:  # NOSONAR
            config = json.load(f)
    except Exception as exc:
        return {"valid": False, "error": f"Failed to parse JSON: {exc}"}

    server_id = config.get("server_id")
    if not server_id:
        return {"valid": False, "error": "Missing 'server_id' in connection configuration"}

    tls_cfg = config.get("tls", {})
    file_checks = {}
    for key in ("ca_cert_path", "client_cert_path", "client_key_path"):
        p_str = tls_cfg.get(key)
        if p_str:
            p = (resolved_path.parent / p_str).resolve() if not Path(p_str).is_absolute() else Path(p_str).resolve()
            file_checks[key] = {
                "configured_path": p_str,
                "resolved_path": str(p),
                "exists": p.exists(),
                "readable": p.is_file() and p.stat().st_size > 0 if p.exists() else False
            }

    network_check = None
    if check_network:
        endpoints = config.get("endpoints", {})
        raw_tls = endpoints.get("raw_tls")
        if raw_tls:
            host = raw_tls.get("host")
            port = raw_tls.get("port")
            try:
                sock = socket.create_connection((host, port), timeout=timeout_s)
                sock.close()
                network_check = {"reachable": True, "host": host, "port": port}
            except Exception as net_err:
                network_check = {"reachable": False, "host": host, "port": port, "error": str(net_err)}

    return {
        "valid": True,
        "server_id": server_id,
        "file_checks": file_checks,
        "network_check": network_check
    }


def audit_profile_diff(profile_path: Path, actual_capabilities_path: Path) -> dict:
    resolved_prof = profile_path.resolve()
    resolved_actual = actual_capabilities_path.resolve()

    if not resolved_prof.exists():
        return {"error": f"Profile not found at {resolved_prof}"}
    if not resolved_actual.exists():
        return {"error": f"Actual capabilities not found at {resolved_actual}"}

    with open(resolved_prof, "r", encoding="utf-8") as f:  # NOSONAR
        prof = json.load(f)
    with open(resolved_actual, "r", encoding="utf-8") as f:  # NOSONAR
        actual = json.load(f)

    discrepancies = []

    # 1. Compare operations
    prof_ops = prof.get("operations", {})
    actual_ops = actual.get("operations", {})
    all_ops = sorted(set(prof_ops.keys()) | set(actual_ops.keys()))
    for op in all_ops:
        p_val = prof_ops.get(op, {}).get("supported", False)
        a_val = actual_ops.get(op, {}).get("supported", False)
        if p_val != a_val:
            discrepancies.append({
                "category": "operation",
                "item": op,
                "profile_supported": p_val,
                "actual_supported": a_val
            })

    # 2. Compare object types
    prof_obj = prof.get("object_types", {})
    actual_obj = actual.get("object_types", {})
    all_objs = sorted(set(prof_obj.keys()) | set(actual_obj.keys()))
    for obj in all_objs:
        p_val = prof_obj.get(obj, {}).get("supported", False)
        a_val = actual_obj.get(obj, {}).get("supported", False)
        if p_val != a_val:
            discrepancies.append({
                "category": "object_type",
                "item": obj,
                "profile_supported": p_val,
                "actual_supported": a_val
            })

    # 3. Compare attributes
    prof_attr = prof.get("attributes", {})
    actual_attr = actual.get("attributes", {})
    all_attrs = sorted(set(prof_attr.keys()) | set(actual_attr.keys()))
    for attr in all_attrs:
        p_val = prof_attr.get(attr, {}).get("supported", False)
        a_val = actual_attr.get(attr, {}).get("supported", False)
        if p_val != a_val:
            discrepancies.append({
                "category": "attribute",
                "item": attr,
                "profile_supported": p_val,
                "actual_supported": a_val
            })

    return {
        "profile_server": prof.get("server_identifier"),
        "actual_server": actual.get("server_identifier"),
        "total_discrepancies": len(discrepancies),
        "discrepancies": discrepancies
    }


def main():
    parser = argparse.ArgumentParser(description="KMIPKit Pre-flight Diagnoser and Profile Auditor")
    subparsers = parser.add_subparsers(dest="command")

    # validate-config
    cfg_parser = subparsers.add_parser("validate-config", help="Validate connection configuration and certificate files")
    cfg_parser.add_argument("--config", required=True, type=Path, help="Path to server connection JSON")
    cfg_parser.add_argument("--check-network", action="store_true", help="Attempt TCP connection to server endpoint")
    cfg_parser.add_argument("--timeout", type=float, default=3.0, help="Connection timeout in seconds")

    # profile-diff
    diff_parser = subparsers.add_parser("profile-diff", help="Audit discrepancies between static profile and actual capabilities")
    diff_parser.add_argument("--profile", required=True, type=Path, help="Path to static profile JSON")
    diff_parser.add_argument("--actual", required=True, type=Path, help="Path to actual server capabilities JSON")

    args = parser.parse_args()

    if args.command == "validate-config":
        res = validate_connection_config(args.config, check_network=args.check_network, timeout_s=args.timeout)
        print(json.dumps(res, indent=2))
        if not res.get("valid"):
            sys.exit(1)
        # Check files
        for fkey, finfo in res.get("file_checks", {}).items():
            if not finfo.get("readable"):
                print(f"Warning: {fkey} ({finfo.get('configured_path')}) is not readable or missing", file=sys.stderr)
        if args.check_network and res.get("network_check") and not res["network_check"].get("reachable"):
            print(f"Network error: cannot reach {res['network_check'].get('host')}:{res['network_check'].get('port')}", file=sys.stderr)
            sys.exit(2)
    elif args.command == "profile-diff":
        diff = audit_profile_diff(args.profile, args.actual)
        print(json.dumps(diff, indent=2))
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
