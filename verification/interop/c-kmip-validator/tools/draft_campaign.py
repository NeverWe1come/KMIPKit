#!/usr/bin/env python3
"""Collect local KMIP 3.0 constructor evidence from the installed C ABI."""

import argparse
import json
from pathlib import Path
import subprocess
import uuid

from artifact_support import PIN, verify_installed_artifact


PROBE = Path(__file__).resolve().parents[1] / "build" / "kmip30_c_constructors"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--matrix-sha256", required=True)
    parser.add_argument("--scope", choices=("local",), required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--profile", type=Path)
    args = parser.parse_args()

    artifact = verify_installed_artifact()
    completed = subprocess.run([str(PROBE)], text=True, capture_output=True, check=False)
    commit, digest, *results = completed.stdout.splitlines()
    if commit != PIN or artifact["files"].get("lib/libkmipkit_ffi.so") != digest:
        raise ValueError("The C probe was built with a different library artifact")
    outcomes = dict(result.split() for result in results)
    events = []
    with args.plan.open(encoding="utf-8") as plan:
        for line in plan:
            row = json.loads(line)
            if (row.get("language"), row.get("protocol_version"), row.get("kind"),
                row.get("action"), row.get("execution_status")) != ("c", "3.0", "public_route", "construct", "required"):
                continue
            symbol = row["public_api_symbol"]
            if symbol in outcomes:
                events.append({
                    "obligation_id": row["obligation_id"], "public_api_symbol": symbol,
                    "status": outcomes[symbol], "assertion": "draft_operation_code",
                })
    if not events:
        parser.exit(2, "The plan contains no supported C draft constructor obligations.\n")
    args.evidence.write_text(json.dumps({
        "schema_version": "1.0", "scope": "local", "language": "c", "run_id": str(uuid.uuid4()),
        "library_commit": commit, "artifact_sha256": digest,
        "matrix_sha256": args.matrix_sha256, "events": events,
    }) + "\n", encoding="utf-8")
    return int(completed.returncode != 0)


if __name__ == "__main__":
    raise SystemExit(main())
