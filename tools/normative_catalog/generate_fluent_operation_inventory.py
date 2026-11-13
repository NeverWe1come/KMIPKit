"""Generate the source-backed operation inventory for the fluent API roadmap."""

from __future__ import annotations

import argparse
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from typing import Any

if __package__:
    from .generate_named_values import PINNED_VERSION_SOURCES, load_versioned_values
    from .safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from .validate import MAX_BYTES
else:
    from generate_named_values import PINNED_VERSION_SOURCES, load_versioned_values
    from safe_io import PathSecurityError, atomic_write_bytes, safe_read_bytes
    from validate import MAX_BYTES

BASE_PATH = "specification/catalog/kmip-2.1.json"
VERSIONED_PATH = "specification/catalog/kmip-versioned-values.json"
MANIFEST_PATH = "specification/api/public-api.json"
EVIDENCE_PATH = "specification/catalog/operation-matrix-evidence.json"
OUTPUT_PATH = "specification/catalog/fluent-operation-inventory.json"
MAX_SOURCE_BYTES = 64 * 1024 * 1024
V3_OPERATION_SECTIONS = {
    "Create Credential": "6.1.9",
    "Create Group": "6.1.10",
    "Create User": "6.1.13",
    "Deactivate": "6.1.14",
    "Decapsulate": "6.1.15",
    "Encapsulate": "6.1.22",
    "Obliterate": "6.1.41",
}


def _read_json(root: Path, relative_path: str, *, max_bytes: int = MAX_BYTES) -> Any:
    return json.loads(safe_read_bytes(root, relative_path, max_bytes=max_bytes))


def _v3_operations(document: dict[str, Any]) -> list[dict[str, Any]]:
    versions = document.get("versions")
    groups = versions.get("v3_0") if isinstance(versions, dict) else None
    if not isinstance(groups, list):
        raise ValueError("versioned values catalog has no KMIP 3.0 groups")
    operation_groups = [
        group for group in groups
        if isinstance(group, dict)
        and group.get("kind") == "enumeration"
        and group.get("name") == "Operation"
    ]
    if len(operation_groups) != 1:
        raise ValueError("versioned values catalog must have one KMIP 3.0 Operation group")
    values = operation_groups[0].get("values")
    if not isinstance(values, list) or len(values) != 7:
        raise ValueError("KMIP 3.0 operation delta must contain seven assigned values")
    return values


def build_inventory(
    base_catalog: dict[str, Any],
    versioned_catalog: dict[str, Any],
    manifest: dict[str, Any],
    evidence_catalog: dict[str, Any] | None = None,
) -> dict[str, Any]:
    """Join assigned operations with direction, owner clause and API scope."""
    base_rows = [
        element for element in base_catalog.get("elements", [])
        if isinstance(element, dict) and element.get("kind") == "operation"
    ]
    operation_values = {
        element.get("name"): element.get("wire_value")
        for element in base_catalog.get("elements", [])
        if isinstance(element, dict)
        and element.get("kind") == "enumeration_value"
        and "KMIPKIT-ELEM-ENUMERATION-OPERATION" in element.get("parent_element_ids", [])
    }
    # The wildcard extension allocation is not an assigned operation symbol.
    assigned_values = {value for name, value in operation_values.items() if name != "Extensions"}
    if assigned_values != {f"{value:08X}" for value in range(1, 60)}:
        raise ValueError("KMIP 2.1 operation assignment set is incomplete or reserved")
    route_keys: set[tuple[str, str, str]] = set()
    seen_values: set[str] = set()
    rows: list[dict[str, Any]] = []
    for element in base_rows:
        name = element.get("name")
        refs = element.get("source_refs")
        if not isinstance(name, str):
            raise ValueError("KMIP 2.1 operation name is missing")
        if not isinstance(refs, list) or not refs:
            raise ValueError(f"KMIP 2.1 operation {name} has no source reference")
        if element.get("direction") not in {"client_to_server", "server_to_client"}:
            raise ValueError(f"KMIP 2.1 operation {name} has invalid direction")
        wire_value = operation_values.get(name)
        if not isinstance(wire_value, str) or len(wire_value) != 8:
            raise ValueError(f"KMIP 2.1 operation {name} has no assigned wire value")
        route_key = (name, wire_value, element.get("direction"))
        if route_key in route_keys:
            raise ValueError(f"duplicate KMIP 2.1 operation route: {route_key}")
        route_keys.add(route_key)
        seen_values.add(wire_value)
        rows.append({
            "name": name,
            "version": "2.1",
            "wireValue": wire_value.upper(),
            "direction": element.get("direction"),
            "scopeState": element.get("scope_state"),
            "sourceRefs": [{"sourceId": ref.get("source_id"), "section": ref.get("section")}
                           for ref in refs],
            "featureSpec": element.get("feature_spec"),
            "payloadTables": element.get("payload_tables", []),
            "catalogImplementationRefs": element.get("implementation_refs", []),
            "catalogVerificationRefs": element.get("verification_refs", []),
        })

    manifest_operations = manifest.get("operations")
    if not isinstance(manifest_operations, list):
        raise ValueError("public API manifest has no operation inventory")
    public_by_name = {
        row.get("name"): row for row in manifest_operations
        if isinstance(row, dict) and row.get("scope") == "client_1_0"
    }
    v3_values = _v3_operations(versioned_catalog)
    if {int(value["wire_value"], 16) for value in v3_values} != set(range(60, 67)):
        raise ValueError("KMIP 3.0 operation assignment set is incomplete or reserved")
    for value in v3_values:
        name = value.get("name")
        wire_value = value.get("wire_value")
        if not isinstance(name, str) or not isinstance(wire_value, str):
            raise ValueError("malformed KMIP 3.0 operation assignment")
        section = V3_OPERATION_SECTIONS.get(name)
        if section is None:
            raise ValueError(f"KMIP 3.0 operation has no reviewed source section: {name}")
        wire_value = wire_value.removeprefix("0x").upper().zfill(8)
        if wire_value in seen_values:
            raise ValueError(f"duplicate operation assignment across versions: {wire_value}")
        seen_values.add(wire_value)
        rows.append({
            "name": name,
            "version": "3.0",
            "wireValue": wire_value,
            "direction": "client_to_server",
            "scopeState": "client_3_0_draft",
            "sourceRefs": [{"sourceId": "kmip-3.0", "section": section}],
            "featureSpec": "KMIPKIT-0071",
            "payloadTables": [],
            "catalogImplementationRefs": [],
            "catalogVerificationRefs": [],
        })

    evidence = _evidence_by_route(evidence_catalog, rows)
    for row in rows:
        detail = evidence[(row["name"], row["direction"])]
        row["versionSourceRefs"] = detail["versionSourceRefs"]
        row["earliestSupportedVersion"] = detail["versionSourceRefs"][0]["version"]
        row["typedOwners"] = detail["typedOwners"]
        row["route"] = detail["route"]
        row["testEvidence"] = {"focused": detail["testRefs"], "parity": []}
        row["serverCapabilityClaim"] = False
        public = public_by_name.get(row["name"]) if row["direction"] == "client_to_server" else None
        row["publicApiScope"] = public.get("scope") if public else None
        row["publicApiState"] = public.get("apiState") if public else None
        row["testEvidence"]["parity"] = public.get("parityEvidenceIds", []) if public else []
        implemented = row["route"]["state"] == "implemented"
        row["clientSend"] = row["route"]["kind"] == "client" and implemented
        row["listenerRoute"] = row["route"]["kind"] == "listener" and implemented

    rows.sort(key=lambda row: (row["version"], int(row["wireValue"], 16)))
    return {
        "formatVersion": 2,
        "sources": [BASE_PATH, VERSIONED_PATH, MANIFEST_PATH, EVIDENCE_PATH],
        "versionMeaning": (
            "Earliest directional availability in supported pinned source editions; "
            "route-version execution is not established by assignment or static evidence."
        ),
        "evidenceMeaning": (
            "Existing executable test references, not recorded passing results "
            "or server capability claims."
        ),
        "operations": rows,
    }


def _evidence_by_route(
    document: dict[str, Any] | None, rows: list[dict[str, Any]],
) -> dict[tuple[str, str], dict[str, Any]]:
    """Require reviewed metadata for each directional assignment without inventing owners."""
    if not isinstance(document, dict) or document.get("formatVersion") != 1:
        raise ValueError("operation matrix evidence is missing or unsupported")
    details = document.get("operations")
    if not isinstance(details, list):
        raise ValueError("operation matrix evidence has no operation rows")
    result = {}
    for detail in details:
        key = (detail["name"], detail["direction"])
        if key in result:
            raise ValueError(f"duplicate operation matrix evidence: {key}")
        refs = detail.get("versionSourceRefs")
        if not isinstance(refs, list) or not refs:
            raise ValueError(f"operation matrix evidence has no version sources: {key}")
        versions = [ref.get("version") for ref in refs]
        if (any(v not in {"2.0", "2.1", "3.0"} for v in versions)
                or versions != sorted(set(versions))):
            raise ValueError(f"operation matrix evidence has invalid version order: {key}")
        owners = detail.get("typedOwners", {})
        route = detail.get("route", {})
        if owners.get("state") not in {"available", "unverified"}:
            raise ValueError(f"operation matrix evidence has invalid owner state: {key}")
        if owners["state"] == "available" and not all(
            owners.get(role) for role in ("request", "response")
        ):
            raise ValueError(f"operation matrix evidence has missing typed owners: {key}")
        expected_kind = "client" if detail["direction"] == "client_to_server" else "listener"
        if route.get("kind") != expected_kind or route.get("state") not in {"implemented", "unverified"}:
            raise ValueError(f"operation matrix evidence has invalid route direction/state: {key}")
        if route.get("versionVerification") != "not_established":
            raise ValueError(f"operation matrix evidence cannot claim route-version execution: {key}")
        if route["state"] == "implemented" and (
            owners["state"] != "available" or not detail.get("testRefs")
        ):
            raise ValueError(f"implemented route lacks typed owners or test evidence: {key}")
        result[key] = detail
    if set(result) != {(row["name"], row["direction"]) for row in rows}:
        raise ValueError("operation matrix evidence does not match assigned directions")
    return result


class _OperationHeadings(HTMLParser):
    """Read only numbered operation headings from the pinned Word HTML editions."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.heading: str | None = None
        self.parts: list[str] = []
        self.operations: dict[str, str] = {}

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        classes = dict(attrs).get("class") or ""
        if tag in {"h1", "h2", "h3", "h4", "h5", "h6"} or (
            tag == "p" and re.fullmatch(r"MsoHeading[1-6]", classes)
        ):
            self.heading = tag
            self.parts = []

    def handle_data(self, data: str) -> None:
        if self.heading:
            self.parts.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == self.heading:
            title = " ".join("".join(self.parts).split())
            match = re.fullmatch(r"(6\.[12]\.\d+)\s+(.*)", title)
            if match:
                self.operations[match[1]] = match[2]
            self.heading = None



def _declares_owner(text: str, name: str, role: str) -> bool:
    """Recognize existing concrete models, including the reviewed create-model macro."""
    if re.search(r"pub (?:struct|enum) " + re.escape(name) + r"\b", text):
        return True
    stem = re.escape(name.removesuffix(role.title()))
    return re.search(
        r"define_create_request!\(\s*" + stem + r"Request,\s*" + stem + r"Response,",
        text,
    ) is not None


def _validate_evidence_files(root: Path, inventory: dict[str, Any]) -> None:
    headings = {}
    for source_id, (path, _) in PINNED_VERSION_SOURCES.items():
        parser = _OperationHeadings()
        raw = safe_read_bytes(root, path, max_bytes=MAX_SOURCE_BYTES)
        parser.feed(raw.decode("utf-8" if source_id == "kmip-3.0" else "cp1252"))
        headings[source_id] = parser.operations
    texts: dict[str, str] = {}

    def read(path: str) -> str:
        if path not in texts:
            texts[path] = safe_read_bytes(root, path, max_bytes=MAX_SOURCE_BYTES).decode("utf-8")
        return texts[path]

    for row in inventory["operations"]:
        prefix = "6.1." if row["direction"] == "client_to_server" else "6.2."
        available = [source_id.removeprefix("kmip-") for source_id, sections in headings.items()
                     if any(name == row["name"] and section.startswith(prefix)
                            for section, name in sections.items())]
        if row["earliestSupportedVersion"] != min(available):
            raise ValueError(f"incorrect earliest directional source version: {row['name']}")
        for ref in row["versionSourceRefs"]:
            source_id = "kmip-" + ref["version"]
            if (ref["sourceId"] != source_id or ref["path"] != PINNED_VERSION_SOURCES[source_id][0]
                    or not ref["section"].startswith(prefix)
                    or headings[source_id].get(ref["section"]) != row["name"]):
                raise ValueError(f"invalid directional version source: {row['name']}")
        owners = row["typedOwners"]
        if owners["state"] == "available":
            for role in ("request", "response"):
                owner = owners[role]
                name = owner["symbol"].split("::")[-1]
                text = read(owner["path"])
                if not _declares_owner(text, name, role):
                    raise ValueError(f"missing typed {role} owner: {row['name']}")
        route = row["route"]
        if route["state"] == "implemented":
            text = read(route["path"])
            function = route["symbol"].split("::")[-1]
            if not re.search(r"\bfn\s+" + re.escape(function) + r"\s*\(", text):
                raise ValueError(f"missing route symbol: {row['name']}")
            variant = route.get("requestVariant")
            if variant:
                variant_name = variant.split("::")[-1]
                request_name = owners["request"]["symbol"].split("::")[-1]
                if not re.search(r"\b" + re.escape(variant_name) + r"\(" + re.escape(request_name) + r"\)", text):
                    raise ValueError(f"missing typed route variant: {row['name']}")
        for ref in row["testEvidence"]["focused"]:
            path, name = ref.split("::", 1)
            if not re.search(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+" + re.escape(name) + r"\s*\(", read(path)):
                raise ValueError(f"missing executable test evidence: {ref}")
        for ref in row["testEvidence"]["parity"]:
            read(ref.split("::", 1)[0])

def render(root: Path) -> bytes:
    # Reuse the existing checksum and versioned-catalog validation boundary.
    load_versioned_values(root)
    inventory = build_inventory(
        _read_json(root, BASE_PATH),
        _read_json(root, VERSIONED_PATH, max_bytes=MAX_SOURCE_BYTES),
        _read_json(root, MANIFEST_PATH, max_bytes=MAX_SOURCE_BYTES),
        _read_json(root, EVIDENCE_PATH),
    )
    _validate_evidence_files(root, inventory)
    return (json.dumps(inventory, indent=2, ensure_ascii=False) + "\n").encode()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        root = args.repo_root.resolve(strict=True)
        rendered = render(root)
        if args.check:
            actual = safe_read_bytes(root, OUTPUT_PATH, max_bytes=MAX_SOURCE_BYTES)
            if actual != rendered:
                print(f"{OUTPUT_PATH} is stale", file=sys.stderr)
                return 1
        else:
            atomic_write_bytes(root, OUTPUT_PATH, rendered)
    except (OSError, ValueError, json.JSONDecodeError, PathSecurityError) as error:
        print(f"fluent operation inventory: {error}", file=sys.stderr)
        return 1
    if not args.check:
        print(f"wrote {OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
