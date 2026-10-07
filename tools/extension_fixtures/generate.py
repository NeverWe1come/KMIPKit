#!/usr/bin/env python3
"""Generate dependency-free adapter fixture descriptors from reviewed inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
CORPUS = ROOT / "tests" / "fixtures" / "extensions" / "cases.json"
OUTPUTS = {
    ROOT / "bindings" / "c" / "tests" / "extension_fixtures.generated.h": "c",
    ROOT / "bindings" / "java" / "src" / "test" / "java" / "org" / "kmipkit" / "SharedExtensionFixtures.java": "java",
    ROOT / "crates" / "kmipkit-client" / "tests" / "fixtures" / "extensions" / "extension_fixtures.generated.rs": "rust",
}

ITEM_TYPES = {
    "Structure": "Structure",
    "TextString": "TextString",
    "Enumeration": "Enumeration",
    "LongInteger": "LongInteger",
}
OUTCOMES = {
    "recognized": "Recognized",
    "unrecognized.schema_invalid": "SchemaInvalid",
    "unrecognized.ambiguous": "Ambiguous",
    "unrecognized.no_match": "NoMatch",
}

REQUIRED_TYPE_IDS = {
    "ExtensionIdentity", "Compatibility", "TtlvPath", "Discriminator",
    "ExtensionSchema", "ExtensionDefinition", "ExtensionChildRule",
    "ClientExtensionRegistry", "ExtensionRecognition", "ValidatedExtensionValue",
    "TtlvStructure", "TtlvItem", "TtlvValue", "TtlvStructureView",
    "TtlvItemView", "TtlvValueView", "TtlvItemType",
    "ClientBatchItem", "ClientRequestMessageExtension",
}
REQUIRED_FUNCTION_IDS = {
    "extension_identity_create", "extension_compatibility_create", "ttlv_path_create",
    "ttlv_path_with_child_tag", "ttlv_value_text_string", "extension_discriminator_create",
    "extension_schema_scalar", "extension_schema_signed_numeric_range",
    "extension_schema_structure", "extension_child_rule_required", "extension_definition_create",
    "client_extension_registry_create", "client_extension_registry_inspect",
    "extension_recognition_is_recognized", "extension_recognition_validated_value",
    "extension_recognition_generic_value", "ttlv_structure_create", "ttlv_item_create",
    "ttlv_structure_with_item", "ttlv_value_structure", "ttlv_value_long_integer",
    "ttlv_value_enumeration", "ttlv_structure_view", "ttlv_structure_view_item_count",
    "ttlv_structure_view_item_at", "ttlv_item_view_tag", "ttlv_item_view_type",
    "ttlv_item_view_value", "ttlv_value_view_type", "ttlv_value_view_structure",
    "ttlv_value_view_long_integer", "ttlv_value_view_enumeration", "ttlv_value_view_byte_length",
    "ttlv_value_view_byte_at", "client_request_message_extension_create",
    "client_batch_item_with_extension", "client_batch_item_discover_versions",
    "client_batch_item_extension_count", "client_batch_item_extension_identity_at",
    "client_batch_item_extension_criticality_indicator_at",
}


class FixtureError(ValueError):
    """An unsupported or inconsistent reviewed fixture input."""


@dataclass(frozen=True)
class Item:
    tag: int
    item_type: str
    text: str = ""
    number: int = 0
    children: tuple["Item", ...] = ()


@dataclass(frozen=True)
class ChildRule:
    tag: int
    schema: "Schema"


@dataclass(frozen=True)
class Schema:
    item_type: str
    text: str = ""
    has_range: bool = False
    minimum: int = 0
    maximum: int = 0
    children: tuple[ChildRule, ...] = ()


@dataclass(frozen=True)
class Definition:
    fixture_id: str
    vendor: str
    name: str
    version: str
    path: tuple[int, ...]
    discriminator_type: str
    discriminator_text: str
    discriminator_number: int
    schema: Schema


@dataclass(frozen=True)
class OutboundAttachment:
    fixture_id: str
    criticality_indicator: bool


@dataclass(frozen=True)
class Case:
    fixture_id: str
    vendor: str
    critical: bool
    payload: tuple[Item, ...]
    outcome: str
    matched_ids: tuple[str, ...]
    typed: bool
    outbound_outcome: str
    outbound_attachments: tuple[OutboundAttachment, ...]


def _keys(value: Any, expected: set[str], where: str, optional: set[str] = frozenset()) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise FixtureError(f"{where} must be an object")
    actual = set(value)
    missing = expected - actual
    extra = actual - expected - set(optional)
    if missing or extra:
        raise FixtureError(f"{where} has unsupported keys (missing={sorted(missing)}, extra={sorted(extra)})")
    return value


def _string(value: Any, where: str, pattern: str | None = None) -> str:
    if not isinstance(value, str) or not value:
        raise FixtureError(f"{where} must be a non-empty string")
    if pattern is not None and re.fullmatch(pattern, value) is None:
        raise FixtureError(f"{where} has an unsupported value")
    return value


def _integer(value: Any, where: str, minimum: int, maximum: int) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not minimum <= value <= maximum:
        raise FixtureError(f"{where} is outside its supported integer range")
    return value


def _tag(value: Any, where: str) -> int:
    text = _string(value, where, r"0x[0-9A-Fa-f]{6}")
    tag = int(text[2:], 16)
    if not 0x420000 <= tag <= 0x54FFFF:
        raise FixtureError(f"{where} is outside the reviewed extension tag allocation")
    return tag


def _parse_item(value: Any, where: str, *, root: bool = False) -> Item:
    if not isinstance(value, dict):
        raise FixtureError(f"{where} must be an object")
    item_type = _string(value.get("type"), f"{where}.type")
    if item_type not in ITEM_TYPES:
        raise FixtureError(f"{where} uses unsupported TTLV type {item_type!r}")
    if item_type == "Structure":
        expected = {"type", "children"} if root else {"tag", "type", "children"}
        _keys(value, expected, where)
        children = value["children"]
        if not isinstance(children, list):
            raise FixtureError(f"{where}.children must be an array")
        tag = _tag(value["tag"], f"{where}.tag") if "tag" in value else 0
        return Item(tag, item_type, children=tuple(_parse_item(child, f"{where}.children[{i}]") for i, child in enumerate(children)))
    _keys(value, {"tag", "type", "value"}, where)
    tag = _tag(value["tag"], f"{where}.tag")
    raw = value["value"]
    if item_type == "TextString":
        return Item(tag, item_type, text=_string(raw, f"{where}.value", r"[\s\S]+"))
    if item_type == "Enumeration":
        return Item(tag, item_type, number=_integer(raw, f"{where}.value", 0, 0xFFFFFFFF))
    return Item(tag, item_type, number=_integer(raw, f"{where}.value", -(1 << 63), (1 << 63) - 1))


def _parse_schema_node(item_schema: Any, where: str) -> tuple[int, Schema]:
    if not isinstance(item_schema, dict):
        raise FixtureError(f"{where} must be an object")
    _keys(item_schema, {"type", "required", "properties"}, where)
    if item_schema["type"] != "object":
        raise FixtureError(f"{where}.type must be object")
    required = item_schema["required"]
    if not isinstance(required, list) or len(required) != len(set(required)):
        raise FixtureError(f"{where}.required must be a unique array")
    props = item_schema["properties"]
    if not isinstance(props, dict):
        raise FixtureError(f"{where}.properties must be an object")
    try:
        tag = _tag(props["tag"]["const"], f"{where}.properties.tag.const")
        item_type = _string(props["type"]["const"], f"{where}.properties.type.const")
    except (KeyError, TypeError) as error:
        raise FixtureError(f"{where} is missing a supported tag/type constraint") from error
    if item_type not in ITEM_TYPES or item_type == "Structure" and "children" not in props:
        raise FixtureError(f"{where} uses an unsupported schema TTLV type")
    if item_type == "Structure":
        _keys(props, {"tag", "type", "children"}, f"{where}.properties")
        if set(required) != {"tag", "type", "children"}:
            raise FixtureError(f"{where} must require tag, type, and children")
        return tag, _parse_children_schema(props["children"], f"{where}.properties.children")
    _keys(props, {"tag", "type", "value"}, f"{where}.properties")
    if set(required) != {"tag", "type", "value"}:
        raise FixtureError(f"{where} must require tag, type, and value")
    value_schema = props["value"]
    if not isinstance(value_schema, dict):
        raise FixtureError(f"{where}.properties.value must be an object")
    if set(value_schema) == {"const"} and item_type == "TextString":
        return tag, Schema(item_type, text=_string(value_schema["const"], f"{where}.properties.value.const"))
    if set(value_schema) == {"type", "minimum", "maximum"} and item_type == "LongInteger" and value_schema["type"] == "integer":
        low = _integer(value_schema["minimum"], f"{where}.properties.value.minimum", -(1 << 63), (1 << 63) - 1)
        high = _integer(value_schema["maximum"], f"{where}.properties.value.maximum", low, (1 << 63) - 1)
        return tag, Schema(item_type, has_range=True, minimum=low, maximum=high)
    raise FixtureError(f"{where}.properties.value uses an unsupported scalar schema")


def _parse_children_schema(value: Any, where: str) -> Schema:
    _keys(value, {"type"}, where, {"contains", "allOf"})
    if value["type"] != "array":
        raise FixtureError(f"{where}.type must be array")
    if ("contains" in value) == ("allOf" in value):
        raise FixtureError(f"{where} must use exactly one of contains or allOf")
    if "contains" in value:
        schemas = [value["contains"]]
    else:
        schemas = value["allOf"]
        if not isinstance(schemas, list) or not schemas:
            raise FixtureError(f"{where}.allOf must be a non-empty array")
    rules: list[ChildRule] = []
    seen: set[int] = set()
    for index, node in enumerate(schemas):
        _keys(node, {"contains"}, f"{where}.allOf[{index}]") if "allOf" in value else None
        tag, child_schema = _parse_schema_node(node["contains"] if "allOf" in value else node, f"{where}[{index}]")
        if tag in seen:
            raise FixtureError(f"{where} repeats a required child tag")
        seen.add(tag)
        rules.append(ChildRule(tag, child_schema))
    return Schema("Structure", children=tuple(rules))


def _parse_definition(value: Any, where: str) -> Definition:
    _keys(value, {"id", "vendorIdentification", "extensionName", "extensionVersion", "discriminator", "payloadSchema"}, where)
    fixture_id = _string(value["id"], f"{where}.id", r"[a-z0-9.-]+")
    discr = _keys(value["discriminator"], {"path", "itemType", "value"}, f"{where}.discriminator")
    path = discr["path"]
    if not isinstance(path, list) or not path:
        raise FixtureError(f"{where}.discriminator.path must be a non-empty array")
    parsed_path = tuple(_tag(tag, f"{where}.discriminator.path[{i}]") for i, tag in enumerate(path))
    discriminator_type = _string(discr["itemType"], f"{where}.discriminator.itemType")
    if discriminator_type not in {"TextString", "Enumeration", "LongInteger"}:
        raise FixtureError(f"{where} has unsupported discriminator type")
    if discriminator_type == "TextString":
        text, number = _string(discr["value"], f"{where}.discriminator.value"), 0
    elif discriminator_type == "Enumeration":
        text, number = "", _integer(discr["value"], f"{where}.discriminator.value", 0, 0xFFFFFFFF)
    else:
        text, number = "", _integer(discr["value"], f"{where}.discriminator.value", -(1 << 63), (1 << 63) - 1)
    root = value["payloadSchema"]
    _keys(root, {"$schema", "title", "type", "required", "properties"}, f"{where}.payloadSchema")
    if root["$schema"] != "https://json-schema.org/draft/2020-12/schema" or root["type"] != "object":
        raise FixtureError(f"{where}.payloadSchema has an unsupported dialect or root type")
    if not isinstance(root["title"], str) or root["required"] != ["type", "children"]:
        raise FixtureError(f"{where}.payloadSchema has unsupported root metadata")
    props = _keys(root["properties"], {"type", "children"}, f"{where}.payloadSchema.properties")
    if props["type"] != {"const": "Structure"}:
        raise FixtureError(f"{where}.payloadSchema root must constrain Structure")
    return Definition(
        fixture_id,
        _string(value["vendorIdentification"], f"{where}.vendorIdentification"),
        _string(value["extensionName"], f"{where}.extensionName"),
        _string(value["extensionVersion"], f"{where}.extensionVersion"),
        parsed_path,
        discriminator_type,
        text,
        number,
        _parse_children_schema(props["children"], f"{where}.payloadSchema.properties.children"),
    )


def _matches_schema(schema: Schema, item: Item) -> bool:
    if schema.item_type != item.item_type:
        return False
    if schema.item_type == "Structure":
        for rule in schema.children:
            found = [child for child in item.children if child.tag == rule.tag]
            if len(found) != 1 or not _matches_schema(rule.schema, found[0]):
                return False
        return True
    if schema.text:
        return schema.text == item.text
    return not schema.has_range or schema.minimum <= item.number <= schema.maximum


def _at_path(items: tuple[Item, ...], path: tuple[int, ...]) -> Item | None:
    current_items = items
    current = None
    for tag in path:
        matches = [item for item in current_items if item.tag == tag]
        if len(matches) != 1:
            return None
        current = matches[0]
        current_items = current.children
    return current


def _validate_manifest(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict) or value.get("formatVersion") != 1 or value.get("scope") != "registry-slice":
        raise FixtureError("public API manifest has unsupported format or scope")
    types = {entry.get("id"): entry for entry in value.get("types", []) if isinstance(entry, dict)}
    functions = {entry.get("id"): entry for entry in value.get("functions", []) if isinstance(entry, dict)}
    if REQUIRED_TYPE_IDS - types.keys() or REQUIRED_FUNCTION_IDS - functions.keys():
        raise FixtureError("public API manifest is missing fixture adapter contract entries")
    for type_id in REQUIRED_TYPE_IDS:
        entry = types[type_id]
        for adapter, required in {
            "rust": {"module", "name", "owner"},
            "c": {"name", "kind"},
            "java": {"package", "name"},
            "python": {"module", "name"},
        }.items():
            mapping = entry.get(adapter)
            if not isinstance(mapping, dict) or required - mapping.keys():
                raise FixtureError(f"manifest type {type_id} has incomplete {adapter} mapping")
            if any(not isinstance(mapping[field], str) or not mapping[field] for field in required):
                raise FixtureError(f"manifest type {type_id} has malformed {adapter} mapping metadata")
    for function_id in REQUIRED_FUNCTION_IDS:
        entry = functions[function_id]
        requirements = entry.get("requirementIds")
        if not isinstance(requirements, list) or "KMIPKIT-0012-FR-010" not in requirements:
            raise FixtureError(f"manifest function {function_id} lacks FR-010 traceability")
        for adapter, required in {
            "rust": {"module", "name", "owner", "parameters", "returnType"},
            "c": {"symbol", "parameters", "returnType", "errorCategories"},
            "java": {"package", "class", "method", "parameters", "jniSymbol", "returnType"},
            "python": {"module", "function", "parameters", "cffiSymbol", "returnType"},
        }.items():
            mapping = entry.get(adapter)
            if not isinstance(mapping, dict) or required - mapping.keys():
                raise FixtureError(f"manifest function {function_id} has incomplete {adapter} mapping")
            string_fields = required - {"parameters", "errorCategories"}
            if any(not isinstance(mapping[field], str) or not mapping[field] for field in string_fields):
                raise FixtureError(f"manifest function {function_id} has malformed {adapter} mapping names")
            if not isinstance(mapping["parameters"], list) or not isinstance(mapping["returnType"], str) or not mapping["returnType"]:
                raise FixtureError(f"manifest function {function_id} has malformed {adapter} signature metadata")
            for parameter in mapping["parameters"]:
                if not isinstance(parameter, dict) or not isinstance(parameter.get("name"), str) or not parameter["name"] or not isinstance(parameter.get("type"), str) or not parameter["type"]:
                    raise FixtureError(f"manifest function {function_id} has malformed {adapter} parameter metadata")
        if not isinstance(functions[function_id]["c"]["errorCategories"], list) or any(not isinstance(category, str) or not category for category in functions[function_id]["c"]["errorCategories"]):
            raise FixtureError(f"manifest function {function_id} has malformed C error metadata")
        if functions[function_id]["c"]["symbol"] != f"kmipkit_{function_id}":
            raise FixtureError(f"manifest C symbol for {function_id} does not match its stable ABI name")
        if functions[function_id]["python"]["cffiSymbol"] != functions[function_id]["c"]["symbol"]:
            raise FixtureError(f"manifest Python CFFI symbol for {function_id} disagrees with the stable C ABI")
        if not functions[function_id]["java"]["jniSymbol"].startswith("Java_"):
            raise FixtureError(f"manifest Java JNI symbol for {function_id} is malformed")
    order_preserving_functions = (
        "client_request_message_extension_create",
        "client_batch_item_with_extension",
        "client_batch_item_discover_versions",
    )
    if any(
        functions[function_id].get("ordering") != "preserves-caller-batch-and-extension-order"
        for function_id in order_preserving_functions
    ):
        raise FixtureError("public API manifest does not guarantee outbound extension ordering")
    criticality_metadata = functions["client_request_message_extension_create"].get(
        "criticalityIndicator"
    )
    expected_criticality_metadata = {
        "rustParameter": {"name": "criticality_indicator", "type": "bool"},
        "cParameter": {
            "name": "criticality_indicator",
            "type": "uint8_t",
            "acceptedValues": [0, 1],
        },
        "javaParameter": {"name": "criticalityIndicator", "type": "boolean"},
        "pythonParameter": {"name": "criticality_indicator", "type": "bool"},
    }
    if criticality_metadata != expected_criticality_metadata:
        raise FixtureError("public API manifest criticality metadata is unsupported")
    enum = types["TtlvItemType"].get("enumRepresentation", {}).get("values", [])
    enum_metadata = types["TtlvItemType"].get("enumRepresentation", {})
    if enum_metadata.get("shape") != "closed-numeric" or enum_metadata.get("unknownValuePolicy") != "reject-unsupported":
        raise FixtureError("public API manifest TTLV Item Type metadata is unsupported")
    enum_values = {entry.get("name"): entry.get("value") for entry in enum if isinstance(entry, dict)}
    if any(name not in enum_values for name in ITEM_TYPES):
        raise FixtureError("public API manifest omits a supported fixture TTLV Item Type")
    if len(set(enum_values.values())) != len(enum_values):
        raise FixtureError("public API manifest repeats a TTLV Item Type value")
    api_contract = {
        "types": {type_id: types[type_id] for type_id in sorted(REQUIRED_TYPE_IDS)},
        "functions": {function_id: functions[function_id] for function_id in sorted(REQUIRED_FUNCTION_IDS)},
        "itemTypes": enum,
    }
    fingerprint = hashlib.sha256(
        json.dumps(api_contract, sort_keys=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    return {"types": types, "functions": functions, "item_types": enum_values, "fingerprint": fingerprint}


def _parse_inputs(api: Any, corpus: Any) -> tuple[dict[str, Any], tuple[Definition, ...], tuple[Case, ...]]:
    manifest = _validate_manifest(api)
    _keys(corpus, {"format", "formatVersion", "outcomeEncoding", "definitions", "cases", "limitCases", "algorithmWorkCases"}, "fixture corpus")
    if corpus["format"] != "kmipkit-extension-fixtures" or corpus["formatVersion"] != 1:
        raise FixtureError("fixture corpus has unsupported format or version")
    encoding = _keys(corpus["outcomeEncoding"], {"version", "outcomeCodes", "matchedDefinitionIdsOrder", "genericPayloadRepresentation", "arrayOrder"}, "outcomeEncoding")
    if encoding != {
        "version": 1,
        "outcomeCodes": ["recognized", "unrecognized.schema_invalid", "unrecognized.ambiguous", "unrecognized.no_match", "outbound.validated"],
        "matchedDefinitionIdsOrder": "lexicographic-ascending",
        "genericPayloadRepresentation": "normalized-ttlv-v1",
        "arrayOrder": "significant-and-preserved",
    }:
        raise FixtureError("fixture outcome encoding is not the reviewed normalized-ttlv-v1 contract")
    raw_definitions, raw_cases = corpus["definitions"], corpus["cases"]
    if not isinstance(raw_definitions, list) or not raw_definitions or not isinstance(raw_cases, list) or not raw_cases:
        raise FixtureError("fixture definitions and cases must be non-empty arrays")
    definitions = tuple(_parse_definition(node, f"definitions[{i}]") for i, node in enumerate(raw_definitions))
    definition_ids = [definition.fixture_id for definition in definitions]
    if len(set(definition_ids)) != len(definition_ids):
        raise FixtureError("fixture definition IDs must be unique")
    cases: list[Case] = []
    seen_cases: set[str] = set()
    outbound_requests: list[tuple[int, Any, str]] = []
    for index, raw in enumerate(raw_cases):
        where = f"cases[{index}]"
        _keys(raw, {"id", "kind", "extension", "expected"}, where, {"secretSentinel"})
        case_id = _string(raw["id"], f"{where}.id", r"[a-z0-9-]+")
        if case_id in seen_cases:
            raise FixtureError(f"{where}.id is duplicated")
        seen_cases.add(case_id)
        extension = _keys(raw["extension"], {"vendorIdentification", "criticalityIndicator", "payload"}, f"{where}.extension")
        if not isinstance(extension["criticalityIndicator"], bool):
            raise FixtureError(f"{where}.extension.criticalityIndicator must be Boolean")
        payload_root = _keys(extension["payload"], {"type", "children"}, f"{where}.extension.payload")
        if payload_root["type"] != "Structure":
            raise FixtureError(f"{where}.extension.payload must be a Structure")
        payload = _parse_item(payload_root, f"{where}.extension.payload", root=True)
        if payload.item_type != "Structure":
            raise FixtureError(f"{where}.extension.payload must be a Structure")
        expected = _keys(raw["expected"], {"outcomeCode", "matchedDefinitionIds", "typedValueAvailable", "genericPayloadPreserved", "preservedPayload", "diagnostics"}, f"{where}.expected", {"outboundRequest"})
        if not isinstance(expected["genericPayloadPreserved"], bool) or not expected["genericPayloadPreserved"]:
            raise FixtureError(f"{where} must expect generic TTLV preservation")
        preserved_root = _keys(expected["preservedPayload"], {"type", "children"}, f"{where}.expected.preservedPayload")
        if preserved_root["type"] != "Structure":
            raise FixtureError(f"{where}.expected.preservedPayload must be a Structure")
        preserved = _parse_item(preserved_root, f"{where}.expected.preservedPayload", root=True)
        if payload != preserved:
            raise FixtureError(f"{where} expected generic TTLV tree differs from inbound payload")
        matches: list[Definition] = []
        for definition in definitions:
            found = _at_path(payload.children, definition.path)
            if definition.vendor == extension["vendorIdentification"] and found is not None and found.item_type == definition.discriminator_type and (found.text == definition.discriminator_text if found.item_type == "TextString" else found.number == definition.discriminator_number):
                matches.append(definition)
        match_ids = tuple(sorted(definition.fixture_id for definition in matches))
        expected_ids = expected["matchedDefinitionIds"]
        if not isinstance(expected_ids, list) or tuple(expected_ids) != match_ids:
            raise FixtureError(f"{where} expected matched definition IDs disagree with discriminators")
        outcome = _string(expected["outcomeCode"], f"{where}.expected.outcomeCode")
        if outcome not in OUTCOMES:
            raise FixtureError(f"{where} has an unsupported inbound outcome code")
        typed = expected["typedValueAvailable"]
        if not isinstance(typed, bool) or typed != (outcome == "recognized"):
            raise FixtureError(f"{where} typed availability disagrees with outcome")
        if len(matches) > 1:
            calculated = "unrecognized.ambiguous"
        elif not matches:
            calculated = "unrecognized.no_match"
        elif _matches_schema(matches[0].schema, payload_item := Item(0, "Structure", children=payload.children)):
            calculated = "recognized"
        else:
            calculated = "unrecognized.schema_invalid"
        if calculated != outcome:
            raise FixtureError(f"{where} outcome disagrees with discriminator/schema evaluation")
        diagnostics = _keys(expected["diagnostics"], {"redacted", "containsPayloadValues"}, f"{where}.expected.diagnostics")
        if diagnostics != {"redacted": True, "containsPayloadValues": False}:
            raise FixtureError(f"{where} diagnostics must preserve the redacted metadata contract")
        cases.append(
            Case(
                case_id,
                _string(extension["vendorIdentification"], f"{where}.extension.vendorIdentification"),
                extension["criticalityIndicator"],
                payload.children,
                outcome,
                match_ids,
                typed,
                "",
                (),
            )
        )
        if "outboundRequest" in expected:
            outbound_requests.append(
                (
                    len(cases) - 1,
                    expected["outboundRequest"],
                    f"{where}.expected.outboundRequest",
                )
            )
    cases_by_id = {case.fixture_id: case for case in cases}
    for case_index, raw_request, where in outbound_requests:
        request = _keys(raw_request, {"outcomeCode", "attachments"}, where)
        outbound_outcome = _string(request["outcomeCode"], f"{where}.outcomeCode")
        if outbound_outcome != "outbound.validated":
            raise FixtureError(f"{where} has an unsupported outbound outcome code")
        raw_attachments = request["attachments"]
        if not isinstance(raw_attachments, list) or not raw_attachments:
            raise FixtureError(f"{where}.attachments must be a non-empty array")
        attachments = []
        for attachment_index, raw_attachment in enumerate(raw_attachments):
            attachment_where = f"{where}.attachments[{attachment_index}]"
            attachment = _keys(
                raw_attachment,
                {"fixtureId", "criticalityIndicator"},
                attachment_where,
            )
            fixture_id = _string(attachment["fixtureId"], f"{attachment_where}.fixtureId")
            if not isinstance(attachment["criticalityIndicator"], bool):
                raise FixtureError(f"{attachment_where}.criticalityIndicator must be Boolean")
            target_case = cases_by_id.get(fixture_id)
            if target_case is None:
                raise FixtureError(f"{attachment_where} references an unknown fixture case")
            if target_case.outcome != "recognized" or not target_case.typed or len(target_case.matched_ids) != 1:
                raise FixtureError(f"{attachment_where} must reference one typed recognized fixture")
            attachments.append(
                OutboundAttachment(fixture_id, attachment["criticalityIndicator"])
            )
        case = cases[case_index]
        cases[case_index] = Case(
            case.fixture_id,
            case.vendor,
            case.critical,
            case.payload,
            case.outcome,
            case.matched_ids,
            case.typed,
            outbound_outcome,
            tuple(attachments),
        )
    return manifest, definitions, tuple(cases)


def _cpp_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def _rust_string(value: str) -> str:
    escaped = []
    for character in value:
        codepoint = ord(character)
        if character == '"':
            escaped.append('\\"')
        elif character == "\\":
            escaped.append("\\\\")
        elif character == "\n":
            escaped.append("\\n")
        elif character == "\r":
            escaped.append("\\r")
        elif character == "\t":
            escaped.append("\\t")
        elif character == "\0":
            escaped.append("\\0")
        elif codepoint < 0x20 or codepoint == 0x7F:
            escaped.append(f"\\u{{{codepoint:x}}}")
        else:
            escaped.append(character)
    return '"' + "".join(escaped) + '"'


def _rust_tag_literal(tag: int) -> str:
    digits = f"{tag:08X}"
    return f"0x{digits[:-4]}_{digits[-4:]}"


def _rust_integer_literal(value: int) -> str:
    digits = str(abs(value))
    first_group_length = len(digits) % 3 or 3
    groups = [digits[:first_group_length]]
    groups.extend(
        digits[index : index + 3]
        for index in range(first_group_length, len(digits), 3)
    )
    sign = "-" if value < 0 else ""
    return sign + "_".join(groups)


def _generate_c(manifest: dict[str, Any], definitions: tuple[Definition, ...], cases: tuple[Case, ...]) -> str:
    # C test descriptors use a tagged flat representation: each structure owns a
    # depth-first item span, making construction independent of JSON parsing.
    type_values = manifest["item_types"]
    lines = [f"/* Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: {manifest['fingerprint']}. Do not edit. */", "#ifndef KMIPKIT_EXTENSION_FIXTURES_GENERATED_H", "#define KMIPKIT_EXTENSION_FIXTURES_GENERATED_H", "#include <stdbool.h>", "#include <stdint.h>", "#include <stddef.h>", "typedef struct kmipkit_fixture_item { uint32_t tag; uint8_t type; const char *text; int64_t signed_value; uint32_t enum_value; const struct kmipkit_fixture_item *children; size_t child_count; } kmipkit_fixture_item_t;", "typedef struct { uint32_t tag; uint8_t type; const char *text; int64_t minimum; int64_t maximum; bool has_range; const struct kmipkit_fixture_schema *nested; } kmipkit_fixture_child_rule_t;", "typedef struct kmipkit_fixture_schema { uint8_t type; const char *text; int64_t minimum; int64_t maximum; bool has_range; const kmipkit_fixture_child_rule_t *children; size_t child_count; } kmipkit_fixture_schema_t;", "typedef struct { const char *id; const char *vendor; const char *name; const char *version; const uint32_t *path; size_t path_count; uint8_t discriminator_type; const char *discriminator_text; int64_t discriminator_number; const kmipkit_fixture_schema_t *schema; } kmipkit_fixture_definition_t;", "typedef struct { const char *id; const char *vendor; bool critical; const kmipkit_fixture_item_t *payload; size_t payload_count; const char *outcome; const char *const *matched_ids; size_t matched_count; bool typed; } kmipkit_fixture_case_t;"]
    lines.extend(
        [
            "typedef struct { const char *fixture_id; bool criticality_indicator; } kmipkit_fixture_attachment_t;",
            "typedef struct { const char *fixture_id; const char *outcome; const kmipkit_fixture_attachment_t *attachments; size_t attachment_count; } kmipkit_fixture_outbound_request_t;",
        ]
    )
    # For C descriptor generation use flat depth-first item arrays with parent-child spans.
    def emit_item_array(items: tuple[Item, ...], name: str) -> None:
        flat: list[tuple[Item, int, int]] = []
        def visit(nodes: tuple[Item, ...]) -> tuple[int, int]:
            start = len(flat)
            # Reserve this level's direct children before recursively appending descendants.
            flat.extend((node, -1, 0) for node in nodes)
            for offset, node in enumerate(nodes):
                if node.item_type == "Structure" and node.children:
                    child_start, child_count = visit(node.children)
                    flat[start + offset] = (node, child_start, child_count)
            return start, len(nodes)
        visit(items)
        rows = []
        for node, child_start, child_count in flat:
            rows.append("    { UINT32_C(0x%06X), %dU, %s, INT64_C(%d), UINT32_C(%d), %s, %dU }" % (
                node.tag, type_values[node.item_type], _cpp_string(node.text) if node.text else "NULL", node.number if node.item_type == "LongInteger" else 0, node.number if node.item_type == "Enumeration" else 0,
                f"{name} + {child_start}U" if child_count else "NULL", child_count))
        if rows:
            lines.append(f"static const kmipkit_fixture_item_t {name}[] = {{\n" + ",\n".join(rows) + "\n};")
        else:
            lines.append(f"static const kmipkit_fixture_item_t {name}[1] = {{ {{ 0U, 0U, NULL, 0, 0U, NULL, 0U }} }};")
    def emit_schema(schema: Schema, name: str) -> None:
        child_names = []
        for index, child in enumerate(schema.children):
            nested = f"{name}_schema_{index}"
            emit_schema(child.schema, nested)
            child_names.append((child, nested))
        if child_names:
            rows = ["    { UINT32_C(0x%06X), %dU, %s, INT64_C(%d), INT64_C(%d), %s, &%s }" % (c.tag, type_values[c.schema.item_type], _cpp_string(c.schema.text) if c.schema.text else "NULL", c.schema.minimum, c.schema.maximum, "true" if c.schema.has_range else "false", nested) for c, nested in child_names]
            rules = f"{name}_rules"
            lines.append(f"static const kmipkit_fixture_child_rule_t {rules}[] = {{\n" + ",\n".join(rows) + "\n};")
            lines.append(f"static const kmipkit_fixture_schema_t {name} = {{ {type_values[schema.item_type]}U, NULL, 0, 0, false, {rules}, {len(child_names)}U }};")
        else:
            lines.append(f"static const kmipkit_fixture_schema_t {name} = {{ {type_values[schema.item_type]}U, {_cpp_string(schema.text) if schema.text else 'NULL'}, INT64_C({schema.minimum}), INT64_C({schema.maximum}), {'true' if schema.has_range else 'false'}, NULL, 0U }};")
    for index, definition in enumerate(definitions):
        emit_schema(definition.schema, f"kmipkit_fixture_schema_{index}")
        path = ", ".join(f"UINT32_C(0x{tag:06X})" for tag in definition.path)
        lines.append(f"static const uint32_t kmipkit_fixture_path_{index}[] = {{ {path} }};")
    lines.append("static const kmipkit_fixture_definition_t kmipkit_fixture_definitions[] = {")
    for index, definition in enumerate(definitions):
        discr_value = _cpp_string(definition.discriminator_text) if definition.discriminator_text else "NULL"
        lines.append(f"    {{ {_cpp_string(definition.fixture_id)}, {_cpp_string(definition.vendor)}, {_cpp_string(definition.name)}, {_cpp_string(definition.version)}, kmipkit_fixture_path_{index}, {len(definition.path)}U, {type_values[definition.discriminator_type]}U, {discr_value}, INT64_C({definition.discriminator_number}), &kmipkit_fixture_schema_{index} }},")
    lines.append("};")
    for index, case in enumerate(cases):
        emit_item_array(case.payload, f"kmipkit_fixture_payload_{index}")
        ids = ", ".join(_cpp_string(value) for value in case.matched_ids) or "NULL"
        if case.matched_ids:
            lines.append(f"static const char *const kmipkit_fixture_matched_{index}[] = {{ {ids} }};")
        if case.outbound_attachments:
            attachments = ", ".join(
                f"{{ {_cpp_string(attachment.fixture_id)}, {'true' if attachment.criticality_indicator else 'false'} }}"
                for attachment in case.outbound_attachments
            )
            lines.append(
                f"static const kmipkit_fixture_attachment_t kmipkit_fixture_attachments_{index}[] = {{ {attachments} }};"
            )
    lines.append("static const kmipkit_fixture_case_t kmipkit_fixture_cases[] = {")
    for index, case in enumerate(cases):
        lines.append(f"    {{ {_cpp_string(case.fixture_id)}, {_cpp_string(case.vendor)}, {'true' if case.critical else 'false'}, kmipkit_fixture_payload_{index}, {len(case.payload)}U, {_cpp_string(case.outcome)}, {'kmipkit_fixture_matched_' + str(index) if case.matched_ids else 'NULL'}, {len(case.matched_ids)}U, {'true' if case.typed else 'false'} }},")
    lines.append("};")
    outbound_cases = [
        (index, case) for index, case in enumerate(cases) if case.outbound_attachments
    ]
    if outbound_cases:
        lines.append("static const kmipkit_fixture_outbound_request_t kmipkit_fixture_outbound_requests[] = {")
        for index, case in outbound_cases:
            lines.append(
                f"    {{ {_cpp_string(case.fixture_id)}, {_cpp_string(case.outbound_outcome)}, kmipkit_fixture_attachments_{index}, {len(case.outbound_attachments)}U }},"
            )
        lines.append("};")
    else:
        lines.append(
            "static const kmipkit_fixture_outbound_request_t kmipkit_fixture_outbound_requests[1] = { { NULL, NULL, NULL, 0U } };"
        )
    lines.extend([f"#define KMIPKIT_FIXTURE_CASE_COUNT {len(cases)}U", f"#define KMIPKIT_FIXTURE_OUTBOUND_REQUEST_COUNT {len(outbound_cases)}U", f"#define KMIPKIT_FIXTURE_DEFINITION_COUNT {len(definitions)}U", "#endif"])
    return "\n".join(lines) + "\n"


def _java_literal(value: str) -> str:
    return json.dumps(value, ensure_ascii=True)


def _java_item(item: Item, values: dict[str, int]) -> str:
    children = "List.of(" + ", ".join(_java_item(child, values) for child in item.children) + ")"
    return f"new Item(0x{item.tag:06X}, {values[item.item_type]}, {_java_literal(item.text)}, {item.number}L, {children})"


def _java_schema(schema: Schema, values: dict[str, int]) -> str:
    rules = "List.of(" + ", ".join(f"new ChildRule(0x{rule.tag:06X}, {_java_schema(rule.schema, values)})" for rule in schema.children) + ")"
    return f"new Schema({values[schema.item_type]}, {_java_literal(schema.text)}, {str(schema.has_range).lower()}, {schema.minimum}L, {schema.maximum}L, {rules})"


def _generate_java(manifest: dict[str, Any], definitions: tuple[Definition, ...], cases: tuple[Case, ...]) -> str:
    values = manifest["item_types"]
    lines = [f"// Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: {manifest['fingerprint']}. Do not edit.", "package org.kmipkit;", "", "import java.util.List;", "", "final class SharedExtensionFixtures {", "    record Item(int tag, int type, String text, long number, List<Item> children) {}", "    record ChildRule(int tag, Schema schema) {}", "    record Schema(int type, String text, boolean hasRange, long minimum, long maximum, List<ChildRule> children) {}", "    record Definition(String id, String vendor, String name, String version, List<Integer> path, int discriminatorType, String discriminatorText, long discriminatorNumber, Schema schema) {}", "    record Case(String id, String vendor, boolean critical, List<Item> payload, String outcome, List<String> matchedIds, boolean typed) {}", "    static final List<Definition> DEFINITIONS = List.of("]
    lines[11:11] = [
        "    record Attachment(String fixtureId, boolean criticalityIndicator) {}",
        "    record OutboundRequest(String fixtureId, String outcome, List<Attachment> attachments) {}",
    ]
    for index, definition in enumerate(definitions):
        path = "List.of(" + ", ".join(f"0x{tag:06X}" for tag in definition.path) + ")"
        comma = "," if index + 1 < len(definitions) else ""
        lines.append(f"        new Definition({_java_literal(definition.fixture_id)}, {_java_literal(definition.vendor)}, {_java_literal(definition.name)}, {_java_literal(definition.version)}, {path}, {values[definition.discriminator_type]}, {_java_literal(definition.discriminator_text)}, {definition.discriminator_number}L, {_java_schema(definition.schema, values)}){comma}")
    lines.extend(["    );", "    static final List<OutboundRequest> OUTBOUND_REQUESTS = List.of("])
    outbound_cases = [case for case in cases if case.outbound_attachments]
    for index, case in enumerate(outbound_cases):
        attachments = "List.of(" + ", ".join(
            f"new Attachment({_java_literal(attachment.fixture_id)}, {str(attachment.criticality_indicator).lower()})"
            for attachment in case.outbound_attachments
        ) + ")"
        comma = "," if index + 1 < len(outbound_cases) else ""
        lines.append(
            f"        new OutboundRequest({_java_literal(case.fixture_id)}, {_java_literal(case.outbound_outcome)}, {attachments}){comma}"
        )
    lines.extend(["    );", "    static final List<Case> CASES = List.of("])
    for index, case in enumerate(cases):
        payload = "List.of(" + ", ".join(_java_item(item, values) for item in case.payload) + ")"
        ids = "List.of(" + ", ".join(_java_literal(value) for value in case.matched_ids) + ")"
        comma = "," if index + 1 < len(cases) else ""
        lines.append(f"        new Case({_java_literal(case.fixture_id)}, {_java_literal(case.vendor)}, {str(case.critical).lower()}, {payload}, {_java_literal(case.outcome)}, {ids}, {str(case.typed).lower()}){comma}")
    lines.extend(["    );", "    private SharedExtensionFixtures() {}", "}"])
    return "\n".join(lines) + "\n"


def _rust_item(item: Item) -> str:
    children = "&[" + ", ".join(_rust_item(child) for child in item.children) + "]"
    value = _rust_string(item.text) if item.item_type == "TextString" else _rust_integer_literal(item.number)
    return f"Item {{ tag: {_rust_tag_literal(item.tag)}, kind: ItemType::{item.item_type}, text: {value if item.item_type == 'TextString' else '""'}, number: {_rust_integer_literal(item.number)}, children: {children} }}"


def _rust_schema(schema: Schema) -> str:
    children = "&[" + ", ".join(f"ChildRule {{ tag: {_rust_tag_literal(rule.tag)}, schema: {_rust_schema(rule.schema)} }}" for rule in schema.children) + "]"
    return f"Schema {{ kind: ItemType::{schema.item_type}, has_range: {str(schema.has_range).lower()}, minimum: {_rust_integer_literal(schema.minimum)}, maximum: {_rust_integer_literal(schema.maximum)}, children: {children} }}"


def _generate_rust(manifest: dict[str, Any], definitions: tuple[Definition, ...], cases: tuple[Case, ...]) -> str:
    lines = [
        f"// Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: {manifest['fingerprint']}. Do not edit.",
        "use kmipkit_ttlv::ItemType;",
        "",
        "#[derive(Clone, Copy)]",
        "#[rustfmt::skip]",
        "pub struct Item { pub tag: u32, pub kind: ItemType, pub text: &'static str, pub number: i64, pub children: &'static [Item] }",
        "#[derive(Clone, Copy)]",
        "#[rustfmt::skip]",
        "pub struct ChildRule { pub tag: u32, pub schema: Schema }",
        "#[derive(Clone, Copy)]",
        "#[rustfmt::skip]",
        "pub struct Schema { pub kind: ItemType, pub has_range: bool, pub minimum: i64, pub maximum: i64, pub children: &'static [ChildRule] }",
        "#[rustfmt::skip]",
        "pub struct Definition { pub id: &'static str, pub vendor: &'static str, pub name: &'static str, pub version: &'static str, pub path: &'static [u32], pub discriminator_type: ItemType, pub discriminator_text: &'static str, pub discriminator_number: i64, pub schema: Schema }",
        "#[rustfmt::skip]",
        "pub struct Case { pub id: &'static str, pub vendor: &'static str, pub critical: bool, pub payload: &'static [Item], pub outcome: &'static str, pub matched_ids: &'static [&'static str], pub typed: bool }",
        "#[derive(Clone, Copy)]",
        "#[rustfmt::skip]",
        "pub struct OutboundAttachment { pub fixture_id: &'static str, pub criticality_indicator: bool }",
        "#[derive(Clone, Copy)]",
        "#[rustfmt::skip]",
        "pub struct OutboundRequest { pub fixture_id: &'static str, pub outcome: &'static str, pub attachments: &'static [OutboundAttachment] }",
        "",
        "#[rustfmt::skip]",
        "pub static DEFINITIONS: &[Definition] = &[",
    ]
    for definition in definitions:
        path = "&[" + ", ".join(_rust_tag_literal(tag) for tag in definition.path) + "]"
        lines.append(f"    Definition {{ id: {_rust_string(definition.fixture_id)}, vendor: {_rust_string(definition.vendor)}, name: {_rust_string(definition.name)}, version: {_rust_string(definition.version)}, path: {path}, discriminator_type: ItemType::{definition.discriminator_type}, discriminator_text: {_rust_string(definition.discriminator_text)}, discriminator_number: {_rust_integer_literal(definition.discriminator_number)}, schema: {_rust_schema(definition.schema)} }},")
    lines.extend(["];", "", "#[rustfmt::skip]", "pub static CASES: &[Case] = &["])
    for case in cases:
        items = "&[" + ", ".join(_rust_item(item) for item in case.payload) + "]"
        ids = "&[" + ", ".join(_rust_string(value) for value in case.matched_ids) + "]"
        lines.append(f"    Case {{ id: {_rust_string(case.fixture_id)}, vendor: {_rust_string(case.vendor)}, critical: {str(case.critical).lower()}, payload: {items}, outcome: {_rust_string(case.outcome)}, matched_ids: {ids}, typed: {str(case.typed).lower()} }},")
    lines.extend(["];"])
    lines.extend(["", "#[rustfmt::skip]", "pub static OUTBOUND_REQUESTS: &[OutboundRequest] = &["])
    for case in cases:
        if not case.outbound_attachments:
            continue
        attachments = "&[" + ", ".join(
            f"OutboundAttachment {{ fixture_id: {_rust_string(attachment.fixture_id)}, criticality_indicator: {str(attachment.criticality_indicator).lower()} }}"
            for attachment in case.outbound_attachments
        ) + "]"
        lines.append(
            f"    OutboundRequest {{ fixture_id: {_rust_string(case.fixture_id)}, outcome: {_rust_string(case.outbound_outcome)}, attachments: {attachments} }},"
        )
    lines.extend(["];", ""])
    return "\n".join(lines)


def render(api: Any, corpus: Any) -> dict[Path, str]:
    manifest, definitions, cases = _parse_inputs(api, corpus)
    renderers = {"c": _generate_c, "java": _generate_java, "rust": _generate_rust}
    return {
        path: renderers[language](manifest, definitions, cases)
        for path, language in OUTPUTS.items()
    }


def _read(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise FixtureError(f"cannot read valid UTF-8 JSON from {path.relative_to(ROOT)}") from error


def _is_reparse_point(path: Path) -> bool:
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return False
    if stat.S_ISLNK(metadata.st_mode) or getattr(path, "is_junction", lambda: False)():
        return True
    return bool(getattr(metadata, "st_file_attributes", 0) & 0x400)


def _safe_destination(root: Path, destination: Path) -> Path:
    try:
        relative = destination.relative_to(root)
    except ValueError as error:
        raise FixtureError("generated destination escapes the repository root") from error
    if relative.is_absolute() or any(part in ("", ".", "..") for part in relative.parts):
        raise FixtureError("generated destination is not a safe relative path")

    current = root
    for index, part in enumerate(relative.parts):
        current = current / part
        if _is_reparse_point(current):
            raise FixtureError("generated destination traverses a symbolic link or reparse point")
        if current.exists() and index < len(relative.parts) - 1 and not current.is_dir():
            raise FixtureError("generated destination parent is not a directory")

    try:
        current.resolve(strict=False).relative_to(root)
    except (OSError, ValueError) as error:
        raise FixtureError("generated destination escapes the repository root") from error
    if current.exists() and not current.is_file():
        raise FixtureError("generated destination is not a regular file")
    return current


def _preflight_outputs(root: Path, outputs: dict[Path, str]) -> dict[Path, Path]:
    return {path: _safe_destination(root, path) for path in outputs}


def _atomic_write(root: Path, destination: Path, contents: str) -> None:
    _safe_destination(root, destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    _safe_destination(root, destination)
    temporary_name: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            "wb", dir=destination.parent, prefix=".kmipkit-extension-fixture-", delete=False
        ) as temporary:
            temporary_name = temporary.name
            temporary.write(contents.encode("utf-8"))
            temporary.flush()
            os.fsync(temporary.fileno())
        _safe_destination(root, destination)
        os.replace(temporary_name, destination)
        temporary_name = None
    finally:
        if temporary_name is not None:
            try:
                os.unlink(temporary_name)
            except FileNotFoundError:
                pass


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated files are stale")
    args = parser.parse_args(argv)
    try:
        outputs = render(_read(MANIFEST), _read(CORPUS))
        root = ROOT.resolve(strict=True)
        destinations = _preflight_outputs(root, outputs)
        stale = []
        for path, contents in outputs.items():
            if args.check:
                try:
                    current = destinations[path].read_text(encoding="utf-8")
                except (OSError, UnicodeError):
                    stale.append(path.relative_to(ROOT).as_posix())
                    continue
                if current != contents:
                    stale.append(path.relative_to(ROOT).as_posix())
            else:
                _atomic_write(root, destinations[path], contents)
        if stale:
            print("stale extension fixture outputs: " + ", ".join(stale), file=sys.stderr)
            return 1
        return 0
    except FixtureError as error:
        print(f"extension fixture generation failed: {error}", file=sys.stderr)
        return 2
    except OSError:
        print("extension fixture generation failed: repository I/O failed", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
