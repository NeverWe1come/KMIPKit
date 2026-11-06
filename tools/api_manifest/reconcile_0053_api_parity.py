"""Reconcile reviewed four-language mappings for KMIPKIT-0053 candidates."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path, PurePosixPath
from typing import Any

SPEC_ID = "KMIPKIT-0053"
LANGUAGE_PACKAGES = {
    "rust": {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client"},
    "c": {"c-kmipkit"},
    "java": {"java-kmipkit"},
    "python": {"python-kmipkit"},
}
REQUIRED_LANGUAGES = frozenset(LANGUAGE_PACKAGES)
DECLARATION_RESOLUTION_COMMON_FIELDS = frozenset({
    "declarationId",
    "capabilityId",
    "packageId",
    "exportName",
    "resolution",
    "evidencePaths",
    "approvalSource",
})
TYPE_SYSTEM_EXCLUSION_REASONS = {
    "rust-trait": (
        "This Rust trait is a compile-time abstraction; its runtime behavior is "
        "exercised through the exact mapped declarations listed as counterparts."
    ),
    "rust-associated-type": (
        "This Rust associated type is a compile-time value contract; its values "
        "are represented by the exact mapped typed-value declarations listed as counterparts."
    ),
    "rust-generic-type": (
        "This Rust generic type parameterization has no distinct foreign runtime type; "
        "the exact mapped foreign wrapper listed as counterpart preserves its behavior."
    ),
    "rust-enum-variant": (
        "This Rust enum variant is a type-system representation detail; its runtime "
        "value is represented by the exact mapped parent model listed as counterpart."
    ),
    "rust-enum-field": (
        "This Rust enum payload field is a type-system representation detail; its runtime "
        "value is represented by the exact mapped parent model listed as counterpart."
    ),
    "rust-module": (
        "This Rust module is a namespace boundary only; its runtime API is represented by "
        "the exact mapped declarations listed as counterparts."
    ),
}
TRAIT_PROJECTION_REASON = (
    "This exact compiler-visible Rust trait projection is language-level behavior "
    "of the separately mapped KMIPKit model and has no independent foreign API route."
)


class ReconciliationError(ValueError):
    """Raised when a candidate mapping or exclusion is incomplete or too broad."""


def _validate_mapping(declaration: dict[str, Any], route: dict[str, Any]) -> dict[str, dict[str, str]]:
    if "exclusion" in route:
        raise ReconciliationError("exclusions cannot be interpreted as language mappings")
    if set(route) != {*REQUIRED_LANGUAGES, "parityEvidenceIds", "approvalSource"}:
        raise ReconciliationError("each mapped route must name all four languages, evidence, and approval")
    mappings: dict[str, dict[str, str]] = {}
    for language in sorted(REQUIRED_LANGUAGES):
        mapping = route.get(language)
        if not isinstance(mapping, dict) or set(mapping) != {"packageId", "name"}:
            raise ReconciliationError(f"{language} route must contain exactly packageId and name")
        package_id = mapping.get("packageId")
        name = mapping.get("name")
        if package_id not in LANGUAGE_PACKAGES[language] or not isinstance(name, str) or not name.strip():
            raise ReconciliationError(f"{language} route is not an exact public declaration mapping")
        mappings[language] = {"packageId": package_id, "name": name}
    evidence = route.get("parityEvidenceIds")
    if (
        not isinstance(evidence, list)
        or not evidence
        or any(not isinstance(path, str) or not path.strip() for path in evidence)
        or len(evidence) != len(set(evidence))
    ):
        raise ReconciliationError("every mapping needs unique, non-empty behavior evidence paths")
    approval = route.get("approvalSource")
    if (
        not isinstance(approval, dict)
        or set(approval) != {"kind", "path", "status", "evidence"}
        or any(not isinstance(value, str) or not value.strip() for value in approval.values())
        or approval.get("kind") != "feature-specification"
        or approval.get("status") != "Approved"
    ):
        raise ReconciliationError("every mapping needs exact feature approval evidence")
    return mappings


def _validate_exclusion(declaration: dict[str, Any], route: dict[str, Any]) -> dict[str, Any]:
    exclusion = route.get("exclusion")
    if set(route) != {"exclusion"} or not isinstance(exclusion, dict):
        raise ReconciliationError("an exclusion must be the only route entry")
    if declaration.get("kind") != "associated-item" or not declaration.get("exportName", "").startswith("<"):
        raise ReconciliationError("only one exact compiler-visible trait projection may be excluded")
    if set(exclusion) != {"reason", "evidence"}:
        raise ReconciliationError("an exclusion needs its exact reason and evidence")
    reason = exclusion.get("reason")
    evidence = exclusion.get("evidence")
    if reason != TRAIT_PROJECTION_REASON:
        raise ReconciliationError("the exclusion reason does not justify this exact trait projection")
    if not isinstance(evidence, list) or not evidence or any(
        not isinstance(path, str) or not path.strip() for path in evidence
    ):
        raise ReconciliationError("an exact exclusion needs evidence paths")
    return {
        "packageId": declaration["packageId"],
        "exportName": declaration["exportName"],
        "reason": reason,
        "approvalSource": {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/spec.md",
            "status": "Approved",
            "evidence": "Approved by the maintainer's direct approval of the Phase 5 plan",
        },
        "evidence": "; ".join(evidence),
        "candidateSpecId": SPEC_ID,
    }


def _validate_resolution_identity(
    declaration: dict[str, Any], resolution: dict[str, Any]
) -> None:
    if (
        resolution.get("declarationId") != declaration.get("declarationId")
        or resolution.get("capabilityId") != declaration.get("capabilityId")
        or resolution.get("packageId") != declaration.get("packageId")
        or resolution.get("exportName") != declaration.get("exportName")
    ):
        raise ReconciliationError(
            "candidate resolution must repeat the exact declaration ID, capability, and export name"
        )
    if declaration.get("candidateSpecId") != SPEC_ID or declaration.get("declarationState") != "blocked":
        raise ReconciliationError("only blocked KMIPKIT-0053 candidates may be resolved")


def _validate_resolution_evidence(resolution: dict[str, Any]) -> list[str]:
    evidence = resolution.get("evidencePaths")
    if not isinstance(evidence, list) or not evidence or any(
        not isinstance(path, str) or not path.strip() for path in evidence
    ):
        raise ReconciliationError("every declaration resolution needs unique evidence paths")
    if len(evidence) != len(set(evidence)):
        raise ReconciliationError("every declaration resolution needs unique evidence paths")
    for path in evidence:
        parsed = PurePosixPath(path)
        if parsed.is_absolute() or ".." in parsed.parts or any(char in path for char in "*?"):
            raise ReconciliationError("evidence paths must be exact repository-relative paths")

    approval = resolution.get("approvalSource")
    if (
        not isinstance(approval, dict)
        or set(approval) != {"kind", "path", "status", "evidence"}
        or any(not isinstance(value, str) or not value.strip() for value in approval.values())
        or approval.get("kind") != "feature-specification"
        or approval.get("status") != "Approved"
    ):
        raise ReconciliationError("every declaration resolution needs an approved feature source")
    return evidence


def _candidate_parent(
    declaration: dict[str, Any], declarations_by_export: dict[tuple[str, str], dict[str, Any]]
) -> dict[str, Any] | None:
    export_name = declaration.get("exportName")
    if not isinstance(export_name, str) or "::" not in export_name:
        return None
    parent_name = export_name.rsplit("::", maxsplit=1)[0]
    return declarations_by_export.get((declaration.get("packageId", ""), parent_name))


def _validate_type_system_exclusion(
    declaration: dict[str, Any],
    resolution: dict[str, Any],
    declarations_by_export: dict[tuple[str, str], dict[str, Any]],
) -> tuple[dict[str, Any], list[str]]:
    exclusion = resolution.get("exclusion")
    if not isinstance(exclusion, dict) or set(exclusion) != {
        "artifactKind",
        "counterpartDeclarationIds",
    }:
        raise ReconciliationError("an exact type-system exclusion needs an artifact and counterpart IDs")
    artifact_kind = exclusion.get("artifactKind")
    if artifact_kind not in TYPE_SYSTEM_EXCLUSION_REASONS:
        raise ReconciliationError("the exclusion kind is not an approved Rust type-system artifact")

    declaration_kind = declaration.get("kind")
    export_name = declaration.get("exportName", "")
    valid_artifact = False
    parent: dict[str, Any] | None = None
    if artifact_kind == "rust-trait":
        valid_artifact = declaration_kind == "trait"
    elif artifact_kind == "rust-associated-type":
        valid_artifact = (
            declaration_kind == "associated-item"
            and export_name.rsplit("::", maxsplit=1)[-1] in {"Value", "ValueView"}
        )
        parent = _candidate_parent(declaration, declarations_by_export)
        valid_artifact = valid_artifact and parent is not None and parent.get("kind") == "trait"
    elif artifact_kind == "rust-generic-type":
        valid_artifact = declaration_kind == "type" and "<" in export_name and ">" in export_name
    elif artifact_kind == "rust-enum-variant":
        valid_artifact = declaration_kind == "variant"
        parent = _candidate_parent(declaration, declarations_by_export)
        valid_artifact = (
            valid_artifact
            and parent is not None
            and parent.get("kind") == "type"
            and not str(parent.get("exportName", "")).endswith("Error")
        )
    elif artifact_kind == "rust-enum-field":
        valid_artifact = declaration_kind == "field"
        parent = _candidate_parent(declaration, declarations_by_export)
        enum_type = _candidate_parent(parent, declarations_by_export) if parent is not None else None
        valid_artifact = (
            valid_artifact
            and parent is not None
            and parent.get("kind") == "variant"
            and enum_type is not None
            and enum_type.get("kind") == "type"
            and not str(enum_type.get("exportName", "")).endswith("Error")
        )
    elif artifact_kind == "rust-module":
        valid_artifact = declaration_kind == "module"
    if not valid_artifact:
        raise ReconciliationError("the exclusion does not match this exact Rust type-system declaration")

    counterpart_ids = exclusion.get("counterpartDeclarationIds")
    if (
        not isinstance(counterpart_ids, list)
        or not counterpart_ids
        or any(not isinstance(identifier, str) or not identifier or any(c in identifier for c in "*?")
               for identifier in counterpart_ids)
    ):
        raise ReconciliationError("type-system exclusions require exact counterpart declaration IDs")
    if len(counterpart_ids) != len(set(counterpart_ids)):
        raise ReconciliationError("type-system exclusions require unique counterpart declaration IDs")
    return exclusion, counterpart_ids


def _required_parent_counterpart(
    declaration: dict[str, Any],
    artifact_kind: str,
    declarations_by_export: dict[tuple[str, str], dict[str, Any]],
) -> str | None:
    if artifact_kind == "rust-enum-variant":
        parent = _candidate_parent(declaration, declarations_by_export)
        return parent.get("declarationId") if parent is not None else None
    if artifact_kind == "rust-enum-field":
        variant = _candidate_parent(declaration, declarations_by_export)
        enum_type = _candidate_parent(variant, declarations_by_export) if variant is not None else None
        return enum_type.get("declarationId") if enum_type is not None else None
    return None


def _resolution_exclusion_record(
    declaration: dict[str, Any],
    resolution: dict[str, Any],
    exclusion: dict[str, Any],
    evidence: list[str],
) -> dict[str, Any]:
    artifact_kind = exclusion["artifactKind"]
    return {
        "declarationId": declaration["declarationId"],
        "capabilityId": declaration["capabilityId"],
        "packageId": declaration["packageId"],
        "exportName": declaration["exportName"],
        "declarationKind": declaration["kind"],
        "artifactKind": artifact_kind,
        "reason": TYPE_SYSTEM_EXCLUSION_REASONS[artifact_kind],
        "counterpartDeclarationIds": list(exclusion["counterpartDeclarationIds"]),
        "approvalSource": copy.deepcopy(resolution["approvalSource"]),
        "evidence": "; ".join(evidence),
        "evidencePaths": list(evidence),
        "candidateSpecId": SPEC_ID,
    }


def reconcile_candidate_resolutions(
    manifest: dict[str, Any], resolutions: list[dict[str, Any]]
) -> dict[str, Any]:
    """Resolve exact KMIPKIT-0053 candidates by route mapping or narrow type-system exclusion."""
    if not isinstance(resolutions, list):
        raise ReconciliationError("candidate resolutions must be a list")
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ReconciliationError("manifest lacks publicDeclarations")
    candidates = [row for row in declarations if row.get("candidateSpecId") == SPEC_ID]
    candidates_by_id = {row.get("declarationId"): row for row in candidates}
    all_by_id = {row.get("declarationId"): row for row in declarations}
    by_export = {
        (row.get("packageId", ""), row.get("exportName", "")): row
        for row in declarations
    }
    identifiers = [row.get("declarationId") for row in resolutions if isinstance(row, dict)]
    if len(identifiers) != len(resolutions) or any(
        not isinstance(identifier, str) or not identifier or any(c in identifier for c in "*?")
        for identifier in identifiers
    ):
        raise ReconciliationError("each resolution needs one exact stable declaration ID")
    if len(identifiers) != len(set(identifiers)):
        raise ReconciliationError("a declaration may be resolved only once per reconciliation")

    mapping_ids: set[str] = set()
    exclusion_records: list[dict[str, Any]] = []
    for resolution in resolutions:
        if not isinstance(resolution, dict):
            raise ReconciliationError("candidate resolution has missing or unrecognized fields")
        resolution_kind = resolution.get("resolution")
        expected_fields = DECLARATION_RESOLUTION_COMMON_FIELDS | (
            {"languageMappings"} if resolution_kind == "mapped" else {"exclusion"}
        )
        if set(resolution) != expected_fields:
            raise ReconciliationError("candidate resolution has missing or unrecognized fields")
        identifier = resolution["declarationId"]
        declaration = candidates_by_id.get(identifier)
        if declaration is None:
            existing = all_by_id.get(identifier)
            if existing is not None and existing.get("declarationState") == "verified":
                _validate_resolution_identity_existing(existing, resolution)
                route = _mapping_route_from_resolution(resolution)
                mappings = _validate_mapping(existing, route)
                if (
                    resolution_kind != "mapped"
                    or mappings["rust"]["name"] != existing.get("exportName")
                    or mappings != existing.get("languageMappings")
                    or resolution.get("evidencePaths") != existing.get("parityEvidenceIds")
                    or resolution.get("approvalSource") != existing.get("approvalSource")
                ):
                    raise ReconciliationError("replayed mapped resolution differs from verified declaration")
                mapping_ids.add(identifier)
                continue
            existing_exclusion = next(
                (row for row in result.get("approvedExclusions", [])
                 if row.get("declarationId") == identifier),
                None,
            )
            if existing_exclusion is not None:
                _validate_resolution_identity_existing(existing_exclusion, resolution)
                expected = _resolution_exclusion_replay_record(resolution)
                if existing_exclusion != expected:
                    raise ReconciliationError("replayed exclusion differs from approved record")
                for counterpart_id in expected["counterpartDeclarationIds"]:
                    counterpart = all_by_id.get(counterpart_id)
                    if (
                        counterpart is None
                        or counterpart.get("declarationState") != "verified"
                        or counterpart.get("capabilityId") != existing_exclusion.get("capabilityId")
                    ):
                        raise ReconciliationError("replayed exclusion counterpart is no longer mapped")
                continue
            raise ReconciliationError("candidate resolution references no exact 0053 candidate")

        _validate_resolution_identity(declaration, resolution)
        evidence = _validate_resolution_evidence(resolution)
        if resolution_kind == "mapped":
            route = _mapping_route_from_resolution(resolution)
            mappings = _validate_mapping(declaration, route)
            if (
                mappings["rust"]["packageId"] != declaration["packageId"]
                or mappings["rust"]["name"] != declaration["exportName"]
            ):
                raise ReconciliationError("Rust mapping must exactly match the candidate export")
            updated = copy.deepcopy(declaration)
            updated["declarationState"] = "verified"
            updated["languageMappings"] = mappings
            updated["parityEvidenceIds"] = evidence
            updated["approvalSource"] = copy.deepcopy(resolution["approvalSource"])
            updated.pop("candidateSpecId", None)
            updated.pop("blockedReason", None)
            all_by_id[identifier] = updated
            by_export[(updated["packageId"], updated["exportName"])] = updated
            mapping_ids.add(identifier)
        elif resolution_kind == "excluded":
            exclusion, _ = _validate_type_system_exclusion(declaration, resolution, by_export)
            exclusion_records.append(
                _resolution_exclusion_record(declaration, resolution, exclusion, evidence)
            )
        else:
            raise ReconciliationError("resolution must be mapped or excluded")

    for resolution in resolutions:
        if resolution["resolution"] != "excluded":
            continue
        declaration_id = resolution["declarationId"]
        declaration = candidates_by_id.get(declaration_id)
        if declaration is None:
            continue
        exclusion, counterpart_ids = _validate_type_system_exclusion(
            declaration, resolution, by_export
        )
        required_parent = _required_parent_counterpart(
            declaration, exclusion["artifactKind"], by_export
        )
        if required_parent is not None and required_parent not in counterpart_ids:
            raise ReconciliationError("enum-shape exclusion must name its exact mapped parent model")
        for counterpart_id in counterpart_ids:
            counterpart = all_by_id.get(counterpart_id)
            if counterpart is None or counterpart.get("capabilityId") != declaration["capabilityId"]:
                raise ReconciliationError("exclusion counterpart must be an exact same-capability declaration")
            if counterpart.get("declarationState") != "verified" or counterpart_id not in mapping_ids:
                raise ReconciliationError("exclusion counterpart must have an exact verified four-language mapping")

    processed_candidate_ids = set(identifiers) & set(candidates_by_id)
    result["publicDeclarations"] = [
        row for row in declarations if row.get("declarationId") not in processed_candidate_ids
    ] + [
        all_by_id[identifier]
        for identifier in identifiers
        if identifier in mapping_ids and identifier in processed_candidate_ids
    ]

    existing_exclusions = result.setdefault("approvedExclusions", [])
    existing_exclusion_ids = {row.get("declarationId") for row in existing_exclusions}
    for exclusion in exclusion_records:
        if exclusion["declarationId"] in existing_exclusion_ids:
            raise ReconciliationError("candidate exclusion already exists in approvedExclusions")
        existing_exclusions.append(exclusion)
        existing_exclusion_ids.add(exclusion["declarationId"])

    capabilities = result.get("capabilities")
    if not isinstance(capabilities, list):
        raise ReconciliationError("manifest lacks capabilities")
    capability_by_id = {row.get("capabilityId"): row for row in capabilities}
    for capability_id in {resolution["capabilityId"] for resolution in resolutions}:
        capability = capability_by_id.get(capability_id)
        if capability is None:
            raise ReconciliationError("candidate capability is missing from manifest")
        unresolved = any(
            row.get("candidateSpecId") == SPEC_ID and row.get("capabilityId") == capability_id
            for row in result["publicDeclarations"]
        )
        if unresolved or capability.get("capabilityState") == "verified":
            continue
        if capability.get("capabilityState") != "blocked":
            raise ReconciliationError("candidate capability is not blocked pending reconciliation")
        mapped_rows = [
            row for row in result["publicDeclarations"]
            if row.get("capabilityId") == capability_id and row.get("declarationState") == "verified"
        ]
        if not mapped_rows:
            raise ReconciliationError("a capability cannot be verified by exclusions without mapped behavior")
        capability["capabilityState"] = "verified"
        capability["mappings"] = {
            language: sorted({row["languageMappings"][language]["name"] for row in mapped_rows})
            for language in sorted(REQUIRED_LANGUAGES)
        }
        mapped_resolution = next(
            (resolution for resolution in resolutions
             if resolution["resolution"] == "mapped"
             and resolution["capabilityId"] == capability_id),
            None,
        )
        if mapped_resolution is not None:
            capability["ownerSpecId"] = SPEC_ID
            capability["approvalEvidence"] = copy.deepcopy(mapped_resolution["approvalSource"])
            capability["parityEvidenceIds"] = sorted({
                path for resolution in resolutions
                if resolution["resolution"] == "mapped"
                and resolution["capabilityId"] == capability_id
                for path in resolution["evidencePaths"]
            })
        capability.pop("candidateSpecId", None)
        capability.pop("blockedReason", None)
    return result


def _mapping_route_from_resolution(resolution: dict[str, Any]) -> dict[str, Any]:
    mappings = resolution.get("languageMappings")
    if not isinstance(mappings, dict):
        raise ReconciliationError("mapped candidate needs all four exact language mappings")
    return {
        **mappings,
        "parityEvidenceIds": resolution.get("evidencePaths"),
        "approvalSource": resolution.get("approvalSource"),
    }


def _validate_resolution_identity_existing(
    declaration: dict[str, Any], resolution: dict[str, Any]
) -> None:
    if (
        resolution.get("declarationId") != declaration.get("declarationId")
        or resolution.get("capabilityId") != declaration.get("capabilityId")
        or resolution.get("packageId") != declaration.get("packageId")
        or resolution.get("exportName") != declaration.get("exportName")
    ):
        raise ReconciliationError("replayed resolution identity does not match its exact record")


def _resolution_exclusion_replay_record(resolution: dict[str, Any]) -> dict[str, Any]:
    exclusion = resolution.get("exclusion")
    if not isinstance(exclusion, dict) or set(exclusion) != {
        "artifactKind",
        "counterpartDeclarationIds",
    }:
        raise ReconciliationError("replayed exclusion is incomplete")
    artifact_kind = exclusion.get("artifactKind")
    if artifact_kind not in TYPE_SYSTEM_EXCLUSION_REASONS:
        raise ReconciliationError("replayed exclusion kind is not approved")
    evidence = _validate_resolution_evidence(resolution)
    counterpart_ids = exclusion.get("counterpartDeclarationIds")
    if (
        not isinstance(counterpart_ids, list)
        or not counterpart_ids
        or any(not isinstance(identifier, str) or not identifier or any(char in identifier for char in "*?")
               for identifier in counterpart_ids)
    ):
        raise ReconciliationError("replayed exclusion needs exact counterpart IDs")
    if len(counterpart_ids) != len(set(counterpart_ids)):
        raise ReconciliationError("replayed exclusion needs unique exact counterpart IDs")
    return {
        "declarationId": resolution["declarationId"],
        "capabilityId": resolution["capabilityId"],
        "packageId": resolution["packageId"],
        "exportName": resolution["exportName"],
        "declarationKind": _replayed_exclusion_declaration_kind(artifact_kind),
        "artifactKind": artifact_kind,
        "reason": TYPE_SYSTEM_EXCLUSION_REASONS[artifact_kind],
        "counterpartDeclarationIds": list(counterpart_ids),
        "approvalSource": copy.deepcopy(resolution["approvalSource"]),
        "evidence": "; ".join(evidence),
        "evidencePaths": list(evidence),
        "candidateSpecId": SPEC_ID,
    }


def _replayed_exclusion_declaration_kind(artifact_kind: str) -> str:
    return {
        "rust-trait": "trait",
        "rust-associated-type": "associated-item",
        "rust-generic-type": "type",
        "rust-enum-variant": "variant",
        "rust-enum-field": "field",
        "rust-module": "module",
    }[artifact_kind]


def reconcile_manifest(
    manifest: dict[str, Any], routes: dict[str, Any]
) -> dict[str, Any]:
    """Return an updated manifest copy after exact route and evidence checks."""
    if not isinstance(routes, dict):
        raise ReconciliationError("route mapping must be an object keyed by declarationId")
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    capabilities = result.get("capabilities")
    if not isinstance(declarations, list) or not isinstance(capabilities, list):
        raise ReconciliationError("manifest lacks publicDeclarations or capabilities")
    candidates = [row for row in declarations if row.get("candidateSpecId") == SPEC_ID]
    candidate_ids = [row.get("declarationId") for row in candidates]
    if any(not isinstance(identifier, str) or not identifier for identifier in candidate_ids):
        raise ReconciliationError("every candidate needs a stable declarationId")
    if len(candidate_ids) != len(set(candidate_ids)):
        raise ReconciliationError("candidate declaration IDs must be unique")
    candidate_by_id = {row["declarationId"]: row for row in candidates}
    existing_by_id = {row.get("declarationId"): row for row in declarations}
    extra = sorted(set(routes) - set(candidate_ids) - set(existing_by_id))
    if extra:
        raise ReconciliationError(f"route inventory contains unowned IDs (extra={len(extra)})")

    by_capability: dict[str, dict[str, list[str]]] = {}
    verified: list[dict[str, Any]] = []
    exclusions: list[dict[str, Any]] = []
    for declaration_id, route in routes.items():
        declaration = candidate_by_id.get(declaration_id)
        if declaration is None:
            existing = existing_by_id[declaration_id]
            expected = route.get("exclusion") if isinstance(route, dict) else None
            if expected is not None or existing.get("declarationState") != "verified":
                raise ReconciliationError("replayed routes must match an already verified declaration")
            mappings = _validate_mapping(existing, route)
            if mappings != existing.get("languageMappings") or route["parityEvidenceIds"] != existing.get("parityEvidenceIds"):
                raise ReconciliationError("replayed route differs from its verified declaration")
            continue
        if declaration.get("declarationState") != "blocked":
            raise ReconciliationError("unreconciled 0053 candidates must remain blocked")
        if not isinstance(route, dict):
            raise ReconciliationError("each route must be an object")
        if "exclusion" in route:
            exclusions.append(_validate_exclusion(declaration, route))
            continue
        mappings = _validate_mapping(declaration, route)
        declaration["declarationState"] = "verified"
        declaration["languageMappings"] = mappings
        declaration["parityEvidenceIds"] = route["parityEvidenceIds"]
        declaration["approvalSource"] = route["approvalSource"]
        declaration.pop("candidateSpecId", None)
        declaration.pop("blockedReason", None)
        verified.append(declaration)
        language_names = by_capability.setdefault(declaration["capabilityId"], {
            language: [] for language in REQUIRED_LANGUAGES
        })
        for language, mapping in mappings.items():
            language_names[language].append(mapping["name"])

    exclusion_keys = {(row["packageId"], row["exportName"]) for row in exclusions}
    processed_ids = {row["declarationId"] for row in candidates if row["declarationId"] in routes}
    result["publicDeclarations"] = [
        row for row in declarations
        if row.get("declarationId") not in processed_ids
        if (row.get("packageId"), row.get("exportName")) not in exclusion_keys
    ] + verified
    existing_exclusions = result.setdefault("approvedExclusions", [])
    existing_keys = {(row.get("packageId"), row.get("exportName")) for row in existing_exclusions}
    if any((row["packageId"], row["exportName"]) in existing_keys for row in exclusions):
        raise ReconciliationError("candidate exclusion already exists in approvedExclusions")
    existing_exclusions.extend(exclusions)

    capability_rows = {row.get("capabilityId"): row for row in capabilities}
    candidate_capabilities = {row.get("capabilityId") for row in candidates}
    for capability_id in candidate_capabilities:
        capability = capability_rows.get(capability_id)
        if capability is None:
            raise ReconciliationError("each candidate capability must be present and blocked")
        unresolved = any(
            row.get("candidateSpecId") == SPEC_ID and row.get("capabilityId") == capability_id
            for row in result["publicDeclarations"]
        )
        if unresolved:
            continue
        if capability.get("capabilityState") != "blocked":
            raise ReconciliationError("candidate capability was already reconciled")
        capability["capabilityState"] = "verified"
        all_ready = [
            row for row in result["publicDeclarations"]
            if row.get("capabilityId") == capability_id
            and row.get("declarationState") == "verified"
        ]
        capability["mappings"] = {
            language: sorted({row["languageMappings"][language]["name"] for row in all_ready})
            for language in sorted(REQUIRED_LANGUAGES)
        }
        capability_routes = [
            routes[row["declarationId"]]
            for row in candidates
            if row.get("capabilityId") == capability_id
            and row["declarationId"] in routes
            and "exclusion" not in routes[row["declarationId"]]
        ]
        if capability_routes:
            capability.setdefault("ownerSpecId", SPEC_ID)
            capability["approvalEvidence"] = copy.deepcopy(
                capability_routes[0]["approvalSource"]
            )
            capability["parityEvidenceIds"] = sorted({
                evidence
                for route in capability_routes
                for evidence in route["parityEvidenceIds"]
            })
        capability.pop("candidateSpecId", None)
        capability.pop("blockedReason", None)
    return result


def add_additional_functions(
    manifest: dict[str, Any], additions: list[dict[str, Any]]
) -> dict[str, Any]:
    """Return a copy with exact, reviewed API functions added idempotently."""
    if not isinstance(additions, list):
        raise ReconciliationError("additional API functions must be a list")
    result = copy.deepcopy(manifest)
    functions = result.get("functions")
    if not isinstance(functions, list):
        raise ReconciliationError("manifest lacks its function inventory")
    by_id = {row.get("id"): row for row in functions}
    by_symbol = {
        row.get("c", {}).get("symbol"): row
        for row in functions
        if isinstance(row.get("c"), dict)
    }
    requirement_ids = set(result.get("requirementIds", []))
    additions_by_id: dict[str, dict[str, Any]] = {}
    additions_by_symbol: dict[str, dict[str, Any]] = {}
    for function in additions:
        if not isinstance(function, dict):
            raise ReconciliationError("additional function record must be an object")
        identifier = function.get("id")
        c_record = function.get("c")
        symbol = c_record.get("symbol") if isinstance(c_record, dict) else None
        if not isinstance(identifier, str) or not identifier or not isinstance(symbol, str):
            raise ReconciliationError("additional function needs a stable ID and C symbol")
        if identifier in additions_by_id or symbol in additions_by_symbol:
            raise ReconciliationError("additional function IDs and C symbols must be unique")
        mappings = (function.get("rust"), c_record, function.get("java"), function.get("python"))
        if any(not isinstance(mapping, dict) for mapping in mappings):
            raise ReconciliationError("additional functions must define Rust, C, Java, and Python routes")
        requirements = function.get("requirementIds")
        if not isinstance(requirements, list) or not requirements or any(
            not isinstance(requirement, str) or not requirement for requirement in requirements
        ):
            raise ReconciliationError("additional functions need stable requirement IDs")
        function_requirements = set(requirements)
        for language, mapping in zip(("rust", "c", "java", "python"), mappings, strict=True):
            if not mapping:
                raise ReconciliationError(f"additional function has an empty {language} route")
        if not isinstance(function.get("ordering"), str):
            raise ReconciliationError("additional function must declare its ordering contract")
        additions_by_id[identifier] = function
        additions_by_symbol[symbol] = function
        if identifier in by_id and by_id[identifier] != function:
            raise ReconciliationError(f"additional function ID already has a different definition: {identifier}")
        if symbol in by_symbol and by_symbol[symbol] != function:
            raise ReconciliationError(f"additional C symbol already has a different definition: {symbol}")
        requirement_ids.update(function_requirements)
    for identifier, function in additions_by_id.items():
        if identifier not in by_id:
            functions.append(copy.deepcopy(function))
    result["requirementIds"] = sorted(requirement_ids)
    return result


def add_additional_types(
    manifest: dict[str, Any], additions: list[dict[str, Any]]
) -> dict[str, Any]:
    """Add exact, reviewed four-language C ABI type declarations idempotently."""
    if not isinstance(additions, list):
        raise ReconciliationError("additional API types must be a list")
    result = copy.deepcopy(manifest)
    types = result.get("types")
    if not isinstance(types, list):
        raise ReconciliationError("manifest lacks its type inventory")
    by_id = {row.get("id"): row for row in types}
    additions_by_id: dict[str, dict[str, Any]] = {}
    requirement_ids = set(result.get("requirementIds", []))
    for type_row in additions:
        if not isinstance(type_row, dict):
            raise ReconciliationError("additional API type record must be an object")
        identifier = type_row.get("id")
        if not isinstance(identifier, str) or not identifier.strip():
            raise ReconciliationError("additional API type needs a stable ID")
        if identifier in additions_by_id:
            raise ReconciliationError("additional API type IDs must be unique")
        if set(type_row) != {"id", "kind", "requirementIds", "rust", "c", "java", "python"}:
            raise ReconciliationError("additional API type must define exact Rust, C, Java, and Python mappings")
        if not isinstance(type_row.get("kind"), str) or not type_row["kind"].strip():
            raise ReconciliationError("additional API type needs a kind")
        requirements = type_row.get("requirementIds")
        if not isinstance(requirements, list) or not requirements or any(
            not isinstance(requirement, str) or not requirement.strip() for requirement in requirements
        ):
            raise ReconciliationError("additional API types need stable requirement IDs")
        if len(requirements) != len(set(requirements)):
            raise ReconciliationError("additional API type requirement IDs must be unique")
        if any(not isinstance(type_row.get(language), dict) or not type_row[language]
               for language in REQUIRED_LANGUAGES):
            raise ReconciliationError("additional API type must map every language")
        c_record = type_row["c"]
        if not isinstance(c_record.get("name"), str) or not c_record["name"].strip():
            raise ReconciliationError("additional API type needs an exact public C name")
        additions_by_id[identifier] = type_row
        if identifier in by_id and by_id[identifier] != type_row:
            raise ReconciliationError(f"public type ID already has another definition: {identifier}")
        requirement_ids.update(requirements)
    for identifier, type_row in additions_by_id.items():
        if identifier not in by_id:
            types.append(copy.deepcopy(type_row))
    result["requirementIds"] = sorted(requirement_ids)
    return result


def add_public_declarations(
    manifest: dict[str, Any], declarations: list[dict[str, Any]]
) -> dict[str, Any]:
    """Add exact, evidenced four-language declarations idempotently."""
    if not isinstance(declarations, list):
        raise ReconciliationError("additional public declarations must be a list")
    result = copy.deepcopy(manifest)
    current = result.get("publicDeclarations")
    if not isinstance(current, list):
        raise ReconciliationError("manifest lacks its public declaration inventory")
    by_id = {row.get("declarationId"): row for row in current}
    by_export = {
        (row.get("packageId"), row.get("exportName")): row for row in current
    }
    additions_by_id: dict[str, dict[str, Any]] = {}
    additions_by_export: dict[tuple[str, str], dict[str, Any]] = {}
    for declaration in declarations:
        if not isinstance(declaration, dict):
            raise ReconciliationError("additional public declaration must be an object")
        identifier = declaration.get("declarationId")
        package_id = declaration.get("packageId")
        export_name = declaration.get("exportName")
        if not all(isinstance(value, str) and value.strip() for value in (identifier, package_id, export_name)):
            raise ReconciliationError("additional public declaration needs stable identity fields")
        key = (package_id, export_name)
        if identifier in additions_by_id or key in additions_by_export:
            raise ReconciliationError("additional public declaration IDs and exports must be unique")
        if declaration.get("declarationState") != "verified" or declaration.get("parityScope") not in {"four-language", "c-only"}:
            raise ReconciliationError("additional declaration requires verified four-language parity or an exact C-only surface")
        mappings = declaration.get("languageMappings")
        required_languages = {"c"} if declaration["parityScope"] == "c-only" else REQUIRED_LANGUAGES
        if not isinstance(mappings, dict) or set(mappings) != required_languages:
            raise ReconciliationError("additional declaration must map all languages in its declared parity scope")
        for language, mapping in mappings.items():
            if (
                not isinstance(mapping, dict)
                or set(mapping) != {"packageId", "name"}
                or mapping.get("packageId") not in LANGUAGE_PACKAGES[language]
                or not isinstance(mapping.get("name"), str)
                or not mapping["name"].strip()
            ):
                raise ReconciliationError(f"additional declaration has an invalid {language} mapping")
        evidence = declaration.get("parityEvidenceIds")
        if not isinstance(evidence, list) or not evidence or any(
            not isinstance(path, str) or not path.strip() for path in evidence
        ) or len(evidence) != len(set(evidence)):
            raise ReconciliationError("additional declaration needs unique parity evidence paths")
        requirements = declaration.get("requirementIds")
        if not isinstance(requirements, list) or not requirements or any(
            not isinstance(requirement, str) or not requirement.strip() for requirement in requirements
        ):
            raise ReconciliationError("additional declaration needs stable requirement IDs")
        additions_by_id[identifier] = declaration
        additions_by_export[key] = declaration
        if identifier in by_id and by_id[identifier] != declaration:
            raise ReconciliationError(f"public declaration ID already has another definition: {identifier}")
        if key in by_export and by_export[key] != declaration:
            raise ReconciliationError(f"public export already has another declaration: {export_name}")
    for identifier, declaration in additions_by_id.items():
        if identifier not in by_id:
            current.append(copy.deepcopy(declaration))
    result["requirementIds"] = sorted(
        set(result.get("requirementIds", []))
        | {requirement for row in declarations for requirement in row.get("requirementIds", [])}
    )
    return result


def resolve_public_declarations(
    manifest: dict[str, Any], resolutions: list[dict[str, Any]]
) -> dict[str, Any]:
    """Transfer exact blocked declarations from the shared facade inventory to 0053."""
    if not isinstance(resolutions, list):
        raise ReconciliationError("declaration resolutions must be a list")
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ReconciliationError("manifest lacks its public declaration inventory")
    by_id = {row.get("declarationId"): row for row in declarations}
    seen: set[str] = set()
    for resolution in resolutions:
        if not isinstance(resolution, dict):
            raise ReconciliationError("declaration resolution must be an object")
        identifier = resolution.get("declarationId")
        if not isinstance(identifier, str) or identifier in seen:
            raise ReconciliationError("declaration resolution IDs must be unique")
        seen.add(identifier)
        row = by_id.get(identifier)
        if (
            row is not None
            and row.get("declarationState") == "verified"
            and row.get("capabilityId") == resolution.get("capabilityId")
            and row.get("requirementIds") == resolution.get("requirementIds")
            and row.get("parityScope") == "four-language"
            and row.get("languageMappings") == {
                language: resolution[language] for language in REQUIRED_LANGUAGES
            }
            and row.get("parityEvidenceIds") == resolution.get("parityEvidenceIds")
            and row.get("approvalSource") == resolution.get("approvalSource")
        ):
            continue
        if (
            row is None
            or row.get("declarationState") != "blocked"
            or row.get("candidateSpecId") != "KMIPKIT-0043"
            or resolution.get("fromSpecId") != "KMIPKIT-0043"
        ):
            raise ReconciliationError("only an exact blocked KMIPKIT-0043 declaration may be transferred")
        if set(resolution) != {
            "declarationId", "fromSpecId", "capabilityId", "requirementIds",
            "rust", "c", "java", "python", "parityEvidenceIds", "approvalSource",
        }:
            raise ReconciliationError("declaration resolution has missing or unrecognized fields")
        requirements = resolution["requirementIds"]
        if not isinstance(requirements, list) or not requirements or len(requirements) != len(set(requirements)):
            raise ReconciliationError("declaration resolution needs unique requirement IDs")
        route = {language: resolution[language] for language in REQUIRED_LANGUAGES}
        route["parityEvidenceIds"] = resolution["parityEvidenceIds"]
        route["approvalSource"] = resolution["approvalSource"]
        mappings = _validate_mapping({"kind": row.get("kind")}, route)
        for language, mapping in mappings.items():
            if mapping["packageId"] not in LANGUAGE_PACKAGES[language]:
                raise ReconciliationError(f"{language} resolution targets an unsupported package")
        row["capabilityId"] = resolution["capabilityId"]
        row["requirementIds"] = requirements
        row["declarationState"] = "verified"
        row["parityScope"] = "four-language"
        row["languageMappings"] = mappings
        row["parityEvidenceIds"] = resolution["parityEvidenceIds"]
        row["approvalSource"] = resolution["approvalSource"]
        row.pop("candidateSpecId", None)
        row.pop("blockedReason", None)
    result["requirementIds"] = sorted(
        set(result.get("requirementIds", []))
        | {requirement for row in resolutions for requirement in row["requirementIds"]}
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--routes",
        type=Path,
        default=Path("specs/053-api-parity-facade/additional-api-routes.json"),
    )
    parser.add_argument("--write", action="store_true", help="update the reviewed manifest input")
    arguments = parser.parse_args()
    manifest_path = arguments.repo_root / "specification" / "api" / "public-api.json"
    route_path = arguments.routes
    if not route_path.is_absolute():
        route_path = arguments.repo_root / route_path
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        route_data = json.loads(route_path.read_text(encoding="utf-8"))
        if not isinstance(route_data, dict):
            raise ReconciliationError("route file must contain an object")
        result = manifest
        if "candidateRoutes" in route_data:
            result = reconcile_manifest(result, route_data["candidateRoutes"])
        result = reconcile_candidate_resolutions(
            result, route_data.get("candidateResolutions", [])
        )
        result = resolve_public_declarations(result, route_data.get("declarationResolutions", []))
        result = add_additional_types(result, route_data.get("types", []))
        result = add_additional_functions(result, route_data.get("functions", []))
        result = add_public_declarations(result, route_data.get("publicDeclarations", []))
    except (OSError, json.JSONDecodeError, ReconciliationError) as error:
        print(f"KMIPKIT-0053 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    added = len(result["functions"]) - len(manifest["functions"])
    action = "write" if arguments.write else "would write"
    print(f"Prepared 0053 reconciliation: {added:+} functions; {action} reviewed manifest input.")
    if arguments.write:
        content = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
        manifest_path.write_bytes(content.encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
