#!/usr/bin/env python3
"""Verify generic TTLV mode runs locally and never opens a KMIP connection."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def main() -> int:
    binary, repository = Path(sys.argv[1]), Path(sys.argv[2])
    catalog = repository / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"
    with tempfile.TemporaryDirectory(prefix="kmipkit-c-offline-") as raw_directory:
        directory = Path(raw_directory)
        connection = {
            "server_id": "offline-generic-route",
            "server_vendor": "",
            "server_product": "offline test target",
            "server_version": "fixture-1",
            "server_configuration_fingerprint": hashlib.sha256(b"offline-generic-route").hexdigest(),
            "protocol_version": "2.1",
            "transport": "raw_tls",
            "encoding": "TTLV",
            "api_level": "generic-ttlv",
            "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": 1}},
            "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
            "execution_policy": {"probe_unsupported": True, "allow_mutations": False},
        }
        connection_path = directory / "connection.json"
        result_path = directory / "results.json"
        connection_path.write_text(json.dumps(connection), encoding="utf-8")
        preflight = subprocess.run(
            [str(binary), "preflight", "--connection", str(connection_path)],
            cwd=repository,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
        if preflight.returncode != 0 or "no DNS or network exchange" not in preflight.stdout:
            raise AssertionError("generic TTLV preflight must not require TLS or open a network connection")
        result = subprocess.run(
            [str(binary), "run", "--connection", str(connection_path), "--catalog", str(catalog),
             "--output", str(result_path)],
            cwd=repository,
            capture_output=True,
            text=True,
            timeout=10,
            check=False,
        )
        if result.returncode != 0:
            raise AssertionError(f"offline generic TTLV run failed: {result.stderr[-300:]!r}")
        report = json.loads(result_path.read_text(encoding="utf-8"))
        if report["summary"]["passed"] != 67 or report["summary"]["failed"] != 0:
            raise AssertionError("generic TTLV mode must execute and verify the 67 local value cases")
        if report["summary"]["not_run"] == 0 or report["summary"]["policy_restricted"] == 0:
            raise AssertionError("server exchange and mutation states must remain explicit in offline mode")
        for case in report["cases"]:
            if case["status"] in ("not_run", "policy_restricted") and not case.get("diagnostic"):
                raise AssertionError("every unavailable case must include a sanitized reason")
        if report["execution"]["api_level"] != "generic-ttlv":
            raise AssertionError("result metadata must state the selected local-only API route")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
