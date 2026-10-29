"""Generate deterministic Rust metadata for the reviewed KMIP client profiles."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

if __package__:
    from .safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
else:
    from safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes


_SECTION = re.compile(r"[0-9]+(?:\.[0-9]+)*\Z")
_TARGET_DECISION = re.compile(r"KMIPKIT-0010/(FR-[0-9]{3})\Z")
MAX_PROFILE_TARGET_BYTES = 65_536
PROFILE_TARGET_PATH = "specification/catalog/profile-targets.json"
PROFILE_SPEC_PATH = "specs/010-profile-conformance/spec.md"
PROFILE_OUTPUT_PATH = "crates/kmipkit-protocol/src/profiles_generated.rs"
CATALOG_PATH = "specification/catalog/kmip-2.1.json"
MAX_CATALOG_BYTES = 16 * 1024 * 1024
_VERIFICATION_CATEGORIES = {
    "MUST": "RequiredBehavior",
    "SHALL": "RequiredBehavior",
    "REQUIRED": "RequiredBehavior",
    "MUST NOT": "ProhibitedBehavior",
    "SHALL NOT": "ProhibitedBehavior",
    "SHOULD": "RecommendedBehaviorOrDeviation",
    "RECOMMENDED": "RecommendedBehaviorOrDeviation",
    "SHOULD NOT": "DiscouragedAbsenceOrDeviation",
    "MAY": "OptionalCapability",
    "OPTIONAL": "OptionalCapability",
    "required_table_body_value": "RequiredBehavior",
    "optional_table_body_value": "OptionalCapability",
}


class ProfileGenerationError(ValueError):
    """Raised when profile metadata cannot be expanded without ambiguity."""


def _is_unresolved_client_scope(requirement: Mapping[str, Any]) -> bool:
    """Return whether a source obligation may apply to clients but is unresolved."""
    scope_state = requirement.get("scope_state")
    if requirement.get("role") in {"unclear", "unspecified"}:
        return scope_state == "unclear"
    if requirement.get("role") not in {"client", "both"}:
        return False
    return scope_state == "unclear" or (
        scope_state == "profile_conditional"
        and not requirement.get("profile_ids")
    )


@dataclass(frozen=True)
class ProfileClosure:
    """The deterministic normative clause and applicable requirement closure."""

    source_clause_ids: tuple[str, ...]
    requirement_ids: tuple[str, ...]
    unresolved_scope_obligations: tuple[UnresolvedProfileScopeObligation, ...]


@dataclass(frozen=True)
class UnresolvedProfileScopeObligation:
    """A client obligation whose applicability cannot yet be assigned safely."""

    requirement_id: str
    source_clause_id: str
    scope_state: str


@dataclass(frozen=True)
class ProfileTargets:
    """Strict project target selection, kept separate from runtime selection."""

    release_line: str
    profile_ids: tuple[str, ...]
    decision_ref: str


def _reject_duplicate_target_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ProfileGenerationError("profile target manifest contains duplicate JSON keys")
        result[key] = value
    return result


def parse_profile_targets(raw: bytes, catalog: Mapping[str, Any], repo_root: Path) -> ProfileTargets:
    """Parse and validate the exact bounded target manifest schema."""
    if len(raw) > MAX_PROFILE_TARGET_BYTES:
        raise ProfileGenerationError("profile target manifest exceeds its size limit")
    try:
        document = json.loads(raw.decode("utf-8"), object_pairs_hook=_reject_duplicate_target_pairs)
    except ProfileGenerationError:
        raise
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as error:
        raise ProfileGenerationError("profile target manifest is not valid UTF-8 JSON") from error
    if not isinstance(document, dict) or set(document) != {"release_line", "profile_ids", "decision_ref"}:
        raise ProfileGenerationError("profile target manifest has missing or unknown fields")
    release_line = document["release_line"]
    profile_ids = document["profile_ids"]
    decision_ref = document["decision_ref"]
    if release_line != "1.0" or not isinstance(release_line, str):
        raise ProfileGenerationError("profile target manifest release line must be 1.0")
    if (
        not isinstance(profile_ids, list)
        or not profile_ids
        or any(not isinstance(profile_id, str) or not profile_id.strip() for profile_id in profile_ids)
        or len(profile_ids) != len(set(profile_ids))
    ):
        raise ProfileGenerationError("profile target IDs must be unique non-empty strings")
    if not isinstance(decision_ref, str) or not decision_ref.strip():
        raise ProfileGenerationError("profile target decision reference must be non-empty")
    decision_match = _TARGET_DECISION.fullmatch(decision_ref)
    if decision_match is None:
        raise ProfileGenerationError("profile target decision reference is unresolved")
    try:
        specification = safe_read_bytes(repo_root, PROFILE_SPEC_PATH, max_bytes=1_048_576).decode("utf-8")
    except (OSError, UnicodeDecodeError, PathSecurityError) as error:
        raise ProfileGenerationError("profile target decision source is unavailable") from error
    if f"**{decision_match.group(1)}**" not in specification:
        raise ProfileGenerationError("profile target decision reference is unresolved")

    raw_profiles = catalog.get("profiles")
    if not isinstance(raw_profiles, list):
        raise ProfileGenerationError("catalog has invalid profile records")
    profiles_by_id = {
        profile.get("profile_id"): profile
        for profile in raw_profiles
        if isinstance(profile, dict) and isinstance(profile.get("profile_id"), str)
    }
    for profile_id in profile_ids:
        profile = profiles_by_id.get(profile_id)
        if profile is None:
            raise ProfileGenerationError("profile target refers to an unresolved profile")
        if profile.get("role") != "client" or profile.get("applicability") not in {"client_1_0", "conditional"}:
            raise ProfileGenerationError("profile target includes a profile outside approved client scope")
    return ProfileTargets(release_line, tuple(profile_ids), decision_ref)


def load_profile_targets(repo_root: Path, catalog: Mapping[str, Any]) -> ProfileTargets:
    """Read the fixed target path safely with the 64 KiB input cap."""
    try:
        raw = safe_read_bytes(repo_root, PROFILE_TARGET_PATH, max_bytes=MAX_PROFILE_TARGET_BYTES)
    except (OSError, PathSecurityError) as error:
        raise ProfileGenerationError("profile target manifest is unavailable or unsafe") from error
    return parse_profile_targets(raw, catalog, repo_root)


def _section_in_scope(section: str, included_section: str | None) -> bool:
    return included_section is None or section == included_section or section.startswith(f"{included_section}.")


def compute_profile_closure(
    catalog: Mapping[str, Any],
    direct_source_clause_ids: Sequence[str],
    *,
    profile_id: str,
) -> ProfileClosure:
    """Expand direct clauses through same-source descendants and normative inclusions.

    Only client-applicable 1.0, mixed client/server, and profile-conditionally
    applicable requirements are included. A mixed scope is eligible when the
    requirement's reviewed role identifies a client obligation; server-only
    and out-of-scope clause records remain in the source clause closure for
    reporting.
    """
    raw_clauses = catalog.get("source_clauses")
    raw_inclusions = catalog.get("normative_inclusions", [])
    raw_requirements = catalog.get("requirements")
    if not isinstance(raw_clauses, list) or not isinstance(raw_inclusions, list):
        raise ProfileGenerationError("catalog has invalid source clause or inclusion records")
    if not isinstance(raw_requirements, list):
        raise ProfileGenerationError("catalog has invalid requirement records")
    if not isinstance(profile_id, str) or not profile_id:
        raise ProfileGenerationError("profile closure requires a stable profile identity")
    if (
        not direct_source_clause_ids
        or isinstance(direct_source_clause_ids, (str, bytes))
        or any(not isinstance(value, str) or not value for value in direct_source_clause_ids)
    ):
        raise ProfileGenerationError("profile has no valid direct source clauses")

    clauses_by_id: dict[str, dict[str, Any]] = {}
    clauses_by_source: dict[str, list[dict[str, Any]]] = {}
    for clause in raw_clauses:
        if not isinstance(clause, dict):
            raise ProfileGenerationError("catalog has an invalid source clause record")
        clause_id = clause.get("clause_id")
        source_id = clause.get("source_id")
        section = clause.get("section")
        if (
            not isinstance(clause_id, str)
            or not clause_id
            or clause_id in clauses_by_id
            or not isinstance(source_id, str)
            or not source_id
            or not isinstance(section, str)
            or not _SECTION.fullmatch(section)
        ):
            raise ProfileGenerationError("catalog has an invalid or duplicate source clause identity")
        clauses_by_id[clause_id] = clause
        clauses_by_source.setdefault(source_id, []).append(clause)

    inclusions_by_clause: dict[str, list[dict[str, Any]]] = {}
    inclusion_ids: set[str] = set()
    for inclusion in raw_inclusions:
        if not isinstance(inclusion, dict):
            raise ProfileGenerationError("catalog has an invalid normative inclusion record")
        inclusion_id = inclusion.get("inclusion_id")
        clause_id = inclusion.get("source_clause_id")
        included_source = inclusion.get("included_source_id")
        included_section = inclusion.get("included_section")
        if (
            not isinstance(inclusion_id, str)
            or not inclusion_id
            or inclusion_id in inclusion_ids
            or clause_id not in clauses_by_id
            or not isinstance(included_source, str)
            or not included_source
            or (included_section is not None and (
                not isinstance(included_section, str) or not _SECTION.fullmatch(included_section)
            ))
        ):
            raise ProfileGenerationError("catalog has an invalid or duplicate normative inclusion")
        inclusion_ids.add(inclusion_id)
        inclusions_by_clause.setdefault(clause_id, []).append(inclusion)

    source_clause_ids: set[str] = set()

    def expand_scope(
        source_id: str,
        included_section: str | None,
        active_scopes: frozenset[tuple[str, str | None]],
    ) -> None:
        scope = (source_id, included_section)
        if scope in active_scopes:
            raise ProfileGenerationError("cyclic normative inclusion")
        matching = [
            clause
            for clause in clauses_by_source.get(source_id, ())
            if _section_in_scope(clause["section"], included_section)
        ]
        if not matching:
            raise ProfileGenerationError("normative inclusion does not resolve to a source clause")
        next_active = active_scopes | {scope}
        for clause in matching:
            clause_id = clause["clause_id"]
            source_clause_ids.add(clause_id)
            for inclusion in inclusions_by_clause.get(clause_id, ()):
                expand_scope(
                    inclusion["included_source_id"],
                    inclusion["included_section"],
                    next_active,
                )

    for clause_id in direct_source_clause_ids:
        clause = clauses_by_id.get(clause_id)
        if clause is None:
            raise ProfileGenerationError(f"direct source clause does not resolve: {clause_id}")
        expand_scope(clause["source_id"], clause["section"], frozenset())

    requirement_ids: set[str] = set()
    unresolved_scope_obligations: set[UnresolvedProfileScopeObligation] = set()
    closed_source_sections = {
        (clauses_by_id[clause_id]["source_id"], clauses_by_id[clause_id]["section"])
        for clause_id in source_clause_ids
    }
    for requirement in raw_requirements:
        if not isinstance(requirement, dict):
            raise ProfileGenerationError("catalog has an invalid requirement record")
        requirement_id = requirement.get("requirement_id")
        linked_clauses = requirement.get("source_clause_ids")
        if not isinstance(requirement_id, str) or not requirement_id or not isinstance(linked_clauses, list):
            raise ProfileGenerationError("catalog has an invalid requirement identity or clause links")
        scope_state = requirement.get("scope_state")
        intersects_closure = [
            clause_id for clause_id in linked_clauses
            if isinstance(clause_id, str) and clause_id in source_clause_ids
        ]
        table_row_intersects_closure = any(
            isinstance(source_ref, dict)
            and isinstance(source_ref.get("source_id"), str)
            and isinstance(source_ref.get("section"), str)
            and any(
                closed_source_id == source_ref["source_id"]
                and _section_in_scope(source_ref["section"], closed_section)
                for closed_source_id, closed_section in closed_source_sections
            )
            for source_ref in requirement.get("source_refs", [])
        ) and requirement.get("source_force") in {
            "required_table_body_value",
            "optional_table_body_value",
        }
        if _is_unresolved_client_scope(requirement):
            unresolved_scope_obligations.update(
                UnresolvedProfileScopeObligation(requirement_id, clause_id, scope_state)
                for clause_id in intersects_closure
            )
        profile_membership_applies = (
            scope_state == "profile_conditional"
            and profile_id in requirement.get("profile_ids", [])
        )
        in_scope = scope_state in {"client_1_0", "mixed"} or profile_membership_applies
        if (
            requirement.get("role") in {"client", "both"}
            and in_scope
            and (
                profile_membership_applies
                or any(clause_id in source_clause_ids for clause_id in linked_clauses)
                or table_row_intersects_closure
            )
        ):
            requirement_ids.add(requirement_id)

    return ProfileClosure(
        tuple(sorted(source_clause_ids)),
        tuple(sorted(requirement_ids)),
        tuple(sorted(
            unresolved_scope_obligations,
            key=lambda obligation: (obligation.requirement_id, obligation.source_clause_id),
        )),
    )


def rust_string_literal(value: str) -> str:
    """Encode metadata as one Rust string literal without source injection."""
    if not isinstance(value, str):
        raise ProfileGenerationError("generated Rust string value must be text")
    escaped: list[str] = []
    for character in value:
        codepoint = ord(character)
        if 0xD800 <= codepoint <= 0xDFFF:
            raise ProfileGenerationError("generated Rust string value contains invalid Unicode")
        if character == '"':
            escaped.append(r'\"')
        elif character == "\\":
            escaped.append(r"\\")
        elif character == "\n":
            escaped.append(r"\n")
        elif character == "\r":
            escaped.append(r"\r")
        elif character == "\t":
            escaped.append(r"\t")
        elif character == "\0":
            escaped.append(r"\0")
        elif codepoint < 0x20 or 0x7F <= codepoint <= 0x9F:
            escaped.append(f"\\u{{{codepoint:x}}}")
        else:
            escaped.append(character)
    return f'"{"".join(escaped)}"'


def _rust_string_field(record: Mapping[str, Any], field: str) -> str:
    value = record.get(field)
    if not isinstance(value, str) or not value:
        raise ProfileGenerationError(f"profile metadata field {field} must be non-empty text")
    return rust_string_literal(value)


def _rust_string_array(record: Mapping[str, Any], field: str) -> str:
    values = record.get(field)
    if (
        not isinstance(values, list)
        or any(not isinstance(value, str) or not value for value in values)
        or len(values) != len(set(values))
    ):
        raise ProfileGenerationError(f"profile metadata field {field} must contain unique text values")
    if not values:
        return "&[]"
    encoded = [rust_string_literal(value) for value in values]
    if len(encoded) == 1:
        return f"&[{encoded[0]}]"
    encoded_values = ",\n            ".join(encoded) + ","
    return f"&[\n            {encoded_values}\n        ]"


def _verification_category(source_keyword: str) -> str:
    category = _VERIFICATION_CATEGORIES.get(source_keyword)
    if category is None:
        raise ProfileGenerationError("client requirement has an unsupported normative keyword")
    return category


def _rust_optional_string(value: Any) -> str:
    if value is None:
        return "None"
    if not isinstance(value, str) or not value:
        raise ProfileGenerationError("optional generated metadata must be null or non-empty text")
    return f"Some({rust_string_literal(value)})"


def _source_gap_locator(requirement: Mapping[str, Any]) -> str | None:
    if requirement.get("source_gap_id") is None:
        return None
    locator = requirement.get("source_locator")
    if not isinstance(locator, Mapping):
        raise ProfileGenerationError("source-gap requirement lacks a structured source locator")
    section = locator.get("section")
    row_label = locator.get("row_label")
    line_range = locator.get("html_line_range")
    table = locator.get("table")
    if (
        not isinstance(section, str)
        or not section
        or not isinstance(row_label, str)
        or not row_label
        or not isinstance(line_range, list)
        or len(line_range) != 2
        or any(type(line) is not int or line < 1 for line in line_range)
        or (table is not None and (not isinstance(table, str) or not table))
    ):
        raise ProfileGenerationError("source-gap requirement has an invalid source locator")
    table_reference = f", table {table}" if table is not None else ""
    return (
        f"Section {section}{table_reference}, row {row_label}, "
        f"HTML lines {line_range[0]}-{line_range[1]}"
    )


def _encode_profile_id_bitmap(values: Sequence[str], universe: Sequence[str]) -> str:
    selected_values = set(values)
    bitmap = bytearray((len(universe) + 7) // 8)
    for bit_index, value in enumerate(universe):
        if value in selected_values:
            bitmap[bit_index // 8] |= 1 << (bit_index % 8)
    return bytes(bitmap).hex()


def render_profiles_source(catalog: Mapping[str, Any], targets: ProfileTargets) -> str:
    """Render deterministic immutable client metadata without executable rules."""
    raw_profiles = catalog.get("profiles")
    raw_test_cases = catalog.get("test_cases")
    raw_requirements = catalog.get("requirements")
    raw_decisions = catalog.get("decisions")
    if (
        not isinstance(raw_profiles, list)
        or not isinstance(raw_test_cases, list)
        or not isinstance(raw_requirements, list)
        or not isinstance(raw_decisions, list)
    ):
        raise ProfileGenerationError("catalog has invalid profile, requirement, decision, or test records")
    tests_by_id = {
        test_case.get("test_id"): test_case
        for test_case in raw_test_cases
        if isinstance(test_case, dict) and isinstance(test_case.get("test_id"), str)
    }
    accepted_decision_ids = {
        decision.get("decision_id")
        for decision in raw_decisions
        if isinstance(decision, dict) and decision.get("status") == "accepted"
    }
    profiles = sorted(
        (profile for profile in raw_profiles if isinstance(profile, dict) and profile.get("role") == "client"),
        key=lambda profile: profile.get("profile_id", ""),
    )
    requirements = sorted(
        (
            requirement
            for requirement in raw_requirements
            if isinstance(requirement, dict)
            and requirement.get("role") in {"client", "both"}
            and requirement.get("scope_state") in {"client_1_0", "mixed", "profile_conditional"}
        ),
        key=lambda requirement: requirement.get("requirement_id", ""),
    )
    if not profiles:
        raise ProfileGenerationError("catalog contains no client profiles")

    test_ids_by_profile: dict[str, tuple[str, ...]] = {}
    for profile in profiles:
        profile_id = profile.get("profile_id")
        test_ids = profile.get("test_case_ids")
        if (
            not isinstance(profile_id, str)
            or not profile_id
            or not isinstance(test_ids, list)
            or any(not isinstance(test_id, str) or test_id not in tests_by_id for test_id in test_ids)
            or len(test_ids) != len(set(test_ids))
        ):
            raise ProfileGenerationError("client profile has unresolved or duplicate test identifiers")
        test_ids_by_profile[profile_id] = tuple(test_ids)
    shared_test_ids = sorted({test_id for test_ids in test_ids_by_profile.values() for test_id in test_ids})
    test_case_indexes = {test_id: index for index, test_id in enumerate(shared_test_ids)}

    lines = [
        "//! Generated from the reviewed normative catalog and fixed profile target manifest.",
        "//! This file contains metadata only; it does not contain executable conformance rules.",
        "",
        "/// Official conformance test evidence linked to a profile.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub struct GeneratedProfileTestCase {",
        "    /// Stable catalog test identifier.",
        "    pub test_id: &'static str,",
        "    /// Official OASIS case identifier.",
        "    pub official_case_id: &'static str,",
        "    /// Catalogued mandatory, optional, or unspecified status.",
        "    pub mandatory_status: &'static str,",
        "    /// Whether the exact local fixture is available.",
        "    pub fixture_availability: &'static str,",
        "}",
        "",
        "/// Client requirement whose applicability is unresolved for a profile.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub struct GeneratedProfileScopeBlocker {",
        "    /// Stable catalog requirement identifier.",
        "    pub requirement_id: &'static str,",
        "    /// Exact catalog source clause that links the obligation to the profile.",
        "    pub source_clause_id: &'static str,",
        "    /// Catalog scope state that still requires a reviewed disposition.",
        "    pub scope_state: &'static str,",
        "}",
        "",
        "/// Verification obligation derived from an exact keyword or typed table force.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub enum GeneratedVerificationCategory {",
        "    /// Positive tests must show that required behavior is provided.",
        "    RequiredBehavior,",
        "    /// Negative tests must show that prohibited behavior is rejected or absent.",
        "    ProhibitedBehavior,",
        "    /// Behavior must be tested or an accepted deviation must exist.",
        "    RecommendedBehaviorOrDeviation,",
        "    /// Discouraged behavior must be absent or have an accepted deviation.",
        "    DiscouragedAbsenceOrDeviation,",
        "    /// Optional capability must be represented, usable, and positively tested.",
        "    OptionalCapability,",
        "}",
        "",
        "/// Source-backed normative metadata for a client-applicable requirement.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub struct GeneratedClientRequirement {",
        "    /// Stable catalog requirement identifier.",
        "    pub requirement_id: &'static str,",
        "    /// Exact OASIS keyword, absent for table-field classifications.",
        "    pub source_keyword: Option<&'static str>,",
        "    /// Typed table-field force, absent for formal keyword requirements.",
        "    pub source_force: Option<&'static str>,",
        "    /// Stable source-only gap ID for a table-field classification.",
        "    pub source_gap_id: Option<&'static str>,",
        "    /// Exact catalogued source locator for a table-field classification.",
        "    pub source_locator: Option<&'static str>,",
        "    /// Category-specific verification obligation derived from the source force.",
        "    pub verification_category: GeneratedVerificationCategory,",
        "    /// Catalogued client applicability state.",
        "    pub scope_state: &'static str,",
        "    /// Stable source clauses that contain this requirement.",
        "    pub source_clause_ids: &'static [&'static str],",
        "    /// Profiles to which a conditional requirement applies.",
        "    pub profile_ids: &'static [&'static str],",
        "    /// Official OASIS test cases linked by the catalog.",
        "    pub official_test_case_ids: &'static [&'static str],",
        "    /// Whether the catalog records a reviewed deviation.",
        "    pub has_accepted_deviation: bool,",
        "}",
        "",
        "/// Immutable source and applicability metadata for one client profile.",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]",
        "pub struct GeneratedClientProfile {",
        "    /// Stable catalog profile identifier.",
        "    pub profile_id: &'static str,",
        "    /// Human-readable source name.",
        "    pub name: &'static str,",
        "    /// Source applicability lifecycle.",
        "    pub applicability: &'static str,",
        "    /// Catalog evidence lifecycle, independent of target membership.",
        "    pub claim_state: &'static str,",
        "    /// Whether the fixed product target manifest selects this profile.",
        "    pub selected_target: bool,",
        "    /// Profile clauses directly selected as closure roots.",
        "    pub direct_source_clause_ids: &'static [&'static str],",
        "    /// Complete computed normative clause closure.",
        "    pub source_clause_ids: &'static [&'static str],",
        "    /// Applicable stable normative requirement identifiers.",
        "    pub requirement_ids: &'static [&'static str],",
        "    /// Client obligations in the clause closure whose scope is unresolved.",
        "    pub unresolved_scope_obligations: &'static [GeneratedProfileScopeBlocker],",
        "    /// Profile dependencies resolved by the selection layer.",
        "    pub dependency_profile_ids: &'static [&'static str],",
        "    /// Transport requirements retained as metadata.",
        "    pub transport_requirements: &'static [&'static str],",
        "    /// Encoding requirements retained as metadata.",
        "    pub encoding_requirements: &'static [&'static str],",
        "    /// Linked official tests and local fixture availability.",
        "    pub test_cases: &'static [GeneratedProfileTestCase],",
        "}",
        "",
        "const fn profile_bitmap_nibble(value: u8) -> u8 {",
        "    match value {",
        "        b'0'..=b'9' => value - b'0',",
        "        b'a'..=b'f' => value - b'a' + 10,",
        "        _ => 0,",
        "    }",
        "}",
        "",
        "const fn profile_bitmap_contains(bitmap: &str, bit_index: usize) -> bool {",
        "    let byte_index = bit_index / 8;",
        "    let character_index = byte_index * 2;",
        "    let encoded = bitmap.as_bytes();",
        "    if character_index + 1 >= encoded.len() {",
        "        return false;",
        "    }",
        "    let byte = (profile_bitmap_nibble(encoded[character_index]) << 4)",
        "        | profile_bitmap_nibble(encoded[character_index + 1]);",
        "    byte & (1 << (bit_index % 8)) != 0",
        "}",
        "",
        "const fn profile_id_set_len(pool: &[&str], bitmap: &str) -> usize {",
        "    let mut index = 0;",
        "    let mut count = 0;",
        "    while index < pool.len() {",
        "        if profile_bitmap_contains(bitmap, index) {",
        "            count += 1;",
        "        }",
        "        index += 1;",
        "    }",
        "    count",
        "}",
        "",
        "const fn decode_profile_id_set<const N: usize>(",
        "    pool: &[&'static str],",
        "    bitmap: &str,",
        ") -> [&'static str; N] {",
        "    let mut result = [\"\"; N];",
        "    let mut pool_index = 0;",
        "    let mut result_index = 0;",
        "    while pool_index < pool.len() {",
        "        if profile_bitmap_contains(bitmap, pool_index) && result_index < N {",
        "            result[result_index] = pool[pool_index];",
        "            result_index += 1;",
        "        }",
        "        pool_index += 1;",
        "    }",
        "    result",
        "}",
        "",
    ]
    selected_ids = set(targets.profile_ids)
    lines.extend(
        [
            "",
            "/// Client requirements in the approved product scope, sorted by stable identifier.",
            "pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[",
        ]
    )
    for requirement in requirements:
        requirement_id = requirement.get("requirement_id")
        source_keyword = requirement.get("source_keyword")
        source_force = requirement.get("source_force")
        category_source = source_keyword if isinstance(source_keyword, str) else source_force
        if (
            not isinstance(requirement_id, str)
            or not requirement_id
            or not isinstance(category_source, str)
            or not category_source
        ):
            raise ProfileGenerationError("client requirement lacks stable source metadata")
        lines.extend(
            [
                "    GeneratedClientRequirement {",
                f"        requirement_id: {rust_string_literal(requirement_id)},",
                f"        source_keyword: {_rust_optional_string(source_keyword)},",
                f"        source_force: {_rust_optional_string(source_force)},",
                f"        source_gap_id: {_rust_optional_string(requirement.get('source_gap_id'))},",
                f"        source_locator: {_rust_optional_string(_source_gap_locator(requirement))},",
                (
                    "        verification_category: GeneratedVerificationCategory::"
                    f"{_verification_category(category_source)},"
                ),
                f"        scope_state: {_rust_string_field(requirement, 'scope_state')},",
                f"        source_clause_ids: {_rust_string_array(requirement, 'source_clause_ids')},",
                f"        profile_ids: {_rust_string_array(requirement, 'profile_ids')},",
                f"        official_test_case_ids: {_rust_string_array(requirement, 'test_case_ids')},",
                (
                    "        has_accepted_deviation: "
                    f"{str(requirement.get('status') == 'deviated' and requirement.get('decision_id') in accepted_decision_ids).lower()},"
                ),
                "    },",
            ]
        )
    lines.extend(["];", ""])

    lines.append("")
    lines.append("/// Official conformance cases referenced by client profiles, interned by stable test identifier.")
    lines.append("pub const PROFILE_TEST_CASES: &[GeneratedProfileTestCase] = &[")
    for test_id in shared_test_ids:
        test_case = tests_by_id[test_id]
        lines.extend(
            [
                "    GeneratedProfileTestCase {",
                f"        test_id: {rust_string_literal(test_id)},",
                f"        official_case_id: {_rust_string_field(test_case, 'official_case_id')},",
                f"        mandatory_status: {_rust_string_field(test_case, 'mandatory_status')},",
                f"        fixture_availability: {_rust_string_field(test_case, 'fixture_availability')},",
                "    },",
            ]
        )
    lines.extend(["];", ""])

    source_clause_id_pool = sorted(
        {
            source_clause_id
            for profile in profiles
            for source_clause_id in profile.get("source_clause_ids", [])
        }
    )
    requirement_id_pool = sorted(
        {
            requirement_id
            for profile in profiles
            for requirement_id in profile.get("requirement_ids", [])
        }
    )
    lines.extend(
        [
            "",
            "/// Unique sorted clause IDs used by compact profile closure bitmaps.",
            "const PROFILE_SOURCE_CLAUSE_ID_POOL: &[&str] = &[",
            *[f"    {rust_string_literal(value)}," for value in source_clause_id_pool],
            "];",
            "",
            "/// Unique sorted requirement IDs used by compact profile closure bitmaps.",
            "const PROFILE_REQUIREMENT_ID_POOL: &[&str] = &[",
            *[f"    {rust_string_literal(value)}," for value in requirement_id_pool],
            "];",
            "",
        ]
    )
    for index, profile in enumerate(profiles):
        for field, constant, pool, universe in (
            ("source_clause_ids", "SOURCE_CLAUSE", "PROFILE_SOURCE_CLAUSE_ID_POOL", source_clause_id_pool),
            ("requirement_ids", "REQUIREMENT", "PROFILE_REQUIREMENT_ID_POOL", requirement_id_pool),
        ):
            values = profile[field]
            bitmap_name = f"PROFILE_{constant}_IDS_{index}_BITMAP"
            array_name = f"PROFILE_{constant}_IDS_{index}"
            encoded = _encode_profile_id_bitmap(values, universe)
            lines.extend(
                [
                    f"const {bitmap_name}: &str = {rust_string_literal(encoded)};",
                    f"const {array_name}: [&str; {len(values)}] =",
                    f"    decode_profile_id_set::<{len(values)}>({pool}, {bitmap_name});",
                    f"const _: [(); {len(values)}] = [(); profile_id_set_len({pool}, {bitmap_name})];",
                    "",
                ]
            )

    lines.append("")
    lines.append("/// Client profile metadata sorted by stable profile identifier.")
    lines.append("pub const CLIENT_PROFILES: &[GeneratedClientProfile] = &[")
    for profile_index, profile in enumerate(profiles):
        profile_id = profile.get("profile_id")
        if not isinstance(profile_id, str) or not profile_id:
            raise ProfileGenerationError("client profile lacks a stable identity")
        direct_clause_ids = profile.get("direct_source_clause_ids")
        closure = compute_profile_closure(
            catalog,
            direct_clause_ids,
            profile_id=profile_id,
        )
        scope_blockers = [
            "\n".join(
                [
                    "            GeneratedProfileScopeBlocker {",
                    f"                requirement_id: {rust_string_literal(obligation.requirement_id)},",
                    f"                source_clause_id: {rust_string_literal(obligation.source_clause_id)},",
                    f"                scope_state: {rust_string_literal(obligation.scope_state)},",
                    "            },",
                ]
            )
            for obligation in closure.unresolved_scope_obligations
        ]
        test_ids = test_ids_by_profile[profile_id]
        test_case_references = [
            f"            PROFILE_TEST_CASES[{test_case_indexes[test_id]}],"
            for test_id in test_ids
        ]
        lines.extend(
            [
                "    GeneratedClientProfile {",
                f"        profile_id: {rust_string_literal(profile_id)},",
                f"        name: {_rust_string_field(profile, 'name')},",
                f"        applicability: {_rust_string_field(profile, 'applicability')},",
                f"        claim_state: {_rust_string_field(profile, 'claim_state')},",
                f"        selected_target: {str(profile_id in selected_ids).lower()},",
                f"        direct_source_clause_ids: {_rust_string_array(profile, 'direct_source_clause_ids')},",
                f"        source_clause_ids: &PROFILE_SOURCE_CLAUSE_IDS_{profile_index},",
                f"        requirement_ids: &PROFILE_REQUIREMENT_IDS_{profile_index},",
                "        unresolved_scope_obligations: &[",
                *scope_blockers,
                "        ],",
                f"        dependency_profile_ids: {_rust_string_array(profile, 'dependency_profile_ids')},",
                f"        transport_requirements: {_rust_string_array(profile, 'transport_requirements')},",
                f"        encoding_requirements: {_rust_string_array(profile, 'encoding_requirements')},",
                "        test_cases: &[",
                *test_case_references,
                "        ],",
                "    },",
            ]
        )
    lines.extend(["];", ""])
    try:
        formatted = subprocess.run(
            ["rustfmt", "--edition", "2024", "--emit", "stdout"],
            input="\n".join(lines),
            capture_output=True,
            check=False,
            text=True,
            encoding="utf-8",
        )
    except OSError as error:
        raise ProfileGenerationError("rustfmt is unavailable for deterministic profile output") from error
    if formatted.returncode != 0:
        raise ProfileGenerationError("rustfmt rejected generated profile metadata")
    return formatted.stdout


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write generated client profile metadata")
    mode.add_argument("--check", action="store_true", help="fail if generated metadata is stale")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    arguments = parser.parse_args(argv)
    try:
        repository = arguments.repo_root.resolve(strict=True)
        raw_catalog = safe_read_bytes(repository, CATALOG_PATH, max_bytes=MAX_CATALOG_BYTES)
        if __package__:
            from .validate import load_validated_catalog
        else:
            from validate import load_validated_catalog
        catalog = load_validated_catalog(raw_catalog, repository, require_complete=True)
        targets = load_profile_targets(repository, catalog)
        expected = render_profiles_source(catalog, targets).encode("utf-8")
        if arguments.check:
            current = safe_read_bytes(repository, PROFILE_OUTPUT_PATH, max_bytes=MAX_CATALOG_BYTES)
            if current != expected:
                print("generated profile metadata is stale; run generate_profiles.py --write", file=sys.stderr)
                return 1
        else:
            atomic_write_bytes(repository, PROFILE_OUTPUT_PATH, expected)
    except (OSError, PathSecurityError, ProfileGenerationError, ValueError) as error:
        print(f"profile metadata generation failed: {error}", file=sys.stderr)
        return 1
    print(f"Profile metadata {'verified' if arguments.check else 'written'}: {PROFILE_OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
