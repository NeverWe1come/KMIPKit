"""Read the pinned KMIP source rules used by attribute policy validation."""

from __future__ import annotations

import hashlib
import re
from functools import lru_cache
from html.parser import HTMLParser
from typing import Any


SPEC_SHA256 = "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf"
SPEC_SOURCE_ID = "KMIPKIT-SRC-spec"


class _SourceParser(HTMLParser):
    """Extract section headings, captions, tables, and body paragraphs."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.tables: list[dict[str, Any]] = []
        self.paragraphs: list[dict[str, str]] = []
        self.headings: list[str] = []
        self._heading = ""
        self._heading_tag: str | None = None
        self._heading_buffer: list[str] | None = None
        self._sup_depth = 0
        self._table: dict[str, Any] | None = None
        self._row: list[str] | None = None
        self._cell: list[str] | None = None
        self._caption: list[str] | None = None
        self._paragraph: list[str] | None = None
        self._paragraph_class: str | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag in {"h1", "h2", "h3", "h4"}:
            self._heading_tag = tag
            self._heading_buffer = []
        elif tag == "table":
            self._table = {"heading": self._heading, "rows": [], "captions": []}
            self.tables.append(self._table)
        elif tag == "tr" and self._table is not None:
            self._row = []
            self._table["rows"].append(self._row)
        elif tag in {"td", "th"} and self._row is not None:
            self._cell = []
        elif tag == "p" and attributes.get("class") == "MsoCaption":
            self._caption = []
        elif tag == "p" and attributes.get("class") in {"MsoBodyText", "MsoNormal"}:
            self._paragraph = []
            self._paragraph_class = attributes["class"]
        elif tag == "sup":
            self._sup_depth += 1

    def handle_data(self, data: str) -> None:
        if self._sup_depth:
            data = data.translate(str.maketrans("0123456789+-=()", "⁰¹²³⁴⁵⁶⁷⁸⁹⁺⁻⁼⁽⁾"))
        if self._cell is not None:
            self._cell.append(data)
        if self._caption is not None:
            self._caption.append(data)
        if self._paragraph is not None:
            self._paragraph.append(data)
        if self._heading_buffer is not None:
            self._heading_buffer.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag in {"td", "th"} and self._cell is not None:
            if self._row is not None:
                self._row.append(_normalized_text(self._cell))
            self._cell = None
        elif tag == "tr":
            self._row = None
        elif tag == "table":
            self._table = None
        elif tag == "p" and self._caption is not None:
            if self.tables:
                caption = _normalized_text(self._caption)
                self.tables[-1]["captions"].append(caption)
                self.tables[-1].setdefault("caption", caption)
            self._caption = None
        elif tag == "p" and self._paragraph is not None:
            self.paragraphs.append(
                {
                    "heading": self._heading,
                    "class": self._paragraph_class or "",
                    "text": _normalized_text(self._paragraph),
                }
            )
            self._paragraph = None
            self._paragraph_class = None
        elif tag == "sup" and self._sup_depth:
            self._sup_depth -= 1
        elif tag == self._heading_tag and self._heading_buffer is not None:
            self._heading = _normalized_text(self._heading_buffer)
            self.headings.append(self._heading)
            self._heading_tag = None
            self._heading_buffer = None


def _normalized_text(parts: list[str]) -> str:
    return " ".join("".join(parts).split())


def _section_paragraphs(parser: _SourceParser, section: str) -> list[str]:
    return [
        paragraph["text"]
        for paragraph in parser.paragraphs
        if re.match(rf"^{re.escape(section)}\s", paragraph["heading"])
    ]


def _source_sentence(parser: _SourceParser, section: str, marker: str) -> str:
    for paragraph in _section_paragraphs(parser, section):
        for sentence in paragraph.split(". "):
            if marker in sentence:
                return sentence if sentence.endswith(".") else f"{sentence}."
    raise ValueError(f"pinned KMIP §{section} has no sentence containing the expected marker")


def _attribute_rule_tables(parser: _SourceParser) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    for table in parser.tables:
        heading = table.get("heading")
        rows = table.get("rows")
        captions = table.get("captions")
        if not isinstance(heading, str) or not isinstance(rows, list) or not isinstance(captions, list):
            continue
        caption = next((item for item in captions if isinstance(item, str) and "Rules" in item), None)
        if caption is None:
            continue
        heading_match = re.match(r"^(4\.[0-9]+)\s+", heading)
        table_match = re.match(r"^(Table\s+[0-9]+)", caption)
        if heading_match is None or table_match is None:
            continue
        section = heading_match.group(1)
        if section == "4.60":
            continue
        cells = {row[0].casefold(): row[1] for row in rows if isinstance(row, list) and len(row) == 2}
        required_cells = {
            "shall always have a value",
            "initially set by",
            "modifiable by client",
            "deletable by client",
        }
        if not required_cells.issubset(cells):
            raise ValueError(f"pinned attribute policy table is incomplete: {caption}")
        always_match = re.match(r"^(Yes|No)(?:\b|$)", cells["shall always have a value"])
        if always_match is None:
            raise ValueError(f"pinned always-required cell has no Yes/No value: {caption}")
        if section in result:
            raise ValueError(f"duplicate pinned attribute policy table: {heading}")
        result[section] = {
            "source_policy_table": table_match.group(1),
            "source_always_required": always_match.group(1),
            "source_always_required_text": cells["shall always have a value"],
            "source_initially_set_by": cells["initially set by"],
            "source_modifiable_by_client": cells["modifiable by client"],
            "source_deletable_by_client": cells["deletable by client"],
        }
    return result


def _source_refs(*sections: str) -> list[dict[str, str]]:
    return [{"source_id": SPEC_SOURCE_ID, "section": section} for section in sections]


def _rule(source_text: str, *sections: str, structure_table: str | None = None) -> dict[str, Any]:
    entry: dict[str, Any] = {"source_refs": _source_refs(*sections), "source_text": source_text}
    if structure_table is not None:
        entry["structure_table"] = structure_table
    return entry


def _operation_rules(parser: _SourceParser, attributes: dict[str, dict[str, str]]) -> dict[str, list[dict[str, Any]]]:
    rules: dict[str, list[dict[str, Any]]] = {section: [] for section in attributes}
    specs = (
        ("4.28", "4.28", "It SHALL NOT be specified by the client in a Register request"),
        ("4.28", "4.28", "Key Value Present SHALL NOT be modified by either the client or the server"),
        ("4.30", "4.30", "This attribute is read-only for clients"),
        ("4.30", "4.30", "It SHALL be modified by the server only"),
        ("4.30", "6.1.2", "Read-Only attributes SHALL NOT be added using the Add Attribute operation"),
        ("4.30", "6.1.3", "Read-Only attributes SHALL NOT be added or modified using this operation"),
        ("4.30", "6.1.51", "Read-Only attributes SHALL NOT be added or modified using this operation"),
        ("4.57", "4.57", "The State SHALL NOT be changed by using the Modify Attribute operation"),
        ("4.57", "4.57", "The State SHALL only be changed by the server as a part of other operations"),
        ("4.59", "4.59", "The Usage Limits Count value SHALL NOT be set or modified by the client via the Add Attribute or Modify Attribute operations"),
    )
    for attribute_section, source_section, marker in specs:
        refs = [attribute_section] if source_section == attribute_section else [attribute_section, source_section]
        table_id = None
        if attribute_section == "4.59":
            table_id = "Table 392"
            if not any(
                table.get("heading") == "7.40 Usage Limits"
                and any(str(caption).startswith("Table 392:") for caption in table.get("captions", []))
                and ["Usage Limits Count", "Long Integer", "Yes"] in table.get("rows", [])
                for table in parser.tables
            ):
                raise ValueError("pinned Table 392 Usage Limits Count row is missing or changed")
            refs = ["4.59", "7.40"]
        rules[attribute_section].append(
            _rule(_source_sentence(parser, source_section, marker), *refs, structure_table=table_id)
        )

    always_required_deletion = _source_sentence(
        parser,
        "6.1.13",
        "Attributes that are always REQUIRED to have a value SHALL never be deleted",
    )
    for section, policy in attributes.items():
        if policy["source_always_required_text"] == "Yes":
            rules[section].append(_rule(always_required_deletion, section, "6.1.13"))
    return rules


def _conditional_rules(parser: _SourceParser, attributes: dict[str, dict[str, str]]) -> dict[str, list[dict[str, Any]]]:
    rules: dict[str, list[dict[str, Any]]] = {section: [] for section in attributes}
    prose_specs = (
        ("4.1", "Once the state transition from Pre-Active has occurred, then this attribute SHALL NOT be changed or deleted"),
        ("4.7", "The Certificate Type value SHALL be set by the server when the certificate is created or registered and then SHALL NOT be changed or deleted"),
        ("4.8", "The Certificate Length SHALL be set by the server when the object is created or registered, and then SHALL NOT be changed or deleted"),
        ("4.13", "This attribute SHALL be set by the server when the object is created or registered and then SHALL NOT be changed or deleted"),
        ("4.15", "This attribute SHALL be set by the server when the object is created or registered, and then SHALL NOT be changed or deleted"),
        ("4.18", "This attribute SHALL NOT be changed or deleted before the object is destroyed, unless the object is in the Pre-Active or Active state"),
        ("4.21", "The digest(s) are static and SHALL be set by the server when the object is created or registered"),
        ("4.22", "This attribute SHALL be set by the server when the object is created or registered and then SHALL NOT be changed or deleted"),
        ("4.25", "This attribute SHALL be set by the server when the object is created or registered, and then SHALL NOT be changed or deleted"),
        ("4.34", "Although the attribute is optional, once set, MAY NOT be deleted or modified"),
        ("4.36", "SHALL be set by the server when the object is created or registered and then SHALL NOT be changed or deleted"),
        ("4.37", "The Opaque Data Type of an Opaque Object SHALL be set by the server when the object is registered and then SHALL NOT be changed or deleted"),
        ("4.38", "In all cases, once the Original Creation Date is set, it SHALL NOT be deleted or updated"),
        ("4.40", "Once the Process Start Date has occurred, then this attribute SHALL NOT be changed or deleted"),
        ("4.41", "Once the Protect Stop Date has occurred, then this attribute SHALL NOT be changed or deleted"),
        ("4.46", "In all cases, once the Random Number Generator attribute is set, it SHALL NOT be deleted or updated"),
        ("4.56", "This attribute SHALL be assigned by the key management system upon creation or registration of a Unique Identifier, and then SHALL NOT be changed or deleted"),
        ("4.58", "This attribute SHALL be assigned by the key management system at creation or registration time, and then SHALL NOT be changed or deleted"),
        ("4.59", "Changes made via the Modify Attribute operation reflect corrections to the Usage Limits Total value, but they SHALL NOT be changed once the Usage Limits Count value has changed by a Get Usage Allocation operation"),
        ("4.61", "The X.509 Certificate Identifier SHALL be set by the server when the X.509 certificate is created or registered and then SHALL NOT be changed or deleted"),
        ("4.62", "These values SHALL NOT be changed or deleted before the object is destroyed"),
        ("4.63", "The X.509 Certificate Subject SHALL be set by the server based on the information it extracts from the X.509 certificate"),
    )
    for section, marker in prose_specs:
        rules[section].append(_rule(_source_sentence(parser, section, marker), section))

    for section, policy in attributes.items():
        for field in (
            "source_always_required_text",
            "source_modifiable_by_client",
            "source_deletable_by_client",
        ):
            text = policy[field]
            if text not in {"Yes", "No"}:
                candidate = _rule(text, section)
                if candidate not in rules[section]:
                    rules[section].append(candidate)
    return rules


@lru_cache(maxsize=1)
def expected_attribute_policies(raw: bytes) -> tuple[dict[str, dict[str, Any]], dict[str, Any]]:
    """Parse exact standard and Vendor Attribute policies from pinned KMIP 2.1."""
    if hashlib.sha256(raw).hexdigest() != SPEC_SHA256:
        raise ValueError("pinned KMIP Specification checksum changed")
    parser = _SourceParser()
    parser.feed(raw.decode("cp1252"))
    standard = _attribute_rule_tables(parser)
    if len(standard) != 62:
        raise ValueError(f"pinned KMIP Specification yielded {len(standard)} standard attribute rule tables, expected 62")

    operation_rules = _operation_rules(parser, standard)
    conditional_rules = _conditional_rules(parser, standard)
    for section, policy in standard.items():
        policy["source_operation_restrictions"] = operation_rules[section]
        policy["source_conditional_rules"] = conditional_rules[section]

    vendor_paragraphs = _section_paragraphs(parser, "4.60")
    vendor_text = next(
        (text for text in vendor_paragraphs if text.startswith("Vendor Attributes created by the server with Vendor Identification")),
        None,
    )
    has_table_150 = any(text.startswith("Table 150:") for text in vendor_paragraphs)
    if vendor_text is None or not has_table_150:
        raise ValueError("pinned §4.60 Vendor Attribute policy or Table 150 is missing")
    vendor = {
        "source_refs": _source_refs("4.60"),
        "structure_table": "Table 150",
        "source_text": vendor_text,
        "value_predicate": {
            "member_element_id": "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-VENDOR-IDENTIFICATION",
            "equals": "y",
            "source_indicates_origin": "server_created",
        },
        "prohibited_client_operations": [
            "created (provided during object creation)",
            "Set Attribute",
            "Add Attribute",
            "Adjust Attribute",
            "Modify Attribute",
            "Delete Attribute",
        ],
    }
    return standard, vendor

