"""Emit the field inventory for the fluent API from pinned OASIS HTML."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

from .attribute_policy_source import _SourceParser
from .generate_fluent_operation_inventory import OUTPUT_PATH
from .generate_named_values import PINNED_VERSION_SOURCES, VERSIONED_SOURCE_MAX_BYTES
from .safe_io import safe_read_bytes

LANGUAGES = ("rust", "c", "java", "python")
OPERATION = re.compile(r"6\.([12])\.\d+ (.+)")
PAYLOADS = {
    "Request Payload": "request",
    "Response Payload": "response",
    "Reponse Payload": "response",  # KMIP 3.0 Obliterate table heading.
    "Message Payload": "message",
}
ENVELOPES = {
    "Request Header", "Response Header", "Request Batch Item", "Response Batch Item",
}
ROUTES = ("fluent", "manual", "direct", "batch", "validate", "typedResponse", "raw", "remoteError")


def _operation_key(name: str) -> str:
    return "".join(character for character in name.casefold() if character.isalnum())


def _table_record(table: dict) -> dict:
    rows = table["rows"]
    header = rows[1]
    if len(header) != 3:
        raise ValueError(f"unexpected field table header in {table['heading']}: {header}")
    fields = []
    for position, row in enumerate(rows[2:], start=1):
        if not any(row):
            continue
        if len(row) != len(header):
            raise ValueError(f"unexpected field table row in {table['heading']}: {row}")
        fields.append({
            "row": position,
            "name": row[0],
            "required": row[1],
            "condition": row[2],
            "evidence": {},
        })
    return {
        "section": table["heading"].split()[0],
        "caption": next((caption for caption in table["captions"] if caption), None),
        "fields": fields,
    }


def inventory(root: Path) -> dict:
    existing = json.loads(safe_read_bytes(root, OUTPUT_PATH, max_bytes=VERSIONED_SOURCE_MAX_BYTES))
    routes = {
        (row["direction"], _operation_key(row["name"])): row
        for row in existing["operations"]
    }
    versions = {}
    observed = set()
    for version, (path, checksum) in PINNED_VERSION_SOURCES.items():
        source = safe_read_bytes(root, path, max_bytes=VERSIONED_SOURCE_MAX_BYTES)
        if hashlib.sha256(source).hexdigest() != checksum:
            raise ValueError(f"unexpected OASIS source checksum: {path}")
        parser = _SourceParser()
        parser.feed(source.decode("utf-8-sig" if version == "kmip-3.0" else "cp1252"))
        parser.close()
        operations = {}
        for heading in parser.headings:
            match = OPERATION.fullmatch(heading)
            if not match:
                continue
            side, name = match.groups()
            direction = "client_to_server" if side == "1" else "server_to_client"
            key = (direction, _operation_key(name))
            operation = routes.get(key)
            if operation is not None:
                observed.add(key)
            operations[heading] = {
                "name": operation["name"] if operation else name,
                "direction": direction,
                "wireValue": operation["wireValue"] if operation else None,
                "section": heading.split()[0],
                "payloads": {},
                "modelCandidates": {
                    role: operation[f"typed{role.title()}Owner"]
                    for role in ("request", "response")
                    if operation and operation.get(f"typed{role.title()}Owner")
                },
                "evidence": {},
            }
        envelopes = {}
        for table in parser.tables:
            if not table["rows"] or len(table["rows"][0]) != 1:
                continue
            title = table["rows"][0][0]
            heading = table["heading"]
            if heading in operations and title in PAYLOADS:
                payloads = operations[heading]["payloads"]
                role = PAYLOADS[title]
                if role in payloads:
                    raise ValueError(f"{version}: duplicate {role} table in {heading}")
                payloads[role] = _table_record(table)
            elif heading.startswith("8.") and title in ENVELOPES:
                if title in envelopes:
                    raise ValueError(f"{version}: duplicate {title} table")
                envelopes[title] = _table_record(table)
        for operation in operations.values():
            expected = {
                "Notify": {"message"}, "Put": {"message"}, "Poll": {"request"},
            }.get(operation["name"], {"request", "response"})
            if operation["payloads"].keys() != expected:
                raise ValueError(f"{version}: unexpected payload tables for {operation['name']}")
            if operation["name"] == "Poll":
                operation["responsePayload"] = "Depends on the original operation"
        if envelopes.keys() != ENVELOPES:
            raise ValueError(f"{version}: incomplete envelope fields")
        versions[version.removeprefix("kmip-")] = {
            "source": path,
            "envelope": envelopes,
            "operations": list(operations.values()),
        }
    if missing := routes.keys() - observed:
        raise ValueError(f"operation inventory entries unmatched by OASIS: {sorted(missing)}")
    return {
        "formatVersion": 1,
        "languages": LANGUAGES,
        "routes": ROUTES,
        "versions": versions,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--check", action="store_true", help="Validate source coverage without printing")
    args = parser.parse_args()
    result = inventory(args.repo_root)
    if not args.check:
        print(json.dumps(result, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
