"""Compile exact public routes from the KMIPKIT-0053 export audits."""

from __future__ import annotations

import hashlib
import json
import argparse
import copy
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
APPROVAL_PATH = "specs/053-api-parity-facade/approval-record.md"
LANGUAGE_PACKAGES = {
    "rust": {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client"},
    "c": {"c-kmipkit"},
    "java": {"java-kmipkit"},
    "python": {"python-kmipkit"},
}
REQUIRED_LANGUAGES = {"rust", "c", "java", "python"}
AUDIT_PATHS = (
    Path("specs/053-api-parity-facade/candidate-resolutions/java-export-audit.json"),
    Path("specs/053-api-parity-facade/candidate-resolutions/python-export-audit.json"),
    Path("specs/053-api-parity-facade/candidate-resolutions/rust-export-audit.json"),
)
ROUTE_PATH = Path("specs/053-api-parity-facade/additional-api-routes.json")
MANIFEST_PATH = Path("specification/api/public-api.json")


class ResolutionError(ValueError):
    """An export-audit resolution is not complete or exact enough to import."""


def _exact_name(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip() or any(c in value for c in "*?"):
        raise ResolutionError(f"{label} must be a non-empty exact export name")
    return value


def _route(value: Any, language: str) -> dict[str, str]:
    if isinstance(value, str):
        package_id = {
            "rust": "rust-kmipkit-protocol",
            "c": "c-kmipkit",
            "java": "java-kmipkit",
            "python": "python-kmipkit",
        }[language]
        name = value
    elif isinstance(value, dict):
        package_id = value.get("packageId")
        name = value.get("name")
    else:
        raise ResolutionError(f"{language} mapping must identify one exact route")
    if package_id not in LANGUAGE_PACKAGES[language]:
        raise ResolutionError(f"{language} route uses an unsupported package")
    return {"packageId": package_id, "name": _exact_name(name, f"{language} route")}


def _approval(row: dict[str, Any]) -> dict[str, Any]:
    approval = row.get("approval")
    if not isinstance(approval, dict) or approval.get("status") != "Approved":
        raise ResolutionError("every imported route needs approved feature evidence")
    approval_path = approval.get("path")
    if (
        not isinstance(approval_path, str)
        or not approval_path.strip()
        or Path(approval_path).is_absolute()
        or "\\" in approval_path
        or ".." in Path(approval_path).parts
        or not (ROOT / approval_path).is_file()
    ):
        raise ResolutionError("route approval must reference an exact approved repository file")
    return {
        "kind": approval.get("kind", "feature-specification"),
        "path": approval_path,
        "status": "Approved",
        "evidence": approval.get("evidence") or "Maintainer-approved KMIPKIT-0053 scope.",
    }


def _evidence(row: dict[str, Any]) -> list[str]:
    paths = row.get("evidencePaths")
    if not isinstance(paths, list) or not paths:
        raise ResolutionError("every imported route needs evidence paths")
    normalized = []
    for path in paths:
        if (
            not isinstance(path, str)
            or not path.strip()
            or Path(path).is_absolute()
            or "\\" in path
            or ".." in Path(path).parts
        ):
            raise ResolutionError("route evidence paths must be exact repository-relative paths")
        if not (ROOT / path).is_file():
            raise ResolutionError(f"route evidence file does not exist: {path}")
        normalized.append(path)
    return sorted(set(normalized))


def _declaration(document: dict[str, Any], row: dict[str, Any]) -> dict[str, Any]:
    package_id = _exact_name(row.get("packageId"), "source package")
    export_name = _exact_name(row.get("name", row.get("exportName")), "source export")
    capability_id = _exact_name(row.get("capabilityId"), "capability ID")
    requirements = row.get("requirementIds")
    if not isinstance(requirements, list) or not requirements:
        raise ResolutionError("every imported route needs stable requirement IDs")
    if any(not isinstance(item, str) or not item.strip() for item in requirements):
        raise ResolutionError("requirement IDs must be non-empty strings")

    source_package = document.get("packageId", package_id)
    source_language = next(
        (lang for lang, packages in LANGUAGE_PACKAGES.items() if source_package in packages),
        None,
    )
    if source_language is None:
        raise ResolutionError(f"unsupported source package: {source_package}")

    language_mappings = row.get("manifestMapping", row.get("languageMappings"))
    if isinstance(language_mappings, dict):
        mappings = {lang: _route(value, lang) for lang, value in language_mappings.items()}
    else:
        equivalents = row.get("equivalents")
        if not isinstance(equivalents, dict):
            raise ResolutionError("route needs exact language mappings")
        mappings = {source_language: _route({"packageId": package_id, "name": export_name}, source_language)}
        for language, value in equivalents.items():
            if language in mappings:
                raise ResolutionError("route repeats its source-language mapping")
            mappings[language] = _route(value, language)
    if set(mappings) != REQUIRED_LANGUAGES:
        raise ResolutionError("a verified route must cover Rust, C, Java, and Python")
    if mappings[source_language] != {"packageId": package_id, "name": export_name}:
        raise ResolutionError("source route differs from the audited export identity")

    kind = row.get("kind", row.get("observedKind", row.get("declarationKind")))
    valid_kinds = {
        "module", "type", "variant", "field", "trait", "function", "associated-item",
        "constant", "macro", "c-symbol", "java-member", "python-export",
    }
    if kind not in valid_kinds:
        raise ResolutionError("route needs its observed declaration kind")
    canonical = [
        capability_id,
        package_id,
        export_name,
        sorted((lang, value["packageId"], value["name"]) for lang, value in mappings.items()),
    ]
    identifier = "KMIPKIT-0053:export:" + hashlib.sha256(
        json.dumps(canonical, separators=(",", ":")).encode("utf-8")
    ).hexdigest()[:24]
    return {
        "declarationId": identifier,
        "packageId": package_id,
        "kind": kind,
        "exportName": export_name,
        "capabilityId": capability_id,
        "requirementIds": sorted(set(requirements)),
        "declarationState": "verified",
        "parityScope": "four-language",
        "languageMappings": mappings,
        "parityEvidenceIds": _evidence(row),
        "approvalSource": _approval(row),
    }


def _exact_exclusion(document: dict[str, Any], row: dict[str, Any]) -> dict[str, Any]:
    details = row.get("exclusion")
    export_name = row.get("name") or row.get("exportName")
    if not export_name and isinstance(details, dict):
        export_name = details.get("exportName")
    package_id = _exact_name(
        row.get("packageId", document.get("packageId")), "excluded package"
    )
    export_name = _exact_name(export_name, "excluded export")
    if package_id != document.get("packageId", package_id):
        raise ResolutionError("exclusion package differs from its audited package")
    if row.get("resolution", "excluded") != "excluded":
        raise ResolutionError("only explicitly excluded routes can be imported")
    reason = row.get("reason")
    if not reason and isinstance(details, dict):
        reason = details.get("reason")
    if not isinstance(reason, str) or not reason.strip():
        raise ResolutionError("exact exclusions need a specific reason")
    paths = _evidence(row)
    return {
        "packageId": package_id,
        "exportName": export_name,
        "reason": reason,
        "approvalSource": _approval(row),
        "evidence": reason + " Evidence files: " + ", ".join(paths) + ".",
        "evidencePaths": paths,
        "candidateSpecId": "KMIPKIT-0053",
    }


def _normalize_python_audit(document: dict[str, Any]) -> dict[str, Any]:
    """Apply exact auditor corrections to observed Python export identities."""
    result = copy.deepcopy(document)
    rows = result.get("mappings")
    if not isinstance(rows, list):
        raise ResolutionError("Python export audit lacks mappings")

    for correction in result.get("staleMappings", []):
        package_id = _exact_name(correction.get("packageId"), "stale Python package")
        old_name = _exact_name(correction.get("name"), "stale Python export")
        corrected_name = _exact_name(
            correction.get("correctedName"), "corrected Python export"
        )
        if package_id != "python-kmipkit":
            raise ResolutionError(
                f"stale Python route uses an unsupported package: {old_name}"
            )
        for row in rows:
            if row.get("packageId", result.get("packageId")) != package_id:
                continue
            for key in ("manifestMapping", "languageMappings"):
                mappings = row.get(key)
                python_route = mappings.get("python") if isinstance(mappings, dict) else None
                if (
                    isinstance(python_route, dict)
                    and python_route.get("packageId") == package_id
                    and python_route.get("name") == old_name
                ):
                    python_route["name"] = corrected_name
                elif python_route == old_name:
                    mappings["python"] = corrected_name
    observed_python_names = {
        row.get("name", row.get("exportName"))
        for row in rows
        if row.get("packageId", result.get("packageId")) == "python-kmipkit"
    }
    for correction in result.get("staleMappings", []):
        corrected_name = correction.get("correctedName")
        if corrected_name not in observed_python_names:
            raise ResolutionError(
                f"corrected Python route is absent from the audited exports: {corrected_name}"
            )

    for correction in result.get("kindCorrections", []):
        package_id = _exact_name(correction.get("packageId"), "kind-corrected package")
        export_name = _exact_name(correction.get("name"), "kind-corrected export")
        from_kind = _exact_name(correction.get("fromKind"), "observed declaration kind")
        to_kind = _exact_name(correction.get("toKind"), "corrected declaration kind")
        if package_id != "python-kmipkit":
            raise ResolutionError("Python kind correction uses a non-Python package")
        _evidence(correction)
        _approval(correction)
        matches = [
            row for row in rows
            if row.get("packageId", result.get("packageId")) == package_id
            and row.get("name", row.get("exportName")) == export_name
        ]
        for row in matches:
            observed_kind = row.get("kind", row.get("observedKind"))
            if observed_kind == from_kind:
                row["kind"] = to_kind
            elif observed_kind != to_kind:
                raise ResolutionError(f"Python route has an unexpected declaration kind: {export_name}")
    return result


def normalize_python_stale_mappings(
    route_data: dict[str, Any], stale_mappings: list[dict[str, Any]] | None = None
) -> dict[str, Any]:
    """Replace exact stale Python paths in every reviewed declaration mapping."""
    result = copy.deepcopy(route_data)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ResolutionError("Python stale-route normalization needs publicDeclarations")
    for stale in stale_mappings or []:
        old_name = _exact_name(stale.get("name"), "stale Python export")
        corrected_name = _exact_name(stale.get("correctedName"), "corrected Python export")
        package_id = _exact_name(stale.get("packageId"), "stale Python package")
        matches = []
        for row in declarations:
            python_mapping = row.get("languageMappings", {}).get("python", {})
            if (
                row.get("packageId") == package_id and row.get("exportName") == old_name
            ) or (
                python_mapping.get("packageId") == package_id
                and python_mapping.get("name") == old_name
            ):
                matches.append(row)
        if not matches:
            already_normalized = any(
                (
                    row.get("packageId") == package_id
                    and row.get("exportName") == corrected_name
                )
                or (
                    row.get("languageMappings", {}).get("python", {}).get("packageId")
                    == package_id
                    and row.get("languageMappings", {}).get("python", {}).get("name")
                    == corrected_name
                )
                for row in declarations
            )
            if already_normalized:
                continue
            raise ResolutionError(f"stale Python export is absent from reviewed routes: {old_name}")
        for row in matches:
            if row.get("packageId") == package_id and row.get("exportName") == old_name:
                row["exportName"] = corrected_name
            python_mapping = row.get("languageMappings", {}).get("python")
            if isinstance(python_mapping, dict) and python_mapping.get("name") == old_name:
                python_mapping["name"] = corrected_name
    export_keys = [
        (row.get("packageId"), row.get("exportName"))
        for row in declarations
        if isinstance(row, dict)
    ]
    if len(export_keys) != len(set(export_keys)):
        raise ResolutionError("Python stale-route normalization creates duplicate public exports")
    return result


def apply_kind_corrections(
    route_data: dict[str, Any],
    python_corrections: list[dict[str, Any]] | None = None,
    rust_corrections: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Update exact reviewed declaration kinds to match audited public exports."""
    result = copy.deepcopy(route_data)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ResolutionError("kind correction needs publicDeclarations")
    corrections = [
        (
            row.get("packageId"), row.get("name"), row.get("fromKind"), row.get("toKind")
        )
        for row in (python_corrections or [])
    ]
    corrections.extend(
        (
            row.get("packageId"), row.get("exportName"),
            row.get("manifestKind"), row.get("observedKind"),
        )
        for row in (rust_corrections or [])
    )
    for package_id, export_name, current_kind, audited_kind in corrections:
        package_id = _exact_name(package_id, "kind-corrected package")
        export_name = _exact_name(export_name, "kind-corrected export")
        current_kind = _exact_name(current_kind, "current declaration kind")
        audited_kind = _exact_name(audited_kind, "audited declaration kind")
        matches = [
            row for row in declarations
            if row.get("packageId") == package_id and row.get("exportName") == export_name
        ]
        if len(matches) != 1:
            raise ResolutionError(
                f"kind correction must match exactly one reviewed declaration: {export_name}"
            )
        if matches[0].get("kind") == current_kind:
            matches[0]["kind"] = audited_kind
        elif matches[0].get("kind") != audited_kind:
            raise ResolutionError(f"reviewed declaration has an unexpected kind: {export_name}")
    return result


def remove_reviewed_declarations(
    route_data: dict[str, Any], removals: list[dict[str, Any]] | None = None
) -> dict[str, Any]:
    """Remove only exact, approved duplicate declarations from reviewed route input."""
    result = copy.deepcopy(route_data)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ResolutionError("approved declaration removal needs publicDeclarations")
    for removal in removals or []:
        expected_keys = {
            "declarationId", "packageId", "exportName", "reason", "candidateSpecId",
            "approvalSource", "evidencePaths",
        }
        if not isinstance(removal, dict) or set(removal) != expected_keys:
            raise ResolutionError("approved declaration removal has missing or extra fields")
        if removal.get("candidateSpecId") != "KMIPKIT-0053":
            raise ResolutionError("declaration removal must be owned by KMIPKIT-0053")
        declaration_id = _exact_name(removal.get("declarationId"), "removed declaration ID")
        package_id = _exact_name(removal.get("packageId"), "removed package")
        export_name = _exact_name(removal.get("exportName"), "removed export")
        _exact_name(removal.get("reason"), "declaration removal reason")
        _evidence(removal)
        _approval({"approval": removal["approvalSource"]})
        identified = [row for row in declarations if row.get("declarationId") == declaration_id]
        if identified and (
            len(identified) != 1
            or identified[0].get("packageId") != package_id
            or identified[0].get("exportName") != export_name
        ):
            raise ResolutionError("approved removal does not match the exact reviewed declaration")
        declarations = [row for row in declarations if row.get("declarationId") != declaration_id]
    result["publicDeclarations"] = declarations
    return result


def _feature_approval() -> dict[str, Any]:
    return {
        "kind": "feature-specification",
        "path": APPROVAL_PATH,
        "status": "Approved",
        "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
    }


def _exclusion_id(package_id: str, export_name: str) -> str:
    identity = json.dumps([package_id, export_name], separators=(",", ":"))
    digest = hashlib.sha256(identity.encode("utf-8")).hexdigest()[:24]
    return f"KMIPKIT-0053:exclude:{digest}"


def _rust_type_exclusion(
    row: dict[str, Any],
    declarations_by_export: dict[tuple[str, str], dict[str, Any]],
    rust_audit_routes: dict[tuple[str, str], dict[str, Any]],
) -> dict[str, Any]:
    package_id = _exact_name(row.get("packageId"), "Rust excluded package")
    export_name = _exact_name(row.get("exportName"), "Rust excluded export")
    artifact_kind = _exact_name(row.get("artifactKind"), "Rust artifact kind")
    if artifact_kind not in {"rust-enum-variant", "rust-enum-field"}:
        raise ResolutionError("Rust shape exclusion must identify an enum variant or field")
    parent_name = export_name.rsplit("::", 1)[0]
    if artifact_kind == "rust-enum-field":
        parent_name = parent_name.rsplit("::", 1)[0]
    counterpart = declarations_by_export.get((package_id, parent_name))
    if counterpart is None:
        counterpart = rust_audit_routes.get((package_id, parent_name))
    if counterpart is None:
        raise ResolutionError(f"Rust enum exclusion has no exact mapped parent: {parent_name}")
    evidence_paths = _evidence(row)
    return {
        "declarationId": _exclusion_id(package_id, export_name),
        "capabilityId": counterpart["capabilityId"],
        "packageId": package_id,
        "exportName": export_name,
        "declarationKind": "variant" if artifact_kind == "rust-enum-variant" else "field",
        "artifactKind": artifact_kind,
        "reason": _exact_name(row.get("reason"), "Rust shape exclusion reason"),
        "counterpartDeclarationIds": [counterpart["declarationId"]],
        "approvalSource": _feature_approval(),
        "evidence": "; ".join(evidence_paths),
        "evidencePaths": evidence_paths,
        "candidateSpecId": "KMIPKIT-0053",
    }


def _validate_rust_functional_routes(
    rows: list[dict[str, Any]], existing_declarations: list[dict[str, Any]]
) -> dict[tuple[str, str], dict[str, Any]]:
    """Validate grouped Rust exports against their exact reviewed manifest routes."""
    by_id = {
        row.get("declarationId"): row
        for row in existing_declarations
        if isinstance(row, dict)
    }
    source_routes: dict[tuple[str, str], dict[str, Any]] = {}
    for row in rows:
        package_id = _exact_name(row.get("packageId"), "Rust audited package")
        export_name = _exact_name(
            row.get("exportName", row.get("name")), "Rust audited export"
        )
        _exact_name(row.get("capabilityId"), "Rust capability ID")
        requirements = row.get("requirementIds")
        if not isinstance(requirements, list) or not requirements:
            raise ResolutionError("Rust functional route needs stable requirement IDs")
        _evidence(row)
        _approval(row)
        manifest_mapping = row.get("manifestMapping")
        if not isinstance(manifest_mapping, dict) or set(manifest_mapping) != REQUIRED_LANGUAGES:
            raise ResolutionError("Rust functional route needs one exact manifest route per language")
        normalized = {
            language: _route(value, language)
            for language, value in manifest_mapping.items()
        }
        reference = row.get("manifestMappingRef")
        if (
            not isinstance(reference, dict)
            or set(reference) != {"path", "declarationId"}
            or reference.get("path") != "specification/api/public-api.json"
        ):
            raise ResolutionError("Rust functional route needs an exact manifest declaration reference")
        declaration = by_id.get(_exact_name(reference.get("declarationId"), "manifest declaration ID"))
        if (
            declaration is None
            or declaration.get("declarationState") != "verified"
            or declaration.get("capabilityId") != row.get("capabilityId")
            or not set(requirements) <= set(declaration.get("requirementIds", []))
            or declaration.get("languageMappings") != normalized
        ):
            raise ResolutionError(f"Rust route does not match its reviewed manifest reference: {export_name}")
        composition = row.get("manifestCompositionDeclarationIds")
        if not isinstance(composition, list) or not composition:
            raise ResolutionError("Rust functional route needs its complete manifest composition references")
        if any(not isinstance(item, str) or item not in by_id for item in composition):
            raise ResolutionError(f"Rust route composition contains an unknown manifest reference: {export_name}")
        source_routes[(package_id, export_name)] = declaration
    return source_routes


def _rust_route_exclusion(row: dict[str, Any], *, prior: bool = False) -> dict[str, Any]:
    package_id = _exact_name(row.get("packageId"), "Rust excluded package")
    export_name = _exact_name(row.get("exportName"), "Rust excluded export")
    reason = _exact_name(row.get("reason"), "Rust exclusion reason")
    evidence_paths = _evidence(row)
    approval = row.get("approvalSource") if prior else _feature_approval()
    if not isinstance(approval, dict) or approval.get("status") != "Approved":
        raise ResolutionError("Rust exclusion needs an approved source")
    record: dict[str, Any] = {
        "declarationId": _exclusion_id(package_id, export_name),
        "packageId": package_id,
        "exportName": export_name,
        "reason": reason,
        "approvalSource": approval,
        "evidence": row.get("evidence") or "; ".join(evidence_paths),
        "evidencePaths": evidence_paths,
    }
    if not prior:
        record["candidateSpecId"] = "KMIPKIT-0053"
        record["rustArtifactKind"] = row.get("artifactKind", "exact-route")
        for key in ("identity", "condition", "ownerExportName", "ownerDeclarationId"):
            value = row.get(key)
            if value is not None:
                if key == "condition" and not isinstance(value, str):
                    value = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
                record[key] = value
        if row.get("exclusionScope") is not None:
            record["exclusionScope"] = row["exclusionScope"]
        if row.get("exclusionCategory") is not None:
            record["exclusionCategory"] = row["exclusionCategory"]
        if row.get("precedent") is not None:
            record["precedent"] = row["precedent"]
    else:
        record["candidateSpecId"] = "KMIPKIT-0004"
    return record


def compile_documents(
    documents: list[dict[str, Any]],
    existing_declarations: list[dict[str, Any]] | None = None,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Compile mapped rows and refuse unresolved exports before route generation."""
    declarations: dict[str, dict[str, Any]] = {}
    exclusions: dict[tuple[str, str], dict[str, Any]] = {}
    rust_shape_exclusions: list[dict[str, Any]] = []
    rust_route_exclusions: list[dict[str, Any]] = []
    rust_functional_rows: list[dict[str, Any]] = []
    for document in documents:
        if document.get("formatVersion") != 1 or document.get("specId") != "KMIPKIT-0053":
            raise ResolutionError("export audit document has an unsupported format or spec ID")
        if document.get("packageId") == "python-kmipkit" or (
            "mappings" in document and "kindCorrections" in document
        ):
            document = _normalize_python_audit(document)
        unresolved = list(document.get("unresolved", []))
        if "functionalRoutes" in document:
            rust_functional_rows.extend(document.get("functionalRoutes", []))
            unresolved.extend(row for row in rust_functional_rows if row.get("resolution") == "unresolved")
            rows = []
            rust_shape_exclusions.extend(document.get("enumShapeExclusions", []))
            rust_route_exclusions.extend(document.get("traitImplementationExclusions", []))
            rust_route_exclusions.extend(document.get("exactRouteExclusions", []))
            rust_route_exclusions.extend(
                {**row, "approvalSource": row.get("approvalSource"), "priorApproved": True}
                for row in document.get("priorApprovedExclusions", [])
            )
            exclusion_rows = []
        else:
            rows = document.get("mappings", [])
            unresolved.extend(
                row for row in rows if row.get("resolution", "mapped") != "mapped"
            )
            exclusion_rows = document.get("exclusions", [])
        if unresolved:
            first = unresolved[0]
            name = first.get("name", first.get("exportName", "<unnamed>"))
            raise ResolutionError(f"unresolved public export blocks route generation: {name}")
        for row in rows:
            declaration = _declaration(document, row)
            identifier = declaration["declarationId"]
            previous = declarations.get(identifier)
            if previous is None:
                declarations[identifier] = declaration
                continue
            if previous["languageMappings"] != declaration["languageMappings"]:
                raise ResolutionError("stable route identity has conflicting language mappings")
            previous["requirementIds"] = sorted(
                set(previous["requirementIds"]) | set(declaration["requirementIds"])
            )
            previous["parityEvidenceIds"] = sorted(
                set(previous["parityEvidenceIds"]) | set(declaration["parityEvidenceIds"])
            )
        for row in exclusion_rows:
            exclusion = _exact_exclusion(document, row)
            key = exclusion["packageId"], exclusion["exportName"]
            previous = exclusions.get(key)
            if previous is not None and previous != exclusion:
                raise ResolutionError(f"conflicting exact exclusion record: {key[1]}")
            exclusions[key] = exclusion
    existing_declarations = existing_declarations or []
    declarations_by_export = {
        (row["packageId"], row["exportName"]): row
        for row in [*existing_declarations, *declarations.values()]
    }
    rust_audit_routes = _validate_rust_functional_routes(
        rust_functional_rows, existing_declarations
    )
    for row in rust_shape_exclusions:
        exclusion = _rust_type_exclusion(row, declarations_by_export, rust_audit_routes)
        key = exclusion["packageId"], exclusion["exportName"]
        previous = exclusions.get(key)
        if previous is not None and previous != exclusion:
            raise ResolutionError(f"conflicting Rust shape exclusion: {key[1]}")
        exclusions[key] = exclusion
    for row in rust_route_exclusions:
        exclusion = _rust_route_exclusion(row, prior=bool(row.get("priorApproved")))
        key = exclusion["packageId"], exclusion["exportName"]
        previous = exclusions.get(key)
        if previous is not None and previous != exclusion:
            raise ResolutionError(f"conflicting Rust route exclusion: {key[1]}")
        exclusions[key] = exclusion
    return (
        sorted(declarations.values(), key=lambda row: (row["packageId"], row["exportName"])),
        sorted(exclusions.values(), key=lambda row: (row["packageId"], row["exportName"])),
    )


def merge_declarations(
    route_data: dict[str, Any],
    declarations: list[dict[str, Any]],
    exclusions: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Merge exact audited routes without changing hand-authored declarations."""
    result = copy.deepcopy(route_data)
    current = result.get("publicDeclarations")
    if not isinstance(current, list):
        raise ResolutionError("additional route input lacks publicDeclarations")
    prefix = "KMIPKIT-0053:export:"
    current = [
        row for row in current
        if not str(row.get("declarationId", "")).startswith(prefix)
    ]
    by_id = {row.get("declarationId"): row for row in current}
    by_export = {(row.get("packageId"), row.get("exportName")): row for row in current}
    for declaration in declarations:
        key = (declaration["packageId"], declaration["exportName"])
        existing_id = by_id.get(declaration["declarationId"])
        existing_export = by_export.get(key)
        if existing_export is not None:
            if existing_id is not None and existing_id is not existing_export:
                raise ResolutionError("imported declaration ID conflicts with another reviewed route")
            exact_route_match = all(
                existing_export.get(field) == declaration.get(field)
                for field in (
                    "kind", "capabilityId", "declarationState", "parityScope",
                    "languageMappings", "approvalSource",
                )
            )
            if not exact_route_match:
                raise ResolutionError(f"imported export conflicts with a reviewed route: {key[1]}")
            existing_export["requirementIds"] = sorted(
                set(existing_export.get("requirementIds", []))
                | set(declaration["requirementIds"])
            )
            existing_export["parityEvidenceIds"] = sorted(
                set(existing_export.get("parityEvidenceIds", []))
                | set(declaration["parityEvidenceIds"])
            )
            continue
        if existing_id is not None:
            raise ResolutionError("imported declaration ID conflicts with a reviewed route")
        if existing_id is None and existing_export is None:
            current.append(declaration)
            by_id[declaration["declarationId"]] = declaration
            by_export[key] = declaration
    current.sort(key=lambda row: (row.get("packageId", ""), row.get("exportName", "")))
    result["publicDeclarations"] = current
    current_exclusions = result.get("approvedExclusions", [])
    if not isinstance(current_exclusions, list):
        raise ResolutionError("additional route input lacks approvedExclusions")
    exclusions_by_export = {
        (row.get("packageId"), row.get("exportName")): row for row in current_exclusions
    }
    for exclusion in exclusions or []:
        key = exclusion["packageId"], exclusion["exportName"]
        previous = exclusions_by_export.get(key)
        if previous is not None and previous != exclusion:
            raise ResolutionError(f"exact exclusion conflicts with a reviewed record: {key[1]}")
        if previous is None:
            current_exclusions.append(exclusion)
            exclusions_by_export[key] = exclusion
    current_exclusions.sort(
        key=lambda row: (row.get("packageId", ""), row.get("exportName", ""))
    )
    result["approvedExclusions"] = current_exclusions
    return result


def expand_java_stale_mappings(
    route_data: dict[str, Any], stale_mappings: list[dict[str, Any]] | None = None
) -> dict[str, Any]:
    """Replace package-private Java base routes with exact public subclass exports."""
    result = copy.deepcopy(route_data)
    declarations = result.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise ResolutionError("Java stale-route normalization needs publicDeclarations")
    stale_mappings = stale_mappings or result.get("javaStaleMappings", [])
    for stale in stale_mappings:
        old_name = _exact_name(stale.get("oldName"), "stale Java export")
        targets = stale.get("observedPublicTargets")
        if not isinstance(targets, list) or not targets:
            raise ResolutionError("stale Java export needs observed public targets")
        target_names = [_exact_name(row.get("name"), "Java public target") for row in targets]
        if len(target_names) != len(set(target_names)):
            raise ResolutionError("Java public targets must be unique")
        declaration_id = _exact_name(stale.get("declarationId"), "stale declaration ID")
        canonical_name = target_names[0]
        rows_for_old = [
            row for row in declarations
            if row.get("languageMappings", {}).get("java", {}).get("name") == old_name
        ]
        canonical_rows = [
            row for row in declarations
            if row.get("packageId") == "java-kmipkit"
            and row.get("declarationId") == declaration_id
        ]
        if not canonical_rows:
            raise ResolutionError(f"stale Java declaration is missing: {old_name}")
        canonical_row = canonical_rows[0]
        if canonical_row.get("exportName") not in {old_name, canonical_name}:
            raise ResolutionError("stale Java declaration identity changed unexpectedly")
        if canonical_row.get("exportName") == old_name:
            canonical_row["exportName"] = canonical_name
        canonical_row["languageMappings"]["java"]["name"] = canonical_name
        for row in rows_for_old:
            row["languageMappings"]["java"]["name"] = canonical_name
        for target_name in target_names[1:]:
            owner = target_name.rsplit(".", 1)[0].rsplit(".", 1)[-1]
            clone = copy.deepcopy(canonical_row)
            clone["declarationId"] = declaration_id + ":wrapper:" + owner.lower()
            clone["exportName"] = target_name
            clone["languageMappings"]["java"]["name"] = target_name
            existing = next(
                (
                    row for row in declarations
                    if row.get("packageId") == "java-kmipkit"
                    and row.get("exportName") == target_name
                ),
                None,
            )
            if existing is not None:
                exact_route_match = all(
                    existing.get(field) == clone.get(field)
                    for field in (
                        "declarationId", "packageId", "kind", "exportName",
                        "capabilityId", "declarationState", "parityScope",
                        "languageMappings", "approvalSource",
                    )
                )
                if not exact_route_match:
                    raise ResolutionError(
                        f"Java wrapper route conflicts with an existing declaration: {target_name}"
                    )
                existing["requirementIds"] = sorted(
                    set(existing.get("requirementIds", []))
                    | set(clone.get("requirementIds", []))
                )
                existing["parityEvidenceIds"] = sorted(
                    set(existing.get("parityEvidenceIds", []))
                    | set(clone.get("parityEvidenceIds", []))
                )
            if existing is None:
                declarations.append(clone)
    result.pop("javaStaleMappings", None)
    declarations.sort(key=lambda row: (row.get("packageId", ""), row.get("exportName", "")))
    return result


def _read_documents(root: Path) -> list[dict[str, Any]]:
    return [
        json.loads((root / path).read_text(encoding="utf-8"))
        for path in AUDIT_PATHS
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--write", action="store_true", help="update reviewed route input")
    arguments = parser.parse_args()
    route_path = arguments.repo_root / ROUTE_PATH
    manifest_path = arguments.repo_root / MANIFEST_PATH
    try:
        documents = _read_documents(arguments.repo_root)
        route_data = json.loads(route_path.read_text(encoding="utf-8"))
        manifest_data = json.loads(manifest_path.read_text(encoding="utf-8"))
        declarations, exclusions = compile_documents(
            documents, manifest_data.get("publicDeclarations", [])
        )
        route_data = remove_reviewed_declarations(
            route_data, route_data.get("removePublicDeclarations", [])
        )
        route_data = expand_java_stale_mappings(
            route_data, documents[0].get("staleMappings", [])
        )
        python_document = next(
            document for document in documents if document.get("packageId") == "python-kmipkit"
        )
        route_data = normalize_python_stale_mappings(
            route_data, python_document.get("staleMappings", [])
        )
        rust_document = next(document for document in documents if "functionalRoutes" in document)
        route_data = apply_kind_corrections(
            route_data,
            python_document.get("kindCorrections", []),
            rust_document.get("kindCorrections", []),
        )
        result = merge_declarations(route_data, declarations, exclusions)
        content = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
        current = route_path.read_text(encoding="utf-8")
    except (OSError, json.JSONDecodeError, ResolutionError) as error:
        print(f"KMIPKIT-0053 export route preparation failed: {error}", file=sys.stderr)
        return 2
    if content != current:
        if not arguments.write:
            print("KMIPKIT-0053 export routes are stale; rerun with --write.", file=sys.stderr)
            return 1
        route_path.write_bytes(content.encode("utf-8"))
    action = "updated" if arguments.write else "checked"
    print(f"{len(declarations)} audited export declarations {action} in reviewed routes.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
