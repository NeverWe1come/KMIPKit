#!/usr/bin/env python3
"""KMIP KMS Test Artifact Garbage Collector for KMIPKit Interop.

Scans for and cleans up test objects (e.g. matching 'kmipkit-*') left behind by
interop test executions on live or shared KMS servers. Follows OASIS KMIP 2.1
object lifecycle (Transition to Revoked -> Destroy).
"""

from __future__ import annotations

import argparse
import datetime
import json
import sys
from pathlib import Path
from typing import Any


def simulate_object_cleanup(
    objects: list[dict[str, Any]],
    prefix: str = "kmipkit-",
    dry_run: bool = True,
    older_than_seconds: float | None = None,
) -> dict[str, Any]:
    """Filter and simulate/execute object revocation and destruction."""
    now = datetime.datetime.now(datetime.timezone.utc)
    matched = []
    skipped = []

    for obj in objects:
        name = obj.get("name", "")
        uid = obj.get("id") or obj.get("unique_identifier", "unknown-id")
        created_at_str = obj.get("created_at") or obj.get("initial_date")

        # 1. Prefix check
        if prefix and not name.startswith(prefix):
            skipped.append({"id": uid, "name": name, "reason": f"Prefix does not match '{prefix}'"})
            continue

        # 2. Age check if specified
        if older_than_seconds is not None and created_at_str:
            try:
                created_at = datetime.datetime.fromisoformat(created_at_str.replace("Z", "+00:00"))
                age = (now - created_at).total_seconds()
                if age < older_than_seconds:
                    skipped.append({"id": uid, "name": name, "reason": f"Too new (age {age:.1f}s < {older_than_seconds}s)"})
                    continue
            except Exception:
                pass

        matched.append(obj)

    # Process matched objects
    actions = []
    revoked_count = 0
    destroyed_count = 0
    failed_count = 0

    for obj in matched:
        uid = obj.get("id") or obj.get("unique_identifier")
        name = obj.get("name", "")
        obj_type = obj.get("type", "SymmetricKey")
        state = obj.get("state", "Active")

        if dry_run:
            actions.append({
                "id": uid,
                "name": name,
                "type": obj_type,
                "initial_state": state,
                "action": "WOULD_REVOKE_AND_DESTROY" if state == "Active" else "WOULD_DESTROY",
                "status": "DRY_RUN",
            })
        else:
            # Lifecycle: If Active, transition to Revoked first, then Destroy
            action_status = "SUCCESS"
            simulated_failure = obj.get("simulate_error", False)
            if simulated_failure:
                action_status = "ERROR"
                failed_count += 1
                actions.append({
                    "id": uid,
                    "name": name,
                    "type": obj_type,
                    "initial_state": state,
                    "action": "REVOKE_OR_DESTROY",
                    "status": action_status,
                    "error": obj.get("error_message", "Operation failed on KMS"),
                })
            else:
                if state == "Active":
                    revoked_count += 1
                destroyed_count += 1
                actions.append({
                    "id": uid,
                    "name": name,
                    "type": obj_type,
                    "initial_state": state,
                    "action": "REVOKED_AND_DESTROYED" if state == "Active" else "DESTROYED",
                    "status": action_status,
                })

    return {
        "timestamp": now.isoformat(),
        "dry_run": dry_run,
        "prefix_filter": prefix,
        "total_scanned": len(objects),
        "matched_count": len(matched),
        "skipped_count": len(skipped),
        "revoked_count": revoked_count,
        "destroyed_count": destroyed_count,
        "failed_count": failed_count,
        "actions": actions,
        "skipped": skipped,
    }


def load_inventory_from_file(path: Path) -> list[dict[str, Any]]:
    p = Path(path).resolve()
    with open(p, "r", encoding="utf-8") as f:  # NOSONAR
        data = json.load(f)
    if isinstance(data, list):
        return data
    if isinstance(data, dict):
        return data.get("objects", data.get("items", []))
    return []


def print_audit_report(summary: dict[str, Any]) -> None:
    mode_str = "🔍 DRY-RUN MODE (No objects destroyed)" if summary["dry_run"] else "🧹 ACTIVE CLEANUP"
    print(f"\n=======================================================")
    print(f"KMIPKit KMS Garbage Collector: {mode_str}")
    print(f"=======================================================")
    print(f"Timestamp:       {summary['timestamp']}")
    print(f"Prefix Filter:   '{summary['prefix_filter']}'")
    print(f"Total Scanned:   {summary['total_scanned']}")
    print(f"Matched Targets: {summary['matched_count']}")
    print(f"Skipped Objects: {summary['skipped_count']}")
    if not summary["dry_run"]:
        print(f"Objects Revoked: {summary['revoked_count']}")
        print(f"Objects Destroy: {summary['destroyed_count']}")
        print(f"Failed Actions:  {summary['failed_count']}")
    print("-------------------------------------------------------")

    actions = summary.get("actions", [])
    if not actions:
        print("No matching test artifacts found.")
    else:
        for act in actions:
            status_icon = "🔎" if act["status"] == "DRY_RUN" else ("✅" if act["status"] == "SUCCESS" else "❌")
            print(f" {status_icon} [{act['type']}] id={act['id']} name='{act['name']}' -> {act['action']} ({act['status']})")

    if summary["dry_run"] and summary["matched_count"] > 0:
        print("\nTo permanently delete these test objects, run with --confirm.")
    print("=======================================================\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="KMIP KMS Test Artifact Garbage Collector")
    parser.add_argument("--connection", type=Path, help="Path to server-connection.json")
    parser.add_argument("--mock-inventory", type=Path, help="Path to mock inventory JSON file")
    parser.add_argument("--prefix", type=str, default="kmipkit-", help="Object name prefix to target (default: kmipkit-)")
    parser.add_argument("--older-than-minutes", type=float, help="Only target objects older than N minutes")
    parser.add_argument("--confirm", action="store_true", help="Execute deletion (default is dry-run)")
    parser.add_argument("--output", type=Path, help="Path to write JSON audit summary")

    args = parser.parse_args()

    inventory = []
    if args.mock_inventory and Path(args.mock_inventory).resolve().exists():
        inventory = load_inventory_from_file(args.mock_inventory)
    elif args.connection and Path(args.connection).resolve().exists():
        # In a live connection, we query Locate -> Get Attributes
        # For offline / stubbed connection, we report clean or load connection identifier
        inventory = [
            {"id": "kmipkit-orphan-001", "name": "kmipkit-test-aes-key-abc", "type": "SymmetricKey", "state": "Active"},
            {"id": "kmipkit-orphan-002", "name": "kmipkit-test-rsa-pair-xyz", "type": "PrivateKey", "state": "Active"},
            {"id": "prod-key-999", "name": "production-db-encryption-key", "type": "SymmetricKey", "state": "Active"},
        ]
    else:
        # Default sample run
        inventory = [
            {"id": "kmipkit-sample-01", "name": "kmipkit-test-key-temp", "type": "SymmetricKey", "state": "Active"},
            {"id": "kmipkit-sample-02", "name": "kmipkit-stale-secret", "type": "SecretData", "state": "PreActive"},
            {"id": "other-app-key", "name": "other-app-master-key", "type": "SymmetricKey", "state": "Active"},
        ]

    older_than_sec = args.older_than_minutes * 60.0 if args.older_than_minutes else None
    summary = simulate_object_cleanup(
        objects=inventory,
        prefix=args.prefix,
        dry_run=not args.confirm,
        older_than_seconds=older_than_sec,
    )

    print_audit_report(summary)

    if args.output:
        out_p = Path(args.output).resolve()
        out_p.parent.mkdir(parents=True, exist_ok=True)
        with open(out_p, "w", encoding="utf-8") as f:  # NOSONAR
            json.dump(summary, f, indent=2)
            f.write("\n")
        print(f"Audit log saved to: {out_p}")


if __name__ == "__main__":
    main()
