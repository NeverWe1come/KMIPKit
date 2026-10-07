"""Python adapter parity tests for the KMIP 2.1 vendor extension registry.

Traceability: KMIPKIT-0012-FR-001, FR-005 through FR-012, and SC-001
through SC-007. Wire preservation follows KMIP 2.1 §8.3, Table 396;
§9.13, Table 418; §7.13, Table 365; and §11.44, Table 476.
"""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from typing import Any

from kmipkit import errors, ttlv
from kmipkit import extensions as registry_api


ROOT = Path(__file__).resolve().parents[3]
FIXTURE_PATH = ROOT / "tests" / "fixtures" / "extensions" / "cases.json"
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
VENDOR_IDENTIFIER = "example.vendor"


def _read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _tag(raw_tag: str) -> Any:
    raw = ttlv.raw_tag_from_raw(int(raw_tag, 16))
    return ttlv.raw_tag_try_checked(raw)


def _item_type(name: str) -> Any:
    return getattr(ttlv.ItemType, name)


def _codec_limits() -> Any:
    return ttlv.codec_limits_defaults()


def _value_from_fixture(item: dict[str, Any]) -> Any:
    item_type = item["type"]
    if item_type == "Structure":
        return ttlv.ttlv_value_structure(
            _structure_from_fixture(item["children"]), _codec_limits()
        )
    value = item["value"]
    if item_type == "Integer":
        return ttlv.ttlv_value_integer(value)
    if item_type == "LongInteger":
        return ttlv.ttlv_value_long_integer(value)
    if item_type == "Enumeration":
        return ttlv.ttlv_value_enumeration(value)
    if item_type == "Boolean":
        return ttlv.ttlv_value_boolean(value)
    if item_type in {"TextString", "ByteString", "BigInteger"}:
        raw = value.encode("utf-8") if isinstance(value, str) else bytes(value)
        constructor = {
            "TextString": ttlv.ttlv_value_text_string,
            "ByteString": ttlv.ttlv_value_byte_string,
            "BigInteger": ttlv.ttlv_value_big_integer,
        }[item_type]
        return constructor(raw, _codec_limits())
    if item_type in {"DateTime", "DateTimeExtended"}:
        if item_type == "DateTime":
            return ttlv.ttlv_value_date_time(value)
        return ttlv.ttlv_value_date_time_extended(value)
    if item_type == "Interval":
        return ttlv.ttlv_value_interval(value)
    raise AssertionError(f"unsupported shared fixture Item Type: {item_type}")


def _structure_from_fixture(children: list[dict[str, Any]]) -> Any:
    structure = ttlv.ttlv_structure_create()
    limits = _codec_limits()
    for child in children:
        item = ttlv.ttlv_item_create(
            _tag(child["tag"]), _value_from_fixture(child), limits
        )
        structure = ttlv.ttlv_structure_with_item(structure, item, limits)
    return structure


def _fixture_child_schemas(children_schema: dict[str, Any]) -> list[dict[str, Any]]:
    if "contains" in children_schema:
        return [children_schema["contains"]]
    return [entry["contains"] for entry in children_schema.get("allOf", []) if "contains" in entry]


def _schema_from_fixture_node(node_schema: dict[str, Any]) -> Any:
    properties = node_schema["properties"]
    item_type = properties["type"]["const"]
    if item_type == "Structure":
        children_schema = properties["children"]
        rules = []
        for child_schema in _fixture_child_schemas(children_schema):
            child_properties = child_schema["properties"]
            child_tag = _tag(child_properties["tag"]["const"])
            child = _schema_from_fixture_node(child_schema)
            rules.append(registry_api.required_child_rule(child_tag, child))
        return registry_api.structure_schema(rules, [], True)

    schema = registry_api.scalar_schema(_item_type(item_type))
    value_schema = properties.get("value", {})
    if "minimum" in value_schema and "maximum" in value_schema:
        if item_type in {"Integer", "LongInteger", "DateTime", "Interval", "DateTimeExtended"}:
            schema = registry_api.with_signed_range(
                schema, value_schema["minimum"], value_schema["maximum"]
            )
        else:
            schema = registry_api.with_unsigned_range(
                schema, value_schema["minimum"], value_schema["maximum"]
            )
    return schema


def _definition_from_fixture(record: dict[str, Any]) -> Any:
    identity = registry_api.create_extension_identity(
        record["vendorIdentification"], record["extensionName"], record["extensionVersion"]
    )
    compatibility = registry_api.create_compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
    discriminator = record["discriminator"]
    path = registry_api.create_ttlv_path(_tag(discriminator["path"][0]))
    for raw_tag in discriminator["path"][1:]:
        path = registry_api.with_child_tag(path, _tag(raw_tag))
    scalar = ttlv.ttlv_value_text_string(
        discriminator["value"].encode("utf-8"), _codec_limits()
    )
    schema = _schema_from_fixture_node(record["payloadSchema"])
    definition = registry_api.create_extension_definition(
        identity,
        compatibility,
        registry_api.create_discriminator(path, scalar),
        schema,
    )
    return registry_api.with_extension_information(
        definition, registry_api.create_extension_information(record["extensionName"])
    )


def _definition_map(corpus: dict[str, Any]) -> dict[str, Any]:
    return {record["id"]: _definition_from_fixture(record) for record in corpus["definitions"]}


def _definition_with_enum_constraints() -> Any:
    identity = registry_api.create_extension_identity(VENDOR_IDENTIFIER, "limited", "1")
    compatibility = registry_api.create_compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
    discriminator_tag = _tag("0x420001")
    discriminator_path = registry_api.create_ttlv_path(discriminator_tag)
    discriminator = registry_api.create_discriminator(
        discriminator_path,
        ttlv.ttlv_value_text_string(b"limited-v1", _codec_limits()),
    )
    text_schema = registry_api.scalar_schema(_item_type("TextString"))
    enum_schema = registry_api.scalar_schema(_item_type("Enumeration"))
    enum_schema = registry_api.with_allowed_enumeration(enum_schema, 1)
    enum_schema = registry_api.with_allowed_enumeration(enum_schema, 2)
    schema = registry_api.structure_schema(
        [
            registry_api.required_child_rule(discriminator_tag, text_schema),
            registry_api.required_child_rule(_tag("0x420002"), enum_schema),
        ],
        [],
        True,
    )
    return registry_api.create_extension_definition(
        identity, compatibility, discriminator, schema
    )


def _normalize_structure(view: Any) -> dict[str, Any]:
    children = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        tag_value = ttlv.tag_value(ttlv.ttlv_item_view_tag(item))
        item_type = ttlv.ttlv_item_view_type(item)
        item_type_name = item_type.name
        value_view = ttlv.ttlv_item_view_value(item)
        normalized: dict[str, Any] = {
            "tag": f"0x{tag_value:06X}",
            "type": item_type_name,
        }
        if item_type_name == "Structure":
            normalized["children"] = _normalize_structure(
                ttlv.ttlv_value_view_structure(value_view)
            )["children"]
        elif item_type_name in {"TextString", "ByteString", "BigInteger"}:
            raw = bytes(
                ttlv.ttlv_value_view_byte_at(value_view, byte_index)
                for byte_index in range(ttlv.ttlv_value_view_byte_length(value_view))
            )
            normalized["value"] = (
                raw.decode("utf-8") if item_type_name == "TextString" else list(raw)
            )
        elif item_type_name in {"Integer", "Interval"}:
            normalized["value"] = ttlv.ttlv_value_view_integer(value_view)
        elif item_type_name in {"LongInteger", "DateTime", "DateTimeExtended"}:
            getter = {
                "LongInteger": ttlv.ttlv_value_view_long_integer,
                "DateTime": ttlv.ttlv_value_view_date_time,
                "DateTimeExtended": ttlv.ttlv_value_view_date_time_extended,
            }[item_type_name]
            normalized["value"] = getter(value_view)
        elif item_type_name == "Enumeration":
            normalized["value"] = ttlv.ttlv_value_view_enumeration(value_view)
        elif item_type_name == "Boolean":
            normalized["value"] = ttlv.ttlv_value_view_boolean(value_view)
        else:
            raise AssertionError(f"unsupported shared fixture Item Type: {item_type_name}")
        children.append(normalized)
    return {"type": "Structure", "children": children}


def _limit_values(manifest: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, int]]:
    records = manifest["limits"]
    return records, {record["pythonField"]: record["default"] for record in records}


def _create_limits(values: dict[str, int], records: list[dict[str, Any]]) -> Any:
    return registry_api.create_extension_registry_limits(
        *(values[record["pythonField"]] for record in records)
    )


class ExtensionRegistryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.corpus = _read_json(FIXTURE_PATH)
        cls.manifest = _read_json(MANIFEST_PATH)
        cls.fixtures = {item["id"]: item for item in cls.corpus["cases"]}
        cls.definitions = _definition_map(cls.corpus)

    def test_registry_registration_is_immutable_owned_by_configuration_and_isolated(self) -> None:
        defaults = registry_api.default_extension_registry_limits()
        alpha_source = [self.definitions["known.alpha"]]
        beta_source = [self.definitions["ambiguous.beta"]]
        alpha = registry_api.create_client_extension_registry(alpha_source, defaults)
        beta = registry_api.create_client_extension_registry(beta_source, defaults)
        alpha_configuration = registry_api.create_client_configuration(alpha)
        beta_configuration = registry_api.create_client_configuration(beta)

        alpha_source.clear()
        alpha.close()
        beta.close()
        alpha_owned = registry_api.client_configuration_extension_registry(alpha_configuration)
        beta_owned = registry_api.client_configuration_extension_registry(beta_configuration)
        try:
            self.assertEqual(registry_api.definition_count(alpha_owned), 1)
            self.assertEqual(registry_api.definition_count(beta_owned), 1)
            self.assertIsNotNone(
                registry_api.definition_for_identity(
                    alpha_owned,
                    registry_api.create_extension_identity(VENDOR_IDENTIFIER, "alpha", "1"),
                )
            )
            self.assertIsNone(
                registry_api.definition_for_identity(
                    alpha_owned,
                    registry_api.create_extension_identity(
                        VENDOR_IDENTIFIER, "ambiguous-beta", "1"
                    ),
                )
            )
            self.assertIsNotNone(
                registry_api.definition_for_identity(
                    beta_owned,
                    registry_api.create_extension_identity(
                        VENDOR_IDENTIFIER, "ambiguous-beta", "1"
                    ),
                )
            )
        finally:
            alpha_owned.close()
            beta_owned.close()
            alpha_configuration.close()
            beta_configuration.close()

    def test_registry_limit_defaults_and_every_hard_boundary_match_the_manifest(self) -> None:
        records, defaults = _limit_values(self.manifest)
        fixture_limits = self.corpus["limitCases"]
        self.assertEqual(
            [item["id"] for item in fixture_limits],
            [item["id"] for item in records],
        )
        actual_defaults = registry_api.default_extension_registry_limits()
        for record, fixture in zip(records, fixture_limits, strict=True):
            field = record["pythonField"]
            with self.subTest(limit=record["id"], profile="default"):
                self.assertEqual(getattr(actual_defaults, field), record["default"])
                self.assertEqual(fixture["default"], record["default"])
                self.assertEqual(fixture["hardMaximum"], record["hardMaximum"])

        for record, fixture in zip(records, fixture_limits, strict=True):
            field = record["pythonField"]
            default = record["default"]
            hard_maximum = record["hardMaximum"]
            probes = {probe["profile"]: probe for probe in fixture["probes"]}
            expected_profiles = {"default", "lowered", "hard", "over-hard"}
            if default < hard_maximum:
                expected_profiles.add("raised-default")
            self.assertEqual(set(probes), expected_profiles)
            for profile, probe in probes.items():
                selected = probe["value"]
                expected_outcome = "resource_limit" if profile == "over-hard" else "accepted"
                values = dict(defaults)
                values[field] = selected
                with self.subTest(limit=record["id"], profile=profile):
                    self.assertEqual(probe["expectedOutcome"], expected_outcome)
                    self.assertEqual(probe["expectedByAdapter"]["python"], expected_outcome)
                    if expected_outcome == "resource_limit":
                        with self.assertRaises(errors.ResourceLimitError):
                            _create_limits(values, records)
                    else:
                        limits = _create_limits(values, records)
                        self.assertEqual(getattr(limits, field), selected)

    def test_shared_inbound_fixtures_are_inspected_and_generic_ttlv_is_preserved(self) -> None:
        registry = registry_api.create_client_extension_registry(
            list(self.definitions.values()), registry_api.default_extension_registry_limits()
        )
        codec_limits = _codec_limits()
        try:
            expected_recognition = {
                "valid-recognized": True,
                "synthetic-secret-bearing": True,
                "invalid-schema": False,
                "multiply-matching": False,
                "unknown-preserved": False,
            }
            for case_id, expected_recognized in expected_recognition.items():
                fixture = self.fixtures[case_id]
                extension = fixture["extension"]
                with self.subTest(fixture=case_id):
                    recognition = registry_api.inspect_extension(
                        registry,
                        extension["vendorIdentification"],
                        _structure_from_fixture(extension["payload"]["children"]),
                        codec_limits,
                    )
                    try:
                        self.assertEqual(
                            registry_api.is_recognized(recognition), expected_recognized
                        )
                        validated = registry_api.validated_value(recognition)
                        self.assertEqual(
                            validated is not None,
                            fixture["expected"]["typedValueAvailable"],
                        )
                        generic_view = registry_api.extension_recognition_generic_value(
                            recognition
                        )
                        self.assertEqual(
                            _normalize_structure(generic_view),
                            fixture["expected"]["preservedPayload"],
                        )
                        if case_id == "valid-recognized":
                            preserved = _normalize_structure(generic_view)
                            # KMIP 2.1 §11.56 and ADR-0010 require preserving this tag
                            # even though the registered schema does not declare it.
                            self.assertEqual(
                                [child["tag"] for child in preserved["children"]],
                                [
                                    "0x420001",
                                    "0x420002",
                                    "0x420004",
                                    "0x420006",
                                    "0x540001",
                                ],
                            )
                        if validated is not None:
                            self.assertEqual(
                                _normalize_structure(
                                    registry_api.validated_extension_value_generic_value(validated)
                                ),
                                fixture["expected"]["preservedPayload"],
                            )
                    finally:
                        recognition.close()
        finally:
            registry.close()

    def test_registry_list_map_and_extension_information_metadata_are_deterministic(self) -> None:
        information = registry_api.create_extension_information("alpha")
        information = registry_api.with_tag(information, 0x00540001)
        information = registry_api.with_type(information, _item_type("TextString"))
        information = registry_api.with_enumeration(information, 17)
        information = registry_api.with_attribute(information, False)
        information = registry_api.with_parent_structure_tag(information, 0x00540002)
        information = registry_api.with_description(information, "local metadata")
        metadata_structure = registry_api.extension_information_to_ttlv(information)
        self.assertEqual(
            _normalize_structure(ttlv.ttlv_structure_view(metadata_structure)),
            {
                "type": "Structure",
                "children": [
                    {"tag": "0x4200A5", "type": "TextString", "value": "alpha"},
                    {"tag": "0x4200A6", "type": "Integer", "value": 0x00540001},
                    {"tag": "0x4200A7", "type": "Enumeration", "value": 7},
                    {"tag": "0x4200A8", "type": "Integer", "value": 17},
                    {"tag": "0x42012A", "type": "Boolean", "value": False},
                    {"tag": "0x42012B", "type": "Integer", "value": 0x00540002},
                    {"tag": "0x42012C", "type": "TextString", "value": "local metadata"},
                ],
            },
        )
        alpha = registry_api.with_extension_information(
            self.definitions["known.alpha"], information
        )
        definitions = [self.definitions["ambiguous.beta"], alpha]
        limits = registry_api.default_extension_registry_limits()
        forward = registry_api.create_client_extension_registry(definitions, limits)
        reverse = registry_api.create_client_extension_registry(
            list(reversed(definitions)), limits
        )
        try:
            def metadata_names(registry: Any) -> list[str]:
                names = []
                for index in range(registry_api.definition_count(registry)):
                    definition = registry_api.definition_at(registry, index)
                    identity = registry_api.extension_definition_identity(definition)
                    names.append(identity.name)
                    self.assertIsNotNone(
                        registry_api.definition_for_identity(registry, identity)
                    )
                    information = registry_api.extension_definition_information(definition)
                    self.assertIsNotNone(information)
                    encoded = registry_api.extension_information_to_ttlv(information)
                    if identity.name == "alpha":
                        self.assertEqual(
                            _normalize_structure(ttlv.ttlv_structure_view(encoded)),
                            _normalize_structure(ttlv.ttlv_structure_view(metadata_structure)),
                        )
                return names

            self.assertEqual(metadata_names(forward), ["alpha", "ambiguous-beta"])
            self.assertEqual(metadata_names(reverse), ["alpha", "ambiguous-beta"])
        finally:
            forward.close()
            reverse.close()

    def test_explicit_criticality_is_preserved_for_ordered_outbound_extensions(self) -> None:
        registry = registry_api.create_client_extension_registry(
            list(self.definitions.values()), registry_api.default_extension_registry_limits()
        )
        limits = _codec_limits()
        try:
            outbound = self.fixtures["synthetic-secret-bearing"]["expected"]
            outbound = outbound["outboundRequest"]["attachments"]
            message_extensions = []
            caller_selections = []
            for attachment in outbound:
                fixture = self.fixtures[attachment["fixtureId"]]
                definition_id = fixture["expected"]["matchedDefinitionIds"][0]
                identity = registry_api.extension_definition_identity(
                    self.definitions[definition_id]
                )
                value = registry_api.validate_extension_value(
                    registry,
                    identity,
                        _structure_from_fixture(
                            fixture["extension"]["payload"]["children"]
                        ),
                    limits,
                )
                caller_selections.append(
                    (attachment["fixtureId"], attachment["criticalityIndicator"])
                )
                message_extensions.append(
                    registry_api.create_client_request_message_extension(
                        value, attachment["criticalityIndicator"]
                    )
                )

            self.assertEqual(
                caller_selections,
                [("synthetic-secret-bearing", False), ("valid-recognized", True)],
            )
            self.assertEqual(
                len(message_extensions),
                len(outbound),
                "one request-extension wrapper is built for each ordered fixture attachment",
            )
        finally:
            registry.close()

    def test_discover_versions_batch_item_preserves_extension_identity_and_order(self) -> None:
        limits = _codec_limits()
        registry = registry_api.create_client_extension_registry(
            [self.definitions["known.alpha"], self.definitions["ambiguous.beta"]],
            registry_api.default_extension_registry_limits(),
        )
        item = None
        identities = []
        try:
            alpha_identity = registry_api.extension_definition_identity(
                self.definitions["known.alpha"]
            )
            beta_identity = registry_api.extension_definition_identity(
                self.definitions["ambiguous.beta"]
            )
            identities.extend((alpha_identity, beta_identity))
            alpha_value = registry_api.validate_extension_value(
                registry,
                alpha_identity,
                _structure_from_fixture(
                    self.fixtures["valid-recognized"]["extension"]["payload"]["children"]
                ),
                limits,
            )
            beta_value = registry_api.validate_extension_value(
                registry,
                beta_identity,
                _structure_from_fixture(
                    self.fixtures["multiply-matching"]["extension"]["payload"]["children"]
                ),
                limits,
            )
            non_critical = registry_api.create_client_request_message_extension(
                alpha_value, False
            )
            critical = registry_api.create_client_request_message_extension(
                beta_value, True
            )

            item = registry_api.client_batch_item_discover_versions()
            item = registry_api.with_extension(item, non_critical)
            item = registry_api.with_extension(item, critical)

            self.assertEqual(registry_api.client_batch_item_extension_count(item), 2)
            for index, expected in enumerate((alpha_identity, beta_identity)):
                actual = registry_api.client_batch_item_extension_identity_at(item, index)
                self.assertIsNotNone(actual)
                identities.append(actual)
                self.assertEqual(actual.vendor_identifier, expected.vendor_identifier)
                self.assertEqual(actual.name, expected.name)
                self.assertEqual(actual.version, expected.version)
                self.assertEqual(
                    registry_api.client_batch_item_extension_criticality_indicator_at(
                        item, index
                    ),
                    index == 1,
                )

            with self.assertRaises(errors.InvalidInputError):
                registry_api.client_batch_item_extension_identity_at(item, 2)
            with self.assertRaises(errors.InvalidInputError):
                registry_api.client_batch_item_extension_criticality_indicator_at(item, 2)
        finally:
            for identity in identities:
                identity.close()
            if item is not None:
                item.close()
            registry.close()
            limits.close()

    def test_registry_construction_enforces_schema_and_metadata_limits(self) -> None:
        records, defaults = _limit_values(self.manifest)
        definition = self.definitions["known.alpha"]
        failing_profiles = {
            "max_definitions": 0,
            "max_schema_nodes": 1,
            "max_child_rules_per_structure": 1,
            "max_text_bytes_per_field": 1,
            "max_registry_text_bytes": 1,
            "max_discriminator_scalar_bytes": 1,
            "max_total_discriminator_scalar_bytes": 1,
            "max_depth": 0,
        }
        for field, selected in failing_profiles.items():
            values = dict(defaults)
            values[field] = selected
            with self.subTest(limit=field):
                with self.assertRaises(errors.ResourceLimitError):
                    registry_api.create_client_extension_registry(
                        [definition], _create_limits(values, records)
                    )

        constrained = _definition_with_enum_constraints()
        for field in ("max_constraint_members_per_rule", "max_total_constraint_members"):
            values = dict(defaults)
            values[field] = 1
            with self.subTest(limit=field):
                with self.assertRaises(errors.ResourceLimitError):
                    registry_api.create_client_extension_registry(
                        [constrained], _create_limits(values, records)
                    )

    def test_lower_lookup_and_payload_index_budgets_return_resource_limit_errors(self) -> None:
        records, defaults = _limit_values(self.manifest)
        fixture = self.fixtures["valid-recognized"]["extension"]
        payload = _structure_from_fixture(fixture["payload"]["children"])
        definition = self.definitions["known.alpha"]
        for field in ("max_payload_index_records", "max_lookup_comparisons"):
            values = dict(defaults)
            values[field] = 1
            with self.subTest(limit=field):
                registry = registry_api.create_client_extension_registry(
                    [definition], _create_limits(values, records)
                )
                try:
                    with self.assertRaises(errors.ResourceLimitError):
                        registry_api.inspect_extension(
                            registry, fixture["vendorIdentification"], payload, _codec_limits()
                        )
                finally:
                    registry.close()

    def test_closed_registry_context_manager_rejects_use_with_invalid_input(self) -> None:
        registry = registry_api.create_client_extension_registry(
            [self.definitions["known.alpha"]], registry_api.default_extension_registry_limits()
        )
        with registry as active_registry:
            self.assertEqual(registry_api.definition_count(active_registry), 1)
        with self.assertRaises(errors.InvalidInputError):
            registry_api.definition_count(registry)

    def test_failure_and_default_diagnostics_redact_secret_fixture_values(self) -> None:
        fixture = self.fixtures["synthetic-secret-bearing"]
        secret = fixture["secretSentinel"]
        registry = registry_api.create_client_extension_registry(
            list(self.definitions.values()), registry_api.default_extension_registry_limits()
        )
        try:
            recognition = registry_api.inspect_extension(
                registry,
                fixture["extension"]["vendorIdentification"],
                _structure_from_fixture(fixture["extension"]["payload"]["children"]),
                _codec_limits(),
            )
            try:
                self.assertNotIn(secret, repr(recognition))
                self.assertNotIn(secret, str(recognition))
            finally:
                recognition.close()

            invalid_payload = copy.deepcopy(fixture["extension"]["payload"]["children"])
            long_integer = next(item for item in invalid_payload if item["tag"] == "0x420004")
            long_integer["value"] = 100
            identity = registry_api.create_extension_identity(VENDOR_IDENTIFIER, "alpha", "1")
            with self.assertRaises(errors.InvalidExtensionSchemaError) as raised:
                registry_api.validate_extension_value(
                    registry, identity, _structure_from_fixture(invalid_payload), _codec_limits()
                )
            self.assertNotIn(secret, str(raised.exception))
            self.assertNotIn(secret, repr(raised.exception))
        finally:
            registry.close()


if __name__ == "__main__":
    unittest.main()
