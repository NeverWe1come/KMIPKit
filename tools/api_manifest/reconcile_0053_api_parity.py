"""Reconcile reviewed four-language mappings for KMIPKIT-0053 candidates."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path, PurePosixPath
from typing import Any

SPEC_ID = "KMIPKIT-0053"
APPROVAL_RECORD_PATH = "specs/053-api-parity-facade/approval-record.md"
QUERY_RESPONSE_SCOPE_SUPERSESSION = (
    "KMIPKIT-0053 approves typed results and final Rust/C/Java/Python parity, "
    "superseding only the prior KMIPKIT-0020 Rust-only exclusions for the exact "
    "QueryResponse and QueryResponseField declarations listed in "
    "additional-api-routes.json."
)
QUERY_RESPONSE_SCOPE_EVIDENCE = frozenset(
    {
        "specs/053-api-parity-facade/spec.md",
        APPROVAL_RECORD_PATH,
    }
)
QUERY_RESPONSE_EXCLUSION_EXPORTS = frozenset(
    {
        "kmipkit::protocol::QueryResponse",
        "kmipkit::protocol::QueryResponse::application_namespaces",
        "kmipkit::protocol::QueryResponse::attestation_types",
        "kmipkit::protocol::QueryResponse::capability_information",
        "kmipkit::protocol::QueryResponse::client_registration_methods",
        "kmipkit::protocol::QueryResponse::defaults_information",
        "kmipkit::protocol::QueryResponse::extension_information",
        "kmipkit::protocol::QueryResponse::object_types",
        "kmipkit::protocol::QueryResponse::operations",
        "kmipkit::protocol::QueryResponse::profile_information",
        "kmipkit::protocol::QueryResponse::protection_storage_masks",
        "kmipkit::protocol::QueryResponse::response_fields",
        "kmipkit::protocol::QueryResponse::rng_parameters",
        "kmipkit::protocol::QueryResponse::server_information",
        "kmipkit::protocol::QueryResponse::try_from_response_item",
        "kmipkit::protocol::QueryResponse::unknown_items",
        "kmipkit::protocol::QueryResponse::validation_information",
        "kmipkit::protocol::QueryResponse::vendor_identification",
        "kmipkit::protocol::QueryResponseField",
        "kmipkit::protocol::QueryResponseField::tag",
        "kmipkit::protocol::QueryResponseField::with_ttlv",
    }
)
JAVA_BUILDER_CLOSE_DECLARATION_ID = "KMIPKIT-0053:export:946cd939aa398dd68a5ca060"
JAVA_BUILDER_CLOSE_PACKAGE = "java-kmipkit"
JAVA_BUILDER_CLOSE_EXPORT = (
    "org.kmipkit.protocol.CryptographicParameters.Builder.close"
)
JAVA_BUILDER_CLOSE_EXCLUSION_REASON = (
    "This Java-only builder close method clears staged P Source and other builder-owned state. "
    "C has no builder handle because the versioned fields and their bytes remain caller-owned "
    "until from_fields_v1 copies them; Rust cleanup is automatic through Drop. This lifecycle "
    "detail is verified per language and is not claimed as a four-language API route."
)
JAVA_BUILDER_CLOSE_APPROVAL_EVIDENCE = (
    "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation."
)
JAVA_BUILDER_CLOSE_SUPERSESSION_RATIONALE = (
    "The approved Phase 5 contract requires KMIPKit-owned secret copies to be wiped in all supported "
    "bindings. Java Builder.close, Python builder close, Rust Drop, and C release all wipe KMIPKit-owned "
    "P Source copies; C has no pre-build builder handle, so its handle release is the corresponding "
    "owned-secret lifecycle boundary."
)
JAVA_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS = frozenset(
    {
        "bindings/java/src/test/java/org/kmipkit/typed/ParametersAuthenticationParityTest.java",
        "bindings/python/tests/test_parameters_authentication_parity.py",
        "crates/kmipkit-protocol/tests/cryptographic_parameters_public.rs",
    }
)
JAVA_BUILDER_CLOSE_SUPERSESSION_EVIDENCE_PATHS = frozenset(
    {
        "specs/053-api-parity-facade/spec.md",
        APPROVAL_RECORD_PATH,
        *JAVA_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS,
        "bindings/c/tests/authentication_consumer.c",
        "crates/kmipkit-ffi/tests/support/parameters_auth_0053_tests.rs",
        "crates/kmipkit-protocol/src/credential/secret.rs",
        "crates/kmipkit-protocol/src/cryptographic_parameters.rs",
        "crates/kmipkit-protocol/tests/credential_secret.rs",
    }
)
PYTHON_BUILDER_CLOSE_DECLARATION_ID = "KMIPKIT-0053:export:b6848d45007bbf0e29966906"
PYTHON_BUILDER_CLOSE_PACKAGE = "python-kmipkit"
PYTHON_BUILDER_CLOSE_EXPORT = (
    "kmipkit.cryptographic_parameters.CryptographicParametersBuilder.close"
)
PYTHON_BUILDER_CLOSE_EXCLUSION_REASON = (
    "This closes and zeroizes Python-owned staged P Source. Java Builder.close and Rust builder Drop "
    "provide corresponding local cleanup; C has no builder-owned staging allocation because the "
    "versioned fields record and its bytes remain caller-owned until from_fields_v1 copies them."
)
PYTHON_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS = (
    "bindings/java/src/test/java/org/kmipkit/typed/ParametersAuthenticationParityTest.java",
    "bindings/python/tests/test_parameters_authentication_parity.py",
    "crates/kmipkit-ffi/tests/support/parameters_auth_0053_tests.rs",
    "crates/kmipkit-protocol/tests/cryptographic_parameters_public.rs",
)
LANGUAGE_PACKAGES = {
    "rust": {
        "rust-kmipkit",
        "rust-kmipkit-ttlv",
        "rust-kmipkit-protocol",
        "rust-kmipkit-client",
    },
    "c": {"c-kmipkit"},
    "java": {"java-kmipkit"},
    "python": {"python-kmipkit"},
}
REQUIRED_LANGUAGES = frozenset(LANGUAGE_PACKAGES)
MANAGED_OBJECT_VARIANT_SUFFIXES = {
    "ManagedFieldValue": frozenset(
        {
            "BigInteger", "BigInteger::0", "Boolean", "Boolean::0",
            "ByteString", "ByteString::0", "DateTime", "DateTime::0",
            "DateTimeExtended", "DateTimeExtended::0", "Enumeration",
            "Enumeration::0", "Integer", "Integer::0", "Interval",
            "Interval::0", "LongInteger", "LongInteger::0", "Structure",
            "TextString", "TextString::0", "Unknown", "Unknown::0",
        }
    ),
    "ManagedValueKind": frozenset(
        {
            "KeyBlock", "KeyMaterial", "KeyMaterial::0", "KeyValue",
            "KeyValue::0", "KeyWrappingData", "ManagedObject",
            "ManagedObject::0",
        }
    ),
}
MANAGED_OBJECT_VARIANT_COUNTERPARTS = {
    ("rust-kmipkit", "ManagedFieldValue"): (
        "managed_value_field_item_read:rust",
        "rust-observed-a95a34edc0285cd6ad3a",
    ),
    ("rust-kmipkit", "ManagedValueKind"): (
        "managed_object_from_item:c",
        "rust-observed-ba76f0e0208f5cac6eae",
    ),
    ("rust-kmipkit-protocol", "ManagedFieldValue"): (
        "managed_value_field_item_read:rust",
        "rust-observed-2a78ab726eb8c96d0e37",
    ),
    ("rust-kmipkit-protocol", "ManagedValueKind"): (
        "managed_object_from_item:c",
        "rust-observed-0d3351bf7b7739e95d55",
    ),
}
DECLARATION_RESOLUTION_COMMON_FIELDS = frozenset(
    {
        "declarationId",
        "capabilityId",
        "packageId",
        "exportName",
        "resolution",
        "evidencePaths",
        "approvalSource",
    }
)
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
CFFI_BRIDGE_EXCLUSION = (
    "kmipkit_ttlv_value_view_copy_bytes",
    "This exact exported C symbol is an internal block-copy bridge used by the Java JNI and Python CFFI adapters; "
    "it is intentionally absent from the supported C header and adds no independent C product capability.",
    {
        "kind": "feature-specification",
        "path": "specs/053-api-parity-facade/approval-record.md",
        "status": "Approved",
        "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
    },
    "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
)


class ReconciliationError(ValueError):
    """Raised when a candidate mapping or exclusion is incomplete or too broad."""


def _validate_mapping(
    declaration: dict[str, Any], route: dict[str, Any]
) -> dict[str, dict[str, str]]:
    if "exclusion" in route:
        raise ReconciliationError(
            "exclusions cannot be interpreted as language mappings"
        )
    if set(route) != {*REQUIRED_LANGUAGES, "parityEvidenceIds", "approvalSource"}:
        raise ReconciliationError(
            "each mapped route must name all four languages, evidence, and approval"
        )
    mappings: dict[str, dict[str, str]] = {}
    for language in sorted(REQUIRED_LANGUAGES):
        mapping = route.get(language)
        if not isinstance(mapping, dict) or set(mapping) != {"packageId", "name"}:
            raise ReconciliationError(
                f"{language} route must contain exactly packageId and name"
            )
        package_id = mapping.get("packageId")
        name = mapping.get("name")
        if (
            package_id not in LANGUAGE_PACKAGES[language]
            or not isinstance(name, str)
            or not name.strip()
        ):
            raise ReconciliationError(
                f"{language} route is not an exact public declaration mapping"
            )
        mappings[language] = {"packageId": package_id, "name": name}
    evidence = route.get("parityEvidenceIds")
    if (
        not isinstance(evidence, list)
        or not evidence
        or any(not isinstance(path, str) or not path.strip() for path in evidence)
        or len(evidence) != len(set(evidence))
    ):
        raise ReconciliationError(
            "every mapping needs unique, non-empty behavior evidence paths"
        )
    approval = route.get("approvalSource")
    if (
        not isinstance(approval, dict)
        or set(approval) != {"kind", "path", "status", "evidence"}
        or any(
            not isinstance(value, str) or not value.strip()
            for value in approval.values()
        )
        or approval.get("kind") != "feature-specification"
        or approval.get("status") != "Approved"
    ):
        raise ReconciliationError("every mapping needs exact feature approval evidence")
    return mappings


def _validate_exclusion(
    declaration: dict[str, Any], route: dict[str, Any]
) -> dict[str, Any]:
    exclusion = route.get("exclusion")
    if set(route) != {"exclusion"} or not isinstance(exclusion, dict):
        raise ReconciliationError("an exclusion must be the only route entry")
    if declaration.get("kind") != "associated-item" or not declaration.get(
        "exportName", ""
    ).startswith("<"):
        raise ReconciliationError(
            "only one exact compiler-visible trait projection may be excluded"
        )
    if set(exclusion) != {"reason", "evidence"}:
        raise ReconciliationError("an exclusion needs its exact reason and evidence")
    reason = exclusion.get("reason")
    evidence = exclusion.get("evidence")
    if reason != TRAIT_PROJECTION_REASON:
        raise ReconciliationError(
            "the exclusion reason does not justify this exact trait projection"
        )
    if (
        not isinstance(evidence, list)
        or not evidence
        or any(not isinstance(path, str) or not path.strip() for path in evidence)
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
    if (
        declaration.get("candidateSpecId") != SPEC_ID
        or declaration.get("declarationState") != "blocked"
    ):
        raise ReconciliationError(
            "only blocked KMIPKIT-0053 candidates may be resolved"
        )


def _validate_resolution_evidence(resolution: dict[str, Any]) -> list[str]:
    evidence = resolution.get("evidencePaths")
    if (
        not isinstance(evidence, list)
        or not evidence
        or any(not isinstance(path, str) or not path.strip() for path in evidence)
    ):
        raise ReconciliationError(
            "every declaration resolution needs unique evidence paths"
        )
    if len(evidence) != len(set(evidence)):
        raise ReconciliationError(
            "every declaration resolution needs unique evidence paths"
        )
    for path in evidence:
        parsed = PurePosixPath(path)
        if (
            parsed.is_absolute()
            or ".." in parsed.parts
            or "\\" in path
            or any(char in path for char in "*?")
        ):
            raise ReconciliationError(
                "evidence paths must be exact repository-relative paths"
            )

    approval = resolution.get("approvalSource")
    if (
        not isinstance(approval, dict)
        or set(approval) != {"kind", "path", "status", "evidence"}
        or any(
            not isinstance(value, str) or not value.strip()
            for value in approval.values()
        )
        or approval.get("kind") != "feature-specification"
        or approval.get("status") != "Approved"
    ):
        raise ReconciliationError(
            "every declaration resolution needs an approved feature source"
        )
    approval_path = PurePosixPath(approval["path"])
    if (
        approval["path"] != APPROVAL_RECORD_PATH
        or approval_path.is_absolute()
        or ".." in approval_path.parts
        or "\\" in approval["path"]
        or any(char in approval["path"] for char in "*?")
    ):
        raise ReconciliationError(
            "approval source must use an exact repository-relative path"
        )
    return evidence


def _candidate_parent(
    declaration: dict[str, Any],
    declarations_by_export: dict[tuple[str, str], dict[str, Any]],
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
        raise ReconciliationError(
            "an exact type-system exclusion needs an artifact and counterpart IDs"
        )
    artifact_kind = exclusion.get("artifactKind")
    if (
        not isinstance(artifact_kind, str)
        or artifact_kind not in TYPE_SYSTEM_EXCLUSION_REASONS
    ):
        raise ReconciliationError(
            "the exclusion kind is not an approved Rust type-system artifact"
        )

    declaration_kind = declaration.get("kind")
    export_name = declaration.get("exportName", "")
    valid_artifact = False
    parent: dict[str, Any] | None = None
    if artifact_kind == "rust-trait":
        valid_artifact = declaration_kind == "trait"
    elif artifact_kind == "rust-associated-type":
        valid_artifact = declaration_kind == "associated-item" and export_name.rsplit(
            "::", maxsplit=1
        )[-1] in {"Value", "ValueView"}
        parent = _candidate_parent(declaration, declarations_by_export)
        valid_artifact = (
            valid_artifact and parent is not None and parent.get("kind") == "trait"
        )
    elif artifact_kind == "rust-generic-type":
        valid_artifact = (
            declaration_kind == "type" and "<" in export_name and ">" in export_name
        )
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
        enum_type = (
            _candidate_parent(parent, declarations_by_export)
            if parent is not None
            else None
        )
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
        raise ReconciliationError(
            "the exclusion does not match this exact Rust type-system declaration"
        )

    counterpart_ids = exclusion.get("counterpartDeclarationIds")
    if (
        not isinstance(counterpart_ids, list)
        or not counterpart_ids
        or any(
            not isinstance(identifier, str)
            or not identifier
            or any(c in identifier for c in "*?")
            for identifier in counterpart_ids
        )
    ):
        raise ReconciliationError(
            "type-system exclusions require exact counterpart declaration IDs"
        )
    if len(counterpart_ids) != len(set(counterpart_ids)):
        raise ReconciliationError(
            "type-system exclusions require unique counterpart declaration IDs"
        )
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
        enum_type = (
            _candidate_parent(variant, declarations_by_export)
            if variant is not None
            else None
        )
        return enum_type.get("declarationId") if enum_type is not None else None
    return None


def _validate_counterparts(
    counterpart_ids: list[str],
    capability_id: str,
    declarations_by_id: dict[str, dict[str, Any]],
) -> None:
    for counterpart_id in counterpart_ids:
        counterpart = declarations_by_id.get(counterpart_id)
        mappings = (
            counterpart.get("languageMappings") if counterpart is not None else None
        )
        if (
            counterpart is None
            or counterpart.get("declarationState") != "verified"
            or counterpart.get("capabilityId") != capability_id
            or not isinstance(mappings, dict)
            or set(mappings) != REQUIRED_LANGUAGES
            or mappings.get("rust")
            != {
                "packageId": counterpart.get("packageId"),
                "name": counterpart.get("exportName"),
            }
        ):
            raise ReconciliationError(
                "exclusion counterpart must be an exact verified four-language declaration"
            )
        for language in REQUIRED_LANGUAGES:
            mapping = mappings[language]
            if (
                not isinstance(mapping, dict)
                or set(mapping) != {"packageId", "name"}
                or mapping["packageId"] not in LANGUAGE_PACKAGES[language]
                or not isinstance(mapping["name"], str)
                or not mapping["name"].strip()
            ):
                raise ReconciliationError(
                    "exclusion counterpart contains an invalid language mapping"
                )


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
    candidate_identifiers = [row.get("declarationId") for row in candidates]
    if any(
        not isinstance(identifier, str)
        or not identifier
        or any(char in identifier for char in "*?")
        for identifier in candidate_identifiers
    ):
        raise ReconciliationError(
            "every candidate needs one exact stable declaration ID"
        )
    if len(candidate_identifiers) != len(set(candidate_identifiers)):
        raise ReconciliationError("candidate declaration IDs must be unique")
    candidates_by_id = {row.get("declarationId"): row for row in candidates}
    all_by_id = {row.get("declarationId"): row for row in declarations}
    by_export = {
        (row.get("packageId", ""), row.get("exportName", "")): row
        for row in declarations
    }
    identifiers = [
        row.get("declarationId") for row in resolutions if isinstance(row, dict)
    ]
    if len(identifiers) != len(resolutions) or any(
        not isinstance(identifier, str)
        or not identifier
        or any(c in identifier for c in "*?")
        for identifier in identifiers
    ):
        raise ReconciliationError(
            "each resolution needs one exact stable declaration ID"
        )
    if len(identifiers) != len(set(identifiers)):
        raise ReconciliationError(
            "a declaration may be resolved only once per reconciliation"
        )

    mapping_ids: set[str] = set()
    exclusion_records: list[dict[str, Any]] = []
    for resolution in resolutions:
        if not isinstance(resolution, dict):
            raise ReconciliationError(
                "candidate resolution has missing or unrecognized fields"
            )
        resolution_kind = resolution.get("resolution")
        expected_fields = DECLARATION_RESOLUTION_COMMON_FIELDS | (
            {"languageMappings"} if resolution_kind == "mapped" else {"exclusion"}
        )
        if set(resolution) != expected_fields:
            raise ReconciliationError(
                "candidate resolution has missing or unrecognized fields"
            )
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
                    or resolution.get("evidencePaths")
                    != existing.get("parityEvidenceIds")
                    or resolution.get("approvalSource")
                    != existing.get("approvalSource")
                ):
                    raise ReconciliationError(
                        "replayed mapped resolution differs from verified declaration"
                    )
                mapping_ids.add(identifier)
                continue
            existing_exclusion = next(
                (
                    row
                    for row in result.get("approvedExclusions", [])
                    if row.get("declarationId") == identifier
                ),
                None,
            )
            if existing_exclusion is not None:
                _validate_resolution_identity_existing(existing_exclusion, resolution)
                expected = _resolution_exclusion_replay_record(resolution)
                if existing_exclusion != expected:
                    raise ReconciliationError(
                        "replayed exclusion differs from approved record"
                    )
                _validate_counterparts(
                    expected["counterpartDeclarationIds"],
                    existing_exclusion.get("capabilityId"),
                    all_by_id,
                )
                continue
            raise ReconciliationError(
                "candidate resolution references no exact 0053 candidate"
            )

        _validate_resolution_identity(declaration, resolution)
        evidence = _validate_resolution_evidence(resolution)
        if resolution_kind == "mapped":
            route = _mapping_route_from_resolution(resolution)
            mappings = _validate_mapping(declaration, route)
            if (
                mappings["rust"]["packageId"] != declaration["packageId"]
                or mappings["rust"]["name"] != declaration["exportName"]
            ):
                raise ReconciliationError(
                    "Rust mapping must exactly match the candidate export"
                )
            if any(
                char in mapping["name"]
                for mapping in mappings.values()
                for char in "*?"
            ):
                raise ReconciliationError(
                    "language mappings must name exact public routes"
                )
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
            exclusion, _ = _validate_type_system_exclusion(
                declaration, resolution, by_export
            )
            exclusion_records.append(
                _resolution_exclusion_record(
                    declaration, resolution, exclusion, evidence
                )
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
            raise ReconciliationError(
                "enum-shape exclusion must name its exact mapped parent model"
            )
        _validate_counterparts(counterpart_ids, declaration["capabilityId"], all_by_id)

    processed_candidate_ids = set(identifiers) & set(candidates_by_id)
    result["publicDeclarations"] = [
        row
        for row in declarations
        if row.get("declarationId") not in processed_candidate_ids
    ] + [
        all_by_id[identifier]
        for identifier in identifiers
        if identifier in mapping_ids and identifier in processed_candidate_ids
    ]

    existing_exclusions = result.setdefault("approvedExclusions", [])
    existing_exclusion_ids = {row.get("declarationId") for row in existing_exclusions}
    for exclusion in exclusion_records:
        if exclusion["declarationId"] in existing_exclusion_ids:
            raise ReconciliationError(
                "candidate exclusion already exists in approvedExclusions"
            )
        existing_exclusions.append(exclusion)
        existing_exclusion_ids.add(exclusion["declarationId"])

    _finalize_candidate_capabilities(result, resolutions)
    return result


def _finalize_candidate_capabilities(
    result: dict[str, Any], resolutions: list[dict[str, Any]]
) -> None:
    capabilities = result.get("capabilities")
    declarations = result.get("publicDeclarations")
    if not isinstance(capabilities, list) or not isinstance(declarations, list):
        raise ReconciliationError("manifest lacks capabilities or publicDeclarations")
    capability_by_id = {row.get("capabilityId"): row for row in capabilities}
    for capability_id in {resolution["capabilityId"] for resolution in resolutions}:
        capability = capability_by_id.get(capability_id)
        if capability is None:
            raise ReconciliationError("candidate capability is missing from manifest")
        unresolved = any(
            row.get("candidateSpecId") == SPEC_ID
            and row.get("capabilityId") == capability_id
            and row.get("declarationState") == "blocked"
            for row in declarations
        )
        if unresolved and capability.get("capabilityState") == "verified":
            raise ReconciliationError(
                "a verified capability cannot retain unresolved 0053 candidates"
            )
        if unresolved or capability.get("capabilityState") == "verified":
            continue
        if capability.get("capabilityState") != "blocked":
            raise ReconciliationError(
                "candidate capability is not blocked pending reconciliation"
            )
        mapped_rows = [
            row
            for row in declarations
            if row.get("capabilityId") == capability_id
            and row.get("declarationState") == "verified"
        ]
        if not mapped_rows:
            raise ReconciliationError(
                "a capability cannot be verified by exclusions without mapped behavior"
            )
        capability["capabilityState"] = "verified"
        capability["mappings"] = {
            language: sorted(
                {row["languageMappings"][language]["name"] for row in mapped_rows}
            )
            for language in sorted(REQUIRED_LANGUAGES)
        }
        phase5_rows = [
            row
            for row in mapped_rows
            if row.get("approvalSource", {}).get("path") == APPROVAL_RECORD_PATH
        ]
        if phase5_rows:
            capability.setdefault("ownerSpecId", SPEC_ID)
            capability["approvalEvidence"] = copy.deepcopy(
                phase5_rows[0]["approvalSource"]
            )
            capability["parityEvidenceIds"] = sorted(
                {
                    path
                    for row in phase5_rows
                    for path in row.get("parityEvidenceIds", [])
                }
            )
        capability.pop("candidateSpecId", None)
        capability.pop("blockedReason", None)


def _mapping_route_from_resolution(resolution: dict[str, Any]) -> dict[str, Any]:
    mappings = resolution.get("languageMappings")
    if not isinstance(mappings, dict):
        raise ReconciliationError(
            "mapped candidate needs all four exact language mappings"
        )
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
        raise ReconciliationError(
            "replayed resolution identity does not match its exact record"
        )


def _is_approved_source_refresh(
    old: Any, new: Any, owner_approval_source: Any = None
) -> bool:
    """Allow only an exact evidence refresh from the same approved feature record."""
    same_record = (
        isinstance(old, dict)
        and isinstance(new, dict)
        and set(old) == {"kind", "path", "status", "evidence"}
        and set(new) == {"kind", "path", "status", "evidence"}
        and old.get("kind") == new.get("kind") == "feature-specification"
        and old.get("path") == new.get("path") == APPROVAL_RECORD_PATH
        and old.get("status") == new.get("status") == "Approved"
    )
    owner_record = (
        isinstance(owner_approval_source, dict)
        and isinstance(owner_approval_source.get("path"), str)
        and isinstance(old, dict)
        and isinstance(new, dict)
        and set(old) == {"kind", "path", "status", "evidence"}
        and set(new) == {"kind", "path", "status", "evidence"}
        and old.get("kind") == new.get("kind") == "feature-specification"
        and old.get("status") == new.get("status") == "Approved"
        and new.get("path") == owner_approval_source.get("path")
        and owner_approval_source.get("status") == "Approved"
    )
    if not (same_record or owner_record) or not isinstance(new.get("evidence"), str):
        return False
    try:
        evidence_content = Path(new["path"]).read_text(encoding="utf-8")
    except OSError:
        return False
    return bool(new["evidence"]) and new["evidence"] in evidence_content


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
        or any(
            not isinstance(identifier, str)
            or not identifier
            or any(char in identifier for char in "*?")
            for identifier in counterpart_ids
        )
    ):
        raise ReconciliationError("replayed exclusion needs exact counterpart IDs")
    if len(counterpart_ids) != len(set(counterpart_ids)):
        raise ReconciliationError(
            "replayed exclusion needs unique exact counterpart IDs"
        )
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
        raise ReconciliationError(
            "route mapping must be an object keyed by declarationId"
        )
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    capabilities = result.get("capabilities")
    if not isinstance(declarations, list) or not isinstance(capabilities, list):
        raise ReconciliationError("manifest lacks publicDeclarations or capabilities")
    candidates = [row for row in declarations if row.get("candidateSpecId") == SPEC_ID]
    candidate_ids = [row.get("declarationId") for row in candidates]
    if any(
        not isinstance(identifier, str) or not identifier
        for identifier in candidate_ids
    ):
        raise ReconciliationError("every candidate needs a stable declarationId")
    if len(candidate_ids) != len(set(candidate_ids)):
        raise ReconciliationError("candidate declaration IDs must be unique")
    candidate_by_id = {row["declarationId"]: row for row in candidates}
    existing_by_id = {row.get("declarationId"): row for row in declarations}
    extra = sorted(set(routes) - set(candidate_ids) - set(existing_by_id))
    if extra:
        raise ReconciliationError(
            f"route inventory contains unowned IDs (extra={len(extra)})"
        )

    by_capability: dict[str, dict[str, list[str]]] = {}
    verified: list[dict[str, Any]] = []
    exclusions: list[dict[str, Any]] = []
    for declaration_id, route in routes.items():
        declaration = candidate_by_id.get(declaration_id)
        if declaration is None:
            existing = existing_by_id[declaration_id]
            expected = route.get("exclusion") if isinstance(route, dict) else None
            if expected is not None or existing.get("declarationState") != "verified":
                raise ReconciliationError(
                    "replayed routes must match an already verified declaration"
                )
            mappings = _validate_mapping(existing, route)
            if mappings != existing.get("languageMappings") or route[
                "parityEvidenceIds"
            ] != existing.get("parityEvidenceIds"):
                raise ReconciliationError(
                    "replayed route differs from its verified declaration"
                )
            continue
        if declaration.get("declarationState") == "verified":
            mappings = _validate_mapping(declaration, route)
            if (
                mappings != declaration.get("languageMappings")
                or route["parityEvidenceIds"]
                != declaration.get("parityEvidenceIds")
                or route["approvalSource"] != declaration.get("approvalSource")
            ):
                raise ReconciliationError(
                    "replayed route differs from its verified declaration"
                )
            continue
        if declaration.get("declarationState") != "blocked":
            raise ReconciliationError(
                "unreconciled 0053 candidates must remain blocked"
            )
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
        language_names = by_capability.setdefault(
            declaration["capabilityId"],
            {language: [] for language in REQUIRED_LANGUAGES},
        )
        for language, mapping in mappings.items():
            language_names[language].append(mapping["name"])

    exclusion_keys = {(row["packageId"], row["exportName"]) for row in exclusions}
    processed_ids = {
        row["declarationId"] for row in candidates if row["declarationId"] in routes
    }
    result["publicDeclarations"] = [
        row
        for row in declarations
        if row.get("declarationId") not in processed_ids
        if (row.get("packageId"), row.get("exportName")) not in exclusion_keys
    ] + verified
    existing_exclusions = result.setdefault("approvedExclusions", [])
    existing_keys = {
        (row.get("packageId"), row.get("exportName")) for row in existing_exclusions
    }
    if any(
        (row["packageId"], row["exportName"]) in existing_keys for row in exclusions
    ):
        raise ReconciliationError(
            "candidate exclusion already exists in approvedExclusions"
        )
    existing_exclusions.extend(exclusions)

    capability_rows = {row.get("capabilityId"): row for row in capabilities}
    candidate_capabilities = {
        candidate_by_id[declaration_id].get("capabilityId")
        for declaration_id in routes
        if declaration_id in candidate_by_id
    }
    for capability_id in candidate_capabilities:
        capability = capability_rows.get(capability_id)
        if capability is None:
            raise ReconciliationError(
                "each candidate capability must be present and blocked"
            )
        unresolved = any(
            row.get("candidateSpecId") == SPEC_ID
            and row.get("capabilityId") == capability_id
            and row.get("declarationState") == "blocked"
            for row in result["publicDeclarations"]
        )
        if unresolved:
            continue
        if capability.get("capabilityState") == "verified":
            continue
        if capability.get("capabilityState") != "blocked":
            raise ReconciliationError("candidate capability was already reconciled")
        capability["capabilityState"] = "verified"
        all_ready = [
            row
            for row in result["publicDeclarations"]
            if row.get("capabilityId") == capability_id
            and row.get("declarationState") == "verified"
        ]
        capability["mappings"] = {
            language: sorted(
                {row["languageMappings"][language]["name"] for row in all_ready}
            )
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
            capability["parityEvidenceIds"] = sorted(
                {
                    evidence
                    for route in capability_routes
                    for evidence in route["parityEvidenceIds"]
                }
            )
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
        if (
            not isinstance(identifier, str)
            or not identifier
            or not isinstance(symbol, str)
        ):
            raise ReconciliationError(
                "additional function needs a stable ID and C symbol"
            )
        if identifier in additions_by_id or symbol in additions_by_symbol:
            raise ReconciliationError(
                "additional function IDs and C symbols must be unique"
            )
        mappings = (
            function.get("rust"),
            c_record,
            function.get("java"),
            function.get("python"),
        )
        if any(not isinstance(mapping, dict) for mapping in mappings):
            raise ReconciliationError(
                "additional functions must define Rust, C, Java, and Python routes"
            )
        requirements = function.get("requirementIds")
        if (
            not isinstance(requirements, list)
            or not requirements
            or any(
                not isinstance(requirement, str) or not requirement
                for requirement in requirements
            )
        ):
            raise ReconciliationError(
                "additional functions need stable requirement IDs"
            )
        function_requirements = set(requirements)
        for language, mapping in zip(
            ("rust", "c", "java", "python"), mappings, strict=True
        ):
            if not mapping:
                raise ReconciliationError(
                    f"additional function has an empty {language} route"
                )
        if not isinstance(function.get("ordering"), str):
            raise ReconciliationError(
                "additional function must declare its ordering contract"
            )
        additions_by_id[identifier] = function
        additions_by_symbol[symbol] = function
        if identifier in by_id and by_id[identifier] != function:
            raise ReconciliationError(
                f"additional function ID already has a different definition: {identifier}"
            )
        if symbol in by_symbol and by_symbol[symbol] != function:
            raise ReconciliationError(
                f"additional C symbol already has a different definition: {symbol}"
            )
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
        allowed_type_keys = {
            "id",
            "kind",
            "requirementIds",
            "rust",
            "c",
            "java",
            "python",
        }
        if type_row.get("kind") == "enum":
            allowed_type_keys.add("enumRepresentation")
        if set(type_row) != allowed_type_keys:
            raise ReconciliationError(
                "additional API type must define exact Rust, C, Java, and Python mappings"
            )
        if not isinstance(type_row.get("kind"), str) or not type_row["kind"].strip():
            raise ReconciliationError("additional API type needs a kind")
        requirements = type_row.get("requirementIds")
        if (
            not isinstance(requirements, list)
            or not requirements
            or any(
                not isinstance(requirement, str) or not requirement.strip()
                for requirement in requirements
            )
        ):
            raise ReconciliationError(
                "additional API types need stable requirement IDs"
            )
        if len(requirements) != len(set(requirements)):
            raise ReconciliationError(
                "additional API type requirement IDs must be unique"
            )
        if any(
            not isinstance(type_row.get(language), dict) or not type_row[language]
            for language in REQUIRED_LANGUAGES
        ):
            raise ReconciliationError("additional API type must map every language")
        c_record = type_row["c"]
        if not isinstance(c_record.get("name"), str) or not c_record["name"].strip():
            raise ReconciliationError(
                "additional API type needs an exact public C name"
            )
        if type_row["kind"] == "enum":
            representation = type_row.get("enumRepresentation")
            if (
                c_record.get("kind") != "enum"
                or not isinstance(representation, dict)
                or representation.get("shape") not in {"open-numeric", "closed-numeric"}
                or representation.get("unknownValuePolicy")
                not in {"preserve-raw", "reject-unsupported"}
                or not isinstance(representation.get("values"), list)
                or not representation["values"]
            ):
                raise ReconciliationError(
                    "additional enum types need an exact numeric representation"
                )
        additions_by_id[identifier] = type_row
        if identifier in by_id and by_id[identifier] != type_row:
            raise ReconciliationError(
                f"public type ID already has another definition: {identifier}"
            )
        requirement_ids.update(requirements)
    for identifier, type_row in additions_by_id.items():
        if identifier not in by_id:
            types.append(copy.deepcopy(type_row))
    result["requirementIds"] = sorted(requirement_ids)
    return result


def add_public_declarations(
    manifest: dict[str, Any],
    declarations: list[dict[str, Any]],
    python_stale_mappings: list[dict[str, Any]] | None = None,
    java_stale_mappings: list[dict[str, Any]] | None = None,
    python_kind_corrections: list[dict[str, Any]] | None = None,
    rust_kind_corrections: list[dict[str, Any]] | None = None,
    language_mapping_corrections: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Add exact declarations, applying only approved Python path corrections."""
    if not isinstance(declarations, list):
        raise ReconciliationError("additional public declarations must be a list")
    stale_python_routes: dict[tuple[str, str], dict[str, Any]] = {}
    stale_keys = {
        "packageId", "name", "correctedName", "capabilityId", "requirementIds",
        "evidencePaths", "approval", "reason",
    }
    for correction in python_stale_mappings or []:
        if not isinstance(correction, dict) or set(correction) != stale_keys:
            raise ReconciliationError("Python stale route correction has missing or extra fields")
        if correction.get("packageId") != "python-kmipkit":
            raise ReconciliationError("only exact Python export path corrections are supported")
        old_name = correction.get("name")
        corrected_name = correction.get("correctedName")
        capability_id = correction.get("capabilityId")
        if not all(
            isinstance(value, str) and value.strip()
            for value in (old_name, corrected_name, capability_id, correction.get("reason"))
        ) or any(any(marker in value for marker in "*?") for value in (old_name, corrected_name)):
            raise ReconciliationError("Python stale route correction needs exact names and a reason")
        requirements = correction.get("requirementIds")
        evidence = correction.get("evidencePaths")
        approval = correction.get("approval")
        if (
            not isinstance(requirements, list)
            or not requirements
            or any(not isinstance(value, str) or not value.strip() for value in requirements)
            or len(requirements) != len(set(requirements))
            or not isinstance(evidence, list)
            or not evidence
            or any(not isinstance(value, str) or not value.strip() for value in evidence)
            or len(evidence) != len(set(evidence))
            or not isinstance(approval, dict)
            or approval.get("status") != "Approved"
        ):
            raise ReconciliationError("Python stale route correction needs approved traceability")
        key = ("python-kmipkit", old_name)
        if key in stale_python_routes:
            raise ReconciliationError("Python stale route corrections must have unique old paths")
        stale_python_routes[key] = correction
    python_kind_routes: dict[tuple[str, str], dict[str, Any]] = {}
    kind_correction_keys = {
        "packageId", "name", "fromKind", "toKind", "capabilityId",
        "requirementIds", "evidencePaths", "approval", "reason",
    }
    for correction in python_kind_corrections or []:
        if not isinstance(correction, dict) or set(correction) != kind_correction_keys:
            raise ReconciliationError("Python kind correction has missing or extra fields")
        if correction.get("packageId") != "python-kmipkit":
            raise ReconciliationError("only exact Python export kind corrections are supported")
        export_name = correction.get("name")
        if (
            not isinstance(export_name, str)
            or not export_name.strip()
            or any(marker in export_name for marker in "*?")
            or not isinstance(correction.get("fromKind"), str)
            or not correction["fromKind"].strip()
            or not isinstance(correction.get("toKind"), str)
            or not correction["toKind"].strip()
            or not isinstance(correction.get("capabilityId"), str)
            or not correction["capabilityId"].strip()
            or not isinstance(correction.get("reason"), str)
            or not correction["reason"].strip()
            or not isinstance(correction.get("approval"), dict)
            or correction["approval"].get("status") != "Approved"
        ):
            raise ReconciliationError("Python kind correction needs an exact approved route")
        requirements = correction.get("requirementIds")
        evidence_paths = correction.get("evidencePaths")
        if (
            not isinstance(requirements, list)
            or not requirements
            or any(not isinstance(value, str) or not value.strip() for value in requirements)
            or len(requirements) != len(set(requirements))
            or not isinstance(evidence_paths, list)
            or not evidence_paths
            or any(
                not isinstance(path, str)
                or not path.strip()
                or Path(path).is_absolute()
                or "\\" in path
                or ".." in Path(path).parts
                or not Path(path).is_file()
                for path in evidence_paths
            )
        ):
            raise ReconciliationError("Python kind correction needs approved traceability")
        key = ("python-kmipkit", export_name)
        if key in python_kind_routes:
            raise ReconciliationError("Python kind corrections must have unique exports")
        python_kind_routes[key] = correction
    rust_kind_routes: dict[tuple[str, str], dict[str, Any]] = {}
    rust_kind_keys = {
        "packageId", "exportName", "manifestKind", "observedKind", "reason",
        "evidencePaths",
    }
    for correction in rust_kind_corrections or []:
        if not isinstance(correction, dict) or set(correction) != rust_kind_keys:
            raise ReconciliationError("Rust kind correction has missing or extra fields")
        package_id = correction.get("packageId")
        export_name = correction.get("exportName")
        evidence_paths = correction.get("evidencePaths")
        if (
            package_id not in LANGUAGE_PACKAGES["rust"]
            or not isinstance(export_name, str)
            or not export_name.strip()
            or any(marker in export_name for marker in "*?")
            or not isinstance(correction.get("manifestKind"), str)
            or not correction["manifestKind"].strip()
            or not isinstance(correction.get("observedKind"), str)
            or not correction["observedKind"].strip()
            or not isinstance(correction.get("reason"), str)
            or not correction["reason"].strip()
            or not isinstance(evidence_paths, list)
            or not evidence_paths
            or any(
                not isinstance(path, str)
                or not path.strip()
                or Path(path).is_absolute()
                or "\\" in path
                or ".." in Path(path).parts
                or not Path(path).is_file()
                for path in evidence_paths
            )
        ):
            raise ReconciliationError("Rust kind correction needs exact source evidence")
        key = (package_id, export_name)
        if key in rust_kind_routes:
            raise ReconciliationError("Rust kind corrections must have unique exports")
        rust_kind_routes[key] = correction
    reviewed_mapping_routes: dict[tuple[str, str], dict[str, Any]] = {}
    for correction in language_mapping_corrections or []:
        if not isinstance(correction, dict):
            raise ReconciliationError("language mapping correction must be an object")
        key = correction.get("declarationId"), correction.get("language")
        if key in reviewed_mapping_routes:
            raise ReconciliationError("language mapping corrections must be unique")
        reviewed_mapping_routes[key] = correction
    stale_java_routes: dict[tuple[str, str, str], dict[str, Any]] = {}
    java_stale_keys = {
        "packageId", "declarationId", "capabilityId", "oldName",
        "observedPublicTargets", "requirementIds", "reason", "evidencePaths",
        "approval", "name",
    }
    for correction in java_stale_mappings or []:
        if not isinstance(correction, dict) or set(correction) != java_stale_keys:
            raise ReconciliationError("Java stale route correction has missing or extra fields")
        if (
            correction.get("packageId") != "java-kmipkit"
            or correction.get("name") != correction.get("oldName")
        ):
            raise ReconciliationError("only exact Java base-wrapper route corrections are supported")
        old_name = correction.get("oldName")
        capability_id = correction.get("capabilityId")
        reason = correction.get("reason")
        approval = correction.get("approval")
        targets = correction.get("observedPublicTargets")
        requirements = correction.get("requirementIds")
        evidence_paths = correction.get("evidencePaths")
        if (
            not isinstance(old_name, str)
            or not old_name.strip()
            or any(marker in old_name for marker in "*?")
            or not isinstance(capability_id, str)
            or not capability_id.strip()
            or not isinstance(reason, str)
            or not reason.strip()
            or not isinstance(approval, dict)
            or approval.get("status") != "Approved"
            or not isinstance(requirements, list)
            or not requirements
            or any(not isinstance(value, str) or not value.strip() for value in requirements)
            or not isinstance(evidence_paths, list)
            or not evidence_paths
            or any(
                not isinstance(path, str)
                or not path.strip()
                or Path(path).is_absolute()
                or "\\" in path
                or ".." in Path(path).parts
                or not Path(path).is_file()
                for path in evidence_paths
            )
            or not isinstance(targets, list)
            or not targets
        ):
            raise ReconciliationError("Java stale route correction needs approved exact evidence")
        target_names: set[str] = set()
        for target in targets:
            if (
                not isinstance(target, dict)
                or set(target) != {"owner", "name"}
                or not isinstance(target.get("owner"), str)
                or not isinstance(target.get("name"), str)
                or not target["owner"].strip()
                or not target["name"].strip()
                or any(marker in target["name"] for marker in "*?")
            ):
                raise ReconciliationError("Java stale route targets must be exact public names")
            target_names.add(target["name"])
        for target_name in target_names:
            key = ("java-kmipkit", old_name, target_name)
            if key in stale_java_routes:
                raise ReconciliationError("Java stale route corrections must be unique")
            stale_java_routes[key] = correction
    result = copy.deepcopy(manifest)
    current = result.get("publicDeclarations")
    if not isinstance(current, list):
        raise ReconciliationError("manifest lacks its public declaration inventory")
    by_id = {row.get("declarationId"): row for row in current}
    by_export = {(row.get("packageId"), row.get("exportName")): row for row in current}
    capability_sources = {
        row.get("capabilityId"): row.get("approvalEvidence")
        for row in result.get("capabilities", [])
        if isinstance(row, dict)
    }
    capability_owners = {
        row.get("capabilityId"): row.get("ownerSpecId")
        for row in result.get("capabilities", [])
        if isinstance(row, dict)
    }
    additions_by_id: dict[str, dict[str, Any]] = {}
    additions_by_export: dict[tuple[str, str], dict[str, Any]] = {}
    for declaration in declarations:
        if not isinstance(declaration, dict):
            raise ReconciliationError("additional public declaration must be an object")
        identifier = declaration.get("declarationId")
        package_id = declaration.get("packageId")
        export_name = declaration.get("exportName")
        if not all(
            isinstance(value, str) and value.strip()
            for value in (identifier, package_id, export_name)
        ):
            raise ReconciliationError(
                "additional public declaration needs stable identity fields"
            )
        key = (package_id, export_name)
        if identifier in additions_by_id or key in additions_by_export:
            raise ReconciliationError(
                "additional public declaration IDs and exports must be unique"
            )
        if declaration.get("declarationState") != "verified" or declaration.get(
            "parityScope"
        ) not in {"four-language", "c-only"}:
            raise ReconciliationError(
                "additional declaration requires verified four-language parity or an exact C-only surface"
            )
        mappings = declaration.get("languageMappings")
        required_languages = (
            {"c"} if declaration["parityScope"] == "c-only" else REQUIRED_LANGUAGES
        )
        if not isinstance(mappings, dict) or set(mappings) != required_languages:
            raise ReconciliationError(
                "additional declaration must map all languages in its declared parity scope"
            )
        for language, mapping in mappings.items():
            if (
                not isinstance(mapping, dict)
                or set(mapping) != {"packageId", "name"}
                or mapping.get("packageId") not in LANGUAGE_PACKAGES[language]
                or not isinstance(mapping.get("name"), str)
                or not mapping["name"].strip()
            ):
                raise ReconciliationError(
                    f"additional declaration has an invalid {language} mapping"
                )
        evidence = declaration.get("parityEvidenceIds")
        if (
            not isinstance(evidence, list)
            or not evidence
            or any(not isinstance(path, str) or not path.strip() for path in evidence)
            or len(evidence) != len(set(evidence))
        ):
            raise ReconciliationError(
                "additional declaration needs unique parity evidence paths"
            )
        requirements = declaration.get("requirementIds")
        if (
            not isinstance(requirements, list)
            or not requirements
            or any(
                not isinstance(requirement, str) or not requirement.strip()
                for requirement in requirements
            )
        ):
            raise ReconciliationError(
                "additional declaration needs stable requirement IDs"
            )
        candidate_spec_id = declaration.get("candidateSpecId")
        if candidate_spec_id is not None:
            approval_source = declaration.get("approvalSource")
            owner_approval_source = capability_sources.get(
                declaration.get("capabilityId")
            )
            if (
                candidate_spec_id != SPEC_ID
                or not isinstance(approval_source, dict)
                or approval_source.get("status") != "Approved"
                or not (
                    approval_source.get("path") == APPROVAL_RECORD_PATH
                    or approval_source == owner_approval_source
                )
                or not any(
                    isinstance(requirement, str)
                    and requirement.startswith(f"{SPEC_ID}-FR-")
                    for requirement in requirements
                )
            ):
                raise ReconciliationError(
                    "additional declaration needs exact KMIPKIT-0053 approval and requirement traceability"
                )
        additions_by_id[identifier] = declaration
        additions_by_export[key] = declaration
        existing_id = by_id.get(identifier)
        existing_export = by_export.get(key)
        if existing_id is not None and existing_id is not existing_export:
            old_export_name = existing_id.get("exportName")
            existing_mappings = existing_id.get("languageMappings")
            candidate_mappings = declaration.get("languageMappings")
            python_correction = stale_python_routes.get(
                (package_id, old_export_name)
            )
            other_languages_match = (
                isinstance(existing_mappings, dict)
                and isinstance(candidate_mappings, dict)
                and all(
                    existing_mappings.get(language) == candidate_mappings.get(language)
                    for language in REQUIRED_LANGUAGES - {"python"}
                )
            )
            valid_python_rename = (
                python_correction is not None
                and package_id == "python-kmipkit"
                and existing_id.get("packageId") == package_id
                and existing_id.get("capabilityId") == declaration.get("capabilityId")
                and existing_id.get("capabilityId") == python_correction.get("capabilityId")
                and existing_id.get("kind") == declaration.get("kind")
                and existing_id.get("declarationState") == declaration.get("declarationState")
                and existing_id.get("parityScope") == declaration.get("parityScope")
                and existing_id.get("approvalSource") == declaration.get("approvalSource")
                and python_correction.get("correctedName") == export_name
                and isinstance(existing_mappings, dict)
                and isinstance(candidate_mappings, dict)
                and existing_mappings.get("python", {}).get("name") == old_export_name
                and candidate_mappings.get("python", {}).get("name") == export_name
                and other_languages_match
                and set(python_correction.get("requirementIds", []))
                <= set(declaration.get("requirementIds", []))
            )
            if valid_python_rename:
                old_key = (existing_id.get("packageId"), old_export_name)
                by_export.pop(old_key, None)
                existing_id["exportName"] = export_name
                existing_id["languageMappings"]["python"] = copy.deepcopy(
                    candidate_mappings["python"]
                )
                existing_id["requirementIds"] = sorted(
                    set(existing_id.get("requirementIds", [])) | set(requirements)
                )
                existing_id["parityEvidenceIds"] = sorted(
                    set(existing_id.get("parityEvidenceIds", []))
                    | set(evidence)
                    | set(python_correction.get("evidencePaths", []))
                )
                existing_export = existing_id
                by_export[key] = existing_export
            else:
                java_correction = stale_java_routes.get(
                    ("java-kmipkit", old_export_name, export_name)
                )
                valid_java_rename = (
                    java_correction is not None
                    and existing_id.get("packageId") == "java-kmipkit"
                    and package_id == "java-kmipkit"
                    and existing_id.get("kind") == declaration.get("kind")
                    and existing_id.get("capabilityId") == declaration.get("capabilityId")
                    and existing_id.get("declarationState") == declaration.get("declarationState")
                    and existing_id.get("parityScope") == declaration.get("parityScope")
                    and existing_id.get("approvalSource") == declaration.get("approvalSource")
                    and java_correction.get("approval") == declaration.get("approvalSource")
                    and existing_id.get("languageMappings", {}).get("java", {}).get("name")
                    == java_correction.get("oldName")
                    and declaration.get("languageMappings", {}).get("java", {}).get("name")
                    == export_name
                    and isinstance(existing_mappings, dict)
                    and isinstance(candidate_mappings, dict)
                    and all(
                        existing_mappings.get(language) == candidate_mappings.get(language)
                        for language in REQUIRED_LANGUAGES - {"java"}
                    )
                    and set(java_correction.get("requirementIds", []))
                    <= set(declaration.get("requirementIds", []))
                )
                if not valid_java_rename:
                    raise ReconciliationError(
                        f"public declaration ID already has another definition: {identifier}"
                    )
                old_key = (existing_id.get("packageId"), old_export_name)
                by_export.pop(old_key, None)
                existing_id["exportName"] = export_name
                existing_id["languageMappings"]["java"] = copy.deepcopy(
                    candidate_mappings["java"]
                )
                existing_id["requirementIds"] = sorted(
                    set(existing_id.get("requirementIds", [])) | set(requirements)
                )
                existing_id["parityEvidenceIds"] = sorted(
                    set(existing_id.get("parityEvidenceIds", []))
                    | set(evidence)
                    | set(java_correction.get("evidencePaths", []))
                )
                existing_export = existing_id
                by_export[key] = existing_export
        if existing_export is not None:
            corrected_evidence: set[str] = set()
            if existing_export.get("kind") != declaration.get("kind"):
                kind_correction = python_kind_routes.get(
                    (package_id, export_name)
                )
                rust_correction = rust_kind_routes.get((package_id, export_name))
                python_kind_correction_matches = (
                    package_id == "python-kmipkit"
                    and kind_correction is not None
                    and existing_export.get("kind") == kind_correction.get("fromKind")
                    and declaration.get("kind") == kind_correction.get("toKind")
                    and existing_export.get("capabilityId")
                    == declaration.get("capabilityId")
                    == kind_correction.get("capabilityId")
                    and set(kind_correction.get("requirementIds", []))
                    <= set(declaration.get("requirementIds", []))
                )
                rust_kind_correction_matches = (
                    package_id in LANGUAGE_PACKAGES["rust"]
                    and rust_correction is not None
                    and existing_export.get("kind") == rust_correction.get("manifestKind")
                    and declaration.get("kind") == rust_correction.get("observedKind")
                )
                if not (
                    python_kind_correction_matches or rust_kind_correction_matches
                ):
                    raise ReconciliationError(
                        f"public export already has another declaration: {export_name}"
                    )
                existing_export["kind"] = declaration["kind"]
                correction_paths = (
                    kind_correction["evidencePaths"]
                    if python_kind_correction_matches
                    else rust_correction["evidencePaths"]
                )
                corrected_evidence.update(correction_paths)
            exact_route_shape = all(
                existing_export.get(field) == declaration.get(field)
                for field in (
                    "packageId", "exportName", "capabilityId",
                    "declarationState", "parityScope",
                )
            )
            if not exact_route_shape:
                raise ReconciliationError(
                    f"public export already has another declaration: {export_name}"
                )
            if existing_export.get("approvalSource") != declaration.get(
                "approvalSource"
            ):
                if not _is_approved_source_refresh(
                    existing_export.get("approvalSource"),
                    declaration.get("approvalSource"),
                    capability_sources.get(declaration.get("capabilityId")),
                ) and not _is_approved_0053_query_scope_transfer(
                    existing_export,
                    declaration,
                    candidate_spec_id,
                    capability_owners.get(declaration.get("capabilityId")),
                ):
                    raise ReconciliationError(
                        f"public export already has another approval source: {export_name}"
                    )
                existing_export["approvalSource"] = copy.deepcopy(
                    declaration["approvalSource"]
                )
            if candidate_spec_id is not None:
                existing_candidate = existing_export.get("candidateSpecId")
                if existing_candidate not in {None, candidate_spec_id}:
                    raise ReconciliationError(
                        f"public export already has another approval owner: {export_name}"
                    )
                if declaration.get("approvalSource") == capability_sources.get(
                    declaration.get("capabilityId")
                ):
                    existing_export.pop("candidateSpecId", None)
                else:
                    existing_export["candidateSpecId"] = candidate_spec_id
            existing_mappings = existing_export.get("languageMappings")
            candidate_mappings = declaration.get("languageMappings")
            if existing_mappings != candidate_mappings:
                if not isinstance(existing_mappings, dict) or not isinstance(
                    candidate_mappings, dict
                ):
                    raise ReconciliationError(
                        f"public export already has another declaration: {export_name}"
                    )
                for language in REQUIRED_LANGUAGES:
                    old_route = existing_mappings.get(language)
                    new_route = candidate_mappings.get(language)
                    if old_route == new_route:
                        continue
                    if not isinstance(old_route, dict) or not isinstance(new_route, dict):
                        raise ReconciliationError(
                            f"public export already has another declaration: {export_name}"
                        )
                    reviewed_mapping = reviewed_mapping_routes.get(
                        (identifier, language)
                    )
                    reviewed_mapping_matches = (
                        reviewed_mapping is not None
                        and reviewed_mapping.get("from") == old_route
                        and reviewed_mapping.get("to") == new_route
                        and reviewed_mapping.get("capabilityId")
                        == declaration.get("capabilityId")
                        and set(reviewed_mapping.get("requirementIds", []))
                        <= set(declaration.get("requirementIds", []))
                        and reviewed_mapping.get("approvalSource")
                        == declaration.get("approvalSource")
                        and isinstance(reviewed_mapping.get("evidencePaths"), list)
                        and bool(reviewed_mapping["evidencePaths"])
                    )
                    if reviewed_mapping_matches:
                        correction_matches = True
                        corrected_evidence.update(reviewed_mapping["evidencePaths"])
                    elif language == "python":
                        correction = stale_python_routes.get(
                            ("python-kmipkit", old_route.get("name"))
                        )
                        correction_matches = (
                            correction is not None
                            and old_route.get("packageId") == "python-kmipkit"
                            and new_route.get("packageId") == "python-kmipkit"
                            and new_route.get("name") == correction.get("correctedName")
                            and correction.get("capabilityId")
                            == declaration.get("capabilityId")
                            and set(correction.get("requirementIds", []))
                            <= set(declaration.get("requirementIds", []))
                        )
                        if correction_matches:
                            corrected_evidence.update(correction["evidencePaths"])
                    elif language == "java":
                        correction = stale_java_routes.get(
                            (
                                "java-kmipkit",
                                old_route.get("name"),
                                new_route.get("name"),
                            )
                        )
                        correction_matches = (
                            correction is not None
                            and old_route.get("packageId") == "java-kmipkit"
                            and new_route.get("packageId") == "java-kmipkit"
                            and correction.get("capabilityId")
                            == declaration.get("capabilityId")
                            and correction.get("approval")
                            == declaration.get("approvalSource")
                            and set(correction.get("requirementIds", []))
                            <= set(declaration.get("requirementIds", []))
                        )
                        if correction_matches:
                            corrected_evidence.update(correction["evidencePaths"])
                    else:
                        correction_matches = False
                    if not correction_matches:
                        raise ReconciliationError(
                            f"public export already has another declaration: {export_name}"
                        )
                    existing_mappings[language] = copy.deepcopy(new_route)
            if corrected_evidence:
                existing_export["parityEvidenceIds"] = sorted(
                    set(existing_export.get("parityEvidenceIds", []))
                    | corrected_evidence
                )
            for field in ("cValue", "cValueType"):
                if field not in declaration:
                    continue
                existing_value = existing_export.get(field)
                declared_value = declaration[field]
                if existing_value not in {None, declared_value}:
                    raise ReconciliationError(
                        f"public export has conflicting {field}: {export_name}"
                    )
                existing_export[field] = declared_value
            canonical = copy.deepcopy(existing_export)
            canonical["requirementIds"] = sorted(
                set(existing_export.get("requirementIds", [])) | set(requirements)
            )
            canonical["parityEvidenceIds"] = sorted(
                set(existing_export.get("parityEvidenceIds", [])) | set(evidence)
            )
            existing_export["requirementIds"] = canonical["requirementIds"]
            existing_export["parityEvidenceIds"] = canonical["parityEvidenceIds"]
            del additions_by_id[identifier]
            additions_by_id[canonical["declarationId"]] = canonical
            additions_by_export[key] = canonical
    for identifier, declaration in additions_by_id.items():
        if identifier not in by_id:
            approved = copy.deepcopy(declaration)
            if approved.get("approvalSource") == capability_sources.get(
                approved.get("capabilityId")
            ):
                approved.pop("candidateSpecId", None)
            current.append(approved)
    capabilities = result.get("capabilities")
    if not isinstance(capabilities, list):
        raise ReconciliationError("manifest lacks its capability inventory")
    capabilities_by_id = {
        row.get("capabilityId"): row for row in capabilities if isinstance(row, dict)
    }
    for declaration in additions_by_id.values():
        capability_id = declaration.get("capabilityId")
        capability = capabilities_by_id.get(capability_id)
        if capability is None:
            raise ReconciliationError(
                f"public declaration references an unknown capability: {capability_id}"
            )
        mappings = capability.get("mappings")
        if not isinstance(mappings, dict):
            raise ReconciliationError(
                f"capability lacks reviewed language mappings: {capability_id}"
            )
        for language, mapping in declaration["languageMappings"].items():
            language_mappings = mappings.get(language)
            if not isinstance(language_mappings, list):
                raise ReconciliationError(
                    f"capability lacks {language} mappings: {capability_id}"
                )
            name = mapping["name"]
            if name not in language_mappings:
                language_mappings.append(name)
                language_mappings.sort()
        capability["requirementIds"] = sorted(
            set(capability.get("requirementIds", []))
            | set(declaration["requirementIds"])
        )
        capability["parityEvidenceIds"] = sorted(
            set(capability.get("parityEvidenceIds", []))
            | set(declaration["parityEvidenceIds"])
        )
    result["requirementIds"] = sorted(
        set(result.get("requirementIds", []))
        | {
            requirement
            for row in additions_by_id.values()
            for requirement in row.get("requirementIds", [])
        }
    )
    return result


def _is_approved_0053_query_scope_transfer(
    existing: dict[str, Any],
    replacement: dict[str, Any],
    candidate_spec_id: Any,
    owner_spec_id: Any,
) -> bool:
    """Allow 0053 to own only the explicitly approved 0052 typed-query parity surface."""
    old_approval = existing.get("approvalSource")
    new_approval = replacement.get("approvalSource")
    requirements = replacement.get("requirementIds", [])
    declaration_id = replacement.get("declarationId", "")
    export_name = replacement.get("exportName", "")
    return (
        candidate_spec_id == SPEC_ID
        and owner_spec_id == "KMIPKIT-0052"
        and replacement.get("capabilityId") == "response_ttlv_navigation"
        and isinstance(declaration_id, str)
        and (
            declaration_id.startswith(f"{SPEC_ID}:")
            or declaration_id.startswith("T064:query-field-")
        )
        and isinstance(export_name, str)
        and export_name in QUERY_RESPONSE_EXCLUSION_EXPORTS
        and isinstance(requirements, list)
        and {"KMIPKIT-0053-FR-008", "KMIPKIT-0053-FR-012"}
        <= set(requirements)
        and isinstance(old_approval, dict)
        and old_approval.get("kind") == "feature-specification"
        and old_approval.get("path")
        in {
            "specs/052-response-navigation/spec.md",
            "specs/052-response-navigation/approval-record.md",
        }
        and isinstance(new_approval, dict)
        and new_approval.get("kind") == "feature-specification"
        and new_approval.get("path") == APPROVAL_RECORD_PATH
        and new_approval.get("status") == "Approved"
        and "Direct maintainer instruction approving the complete Phase 5 plan"
        in new_approval.get("evidence", "")
    )


def apply_language_mapping_corrections(
    manifest: dict[str, Any], corrections: list[dict[str, Any]] | None = None
) -> dict[str, Any]:
    """Apply reviewed, declaration-specific mapping corrections to listed exports."""
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    packages = result.get("publicPackages")
    if not isinstance(declarations, list) or not isinstance(packages, list):
        raise ReconciliationError(
            "mapping corrections need the public declaration and package inventories"
        )
    package_languages = {
        row.get("id"): row.get("language")
        for row in packages
        if isinstance(row, dict)
    }
    by_id: dict[str, dict[str, Any]] = {}
    by_export: dict[tuple[str, str], dict[str, Any]] = {}
    listed_mapping_routes: dict[tuple[str, str, str], str] = {}
    for declaration in declarations:
        if not isinstance(declaration, dict):
            raise ReconciliationError("public declaration inventory contains a non-object")
        identifier = declaration.get("declarationId")
        key = declaration.get("packageId"), declaration.get("exportName")
        if not isinstance(identifier, str) or not identifier or identifier in by_id:
            raise ReconciliationError("public declaration IDs must be unique for mapping corrections")
        if not all(isinstance(value, str) and value for value in key) or key in by_export:
            raise ReconciliationError("public export paths must be unique for mapping corrections")
        by_id[identifier] = declaration
        by_export[key] = declaration
        mappings = declaration.get("languageMappings")
        if isinstance(mappings, dict):
            for route_language, route in mappings.items():
                if (
                    route_language in LANGUAGE_PACKAGES
                    and isinstance(route, dict)
                    and route.get("packageId") in LANGUAGE_PACKAGES[route_language]
                    and isinstance(route.get("name"), str)
                    and route.get("name")
                ):
                    listed_mapping_routes[
                        (route["packageId"], route["name"], route_language)
                    ] = declaration.get("declarationState")

    correction_keys = {
        "declarationId", "language", "from", "to", "capabilityId",
        "requirementIds", "evidencePaths", "reason", "approvalSource",
    }
    optional_correction_keys = {
        "mappingSemantics", "lifecycleSemanticsByLanguage",
    }
    seen: set[tuple[str, str]] = set()
    for correction in corrections or []:
        if (
            not isinstance(correction, dict)
            or not correction_keys <= set(correction)
            or set(correction) - correction_keys - optional_correction_keys
        ):
            raise ReconciliationError(
                "language mapping correction has missing or extra fields"
            )
        declaration_id = correction.get("declarationId")
        language = correction.get("language")
        identity = (declaration_id, language)
        if identity in seen:
            raise ReconciliationError("language mapping corrections must be unique")
        seen.add(identity)
        if language not in LANGUAGE_PACKAGES:
            raise ReconciliationError("language mapping correction uses an unsupported language")
        declaration = by_id.get(declaration_id)
        if declaration is None:
            raise ReconciliationError(
                f"language mapping correction references an unknown declaration: {declaration_id}"
            )
        if (
            declaration.get("declarationState") not in {"declared", "verified"}
            or declaration.get("capabilityId") != correction.get("capabilityId")
        ):
            raise ReconciliationError(
                "language mapping correction does not match a ready declaration capability"
            )
        for route_key in ("from", "to"):
            route = correction.get(route_key)
            if (
                not isinstance(route, dict)
                or set(route) != {"packageId", "name"}
                or route.get("packageId") not in LANGUAGE_PACKAGES[language]
                or not isinstance(route.get("name"), str)
                or not route["name"].strip()
                or any(marker in route["name"] for marker in "*?")
            ):
                raise ReconciliationError(
                    f"language mapping correction {route_key} route must be exact"
                )
        requirements = correction.get("requirementIds")
        evidence = correction.get("evidencePaths")
        reason = correction.get("reason")
        approval = correction.get("approvalSource")
        if (
            not isinstance(requirements, list)
            or not requirements
            or any(
                not isinstance(value, str)
                or not value.startswith(f"{SPEC_ID}-FR-")
                for value in requirements
            )
            or len(requirements) != len(set(requirements))
            or not isinstance(evidence, list)
            or not evidence
            or len(evidence) != len(set(evidence))
            or any(
                not isinstance(path, str)
                or not path.strip()
                or Path(path).is_absolute()
                or "\\" in path
                or ".." in Path(path).parts
                or not (Path.cwd() / path).is_file()
                for path in evidence
            )
            or not isinstance(reason, str)
            or not reason.strip()
            or not isinstance(approval, dict)
            or set(approval) != {"kind", "path", "status", "evidence"}
            or approval.get("kind") != "feature-specification"
            or approval.get("path") != APPROVAL_RECORD_PATH
            or approval.get("status") != "Approved"
            or any(
                not isinstance(value, str) or not value.strip()
                for value in approval.values()
            )
        ):
            raise ReconciliationError(
                "language mapping correction needs exact approved traceability"
            )
        approval_path = Path.cwd() / approval["path"]
        if (
            not approval_path.is_file()
            or approval["evidence"]
            not in approval_path.read_text(encoding="utf-8")
        ):
            raise ReconciliationError(
                "approval evidence does not appear in its cited source"
            )
        mappings = declaration.get("languageMappings")
        if not isinstance(mappings, dict) or language not in mappings:
            raise ReconciliationError("language mapping correction lacks its current route")
        current = mappings[language]
        old_route = correction["from"]
        new_route = correction["to"]
        if current not in (old_route, new_route):
            raise ReconciliationError(
                f"language mapping correction source differs from the exact current route: {declaration_id}"
            )
        target = by_export.get((new_route["packageId"], new_route["name"]))
        mapped_target_state = listed_mapping_routes.get(
            (new_route["packageId"], new_route["name"], language)
        )
        if (
            package_languages.get(new_route["packageId"]) != language
            or not (
                target is not None
                and package_languages.get(target.get("packageId")) == language
                and target.get("declarationState") in {"declared", "verified"}
                or mapped_target_state in {"declared", "verified"}
            )
        ):
            raise ReconciliationError(
                f"language mapping correction target is not an exact listed export: {new_route['name']}"
            )
        if current == old_route:
            mappings[language] = copy.deepcopy(new_route)
        mapping_semantics = correction.get("mappingSemantics")
        lifecycle_semantics = correction.get("lifecycleSemanticsByLanguage")
        if mapping_semantics is not None:
            if mapping_semantics not in {
                "native-handle-release",
                "native-handle-release-managed-java-value",
                "secret-buffer-wipe",
            } or declaration.get("mappingSemantics") not in {None, mapping_semantics}:
                raise ReconciliationError(
                    "mapping semantics correction must name one exact approved lifecycle"
                )
            if lifecycle_semantics is not None and (
                mapping_semantics != "secret-buffer-wipe"
                or lifecycle_semantics
                != {
                    "rust": "wipe-on-final-owner-drop",
                    "c": "release-shared-reference-wipe-on-final-owner-drop",
                    "java": "wipe-owned-copy-on-close",
                    "python": "wipe-owned-copy-on-close",
                }
            ):
                raise ReconciliationError(
                    "secret buffer lifecycle correction must describe each language boundary"
                )
            if (
                mapping_semantics == "secret-buffer-wipe"
                and lifecycle_semantics is None
                and declaration.get("lifecycleSemanticsByLanguage") is None
            ):
                raise ReconciliationError(
                    "secret buffer lifecycle correction requires ownership semantics"
                )
            declaration["mappingSemantics"] = mapping_semantics
            if lifecycle_semantics is not None:
                declaration["lifecycleSemanticsByLanguage"] = copy.deepcopy(
                    lifecycle_semantics
                )
        candidate_spec = declaration.get("candidateSpecId")
        if candidate_spec not in {None, SPEC_ID}:
            raise ReconciliationError(
                f"language mapping correction conflicts with another approval owner: {declaration_id}"
            )
        declaration["requirementIds"] = sorted(
            set(declaration.get("requirementIds", [])) | set(requirements)
        )
        declaration["parityEvidenceIds"] = sorted(
            set(declaration.get("parityEvidenceIds", [])) | set(evidence)
        )
    return result


def remove_public_declarations(
    manifest: dict[str, Any], removals: list[dict[str, Any]]
) -> dict[str, Any]:
    """Remove only exact declarations explicitly corrected by the owning 0053 audit."""
    if not isinstance(removals, list):
        raise ReconciliationError("public declaration removals must be a list")
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ReconciliationError("manifest lacks its public declaration inventory")
    required = {
        "declarationId", "packageId", "exportName", "reason", "candidateSpecId",
        "approvalSource", "evidencePaths",
    }
    seen: set[str] = set()
    for removal in removals:
        if not isinstance(removal, dict) or set(removal) != required:
            raise ReconciliationError("public declaration removal has missing or extra fields")
        if removal.get("candidateSpecId") != SPEC_ID:
            raise ReconciliationError("public declaration removal must be owned by KMIPKIT-0053")
        identifier = removal.get("declarationId")
        package_id = removal.get("packageId")
        export_name = removal.get("exportName")
        reason = removal.get("reason")
        if (
            not isinstance(identifier, str) or not identifier.strip()
            or not isinstance(package_id, str) or not package_id.strip()
            or not isinstance(export_name, str) or not export_name.strip()
            or any(char in export_name for char in "*?")
            or not isinstance(reason, str) or not reason.strip()
            or identifier in seen
        ):
            raise ReconciliationError("public declaration removal must identify one exact route")
        seen.add(identifier)
        _validate_resolution_evidence(removal)
        if removal["approvalSource"].get("path") != APPROVAL_RECORD_PATH:
            raise ReconciliationError("public declaration removal needs the 0053 approval record")
        matched_id = [row for row in declarations if row.get("declarationId") == identifier]
        if matched_id and (
            len(matched_id) != 1
            or matched_id[0].get("packageId") != package_id
            or matched_id[0].get("exportName") != export_name
        ):
            raise ReconciliationError("public declaration removal does not match its exact reviewed target")
        declarations = [row for row in declarations if row.get("declarationId") != identifier]
    result["publicDeclarations"] = declarations
    return result


def remove_approved_exclusions(
    manifest: dict[str, Any], removals: list[dict[str, Any]]
) -> dict[str, Any]:
    """Remove only explicitly approved Query and secret-cleanup exclusions."""
    if not isinstance(removals, list):
        raise ReconciliationError("approved exclusion removals must be a list")
    result = copy.deepcopy(manifest)
    exclusions = result.get("approvedExclusions")
    declarations = result.get("publicDeclarations")
    if not isinstance(exclusions, list) or not isinstance(declarations, list):
        raise ReconciliationError(
            "manifest needs approved exclusions and public declarations for exact supersession"
        )

    seen: set[str] = set()
    remove_indexes: set[int] = set()
    for removal in removals:
        required = {
            "declarationId",
            "packageId",
            "exportName",
            "supersedesSpecId",
            "approvalSource",
            "evidencePaths",
        }
        allowed = required | {"supersessionRationale"}
        if (
            not isinstance(removal, dict)
            or not required <= set(removal)
            or not set(removal) <= allowed
        ):
            raise ReconciliationError(
                "approved exclusion removal needs its exact reviewed identity and evidence"
            )
        declaration_id = removal.get("declarationId")
        package_id = removal.get("packageId")
        export_name = removal.get("exportName")
        is_query_supersession = (
            isinstance(declaration_id, str)
            and declaration_id.startswith("KMIPKIT-0053:rust-query-response")
            and package_id == "rust-kmipkit"
            and export_name in QUERY_RESPONSE_EXCLUSION_EXPORTS
            and removal.get("supersedesSpecId") == "KMIPKIT-0020"
            and "supersessionRationale" not in removal
        )
        is_java_builder_close_supersession = (
            declaration_id == JAVA_BUILDER_CLOSE_DECLARATION_ID
            and package_id == JAVA_BUILDER_CLOSE_PACKAGE
            and export_name == JAVA_BUILDER_CLOSE_EXPORT
            and removal.get("supersedesSpecId") == SPEC_ID
            and removal.get("supersessionRationale")
            == JAVA_BUILDER_CLOSE_SUPERSESSION_RATIONALE
        )
        is_python_builder_close_supersession = (
            declaration_id == PYTHON_BUILDER_CLOSE_DECLARATION_ID
            and package_id == PYTHON_BUILDER_CLOSE_PACKAGE
            and export_name == PYTHON_BUILDER_CLOSE_EXPORT
            and removal.get("supersedesSpecId") == SPEC_ID
            and removal.get("supersessionRationale")
            == JAVA_BUILDER_CLOSE_SUPERSESSION_RATIONALE
        )
        if (
            (
                not is_query_supersession
                and not is_java_builder_close_supersession
                and not is_python_builder_close_supersession
            )
            or declaration_id in seen
        ):
            raise ReconciliationError(
                "only an exact KMIPKIT-0020 exclusion for a QueryResponse route, or an exact KMIPKIT-0053 Java/Python secret-cleanup exclusion, may be superseded"
            )
        seen.add(declaration_id)

        approval = removal.get("approvalSource")
        evidence_paths = removal.get("evidencePaths")
        expected_approval = {
            "kind": "feature-specification",
            "path": APPROVAL_RECORD_PATH,
            "status": "Approved",
            "evidence": (
                QUERY_RESPONSE_SCOPE_SUPERSESSION
                if is_query_supersession
                else JAVA_BUILDER_CLOSE_APPROVAL_EVIDENCE
            ),
        }
        expected_evidence = (
            QUERY_RESPONSE_SCOPE_EVIDENCE
            if is_query_supersession
            else JAVA_BUILDER_CLOSE_SUPERSESSION_EVIDENCE_PATHS
            if is_java_builder_close_supersession
            else frozenset(
                {
                    "specs/053-api-parity-facade/spec.md",
                    APPROVAL_RECORD_PATH,
                    *PYTHON_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS,
                    "bindings/c/tests/authentication_consumer.c",
                    "crates/kmipkit-protocol/src/credential/secret.rs",
                    "crates/kmipkit-protocol/src/cryptographic_parameters.rs",
                    "crates/kmipkit-protocol/tests/credential_secret.rs",
                }
            )
        )
        if (
            not isinstance(approval, dict)
            or set(approval) != {"kind", "path", "status", "evidence"}
            or approval != expected_approval
            or not isinstance(evidence_paths, list)
            or any(not isinstance(path, str) for path in evidence_paths)
            or set(evidence_paths) != expected_evidence
            or len(evidence_paths) != len(expected_evidence)
        ):
            raise ReconciliationError(
                "approved exclusion supersession needs exact approved 0053 scope evidence"
            )

        matching_declarations = [
            row for row in declarations
            if isinstance(row, dict) and row.get("declarationId") == declaration_id
        ]
        if (
            len(matching_declarations) != 1
            or matching_declarations[0].get("packageId") != package_id
            or matching_declarations[0].get("exportName") != export_name
            or matching_declarations[0].get("declarationState") != "verified"
            or (
                (is_java_builder_close_supersession or is_python_builder_close_supersession)
                and (
                    matching_declarations[0].get("capabilityId")
                    != "typed_cryptographic_parameters"
                    or matching_declarations[0].get("parityScope")
                    != "four-language"
                )
            )
        ):
            raise ReconciliationError(
                "superseded exclusion must match one verified exact KMIPKIT-0053 declaration"
            )

        matching_exclusions = [
            (index, row)
            for index, row in enumerate(exclusions)
            if isinstance(row, dict)
            and row.get("packageId") == package_id
            and row.get("exportName") == export_name
        ]
        if len(matching_exclusions) > 1:
            raise ReconciliationError(
                "expected at most one exact exclusion for the reviewed route"
            )
        if not matching_exclusions:
            # Reconciliation may be rerun after a prior successful write.
            # The verified declaration and exact approved removal record are
            # sufficient to make the already-removed state idempotent.
            continue
        index, old = matching_exclusions[0]
        old_approval = old.get("approvalSource")
        reason = old.get("reason")
        if is_query_supersession:
            expected_old_exclusion = (
                old.get("candidateSpecId") == "KMIPKIT-0020"
                and isinstance(old_approval, dict)
                and old_approval.get("kind") == "feature-specification"
                and old_approval.get("path") == "specs/020-query-ping/spec.md"
                and old_approval.get("status") == "Approved"
                and isinstance(reason, str)
                and "KMIPKIT-0020 excludes" in reason
                and (
                    "typed Query response model to C, Java, or Python" in reason
                    or "typed binding model" in reason
                )
            )
        elif is_java_builder_close_supersession:
            old_evidence_paths = old.get("evidencePaths")
            expected_old_exclusion = (
                old.get("candidateSpecId") == SPEC_ID
                and old_approval == expected_approval
                and reason == JAVA_BUILDER_CLOSE_EXCLUSION_REASON
                and isinstance(old_evidence_paths, list)
                and set(old_evidence_paths)
                == JAVA_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS
                and len(old_evidence_paths)
                == len(JAVA_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS)
                and old.get("evidence")
                == f"{reason} Evidence files: {', '.join(old_evidence_paths)}."
            )
        else:
            old_evidence_paths = old.get("evidencePaths")
            expected_old_exclusion = (
                old.get("candidateSpecId") == SPEC_ID
                and old_approval == expected_approval
                and reason == PYTHON_BUILDER_CLOSE_EXCLUSION_REASON
                and isinstance(old_evidence_paths, list)
                and tuple(old_evidence_paths)
                == PYTHON_BUILDER_CLOSE_EXCLUSION_EVIDENCE_PATHS
                and old.get("evidence")
                == f"{reason} Evidence files: {', '.join(old_evidence_paths)}."
            )
        if not expected_old_exclusion:
            raise ReconciliationError(
                "the exact approved exclusion no longer matches its reviewed rationale"
            )
        remove_indexes.add(index)

    result["approvedExclusions"] = [
        row for index, row in enumerate(exclusions) if index not in remove_indexes
    ]
    return result


def add_approved_exclusions(
    manifest: dict[str, Any], exclusions: list[dict[str, Any]]
) -> dict[str, Any]:
    """Add exact, evidence-backed exclusions from the reviewed public export audits."""
    if not isinstance(exclusions, list):
        raise ReconciliationError("approved exclusions must be a list")
    result = copy.deepcopy(manifest)
    current = result.get("approvedExclusions")
    if not isinstance(current, list):
        raise ReconciliationError("manifest lacks its approved exclusion inventory")
    by_export = {
        (row.get("packageId"), row.get("exportName")): row
        for row in current
        if isinstance(row, dict)
    }
    seen: set[tuple[str, str]] = set()
    for exclusion in exclusions:
        required = {"packageId", "exportName", "reason", "approvalSource", "evidence"}
        allowed = required | {
            "candidateSpecId", "declarationId", "capabilityId", "declarationKind", "artifactKind",
            "counterpartDeclarationIds", "evidencePaths", "rustArtifactKind", "identity",
            "condition", "ownerExportName", "ownerDeclarationId", "exclusionScope",
            "exclusionCategory", "precedent",
        }
        if (
            not isinstance(exclusion, dict)
            or not required <= set(exclusion)
            or not set(exclusion) <= allowed
        ):
            raise ReconciliationError("approved export exclusion has missing or extra fields")
        package_id = exclusion.get("packageId")
        export_name = exclusion.get("exportName")
        if (
            package_id not in set.union(*LANGUAGE_PACKAGES.values())
            or not isinstance(export_name, str)
            or not export_name.strip()
            or any(char in export_name for char in "*?")
            or not isinstance(exclusion.get("reason"), str)
            or not exclusion["reason"].strip()
            or not isinstance(exclusion.get("evidence"), str)
            or not exclusion["evidence"].strip()
        ):
            raise ReconciliationError("approved export exclusion must identify one exact route")
        capability_id = exclusion.get("capabilityId")
        if capability_id is not None:
            capabilities = result.get("capabilities")
            matching_capabilities = [
                row
                for row in capabilities or []
                if isinstance(row, dict) and row.get("capabilityId") == capability_id
            ]
            if (
                not isinstance(capability_id, str)
                or not capability_id.strip()
                or len(matching_capabilities) != 1
            ):
                raise ReconciliationError("export exclusion must reference one exact capability")
            counterpart_ids = exclusion.get("counterpartDeclarationIds")
            if counterpart_ids is not None:
                declarations = result.get("publicDeclarations")
                declarations_by_id = {
                    row.get("declarationId"): row
                    for row in declarations or []
                    if isinstance(row, dict)
                }
                if (
                    not isinstance(counterpart_ids, list)
                    or not counterpart_ids
                    or any(
                        not isinstance(identifier, str)
                        for identifier in counterpart_ids
                    )
                    or len(counterpart_ids) != len(set(counterpart_ids))
                    or any(
                        declarations_by_id.get(identifier, {}).get("capabilityId")
                        != capability_id
                        for identifier in counterpart_ids
                    )
                ):
                    raise ReconciliationError(
                        "export exclusion capability must match its exact counterpart declarations"
                    )
        key = (package_id, export_name)
        if package_id == "c-kmipkit" and export_name == CFFI_BRIDGE_EXCLUSION[0]:
            expected = (
                CFFI_BRIDGE_EXCLUSION[1],
                CFFI_BRIDGE_EXCLUSION[2],
                "KMIPKIT-0053",
                ("bindings/java/native/kmipkit_jni.cpp", "bindings/python/src/kmipkit/ttlv.py"),
            )
        else:
            expected = None
        if expected is not None:
            if (
                exclusion.get("reason") != expected[0]
                or exclusion.get("approvalSource") != expected[1]
                or exclusion.get("candidateSpecId") != expected[2]
                or not all(reference in exclusion["evidence"] for reference in expected[3])
            ):
                raise ReconciliationError("CFFI exclusion is not the exact approved bridge route")
        else:
            approval = exclusion.get("approvalSource")
            if (
                not isinstance(approval, dict)
                or approval.get("status") != "Approved"
                or not isinstance(approval.get("path"), str)
                or not approval["path"].startswith("specs/")
                or ".." in PurePosixPath(approval["path"]).parts
                or "\\" in approval["path"]
                or not (Path.cwd() / approval["path"]).is_file()
            ):
                raise ReconciliationError("export exclusion needs an exact approved source")
            candidate_spec = exclusion.get("candidateSpecId")
            if candidate_spec not in {None, "KMIPKIT-0004", SPEC_ID}:
                raise ReconciliationError("export exclusion has an unsupported owner specification")
            if candidate_spec == SPEC_ID and approval["path"] not in {
                APPROVAL_RECORD_PATH,
                "specs/052-response-navigation/spec.md",
            }:
                raise ReconciliationError("KMIPKIT-0053 exclusion source is not an approved in-scope specification")
            if candidate_spec == "KMIPKIT-0004" and approval["path"] != "specs/004-generic-ttlv-model/spec.md":
                raise ReconciliationError("prior ModelError exclusion must cite its owning approved specification")
            if candidate_spec is None:
                raise ReconciliationError("non-legacy export exclusions need an exact owning specification")
            evidence_paths = exclusion.get("evidencePaths")
            if evidence_paths is not None:
                if not isinstance(evidence_paths, list) or not evidence_paths or len(evidence_paths) != len(set(evidence_paths)):
                    raise ReconciliationError("export exclusions need unique evidence paths")
                for path in evidence_paths:
                    parsed = PurePosixPath(path) if isinstance(path, str) else None
                    if (
                        parsed is None or parsed.is_absolute() or ".." in parsed.parts
                        or "\\" in path or any(char in path for char in "*?")
                        or not (Path.cwd() / path).is_file()
                    ):
                        raise ReconciliationError("export exclusion evidence must reference exact repository files")
            rust_kind = exclusion.get("rustArtifactKind")
            if rust_kind is not None:
                if rust_kind not in {
                    "rust-trait-implementation-member",
                    "rust-conditional-trait-implementation-member",
                    "exact-route",
                    "ownership-transfer-helper",
                }:
                    raise ReconciliationError("Rust exclusion uses an unsupported export-audit category")
                if package_id not in LANGUAGE_PACKAGES["rust"]:
                    raise ReconciliationError("Rust artifact exclusions must use a Rust package")
                if "trait-implementation-member" in rust_kind and not exclusion.get("identity"):
                    raise ReconciliationError("Rust trait implementation exclusion needs its exact Rustdoc identity")
                if "conditional-trait-implementation-member" in rust_kind and not exclusion.get("condition"):
                    raise ReconciliationError("conditional Rust trait implementation exclusion needs its exact condition")
        if key in seen:
            raise ReconciliationError("approved export exclusions must be unique")
        seen.add(key)
        existing = by_export.get(key)
        if existing is not None and existing != exclusion:
            if _is_exact_legacy_model_error_approval_correction(existing, exclusion):
                current[current.index(existing)] = copy.deepcopy(exclusion)
                by_export[key] = exclusion
            elif _is_exact_0053_query_field_kind_counterpart_correction(
                existing, exclusion
            ):
                current[current.index(existing)] = copy.deepcopy(exclusion)
                by_export[key] = exclusion
            elif _is_exact_0053_managed_object_counterpart_correction(
                existing, exclusion
            ):
                current[current.index(existing)] = copy.deepcopy(exclusion)
                by_export[key] = exclusion
            else:
                raise ReconciliationError(
                    f"approved export exclusion conflicts with its reviewed record: {export_name}"
                )
        if existing is None:
            current.append(copy.deepcopy(exclusion))
            by_export[key] = exclusion
    return result


def _is_exact_0053_query_field_kind_counterpart_correction(
    previous: dict[str, Any], corrected: dict[str, Any]
) -> bool:
    """Allow the four exact facade variant exclusions to cite their facade parent."""
    names = {
        "kmipkit::protocol::QueryResponseFieldKind::Enumeration",
        "kmipkit::protocol::QueryResponseFieldKind::Structure",
        "kmipkit::protocol::QueryResponseFieldKind::TextString",
        "kmipkit::protocol::QueryResponseFieldKind::Unknown",
    }
    if (
        previous.get("packageId") != "rust-kmipkit"
        or previous.get("exportName") not in names
        or corrected.get("packageId") != previous.get("packageId")
        or corrected.get("exportName") != previous.get("exportName")
        or previous.get("counterpartDeclarationIds")
        != ["T064:query-field-kind:rust"]
        or corrected.get("counterpartDeclarationIds")
        != ["KMIPKIT-0053:rust-facade-query-field-kind"]
    ):
        return False
    expected = copy.deepcopy(previous)
    expected["counterpartDeclarationIds"] = corrected[
        "counterpartDeclarationIds"
    ]
    return expected == corrected


def _is_exact_0053_managed_object_counterpart_correction(
    previous: dict[str, Any], corrected: dict[str, Any]
) -> bool:
    """Correct only the exact managed-object enum-shape counterpart records."""
    package_id = previous.get("packageId")
    package = "kmipkit::protocol" if package_id == "rust-kmipkit" else (
        "kmipkit_protocol" if package_id == "rust-kmipkit-protocol" else None
    )
    export_name = previous.get("exportName")
    if not isinstance(package, str) or not isinstance(export_name, str):
        return False
    for enum_name, allowed_suffixes in MANAGED_OBJECT_VARIANT_SUFFIXES.items():
        prefix = f"{package}::{enum_name}::"
        if not export_name.startswith(prefix):
            continue
        suffix = export_name.removeprefix(prefix)
        old_id, new_id = MANAGED_OBJECT_VARIANT_COUNTERPARTS[
            (package_id, enum_name)
        ]
        if (
            suffix not in allowed_suffixes
            or corrected.get("packageId") != package_id
            or corrected.get("exportName") != export_name
            or previous.get("counterpartDeclarationIds") != [old_id]
            or corrected.get("counterpartDeclarationIds") != [new_id]
        ):
            return False
        expected = copy.deepcopy(previous)
        expected["counterpartDeclarationIds"] = corrected[
            "counterpartDeclarationIds"
        ]
        return expected == corrected
    return False


def _is_exact_legacy_model_error_approval_correction(
    previous: dict[str, Any], corrected: dict[str, Any]
) -> bool:
    """Restore the exact KMIPKIT-0004 approval quotation for two legacy routes."""
    expected_routes = {
        ("rust-kmipkit", "kmipkit::ttlv::ModelError::UnsupportedValue"),
        ("rust-kmipkit-ttlv", "kmipkit_ttlv::ModelError::UnsupportedValue"),
    }
    key = (previous.get("packageId"), previous.get("exportName"))
    if key not in expected_routes or key != (
        corrected.get("packageId"), corrected.get("exportName")
    ):
        return False
    approval_source_prefix = {
        "kind": "feature-specification",
        "path": "specs/004-generic-ttlv-model/spec.md",
        "status": "Approved",
    }
    allowed_previous_approvals = [
        {
            **approval_source_prefix,
            "evidence": "- **KMIPKIT-0004-FR-013**: Errors created by the TTLV value model MUST use safe, value-free diagnostics and MUST NOT expose payload contents. Because `kmipkit-ttlv` is below protocol/client crates, its model error type MUST remain local and MUST NOT introduce a dependency cycle; higher layers may map or wrap it through the shared error model.",
        },
        {
            **approval_source_prefix,
            "evidence": (
                "KMIPKIT-0004-FR-013 approves excluding these exact local ModelError "
                "variants from the cross-language public route inventory."
            ),
        },
    ]
    exact_approved_quotation = {
        **approval_source_prefix,
        "evidence": (
            "Because `kmipkit-ttlv` is below protocol/client crates, its model error "
            "type MUST remain local and MUST NOT introduce a dependency cycle; higher "
            "layers may map or wrap it through the shared error model."
        ),
    }
    if (
        previous.get("candidateSpecId") != "KMIPKIT-0004"
        or corrected.get("candidateSpecId") != "KMIPKIT-0004"
        or previous.get("approvalSource") not in allowed_previous_approvals
        or corrected.get("approvalSource") != exact_approved_quotation
    ):
        return False
    normalized_previous = copy.deepcopy(previous)
    normalized_previous["approvalSource"] = exact_approved_quotation
    return normalized_previous == corrected


def _synchronize_capability_mappings(manifest: dict[str, Any]) -> dict[str, Any]:
    """Rebuild capability route lists from their exact ready declarations."""
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    capabilities = result.get("capabilities")
    packages = result.get("publicPackages")
    if not all(isinstance(value, list) for value in (declarations, capabilities, packages)):
        raise ReconciliationError(
            "capability mapping synchronization requires declarations, capabilities, and packages"
        )
    package_languages = {
        package.get("id"): package.get("language")
        for package in packages
        if isinstance(package, dict)
    }
    for capability in capabilities:
        if not isinstance(capability, dict):
            raise ReconciliationError("capability mapping record must be an object")
        scope = capability.get("parityScope")
        languages = (
            ("rust",)
            if scope == "rust-only-adr"
            else ("c",)
            if scope == "c-only"
            else tuple(sorted(REQUIRED_LANGUAGES))
        )
        capability_mappings = capability.get("mappings")
        if not isinstance(capability_mappings, dict):
            raise ReconciliationError("capability mappings must be an object")
        for language in languages:
            capability_mappings[language] = sorted(
                {
                    declaration["languageMappings"][language]["name"]
                    for declaration in declarations
                    if isinstance(declaration, dict)
                    and declaration.get("capabilityId") == capability.get("capabilityId")
                    and declaration.get("declarationState") in {"declared", "verified"}
                    and isinstance(declaration.get("languageMappings"), dict)
                    and language in declaration["languageMappings"]
                    and isinstance(declaration["languageMappings"][language], dict)
                    and package_languages.get(
                        declaration["languageMappings"][language].get("packageId")
                    ) == language
                }
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
        exact_verified_replay = (
            row is not None
            and row.get("declarationState") == "verified"
            and row.get("capabilityId") == resolution.get("capabilityId")
            and row.get("requirementIds") == resolution.get("requirementIds")
            and row.get("parityScope") == "four-language"
            and row.get("languageMappings")
            == {language: resolution[language] for language in REQUIRED_LANGUAGES}
            and row.get("parityEvidenceIds") == resolution.get("parityEvidenceIds")
        )
        if exact_verified_replay:
            old_approval = row.get("approvalSource")
            new_approval = resolution.get("approvalSource")
            if old_approval == new_approval:
                if (
                    new_approval == resolution.get("approvalSource")
                    and isinstance(new_approval, dict)
                    and new_approval.get("path") == APPROVAL_RECORD_PATH
                    and new_approval.get("status") == "Approved"
                    and any(
                        isinstance(requirement, str)
                        and requirement.startswith(f"{SPEC_ID}-FR-")
                        for requirement in resolution.get("requirementIds", [])
                    )
                ):
                    existing_candidate = row.get("candidateSpecId")
                    if existing_candidate not in {None, SPEC_ID}:
                        raise ReconciliationError(
                            "verified declaration replay conflicts with another approval owner"
                        )
                    row["candidateSpecId"] = SPEC_ID
                continue
            if _is_approved_source_refresh(old_approval, new_approval):
                row["approvalSource"] = copy.deepcopy(new_approval)
                continue
            raise ReconciliationError(
                "verified declaration replay may only refresh stale evidence from the same approved record"
            )
        transfer_from_0043 = (
            row is not None
            and row.get("candidateSpecId") == "KMIPKIT-0043"
            and resolution.get("fromSpecId") == "KMIPKIT-0043"
        )
        transfer_client_operation_from_0012 = (
            row is not None
            and row.get("candidateSpecId") == "KMIPKIT-0012"
            and resolution.get("fromSpecId") == "KMIPKIT-0012"
            and row.get("packageId") == "rust-kmipkit-client"
            and row.get("exportName")
            == "kmipkit_client::ClientBatchItem::operation"
        )
        if (
            row is None
            or row.get("declarationState") != "blocked"
            or not (transfer_from_0043 or transfer_client_operation_from_0012)
        ):
            raise ReconciliationError(
                "only an exact blocked KMIPKIT-0043 declaration may be transferred"
            )
        if set(resolution) != {
            "declarationId",
            "fromSpecId",
            "capabilityId",
            "requirementIds",
            "rust",
            "c",
            "java",
            "python",
            "parityEvidenceIds",
            "approvalSource",
        }:
            raise ReconciliationError(
                "declaration resolution has missing or unrecognized fields"
            )
        requirements = resolution["requirementIds"]
        if (
            not isinstance(requirements, list)
            or not requirements
            or len(requirements) != len(set(requirements))
        ):
            raise ReconciliationError(
                "declaration resolution needs unique requirement IDs"
            )
        if not any(
            isinstance(requirement, str)
            and requirement.startswith(f"{SPEC_ID}-FR-")
            for requirement in requirements
        ):
            raise ReconciliationError(
                "declaration resolution must include a KMIPKIT-0053 requirement"
            )
        route = {language: resolution[language] for language in REQUIRED_LANGUAGES}
        route["parityEvidenceIds"] = resolution["parityEvidenceIds"]
        route["approvalSource"] = resolution["approvalSource"]
        mappings = _validate_mapping({"kind": row.get("kind")}, route)
        for language, mapping in mappings.items():
            if mapping["packageId"] not in LANGUAGE_PACKAGES[language]:
                raise ReconciliationError(
                    f"{language} resolution targets an unsupported package"
                )
        row["capabilityId"] = resolution["capabilityId"]
        row["requirementIds"] = requirements
        row["declarationState"] = "verified"
        row["parityScope"] = "four-language"
        row["languageMappings"] = mappings
        row["parityEvidenceIds"] = resolution["parityEvidenceIds"]
        row["approvalSource"] = resolution["approvalSource"]
        row["candidateSpecId"] = SPEC_ID
        row.pop("blockedReason", None)
    result["requirementIds"] = sorted(
        set(result.get("requirementIds", []))
        | {requirement for row in resolutions for requirement in row["requirementIds"]}
    )
    return result


def _is_partial_predecessor_correction(
    declaration: dict[str, Any],
    source: dict[str, Any],
    target: dict[str, Any],
) -> bool:
    """Match only the predecessor state with corrected ownership and stale foreign routes."""
    current_mappings = declaration.get("languageMappings")
    source_mappings = source.get("languageMappings")
    target_mappings = target.get("languageMappings")
    return (
        declaration.get("capabilityId") == target.get("capabilityId")
        and declaration.get("requirementIds") == target.get("requirementIds")
        and isinstance(current_mappings, dict)
        and current_mappings.get("rust")
        == {
            "packageId": declaration["packageId"],
            "name": declaration["exportName"],
        }
        and isinstance(source_mappings, dict)
        and isinstance(target_mappings, dict)
        and all(
            current_mappings.get(language)
            in (source_mappings.get(language), target_mappings.get(language))
            for language in ("c", "java", "python")
        )
        and any(
            current_mappings.get(language) != target_mappings.get(language)
            for language in ("c", "java", "python")
        )
    )


def apply_declaration_corrections(
    manifest: dict[str, Any], corrections: list[dict[str, Any]]
) -> dict[str, Any]:
    """Correct exact approved declaration ownership using same-package operation anchors."""
    if not isinstance(corrections, list):
        raise ReconciliationError("declaration corrections must be a list")
    result = copy.deepcopy(manifest)
    declarations = result.get("publicDeclarations")
    operations = result.get("operations")
    if not isinstance(declarations, list) or not isinstance(operations, list):
        raise ReconciliationError("declaration corrections require declarations and operations")
    by_id = {row.get("declarationId"): row for row in declarations}
    if len(by_id) != len(declarations):
        raise ReconciliationError("declaration correction input contains duplicate IDs")
    seen: set[str] = set()
    required_fields = {
        "declarationId",
        "exportName",
        "sourceDeclarationId",
        "targetDeclarationId",
        "fromCapabilityId",
        "toCapabilityId",
        "reason",
        "evidencePaths",
    }

    for correction in corrections:
        if not isinstance(correction, dict) or set(correction) != required_fields:
            raise ReconciliationError("declaration correction has missing or unrecognized fields")
        declaration_id = correction.get("declarationId")
        if (
            not isinstance(declaration_id, str)
            or not declaration_id.strip()
            or any(marker in declaration_id for marker in "*?")
            or declaration_id in seen
        ):
            raise ReconciliationError("declaration correction IDs must be exact and unique")
        seen.add(declaration_id)

        declaration = by_id.get(declaration_id)
        source = by_id.get(correction.get("sourceDeclarationId"))
        target = by_id.get(correction.get("targetDeclarationId"))
        reason = correction.get("reason")
        evidence = correction.get("evidencePaths")
        if (
            declaration is None
            or source is None
            or target is None
            or not isinstance(reason, str)
            or not reason.strip()
            or not isinstance(evidence, list)
            or not evidence
            or any(
                not isinstance(path, str)
                or not path.strip()
                or Path(path).is_absolute()
                or "\\" in path
                or ".." in Path(path).parts
                or not Path(path).is_file()
                for path in evidence
            )
            or len(evidence) != len(set(evidence))
        ):
            raise ReconciliationError("declaration correction needs exact records and evidence")
        approval = declaration.get("approvalSource")
        if (
            declaration.get("exportName") != correction.get("exportName")
            or declaration.get("kind") != "associated-item"
            or declaration.get("packageId") not in LANGUAGE_PACKAGES["rust"]
            or declaration.get("declarationState") != "verified"
            or declaration.get("parityScope") != "four-language"
            or source.get("packageId") != declaration.get("packageId")
            or target.get("packageId") != declaration.get("packageId")
            or source.get("declarationState") != "verified"
            or target.get("declarationState") != "verified"
            or source.get("capabilityId") != correction.get("fromCapabilityId")
            or target.get("capabilityId") != correction.get("toCapabilityId")
            or correction.get("fromCapabilityId") == correction.get("toCapabilityId")
            or not isinstance(approval, dict)
            or approval != source.get("approvalSource")
            or approval != target.get("approvalSource")
            or approval.get("status") != "Approved"
        ):
            raise ReconciliationError("declaration correction does not match its exact approved anchors")

        mappings = (
            declaration.get("languageMappings"),
            source.get("languageMappings"),
            target.get("languageMappings"),
        )
        if any(
            not isinstance(language_mappings, dict)
            or set(language_mappings) != set(REQUIRED_LANGUAGES)
            for language_mappings in mappings
        ):
            raise ReconciliationError("declaration correction anchors require four-language mappings")
        expected_language_mappings = {
            "rust": {
                "packageId": declaration["packageId"],
                "name": declaration["exportName"],
            },
            **{
                language: copy.deepcopy(target["languageMappings"][language])
                for language in ("c", "java", "python")
            },
        }
        for language in ("c", "java", "python"):
            if (
                expected_language_mappings[language].get("packageId")
                not in LANGUAGE_PACKAGES[language]
                or not isinstance(expected_language_mappings[language].get("name"), str)
                or not expected_language_mappings[language]["name"].strip()
            ):
                raise ReconciliationError("declaration correction target mapping is invalid")

        already_corrected = (
            declaration.get("capabilityId") == target.get("capabilityId")
            and declaration.get("requirementIds") == target.get("requirementIds")
            and declaration.get("languageMappings") == expected_language_mappings
            and set(evidence) <= set(declaration.get("parityEvidenceIds", []))
        )
        if already_corrected:
            _move_operation_declaration_reference(
                operations,
                declaration_id,
                correction["fromCapabilityId"],
                correction["toCapabilityId"],
            )
            continue
        if _is_partial_predecessor_correction(declaration, source, target):
            declaration["languageMappings"] = expected_language_mappings
            declaration["parityEvidenceIds"] = sorted(
                set(declaration.get("parityEvidenceIds", [])) | set(evidence)
            )
            _move_operation_declaration_reference(
                operations,
                declaration_id,
                correction["fromCapabilityId"],
                correction["toCapabilityId"],
            )
            continue
        for language in ("c", "java", "python"):
            if declaration["languageMappings"][language] != source["languageMappings"][language]:
                raise ReconciliationError("declaration correction source mapping no longer matches")
        if (
            declaration.get("capabilityId") != source.get("capabilityId")
            or declaration.get("requirementIds") != source.get("requirementIds")
            or declaration.get("languageMappings")
            != {
                "rust": {
                    "packageId": declaration["packageId"],
                    "name": declaration["exportName"],
                },
                **{
                    language: copy.deepcopy(source["languageMappings"][language])
                    for language in ("c", "java", "python")
                },
            }
        ):
            raise ReconciliationError("declaration correction source state differs from its reviewed anchor")

        declaration["capabilityId"] = target["capabilityId"]
        declaration["requirementIds"] = copy.deepcopy(target["requirementIds"])
        declaration["languageMappings"] = expected_language_mappings
        declaration["parityEvidenceIds"] = sorted(
            set(declaration.get("parityEvidenceIds", [])) | set(evidence)
        )
        _move_operation_declaration_reference(
            operations,
            declaration_id,
            correction["fromCapabilityId"],
            correction["toCapabilityId"],
        )

    return result


def _move_operation_declaration_reference(
    operations: list[dict[str, Any]], declaration_id: str, source_id: str, target_id: str
) -> None:
    """Move only existing exact operation references, preserving language scope."""
    source_operations = [row for row in operations if row.get("operationId") == source_id]
    target_operations = [row for row in operations if row.get("operationId") == target_id]
    if len(source_operations) != 1 or len(target_operations) != 1:
        raise ReconciliationError("declaration correction operation anchors are ambiguous")
    source_refs = source_operations[0].get("declarations")
    target_refs = target_operations[0].get("declarations")
    if not isinstance(source_refs, dict) or not isinstance(target_refs, dict):
        raise ReconciliationError("declaration correction operation mappings are malformed")
    for language, references in source_refs.items():
        if not isinstance(references, list) or declaration_id not in references:
            continue
        if references.count(declaration_id) != 1:
            raise ReconciliationError("source operation repeats a declaration correction ID")
        references.remove(declaration_id)
        target_language_refs = target_refs.get(language)
        if not isinstance(target_language_refs, list):
            raise ReconciliationError("target operation lacks the corrected language mapping")
        if declaration_id not in target_language_refs:
            target_language_refs.append(declaration_id)
            target_language_refs.sort()
    for operation in operations:
        for references in operation.get("declarations", {}).values():
            if declaration_id in references and operation.get("operationId") not in {source_id, target_id}:
                raise ReconciliationError("declaration correction ID is linked to another operation")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--routes",
        type=Path,
        default=Path("specs/053-api-parity-facade/additional-api-routes.json"),
    )
    parser.add_argument(
        "--write", action="store_true", help="update the reviewed manifest input"
    )
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
        result = resolve_public_declarations(
            result, route_data.get("declarationResolutions", [])
        )
        result = remove_public_declarations(
            result, route_data.get("removePublicDeclarations", [])
        )
        result = add_additional_types(result, route_data.get("types", []))
        result = add_additional_functions(result, route_data.get("functions", []))
        result = add_public_declarations(
            result,
            route_data.get("publicDeclarations", []),
            route_data.get("pythonStaleMappings", []),
            route_data.get("javaStaleMappings", []),
            route_data.get("pythonKindCorrections", []),
            route_data.get("rustKindCorrections", []),
            route_data.get("languageMappingCorrections", []),
        )
        result = apply_language_mapping_corrections(
            result, route_data.get("languageMappingCorrections", [])
        )
        result = apply_declaration_corrections(
            result, route_data.get("declarationCorrections", [])
        )
        result = remove_approved_exclusions(
            result, route_data.get("removeApprovedExclusions", [])
        )
        result = add_approved_exclusions(
            result, route_data.get("approvedExclusions", [])
        )
        result = _synchronize_capability_mappings(result)
    except (OSError, json.JSONDecodeError, ReconciliationError) as error:
        print(f"KMIPKIT-0053 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    added = len(result["functions"]) - len(manifest["functions"])
    action = "write" if arguments.write else "would write"
    print(
        f"Prepared 0053 reconciliation: {added:+} functions; {action} reviewed manifest input."
    )
    if arguments.write:
        content = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
        manifest_path.write_bytes(content.encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
