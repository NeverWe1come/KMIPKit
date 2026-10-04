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


MAX_BYTES = 16 * 1024 * 1024
MAX_DEPTH = 32
MAX_RECORDS = 100_000
MAX_TOKENS = 1_000_000
MAX_MEMBERS = 1_000_000
MAX_STRING_BYTES = 65_536
TOP_LEVEL_FIELDS = {
    "schema_version",
    "sources",
    "source_clauses",
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
CLAUSE_DISPOSITIONS = {
    "requirement",
    "profile_conditional",
    "server_only",
    "later_1_1",
    "non_applicable",
    "informative_context",
    "source_discrepancy",
}
TOP_LEVEL_ID_FIELDS = {
    "source_clauses": ("clause_id", r"KMIPKIT-CLAUSE-(?:SPEC|PROF)-[0-9]+(?:\.[0-9]+)*-[0-9]{3}"),
    "elements": ("element_id", r"KMIPKIT-ELEM-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "tag_ranges": ("range_id", r"KMIPKIT-RANGE-[0-9]{3}"),
    "requirements": ("requirement_id", r"KMIPKIT-REQ-(?:SPEC|PROF)-[0-9]+(?:\.[0-9]+)*-[0-9]{3}"),
    "policies": ("policy_id", r"KMIPKIT-POLICY-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "profiles": ("profile_id", r"KMIPKIT-PROFILE-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "test_cases": ("test_id", r"KMIPKIT-TEST-[A-Z0-9]+(?:-[A-Z0-9]+)*"),
    "discrepancies": ("discrepancy_id", r"KMIPKIT-DISC-[0-9]{3}"),
    "decisions": ("decision_id", r"KMIPKIT-DEC-[0-9]{3}"),
}
ELEMENT_FIELDS = {
    "element_id", "kind", "name", "source_refs", "wire_value", "allocation",
    "direction", "scope_state", "scope_reason", "parent_element_ids",
    "requirement_ids", "profile_ids", "test_case_ids",
}
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


class CatalogValidationError(ValueError):
    """Raised when the catalog fails a structural or semantic rule."""


def _fail(message: str) -> None:
    raise CatalogValidationError(message)


def _reject_duplicate_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            _fail(f"duplicate JSON object key: {key}")
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
        while self.position < len(self.text):
            character = self.text[self.position]
            if character == '"':
                self.position += 1
                encoded = self.text[start:self.position].encode("utf-8")
                if len(encoded) > MAX_STRING_BYTES * 6 + 2:
                    _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
                try:
                    value = json.loads(encoded.decode("utf-8"))
                except (json.JSONDecodeError, UnicodeDecodeError) as error:
                    raise CatalogValidationError("catalog contains an invalid JSON string") from error
                _string_utf8_length(value)
                return value
            if ord(character) < 0x20:
                _fail("catalog contains an unescaped JSON control character")
            if character == "\\":
                self.position += 1
                if self.position >= len(self.text):
                    _fail("catalog contains a truncated JSON escape")
                escape = self.text[self.position]
                if escape == "u":
                    digits = self.text[self.position + 1:self.position + 5]
                    if len(digits) != 4 or re.fullmatch(r"[0-9a-fA-F]{4}", digits) is None:
                        _fail("catalog contains an invalid Unicode escape")
                    self.position += 5
                    continue
                if escape not in '"\\/bfnrt':
                    _fail("catalog contains an invalid JSON escape")
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
                    _fail(f"duplicate JSON object key: {key}")
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
        number = re.match(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?", self.text[self.position:])
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
    source_root = root / "specification" / "oasis" / "kmip-2.1"
    inventory_path = source_root / "SOURCES.md"
    checksums_path = source_root / "CHECKSUMS.sha256"
    try:
        inventory = inventory_path.read_text(encoding="utf-8")
        checksums = checksums_path.read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
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


def _git_tree(root: Path) -> dict[str, tuple[str, str]]:
    try:
        result = subprocess.run(
            ["git", "ls-tree", "-r", "-z", "--full-tree", "HEAD", "--", "specification/oasis/"],
            cwd=root,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise CatalogValidationError("could not read the pinned OASIS Git tree metadata") from error
    tree: dict[str, tuple[str, str]] = {}
    for entry in result.stdout.split(b"\0"):
        if not entry:
            continue
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
        path = root / Path(relative)
        try:
            metadata = path.lstat()
        except OSError as error:
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
    if any(value not in known_ids for value in values):
        _fail(f"{collection} contains an unresolved {field} reference")


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
            "normative_strength", "disposition", "requirement_ids", "exclusion_rationale",
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
        strengths = {KEYWORD_STRENGTH[keyword] for keyword in keywords}
        if len(strengths) != 1 or clause["normative_strength"] not in strengths:
            _fail("source clause keyword and normative strength disagree")
        if clause["disposition"] not in CLAUSE_DISPOSITIONS:
            _fail("source clause has an invalid review disposition")
        if not isinstance(clause["requirement_ids"], list):
            _fail("source clause requirement_ids must be an array")
        rationale = clause["exclusion_rationale"]
        if clause["disposition"] == "requirement":
            if not clause["requirement_ids"] or rationale is not None:
                _fail("requirement clauses need requirement links and no exclusion rationale")
        elif clause["requirement_ids"] or not isinstance(rationale, str) or not rationale.strip():
            _fail("excluded source clauses need a rationale and no requirement links")
    return clauses


def _check_semantics(
    catalog: dict[str, Any],
    sources: set[str],
    clauses: set[str],
    tree: dict[str, tuple[str, str]],
) -> None:
    requirements = {record["requirement_id"]: record for record in catalog["requirements"]}
    elements = _records_by_id(catalog, "elements", "element_id")
    profiles = _records_by_id(catalog, "profiles", "profile_id")
    test_cases = _records_by_id(catalog, "test_cases", "test_id")
    decisions = _records_by_id(catalog, "decisions", "decision_id")
    for requirement in catalog["requirements"]:
        _source_refs(requirement.get("source_refs"), sources, "requirement source_refs")
        if not requirement.get("source_clause_ids") or any(item not in clauses for item in requirement["source_clause_ids"]):
            _fail("requirement has missing or unresolved source clause references")
        keyword = requirement.get("source_keyword")
        if keyword not in KEYWORD_STRENGTH or requirement.get("normative_strength") != KEYWORD_STRENGTH.get(keyword):
            _fail("requirement keyword and normative strength disagree")
        if requirement.get("scope_state") not in {"client_1_0", "client_1_1", "profile_conditional", "server_only", "out_of_scope"}:
            _fail("requirement has an invalid scope state")
        if keyword in {"MUST NOT", "SHALL NOT"} and requirement.get("negative_verification_required") is not True:
            _fail("prohibited requirements require negative verification")
        if keyword in {"SHOULD", "SHOULD NOT", "RECOMMENDED"} and requirement.get("status") == "deviated" and not requirement.get("decision_id"):
            _fail("recommendation deviations require an accepted decision")
    for clause in catalog["source_clauses"]:
        if any(item not in requirements for item in clause["requirement_ids"]):
            _fail("source clause has an unresolved requirement reference")

    for element in catalog["elements"]:
        if set(element) - ELEMENT_FIELDS:
            _fail("protocol element has unknown fields")
        if element.get("kind") not in ELEMENT_KINDS:
            _fail("protocol element has an invalid kind")
        if not isinstance(element.get("name"), str) or not element["name"].strip():
            _fail("protocol element name is required")
        _source_refs(element.get("source_refs"), sources, "element source_refs")
        if element.get("direction") not in {"client_to_server", "server_to_client", "both", "not_applicable"}:
            _fail("protocol element has an invalid direction")
        if element.get("scope_state") not in {"client_1_0", "client_1_1", "profile_conditional", "server_only", "out_of_scope"}:
            _fail("protocol element has an invalid scope state")
        if element.get("scope_state") != "client_1_0" and not isinstance(element.get("scope_reason"), str):
            _fail("non-default protocol element scope requires a rationale")
        _check_link_ids(element, "parent_element_ids", set(elements), "protocol element")
        _check_link_ids(element, "requirement_ids", set(requirements), "protocol element")
        _check_link_ids(element, "profile_ids", set(profiles), "protocol element")
        _check_link_ids(element, "test_case_ids", set(test_cases), "protocol element")
        if "wire_value" in element and element["wire_value"] is not None and not isinstance(element["wire_value"], str):
            _fail("protocol element wire_value must be a string")
        if "allocation" in element and element["allocation"] not in {None, "assigned", "reserved", "unused"}:
            _fail("protocol element has an invalid tag allocation")

    for requirement in catalog["requirements"]:
        _check_link_ids(requirement, "element_ids", set(elements), "requirement")
        _check_link_ids(requirement, "profile_ids", set(profiles), "requirement")
        _check_link_ids(requirement, "test_case_ids", set(test_cases), "requirement")
        decision_id = requirement.get("decision_id")
        if decision_id is not None and decision_id not in decisions:
            _fail("requirement refers to an unresolved decision")

    for test_case in catalog["test_cases"]:
        if not isinstance(test_case, dict) or set(test_case) != TEST_CASE_FIELDS:
            _fail("test case has missing or unknown fields")
        if not isinstance(test_case["official_case_id"], str) or not test_case["official_case_id"].strip():
            _fail("test case must retain its official case ID")
        if test_case["source_id"] not in {"KMIPKIT-SRC-testcases", "KMIPKIT-SRC-profiles"}:
            _fail("test case points to a source that cannot define test evidence")
        if not re.fullmatch(r"[0-9]+(?:\.[0-9]+)*", test_case["source_section"]):
            _fail("test case has an invalid source section")
        if test_case["mandatory_status"] not in {"mandatory", "optional", "unspecified"}:
            _fail("test case has an invalid mandatory/optional status")
        if test_case["fixture_availability"] not in {"available", "unavailable"}:
            _fail("test case has an invalid fixture availability")
        if test_case["mapping_confidence"] not in {"explicit", "strong", "weak", "unmapped"}:
            _fail("test case has an invalid mapping confidence")
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

    for tag_range in catalog["tag_ranges"]:
        if tag_range.get("allocation") not in {"unused", "reserved", "extension"}:
            _fail("tag range must have a range allocation")
        _source_refs(tag_range.get("source_refs"), sources, "tag range source_refs")

    for policy in catalog["policies"]:
        if policy.get("provenance") not in {"AGENTS.md", "constitution", "ADR", "approved_product_decision"}:
            _fail("project policy has invalid provenance")
        if not isinstance(policy.get("summary"), str) or not policy["summary"].strip():
            _fail("project policy summary is required")


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


def validate_catalog(raw: bytes, repo_root: Path) -> dict[str, Any]:
    """Validate raw UTF-8 JSON bytes and return deterministic aggregate counts."""
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
    _check_semantics(catalog, sources, clauses, tree)

    for source in catalog["sources"]:
        path = repo_root / source["local_path"]
        try:
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            raise CatalogValidationError("pinned OASIS source file is unavailable") from error
        if digest != source["sha256"]:
            _fail(f"pinned source checksum mismatch for {source['source_id']}")

    return {
        "source_count": len(sources),
        "clause_count": len(clauses),
        "record_count": sum(len(catalog[key]) for key in TOP_LEVEL_FIELDS - {"schema_version"}),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    arguments = parser.parse_args(argv)
    root = arguments.repo_root.resolve(strict=True)
    catalog_path = root / "specification" / "catalog" / "kmip-2.1.json"
    try:
        result = validate_catalog(catalog_path.read_bytes(), root)
    except (OSError, CatalogValidationError) as error:
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
