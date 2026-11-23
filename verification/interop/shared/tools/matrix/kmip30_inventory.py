"""Generate draft KMIP 3.0 obligations and OASIS profile scenarios offline."""

from __future__ import annotations

import hashlib
import json
import re
from collections import defaultdict
from pathlib import Path

from verification.api.kmip30_draft_api import validate_kmip30_draft_api
from verification.conformance.attribute_policy_source import _SourceParser

from .oasis_xml import normalize, xml_rows
from .payload_contract import payload_presence_contract
from .pkcs11 import HEADER_DIR, PKCS11_HEADERS, referenced_values

SOURCE_DIR = Path("verification/conformance/oasis/kmip-3.0")
SPEC = SOURCE_DIR / "upstream/kmip-spec-v3.0.html"
HASHES = {
    SPEC: "a7d5e58e63acd9be33fab08b76039fc8db4e072fdbdf4a280ea5b743bae5682b",
}
OPERATION = re.compile(r"(6\.[12]\.\d+) (.+)")
SECTION = re.compile(r"(\d+(?:\.\d+)*)(?: |$)")
NORMATIVE = re.compile(r"\b(?:SHALL|MUST|SHOULD|MAY|REQUIRED)\b")


def pinned_source(root: Path, path: Path) -> str:
    data = (root / path).read_bytes()
    if hashlib.sha256(data).hexdigest() != HASHES[path]:
        raise ValueError(f"draft source checksum mismatch: {path}")
    return data.decode("utf-8")


def source_inventory(root: Path) -> list[dict]:
    """Index draft sections, their normative prose, and all populated table rows."""
    parser = _SourceParser()
    parser.feed(pinned_source(root, SPEC))
    parser.close()
    targets = {}
    for heading in parser.headings:
        section = SECTION.match(heading)
        if not section or not 2 <= int(section[1].split(".")[0]) <= 12:
            continue
        target_id = f"kmip30-section-{section[1]}"
        if target_id in targets:
            raise ValueError(f"duplicate KMIP 3.0 section: {section[1]}")
        op = OPERATION.fullmatch(heading)
        targets[target_id] = {
            "target_id": target_id,
            "kind": "operation" if op else "section",
            "heading": heading,
            "source": {"path": SPEC.as_posix(), "sha256": HASHES[SPEC], "section": section[1]},
        }

    paragraph_numbers = defaultdict(int)
    for paragraph in parser.paragraphs:
        section = SECTION.match(paragraph["heading"])
        if not section or f"kmip30-section-{section[1]}" not in targets:
            continue
        paragraph_numbers[section[1]] += 1
        if not NORMATIVE.search(paragraph["text"]):
            continue
        number = paragraph_numbers[section[1]]
        target_id = f"kmip30-section-{section[1]}-paragraph-{number}"
        targets[target_id] = {
            "target_id": target_id,
            "kind": "normative_prose",
            "heading": paragraph["heading"],
            "source": {"path": SPEC.as_posix(), "sha256": HASHES[SPEC],
                       "section": section[1], "paragraph": number},
        }

    table_numbers = defaultdict(int)
    for table in parser.tables:
        section = SECTION.match(table["heading"])
        if not section or f"kmip30-section-{section[1]}" not in targets:
            continue
        table_numbers[section[1]] += 1
        ordinal = table_numbers[section[1]]
        captions = table["captions"]
        role = None
        if any("Request Payload" in caption for caption in captions):
            role = "request"
        elif any("Response Payload" in caption for caption in captions):
            role = "response"
        for number, cells in enumerate(table["rows"], 1):
            if not any(cells):
                continue
            target_id = f"kmip30-section-{section[1]}-table-{ordinal}-row-{number}"
            field = role and len(cells) == 3 and cells[1] in {"Yes", "No"}
            targets[target_id] = {
                "target_id": target_id,
                "kind": "payload_field" if field else "table_row",
                "heading": table["heading"],
                "source": {"path": SPEC.as_posix(), "sha256": HASHES[SPEC],
                           "section": section[1], "table_in_section": ordinal,
                           "captions": captions, "row": number},
                "cells": cells,
            }
            if field:
                branches, rules = payload_presence_contract(cells[1], cells[2])
                targets[target_id].update(role=role, presence_branches=branches, presence_rules=rules)

    # §§11.40–11.41 incorporate the same pinned PKCS #11 3.0 source as KMIP 2.x.
    for suffix, family, name, wire_value, allocation, artifact in referenced_values(root):
        section = "11.40" if family == "PKCS#11 Function" else "11.41"
        heading = targets[f"kmip30-section-{section}"]["heading"]
        target_id = f"kmip30-pkcs11-{suffix.lower()}"
        targets[target_id] = {
            "target_id": target_id, "kind": "referenced_enum_value", "heading": heading,
            "source": {"path": SPEC.as_posix(), "sha256": HASHES[SPEC], "section": section},
            "external_source": {
                "path": (HEADER_DIR / artifact).as_posix(),
                "sha256": PKCS11_HEADERS[artifact],
            },
            "name": name, "wire_value": wire_value, "allocation": allocation,
        }
    return list(targets.values())


def _field_index(targets: list[dict]) -> dict:
    by_field = defaultdict(list)
    for target in targets:
        op = OPERATION.fullmatch(target["heading"])
        if op and target["kind"] == "payload_field":
            field = target["cells"][0]
            by_field[normalize(op[2]), target["role"], normalize(field)].append(target["target_id"])
    return by_field


def iter_draft_cells(root: Path, artifact_commit: str):
    targets = source_inventory(root)
    scenarios = list(xml_rows(root, "3.0", _field_index(targets)))
    observed = {target_id for row in scenarios for ids in row["observed_fields"].values()
                for target_id in ids}
    draft_api = json.loads((root / "verification/api/catalog/kmip-3.0-draft-api.json").read_text())
    public_api = json.loads((root / "bindings/shared/public-api.json").read_text())
    validate_kmip30_draft_api(draft_api, public_api, root)
    routes = {normalize(entry["operation"]): entry for entry in draft_api["operations"]}
    for target in targets:
        yield {
            "protocol_version": "3.0", "origin": "normative", "kind": "inventory",
            "case_id": target["target_id"], "target": target,
            "xml_presence_observed": target["target_id"] in observed,
            "execution_status": "required", "evidence_collected": False,
        }
        if target["kind"] != "operation":
            continue
        op_name = OPERATION.fullmatch(target["heading"])[2]
        route = routes.get(normalize(op_name))
        if route:
            for language in ("rust", "c", "java", "python"):
                symbol = route[language]
                if language == "c":
                    symbol = f"kmipkit_{symbol}"
                yield {
                    "protocol_version": "3.0", "origin": "normative", "kind": "public_route",
                    "case_id": target["target_id"], "language": language,
                    "operation": op_name, "public_api_symbol": symbol,
                    "artifact_commit": artifact_commit, "action": "construct",
                    "execution_status": "required", "evidence_collected": False,
                }
    yield from scenarios
