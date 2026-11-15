"""Prepare exact KMIP 3.0 system-credential parity resolutions for KMIPKIT-0053."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = Path("specification/api/public-api.json")
ROUTES_PATH = Path("specs/053-api-parity-facade/additional-api-routes.json")
SPEC_ID = "KMIPKIT-0053"
APPROVAL = {
    "kind": "feature-specification",
    "path": "specs/053-api-parity-facade/approval-record.md",
    "status": "Approved",
    "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
}
MODEL_PREFIXES = (
    "PasswordCredential",
    "DeviceCredential",
    "OneTimePasswordCredential",
    "HashedPasswordCredential",
)
CONSTRUCTOR_ROUTES = {
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_object_from_fields"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.ManagedObject.fromFields"},
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.managed_objects.ManagedObject.from_fields",
    },
}
FROM_ITEM_ROUTES = {
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_object_from_item"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.ManagedObject.fromItem"},
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.managed_objects.ManagedObject.from_item",
    },
}
ITEM_VIEW_ROUTES = {
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_ttlv_item_view_from_item"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.ManagedObject.view"},
    "python": {"packageId": "python-kmipkit", "name": "kmipkit.ttlv.ttlv_item_view_from_item"},
}
FIELD_ROUTES = {
    "c": {"packageId": "c-kmipkit", "name": "kmipkit_managed_value_field_item_read"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ttlv.ManagedObject.fieldItem"},
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.managed_objects.ManagedTtlvItem.field_item",
    },
}
EVIDENCE_PATHS = [
    "crates/kmipkit-protocol/src/system_credential_objects.rs",
    "crates/kmipkit-ffi/src/extension_registry/managed_objects.rs",
    "crates/kmipkit-ffi/tests/support/managed_object_0053_tests.rs",
    "bindings/c/tests/managed_object_parity_consumer.c",
    "bindings/java/src/main/java/org/kmipkit/ttlv/ManagedObject.java",
    "bindings/java/src/main/java/org/kmipkit/ttlv/ManagedTtlvValue.java",
    "bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java",
    "bindings/python/src/kmipkit/managed_objects.py",
    "bindings/python/src/kmipkit/ttlv.py",
    "bindings/python/tests/test_managed_object_parity.py",
    "specs/049-managed-objects/spec.md",
    "specs/053-api-parity-facade/spec.md",
]


class PreparationError(ValueError):
    """Raised when source candidates do not match the reviewed route set."""


def _mapping_for(export_name: str, kind: str) -> dict[str, dict[str, str]]:
    if "try_new" in export_name or "ObjectFields" in export_name or kind == "type":
        foreign = CONSTRUCTOR_ROUTES
    elif "try_from_item" in export_name or "::ManagedObject::" in export_name:
        foreign = FROM_ITEM_ROUTES
    elif "into_item" in export_name or "with_item" in export_name:
        foreign = ITEM_VIEW_ROUTES
    else:
        foreign = FIELD_ROUTES
    return {
        "rust": {"packageId": "", "name": export_name},
        **{language: dict(route) for language, route in foreign.items()},
    }


def build_system_credential_resolutions(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    """Build reviewed exact mappings for the four already-approved credential models."""
    declarations = manifest.get("publicDeclarations")
    if not isinstance(declarations, list):
        raise PreparationError("manifest lacks publicDeclarations")
    candidates = [
        row
        for row in declarations
        if row.get("packageId") in {"rust-kmipkit", "rust-kmipkit-protocol"}
        and row.get("declarationState") in {"blocked", "verified"}
        and "KMIPKIT-0049-FR-012" in row.get("requirementIds", [])
        and any(prefix in row.get("exportName", "") for prefix in MODEL_PREFIXES)
    ]
    if len(candidates) != 144:
        raise PreparationError(
            f"expected exactly 144 credential-model candidates; found {len(candidates)}"
        )
    resolutions: list[dict[str, Any]] = []
    for declaration in sorted(candidates, key=lambda row: row["declarationId"]):
        export_name = declaration.get("exportName")
        package_id = declaration.get("packageId")
        declaration_id = declaration.get("declarationId")
        if not all(isinstance(value, str) and value for value in (export_name, package_id, declaration_id)):
            raise PreparationError("credential candidate lacks exact identity fields")
        if declaration.get("capabilityId") != "typed_managed_objects_and_key_material":
            raise PreparationError(f"credential candidate has an unexpected capability: {export_name}")
        if "KMIPKIT-0049-FR-012" not in declaration.get("requirementIds", []):
            raise PreparationError(f"credential candidate lacks its approved KMIPKIT-0049 source: {export_name}")
        mappings = _mapping_for(export_name, str(declaration.get("kind", "")))
        mappings["rust"]["packageId"] = package_id
        resolutions.append(
            {
                "declarationId": declaration_id,
                "capabilityId": declaration["capabilityId"],
                "packageId": package_id,
                "exportName": export_name,
                "resolution": "mapped",
                "languageMappings": mappings,
                "evidencePaths": list(EVIDENCE_PATHS),
                "approvalSource": dict(APPROVAL),
            }
        )
    return resolutions


def prepare_route_data(
    manifest: dict[str, Any], route_data: dict[str, Any]
) -> dict[str, Any]:
    """Replace only the exact credential candidate resolutions with generated records."""
    result = json.loads(json.dumps(route_data))
    current = result.get("candidateResolutions")
    if not isinstance(current, list):
        raise PreparationError("route input lacks candidateResolutions")
    generated = build_system_credential_resolutions(manifest)
    generated_by_id = {row["declarationId"]: row for row in generated}
    retained = [row for row in current if row.get("declarationId") not in generated_by_id]
    result["candidateResolutions"] = [*retained, *generated]
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--write", action="store_true", help="write reviewed route input")
    arguments = parser.parse_args()
    manifest_path = arguments.repo_root / MANIFEST_PATH
    routes_path = arguments.repo_root / ROUTES_PATH
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        routes = json.loads(routes_path.read_text(encoding="utf-8"))
        prepared = prepare_route_data(manifest, routes)
        content = json.dumps(prepared, ensure_ascii=False, indent=2) + "\n"
        current = routes_path.read_text(encoding="utf-8")
    except (OSError, json.JSONDecodeError, PreparationError) as error:
        print(f"KMIPKIT-0053 system-credential route preparation failed: {error}", file=sys.stderr)
        return 2
    if content != current:
        if not arguments.write:
            print("KMIPKIT-0053 system-credential routes are stale; rerun with --write.", file=sys.stderr)
            return 1
        routes_path.write_bytes(content.encode("utf-8"))
    print(f"{len(prepared['candidateResolutions'])} reviewed candidate resolutions; 144 system-credential routes.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
