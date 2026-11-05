#!/usr/bin/env python3
"""Generate audited top-level Request Payload descriptors from the 0051 matrix."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MATRIX = ROOT / "specs/051-pre-send-validation/gap-matrix.md"
OUTPUT = ROOT / "crates/kmipkit-protocol/src/validation/schemas_generated.rs"

ALIASES = {
    "Create Key Pair": "CreateKeyPair",
    "Create Split Key": "CreateSplitKey",
    "Get Attribute List": "GetAttributeList",
    "Get Attributes": "GetAttributes",
    "Get Usage Allocation": "GetUsageAllocation",
    "Query Async Requests": "QueryAsyncRequests",
    "Re-Provision": "ReProvision",
    "Re-Certify": "Recertify",
    "Re-Key": "Rekey",
    "Re-Key Key Pair": "RekeyKeyPair",
    "RNG Retrieve": "RngRetrieve",
    "RNG Seed": "RngSeed",
    "MAC": "Mac",
    "MAC Verify": "MacVerify",
    "PKCS#11": "Pkcs11",
    "Signature Verify": "SignatureVerify",
    "Set Endpoint Role": "SetEndpointRole",
}

TYPE_NAMES = (
    ("Structure", "TYPE_STRUCTURE"),
    ("Integer", "TYPE_INTEGER"),
    ("Long Integer", "TYPE_LONG_INTEGER"),
    ("Big Integer", "TYPE_BIG_INTEGER"),
    ("Enumeration", "TYPE_ENUMERATION"),
    ("Boolean", "TYPE_BOOLEAN"),
    ("Text String", "TYPE_TEXT_STRING"),
    ("Byte String", "TYPE_BYTE_STRING"),
    ("Date-Time Extended", "TYPE_DATE_TIME_EXTENDED"),
    ("Date-Time", "TYPE_DATE_TIME"),
    ("Interval", "TYPE_INTERVAL"),
)


def rust_variant(operation: str) -> str:
    if operation in ALIASES:
        return ALIASES[operation]
    return operation.replace(" ", "")


def split_row(line: str) -> list[str]:
    cells = [part.strip() for part in line.split("|")[1:-1]]
    if len(cells) != 9:
        raise ValueError(f"expected 9 matrix columns, got {len(cells)}: {line}")
    return cells


def source_map(lines: list[str]) -> dict[str, str]:
    sources: dict[str, str] = {}
    for line in lines:
        if not re.match(r"^\| M\d{2} \|", line):
            continue
        cells = [part.strip() for part in line.split("|")[1:-1]]
        operation = cells[1]
        reference = re.search(r"§6\.1\.[^|]+", cells[2])
        if reference is None:
            raise ValueError(f"missing operation source for {operation}")
        sources[operation] = reference.group(0)
    return sources


def type_mask(type_source: str) -> str:
    prefix = type_source.split(";", 1)[0]
    if prefix.startswith("Any TTLV Value"):
        return " | ".join(name for _, name in TYPE_NAMES)
    allowed = [
        constant
        for name, constant in TYPE_NAMES
        if name.casefold() in prefix.casefold()
    ]
    if not allowed:
        raise ValueError(f"unhandled TTLV type description: {type_source}")
    return " | ".join(allowed)


def cardinality(requiredness: str) -> tuple[int, str]:
    normalized = requiredness.casefold()
    if "conditional" in normalized:
        return 0, "Some(1)"
    match = re.search(r"/(?:\s*)0\.\.\*", requiredness)
    if match:
        return 0, "None"
    if re.search(r"/\s*1\.\.\*", requiredness):
        return 1, "None"
    if re.search(r"/\s*0\.\.1", requiredness):
        return 0, "Some(1)"
    if re.search(r"/\s*1(?:\s|$)", requiredness):
        return 1, "Some(1)"
    raise ValueError(f"unhandled requiredness: {requiredness}")


def field_descriptor(cells: list[str]) -> str:
    rule = re.search(r"R\d{3}", cells[0])
    if rule is None:
        raise ValueError(f"missing stable rule identifier in {cells[0]}")
    field_text = cells[3]
    name = field_text.split("/", 1)[0].replace("`", "").strip()
    if name.startswith("Any Object"):
        tag = "FieldTag::ManagedObject"
    else:
        raw_tag = re.search(r"0x([0-9A-Fa-f]{6})", field_text)
        if raw_tag is None:
            raise ValueError(f"missing exact tag for {field_text}")
        digits = f"{int(raw_tag.group(1), 16):08x}"
        tag = f"FieldTag::Exact(0x{digits[:4]}_{digits[4:]})"
    minimum, maximum = cardinality(cells[5])
    nested_match = re.search(r"nested schema: ([A-Za-z]+)", cells[7])
    nested_schema = (
        f"Some(NestedStructureSchema::{nested_match.group(1)})"
        if nested_match
        else "None"
    )
    return (
        "    FieldDescriptor {\n"
        f'        rule_id: "KMIPKIT-0051-{rule.group(0)}",\n'
        f'        name: "{name}",\n'
        f"        tag: {tag},\n"
        f"        types: {type_mask(cells[4])},\n"
        f"        min_count: {minimum},\n"
        f"        max_count: {maximum},\n"
        f"        nested_schema: {nested_schema},\n"
        "    },"
    )


def render() -> str:
    lines = MATRIX.read_text(encoding="utf-8-sig").splitlines()
    sources = source_map(lines)
    records: dict[str, list[list[str]]] = {}
    for line in lines:
        if not line.startswith("| `KMIPKIT-0051-R"):
            continue
        cells = split_row(line)
        records.setdefault(cells[1], []).append(cells)

    variants = sorted(
        {rust_variant(operation) for operation in sources}
        | {"GetConstraints", "Ping", "SetEndpointRole"},
    )
    if len(variants) != 57:
        raise ValueError(f"expected 56 supported and one deferred request schema, found {len(variants)}")
    output = [
        "//! Generated operation Request Payload schema identities and descriptors.",
        "//! Regenerate with `python tools/validation_schema/generate.py`.",
        "",
        "use super::{FieldDescriptor, FieldTag, NestedStructureSchema, RequestSchemaDescriptor};",
        "use super::{",
        "    TYPE_BIG_INTEGER, TYPE_BOOLEAN, TYPE_BYTE_STRING, TYPE_DATE_TIME,",
        "    TYPE_DATE_TIME_EXTENDED, TYPE_ENUMERATION, TYPE_INTEGER, TYPE_INTERVAL,",
        "    TYPE_LONG_INTEGER, TYPE_STRUCTURE, TYPE_TEXT_STRING,",
        "};",
        "",
        "/// Explicit identity for a supported KMIP v2.1 client Request Payload.",
        "#[non_exhaustive]",
        "#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]",
        "pub enum RequestPayloadSchema {",
    ]
    for variant in variants:
        if variant == "SetEndpointRole":
            output.append("    /// Client-initiated operation deferred from `KMIPKit` 1.0 to 1.1.")
            output.append("    SetEndpointRole,")
        else:
            output.append(f"    /// The `{variant}` Request Payload schema.")
            output.append(f"    {variant},")
    operation_by_variant = {rust_variant(operation): operation for operation in sources}
    supported_variants = [variant for variant in variants if variant != "SetEndpointRole"]
    groups = [supported_variants[index : index + 12] for index in range(0, len(supported_variants), 12)]
    output.extend(
        [
            "}",
            "",
            "pub(super) fn descriptor(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {",
            "    match schema {",
        ],
    )
    for group_index, group in enumerate(groups):
        patterns = " | ".join(f"RequestPayloadSchema::{variant}" for variant in group)
        output.append(f"        {patterns} => descriptor_group_{group_index}(schema),")
    output.append("        RequestPayloadSchema::SetEndpointRole => None,")
    output.extend(["    }", "}", ""])

    for group_index, group in enumerate(groups):
        output.extend(
            [
                f"fn descriptor_group_{group_index}(schema: RequestPayloadSchema) -> Option<RequestSchemaDescriptor> {{",
                "    match schema {",
            ],
        )
        for variant in group:
            operation = operation_by_variant.get(variant)
            if operation is None:
                raise ValueError(f"missing operation source for {variant}")
            output.append(
                f"        RequestPayloadSchema::{variant} => Some(RequestSchemaDescriptor {{"
            )
            output.append(f'            source: "{sources[operation]}",')
            output.append(f"            fields: {variant.upper()}_FIELDS,")
            output.append("        }),")
        output.extend(["        _ => None,", "    }", "}", ""])

    for variant in variants:
        if variant == "SetEndpointRole":
            continue
        operation = operation_by_variant.get(variant)
        rows = records.get(operation or "", [])
        rows.sort(key=lambda cells: int(cells[2]))
        output.append(f"static {variant.upper()}_FIELDS: &[FieldDescriptor] = &[")
        output.extend(field_descriptor(cells) for cells in rows)
        output.append("];")
        output.append("")
    return "\n".join(output)


def formatted_render() -> str:
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    temporary = OUTPUT.with_name("schemas_generated.tmp.rs")
    temporary.write_text(render(), encoding="utf-8", newline="\n")
    try:
        subprocess.run(
            ["rustfmt", "--edition", "2024", str(temporary)],
            check=True,
            cwd=ROOT,
        )
        return temporary.read_text(encoding="utf-8")
    finally:
        temporary.unlink(missing_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = formatted_render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != generated:
            print(f"generated schema output is stale: {OUTPUT}", file=sys.stderr)
            return 1
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(generated, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
