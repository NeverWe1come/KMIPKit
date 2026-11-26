#!/usr/bin/env python3
"""Exercise typed Key Material construction and reading in the installed bindings."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.oasis_ttlv import WireValues, encode
from verification.interop.shared.tools.table_ttlv import ALLOWED_VERSIONS, KEY_FORMATS, field, structure
from verification.interop.shared.tools.cosmian_minimum import normal

SYMBOLS = {
    "rust": ("kmipkit::protocol::KeyMaterial::structure", "kmipkit::protocol::KeyMaterial::try_from_item"),
    "c": ("kmipkit_key_material_from_fields", "kmipkit_key_material_from_item"),
    "java": ("org.kmipkit.ttlv.KeyMaterial.fromFields", "org.kmipkit.ttlv.KeyMaterial.fromItem"),
    "python": ("kmipkit.managed_objects.KeyMaterial.from_fields", "kmipkit.managed_objects.KeyMaterial.from_item"),
}


def vectors(root, versions=ALLOWED_VERSIONS):
    for version in versions:
        names = WireValues(root, version, [])
        for name, _, _, _, _, fields in KEY_FORMATS:
            format_code = names.symbol("KeyFormatType", f"Transparent {name}")
            tag, _, _ = fields[-1]
            for scenario, values in (
                ("complete", fields),
                ("missing-field", fields[:-1]),
                ("wrong-type", (*fields[:-1], (tag, "TextString", "wrong-type"))),
                ("duplicate-field", (*fields, fields[0])),
            ):
                material = structure("KeyMaterial", *(field(*item) for item in values))
                wire, _ = encode(material, names)
                children = [encode(child, names)[0].hex() for child in material]
                yield {
                    "protocol_version": version, "name": name,
                    "fixture": f"key-material-{normal(name)}-{scenario}",
                    "fixture_sha256": hashlib.sha256(wire).hexdigest(),
                    "format": format_code, "scenario": scenario,
                    "expected": "passed" if scenario == "complete" else "rejected",
                }, " ".join((str(format_code), wire.hex(), *children))


def validate_provenance(artifact, expected_commit=None):
    if not re.fullmatch("[0-9a-f]{40}", artifact["library_commit"]) or not re.fullmatch(
        "[0-9a-f]{64}", artifact["artifact_sha256"]
    ):
        raise ValueError("invalid installed Key Material artifact identity")
    if expected_commit is not None and artifact["library_commit"] != expected_commit:
        raise ValueError("Key Material evidence does not match the requested library commit")


def validate_checks(checks, *, constructor_inapplicable=False):
    statuses = {"passed", "rejected", "failed"}
    constructor_statuses = statuses | ({"not-applicable"} if constructor_inapplicable else set())
    if set(checks) != {"construct", "read"} or checks["construct"] not in constructor_statuses or checks["read"] not in statuses:
        raise ValueError("invalid typed Key Material result")


def collect(root, runners):
    return collect_node_checks(vectors(root), runners, "--inspect-table-key-material")


def collect_node_checks(cases, runners, command, *, rust_typed_constructor=False):
    cases = list(cases)
    inputs = "".join(line + "\n" for _, line in cases)
    fixtures = [{**metadata, "checks": {}} for metadata, _ in cases]
    artifacts = {}
    for language, executable in runners.items():
        result = subprocess.run(
            [str(executable.resolve(strict=True)), command],
            input=inputs, text=True, capture_output=True, check=False,
        )
        lines = result.stdout.splitlines()
        if result.returncode != 0 or len(lines) != len(fixtures) + 1:
            raise ValueError(f"{language}: {command} returned incomplete evidence")
        artifact = json.loads(lines[0])
        validate_provenance(artifact)
        artifacts[language] = artifact
        for fixture, line in zip(fixtures, lines[1:], strict=True):
            checks = json.loads(line)
            validate_checks(checks, constructor_inapplicable=rust_typed_constructor and language == "rust")
            fixture["checks"][language] = checks
    return {"scope": "local", "api_level": "typed-protocol", "artifacts": artifacts, "fixtures": fixtures}


def attach_key_material_evidence(root, inventory, paths, languages, expected_commit):
    """Check the original fixture identity before crediting the typed Key Material APIs."""
    version = inventory["protocol_version"]
    expected = {case["fixture"]: case for case, _ in vectors(root, (version,))}
    rows = {row["name"]: row for row in inventory["rows"] if row["kind"] == "transparent_key"}
    seen = set()
    for path in paths:
        payload = json.loads(path.read_text(encoding="utf-8"))
        if (payload["scope"], payload["api_level"]) != ("local", "typed-protocol"):
            raise ValueError(f"{path.name}: expected local typed Key Material evidence")
        for language, artifact in payload["artifacts"].items():
            if language not in languages:
                continue
            validate_provenance(artifact, expected_commit)
            construct, read = SYMBOLS[language]
            results = [case for case in payload["fixtures"] if case["protocol_version"] == version]
            if sorted(case["fixture"] for case in results) != sorted(expected):
                raise ValueError(f"{path.name}: incomplete Key Material fixture results for KMIP {version}")
            for case in results:
                fixture = case["fixture"]
                key = language, artifact["artifact_sha256"], fixture
                checks = case["checks"][language]
                if key in seen or {name: value for name, value in case.items() if name != "checks"} != expected[fixture]:
                    raise ValueError(f"{path.name}: mismatched or duplicate Key Material fixture {fixture}")
                seen.add(key)
                validate_checks(checks)
                row = rows[case["name"]]
                obligation = next(item for item in row["obligations"]
                                  if (item["language"], item["api_level"]) == (language, "typed-protocol"))
                obligation.setdefault("local_key_material_results", []).append({
                    **expected[fixture], **artifact, "scope": "local",
                    "public_api": {"construct": construct, "read": read}, "checks": checks,
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
        parser.exit(2, f"Typed Key Material check failed: {error}\n")
    failures = [(case["protocol_version"], case["fixture"], language, checks) for case in report["fixtures"]
                for language, checks in case["checks"].items() if set(checks.values()) != {case["expected"]}]
    for version, fixture, language, checks in failures:
        print(f"{language} KMIP {version} {fixture}: {checks}", file=sys.stderr)
    print(f"Checked {len(report['fixtures'])} typed Key Material fixtures with {len(runners)} installed validators")
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
