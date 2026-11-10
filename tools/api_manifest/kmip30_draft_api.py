"""Validate the separately reviewed, opt-in KMIP 3.0 draft API catalog."""

from __future__ import annotations

import hashlib
import re
from pathlib import Path
from typing import Any


class DraftApiManifestError(ValueError):
    """Raised when the KMIP 3.0 draft catalog differs from reviewed scope."""


_EXPECTED_ROUTES = (
    ("CreateGroup", 0x3C, "§6.1.10, Table 280", "create_group_draft", "client_batch_item_create_group", "createGroup"),
    ("Obliterate", 0x3D, "§6.1.41, Table 374", "obliterate_draft", "client_batch_item_obliterate", "obliterate"),
    ("CreateUser", 0x3E, "§6.1.13, Table 290", "create_user_draft", "client_batch_item_create_user", "createUser"),
    ("CreateCredential", 0x3F, "§6.1.9, Table 277", "create_credential_draft", "client_batch_item_create_credential", "createCredential"),
    ("Deactivate", 0x40, "§6.1.14, Table 293", "deactivate_draft", "client_batch_item_deactivate", "deactivate"),
    ("Encapsulate", 0x41, "§6.1.22, Table 317", "encapsulate_draft", "client_batch_item_encapsulate", "encapsulate"),
    ("Decapsulate", 0x42, "§6.1.15, Table 296", "decapsulate_draft", "client_batch_item_decapsulate", "decapsulate"),
)
_EXPECTED_SOURCE_PATH = "specification/oasis/kmip-3.0/upstream/kmip-spec-v3.0.html"
_EXPECTED_SOURCE_INVENTORY = "specification/oasis/kmip-3.0/SOURCES.md"
_EXPECTED_BASE_MANIFEST = "specification/api/public-api.json"
_MOJIBAKE_MARKERS = ("\u00c2\u00a7", "\u00c3", "\u00e2\u20ac", "\u00ef\u00bf\u00bd", "\ufffd")


def validate_kmip30_draft_api(
    draft_catalog: dict[str, Any], public_manifest: dict[str, Any], root: Path
) -> None:
    """Check draft scope, pinned OASIS provenance, and all four public mappings."""
    _require(draft_catalog.get("formatVersion") == 1, "unsupported catalog format")
    _require(
        draft_catalog.get("scope") == "opt-in-kmip-3.0-working-draft-request-routes",
        "unexpected KMIP 3.0 draft catalog scope",
    )
    _require(draft_catalog.get("defaultProtocolVersion") == "2.1", "default must remain KMIP 2.1")
    _require(draft_catalog.get("profileConformanceClaim") is False, "draft catalog cannot claim profile conformance")

    base = draft_catalog.get("baseManifest")
    _require(isinstance(base, dict), "base manifest declaration is missing")
    _require(base.get("path") == _EXPECTED_BASE_MANIFEST, "draft catalog points at an unexpected base manifest")
    _require(base.get("operationScope") == "client_1_0", "draft routes must stay outside client_1_0")
    _require(base.get("expectedOperationCount") == 56, "client_1_0 operation count must remain 56")

    operations = draft_catalog.get("operations")
    _require(isinstance(operations, list), "draft operation list is missing")
    _require(len(operations) == len(_EXPECTED_ROUTES), "draft catalog must contain exactly seven operations")
    functions = {
        item.get("id"): item
        for item in public_manifest.get("functions", [])
        if isinstance(item, dict)
    }
    expected_names = set()
    for operation, expected in zip(operations, _EXPECTED_ROUTES, strict=True):
        _require(isinstance(operation, dict), "draft operation entry must be an object")
        name, code, source, rust_method, c_name, java_method = expected
        _require(
            (operation.get("operation"), operation.get("operationCode"), operation.get("source"))
            == (name, code, source),
            f"unreviewed operation identity or OASIS reference for {name}",
        )
        _require(not any(marker in source for marker in _MOJIBAKE_MARKERS), f"damaged OASIS reference for {name}")
        expected_names.add(name)
        _validate_route_mapping(operation, functions.get(c_name), name, rust_method, c_name, java_method)

    client_operations = [
        operation
        for operation in public_manifest.get("operations", [])
        if isinstance(operation, dict) and operation.get("scope") == base["operationScope"]
    ]
    _require(len(client_operations) == base["expectedOperationCount"], "client_1_0 operation inventory changed")
    client_names = {operation.get("name") for operation in client_operations}
    _require(not expected_names & client_names, "draft routes must not be counted as KMIP 2.1 operations")

    capabilities = {
        item.get("capabilityId"): item
        for item in public_manifest.get("capabilities", [])
        if isinstance(item, dict)
    }
    capability = capabilities.get("kmip30_draft_routes")
    _require(isinstance(capability, dict), "draft route capability is not declared")
    _require(capability.get("capabilityState") == "declared", "draft routes cannot be marked verified")
    _require(
        "KMIPKIT-0053-FR-018" in capability.get("requirementIds", []),
        "draft route capability lacks its specification requirement",
    )
    _validate_pinned_source(draft_catalog.get("source"), root)


def _validate_route_mapping(
    route: dict[str, Any],
    function: dict[str, Any] | None,
    operation: str,
    rust_method: str,
    c_name: str,
    java_method: str,
) -> None:
    _require(function is not None, f"public API manifest omits {operation}")
    _require(function.get("requirementIds") == ["KMIPKIT-0053-FR-018"], f"wrong requirement mapping for {operation}")

    rust = function.get("rust")
    _require(
        isinstance(rust, dict)
        and route.get("rust") == f"{rust.get('module')}::{rust.get('name')}"
        and rust.get("name") == rust_method,
        f"Rust route mapping drift for {operation}",
    )
    c = function.get("c")
    _require(
        isinstance(c, dict)
        and route.get("c") == c_name
        and c.get("symbol") == f"kmipkit_{c_name}",
        f"C route mapping drift for {operation}",
    )
    java = function.get("java")
    _require(
        isinstance(java, dict)
        and route.get("java")
        == f"{java.get('package')}.{java.get('class')}.{java.get('method')}"
        and java.get("method") == java_method,
        f"Java route mapping drift for {operation}",
    )
    python = function.get("python")
    _require(
        isinstance(python, dict)
        and route.get("python") == python.get("function")
        and python.get("cffiSymbol") == f"kmipkit_{c_name}",
        f"Python route mapping drift for {operation}",
    )


def _validate_pinned_source(source: Any, root: Path) -> None:
    _require(isinstance(source, dict), "pinned OASIS source metadata is missing")
    _require(source.get("path") == _EXPECTED_SOURCE_PATH, "catalog uses an unexpected OASIS source")
    _require(source.get("inventory") == _EXPECTED_SOURCE_INVENTORY, "catalog uses an unexpected OASIS inventory")
    digest = source.get("sha256")
    _require(isinstance(digest, str) and re.fullmatch(r"[0-9a-f]{64}", digest) is not None, "OASIS source digest is malformed")
    source_path = root / _EXPECTED_SOURCE_PATH
    inventory_path = root / _EXPECTED_SOURCE_INVENTORY
    checksum_path = root / "specification/oasis/kmip-3.0/CHECKSUMS.sha256"
    try:
        resolved_root = root.resolve(strict=True)
        for path in (source_path, inventory_path, checksum_path):
            path.resolve(strict=True).relative_to(resolved_root)
        actual_digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
        inventory = inventory_path.read_text(encoding="utf-8")
        checksums = checksum_path.read_text(encoding="utf-8")
    except (OSError, ValueError) as error:
        raise DraftApiManifestError("pinned OASIS source files are unavailable") from error
    _require(actual_digest == digest, "pinned OASIS source digest differs from the draft catalog")
    checksum_line = f"{digest}  upstream/kmip-spec-v3.0.html"
    _require(checksum_line in checksums.splitlines(), "pinned OASIS checksum inventory differs")
    _require(
        f"`upstream/kmip-spec-v3.0.html`" in inventory and f"`{digest}`" in inventory,
        "pinned OASIS source inventory differs",
    )


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise DraftApiManifestError(message)
