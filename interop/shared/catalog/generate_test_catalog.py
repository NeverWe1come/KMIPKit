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


def _payload_presence_contract(requiredness: str, description: str) -> tuple[list[str], list[dict]]:
    """Translate pinned REQUIRED cells and conditional SHALL prose into presence branches."""
    normalized = re.sub(r"\s+", " ", requiredness).strip().casefold()
    branches: list[str] = ["present"]
    presence: list[dict] = []

    def add(condition: str, value: str) -> None:
        branch = {"condition": condition, "presence": value}
        if branch not in presence:
            presence.append(branch)

    if normalized.startswith("yes for single-part"):
        if "unless digested data is supplied" in normalized:
            add("single-part and Digested Data is absent", "required")
            add("single-part and Digested Data is supplied", "optional")
            add("multi-part", "optional")
            branches.extend(["condition_true_and_false", "required_when_condition_true", "optional_when_condition_false"])
        else:
            add("single-part", "required")
            add("multi-part", "optional")
            branches.extend(["condition_true_and_false", "required_when_condition_true", "optional_when_condition_false"])
    elif normalized.startswith("yes, unless a failure"):
        add("no failure", "required")
        add("failure", "optional")
        branches.extend(["condition_true_and_false", "required_when_condition_true", "optional_when_condition_false"])
    elif normalized.startswith("if and only if the key object is wrapped"):
        add("key object is wrapped", "required")
        add("key object is not wrapped", "prohibited")
        branches.extend(["condition_true_and_false", "required_when_condition_true", "prohibited_when_condition_false"])
    elif normalized.startswith("yes"):
        branches.append("missing_rejected")
    elif normalized.startswith("no"):
        branches.append("omitted")
    else:
        raise ValueError(f"Unrecognized pinned operation payload requiredness: {requiredness!r}")

    if "repeat" in normalized or "repeat" in description.casefold():
        branches.append("occurrence_count_boundaries")

    condition_patterns = (
        (r"\bshall be returned if\s+([^.;]+)", "required"),
        (r"\b(?:required|shall be present|must be present)\s+(?:if|when)\s+([^.;]+)", "required"),
        (r"\bshould be specified\s+(?:if|when)\s+([^.;]+)", "recommended"),
        (r"\bshall not be (?:present|specified)\s+(?:if|when|for)\s+([^.;]+)", "prohibited"),
    )
    for pattern, when_true in condition_patterns:
        for match in re.finditer(pattern, description, re.IGNORECASE):
            condition = re.sub(r"\s+", " ", match.group(1)).strip().rstrip(",")
            if condition.casefold().endswith(" then"):
                condition = condition[:-5].rstrip()
            add(condition, when_true)
            false_condition = (
                condition[:-len("is requested")] + "is not requested"
                if condition.casefold().endswith("is requested")
                else f"{condition} is false"
            )
            add(false_condition, "optional")
            branches.extend(["condition_true_and_false", "required_when_condition_true" if when_true == "required" else f"{when_true}_when_condition_true"])
            if when_true == "prohibited":
                branches.append("prohibited_when_condition_true")
            else:
                branches.append("optional_when_condition_false")

    return list(dict.fromkeys(branches)), presence


def add_execution_contract(test_case: dict, elements: list[dict], indexes: dict[str, dict[str, str]]) -> dict:
    """Attach a machine-readable baseline execution recipe to a catalog case."""
    category = test_case["category"]
    req = test_case.get("requires", {})
    protocol_scenario = req.get("protocol_scenario")
    source_element = None if protocol_scenario else _source_element(test_case, elements)
    section_by_version = _element_sections(source_element, indexes) if source_element else {}
    if not section_by_version and re.fullmatch(r"\d+(?:\.\d+)*", str(test_case.get("oasis_section", ""))):
        requested_versions = req.get("target_protocol_versions", [])
        if len(requested_versions) == 1:
            section_by_version = {requested_versions[0]: str(test_case["oasis_section"])}
        else:
            section_by_version = _section_versions(str(test_case["oasis_section"]), indexes)
    if protocol_scenario:
        versions = protocol_scenario["protocol_versions"]
    elif category == "version_boundaries":
        versions = ["2.0"]
    elif req.get("target_protocol_versions"):
        versions = req["target_protocol_versions"]
    else:
        versions = [version for version in ("2.0", "2.1") if version in section_by_version]
    if not versions:
        raise ValueError(f"No pinned OASIS applicability source for {test_case['case_id']}")

    if category == "attributes":
        mode = "server_exchange"
        steps = ["prepare_attribute_compatible_object", "construct_each_value_type_variant", "read_back_attribute", "exercise_policy_permitted_and_prohibited_mutations", "verify_object_state_and_cleanup"]
        assertions = ["attribute_value_matches_normative_type", "readback_matches_request", "policy_branches_match_oasis", "object_state_is_preserved_on_rejection"]
        mutation = "mutating"
    elif category == "enums_and_bitmasks":
        mode = "local_model"
        if req.get("exhaustive_bitmask_combinations"):
            steps = ["enumerate_every_assigned_flag_combination", "construct_typed_bitmask", "encode_ttlv", "decode_ttlv"]
            assertions = ["every_assigned_combination_roundtrips", "typed_and_generic_values_match"]
        else:
            steps = ["construct_known_value", "construct_unknown_value", "ttlv_roundtrip"]
            assertions = ["known_value_preserved", "unknown_value_preserved"]
        mutation = "read_only"
    elif category == "normative_inventory":
        mode = "local_model"
        target_kind = req["target_kind"]
        if target_kind in {"enumeration_value", "bitmask_value", "referenced_enum_value", "tag", "tag_range"}:
            allocation = req.get("allocation", "assigned")
            steps = ["construct_exact_or_boundary_wire_value", "encode_ttlv", "decode_ttlv"]
            assertions = [
                "wire_value_preserved" if allocation == "assigned" else "allocation_behavior_is_explicit",
                "normalized_tree_matches_typed_route",
            ]
        elif target_kind == "operation_payload_field":
            steps = ["construct_each_declared_field_branch", "encode_ttlv", "decode_ttlv", "reject_prohibited_branch"]
            assertions = ["every_branch_class_exercised", "field_shape_matches_normative_source"]
            if "value_type_alternatives" in req.get("branch_classes", []):
                steps.append("exercise_each_declared_value_type_variant")
                assertions.append("each_declared_value_type_variant_exercised")
        elif target_kind == "operation_payload_value_variant":
            steps = ["construct_field_with_exact_declared_ttlv_type", "encode_ttlv", "decode_ttlv"]
            assertions = ["field_ttlv_type_matches_normative_variant", "normalized_tree_preserves_type_and_value"]
        elif target_kind == "data_type":
            steps = ["exercise_declared_boundary_branches", "construct_valid_boundary_values", "encode_ttlv", "decode_ttlv", "reject_malformed_encoding"]
            assertions = ["every_declared_boundary_branch_exercised", "all_values_roundtrip", "malformed_value_rejected_without_panic"]
        else:
            steps = ["construct_required_and_optional_branches", "encode_ttlv", "decode_ttlv"]
            assertions = ["field_shape_matches_normative_source", "normalized_tree_matches_typed_route"]
        mutation = "read_only"
    elif category == "version_boundaries":
        mode = "local_model"
        steps = ["construct_request_with_unsupported_version", "invoke_public_route", "count_transport_calls"]
        assertions = ["version_rejected_before_send", "transport_call_count_is_zero"]
        mutation = "read_only"
    elif protocol_scenario:
        mode = protocol_scenario["mode"]
        steps = ["execute_protocol_scenario", *protocol_scenario["steps"]]
        assertions = protocol_scenario["assertions"]
        mutation = protocol_scenario["mutation"]
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
    test_case["normative_refs"] = [] if protocol_scenario else [
        {"document": f"OASIS KMIP {version} Specification", "section": section_by_version[version]}
        for version in ref_versions
    ]
    if protocol_scenario:
        for reference in protocol_scenario["normative_refs"]:
            if reference["document"] in {f"OASIS KMIP {version} Specification" for version in versions}:
                if reference not in test_case["normative_refs"]:
                    test_case["normative_refs"].append(reference)
    elif req.get("target_kind") == "operation_payload_field" and req.get("source_table") not in {None, "unlabelled-message-payload"}:
        section = str(test_case.get("oasis_section", "")).removeprefix("§")
        for version in ref_versions:
            reference = {"document": f"OASIS KMIP {version} Specification", "section": section, "table": req["source_table"]}
            if reference not in test_case["normative_refs"]:
                test_case["normative_refs"].append(reference)
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


def _operation_payload_targets(elements: list[dict], indexes: dict[str, dict[str, str]], repository_root: Path) -> tuple[list[dict], list[dict]]:
    """Extract every non-header row from each normative operation payload table."""
    # Reuse the repository's checked-in OASIS table parser; this remains an
    # offline build step and the source digest is verified before parsing.
    sys.path.insert(0, str(repository_root))
    try:
        from tools.normative_catalog.attribute_policy_source import _SourceParser
    finally:
        if sys.path and sys.path[0] == str(repository_root):
            sys.path.pop(0)

    tables_by_version = {}
    for version, (relative_path, expected_digest) in OASIS_SOURCES.items():
        if version not in {"2.0", "2.1"}:
            continue
        source = (repository_root / relative_path).read_bytes()
        if hashlib.sha256(source).hexdigest() != expected_digest:
            raise ValueError(f"Pinned KMIP {version} source digest mismatch while reading payload tables")
        parser = _SourceParser()
        parser.feed(source.decode("cp1252"))
        tables_by_version[version] = parser.tables

    targets: list[dict] = []
    cases: list[dict] = []
    ttlv_type_names = ("Date Time Extended", "Long Integer", "Big Integer", "Text String", "Byte String",
                       "Structure", "Integer", "Enumeration", "Boolean", "Date Time", "Interval")
    for operation in elements:
        if operation.get("kind") != "operation" or operation.get("direction") not in {"client_to_server", "server_to_client"}:
            continue
        section_versions = _element_sections(operation, indexes)
        for version in ("2.0", "2.1"):
            section = section_versions.get(version)
            if section is None:
                continue
            payloads_by_role = {
                payload.get("role"): str(payload.get("table_number", ""))
                for payload in operation.get("payload_tables", [])
                if payload.get("role") in {"request", "response"}
            }
            for role in ("request", "response"):
                canonical_table = payloads_by_role.get(role, "")
                selected = []
                for table in tables_by_version[version]:
                    if not str(table.get("heading", "")).startswith(f"{section} "):
                        continue
                    captions = table.get("captions", [])
                    match = (
                        any(re.search(rf"\bTable\s+{re.escape(canonical_table)}\s*:", caption) for caption in captions)
                        if version == "2.1" and canonical_table
                        else any(
                            operation["name"].casefold() in caption.casefold()
                            and role.title() in caption
                            and "Payload" in caption
                            for caption in captions
                        )
                    )
                    if (
                        not match
                        and operation.get("direction") == "server_to_client"
                        and table.get("rows")
                        and table["rows"][0]
                        and (
                            table["rows"][0][0].casefold() == f"{role} payload"
                            or (role == "request" and table["rows"][0][0].casefold() == "message payload")
                        )
                    ):
                        match = True
                    if match:
                        selected.append(table)
                if not selected:
                    continue
                if len(selected) != 1:
                    raise ValueError(
                        f"Expected one KMIP {version} {operation['name']} {role} payload table in §{section}; found {len(selected)}"
                    )
                table = selected[0]
                rows = table.get("rows", [])
                header_index = next(
                    (index for index, row in enumerate(rows) if len(row) >= 2 and row[0].casefold() == "item" and row[1].casefold() == "required"),
                    None,
                )
                if header_index is None:
                    # Empty payload tables are valid and have no Item/Required header.
                    continue
                caption = next((value for value in table.get("captions", []) if "Payload" in value), "")
                table_match = re.search(r"\bTable\s+(\d+)\s*:", caption)
                table_number = table_match.group(1) if table_match else canonical_table
                for ordinal, row in enumerate(rows[header_index + 1 :], start=1):
                    if len(row) < 2 or not row[0].strip():
                        continue
                    field_name, requiredness = row[0].strip(), row[1].strip()
                    description = row[2].strip() if len(row) > 2 else ""
                    branch_classes, presence_branches = _payload_presence_contract(requiredness, description)
                    suffix = re.sub(r"[^A-Za-z0-9]+", "-", operation["name"]).strip("-").upper()
                    direction_label = "SERVER" if operation["direction"] == "server_to_client" else "CLIENT"
                    target_id = f"KMIPKIT-PAYLOAD-{direction_label}-{suffix}-{role.upper()}-{version.replace('.', '')}-{ordinal:03d}"
                    target_source_ref = {"section": section}
                    if table_number:
                        target_source_ref["table"] = table_number
                    source_table = table_number or "unlabelled-message-payload"
                    target = {
                        "target_id": target_id,
                        "kind": "operation_payload_field",
                        "name": field_name,
                        "protocol_versions": [version],
                        "source_refs": [target_source_ref],
                        "operation_id": operation["element_id"],
                        "operation": operation["name"],
                        "direction": operation["direction"],
                        "role": role,
                        "row_index": ordinal,
                        "requiredness": requiredness,
                        "description": description,
                        "branch_classes": branch_classes,
                    }
                    if presence_branches:
                        target["presence_branches"] = presence_branches
                    case_id = f"test-norm-payload-{direction_label.lower()}-{suffix.lower()}-{role}-{version.replace('.', '')}-{ordinal:03d}"
                    targets.append(target)
                    cases.append({
                        "case_id": case_id,
                        "category": "normative_inventory",
                        "operation": operation["name"],
                        "oasis_section": section,
                        "description": f"Construct and roundtrip {role} field {field_name} for {operation['name']} from KMIP {version} Table {table_number}.",
                        "requires": {
                            "normative_target_id": target_id,
                            "target_kind": "operation_payload_field",
                            "target_protocol_versions": [version],
                            "operation_id": operation["element_id"],
                            "payload_role": role,
                            "field_name": field_name,
                            "requiredness": requiredness,
                            "branch_classes": branch_classes,
                            "source_table": source_table,
                        },
                        "isolation_recipe": {
                            "unique_name_prefix": f"kmipkit-payload-{suffix.lower()}",
                            "setup_steps": [],
                            "target_step": f"Construct {field_name} {requiredness} variant",
                            "teardown_steps": [],
                        },
                    })
                    if presence_branches:
                        cases[-1]["requires"]["presence_branches"] = presence_branches
                    type_variants = []
                    for type_name in ttlv_type_names:
                        type_pattern = re.escape(type_name).replace(r"\ ", r"[\s-]+")
                        if re.search(rf"\b(?:a|an)\s+{type_pattern}\b", description, re.IGNORECASE):
                            type_variants.append(type_name)
                    type_variants = list(dict.fromkeys(type_variants))
                    if type_variants:
                        target["value_type_alternatives"] = type_variants
                        target["branch_classes"].append("value_type_alternatives")
                    for value_type in type_variants:
                        value_suffix = re.sub(r"[^A-Za-z0-9]+", "-", value_type).strip("-").upper()
                        variant_id = f"{target_id}-TYPE-{value_suffix}"
                        variant_target = {
                            "target_id": variant_id,
                            "kind": "operation_payload_value_variant",
                            "name": f"{field_name} ({value_type})",
                            "protocol_versions": [version],
                            "source_refs": [target_source_ref],
                            "operation_id": operation["element_id"],
                            "operation": operation["name"],
                            "direction": operation["direction"],
                            "role": role,
                            "field_name": field_name,
                            "value_type": value_type,
                            "parent_target_id": target_id,
                        }
                        targets.append(variant_target)
                        cases.append({
                            "case_id": f"{case_id}-type-{value_suffix.lower()}",
                            "category": "normative_inventory",
                            "operation": operation["name"],
                            "oasis_section": section,
                            "description": f"Construct {field_name} as a {value_type} in the {operation['name']} {role} payload for KMIP {version}.",
                            "requires": {
                                "normative_target_id": variant_id,
                                "target_kind": "operation_payload_value_variant",
                                "target_protocol_versions": [version],
                                "operation_id": operation["element_id"],
                                "payload_role": role,
                                "field_name": field_name,
                                "value_type": value_type,
                                "parent_target_id": target_id,
                                "source_table": source_table,
                            },
                            "isolation_recipe": {
                                "unique_name_prefix": f"kmipkit-payload-{suffix.lower()}",
                                "setup_steps": [],
                                "target_step": f"Construct and roundtrip {field_name} as {value_type}",
                                "teardown_steps": [],
                            },
                        })
    return targets, cases


PKCS11_HEADERS = {
    "pkcs11.h": "260bbef88360ddbc8f7f668622c688ed7e6dab939e1eb2fdd6392abb571637e5",
    "pkcs11f.h": "c52bdac9e30f982cb5ad57db946a8ff6076efea744a90d684835d7269babab9b",
    "pkcs11t.h": "1f6763a72dc33a97d964d3d4b860870e1b4d4bf76b8705bdee26078caf4994cc",
}

TTLV_BOUNDARY_CLASSES = {
    "Structure": ["length_multiple_of_8", "nested_structure_roundtrip", "child_order_preserved", "declared_length_matches_children", "inconsistent_nested_length_rejected"],
    "Integer": ["signed_minimum_and_maximum", "negative_zero_positive", "exact_item_length_4", "invalid_item_length_rejected", "four_byte_padding_consumed"],
    "Long Integer": ["signed_minimum_and_maximum", "negative_zero_positive", "exact_item_length_8", "invalid_item_length_rejected"],
    "Big Integer": ["positive_and_negative", "minimum_sign_extension", "length_multiple_of_8", "non_multiple_length_rejected", "multiword_roundtrip", "alternate_sign_extension_accepted"],
    "Enumeration": ["assigned_reserved_extension_and_unknown_values", "exact_item_length_4", "invalid_item_length_rejected"],
    "Boolean": ["false_and_true", "non_boolean_value_rejected", "exact_item_length_8", "invalid_item_length_rejected"],
    "Text String": ["empty_ascii_and_multibyte_utf8", "invalid_utf8_rejected", "length_boundaries_0_1_7_8_9", "minimal_alignment_padding"],
    "Byte String": ["empty_and_arbitrary_binary", "length_boundaries_0_1_7_8_9", "minimal_alignment_padding"],
    "Date Time": ["epoch_and_adjacent_seconds", "eight_byte_signed_encoding", "exact_item_length_8", "invalid_item_length_rejected"],
    "Interval": ["unsigned_zero_and_maximum", "exact_item_length_4", "invalid_item_length_rejected"],
    "Date Time Extended": ["microsecond_precision", "eight_byte_signed_encoding", "exact_item_length_8", "invalid_item_length_rejected"],
}


def _pkcs11_referenced_targets(repository_root: Path) -> tuple[list[dict], list[dict]]:
    """Inventory the PKCS #11 definitions incorporated by KMIP §§11.38–11.39."""
    header_dir = repository_root / "specification/oasis/pkcs11-3.0/upstream/include"
    contents = {}
    for name, expected_digest in PKCS11_HEADERS.items():
        source_path = header_dir / name
        source_bytes = source_path.read_bytes()
        actual_digest = hashlib.sha256(source_bytes).hexdigest()
        if actual_digest != expected_digest:
            raise ValueError(f"Pinned OASIS PKCS #11 source digest mismatch: {source_path}")
        contents[name] = source_bytes.decode("ascii")

    functions = re.findall(
        r"^CK_PKCS11_FUNCTION_INFO\((C_[A-Za-z0-9_]+)\)", contents["pkcs11f.h"], re.MULTILINE
    )
    if not functions or len(functions) != len(set(functions)):
        raise ValueError("PKCS #11 function inventory is empty or contains duplicate ordered members")
    return_codes = re.findall(
        r"^#define\s+(CKR_[A-Za-z0-9_]+)\s+(0x[0-9A-Fa-f]+)UL\b",
        contents["pkcs11t.h"],
        re.MULTILINE,
    )
    if not return_codes or len(return_codes) != len({name for name, _ in return_codes}):
        raise ValueError("PKCS #11 return-code inventory is empty or contains duplicate names")

    targets: list[dict] = []
    cases: list[dict] = []
    for family, entries, kmip_section, artifact in (
        ("PKCS#11 Function", functions, "11.38", "pkcs11f.h"),
        ("PKCS#11 Return Code", return_codes, "11.39", "pkcs11t.h"),
    ):
        for ordinal, entry in enumerate(entries, start=1):
            if family == "PKCS#11 Function":
                name, wire_value = entry, f"{ordinal:08X}"
                allocation = "assigned"
                target_suffix = f"FUNCTION-{ordinal:03d}"
            else:
                name, raw_value = entry
                wire_value = f"{int(raw_value, 16):08X}"
                allocation = "extension" if name == "CKR_VENDOR_DEFINED" else "assigned"
                target_suffix = "RETURN-" + re.sub(r"[^A-Za-z0-9]+", "-", name.removeprefix("CKR_")).strip("-")
            target_id = f"KMIPKIT-PKCS11-{target_suffix}"
            refs = [
                {"document": "OASIS KMIP 2.0 Specification", "section": kmip_section},
                {"document": "OASIS KMIP 2.1 Specification", "section": kmip_section},
                {
                    "document": "OASIS PKCS #11 3.0",
                    "artifact": f"upstream/include/{artifact}",
                    "sha256": PKCS11_HEADERS[artifact],
                },
            ]
            targets.append({
                "target_id": target_id,
                "kind": "referenced_enum_value",
                "parent_name": family,
                "name": name,
                "wire_value": wire_value,
                "allocation": allocation,
                "protocol_versions": ["2.0", "2.1"],
                "source_refs": refs,
            })
            cases.append({
                "case_id": f"test-norm-{target_id.removeprefix('KMIPKIT-').lower()}",
                "category": "normative_inventory",
                "operation": "TTLV Roundtrip",
                "oasis_section": kmip_section,
                "description": f"Construct and roundtrip referenced {family} value {name} ({wire_value}).",
                "requires": {
                    "normative_target_id": target_id,
                    "target_kind": "referenced_enum_value",
                    "target_protocol_versions": ["2.0", "2.1"],
                    "parent_name": family,
                    "wire_value": wire_value,
                    "allocation": allocation,
                    "external_artifact": f"upstream/include/{artifact}",
                    "external_sha256": PKCS11_HEADERS[artifact],
                },
                "isolation_recipe": {
                    "unique_name_prefix": f"kmipkit-norm-{target_id.lower()}",
                    "setup_steps": [],
                    "target_step": f"Construct and roundtrip {name}",
                    "teardown_steps": [],
                },
            })
    return targets, cases


def _attribute_policy(element: dict) -> dict:
    """Retain the complete source policy and make its required test branches explicit."""
    initial = element.get("source_initially_set_by")
    modifiable = element.get("source_modifiable_by_client")
    deletable = element.get("source_deletable_by_client")
    always_required = element.get("source_always_required")
    multiple = element.get("source_multiple_instances_permitted")
    branches = ["construct_and_roundtrip_each_value_type" if element.get("source_encoding") else "value_type_defined_by_source_child_fields", "read_back_attribute"]
    if initial:
        branches.append("initial_creation_source_" + re.sub(r"[^a-z0-9]+", "_", initial.casefold()).strip("_"))
    for label, policy in (("client_modification", modifiable), ("client_deletion", deletable)):
        if policy is None:
            branches.append(label + "_policy_unspecified")
        elif policy.casefold() == "yes":
            branches.append(label + "_permitted")
        elif policy.casefold() == "no" or policy.casefold().startswith("no,"):
            branches.append(label + "_prohibited")
        elif policy.casefold().startswith("yes"):
            branches.append(label + "_conditional")
        else:
            branches.append(label + "_policy_unspecified")
    if always_required == "Yes":
        branches.append("always_required")
    elif always_required == "No":
        branches.append("optional")
    else:
        branches.append("requiredness_unspecified")
    if multiple == "Yes":
        branches.append("multiple_instances_permitted")
    elif multiple == "No":
        branches.append("single_instance_only")
    else:
        branches.append("multiplicity_unspecified")
    conditional_rules = [rule.get("source_text", "") for rule in element.get("source_conditional_rules", [])]
    operation_restrictions = [rule.get("source_text", "") for rule in element.get("source_operation_restrictions", [])]
    if conditional_rules:
        branches.append("source_conditional_rules_exercised")
    if operation_restrictions:
        branches.append("source_operation_restrictions_exercised")
    return {
        "value_type": element.get("source_encoding"),
        "initially_set_by": initial,
        "modifiable_by_client": modifiable,
        "deletable_by_client": deletable,
        "always_required": always_required,
        "multiple_instances_permitted": multiple,
        "policy_table": element.get("source_policy_table"),
        "conditional_rules": conditional_rules,
        "operation_restrictions": operation_restrictions,
        "branch_classes": branches,
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
        if element["kind"] == "attribute":
            target["attribute_policy"] = _attribute_policy(element)
        if element["kind"] == "data_type":
            target["branch_classes"] = TTLV_BOUNDARY_CLASSES[element["name"]]
            for version in versions:
                document = f"OASIS KMIP {version} Specification"
                target["source_refs"].extend([
                    {"document": document, "section": "10.1.2"},
                    {"document": document, "section": "10.1.3", "table": "389" if version == "2.0" else "427"},
                    {"document": document, "section": "10.1.5"},
                ])
                if element["name"] == "Date Time Extended":
                    target["source_refs"].append({"document": document, "section": "11.1"})
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
        if element["kind"] == "data_type":
            case["requires"]["branch_classes"] = TTLV_BOUNDARY_CLASSES[element["name"]]
            case["requires"]["boundary_vector_file"] = "interop/shared/vectors/ttlv-boundaries.json"
        local_cases.append(case)

    payload_targets, payload_cases = _operation_payload_targets(elements, indexes, repository_root)
    targets.extend(payload_targets)
    local_cases.extend(payload_cases)

    referenced_targets, referenced_cases = _pkcs11_referenced_targets(repository_root)
    targets.extend(referenced_targets)
    local_cases.extend(referenced_cases)

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


def _protocol_scenario_cases(
    normative_targets: list[dict], indexes: dict[str, dict[str, str]],
) -> list[dict]:
    """Build explicit state/branch scenarios for batch and async KMIP behavior."""
    targets_by_id = {target["target_id"]: target for target in normative_targets}

    def find_target(kind: str, name: str, *, section: str | None = None,
                    operation: str | None = None, role: str | None = None,
                    parent_name: str | None = None) -> str:
        matches = []
        for target in normative_targets:
            if target["kind"] != kind or target["name"] != name:
                continue
            if section and not any(ref.get("section") == section for ref in target["source_refs"]):
                continue
            if operation and target.get("operation") != operation:
                continue
            if role and target.get("role") != role:
                continue
            if parent_name and target.get("parent_name") != parent_name:
                continue
            matches.append(target)
        if len(matches) != 1:
            raise ValueError(f"Expected one {kind} target for {name!r} ({section}, {operation}, {role}); found {len(matches)}")
        return matches[0]["target_id"]

    def operation(name: str) -> str:
        return find_target("operation", name)

    def field(name: str, section: str) -> str:
        return find_target("message_field", name, section=section)

    def enum_value(parent_name: str, name: str) -> str:
        parent_suffix = re.sub(r"[^A-Z0-9]+", "-", parent_name.upper()).strip("-")
        parent_id = f"KMIPKIT-ELEM-ENUMERATION-{parent_suffix}"
        matches = [
            target["target_id"] for target in normative_targets
            if target["kind"] == "enumeration_value" and target["name"] == name
            and parent_id in target.get("parent_element_ids", [])
        ]
        if len(matches) != 1:
            raise ValueError(f"Expected one {parent_name} value {name!r}; found {len(matches)}")
        return matches[0]

    def payload(operation_name: str, role: str, name: str) -> list[str]:
        matches = [
            target["target_id"] for target in normative_targets
            if target["kind"] == "operation_payload_field" and target["name"] == name
            and target.get("operation") == operation_name and target.get("role") == role
        ]
        if not matches:
            raise ValueError(f"No {role} payload target for {operation_name} {name!r}")
        return matches

    cases = []

    def add(
        case_id: str, category: str, operations: list[str], target_ids: list[str], *,
        parameters: dict, expected: dict, steps: list[str], assertions: list[str],
        sections: list[str], mode: str = "server_exchange", mutation: str = "mutating",
        versions: list[str] | None = None, setup: list[str] | None = None,
    ) -> None:
        target_ids = list(dict.fromkeys([*target_ids, *(operation(name) for name in operations)]))
        target_versions = {
            version for target_id in target_ids
            for version in targets_by_id[target_id]["protocol_versions"]
        }
        selected_versions = versions or [version for version in ("2.0", "2.1") if version in target_versions]
        if not selected_versions:
            raise ValueError(f"No common KMIP version for protocol scenario {case_id}")
        target_ids = [
            target_id for target_id in target_ids
            if set(selected_versions).intersection(targets_by_id[target_id]["protocol_versions"])
        ]
        for target_id in target_ids:
            if target_id not in targets_by_id:
                raise ValueError(f"Unknown normative target {target_id} in {case_id}")
            if not set(selected_versions).intersection(targets_by_id[target_id]["protocol_versions"]):
                raise ValueError(f"Target {target_id} is not applicable to {case_id}")

        refs = []
        for version in selected_versions:
            for section in sections:
                if section not in indexes[version]:
                    raise ValueError(f"Pinned KMIP {version} has no §{section} reference for {case_id}")
                refs.append({"document": f"OASIS KMIP {version} Specification", "section": section})
            for target_id in target_ids:
                target = targets_by_id[target_id]
                for source_ref in target.get("source_refs", []):
                    section = source_ref.get("section")
                    if not section:
                        continue
                    document = source_ref.get("document")
                    if document:
                        if document != f"OASIS KMIP {version} Specification":
                            continue
                        versioned_section = section
                    else:
                        versioned_sections = _section_versions(section, indexes)
                        if version not in versioned_sections:
                            continue
                        versioned_section = versioned_sections[version]
                        document = f"OASIS KMIP {version} Specification"
                    reference = {"document": document, "section": versioned_section}
                    if source_ref.get("table"):
                        reference["table"] = source_ref["table"]
                    if reference not in refs:
                        refs.append(reference)

        scenario = {
            "scenario_id": case_id,
            "target_ids": target_ids,
            "operations": operations,
            "protocol_versions": selected_versions,
            "parameters": parameters,
            "expected": expected,
            "mode": mode,
            "mutation": mutation,
            "steps": steps,
            "assertions": assertions,
            "normative_refs": refs,
        }
        cases.append({
            "case_id": case_id,
            "category": category,
            "operation": operations[0],
            "description": f"Exercise the normative {category.replace('_', ' ')} branch {case_id}.",
            "requires": {
                "operation": operations[0],
                "protocol_scenario": scenario,
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-{case_id}",
                "setup_steps": setup or [],
                "target_step": "Execute and verify the protocol scenario",
                "teardown_steps": ["Verify fixture state and clean up"] if mutation == "mutating" else [],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "UniqueBatchItemID", "AsynchronousCorrelationValue"],
            },
        })

    batch_count = field("Batch Count", "8.2")
    continuation = field("Batch Error Continuation Option", "8.2")
    order_option = field("Batch Order Option", "8.2")
    request_item = field("Batch Item", "8.3")
    response_item = field("Batch Item", "8.6")
    request_item_id = field("Unique Batch Item ID", "8.3")
    response_item_id = field("Unique Batch Item ID", "8.6")
    stop = enum_value("Batch Error Continuation Option", "Stop")
    continue_after_error = enum_value("Batch Error Continuation Option", "Continue")
    undo = enum_value("Batch Error Continuation Option", "Undo")
    batch_base = ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "9.5", "9.6", "9.7", "9.8", "9.21"]
    get = operation("Get")
    create = operation("Create")
    activate = operation("Activate")
    destroy = operation("Destroy")
    get_attribute_list = operation("Get Attribute List")
    ping = operation("Ping")
    add("test-batch-single-item-default-continuation", "batch_workflows", ["Get"],
        [batch_count, continuation, request_item, response_item, get, stop],
        parameters={"batch_count": 1, "batch_error_continuation_option": "omitted", "items": ["Get(existing)"], "server_default": "Stop"},
        expected={"request_and_response_item_count": 1, "effective_continuation": "Stop"},
        steps=["construct_batch_without_continuation_option", "execute_single_item_batch", "decode_response"],
        assertions=["batch_count_equals_request_items", "response_count_equals_batch_count", "single_item_option_is_omitted", "omitted_option_defaults_to_stop"],
        sections=batch_base, mutation="read_only")
    add("test-batch-stop-on-first-failure", "batch_workflows", ["Get"],
        [batch_count, continuation, request_item, response_item, get, stop],
        parameters={"batch_count": 2, "batch_error_continuation_option": "Stop", "items": ["Get(missing)", "Get(existing)"]},
        expected={"first": "OperationFailed", "second": "OperationUndone", "response_count": 2},
        steps=["send_two_items_with_first_failing", "inspect_each_result_status", "verify_second_item_effect"],
        assertions=["first_failure_is_reported", "later_item_is_not_executed", "all_request_items_have_correlated_results"],
        sections=batch_base, mutation="read_only")
    add("test-batch-continue-after-failure", "batch_workflows", ["Get", "Get Attribute List"],
        [batch_count, continuation, request_item, response_item, get, get_attribute_list, continue_after_error],
        parameters={"batch_count": 2, "batch_error_continuation_option": "Continue", "items": ["Get(missing)", "Get Attribute List"]},
        expected={"first": "OperationFailed", "second": "Success", "response_count": 2},
        steps=["send_failing_and_independent_items", "inspect_each_result_status", "verify_later_item_effect"],
        assertions=["first_failure_is_reported", "later_item_is_executed", "all_request_items_have_correlated_results"],
        sections=batch_base, mutation="read_only")
    add("test-batch-undo-rolls-back-prior-success", "batch_workflows", ["Create", "Get"],
        [batch_count, continuation, request_item, response_item, create, get, undo],
        parameters={"batch_count": 2, "batch_error_continuation_option": "Undo", "items": ["Create(unique-key)", "Get(missing)"], "failure_index": 1},
        expected={"failure": "OperationFailed", "prior_success": "OperationUndone", "created_object_after_batch": "absent"},
        steps=["create_unique_object_in_batch", "cause_later_item_failure", "locate_created_identifier", "verify_rollback_and_cleanup"],
        assertions=["failed_item_status_is_reported", "earlier_success_is_undone", "no_created_object_remains"],
        sections=batch_base, setup=["Create a uniquely named object fixture"])
    add("test-batch-order-required-placeholder-resolution", "batch_workflows", ["Create", "Activate"],
        [batch_count, order_option, request_item, response_item, create, activate],
        parameters={"batch_count": 2, "batch_order_option": True, "items": ["Create(unique-key)", "Activate(IDPlaceholder from item 1)"], "placeholder_direction": "earlier item to later item"},
        expected={"execution_order": [1, 2], "second_item_identifier": "first_item_result"},
        steps=["create_preceding_object", "reference_its_identifier_with_placeholder", "inspect_ordered_responses", "verify_persisted_state"],
        assertions=["batch_order_is_preserved", "placeholder_resolves_from_prior_item", "response_item_identity_matches_request"],
        sections=batch_base, setup=["Create a uniquely named object fixture"])
    add("test-batch-order-false-correlates-response-items", "batch_workflows", ["Get", "Get Attribute List"],
        [batch_count, order_option, request_item, response_item, request_item_id, response_item_id, get, get_attribute_list],
        parameters={"batch_count": 2, "batch_order_option": False, "items": [{"operation": "Get", "id": "item-a"}, {"operation": "Get Attribute List", "id": "item-b"}], "response_order": "arbitrary"},
        expected={"responses": "matched by Unique Batch Item ID", "positional_order_required": False},
        steps=["send_independent_items_with_unique_ids", "receive_responses_in_any_permitted_order", "match_each_response_by_id"],
        assertions=["request_response_ids_match", "each_result_matches_its_operation", "no_positional_order_assumption"],
        sections=batch_base, mutation="read_only")
    add("test-batch-order-omitted-defaults-true", "batch_workflows", ["Create", "Activate"],
        [batch_count, order_option, request_item, response_item, create, activate],
        parameters={"batch_count": 2, "batch_order_option": "omitted", "items": ["Create(unique-key)", "Activate(IDPlaceholder from item 1)"], "server_default": True},
        expected={"execution_order": [1, 2], "effective_batch_order_option": True},
        steps=["omit_batch_order_option", "execute_dependent_items", "inspect_response_sequence"],
        assertions=["omitted_order_option_defaults_to_true", "dependent_placeholder_is_resolved_in_order"],
        sections=batch_base, setup=["Create a uniquely named object fixture"])
    add("test-batch-count-matches-items", "batch_workflows", ["Get", "Get Attribute List"],
        [batch_count, request_item, response_item, get, get_attribute_list],
        parameters={"batch_count": 2, "request_item_count": 2},
        expected={"response_batch_count": 2, "response_item_count": 2},
        steps=["encode_batch_count_and_items", "decode_response_batch_count_and_items"],
        assertions=["request_batch_count_equals_items", "response_batch_count_equals_results", "item_results_correspond_one_to_one"],
        sections=batch_base, mutation="read_only")
    add("test-batch-unique-item-id-response-correlation", "batch_workflows", ["Get", "Get Attribute List"],
        [request_item_id, response_item_id, request_item, response_item, get, get_attribute_list],
        parameters={"items": [{"operation": "Get", "id": "item-a"}, {"operation": "Get Attribute List", "id": "item-b"}]},
        expected={"response_id_for_each_request_id": "identical"},
        steps=["attach_unique_id_to_each_request_item", "execute_batch", "compare_request_and_response_ids"],
        assertions=["each_response_echoes_its_request_id", "response_is_associated_with_correct_operation"],
        sections=batch_base, mutation="read_only")
    add("test-batch-count-zero-rejected", "batch_workflows", ["Get"],
        [batch_count, request_item, get], parameters={"batch_count": 0, "request_item_count": 0},
        expected={"construction": "rejected_before_transport"},
        steps=["attempt_empty_batch_construction", "count_transport_calls"],
        assertions=["empty_batch_is_rejected_or_cannot_be_constructed", "transport_call_count_is_zero"],
        sections=batch_base, mode="local_model", mutation="read_only")
    add("test-batch-count-mismatch-rejected", "batch_workflows", ["Get", "Get Attribute List"],
        [batch_count, request_item, get, get_attribute_list], parameters={"declared_batch_count": 1, "request_item_count": 2},
        expected={"construction": "rejected_or_count_recomputed_before_transport"},
        steps=["attempt_inconsistent_batch_construction", "inspect_encoded_count_or_validation", "count_transport_calls"],
        assertions=["batch_count_matches_actual_items_or_request_is_rejected", "transport_never_sends_inconsistent_count"],
        sections=batch_base, mode="local_model", mutation="read_only")
    add("test-batch-single-item-options-omitted", "batch_workflows", ["Get"],
        [batch_count, continuation, order_option, request_item, get],
        parameters={"batch_count": 1, "batch_error_continuation_option": "omitted", "batch_order_option": "omitted"},
        expected={"continuation_option": "absent", "order_option": "absent"},
        steps=["construct_single_item_batch", "inspect_request_header"],
        assertions=["batch_error_continuation_option_absent_for_single_item", "single_item_order_option_omitted"],
        sections=batch_base, mode="local_model", mutation="read_only")
    add("test-batch-placeholder-forward-reference-rejected", "batch_workflows", ["Activate", "Create"],
        [request_item, activate, create], parameters={"items": ["Activate(IDPlaceholder from item 2)", "Create(unique-key)"], "placeholder_direction": "later item to earlier item"},
        expected={"construction": "rejected_or_server_reports_protocol_error"},
        steps=["construct_forward_placeholder_reference", "invoke_available_validation", "count_transport_calls"],
        assertions=["forward_placeholder_never_resolves_to_later_item", "invalid_reference_is_reported_not_silently_rebound"],
        sections=["8.3", "9.7"], mode="local_model", mutation="read_only")
    add("test-batch-placeholder-without-producer-rejected", "batch_workflows", ["Activate"],
        [request_item, activate], parameters={"items": ["Activate(IDPlaceholder)"]},
        expected={"construction": "rejected_or_operation_failed"},
        steps=["construct_placeholder_without_producer", "invoke_available_validation", "count_transport_calls"],
        assertions=["unbound_placeholder_is_reported", "no_unrelated_identifier_is_used"],
        sections=["8.3", "9.7"], mode="local_model", mutation="read_only")
    add("test-batch-placeholder-chain-resolves", "batch_workflows", ["Create", "Activate", "Get"],
        [request_item, response_item, create, activate, get],
        parameters={"items": ["Create(unique-key)", "Activate(IDPlaceholder from item 1)", "Get(IDPlaceholder from item 1)"], "placeholder_depth": 2},
        expected={"identifier_source": "first successful response", "all_steps": "same object"},
        steps=["create_object", "activate_using_placeholder", "read_using_placeholder", "compare_identifiers_and_state"],
        assertions=["each_placeholder_resolves_to_the_original_identifier", "all_operations_target_the_same_object", "cleanup_removes_object"],
        sections=["8.3", "8.6", "9.7"], setup=["Create a uniquely named object fixture"])

    indicator = field("Asynchronous Indicator", "8.2")
    async_mandatory = enum_value("Asynchronous Indicator", "Mandatory")
    async_optional = enum_value("Asynchronous Indicator", "Optional")
    async_prohibited = enum_value("Asynchronous Indicator", "Prohibited")
    correlation = field("Asynchronous Correlation Value", "8.6")
    create_request = payload("Create", "request", "Object Type")
    cancel_request = payload("Cancel", "request", "Asynchronous Correlation Value")
    cancel_response = payload("Cancel", "response", "Cancellation Result")
    async_base = ["8.2", "8.3", "8.6", "9.1", "9.2"]
    create_op = operation("Create")
    cancel_op = operation("Cancel")
    poll_op = operation("Poll")
    process_op = operation("Process")
    query_async_op = operation("Query Asynchronous Requests")
    add("test-async-indicator-omitted-default", "asynchronous_workflows", ["Create"],
        [indicator, async_prohibited, *create_request, create_op], parameters={"asynchronous_indicator": "omitted"},
        expected={"effective_indicator": "Prohibited", "behavior": "synchronous"},
        steps=["omit_asynchronous_indicator", "execute_create", "inspect_result_status_and_correlation"],
        assertions=["omitted_indicator_defaults_to_prohibited", "server_processes_synchronously", "correlation_value_is_absent"],
        sections=async_base + ["11.3"], setup=["Create a uniquely named test object"])
    add("test-async-indicator-prohibited-synchronous", "asynchronous_workflows", ["Create"],
        [indicator, async_prohibited, *create_request, create_op], parameters={"asynchronous_indicator": "Prohibited"},
        expected={"behavior": "synchronous completion", "pending_response": False},
        steps=["set_asynchronous_indicator_prohibited", "execute_create", "inspect_response"],
        assertions=["prohibited_indicator_forces_synchronous_processing", "no_poll_correlation_is_required", "create_result_is_complete"],
        sections=async_base + ["11.3"], setup=["Create a uniquely named test object"])
    add("test-async-indicator-optional-completes-or-pends", "asynchronous_workflows", ["Create"],
        [indicator, async_optional, *create_request, create_op], parameters={"asynchronous_indicator": "Optional"},
        expected={"behavior": ["Pending", "completed operation"], "poll_only_if_pending": True},
        steps=["set_asynchronous_indicator_true", "execute_create", "branch_on_result_status", "poll_only_when_pending"],
        assertions=["pending_response_contains_correlation_value", "completed_response_uses_original_operation_shape", "no_unnecessary_poll_after_completion"],
        sections=async_base + ["11.3"], setup=["Create a uniquely named test object"])
    add("test-async-indicator-mandatory-pending", "asynchronous_workflows", ["Create"],
        [indicator, async_mandatory, *create_request, create_op], parameters={"asynchronous_indicator": "Mandatory"},
        expected={"behavior": "Pending when the operation is accepted for asynchronous processing", "otherwise": "normative rejection"},
        steps=["request_mandatory_asynchronous_processing", "execute_create_on_async_controlled_peer", "inspect_result_status_and_correlation"],
        assertions=["accepted_mandatory_request_returns_pending", "pending_response_contains_correlation_value", "rejected_request_preserves_typed_reason"],
        sections=async_base + ["11.3"], setup=["Configure controlled peer to accept asynchronous Create"])
    add("test-async-poll-repeated-pending-to-success", "asynchronous_workflows", ["Create", "Poll"],
        [indicator, correlation, create_op, poll_op], parameters={"asynchronous_indicator": True, "poll_responses": ["Pending", "Pending", "Success"]},
        expected={"correlation_value": "stable across polls", "final_payload": "original Create response"},
        steps=["submit_async_create", "poll_pending_twice", "poll_until_success", "inspect_original_operation_payload", "cleanup_created_object"],
        assertions=["correlation_value_is_preserved", "each_pending_response_has_no_final_payload", "final_result_matches_create_shape", "no_poll_is_retried_automatically"],
        sections=async_base + ["6.1.38"], setup=["Create a uniquely named test object"])
    add("test-async-poll-final-operation-failure", "asynchronous_workflows", ["Create", "Poll"],
        [indicator, correlation, create_op, poll_op], parameters={"poll_responses": ["Pending", "OperationFailed"]},
        expected={"final_status": "OperationFailed", "final_shape": "original Create response"},
        steps=["submit_async_create_with_controlled_failure", "poll_until_final_failure", "inspect_result_reason_and_payload"],
        assertions=["final_status_is_operation_failed", "original_operation_result_shape_is_used", "correlation_value_matches_request"],
        sections=async_base + ["6.1.38"], setup=["Configure controlled peer to fail Create"])
    add("test-async-unknown-correlation-rejected", "asynchronous_workflows", ["Poll"],
        [correlation, poll_op], parameters={"correlation_value": "unknown-unissued-value"},
        expected={"status": "OperationFailed or a more specific normative failure"},
        steps=["poll_with_unknown_correlation_value", "inspect_typed_failure"],
        assertions=["unknown_correlation_is_not_mapped_to_another_request", "response_status_and_reason_are_preserved"],
        sections=async_base + ["6.1.38"], mutation="read_only")
    cancellation_values = [
        ("canceled", "Canceled"), ("unable-to-cancel", "Unable to Cancel"),
        ("completed", "Completed"), ("failed", "Failed"), ("unavailable", "Unavailable"),
    ]
    for slug, value in cancellation_values:
        result_target = enum_value("Cancellation Result", value)
        add(f"test-async-cancel-result-{slug}", "asynchronous_workflows", ["Create", "Cancel"],
            [result_target, indicator, correlation, create_op, cancel_op, *cancel_request, *cancel_response],
            parameters={"cancel_result": value, "asynchronous_indicator": True},
            expected={"cancel_response_value": value, "follow_up_state": value},
            steps=["submit_or_seed_pending_operation", "send_cancel_for_exact_correlation", "decode_cancellation_result", "verify_async_request_state"],
            assertions=["each_assigned_cancellation_result_roundtrips", "response_correlation_matches_pending_request", "follow_up_poll_matches_reported_state"],
            sections=async_base + ["6.1.5", "11.7"], setup=["Configure controlled peer for the selected cancellation result"])
    add("test-async-cancel-vs-poll-race", "asynchronous_workflows", ["Create", "Cancel", "Poll"],
        [indicator, correlation, create_op, cancel_op, poll_op, *cancel_request],
        parameters={"interleaving": ["Poll", "Cancel"], "terminal_states": ["Canceled", "Completed", "Failed"]},
        expected={"exactly_one_terminal_state": True, "correlation_value": "stable"},
        steps=["seed_pending_request", "interleave_poll_and_cancel", "observe_terminal_state", "verify_no_duplicate_completion"],
        assertions=["race_resolves_to_a_normative_terminal_state", "request_is_not_completed_twice", "poll_and_cancel_use_same_correlation_value"],
        sections=async_base + ["6.1.5", "6.1.38"], versions=["2.1"], setup=["Configure controlled peer with deterministic interleaving"])
    add("test-async-process-then-poll-state", "asynchronous_workflows", ["Create", "Process", "Poll"],
        [indicator, correlation, create_op, process_op, poll_op],
        parameters={"sequence": ["Create(Pending)", "Process(correlation)", "Poll(correlation)"], "protocol_version": "2.1"},
        expected={"poll_state": "reflects Process result"},
        steps=["submit_pending_request", "process_correlated_request", "poll_same_correlation", "compare_state_transition"],
        assertions=["process_and_poll_share_correlation", "poll_observes_post_process_state", "Process_response_is_not_itself_asynchronous"],
        sections=async_base + ["6.1.39", "6.1.38"], versions=["2.1"], setup=["Configure controlled peer for Process state transition"])
    add("test-async-query-outstanding-filter", "asynchronous_workflows", ["Create", "Query Asynchronous Requests"],
        [indicator, correlation, create_op, query_async_op],
        parameters={"filters": {"operations": ["Create"], "correlation_values": ["issued-correlation"]}, "protocol_version": "2.1"},
        expected={"results": "only matching outstanding requests"},
        steps=["seed_multiple_pending_requests", "query_with_operation_and_correlation_filters", "compare_outstanding_ids"],
        assertions=["filters_intersect_as_specified", "completed_requests_are_excluded", "correlation_values_are_preserved"],
        sections=async_base + ["6.1.41"], versions=["2.1"], setup=["Configure multiple pending requests with distinct correlations"])
    add("test-async-no-recursive-async-poll-cancel", "asynchronous_workflows", ["Cancel", "Poll"],
        [correlation, cancel_op, poll_op, *cancel_request, *cancel_response],
        parameters={"operations": ["Cancel", "Poll"], "asynchronous_indicator": "omitted"},
        expected={"poll_or_cancel_response": "not Pending and contains no async correlation"},
        steps=["execute_poll_and_cancel_responses", "inspect_response_headers_and_payloads"],
        assertions=["poll_and_cancel_are_not_returned_as_pending", "poll_and_cancel_responses_do_not_create_async_correlations"],
        sections=async_base + ["6.1.5", "6.1.38"], versions=["2.1"], mutation="read_only")

    sign_operation = operation("Sign")
    sign_parameters = [
        target for target in normative_targets
        if target["kind"] == "operation_payload_field"
        and target.get("operation") == "Sign"
        and target.get("role") == "request"
        and target["name"] == "Cryptographic Parameters"
    ]
    if {target["protocol_versions"][0] for target in sign_parameters} != {"2.0", "2.1"}:
        raise ValueError("Expected KMIP 2.0 and 2.1 Sign Cryptographic Parameters payload targets")
    result_status = enum_value("Result Status", "Operation Failed")
    bad_parameters = enum_value("Result Reason", "Bad Cryptographic Parameters")
    target_ids = list(dict.fromkeys([
        sign_operation,
        *(target["target_id"] for target in sign_parameters),
        result_status,
        bad_parameters,
    ]))
    sign_refs = []
    sign_error_sections = {"2.0": "6.1.49.1", "2.1": "6.1.55.1"}  # NOSONAR: OASIS clause numbers, not IPv4 destinations
    sign_error_tables = {"2.0": "304", "2.1": "336"}
    for version in ("2.0", "2.1"):
        document = f"OASIS KMIP {version} Specification"
        payload_target = next(target for target in sign_parameters if target["protocol_versions"] == [version])
        payload_ref = payload_target["source_refs"][0]
        sign_refs.append({"document": document, "section": payload_ref["section"]})
        sign_refs.append({"document": document, "section": payload_ref["section"], "table": payload_ref["table"]})
        sign_refs.append({"document": document, "section": sign_error_sections[version]})
        sign_refs.append({"document": document, "section": sign_error_sections[version], "table": sign_error_tables[version]})
        for target_id in (result_status, bad_parameters):
            enum_target = targets_by_id[target_id]
            for source_ref in enum_target.get("source_refs", []):
                if not source_ref.get("section"):
                    continue
                versioned_sections = _section_versions(source_ref["section"], indexes)
                if version in versioned_sections:
                    sign_refs.append({"document": document, "section": versioned_sections[version]})
    scenario = {
        "scenario_id": "test-sign-missing-managed-object-cryptographic-parameters",
        "target_ids": target_ids,
        "operations": ["Sign"],
        "protocol_versions": ["2.0", "2.1"],
        "parameters": {
            "managed_object_has_cryptographic_parameters": False,
            "algorithm_requires_parameters": True,
        },
        "expected": {
            "result_status": "Operation Failed",
            "allowed_result_reasons": ["Bad Cryptographic Parameters"],
        },
        "mode": "server_exchange",
        "mutation": "mutating",
        "steps": [
            "create_signing_key_without_cryptographic_parameters",
            "request_sign_algorithm_that_requires_parameters",
            "execute_sign",
            "inspect_operation_failed_status_and_result_reason",
            "destroy_signing_key_fixture",
        ],
        "assertions": [
            "operation_failed_required_when_managed_object_lacks_required_parameters",
            "allowed_sign_error_reason_is_source_backed",
            "request_parameters_are_omitted_when_object_has_none",
        ],
        "normative_refs": [reference for index, reference in enumerate(sign_refs) if reference not in sign_refs[:index]],
    }
    cases.append({
        "case_id": scenario["scenario_id"],
        "category": "negative_and_boundary",
        "operation": "Sign",
        "description": "Require the Sign failure response when the managed object lacks parameters required by the signing algorithm.",
        "requires": {"operation": "Sign", "protocol_scenario": scenario},
        "isolation_recipe": {
            "unique_name_prefix": "kmipkit-sign-missing-cryptographic-parameters",
            "setup_steps": ["Create a signing key without associated Cryptographic Parameters"],
            "target_step": "Execute Sign with an algorithm that requires the unavailable parameters",
            "teardown_steps": ["Destroy the signing key fixture"],
            "normalized_fields": ["UniqueIdentifier", "TimeStamp"],
        },
    })
    return cases


def _add_legacy_lifecycle_scenarios(
    test_cases: list[dict], normative_targets: list[dict], indexes: dict[str, dict[str, str]],
) -> None:
    """Attach complete operation sequences and per-version OASIS refs to legacy workflows."""
    workflows = {
        "test-batch-create-activate-destroy": {
            "operations": ["Create", "Activate", "Get Attribute List", "Destroy"],
            "versions": ["2.0", "2.1"],
            "sections": ["8.2", "8.3", "8.4", "8.6", "9.5", "9.6", "9.7", "9.8"],
            "category": "batch_workflows",
            "steps": ["construct_ordered_batch_items", "execute_single_batch_request", "verify_each_operation_response", "cleanup_created_object"],
            "assertions": ["batch_request_contains_complete_ordered_operation_list", "each_operation_has_a_normative_request_constructor", "batch_response_corresponds_to_each_ordered_operation"],
        },
        "test-batch-register-encrypt-decrypt-destroy": {
            "operations": ["Register", "Activate", "Encrypt", "Decrypt", "Destroy"],
            "versions": ["2.0", "2.1"],
            "sections": ["8.2", "8.3", "8.4", "8.6", "9.5", "9.6", "9.7", "9.8"],
            "category": "batch_workflows",
            "steps": ["construct_ordered_batch_items", "execute_single_batch_request", "verify_each_operation_response", "cleanup_registered_object"],
            "assertions": ["batch_request_contains_complete_ordered_operation_list", "each_operation_has_a_normative_request_constructor", "batch_response_corresponds_to_each_ordered_operation"],
        },
        "test-async-lifecycle-poll": {
            "operations": ["Create", "Query Asynchronous Requests", "Poll"],
            "versions": ["2.1"],
            "sections": ["8.2", "8.3", "8.6", "9.1", "9.2", "6.1.41"],
            "category": "asynchronous_workflows",
            "steps": ["submit_async_create", "query_outstanding_async_requests", "poll_until_completion", "cleanup_created_object"],
            "assertions": ["all_lifecycle_operations_are_listed_in_execution_order", "each_operation_has_a_normative_request_constructor", "poll_uses_the_correlated_async_request"],
        },
    }
    target_by_operation = {
        target["name"]: target for target in normative_targets
        if target["kind"] == "operation" and target["target_id"].startswith("KMIPKIT-ELEM-OP-C2S-")
    }
    cases_by_id = {case["case_id"]: case for case in test_cases}
    for case_id, definition in workflows.items():
        case = cases_by_id.get(case_id)
        if case is None:
            raise ValueError(f"Missing legacy lifecycle case {case_id}")
        operations = definition["operations"]
        operation_targets = []
        for name in operations:
            target = target_by_operation.get(name)
            if target is None:
                raise ValueError(f"No client-to-server operation target for {name} in {case_id}")
            operation_targets.append(target)
        versions = definition["versions"]
        if any(not set(versions).issubset(target["protocol_versions"]) for target in operation_targets):
            raise ValueError(f"Legacy lifecycle versions exceed an operation's applicability in {case_id}")

        refs = []
        for version in versions:
            document = f"OASIS KMIP {version} Specification"
            for target in operation_targets:
                for source_ref in target.get("source_refs", []):
                    versioned_sections = _section_versions(source_ref["section"], indexes)
                    if version not in versioned_sections:
                        continue
                    reference = {"document": document, "section": versioned_sections[version]}
                    if source_ref.get("table"):
                        reference["table"] = source_ref["table"]
                    if reference not in refs:
                        refs.append(reference)
            for section in definition["sections"]:
                if section in indexes[version]:
                    reference = {"document": document, "section": section}
                    if reference not in refs:
                        refs.append(reference)

        scenario = {
            "scenario_id": case_id,
            "target_ids": [target["target_id"] for target in operation_targets],
            "operations": operations,
            "protocol_versions": versions,
            "parameters": {"operations_in_request_order": operations},
            "expected": {"operation_order": operations},
            "mode": "server_exchange",
            "mutation": "mutating",
            "steps": definition["steps"],
            "assertions": definition["assertions"],
            "normative_refs": refs,
        }
        case["requires"]["protocol_scenario"] = scenario


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
        policy = _attribute_policy(attr)
        test_cases.append({
            "case_id": case_id,
            "category": "attributes",
            "operation": "Get Attributes",
            "oasis_section": "§4",
            "description": f"Verify read/write semantics and policies for attribute '{attr_name}'.",
            "requires": {
                "attribute": attr_name,
                "attribute_policy": policy,
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-attr-{attr_name.lower().replace(' ', '-')}",
                "setup_steps": ["Create or register an object allowed by the source attribute policy"],
                "target_step": f"Construct, read back, and exercise every declared policy branch for {attr_name}",
                "teardown_steps": ["Restore permitted state and destroy test object"],
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
            "id": "test-neg-modify-attribute-current-mismatch",
            "op": "Modify Attribute",
            "desc": "Verify server rejects Modify Attribute when the Current Attribute value does not match the existing Comment value.",
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
                "setup_steps": (
                    ["Create test object with Comment='KMIPKit interop test'"]
                    if nt["id"] == "test-neg-modify-attribute-current-mismatch"
                    else ["Create test object in initial state"]
                ),
                "target_step": (
                    "Attempt Modify Attribute with Current Attribute='old test value' and New Attribute='new test value'"
                    if nt["id"] == "test-neg-modify-attribute-current-mismatch"
                    else f"Attempt illegal {nt['op']}"
                ),
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
    bitmask_targets_by_parent = {}
    for target in normative_targets:
        if target["kind"] == "bitmask_value" and target.get("allocation") == "assigned":
            for parent_id in target.get("parent_element_ids", []):
                bitmask_targets_by_parent.setdefault(parent_id, []).append(target)
    bitmask_elements = {element["element_id"]: element for element in elements if element.get("kind") == "bitmask"}
    for parent_id, flag_targets in sorted(bitmask_targets_by_parent.items()):
        family = bitmask_elements[parent_id]
        family_name = family["name"]
        slug = re.sub(r"[^a-z0-9]+", "-", family_name.casefold()).strip("-")
        flag_targets.sort(key=lambda target: int(target["wire_value"], 16))
        test_cases.append({
            "case_id": f"test-bitmask-{slug}-all-combinations",
            "category": "enums_and_bitmasks",
            "operation": "Query",
            "description": f"Construct, encode, decode, and compare every combination of the {len(flag_targets)} assigned {family_name} flags.",
            "requires": {
                "bitmask_type": family_name,
                "target_protocol_versions": ["2.0", "2.1"],
                "normative_target_ids": [target["target_id"] for target in flag_targets],
                "exhaustive_bitmask_combinations": {
                    "flag_target_ids": [target["target_id"] for target in flag_targets],
                    "flag_wire_values": [target["wire_value"] for target in flag_targets],
                    "combination_count": 1 << len(flag_targets),
                    "algorithm": "OR every subset of the assigned flag values",
                    "includes_zero": True,
                    "unknown_bits_tested_separately": True,
                },
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-mask-{slug}-all-combinations",
                "setup_steps": [],
                "target_step": f"Roundtrip all 2^{len(flag_targets)} assigned {family_name} combinations",
                "teardown_steps": [],
            },
        })
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

    test_cases.extend(_protocol_scenario_cases(normative_targets, section_indexes))
    _add_legacy_lifecycle_scenarios(test_cases, normative_targets, section_indexes)
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
            "batch_scenarios": sum(1 for case in test_cases if case["category"] == "batch_workflows" and case.get("requires", {}).get("protocol_scenario")),
            "asynchronous_scenarios": sum(1 for case in test_cases if case["category"] == "asynchronous_workflows" and case.get("requires", {}).get("protocol_scenario")),
            "exhaustive_bitmask_families": sum(1 for case in test_cases if case.get("requires", {}).get("exhaustive_bitmask_combinations")),
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
