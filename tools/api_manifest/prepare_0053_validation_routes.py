"""Prepare exact 0053 mappings and exclusions for the synced validation exports."""

from __future__ import annotations

import copy
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from typing import Any


SPEC_ID = "KMIPKIT-0053"
APPROVAL_PATH = "specs/053-api-parity-facade/approval-record.md"
APPROVAL_EVIDENCE = (
    "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation."
)
OWNERS = {"KMIPKIT-0009", "KMIPKIT-0043", "KMIPKIT-0044"}
PINNED_SOURCE_COMMIT = "63b1f3c9e1ee1f96b8b0b26711cfb16e6e06d0fd"
EVIDENCE_PATHS = [
    "specification/api/public-api.json",
    "specs/051-pre-send-validation/spec.md",
    "specs/053-api-parity-facade/spec.md",
    "specs/053-api-parity-facade/gap-matrix.md",
    "specs/053-api-parity-facade/reports/rust-export-report-2026-11-07.json",
    "crates/kmipkit-protocol/src/validation/schema.rs",
    "crates/kmipkit-protocol/src/validation/schemas_generated.rs",
    "crates/kmipkit-protocol/src/validation/typed_requests.rs",
    "crates/kmipkit-client/src/execute.rs",
    "crates/kmipkit-ffi/src/extension_registry/request_validation.rs",
    "bindings/c/tests/request_validation_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ValidationParityTest.java",
    "bindings/python/tests/test_validation_parity.py",
    "crates/kmipkit-protocol/tests/unit/typed_request_validation_tests.rs",
]

ALIAS_CANONICAL_NAMES = {
    "kmipkit_client::ClientRequest::validation_report": (
        "kmipkit::ClientRequest::validation_report"
    ),
    "kmipkit_protocol::ValidateKmipRequest": (
        "kmipkit::protocol::ValidateKmipRequest"
    ),
    "kmipkit_protocol::ValidateKmipRequest::validate": (
        "kmipkit::protocol::ValidateKmipRequest::validate"
    ),
    "kmipkit_protocol::ValidationIssue": "kmipkit::protocol::ValidationIssue",
    "kmipkit_protocol::ValidationIssue::code": (
        "kmipkit::protocol::ValidationIssue::code"
    ),
    "kmipkit_protocol::ValidationIssue::path": (
        "kmipkit::protocol::ValidationIssue::path"
    ),
    "kmipkit_protocol::ValidationIssue::severity": (
        "kmipkit::protocol::ValidationIssue::severity"
    ),
    "kmipkit_protocol::ValidationIssue::source": (
        "kmipkit::protocol::ValidationIssue::source"
    ),
    "kmipkit_protocol::ValidationReport": "kmipkit::protocol::ValidationReport",
    "kmipkit_protocol::ValidationReport::is_valid": (
        "kmipkit::protocol::ValidationReport::is_valid"
    ),
    "kmipkit_protocol::ValidationReport::issues": (
        "kmipkit::protocol::ValidationReport::issues"
    ),
    "kmipkit_protocol::validate_structure": "kmipkit::protocol::validate_structure",
}

APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": APPROVAL_PATH,
    "status": "Approved",
    "evidence": APPROVAL_EVIDENCE,
}


class PreparationError(ValueError):
    """Raised when the exact audit and candidate manifest do not agree."""


def _observed_declaration_id(package_id: str, export_name: str) -> str:
    digest = hashlib.sha256(f"{package_id}\0{export_name}".encode("utf-8")).hexdigest()[:20]
    return f"rust-observed-{digest}"


def _exclusion_id(package_id: str, export_name: str) -> str:
    identity = json.dumps([package_id, export_name], separators=(",", ":"))
    digest = hashlib.sha256(identity.encode("utf-8")).hexdigest()[:24]
    return f"{SPEC_ID}:exclude:{digest}"


def _approval() -> dict[str, str]:
    return copy.deepcopy(APPROVAL_SOURCE)


def _audit_identity(row: dict[str, Any]) -> str | None:
    identity = row.get("identity")
    if isinstance(identity, str) and identity:
        return identity
    condition = row.get("condition")
    implementation_id = row.get("implementationId")
    if isinstance(condition, dict) and isinstance(implementation_id, str):
        return json.dumps(
            {"condition": condition, "implementationId": implementation_id},
            ensure_ascii=False,
            sort_keys=True,
        )
    raise PreparationError(f"Rust audit row has no exact identity: {row.get('exportName')}")


def _reason(export_name: str, identity: str | None) -> str:
    parsed_identity = json.loads(identity) if identity is not None else None
    trait_path = None
    if isinstance(parsed_identity, list) and parsed_identity[:1] == ["rust-trait-implementation"]:
        trait_path = parsed_identity[3]
    elif isinstance(parsed_identity, dict):
        predicates = parsed_identity.get("condition", {}).get("predicates", [])
        trait_path = next(
            (predicate.get("traitPath") for predicate in predicates if isinstance(predicate, dict)),
            None,
        )
    if trait_path is not None:
        trait_name = "::".join(trait_path) if isinstance(trait_path, list) else str(trait_path)
        if trait_name.endswith("ValidateKmipRequest"):
            return (
                "This per-request ValidateKmipRequest implementation member is an exact Rust trait "
                "projection. The same report behavior is exposed through the verified four-language "
                "request_validation routes; its Rustdoc identity is retained for this exact exclusion."
            )
    if any(name in export_name for name in ("RequestPayloadSchema", "NestedStructureSchema", "ValidationSchema")):
        return (
            "This exact Rust schema enum, variant, or field represents schema selection as a typed "
            "Rust value. C, Java, and Python use explicit RequestPayload/<name>, Nested/<name>, or "
            "Unsupported identifiers through the same schema-aware validator; this shape adds no "
            "separate validation capability."
        )
    if trait_path is not None:
        return (
            "This exact Rust trait implementation member is implicit type-system behavior, not an "
            "independent KMIPKit route. Its owning public type and the verified request_validation "
            "report routes cover the observable capability; the Rustdoc identity is retained."
        )
    if any(name in export_name for name in ("ValidationPath", "ValidationSeverity", "ValidationSourceRef", "ValidationCode")):
        return (
            "This exact Rust diagnostic wrapper or helper is a representation detail of the shared "
            "validation report. The verified four-language ValidationIssue route exposes the same "
            "safe code, path, severity, and source metadata without requiring Rust-specific wrapper "
            "constructors or enum layouts."
        )
    return (
        "This exact observed Rust validation support route adds no independent cross-language "
        "capability beyond the verified request_validation report and explicit-schema APIs."
    )


def _audit_rows(report: dict[str, Any]) -> dict[tuple[str, str], dict[str, Any]]:
    result: dict[tuple[str, str], dict[str, Any]] = {}
    for section in ("declarations", "conditionalDeclarations"):
        rows = report.get(section)
        if not isinstance(rows, list):
            raise PreparationError(f"Rust audit lacks {section}")
        for row in rows:
            if not isinstance(row, dict):
                raise PreparationError(f"Rust audit has a malformed {section} row")
            key = (row.get("packageId"), row.get("exportName"))
            if key in result:
                raise PreparationError(f"Rust audit repeats exact export {key}")
            result[key] = row
    return result


def _validate_candidate_provenance(
    candidate_inventory: dict[str, Any], repo_root: Path
) -> list[dict[str, Any]]:
    """Require the candidate list to equal the blocked rows from its pinned Git tree."""
    source_commit = candidate_inventory.get("sourceCommit")
    if source_commit != PINNED_SOURCE_COMMIT:
        raise PreparationError("validation candidates are not pinned to the reviewed source commit")
    try:
        completed = subprocess.run(
            [
                "git",
                "show",
                f"{source_commit}:specification/api/public-api.json",
            ],
            cwd=repo_root,
            check=True,
            capture_output=True,
        )
        source_manifest = json.loads(completed.stdout)
    except (OSError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        raise PreparationError("pinned validation source manifest cannot be read from Git") from error
    expected = [
        {
            key: row[key]
            for key in (
                "declarationId",
                "packageId",
                "kind",
                "exportName",
                "candidateSpecId",
            )
        }
        for row in source_manifest.get("publicDeclarations", [])
        if row.get("declarationState") == "blocked"
        and row.get("candidateSpecId") in OWNERS
    ]
    candidates = candidate_inventory.get("candidates")
    if candidates != expected:
        raise PreparationError("validation candidate snapshot differs from its pinned Git manifest")
    identifiers = [row["declarationId"] for row in candidates]
    exports = [(row["packageId"], row["exportName"]) for row in candidates]
    if len(identifiers) != len(set(identifiers)) or len(exports) != len(set(exports)):
        raise PreparationError("validation candidate snapshot repeats an exact identity")
    return candidates


def _validate_candidate_classification(
    package_id: str,
    export_name: str,
    audit_row: dict[str, Any],
) -> None:
    """Reject any candidate that is not a direct alias, trait projection, or validator model."""
    if export_name in ALIAS_CANONICAL_NAMES:
        return
    identity = audit_row.get("identity")
    if isinstance(identity, str):
        parsed = json.loads(identity)
        if isinstance(parsed, list) and parsed[:1] == ["rust-trait-implementation"]:
            return
    if (
        isinstance(audit_row.get("condition"), dict)
        and isinstance(audit_row.get("implementationId"), str)
        and export_name.startswith("<")
    ):
        return
    if any(
        marker in export_name
        for marker in (
            "RequestPayloadSchema",
            "NestedStructureSchema",
            "ValidationSchema",
            "ValidateKmipRequest",
            "ValidationCode",
            "ValidationIssue",
            "ValidationPath",
            "ValidationReport",
            "ValidationSeverity",
            "ValidationSourceRef",
        )
    ):
        return
    raise PreparationError(
        f"validation candidate is outside the reviewed export classes: {package_id}::{export_name}"
    )


def prepare_validation_routes(
    manifest: dict[str, Any],
    report: dict[str, Any],
    route_data: dict[str, Any],
    candidate_inventory: dict[str, Any] | None = None,
    repo_root: Path | None = None,
) -> dict[str, Any]:
    """Return deterministic exact routes for every synced blocked validation export."""
    if candidate_inventory is None:
        blocked = [
            row
            for row in manifest.get("publicDeclarations", [])
            if row.get("declarationState") == "blocked" and row.get("candidateSpecId") in OWNERS
        ]
    else:
        if (
            candidate_inventory.get("formatVersion") != 1
            or candidate_inventory.get("specId") != SPEC_ID
            or candidate_inventory.get("candidateCount") != 934
        ):
            raise PreparationError("validation candidate snapshot has invalid provenance")
        blocked = _validate_candidate_provenance(candidate_inventory, repo_root or Path.cwd())
        current_blocked = [
            row
            for row in manifest.get("publicDeclarations", [])
            if row.get("declarationState") == "blocked" and row.get("candidateSpecId") in OWNERS
        ]
        if current_blocked and {
            row["declarationId"] for row in current_blocked
        } != {row["declarationId"] for row in blocked}:
            raise PreparationError("current blocked inventory diverges from the reviewed candidate snapshot")
    if len(blocked) != 934:
        raise PreparationError(f"expected 934 synced validation candidates, found {len(blocked)}")

    audit = _audit_rows(report)
    manifest_routes = route_data.get("publicDeclarations")
    if not isinstance(manifest_routes, list):
        raise PreparationError("route data lacks publicDeclarations")

    available: dict[str, dict[str, Any]] = {}
    for row in [*manifest.get("publicDeclarations", []), *manifest_routes]:
        rust = row.get("languageMappings", {}).get("rust")
        if (
            row.get("capabilityId") == "request_validation"
            and isinstance(rust, dict)
            and isinstance(rust.get("name"), str)
            and row.get("declarationState") == "verified"
        ):
            available[rust["name"]] = row

    aliases = {
        (row["packageId"], row["exportName"]): row
        for row in blocked
        if row["exportName"] in ALIAS_CANONICAL_NAMES
    }
    if set(aliases) != {
        ("rust-kmipkit-client", "kmipkit_client::ClientRequest::validation_report"),
        *[("rust-kmipkit-protocol", name) for name in ALIAS_CANONICAL_NAMES if name.startswith("kmipkit_protocol::")],
    }:
        raise PreparationError("the exact 12 direct validation aliases changed")

    additions = list(route_data.get("publicDeclarations", []))
    existing_addition_keys = {
        (row.get("packageId"), row.get("exportName")) for row in additions
    }
    removals = list(route_data.get("removePublicDeclarations", []))
    removal_by_id = {row.get("declarationId"): row for row in removals}
    exclusions = list(route_data.get("approvedExclusions", []))
    exclusion_by_key = {
        (row.get("packageId"), row.get("exportName")): row for row in exclusions
    }
    report_by_key = _audit_rows(report)
    validation_report = next(
        (
            row
            for row in manifest.get("publicDeclarations", [])
            if row.get("packageId") == "rust-kmipkit"
            and row.get("exportName") == "kmipkit::protocol::ValidationReport"
            and row.get("declarationState") == "verified"
            and row.get("capabilityId") == "request_validation"
        ),
        None,
    )
    if validation_report is None:
        raise PreparationError("verified ValidationReport capability anchor is missing")

    mapped_keys: set[tuple[str, str]] = set()
    for row in blocked:
        package_id, export_name = row["packageId"], row["exportName"]
        key = (package_id, export_name)
        audit_row = report_by_key.get(key)
        if audit_row is None or row.get("kind") != audit_row.get("kind"):
            raise PreparationError(f"blocked declaration does not match current Rust audit: {key}")
        identity = _audit_identity(audit_row)
        _validate_candidate_classification(package_id, export_name, audit_row)

        removal = {
            "declarationId": row["declarationId"],
            "packageId": package_id,
            "exportName": export_name,
            "reason": "Superseded by the exact KMIPKIT-0053 validation mapping or exclusion recorded for this Rust export.",
            "candidateSpecId": SPEC_ID,
            "approvalSource": _approval(),
            "evidencePaths": copy.deepcopy(EVIDENCE_PATHS),
        }
        previous_removal = removal_by_id.get(row["declarationId"])
        if previous_removal is None:
            removals.append(removal)
            removal_by_id[row["declarationId"]] = removal
        elif previous_removal != removal:
            raise PreparationError(f"existing exact removal differs from audit evidence: {key}")

        canonical_name = ALIAS_CANONICAL_NAMES.get(export_name)
        if canonical_name is not None:
            source = available.get(canonical_name)
            if source is None:
                raise PreparationError(f"verified canonical validation route is missing: {canonical_name}")
            declaration = copy.deepcopy(source)
            declaration.update(
                {
                    "declarationId": row["declarationId"],
                    "packageId": package_id,
                    "kind": row["kind"],
                    "exportName": export_name,
                    "candidateSpecId": SPEC_ID,
                    "approvalSource": _approval(),
                }
            )
            declaration["languageMappings"]["rust"] = {
                "packageId": package_id,
                "name": export_name,
            }
            declaration["requirementIds"] = sorted(
                set(declaration.get("requirementIds", []))
                | {"KMIPKIT-0053-FR-006", "KMIPKIT-0053-FR-011", "KMIPKIT-0053-FR-012"}
            )
            declaration["parityEvidenceIds"] = sorted(
                set(declaration.get("parityEvidenceIds", [])) | set(EVIDENCE_PATHS)
            )
            existing_addition = next(
                (
                    candidate
                    for candidate in additions
                    if (candidate.get("packageId"), candidate.get("exportName")) == key
                ),
                None,
            )
            if existing_addition is None:
                additions.append(declaration)
                existing_addition_keys.add(key)
            elif existing_addition != declaration:
                raise PreparationError(f"existing exact alias mapping differs: {key}")
            mapped_keys.add(key)
            continue

        exclusion_key = key
        identity_parts = json.loads(identity) if identity is not None else None
        is_conditional = isinstance(audit_row.get("condition"), dict)
        is_trait = (
            isinstance(identity_parts, list)
            and identity_parts[:1] == ["rust-trait-implementation"]
        ) or (is_conditional and export_name.startswith("<"))
        artifact_kind = (
            "rust-conditional-trait-implementation-member"
            if is_trait and is_conditional
            else "rust-trait-implementation-member"
            if is_trait
            else "exact-route"
        )
        exclusion: dict[str, Any] = {
            "declarationId": _exclusion_id(package_id, export_name),
            "packageId": package_id,
            "exportName": export_name,
            "reason": _reason(export_name, identity),
            "approvalSource": _approval(),
            "evidence": "; ".join(EVIDENCE_PATHS),
            "evidencePaths": copy.deepcopy(EVIDENCE_PATHS),
            "candidateSpecId": SPEC_ID,
            "capabilityId": "request_validation",
            "counterpartDeclarationIds": [validation_report["declarationId"]],
            "rustArtifactKind": artifact_kind,
        }
        if identity is not None:
            exclusion["identity"] = identity
        if is_trait:
            exclusion["ownerExportName"] = export_name[1:].split(" as ", 1)[0]
            exclusion["ownerDeclarationId"] = _observed_declaration_id(
                package_id, exclusion["ownerExportName"]
            )
        if is_conditional and audit_row.get("condition") is not None:
            exclusion["condition"] = json.dumps(
                audit_row["condition"],
                ensure_ascii=False,
                sort_keys=True,
                separators=(",", ":"),
            )
        previous_exclusion = exclusion_by_key.get(exclusion_key)
        if previous_exclusion is None:
            exclusions.append(exclusion)
            exclusion_by_key[exclusion_key] = exclusion
        elif previous_exclusion != exclusion:
            exact_fields = (
                "declarationId",
                "packageId",
                "exportName",
                "candidateSpecId",
                "capabilityId",
                "counterpartDeclarationIds",
                "rustArtifactKind",
                "approvalSource",
                "evidence",
                "evidencePaths",
                "ownerExportName",
                "ownerDeclarationId",
            )
            if any(previous_exclusion.get(field) != exclusion.get(field) for field in exact_fields):
                raise PreparationError(f"existing exact exclusion differs from audit evidence: {key}")
            previous_identity = previous_exclusion.get("identity")
            if previous_identity is not None and previous_identity != exclusion.get("identity"):
                raise PreparationError(f"existing Rustdoc identity differs from audit evidence: {key}")
            previous_condition = previous_exclusion.get("condition")
            if is_conditional and previous_condition is not None:
                normalized_condition = (
                    json.loads(previous_condition)
                    if isinstance(previous_condition, str)
                    else previous_condition
                )
                if normalized_condition != audit_row["condition"]:
                    raise PreparationError(f"existing conditional bounds differ from audit evidence: {key}")
            if not isinstance(previous_exclusion.get("reason"), str) or not previous_exclusion["reason"].strip():
                raise PreparationError(f"existing exact exclusion lacks its rationale: {key}")
            if previous_exclusion != exclusion:
                refreshed = copy.deepcopy(previous_exclusion)
                refreshed["identity"] = exclusion["identity"]
                if is_conditional:
                    refreshed["condition"] = exclusion["condition"]
                exclusions[exclusions.index(previous_exclusion)] = refreshed
                exclusion_by_key[exclusion_key] = refreshed

    prepared = copy.deepcopy(route_data)
    prepared["publicDeclarations"] = additions
    prepared["removePublicDeclarations"] = removals
    prepared["approvedExclusions"] = exclusions
    if len(mapped_keys) != 12:
        raise PreparationError(f"expected 12 mapped validation aliases, found {len(mapped_keys)}")
    new_exclusions = [
        row for row in exclusions
        if row.get("packageId") in {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client"}
        and row.get("capabilityId") == "request_validation"
        and row.get("candidateSpecId") == SPEC_ID
        and (row.get("packageId"), row.get("exportName"))
        in {(item["packageId"], item["exportName"]) for item in blocked}
    ]
    if len(new_exclusions) != 922:
        raise PreparationError(f"expected 922 exact validation exclusions, found {len(new_exclusions)}")
    return prepared


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--write", action="store_true", help="write the reviewed route input")
    arguments = parser.parse_args()
    root = arguments.repo_root
    manifest_path = root / "specification" / "api" / "public-api.json"
    report_path = (
        root
        / "specs"
        / "053-api-parity-facade"
        / "reports"
        / "rust-export-report-2026-11-07.json"
    )
    candidate_path = (
        root
        / "specs"
        / "053-api-parity-facade"
        / "candidate-resolutions"
        / "validation-export-candidates.json"
    )
    routes_path = root / "specs" / "053-api-parity-facade" / "additional-api-routes.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        report = json.loads(report_path.read_text(encoding="utf-8"))
        candidate_inventory = json.loads(candidate_path.read_text(encoding="utf-8"))
        routes = json.loads(routes_path.read_text(encoding="utf-8"))
        prepared = prepare_validation_routes(
            manifest, report, routes, candidate_inventory, root
        )
    except (OSError, json.JSONDecodeError, PreparationError) as error:
        print(f"KMIPKIT-0053 validation route preparation failed: {error}", file=sys.stderr)
        return 2
    content = json.dumps(prepared, ensure_ascii=False, indent=2) + "\n"
    action = "write" if arguments.write else "would write"
    print(
        "Prepared exact validation reconciliation: +12 mapped aliases, "
        f"+922 exclusions, +934 exact replacements; {action} reviewed route input."
    )
    if arguments.write:
        routes_path.write_bytes(content.encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
