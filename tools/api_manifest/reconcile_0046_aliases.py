"""Reconcile exact KMIPKIT-0046 facade aliases into the reviewed manifest."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

MODULE_EXCLUSIONS = {
    "kmipkit::protocol",
    "kmipkit::ttlv",
    "kmipkit::client_config",
    "kmipkit::transport_config",
}
FACADE_PREFIXES = {
    "kmipkit::protocol::": (
        "kmipkit_protocol::",
        "kmipkit_protocol::message::",
    ),
    "kmipkit::ttlv::": ("kmipkit_ttlv::", "kmipkit_ttlv::codec::"),
    "kmipkit::client_config::": (
        "kmipkit_client::",
        "kmipkit_client::extension_registry::",
    ),
    "kmipkit::transport_config::": (
        "kmipkit_transport::",
        "kmipkit_transport::config::",
    ),
}
RUST_PACKAGES = {
    "rust-kmipkit-protocol",
    "rust-kmipkit-ttlv",
    "rust-kmipkit-client",
    "rust-kmipkit-transport",
}
ADR_APPROVAL = {
    "kind": "accepted-adr",
    "path": "docs/adr/0017-single-dependency-rust-facade.md",
    "status": "Accepted",
    "evidence": "The `kmipkit` crate re-exports the complete protocol and TTLV public namespaces and exposes the client and transport configuration types needed to construct its supported client.",
}


class ReconciliationError(ValueError):
    """The export report contains a path that cannot be mapped exactly."""


def _stable_declaration_id(export_name: str) -> str:
    digest = hashlib.sha256(f"rust-kmipkit\0{export_name}".encode()).hexdigest()[:20]
    return f"rust-observed-{digest}"


def _source_candidates(export_name: str, aliases: list[str], kind: str) -> set[tuple[str, str]]:
    candidates: set[tuple[str, str]] = set()
    for alias in [export_name, *aliases]:
        for facade_prefix, source_prefixes in FACADE_PREFIXES.items():
            if facade_prefix not in alias:
                continue
            for source_prefix in source_prefixes:
                source_name = alias.replace(facade_prefix, source_prefix)
                candidates.add(("", source_name))
                if kind == "field":
                    owner, separator, member = source_name.rpartition("::")
                    if separator:
                        candidates.add(("", f"{owner}.{member}"))
                if not alias.startswith("<"):
                    candidates.add(("", re.sub(r"<'[^>]+>", "", source_name)))
                    if kind == "field":
                        normalized = re.sub(r"<'[^>]+>", "", source_name)
                        owner, separator, member = normalized.rpartition("::")
                        if separator:
                            candidates.add(("", f"{owner}.{member}"))
    return candidates


def _find_source_record(
    row: dict[str, Any],
    manifest: dict[str, Any],
    matched_by_identity: dict[str, list[tuple[str, str]]],
    source_records: dict[tuple[str, str], tuple[str, dict[str, Any]]],
) -> tuple[str, dict[str, Any]] | None:
    identity = row.get("identity")
    if identity is not None:
        identity_key = json.dumps(identity, sort_keys=True, separators=(",", ":"))
        for package_id, export_name in sorted(matched_by_identity.get(identity_key, [])):
            source = source_records.get((package_id, export_name))
            if source is not None:
                return source

    for _, export_name in sorted(
        _source_candidates(row["exportName"], row.get("aliases", []), row["kind"])
    ):
        for package_id in sorted(RUST_PACKAGES):
            source = source_records.get((package_id, export_name))
            if source is not None:
                return source
    return None


def reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    """Return a manifest with exact aliases from one complete rustdoc report."""
    if not report.get("complete") or report.get("issues"):
        raise ReconciliationError("Rust export report is incomplete or contains audit issues")

    result = copy.deepcopy(manifest)
    source_records: dict[tuple[str, str], tuple[str, dict[str, Any]]] = {}
    for key, record_kind in (
        ("publicDeclarations", "declaration"),
        ("approvedExclusions", "exclusion"),
    ):
        for record in result.get(key, []):
            if record.get("packageId") in RUST_PACKAGES:
                source_records[(record["packageId"], record["exportName"])] = (
                    record_kind,
                    record,
                )

    matched_by_identity: dict[str, list[tuple[str, str]]] = {}
    for row in report.get("matched", []):
        if row.get("packageId") not in RUST_PACKAGES or row.get("identity") is None:
            continue
        identity_key = json.dumps(row["identity"], sort_keys=True, separators=(",", ":"))
        matched_by_identity.setdefault(identity_key, []).append(
            (row["packageId"], row["exportName"])
        )

    existing_paths = {
        (record["packageId"], record["exportName"])
        for key in ("publicDeclarations", "approvedExclusions")
        for record in result.get(key, [])
    }
    new_declarations: list[dict[str, Any]] = []
    new_exclusions: list[dict[str, Any]] = []
    capability_aliases: dict[str, set[str]] = {}
    operation_aliases: dict[str, set[str]] = {}

    for row in sorted(
        (entry for entry in report.get("missingFromManifest", []) if entry.get("packageId") == "rust-kmipkit"),
        key=lambda entry: entry["exportName"],
    ):
        path = row["exportName"]
        path_key = ("rust-kmipkit", path)
        if path_key in existing_paths:
            if path in MODULE_EXCLUSIONS:
                existing = next(
                    entry
                    for entry in result.get("approvedExclusions", [])
                    if (entry["packageId"], entry["exportName"]) == path_key
                )
                existing["approvalSource"] = copy.deepcopy(ADR_APPROVAL)
            continue
        if path in MODULE_EXCLUSIONS:
            new_exclusions.append({
                "packageId": "rust-kmipkit",
                "exportName": path,
                "reason": "This exact public module groups existing KMIPKit crate exports and does not define a separate cross-language capability.",
                "approvalSource": copy.deepcopy(ADR_APPROVAL),
                "evidence": f"Exact namespace alias observed by the KMIPKIT-0043 Rust export audit: {path}.",
            })
            existing_paths.add(path_key)
            continue

        source = _find_source_record(row, result, matched_by_identity, source_records)
        if source is None:
            raise ReconciliationError(f"no exact approved source declaration or exclusion for {path}")
        source_kind, source_record = source
        if source_kind == "exclusion":
            exclusion = copy.deepcopy(source_record)
            exclusion["packageId"] = "rust-kmipkit"
            exclusion["exportName"] = path
            source_evidence = exclusion.get("evidence", "")
            exclusion["evidence"] = (
                f"{source_evidence} Exact KMIPKIT-0046 facade alias observed by the Rust export audit: {path}."
            )
            new_exclusions.append(exclusion)
        elif source_record.get("mappingSemantics") in {
            "normalized-client-protocol-error",
            "normalized-client-protocol-error-outcome",
        }:
            requirement_ids = ", ".join(source_record.get("requirementIds", []))
            new_exclusions.append({
                "packageId": "rust-kmipkit",
                "exportName": path,
                "reason": "This exact Rust protocol-error alias is not the normalized cross-language client error surface; that capability remains mapped through the approved client execution error contract.",
                "approvalSource": copy.deepcopy(source_record["approvalSource"]),
                "evidence": (
                    f"Source declaration {source_record['declarationId']} is mapped as "
                    f"{source_record['mappingSemantics']}; requirements {requirement_ids}. "
                    f"This exact facade path is observed by the KMIPKIT-0043 Rust export audit: {path}."
                ),
            })
        else:
            declaration = copy.deepcopy(source_record)
            declaration["declarationId"] = _stable_declaration_id(path)
            declaration["packageId"] = "rust-kmipkit"
            declaration["exportName"] = path
            declaration["languageMappings"]["rust"] = {
                "packageId": "rust-kmipkit",
                "name": path,
            }
            new_declarations.append(declaration)
            capability_id = declaration.get("capabilityId")
            if capability_id:
                capability_aliases.setdefault(capability_id, set()).add(path)
                operation_aliases.setdefault(capability_id, set()).add(declaration["declarationId"])
        existing_paths.add(path_key)

    if not new_declarations and not new_exclusions:
        return result

    result["publicDeclarations"].extend(new_declarations)
    result.setdefault("approvedExclusions", []).extend(new_exclusions)
    for capability in result.get("capabilities", []):
        paths = capability_aliases.get(capability.get("capabilityId"))
        if paths:
            mappings = capability.setdefault("mappings", {})
            mappings["rust"] = sorted(set(mappings.get("rust", [])) | paths)
    for operation in result.get("operations", []):
        identifiers = operation_aliases.get(operation.get("operationId"))
        if identifiers:
            declarations = operation.setdefault("declarations", {}).setdefault("rust", [])
            operation["declarations"]["rust"] = sorted(set(declarations) | identifiers)
    return result


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
        print(f"KMIPKIT-0046 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    added = len(reconciled["publicDeclarations"]) - len(manifest["publicDeclarations"])
    exclusions_added = len(reconciled["approvedExclusions"]) - len(manifest.get("approvedExclusions", []))
    print(f"Prepared {added} exact declarations and {exclusions_added} exact exclusions.")
    if args.write:
        manifest_path.write_text(
            json.dumps(reconciled, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
