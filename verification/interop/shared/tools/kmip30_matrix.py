"""Generate draft KMIP 3.0 obligations and OASIS profile scenarios offline."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path

# Script entry points run from shared/tools, while the source parser is repo-wide.
sys.path.insert(0, str(Path(__file__).resolve().parents[4]))
from verification.conformance.attribute_policy_source import _SourceParser
from verification.api.kmip30_draft_api import validate_kmip30_draft_api
from verification.interop.shared.catalog.generate_matrix_catalog import (
    _payload_presence_contract,
    _pkcs11_referenced_targets,
)

SOURCE_DIR = Path("verification/conformance/oasis/kmip-3.0")
FIXTURE_DIR = Path("verification/interop/shared/catalog/oasis/kmip-3.0")
SPEC = SOURCE_DIR / "upstream/kmip-spec-v3.0.html"
PROFILES = SOURCE_DIR / "upstream/kmip-profiles-v3.0.html"
HASHES = {
    SPEC: "a7d5e58e63acd9be33fab08b76039fc8db4e072fdbdf4a280ea5b743bae5682b",
    PROFILES: "2485e3f5c82d2ca42f43bbcaa4b4b2d8d7853eaf8f4208742c244071de6aab66",
}
OPERATION = re.compile(r"(6\.[12]\.\d+) (.+)")
SECTION = re.compile(r"(\d+(?:\.\d+)*)(?: |$)")
NORMATIVE = re.compile(r"\b(?:SHALL|MUST|SHOULD|MAY|REQUIRED)\b")
XML_LINK = re.compile(r"""href=["']test-cases/kmip-v3\.0/((?:mandatory|optional)/[^"']+\.xml)["']""")


def pinned_source(root: Path, path: Path) -> str:
    data = (root / path).read_bytes()
    if hashlib.sha256(data).hexdigest() != HASHES[path]:
        raise ValueError(f"draft source checksum mismatch: {path}")
    return data.decode("utf-8")


def normalize(name: str) -> str:
    return re.sub(r"[^a-z0-9]", "", name.casefold())


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
                branches, rules = _payload_presence_contract(cells[1], cells[2])
                targets[target_id].update(role=role, presence_branches=branches, presence_rules=rules)

    # §§11.40–11.41 incorporate the same pinned PKCS #11 3.0 source as KMIP 2.x.
    external, _ = _pkcs11_referenced_targets(root)
    for entry in external:
        section = "11.40" if entry["parent_name"] == "PKCS#11 Function" else "11.41"
        heading = targets[f"kmip30-section-{section}"]["heading"]
        target_id = f"kmip30-{entry['target_id'].removeprefix('KMIPKIT-').lower()}"
        ref = entry["source_refs"][-1]
        artifact = Path(ref["artifact"]).name
        targets[target_id] = {
            "target_id": target_id, "kind": entry["kind"], "heading": heading,
            "source": {"path": SPEC.as_posix(), "sha256": HASHES[SPEC], "section": section},
            "external_source": {
                "path": (FIXTURE_DIR.parent / "pkcs11-3.0/include" / artifact).as_posix(),
                "sha256": ref["sha256"],
            },
            "name": entry["name"], "wire_value": entry["wire_value"], "allocation": entry["allocation"],
        }
    return list(targets.values())


def _message_items(message: ET.Element, kind: str, path: str) -> list[ET.Element]:
    version = message.find(f"{kind}Header/ProtocolVersion")
    if version is None or [
        (child.tag, child.get("value")) for child in version
    ] != [("ProtocolVersionMajor", "3"), ("ProtocolVersionMinor", "0")]:
        raise ValueError(f"expected KMIP 3.0 {kind} version: {path}")
    items = message.findall("BatchItem")
    if not items:
        raise ValueError(f"missing {kind} BatchItem: {path}")
    return items


def _field_index(targets: list[dict]) -> dict:
    by_field = defaultdict(list)
    for target in targets:
        op = OPERATION.fullmatch(target["heading"])
        if op and target["kind"] == "payload_field":
            field = target["cells"][0]
            by_field[normalize(op[2]), target["role"], normalize(field)].append(target["target_id"])
    return by_field


def xml_rows(root: Path, targets: list[dict]):
    directory = root / FIXTURE_DIR
    expected = set(XML_LINK.findall(pinned_source(root, PROFILES)))
    checksums = {}
    for line in (directory / "CHECKSUMS.sha256").read_text().splitlines():
        digest, relative = line.split("  ", 1)
        if relative in checksums:
            raise ValueError(f"duplicate fixture checksum: {relative}")
        checksums[relative] = digest
    files = {path.relative_to(directory).as_posix() for path in directory.glob("*/*.xml")}
    if not expected or files != expected or checksums.keys() != expected:
        raise ValueError("KMIP 3.0 XML fixtures must match the pinned OASIS profile links and checksum inventory")
    by_field = _field_index(targets)
    transport_cases = {"MSGENC-HTTPS-M-1-30": "https"}
    encoding_cases = {"MSGENC-JSON-M-1-30": "JSON", "MSGENC-XML-M-1-30": "XML"}

    for relative in sorted(expected):
        path = directory / relative
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != checksums[relative]:
            raise ValueError(f"fixture checksum mismatch: {relative}")
        if re.search(rb"<!\s*(?:DOCTYPE|ENTITY)\b", data, re.IGNORECASE):
            raise ValueError(f"unexpected XML declaration in fixture: {relative}")
        fixture = ET.fromstring(data)
        if fixture.tag != "KMIP" or not len(fixture) or len(fixture) % 2:
            raise ValueError(f"expected paired messages: {relative}")
        for step in range(0, len(fixture), 2):
            request, response = fixture[step:step + 2]
            if (request.tag, response.tag) != ("RequestMessage", "ResponseMessage"):
                raise ValueError(f"unexpected message order: {relative}")
            requests = _message_items(request, "Request", relative)
            responses = _message_items(response, "Response", relative)
            for item, (sent, received) in enumerate(zip(requests, responses, strict=True)):
                operation = sent.find("Operation")
                response_operation = received.find("Operation")
                status = received.find("ResultStatus")
                reason = received.find("ResultReason")
                if operation is None or response_operation is None or status is None:
                    raise ValueError(f"missing operation or result status: {relative}")
                name = operation.attrib["value"]
                if name != response_operation.attrib["value"]:
                    raise ValueError(f"operation mismatch: {relative}")

                observations = defaultdict(set)
                for role, message in (("request", sent), ("response", received)):
                    payload = message.find(role.title() + "Payload")
                    for node in () if payload is None else payload:
                        matches = by_field[normalize(name), role, normalize(node.tag)]
                        if len(matches) == 1:
                            observations[role].add(matches[0])
                yield {
                    "protocol_version": "3.0", "origin": "xml", "kind": "scenario_item",
                    "case_id": path.stem, "step_index": step // 2, "item_index": item,
                    "operation": name, "result_status": status.attrib["value"],
                    "result_reason": reason.get("value") if reason is not None else None,
                    "fixture": {"path": (FIXTURE_DIR / relative).as_posix(),
                                "sha256": checksums[relative], "category": relative.split("/")[0]},
                    "encoding": encoding_cases.get(path.stem),
                    "transport": transport_cases.get(path.stem),
                    "observed_fields": {role: sorted(ids) for role, ids in sorted(observations.items())},
                    "execution_status": "required", "evidence_collected": False,
                }


def iter_draft_cells(root: Path, artifact_commit: str):
    targets = source_inventory(root)
    scenarios = list(xml_rows(root, targets))
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
