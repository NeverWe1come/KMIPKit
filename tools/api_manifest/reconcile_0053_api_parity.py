"""Reconcile reviewed four-language mappings for KMIPKIT-0053 candidates."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

SPEC_ID = "KMIPKIT-0053"
LANGUAGE_PACKAGES = {
    "rust": {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client"},
    "c": {"c-kmipkit"},
    "java": {"java-kmipkit"},
    "python": {"python-kmipkit"},
}
REQUIRED_LANGUAGES = frozenset(LANGUAGE_PACKAGES)
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
        if declaration.get("declarationState") != "verified" or declaration.get("parityScope") != "four-language":
            raise ReconciliationError("additional declaration requires verified four-language parity")
        mappings = declaration.get("languageMappings")
        if not isinstance(mappings, dict) or set(mappings) != REQUIRED_LANGUAGES:
            raise ReconciliationError("additional declaration must map all four languages")
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
