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


class ProfileGenerationError(ValueError):
    """Raised when profile metadata cannot be expanded without ambiguity."""


@dataclass(frozen=True)
class ProfileClosure:
    """The deterministic normative clause and applicable requirement closure."""

    source_clause_ids: tuple[str, ...]
    requirement_ids: tuple[str, ...]


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

    Only client-applicable 1.0 and conditionally applicable requirements are
    included. Server-only and out-of-scope clause records remain in the source
    clause closure for reporting.
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
    for requirement in raw_requirements:
        if not isinstance(requirement, dict):
            raise ProfileGenerationError("catalog has an invalid requirement record")
        requirement_id = requirement.get("requirement_id")
        linked_clauses = requirement.get("source_clause_ids")
        if not isinstance(requirement_id, str) or not requirement_id or not isinstance(linked_clauses, list):
            raise ProfileGenerationError("catalog has an invalid requirement identity or clause links")
        scope_state = requirement.get("scope_state")
        in_scope = scope_state == "client_1_0" or (
            scope_state == "profile_conditional"
            and profile_id in requirement.get("profile_ids", [])
        )
        if (
            requirement.get("role") in {"client", "both"}
            and in_scope
            and any(clause_id in source_clause_ids for clause_id in linked_clauses)
        ):
            requirement_ids.add(requirement_id)

    return ProfileClosure(
        tuple(sorted(source_clause_ids)),
        tuple(sorted(requirement_ids)),
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


def render_profiles_source(catalog: Mapping[str, Any], targets: ProfileTargets) -> str:
    """Render deterministic immutable client metadata without executable rules."""
    raw_profiles = catalog.get("profiles")
    raw_test_cases = catalog.get("test_cases")
    if not isinstance(raw_profiles, list) or not isinstance(raw_test_cases, list):
        raise ProfileGenerationError("catalog has invalid profile or test records")
    tests_by_id = {
        test_case.get("test_id"): test_case
        for test_case in raw_test_cases
        if isinstance(test_case, dict) and isinstance(test_case.get("test_id"), str)
    }
    profiles = sorted(
        (profile for profile in raw_profiles if isinstance(profile, dict) and profile.get("role") == "client"),
        key=lambda profile: profile.get("profile_id", ""),
    )
    if not profiles:
        raise ProfileGenerationError("catalog contains no client profiles")

    def string_field(record: Mapping[str, Any], field: str) -> str:
        value = record.get(field)
        if not isinstance(value, str) or not value:
            raise ProfileGenerationError(f"profile metadata field {field} must be non-empty text")
        return rust_string_literal(value)

    def string_array(record: Mapping[str, Any], field: str) -> str:
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
        "/// Client profiles sorted by stable profile identifier.",
        "pub const CLIENT_PROFILES: &[GeneratedClientProfile] = &[",
    ]
    selected_ids = set(targets.profile_ids)
    for profile in profiles:
        profile_id = profile.get("profile_id")
        if not isinstance(profile_id, str) or not profile_id:
            raise ProfileGenerationError("client profile lacks a stable identity")
        test_ids = profile.get("test_case_ids")
        if (
            not isinstance(test_ids, list)
            or any(not isinstance(test_id, str) or test_id not in tests_by_id for test_id in test_ids)
            or len(test_ids) != len(set(test_ids))
        ):
            raise ProfileGenerationError("client profile has unresolved or duplicate test identifiers")
        tests = []
        for test_id in test_ids:
            test_case = tests_by_id[test_id]
            tests.append(
                "\n".join(
                    [
                        "            GeneratedProfileTestCase {",
                        f"                test_id: {rust_string_literal(test_id)},",
                        f"                official_case_id: {string_field(test_case, 'official_case_id')},",
                        f"                mandatory_status: {string_field(test_case, 'mandatory_status')},",
                        f"                fixture_availability: {string_field(test_case, 'fixture_availability')},",
                        "            },",
                    ]
                )
            )
        lines.extend(
            [
                "    GeneratedClientProfile {",
                f"        profile_id: {rust_string_literal(profile_id)},",
                f"        name: {string_field(profile, 'name')},",
                f"        applicability: {string_field(profile, 'applicability')},",
                f"        claim_state: {string_field(profile, 'claim_state')},",
                f"        selected_target: {str(profile_id in selected_ids).lower()},",
                f"        direct_source_clause_ids: {string_array(profile, 'direct_source_clause_ids')},",
                f"        source_clause_ids: {string_array(profile, 'source_clause_ids')},",
                f"        requirement_ids: {string_array(profile, 'requirement_ids')},",
                f"        dependency_profile_ids: {string_array(profile, 'dependency_profile_ids')},",
                f"        transport_requirements: {string_array(profile, 'transport_requirements')},",
                f"        encoding_requirements: {string_array(profile, 'encoding_requirements')},",
                "        test_cases: &[",
                *tests,
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
