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
    from .safe_io import PathSecurityError, confined_path, safe_read_bytes
else:
    from safe_io import PathSecurityError, confined_path, safe_read_bytes


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
    "requirement_ids", "profile_ids", "test_case_ids", "feature_spec",
    "implementation_refs", "verification_refs", "payload_tables", "asynchronous_response",
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
PROFILE_FIELDS = {
    "profile_id", "name", "role", "source_refs", "source_clause_ids",
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
        _enum(clause["disposition"], CLAUSE_DISPOSITIONS, "source clause disposition")
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
    repo_root: Path,
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
        if (
            not isinstance(keyword, str)
            or keyword not in KEYWORD_STRENGTH
            or requirement.get("normative_strength") != KEYWORD_STRENGTH.get(keyword)
        ):
            _fail("requirement keyword and normative strength disagree")
        _enum(
            requirement.get("scope_state"),
            {"client_1_0", "client_1_1", "profile_conditional", "server_only", "out_of_scope"},
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
        if any(item not in requirements for item in clause["requirement_ids"]):
            _fail("source clause has an unresolved requirement reference")

    for element in catalog["elements"]:
        if set(element) - ELEMENT_FIELDS:
            _fail("protocol element has unknown fields")
        if not {"feature_spec", "implementation_refs", "verification_refs"}.issubset(element):
            _fail("protocol element is missing coverage assignment fields")
        _enum(element.get("kind"), ELEMENT_KINDS, "protocol element kind")
        if not isinstance(element.get("name"), str) or not element["name"].strip():
            _fail("protocol element name is required")
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

    for requirement in catalog["requirements"]:
        _check_link_ids(requirement, "element_ids", set(elements), "requirement")
        _check_link_ids(requirement, "profile_ids", set(profiles), "requirement")
        _check_link_ids(requirement, "test_case_ids", set(test_cases), "requirement")
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
        for field in ("source_clause_ids", "transport_requirements", "encoding_requirements"):
            _string_values(profile, field, "profile")
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
            if not any(test_cases[test_id]["source_id"] == "KMIPKIT-SRC-testcases" for test_id in profile["test_case_ids"]):
                _fail("profile evidence is incomplete without an official KMIP Test Case")

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
        required_scope = "client_1_0" if key[0] == "client_to_server" else "client_1_1"
        if operation.get("scope_state") != required_scope:
            _fail(f"operation {operation.get('name')} has an incorrect scope disposition")
        if key[0] == "server_to_client" and not operation.get("scope_reason"):
            _fail(f"operation {operation.get('name')} requires a 1.1 scope rationale")
        name = key[1]
        expected_tables = _expected_operation_payload_tables(name, key[0])
        if operation.get("payload_tables") != expected_tables:
            _fail(f"operation {name} has incorrect payload table references")
        if operation.get("asynchronous_response") != ASYNC_RESPONSE_BEHAVIORS.get(name):
            _fail(f"operation {name} has incorrect asynchronous response classification")


COMPLETE_ELEMENT_COUNTS = {
    "data_type": 11,
    "object_type": 9,
    "object_structure": 12,
    "attribute": 63,
    "attribute_structure": 7,
    "operation_structure": 41,
    "enumeration": 64,
    "enumeration_value": 723,
    "bitmask": 3,
    "bitmask_value": 44,
    "tag": 374,
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
