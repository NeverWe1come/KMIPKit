#!/usr/bin/env python3
"""Inventory KMIP request, response and attribute variants for parity campaigns."""

from __future__ import annotations

import argparse
from collections import Counter
from collections.abc import Iterator
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.conformance.attribute_policy_source import _SourceParser
from verification.conformance.fluent_field_inventory import LANGUAGES, inventory
from verification.conformance.generate_named_values import PINNED_VERSION_SOURCES
from verification.interop.shared.tools.matrix.payload_contract import payload_presence_contract

VERSIONS = ("2.0", "2.1", "3.0")
ROUTES = ("typed-protocol", "fluent")


def routes_for_version(version: str) -> tuple[str, ...]:
    return (*ROUTES, "generic-ttlv") if version in ("2.0", "2.1") else ROUTES
ATTRIBUTE_HEADING = re.compile(r"(4\.\d+) (.+)")


def field_variants(version: str, operation: dict, role: str, field: dict) -> Iterator[dict]:
    try:
        branches, rules = payload_presence_contract(field["required"], field["condition"])
    except ValueError:
        branches, rules = ["requiredness_review"], []
    yield from make_variants(version, {
        "kind": "operation_field", "operation": operation["name"], "direction": operation["direction"],
        "role": role, "section": operation["section"], "field": field["name"], "table_row": field["row"],
        "requiredness": field["required"], "description": field["condition"], "conditions": rules,
    }, branches)


def make_variants(version: str, target: dict, branches: list[str]) -> Iterator[dict]:
    for branch in branches:
        row = {"protocol_version": version, **target, "variant": branch}
        canonical = json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
        variant_id = hashlib.sha256(canonical.encode("ascii")).hexdigest()
        yield {"variant_id": variant_id, **row, "api_levels": list(routes_for_version(version)), "status": "pending"}


def attribute_variants(root: Path, version: str) -> Iterator[dict]:
    path, digest = PINNED_VERSION_SOURCES[f"kmip-{version}"]
    raw = (root / path).read_bytes()
    if hashlib.sha256(raw).hexdigest() != digest:
        raise ValueError(f"pinned KMIP {version} attribute source changed")
    parser = _SourceParser()
    parser.feed(raw.decode("utf-8-sig" if version == "3.0" else "cp1252"))
    parser.close()
    for table in parser.tables:
        heading = ATTRIBUTE_HEADING.fullmatch(table["heading"])
        if not heading:
            continue
        policy = {cells[0].casefold(): cells[1] for cells in table["rows"] if len(cells) == 2}
        cardinality = policy.get("multiple instances permitted")
        if cardinality is None:
            continue
        section, name = heading.groups()
        target = {
            "kind": "managed_object_attribute", "section": section, "field": name,
            "source": str(path), "rules": policy,
        }
        branches = ["encoded_value", "wrong_type_rejected", "omitted", "read_by_exact_reference"]
        if cardinality.startswith("Yes"):
            branches.append("repeated_values")
        else:
            branches.append("duplicate_rejected")
        for operation, key in (("Create", "initially set by"), ("Add Attribute", "modifiable by client"),
                               ("Modify Attribute", "modifiable by client"), ("Set Attribute", "modifiable by client"),
                               ("Delete Attribute", "deletable by client")):
            if policy.get(key):
                branches.append(f"{operation.lower().replace(' ', '_')}_policy")
        yield from make_variants(version, target, branches)


def plan(root: Path, versions: tuple[str, ...] = VERSIONS) -> Iterator[dict]:
    specification = inventory(root)["versions"]
    for version in versions:
        if version not in VERSIONS:
            raise ValueError(f"unsupported KMIP version: {version}")
    for version in versions:
        for operation in specification[version]["operations"]:
            for role, payload in operation["payloads"].items():
                for field in payload["fields"]:
                    yield from field_variants(version, operation, role, field)
        yield from attribute_variants(root, version)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", action="append", choices=VERSIONS)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    counts = Counter()
    rows = plan(ROOT, tuple(dict.fromkeys(args.version or VERSIONS)))
    if args.output is not None:
        with args.output.open("x", encoding="utf-8") as stream:
            for row in rows:
                counts[f'{row["protocol_version"]} {row["kind"]}'] += 1
                stream.write(json.dumps(row, separators=(",", ":")) + "\n")
    else:
        for row in rows:
            counts[f'{row["protocol_version"]} {row["kind"]}'] += 1
    print(json.dumps({"variants": counts, "languages": LANGUAGES,
                      "routes_by_version": {version: routes_for_version(version)
                                            for version in args.version or VERSIONS}}, indent=2))


if __name__ == "__main__":
    main()
