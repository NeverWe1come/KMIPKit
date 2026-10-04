"""Contract tests for the offline normative-catalog validator."""

from __future__ import annotations

import json
import hashlib
import re
import subprocess
import sys
import unittest
from html.parser import HTMLParser
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.validate import (
    CatalogValidationError,
    _JsonPreflight,
    _check_operation_inventory,
    _check_tag_registry,
    validate_catalog,
)
import tools.normative_catalog.validate as catalog_validate


ROOT = Path(__file__).resolve().parents[3]


OFFICIAL_NON_OPERATION_TEST_ELEMENTS = {
            "KMIPKIT-TEST-CN01-2-11": {
                "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-63-SUBJECT-DISTINGUISHED-NAME",
                "KMIPKIT-ELEM-OBJECT-TYPE-CERTIFICATE",
            },
            "KMIPKIT-TEST-CN01-2-12": {"KMIPKIT-ELEM-OBJECT-TYPE-SECRET-DATA"},
            "KMIPKIT-TEST-CN01-2-13": {"KMIPKIT-ELEM-TAG-420105", "KMIPKIT-ELEM-TAG-420106"},
            "KMIPKIT-TEST-CN01-2-14": {"KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-256-00000006"},
            "KMIPKIT-TEST-CN01-2-15": {"KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA256-00000009"},
            "KMIPKIT-TEST-CN01-2-16": {"KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001"},
            "KMIPKIT-TEST-CN01-2-17": {"KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001"},
            "KMIPKIT-TEST-CN01-2-18": {
                "KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001",
                "KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-256-00000006",
            },
            "KMIPKIT-TEST-CN01-2-34": {
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-ECPRIVATEKEY-00000006",
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005",
                "KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A",
                "KMIPKIT-ELEM-OBJECT-TYPE-PRIVATE-KEY",
            },
            "KMIPKIT-TEST-CN01-2-35": {
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-8-00000004",
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005",
                "KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A",
                "KMIPKIT-ELEM-OBJECT-TYPE-PRIVATE-KEY",
            },
            "KMIPKIT-TEST-CN01-2-36": {
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005",
                "KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A",
                "KMIPKIT-ELEM-OBJECT-TYPE-PUBLIC-KEY",
            },
            "KMIPKIT-TEST-CN01-2-37": {
                "KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDSA-00000006",
                "KMIPKIT-ELEM-OBJECT-TYPE-PRIVATE-KEY",
                "KMIPKIT-ELEM-OBJECT-TYPE-PUBLIC-KEY",
            },
            "KMIPKIT-TEST-CN01-2-38": {
                "KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDSA-00000006",
                "KMIPKIT-ELEM-TAG-420107",
            },
            "KMIPKIT-TEST-CN01-2-39": {
                "KMIPKIT-ELEM-ATTRIBUTE-EXTRACTABLE",
                "KMIPKIT-ELEM-ATTRIBUTE-NEVER-EXTRACTABLE",
            },
            "KMIPKIT-TEST-CN01-2-40": {"KMIPKIT-ELEM-ATTRIBUTE-NAME"},
            "KMIPKIT-TEST-CN01-2-41": {"KMIPKIT-ELEM-ATTRIBUTE-ALTERNATIVE-NAME"},
            "KMIPKIT-TEST-CN01-2-42": {"KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-VALUE"},
            "KMIPKIT-TEST-CN01-2-61": {"KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-KEY-MATERIAL"},
            "KMIPKIT-TEST-CN01-2-62": {"KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-KEY-MATERIAL"},
            "KMIPKIT-TEST-CN01-2-63": {
                "KMIPKIT-ELEM-ATTRIBUTE-KEY-VALUE-LOCATION",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-2-KEY-VALUE-KEY-MATERIAL",
            },
            "KMIPKIT-TEST-CN01-2-64": {"KMIPKIT-ELEM-TAG-420058"},
            "KMIPKIT-TEST-CN01-2-65": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-66": {"KMIPKIT-ELEM-ATTRIBUTE-LINK"},
            "KMIPKIT-TEST-CN01-2-68": {"KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-12-00000016"},
            "KMIPKIT-TEST-CN01-2-69": {"KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-12-00000016"},
            "KMIPKIT-TEST-CN01-2-88": {"KMIPKIT-ELEM-ATTRIBUTE-RANDOM-NUMBER-GENERATOR"},
            "KMIPKIT-TEST-CN01-2-89": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-91": {
                "KMIPKIT-ELEM-ATTRIBUTE-SENSITIVE",
                "KMIPKIT-ELEM-ATTRIBUTE-ALWAYS-SENSITIVE",
            },
            "KMIPKIT-TEST-CN01-2-95": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-96": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-97": {"KMIPKIT-ELEM-OBJECT-TYPE-SPLIT-KEY"},
            "KMIPKIT-TEST-CN01-2-98": {
                "KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY",
                "KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-XOR-00000001",
            },
            "KMIPKIT-TEST-CN01-2-99": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-100": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-101": {"KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY"},
            "KMIPKIT-TEST-CN01-2-108": {
                "KMIPKIT-ELEM-ENUMERATION-KEY-WRAP-TYPE",
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-AS-REGISTERED-00000002",
            },
            "KMIPKIT-TEST-CN01-2-109": {
                "KMIPKIT-ELEM-ENUMERATION-KEY-WRAP-TYPE",
                "KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-NOT-WRAPPED-00000001",
            },
            "KMIPKIT-TEST-PROF-5-3-3-1": {
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES",
            },
            "KMIPKIT-TEST-PROF-5-4-4-1": {
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES",
            },
            "KMIPKIT-TEST-PROF-5-5-4-1": {
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES",
                "KMIPKIT-ELEM-TAG-42000D",
                "KMIPKIT-ELEM-TAG-42000F",
                "KMIPKIT-ELEM-TAG-420050",
                "KMIPKIT-ELEM-TAG-420057",
                "KMIPKIT-ELEM-TAG-420069",
                "KMIPKIT-ELEM-TAG-42006A",
                "KMIPKIT-ELEM-TAG-42006B",
                "KMIPKIT-ELEM-TAG-420074",
                "KMIPKIT-ELEM-TAG-420077",
                "KMIPKIT-ELEM-TAG-420078",
                "KMIPKIT-ELEM-TAG-420079",
                "KMIPKIT-ELEM-TAG-42007A",
                "KMIPKIT-ELEM-TAG-42007B",
                "KMIPKIT-ELEM-TAG-42007C",
                "KMIPKIT-ELEM-TAG-42007D",
                "KMIPKIT-ELEM-TAG-42007E",
                "KMIPKIT-ELEM-TAG-42007F",
                "KMIPKIT-ELEM-TAG-420092",
                "KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OPERATIONS-00000001",
                "KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OBJECTS-00000002",
                "KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-FAILED-00000001",
                "KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-SUCCESS-00000000",
                "KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESPONSE-TOO-LARGE-00000002",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-00000001",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SYMMETRIC-KEY-00000002",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SECRET-DATA-00000007",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PUBLIC-KEY-00000003",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PRIVATE-KEY-00000004",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-OPAQUE-OBJECT-00000008",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SPLIT-KEY-00000005",
                "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PGP-KEY-00000009",
            },
            "KMIPKIT-TEST-PROF-5-11-3-1": {
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-37-SERVER-INFORMATION",
            },
            "KMIPKIT-TEST-PROF-5-11-3-2": {"KMIPKIT-ELEM-OBJECT-TYPE-SECRET-DATA"},
            "KMIPKIT-TEST-PROF-5-12-6-1": {
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-25-OPERATIONS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-21-OBJECTS",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-24-OBJECT-TYPES",
                "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-37-SERVER-INFORMATION",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-4-APPLICATION-NAMESPACE",
            },
            "KMIPKIT-TEST-PROF-5-12-6-2": {
                "KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY",
                "KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-4-APPLICATION-NAMESPACE",
                "KMIPKIT-ELEM-ATTRIBUTE-APPLICATION-SPECIFIC-INFORMATION",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-1-BATCH-ITEM",
            },
            "KMIPKIT-TEST-PROF-5-12-6-3": {
                "KMIPKIT-ELEM-ATTRIBUTE-APPLICATION-SPECIFIC-INFORMATION",
                "KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER",
                "KMIPKIT-ELEM-OBJECT-TYPE-SYMMETRIC-KEY",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-1-BATCH-ITEM",
            },
            "KMIPKIT-TEST-PROF-5-17-2": {"KMIPKIT-ELEM-ATTRIBUTE-PROTECTION-PERIOD"},
        }




class _CaptionedTableParser(HTMLParser):
    """Read table rows and captions from a pinned legacy OASIS HTML source."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.tables: list[dict[str, object]] = []
        self.headings: list[str] = []
        self._heading = ""
        self._heading_tag: str | None = None
        self._heading_buffer: list[str] | None = None
        self._sup_depth = 0
        self._table: dict[str, object] | None = None
        self._row: list[str] | None = None
        self._cell: list[str] | None = None
        self._caption: list[str] | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag in {"h1", "h2", "h3", "h4"}:
            self._heading_tag = tag
            self._heading_buffer = []
        elif tag == "table":
            self._table = {"heading": self._heading, "rows": []}
            self.tables.append(self._table)
        elif tag == "tr" and self._table is not None:
            self._row = []
            self._table["rows"].append(self._row)
        elif tag in {"td", "th"} and self._row is not None:
            self._cell = []
        elif tag == "p" and attributes.get("class") == "MsoCaption":
            self._caption = []
        elif tag == "sup":
            self._sup_depth += 1

    def handle_data(self, data: str) -> None:
        if self._sup_depth:
            data = data.translate(str.maketrans("0123456789+-=()", "⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾"))
        if self._cell is not None:
            self._cell.append(data)
        if self._caption is not None:
            self._caption.append(data)
        if self._heading_buffer is not None:
            self._heading_buffer.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag in {"td", "th"} and self._cell is not None:
            assert self._row is not None
            self._row.append(" ".join("".join(self._cell).split()))
            self._cell = None
        elif tag == "tr":
            self._row = None
        elif tag == "table":
            self._table = None
        elif tag == "p" and self._caption is not None:
            if self.tables:
                self.tables[-1]["caption"] = " ".join("".join(self._caption).split())
            self._caption = None
        elif tag == "sup" and self._sup_depth:
            self._sup_depth -= 1
        elif tag == self._heading_tag and self._heading_buffer is not None:
            self._heading = " ".join("".join(self._heading_buffer).split())
            self.headings.append(self._heading)
            self._heading_tag = None
            self._heading_buffer = None


def _pinned_tag_rows() -> tuple[list[tuple[str, str, str]], list[tuple[str, str]]]:
    """Return singleton and range rows from the checksum-pinned Table 487."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    table = next(item for item in parser.tables if item.get("caption") == "Table 487: Tag Enumeration")
    rows = table["rows"]
    assert isinstance(rows, list)
    singletons: list[tuple[str, str, str]] = []
    ranges: list[tuple[str, str]] = []
    for row in rows[2:]:
        if len(row) != 2:
            continue
        name, value = row
        if re.fullmatch(r"(?:0x)?[0-9A-Fa-f]{6}", value):
            allocation = "reserved" if name in {"(Reserved)", "Reserved"} else "assigned"
            singletons.append((name, value, allocation))
        else:
            allocation = {"(Unused)": "unused", "(Reserved)": "reserved", "Extensions": "extension"}.get(name)
            if allocation is None:
                raise AssertionError("unclassified tag range in pinned Table 487")
            ranges.append((allocation, value))
    return singletons, ranges


def _pinned_enumeration_groups() -> dict[str, tuple[str, list[tuple[str, str]]]]:
    """Return every §11 enumeration heading and its literal Name/Value rows."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    groups: dict[str, tuple[str, list[tuple[str, str]]]] = {}
    for table in parser.tables:
        heading = table.get("heading")
        if not isinstance(heading, str):
            continue
        match = re.match(r"^(11\.[0-9]+)\s+(.+?)\s+Enumeration$", heading)
        if not match:
            continue
        section, name = match.groups()
        rows = table.get("rows")
        if not isinstance(rows, list):
            continue
        header_index = next(
            (index for index, row in enumerate(rows) if [cell.casefold() for cell in row] == ["name", "value"]),
            None,
        )
        if header_index is None:
            continue
        values = [tuple(row) for row in rows[header_index + 1 :] if len(row) == 2]
        groups[section] = (name, values)

    headings: dict[str, str] = {}
    for heading in parser.headings:
        match = re.match(r"^(11\.[0-9]+)\s+(.+?)\s+Enumeration$", heading)
        if match:
            headings[match.group(1)] = match.group(2)
    for section, name in headings.items():
        groups.setdefault(section, (name, []))
    return groups


def _pinned_bitmask_groups() -> dict[str, tuple[str, list[tuple[str, str]]]]:
    """Return the three §12 bitmask enumerations from the pinned Specification."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    groups: dict[str, tuple[str, list[tuple[str, str]]]] = {}
    for table in parser.tables:
        caption = table.get("caption")
        if not isinstance(caption, str) or not re.match(r"Table 49[6-8]:", caption):
            continue
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list):
            continue
        match = re.match(r"^(12\.[1-3])\s+(.+)$", heading)
        if not match:
            continue
        section, name = match.groups()
        header_index = next(
            (index for index, row in enumerate(rows) if [cell.casefold() for cell in row] == ["name", "value"]),
            None,
        )
        if header_index is None:
            raise AssertionError(f"pinned bitmask table has no Name/Value header: {caption}")
        groups[section] = (name, [tuple(row) for row in rows[header_index + 1 :] if len(row) == 2])
    return groups


def _pinned_attribute_headings() -> dict[str, str]:
    """Return every attribute heading and section from Specification §4."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    result: dict[str, str] = {}
    for heading in parser.headings:
        match = re.match(r"^(4\.[0-9]+)\s+(.+)$", heading)
        if match:
            section, name = match.groups()
            result[section] = name
    return result


def _pinned_attribute_structures() -> tuple[dict[str, tuple[str, str]], set[tuple[str, str, str, str]]]:
    """Return §5 structure roots and literal member rows from Tables 157–163."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    structures: dict[str, tuple[str, str]] = {}
    members: set[tuple[str, str, str, str]] = set()
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list):
            continue
        match = re.match(r"^(5\.[1-7])\s+(.+)$", heading)
        if not match or not rows or rows[0][:3] not in (["Item", "Encoding", "REQUIRED"], ["Object", "Encoding", "REQUIRED"]):
            continue
        section, _ = match.groups()
        if len(rows) < 2 or len(rows[1]) != 3:
            raise AssertionError(f"pinned attribute structure table is incomplete: {heading}")
        root_name, root_encoding, _ = rows[1]
        if root_encoding == "Structure":
            structures[section] = (root_name, root_encoding)
            member_rows = rows[2:]
        else:
            member_rows = rows[1:]
        for row in member_rows:
            if len(row) == 3:
                member_name, encoding, requiredness = row
                members.add((section, member_name, encoding, requiredness))
    return structures, members


def _pinned_attribute_value_structures() -> tuple[dict[str, tuple[str, str, str | None]], set[tuple[str, str, str, str | None]]]:
    """Return every §4 Structure-valued attribute root and its literal members."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    structures: dict[str, tuple[str, str, str | None]] = {}
    members: set[tuple[str, str, str, str | None]] = set()
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list) or not rows:
            continue
        match = re.match(r"^(4\.[0-9]+)\s+", heading)
        if not match:
            continue
        section = match.group(1)
        headers = [cell.strip().casefold() for cell in rows[0]]
        if "encoding" not in headers:
            continue
        encoding_index = headers.index("encoding")
        required_index = next((index for index, value in enumerate(headers) if value == "required"), None)
        structure_row_index = next(
            (
                index for index, row in enumerate(rows[1:], start=1)
                if len(row) > encoding_index and row[encoding_index] == "Structure"
            ),
            None,
        )
        if structure_row_index is None:
            continue
        root_row = rows[structure_row_index]
        requiredness = root_row[required_index] if required_index is not None and len(root_row) > required_index else None
        structures[section] = (root_row[0], root_row[encoding_index], requiredness)
        for row in rows[structure_row_index + 1 :]:
            if len(row) > encoding_index:
                raw_requiredness = row[required_index] if required_index is not None else None
                members.add((section, row[0], row[encoding_index], raw_requiredness))
    return structures, members


def _pinned_attribute_encodings() -> dict[str, tuple[str, str, str | None, str]]:
    """Return attribute names, encodings, and raw requiredness from the source tables."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    headings = _pinned_attribute_headings()
    aliases = {"4.3": "Sensitive", "4.9": "Description", "4.60": "Attribute"}
    result: dict[str, tuple[str, str, str | None, str]] = {}
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list) or not rows:
            continue
        match = re.match(r"^(4\.[0-9]+)\s+", heading)
        if not match:
            continue
        section = match.group(1)
        headers = [cell.strip().casefold() for cell in rows[0]]
        if not headers or headers[0] not in {"item", "object"} or "encoding" not in headers:
            continue
        encoding_index = headers.index("encoding")
        required_index = next((index for index, value in enumerate(headers) if value == "required"), None)
        source_name = aliases.get(section, headings[section])
        root_row = next(
            (row for row in rows[1:] if row and row[0] == source_name and len(row) > encoding_index),
            None,
        )
        if root_row is not None:
            requiredness = root_row[required_index] if required_index is not None and len(root_row) > required_index else None
            result[section] = (source_name, root_row[encoding_index], requiredness, section)
    usage_limits = next(
        table for table in parser.tables
        if table.get("heading", "").startswith("7.40 ")
        and table.get("rows", [])[1][0] == "Usage Limits"
    )
    result["4.59"] = ("Usage Limits", usage_limits["rows"][1][1], None, "7.40")
    return result


def _pinned_operation_structures() -> tuple[dict[str, tuple[str, str, str | None]], set[tuple[str, str, str, str | None]]]:
    """Return §7 operation data structure roots and literal members from Tables 352–393."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    structures: dict[str, tuple[str, str, str | None]] = {}
    members: set[tuple[str, str, str, str | None]] = set()
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list) or len(rows) < 2:
            continue
        match = re.match(r"^(7\.[0-9]+)\s+", heading)
        if not match:
            continue
        headers = [cell.strip().casefold() for cell in rows[0]]
        if len(headers) < 2 or headers[0] not in {"object", "item"} or headers[1] != "encoding":
            continue
        section = match.group(1)
        encoding_index = 1
        required_index = next((index for index, value in enumerate(headers) if value == "required"), None)
        root_row = rows[1]
        if len(root_row) <= encoding_index:
            raise AssertionError(f"pinned operation structure table is incomplete: {heading}")
        requiredness = root_row[required_index] if required_index is not None and len(root_row) > required_index else None
        structures[section] = (root_row[0], root_row[encoding_index], requiredness)
        for row in rows[2:]:
            if len(row) > encoding_index:
                raw_requiredness = row[required_index] if required_index is not None and len(row) > required_index else None
                members.add((section, row[0], row[encoding_index], raw_requiredness))
    return structures, members


def _pinned_message_structure_rows() -> set[tuple[str, str, str | None, str | None, str | None]]:
    """Return exact root and member rows from the six §8 message structure tables."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    records: set[tuple[str, str, str | None, str | None, str | None]] = set()
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list):
            continue
        match = re.match(r"^(8\.[1-6])\s+", heading)
        if not match:
            continue
        section = match.group(1)
        header_index = next(
            (
                index for index, row in enumerate(rows)
                if len(row) > 1 and row[0].strip().casefold() == "object"
                and row[1].strip().casefold() in {"encoding", "required in message"}
            ),
            None,
        )
        if header_index is None:
            continue
        headers = [cell.strip().casefold() for cell in rows[header_index]]
        encoding_index = headers.index("encoding") if "encoding" in headers else None
        required_index = next((index for index, value in enumerate(headers) if value.startswith("required")), None)
        comment_index = headers.index("comment") if "comment" in headers else None
        data_rows = [row for row in rows[header_index + 1 :] if row]
        if not data_rows:
            continue
        for row in data_rows:
            name = row[0]
            encoding = row[encoding_index] if encoding_index is not None and len(row) > encoding_index else None
            requiredness = row[required_index] if required_index is not None and len(row) > required_index else None
            comment = row[comment_index] if comment_index is not None and len(row) > comment_index else None
            if encoding_index is None and name in {"Request Header", "Response Header", "Batch Item"} and comment == "Structure":
                encoding = comment
            records.add((section, name, encoding, requiredness, comment or None))
    return records


def _pinned_message_field_types() -> set[tuple[str, str, str, str | None]]:
    """Return the non-credential field types and nested rows in Tables 400–426."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    records: set[tuple[str, str, str, str | None]] = set()
    for table in parser.tables:
        caption = table.get("caption")
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(caption, str) or not isinstance(heading, str) or not isinstance(rows, list) or not rows:
            continue
        table_match = re.match(r"^Table\s+(\d+)", caption)
        if not table_match or not (400 <= int(table_match.group(1)) <= 409 or 417 <= int(table_match.group(1)) <= 426):
            continue
        section_match = re.match(r"^(9\.[0-9]+)\s+", heading)
        if not section_match:
            continue
        section = section_match.group(1)
        headers = [cell.strip().casefold() for cell in rows[0]]
        if len(headers) < 2 or headers[0] != "object" or headers[1] != "encoding":
            continue
        required_index = next((index for index, value in enumerate(headers) if value == "required"), None)
        for row in rows[1:]:
            if len(row) > 1:
                requiredness = row[required_index] if required_index is not None and len(row) > required_index else None
                records.add((section, row[0], row[1], requiredness))
    return records


def _pinned_credential_forms() -> tuple[dict[str, tuple[str, str]], set[tuple[str, str, str, str]]]:
    """Return the Credential root, six forms, and literal member rows from Tables 410–416."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    roots: dict[str, tuple[str, str]] = {}
    members: set[tuple[str, str, str, str]] = set()
    for table in parser.tables:
        caption = table.get("caption")
        rows = table.get("rows")
        if not isinstance(caption, str) or not isinstance(rows, list) or not rows:
            continue
        table_match = re.match(r"^Table\s+(41[0-6])\s*:", caption)
        if not table_match:
            continue
        table_number = int(table_match.group(1))
        headers = [cell.strip().casefold() for cell in rows[0]]
        if len(headers) < 3 or headers[:2] != ["object", "encoding"]:
            continue
        root_row = rows[1]
        form_name = "Credential" if table_number == 410 else re.search(
            r"Credential Value Structure for the (.+?)(?: Credential)?$", caption
        ).group(1)
        roots[form_name] = (root_row[1], root_row[2] if len(root_row) > 2 else "")
        for row in rows[2:]:
            if len(row) >= 3:
                members.add((form_name, row[0], row[1], row[2]))
    return roots, members


def _pinned_object_structures() -> tuple[
    dict[tuple[str, str], tuple[str, str]],
    set[tuple[str, str, str, str, str]],
]:
    """Return object-structure roots and literal members from Specification §§2–3.12."""
    source_path = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
    raw = source_path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf":
        raise AssertionError("pinned KMIP Specification checksum changed")
    parser = _CaptionedTableParser()
    parser.feed(raw.decode("cp1252"))
    roots: dict[tuple[str, str], tuple[str, str]] = {}
    members: set[tuple[str, str, str, str, str]] = set()
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        if not isinstance(heading, str) or not isinstance(rows, list):
            continue
        match = re.match(r"^(2\.[1-9]|3\.(?:[1-9]|1[0-2]))\s+", heading)
        if not match or not rows or rows[0] != ["Object", "Encoding", "REQUIRED"]:
            continue
        section = match.group(1)
        if len(rows) < 2 or len(rows[1]) != 3:
            raise AssertionError(f"pinned object-structure table is incomplete: {heading}")
        root_name, root_encoding, root_requiredness = rows[1]
        if root_encoding != "Structure":
            raise AssertionError(f"pinned object-structure root is not Structure: {heading}")
        root_key = (section, root_name)
        if root_key in roots:
            raise AssertionError(f"duplicate pinned object-structure root: {heading}")
        roots[root_key] = (root_encoding, root_requiredness)
        for row in rows[2:]:
            if len(row) == 3:
                members.add((section, root_name, row[0], row[1], row[2]))
    return roots, members


def minimal_catalog() -> dict[str, object]:
    """Return the smallest catalog with exact pinned source metadata."""
    source_rows = [
        (
            "profiles",
            "Key Management Interoperability Protocol Profiles Version 2.1",
            "OASIS Standard",
            "14 December 2020",
            "upstream/kmip-profiles-v2.1-os.html",
            "https://docs.oasis-open.org/kmip/kmip-profiles/v2.1/os/kmip-profiles-v2.1-os.html",
            "f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9",
            "profile_normative",
        ),
        (
            "spec",
            "Key Management Interoperability Protocol Specification Version 2.1",
            "OASIS Standard",
            "14 December 2020",
            "upstream/kmip-spec-v2.1-os.html",
            "https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html",
            "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf",
            "primary_normative",
        ),
        (
            "testcases",
            "Key Management Interoperability Protocol Test Cases Version 2.1",
            "Committee Note 01",
            "16 November 2020",
            "upstream/kmip-testcases-v2.1-cn01.html",
            "https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/kmip-testcases-v2.1-cn01.html",
            "305e0638af25aa8163a2b4a67a1a617a0207af67b3b3032fa499966b22d42b88",
            "test_evidence",
        ),
        (
            "usage-guide",
            "Key Management Interoperability Protocol Usage Guide Version 2.1",
            "Committee Note 01",
            "16 November 2020",
            "upstream/kmip-ug-v2.1-cn01.html",
            "https://docs.oasis-open.org/kmip/kmip-ug/v2.1/cn01/kmip-ug-v2.1-cn01.html",
            "f6997ac3ac34d588b2b43ace49835dceaa6af2086f9472a79c27bce43f5cbe38",
            "informative",
        ),
    ]
    sources = [
        {
            "source_id": f"KMIPKIT-SRC-{slug}",
            "title": title,
            "version": "2.1",
            "stage": stage,
            "publication_date": date,
            "local_path": f"specification/oasis/kmip-2.1/{path}",
            "canonical_url": url,
            "sha256": digest,
            "authority_class": authority,
        }
        for slug, title, stage, date, path, url, digest, authority in source_rows
    ]
    return {
        "schema_version": 1,
        "sources": sources,
        "source_clauses": [],
        "elements": [],
        "tag_ranges": [],
        "requirements": [],
        "policies": [],
        "profiles": [],
        "test_cases": [],
        "discrepancies": [],
        "decisions": [],
    }


def validate(document: dict[str, object]) -> dict[str, object]:
    return validate_catalog(json.dumps(document).encode("utf-8"), ROOT)


def test_case(*, fixture_path: str | None, fixture_availability: str) -> dict[str, object]:
    return {
        "test_id": "KMIPKIT-TEST-SPEC-001",
        "official_case_id": "KMIP-TC-001",
        "source_id": "KMIPKIT-SRC-testcases",
        "source_section": "1",
        "evidence_category": "conformance",
        "mandatory_status": "unknown",
        "profile_ids": [],
        "requirement_ids": [],
        "element_ids": [],
        "raw_href": "../fixtures/TC-001.xml",
        "fixture_path": fixture_path,
        "fixture_availability": fixture_availability,
        "mapping_confidence": "unmapped",
    }


def profile_record(**overrides: object) -> dict[str, object]:
    record: dict[str, object] = {
        "profile_id": "KMIPKIT-PROFILE-BASELINE",
        "name": "Baseline profile",
        "role": "client",
        "source_refs": [{"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"}],
        "source_clause_ids": [],
        "dependency_profile_ids": [],
        "transport_requirements": ["ttlv_tls"],
        "encoding_requirements": ["ttlv"],
        "applicability": "client_1_0",
        "claim_state": "not_claimed",
        "requirement_ids": [],
        "element_ids": [],
        "test_case_ids": [],
    }
    record.update(overrides)
    return record


def discrepancy_record(**overrides: object) -> dict[str, object]:
    record: dict[str, object] = {
        "discrepancy_id": "KMIPKIT-DISC-001",
        "summary": "Conflicting continuation behavior wording.",
        "source_refs": [
            {"source_id": "KMIPKIT-SRC-spec", "section": "11.5"},
            {"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"},
        ],
        "source_authority": "primary_normative",
        "normative_status": "normative_conflict",
        "alternatives": ["continue", "stop"],
        "affected_requirement_ids": [],
        "affected_element_ids": [],
        "affected_profile_ids": [],
        "affected_policy_ids": [],
        "downstream_impact": "Batch error behavior remains gated.",
        "state": "open",
        "decision_id": None,
        "erratum_source_refs": [],
    }
    record.update(overrides)
    return record


def decision_record(**overrides: object) -> dict[str, object]:
    record: dict[str, object] = {
        "decision_id": "KMIPKIT-DEC-001",
        "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
        "requirement_ids": [],
        "discrepancy_ids": [],
        "policy_ids": [],
        "interpretation": "Use the reviewed client interpretation.",
        "approver": "Qualified reviewer",
        "approval_evidence": "https://example.invalid/approval",
        "approved_at": "2026-10-04",
        "consequence": "The affected requirement is implementable.",
        "status": "accepted",
    }
    record.update(overrides)
    return record


def deviation_catalog(decision: dict[str, object]) -> dict[str, object]:
    document = minimal_catalog()
    document["source_clauses"] = [
        {
            "clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "source_id": "KMIPKIT-SRC-spec",
            "section": "8.1",
            "locator": {"ordinal": 1, "block_kind": "paragraph"},
            "source_keywords": ["SHOULD"],
            "disposition": "requirement",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
            "exclusion_rationale": None,
            "role": "client",
            "direction": "client_to_server",
            "scope_state": "client_1_0",
            "condition": None,
        }
    ]
    document["requirements"] = [
        {
            "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
            "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
            "source_keyword": "SHOULD",
            "normative_strength": "recommended",
            "subject": "client",
            "summary": "A recommended client behavior.",
            "role": "client",
            "direction": "client_to_server",
            "condition": None,
            "scope_state": "client_1_0",
            "element_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": None,
            "implementation_refs": [],
            "verification_refs": [],
            "negative_verification_required": False,
            "decision_id": "KMIPKIT-DEC-001",
            "status": "deviated",
            "review_note": "Reviewed deviation.",
        }
    ]
    document["decisions"] = [decision]
    return document


CLIENT_OPERATIONS = {
    "Activate": "6.1.1", "Add Attribute": "6.1.2", "Adjust Attribute": "6.1.3",
    "Archive": "6.1.4", "Cancel": "6.1.5", "Certify": "6.1.6", "Check": "6.1.7",
    "Create": "6.1.8", "Create Key Pair": "6.1.9", "Create Split Key": "6.1.10",
    "Decrypt": "6.1.11", "Delegated Login": "6.1.12", "Delete Attribute": "6.1.13",
    "Derive Key": "6.1.14", "Destroy": "6.1.15", "Discover Versions": "6.1.16",
    "Encrypt": "6.1.17", "Export": "6.1.18", "Get": "6.1.19",
    "Get Attributes": "6.1.20", "Get Attribute List": "6.1.21", "Get Constraints": "6.1.22",
    "Get Usage Allocation": "6.1.23", "Hash": "6.1.24", "Import": "6.1.25",
    "Interop": "6.1.26", "Join Split Key": "6.1.27", "Locate": "6.1.28", "Log": "6.1.29",
    "Login": "6.1.30", "Logout": "6.1.31", "MAC": "6.1.32", "MAC Verify": "6.1.33",
    "Modify Attribute": "6.1.34", "Obtain Lease": "6.1.35", "Ping": "6.1.36",
    "PKCS#11": "6.1.37", "Poll": "6.1.38", "Process": "6.1.39", "Query": "6.1.40",
    "Query Asynchronous Requests": "6.1.41", "Recover": "6.1.42", "Register": "6.1.43",
    "Revoke": "6.1.44", "Re-certify": "6.1.45", "Re-key": "6.1.46",
    "Re-key Key Pair": "6.1.47", "Re-Provision": "6.1.48", "RNG Retrieve": "6.1.49",
    "RNG Seed": "6.1.50", "Set Attribute": "6.1.51", "Set Constraints": "6.1.52",
    "Set Defaults": "6.1.53", "Set Endpoint Role": "6.1.54", "Sign": "6.1.55",
    "Signature Verify": "6.1.56", "Validate": "6.1.57",
}
SERVER_OPERATIONS = {
    "Discover Versions": "6.2.1", "Notify": "6.2.2", "Put": "6.2.3",
    "Query": "6.2.4", "Set Endpoint Role": "6.2.5",
}
CLIENT_OPERATION_PAYLOAD_TABLES = (
    (164, 165), (167, 168), (170, 171), (173, 174), (176, 177), (179, 180), (183, 184),
    (186, 187), (189, 190), (193, 194), (196, 197), (199, 200), (202, 203), (205, 206),
    (208, 209), (211, 212), (214, 215), (217, 218), (220, 221), (223, 224), (226, 227),
    (229, 230), (232, 233), (235, 236), (238, 239), (241, 242), (244, 245), (247, 248),
    (250, 251), (253, 254), (256, 257), (259, 260), (262, 263), (265, 266), (268, 269),
    (271, 272), (273, 274), (276, None), (278, 279), (282, 283), (285, 286), (288, 289),
    (291, 292), (295, 296), (300, 301), (305, 306), (310, 311), (313, 314), (316, 317),
    (319, 320), (322, 323), (325, 326), (328, 329), (331, 332), (334, 335), (337, 338),
    (340, 341),
)
SERVER_OPERATION_PAYLOAD_TABLES = {
    "Discover Versions": (343, None), "Notify": (None, None), "Put": (None, None),
    "Query": (347, None), "Set Endpoint Role": (349, 350),
}
RESERVED_TAGS = {
    "420009": "(Reserved)", "420014": "(Reserved)", "420015": "(Reserved)",
    "420016": "(Reserved)", "420017": "(Reserved)", "42001A": "(Reserved)",
    "42001B": "(Reserved)", "42001C": "(Reserved)", "42001F": "(Reserved)",
    "42002D": "(Reserved)", "42003B": "(Reserved)", "42005D": "(Reserved)",
    "420065": "(Reserved)", "42006E": "(Reserved)", "420087": "(Reserved)",
    "420090": "(Reserved)", "420091": "(Reserved)", "420137": "Reserved",
    "42013E": "(Reserved)", "42013F": "(Reserved)",
}
EXPECTED_TAG_RANGES = (
    ("unused", "000000 - 420000"),
    ("reserved", "420XXX \u2013 42FFFF"),
    ("unused", "430000 \u2013 53FFFF"),
    ("extension", "540000 \u2013 54FFFF"),
    ("unused", "550000 - FFFFFF"),
)


def operation_element(name: str, section: str, direction: str) -> dict[str, object]:
    slug = re.sub(r"[^A-Z0-9]+", "-", name.upper()).strip("-")
    direction_code = "C2S" if direction == "client_to_server" else "S2C"
    scope = "client_1_0" if direction == "client_to_server" else "client_1_1"
    if direction == "client_to_server":
        operation_names = list(CLIENT_OPERATIONS)
        request_table, response_table = CLIENT_OPERATION_PAYLOAD_TABLES[operation_names.index(name)]
    else:
        request_table, response_table = SERVER_OPERATION_PAYLOAD_TABLES[name]
    payload_tables: list[dict[str, object]] = []
    for role, table_number in (("request", request_table), ("response", response_table)):
        if table_number is not None:
            caption = f"{name} {role.title()} Payload"
            if name == "Query Asynchronous Requests" and role == "response":
                caption = "PKCS#11 Response Payload"
            payload_tables.append({"role": role, "table_number": table_number, "caption": caption})
    record: dict[str, object] = {
        "element_id": f"KMIPKIT-ELEM-OP-{direction_code}-{slug}",
        "kind": "operation",
        "name": name,
        "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": section}],
        "direction": direction,
        "scope_state": scope,
        "payload_tables": payload_tables,
        "asynchronous_response": {
            "Cancel": "cancellation_result_not_async",
            "Poll": "pending_or_original_operation_payload",
        }.get(name),
        "parent_element_ids": [],
        "requirement_ids": [],
        "profile_ids": [],
        "test_case_ids": [],
        "feature_spec": None,
        "implementation_refs": [],
        "verification_refs": [],
    }
    if direction == "server_to_client":
        record["scope_reason"] = "Server-initiated operation support is scheduled for KMIPKit 1.1."
    return record


class CatalogValidationTests(unittest.TestCase):
    def test_checked_in_clause_ledger_covers_all_keyword_strength_mappings(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        expected_keywords = set(catalog_validate.KEYWORD_STRENGTH)
        observed_keywords = {
            keyword for clause in catalog["source_clauses"] for keyword in clause["source_keywords"]
        }

        self.assertEqual(observed_keywords, expected_keywords)
        for requirement in catalog["requirements"]:
            self.assertEqual(
                requirement["normative_strength"],
                catalog_validate.KEYWORD_STRENGTH[requirement["source_keyword"]],
            )

    def test_checked_in_catalog_orders_evidence_policies_and_relationships_deterministically(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))

        def section_key(value: str) -> tuple[int, ...]:
            return tuple(int(part) for part in value.split("."))

        expected_tests = sorted(
            catalog["test_cases"],
            key=lambda row: (row["source_id"], section_key(row["source_section"]), row["test_id"]),
        )

        self.assertEqual(catalog["test_cases"], expected_tests)
        self.assertEqual(
            [row["policy_id"] for row in catalog["policies"]],
            sorted(row["policy_id"] for row in catalog["policies"]),
        )
        relationship_fields = (
            "source_clause_ids", "element_ids", "profile_ids", "test_case_ids",
            "dependency_profile_ids", "parent_element_ids", "requirement_ids",
        )
        for collection in ("requirements", "elements", "profiles", "test_cases"):
            for record in catalog[collection]:
                for field in relationship_fields:
                    if field in record:
                        self.assertEqual(
                            record[field], sorted(record[field]),
                            (collection, record.get("test_id"), field),
                        )

    def test_official_cases_link_only_html_explicitly_named_operations(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        expected_by_source_section: dict[tuple[str, str], set[str]] = {}

        cases_source = "KMIPKIT-SRC-testcases"
        profiles_source = "KMIPKIT-SRC-profiles"
        def link(
            source_id: str,
            sections: tuple[str, ...] | tuple[int, ...] | range,
            *element_ids: str,
        ) -> None:
            for section in sections:
                normalized = (
                    f"2.{section}"
                    if source_id == cases_source and isinstance(section, int)
                    else str(section)
                )
                expected_by_source_section[(source_id, normalized)] = set(element_ids)

        def link_cases(sections: range | tuple[int, ...], *element_ids: str) -> None:
            link(cases_source, sections, *element_ids)

        sign = "KMIPKIT-ELEM-OP-C2S-SIGN"
        verify = "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY"
        link_cases(range(14, 20), "KMIPKIT-ELEM-OP-C2S-DERIVE-KEY")
        link_cases(range(21, 34), "KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN")
        link_cases((37,), sign, verify)
        link_cases(range(43, 48), "KMIPKIT-ELEM-OP-C2S-IMPORT", "KMIPKIT-ELEM-OP-C2S-EXPORT")
        link_cases(range(48, 51), "KMIPKIT-ELEM-OP-C2S-LOGIN")
        link_cases(range(62, 66), "KMIPKIT-ELEM-OP-C2S-LOCATE")
        link_cases((67,), "KMIPKIT-ELEM-OP-C2S-PING")
        link_cases(range(68, 70), "KMIPKIT-ELEM-OP-C2S-GET")
        link_cases(range(76, 88), "KMIPKIT-ELEM-OP-C2S-RE-KEY")
        link_cases(range(92, 95), "KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE")
        link_cases((99, 101), "KMIPKIT-ELEM-OP-C2S-ENCRYPT")
        link_cases((100,), "KMIPKIT-ELEM-OP-C2S-ENCRYPT", "KMIPKIT-ELEM-OP-C2S-DECRYPT")
        link_cases(range(102, 105), "KMIPKIT-ELEM-OP-C2S-HASH")
        link_cases((105,), "KMIPKIT-ELEM-OP-C2S-MAC")
        link_cases((106,), sign)
        link_cases((107,), sign, verify)

        link(
            profiles_source,
            ("5.3.3.1", "5.4.4.1", "5.5.4.1", "5.17.1"),
            "KMIPKIT-ELEM-OP-C2S-QUERY",
        )
        expected_by_source_section[(profiles_source, "5.12.6.3")] = {
            "KMIPKIT-ELEM-OP-C2S-GET", "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST",
            "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", "KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE",
        }
        link(profiles_source, ("5.17.2",), "KMIPKIT-ELEM-OP-C2S-CREATE")

        operation_ids = {row["element_id"] for row in catalog["elements"] if row["kind"] == "operation"}
        for test_case in catalog["test_cases"]:
            key = (test_case["source_id"], test_case["source_section"])
            actual = set(test_case["element_ids"]) & operation_ids
            self.assertEqual(actual, expected_by_source_section.get(key, set()), key)

    def test_official_cases_link_only_html_explicitly_named_protocol_elements(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = {row["element_id"]: row for row in catalog["elements"]}
        non_operation_ids = {
            element_id for element_id, row in elements.items() if row["kind"] != "operation"
        }
        cases = {row["test_id"]: row for row in catalog["test_cases"]}

        for test_id, expected in OFFICIAL_NON_OPERATION_TEST_ELEMENTS.items():
            with self.subTest(test_id=test_id):
                actual = set(cases[test_id]["element_ids"]) & non_operation_ids
                self.assertEqual(actual, expected)
                for element_id in expected:
                    self.assertIn(test_id, elements[element_id]["test_case_ids"])

        self.assertEqual(
            set(cases["KMIPKIT-TEST-CN01-2-42"]["element_ids"]),
            {"KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-VALUE"},
            "the source does not equate its customer-specific attribute with Vendor Attribute",
        )
        self.assertNotIn(
            "KMIPKIT-ELEM-ATTRIBUTE-VENDOR",
            cases["KMIPKIT-TEST-CN01-2-42"]["element_ids"],
        )

    def test_profile_json_template_value_is_gated_as_a_source_defect(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        discrepancy = next(
            (row for row in catalog["discrepancies"] if row["discrepancy_id"] == "KMIPKIT-DISC-038"),
            None,
        )

        self.assertIsNotNone(discrepancy)
        assert discrepancy is not None
        self.assertEqual(discrepancy["normative_status"], "source_defect")
        self.assertEqual(discrepancy["state"], "open")
        self.assertIsNone(discrepancy["decision_id"])
        self.assertEqual(
            {(row["source_id"], row["section"]) for row in discrepancy["source_refs"]},
            {
                ("KMIPKIT-SRC-profiles", "5.5.4.1"),
                ("KMIPKIT-SRC-spec", "11.34"),
            },
        )
        self.assertIn(
            "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-RESERVED-00000006",
            discrepancy["affected_element_ids"],
        )
        self.assertEqual(
            set(discrepancy["affected_profile_ids"]),
            {"KMIPKIT-PROFILE-JSON-CLIENT", "KMIPKIT-PROFILE-JSON-SERVER"},
        )

        json_sample = next(
            row for row in catalog["test_cases"] if row["test_id"] == "KMIPKIT-TEST-PROF-5-5-4-1"
        )
        self.assertIn("KMIPKIT-ELEM-TAG-420057", json_sample["element_ids"])
        self.assertNotIn(
            "KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-RESERVED-00000006",
            json_sample["element_ids"],
            "Template must not be treated as an assigned Object Type value",
        )

    def test_json_preflight_enforces_depth_record_token_and_global_member_limits(self) -> None:
        cases = (
            ("MAX_DEPTH", 1, '{"schema_version":{"nested":{"again":1}}}', "nesting depth"),
            ("MAX_RECORDS", 1, '{"schema_version":1,"sources":[1,2]}', "record limit"),
            ("MAX_TOKENS", 1, '{"schema_version":1}', "token limit"),
            ("MAX_MEMBERS", 0, '{"schema_version":1}', "member limit"),
        )
        for constant, limit, raw, expected_error in cases:
            with self.subTest(limit=constant), patch.object(catalog_validate, constant, limit):
                with self.assertRaisesRegex(CatalogValidationError, expected_error):
                    _JsonPreflight(raw).validate()

    def test_json_number_preflight_matches_without_copying_remaining_input(self) -> None:
        class NoSlice(str):
            def __getitem__(self, key: object) -> str:
                if isinstance(key, slice) and key.stop is None:
                    raise AssertionError("preflight copied the unconsumed JSON suffix")
                return super().__getitem__(key)  # type: ignore[arg-type]

        _JsonPreflight(NoSlice('{"schema_version":1}')).validate()

    def test_operation_inventory_matches_all_client_and_server_definitions(self) -> None:
        elements = [
            *(operation_element(name, section, "client_to_server") for name, section in CLIENT_OPERATIONS.items()),
            *(operation_element(name, section, "server_to_client") for name, section in SERVER_OPERATIONS.items()),
        ]
        _check_operation_inventory(elements)

        missing_operation = elements[:-1]
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(missing_operation)

    def test_operation_inventory_rejects_a_wrong_source_section(self) -> None:
        elements = [
            *(operation_element(name, section, "client_to_server") for name, section in CLIENT_OPERATIONS.items()),
            *(operation_element(name, section, "server_to_client") for name, section in SERVER_OPERATIONS.items()),
        ]
        elements[0]["source_refs"] = [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.2"}]
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(elements)

    def test_operation_inventory_requires_exact_payload_tables_and_async_responses(self) -> None:
        elements = [
            *(operation_element(name, section, "client_to_server") for name, section in CLIENT_OPERATIONS.items()),
            *(operation_element(name, section, "server_to_client") for name, section in SERVER_OPERATIONS.items()),
        ]
        _check_operation_inventory(elements)

        query_async = next(row for row in elements if row["name"] == "Query Asynchronous Requests")
        query_async["payload_tables"][-1]["caption"] = "Query Asynchronous Requests Response Payload"
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(elements)

        poll = next(row for row in elements if row["name"] == "Poll")
        poll["payload_tables"].append({"role": "response", "table_number": 277, "caption": "Poll Response Payload"})
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(elements)

    def test_item_types_managed_object_types_and_object_structures_reconcile(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        data_types = {row["name"]: row for row in elements if row.get("kind") == "data_type"}
        object_types = {row["name"]: row for row in elements if row.get("kind") == "object_type"}

        expected_data_types = {
            "Structure": "00000001", "Integer": "00000002", "Long Integer": "00000003",
            "Big Integer": "00000004", "Enumeration": "00000005", "Boolean": "00000006",
            "Text String": "00000007", "Byte String": "00000008", "Date Time": "00000009",
            "Interval": "0000000A", "Date Time Extended": "0000000B",
        }
        self.assertEqual({name: row.get("wire_value") for name, row in data_types.items()}, expected_data_types)
        self.assertEqual(len(object_types), 9)
        self.assertEqual(
            {name: row.get("wire_value") for name, row in object_types.items()},
            {
                "Certificate": "00000001", "Symmetric Key": "00000002", "Public Key": "00000003",
                "Private Key": "00000004", "Split Key": "00000005", "Secret Data": "00000007",
                "Opaque Object": "00000008", "PGP Key": "00000009", "Certificate Request": "0000000A",
            },
        )
        for kind, records in (("data_type", data_types), ("object_type", object_types)):
            expected_counts = {"data_type": 11, "object_type": 9}
            self.assertEqual(len(records), expected_counts[kind])
            for record in records.values():
                name = record["name"]
                slug = re.sub(r"[^A-Z0-9]+", "-", name.upper()).strip("-")
                prefix = {"data_type": "DATA-TYPE", "object_type": "OBJECT-TYPE"}[kind]
                self.assertEqual(record["element_id"], f"KMIPKIT-ELEM-{prefix}-{slug}")
                self.assertEqual(record["direction"], "both")
                self.assertEqual(record["scope_state"], "client_1_0")
                self.assertTrue(record["source_refs"])

    def test_all_object_structure_roots_and_members_match_pinned_tables(self) -> None:
        source_roots, source_members = _pinned_object_structures()
        self.assertEqual(len(source_roots), 23)
        self.assertEqual(len(source_members), 70)
        expected_roots = {
            (f"2.{number}", name)
            for number, name in enumerate(
                (
                    "Certificate", "Certificate Request", "Opaque Object", "PGP Key", "Private Key",
                    "Public Key", "Secret Data", "Split Key", "Symmetric Key",
                ),
                start=1,
            )
        }
        expected_roots.update(
            {
                ("3.1", "Key Block"),
                ("3.2", "Key Value"),
                ("3.3", "Key Wrapping Data"),
                ("3.3", "Encryption Key Information"),
                ("3.3", "MAC/Signature Key Information"),
                *((f"3.{number}", "Key Material") for number in range(4, 13)),
            }
        )
        self.assertEqual(set(source_roots), expected_roots)

        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        structures = [row for row in elements if row.get("kind") == "object_structure"]
        structures_by_key = {
            (row["source_refs"][0]["section"], row["name"]): row
            for row in structures
        }
        self.assertEqual(
            {
                key: (row.get("source_encoding"), row.get("source_requiredness"))
                for key, row in structures_by_key.items()
            },
            source_roots,
        )
        self.assertEqual(set(structures_by_key), expected_roots)
        for (section, name), row in structures_by_key.items():
            suffix = re.sub(r"[^A-Z0-9]+", "-", name.upper()).strip("-")
            if name == "Key Material":
                suffix = f"{section.replace('.', '-')}-{suffix}"
            self.assertEqual(row["element_id"], f"KMIPKIT-ELEM-OBJECT-STRUCTURE-{suffix}")
            self.assertEqual(row["direction"], "both")
            self.assertEqual(row["scope_state"], "client_1_0")

        structure_names_by_id = {
            row["element_id"]: row["name"]
            for row in structures
        }
        members = [
            row for row in elements
            if row.get("kind") == "structure_member"
            and row["source_refs"][0]["section"].startswith(("2.", "3."))
        ]
        actual_members = {
            (
                row["source_refs"][0]["section"],
                structure_names_by_id[row["parent_element_ids"][0]],
                row["name"],
                row["source_encoding"],
                row["source_requiredness"],
            )
            for row in members
        }
        self.assertEqual(actual_members, source_members)
        self.assertEqual(len(members), len(source_members))
        self.assertEqual(
            sum(row.get("kind") == "structure_member" for row in elements),
            241,
        )
        for row in members:
            self.assertEqual(row["direction"], "both")
            self.assertEqual(row["scope_state"], "client_1_0")

    def test_tag_inventory_reconciles_all_reserved_values_and_ranges(self) -> None:
        elements = [
            {
                "element_id": f"KMIPKIT-ELEM-TAG-{value}",
                "kind": "tag",
                "name": name,
                "wire_value": value,
                "allocation": "reserved",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
            for value, name in RESERVED_TAGS.items()
        ]
        for index in range(354):
            value = f"A{index:05X}"
            elements.append(
                {
                    "element_id": f"KMIPKIT-ELEM-TAG-{value}",
                    "kind": "tag",
                    "name": f"Test Tag {index}",
                    "wire_value": value,
                    "allocation": "assigned",
                    "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
                }
            )
        ranges = [
            {
                "range_id": f"KMIPKIT-RANGE-{index:03}",
                "value_range": value_range,
                "allocation": allocation,
                "source_order": index,
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
            for index, (allocation, value_range) in enumerate(EXPECTED_TAG_RANGES, start=1)
        ]
        _check_tag_registry(elements, ranges)

        elements[17]["name"] = "(Reserved)"
        with self.assertRaisesRegex(CatalogValidationError, "reserved tag"):
            _check_tag_registry(elements, ranges)

    def test_tag_registry_matches_every_pinned_source_value(self) -> None:
        source_singletons, source_ranges = _pinned_tag_rows()
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        tags = [row for row in catalog["elements"] if row.get("kind") == "tag"]
        actual_singletons = {(row["name"], row["wire_value"], row["allocation"]) for row in tags}
        self.assertEqual(actual_singletons, set(source_singletons))
        self.assertEqual(
            [(row["allocation"], row["value_range"]) for row in sorted(catalog["tag_ranges"], key=lambda item: item["source_order"])],
            source_ranges,
        )

        expected_digest = hashlib.sha256(
            json.dumps(sorted(source_singletons, key=lambda row: row[1]), ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        self.assertEqual(catalog_validate.TAG_REGISTRY_SHA256, expected_digest)
        catalog_validate._check_tag_registry_fingerprint(tags)

        tampered = [dict(row) for row in tags]
        assigned = next(row for row in tampered if row["allocation"] == "assigned")
        assigned["wire_value"] = "FF1234"
        assigned["element_id"] = "KMIPKIT-ELEM-TAG-FF1234"
        with self.assertRaisesRegex(CatalogValidationError, "tag registry source fingerprint"):
            catalog_validate._check_tag_registry_fingerprint(tampered)

    def test_enumerations_and_values_reconcile_with_all_pinned_section_11_tables(self) -> None:
        groups = _pinned_enumeration_groups()
        self.assertEqual(set(groups), {f"11.{number}" for number in range(1, 65)})
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        definitions = [row for row in elements if row.get("kind") == "enumeration"]
        values = [row for row in elements if row.get("kind") == "enumeration_value"]
        expected_definitions = {(name, section) for section, (name, _) in groups.items()}
        actual_definitions = {(row["name"], row["source_refs"][0]["section"]) for row in definitions}
        self.assertEqual(actual_definitions, expected_definitions)

        expected_values: set[tuple[str, str, str, str]] = set()
        for section, (name, rows) in groups.items():
            if section == "11.56":
                continue  # Table 487 is represented by the dedicated tag and tag-range records.
            definition_slug = re.sub(r"[^A-Z0-9]+", "-", name.upper()).strip("-")
            parent_id = f"KMIPKIT-ELEM-ENUMERATION-{definition_slug}"
            for value_name, wire_value in rows:
                allocation = (
                    "extension" if wire_value.startswith("8X")
                    else "reserved" if value_name == "(Reserved)" or "-" in wire_value
                    else "assigned"
                )
                expected_values.add((parent_id, value_name, wire_value, allocation))
        actual_values = {
            (row["parent_element_ids"][0], row["name"], row["wire_value"], row["allocation"])
            for row in values
            if len(row["parent_element_ids"]) == 1
        }
        self.assertEqual(actual_values, expected_values)
        self.assertEqual(len(values), len(expected_values))

        definitions_by_name = {row["name"]: row for row in definitions}
        for kind in ("data_type", "object_type", "tag"):
            for row in (item for item in elements if item.get("kind") == kind):
                parent_name = {"data_type": "Item Type", "object_type": "Object Type", "tag": "Tag"}[kind]
                self.assertIn(definitions_by_name[parent_name]["element_id"], row["parent_element_ids"])

    def test_bitmask_definitions_and_bits_reconcile_with_pinned_section_12(self) -> None:
        groups = _pinned_bitmask_groups()
        self.assertEqual(set(groups), {"12.1", "12.2", "12.3"})
        self.assertEqual(sum(len(values) for _, values in groups.values()), 44)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        definitions = [row for row in elements if row.get("kind") == "bitmask"]
        values = [row for row in elements if row.get("kind") == "bitmask_value"]
        expected_definitions = {(name, section) for section, (name, _) in groups.items()}
        actual_definitions = {(row["name"], row["source_refs"][0]["section"]) for row in definitions}
        self.assertEqual(actual_definitions, expected_definitions)

        expected_values: set[tuple[str, str, str, str]] = set()
        for section, (name, rows) in groups.items():
            definition_slug = re.sub(r"[^A-Z0-9]+", "-", name.upper()).strip("-")
            parent_id = f"KMIPKIT-ELEM-BITMASK-{definition_slug}"
            for value_name, wire_value in rows:
                allocation = (
                    "extension" if "X" in wire_value
                    else "reserved" if value_name == "(Reserved)"
                    else "assigned"
                )
                expected_values.add((parent_id, value_name, wire_value, allocation))
        actual_values = {
            (row["parent_element_ids"][0], row["name"], row["wire_value"], row["allocation"])
            for row in values
            if len(row["parent_element_ids"]) == 1
        }
        self.assertEqual(actual_values, expected_values)
        self.assertEqual(len(values), 44)

    def test_attribute_records_reconcile_with_all_pinned_section_4_headings(self) -> None:
        headings = _pinned_attribute_headings()
        self.assertEqual(set(headings), {f"4.{number}" for number in range(1, 64)})
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        attributes = [row for row in catalog["elements"] if row.get("kind") == "attribute"]
        expected = {(name, section) for section, name in headings.items()}
        actual = {(row["name"], row["source_refs"][0]["section"]) for row in attributes}
        self.assertEqual(actual, expected)
        self.assertEqual(len(attributes), 63)
        tag_ids = {row["name"]: row["element_id"] for row in catalog["elements"] if row.get("kind") == "tag"}
        for row in attributes:
            slug = re.sub(r"[^A-Z0-9]+", "-", row["name"].upper()).strip("-")
            self.assertEqual(row["element_id"], f"KMIPKIT-ELEM-ATTRIBUTE-{slug}")
            self.assertEqual(row["direction"], "both")
            self.assertEqual(row["scope_state"], "client_1_0")
            section = next(section for section, name in headings.items() if name == row["name"])
            expected_refs = [{"source_id": "KMIPKIT-SRC-spec", "section": section}]
            expected_parent_ids = []
            if row["name"] == "Usage Limits":
                expected_refs.append({"source_id": "KMIPKIT-SRC-spec", "section": "7.40"})
            if row["name"] in tag_ids:
                expected_refs.append({"source_id": "KMIPKIT-SRC-spec", "section": "11.56"})
                expected_parent_ids = [tag_ids[row["name"]]]
            expected_refs.sort(
                key=lambda reference: tuple(int(part) for part in reference["section"].split("."))
            )
            self.assertEqual(row["source_refs"], expected_refs)
            self.assertEqual(row["parent_element_ids"], expected_parent_ids)

    def test_attribute_structures_and_members_match_pinned_tables_157_to_163(self) -> None:
        source_structures, source_members = _pinned_attribute_structures()
        self.assertEqual(set(source_structures), {f"5.{number}" for number in range(1, 8)})
        self.assertEqual(len(source_members), 9)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        structures = [row for row in elements if row.get("kind") == "attribute_structure"]
        members = [
            row for row in elements
            if row.get("kind") == "structure_member" and row["source_refs"][0]["section"].startswith("5.")
        ]
        self.assertEqual(
            {(row["name"], row["source_refs"][0]["section"]) for row in structures},
            {(name, section) for section, (name, _) in source_structures.items()},
        )
        actual_members = {
            (
                row["source_refs"][0]["section"],
                row["name"],
                row["source_encoding"],
                row["source_requiredness"],
            )
            for row in members
        }
        self.assertEqual(actual_members, source_members)
        self.assertEqual(len(members), len(source_members))
        structures_by_section = {row["source_refs"][0]["section"]: row for row in structures}
        for member in members:
            section = member["source_refs"][0]["section"]
            self.assertEqual(member["parent_element_ids"], [structures_by_section[section]["element_id"]])

    def test_structure_valued_attributes_match_pinned_section_four_tables(self) -> None:
        source_structures, source_members = _pinned_attribute_value_structures()
        self.assertEqual(len(source_structures), 14)
        self.assertEqual(len(source_members), 46)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        attributes = [row for row in elements if row.get("kind") == "attribute"]
        attributes_by_section = {
            reference["section"]: row
            for row in attributes
            for reference in row["source_refs"]
            if reference["source_id"] == "KMIPKIT-SRC-spec" and reference["section"].startswith("4.")
        }
        for section, (_source_name, encoding, requiredness) in source_structures.items():
            self.assertEqual(attributes_by_section[section].get("source_encoding"), encoding)
            if requiredness is not None:
                self.assertEqual(attributes_by_section[section].get("source_requiredness"), requiredness)
        members = [row for row in elements if row.get("kind") == "structure_member"]
        actual_members = {
            (
                row["source_refs"][0]["section"],
                row["name"],
                row["source_encoding"],
                row.get("source_requiredness"),
            )
            for row in members
            if row["source_refs"][0]["section"].startswith("4.")
        }
        self.assertEqual(actual_members, source_members)
        for member in members:
            section = member["source_refs"][0]["section"]
            if section in source_structures:
                self.assertEqual(member["parent_element_ids"], [attributes_by_section[section]["element_id"]])

    def test_operation_structures_match_pinned_tables_352_to_393(self) -> None:
        source_structures, source_members = _pinned_operation_structures()
        self.assertEqual(set(source_structures), {f"7.{number}" for number in range(1, 42)})
        self.assertEqual(len(source_members), 94)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        structures = [row for row in elements if row.get("kind") == "operation_structure"]
        self.assertEqual(len(structures), 41)
        actual_structures = {
            (
                row["source_refs"][0]["section"], row["name"], row["source_encoding"],
                row.get("source_requiredness"),
            )
            for row in structures
        }
        self.assertEqual(
            actual_structures,
            {(section, name, encoding, requiredness) for section, (name, encoding, requiredness) in source_structures.items()},
        )
        structure_ids = {row["source_refs"][0]["section"]: row["element_id"] for row in structures}
        members = [
            row for row in elements
            if row.get("kind") == "structure_member" and row["source_refs"][0]["section"].startswith("7.")
        ]
        actual_members = {
            (
                row["source_refs"][0]["section"], row["name"], row["source_encoding"],
                row.get("source_requiredness"),
            )
            for row in members
        }
        self.assertEqual(actual_members, source_members)
        self.assertEqual(len(members), len(source_members))
        for member in members:
            section = member["source_refs"][0]["section"]
            self.assertEqual(member["parent_element_ids"], [structure_ids[section]])

    def test_attribute_encodings_match_pinned_value_tables(self) -> None:
        source_encodings = _pinned_attribute_encodings()
        self.assertEqual(len(source_encodings), 62)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        attributes = [row for row in catalog["elements"] if row.get("kind") == "attribute"]
        attributes_by_section = {
            reference["section"]: row
            for row in attributes
            for reference in row["source_refs"]
            if reference["source_id"] == "KMIPKIT-SRC-spec" and reference["section"].startswith("4.")
        }
        for section, (source_name, encoding, requiredness, source_section) in source_encodings.items():
            element = attributes_by_section[section]
            self.assertEqual(element.get("source_name"), source_name)
            self.assertEqual(element.get("source_encoding"), encoding)
            if requiredness is not None:
                self.assertEqual(element.get("source_requiredness"), requiredness)
            self.assertIn(
                {"source_id": "KMIPKIT-SRC-spec", "section": source_section},
                element["source_refs"],
            )
        self.assertNotIn("source_encoding", attributes_by_section["4.6"])

    def test_options_and_result_values_have_explicit_inventory_records(self) -> None:
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        expected = {
            ("option", "Asynchronous Indicator", ("9.2", "11.3")),
            ("option", "Batch Error Continuation Option", ("9.6", "11.5")),
            ("option", "Batch Order Option", ("9.8",)),
            ("result", "Cancellation Result", ("11.7",)),
            ("result", "Result Message", ("9.17",)),
            ("result", "Result Reason", ("9.18", "11.46")),
            ("result", "Result Status", ("9.19", "11.47")),
        }
        actual = {
            (
                row["kind"], row["name"],
                tuple(reference["section"] for reference in row["source_refs"]),
            )
            for row in elements
            if row.get("kind") in {"option", "result"}
        }
        self.assertEqual(actual, expected)

    def test_message_fields_match_all_rows_in_tables_394_to_399(self) -> None:
        source_rows = _pinned_message_structure_rows()
        self.assertEqual(len(source_rows), 43)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        messages = [
            row for row in elements
            if row.get("kind") == "message_field"
            and any(reference["section"].startswith("8.") for reference in row["source_refs"])
        ]
        actual_rows = {
            (
                reference["section"], row["name"], row.get("source_encoding"),
                row.get("source_requiredness"), row.get("source_comment"),
            )
            for row in messages
            for reference in row["source_refs"]
            if reference["section"].startswith("8.")
        }
        self.assertEqual(actual_rows, source_rows)
        self.assertEqual(len(messages), len(source_rows))

    def test_message_field_types_match_tables_400_to_409_and_417_to_426(self) -> None:
        source_rows = _pinned_message_field_types()
        self.assertEqual(len(source_rows), 28)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        messages = [
            row for row in catalog["elements"]
            if row.get("kind") == "message_field"
            and any(reference["section"].startswith("9.") for reference in row["source_refs"])
        ]
        actual_rows = {
            (reference["section"], row["name"], row["source_encoding"], row.get("source_requiredness"))
            for row in messages
            for reference in row["source_refs"]
            if reference["section"].startswith("9.")
        }
        self.assertEqual(actual_rows, source_rows)
        self.assertEqual(len(messages), len(source_rows))

    def test_credential_forms_and_fields_match_tables_410_to_416(self) -> None:
        source_roots, source_members = _pinned_credential_forms()
        self.assertEqual(len(source_roots), 7)
        self.assertEqual(len(source_members), 22)
        catalog = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        elements = catalog["elements"]
        credentials = [row for row in elements if row.get("kind") == "credential"]
        self.assertEqual(
            {row["name"]: (row["source_encoding"], row["source_requiredness"]) for row in credentials},
            source_roots,
        )
        credential_names = {row["element_id"]: row["name"] for row in credentials}
        members = [
            row for row in elements
            if row.get("kind") == "structure_member"
            and row["source_refs"][0]["section"] == "9.11"
        ]
        actual_members = {
            (
                credential_names[row["parent_element_ids"][0]], row["name"],
                row["source_encoding"], row["source_requiredness"],
            )
            for row in members
        }
        self.assertEqual(actual_members, source_members)
        self.assertEqual(len(members), len(source_members))

    def test_accepts_source_encodings_on_option_and_result_records(self) -> None:
        document = minimal_catalog()
        common = {
            "direction": "both",
            "scope_state": "client_1_0",
            "parent_element_ids": [],
            "requirement_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": None,
            "implementation_refs": [],
            "verification_refs": [],
        }
        document["elements"] = [
            {
                **common,
                "element_id": "KMIPKIT-ELEM-OPTION-TEST",
                "kind": "option",
                "name": "Test Option",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "9.2"}],
                "source_encoding": "Enumeration",
            },
            {
                **common,
                "element_id": "KMIPKIT-ELEM-RESULT-TEST",
                "kind": "result",
                "name": "Test Result",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "9.17"}],
                "source_encoding": "Text String",
            },
        ]
        self.assertEqual(validate(document)["record_count"], 6)

    def test_accepts_raw_message_field_comments_and_requiredness(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-TEST",
                "kind": "message_field",
                "name": "Test Message Field",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.2"}],
                "direction": "both",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "source_requiredness": "No, MAY be repeated",
                "source_comment": "If omitted, the default applies.",
            },
        ]
        self.assertEqual(validate(document)["record_count"], 5)

    def test_accepts_source_metadata_on_credential_roots(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-CREDENTIAL-TEST",
                "kind": "credential",
                "name": "Test Credential",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "9.11"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "source_encoding": "Structure",
                "source_requiredness": "",
            },
        ]
        self.assertEqual(validate(document)["record_count"], 5)

    def test_accepts_exact_pinned_source_manifest_and_empty_record_collections(self) -> None:
        result = validate(minimal_catalog())
        self.assertEqual(result["source_count"], 4)

    def test_accepts_an_enumeration_extension_marker(self) -> None:
        document = minimal_catalog()
        definition_id = "KMIPKIT-ELEM-ENUMERATION-TEST"
        common = {
            "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.1"}],
            "direction": "both",
            "scope_state": "client_1_0",
            "requirement_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": None,
            "implementation_refs": [],
            "verification_refs": [],
        }
        document["elements"] = [
            {
                **common,
                "element_id": definition_id,
                "kind": "enumeration",
                "name": "Test Enumeration",
                "parent_element_ids": [],
            },
            {
                **common,
                "element_id": "KMIPKIT-ELEM-ENUM-VALUE-TEST-EXTENSIONS-8XXXXXXX",
                "kind": "enumeration_value",
                "name": "Extensions",
                "wire_value": "8XXXXXXX",
                "allocation": "extension",
                "parent_element_ids": [definition_id],
            },
        ]
        self.assertEqual(validate(document)["record_count"], 6)

    def test_accepts_structure_member_source_encoding_and_requiredness(self) -> None:
        document = minimal_catalog()
        structure_id = "KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-TEST"
        common = {
            "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "5.1"}],
            "direction": "both",
            "scope_state": "client_1_0",
            "requirement_ids": [],
            "profile_ids": [],
            "test_case_ids": [],
            "feature_spec": None,
            "implementation_refs": [],
            "verification_refs": [],
        }
        document["elements"] = [
            {
                **common,
                "element_id": structure_id,
                "kind": "attribute_structure",
                "name": "Test Structure",
                "parent_element_ids": [],
            },
            {
                **common,
                "element_id": "KMIPKIT-ELEM-STRUCTURE-MEMBER-TEST-FIELD",
                "kind": "structure_member",
                "name": "Test Field",
                "source_encoding": "Text String",
                "source_requiredness": "Yes",
                "parent_element_ids": [structure_id],
            },
        ]
        self.assertEqual(validate(document)["record_count"], 6)

    def test_accepts_source_encoding_and_blank_requiredness_on_attribute_roots(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-ATTRIBUTE-TEST",
                "kind": "attribute",
                "name": "Test Attribute",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "4.2"}],
                "direction": "both",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "source_encoding": "Structure",
                "source_requiredness": "",
            },
        ]
        self.assertEqual(validate(document)["record_count"], 5)

    def test_accepts_a_source_name_on_attribute_records(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-ATTRIBUTE-TEST",
                "kind": "attribute",
                "name": "Canonical Attribute",
                "source_name": "Source Attribute Label",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "4.3"}],
                "direction": "both",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
            },
        ]
        self.assertEqual(validate(document)["record_count"], 5)

    def test_accepts_source_metadata_on_operation_structure_roots(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-OPERATION-STRUCTURE-TEST",
                "kind": "operation_structure",
                "name": "Test Structure",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "7.1"}],
                "direction": "both",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "source_encoding": "Structure",
                "source_requiredness": "",
            },
        ]
        self.assertEqual(validate(document)["record_count"], 5)

    def test_complete_validation_rejects_omitted_operation_and_element_records(self) -> None:
        raw = json.dumps(minimal_catalog()).encode("utf-8")
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            validate_catalog(raw, ROOT, require_complete=True)

    def test_checked_in_catalog_reconciles_profile_and_official_test_evidence(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        raw = catalog_path.read_bytes()

        validated = validate_catalog(raw, ROOT, require_complete=True)

        self.assertGreater(validated["record_count"], 0)
        document = json.loads(raw)
        profiles = document["profiles"]
        self.assertEqual(len(profiles), 35)
        self.assertTrue({"client", "server"}.issubset({profile["role"] for profile in profiles}))
        self.assertTrue(all(profile["claim_state"] == "not_claimed" for profile in profiles))
        self.assertTrue(all(profile["source_refs"] for profile in profiles))
        self.assertTrue(all(profile["source_clause_ids"] for profile in profiles))
        self.assertTrue(all(profile["requirement_ids"] for profile in profiles))
        self.assertTrue(all(profile["test_case_ids"] for profile in profiles))
        self.assertTrue(all(isinstance(profile["transport_requirements"], list) for profile in profiles))
        self.assertTrue(all(isinstance(profile["encoding_requirements"], list) for profile in profiles))
        self.assertTrue(all(profile["applicability"] in {
            "client_1_0", "client_1_1", "server_only", "conditional", "out_of_scope"
        } for profile in profiles))
        conformance_sections = {
            reference["section"]
            for profile in profiles
            for reference in profile["source_refs"]
            if reference["source_id"] == "KMIPKIT-SRC-profiles"
            and reference["section"].startswith("6.")
        }
        self.assertEqual(conformance_sections, {f"6.{section}" for section in range(1, 36)})

    def test_checked_in_catalog_preserves_pinned_source_discrepancies(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        document = json.loads(catalog_path.read_bytes())
        discrepancies = document["discrepancies"]
        cited_sections = {
            (reference["source_id"], reference["section"])
            for discrepancy in discrepancies
            for reference in discrepancy["source_refs"]
        }
        expected_sections = {
            ("KMIPKIT-SRC-spec", "11.5"),
            ("KMIPKIT-SRC-profiles", "5.3.1"),
            ("KMIPKIT-SRC-profiles", "5.9.7.13"),
            ("KMIPKIT-SRC-profiles", "5.9.7.14"),
            ("KMIPKIT-SRC-profiles", "6.1"),
            ("KMIPKIT-SRC-profiles", "6.2"),
            ("KMIPKIT-SRC-profiles", "6.3"),
            ("KMIPKIT-SRC-profiles", "6.9"),
            ("KMIPKIT-SRC-testcases", "2.48"),
            ("KMIPKIT-SRC-testcases", "2.38"),
            ("KMIPKIT-SRC-testcases", "2.60"),
            ("KMIPKIT-SRC-testcases", "2.68"),
            ("KMIPKIT-SRC-testcases", "2.69"),
            ("KMIPKIT-SRC-testcases", "2.90"),
            ("KMIPKIT-SRC-testcases", "2.92"),
            ("KMIPKIT-SRC-testcases", "2.93"),
            ("KMIPKIT-SRC-testcases", "2.94"),
            ("KMIPKIT-SRC-testcases", "2.97"),
            ("KMIPKIT-SRC-profiles", "5.17.1"),
        }
        expected_sections.update(
            (clause["source_id"], clause["section"])
            for clause in document["source_clauses"]
            if clause["disposition"] == "source_discrepancy"
        )

        self.assertTrue(expected_sections.issubset(cited_sections))
        self.assertTrue(discrepancies)
        self.assertTrue(all(row["state"] == "open" for row in discrepancies))
        self.assertTrue(all(row["decision_id"] is None for row in discrepancies))

    def test_checked_in_catalog_separates_unknown_vendor_and_extension_policies(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        document = json.loads(catalog_path.read_bytes())
        policies = {row["policy_id"]: row for row in document["policies"]}
        expected_ids = {
            "KMIPKIT-POLICY-UNKNOWN-FUTURE-VALUE-PRESERVATION",
            "KMIPKIT-POLICY-VENDOR-VALUE-PRESERVATION",
            "KMIPKIT-POLICY-EXTENSION-PRESERVATION",
        }

        self.assertTrue(expected_ids.issubset(policies))
        for policy_id in expected_ids:
            policy = policies[policy_id]
            self.assertEqual(policy["provenance"], "AGENTS.md")
            self.assertEqual(policy["provenance_ref"], {
                "path": "AGENTS.md",
                "heading": "9. Public API and compatibility",
            })
        summaries = " ".join(policies[policy_id]["summary"].casefold() for policy_id in expected_ids)
        self.assertIn("unknown", summaries)
        self.assertIn("vendor", summaries)
        self.assertIn("extension", summaries)

    def test_rejects_unknown_top_level_fields(self) -> None:
        document = minimal_catalog()
        document["unexpected"] = []
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_source_checksum_mismatch(self) -> None:
        document = minimal_catalog()
        document["sources"][1]["sha256"] = "0" * 64
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_duplicate_json_keys(self) -> None:
        raw = b'{"schema_version":1,"schema_version":1}'
        with self.assertRaises(CatalogValidationError):
            validate_catalog(raw, ROOT)

    def test_duplicate_key_errors_do_not_echo_untrusted_control_text(self) -> None:
        raw = b'{"schema_version":{"attacker\\u001b\\n":1,"attacker\\u001b\\n":2}}'
        with self.assertRaisesRegex(CatalogValidationError, "duplicate JSON object keys") as raised:
            validate_catalog(raw, ROOT)
        self.assertNotIn("attacker", str(raised.exception))
        self.assertNotIn("\x1b", str(raised.exception))

    def test_preflight_rejects_unknown_top_level_key_before_object_decode(self) -> None:
        raw = b'{"unexpected":{"nested":[1,2,3]}}'
        with patch("tools.normative_catalog.validate.json.loads", wraps=json.loads) as decoder:
            with self.assertRaisesRegex(CatalogValidationError, "unknown top-level"):
                validate_catalog(raw, ROOT)
        self.assertNotIn(raw.decode("utf-8"), [call.args[0] for call in decoder.call_args_list])

    def test_preflight_rejects_duplicate_keys_before_object_decode(self) -> None:
        raw = b'{"schema_version":1,"schema_version":1}'
        with patch("tools.normative_catalog.validate.json.loads", wraps=json.loads) as decoder:
            with self.assertRaisesRegex(CatalogValidationError, "duplicate"):
                validate_catalog(raw, ROOT)
        self.assertNotIn(raw.decode("utf-8"), [call.args[0] for call in decoder.call_args_list])

    def test_preflight_rejects_oversized_string_before_object_decode(self) -> None:
        raw = b'{"schema_version":"' + b'a' * 65_537 + b'"}'
        with patch("tools.normative_catalog.validate.json.loads", wraps=json.loads) as decoder:
            with self.assertRaisesRegex(CatalogValidationError, "string"):
                validate_catalog(raw, ROOT)
        self.assertNotIn(raw.decode("utf-8"), [call.args[0] for call in decoder.call_args_list])
        self.assertFalse(any(len(call.args[0]) > 65_536 for call in decoder.call_args_list))

    def test_rejects_invalid_utf8_and_unpaired_escaped_surrogate(self) -> None:
        for raw in (b"\xff", b'{"schema_version":"\\ud800"}'):
            with self.subTest(raw=raw), self.assertRaises(CatalogValidationError):
                validate_catalog(raw, ROOT)

    def test_accepts_a_well_formed_escaped_surrogate_pair(self) -> None:
        from tools.normative_catalog.validate import _reject_surrogates

        _reject_surrogates("\ud83d\ude00")

    def test_rejects_excessive_input_before_object_construction(self) -> None:
        with self.assertRaises(CatalogValidationError):
            validate_catalog(b" " * (16 * 1024 * 1024 + 1), ROOT)

    def test_rejects_unanchored_record_identifiers(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "prefix-KMIPKIT-ELEM-OP-CREATE",
                "kind": "operation",
                "name": "Create",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.1"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_unknown_protocol_element_fields(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-OP-CREATE",
                "kind": "operation",
                "name": "Create",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.1"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "unexpected": "value",
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_unresolved_protocol_element_relationships(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-OP-CREATE",
                "kind": "operation",
                "name": "Create",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.1"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": ["KMIPKIT-PROFILE-MISSING"],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_unsafe_fixture_paths_without_opening_them(self) -> None:
        for unsafe_path in (
            "C:/secret.xml",
            "/secret.xml",
            "../outside.xml",
            "specification/oasis/kmip-2.1/../../outside.xml",
            "specification/oasis/kmip-2.1/fixtures\\secret.xml",
            "\\\\server\\share\\fixture.xml",
        ):
            document = minimal_catalog()
            document["test_cases"] = [test_case(fixture_path=unsafe_path, fixture_availability="available")]
            with self.subTest(path=unsafe_path), self.assertRaises(CatalogValidationError):
                validate(document)

    def test_rejects_fixture_claimed_available_when_git_tree_has_no_file(self) -> None:
        document = minimal_catalog()
        document["test_cases"] = [
            test_case(
                fixture_path="specification/oasis/kmip-2.1/fixtures/TC-001.xml",
                fixture_availability="available",
            )
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_command_line_validator_reports_catalog_counts(self) -> None:
        script = ROOT / "tools" / "normative_catalog" / "validate.py"
        result = subprocess.run(
            [sys.executable, str(script), "--repo-root", str(ROOT), "--structural-only"],
            cwd=ROOT,
            capture_output=True,
            check=False,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("sources=4", result.stdout)

    def test_requires_every_top_level_record_collection(self) -> None:
        document = minimal_catalog()
        del document["policies"]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_clause_with_unknown_disposition_or_no_review_rationale(self) -> None:
        document = minimal_catalog()
        document["source_clauses"] = [
            {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-1.1-001",
                "source_id": "KMIPKIT-SRC-spec",
                "section": "1.1",
                "locator": {"ordinal": 1, "block_kind": "paragraph"},
                "source_keywords": ["MAY"],
                "disposition": "maybe",
                "requirement_ids": [],
                "exclusion_rationale": None,
                "role": "not_applicable",
                "direction": "not_applicable",
                "scope_state": "out_of_scope",
                "condition": None,
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_source_clause_can_preserve_keywords_with_different_strengths(self) -> None:
        document = minimal_catalog()
        document["source_clauses"] = [
            {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-2.8-001",
                "source_id": "KMIPKIT-SRC-spec",
                "section": "2.8",
                "locator": {"ordinal": 1, "block_kind": "table_row"},
                "source_keywords": ["MAY", "SHALL"],
                "disposition": "informative_context",
                "requirement_ids": [],
                "exclusion_rationale": "The containing source row has separately classified obligations.",
                "role": "not_applicable",
                "direction": "not_applicable",
                "scope_state": "out_of_scope",
                "condition": None,
            }
        ]

        self.assertEqual(validate(document)["record_count"], 5)

    def test_source_clause_requires_valid_role_direction_scope_and_condition(self) -> None:
        document = deviation_catalog(decision_record())
        clause = document["source_clauses"][0]
        clause.update({
            "role": "client",
            "direction": "client_to_server",
            "scope_state": "invalid",
            "condition": None,
        })

        with self.assertRaisesRegex(CatalogValidationError, "source clause scope"):
            validate(document)

    def test_accepts_stable_subrecords_for_multiple_obligations_in_one_clause(self) -> None:
        document = deviation_catalog(decision_record())
        base_id = "KMIPKIT-REQ-SPEC-8.1-001"
        subrecord_id = f"{base_id}-002"
        document["requirements"][0]["requirement_id"] = subrecord_id
        document["source_clauses"][0]["requirement_ids"] = [subrecord_id]
        document["requirements"][0]["decision_id"] = None
        document["requirements"][0]["status"] = "unassigned"
        document["decisions"] = []

        self.assertGreater(validate(document)["record_count"], 0)

    def test_source_clause_can_summarize_separable_mixed_scope_requirements(self) -> None:
        document = deviation_catalog(decision_record())
        clause = document["source_clauses"][0]
        first = document["requirements"][0]
        second = dict(first)
        second["requirement_id"] = "KMIPKIT-REQ-SPEC-8.1-001-002"
        second["source_keyword"] = "MAY"
        second["normative_strength"] = "permission_or_optional"
        second["scope_state"] = "out_of_scope"
        second["condition"] = "When XML or JSON encoding is used."
        second["status"] = "unassigned"
        second["decision_id"] = None
        clause["source_keywords"] = ["SHOULD", "MAY"]
        clause["requirement_ids"] = [first["requirement_id"], second["requirement_id"]]
        clause["scope_state"] = "mixed"
        first["status"] = "unassigned"
        first["decision_id"] = None
        document["requirements"].append(second)
        document["decisions"] = []

        self.assertGreater(validate(document)["record_count"], 0)

    def test_source_clause_scope_must_match_linked_requirement_scopes(self) -> None:
        document = deviation_catalog(decision_record())
        clause = document["source_clauses"][0]
        first = document["requirements"][0]
        second = dict(first)
        second["requirement_id"] = "KMIPKIT-REQ-SPEC-8.1-001-002"
        second["source_keyword"] = "MAY"
        second["normative_strength"] = "permission_or_optional"
        second["scope_state"] = "out_of_scope"
        second["condition"] = "When XML or JSON encoding is used."
        second["status"] = "unassigned"
        second["decision_id"] = None
        clause["source_keywords"] = ["SHOULD", "MAY"]
        clause["requirement_ids"] = [first["requirement_id"], second["requirement_id"]]
        clause["scope_state"] = "client_1_0"
        first["status"] = "unassigned"
        first["decision_id"] = None
        document["requirements"].append(second)
        document["decisions"] = []

        with self.assertRaisesRegex(CatalogValidationError, "source clause scope"):
            validate(document)

    def test_source_discrepancy_can_preserve_unresolved_scope(self) -> None:
        document = deviation_catalog(decision_record())
        clause = document["source_clauses"][0]
        clause["disposition"] = "source_discrepancy"
        clause["requirement_ids"] = []
        clause["exclusion_rationale"] = "The source does not identify the constrained actor or scope."
        clause["role"] = "unclear"
        clause["direction"] = "unclear"
        clause["scope_state"] = "unclear"
        document["requirements"] = []
        document["decisions"] = []

        self.assertGreater(validate(document)["record_count"], 0)

    def test_requirement_keyword_must_appear_in_a_linked_source_clause(self) -> None:
        document = deviation_catalog(decision_record())
        requirement = document["requirements"][0]
        requirement["source_keyword"] = "MUST"
        requirement["normative_strength"] = "mandatory"
        requirement["status"] = "unassigned"
        requirement["decision_id"] = None

        with self.assertRaisesRegex(CatalogValidationError, "keyword.*linked source clause"):
            validate(document)

    def test_profile_conditional_clause_can_link_to_its_requirement(self) -> None:
        document = deviation_catalog(decision_record())
        document["source_clauses"][0]["disposition"] = "profile_conditional"
        document["source_clauses"][0]["scope_state"] = "profile_conditional"
        document["requirements"][0]["scope_state"] = "profile_conditional"
        document["requirements"][0]["status"] = "unassigned"
        document["requirements"][0]["decision_id"] = None
        document["decisions"] = []

        self.assertGreater(validate(document)["record_count"], 0)

    def test_source_discrepancy_can_link_to_a_separable_requirement(self) -> None:
        document = deviation_catalog(decision_record())
        document["source_clauses"][0]["disposition"] = "source_discrepancy"
        document["requirements"][0]["status"] = "unassigned"
        document["requirements"][0]["decision_id"] = None
        document["decisions"] = []

        self.assertGreater(validate(document)["record_count"], 0)

    def test_rejects_tag_range_using_singleton_tag_allocation(self) -> None:
        document = minimal_catalog()
        document["tag_ranges"] = [
            {
                "range_id": "KMIPKIT-RANGE-001",
                "value_range": "0x420000",
                "allocation": "assigned",
                "source_order": 1,
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_project_policy_without_valid_provenance(self) -> None:
        document = minimal_catalog()
        document["policies"] = [
            {
                "policy_id": "KMIPKIT-POLICY-UNKNOWN-VALUES",
                "summary": "Preserve unknown values.",
                "provenance": "a random note",
                "affected_element_kinds": ["enumeration_value"],
                "requirement_ids": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_profile_with_unknown_claim_state(self) -> None:
        document = minimal_catalog()
        document["profiles"] = [profile_record(claim_state="certified")]
        with self.assertRaisesRegex(CatalogValidationError, "profile"):
            validate(document)

    def test_profile_enum_with_non_string_json_type_fails_as_validation_error(self) -> None:
        document = minimal_catalog()
        document["profiles"] = [profile_record(role=[])]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_open_discrepancy_cannot_select_a_decision(self) -> None:
        document = minimal_catalog()
        document["decisions"] = [decision_record()]
        document["discrepancies"] = [discrepancy_record(decision_id="KMIPKIT-DEC-001")]
        with self.assertRaisesRegex(CatalogValidationError, "open discrepancy"):
            validate(document)

    def test_recommendation_deviation_requires_accepted_decision_evidence(self) -> None:
        document = deviation_catalog(
            decision_record(status="proposed", approver="", approval_evidence="")
        )
        with self.assertRaisesRegex(CatalogValidationError, "accepted decision"):
            validate(document)

    def test_deviation_decision_must_name_the_affected_requirement(self) -> None:
        with self.assertRaisesRegex(CatalogValidationError, "decision.*requirement"):
            validate(deviation_catalog(decision_record()))

    def test_resolved_discrepancy_decision_must_name_the_discrepancy(self) -> None:
        document = minimal_catalog()
        document["decisions"] = [decision_record()]
        document["discrepancies"] = [
            discrepancy_record(state="resolved_by_approved_decision", decision_id="KMIPKIT-DEC-001")
        ]
        with self.assertRaisesRegex(CatalogValidationError, "decision.*discrepancy"):
            validate(document)

    def test_profile_cannot_claim_complete_evidence_without_resolved_links(self) -> None:
        document = minimal_catalog()
        document["profiles"] = [profile_record(claim_state="evidence_complete")]
        with self.assertRaisesRegex(CatalogValidationError, "profile.*evidence"):
            validate(document)

    def test_discrepancy_authority_must_derive_from_cited_source(self) -> None:
        document = minimal_catalog()
        document["discrepancies"] = [
            discrepancy_record(
                source_refs=[{"source_id": "KMIPKIT-SRC-usage-guide", "section": "4.1"}],
                source_authority="primary_normative",
            )
        ]
        with self.assertRaisesRegex(CatalogValidationError, "authority does not match"):
            validate(document)

    def test_erratum_resolution_requires_pinned_erratum_source(self) -> None:
        document = minimal_catalog()
        document["discrepancies"] = [discrepancy_record(state="resolved_by_erratum")]
        with self.assertRaisesRegex(CatalogValidationError, "erratum.*pinned"):
            validate(document)

    def test_policy_provenance_requires_a_verifiable_locator(self) -> None:
        document = minimal_catalog()
        document["policies"] = [
            {
                "policy_id": "KMIPKIT-POLICY-UNKNOWN-VALUES",
                "summary": "Preserve unknown values.",
                "provenance": "AGENTS.md",
                "affected_element_kinds": ["enumeration_value"],
                "requirement_ids": [],
            }
        ]
        with self.assertRaisesRegex(CatalogValidationError, "project policy"):
            validate(document)

    def test_policy_provenance_accepts_an_existing_exact_heading(self) -> None:
        document = minimal_catalog()
        document["policies"] = [
            {
                "policy_id": "KMIPKIT-POLICY-UNKNOWN-VALUES",
                "summary": "Preserve unknown values.",
                "provenance": "AGENTS.md",
                "provenance_ref": {"path": "AGENTS.md", "heading": "9. Public API and compatibility"},
                "affected_element_kinds": ["enumeration_value"],
                "requirement_ids": [],
            }
        ]
        self.assertEqual(validate(document)["source_count"], 4)

    def test_rejects_requirement_with_unresolved_source_references(self) -> None:
        document = minimal_catalog()
        document["requirements"] = [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-404"],
                "source_refs": [{"source_id": "KMIPKIT-SRC-missing", "section": "8.1"}],
                "source_keyword": "MUST",
                "normative_strength": "mandatory",
                "subject": "client",
                "summary": "A client obligation.",
                "role": "client",
                "direction": "client_to_server",
                "condition": None,
                "scope_state": "client_1_0",
                "element_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "negative_verification_required": False,
                "decision_id": None,
                "status": "unassigned",
                "review_note": None,
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_requires_prohibited_requirements_to_have_negative_verification(self) -> None:
        document = minimal_catalog()
        document["source_clauses"] = [
            {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
                "source_id": "KMIPKIT-SRC-spec",
                "section": "8.1",
                "locator": {"ordinal": 1, "block_kind": "paragraph"},
                "source_keywords": ["MUST NOT"],
                "disposition": "requirement",
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "exclusion_rationale": None,
                "role": "client",
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "condition": None,
            }
        ]
        document["requirements"] = [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_keyword": "MUST NOT",
                "normative_strength": "prohibited",
                "subject": "client",
                "summary": "A prohibited behavior.",
                "role": "client",
                "direction": "client_to_server",
                "condition": None,
                "scope_state": "client_1_0",
                "element_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "negative_verification_required": False,
                "decision_id": None,
                "status": "unassigned",
                "review_note": None,
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)


if __name__ == "__main__":
    unittest.main()
