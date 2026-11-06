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
PYTHON_VALUES = ROOT / "bindings" / "python" / "src" / "kmipkit" / "protocol_values.py"
_OPERATION_MEMBER = re.compile(r"^    ([A-Z][A-Z0-9_]*) = 0x([0-9a-fA-F]+)$", re.MULTILINE)


def operation_member_name(operation_name: str) -> str:
    """Map the reviewed operation label to its generated enum member spelling."""
    return re.sub(r"[^A-Za-z0-9]+", "_", operation_name).upper()


def render_fixture(manifest: dict[str, Any], python_values: str) -> str:
    """Render catalog identifiers joined to generated raw KMIP Operation values."""
    enum_start = python_values.index("class Operation")
    enum_end = python_values.find("\nclass ", enum_start + 1)
    if enum_end < 0:
        enum_end = len(python_values)
    operation_enum = python_values[enum_start:enum_end]
    values = {
        member: int(raw, 16)
        for member, raw in _OPERATION_MEMBER.findall(operation_enum)
    }

    operations = [
        operation
        for operation in manifest["operations"]
        if operation.get("scope") == "client_1_0"
    ]
    operation_ids = [operation["operationId"] for operation in operations]
    if len(operation_ids) != len(set(operation_ids)):
        raise ValueError("client operation catalog contains duplicate operationId values")

    rows = ["operationId\toperationRaw"]
    for operation in operations:
        member = operation_member_name(operation["name"])
        try:
            raw = values[member]
        except KeyError as error:
            raise ValueError(
                f"missing generated KMIP Operation member {member} for {operation['operationId']}"
            ) from error
        rows.append(f"{operation['operationId']}\t{raw}")
    return "\n".join(rows) + "\n"


def generate(*, check: bool = False) -> bool:
    """Write the shared fixture, or check that its committed copy is current."""
    rendered = render_fixture(
        json.loads(MANIFEST.read_text(encoding="utf-8")),
        PYTHON_VALUES.read_text(encoding="utf-8"),
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
