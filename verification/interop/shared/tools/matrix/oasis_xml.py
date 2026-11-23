"""Index ordered OASIS profile XML exchanges for the shared matrix."""

from __future__ import annotations

import hashlib
import re
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path

from . import OASIS_FIXTURES
PROFILES = {
    "2.0": ("kmip-profiles-v2.0.html", "3c48352c8c5c201461dba76315af5e4d81107577be52c89573a122ee3f975949"),
    "2.1": ("kmip-profiles-v2.1-os.html", "f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9"),
    "3.0": ("kmip-profiles-v3.0.html", "2485e3f5c82d2ca42f43bbcaa4b4b2d8d7853eaf8f4208742c244071de6aab66"),
}


def normalize(name: str) -> str:
    return re.sub(r"[^a-z0-9]", "", name.casefold())


def legacy_field_index(targets: list[dict], version: str) -> dict:
    by_field = defaultdict(list)
    for target in targets:
        if target["kind"] == "operation_payload_field" and version in target["protocol_versions"]:
            key = normalize(target["operation"]), target["role"], normalize(target["name"])
            by_field[key].append(target["target_id"])
    return by_field


def _batch_id(item: ET.Element) -> str | None:
    identifier = item.find("UniqueBatchItemID")
    return identifier.get("value") if identifier is not None else None


def _message_items(message: ET.Element, kind: str, path: str, protocol_version: str) -> list[ET.Element]:
    major, minor = protocol_version.split(".")
    version = message.find(f"{kind}Header/ProtocolVersion")
    if version is None or [
        (child.tag, child.get("value")) for child in version
    ] != [("ProtocolVersionMajor", major), ("ProtocolVersionMinor", minor)]:
        raise ValueError(f"expected KMIP {protocol_version} {kind} version: {path}")
    items = message.findall("BatchItem")
    if not items:
        raise ValueError(f"missing {kind} BatchItem: {path}")
    return items


def xml_rows(root: Path, version: str, by_field: dict):
    fixture_dir = OASIS_FIXTURES / f"kmip-{version}"
    directory = root / fixture_dir
    filename, expected_digest = PROFILES[version]
    profile = (root / f"verification/conformance/oasis/kmip-{version}/upstream" / filename).read_bytes()
    if hashlib.sha256(profile).hexdigest() != expected_digest:
        raise ValueError(f"KMIP {version} profile checksum mismatch: {filename}")
    prefix = re.escape(f"test-cases/kmip-v{version}/".encode())
    links = re.findall(rb"""href=["']""" + prefix + rb"""((?:mandatory|optional)/[^"']+\.xml)["']""", profile)
    expected = {link.decode("ascii") for link in links}
    suffix = version.replace(".", "")
    # Both 2.x §5.9.7.13 headings name M-13 but point to the M-14 file.
    if version in {"2.0", "2.1"}:
        expected.add(f"mandatory/CS-BC-M-13-{suffix}.xml")
    checksums = {}
    for line in (directory / "CHECKSUMS.sha256").read_text().splitlines():
        digest, relative = line.split("  ", 1)
        if relative in checksums:
            raise ValueError(f"duplicate fixture checksum: {relative}")
        checksums[relative] = digest
    files = {path.relative_to(directory).as_posix() for path in directory.glob("*/*.xml")}
    if not expected or files != expected or checksums.keys() != expected:
        raise ValueError(f"KMIP {version} XML fixtures must match the pinned OASIS profile links and checksum inventory")
    transport_cases = {f"MSGENC-HTTPS-M-1-{suffix}": "https"}
    encoding_cases = {f"MSGENC-JSON-M-1-{suffix}": "JSON", f"MSGENC-XML-M-1-{suffix}": "XML"}

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
            requests = _message_items(request, "Request", relative, version)
            responses = _message_items(response, "Response", relative, version)
            for item, (sent, received) in enumerate(zip(requests, responses, strict=True)):
                operation = sent.find("Operation")
                response_operation = received.find("Operation")
                status = received.find("ResultStatus")
                reason = received.find("ResultReason")
                if operation is None or response_operation is None or status is None:
                    raise ValueError(f"missing operation or result status: {relative}")
                if version != "3.0" and _batch_id(sent) != _batch_id(received):
                    raise ValueError(f"batch item identifier mismatch: {relative}")
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
                    "protocol_version": version, "origin": "xml", "kind": "scenario_item",
                    "case_id": path.stem, "step_index": step // 2, "item_index": item,
                    "operation": name, "result_status": status.attrib["value"],
                    "result_reason": reason.get("value") if reason is not None else None,
                    "fixture": {"path": (fixture_dir / relative).as_posix(),
                                "sha256": checksums[relative], "category": relative.split("/")[0]},
                    "encoding": encoding_cases.get(path.stem),
                    "transport": transport_cases.get(path.stem),
                    "observed_fields": {role: sorted(ids) for role, ids in sorted(observations.items())},
                    "execution_status": "required", "evidence_collected": False,
                }
