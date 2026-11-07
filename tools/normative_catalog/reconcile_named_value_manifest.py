"""Reconcile the approved named-value surface into the reviewed API manifest."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

if __package__:
    from .generate_named_values import CATALOG_PATH, collect_values
    from .safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from .validate import MAX_BYTES, CatalogValidationError, load_validated_catalog
else:
    from generate_named_values import CATALOG_PATH, collect_values
    from safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from validate import MAX_BYTES, CatalogValidationError, load_validated_catalog

MANIFEST_PATH = "specification/api/public-api.json"
# Keep manifest reads bounded while allowing the current 48 MiB inventory and one measured growth increment.
MAX_MANIFEST_BYTES = 64 * 1024 * 1024
CAPABILITY_ID = "named_protocol_values"
C_ONLY_MASK_CAPABILITY_ID = "named_protocol_mask_single_bit_lookup"
SPEC_ID = "KMIPKIT-0047"
REQUIREMENT_IDS = tuple(f"{SPEC_ID}-FR-{index:03}" for index in range(1, 8))
LANGUAGE_PACKAGES = {
    "rust": "rust-kmipkit",
    "c": "c-kmipkit",
    "java": "java-kmipkit",
    "python": "python-kmipkit",
}
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/047-named-protocol-values/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved",
}


def _language_names(kind: str, group: Any, member: str) -> dict[str, str]:
    group_snake = "_".join(word.lower() for word in group.identifier.split("_"))
    group_type = "".join(word.title() for word in group.identifier.split("_"))
    group_prefix = f"{kind.upper()}_{group.identifier}"
    if member == "KNOWN_MASK":
        return {
            "rust": f"kmipkit::protocol::named_values_generated::values::bitmask::{group_snake}::KNOWN_MASK",
            "c": f"KMIPKIT_BITMASK_{group.identifier}_KNOWN_MASK",
            "java": f"org.kmipkit.protocol.KmipValues.{group_type}.KNOWN_MASK",
            "python": f"kmipkit.protocol_values.{group_type}.KNOWN_MASK",
        }
    if member in {"name", "names", "known_bits", "unknown_bits"}:
        java_member = {
            "known_bits": "knownBits",
            "unknown_bits": "unknownBits",
        }.get(member, member)
        return {
            "rust": f"kmipkit::protocol::named_values_generated::values::{kind}::{group_snake}::{member}",
            "c": f"KMIPKIT_{group_prefix}_{member}",
            "java": f"org.kmipkit.protocol.KmipValues.{group_type}.{java_member}",
            "python": f"kmipkit.protocol_values.{group_type}.{member}",
        }
    return {
        "rust": f"kmipkit::protocol::named_values_generated::values::{kind}::{group_snake}::{member}",
        "c": f"KMIPKIT_{group_prefix}_{member}",
        "java": f"org.kmipkit.protocol.KmipValues.{group_type}.{member}",
        "python": f"kmipkit.protocol_values.{group_type}.{member}",
    }


def _group_names(kind: str, group: Any) -> dict[str, str]:
    group_snake = "_".join(word.lower() for word in group.identifier.split("_"))
    group_type = "".join(word.title() for word in group.identifier.split("_"))
    return {
        "rust": f"kmipkit::protocol::named_values_generated::values::{kind}::{group_snake}",
        "c": f"kmipkit_{kind}_{group_snake}_t",
        "java": f"org.kmipkit.protocol.KmipValues.{group_type}",
        "python": f"kmipkit.protocol_values.{group_type}",
    }


def _root_namespace_names() -> dict[str, str]:
    return {
        "rust": "kmipkit::protocol::named_values_generated::values",
        "c": "kmipkit_named_values_namespace_t",
        "java": "org.kmipkit.protocol.KmipValues",
        "python": "kmipkit.protocol_values",
    }


def _declaration(
    language: str,
    export_name: str,
    requirement_ids: tuple[str, ...],
    *,
    c_value: int | None = None,
    is_c_helper: bool = False,
    declaration_kind: str | None = None,
    language_names: dict[str, str] | None = None,
    capability_id: str = CAPABILITY_ID,
    parity_scope: str = "four-language",
    package_id: str | None = None,
) -> dict[str, Any]:
    kind = declaration_kind or (
        "macro"
        if language == "c" and c_value is not None
        else "function"
        if language == "c" or language == "rust"
        else "java-member"
        if language == "java"
        else "python-export"
        if language == "python"
        else "function"
    )
    record: dict[str, Any] = {
        "declarationId": f"kmipkit-0047-{language}-{export_name}",
        "packageId": package_id or LANGUAGE_PACKAGES[language],
        "kind": kind,
        "exportName": export_name,
        "capabilityId": capability_id,
        "requirementIds": list(requirement_ids),
        "declarationState": "declared",
        "parityScope": parity_scope,
        "approvalSource": APPROVAL_SOURCE,
        "languageMappings": {
            key: {
                "packageId": (
                    package_id
                    if key == language and package_id
                    else LANGUAGE_PACKAGES[key]
                ),
                "name": (language_names or {language: export_name}).get(key, export_name),
            }
            for key in LANGUAGE_PACKAGES
            if key in (language_names or {language: export_name})
        },
    }
    if c_value is not None:
        record["cValue"] = c_value
        record["cValueType"] = "uint32_t"
    if is_c_helper and c_value is None:
        record["kind"] = "function"
    return record


def _append_declaration(
    declarations: list[dict[str, Any]],
    language: str,
    export_name: str,
    requirement_ids: tuple[str, ...],
    **options: Any,
) -> None:
    declarations.append(
        _declaration(language, export_name, requirement_ids, **options)
    )
    if language != "rust":
        return

    protocol_export = export_name.replace(
        "kmipkit::protocol::", "kmipkit_protocol::", 1
    )
    if protocol_export == export_name:
        return
    protocol_names = dict(options.get("language_names") or {})
    protocol_names["rust"] = protocol_export
    protocol_options = dict(options)
    protocol_options["language_names"] = protocol_names
    protocol_options["package_id"] = "rust-kmipkit-protocol"
    declarations.append(
        _declaration("rust", protocol_export, requirement_ids, **protocol_options)
    )


def build_named_value_records(catalog: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, list[str]]]:
    values = collect_values(catalog)
    declarations: list[dict[str, Any]] = []
    root_names = _root_namespace_names()
    for language, export_name in root_names.items():
        _append_declaration(
            declarations,
            language,
            export_name,
            (f"{SPEC_ID}-FR-005",),
            declaration_kind=("module" if language in {"rust", "python"} else "type"),
            language_names=root_names,
        )
    for rust_module in (
        "kmipkit::protocol::named_values_generated",
        "kmipkit::protocol::named_values_generated::values::enumeration",
        "kmipkit::protocol::named_values_generated::values::bitmask",
    ):
        module_names = {**root_names, "rust": rust_module}
        _append_declaration(
            declarations,
            "rust",
            rust_module,
            (f"{SPEC_ID}-FR-005",),
            declaration_kind="module",
            language_names=module_names,
        )
    for kind, groups in (("enumeration", values.enumerations), ("bitmask", values.bitmasks)):
        requirement_id = f"{SPEC_ID}-FR-001" if kind == "enumeration" else f"{SPEC_ID}-FR-002"
        for group in groups:
            group_names = _group_names(kind, group)
            for language, export_name in group_names.items():
                _append_declaration(
                    declarations,
                    language,
                    export_name,
                    (requirement_id, f"{SPEC_ID}-FR-005"),
                    declaration_kind=("module" if language == "rust" else "type"),
                    language_names=group_names,
                )
            for value in group.values:
                names = _language_names(kind, group, value.identifier)
                for language, export_name in names.items():
                    _append_declaration(
                        declarations,
                        language,
                        export_name,
                        (requirement_id, f"{SPEC_ID}-FR-005"),
                        c_value=value.value if language == "c" else None,
                        declaration_kind=("constant" if language == "rust" else None),
                        language_names=names,
                    )
            if kind == "enumeration":
                helper_ids = (f"{SPEC_ID}-FR-003", f"{SPEC_ID}-FR-005")
                helpers = ("name",)
            else:
                helper_ids = (f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-005")
                helpers = ("KNOWN_MASK", "names", "known_bits", "unknown_bits")
            for helper in helpers:
                names = _language_names(kind, group, helper)
                for language, export_name in names.items():
                    _append_declaration(
                        declarations,
                        language,
                        export_name,
                        helper_ids,
                        c_value=(
                            sum(value.value for value in group.values)
                            if language == "c" and helper == "KNOWN_MASK"
                            else None
                        ),
                        is_c_helper=language == "c",
                        declaration_kind=(
                            "constant"
                            if language == "rust" and helper == "KNOWN_MASK"
                            else None
                        ),
                        language_names=names,
                    )

            if kind == "bitmask":
                c_name = _language_names(kind, group, "name")["c"]
                declarations.append(
                    _declaration(
                        "c",
                        c_name,
                        (f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-005"),
                        language_names={"c": c_name},
                        capability_id=C_ONLY_MASK_CAPABILITY_ID,
                        parity_scope="c-only",
                    )
                )

    mappings = {
        language: sorted(
            {
                declaration["languageMappings"][language]["name"]
                for declaration in declarations
                if declaration["capabilityId"] == CAPABILITY_ID
                and language in declaration["languageMappings"]
            }
        )
        for language in LANGUAGE_PACKAGES
    }
    declarations.sort(key=lambda declaration: declaration["declarationId"])
    return declarations, mappings


def _upsert_capability(
    capabilities: list[dict[str, Any]], capability: dict[str, Any]
) -> list[dict[str, Any]]:
    capability_id = capability["capabilityId"]
    existing = [
        row for row in capabilities if row.get("capabilityId") == capability_id
    ]
    if existing and existing[0].get("ownerSpecId") != SPEC_ID:
        raise ValueError(
            f"{capability_id} differs from its deterministic approved mapping"
        )
    if existing:
        return [
            capability if row.get("capabilityId") == capability_id else row
            for row in capabilities
        ]
    return [*capabilities, capability]


def reconcile(manifest: dict[str, Any], declarations: list[dict[str, Any]], mappings: dict[str, list[str]]) -> dict[str, Any]:
    updated = dict(manifest)
    requirements = set(updated["requirementIds"])
    requirements.update(REQUIREMENT_IDS)
    updated["requirementIds"] = sorted(requirements)
    existing_capabilities = list(updated["capabilities"])
    c_only_declarations = [
        declaration
        for declaration in declarations
        if declaration.get("capabilityId") == C_ONLY_MASK_CAPABILITY_ID
    ]
    capability_records = [
        {
            "capabilityId": CAPABILITY_ID,
            "requirementIds": list(REQUIREMENT_IDS),
            "capabilityState": "declared",
            "parityScope": "four-language",
            "ownerSpecId": SPEC_ID,
            "approvalEvidence": APPROVAL_SOURCE,
            "mappings": mappings,
        }
    ]
    if c_only_declarations:
        capability_records.append(
            {
                "capabilityId": C_ONLY_MASK_CAPABILITY_ID,
                "requirementIds": [f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-005"],
                "capabilityState": "declared",
                "parityScope": "c-only",
                "ownerSpecId": SPEC_ID,
                "approvalEvidence": APPROVAL_SOURCE,
                "mappings": {
                    "c": sorted(
                        declaration["languageMappings"]["c"]["name"]
                        for declaration in c_only_declarations
                    )
                },
            }
        )
    for capability in capability_records:
        existing_capabilities = _upsert_capability(
            existing_capabilities, capability
        )
    updated["capabilities"] = existing_capabilities

    generated_ids = {row["declarationId"] for row in declarations}
    existing_declarations = [
        row
        for row in updated["publicDeclarations"]
        if not (
            row.get("capabilityId") in {CAPABILITY_ID, C_ONLY_MASK_CAPABILITY_ID}
            and row.get("approvalSource", {}).get("path") == APPROVAL_SOURCE["path"]
            and row.get("declarationId") not in generated_ids
        )
    ]
    by_id = {row["declarationId"]: row for row in existing_declarations}
    for declaration in declarations:
        current = by_id.get(declaration["declarationId"])
        if (
            current is not None
            and current != declaration
            and current.get("approvalSource", {}).get("path") != APPROVAL_SOURCE["path"]
        ):
            raise ValueError(f"named-value declaration conflicts with existing {declaration['declarationId']}")
        by_id[declaration["declarationId"]] = declaration
    existing_ids = {row["declarationId"] for row in existing_declarations}
    reconciled = [by_id[row["declarationId"]] for row in existing_declarations]
    reconciled.extend(
        by_id[declaration_id]
        for declaration_id in sorted(by_id.keys() - existing_ids)
    )
    updated["publicDeclarations"] = reconciled
    return updated


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    args = parser.parse_args(argv)
    try:
        root = args.repo_root.resolve(strict=True)
        catalog = load_validated_catalog(
            safe_read_bytes(root, CATALOG_PATH, max_bytes=MAX_BYTES), root, require_complete=True
        )
        manifest = json.loads(safe_read_bytes(root, MANIFEST_PATH, max_bytes=MAX_MANIFEST_BYTES))
        declarations, mappings = build_named_value_records(catalog)
        updated = reconcile(manifest, declarations, mappings)
        expected = (json.dumps(updated, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
        if args.check:
            current = safe_read_bytes(root, MANIFEST_PATH, max_bytes=MAX_MANIFEST_BYTES)
            if current != expected:
                print("public API manifest is not reconciled with named values", file=sys.stderr)
                return 1
        else:
            atomic_write_bytes(root, MANIFEST_PATH, expected)
    except (OSError, PathSecurityError, CatalogValidationError, json.JSONDecodeError, ValueError) as error:
        print(f"named-value manifest reconciliation failed: {error}", file=sys.stderr)
        return 1
    print("Named-value public API declarations reconciled")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
