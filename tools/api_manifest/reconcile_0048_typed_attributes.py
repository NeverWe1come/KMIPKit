"""Deterministically reconcile KMIPKIT-0048 Rust exports into the API manifest."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

SPEC_ID = "KMIPKIT-0048"
PARITY_SPEC_ID = "KMIPKIT-0053"
TYPED_CAPABILITY_ID = "typed_object_attributes"
TYPED_REQUIREMENTS = [
    f"KMIPKIT-0048-FR-{number:03}"
    for number in range(1, 11)
]
TYPED_BLOCKED_REASON = (
    "The typed attribute API is implemented in Rust under KMIPKIT-0048. Its C, "
    "Java, and Python equivalents are assigned to KMIPKIT-0053; this exact "
    "four-language route remains blocked until those mappings and parity evidence exist."
)
TRAIT_PROJECTION_REASON = (
    "This exact Rust trait-implementation projection is language-level behavior "
    "of a typed attribute value, not an independent KMIPKit capability. The "
    "underlying type, trait contract, and supported API routes are inventoried separately."
)
TRAIT_PROJECTION_EVIDENCE = (
    "KMIPKIT-0048-FR-001, KMIPKIT-0048-FR-002, KMIPKIT-0048-FR-008; exact "
    "rustdoc associated-item path for a compiler-visible trait implementation."
)
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/048-typed-attributes/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved by maintainer plan authorization",
}
NAMED_VALUES_FACADE_PREFIX = "kmipkit::protocol::named_values_generated"
NAMED_VALUES_PROTOCOL_PREFIX = "kmipkit_protocol::named_values_generated"
UNIMPLEMENTED_MASK_NAME_EXPORTS = {
    "rust-kmipkit": {
        "kmipkit::protocol::named_values_generated::values::bitmask::cryptographic_usage_mask::name",
        "kmipkit::protocol::named_values_generated::values::bitmask::protection_storage_mask::name",
        "kmipkit::protocol::named_values_generated::values::bitmask::storage_status_mask::name",
    },
    "java-kmipkit": {
        "org.kmipkit.protocol.KmipValues.CryptographicUsageMask.name",
        "org.kmipkit.protocol.KmipValues.ProtectionStorageMask.name",
        "org.kmipkit.protocol.KmipValues.StorageStatusMask.name",
    },
    "python-kmipkit": {
        "kmipkit.protocol_values.CryptographicUsageMask.name",
        "kmipkit.protocol_values.ProtectionStorageMask.name",
        "kmipkit.protocol_values.StorageStatusMask.name",
    },
}
C_ONLY_MASK_NAME_EXPORTS = {
    "KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_name",
    "KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_name",
    "KMIPKIT_BITMASK_STORAGE_STATUS_MASK_name",
}
JAVA_MASK_MEMBER_RENAMES = {
    f"org.kmipkit.protocol.KmipValues.{class_name}.{snake}":
    f"org.kmipkit.protocol.KmipValues.{class_name}.{camel}"
    for class_name in ("CryptographicUsageMask", "ProtectionStorageMask", "StorageStatusMask")
    for snake, camel in (("known_bits", "knownBits"), ("unknown_bits", "unknownBits"))
}
C_ONLY_MASK_CAPABILITY = {
    "capabilityId": "named_protocol_mask_single_bit_lookup",
    "requirementIds": ["KMIPKIT-0047-FR-004", "KMIPKIT-0047-FR-005"],
    "capabilityState": "declared",
    "parityScope": "c-only",
    "ownerSpecId": "KMIPKIT-0047",
    "approvalEvidence": {
        "kind": "feature-specification",
        "path": "specs/047-named-protocol-values/spec.md",
        "status": "Approved",
        "evidence": "**Status**: Approved",
    },
}


class ReconciliationError(ValueError):
    """Raised when an observed Rust export cannot be reconciled safely."""


def _without_names(names: list[str], excluded: set[str]) -> list[str]:
    """Filter exact paths while preserving the reviewed mapping order."""
    return [name for name in names if name not in excluded]


def reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    """Return a reconciled copy of *manifest* for the supplied audit report."""
    result = copy.deepcopy(manifest)
    declarations = result.setdefault("publicDeclarations", [])
    exclusions = result.setdefault("approvedExclusions", [])
    capabilities = result.setdefault("capabilities", [])
    capability_by_id = {row["capabilityId"]: row for row in capabilities}
    result["requirementIds"] = sorted(set(result.get("requirementIds", [])) | set(TYPED_REQUIREMENTS))
    for declaration in declarations:
        if declaration.get("packageId") == "java-kmipkit" and declaration.get("exportName") in JAVA_MASK_MEMBER_RENAMES:
            new_name = JAVA_MASK_MEMBER_RENAMES[declaration["exportName"]]
            declaration["exportName"] = new_name
            declaration["declarationId"] = f"kmipkit-0047-java-{new_name}"
            declaration["languageMappings"]["java"]["name"] = new_name
        for mapping in declaration.get("languageMappings", {}).values():
            if mapping.get("packageId") == "java-kmipkit" and mapping.get("name") in JAVA_MASK_MEMBER_RENAMES:
                mapping["name"] = JAVA_MASK_MEMBER_RENAMES[mapping["name"]]
    result["publicDeclarations"] = [
        row for row in declarations
        if row.get("exportName") not in UNIMPLEMENTED_MASK_NAME_EXPORTS.get(row.get("packageId"), set())
    ]
    declarations = result["publicDeclarations"]
    c_only_mask_names: list[str] = []
    for declaration in declarations:
        if declaration.get("packageId") == "c-kmipkit" and declaration.get("exportName") in C_ONLY_MASK_NAME_EXPORTS:
            c_mapping = declaration.get("languageMappings", {}).get("c")
            if c_mapping != {"packageId": "c-kmipkit", "name": declaration.get("exportName")}:
                raise ReconciliationError("C bit-name helper lacks its exact C-only mapping")
            declaration["parityScope"] = "c-only"
            declaration["capabilityId"] = C_ONLY_MASK_CAPABILITY["capabilityId"]
            declaration["languageMappings"] = {"c": c_mapping}
            c_only_mask_names.append(declaration["exportName"])
    if c_only_mask_names:
        capability = capability_by_id.get(C_ONLY_MASK_CAPABILITY["capabilityId"])
        if capability is None:
            capability = copy.deepcopy(C_ONLY_MASK_CAPABILITY)
            capability["mappings"] = {"c": sorted(c_only_mask_names)}
            capabilities.append(capability)
            capability_by_id[capability["capabilityId"]] = capability
        else:
            mappings = capability.setdefault("mappings", {})
            c_mappings = mappings.setdefault("c", [])
            c_mappings.extend(name for name in sorted(c_only_mask_names) if name not in c_mappings)
    for capability in capabilities:
        if capability.get("capabilityId") == "named_protocol_values":
            for language in ("rust", "java", "python"):
                capability["mappings"][language] = _without_names(
                    capability["mappings"].get(language, []),
                    UNIMPLEMENTED_MASK_NAME_EXPORTS[f"{language}-kmipkit"],
                )
            capability["mappings"]["java"] = [
                JAVA_MASK_MEMBER_RENAMES.get(name, name)
                for name in capability["mappings"].get("java", [])
            ]
            capability["mappings"]["c"] = _without_names(
                capability["mappings"].get("c", []), C_ONLY_MASK_NAME_EXPORTS
            )
    declaration_keys = {(row["packageId"], row["exportName"]) for row in declarations}
    exclusion_keys = {(row["packageId"], row["exportName"]) for row in exclusions}
    typed_routes: list[dict[str, Any]] = []
    named_routes: list[dict[str, Any]] = []

    missing = report.get("missingFromManifest")
    if not isinstance(missing, list):
        raise ReconciliationError("Rust export report has no missingFromManifest array")

    for observed in sorted(missing, key=lambda row: (row.get("packageId", ""), row.get("exportName", ""))):
        package_id = observed.get("packageId")
        export_name = observed.get("exportName")
        kind = observed.get("kind")
        if not isinstance(package_id, str) or not isinstance(export_name, str) or not isinstance(kind, str):
            raise ReconciliationError("Rust export report contains an incomplete declaration")
        key = package_id, export_name
        if key in declaration_keys or key in exclusion_keys:
            continue

        if package_id == "rust-kmipkit-protocol" and export_name.startswith(NAMED_VALUES_PROTOCOL_PREFIX):
            named_routes.append(observed)
            continue

        if not _is_typed_attribute_route(package_id, export_name):
            raise ReconciliationError(f"unowned Rust export cannot be attributed to KMIPKIT-0048: {package_id}:{export_name}")

        if kind == "associated-item" and export_name.startswith("<"):
            exclusions.append({
                "packageId": package_id,
                "exportName": export_name,
                "reason": TRAIT_PROJECTION_REASON,
                "approvalSource": copy.deepcopy(APPROVAL_SOURCE),
                "evidence": TRAIT_PROJECTION_EVIDENCE,
                "candidateSpecId": SPEC_ID,
            })
            exclusion_keys.add(key)
            continue

        typed_routes.append(observed)

    for observed in named_routes:
        _add_raw_named_values_alias(
            declarations,
            declaration_keys,
            capability_by_id,
            observed,
        )

    if typed_routes:
        capability = capability_by_id.get(TYPED_CAPABILITY_ID)
        if capability is None:
            capability = {
                "capabilityId": TYPED_CAPABILITY_ID,
                "requirementIds": list(TYPED_REQUIREMENTS),
                "capabilityState": "blocked",
                "parityScope": "four-language",
                "candidateSpecId": PARITY_SPEC_ID,
                "blockedReason": TYPED_BLOCKED_REASON,
            }
            capabilities.append(capability)
            capability_by_id[TYPED_CAPABILITY_ID] = capability
        elif capability.get("capabilityState") != "blocked" or "mappings" in capability:
            raise ReconciliationError("typed attribute capability must remain blocked until KMIPKIT-0053")

        for observed in typed_routes:
            package_id = observed["packageId"]
            export_name = observed["exportName"]
            key = package_id, export_name
            declarations.append({
                "declarationId": _declaration_id(package_id, export_name),
                "packageId": package_id,
                "kind": observed["kind"],
                "exportName": export_name,
                "capabilityId": TYPED_CAPABILITY_ID,
                "requirementIds": _typed_requirement_ids(export_name),
                "declarationState": "blocked",
                "parityScope": "four-language",
                "candidateSpecId": PARITY_SPEC_ID,
                "blockedReason": TYPED_BLOCKED_REASON,
            })
            declaration_keys.add(key)

    return result


def _is_typed_attribute_route(package_id: str, export_name: str) -> bool:
    """Accept only public paths whose visible owner is the 0048 attribute API."""
    prefix = {
        "rust-kmipkit": "kmipkit::protocol::",
        "rust-kmipkit-protocol": "kmipkit_protocol::",
    }.get(package_id)
    owner = export_name.removeprefix("<").split(" as ", 1)[0]
    if prefix is None or not owner.startswith(prefix):
        return False
    relative_owner = owner.removeprefix(prefix).split("<", 1)[0]
    if relative_owner == "attribute" or relative_owner.startswith("attribute::"):
        return True
    return relative_owner.split("::", 1)[0] in {
        "Attribute", "AttributeDefinition", "AttributeSet", "AttributeValueError",
        "AttributeView", "CurrentAttribute", "NewAttribute",
        "UniqueIdentifierValue", "UniqueIdentifierView",
    }


def _typed_requirement_ids(export_name: str) -> list[str]:
    if "::AttributeSet::" in export_name:
        relevant = ["KMIPKIT-0048-FR-003"]
    elif "::CurrentAttribute::" in export_name or "::NewAttribute::" in export_name:
        relevant = ["KMIPKIT-0048-FR-004"]
    elif "AttributeValueError" in export_name:
        relevant = ["KMIPKIT-0048-FR-005"]
    elif "UniqueIdentifier" in export_name:
        relevant = ["KMIPKIT-0048-FR-006"]
    else:
        relevant = ["KMIPKIT-0048-FR-001", "KMIPKIT-0048-FR-002"]
    return [*relevant, "KMIPKIT-0048-FR-008", "KMIPKIT-0048-FR-010"]


def _declaration_id(package_id: str, export_name: str) -> str:
    digest = hashlib.sha256(f"{package_id}:{export_name}".encode("utf-8")).hexdigest()[:20]
    package = package_id.removeprefix("rust-")
    return f"kmipkit-0048-{package}-{digest}"


def _add_raw_named_values_alias(
    declarations: list[dict[str, Any]],
    declaration_keys: set[tuple[str, str]],
    capability_by_id: dict[str, dict[str, Any]],
    observed: dict[str, Any],
) -> None:
    export_name = observed["exportName"]
    facade_name = export_name.replace("kmipkit_protocol::", "kmipkit::protocol::", 1)
    template = next((
        row for row in declarations
        if row["packageId"] == "rust-kmipkit"
        and row["exportName"] == facade_name
        and row["capabilityId"] == "named_protocol_values"
        and row["kind"] == observed["kind"]
    ), None)
    if template is None:
        raise ReconciliationError(f"raw named-value route has no facade equivalent: {export_name}")

    row = copy.deepcopy(template)
    row["packageId"] = "rust-kmipkit-protocol"
    row["exportName"] = export_name
    row["declarationId"] = _declaration_id("rust-kmipkit-protocol", export_name)
    row["languageMappings"]["rust"] = {
        "packageId": "rust-kmipkit-protocol",
        "name": export_name,
    }
    declarations.append(row)
    declaration_keys.add((row["packageId"], export_name))

    capability = capability_by_id.get("named_protocol_values")
    if capability is None or capability.get("capabilityState") == "blocked":
        raise ReconciliationError("named_protocol_values capability is missing or blocked")
    rust_mappings = capability.setdefault("mappings", {}).setdefault("rust", [])
    if export_name not in rust_mappings:
        rust_mappings.append(export_name)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--write", action="store_true", help="update the reviewed manifest input")
    args = parser.parse_args()
    manifest_path = args.repo_root / "specification/api/public-api.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        report = json.loads(args.report.read_text(encoding="utf-8"))
        reconciled = reconcile(manifest, report)
    except (OSError, json.JSONDecodeError, ReconciliationError) as error:
        print(f"KMIPKIT-0048 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    declarations_added = len(reconciled["publicDeclarations"]) - len(manifest["publicDeclarations"])
    exclusions_added = len(reconciled["approvedExclusions"]) - len(manifest.get("approvedExclusions", []))
    declaration_action = "write" if args.write else "would write"
    print(
        f"Prepared manifest reconciliation: {declarations_added:+} declarations, "
        f"{exclusions_added:+} exclusions; {declaration_action} reviewed input."
    )
    if args.write:
        manifest_path.write_text(
            json.dumps(reconciled, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
