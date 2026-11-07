"""Deterministically reconcile KMIPKIT-0049 Rust exports into manifest v2."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

SPEC_ID = "KMIPKIT-0049"
PARITY_SPEC_ID = "KMIPKIT-0053"
CAPABILITY_ID = "typed_managed_objects_and_key_material"
REQUIREMENTS = [f"{SPEC_ID}-FR-{number:03}" for number in range(1, 11)]
BLOCKED_REASON = (
    "This exact typed managed-object Rust route is implemented under KMIPKIT-0049. "
    "Its C, Java, and Python equivalents are assigned to KMIPKIT-0053; the route "
    "remains blocked for four-language parity until those mappings and evidence exist."
)
TRAIT_EXCLUSION_REASON = (
    "This exact Rust trait-implementation projection is compiler-visible behavior "
    "of the typed managed-object API, not an independent KMIPKit protocol capability."
)
TRAIT_EXCLUSION_EVIDENCE = (
    "KMIPKIT-0049-FR-001 through KMIPKIT-0049-FR-008; the owned public type and "
    "its protocol capability are inventoried as separate exact declarations."
)
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/049-managed-objects/spec.md",
    "status": "Approved",
    "evidence": "**Status**: Approved by maintainer plan authorization",
}
PUBLIC_TYPES = {
    "Certificate", "CertificateRequest", "EncryptionKeyInformation", "KeyBlock",
    "KeyBlockValue", "KeyMaterial", "KeyValue", "KeyWrappingData",
    "MacSignatureKeyInformation", "ManagedObject", "ManagedObjectError",
    "OpaqueObject", "PgpKey", "PrivateKey", "PublicKey", "SecretData",
    "SplitKey", "SymmetricKey", "TransparentDhPrivateKey",
    "TransparentDhPublicKey", "TransparentDsaPrivateKey", "TransparentDsaPublicKey",
    "TransparentEcPrivateKey", "TransparentEcPublicKey", "TransparentRsaPrivateKey",
    "TransparentRsaPublicKey", "TransparentSymmetricKey", "RegisterRequest",
    "ImportRequest",
}
PACKAGE_PREFIXES = {
    "rust-kmipkit": "kmipkit::protocol::",
    "rust-kmipkit-protocol": "kmipkit_protocol::",
}


class ReconciliationError(ValueError):
    """Raised when an observed public route does not belong to this feature."""


def _root_type(export_name: str, package_id: str) -> str | None:
    prefix = PACKAGE_PREFIXES.get(package_id)
    path = export_name
    if export_name.startswith("<"):
        separator = export_name.find(" as ")
        if separator < 2:
            raise ReconciliationError(f"malformed trait projection: {export_name}")
        path = export_name[1:separator]
    if prefix is None or not path.startswith(prefix):
        return None
    tail = path[len(prefix):]
    return tail.split("::", 1)[0] if tail else None


def _requirements_for(export_name: str, package_id: str) -> list[str]:
    root = _root_type(export_name, package_id)
    if root in {"RegisterRequest", "ImportRequest"}:
        relevant = [f"{SPEC_ID}-FR-006", f"{SPEC_ID}-FR-007"]
    elif root == "ManagedObjectError":
        relevant = [f"{SPEC_ID}-FR-007", f"{SPEC_ID}-FR-009"]
    elif root in {
        "KeyBlock", "KeyBlockValue", "KeyMaterial", "KeyValue", "KeyWrappingData",
        "EncryptionKeyInformation", "MacSignatureKeyInformation",
        "TransparentDhPrivateKey", "TransparentDhPublicKey", "TransparentDsaPrivateKey",
        "TransparentDsaPublicKey", "TransparentEcPrivateKey", "TransparentEcPublicKey",
        "TransparentRsaPrivateKey", "TransparentRsaPublicKey", "TransparentSymmetricKey",
    }:
        relevant = [f"{SPEC_ID}-FR-002", f"{SPEC_ID}-FR-003", f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-007"]
    else:
        relevant = [f"{SPEC_ID}-FR-001", f"{SPEC_ID}-FR-004", f"{SPEC_ID}-FR-005", f"{SPEC_ID}-FR-007"]
    return sorted(set(relevant))


def _declaration_id(package_id: str, export_name: str) -> str:
    digest = hashlib.sha256(f"{package_id}:{export_name}".encode("utf-8")).hexdigest()[:20]
    return f"kmipkit-0049-{package_id.removeprefix('rust-')}-{digest}"


def reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    """Return a copy with exact blocked routes and justified trait exclusions."""
    result = copy.deepcopy(manifest)
    declarations = result.setdefault("publicDeclarations", [])
    exclusions = result.setdefault("approvedExclusions", [])
    capabilities = result.setdefault("capabilities", [])
    result["requirementIds"] = sorted(set(result.get("requirementIds", [])) | set(REQUIREMENTS))
    capability_by_id = {row["capabilityId"]: row for row in capabilities}
    capability = capability_by_id.get(CAPABILITY_ID)
    if capability is None:
        capability = {
            "capabilityId": CAPABILITY_ID,
            "requirementIds": REQUIREMENTS.copy(),
            "capabilityState": "blocked",
            "parityScope": "four-language",
            "candidateSpecId": PARITY_SPEC_ID,
            "blockedReason": BLOCKED_REASON,
        }
        capabilities.append(capability)
    elif capability.get("capabilityState") != "blocked" or "mappings" in capability:
        raise ReconciliationError("0049 capability must remain blocked until four-language parity is verified")

    declaration_keys = {(row.get("packageId"), row.get("exportName")) for row in declarations}
    exclusion_keys = {(row.get("packageId"), row.get("exportName")) for row in exclusions}
    missing = report.get("missingFromManifest")
    if not isinstance(missing, list):
        raise ReconciliationError("Rust export report has no missingFromManifest array")

    for observed in sorted(missing, key=lambda row: (row.get("packageId", ""), row.get("exportName", ""))):
        package_id = observed.get("packageId")
        export_name = observed.get("exportName")
        kind = observed.get("kind")
        if not isinstance(package_id, str) or not isinstance(export_name, str) or not isinstance(kind, str):
            raise ReconciliationError("Rust export report contains an incomplete declaration")
        root = _root_type(export_name, package_id)
        if root not in PUBLIC_TYPES:
            raise ReconciliationError(f"unowned Rust declaration cannot be assigned to 0049: {package_id}:{export_name}")
        key = package_id, export_name
        if key in declaration_keys or key in exclusion_keys:
            continue
        if kind == "associated-item" and export_name.startswith("<"):
            exclusions.append({
                "packageId": package_id,
                "exportName": export_name,
                "reason": TRAIT_EXCLUSION_REASON,
                "approvalSource": copy.deepcopy(APPROVAL_SOURCE),
                "evidence": TRAIT_EXCLUSION_EVIDENCE,
                "candidateSpecId": SPEC_ID,
            })
            exclusion_keys.add(key)
            continue
        declarations.append({
            "declarationId": _declaration_id(package_id, export_name),
            "packageId": package_id,
            "kind": kind,
            "exportName": export_name,
            "capabilityId": CAPABILITY_ID,
            "requirementIds": _requirements_for(export_name, package_id),
            "declarationState": "blocked",
            "parityScope": "four-language",
            "candidateSpecId": PARITY_SPEC_ID,
            "blockedReason": BLOCKED_REASON,
        })
        declaration_keys.add(key)

    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--write", action="store_true", help="update the reviewed manifest input")
    args = parser.parse_args()
    manifest_path = args.repo_root / "specification" / "api" / "public-api.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        report = json.loads(args.report.read_text(encoding="utf-8"))
        reconciled = reconcile(manifest, report)
    except (OSError, json.JSONDecodeError, ReconciliationError) as error:
        print(f"KMIPKIT-0049 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    declarations_added = len(reconciled["publicDeclarations"]) - len(manifest["publicDeclarations"])
    exclusions_added = len(reconciled["approvedExclusions"]) - len(manifest.get("approvedExclusions", []))
    action = "write" if args.write else "would write"
    print(f"Prepared reconciliation: {declarations_added:+} declarations, {exclusions_added:+} exclusions; {action} manifest input.")
    if args.write:
        content = json.dumps(reconciled, ensure_ascii=False, indent=2) + "\n"
        manifest_path.write_bytes(content.encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
