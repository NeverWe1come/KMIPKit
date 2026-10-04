"""Strict offline validation for the reviewed KMIP 2.1 inventory."""

from __future__ import annotations

import hashlib
import json
import re
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


def _preflight(raw: bytes) -> str:
    if len(raw) > MAX_BYTES:
        _fail("catalog exceeds the 16 MiB input limit")
    try:
        text = raw.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise CatalogValidationError("catalog is not valid UTF-8") from error

    depth = 0
    in_string = False
    escaped = False
    tokens = 0
    for character in text:
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
            tokens += 1
        elif character in "[{":
            depth += 1
            tokens += 1
            if depth > MAX_DEPTH:
                _fail("catalog exceeds the maximum nesting depth of 32")
        elif character in "]}":
            depth -= 1
            tokens += 1
            if depth < 0:
                _fail("catalog has invalid JSON structure")
        elif character in ":,":
            tokens += 1
        elif not character.isspace():
            tokens += 1
        if tokens > MAX_TOKENS:
            _fail("catalog exceeds the global JSON token limit")
    if in_string or depth != 0:
        _fail("catalog has invalid JSON structure")
    return text


def _reject_surrogates(value: Any) -> None:
    if isinstance(value, str):
        if any(0xD800 <= ord(character) <= 0xDFFF for character in value):
            _fail("catalog contains an unpaired Unicode surrogate")
        if len(value.encode("utf-8")) > MAX_STRING_BYTES:
            _fail("catalog contains a string longer than 65,536 UTF-8 bytes")
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


def _check_source_records(catalog: dict[str, Any], root: Path) -> set[str]:
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


def _check_semantics(catalog: dict[str, Any], sources: set[str], clauses: set[str]) -> None:
    requirements = {record["requirement_id"]: record for record in catalog["requirements"]}
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
        _source_refs(element.get("source_refs"), sources, "element source_refs")
        if element.get("direction") not in {"client_to_server", "server_to_client", "both", "not_applicable"}:
            _fail("protocol element has an invalid direction")
        if element.get("scope_state") not in {"client_1_0", "client_1_1", "profile_conditional", "server_only", "out_of_scope"}:
            _fail("protocol element has an invalid scope state")

    for tag_range in catalog["tag_ranges"]:
        if tag_range.get("allocation") not in {"unused", "reserved", "extension"}:
            _fail("tag range must have a range allocation")
        _source_refs(tag_range.get("source_refs"), sources, "tag range source_refs")

    for policy in catalog["policies"]:
        if policy.get("provenance") not in {"AGENTS.md", "constitution", "ADR", "approved_product_decision"}:
            _fail("project policy has invalid provenance")
        if not isinstance(policy.get("summary"), str) or not policy["summary"].strip():
            _fail("project policy summary is required")


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

    sources = _check_source_records(catalog, repo_root)
    clauses = _check_clauses(catalog, sources)
    _check_identifiers(catalog)
    _check_semantics(catalog, sources, clauses)

    for source in catalog["sources"]:
        path = repo_root / source["local_path"]
        try:
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            raise CatalogValidationError("pinned OASIS source file is unavailable") from error
        if digest != source["sha256"]:
            _fail(f"pinned source checksum mismatch for {source['source_id']}")

    return {"source_count": len(sources), "clause_count": len(clauses), "record_count": sum(len(catalog[key]) for key in TOP_LEVEL_FIELDS - {"schema_version"})}
