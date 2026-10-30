"""Reconcile the approved shared Rust API slice with the full Rust export audit.

The generated report is a review artifact. It does not promote blocked rows or
create C, Java, or Python declarations; those remain gated by their real
four-language mappings and implementations.
"""

from __future__ import annotations

import argparse
from collections import Counter
import importlib.util
import json
from pathlib import Path
import re
import sys
from typing import Any


RUST_PACKAGES = (
    "rust-kmipkit-ttlv",
    "rust-kmipkit-protocol",
    "rust-kmipkit-transport",
    "rust-kmipkit-client",
    "rust-kmipkit",
)
_RUNTIME_METADATA_NAMESPACE_BLOCKS = frozenset(
    {
        (
            "org.kmipkit.RuntimeMetadata:java",
            "KMIPKIT-0043",
            "The Rust crate root is not an exported declaration matching this binding namespace; parity remains blocked.",
        ),
        (
            "kmipkit.metadata:python",
            "KMIPKIT-0043",
            "The Rust crate root is not an exported declaration matching this binding namespace; parity remains blocked.",
        ),
    }
)
RUST_ONLY_CAPABILITY = "rust_transport_exchange"
RUST_ONLY_DECLARATIONS = {
    "kmipkit_transport::Transport",
    "kmipkit_transport::Transport::exchange",
    "kmipkit_transport::TransportResponse",
    "kmipkit_transport::TransportResponse::new",
    "kmipkit_transport::TransportResponse::as_bytes",
    "kmipkit_transport::RequestDeliveryState",
    "kmipkit_transport::RequestDeliveryState::NotSent",
    "kmipkit_transport::RequestDeliveryState::PossiblySent",
    "kmipkit_transport::RequestDeliveryState::ResponseStarted",
    "kmipkit_transport::RequestDeliveryState::not_sent",
    "kmipkit_transport::RequestDeliveryState::write_started",
    "kmipkit_transport::RequestDeliveryState::response_bytes_received",
    "kmipkit_transport::TransportCauseCategory",
    "kmipkit_transport::TransportCauseCategory::Io",
    "kmipkit_transport::TransportCauseCategory::Tls",
    "kmipkit_transport::TransportCauseCategory::Http",
    "kmipkit_transport::TransportCauseCategory::Timeout",
    "kmipkit_transport::TransportCauseCategory::Other",
    "kmipkit_transport::TransportError",
    "kmipkit_transport::TransportError::new",
    "kmipkit_transport::TransportError::delivery_state",
    "kmipkit_transport::TransportError::cause_category",
    "kmipkit::RequestDeliveryState",
    "kmipkit::RequestDeliveryState::NotSent",
    "kmipkit::RequestDeliveryState::PossiblySent",
    "kmipkit::RequestDeliveryState::ResponseStarted",
    "kmipkit::RequestDeliveryState::not_sent",
    "kmipkit::RequestDeliveryState::response_bytes_received",
    "kmipkit::RequestDeliveryState::write_started",
    "kmipkit::TransportCauseCategory",
    "kmipkit::TransportCauseCategory::Http",
    "kmipkit::TransportCauseCategory::Io",
    "kmipkit::TransportCauseCategory::Other",
    "kmipkit::TransportCauseCategory::Timeout",
    "kmipkit::TransportCauseCategory::Tls",
    "kmipkit::TransportError",
    "kmipkit::TransportError::cause_category",
    "kmipkit::TransportError::delivery_state",
    "kmipkit::TransportError::new",
}
DRAFT_SPEC_IDS = {"KMIPKIT-0010"}
REPORT_PATH = Path("specs/043-public-api-bindings/reports/shared-rust-inventory-2026-10-30.json")
_RUST_AUDIT_SPEC = importlib.util.spec_from_file_location(
    "audit_rust_exports", Path(__file__).with_name("audit_rust_exports.py")
)
_RUST_AUDIT = importlib.util.module_from_spec(_RUST_AUDIT_SPEC)
_RUST_AUDIT_SPEC.loader.exec_module(_RUST_AUDIT)


class InventoryError(ValueError):
    """The manifest or export audit cannot support a safe shared API inventory."""


def _export_key(row: dict[str, Any]) -> tuple[str, str]:
    return row.get("packageId"), _RUST_AUDIT._canonical_export_name(row.get("exportName", ""))


def _read_approved_owner(
    owner_spec_id: str,
    root: Path,
    approval_path: str | None = None,
) -> set[str]:
    root = root.resolve()
    spec_file: Path | None = None
    if approval_path is not None:
        candidate = (root / approval_path).resolve()
        try:
            candidate.relative_to(root)
        except ValueError as error:
            raise InventoryError("approved Rust declaration owner path escapes the repository") from error
        if candidate.is_file():
            spec_file = candidate
    else:
        spec_dir = next(
            (path for path in (root / "specs").iterdir() if path.name.startswith(owner_spec_id[9:])),
            None,
        )
        if spec_dir is not None:
            spec_file = spec_dir / "spec.md"
        else:
            owner_pattern = re.compile(
                rf"(?:\*\*Feature Branch\*\*:\s*`feature/{re.escape(owner_spec_id)}(?:-|`)|"
                rf"\b{re.escape(owner_spec_id)}-FR-\d{{3}}\b)"
            )
            matching_specs = [
                path
                for path in (root / "specs").glob("*/spec.md")
                if owner_pattern.search(path.read_text(encoding="utf-8"))
            ]
            if len(matching_specs) == 1:
                spec_file = matching_specs[0]
    if spec_file is None:
        raise InventoryError(f"approved Rust declaration owner has no spec document: {owner_spec_id}")
    text = spec_file.read_text(encoding="utf-8")
    status = re.search(r"^\*\*Status\*\*: *([^\r\n]+)", text, re.MULTILINE)
    if status is None or re.match(r"(?:Approved|Accepted)\b", status.group(1)) is None:
        raise InventoryError(f"Rust declaration owner is not formally Approved/Accepted: {owner_spec_id}")
    owner_requirements = set(re.findall(rf"\b{re.escape(owner_spec_id)}-FR-\d{{3}}\b", text))
    if not owner_requirements:
        local_requirement_ids = re.findall(
            r"^\s*-\s+\*\*FR-(\d{3})\*\*:", text, re.MULTILINE
        )
        owner_requirements = {
            f"{owner_spec_id}-FR-{requirement_id}"
            for requirement_id in local_requirement_ids
        }
    if not owner_requirements:
        raise InventoryError(f"approved Rust declaration owner has no requirements: {owner_spec_id}")
    return owner_requirements


def _observed_exports(
    rust_report: dict[str, Any],
) -> tuple[set[tuple[str, str]], set[tuple[str, str]], int, int]:
    if rust_report.get("complete") is not True:
        raise InventoryError("Rust export audit is not complete")
    for field in (
        "missingFromManifest",
        "notObserved",
        "issues",
        "kindMismatches",
        "conditionalConditionMismatches",
        "conditionalStateMismatches",
    ):
        if rust_report.get(field) != []:
            raise InventoryError(f"Rust export audit has unresolved {field}")
    direct = rust_report.get("declarations")
    conditional = rust_report.get("conditionalDeclarations")
    matched = rust_report.get("matched")
    excluded = rust_report.get("excluded")
    if (
        not isinstance(direct, list)
        or not isinstance(conditional, list)
        or not isinstance(matched, list)
        or not isinstance(excluded, list)
    ):
        raise InventoryError("Rust export audit is missing canonical declaration arrays")
    observed_keys = {_export_key(row) for row in [*direct, *conditional]}
    matched_keys = {_export_key(row) for row in matched}
    excluded_keys = {_export_key(row) for row in excluded}
    if len(matched_keys) != len(matched) or len(excluded_keys) != len(excluded):
        raise InventoryError("Rust export audit has duplicate matched or excluded paths")
    if matched_keys & excluded_keys or matched_keys | excluded_keys != observed_keys:
        raise InventoryError("Rust export audit matched and excluded paths differ from observed paths")
    if rust_report.get("matchesManifest") is not True:
        raise InventoryError("Rust export audit did not reconcile the observed paths to the manifest")
    return observed_keys, excluded_keys, len(direct), len(conditional)


def _verify_draft_gates(manifest: dict[str, Any]) -> dict[str, dict[str, int]]:
    declaration_counts: Counter[str] = Counter()
    for row in manifest.get("publicDeclarations", []):
        spec_id = row.get("candidateSpecId")
        if spec_id not in DRAFT_SPEC_IDS:
            continue
        declaration_counts[spec_id] += 1
        if row.get("declarationState") != "blocked":
            raise InventoryError(f"draft-owned declaration is not blocked: {row.get('exportName')}")
        if "approvalSource" in row or "languageMappings" in row:
            raise InventoryError(f"draft-owned declaration advertises an unapproved mapping: {row.get('exportName')}")

    operation_counts: Counter[str] = Counter()
    for row in manifest.get("operations", []):
        dependency = row.get("ownerDependency", {})
        spec_id = dependency.get("specId") if dependency.get("state") == "candidate" else None
        if spec_id not in DRAFT_SPEC_IDS:
            continue
        operation_counts[spec_id] += 1
        if row.get("apiState") != "blocked":
            raise InventoryError(f"draft-owned operation is not blocked: {row.get('name')}")
        if row.get("declarations") not in (None, [], {"rust": [], "c": [], "java": [], "python": []}):
            raise InventoryError(f"draft-owned operation advertises declarations: {row.get('name')}")

    return {
        spec_id: {
            "blockedDeclarations": declaration_counts[spec_id],
            "blockedOperations": operation_counts[spec_id],
        }
        for spec_id in sorted(DRAFT_SPEC_IDS)
    }


def _verify_ready_mapping_exports(
    manifest: dict[str, Any], rust_report: dict[str, Any]
) -> None:
    observed_names: dict[str, set[str]] = {}
    for row in rust_report["matched"]:
        names = observed_names.setdefault(row["packageId"], set())
        names.add(_RUST_AUDIT._canonical_export_name(row["exportName"]))
        names.update(
            _RUST_AUDIT._canonical_export_name(alias)
            for alias in row.get("aliases", [])
        )

    ready_rows = [
        row
        for row in manifest.get("publicDeclarations", [])
        if row.get("declarationState") in {"declared", "verified"}
    ]
    shared_capability_ids = {
        row.get("capabilityId") for row in manifest.get("capabilities", [])
    }
    operation_ids = {
        row.get("operationId") for row in manifest.get("operations", [])
    }
    aggregate: dict[str, dict[str, set[str]]] = {}
    for row in ready_rows:
        capability_id = row.get("capabilityId")
        mappings = row.get("languageMappings", {})
        rust = mappings.get("rust", {})
        package_id = rust.get("packageId")
        rust_name = rust.get("name", "")
        if _RUST_AUDIT._canonical_export_name(rust_name) not in observed_names.get(package_id, set()):
            raise InventoryError(
                f"ready declaration maps to no exact published Rust export: {row.get('declarationId')}"
            )
        if capability_id not in shared_capability_ids:
            if capability_id in operation_ids:
                continue
            raise InventoryError(
                f"ready declaration references no shared capability or operation: {row.get('declarationId')}"
            )
        language_names = aggregate.setdefault(capability_id, {})
        for language, mapping in mappings.items():
            if language not in {"rust", "c", "java", "python"}:
                continue
            language_names.setdefault(language, set()).add(mapping.get("name", ""))

    for capability in manifest.get("capabilities", []):
        if capability.get("capabilityState") not in {"declared", "verified"}:
            continue
        expected = aggregate.get(capability.get("capabilityId"), {})
        actual = capability.get("mappings", {})
        languages = ("rust",) if capability.get("parityScope") == "rust-only-adr" else (
            "rust", "c", "java", "python"
        )
        for language in languages:
            if sorted(expected.get(language, set())) != actual.get(language):
                raise InventoryError(
                    f"capability {capability.get('capabilityId')} {language} inventory differs from ready declarations"
                )


def _blocked_unreconciled_rows(manifest: dict[str, Any]) -> list[dict[str, str]]:
    blocked: list[dict[str, str]] = []
    for row in manifest.get("publicDeclarations", []):
        reason = row.get("blockedReason", "")
        declaration_id = row.get("declarationId")
        registry_candidate = row.get("candidateSpecId") == "KMIPKIT-0012"
        runtime_namespace_candidate = (
            declaration_id,
            row.get("candidateSpecId"),
            reason,
        ) in _RUNTIME_METADATA_NAMESPACE_BLOCKS
        if not (registry_candidate or runtime_namespace_candidate) or not reason:
            continue
        if "callable public declaration" not in reason and "No exact equivalent public Rust declaration" not in reason and "crate root is not an exported declaration" not in reason:
            continue
        if row.get("declarationState") != "blocked" or "languageMappings" in row or "approvalSource" in row:
            raise InventoryError(f"unreconciled declaration carries a parity claim: {row.get('exportName')}")
        blocked.append(
            {
                "declarationId": row["declarationId"],
                "packageId": row["packageId"],
                "exportName": row["exportName"],
                "candidateSpecId": row["candidateSpecId"],
                "blockedReason": reason,
            }
        )
    return sorted(blocked, key=lambda row: (row["packageId"], row["exportName"]))


def build_inventory(
    manifest: dict[str, Any], rust_report: dict[str, Any], root: Path
) -> dict[str, Any]:
    """Build a deterministic report for exact, already-mapped shared Rust APIs."""
    packages = {
        row.get("id"): row
        for row in manifest.get("publicPackages", [])
        if row.get("id") in RUST_PACKAGES
    }
    if set(packages) != set(RUST_PACKAGES):
        raise InventoryError("public package inventory does not contain the exact five published Rust crates")

    observed, audited_exclusions, direct_count, conditional_count = _observed_exports(rust_report)
    _verify_ready_mapping_exports(manifest, rust_report)
    manifest_rows = [
        row for row in manifest.get("publicDeclarations", []) if row.get("packageId") in RUST_PACKAGES
    ]
    manifest_by_key = {_export_key(row): row for row in manifest_rows}
    approved_exclusion_rows = [
        row for row in manifest.get("approvedExclusions", []) if row.get("packageId") in RUST_PACKAGES
    ]
    approved_exclusions = {_export_key(row) for row in approved_exclusion_rows}
    if len(approved_exclusions) != len(approved_exclusion_rows):
        raise InventoryError("approved Rust export exclusions contain duplicate exact paths")
    if approved_exclusions != audited_exclusions:
        raise InventoryError("Rust export audit exclusions differ from exact approved manifest exclusions")
    if set(manifest_by_key) & approved_exclusions:
        raise InventoryError("a Rust export is listed as both a public declaration and an approved exclusion")
    missing = observed - set(manifest_by_key) - approved_exclusions
    if missing:
        package_id, export_name = sorted(missing)[0]
        raise InventoryError(f"observed Rust export is missing a manifest row: {package_id} {export_name}")

    capabilities = {
        row.get("capabilityId"): row for row in manifest.get("capabilities", [])
    }
    operation_ids = {row.get("operationId") for row in manifest.get("operations", [])}
    mapped: list[dict[str, Any]] = []
    rust_only: list[dict[str, Any]] = []
    for key in sorted(observed):
        if key in approved_exclusions:
            continue
        declaration = manifest_by_key[key]
        if declaration.get("declarationState") not in {"declared", "verified"}:
            continue
        if declaration.get("capabilityId") in operation_ids:
            continue
        capability = capabilities.get(declaration.get("capabilityId"))
        if capability is None or capability.get("capabilityState") not in {"declared", "verified"}:
            raise InventoryError(f"ready Rust declaration has no ready capability: {declaration['exportName']}")
        requirement_ids = declaration.get("requirementIds", [])
        if not requirement_ids:
            raise InventoryError(f"ready Rust declaration has no requirement traceability: {declaration['exportName']}")

        owner_spec_id = capability.get("ownerSpecId")
        owner_adr_id = capability.get("ownerAdrId")
        if owner_spec_id:
            owner_requirements = sorted(
                requirement_id
                for requirement_id in requirement_ids
                if requirement_id.startswith(f"{owner_spec_id}-FR-")
            )
            approved_path = capability.get("approvalEvidence", {}).get("path")
            declared_owner_requirements = _read_approved_owner(owner_spec_id, root, approved_path)
            if not owner_requirements:
                raise InventoryError(f"ready Rust declaration lacks owner requirements: {declaration['exportName']}")
            unknown = set(owner_requirements) - declared_owner_requirements
            if unknown:
                raise InventoryError(f"Rust declaration references absent owner requirements: {sorted(unknown)}")
            if declaration.get("approvalSource") != capability.get("approvalEvidence"):
                raise InventoryError(f"Rust declaration approval evidence differs from its capability: {declaration['exportName']}")
        elif owner_adr_id == "ADR-0014":
            owner_requirements = []
            if not {"KMIPKIT-0043-FR-020", "KMIPKIT-0043-FR-021"} <= set(requirement_ids):
                raise InventoryError(f"ADR-owned Rust declaration lacks 0043 traceability: {declaration['exportName']}")
            if declaration.get("approvalSource") != capability.get("acceptedEvidence"):
                raise InventoryError(f"Rust declaration acceptance evidence differs from its capability: {declaration['exportName']}")
            if declaration.get("parityScope") != "rust-only-adr" or declaration.get("capabilityId") != RUST_ONLY_CAPABILITY:
                raise InventoryError("only ADR-0014 transport declarations may use Rust-only parity")
            rust_only.append(declaration)
        else:
            raise InventoryError(f"ready Rust declaration has no approved feature/ADR owner: {declaration['exportName']}")

        if declaration.get("parityScope") != "rust-only-adr":
            mappings = declaration.get("languageMappings", {})
            if set(mappings) != {"rust", "c", "java", "python"}:
                raise InventoryError(f"four-language Rust declaration has incomplete mappings: {declaration['exportName']}")
            for language in ("rust", "c", "java", "python"):
                if not mappings[language].get("name"):
                    raise InventoryError(f"Rust declaration has an empty {language} mapping: {declaration['exportName']}")

        entry = {
            "declarationId": declaration["declarationId"],
            "packageId": declaration["packageId"],
            "kind": declaration["kind"],
            "exportName": declaration["exportName"],
            "capabilityId": declaration["capabilityId"],
            "ownerSpecId": owner_spec_id,
            "ownerAdrId": owner_adr_id,
            "ownerRequirementIds": owner_requirements,
            "requirementIds": sorted(requirement_ids),
            "parityScope": declaration["parityScope"],
        }
        mapped.append(entry)

    rust_only_names = {declaration["exportName"] for declaration in rust_only}
    if rust_only_names != RUST_ONLY_DECLARATIONS:
        raise InventoryError("Rust-only inventory differs from the sole accepted ADR-0014 transport exception")

    draft_gates = _verify_draft_gates(manifest)
    blocked_rows = _blocked_unreconciled_rows(manifest)
    mapped_by_package = Counter(row["packageId"] for row in mapped)
    excluded_by_package = Counter(package_id for package_id, _ in approved_exclusions)
    observed_by_package = Counter(package_id for package_id, _ in observed)
    crate_inventory = [
        {
            "packageId": package_id,
            "crateName": package_id.removeprefix("rust-"),
            "observedDeclarationCount": observed_by_package[package_id],
            "mappedDeclarationCount": mapped_by_package[package_id],
            "excludedDeclarationCount": excluded_by_package[package_id],
            "blockedDeclarationCount": (
                observed_by_package[package_id]
                - mapped_by_package[package_id]
                - excluded_by_package[package_id]
            ),
        }
        for package_id in RUST_PACKAGES
    ]
    return {
        "formatVersion": 1,
        "scope": "approved-shared-rust-api",
        "source": "specs/043-public-api-bindings/reports/rust-export-report-2026-10-30.json",
        "observedRustDeclarationCount": len(observed),
        "directRustDeclarationCount": direct_count,
        "conditionalRustDeclarationCount": conditional_count,
        "mappedRustDeclarationCount": len(mapped),
        "approvedExcludedRustDeclarationCount": len(approved_exclusions),
        "crates": crate_inventory,
        "mappedDeclarations": mapped,
        "rustOnlyDeclarations": [
            {
                "declarationId": declaration["declarationId"],
                "exportName": declaration["exportName"],
                "ownerAdrId": "ADR-0014",
                "approvalPath": "docs/adr/0014-public-transport-exchange-contract.md",
                "requirementIds": sorted(declaration["requirementIds"]),
            }
            for declaration in sorted(rust_only, key=lambda row: row["exportName"])
        ],
        "blockedUnreconciledDeclarations": blocked_rows,
        "draftOwnerGates": draft_gates,
    }


def _json_bytes(value: dict[str, Any]) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path, default=REPORT_PATH)
    parser.add_argument("--check", action="store_true", help="compare generated bytes without writing")
    args = parser.parse_args(argv)

    root = args.repo_root.resolve()
    target = root / args.report
    manifest_path = root / "specification" / "api" / "public-api.json"
    audit_path = root / "specs" / "043-public-api-bindings" / "reports" / "rust-export-report-2026-10-30.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        rust_report = json.loads(audit_path.read_text(encoding="utf-8"))
        output = _json_bytes(build_inventory(manifest, rust_report, root))
        if args.check:
            if not target.is_file() or target.read_bytes() != output:
                print(f"stale or missing shared Rust inventory report: {target}", file=sys.stderr)
                return 1
            print(f"shared Rust inventory report is current: {target}")
            return 0
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(output)
        print(f"wrote shared Rust inventory report: {target}")
        return 0
    except (InventoryError, OSError, json.JSONDecodeError) as error:
        print(f"shared Rust inventory audit failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
