"""Generate the test-only KMIP operationId-to-raw-value fixture."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
OUTPUT = ROOT / "tests" / "fixtures" / "api" / "operation_catalog_parity.tsv"
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
NORMATIVE_CATALOG = ROOT / "specification" / "catalog" / "kmip-2.1.json"
OPERATION_ENUMERATION_ID = "KMIPKIT-ELEM-ENUMERATION-OPERATION"
OPERATION_VALUE_ID_PREFIX = "KMIPKIT-ELEM-ENUM-VALUE-OPERATION-"
_HEX_VALUE = re.compile(r"[0-9A-Fa-f]{8}")


def operation_member_name(operation_name: str) -> str:
    """Map a reviewed operation name to the conventional language enum member."""
    return re.sub(r"[^A-Za-z0-9]+", "_", operation_name).upper()


def operation_catalog_rows(
    manifest: dict[str, Any], normative_catalog: dict[str, Any]
) -> list[tuple[str, str, int]]:
    """Join client operations to their normative KMIP Operation values."""
    normative_values: dict[str, dict[str, Any]] = {}
    for element in normative_catalog["elements"]:
        if (
            element.get("kind") != "enumeration_value"
            or OPERATION_ENUMERATION_ID not in element.get("parent_element_ids", [])
        ):
            continue
        name = element["name"]
        if name in normative_values:
            raise ValueError(f"duplicate normative KMIP Operation name: {name}")
        normative_values[name] = element

    operations = [
        operation
        for operation in manifest["operations"]
        if operation.get("scope") == "client_1_0"
    ]
    operation_ids = [operation["operationId"] for operation in operations]
    operation_names = [operation["name"] for operation in operations]
    if len(operation_ids) != len(set(operation_ids)):
        raise ValueError("client operation catalog contains duplicate operationId values")
    if len(operation_names) != len(set(operation_names)):
        raise ValueError("client operation catalog contains duplicate operation names")

    rows: list[tuple[str, str, int]] = []
    for operation in operations:
        operation_id = operation["operationId"]
        name = operation["name"]
        element = normative_values.get(name)
        if element is None:
            raise ValueError(
                f"no normative KMIP Operation value for {operation_id} ({name})"
            )
        wire_value = element.get("wire_value", "")
        if not isinstance(wire_value, str) or not _HEX_VALUE.fullmatch(wire_value):
            raise ValueError(
                f"invalid normative KMIP Operation value for {operation_id}: {wire_value}"
            )
        operation_suffix = operation_id.removeprefix("KMIPKIT-ELEM-OP-C2S-")
        expected_element_id = (
            f"{OPERATION_VALUE_ID_PREFIX}{operation_suffix}-{wire_value.upper()}"
        )
        if element.get("element_id") != expected_element_id:
            raise ValueError(
                f"normative KMIP Operation identity mismatch for {operation_id}: "
                f"expected {expected_element_id}, got {element.get('element_id')}"
            )
        if element.get("scope_state") != "client_1_0":
            raise ValueError(
                f"normative KMIP Operation is outside client 1.0 scope: {operation_id}"
            )
        rows.append((operation_id, name, int(wire_value, 16)))

    raw_values = [raw for _, _, raw in rows]
    if len(raw_values) != len(set(raw_values)):
        raise ValueError("client operation catalog maps multiple operationIds to one raw value")
    return rows


def render_fixture(manifest: dict[str, Any], normative_catalog: dict[str, Any]) -> str:
    """Render catalog operationIds joined to normative KMIP Operation values."""
    rows = ["operationId\toperationRaw"]
    rows.extend(f"{operation_id}\t{raw}" for operation_id, _, raw in operation_catalog_rows(
        manifest, normative_catalog
    ))
    return "\n".join(rows) + "\n"


def find_language_value_mismatches(
    manifest: dict[str, Any],
    normative_catalog: dict[str, Any],
    language_values: dict[str, int],
) -> list[str]:
    """Return operationId-scoped mismatches between a language enum and KMIP."""
    mismatches: list[str] = []
    for operation_id, name, expected in operation_catalog_rows(manifest, normative_catalog):
        member = operation_member_name(name)
        actual = language_values.get(member)
        if actual != expected:
            rendered_actual = "<missing>" if actual is None else f"0x{actual:08X}"
            mismatches.append(
                f"{operation_id}: expected 0x{expected:08X}, got {rendered_actual}"
            )
    return mismatches


def generate(*, check: bool = False) -> bool:
    """Write the shared fixture, or check that its committed copy is current."""
    rendered = render_fixture(
        json.loads(MANIFEST.read_text(encoding="utf-8")),
        json.loads(NORMATIVE_CATALOG.read_text(encoding="utf-8")),
    )
    if check:
        return OUTPUT.is_file() and OUTPUT.read_text(encoding="utf-8") == rendered
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(rendered, encoding="utf-8", newline="\n")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check", action="store_true", help="fail if the checked-in fixture is stale"
    )
    arguments = parser.parse_args()
    if not generate(check=arguments.check):
        print(f"{OUTPUT.relative_to(ROOT)} is stale; regenerate it with this module")
        return 1
    if not arguments.check:
        print(f"wrote {OUTPUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
