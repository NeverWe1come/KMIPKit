"""Deterministically reconcile KMIPKIT-0050 Rust API paths into manifest v2."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

SPEC_ID = "KMIPKIT-0050"
PARITY_SPEC_ID = "KMIPKIT-0053"
REQUIREMENTS = [f"{SPEC_ID}-FR-{number:03}" for number in range(1, 14)]
PARAMETER_REQUIREMENTS = REQUIREMENTS[:5]
AUTH_REQUIREMENTS = REQUIREMENTS[5:9]
DIRECT_REQUIREMENTS = REQUIREMENTS[9:11]

APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": "specs/050-parameters-auth-client/spec.md",
    "status": "Approved",
    "evidence": "Approved for implementation by the maintainer's direct approval of the Phase 5 plan",
}

PARAMETER_CAPABILITY = "typed_cryptographic_parameters"
PARAMETER_BLOCKED_REASON = (
    "This exact typed Cryptographic Parameters Rust route is implemented under KMIPKIT-0050. "
    "Its C, Java, and Python equivalents are assigned to KMIPKIT-0053; four-language parity "
    "remains blocked until those routes and evidence are recorded."
)
AUTH_CAPABILITY = "request_message_authentication"
AUTH_BLOCKED_REASON = (
    "This exact Request Message Authentication execution route is implemented in Rust under "
    "KMIPKIT-0050. C, Java, and Python execution equivalents are assigned to KMIPKIT-0053; "
    "four-language parity remains blocked until those routes and evidence are recorded."
)
TRAIT_EXCLUSION_REASON = (
    "This exact compiler-visible Rust trait projection is behavior of the KMIPKIT-0050 "
    "Cryptographic Parameters model or builder, not an independent KMIP capability."
)
TRAIT_EXCLUSION_EVIDENCE = (
    "KMIPKIT-0050-FR-001 through KMIPKIT-0050-FR-005; the public Cryptographic Parameters "
    "type and its exact supported routes are inventoried separately."
)

DIRECT_METHODS = {
    "add_attribute": "ADD-ATTRIBUTE",
    "adjust_attribute": "ADJUST-ATTRIBUTE",
    "decrypt": "DECRYPT",
    "delegated_login": "DELEGATED-LOGIN",
    "delete_attribute": "DELETE-ATTRIBUTE",
    "discover_versions": "DISCOVER-VERSIONS",
    "encrypt": "ENCRYPT",
    "get": "GET",
    "get_attribute_list": "GET-ATTRIBUTE-LIST",
    "get_attributes": "GET-ATTRIBUTES",
    "get_constraints": "GET-CONSTRAINTS",
    "get_usage_allocation": "GET-USAGE-ALLOCATION",
    "interop": "INTEROP",
    "locate": "LOCATE",
    "log": "LOG",
    "login": "LOGIN",
    "logout": "LOGOUT",
    "modify_attribute": "MODIFY-ATTRIBUTE",
    "rng_retrieve": "RNG-RETRIEVE",
    "rng_seed": "RNG-SEED",
    "set_attribute": "SET-ATTRIBUTE",
    "set_constraints": "SET-CONSTRAINTS",
    "set_defaults": "SET-DEFAULTS",
    "validate": "VALIDATE",
}
RUST_CLIENT_PACKAGES = {"rust-kmipkit", "rust-kmipkit-client"}
PARAMETER_PACKAGES = {"rust-kmipkit", "rust-kmipkit-protocol"}
CLIENT_PREFIXES = {
    "rust-kmipkit": "kmipkit::Client::",
    "rust-kmipkit-client": "kmipkit_client::Client::",
}
PARAMETER_PREFIXES = {
    "rust-kmipkit": "kmipkit::protocol::",
    "rust-kmipkit-protocol": "kmipkit_protocol::",
}
PARAMETER_TYPES = {"CryptographicParameters", "CryptographicParametersBuilder"}
TYPED_SETTER_TYPES = {
    "DecryptRequest",
    "DerivationParameters",
    "EncryptRequest",
    "EncryptionKeyInformation",
    "MacRequest",
    "MacSignatureKeyInformation",
    "MacVerifyRequest",
    "SignRequest",
    "SignatureVerifyRequest",
}


class ReconciliationError(ValueError):
    """Raised when a Rust route cannot be assigned without guessing."""


def _declaration_id(package_id: str, export_name: str) -> str:
    digest = hashlib.sha256(f"{package_id}:{export_name}".encode("utf-8")).hexdigest()[:20]
    return f"kmipkit-0050-{package_id.removeprefix('rust-')}-{digest}"


def _parameter_root(export_name: str, package_id: str) -> str | None:
    prefix = PARAMETER_PREFIXES.get(package_id)
    if prefix is None or not export_name.startswith(prefix):
        return None
    rest = export_name[len(prefix):]
    return rest.split("::", 1)[0] if rest else None


def _operation_mapping(
    operation: dict[str, Any], declarations_by_id: dict[str, dict[str, Any]], language: str,
) -> dict[str, str]:
    """Select the existing one-item batch operation as the language equivalent."""
    operation_id = operation["operationId"]
    method = next((
        method for method, suffix in DIRECT_METHODS.items()
        if operation_id == f"KMIPKIT-ELEM-OP-C2S-{suffix}"
    ), None)
    if method is None:
        raise ReconciliationError(f"operation has no reviewed direct method: {operation_id}")
    first, *rest = method.split("_")
    exact_names = {
        "c": f"kmipkit_client_batch_item_{method}",
        "java": "org.kmipkit.extensions.ClientBatchItem." + first + "".join(part.title() for part in rest),
        "python": f"kmipkit.extensions.client_batch_item_{method}",
    }
    candidates = [
        row for row in declarations_by_id.values()
        if row.get("capabilityId") == operation_id
        and row.get("languageMappings", {}).get(language)
    ]
    for candidate in candidates:
        mapping = candidate.get("languageMappings", {}).get(language)
        if not mapping:
            continue
        name = mapping.get("name", "")
        if name == exact_names.get(language):
            return copy.deepcopy(mapping)
    raise ReconciliationError(
        f"operation {operation.get('operationId')} has no reviewed {language} batch-item route"
    )


def _direct_declaration(
    observed: dict[str, Any],
    operation: dict[str, Any],
    package_id: str,
    export_name: str,
    declarations_by_id: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    operation_id = operation["operationId"]
    mappings = {
        "rust": {"packageId": package_id, "name": export_name},
        "c": _operation_mapping(operation, declarations_by_id, "c"),
        "java": _operation_mapping(operation, declarations_by_id, "java"),
        "python": _operation_mapping(operation, declarations_by_id, "python"),
    }
    evidence = sorted(set(operation.get("parityEvidenceIds", [])) | {
        "crates/kmipkit-client/tests/unit/direct_method_surface_tests.rs",
    })
    return {
        "declarationId": _declaration_id(package_id, export_name),
        "packageId": package_id,
        "kind": observed["kind"],
        "exportName": export_name,
        "capabilityId": operation_id,
        "requirementIds": DIRECT_REQUIREMENTS.copy(),
        "declarationState": "verified",
        "parityScope": "four-language",
        "approvalSource": copy.deepcopy(operation["approvalEvidence"]),
        "languageMappings": mappings,
        "parityEvidenceIds": evidence,
    }


def _blocked_declaration(
    observed: dict[str, Any], capability_id: str, requirements: list[str], blocked_reason: str,
) -> dict[str, Any]:
    package_id = observed["packageId"]
    export_name = observed["exportName"]
    return {
        "declarationId": _declaration_id(package_id, export_name),
        "packageId": package_id,
        "kind": observed["kind"],
        "exportName": export_name,
        "capabilityId": capability_id,
        "requirementIds": requirements.copy(),
        "declarationState": "blocked",
        "parityScope": "four-language",
        "candidateSpecId": PARITY_SPEC_ID,
        "blockedReason": blocked_reason,
    }


def _blocked_capability(capability_id: str, requirements: list[str], reason: str) -> dict[str, Any]:
    return {
        "capabilityId": capability_id,
        "requirementIds": requirements.copy(),
        "capabilityState": "blocked",
        "parityScope": "four-language",
        "candidateSpecId": PARITY_SPEC_ID,
        "blockedReason": reason,
    }


def _classify(
    observed: dict[str, Any],
    operation_by_id: dict[str, dict[str, Any]],
    declarations_by_id: dict[str, dict[str, Any]],
) -> tuple[str, dict[str, Any]]:
    """Build an exact mapped, blocked, or excluded record for one observed path."""
    package_id = observed.get("packageId")
    export_name = observed.get("exportName")
    kind = observed.get("kind")
    if not isinstance(package_id, str) or not isinstance(export_name, str) or not isinstance(kind, str):
        raise ReconciliationError("Rust export report contains an incomplete declaration")

    prefix = CLIENT_PREFIXES.get(package_id)
    if prefix and export_name.startswith(prefix):
        method = export_name[len(prefix):]
        if method.endswith("_with_options"):
            method = method.removesuffix("_with_options")
        operation_key = DIRECT_METHODS.get(method)
        if operation_key is None:
            raise ReconciliationError(f"unowned KMIPKIT-0050 Client method: {export_name}")
        operation_id = f"KMIPKIT-ELEM-OP-C2S-{operation_key}"
        operation = operation_by_id.get(operation_id)
        if operation is None or operation.get("apiState") != "verified":
            raise ReconciliationError(f"direct method has no verified operation capability: {export_name}")
        return "declaration", _direct_declaration(
            observed, operation, package_id, export_name, declarations_by_id,
        )

    if package_id in {"rust-kmipkit", "rust-kmipkit-client"} and export_name in {
        "kmipkit::ClientBatch::with_authentication",
        "kmipkit_client::ClientBatch::with_authentication",
    }:
        return "declaration", _blocked_declaration(
            observed, AUTH_CAPABILITY, AUTH_REQUIREMENTS, AUTH_BLOCKED_REASON,
        )

    if package_id in PARAMETER_PACKAGES:
        root = _parameter_root(export_name, package_id)
        if root in PARAMETER_TYPES or (root in TYPED_SETTER_TYPES and export_name.endswith("::with_typed_cryptographic_parameters")):
            return "declaration", _blocked_declaration(
                observed, PARAMETER_CAPABILITY, PARAMETER_REQUIREMENTS, PARAMETER_BLOCKED_REASON,
            )

    if package_id in PARAMETER_PACKAGES and export_name.startswith("<"):
        separator = export_name.find(" as ")
        if separator < 2:
            raise ReconciliationError(f"malformed Rust trait projection: {export_name}")
        owner = export_name[1:separator]
        if _parameter_root(owner, package_id) in PARAMETER_TYPES:
            return "exclusion", {
                "packageId": package_id,
                "exportName": export_name,
                "reason": TRAIT_EXCLUSION_REASON,
                "approvalSource": copy.deepcopy(APPROVAL_SOURCE),
                "evidence": TRAIT_EXCLUSION_EVIDENCE,
                "candidateSpecId": SPEC_ID,
            }

    raise ReconciliationError(f"unowned Rust declaration cannot be assigned to 0050: {package_id}:{export_name}")


def reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> dict[str, Any]:
    """Return a copy with exact routes, blocked capabilities, and trait exclusions."""
    result = copy.deepcopy(manifest)
    declarations = result.setdefault("publicDeclarations", [])
    exclusions = result.setdefault("approvedExclusions", [])
    capabilities = result.setdefault("capabilities", [])
    result["requirementIds"] = sorted(set(result.get("requirementIds", [])) | set(REQUIREMENTS))
    operation_by_id = {row["operationId"]: row for row in result.get("operations", [])}
    declarations_by_id = {row["declarationId"]: row for row in declarations}
    capability_by_id = {row["capabilityId"]: row for row in capabilities}

    for row in [*declarations, *exclusions]:
        source = row.get("approvalSource")
        if isinstance(source, dict) and source.get("path") == APPROVAL_SOURCE["path"]:
            operation = operation_by_id.get(row.get("capabilityId"))
            if operation is not None:
                row["approvalSource"] = copy.deepcopy(operation["approvalEvidence"])
            elif row in exclusions:
                row["approvalSource"] = copy.deepcopy(APPROVAL_SOURCE)

    for capability_id, requirements, reason in (
        (PARAMETER_CAPABILITY, PARAMETER_REQUIREMENTS, PARAMETER_BLOCKED_REASON),
        (AUTH_CAPABILITY, AUTH_REQUIREMENTS, AUTH_BLOCKED_REASON),
    ):
        existing = capability_by_id.get(capability_id)
        expected = _blocked_capability(capability_id, requirements, reason)
        if existing is None:
            capabilities.append(expected)
            capability_by_id[capability_id] = expected
        elif existing != expected:
            raise ReconciliationError(f"unexpected existing capability contract: {capability_id}")

    declaration_keys = {(row.get("packageId"), row.get("exportName")) for row in declarations}
    exclusion_keys = {(row.get("packageId"), row.get("exportName")) for row in exclusions}
    missing = report.get("missingFromManifest")
    if not isinstance(missing, list):
        raise ReconciliationError("Rust export report has no missingFromManifest array")

    for observed in sorted(missing, key=lambda row: (row.get("packageId", ""), row.get("exportName", ""))):
        package_id = observed.get("packageId")
        export_name = observed.get("exportName")
        key = package_id, export_name
        if key in declaration_keys or key in exclusion_keys:
            continue
        target, record = _classify(observed, operation_by_id, declarations_by_id)
        if target == "declaration":
            declarations.append(record)
            declaration_keys.add(key)
            declarations_by_id[record["declarationId"]] = record
        else:
            exclusions.append(record)
            exclusion_keys.add(key)

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
        print(f"KMIPKIT-0050 manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    declaration_delta = len(reconciled["publicDeclarations"]) - len(manifest["publicDeclarations"])
    exclusion_delta = len(reconciled["approvedExclusions"]) - len(manifest.get("approvedExclusions", []))
    action = "write" if args.write else "would write"
    print(
        f"Prepared reconciliation: {declaration_delta:+} declarations, "
        f"{exclusion_delta:+} exclusions; {action} manifest input."
    )
    if args.write:
        content = json.dumps(reconciled, ensure_ascii=False, indent=2) + "\n"
        manifest_path.write_bytes(content.encode("utf-8"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
