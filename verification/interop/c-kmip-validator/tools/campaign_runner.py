#!/usr/bin/env python3
"""Collect planned interoperability evidence from the installed C ABI."""

import argparse
import json
from pathlib import Path
import re
import subprocess
import uuid

from artifact_support import PIN, verify_installed_artifact
from query_campaign import live_query


PROBE = Path(__file__).resolve().parents[1] / "build" / "kmip30_c_constructors"
CATALOG = PROBE.parents[2] / "shared" / "catalog" / "kmipkit-2x-matrix-catalog.json"
TAG_ROUTES = {
    "kmipkit_ttlv_raw_tag_create": (0, "generic_ttlv_raw_tag_value"),
    "kmipkit_ttlv_raw_tag_try_checked": (1, "generic_ttlv_checked_tag_value"),
}


def run_probe(artifact, *args, source=None):
    completed = subprocess.run([str(PROBE), *args], text=True, input=source, capture_output=True, check=False)
    commit, digest, *results = completed.stdout.splitlines()
    if commit != PIN or artifact["files"].get("lib/libkmipkit_ffi.so") != digest:
        raise ValueError("The C probe was built with a different library artifact")
    if args and completed.returncode:
        raise ValueError("The C tag probe could not consume the planned targets")
    return commit, digest, results


def local_events(plan):
    artifact = verify_installed_artifact()
    commit, digest, results = run_probe(artifact)
    outcomes = dict(result.split() for result in results)
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    allocations = {target["target_id"]: target["allocation"] for target in catalog["normative_targets"]
                   if target["kind"] == "tag"}
    events = []
    tags = {}
    for row in plan:
        if (row.get("language"), row.get("action"), row.get("execution_status")) != ("c", "construct", "required"):
            continue
        symbol = row["public_api_symbol"]
        if (row.get("protocol_version") in {"2.0", "2.1"} and row.get("mode") == "local"
                and row.get("case_id") == "test-enum-tag" and symbol in TAG_ROUTES):
            target = re.fullmatch(r"KMIPKIT-ELEM-TAG-([0-9a-fA-F]{6})", row["target_id"])
            if target and row["target_id"] in allocations:
                source = f"{target[1]} {int(allocations[row['target_id']] == 'assigned')}"
                tags.setdefault(source, []).append(row)
            continue
        if (row.get("language"), row.get("protocol_version"), row.get("kind"),
            row.get("action"), row.get("execution_status")) != ("c", "3.0", "public_route", "construct", "required"):
            continue
        if symbol in outcomes:
            events.append({
                "obligation_id": row["obligation_id"], "public_api_symbol": symbol,
                "status": outcomes[symbol], "assertion": "draft_operation_code",
            })
    if tags:
        _, _, results = run_probe(artifact, "--tags", source="\n".join(tags) + "\n")
        for rows, outcome in zip(tags.values(), results, strict=True):
            statuses = outcome.split()
            for row in rows:
                index, assertion = TAG_ROUTES[row["public_api_symbol"]]
                events.append({
                    "obligation_id": row["obligation_id"], "public_api_symbol": row["public_api_symbol"],
                    "status": statuses[index], "assertion": assertion,
                })
    if not events:
        raise ValueError("The plan contains no supported C local obligations")
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
