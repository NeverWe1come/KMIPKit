#!/usr/bin/env python3
"""Collect planned interoperability evidence from the installed C ABI."""

import argparse
import json
from pathlib import Path
import subprocess
import uuid

from artifact_support import PIN, verify_installed_artifact
from query_campaign import live_query


PROBE = Path(__file__).resolve().parents[1] / "build" / "kmip30_c_constructors"


def local_events(plan):
    artifact = verify_installed_artifact()
    completed = subprocess.run([str(PROBE)], text=True, capture_output=True, check=False)
    commit, digest, *results = completed.stdout.splitlines()
    if commit != PIN or artifact["files"].get("lib/libkmipkit_ffi.so") != digest:
        raise ValueError("The C probe was built with a different library artifact")
    outcomes = dict(result.split() for result in results)
    events = []
    for row in plan:
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
        raise ValueError("The plan contains no supported C draft constructor obligations")
    return commit, digest, events


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--matrix-sha256", required=True)
    parser.add_argument("--scope", choices=("local", "live"), required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--profile", type=Path)
    parser.add_argument("--connection", type=Path)
    args = parser.parse_args()

    plan_path = args.plan.resolve(strict=True)
    if plan_path.name != "plan.jsonl" or args.evidence.parent.resolve(strict=True) != plan_path.parent:
        parser.error("Campaign evidence must be written beside the shared plan.jsonl")
    match args.scope:
        case "local":
            evidence = plan_path.with_name("evidence-c-local.json")
        case "live":
            evidence = plan_path.with_name("evidence-c-live.json")
        case _:
            parser.error("Invalid C campaign scope")
    if args.evidence.name != evidence.name:
        parser.error("Evidence filename must match the campaign scope")

    try:
        with plan_path.open(encoding="utf-8") as source:
            rows = (json.loads(line) for line in source)
            if args.scope == "local":
                if args.connection is not None:
                    raise ValueError("Local campaigns do not take a connection")
                commit, digest, events = local_events(rows)
                server = None
            else:
                if args.connection is None:
                    raise ValueError("Live C campaigns require a connection")
                artifact = verify_installed_artifact()
                server, provenance, event = live_query(rows, args.connection)
                commit, digest = provenance["library_commit"], provenance["artifact_sha256"]
                if commit != PIN or artifact["files"].get("lib/libkmipkit_ffi.so") != digest:
                    raise ValueError("The C probe was built with a different library artifact")
                events = [event]
        payload = {
            "schema_version": "1.0", "scope": args.scope, "language": "c", "run_id": str(uuid.uuid4()),
            "library_commit": commit, "artifact_sha256": digest,
            "matrix_sha256": args.matrix_sha256, "events": events,
        }
        if server is not None:
            payload["server"] = server
        with evidence.open("x", encoding="utf-8") as output:
            json.dump(payload, output)
            output.write("\n")
    except (OSError, KeyError, ValueError):
        parser.exit(2, "C campaign failed. Check the plan, installed artifact and connection.\n")
    return int(any(event["status"] == "failed" for event in events))


if __name__ == "__main__":
    raise SystemExit(main())
