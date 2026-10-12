"""Reconcile compiler-observed Rust paths as blocked manifest records."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_MANIFEST = ROOT / "specification" / "api" / "public-api.json"
RUST_PACKAGES = {
    "rust-kmipkit-ttlv",
    "rust-kmipkit-protocol",
    "rust-kmipkit-transport",
    "rust-kmipkit-client",
    "rust-kmipkit",
}
DECLARATION_KINDS = {
    "module", "type", "variant", "field", "trait", "function",
    "associated-item", "constant", "macro",
}
OWNER_SPEC_PATTERN = re.compile(r"^KMIPKIT-[0-9]{4}$")
BLOCKED_REASON = (
    "The observed Rust declaration lacks a reviewed four-language mapping "
    "in the public API manifest."
)
UNRESOLVED_OWNER_REASON = (
    "The public Rust declaration has no resolved owner in the approved "
    "dependency inventory; assign an owner before advertising it."
)

def blocked_record(observed: dict[str, Any], manifest: dict[str, Any]) -> dict[str, Any]:
    """Build a blocked record for an observed Rust path without inventing mappings."""
    package_id = observed.get("packageId")
    export_name = observed.get("exportName")
    kind = observed.get("kind")
    if package_id not in RUST_PACKAGES or not isinstance(export_name, str) or not export_name:
        raise ValueError("invalid observed Rust package or export path")
    if kind not in DECLARATION_KINDS:
        raise ValueError(f"unsupported observed Rust declaration kind: {kind!r}")

    owner_path = _owner_path(export_name)
    owner = _find_owner(manifest, package_id, owner_path, export_name)
    candidate_spec_id = _candidate_spec_for_owner(manifest, owner)
    reason = BLOCKED_REASON if owner is not None else UNRESOLVED_OWNER_REASON
    declaration_hash = hashlib.sha256(
        f"{package_id}\0{export_name}".encode("utf-8")
    ).hexdigest()[:20]
    return {
        "declarationId": f"rust-observed-{declaration_hash}",
        "packageId": package_id,
        "kind": kind,
        "exportName": export_name,
        "capabilityId": "public_binding_surface",
        "requirementIds": ["KMIPKIT-0043-FR-021"],
        "declarationState": "blocked",
        "parityScope": "four-language",
        "candidateSpecId": candidate_spec_id,
        "blockedReason": reason,
    }


def _owner_path(export_name: str) -> str | None:
    """Return the explicit owner path from a trait-qualified declaration path."""
    if not export_name.startswith("<"):
        return None
    separator = export_name.find(" as ")
    if separator < 2:
        raise ValueError(f"malformed trait implementation export path: {export_name}")
    owner_path = export_name[1:separator]
    return owner_path or None


def _export_key(row: dict[str, Any]) -> tuple[Any, Any]:
    """Return the package/path pair used to reconcile one declaration."""
    return row.get("packageId"), row.get("exportName")


def _find_owner(manifest: dict[str, Any], package_id: str, owner_path: str | None,
                export_name: str) -> dict[str, Any] | None:
    if owner_path:
        matches = [
            row for row in manifest["publicDeclarations"]
            if row["packageId"] == package_id and row["exportName"] == owner_path
        ]
    else:
        matches = [
            row for row in manifest["publicDeclarations"]
            if row["packageId"] == package_id
            and isinstance(row.get("exportName"), str)
            and export_name.startswith(row["exportName"] + "::")
        ]
        matches.sort(key=lambda row: len(row["exportName"]), reverse=True)
        if matches:
            longest_name = len(matches[0]["exportName"])
            matches = [row for row in matches if len(row["exportName"]) == longest_name]
    if len(matches) > 1:
        raise ValueError(f"duplicate manifest owner path: {package_id}:{owner_path}")
    return matches[0] if matches else None


def _candidate_spec_for_owner(manifest: dict[str, Any],
                              owner: dict[str, Any] | None) -> str:
    if owner is None:
        return "KMIPKIT-0043"
    candidate = owner.get("candidateSpecId")
    if isinstance(candidate, str) and OWNER_SPEC_PATTERN.fullmatch(candidate):
        return candidate
    capabilities = {
        capability["capabilityId"]: capability
        for capability in manifest.get("capabilities", [])
    }
    capability = capabilities.get(owner.get("capabilityId"), {})
    for key in ("candidateSpecId", "ownerSpecId"):
        candidate = capability.get(key)
        if isinstance(candidate, str) and OWNER_SPEC_PATTERN.fullmatch(candidate):
            return candidate
    return "KMIPKIT-0043"


def reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> list[dict[str, Any]]:
    """Add exact blocked records for report paths not yet represented in the manifest."""
    if report.get("formatVersion") != 1 or report.get("rustdocFormat") != 57:
        raise ValueError("expected Rust export report format 1 and rustdoc JSON 57")
    declarations = manifest.get("publicDeclarations")
    exclusions = manifest.get("approvedExclusions")
    missing = report.get("missingFromManifest")
    if not isinstance(declarations, list) or not isinstance(exclusions, list) \
            or not isinstance(missing, list):
        raise ValueError("manifest or Rust report has an invalid declaration inventory")

    existing_keys: set[tuple[str, str]] = set()
    existing_ids: set[str] = set()
    for row in declarations:
        key = _export_key(row)
        if key in existing_keys:
            raise ValueError(f"duplicate manifest declaration: {key[0]}:{key[1]}")
        existing_keys.add(key)
        existing_ids.add(row.get("declarationId"))
    exclusion_keys = {_export_key(row) for row in exclusions}

    observed_keys: set[tuple[str, str]] = set()
    rows: list[dict[str, Any]] = []
    row_ids: set[str] = set()
    for item in missing:
        if not isinstance(item, dict):
            raise ValueError("Rust report missing entries must be objects")
        key = _export_key(item)
        if key in observed_keys:
            raise ValueError(f"duplicate observed Rust path: {key[0]}:{key[1]}")
        observed_keys.add(key)
        if key in existing_keys or key in exclusion_keys:
            raise ValueError(f"stale Rust report path is already reconciled: {key[0]}:{key[1]}")
        row = blocked_record(item, manifest)
        if row["declarationId"] in existing_ids or row["declarationId"] in row_ids:
            raise ValueError(f"Rust declaration ID collision: {row['declarationId']}")
        row_ids.add(row["declarationId"])
        rows.append(row)

    rows.sort(key=lambda row: (row["packageId"], row["exportName"], row["kind"]))
    declarations.extend(rows)
    return rows


def _write_manifest(path: Path, manifest: dict[str, Any]) -> None:
    original = path.read_bytes()
    line_ending = b"\r\n" if b"\r\n" in original else b"\n"
    serialized = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    path.write_bytes(serialized.replace("\n", line_ending.decode("ascii")).encode("utf-8"))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument(
        "--write", action="store_true",
        help="write exact blocked declarations to the canonical manifest input",
    )
    arguments = parser.parse_args(argv)
    try:
        manifest = json.loads(arguments.manifest.read_text(encoding="utf-8"))
        report = json.loads(arguments.report.read_text(encoding="utf-8"))
        additions = reconcile(manifest, report)
        if arguments.write and additions:
            _write_manifest(arguments.manifest, manifest)
        print(json.dumps({"blockedRecordsAdded": len(additions), "write": arguments.write}))
        return 0 if arguments.write or not additions else 1
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(f"Rust export reconciliation failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
