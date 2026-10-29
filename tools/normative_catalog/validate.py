"""Strict offline validation for the reviewed KMIP 2.1 inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import subprocess
import sys
from pathlib import Path
from typing import Any

if __package__:
    from .attribute_policy_source import expected_attribute_policies
    from .generate_profiles import ProfileGenerationError, compute_profile_closure
    from .safe_io import PathSecurityError, confined_path, safe_read_bytes
else:
    from attribute_policy_source import expected_attribute_policies
    from generate_profiles import ProfileGenerationError, compute_profile_closure
    from safe_io import PathSecurityError, confined_path, safe_read_bytes


MAX_BYTES = 16 * 1024 * 1024
MAX_GIT_TREE_BYTES = MAX_BYTES
MAX_DEPTH = 32
MAX_RECORDS = 100_000
MAX_TOKENS = 1_000_000
MAX_MEMBERS = 1_000_000
MAX_STRING_BYTES = 65_536
TOP_LEVEL_FIELDS = {
    "schema_version",
    "sources",
    "source_clauses",
    "normative_inclusions",
    "elements",
    "tag_ranges",
    "requirements",
    "policies",
    "profiles",
    "test_cases",
    "discrepancies",
    "decisions",
}
SOURCE_ID_BY_FILE = {
    "kmip-profiles-v2.1-os.html": "KMIPKIT-SRC-profiles",
    "kmip-spec-v2.1-os.html": "KMIPKIT-SRC-spec",
    "kmip-testcases-v2.1-cn01.html": "KMIPKIT-SRC-testcases",
    "kmip-ug-v2.1-cn01.html": "KMIPKIT-SRC-usage-guide",
}
KEYWORD_STRENGTH = {
    "MUST": "mandatory",
    "SHALL": "mandatory",
    "REQUIRED": "mandatory",
    "MUST NOT": "prohibited",
    "SHALL NOT": "prohibited",
    "SHOULD": "recommended",
    "RECOMMENDED": "recommended",
    "SHOULD NOT": "discouraged",
    "MAY": "permission_or_optional",
    "OPTIONAL": "permission_or_optional",
}
SOURCE_REVIEWED_STRENGTH_OVERRIDES: dict[str, tuple[str, str]] = {
    "KMIPKIT-REQ-SPEC-6.1.28-007": (
        "KMIPKIT-CLAUSE-SPEC-6.1.28-007",
        "permission_or_optional",
    ),
}
TABLE_BODY_STRENGTH = {
    "required_table_body_value": "mandatory",
    "optional_table_body_value": "permission_or_optional",
}


def _expected_normative_strength(requirement: dict[str, Any]) -> str | None:
    """Return keyword strength, honoring exact source-reviewed negation exceptions."""
    override = SOURCE_REVIEWED_STRENGTH_OVERRIDES.get(requirement.get("requirement_id"))
    if override is not None and requirement.get("source_clause_ids") == [override[0]]:
        return override[1]
    return KEYWORD_STRENGTH.get(requirement.get("source_keyword"))


# §6.1's general Data rule conflicts with these multipart request-table rows.
_SHARED_DATA_RULE_CONFLICT_TABLES = frozenset({235, 259, 334})
SOURCE_AUTHORITY_RANK = {
    "informative": 0,
    "test_evidence": 1,
    "profile_normative": 2,
    "primary_normative": 3,
}
JSON_NUMBER_PATTERN = re.compile(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?")
CLAUSE_DISPOSITIONS = {
    "requirement",
    "profile_conditional",
    "server_only",
    "later_1_1",
    "non_applicable",
    "informative_context",
    "source_discrepancy",
}
CLAUSE_ROLES = {"client", "server", "both", "not_applicable", "unclear"}
CLAUSE_DIRECTIONS = {"client_to_server", "server_to_client", "both", "not_applicable", "unclear"}
CLAUSE_SCOPES = {
    "client_1_0",
    "client_1_1",
    "profile_conditional",
    "server_only",
    "out_of_scope",
    "mixed",
    "unclear",
}
REQUIREMENT_ID_PATTERN = r"KMIPKIT-REQ-(?:SPEC|PROF)-[0-9]+(?:\.[0-9]+)*-[0-9]{3}(?:-[0-9]{3})?"
TOP_LEVEL_ID_FIELDS = {
    "source_clauses": ("clause_id", r"KMIPKIT-CLAUSE-(?:SPEC|PROF)-[0-9]+(?:\.[0-9]+)*-[0-9]{3}"),
    "normative_inclusions": ("inclusion_id", r"KMIPKIT-INCLUSION-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "elements": ("element_id", r"KMIPKIT-ELEM-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "tag_ranges": ("range_id", r"KMIPKIT-RANGE-[0-9]{3}"),
    "requirements": ("requirement_id", REQUIREMENT_ID_PATTERN),
    "policies": ("policy_id", r"KMIPKIT-POLICY-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "profiles": ("profile_id", r"KMIPKIT-PROFILE-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "test_cases": ("test_id", r"KMIPKIT-TEST-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "discrepancies": ("discrepancy_id", r"KMIPKIT-DISC-[0-9]{3}"),
    "decisions": ("decision_id", r"KMIPKIT-DEC-[0-9]{3}"),
}
ELEMENT_FIELDS = {
    "element_id", "kind", "name", "source_refs", "wire_value", "allocation",
    "direction", "scope_state", "scope_reason", "parent_element_ids",
    "requirement_ids", "profile_ids", "test_case_ids", "feature_spec",
    "implementation_refs", "verification_refs", "payload_tables", "asynchronous_response",
    "source_encoding", "source_requiredness", "source_comment", "source_name",
    "source_initially_set_by", "source_modifiable_by_client", "source_deletable_by_client",
    "source_always_required", "source_policy_table", "source_operation_restrictions",
    "source_multiple_instances_permitted", "source_conditional_rules", "source_value_policies",
}
ATTRIBUTE_POLICY_FIELDS = frozenset(
    {
        "source_initially_set_by",
        "source_modifiable_by_client",
        "source_deletable_by_client",
        "source_always_required",
        "source_policy_table",
        "source_multiple_instances_permitted",
        "source_operation_restrictions",
        "source_conditional_rules",
        "source_value_policies",
    }
)
ELEMENT_KINDS = {
    "operation", "message_field", "structure_member", "credential", "data_type",
    "object_type", "object_structure", "attribute", "attribute_structure",
    "operation_structure", "tag", "enumeration", "enumeration_value", "bitmask",
    "bitmask_value", "option", "result", "extension_rule",
}
TEST_CASE_FIELDS = {
    "test_id", "official_case_id", "source_id", "source_section", "evidence_category",
    "mandatory_status", "profile_ids", "requirement_ids", "element_ids", "raw_href",
    "fixture_path", "fixture_availability", "mapping_confidence",
}
PROFILE_FIELDS = {
    "profile_id", "name", "role", "source_refs", "direct_source_clause_ids", "source_clause_ids",
    "dependency_profile_ids", "transport_requirements", "encoding_requirements",
    "applicability", "claim_state", "requirement_ids", "element_ids", "test_case_ids",
}
DISCREPANCY_FIELDS = {
    "discrepancy_id", "summary", "source_refs", "source_authority", "normative_status",
    "alternatives", "affected_requirement_ids", "affected_element_ids",
    "affected_profile_ids", "affected_policy_ids", "downstream_impact", "state", "decision_id",
    "erratum_source_refs",
}
DECISION_FIELDS = {
    "decision_id", "source_refs", "requirement_ids", "discrepancy_ids",
    "policy_ids", "interpretation", "approver", "approval_evidence", "approved_at", "consequence", "status",
}
POLICY_FIELDS = {
    "policy_id", "summary", "provenance", "provenance_ref", "affected_element_kinds", "requirement_ids",
}


class CatalogValidationError(ValueError):
    """Raised when the catalog fails a structural or semantic rule."""


def _fail(message: str) -> None:
    raise CatalogValidationError(message)


def _fail_profile_closure_mismatch(
    label: str,
    actual_ids: list[str],
    expected_ids: list[str],
) -> None:
    """Report stable IDs missing from or unexpected in a computed profile closure."""
    actual = set(actual_ids)
    expected = set(expected_ids)
    missing = [identifier for identifier in expected_ids if identifier not in actual]
    unexpected = [identifier for identifier in actual_ids if identifier not in expected]
    if missing or unexpected:
        details = []
        if missing:
            details.append(f"missing IDs {missing[:10]}")
        if unexpected:
            details.append(f"unexpected IDs {unexpected[:10]}")
        _fail(f"{label} closure mismatch: " + "; ".join(details))

    _fail(
        f"{label} closure order mismatch: "
        f"expected IDs beginning {expected_ids[:10]}; found IDs beginning {actual_ids[:10]}"
    )


def _enum(value: Any, choices: set[str], field: str) -> None:
    if not isinstance(value, str) or value not in choices:
        _fail(f"{field} has an invalid value")


def _reject_duplicate_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            _fail("catalog contains duplicate JSON object keys")
        result[key] = value
    return result


def _string_utf8_length(value: str) -> int:
    length = 0
    index = 0
    while index < len(value):
        codepoint = ord(value[index])
        if 0xD800 <= codepoint <= 0xDBFF:
            if index + 1 >= len(value) or not 0xDC00 <= ord(value[index + 1]) <= 0xDFFF:
                _fail("catalog contains an unpaired Unicode surrogate")
            length += 4
            index += 2
        elif 0xDC00 <= codepoint <= 0xDFFF:
            _fail("catalog contains an unpaired Unicode surrogate")
        else:
            length += len(value[index].encode("utf-8"))
            index += 1
        if length > MAX_STRING_BYTES:
            _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
    return length


class _JsonPreflight:
    """Validate JSON grammar and allocation bounds without building a graph."""

    def __init__(self, text: str) -> None:
        self.text = text
        self.position = 0
        self.tokens = 0
        self.members = 0

    def _skip_space(self) -> None:
        while self.position < len(self.text) and self.text[self.position] in " \t\r\n":
            self.position += 1

    def _count_token(self) -> None:
        self.tokens += 1
        if self.tokens > MAX_TOKENS:
            _fail("catalog exceeds the global JSON token limit")

    def _read_string(self) -> str:
        start = self.position
        self.position += 1
        raw_bytes = 1
        decoded_bytes = 0
        while self.position < len(self.text):
            character = self.text[self.position]
            if character == '"':
                self.position += 1
                raw_bytes += 1
                if raw_bytes > MAX_STRING_BYTES * 6 + 2:
                    _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
                encoded = self.text[start:self.position].encode("utf-8")
                try:
                    value = json.loads(encoded.decode("utf-8"))
                except (json.JSONDecodeError, UnicodeDecodeError) as error:
                    raise CatalogValidationError("catalog contains an invalid JSON string") from error
                _string_utf8_length(value)
                return value
            if ord(character) < 0x20:
                _fail("catalog contains an unescaped JSON control character")
            if character == "\\":
                raw_bytes += 1
                self.position += 1
                if self.position >= len(self.text):
                    _fail("catalog contains a truncated JSON escape")
                escape = self.text[self.position]
                raw_bytes += 1
                if escape == "u":
                    digits = self.text[self.position + 1:self.position + 5]
                    if len(digits) != 4 or re.fullmatch(r"[0-9a-fA-F]{4}", digits) is None:
                        _fail("catalog contains an invalid Unicode escape")
                    raw_bytes += 4
                    codepoint = int(digits, 16)
                    if 0xD800 <= codepoint <= 0xDBFF:
                        low_start = self.position + 5
                        if self.text[low_start:low_start + 2] != "\\u":
                            _fail("catalog contains an unpaired Unicode surrogate")
                        low_digits = self.text[low_start + 2:low_start + 6]
                        if len(low_digits) != 4 or re.fullmatch(r"[0-9a-fA-F]{4}", low_digits) is None:
                            _fail("catalog contains an invalid Unicode escape")
                        low_surrogate = int(low_digits, 16)
                        if not 0xDC00 <= low_surrogate <= 0xDFFF:
                            _fail("catalog contains an unpaired Unicode surrogate")
                        raw_bytes += 6
                        decoded_bytes += 4
                        self.position += 11
                    elif 0xDC00 <= codepoint <= 0xDFFF:
                        _fail("catalog contains an unpaired Unicode surrogate")
                    else:
                        decoded_bytes += len(chr(codepoint).encode("utf-8"))
                        self.position += 5
                    if raw_bytes > MAX_STRING_BYTES * 6 + 2 or decoded_bytes > MAX_STRING_BYTES:
                        _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
                    continue
                if escape not in '"\\/bfnrt':
                    _fail("catalog contains an invalid JSON escape")
                decoded_bytes += 1
                self.position += 1
                if raw_bytes > MAX_STRING_BYTES * 6 + 2 or decoded_bytes > MAX_STRING_BYTES:
                    _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
                continue
            byte_length = len(character.encode("utf-8"))
            raw_bytes += byte_length
            decoded_bytes += byte_length
            if raw_bytes > MAX_STRING_BYTES * 6 + 2 or decoded_bytes > MAX_STRING_BYTES:
                _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
            self.position += 1
        _fail("catalog contains an unterminated JSON string")

    def _value(self, depth: int, *, top_collection: str | None = None, root: bool = False) -> None:
        self._skip_space()
        if self.position >= len(self.text):
            _fail("catalog contains truncated JSON")
        self._count_token()
        character = self.text[self.position]
        if character == "{":
            next_depth = depth + 1
            if next_depth > MAX_DEPTH:
                _fail("catalog exceeds the maximum nesting depth of 32")
            self.position += 1
            self._skip_space()
            keys: set[str] = set()
            if self.position < len(self.text) and self.text[self.position] == "}":
                self.position += 1
                if root:
                    _fail("catalog root must contain required top-level fields")
                return
            while True:
                self._skip_space()
                if self.position >= len(self.text) or self.text[self.position] != '"':
                    _fail("catalog object keys must be strings")
                self._count_token()
                key = self._read_string()
                if key in keys:
                    _fail("catalog contains duplicate JSON object keys")
                keys.add(key)
                self.members += 1
                if self.members > MAX_MEMBERS:
                    _fail("catalog exceeds the global JSON object-member limit")
                if root and key not in TOP_LEVEL_FIELDS:
                    _fail("catalog has unknown top-level fields")
                self._skip_space()
                if self.position >= len(self.text) or self.text[self.position] != ":":
                    _fail("catalog object member is missing a colon")
                self.position += 1
                self._value(next_depth, top_collection=key if root and key != "schema_version" else None)
                self._skip_space()
                if self.position >= len(self.text):
                    _fail("catalog object is truncated")
                delimiter = self.text[self.position]
                self.position += 1
                if delimiter == "}":
                    break
                if delimiter != ",":
                    _fail("catalog object has an invalid delimiter")
            return
        if character == "[":
            next_depth = depth + 1
            if next_depth > MAX_DEPTH:
                _fail("catalog exceeds the maximum nesting depth of 32")
            self.position += 1
            self._skip_space()
            if self.position < len(self.text) and self.text[self.position] == "]":
                self.position += 1
                return
            records = 0
            while True:
                self._value(next_depth)
                if top_collection is not None:
                    records += 1
                    if records > MAX_RECORDS:
                        _fail(f"{top_collection} exceeds the 100,000 record limit")
                self._skip_space()
                if self.position >= len(self.text):
                    _fail("catalog array is truncated")
                delimiter = self.text[self.position]
                self.position += 1
                if delimiter == "]":
                    break
                if delimiter != ",":
                    _fail("catalog array has an invalid delimiter")
            return
        if character == '"':
            self._read_string()
            return
        literal = next((word for word in ("true", "false", "null") if self.text.startswith(word, self.position)), None)
        if literal is not None:
            self.position += len(literal)
            return
        number = JSON_NUMBER_PATTERN.match(self.text, self.position)
        if number is None or len(number.group(0)) > 128:
            _fail("catalog contains an invalid or overlong JSON value")
        self.position += len(number.group(0))

    def validate(self) -> None:
        self._skip_space()
        if not self.text.startswith("{", self.position):
            _fail("catalog root must be an object")
        self._value(0, root=True)
        self._skip_space()
        if self.position != len(self.text):
            _fail("catalog contains trailing JSON data")


def _preflight(raw: bytes) -> str:
    if len(raw) > MAX_BYTES:
        _fail("catalog exceeds the 16 MiB input limit")
    try:
        text = raw.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise CatalogValidationError("catalog is not valid UTF-8") from error
    _JsonPreflight(text).validate()
    return text


def _reject_surrogates(value: Any) -> None:
    if isinstance(value, str):
        _string_utf8_length(value)
    elif isinstance(value, list):
        for item in value:
            _reject_surrogates(item)
    elif isinstance(value, dict):
        for key, item in value.items():
            _reject_surrogates(key)
            _reject_surrogates(item)


def _source_manifest(root: Path) -> dict[str, dict[str, str]]:
    try:
        inventory = safe_read_bytes(root, "specification/oasis/kmip-2.1/SOURCES.md", max_bytes=1_048_576).decode("utf-8")
        checksums = safe_read_bytes(root, "specification/oasis/kmip-2.1/CHECKSUMS.sha256", max_bytes=1_048_576).decode("ascii")
    except (OSError, UnicodeError, PathSecurityError) as error:
        raise CatalogValidationError("pinned OASIS source manifest is unavailable") from error

    digest_by_path: dict[str, str] = {}
    for line in checksums.splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})\s+\*?(upstream/[A-Za-z0-9._-]+)", line)
        if match is None:
            _fail("pinned OASIS checksum manifest has an invalid row")
        digest_by_path[match.group(2)] = match.group(1)

    expected: dict[str, dict[str, str]] = {}
    for line in inventory.splitlines():
        if not line.startswith("| `upstream/"):
            continue
        columns = [column.strip() for column in line.strip().strip("|").split("|")]
        if len(columns) != 5:
            _fail("pinned OASIS source inventory has an invalid row")
        local_path = columns[0].strip("`")
        title, stage, date = columns[1:4]
        url = columns[4].strip("<>")
        filename = Path(local_path).name
        source_id = SOURCE_ID_BY_FILE.get(filename)
        digest = digest_by_path.get(local_path.removeprefix("upstream/"))
        # CHECKSUMS paths are relative to the kmip-2.1 directory's upstream.
        if digest is None:
            digest = digest_by_path.get(local_path)
        if source_id is None or digest is None:
            _fail("source inventory and checksum manifest do not match the allowlist")
        authority = {
            "KMIPKIT-SRC-profiles": "profile_normative",
            "KMIPKIT-SRC-spec": "primary_normative",
            "KMIPKIT-SRC-testcases": "test_evidence",
            "KMIPKIT-SRC-usage-guide": "informative",
        }[source_id]
        expected[source_id] = {
            "title": title,
            "version": "2.1",
            "stage": stage,
            "publication_date": date,
            "local_path": f"specification/oasis/kmip-2.1/{local_path}",
            "canonical_url": url,
            "sha256": digest,
            "authority_class": authority,
        }
    if set(expected) != set(SOURCE_ID_BY_FILE.values()):
        _fail("pinned OASIS source inventory does not contain the four expected documents")
    return expected


def _source_refs(value: Any, sources: set[str], field: str) -> None:
    if not isinstance(value, list) or not value:
        _fail(f"{field} must contain at least one source reference")
    for reference in value:
        if not isinstance(reference, dict) or set(reference) != {"source_id", "section"}:
            _fail(f"{field} contains a malformed source reference")
        if reference["source_id"] not in sources:
            _fail(f"{field} refers to an unknown source")
        if not isinstance(reference["section"], str) or not re.fullmatch(r"[0-9]+(?:\.[0-9]+)*", reference["section"]):
            _fail(f"{field} contains an invalid source section")


def _attribute_rule_signature(value: Any, field: str) -> tuple[str, tuple[tuple[str, str], ...], str | None]:
    if not isinstance(value, dict):
        _fail(f"{field} contains a malformed source rule")
    expected_fields = {"source_refs", "source_text"}
    structure_table = value.get("structure_table")
    if "structure_table" in value:
        expected_fields.add("structure_table")
        if not isinstance(structure_table, str) or not structure_table.strip():
            _fail(f"{field} contains an invalid structure table reference")
    if set(value) != expected_fields:
        _fail(f"{field} contains missing or unsupported rule fields")
    source_text = value.get("source_text")
    if not isinstance(source_text, str) or not source_text:
        _fail(f"{field} contains empty source text")
    references = value.get("source_refs")
    if not isinstance(references, list) or not references:
        _fail(f"{field} contains no source references")
    pairs: list[tuple[str, str]] = []
    for reference in references:
        if not isinstance(reference, dict) or set(reference) != {"source_id", "section"}:
            _fail(f"{field} contains a malformed source reference")
        source_id, section = reference.get("source_id"), reference.get("section")
        if not isinstance(source_id, str) or not isinstance(section, str):
            _fail(f"{field} contains a malformed source reference")
        pairs.append((source_id, section))
    if len(pairs) != len(set(pairs)):
        _fail(f"{field} contains duplicate source references")
    return source_text, tuple(sorted(pairs)), structure_table


def _check_attribute_policy_metadata(catalog: dict[str, Any], repo_root: Path) -> None:
    standard_fields = ATTRIBUTE_POLICY_FIELDS - {"source_value_policies"}
    attributes = [element for element in catalog["elements"] if element.get("kind") == "attribute"]
    policy_fields = ATTRIBUTE_POLICY_FIELDS
    feature_catalog = any(attribute.get("feature_spec") == "KMIPKIT-0016" for attribute in attributes)
    has_policy_metadata = any(policy_fields.intersection(attribute) for attribute in attributes)
    if not feature_catalog and not has_policy_metadata:
        return

    specification_source = next(
        (source for source in catalog["sources"] if source.get("source_id") == "KMIPKIT-SRC-spec"),
        None,
    )
    if specification_source is None:
        _fail("attribute policy validation requires the pinned KMIP Specification source")
    try:
        raw = safe_read_bytes(repo_root, specification_source["local_path"], max_bytes=MAX_BYTES)
        expected_standard, expected_vendor = expected_attribute_policies(raw)
    except (OSError, UnicodeError, PathSecurityError, ValueError) as error:
        raise CatalogValidationError("pinned KMIP attribute policy source is unavailable or invalid") from error

    elements = catalog["elements"]
    standard_by_section: dict[str, dict[str, Any]] = {}
    vendor_attributes: list[dict[str, Any]] = []
    for attribute in attributes:
        sections = [
            reference["section"]
            for reference in attribute.get("source_refs", [])
            if reference.get("source_id") == "KMIPKIT-SRC-spec"
            and isinstance(reference.get("section"), str)
            and reference["section"].startswith("4.")
        ]
        relevant_sections = [section for section in sections if section in expected_standard or section == "4.60"]
        if len(relevant_sections) > 1:
            _fail("attribute source policy has ambiguous §4 source references")
        section = relevant_sections[0] if relevant_sections else None
        if section in expected_standard:
            if section in standard_by_section:
                _fail("standard attribute source policy section is assigned more than once")
            standard_by_section[section] = attribute
            if "source_value_policies" in attribute:
                _fail("standard attribute cannot contain a Vendor Attribute value policy")
        elif section == "4.60":
            vendor_attributes.append(attribute)
            if standard_fields.intersection(attribute):
                _fail("Vendor Attribute cannot contain standard attribute mutation fields")
        elif standard_fields.intersection(attribute) or "source_value_policies" in attribute:
            _fail("attribute policy metadata is attached to an unsupported attribute")

    if set(standard_by_section) != set(expected_standard):
        _fail("catalog must contain exactly the 62 pinned standard attribute policy records")
    for section, attribute in standard_by_section.items():
        expected = expected_standard[section]
        if not standard_fields.issubset(attribute):
            _fail("standard attribute is missing required source policy metadata")
        for field in (
            "source_initially_set_by",
            "source_modifiable_by_client",
            "source_deletable_by_client",
            "source_always_required",
            "source_policy_table",
            "source_multiple_instances_permitted",
        ):
            if attribute.get(field) != expected[field]:
                _fail(f"standard attribute {field} does not match its pinned source cell")
        for field in ("source_operation_restrictions", "source_conditional_rules"):
            actual = attribute.get(field)
            source_field = field
            wanted = expected[source_field]
            if not isinstance(actual, list):
                _fail(f"standard attribute {field} must be an array")
            actual_signatures = [_attribute_rule_signature(entry, field) for entry in actual]
            wanted_signatures = [_attribute_rule_signature(entry, field) for entry in wanted]
            if len(actual_signatures) != len(set(actual_signatures)) or sorted(actual_signatures) != sorted(wanted_signatures):
                _fail(f"standard attribute {field} does not match its exact pinned source rules")

    if len(vendor_attributes) != 1:
        _fail("catalog must contain one separately traceable Vendor Attribute policy record")
    vendor_attribute = vendor_attributes[0]
    policies = vendor_attribute.get("source_value_policies")
    if not isinstance(policies, list) or len(policies) != 1 or policies[0] != expected_vendor:
        _fail("Vendor Attribute value policy does not match pinned §4.60 and Table 150")

    member_expectations = {
        "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-NAME": "Attribute Name",
        "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-VALUE": "Attribute Value",
        "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-VENDOR-IDENTIFICATION": "Vendor Identification",
    }
    by_id = {element["element_id"]: element for element in elements}
    for member_id, name in member_expectations.items():
        member = by_id.get(member_id)
        if (
            member is None
            or member.get("kind") != "structure_member"
            or member.get("name") != name
            or vendor_attribute["element_id"] not in member.get("parent_element_ids", [])
            or not any(
                reference.get("source_id") == "KMIPKIT-SRC-spec" and reference.get("section") == "4.60"
                for reference in member.get("source_refs", [])
            )
        ):
            _fail("Vendor Attribute Table 150 member linkage is incomplete")
    source_test = next(
        (test for test in catalog["test_cases"] if test.get("test_id") == "KMIPKIT-TEST-CN01-2-42"),
        None,
    )
    if (
        source_test is None
        or source_test.get("official_case_id") != "TC-I18N-3-21"
        or source_test.get("source_id") != "KMIPKIT-SRC-testcases"
        or "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-ATTRIBUTE-VALUE" not in source_test.get("element_ids", [])
    ):
        _fail("Vendor Attribute source-linked test case is not assigned to its Table 150 value member")


def _check_clause_scope_metadata(clause: dict[str, Any]) -> None:
    """Validate the reviewed actor, direction, scope, and condition of one candidate."""
    _enum(clause["role"], CLAUSE_ROLES, "source clause role")
    _enum(clause["direction"], CLAUSE_DIRECTIONS, "source clause direction")
    _enum(clause["scope_state"], CLAUSE_SCOPES, "source clause scope")
    if clause["condition"] is not None and (
        not isinstance(clause["condition"], str) or not clause["condition"].strip()
    ):
        _fail("source clause condition must be null or non-empty text")


def _linked_clause_scope(clause: dict[str, Any], requirements: dict[str, dict[str, Any]]) -> str | None:
    """Return the common linked scope, or mixed when separable scopes differ."""
    scopes = {requirements[item]["scope_state"] for item in clause["requirement_ids"]}
    if not scopes:
        return None
    if (
        scopes == {"client_1_0"}
        and clause.get("scope_state") == "mixed"
        and clause.get("disposition") == "source_discrepancy"
    ):
        # A source discrepancy can retain a mixed clause scope when only its
        # client-facing atom has a normative client requirement record.
        return "mixed"
    return next(iter(scopes)) if len(scopes) == 1 else "mixed"


def _strongest_source_authority(
    references: list[dict[str, Any]],
    authority_by_source: dict[str, str],
) -> str:
    """Return the highest authority class among a record's citations."""
    return max(
        (authority_by_source[reference["source_id"]] for reference in references),
        key=SOURCE_AUTHORITY_RANK.__getitem__,
    )


def _check_policy_provenance(
    policy: dict[str, Any],
    decisions: dict[str, dict[str, Any]],
    repo_root: Path,
) -> None:
    """Require a policy citation to resolve to an accepted project record."""
    provenance = policy["provenance"]
    reference = policy["provenance_ref"]
    if provenance == "approved_product_decision":
        if not isinstance(reference, dict) or set(reference) != {"decision_id"}:
            _fail("project policy provenance must identify an approved decision")
        decision = decisions.get(reference["decision_id"])
        if (
            decision is None
            or decision.get("status") != "accepted"
            or policy["policy_id"] not in decision.get("policy_ids", [])
        ):
            _fail("project policy provenance decision is unresolved or unlinked")
        return

    if not isinstance(reference, dict) or set(reference) != {"path", "heading"}:
        _fail("project policy provenance requires a path and heading locator")
    source_path = reference["path"]
    heading = reference["heading"]
    if not isinstance(source_path, str) or not isinstance(heading, str) or not heading.strip():
        _fail("project policy provenance locator is malformed")
    if provenance == "AGENTS.md" and source_path != "AGENTS.md":
        _fail("project policy provenance path does not match AGENTS.md")
    if provenance == "constitution" and source_path != ".specify/memory/constitution.md":
        _fail("project policy provenance path does not match the constitution")
    if provenance == "ADR" and re.fullmatch(r"docs/adr/[0-9]{4}-[a-z0-9-]+\.md", source_path) is None:
        _fail("project policy provenance path is not an ADR")
    try:
        content = safe_read_bytes(repo_root, source_path, max_bytes=1_048_576).decode("utf-8", errors="strict")
    except (OSError, UnicodeError, PathSecurityError) as error:
        raise CatalogValidationError("project policy provenance source is unavailable") from error
    heading_pattern = re.compile(r"(?m)^#{1,6}[ \t]+" + re.escape(heading) + r"[ \t]*#*[ \t]*$")
    if heading_pattern.search(content) is None:
        _fail("project policy provenance heading does not exist in its source")
    if provenance == "ADR" and re.search(r"(?m)^Status:[ \t]*Accepted[ \t]*$", content) is None:
        _fail("project policy ADR provenance is not accepted")


def _git_tree(root: Path) -> dict[str, tuple[str, str]]:
    try:
        process = subprocess.Popen(
            ["git", "ls-tree", "-r", "-z", "--full-tree", "HEAD", "--", "specification/oasis/"],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
    except OSError as error:
        raise CatalogValidationError("could not read the pinned OASIS Git tree metadata") from error
    if process.stdout is None:
        process.kill()
        process.wait()
        raise CatalogValidationError("Git did not provide a readable OASIS tree stream")
    chunks: list[bytes] = []
    total_bytes = 0
    try:
        while True:
            chunk = process.stdout.read(min(65_536, MAX_GIT_TREE_BYTES - total_bytes + 1))
            if not chunk:
                break
            chunks.append(chunk)
            total_bytes += len(chunk)
            if total_bytes > MAX_GIT_TREE_BYTES:
                process.kill()
                process.wait()
                raise CatalogValidationError("pinned OASIS Git tree metadata exceeds its size limit")
        return_code = process.wait()
    except OSError as error:
        process.kill()
        process.wait()
        raise CatalogValidationError("could not read the pinned OASIS Git tree metadata") from error
    finally:
        process.stdout.close()
        if process.poll() is None:
            process.kill()
            process.wait()
    if return_code != 0:
        raise CatalogValidationError("could not read the pinned OASIS Git tree metadata")
    tree: dict[str, tuple[str, str]] = {}
    for entry in b"".join(chunks).split(b"\0"):
        if not entry:
            continue
        if len(tree) >= MAX_RECORDS:
            _fail("pinned OASIS Git tree exceeds its entry limit")
        try:
            metadata, path_bytes = entry.split(b"\t", 1)
            mode, object_type, _object_id = metadata.decode("ascii").split(" ", 2)
            path = path_bytes.decode("utf-8", errors="strict")
        except (ValueError, UnicodeDecodeError) as error:
            raise CatalogValidationError("Git returned malformed OASIS tree metadata") from error
        tree[path] = (mode, object_type)
    if not tree:
        _fail("the pinned OASIS Git tree is empty")
    return tree


def _reject_reparse_points(root: Path, tree: dict[str, tuple[str, str]]) -> None:
    reparse_attribute = getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
    for relative, (mode, object_type) in tree.items():
        if mode == "120000" or object_type != "blob":
            _fail("the pinned OASIS tree contains a symlink or non-file entry")
        try:
            path = confined_path(root, relative)
            metadata = path.lstat()
        except (OSError, PathSecurityError) as error:
            raise CatalogValidationError("a pinned OASIS tree entry is unavailable") from error
        if stat.S_ISLNK(metadata.st_mode) or getattr(metadata, "st_file_attributes", 0) & reparse_attribute:
            _fail("the pinned OASIS tree contains a symlink or reparse point")


def _check_identifiers(catalog: dict[str, Any]) -> None:
    seen: set[str] = set()
    for collection, (field, pattern) in TOP_LEVEL_ID_FIELDS.items():
        for record in catalog[collection]:
            if not isinstance(record, dict):
                _fail(f"{collection} entries must be objects")
            identifier = record.get(field)
            if not isinstance(identifier, str) or re.fullmatch(pattern, identifier) is None:
                _fail(f"{collection} contains an invalid {field}")
            if identifier in seen:
                _fail(f"duplicate stable record identifier: {identifier}")
            seen.add(identifier)


def _records_by_id(catalog: dict[str, Any], collection: str, field: str) -> dict[str, dict[str, Any]]:
    return {record[field]: record for record in catalog[collection]}


def _check_link_ids(record: dict[str, Any], field: str, known_ids: set[str], collection: str) -> None:
    values = record.get(field)
    if not isinstance(values, list) or any(not isinstance(value, str) for value in values):
        _fail(f"{collection} {field} must be an array of identifiers")
    if len(values) != len(set(values)):
        _fail(f"{collection} {field} contains duplicate identifiers")
    if any(value not in known_ids for value in values):
        _fail(f"{collection} contains an unresolved {field} reference")


def _check_requirement_test_evidence(
    requirements: list[dict[str, Any]], test_cases: list[dict[str, Any]]
) -> None:
    """Require reciprocal official test links or a documented evidence gap."""
    expected_by_requirement = {row["requirement_id"]: set() for row in requirements}
    for test_case in test_cases:
        for requirement_id in test_case["requirement_ids"]:
            expected_by_requirement[requirement_id].add(test_case["test_id"])

    for requirement in requirements:
        requirement_id = requirement["requirement_id"]
        if set(requirement["test_case_ids"]) != expected_by_requirement[requirement_id]:
            _fail("requirement/test-case links must be reciprocal")
        if not requirement["test_case_ids"] and not (
            isinstance(requirement["review_note"], str) and requirement["review_note"].strip()
        ):
            _fail("requirement without official test-case links must record an evidence gap in review_note")


def _string_values(record: dict[str, Any], field: str, collection: str, *, non_empty: bool = False) -> list[str]:
    values = record.get(field)
    if (
        not isinstance(values, list)
        or any(not isinstance(value, str) or not value.strip() for value in values)
        or (non_empty and not values)
        or len(values) != len(set(values))
    ):
        _fail(f"{collection} {field} must contain unique non-empty strings")
    return values


def _check_source_records(catalog: dict[str, Any], root: Path, tree: dict[str, tuple[str, str]]) -> set[str]:
    expected = _source_manifest(root)
    sources = catalog["sources"]
    if not isinstance(sources, list) or len(sources) != len(expected):
        _fail("catalog must contain exactly the four pinned source documents")
    by_id: dict[str, dict[str, Any]] = {}
    for source in sources:
        if not isinstance(source, dict):
            _fail("source entries must be objects")
        identifier = source.get("source_id")
        if identifier not in expected or identifier in by_id:
            _fail("catalog contains an unknown or duplicate source identifier")
        if set(source) != {"source_id", *expected[identifier].keys()}:
            _fail("source entry has missing or unknown fields")
        by_id[identifier] = source
    for identifier, metadata in expected.items():
        source = by_id.get(identifier)
        if source is None or any(source.get(key) != value for key, value in metadata.items()):
            _fail(f"pinned source metadata or checksum mismatch for {identifier}")
        if metadata["local_path"] not in tree:
            _fail("pinned source metadata does not exist in the Git tree")
    return set(expected)


def _check_clauses(catalog: dict[str, Any], sources: set[str]) -> set[str]:
    clauses: set[str] = set()
    for clause in catalog["source_clauses"]:
        if not isinstance(clause, dict):
            _fail("source clause entries must be objects")
        required = {
            "clause_id", "source_id", "section", "locator", "source_keywords",
            "disposition", "requirement_ids", "exclusion_rationale", "role", "direction",
            "scope_state", "condition",
        }
        if set(clause) != required:
            _fail("source clause has missing or unknown fields")
        clause_id = clause["clause_id"]
        if clause_id in clauses:
            _fail(f"duplicate source clause identifier: {clause_id}")
        clauses.add(clause_id)
        if clause["source_id"] not in sources:
            _fail("source clause refers to an unknown source")
        if not re.fullmatch(r"[0-9]+(?:\.[0-9]+)*", clause["section"]):
            _fail("source clause has an invalid section")
        locator = clause["locator"]
        if not isinstance(locator, dict) or set(locator) != {"ordinal", "block_kind"}:
            _fail("source clause has an invalid structural locator")
        if not isinstance(locator["ordinal"], int) or locator["ordinal"] < 1:
            _fail("source clause locator ordinal must be positive")
        if locator["block_kind"] not in {"paragraph", "list_item", "table_row", "definition_item"}:
            _fail("source clause has an invalid block kind")
        keywords = clause["source_keywords"]
        if not isinstance(keywords, list) or not keywords or any(keyword not in KEYWORD_STRENGTH for keyword in keywords):
            _fail("source clause has an invalid source keyword")
        _enum(clause["disposition"], CLAUSE_DISPOSITIONS, "source clause disposition")
        _check_clause_scope_metadata(clause)
        if not isinstance(clause["requirement_ids"], list):
            _fail("source clause requirement_ids must be an array")
        rationale = clause["exclusion_rationale"]
        if clause["disposition"] in {"requirement", "profile_conditional"}:
            if not clause["requirement_ids"] or rationale is not None:
                _fail("applicable clauses need requirement links and no exclusion rationale")
        elif clause["disposition"] == "source_discrepancy":
            if clause["requirement_ids"] and rationale is not None:
                _fail("source-discrepancy clauses with linked requirements cannot have an exclusion rationale")
            if not clause["requirement_ids"] and (
                not isinstance(rationale, str) or not rationale.strip()
            ):
                _fail("unresolved source-discrepancy clauses need an exclusion rationale")
        elif clause["requirement_ids"] or not isinstance(rationale, str) or not rationale.strip():
            _fail("excluded source clauses need a rationale and no requirement links")
    return clauses


def _check_normative_inclusions(catalog: dict[str, Any], sources: set[str], clauses: set[str]) -> None:
    """Validate source-backed normative inclusion edges and reject reachable cycles."""
    clause_records = {record["clause_id"]: record for record in catalog["source_clauses"]}
    included_edges: set[tuple[str, str, str | None]] = set()
    for inclusion in catalog["normative_inclusions"]:
        if not isinstance(inclusion, dict) or set(inclusion) != {
            "inclusion_id", "source_clause_id", "included_source_id", "included_section"
        }:
            _fail("normative inclusion has missing or unknown fields")
        source_clause_id = inclusion["source_clause_id"]
        included_source_id = inclusion["included_source_id"]
        included_section = inclusion["included_section"]
        if source_clause_id not in clauses:
            _fail("normative inclusion refers to an unresolved source clause")
        if not isinstance(included_source_id, str) or included_source_id not in sources:
            _fail("normative inclusion refers to an unresolved included source")
        if included_section is not None and (
            not isinstance(included_section, str)
            or re.fullmatch(r"[0-9]+(?:\.[0-9]+)*", included_section) is None
        ):
            _fail("normative inclusion has an invalid included section")
        edge = (source_clause_id, included_source_id, included_section)
        if edge in included_edges:
            _fail("catalog contains a duplicate normative inclusion edge")
        included_edges.add(edge)
        matching_clauses = [
            clause
            for clause in catalog["source_clauses"]
            if clause["source_id"] == included_source_id
            and (
                included_section is None
                or clause["section"] == included_section
                or clause["section"].startswith(f"{included_section}.")
            )
        ]
        if not matching_clauses:
            _fail("normative inclusion does not resolve to a catalogued source section")

    if catalog["normative_inclusions"]:
        try:
            compute_profile_closure(
                catalog,
                [record["source_clause_id"] for record in catalog["normative_inclusions"]],
                profile_id="KMIPKIT-INCLUSION-VALIDATION",
            )
        except ProfileGenerationError as error:
            raise CatalogValidationError("normative inclusion closure is invalid") from error


def _has_valid_table_body_source(requirement: dict[str, Any]) -> bool:
    """Check that a table-derived requirement retains its exact source row."""
    source_force = requirement.get("source_force")
    locator = requirement.get("source_locator")
    source_refs = requirement.get("source_refs", [])
    if source_force not in TABLE_BODY_STRENGTH or not source_refs:
        return False
    line_range = locator.get("html_line_range") if isinstance(locator, dict) else None
    return (
        requirement.get("source_keyword") is None
        and requirement.get("normative_strength") == TABLE_BODY_STRENGTH[source_force]
        and requirement.get("source_clause_ids") == []
        and isinstance(requirement.get("source_gap_id"), str)
        and bool(requirement["source_gap_id"].strip())
        and isinstance(locator, dict)
        and locator.get("kind") == "table_body_row"
        and isinstance(locator.get("section"), str)
        and isinstance(locator.get("row_label"), str)
        and bool(locator["row_label"].strip())
        and isinstance(line_range, list)
        and len(line_range) == 2
        and all(isinstance(line, int) and not isinstance(line, bool) and line >= 1 for line in line_range)
        and line_range[0] <= line_range[1]
        and any(
            reference.get("source_id") == source_refs[0].get("source_id")
            and reference.get("section") == locator["section"]
            for reference in source_refs
        )
    )


def _check_semantics(
    catalog: dict[str, Any],
    sources: set[str],
    clauses: set[str],
    tree: dict[str, tuple[str, str]],
    repo_root: Path,
) -> None:
    requirements = {record["requirement_id"]: record for record in catalog["requirements"]}
    source_clauses = {record["clause_id"]: record for record in catalog["source_clauses"]}
    elements = _records_by_id(catalog, "elements", "element_id")
    profiles = _records_by_id(catalog, "profiles", "profile_id")
    test_cases = _records_by_id(catalog, "test_cases", "test_id")
    decisions = _records_by_id(catalog, "decisions", "decision_id")
    for requirement in catalog["requirements"]:
        _source_refs(requirement.get("source_refs"), sources, "requirement source_refs")
        keyword = requirement.get("source_keyword")
        source_force = requirement.get("source_force")
        source_clause_ids = requirement.get("source_clause_ids")
        if source_force in TABLE_BODY_STRENGTH:
            if not _has_valid_table_body_source(requirement):
                _fail("table-row requirement must use typed force and exact gap/locator metadata without a formal keyword")
        else:
            if not source_clause_ids or any(item not in clauses for item in source_clause_ids):
                _fail("requirement has missing or unresolved source clause references")
            expected_strength = _expected_normative_strength(requirement)
            if (
                not isinstance(keyword, str)
                or keyword not in KEYWORD_STRENGTH
                or requirement.get("normative_strength") != expected_strength
            ):
                _fail("requirement keyword and normative strength disagree")
            if any(
                keyword not in source_clauses[clause_id]["source_keywords"]
                for clause_id in source_clause_ids
            ):
                _fail("requirement keyword is not present in every linked source clause")
        _enum(
            requirement.get("scope_state"),
            {"client_1_0", "client_1_1", "profile_conditional", "server_only", "out_of_scope", "mixed", "unclear"},
            "requirement scope state",
        )
        if keyword in {"MUST NOT", "SHALL NOT"} and requirement.get("negative_verification_required") is not True:
            _fail("prohibited requirements require negative verification")
        if keyword in {"SHOULD", "SHOULD NOT", "RECOMMENDED"} and requirement.get("status") == "deviated":
            decision_id = requirement.get("decision_id")
            if not isinstance(decision_id, str):
                _fail("recommendation deviations require an accepted decision")
            decision = decisions.get(decision_id)
            if decision is None or decision.get("status") != "accepted":
                _fail("recommendation deviations require an accepted decision")
            if requirement["requirement_id"] not in decision.get("requirement_ids", []):
                _fail("decision does not name the deviated requirement")
    for clause in catalog["source_clauses"]:
        for requirement_id in clause["requirement_ids"]:
            if requirement_id not in requirements:
                _fail(
                    f"source clause {clause['clause_id']} has an unresolved requirement reference: "
                    f"{requirement_id}"
                )
        expected_scope = _linked_clause_scope(clause, requirements)
        if expected_scope is not None and clause["scope_state"] != expected_scope:
            _fail("source clause scope does not match linked requirement scopes")

    for element in catalog["elements"]:
        if set(element) - ELEMENT_FIELDS:
            _fail("protocol element has unknown fields")
        if not {"feature_spec", "implementation_refs", "verification_refs"}.issubset(element):
            _fail("protocol element is missing coverage assignment fields")
        _enum(element.get("kind"), ELEMENT_KINDS, "protocol element kind")
        if element["kind"] != "attribute" and ATTRIBUTE_POLICY_FIELDS.intersection(element):
            _fail("attribute policy metadata applies only to attribute records")
        if not isinstance(element.get("name"), str) or not element["name"].strip():
            _fail("protocol element name is required")
        if element["kind"] == "structure_member":
            if not isinstance(element.get("source_encoding"), str) or not element["source_encoding"].strip():
                _fail("structure member source_encoding must be a non-empty string")
            if "source_requiredness" in element and not isinstance(element["source_requiredness"], str):
                _fail("structure member source_requiredness must be a string")
        elif element["kind"] == "message_field":
            for field in ("source_encoding", "source_requiredness", "source_comment"):
                if field in element and not isinstance(element[field], str):
                    _fail(f"message field {field} must be a string")
        elif element["kind"] in {
            "object_structure", "attribute", "operation_structure", "option", "result", "credential",
        }:
            if "source_name" in element and (
                element["kind"] != "attribute"
                or not isinstance(element["source_name"], str)
                or not element["source_name"].strip()
            ):
                _fail("source_name applies only to attributes and must be non-empty")
            if element["kind"] in {"object_structure", "operation_structure", "option", "result", "credential"} and (
                not isinstance(element.get("source_encoding"), str) or not element["source_encoding"].strip()
            ):
                _fail("typed source root requires a non-empty source_encoding")
            if "source_encoding" in element and (
                not isinstance(element["source_encoding"], str) or not element["source_encoding"].strip()
            ):
                _fail("typed element source_encoding must be a non-empty string")
            if "source_requiredness" in element and not isinstance(element["source_requiredness"], str):
                _fail("typed element source_requiredness must be a string")
            if "source_requiredness" in element and "source_encoding" not in element:
                _fail("source_requiredness requires source_encoding")
        elif "source_encoding" in element or "source_requiredness" in element or "source_comment" in element:
            _fail("source structure metadata applies only to typed roots and structure members")
        if "source_comment" in element and element["kind"] != "message_field":
            _fail("source_comment applies only to message fields")
        if "source_name" in element and element["kind"] != "attribute":
            _fail("source_name applies only to attribute records")
        _source_refs(element.get("source_refs"), sources, "element source_refs")
        _enum(element.get("direction"), {"client_to_server", "server_to_client", "both", "not_applicable"}, "protocol element direction")
        _enum(
            element.get("scope_state"),
            {"client_1_0", "client_1_1", "profile_conditional", "server_only", "out_of_scope"},
            "protocol element scope state",
        )
        if element.get("scope_state") != "client_1_0" and not isinstance(element.get("scope_reason"), str):
            _fail("non-default protocol element scope requires a rationale")
        _check_link_ids(element, "parent_element_ids", set(elements), "protocol element")
        _check_link_ids(element, "requirement_ids", set(requirements), "protocol element")
        _check_link_ids(element, "profile_ids", set(profiles), "protocol element")
        _check_link_ids(element, "test_case_ids", set(test_cases), "protocol element")
        if element["feature_spec"] is not None and (
            not isinstance(element["feature_spec"], str) or not element["feature_spec"].strip()
        ):
            _fail("protocol element feature_spec must be a non-empty string or null")
        for field in ("implementation_refs", "verification_refs"):
            if not isinstance(element[field], list) or any(
                not isinstance(reference, str) or not reference.strip() for reference in element[field]
            ):
                _fail(f"protocol element {field} must be an array of non-empty strings")
        if "wire_value" in element and element["wire_value"] is not None and not isinstance(element["wire_value"], str):
            _fail("protocol element wire_value must be a string")
        if element.get("allocation") is not None:
            _enum(
                element["allocation"],
                {"assigned", "reserved", "unused", "extension"},
                "protocol element allocation",
            )
        if element["kind"] == "operation":
            payload_tables = element.get("payload_tables")
            if not isinstance(payload_tables, list):
                _fail("operation payload_tables must be an array")
            for payload in payload_tables:
                if (
                    not isinstance(payload, dict)
                    or set(payload) != {"role", "table_number", "caption"}
                    or payload.get("role") not in {"request", "response"}
                    or not isinstance(payload.get("table_number"), int)
                    or isinstance(payload.get("table_number"), bool)
                    or payload["table_number"] < 1
                    or not isinstance(payload.get("caption"), str)
                    or not payload["caption"].strip()
                ):
                    _fail("operation contains a malformed payload table reference")
            if element.get("asynchronous_response") is not None and not isinstance(
                element["asynchronous_response"], str
            ):
                _fail("operation asynchronous_response must be a string or null")
        elif "payload_tables" in element or "asynchronous_response" in element:
            _fail("payload traceability fields apply only to operation elements")

    _check_attribute_policy_metadata(catalog, repo_root)

    for requirement in catalog["requirements"]:
        _check_link_ids(requirement, "element_ids", set(elements), "requirement")
        _check_link_ids(requirement, "profile_ids", set(profiles), "requirement")
        _check_link_ids(requirement, "test_case_ids", set(test_cases), "requirement")
        if requirement["review_note"] is not None and not isinstance(requirement["review_note"], str):
            _fail("requirement review_note must be a string or null")
        decision_id = requirement.get("decision_id")
        if decision_id is not None and not isinstance(decision_id, str):
            _fail("requirement decision_id must be a string or null")
        if decision_id is not None and decision_id not in decisions:
            _fail("requirement refers to an unresolved decision")

    for profile in catalog["profiles"]:
        if set(profile) != PROFILE_FIELDS:
            _fail("profile has missing or unknown fields")
        if not isinstance(profile["name"], str) or not profile["name"].strip():
            _fail("profile name is required")
        _enum(profile["role"], {"client", "server", "both"}, "profile role")
        _source_refs(profile["source_refs"], sources, "profile source_refs")
        for field in (
            "direct_source_clause_ids", "source_clause_ids", "transport_requirements", "encoding_requirements"
        ):
            _string_values(profile, field, "profile")
        if any(clause_id not in clauses for clause_id in profile["direct_source_clause_ids"]):
            _fail("profile has unresolved direct source clause references")
        if any(clause_id not in clauses for clause_id in profile["source_clause_ids"]):
            _fail("profile has unresolved source clause references")
        for field, valid_ids in (
            ("dependency_profile_ids", set(profiles)),
            ("requirement_ids", set(requirements)),
            ("element_ids", set(elements)),
            ("test_case_ids", set(test_cases)),
        ):
            _check_link_ids(profile, field, valid_ids, "profile")
        _enum(
            profile["applicability"],
            {"client_1_0", "client_1_1", "server_only", "conditional", "out_of_scope"},
            "profile applicability",
        )
        _enum(
            profile["claim_state"],
            {"not_claimed", "candidate", "selected", "evidence_incomplete", "evidence_complete"},
            "profile claim state",
        )
        if profile["claim_state"] == "evidence_complete":
            evidence_links = (
                profile["source_clause_ids"],
                profile["requirement_ids"],
                profile["element_ids"],
                profile["test_case_ids"],
            )
            if any(not linked_ids for linked_ids in evidence_links):
                _fail("profile evidence is incomplete without clauses, requirements, elements, and tests")
            dependency_profiles: list[dict[str, Any]] = []
            pending_profile_ids = [(profile["profile_id"], False)]
            visited_profile_ids: set[str] = set()
            active_profile_ids: set[str] = set()
            while pending_profile_ids:
                dependency_profile_id, exiting = pending_profile_ids.pop()
                if exiting:
                    active_profile_ids.remove(dependency_profile_id)
                    continue
                if dependency_profile_id in active_profile_ids:
                    _fail("profile evidence is incomplete with a dependency cycle")
                if dependency_profile_id in visited_profile_ids:
                    continue
                visited_profile_ids.add(dependency_profile_id)
                dependency_profile = profiles.get(dependency_profile_id)
                if dependency_profile is None:
                    _fail("profile evidence is incomplete with an unresolved dependency profile")
                active_profile_ids.add(dependency_profile_id)
                dependency_profiles.append(dependency_profile)
                pending_profile_ids.append((dependency_profile_id, True))
                pending_profile_ids.extend(
                    (nested_dependency_id, False)
                    for nested_dependency_id in reversed(dependency_profile["dependency_profile_ids"])
                )

            dependency_requirement_ids = {
                requirement_id
                for dependency_profile in dependency_profiles
                for requirement_id in dependency_profile["requirement_ids"]
            }
            if any(
                dependency_profile["profile_id"] != profile["profile_id"]
                and dependency_profile["claim_state"] != "evidence_complete"
                for dependency_profile in dependency_profiles
            ):
                _fail("profile evidence is incomplete because dependency profile evidence is incomplete")

            catalog_linked_test_ids = {
                test_case["test_id"]
                for test_case in catalog["test_cases"]
                if profile["profile_id"] in test_case["profile_ids"]
            }
            if not catalog_linked_test_ids.issubset(profile["test_case_ids"]):
                _fail("profile evidence is incomplete with missing catalog-linked test cases")
            if not set(profile["test_case_ids"]).issubset(catalog_linked_test_ids):
                _fail("profile evidence is incomplete with unlinked profile test cases")

            linked_test_ids = {
                test_id
                for dependency_profile in dependency_profiles
                for test_id in dependency_profile["test_case_ids"]
            }
            for requirement_id in dependency_requirement_ids:
                linked_test_ids.update(requirements[requirement_id]["test_case_ids"])
            profile_test_cases = [
                test_cases[test_id]
                for test_id in linked_test_ids
                if test_id in test_cases
            ]
            if not any(test_case["source_id"] == "KMIPKIT-SRC-testcases" for test_case in profile_test_cases):
                _fail("profile evidence is incomplete without an official KMIP Test Case")
            if any(
                test_case["mandatory_status"] == "mandatory"
                and test_case["fixture_availability"] != "available"
                for test_case in profile_test_cases
            ):
                _fail("profile evidence is incomplete with an unavailable mandatory fixture")
            profile_requirements = [requirements[requirement_id] for requirement_id in dependency_requirement_ids]
            if any(
                not requirement["implementation_refs"] or not requirement["verification_refs"]
                for requirement in profile_requirements
            ):
                _fail("profile evidence is incomplete without requirement implementation and verification assignments")
        _string_values(profile, "direct_source_clause_ids", "profile", non_empty=True)
        _string_values(profile, "source_clause_ids", "profile", non_empty=True)
        if profile["role"] in {"client", "both"}:
            try:
                closure = compute_profile_closure(
                    catalog,
                    profile["direct_source_clause_ids"],
                    profile_id=profile["profile_id"],
                )
            except ProfileGenerationError as error:
                raise CatalogValidationError("profile normative closure is invalid") from error
            if profile["source_clause_ids"] != list(closure.source_clause_ids):
                _fail_profile_closure_mismatch(
                    "profile clause",
                    profile["source_clause_ids"],
                    list(closure.source_clause_ids),
                )
            if profile["requirement_ids"] != list(closure.requirement_ids):
                _fail_profile_closure_mismatch(
                    "profile requirement",
                    profile["requirement_ids"],
                    list(closure.requirement_ids),
                )

    for test_case in catalog["test_cases"]:
        if not isinstance(test_case, dict) or set(test_case) != TEST_CASE_FIELDS:
            _fail("test case has missing or unknown fields")
        if not isinstance(test_case["official_case_id"], str) or not test_case["official_case_id"].strip():
            _fail("test case must retain its official case ID")
        _enum(test_case["source_id"], {"KMIPKIT-SRC-testcases", "KMIPKIT-SRC-profiles"}, "test case source")
        if not re.fullmatch(r"[0-9]+(?:\.[0-9]+)*", test_case["source_section"]):
            _fail("test case has an invalid source section")
        _enum(test_case["mandatory_status"], {"mandatory", "optional", "unspecified"}, "test case mandatory status")
        _enum(test_case["fixture_availability"], {"available", "unavailable"}, "test case fixture availability")
        _enum(test_case["mapping_confidence"], {"explicit", "strong", "weak", "unmapped"}, "test case mapping confidence")
        for field, valid_ids, name in (
            ("profile_ids", set(profiles), "test case"),
            ("requirement_ids", set(requirements), "test case"),
            ("element_ids", set(elements), "test case"),
        ):
            _check_link_ids(test_case, field, valid_ids, name)
        raw_href = test_case["raw_href"]
        if raw_href is not None and not isinstance(raw_href, str):
            _fail("test case raw_href must be a string or null")
        fixture_path = test_case["fixture_path"]
        if fixture_path is None:
            if test_case["fixture_availability"] != "unavailable":
                _fail("test case without a local fixture path must be unavailable")
            continue
        if not isinstance(fixture_path, str) or "\\" in fixture_path:
            _fail("fixture path must be a repository-relative POSIX path")
        parts = fixture_path.split("/")
        if (
            fixture_path.startswith("/")
            or re.match(r"^[A-Za-z]:", fixture_path)
            or not fixture_path.startswith("specification/oasis/kmip-2.1/")
            or any(part in {"", ".", ".."} for part in parts)
        ):
            _fail("fixture path escapes the pinned OASIS directory")
        metadata = tree.get(fixture_path)
        if metadata is None:
            _fail("fixture path is absent from the pinned Git tree")
        mode, object_type = metadata
        if mode == "120000" or object_type != "blob":
            _fail("fixture path resolves to a symlink or non-file entry")
        if test_case["fixture_availability"] != "available":
            _fail("fixture path and availability status disagree")

    _check_requirement_test_evidence(catalog["requirements"], catalog["test_cases"])

    for tag_range in catalog["tag_ranges"]:
        _enum(tag_range.get("allocation"), {"unused", "reserved", "extension"}, "tag range allocation")
        _source_refs(tag_range.get("source_refs"), sources, "tag range source_refs")

    for policy in catalog["policies"]:
        if set(policy) != POLICY_FIELDS:
            _fail("project policy has missing or unknown fields")
        _enum(
            policy.get("provenance"),
            {"AGENTS.md", "constitution", "ADR", "approved_product_decision"},
            "project policy provenance",
        )
        if not isinstance(policy.get("summary"), str) or not policy["summary"].strip():
            _fail("project policy summary is required")
        _string_values(policy, "affected_element_kinds", "project policy", non_empty=True)
        if any(kind not in ELEMENT_KINDS for kind in policy["affected_element_kinds"]):
            _fail("project policy has an unknown affected element kind")
        _check_link_ids(policy, "requirement_ids", set(requirements), "project policy")
        _check_policy_provenance(policy, decisions, repo_root)

    for decision in catalog["decisions"]:
        if set(decision) != DECISION_FIELDS:
            _fail("decision has missing or unknown fields")
        _source_refs(decision["source_refs"], sources, "decision source_refs")
        for field, valid_ids in (
            ("requirement_ids", set(requirements)),
            ("discrepancy_ids", {row["discrepancy_id"] for row in catalog["discrepancies"]}),
            ("policy_ids", {row["policy_id"] for row in catalog["policies"]}),
        ):
            _check_link_ids(decision, field, valid_ids, "decision")
        for field in ("interpretation", "approver", "approval_evidence", "consequence"):
            if not isinstance(decision[field], str) or not decision[field].strip():
                _fail("decision requires interpretation, approver evidence, and consequence")
        if not isinstance(decision["approved_at"], str) or re.fullmatch(
            r"[0-9]{4}-[0-9]{2}-[0-9]{2}", decision["approved_at"]
        ) is None:
            _fail("decision approved_at must be an ISO date")
        if not isinstance(decision["status"], str) or decision["status"] != "accepted":
            _fail("decision records must contain accepted decisions only")

    authority_by_source = {
        row["source_id"]: row["authority_class"] for row in catalog["sources"]
    }
    for discrepancy in catalog["discrepancies"]:
        if set(discrepancy) != DISCREPANCY_FIELDS:
            _fail("source discrepancy has missing or unknown fields")
        if not isinstance(discrepancy["summary"], str) or not discrepancy["summary"].strip():
            _fail("source discrepancy summary is required")
        if not isinstance(discrepancy["downstream_impact"], str) or not discrepancy["downstream_impact"].strip():
            _fail("source discrepancy downstream impact is required")
        _source_refs(discrepancy["source_refs"], sources, "source discrepancy source_refs")
        if discrepancy["erratum_source_refs"]:
            _source_refs(discrepancy["erratum_source_refs"], sources, "source discrepancy erratum_source_refs")
        expected_authority = _strongest_source_authority(discrepancy["source_refs"], authority_by_source)
        if discrepancy["source_authority"] != expected_authority:
            _fail("source authority does not match the cited discrepancy sources")
        _enum(
            discrepancy["source_authority"],
            {"primary_normative", "profile_normative", "test_evidence", "informative"},
            "source discrepancy authority",
        )
        _enum(
            discrepancy["normative_status"],
            {"normative", "informative", "conditional", "normative_conflict", "source_defect", "evidence_gap"},
            "source discrepancy normative status",
        )
        _string_values(discrepancy, "alternatives", "source discrepancy", non_empty=True)
        decision_id = discrepancy["decision_id"]
        if decision_id is not None and not isinstance(decision_id, str):
            _fail("source discrepancy decision_id must be a string or null")
        if decision_id is not None and decision_id not in decisions:
            _fail("source discrepancy refers to an unresolved decision")
        _enum(
            discrepancy["state"],
            {"open", "resolved_by_erratum", "resolved_by_approved_decision"},
            "source discrepancy state",
        )
        if discrepancy["state"] == "open" and decision_id is not None:
            _fail("open discrepancy cannot select a decision")
        if discrepancy["state"] == "resolved_by_approved_decision" and (
            decision_id is None or decisions[decision_id]["status"] != "accepted"
        ):
            _fail("resolved discrepancy requires an accepted decision")
        if discrepancy["state"] == "resolved_by_approved_decision" and discrepancy["discrepancy_id"] not in decisions[
            decision_id
        ].get("discrepancy_ids", []):
            _fail("decision does not name the resolved discrepancy")
        if discrepancy["state"] == "resolved_by_erratum" and decision_id is not None:
            _fail("erratum-resolved discrepancy cannot select a project decision")
        if discrepancy["state"] == "resolved_by_erratum":
            erratum_ids = {reference["source_id"] for reference in discrepancy["erratum_source_refs"]}
            erratum_sources = {
                row["source_id"]
                for row in catalog["sources"]
                if re.search(r"errat(?:um|a)|corrigendum", row["title"], re.IGNORECASE)
            }
            if not erratum_ids or not erratum_ids.issubset(erratum_sources):
                _fail("erratum resolution requires an erratum source in the pinned manifest")
        elif discrepancy["erratum_source_refs"]:
            _fail("erratum source references apply only to erratum-resolved discrepancies")
        for field, valid_ids in (
            ("affected_requirement_ids", set(requirements)),
            ("affected_element_ids", set(elements)),
            ("affected_profile_ids", set(profiles)),
            ("affected_policy_ids", {row["policy_id"] for row in catalog["policies"]}),
        ):
            _check_link_ids(discrepancy, field, valid_ids, "source discrepancy")


def _validate_catalog_header(catalog: Any) -> dict[str, Any]:
    if not isinstance(catalog, dict):
        _fail("catalog root must be an object")
    unknown = set(catalog) - TOP_LEVEL_FIELDS
    missing = TOP_LEVEL_FIELDS - set(catalog)
    if unknown:
        _fail("catalog has unknown top-level fields")
    if missing:
        _fail("catalog is missing required top-level fields")
    if catalog["schema_version"] != 1:
        _fail("unsupported catalog schema version")
    for field in TOP_LEVEL_FIELDS - {"schema_version"}:
        if not isinstance(catalog[field], list):
            _fail(f"{field} must be an array")
        if len(catalog[field]) > MAX_RECORDS:
            _fail(f"{field} exceeds the 100,000 record limit")
    return catalog


CLIENT_OPERATION_SECTIONS = {
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
SERVER_OPERATION_SECTIONS = {
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
ASYNC_RESPONSE_BEHAVIORS = {
    "Cancel": "cancellation_result_not_async",
    "Poll": "pending_or_original_operation_payload",
}


def _expected_operation_payload_tables(
    name: str,
    direction: str,
) -> list[dict[str, Any]]:
    """Build the exact payload-table references verified against pinned captions."""
    if direction == "client_to_server":
        operation_names = tuple(CLIENT_OPERATION_SECTIONS)
        table_pair = CLIENT_OPERATION_PAYLOAD_TABLES[operation_names.index(name)]
    else:
        table_pair = SERVER_OPERATION_PAYLOAD_TABLES[name]

    records: list[dict[str, Any]] = []
    for role, table_number in (("request", table_pair[0]), ("response", table_pair[1])):
        if table_number is None:
            continue
        caption = f"{name} {role.title()} Payload"
        if name == "Query Asynchronous Requests" and role == "response":
            caption = "PKCS#11 Response Payload"
        records.append({"role": role, "table_number": table_number, "caption": caption})
    return records


def _check_operation_inventory(elements: list[dict[str, Any]]) -> None:
    expected = {
        **{("client_to_server", name): section for name, section in CLIENT_OPERATION_SECTIONS.items()},
        **{("server_to_client", name): section for name, section in SERVER_OPERATION_SECTIONS.items()},
    }
    operations = [row for row in elements if row.get("kind") == "operation"]
    observed: dict[tuple[Any, Any], dict[str, Any]] = {}
    for operation in operations:
        key = (operation.get("direction"), operation.get("name"))
        if key in observed:
            _fail("operation inventory contains a duplicate operation direction/name pair")
        observed[key] = operation
    if set(observed) != set(expected):
        _fail("operation inventory does not match all 57 client and 5 server definitions")
    for key, section in expected.items():
        operation = observed[key]
        if operation.get("source_refs") != [{"source_id": "KMIPKIT-SRC-spec", "section": section}]:
            _fail(f"operation {operation.get('name')} has an incorrect source section")
        deferred_client_operations = {("client_to_server", "Set Endpoint Role")}
        if key in deferred_client_operations:
            required_scope = "client_1_1"
        else:
            required_scope = "client_1_0" if key[0] == "client_to_server" else "client_1_1"
        if operation.get("scope_state") != required_scope:
            _fail(f"operation {operation.get('name')} has an incorrect scope disposition")
        if required_scope != "client_1_0" and not (
            isinstance(operation.get("scope_reason"), str) and operation["scope_reason"].strip()
        ):
            _fail(f"operation {operation.get('name')} requires a non-empty 1.1 scope rationale")
        name = key[1]
        expected_tables = _expected_operation_payload_tables(name, key[0])
        if operation.get("payload_tables") != expected_tables:
            _fail(f"operation {name} has incorrect payload table references")
        if operation.get("asynchronous_response") != ASYNC_RESPONSE_BEHAVIORS.get(name):
            _fail(f"operation {name} has incorrect asynchronous response classification")
    endpoint_role_enum_id = "KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ENDPOINT-ROLE-00000032"
    endpoint_role_enum = next((row for row in elements if row.get("element_id") == endpoint_role_enum_id), None)
    if endpoint_role_enum is None:
        _fail("operation inventory is missing the Set Endpoint Role enumeration value")
    if (
        endpoint_role_enum.get("scope_state") != "client_1_1"
        or not isinstance(endpoint_role_enum.get("scope_reason"), str)
        or not endpoint_role_enum["scope_reason"].strip()
    ):
        _fail("Set Endpoint Role operation enumeration value has incorrect scope or lacks a 1.1 rationale")


COMPLETE_ELEMENT_COUNTS = {
    "data_type": 11,
    "object_type": 9,
    "object_structure": 23,
    "attribute": 63,
    "attribute_structure": 7,
    "operation_structure": 41,
    "enumeration": 64,
    "enumeration_value": 723,
    "bitmask": 3,
    "bitmask_value": 44,
    "option": 3,
    "result": 4,
    "credential": 7,
    "tag": 374,
    "message_field": 71,
    "structure_member": 241,
}
TAG_REGISTRY_SHA256 = "ad69b23437d238ae67bfd72e54ba37a8bbe9fd31358f1aadfd41c410369cf8fe"
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


def _check_tag_registry(
    elements: list[dict[str, Any]],
    tag_ranges: list[dict[str, Any]],
) -> None:
    """Reconcile single-value Tag Enumeration rows and separate ranges."""
    tags = [record for record in elements if record.get("kind") == "tag"]
    if len(tags) != 374:
        _fail(f"tag registry requires 374 single-value rows; found {len(tags)}")

    observed_values: set[str] = set()
    observed_reserved: dict[str, str] = {}
    assigned_count = 0
    for tag in tags:
        value = tag.get("wire_value")
        if not isinstance(value, str):
            _fail("tag registry contains a missing wire value")
        normalized = value[2:].upper() if value.lower().startswith("0x") else value.upper()
        if re.fullmatch(r"[0-9A-F]{6}", normalized) is None:
            _fail("tag registry contains a non-singleton wire value")
        if normalized in observed_values:
            _fail("tag registry contains a duplicate single-value tag")
        observed_values.add(normalized)
        if tag.get("element_id") != f"KMIPKIT-ELEM-TAG-{normalized}":
            _fail("tag registry element ID does not match its wire value")
        if tag.get("source_refs") != [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}]:
            _fail("tag registry row has an incorrect source reference")
        allocation = tag.get("allocation")
        if allocation == "assigned":
            assigned_count += 1
            if normalized in RESERVED_TAGS:
                _fail("reserved tag is incorrectly classified as assigned")
        elif allocation == "reserved":
            observed_reserved[normalized] = tag.get("name")
        else:
            _fail("tag registry singleton must be assigned or reserved")

    if assigned_count != 354 or observed_reserved != RESERVED_TAGS:
        _fail("tag registry does not match the 354 assigned and exact reserved tag rows")
    if len(tag_ranges) != len(EXPECTED_TAG_RANGES):
        _fail("tag registry requires five separate range rows")
    for index, (tag_range, expected) in enumerate(zip(tag_ranges, EXPECTED_TAG_RANGES, strict=True), start=1):
        allocation, value_range = expected
        if (
            tag_range.get("range_id") != f"KMIPKIT-RANGE-{index:03}"
            or tag_range.get("source_order") != index
            or tag_range.get("allocation") != allocation
            or tag_range.get("value_range") != value_range
            or tag_range.get("source_refs") != [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}]
        ):
            _fail("tag registry range does not match its exact source row")


def _check_tag_registry_fingerprint(tags: list[dict[str, Any]]) -> None:
    """Bind all singleton names, values, and allocations to pinned Table 487."""
    rows = sorted(
        ((tag.get("name"), tag.get("wire_value"), tag.get("allocation")) for tag in tags),
        key=lambda row: str(row[1]),
    )
    encoded = json.dumps(rows, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    if hashlib.sha256(encoded).hexdigest() != TAG_REGISTRY_SHA256:
        _fail("tag registry source fingerprint mismatch")


def _check_complete_inventory(catalog: dict[str, Any]) -> None:
    elements = catalog["elements"]
    _check_operation_inventory(elements)

    for kind, expected in COMPLETE_ELEMENT_COUNTS.items():
        actual = sum(row.get("kind") == kind for row in elements)
        if actual != expected:
            _fail(f"complete inventory requires {expected} {kind} records; found {actual}")
    _check_tag_registry(elements, catalog["tag_ranges"])
    _check_tag_registry_fingerprint([row for row in elements if row.get("kind") == "tag"])

    required_collections = ("source_clauses", "requirements", "profiles", "test_cases", "policies")
    if any(not catalog[name] for name in required_collections):
        _fail("complete inventory is missing clauses, requirements, profiles, test cases, or policies")
    source_test_count = sum(row.get("source_id") == "KMIPKIT-SRC-testcases" for row in catalog["test_cases"])
    profile_fixture_count = sum(row.get("source_id") == "KMIPKIT-SRC-profiles" for row in catalog["test_cases"])
    if source_test_count != 110 or profile_fixture_count != 93:
        _fail("complete inventory requires 110 Test Cases and 93 profile fixture references")


def load_validated_catalog(raw: bytes, repo_root: Path, *, require_complete: bool = False) -> dict[str, Any]:
    """Validate raw UTF-8 JSON bytes and return the validated catalog records."""
    text = _preflight(raw)
    try:
        catalog = json.loads(text, object_pairs_hook=_reject_duplicate_pairs)
    except CatalogValidationError:
        raise
    except (json.JSONDecodeError, RecursionError) as error:
        raise CatalogValidationError("catalog is not valid bounded JSON") from error
    _reject_surrogates(catalog)
    catalog = _validate_catalog_header(catalog)

    tree = _git_tree(repo_root)
    _reject_reparse_points(repo_root, tree)
    sources = _check_source_records(catalog, repo_root, tree)
    clauses = _check_clauses(catalog, sources)
    _check_identifiers(catalog)
    _check_normative_inclusions(catalog, sources, clauses)
    _check_semantics(catalog, sources, clauses, tree, repo_root)
    if require_complete:
        _check_complete_inventory(catalog)

    for source in catalog["sources"]:
        try:
            raw = safe_read_bytes(repo_root, source["local_path"], max_bytes=MAX_BYTES)
            digest = hashlib.sha256(raw).hexdigest()
        except (OSError, PathSecurityError) as error:
            raise CatalogValidationError("pinned OASIS source file is unavailable") from error
        if digest != source["sha256"]:
            _fail(f"pinned source checksum mismatch for {source['source_id']}")

    return catalog


def _profile_oracle_feature_spec_requirement_ids(root: Path, feature_spec_id: str) -> set[str] | None:
    """Return declared FR IDs for the feature spec linked by an operation."""
    specification_dir = root / "specs"
    for specification_path in specification_dir.glob("*/spec.md"):
        try:
            specification = specification_path.read_text(encoding="utf-8")
        except (OSError, UnicodeError):
            continue
        branch = re.search(
            r"(?m)^\*\*Feature Branch\*\*:\s*`?feature/([^`\s]+)",
            specification,
        )
        if branch is None or not branch.group(1).startswith(f"{feature_spec_id}-"):
            continue
        fr_id_pattern = r"FR-\d{3}"
        declared_ids = set(re.findall(rf"(?m)^\s*-\s+\*\*({fr_id_pattern})\*\*:", specification))
        qualified_ids = set(
            re.findall(
                rf"(?m)^\s*-\s+\*\*{re.escape(feature_spec_id)}-({fr_id_pattern})\*\*:",
                specification,
            )
        )
        return declared_ids | qualified_ids
    return None


def _profile_oracle_reference_issue(
    root: Path,
    reference: object,
    *,
    executable_test: bool,
) -> str | None:
    """Return a diagnostic when a profile-oracle path::symbol reference is unresolved."""
    if not isinstance(reference, str) or "::" not in reference:
        return "reference must use path::symbol form"
    relative_path, symbol_path = reference.split("::", 1)
    symbol = symbol_path.rsplit("::", 1)[-1]
    if not relative_path or not symbol or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", symbol):
        return "reference must name a source file and symbol"
    if executable_test and "tests" not in Path(relative_path).parts:
        return "verification reference must point into a tests directory"
    try:
        resolved = (root / relative_path).resolve(strict=True)
        resolved.relative_to(root)
        text = resolved.read_text(encoding="utf-8")
    except (OSError, ValueError, UnicodeError):
        return "reference path must resolve to a repository file"
    if executable_test:
        pattern = rf"#\[test\]\s*fn\s+{re.escape(symbol)}\b|\bdef\s+{re.escape(symbol)}\b"
    else:
        pattern = rf"\b(?:pub\s+)?fn\s+{re.escape(symbol)}\b|\bdef\s+{re.escape(symbol)}\b"
    if re.search(pattern, text) is None:
        return "reference symbol is not defined in the cited file"
    return None


def _index_source_request_rows_by_gap(
    source_reviews: object,
    diagnostics: list[str],
) -> dict[str, list[dict[str, Any]]]:
    """Index reviewed request rows that have an explicit source-gap link."""
    rows_by_gap: dict[str, list[dict[str, Any]]] = {}
    if not isinstance(source_reviews, list):
        diagnostics.append("source_requirement_reviews must be an array")
        return rows_by_gap

    for review_index, review in enumerate(source_reviews):
        review_label = f"source_requirement_reviews[{review_index}]"
        if not isinstance(review, dict):
            diagnostics.append(f"{review_label} must be an object")
            continue
        request_rows = review.get("source_request_rows", [])
        if not isinstance(request_rows, list):
            diagnostics.append(f"{review_label}.source_request_rows must be an array")
            continue
        for row_index, row in enumerate(request_rows):
            row_label = f"{review_label}.source_request_rows[{row_index}]"
            if not isinstance(row, dict):
                diagnostics.append(f"{row_label} must be an object")
                continue
            source_gap_id = row.get("source_gap_id")
            if source_gap_id is None:
                continue
            if not isinstance(source_gap_id, str) or not source_gap_id.strip():
                diagnostics.append(f"{row_label}.source_gap_id must be a non-empty stable ID")
                continue
            rows_by_gap.setdefault(source_gap_id, []).append(row)
    return rows_by_gap


def _validate_source_request_row_semantics(
    contract: dict[str, Any],
    label: str,
    rows_by_gap: dict[str, list[dict[str, Any]]],
    diagnostics: list[str],
) -> None:
    """Require each linked contract to agree with its reviewed row semantics."""
    gap_id = contract.get("gap_id")
    if not isinstance(gap_id, str) or not gap_id.strip():
        return

    reviewed_rows = rows_by_gap.get(gap_id, [])
    if not reviewed_rows:
        diagnostics.append(
            f"{label}.requiredness and .actor must resolve a reviewed source_request_rows entry by source_gap_id"
        )
        return

    row_semantics: list[tuple[str, str]] = []
    malformed_semantics = False
    for row in reviewed_rows:
        requiredness = row.get("required")
        actor = row.get("actor")
        if (
            not isinstance(requiredness, str)
            or not requiredness.strip()
            or not isinstance(actor, str)
            or not actor.strip()
        ):
            malformed_semantics = True
            continue
        row_semantics.append((requiredness, actor))
    if malformed_semantics or len(row_semantics) != len(reviewed_rows):
        diagnostics.append(
            f"{label}.requiredness and .actor cannot resolve malformed reviewed source_request_rows"
        )
    elif len(set(row_semantics)) != 1:
        diagnostics.append(
            f"{label}.requiredness and .actor are ambiguous across reviewed source_request_rows for {gap_id}"
        )
    else:
        reviewed_requiredness, reviewed_actor = row_semantics[0]
        if contract.get("requiredness") != reviewed_requiredness:
            diagnostics.append(
                f"{label}.requiredness must match reviewed source_request_rows.required"
            )
        if contract.get("actor") != reviewed_actor:
            diagnostics.append(f"{label}.actor must match reviewed source_request_rows.actor")


def validate_source_table_contracts(oracle: object, repo_root: Path) -> list[str]:
    """Validate source-table contract evidence in the independent profile oracle.

    This test-support validator keeps operation-table rows distinct from the
    normative requirement catalog while requiring resolvable source, feature,
    implementation, and executable-test references.
    """
    diagnostics: list[str] = []
    if not isinstance(oracle, dict):
        return ["profile closure oracle must be an object"]

    contracts = oracle.get("source_table_contracts")
    if not isinstance(contracts, list):
        return ["source_table_contracts must be an array"]

    gaps = oracle.get("source_only_requirement_gaps", [])
    source_gap_ids = {
        row.get("gap_id") for row in gaps if isinstance(row, dict)
    } if isinstance(gaps, list) else set()
    source_request_rows_by_gap = _index_source_request_rows_by_gap(
        oracle.get("source_requirement_reviews", []),
        diagnostics,
    )
    seen_ids: set[str] = set()
    required_fields = (
        "gap_id", "contract_kind", "source_id", "source_path", "source_section", "source_locator",
        "source_locator_description", "html_line_range", "source_excerpt", "source_force",
        "requiredness", "actor", "direction", "condition", "scope_disposition",
        "table_number", "operation_element_id", "functional_requirement_ids",
        "implementation_refs", "verification_refs", "evidence_disposition",
    )

    try:
        root = repo_root.resolve(strict=True)
    except OSError:
        return ["repository root is unavailable"]

    catalog_path = root / "specification" / "catalog" / "kmip-2.1.json"
    try:
        catalog_path.resolve(strict=True).relative_to(root)
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    except (OSError, ValueError, UnicodeError, json.JSONDecodeError):
        return ["KMIP 2.1 catalog is unavailable for operation contract validation"]
    elements = catalog.get("elements", []) if isinstance(catalog, dict) else []
    operation_elements = {
        element.get("element_id"): element
        for element in elements
        if isinstance(element, dict) and isinstance(element.get("element_id"), str)
    }
    feature_requirement_cache: dict[str, set[str] | None] = {}

    for index, contract in enumerate(contracts):
        label = f"source_table_contracts[{index}]"
        if not isinstance(contract, dict):
            diagnostics.append(f"{label} must be an object")
            continue
        for field in required_fields:
            if field not in contract:
                diagnostics.append(f"{label}.{field} is required")

        gap_id = contract.get("gap_id")
        if not isinstance(gap_id, str) or not gap_id.strip():
            diagnostics.append(f"{label}.gap_id must be a non-empty stable ID")
        else:
            if gap_id in seen_ids:
                diagnostics.append(f"{label}.gap_id must be unique")
            seen_ids.add(gap_id)
            if gap_id in source_gap_ids:
                diagnostics.append(f"{label}.gap_id must not also be a source-only requirement gap")

        for identifier_field in ("candidate_requirement_id", "proposed_requirement_id", "requirement_id"):
            if identifier_field in contract:
                diagnostics.append(f"{label}.{identifier_field} must not create a per-cell REQ mapping")

        locator = contract.get("source_locator")
        if not isinstance(locator, dict):
            diagnostics.append(f"{label}.source_locator must identify the pinned table row")
        elif (
            locator.get("kind") != "table_body_row"
            or locator.get("section") != contract.get("source_section")
            or locator.get("table") != "request_payload"
            or not isinstance(locator.get("row_label"), str)
            or locator.get("html_line_range") != contract.get("html_line_range")
        ):
            diagnostics.append(f"{label}.source_locator must resolve the exact table body row")
        table_number = contract.get("table_number")
        if not isinstance(table_number, int) or isinstance(table_number, bool) or table_number < 1:
            diagnostics.append(f"{label}.table_number must be a positive integer")

        operation_id = contract.get("operation_element_id")
        operation = operation_elements.get(operation_id) if isinstance(operation_id, str) else None
        operation_tables = operation.get("payload_tables", []) if isinstance(operation, dict) else []
        if (
            not isinstance(operation, dict)
            or operation.get("kind") != "operation"
            or operation.get("direction") != "client_to_server"
            or not isinstance(operation_tables, list)
            or not any(
                isinstance(table, dict)
                and table.get("role") == "request"
                and table.get("table_number") == table_number
                for table in operation_tables
            )
        ):
            diagnostics.append(
                f"{label}.operation_element_id must reference a client-to-server operation with the request table"
            )
        if isinstance(operation, dict):
            operation_scope = operation.get("scope_state")
            contract_scope = contract.get("scope_disposition")
            if not isinstance(operation_scope, str) or not operation_scope.strip():
                diagnostics.append(
                    f"{label}.scope_disposition cannot resolve the linked operation scope_state"
                )
            elif contract_scope == "profile_conditional":
                if operation_scope not in {"client_1_0", "client_1_1"}:
                    diagnostics.append(
                        f"{label}.scope_disposition profile_conditional requires an in-scope linked client operation"
                    )
                condition = contract.get("condition")
                if not isinstance(condition, str) or not condition.strip():
                    diagnostics.append(
                        f"{label}.scope_disposition profile_conditional requires an explicit condition"
                    )
            elif contract_scope != operation_scope:
                diagnostics.append(
                    f"{label}.scope_disposition must match linked operation scope_state"
                )

        source_path = contract.get("source_path")
        if not isinstance(source_path, str):
            diagnostics.append(f"{label}.source_path must name the pinned source file")
        else:
            try:
                source_file = (root / source_path).resolve(strict=True)
                source_file.relative_to(root)
                if not source_file.is_file():
                    raise OSError("not a file")
            except (OSError, ValueError):
                diagnostics.append(f"{label}.source_path must resolve inside the repository")

        for field in ("source_id", "source_section", "source_locator_description", "source_excerpt", "source_force", "requiredness", "actor", "direction", "condition", "scope_disposition", "operation_element_id"):
            if not isinstance(contract.get(field), str) or not contract[field].strip():
                diagnostics.append(f"{label}.{field} must be a non-empty string")
        _validate_source_request_row_semantics(
            contract,
            label,
            source_request_rows_by_gap,
            diagnostics,
        )
        if contract.get("contract_kind") != "operation_request_table_field":
            diagnostics.append(f"{label}.contract_kind must identify an operation request table field")
        functional_requirements = contract.get("functional_requirement_ids")
        if (
            not isinstance(functional_requirements, list)
            or not functional_requirements
            or any(
                not isinstance(value, str) or re.fullmatch(r"FR-\d{3}", value) is None
                for value in functional_requirements
            )
        ):
            diagnostics.append(f"{label}.functional_requirement_ids must contain feature requirement IDs")
        elif isinstance(operation, dict):
            feature_spec_id = operation.get("feature_spec")
            if not isinstance(feature_spec_id, str) or not feature_spec_id.strip():
                diagnostics.append(f"{label}.functional_requirement_ids cannot resolve the operation feature spec")
            else:
                if feature_spec_id not in feature_requirement_cache:
                    feature_requirement_cache[feature_spec_id] = _profile_oracle_feature_spec_requirement_ids(
                        root,
                        feature_spec_id,
                    )
                approved_requirements = feature_requirement_cache[feature_spec_id]
                if approved_requirements is None:
                    diagnostics.append(f"{label}.functional_requirement_ids cannot resolve the approved feature spec")
                elif not set(functional_requirements).issubset(approved_requirements):
                    diagnostics.append(f"{label}.functional_requirement_ids must reference declared feature requirements")

        for field, executable_test in (("implementation_refs", False), ("verification_refs", True)):
            references = contract.get(field)
            if not isinstance(references, list) or not references:
                diagnostics.append(f"{label}.{field} must contain a resolved path::symbol reference")
                continue
            for reference in references:
                issue = _profile_oracle_reference_issue(
                    root,
                    reference,
                    executable_test=executable_test,
                )
                if issue is not None:
                    diagnostics.append(f"{label}.{field}: {issue}")

        disposition = contract.get("evidence_disposition")
        if not isinstance(disposition, dict) or not isinstance(disposition.get("status"), str):
            diagnostics.append(f"{label}.evidence_disposition must record an explicit status")
        elif locator and locator.get("row_label") == "Data" and table_number in _SHARED_DATA_RULE_CONFLICT_TABLES:
            claim = disposition.get("claim")
            normalized_claim = claim.casefold() if isinstance(claim, str) else ""
            if (
                disposition.get("status") != "resolved_by_project_interpretation"
                or disposition.get("decision_id") != "KMIPKIT-DEC-034"
                or disposition.get("conflict_source_gap_id") != "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"
                or "operation-table data=yes controls single-part" not in normalized_claim
                or "neither indicator is true" not in normalized_claim
                or "permits omission when either is true" not in normalized_claim
                or "no standalone paragraph-009 requirement id" not in normalized_claim
            ):
                diagnostics.append(f"{label}.evidence_disposition must cite DEC-015 and preserve its single-part and multipart Data matrix")

    return diagnostics


def validate_catalog(raw: bytes, repo_root: Path, *, require_complete: bool = False) -> dict[str, Any]:
    """Validate raw UTF-8 JSON bytes and return deterministic aggregate counts."""
    catalog = load_validated_catalog(raw, repo_root, require_complete=require_complete)
    return {
        "source_count": len(catalog["sources"]),
        "clause_count": len(catalog["source_clauses"]),
        "record_count": sum(len(catalog[key]) for key in TOP_LEVEL_FIELDS - {"schema_version"}),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument(
        "--structural-only",
        action="store_true",
        help="validate structure without requiring the complete KMIP 2.1 inventory",
    )
    arguments = parser.parse_args(argv)
    root = arguments.repo_root.resolve(strict=True)
    catalog_path = root / "specification" / "catalog" / "kmip-2.1.json"
    try:
        raw = safe_read_bytes(root, "specification/catalog/kmip-2.1.json", max_bytes=MAX_BYTES)
        result = validate_catalog(raw, root, require_complete=not arguments.structural_only)
    except (OSError, CatalogValidationError, PathSecurityError) as error:
        print(f"catalog validation failed: {error}", file=sys.stderr)
        return 1
    print(
        "Catalog valid: "
        f"sources={result['source_count']} "
        f"clauses={result['clause_count']} "
        f"records={result['record_count']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
