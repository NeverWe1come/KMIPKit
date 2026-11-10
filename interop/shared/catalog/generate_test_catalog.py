#!/usr/bin/env python3
"""Generate versioned executable interop cases from the normative KMIP catalog."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path


OASIS_SOURCES = {
    "2.0": (
        "specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html",
        "41346355fc966e555dbcbae68b4d73421b0da0f32fb1a86fc8e3cb5fd35bb4a2",
    ),
    "2.1": (
        "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html",
        "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf",
    ),
}


class OASISSectionParser(HTMLParser):
    """Read numbered section headings from the checked-in OASIS HTML sources."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.heading_tag = None
        self.heading_text = []
        self.sections = {}

    def handle_starttag(self, tag, _attrs):
        if re.fullmatch(r"h[1-6]", tag):
            self.heading_tag = tag
            self.heading_text = []

    def handle_data(self, data):
        if self.heading_tag:
            self.heading_text.append(data)

    def handle_endtag(self, tag):
        if self.heading_tag != tag:
            return
        text = " ".join("".join(self.heading_text).split())
        match = re.match(r"(\d+(?:\.\d+)*)(?:\s+|$)(.*)", text)
        if match:
            self.sections[match.group(1)] = match.group(2).strip()
        self.heading_tag = None
        self.heading_text = []


def oasis_section_indexes(repository_root: Path) -> dict[str, dict[str, str]]:
    indexes = {}
    for version, (relative_path, expected_digest) in OASIS_SOURCES.items():
        source_path = repository_root / relative_path
        source_bytes = source_path.read_bytes()
        actual_digest = hashlib.sha256(source_bytes).hexdigest()
        if actual_digest != expected_digest:
            raise ValueError(f"Pinned OASIS KMIP {version} source digest mismatch: {source_path}")
        parser = OASISSectionParser()
        parser.feed(source_bytes.decode("cp1252"))
        indexes[version] = parser.sections
    return indexes


def _section_for_title(index: dict[str, str], title: str, section_family: str) -> str | None:
    matches = [
        section for section, candidate in index.items()
        if section.startswith(section_family) and candidate.casefold() == title.casefold()
    ]
    if len(matches) > 1:
        raise ValueError(f"Ambiguous OASIS section title {title!r}: {matches}")
    return matches[0] if matches else None


def _source_element(test_case: dict, elements: list[dict]) -> dict | None:
    category = test_case["category"]
    requires = test_case.get("requires", {})
    target_id = requires.get("normative_target_id")
    if isinstance(target_id, str):
        return next((element for element in elements if element.get("element_id") == target_id), None)
    if category == "attributes":
        kind, name = "attribute", requires.get("attribute")
    elif category == "managed_objects":
        kind, name = "object_type", requires.get("object_type")
    elif category == "enums_and_bitmasks":
        kind = "enumeration" if "enum_type" in requires else "bitmask"
        name = requires.get("enum_type", requires.get("bitmask_type"))
    else:
        kind, name = "operation", test_case.get("operation")
    matches = [el for el in elements if el.get("kind") == kind and el.get("name") == name]
    if kind == "operation" and requires.get("workflow_type") == "server_to_client":
        return next((el for el in matches if el.get("direction") == "server_to_client"), None)
    return matches[0] if matches else None


def _element_sections(element: dict, indexes: dict[str, dict[str, str]]) -> dict[str, str]:
    refs = element.get("source_refs", [])
    source_ref = next((ref for ref in refs if ref.get("source_id") == "KMIPKIT-SRC-spec"), None)
    if source_ref is None:
        return {}
    return _section_versions(str(source_ref["section"]), indexes)


def _section_versions(section21: str, indexes: dict[str, dict[str, str]]) -> dict[str, str]:
    title = indexes["2.1"].get(section21)
    if title is None:
        return {}
    family = section21.rsplit(".", 1)[0] + "."
    # A non-dotted top-level section (for example §2) is its own family.
    if "." not in section21:
        family = section21
    sections = {"2.1": section21}
    section20 = _section_for_title(indexes["2.0"], title, family)
    if section20:
        sections["2.0"] = section20
    return sections


def add_execution_contract(test_case: dict, elements: list[dict], indexes: dict[str, dict[str, str]]) -> dict:
    """Attach a machine-readable baseline execution recipe to a catalog case."""
    category = test_case["category"]
    req = test_case.get("requires", {})
    source_element = _source_element(test_case, elements)
    section_by_version = _element_sections(source_element, indexes) if source_element else {}
    if not section_by_version and re.fullmatch(r"\d+(?:\.\d+)*", str(test_case.get("oasis_section", ""))):
        section_by_version = _section_versions(str(test_case["oasis_section"]), indexes)
    if category == "version_boundaries":
        versions = ["2.0"]
    elif req.get("target_protocol_versions"):
        versions = req["target_protocol_versions"]
    else:
        versions = [version for version in ("2.0", "2.1") if version in section_by_version]
    if not versions:
        raise ValueError(f"No pinned OASIS applicability source for {test_case['case_id']}")

    if category == "enums_and_bitmasks":
        mode = "local_model"
        steps = ["construct_known_value", "construct_unknown_value", "ttlv_roundtrip"]
        assertions = ["known_value_preserved", "unknown_value_preserved"]
        mutation = "read_only"
    elif category == "normative_inventory":
        mode = "local_model"
        target_kind = req["target_kind"]
        if target_kind in {"enumeration_value", "bitmask_value", "tag", "tag_range"}:
            allocation = req.get("allocation", "assigned")
            steps = ["construct_exact_or_boundary_wire_value", "encode_ttlv", "decode_ttlv"]
            assertions = [
                "wire_value_preserved" if allocation == "assigned" else "allocation_behavior_is_explicit",
                "normalized_tree_matches_typed_route",
            ]
        elif target_kind == "data_type":
            steps = ["construct_valid_boundary_values", "encode_ttlv", "decode_ttlv", "reject_malformed_encoding"]
            assertions = ["all_values_roundtrip", "malformed_value_rejected_without_panic"]
        else:
            steps = ["construct_required_and_optional_branches", "encode_ttlv", "decode_ttlv"]
            assertions = ["field_shape_matches_normative_source", "normalized_tree_matches_typed_route"]
        mutation = "read_only"
    elif category == "version_boundaries":
        mode = "local_model"
        steps = ["construct_request_with_unsupported_version", "invoke_public_route", "count_transport_calls"]
        assertions = ["version_rejected_before_send", "transport_call_count_is_zero"]
        mutation = "read_only"
    elif category == "negative_and_boundary" and req.get("expected_status"):
        mode = "server_exchange"
        steps = ["prepare_isolated_fixture", "execute_operation", "verify_response"]
        assertions = ["result_status_matches_expected", "delivery_state_recorded"]
        mutation = "mutating"
    elif category in {"attributes", "managed_objects", "object_lifecycle", "cryptography_and_rng", "multipart_crypto", "batch_workflows", "asynchronous_workflows"}:
        mode = "server_exchange"
        steps = ["prepare_isolated_fixture", "execute_operation", "verify_response", "cleanup_fixture"]
        assertions = ["response_matches_operation_contract", "state_effect_matches_expectation"]
        mutation = "mutating"
    else:
        mode = "server_exchange"
        steps = ["execute_operation", "verify_response"]
        assertions = ["response_matches_operation_contract"]
        mutation = "read_only"

    ref_versions = ("2.1",) if category == "version_boundaries" else versions
    test_case["normative_refs"] = [
        {"document": f"OASIS KMIP {version} Specification", "section": section_by_version[version]}
        for version in ref_versions
    ]
    if category == "version_boundaries":
        # The operation is defined in KMIP 2.1 and intentionally has no KMIP 2.0 entry.
        test_case["normative_refs"].append({"document": "OASIS KMIP 2.0 Specification", "section": "6.1"})
    test_case["execution"] = {
        "mode": mode,
        "protocol_versions": versions,
        "steps": steps,
        "assertions": assertions,
        "mutation": mutation,
        "status": "planned",
    }
    return test_case


NORMATIVE_TARGET_KINDS = {
    "operation", "object_type", "attribute", "tag", "enumeration_value",
    "bitmask_value", "data_type", "message_field", "structure_member",
    "operation_structure", "object_structure", "attribute_structure", "credential",
}


def normative_inventory(elements: list[dict], indexes: dict[str, dict[str, str]], repository_root: Path) -> tuple[list[dict], list[dict]]:
    """Return individual source-backed targets and their executable local cases."""
    source_elements = [
        element for element in elements
        if element.get("kind") in NORMATIVE_TARGET_KINDS
        and (
            element.get("scope_state") in {"client_1_0", "client_1_1"}
            or (element.get("kind") == "tag" and element.get("allocation") == "reserved")
        )
    ]
    source_elements.sort(key=lambda element: element["element_id"])
    by_id = {element["element_id"]: element for element in elements}

    versioned_values = json.loads(
        (repository_root / "specification/catalog/kmip-versioned-values.json").read_text(encoding="utf-8")
    )
    kmip21_values = {
        (group["name"], value["name"], value["wire_value"].lower().removeprefix("0x"))
        for group in versioned_values["versions"]["v2_1"]
        for value in group["values"]
    }
    delta_path = repository_root / "interop/shared/catalog/kmip-2.1-enum-deltas.json"
    deltas = json.loads(delta_path.read_text(encoding="utf-8"))
    delta_source = deltas.get("source", {})
    expected_delta_source, expected_delta_hash = OASIS_SOURCES["2.1"]
    if deltas.get("format_version") != 1 or delta_source.get("path") != expected_delta_source or delta_source.get("sha256") != expected_delta_hash:
        raise ValueError("KMIP 2.1 enum-delta inventory does not match the pinned OASIS source")
    source_bytes = (repository_root / expected_delta_source).read_bytes()
    if hashlib.sha256(source_bytes).hexdigest() != expected_delta_hash:
        raise ValueError("Pinned KMIP 2.1 source digest mismatch for enum-delta inventory")
    for group in deltas.get("deltas", []):
        for value in group.get("values", []):
            kmip21_values.add((
                group["enumeration"],
                value["name"],
                value["wire_value"].lower().removeprefix("0x"),
            ))
    targets: list[dict] = []
    local_cases: list[dict] = []
    for element in source_elements:
        source_refs = [
            {"section": ref["section"]}
            for ref in element.get("source_refs", [])
            if ref.get("source_id") == "KMIPKIT-SRC-spec"
        ]
        parent_ids = element.get("parent_element_ids", [])
        parent = by_id.get(parent_ids[0]) if parent_ids else None
        parent_name = parent.get("name") if parent else None
        wire_value = str(element.get("wire_value", "")).lower().removeprefix("0x")
        is_kmip21_value = (
            element["kind"] == "enumeration_value"
            and (parent_name, element["name"], wire_value) in kmip21_values
        )
        section_versions = _element_sections(element, indexes)
        versions = (
            ["2.1"] if is_kmip21_value
            else [version for version in ("2.0", "2.1") if version in section_versions]
        )
        if not versions:
            raise ValueError(f"No pinned OASIS section for normative target {element['element_id']}")

        target = {
            "target_id": element["element_id"],
            "kind": element["kind"],
            "name": element["name"],
            "protocol_versions": versions,
            "source_refs": source_refs,
        }
        for key in ("wire_value", "allocation", "source_encoding", "source_requiredness"):
            if key in element:
                target[key] = element[key]
        if parent_ids:
            target["parent_element_ids"] = parent_ids
        targets.append(target)

        kind = element["kind"]
        if kind in {"operation", "object_type", "attribute"}:
            continue
        case_id = f"test-norm-{kind.replace('_', '-')}-{element['element_id'].removeprefix('KMIPKIT-ELEM-').lower()}"
        case = {
            "case_id": case_id,
            "category": "normative_inventory",
            "operation": parent_name if kind in {"enumeration_value", "bitmask_value"} else "TTLV Roundtrip",
            "description": f"Exercise {element['name']} ({kind}) using its exact normative identity and TTLV representation.",
            "requires": {
                "normative_target_id": element["element_id"],
                "target_kind": kind,
                "target_protocol_versions": versions,
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-norm-{element['element_id'].lower()}",
                "setup_steps": [],
                "target_step": f"Construct and roundtrip {element['name']}",
                "teardown_steps": [],
            },
        }
        for source_key in ("wire_value", "allocation", "source_encoding", "source_requiredness"):
            if source_key in element:
                case["requires"][source_key] = element[source_key]
        local_cases.append(case)

    canonical = json.loads((repository_root / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
    for tag_range in canonical.get("tag_ranges", []):
        section_refs = [ref for ref in tag_range.get("source_refs", []) if ref.get("source_id") == "KMIPKIT-SRC-spec"]
        section = section_refs[0]["section"] if section_refs else None
        if section is None:
            raise ValueError(f"No pinned OASIS section for tag range {tag_range['range_id']}")
        target = {
            "target_id": tag_range["range_id"],
            "kind": "tag_range",
            "name": tag_range["value_range"],
            "protocol_versions": [version for version in ("2.0", "2.1") if version in _section_versions(section, indexes)],
            "source_refs": [{"section": section}],
            "allocation": tag_range["allocation"],
            "wire_value": tag_range["value_range"],
        }
        targets.append(target)
        local_cases.append({
            "case_id": f"test-norm-tag-range-{tag_range['range_id'].lower()}",
            "category": "normative_inventory",
            "operation": "TTLV Roundtrip",
            "oasis_section": section,
            "description": f"Exercise the {tag_range['allocation']} KMIP Tag range at its boundaries and adjacent values.",
            "requires": {
                "normative_target_id": tag_range["range_id"],
                "target_kind": "tag_range",
                "target_protocol_versions": target["protocol_versions"],
                "allocation": tag_range["allocation"],
                "wire_value": tag_range["value_range"],
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-norm-{tag_range['range_id'].lower()}",
                "setup_steps": [],
                "target_step": f"Construct {tag_range['allocation']} Tag range boundary values",
                "teardown_steps": [],
            },
        })

    return targets, local_cases


def generate_catalog(normative_catalog_path: Path) -> dict:
    with open(normative_catalog_path, "r", encoding="utf-8") as f:  # NOSONAR
        catalog = json.load(f)

    elements = catalog.get("elements", [])
    repository_root = normative_catalog_path.resolve().parents[2]
    section_indexes = oasis_section_indexes(repository_root)

    # 1. Collect in-scope client-to-server operations.
    c2s_ops = [
        el for el in elements
        if el.get("kind") == "operation"
        and el.get("direction") == "client_to_server"
        and el.get("scope_state") in {"client_1_0", "client_1_1"}
    ]
    c2s_ops.sort(key=lambda x: x["name"])
    operation_sections = [
        (operation, _element_sections(operation, section_indexes))
        for operation in c2s_ops
    ]
    kmip_21_only_operations = {
        operation["name"]: sections["2.1"]
        for operation, sections in operation_sections
        if "2.1" in sections and "2.0" not in sections
    }
    s2c_ops = [
        el for el in elements
        if el.get("kind") == "operation"
        and el.get("direction") == "server_to_client"
    ]
    s2c_ops.sort(key=lambda x: x["name"])

    # 2. Collect 9 managed object types
    managed_objects = [
        el for el in elements
        if el.get("kind") == "object_type"
    ]
    managed_objects.sort(key=lambda x: x["name"])

    # 3. Collect 63 attributes
    attributes = [
        el for el in elements
        if el.get("kind") == "attribute"
    ]
    attributes.sort(key=lambda x: x["name"])

    # 4. Collect 64 enumerations and 3 bitmasks
    enumerations = [
        el for el in elements
        if el.get("kind") == "enumeration"
    ]
    enumerations.sort(key=lambda x: x["name"])
    bitmasks = [
        el for el in elements
        if el.get("kind") == "bitmask"
    ]
    bitmasks.sort(key=lambda x: x["name"])

    test_cases = []

    # Category A: Protocol, discovery, and session operations
    session_ops = [
        "Discover Versions", "Query", "Ping", "Interop", "Log",
        "Login", "Logout", "Delegated Login"
    ]
    for op in c2s_ops:
        name = op["name"]
        if name in session_ops:
            case_id = f"test-op-{name.lower().replace(' ', '-')}"
            sec = op.get("source_refs", [{}])[0].get("section", "6.1")
            test_cases.append({
                "case_id": case_id,
                "category": "protocol_and_session",
                "operation": name,
                "oasis_section": sec,
                "description": f"Execute {name} operation and verify typed response and raw TTLV preservation.",
                "requires": {
                    "operation": name
                },
                "isolation_recipe": {
                    "unique_name_prefix": f"kmipkit-{name.lower().replace(' ', '-')}",
                    "setup_steps": [],
                    "target_step": f"Execute {name}",
                    "teardown_steps": [],
                    "normalized_fields": ["TimeStamp", "UniqueIdentifier"]
                }
            })

    # Category B: Object Lifecycle & General operations
    crypto_ops = {"Encrypt", "Decrypt", "Sign", "Signature Verify", "MAC", "MAC Verify", "Hash", "RNG Retrieve", "RNG Seed"}
    for op in c2s_ops:
        name = op["name"]
        if name in session_ops:
            continue
        case_id = f"test-op-{name.lower().replace(' ', '-')}"
        sec = op.get("source_refs", [{}])[0].get("section", "6.1")
        is_async = name in {"Cancel", "Poll", "Process", "Query Asynchronous Requests"}

        setup = []
        teardown = []
        if name not in {"Create", "Create Key Pair", "Create Split Key", "Register", "Import", "Locate"}:
            setup = ["Create unique SymmetricKey"]
            teardown = ["Revoke SymmetricKey", "Destroy SymmetricKey"]
        elif name in {"Create", "Create Key Pair", "Create Split Key", "Register", "Import"}:
            teardown = ["Destroy object"]

        test_cases.append({
            "case_id": case_id,
            "category": "object_lifecycle",
            "operation": name,
            "oasis_section": sec,
            "description": f"Execute {name} operation ensuring state isolation and equivalence across API levels.",
            "requires": {
                "operation": name,
                "requires_async": is_async
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-{name.lower().replace(' ', '-')}",
                "setup_steps": setup,
                "target_step": f"Execute {name}",
                "teardown_steps": teardown,
                "normalized_fields": ["TimeStamp", "UniqueIdentifier", "InitialDate", "ActivationDate", "LastChangeDate", "DestroyDate"]
            }
        })

    # Category A2: server-to-client operations and endpoint role inversion.
    for op in s2c_ops:
        name = op["name"]
        section = op.get("source_refs", [{}])[0].get("section", "6.2")
        test_cases.append({
            "case_id": f"test-s2c-{name.lower().replace(' ', '-')}",
            "category": "server_initiated_operations",
            "operation": name,
            "oasis_section": section,
            "description": f"Accept, validate, and respond to server-initiated {name} requests.",
            "requires": {"operation": name, "workflow_type": "server_to_client"},
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-s2c-{name.lower().replace(' ', '-')}",
                "setup_steps": ["Start client listener"],
                "target_step": f"Deliver server-initiated {name}",
                "teardown_steps": ["Stop client listener"],
                "normalized_fields": ["ProtocolVersion", "Operation", "ResultStatus", "ResultReason"],
            },
        })

    # Category C: 9 Managed Object creation/registration variants
    for obj in managed_objects:
        obj_name = obj["name"]
        case_id = f"test-obj-{obj_name.lower().replace(' ', '-')}"
        object_creation_operation = {
            "Private Key": "Create Key Pair",
            "Public Key": "Create Key Pair",
            "Split Key": "Create Split Key",
        }
        test_cases.append({
            "case_id": case_id,
            "category": "managed_objects",
            "operation": object_creation_operation.get(
                obj_name,
                "Register" if obj_name in {"Certificate", "Certificate Request", "PGP Key", "Opaque Object", "Secret Data"} else "Create",
            ),
            "oasis_section": "§2",
            "description": f"Validate {obj_name} object construction, encoding roundtrip, and cleanup.",
            "requires": {
                "object_type": obj_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-obj-{obj_name.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Provision {obj_name}",
                "teardown_steps": ["Destroy object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp"]
            }
        })

    # Category D: 63 Attributes
    for attr in attributes:
        attr_name = attr["name"]
        case_id = f"test-attr-{attr_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "attributes",
            "operation": "Get Attributes",
            "oasis_section": "§4",
            "description": f"Verify read/write semantics and policies for attribute '{attr_name}'.",
            "requires": {
                "attribute": attr_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-attr-{attr_name.lower().replace(' ', '-')}",
                "setup_steps": ["Create an object permitted by the attribute policy"],
                "target_step": f"GetAttributes for {attr_name}",
                "teardown_steps": ["Destroy object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp"]
            }
        })

    # Category E: Cryptographic and RNG operations
    crypto_profiles = [
        ("Encrypt", "AES", 256, "CBC"),
        ("Decrypt", "AES", 256, "CBC"),
        ("Sign", "RSA", 2048, "PKCS1_V1_5"),
        ("Signature Verify", "RSA", 2048, "PKCS1_V1_5"),
        ("Sign", "ECDSA", 256, "P_256"),
        ("Signature Verify", "ECDSA", 256, "P_256"),
        ("MAC", "HMAC_SHA256", 256, None),
        ("MAC Verify", "HMAC_SHA256", 256, None),
        ("Hash", "SHA_256", 0, None),
        ("RNG Retrieve", "RNG", 32, None),
        ("RNG Seed", "RNG", 32, None),
    ]
    for op_name, alg, key_len, mode in crypto_profiles:
        suffix = f"{alg.lower()}-{key_len}" + (f"-{mode.lower()}" if mode else "")
        case_id = f"test-crypto-{op_name.lower().replace(' ', '-')}-{suffix}"
        test_cases.append({
            "case_id": case_id,
            "category": "cryptography_and_rng",
            "operation": op_name,
            "oasis_section": "§6.1",
            "description": f"Execute cryptographic operation {op_name} with {alg} {key_len}-bit.",
            "requires": {
                "operation": op_name,
                "algorithm": alg,
                "key_length": key_len
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-crypto-{suffix}",
                "setup_steps": ["Create test key"] if op_name not in {"Hash", "RNG Retrieve", "RNG Seed"} else [],
                "target_step": f"Execute {op_name}",
                "teardown_steps": ["Destroy test key"] if op_name not in {"Hash", "RNG Retrieve", "RNG Seed"} else [],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "IVCounterNonce"]
            }
        })

    # Category F: 64 Enumerations and 3 Bitmasks
    for enum_el in enumerations:
        enum_name = enum_el["name"]
        case_id = f"test-enum-{enum_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "enums_and_bitmasks",
            "operation": "Query",
            "oasis_section": "§9.1",
            "description": f"Verify roundtrip and custom vendor extension preservation for enumeration {enum_name}.",
            "requires": {
                "operation": "Query",
                "enum_type": enum_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-enum-{enum_name.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Validate {enum_name} serialization",
                "teardown_steps": [],
                "normalized_fields": ["TimeStamp"]
            }
        })

    for mask_el in bitmasks:
        mask_name = mask_el["name"]
        case_id = f"test-bitmask-{mask_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "enums_and_bitmasks",
            "operation": "Query",
            "oasis_section": "§9.1",
            "description": f"Verify bitwise composition and unknown bit preservation for bitmask {mask_name}.",
            "requires": {
                "operation": "Query",
                "bitmask_type": mask_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-mask-{mask_name.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Validate {mask_name} bits",
                "teardown_steps": [],
                "normalized_fields": ["TimeStamp"]
            }
        })

    # Category G: Multi-item Batch Workflows with IDPlaceholder
    batch_workflows = [
        {
            "id": "test-batch-create-activate-destroy",
            "desc": "Single-message batch workflow: Create -> Activate (IDPlaceholder) -> GetAttributes -> Destroy",
            "recipe_prefix": "kmipkit-batch-cad",
            "steps": ["Create AES-256 SymmetricKey", "Activate with IDPlaceholder", "GetAttributes with IDPlaceholder", "Destroy with IDPlaceholder"]
        },
        {
            "id": "test-batch-register-encrypt-decrypt-destroy",
            "desc": "Single-message batch workflow: Register -> Activate -> Encrypt -> Decrypt -> Destroy",
            "recipe_prefix": "kmipkit-batch-crypto",
            "steps": ["Register AES-128 SymmetricKey", "Activate", "Encrypt", "Decrypt", "Destroy"]
        }
    ]
    for bw in batch_workflows:
        test_cases.append({
            "case_id": bw["id"],
            "category": "batch_workflows",
            "operation": "Create",
            "oasis_section": "§5.1",
            "description": bw["desc"],
            "requires": {
                "operation": "Create",
                "workflow_type": "batch_id_placeholder"
            },
            "isolation_recipe": {
                "unique_name_prefix": bw["recipe_prefix"],
                "setup_steps": [],
                "target_step": "Execute Batch Request",
                "teardown_steps": ["Destroy object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "InitialDate", "ActivationDate", "DestroyDate"]
            }
        })

    # Category H: Multipart Streaming Cryptographic Operations
    multipart_crypto = [
        ("test-multipart-encrypt-aes-cbc", "Encrypt", "AES", 128, "CBC", "Init -> Update(64B) -> Update(64B) -> Final Encrypt streaming with CorrelationValue"),
        ("test-multipart-decrypt-aes-cbc", "Decrypt", "AES", 128, "CBC", "Init -> Update(64B) -> Update(64B) -> Final Decrypt streaming with CorrelationValue"),
        ("test-multipart-sign-rsa", "Sign", "RSA", 2048, "PKCS1_V1_5", "Init -> Update -> Final Sign streaming with CorrelationValue"),
        ("test-multipart-hash-sha256", "Hash", "SHA_256", 0, None, "Init -> Update -> Final Hash streaming with CorrelationValue")
    ]
    for cid, op, alg, klen, mode, desc in multipart_crypto:
        test_cases.append({
            "case_id": cid,
            "category": "multipart_crypto",
            "operation": op,
            "oasis_section": "§6.1",
            "description": desc,
            "requires": {
                "operation": op,
                "algorithm": alg,
                "key_length": klen,
                "workflow_type": "multipart_streaming"
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-stream-{alg.lower()}",
                "setup_steps": ["Create test key"] if op != "Hash" else [],
                "target_step": f"Execute Multipart {op}",
                "teardown_steps": ["Destroy test key"] if op != "Hash" else [],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "CorrelationValue", "IVCounterNonce"]
            }
        })

    # Category I: Asynchronous Lifecycle Workflows
    test_cases.append({
        "case_id": "test-async-lifecycle-poll",
        "category": "asynchronous_workflows",
        "operation": "Create",
        "oasis_section": "§6.1.18",
        "description": "Send asynchronous request with AsynchronousIndicator=true, receive Pending status, query and poll until completion.",
        "requires": {
            "operation": "Create",
            "requires_async": True,
            "workflow_type": "async_poll"
        },
        "isolation_recipe": {
            "unique_name_prefix": "kmipkit-async-poll",
            "setup_steps": [],
            "target_step": "Submit Asynchronous Create and Poll",
            "teardown_steps": ["Destroy object"],
            "normalized_fields": ["UniqueIdentifier", "TimeStamp", "AsynchronousCorrelationValue"]
        }
    })

    # Category J: Negative and Boundary Tests
    negative_tests = [
        {
            "id": "test-neg-encrypt-preactive-key",
            "op": "Encrypt",
            "desc": "Verify server rejects Encrypt on Pre-Active SymmetricKey with CryptographicFailure or InvalidField.",
            "exp_status": "OperationFailed",
            "expected_result_reasons": [7, 10],
        },
        {
            "id": "test-neg-modify-readonly-attribute",
            "op": "Modify Attribute",
            "desc": "Verify server rejects mutation of read-only server-generated attribute (InitialDate / UniqueIdentifier).",
            "exp_status": "OperationFailed",
        },
        {
            "id": "test-neg-destroy-active-without-revoke",
            "op": "Destroy",
            "desc": "Verify server enforces KMIP §3 lifecycle rules when attempting Destroy on Active object without prior Revocation.",
            "exp_status": "OperationFailed",
            "expected_result_reasons": [67],
        }
    ]
    for nt in negative_tests:
        test_cases.append({
            "case_id": nt["id"],
            "category": "negative_and_boundary",
            "operation": nt["op"],
            "oasis_section": "§3",
            "description": nt["desc"],
            "requires": {
                "operation": nt["op"],
                "expected_status": nt["exp_status"]
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-neg-{nt['id']}",
                "setup_steps": ["Create test object in initial state"],
                "target_step": f"Attempt illegal {nt['op']}",
                "teardown_steps": ["Revoke and Destroy test object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp"]
            }
        })
        if nt.get("expected_result_reasons"):
            test_cases[-1]["requires"]["expected_result_reasons"] = nt["expected_result_reasons"]

    # KMIP 2.1-only operations must be rejected locally when the effective
    # request version is 2.0. These cases do not contact a server.
    for operation in sorted(kmip_21_only_operations):
        test_cases.append({
            "case_id": f"test-v20-reject-{operation.lower().replace(' ', '-')}",
            "category": "version_boundaries",
            "operation": operation,
            "oasis_section": kmip_21_only_operations[operation],
            "description": f"Reject KMIP 2.1-only {operation} before sending under KMIP 2.0.",
            "requires": {"operation": operation, "expected_status": "NotSent"},
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-v20-{operation.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Attempt {operation} under KMIP 2.0",
                "teardown_steps": [],
                "normalized_fields": ["RequestDeliveryState"],
            },
        })

    normative_targets, normative_cases = normative_inventory(elements, section_indexes, repository_root)
    test_cases.extend(normative_cases)
    for target in normative_targets:
        kind = target["kind"]
        if kind not in {"operation", "object_type", "attribute"}:
            continue
        if kind == "operation":
            element = next(item for item in elements if item["element_id"] == target["target_id"])
            if element.get("direction") == "server_to_client":
                candidates = [case for case in test_cases if case["category"] == "server_initiated_operations" and case["operation"] == target["name"]]
            else:
                candidates = [case for case in test_cases if case["category"] in {"protocol_and_session", "object_lifecycle"} and case["operation"] == target["name"]]
        elif kind == "object_type":
            candidates = [case for case in test_cases if case["category"] == "managed_objects" and case["requires"].get("object_type") == target["name"]]
        else:
            candidates = [case for case in test_cases if case["category"] == "attributes" and case["requires"].get("attribute") == target["name"]]
        if len(candidates) != 1:
            raise ValueError(f"Expected exactly one executable case for normative target {target['target_id']}; found {len(candidates)}")
        candidates[0]["requires"]["normative_target_id"] = target["target_id"]
        candidates[0]["requires"]["target_protocol_versions"] = target["protocol_versions"]

    test_cases = [add_execution_contract(case, elements, section_indexes) for case in test_cases]

    catalog_output = {
        "version": "2.0.0",
        "catalog_source": "specification/catalog/kmip-2.1.json",
        "target_versions": ["2.0", "2.1"],
        "capability_policy": "probe_all_applicable",
        "execution_policy": {
            "requires_dedicated_test_environment": True,
            "mutating_cases_require_explicit_enablement": True,
            "never_retry_requests": True,
        },
        "statistics": {
            "operations_covered": len(c2s_ops),
            "server_to_client_operations_covered": len(s2c_ops),
            "managed_objects_covered": len(managed_objects),
            "attributes_covered": len(attributes),
            "enumerations_covered": len(enumerations),
            "bitmasks_covered": len(bitmasks),
            "normative_targets": len(normative_targets),
            "total_cases": len(test_cases)
        },
        "normative_targets": normative_targets,
        "test_cases": test_cases
    }

    return catalog_output


def main():
    parser = argparse.ArgumentParser(description="Generate versioned KMIP 2.0/2.1 interoperability catalog")
    parser.add_argument("--write", action="store_true", help="Write catalog to destination")
    parser.add_argument("--check", action="store_true", help="Verify catalog is up-to-date")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    normative_path = root / "specification" / "catalog" / "kmip-2.1.json"
    output_path = root / "interop" / "shared" / "catalog" / "kmipkit-1.0-test-catalog.json"

    generated = generate_catalog(normative_path)
    content = json.dumps(generated, indent=2, ensure_ascii=False) + "\n"

    if args.check:
        if not output_path.exists():
            print(f"Error: Catalog does not exist at {output_path}", file=sys.stderr)
            sys.exit(1)
        with open(output_path, "r", encoding="utf-8") as f:  # NOSONAR
            existing = f.read()
        if existing != content:
            print("Error: Catalog is out of date. Run with --write to regenerate.", file=sys.stderr)
            sys.exit(1)
        print("Catalog check passed: pinned KMIP 2.0/2.1 section inventories match")
    elif args.write:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:  # NOSONAR
            f.write(content)
        print(f"Wrote {generated['statistics']['total_cases']} test cases covering "
              f"{generated['statistics']['operations_covered']} operations, "
              f"{generated['statistics']['managed_objects_covered']} objects, "
              f"{generated['statistics']['attributes_covered']} attributes, and "
              f"{generated['statistics']['enumerations_covered']} enumerations to {output_path}")
    else:
        print(content)


if __name__ == "__main__":
    main()
