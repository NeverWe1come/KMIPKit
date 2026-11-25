#!/usr/bin/env python3
"""Prepare generic TTLV decode vectors from the pinned OASIS XML scenarios."""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import struct
import subprocess
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.matrix.kmip30_inventory import source_inventory
from verification.interop.shared.tools.matrix.oasis_xml import normalize, xml_rows
from verification.interop.shared.tools.matrix.pkcs11 import referenced_values

TYPES = (
    "Structure", "Integer", "LongInteger", "BigInteger", "Enumeration", "Boolean",
    "TextString", "ByteString", "DateTime", "Interval", "DateTimeExtended",
    "Identifier", "Reference", "NameReference",
)
VERSION_CODES = ("2.0", "2.1", "3.0")
DECODERS = {
    "rust": "kmipkit_ttlv::codec::decode",
    "c": "kmipkit_ttlv_decode",
    "java": "org.kmipkit.ttlv.TtlvCodec.decode",
    "python": "kmipkit.ttlv.ttlv_decode",
}
CONSTRUCTORS = {
    "rust": "kmipkit::ttlv::Item::new",
    "c": "kmipkit_ttlv_item_create",
    "java": "org.kmipkit.ttlv.TtlvItem.create",
    "python": "kmipkit.ttlv.ttlv_item_create",
}
ENUM_FAMILY = {
    "maskgeneratorhashingalgorithm": "hashingalgorithm",
}
VALUE_ALIASES = {
    "cryptographicalgorithm": {"des3": "3des"},
    "maskgenerator": {"mgf1": "mfg1"},
    "pkcs11returncode": {"ok": "ckrok"},
}


class WireValues:
    def __init__(self, root: Path, version: str, draft: list[dict]):
        directory = root / "verification/conformance/catalog"
        base = json.loads((directory / "kmip-2.1.json").read_text(encoding="utf-8"))["elements"]
        self.tags = {normalize(row["name"]): int(row["wire_value"], 16) for row in base
                     if row["kind"] == "tag" and row.get("allocation") == "assigned"}
        self.values: dict[str, dict[str, int]] = defaultdict(dict)
        families = {row["element_id"]: row["name"] for row in base if row["kind"] in {"enumeration", "bitmask"}}
        for row in base:
            value = row.get("wire_value") or ""
            if row["kind"] not in {"enumeration_value", "bitmask_value"} or not re.fullmatch("[0-9a-fA-F]{8}", value):
                continue
            for parent in row.get("parent_element_ids", []):
                if parent in families:
                    self.add_value(families[parent], row["name"], value)
        versions = json.loads((directory / "kmip-versioned-values.json").read_text(encoding="utf-8"))["versions"]
        for group in versions[f"v{version.replace('.', '_')}"]:
            for value in group["values"]:
                self.add_value(group["name"], value["name"], value["wire_value"])
        for _, family, name, raw, allocation, _ in referenced_values(root):
            if allocation == "assigned":
                self.add_value(family, name, raw)
        if version == "3.0":
            self.add_draft_values(draft)

    def add_value(self, family: str, name: str, value: str) -> None:
        self.values[normalize(family)][normalize(name)] = int(value, 16)

    def add_draft_values(self, targets: list[dict]) -> None:
        for row in targets:
            cells = row.get("cells", [])
            if len(cells) != 2 or cells[0].startswith("("):
                continue
            _, _, family = row["heading"].partition(" ")
            name, raw = cells
            if family == "Tag Enumeration" and re.fullmatch("(?:0x)?[0-9a-fA-F]{6}", raw):
                self.tags[normalize(name)] = int(raw, 16)
            elif family.endswith((" Enumeration", " Mask")) and re.fullmatch("[0-9a-fA-F]{8}", raw):
                self.add_value(family.removesuffix(" Enumeration"), name, raw)

    def symbol(self, tag: str, value: str) -> int:
        family = normalize(tag)
        if family == "attributereference":
            return self.tags[normalize(value)]
        family = ENUM_FAMILY.get(family, family)
        name = normalize(value)
        return self.values[family][VALUE_ALIASES.get(family, {}).get(name, name)]


def timestamp(value: str) -> int:
    now = re.fullmatch(r"\$NOW(?:([+-][0-9]+))?", value)
    if now:
        return 1_700_000_000 + int(now[1] or 0)
    if re.fullmatch(r"-?[0-9]+", value):
        return int(value)
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("fixture date-time requires an offset")
    return int(parsed.timestamp())


def number(value: str) -> int:
    return int(value, 16 if value.lower().startswith("0x") else 10)


def scalar(tag: str, kind: str, value: str, names: WireValues) -> bytes:
    if kind == "Integer":
        try:
            integer = number(value)
        except ValueError:
            integer = 0
            for flag in value.split():
                integer |= names.symbol(tag, flag)
        return integer.to_bytes(4, "big", signed=integer < 0)
    if kind in {"LongInteger", "DateTimeExtended"}:
        return struct.pack(">q", number(value))
    if kind == "BigInteger":
        data = bytes.fromhex(value)
        sign = b"\xff" if data and data[0] & 0x80 else b"\0"
        return sign * (-len(data) % 8) + data
    if kind == "Enumeration":
        try:
            code = number(value)
        except ValueError:
            code = names.symbol(tag, value)
        return struct.pack(">I", code)
    if kind == "Boolean":
        return struct.pack(">Q", {"true": 1, "false": 0}[value.casefold()])
    if kind in {"TextString", "Identifier", "Reference", "NameReference"}:
        return (f"fixture-{value[1:]}" if value.startswith("$") else value).encode("utf-8")
    if kind == "ByteString":
        return hashlib.sha256(value.encode("utf-8")).digest() if value.startswith("$") else bytes.fromhex(value)
    if kind == "DateTime":
        return struct.pack(">q", timestamp(value))
    if kind == "Interval":
        return struct.pack(">I", number(value))
    raise ValueError(f"unsupported fixture type {kind}")


def encode(node: ET.Element, names: WireValues) -> tuple[bytes, list[list[int]]]:
    tag = names.tags[normalize(node.tag)]
    kind = node.get("type", "Structure")
    type_code = TYPES.index(kind) + 1
    flat = [[tag, type_code, len(node)]]
    if kind == "Structure":
        encoded = []
        for child in node:
            wire, expected = encode(child, names)
            encoded.append(wire)
            flat.extend(expected)
        value = b"".join(encoded)
    else:
        if len(node):
            raise ValueError(f"non-structure fixture field has children: {node.tag}")
        value = scalar(node.tag, kind, node.attrib["value"], names)
    header = tag.to_bytes(3, "big") + bytes([type_code]) + struct.pack(">I", len(value))
    return header + value + bytes(-len(value) % 8), flat


def vectors(root: Path, versions: tuple[str, ...]):
    draft = source_inventory(root) if "3.0" in versions else []
    for version in versions:
        names = WireValues(root, version, draft)
        fixtures: dict[str, str] = {}
        for row in xml_rows(root, version, defaultdict(list)):
            fixture = row["fixture"]
            fixtures[fixture["path"]] = fixture["sha256"]
        for path, digest in fixtures.items():
            source = (root / path).read_bytes()
            if hashlib.sha256(source).hexdigest() != digest:
                raise ValueError(f"fixture changed during preparation: {path}")
            for index, node in enumerate(ET.fromstring(source)):
                entry = {"fixture": path, "fixture_sha256": digest, "protocol_version": version,
                         "step": index // 2, "message": node.tag}
                try:
                    wire, structure = encode(node, names)
                except (KeyError, ValueError, OverflowError, struct.error) as error:
                    yield {**entry, "status": "blocked", "reason": str(error)}
                else:
                    yield {**entry, "status": "ready", "ttlv": wire.hex(), "items": structure}


def inspect_vectors(rows, runners, *, construct=False):
    positions = [index for index, row in enumerate(rows) if row["status"] == "ready"]
    source = "".join(rows[index]["ttlv"] + "\n" for index in positions)
    artifacts = {}
    command = "--construct-oasis-ttlv" if construct else "--inspect-oasis-ttlv"
    symbols = CONSTRUCTORS if construct else DECODERS
    for language, executable in runners.items():
        completed = subprocess.run(
            [str(executable.resolve(strict=True)), command],
            input=source, text=True, capture_output=True, check=False,
        )
        output = completed.stdout.splitlines()
        if completed.returncode != 0 or len(output) != len(positions) + 1:
            raise ValueError(f"{language} fixture runner did not return one result for every vector")
        artifact = json.loads(output[0])
        for field, width in (("library_commit", 40), ("artifact_sha256", 64)):
            if not re.fullmatch(rf"[0-9a-f]{{{width}}}", artifact[field]):
                raise ValueError(f"{language} fixture runner returned invalid provenance")
        artifacts[language] = {field: artifact[field] for field in ("library_commit", "artifact_sha256")}
        artifacts[language]["public_api_symbol"] = symbols[language]
        for index, line in zip(positions, output[1:], strict=True):
            observed = json.loads(line)
            row = rows[index]
            outcome = "rejected" if observed is None else ("passed" if observed == row["items"] else "failed")
            row.setdefault("checks", {})[language] = outcome
    return artifacts


def attach_decoder_evidence(path: Path, report: dict, expected_commit: str | None, *, construct=False):
    """Attach local request/response observations to their pinned scenario step."""
    symbols = CONSTRUCTORS if construct else DECODERS
    summary_key = "oasis_ttlv_construction" if construct else "oasis_ttlv"
    check_key = "fixture_construction" if construct else "fixture_codec"
    rows = report["rows"]
    known = {
        (row["protocol_version"], row["fixture"]["path"], row["step_index"]): row["fixture"]["sha256"]
        for row in rows if row.get("kind") == "scenario_item"
    }
    versions = {key[0] for key in known}
    observations = defaultdict(dict)
    counts = defaultdict(Counter)
    with path.open(encoding="utf-8") as source:
        header = json.loads(source.readline())
        if header.get("mode", "decode") != ("construct" if construct else "decode"):
            raise ValueError("fixture evidence mode does not match the requested check")
        artifacts = {lang: value for lang, value in header["artifacts"].items() if lang in report["languages"]}
        for language, artifact in artifacts.items():
            if artifact["public_api_symbol"] != symbols.get(language):
                raise ValueError("fixture report identifies an unexpected public API")
            if expected_commit is not None and artifact["library_commit"] != expected_commit:
                raise ValueError("codec evidence does not match the requested library commit")
        seen = set()
        roles = {"RequestMessage": "request", "ResponseMessage": "response"}
        for line in source:
            fixture = json.loads(line)
            version = fixture["protocol_version"]
            if version not in versions:
                continue
            key = (version, fixture["fixture"], fixture["step"])
            role = roles.get(fixture["message"])
            if known.get(key) != fixture["fixture_sha256"] or role is None or (key, role) in seen:
                raise ValueError("codec evidence does not match a unique pinned scenario message")
            seen.add((key, role))
            for language in artifacts:
                status = fixture.get("checks", {}).get(language, "blocked")
                if status not in ("passed", "failed", "rejected", "blocked"):
                    raise ValueError("unknown codec evidence outcome")
                observations[key].setdefault(language, {})[role] = status
                counts[language][status] += 1
    for row in rows:
        if row.get("kind") != "scenario_item":
            continue
        key = (row["protocol_version"], row["fixture"]["path"], row["step_index"])
        for language, observation in observations.get(key, {}).items():
            row["checks"][language][check_key] = observation
    report[summary_key] = {"artifacts": artifacts, "message_checks": counts}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", choices=VERSION_CODES, action="append")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--require-complete", action="store_true")
    parser.add_argument("--construct", action="store_true",
                        help="Rebuild complete KMIP 2.x messages with the installed generic tag constructors")
    parser.add_argument("--runner", nargs=2, metavar=("LANGUAGE", "EXECUTABLE"), action="append", default=[])
    args = parser.parse_args()
    runners = {language: Path(executable) for language, executable in args.runner}
    if len(runners) != len(args.runner) or not set(runners).issubset(DECODERS):
        parser.error("select at most one runner per supported language")
    versions = tuple(dict.fromkeys(args.version or (VERSION_CODES[:2] if args.construct else VERSION_CODES)))
    if args.construct and (not runners or "3.0" in versions):
        parser.error("--construct requires an installed runner and a KMIP 2.x version")
    try:
        rows = list(vectors(ROOT, versions))
        artifacts = inspect_vectors(rows, runners, construct=args.construct)
        counts = Counter(row["status"] for row in rows)
        checks = {language: Counter(row.get("checks", {}).get(language, "blocked") for row in rows)
                  for language in runners}
        with args.output.open("x", encoding="utf-8") as output:
            if artifacts:
                output.write(json.dumps({"mode": "construct" if args.construct else "decode",
                                         "artifacts": artifacts, "counts": counts, "checks": checks}) + "\n")
            for row in rows:
                result = {key: value for key, value in row.items() if key not in {"ttlv", "items"}} if runners else row
                output.write(json.dumps(result, separators=(",", ":")) + "\n")
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(2, f"OASIS TTLV check failed: {error}\n")
    print(json.dumps({"vectors": counts, "checks": checks}, sort_keys=True))
    return int(args.require_complete and (
        counts["blocked"] or any(counts.get("passed", 0) != len(rows) for counts in checks.values())
    ))


if __name__ == "__main__":
    raise SystemExit(main())
