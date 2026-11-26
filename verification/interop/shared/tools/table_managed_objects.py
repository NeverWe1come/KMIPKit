#!/usr/bin/env python3
"""Exercise typed managed object constructors and readers with shared table fixtures."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.cosmian_minimum import ALLOWED_VERSIONS, normal
from verification.interop.shared.tools.oasis_ttlv import WireValues, encode
from verification.interop.shared.tools.table_key_material import collect_node_checks, validate_checks, validate_provenance
from verification.interop.shared.tools.table_ttlv import (
    KEY_FORMATS, field, other_objects, structure, transparent_object_fixture,
)

OBJECT_NAMES = {"SymmetricKey": "Symmetric Key", "PublicKey": "Public Key", "PrivateKey": "Private Key"}
SYMBOLS = {
    "rust": ("kmipkit::protocol::{}::try_new", "kmipkit::protocol::ManagedObject::try_from_item"),
    "c": ("kmipkit_managed_object_from_fields", "kmipkit_managed_object_from_item"),
    "java": ("org.kmipkit.ttlv.ManagedObject.fromFields", "org.kmipkit.ttlv.ManagedObject.fromItem"),
    "python": ("kmipkit.managed_objects.ManagedObject.from_fields", "kmipkit.managed_objects.ManagedObject.from_item"),
}


def vectors(root, versions=ALLOWED_VERSIONS):
    for version in versions:
        names = WireValues(root, version, [])
        fixtures = [(name, tag, obj, normal(name)) for name, tag, obj, _ in other_objects(root, version)]
        for key_format in KEY_FORMATS:
            name, tag, *_ = key_format
            obj, _ = transparent_object_fixture(*key_format)
            fixtures.append((OBJECT_NAMES[tag], tag, obj, f"transparent-{normal(name)}"))
        for name, tag, obj, label in fixtures:
            object_type = names.symbol("ObjectType", tag)
            fields = tuple(obj)
            for scenario, members in (
                ("complete", fields),
                ("missing-field", fields[:-1]),
                ("wrong-type", (*fields[:-1], field(fields[-1].tag, "TextString", "wrong-type"))),
                ("duplicate-field", (*fields, fields[-1])),
            ):
                wire, _ = encode(structure(tag, *members), names)
                children = [encode(member, names)[0].hex() for member in members]
                yield {
                    "protocol_version": version, "name": name, "model": {"PGPKey": "PgpKey"}.get(tag, tag),
                    "fixture": f"object-{label}-{scenario}",
                    "fixture_sha256": hashlib.sha256(wire).hexdigest(),
                    "object_type": object_type, "scenario": scenario,
                    "expected": "passed" if scenario == "complete" else "rejected",
                }, " ".join((str(object_type), wire.hex(), *children))
        # The generic boundary suite checks the reserved Template tag itself.
        # Use a valid vendor tag here to isolate the reserved Object Type.
        names.tags["opaquevendorobject"] = 0x540090
        wire, _ = encode(structure("OpaqueVendorObject"), names)
        yield {
            "protocol_version": version, "name": "Template", "model": "ManagedObject",
            "fixture": "object-template-reserved", "fixture_sha256": hashlib.sha256(wire).hexdigest(),
            "object_type": 6, "scenario": "reserved-object-type", "expected": "passed",
        }, f"6 {wire.hex()}"


def expected_checks(case, language):
    expected = case["expected"]
    if case["scenario"] == "reserved-object-type":
        return {"construct": "passed" if language == "rust" else "rejected", "read": expected}
    construct = "not-applicable" if language == "rust" and expected == "rejected" else expected
    return {"construct": construct, "read": expected}


def attach_evidence(root, inventory, paths, languages, expected_commit):
    version = inventory["protocol_version"]
    expected = {case["fixture"]: case for case, _ in vectors(root, (version,))}
    rows = {row["name"]: row for row in inventory["rows"] if row["kind"] == "managed_object"}
    seen = set()
    for path in paths:
        report = json.loads(path.read_text(encoding="utf-8"))
        if (report["scope"], report["api_level"]) != ("local", "typed-protocol"):
            raise ValueError(f"{path.name}: expected local typed managed object evidence")
        records = [record for record in report["fixtures"] if record["protocol_version"] == version]
        if sorted(record["fixture"] for record in records) != sorted(expected):
            raise ValueError(f"{path.name}: incomplete managed object evidence for KMIP {version}")
        for language, artifact in report["artifacts"].items():
            if language not in languages:
                continue
            validate_provenance(artifact, expected_commit)
            constructor, reader = SYMBOLS[language]
            for record in records:
                fixture = expected[record["fixture"]]
                identity = language, artifact["artifact_sha256"], record["fixture"]
                if identity in seen or {key: value for key, value in record.items() if key != "checks"} != fixture:
                    raise ValueError(f"{path.name}: mismatched or duplicate managed object fixture {record['fixture']}")
                seen.add(identity)
                checks = record["checks"][language]
                validate_checks(checks, constructor_inapplicable=language == "rust")
                obligation = next(item for item in rows[record["name"]]["obligations"]
                                  if (item["language"], item["api_level"]) == (language, "typed-protocol"))
                constructor_symbol = constructor.format(fixture["model"])
                if language == "rust" and fixture["scenario"] == "reserved-object-type":
                    constructor_symbol = "kmipkit::protocol::ManagedObject::unknown"
                obligation.setdefault("local_managed_object_results", []).append({
                    **fixture, **artifact, "scope": "local", "checks": checks,
                    "expected_checks": expected_checks(fixture, language),
                    "public_api": {"construct": constructor_symbol, "read": reader},
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
        report = collect_node_checks(vectors(ROOT), runners, "--inspect-table-managed-objects",
                                     rust_typed_constructor=True)
        with args.output.open("x", encoding="utf-8") as output:
            json.dump(report, output, indent=2)
            output.write("\n")
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(2, f"Typed managed object check failed: {error}\n")
    failures = [(case["protocol_version"], case["fixture"], language, checks) for case in report["fixtures"]
                for language, checks in case["checks"].items() if checks != expected_checks(case, language)]
    for version, fixture, language, checks in failures:
        print(f"{language} KMIP {version} {fixture}: {checks}", file=sys.stderr)
    print(f"Checked {len(report['fixtures'])} typed managed object fixtures with {len(runners)} installed validators")
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
