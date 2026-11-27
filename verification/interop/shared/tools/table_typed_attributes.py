#!/usr/bin/env python3
"""Run the installed typed attribute APIs against the shared KMIP table values."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from xml.etree.ElementTree import fromstring

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.cosmian_minimum import ALLOWED_VERSIONS
from verification.interop.shared.tools.oasis_ttlv import WireValues, encode
from verification.interop.shared.tools.table_key_material import validate_provenance
from verification.interop.shared.tools.table_ttlv import field

SYMBOLS = {
    "rust": ("kmipkit::protocol::Attribute::new", "kmipkit::protocol::Attribute::from_item"),
    "c": ("kmipkit_typed_attribute_from_value", "kmipkit_typed_attribute_from_item"),
    "java": ("org.kmipkit.ttlv.TypedAttribute.create", "org.kmipkit.ttlv.TypedAttribute.fromItem"),
    "python": ("kmipkit.typed_attributes.TypedAttribute.create", "kmipkit.typed_attributes.TypedAttribute.from_item"),
}


def cases(root, versions=ALLOWED_VERSIONS):
    fixtures = fromstring((root / "verification/interop/shared/vectors/kmip2x-table-attributes.xml").read_bytes())
    for version in versions:
        names = WireValues(root, version, [])
        for entry in fixtures:
            row = entry.attrib["row"]
            # Certificate Attributes denotes a set of flattened certificate Name
            # fields. Protection Storage Masks has its own request context.
            if row == "Certificate Attributes" or (entry.get("since") == "2.1" and version == "2.0"):
                continue
            attribute, = entry
            wire, _ = encode(attribute, names)
            tag = int.from_bytes(wire[:3], "big")
            wrong_tag = names.tags["comment" if attribute.tag == "Description" else "description"]
            wrong_value = ("Boolean", "true") if attribute.get("type") == "TextString" else ("TextString", "wrong-type")
            mistyped, _ = encode(field(attribute.tag, *wrong_value), names)
            for scenario, input_wire in (
                ("value", wire),
                ("wrong-tag", wrong_tag.to_bytes(3, "big") + wire[3:]),
                ("wrong-type", mistyped),
            ):
                yield {
                    "protocol_version": version,
                    "row": row,
                    "fixture": f"{attribute.tag}-{scenario}",
                    "fixture_sha256": hashlib.sha256(input_wire).hexdigest(),
                    "tag": tag,
                    "scenario": scenario,
                }, f"{tag} {input_wire.hex()} {wire.hex()}\n"


def expected_checks(scenario, language):
    return {
        "construct": ("not-applicable" if language == "rust" else "rejected") if scenario == "wrong-type" else "passed",
        "read": "passed" if scenario == "value" else "rejected",
    }


def collect(root, runners):
    fixtures = list(cases(root))
    input_text = "".join(line for _, line in fixtures)
    records = [{**fixture, "checks": {}} for fixture, _ in fixtures]
    artifacts = {}
    for language, executable in runners.items():
        result = subprocess.run(
            [str(executable.resolve(strict=True)), "--inspect-table-typed-attributes"],
            input=input_text, text=True, capture_output=True, check=False,
        )
        lines = result.stdout.splitlines()
        if result.returncode != 0 or len(lines) != len(records) + 1:
            raise ValueError(f"{language}: incomplete typed attribute results ({len(lines)} output lines)")
        artifact = json.loads(lines[0])
        validate_provenance(artifact)
        artifacts[language] = artifact
        for record, line in zip(records, lines[1:], strict=True):
            checks = json.loads(line)
            validate_checks(checks)
            record["checks"][language] = checks
    return {"scope": "local", "api_level": "typed-protocol", "artifacts": artifacts, "fixtures": records}


def validate_checks(checks):
    if set(checks) != {"construct", "read"} or not set(checks.values()) <= {
        "passed", "rejected", "failed", "not-applicable",
    } or checks["read"] == "not-applicable":
        raise ValueError("invalid typed attribute result")


def attach_evidence(root, inventory, paths, languages, expected_commit):
    version = inventory["protocol_version"]
    expected = {fixture["fixture"]: fixture for fixture, _ in cases(root, (version,))}
    rows = {row["name"]: row for row in inventory["rows"] if row["kind"] == "attribute"}
    seen = set()
    for path in paths:
        report = json.loads(path.read_text(encoding="utf-8"))
        if (report["scope"], report["api_level"]) != ("local", "typed-protocol"):
            raise ValueError(f"{path.name}: expected local typed attribute evidence")
        records = [record for record in report["fixtures"] if record["protocol_version"] == version]
        if sorted(record["fixture"] for record in records) != sorted(expected):
            raise ValueError(f"{path.name}: incomplete typed attribute evidence for KMIP {version}")
        for language, artifact in report["artifacts"].items():
            if language not in languages:
                continue
            validate_provenance(artifact, expected_commit)
            construct, read = SYMBOLS[language]
            for record in records:
                fixture = expected[record["fixture"]]
                identity = language, artifact["artifact_sha256"], record["fixture"]
                if identity in seen or {key: value for key, value in record.items() if key != "checks"} != fixture:
                    raise ValueError(f"{path.name}: mismatched or duplicate typed attribute fixture {record['fixture']}")
                seen.add(identity)
                checks = record["checks"][language]
                validate_checks(checks)
                obligation = next(item for item in rows[record["row"]]["obligations"]
                                  if (item["language"], item["api_level"]) == (language, "typed-protocol"))
                obligation.setdefault("local_typed_attribute_results", []).append({
                    **fixture, **artifact, "scope": "local", "public_api": {"construct": construct, "read": read},
                    "checks": checks, "expected_checks": expected_checks(fixture["scenario"], language),
                })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", nargs=2, metavar=("LANGUAGE", "EXECUTABLE"), action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    runners = {language: Path(executable) for language, executable in args.runner}
    if len(runners) != len(args.runner) or not runners.keys() <= SYMBOLS.keys():
        parser.error("give one runner per supported language")
    try:
        report = collect(ROOT, runners)
        with args.output.open("x", encoding="utf-8") as output:
            json.dump(report, output, indent=2)
            output.write("\n")
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(2, f"Typed attribute check failed: {error}\n")
    failures = [(case["protocol_version"], case["fixture"], language, checks) for case in report["fixtures"]
                for language, checks in case["checks"].items()
                if checks != expected_checks(case["scenario"], language)]
    for version, fixture, language, checks in failures:
        print(f"{language} KMIP {version} {fixture}: {checks}", file=sys.stderr)
    print(f"Checked {len(report['fixtures'])} typed attribute fixtures with {len(runners)} installed validators")
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
