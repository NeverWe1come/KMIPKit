"""Generate deterministic Rust metadata for the reviewed KMIP client profiles."""

from __future__ import annotations

import re
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import Any


_SECTION = re.compile(r"[0-9]+(?:\.[0-9]+)*\Z")
_APPLICABLE_CLIENT_SCOPES = {"client_1_0", "profile_conditional"}


class ProfileGenerationError(ValueError):
    """Raised when profile metadata cannot be expanded without ambiguity."""


@dataclass(frozen=True)
class ProfileClosure:
    """The deterministic normative clause and applicable requirement closure."""

    source_clause_ids: tuple[str, ...]
    requirement_ids: tuple[str, ...]


def _section_in_scope(section: str, included_section: str | None) -> bool:
    return included_section is None or section == included_section or section.startswith(f"{included_section}.")


def compute_profile_closure(
    catalog: Mapping[str, Any],
    direct_source_clause_ids: Sequence[str],
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
    if not direct_source_clause_ids or any(not isinstance(value, str) or not value for value in direct_source_clause_ids):
        raise ProfileGenerationError("profile has no valid direct source clauses")

    clauses_by_id: dict[str, dict[str, Any]] = {}
    clauses: list[dict[str, Any]] = []
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
        clauses.append(clause)

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
            clause for clause in clauses
            if clause["source_id"] == source_id and _section_in_scope(clause["section"], included_section)
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
        if (
            requirement.get("role") in {"client", "both"}
            and requirement.get("scope_state") in _APPLICABLE_CLIENT_SCOPES
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
