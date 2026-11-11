from __future__ import annotations

import re
import uuid
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Any, Callable, Protocol
from .isolation import AttemptContext
from .contract import DEFAULT_CATALOG, load_catalog
from .models import TestCase


@dataclass(frozen=True)
class ExecutionOutcome:
    success: bool
    result_reason: str = ""


class KmipDriver(Protocol):
    def execute_case(self, test_case: TestCase, attempt: AttemptContext) -> ExecutionOutcome:
        ...


class LocalRecipeUnavailable(Exception):
    """A documented API-model gap for one local catalog recipe."""


_TTLV_TYPE_NAMES = (
    "Structure",
    "Integer",
    "Long Integer",
    "Big Integer",
    "Enumeration",
    "Boolean",
    "Text String",
    "Byte String",
    "Date Time",
    "Interval",
    "Date Time Extended",
)
_TTLV_TYPE_KINDS = {
    "Structure": "structure",
    "Integer": "integer",
    "Long Integer": "long_integer",
    "Big Integer": "big_integer",
    "Enumeration": "enumeration",
    "Boolean": "boolean",
    "Text String": "text",
    "Byte String": "bytes",
    "Date Time": "date_time",
    "Interval": "interval",
    "Date Time Extended": "date_time_extended",
}
_FIELD_TYPE_FALLBACKS = {
    "Any Object (Section 2)": {"Structure"},
    "All Objects": {"Boolean"},
    "Adjustment Value": {"Integer"},
    "Certificate Request Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Interop Identifier": {"Text String"},
    "Private Key Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Public Key Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Common Protection Storage Masks": {"Structure"},
    "Private Protection Storage Masks": {"Structure"},
    "Public Protection Storage Masks": {"Structure"},
    "Request Count": {"Integer"},
    "Replaced Unique Identifier": {"Text String", "Enumeration", "Integer"},
    "Time Stamp": {"Date Time"},
    "Ephemeral": {"Boolean"},
    "Request Payload": {"Structure"},
    "Response Payload": {"Structure"},
    "Server Hashed Password": {"Structure"},
    "Vendor-Specific": {"Structure"},
    "Credential Value": set(_TTLV_TYPE_NAMES),
    "Attribute Value": set(_TTLV_TYPE_NAMES),
    "Any attribute in §4 - Object Attributes": set(_TTLV_TYPE_NAMES),
}


@lru_cache(maxsize=1)
def _catalog_model_data() -> tuple[
    dict[str, dict[str, Any]], dict[str, set[str]], dict[str, dict[str, Any]]
]:
    """Load only checked-in normative test metadata, never KMIPKit source code."""
    import json

    root = Path(__file__).resolve().parents[4]
    shared = json.loads(
        (root / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text(encoding="utf-8")
    )
    source = json.loads((root / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
    target_by_id = {target["target_id"]: target for target in shared["normative_targets"]}
    element_types_by_name: dict[str, set[str]] = {}

    def types_in(text: str) -> set[str]:
        normalized = text.replace("-", " ")
        found: set[str] = set()
        for name in sorted(_TTLV_TYPE_NAMES, key=len, reverse=True):
            pattern = re.escape(name).replace(r"\ ", r"[\s-]+")
            if re.search(rf"(?<![A-Za-z]){pattern}(?![A-Za-z])", normalized, re.IGNORECASE):
                found.add(name)
        return found

    for element in source.get("elements", []):
        types: set[str] = set()
        for field in ("source_encoding", "value_type"):
            if isinstance(element.get(field), str):
                types.update(types_in(element[field]))
        for alternative in element.get("value_type_alternatives", []):
            if isinstance(alternative, str):
                types.update(types_in(alternative))
        kind = element.get("kind")
        if kind in {"attribute_structure", "object_structure", "operation_structure", "credential"}:
            types.add("Structure")
        if kind == "enumeration":
            types.add("Enumeration")
        elif kind == "bitmask":
            types.add("Integer")
        if types:
            element_types_by_name.setdefault(element.get("name", ""), set()).update(types)

    return target_by_id, element_types_by_name, {
        element["element_id"]: element for element in source.get("elements", [])
    }


class PythonKmipDriver:
    """Execute explicit typed recipes through the independently installed package."""

    _HIGH_LEVEL_OPERATIONS = {
        "Add Attribute", "Adjust Attribute", "Delete Attribute", "Get Attributes",
        "Get Attribute List", "Modify Attribute", "Set Attribute",
        "Query Asynchronous Requests",
    }
    _OPERATIONS = {
        "Discover Versions", "Ping", "Query", "Login", "Logout", "Delegated Login",
        "Log", "Interop", "Create", "Create Key Pair", "Create Split Key", "Register",
        "Import", "Certify", "Re-certify", "Re-Provision", "Validate", "PKCS#11",
        "Activate", "Archive", "Destroy", "Recover", "Revoke", "Get",
        "Export", "Get Attributes", "Get Attribute List", "Locate", "Check", "Obtain Lease",
        "Get Usage Allocation", "Get Constraints", "Set Constraints", "Set Defaults", "Re-key",
        "Re-key Key Pair", "Add Attribute", "Set Attribute", "Modify Attribute",
        "Delete Attribute", "Adjust Attribute", "Derive Key", "Join Split Key", "Hash", "RNG Retrieve", "RNG Seed", "Encrypt",
        "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify",
        "Poll", "Process", "Cancel", "Query Asynchronous Requests",
    }
    _NO_OBJECT = {
        "Discover Versions", "Ping", "Query", "Login", "Logout", "Delegated Login", "Log",
        "Interop", "Create", "Create Key Pair", "Create Split Key", "Register", "Import",
        "Certify", "Re-Provision", "Validate", "PKCS#11", "Get Constraints", "Set Constraints",
        "Set Defaults", "Hash", "RNG Retrieve", "RNG Seed",
    }
    _MASK_TAGS = {
        "Cryptographic Usage Mask": 0x0042_002C,
        "Storage Status Mask": 0x0042_008E,
        "Protection Storage Mask": 0x0042_015E,
    }
    _ENUM_TAGS = {
        "Opaque Data Type": 0x0042_0059,
        "PKCS#11 Function": 0x0042_015A,
        "PKCS#11 Return Code": 0x0042_015D,
        "Tag": 0x0042_0138,
    }

    def __init__(self, connection: dict[str, Any]) -> None:
        from kmipkit import client, extensions

        self._connection = connection
        self._api_level = connection["api_level"]
        self._client_module = client
        self._extensions = extensions
        limits = extensions.default_extension_registry_limits()
        registry = extensions.create_client_extension_registry([], limits)
        configuration = extensions.create_client_configuration(registry)
        version_parts = tuple(int(part) for part in connection["protocol_version"].split("."))
        configuration = configuration.with_protocol_version(client.ProtocolVersion(*version_parts))
        encodings = {
            "TTLV": extensions.WireEncoding.TTLV,
            "JSON": extensions.WireEncoding.JSON,
            "XML": extensions.WireEncoding.XML,
        }
        configuration = configuration.with_wire_encoding(encodings[connection["encoding"]])

        endpoint_values = connection["endpoints"][connection["transport"]]
        tls = connection["tls"]
        name = tls.get("server_name_override")
        endpoint = (
            client.Endpoint.raw_tls(endpoint_values["host"], endpoint_values["port"], tls_server_name=name)
            if connection["transport"] == "raw_tls"
            else client.Endpoint.https(endpoint_values["url"], tls_server_name=name)
        )
        pem = client.CredentialEncoding.PEM
        transport = client.TransportConfig(
            endpoint=endpoint,
            client_certificates=(client.TransportInput.file(tls["client_cert_path"], pem),),
            client_private_key=client.TransportInput.file(tls["client_key_path"], pem),
            trust=client.TrustSource.certificate_authorities(
                [client.TransportInput.file(tls["ca_cert_path"], pem)]
            ),
            connect_timeout=client.TimeoutLimit.bounded(tls.get("connect_timeout_seconds", 10)),
            read_timeout=client.TimeoutLimit.bounded(tls.get("read_timeout_seconds", 30)),
        )
        self._client = client.Client.create(configuration, transport)

    def _authentication(self) -> Any | None:
        from kmipkit import credentials
        import os

        configured = self._connection.get("authentication", {}).get("username_password")
        if configured is None:
            return None
        username = os.environ.get(configured["username_env"])
        password = os.environ.get(configured["password_env"])
        if not username or not password:
            raise ValueError("configured authentication credentials are unavailable")

        username_password = None
        credential_value = None
        credential = None
        try:
            username_password = credentials.UsernameAndPasswordCredential.from_ttlv(
                self._structure(
                    [
                        self._item(0x0042_0099, "text", username),
                        self._item(0x0042_00A1, "text", password),
                    ]
                )
            )
            credential_value = credentials.CredentialValue.from_ttlv(
                1,
                username_password.into_ttlv(),
            )
            credential_value_ttlv = credential_value.into_ttlv()
            try:
                envelope = self._structure(
                    [
                        self._item(0x0042_0024, "enumeration", 1),
                        self._item(0x0042_0025, "structure", credential_value_ttlv),
                    ]
                )
            finally:
                credential_value_ttlv.close()
            credential = credentials.Credential.from_ttlv(envelope)
            return credentials.Authentication.create([credential])
        finally:
            for handle in (credential, credential_value, username_password):
                if handle is not None:
                    handle.close()

    @classmethod
    def supports(cls, case: dict[str, Any], api_level: str) -> bool:
        if case["execution"]["mode"] != "server_exchange":
            return False
        workflow = case.get("requires", {}).get("workflow_type")
        if workflow in {"server_to_client", "multipart_streaming", "batch_id_placeholder"}:
            return False
        if api_level == "typed-protocol":
            return case["operation"] in cls._OPERATIONS
        return (
            api_level == "high-level"
            and workflow != "async_poll"
            and case["operation"] in cls._HIGH_LEVEL_OPERATIONS
        )

    @classmethod
    def route_limitation(cls, case: dict[str, Any], api_level: str) -> str:
        workflow = case.get("requires", {}).get("workflow_type")
        if api_level == "high-level":
            if case.get("operation") in cls._HIGH_LEVEL_OPERATIONS:
                return "the high-level request model is available only for the selected attribute or asynchronous operation"
            return "the pinned Python package exposes no high-level request model for this operation; select typed-protocol"
        if api_level == "generic-ttlv":
            return "the pinned Python client has no public outbound generic-TTLV request API"
        if workflow == "server_to_client":
            return "the pinned Python package provides a KMIP client and no server-initiated Request Message handler"
        if workflow == "multipart_streaming":
            return "the pinned Python package has no public multipart Init, Update and Final client workflow"
        if workflow == "batch_id_placeholder":
            return "the pinned Python package has no public UniqueIdentifier ID Placeholder request builder"
        if case.get("operation") not in cls._OPERATIONS:
            return f"the pinned Python package has no typed client constructor for {case.get('operation', 'this operation')}"
        return "no executable Python request recipe is registered for this catalog case"

    def _build_high_level_item(self, case: dict[str, Any], identifier: Any | None, *, name: str) -> Any:
        from kmipkit import attribute_operations, extensions, ttlv

        operation = case["operation"]
        if operation in {"Add Attribute", "Set Attribute"}:
            attribute = self._item(0x0042_00FD, "text", "KMIPKit interop test")
            request_type = attribute_operations.AddAttributeRequest if operation == "Add Attribute" else attribute_operations.SetAttributeRequest
            try:
                with request_type.create(identifier, attribute) as request:
                    return request.to_batch_item()
            finally:
                attribute.close()
        if operation == "Modify Attribute":
            tag = 0x0042_00FD
            kind = "text"
            current_value = (
                "old test value"
                if case["case_id"] == "test-neg-modify-attribute-current-mismatch"
                else "KMIPKit interop test"
            )
            current = self._legacy_attribute_item(current_value, tag=tag, kind=kind)
            updated = self._legacy_attribute_item("new test value", tag=tag, kind=kind)
            try:
                with attribute_operations.ModifyAttributeRequest.create(identifier, current, updated) as request:
                    return request.to_batch_item()
            finally:
                current.close()
                updated.close()
        if operation == "Delete Attribute":
            current = self._legacy_attribute_item("KMIPKit interop test")
            try:
                with attribute_operations.DeleteAttributeRequest.create(
                    identifier,
                    current,
                    extensions.AttributeReference.tag(0x0042_00FD),
                ) as request:
                    return request.to_batch_item()
            finally:
                current.close()
        if operation == "Adjust Attribute":
            value = ttlv.ttlv_value_integer(1)
            try:
                with attribute_operations.AdjustAttributeRequest.create(
                    identifier,
                    extensions.AttributeReference.tag(0x0042_0001),
                    1,
                    value,
                ) as request:
                    return request.to_batch_item()
            finally:
                value.close()
        if operation == "Get Attributes":
            attribute_name = case.get("requires", {}).get("attribute")
            if not attribute_name:
                references = [extensions.AttributeReference.tag(0x0042_0053)]
            elif attribute_name == "Certificate Attributes":
                references = [
                    extensions.AttributeReference.tag(0x0042_001D),
                    extensions.AttributeReference.tag(0x0042_00AD),
                ]
            elif attribute_name == "Vendor Attribute":
                references = [extensions.AttributeReference.name("KMIPKit.Interop", "probe")]
            else:
                raw_tag = self._attribute_tag(attribute_name)
                if raw_tag is None:
                    raise LocalRecipeUnavailable("standard Attribute Reference tag is unavailable")
                references = [extensions.AttributeReference.tag(raw_tag)]
            with attribute_operations.GetAttributesRequest.create(identifier, references) as request:
                return request.to_batch_item()
        if operation == "Get Attribute List":
            with attribute_operations.GetAttributeListRequest.create(identifier) as request:
                return request.to_batch_item()
        del name
        raise LocalRecipeUnavailable("the Python high-level request model does not cover this operation")

    @classmethod
    def requires_mutation(cls, case: dict[str, Any]) -> bool:
        """Include recipe setup and cleanup effects in the connection safety gate."""
        if case["execution"]["mutation"] == "mutating":
            return True
        recipe = case.get("isolation_recipe", {})
        return any(
            "create" in step.lower() or "destroy" in step.lower() or "register" in step.lower()
            for step in (*recipe.get("setup_steps", []), *recipe.get("teardown_steps", []))
        )

    @staticmethod
    def local_recipe_limitation(case: dict[str, Any]) -> str | None:
        """Return the concrete installed-API gap for offline local recipes."""
        if case["execution"]["mode"] != "local_model":
            return None
        if case["category"] == "version_boundaries":
            return (
                "API gap: the public client has no offline request validator or serialized-request accessor "
                "to prove a KMIP 2.1 operation is rejected before transport under KMIP 2.0"
            )
        scenario = case.get("requires", {}).get("protocol_scenario", {})
        scenario_id = scenario.get("scenario_id")
        limitations = {
            "test-batch-count-zero-rejected": (
                "API gap: Batch exposes no offline request validation or serialization result for an empty batch"
            ),
            "test-batch-single-item-options-omitted": (
                "API gap: Batch exposes no encoded Request Header view to assert omission of default options"
            ),
            "test-batch-placeholder-forward-reference-rejected": (
                "API gap: ClientBatchItem has no public ID Placeholder value or batch-reference builder"
            ),
            "test-batch-placeholder-without-producer-rejected": (
                "API gap: ClientBatchItem has no public ID Placeholder value or batch-reference builder"
            ),
        }
        return limitations.get(scenario_id)

    @staticmethod
    def execute_local_case(test_case: dict[str, Any]) -> None:
        """Execute catalog-driven checks using only the installed package API."""
        from kmipkit import protocol_values, ttlv

        limitation = PythonKmipDriver.local_recipe_limitation(test_case)
        if limitation is not None:
            raise LocalRecipeUnavailable(limitation)
        requirements = test_case["requires"]
        scenario = requirements.get("protocol_scenario")
        if scenario is not None:
            PythonKmipDriver._execute_local_batch_case(test_case, scenario)
            return

        enum_type_name = requirements.get("enum_type")
        mask_type_name = requirements.get("bitmask_type")
        if enum_type_name:
            class_name = "".join(
                part[0].upper() + part[1:].lower()
                for part in re.findall(r"[A-Za-z0-9]+", enum_type_name)
            )
            value_type = getattr(protocol_values, class_name, None)
            if value_type is None or not callable(getattr(value_type, "name", None)):
                PythonKmipDriver._round_trip_catalog_enum(enum_type_name)
                return
            known = next(
                (
                    value
                    for name, value in vars(value_type).items()
                    if not name.startswith("_")
                    and type(value) is int
                    and value_type.name(value) is not None
                ),
                None,
            )
            if known is None:
                PythonKmipDriver._round_trip_catalog_enum(enum_type_name)
                return
            if value_type.name(0x7FFF_FFFE) is not None:
                raise ValueError("unknown enum value was treated as assigned")
            PythonKmipDriver._round_trip_ttlv(0x0042_005C, known, enumeration=True)
            PythonKmipDriver._round_trip_ttlv(0x0042_005C, 0x7FFF_FFFE, enumeration=True)
            return

        if mask_type_name:
            class_name = "".join(
                part[0].upper() + part[1:].lower()
                for part in re.findall(r"[A-Za-z0-9]+", mask_type_name)
            )
            value_type = getattr(protocol_values, class_name, None)
            if value_type is None:
                raise ValueError("catalog bitmask has no public Python value mapping")
            assigned = [
                value
                for name, value in vars(value_type).items()
                if not name.startswith("_") and name != "KNOWN_MASK" and type(value) is int
            ]
            if not assigned:
                raise ValueError("catalog bitmask has no assigned bits")
            known = assigned[0] | (assigned[1] if len(assigned) > 1 else 0)
            unknown = known | 0x4000_0000
            tag = PythonKmipDriver._MASK_TAGS.get(mask_type_name)
            if tag is None:
                raise LocalRecipeUnavailable("Python bitmask tag mapping is unavailable")
            PythonKmipDriver._round_trip_ttlv(tag, known, enumeration=False)
            PythonKmipDriver._round_trip_ttlv(tag, unknown, enumeration=False)
            return

        target_kind = requirements.get("target_kind")
        target_id = requirements.get("normative_target_id")
        target_by_id, element_types_by_name, _ = _catalog_model_data()
        target = target_by_id.get(target_id, {})

        if target_kind in {"enumeration_value", "referenced_enum_value"}:
            PythonKmipDriver._execute_catalog_enumeration(test_case, target)
            return
        if target_kind == "bitmask_value":
            PythonKmipDriver._execute_catalog_bitmask(test_case, target, target_by_id)
            return
        if target_kind in {"tag", "tag_range"}:
            PythonKmipDriver._execute_catalog_tag(test_case)
            return
        if target_kind == "data_type":
            PythonKmipDriver._execute_data_type_case(target["name"])
            PythonKmipDriver._exercise_generic_type(target["name"])
            return
        if target_kind is None:
            raise ValueError("local catalog case has no target kind or supported legacy recipe")

        type_names = PythonKmipDriver._catalog_value_types(
            test_case, target, element_types_by_name
        )
        if not type_names:
            raise ValueError(f"catalog target {target_id} has no TTLV construction type")
        for type_name in sorted(type_names):
            PythonKmipDriver._exercise_generic_type(type_name)

    @staticmethod
    def _execute_local_batch_case(test_case: dict[str, Any], scenario: dict[str, Any]) -> None:
        from kmipkit import client, extensions

        scenario_id = scenario["scenario_id"]
        if scenario_id != "test-batch-count-mismatch-rejected":
            raise ValueError(f"unsupported local batch scenario {scenario_id}")

        identifier = extensions.UniqueIdentifier.text_string("local-batch-count-fixture")
        driver = object.__new__(PythonKmipDriver)
        driver._extensions = extensions
        batch = client.Batch.create()
        items = []
        try:
            for operation in scenario["operations"]:
                case = {"operation": operation, "requires": {}}
                item = driver._build_item(case, identifier, name="local-batch-count-fixture")
                items.append(item)
                batch.add_item(item)
            if batch.item_count != len(scenario["operations"]):
                raise ValueError("batch item count does not match the constructed items")
            if hasattr(batch, "set_batch_count"):
                raise ValueError("public Batch unexpectedly exposes an independently mutable count")
        finally:
            batch.close()
            for item in items:
                item.close()

    @staticmethod
    def _wire_integer(value: str) -> int:
        return int(value.removeprefix("0x").removeprefix("0X"), 16)

    @staticmethod
    def _wire_representatives(value: str) -> list[int]:
        range_match = re.fullmatch(r"\s*((?:0x)?[0-9a-fA-F]+)\s*[-–—]\s*((?:0x)?[0-9a-fA-F]+)\s*", value)
        if range_match:
            return [PythonKmipDriver._wire_integer(part) for part in range_match.groups()]
        if "X" in value.upper():
            return [int(value.upper().replace("X", "0"), 16)]
        return [PythonKmipDriver._wire_integer(value)]

    @staticmethod
    def _execute_catalog_enumeration(test_case: dict[str, Any], target: dict[str, Any]) -> None:
        from kmipkit import protocol_values

        requires = test_case["requires"]
        values = PythonKmipDriver._wire_representatives(requires["wire_value"])
        if target.get("kind") == "referenced_enum_value":
            for raw in values:
                PythonKmipDriver._round_trip_ttlv(0x0042_005C, raw, enumeration=True)
            return
        enum_name = target.get("parent_name") or requires.get("parent_name")
        if enum_name is None and target.get("kind") == "enumeration_value":
            _, _, element_by_id = _catalog_model_data()
            element = element_by_id.get(target["target_id"])
            if element is not None:
                parent_ids = element.get("parent_element_ids", [])
                parent = next(
                    (
                        element_by_id[parent_id]
                        for parent_id in parent_ids
                        if parent_id in element_by_id
                        and element_by_id[parent_id].get("kind") == "enumeration"
                    ),
                    None,
                )
                enum_name = None if parent is None else parent.get("name")
        value_type = getattr(protocol_values, PythonKmipDriver._class_name(enum_name or ""), None)
        name_function = getattr(value_type, "name", None)
        for raw in values:
            if callable(name_function):
                mapped = name_function(raw)
                if requires.get("allocation") == "assigned" and mapped is None:
                    raise ValueError("an assigned catalog enumeration value is absent from the public mapping")
                if requires.get("allocation") in {"reserved", "unused"} and mapped is not None:
                    raise ValueError("a reserved catalog enumeration value is exposed as assigned")
            PythonKmipDriver._round_trip_ttlv(0x0042_005C, raw, enumeration=True)

    @staticmethod
    def _class_name(name: str) -> str:
        return "".join(
            part[0].upper() + part[1:].lower()
            for part in re.findall(r"[A-Za-z0-9]+", name)
        )

    @staticmethod
    def _execute_catalog_bitmask(
        test_case: dict[str, Any], target: dict[str, Any], target_by_id: dict[str, dict[str, Any]]
    ) -> None:
        requires = test_case["requires"]
        wire_value = requires["wire_value"]
        if "X" in wire_value.upper():
            parent_ids = set(target.get("parent_element_ids", []))
            assigned = 0
            for candidate in target_by_id.values():
                if (
                    candidate.get("kind") == "bitmask_value"
                    and parent_ids.intersection(candidate.get("parent_element_ids", []))
                    and candidate.get("allocation") == "assigned"
                    and "X" not in candidate.get("wire_value", "").upper()
                ):
                    assigned |= PythonKmipDriver._wire_integer(candidate["wire_value"])
            wildcard_mask = int(wire_value.upper().replace("X", "F"), 16)
            available = wildcard_mask & ~assigned & 0xFFFF_FFFF
            if not available:
                raise ValueError("catalog bitmask extension range has no unassigned representative")
            raw = available & -available
        else:
            raw = PythonKmipDriver._wire_integer(wire_value)
        parent_ids = target.get("parent_element_ids", [])
        parent_id = parent_ids[0] if parent_ids else ""
        mask_name = parent_id.removeprefix("KMIPKIT-ELEM-BITMASK-")
        tag = {
            "CRYPTOGRAPHIC-USAGE-MASK": 0x0042_002C,
            "STORAGE-STATUS-MASK": 0x0042_008E,
            "PROTECTION-STORAGE-MASK": 0x0042_015E,
        }.get(mask_name)
        if tag is None:
            raise ValueError("catalog bitmask target has no public TTLV tag mapping")
        PythonKmipDriver._round_trip_ttlv(tag, raw, enumeration=False)

    @staticmethod
    def _execute_catalog_tag(test_case: dict[str, Any]) -> None:
        from kmipkit import ttlv

        wire_value = test_case["requires"]["wire_value"]
        if test_case["requires"]["target_kind"] == "tag_range":
            match = re.fullmatch(r"\s*([0-9a-fA-FxX]+)\s*[-–—]\s*([0-9a-fA-FxX]+)\s*", wire_value)
            if match is None:
                raise ValueError("catalog tag range cannot be parsed")
            raw_values = [
                int(match.group(1).upper().replace("X", "0"), 16),
                int(match.group(2).upper().replace("X", "F"), 16),
            ]
        else:
            raw_values = PythonKmipDriver._wire_representatives(wire_value)
        for raw in raw_values:
            with ttlv.raw_tag_from_raw(raw) as raw_tag:
                if ttlv.raw_tag_value(raw_tag) != raw:
                    raise ValueError("raw TTLV tag did not preserve its value")
                if test_case["requires"].get("allocation") in {"assigned", "extension"}:
                    with ttlv.raw_tag_try_checked(raw_tag) as tag:
                        if ttlv.tag_value(tag) != raw:
                            raise ValueError("checked TTLV tag did not preserve its allocated value")
                else:
                    try:
                        invalid_tag = ttlv.raw_tag_try_checked(raw_tag)
                    except ttlv.errors.InvalidExtensionSchemaError:
                        continue
                    invalid_tag.close()
                    raise ValueError("reserved or unused TTLV tag was accepted as checked")

    @staticmethod
    def _catalog_value_types(
        test_case: dict[str, Any],
        target: dict[str, Any],
        element_types_by_name: dict[str, set[str]],
    ) -> set[str]:
        requires = test_case["requires"]
        values: set[str] = set()
        fields = ("source_encoding", "value_type")
        for source in (requires, target):
            for field in fields:
                if isinstance(source.get(field), str):
                    values.update(PythonKmipDriver._types_in(source[field]))
            for alternative in source.get("value_type_alternatives", []):
                if isinstance(alternative, str):
                    values.update(PythonKmipDriver._types_in(alternative))
        name = target.get("name") or requires.get("field_name") or ""
        values.update(element_types_by_name.get(name, set()))
        values.update(_FIELD_TYPE_FALLBACKS.get(name, set()))
        if target.get("kind") in {
            "attribute_structure", "object_structure", "operation_structure", "credential"
        }:
            values.add("Structure")
        if target.get("kind") == "operation_payload_value_variant" and isinstance(
            target.get("value_type"), str
        ):
            values.update(PythonKmipDriver._types_in(target["value_type"]))
        return values

    @staticmethod
    def _types_in(text: str) -> set[str]:
        normalized = text.replace("-", " ")
        found: set[str] = set()
        for name in sorted(_TTLV_TYPE_NAMES, key=len, reverse=True):
            pattern = re.escape(name).replace(r"\ ", r"[\s-]+")
            if re.search(rf"(?<![A-Za-z]){pattern}(?![A-Za-z])", normalized, re.IGNORECASE):
                found.add(name)
        return found

    @staticmethod
    def _execute_data_type_case(type_name: str) -> None:
        import json

        from kmipkit import ttlv

        root = Path(__file__).resolve().parents[4]
        vectors = json.loads(
            (root / "interop/shared/vectors/ttlv-boundaries.json").read_text(encoding="utf-8")
        )
        vector = next((item for item in vectors["vectors"] if item["data_type"] == type_name), None)
        if vector is None:
            raise ValueError("catalog data type has no checked-in TTLV boundary vectors")
        expected_type = type_name.replace(" ", "")
        for sample in vector["samples"]:
            wire = bytes.fromhex(sample["hex"])
            with ttlv.codec_limits_defaults() as limits:
                if not sample["valid"]:
                    try:
                        invalid_item = ttlv.ttlv_decode_with_limits(wire, limits)
                    except ttlv.errors.TtlvDecodeError:
                        continue
                    invalid_item.close()
                    raise ValueError("malformed TTLV boundary vector was accepted")
                with ttlv.ttlv_decode_with_limits(wire, limits) as item:
                    with ttlv.ttlv_item_view_from_item(item) as view:
                        actual_type = ttlv.ttlv_item_view_type(view).name
                        if actual_type != expected_type:
                            raise ValueError("TTLV boundary vector decoded to a different data type")
                        if type_name == "Structure":
                            with ttlv.ttlv_item_view_value(view) as value:
                                with ttlv.ttlv_value_view_structure(value) as structure:
                                    child_count = ttlv.ttlv_structure_view_item_count(structure)
                                    if sample["branch"] in {"ordered_children", "reversed_children"}:
                                        if child_count != 2:
                                            raise ValueError("structure boundary vector lost a child")
                                        observed_tags = []
                                        for index in range(child_count):
                                            with ttlv.ttlv_structure_view_item_at(structure, index) as child:
                                                with ttlv.ttlv_item_view_tag(child) as tag:
                                                    observed_tags.append(ttlv.tag_value(tag))
                                        expected_tags = (
                                            [0x00420001, 0x00420002]
                                            if sample["branch"] == "ordered_children"
                                            else [0x00420002, 0x00420001]
                                        )
                                        if observed_tags != expected_tags:
                                            raise ValueError("structure child order changed during decoding")

    @staticmethod
    @lru_cache(maxsize=None)
    def _exercise_generic_type(type_name: str) -> None:
        values = {
            "Structure": [None],
            "Integer": [-0x8000_0000, -1, 0, 0x7FFF_FFFF],
            "Long Integer": [-(1 << 63), -1, 0, (1 << 63) - 1],
            "Big Integer": [bytes.fromhex("0000000000000001"), bytes.fromhex("ffffffffffffffff")],
            "Enumeration": [0, 1, 0x7FFF_FFFF, 0x8000_0000],
            "Boolean": [False, True],
            "Text String": ["", "KMIP-λ"],
            "Byte String": [b"", b"\x00\x01\x02\x03\x04\x05\x06\x07\x08"],
            "Date Time": [-1, 0, 1],
            "Interval": [0, 1, 0x7FFF_FFFF],
            "Date Time Extended": [-1, 0, 1],
        }
        if type_name not in values:
            raise ValueError(f"unsupported TTLV data type {type_name}")
        for value in values[type_name]:
            PythonKmipDriver._round_trip_generic_value(type_name, value)

    @staticmethod
    def _round_trip_generic_value(type_name: str, value: Any) -> None:
        from kmipkit import ttlv

        kind = _TTLV_TYPE_KINDS[type_name]
        if type_name == "Structure":
            children = [
                PythonKmipDriver._item(0x0042_0001, "integer", 7),
                PythonKmipDriver._item(0x0042_0002, "enumeration", 3),
            ]
            structure = PythonKmipDriver._structure(children)
            try:
                item = PythonKmipDriver._item(0x0042_00FF, kind, structure)
            finally:
                structure.close()
                for child in children:
                    child.close()
        else:
            item = PythonKmipDriver._item(0x0042_00FF, kind, value)
        try:
            with ttlv.ttlv_item_view_from_item(item) as item_view:
                if ttlv.ttlv_item_view_type(item_view).name != type_name.replace(" ", ""):
                    raise ValueError("generic TTLV constructor produced the wrong item type")
                with ttlv.ttlv_item_view_value(item_view) as value_view:
                    if type_name == "Structure":
                        with ttlv.ttlv_value_view_structure(value_view) as structure_view:
                            if ttlv.ttlv_structure_view_item_count(structure_view) != 2:
                                raise ValueError("generic TTLV structure lost a child")
                            observed = []
                            for index in range(2):
                                with ttlv.ttlv_structure_view_item_at(structure_view, index) as child:
                                    observed.append(ttlv.ttlv_item_view_type(child).name)
                            if observed != ["Integer", "Enumeration"]:
                                raise ValueError("generic TTLV structure changed child order or type")
                    elif type_name == "Big Integer":
                        if ttlv.ttlv_value_view_type(value_view).name != "BigInteger":
                            raise ValueError("generic Big Integer constructor changed type")
                    else:
                        getter = {
                            "Integer": ttlv.ttlv_value_view_integer,
                            "Long Integer": ttlv.ttlv_value_view_long_integer,
                            "Enumeration": ttlv.ttlv_value_view_enumeration,
                            "Boolean": ttlv.ttlv_value_view_boolean,
                            "Text String": ttlv.ttlv_value_view_bytes,
                            "Byte String": ttlv.ttlv_value_view_bytes,
                            "Date Time": ttlv.ttlv_value_view_date_time,
                            "Interval": ttlv.ttlv_value_view_interval,
                            "Date Time Extended": ttlv.ttlv_value_view_date_time_extended,
                        }[type_name]
                        expected = value.encode("utf-8") if isinstance(value, str) else value
                        if getter(value_view) != expected:
                            raise ValueError("generic TTLV construction did not preserve its value")
        finally:
            item.close()

    @staticmethod
    def _round_trip_catalog_enum(name: str) -> None:
        tag = PythonKmipDriver._ENUM_TAGS.get(name)
        assigned = PythonKmipDriver._catalog_enum_values(name)
        if tag is None:
            raise LocalRecipeUnavailable("enum TTLV tag is absent from the source catalog")
        known = assigned[0] if assigned else 0
        unknown = 0x7FFF_FFFF
        while unknown in assigned:
            unknown -= 1
        PythonKmipDriver._round_trip_ttlv(tag, known, enumeration=True)
        PythonKmipDriver._round_trip_ttlv(tag, unknown, enumeration=True)

    @staticmethod
    @lru_cache(maxsize=None)
    def _catalog_enum_values(name: str) -> list[int]:
        import json

        root = Path(__file__).resolve().parents[4]
        catalog = json.loads((root / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        enumeration = next(
            (
                element
                for element in catalog["elements"]
                if element.get("kind") == "enumeration" and element.get("name") == name
            ),
            None,
        )
        if enumeration is None:
            return []
        values = []
        for element in catalog["elements"]:
            if (
                element.get("kind") != "enumeration_value"
                or enumeration["element_id"] not in element.get("parent_element_ids", [])
                or element.get("allocation") != "assigned"
            ):
                continue
            wire_value = element.get("wire_value")
            if isinstance(wire_value, int):
                values.append(wire_value)
            elif isinstance(wire_value, str) and re.fullmatch(r"(?:0x)?[0-9A-Fa-f]{1,8}", wire_value):
                values.append(int(wire_value.removeprefix("0x"), 16))
        return values

    @staticmethod
    def _round_trip_ttlv(tag_value: int, raw_value: int, *, enumeration: bool) -> None:
        from kmipkit import ttlv

        with ttlv.codec_limits_defaults() as limits:
            with ttlv.raw_tag_from_raw(tag_value) as raw_tag:
                with ttlv.raw_tag_try_checked(raw_tag) as tag:
                    signed_value = (
                        raw_value if raw_value < 0x8000_0000 else raw_value - 0x1_0000_0000
                    )
                    value = (
                        ttlv.ttlv_value_enumeration(raw_value)
                        if enumeration
                        else ttlv.ttlv_value_integer(signed_value)
                    )
                    with value:
                        with ttlv.ttlv_item_create(tag, value, limits) as item:
                            with ttlv.ttlv_item_view_from_item(item) as item_view:
                                with ttlv.ttlv_item_view_value(item_view) as value_view:
                                    observed = (
                                        ttlv.ttlv_value_view_enumeration(value_view)
                                        if enumeration
                                        else ttlv.ttlv_value_view_integer(value_view) & 0xFFFF_FFFF
                                    )
                                    if observed != raw_value:
                                        raise ValueError("TTLV model did not preserve the raw enum or mask value")

    @staticmethod
    def _item(tag_value: int, kind: str, value: Any) -> Any:
        from kmipkit import ttlv

        with ttlv.codec_limits_defaults() as limits:
            with ttlv.raw_tag_from_raw(tag_value) as raw_tag:
                with ttlv.raw_tag_try_checked(raw_tag) as tag:
                    if kind == "integer":
                        encoded = ttlv.ttlv_value_integer(value)
                    elif kind == "enumeration":
                        encoded = ttlv.ttlv_value_enumeration(value)
                    elif kind == "text":
                        encoded = ttlv.ttlv_value_text_string(value.encode("utf-8"), limits)
                    elif kind == "bytes":
                        encoded = ttlv.ttlv_value_byte_string(value, limits)
                    elif kind == "structure":
                        encoded = ttlv.ttlv_value_structure(value, limits)
                    elif kind == "long_integer":
                        encoded = ttlv.ttlv_value_long_integer(value)
                    elif kind == "big_integer":
                        encoded = ttlv.ttlv_value_big_integer(value, limits)
                    elif kind == "boolean":
                        encoded = ttlv.ttlv_value_boolean(value)
                    elif kind == "date_time":
                        encoded = ttlv.ttlv_value_date_time(value)
                    elif kind == "interval":
                        encoded = ttlv.ttlv_value_interval(value)
                    elif kind == "date_time_extended":
                        encoded = ttlv.ttlv_value_date_time_extended(value)
                    else:
                        raise ValueError("unsupported TTLV value type")
                    with encoded:
                        return ttlv.ttlv_item_create(tag, encoded, limits)

    @classmethod
    def _structure(cls, children: list[Any]) -> Any:
        from kmipkit import ttlv

        with ttlv.codec_limits_defaults() as limits:
            structure = ttlv.ttlv_structure_create()
            for child in children:
                updated = ttlv.ttlv_structure_with_item(structure, child, limits)
                structure.close()
                structure = updated
            return structure

    @classmethod
    def _structure_item(cls, tag_value: int, children: list[Any]) -> Any:
        return cls._item(tag_value, "structure", cls._structure(children))

    @classmethod
    def _legacy_attribute_item(
        cls, value: Any, *, tag: int = 0x0042_00FD, kind: str = "text"
    ) -> Any:
        """Build a direct standard Attribute Item for Current/New Attribute."""
        return cls._item(tag, kind, value)

    @classmethod
    def _attribute(cls, name: str, tag_value: int, kind: str, value: Any) -> Any:
        name_item = cls._item(0x0042_000A, "text", name)
        del tag_value
        value_item = cls._item(0x0042_000B, kind, value)
        return cls._structure_item(0x0042_0008, [name_item, value_item])

    @classmethod
    def _attributes(cls, name: str, algorithm: int = 3, bits: int = 256) -> Any:
        name_value = cls._structure_item(
            0x0042_0053,
            [cls._item(0x0042_0055, "text", name), cls._item(0x0042_0054, "enumeration", 1)],
        )
        attributes = [
            cls._item(0x0042_0028, "enumeration", algorithm),
            cls._item(0x0042_002A, "integer", bits),
            cls._item(
                0x0042_002C,
                "integer",
                0x20 if algorithm in (4, 6) else 0x180 if algorithm == 9 else 0x0C,
            ),
            name_value,
            cls._item(0x0042_00FD, "text", "KMIPKit interop test"),
        ]
        return cls._structure(attributes)

    @staticmethod
    def _cryptographic_parameters(case: dict[str, Any], operation: str) -> Any:
        """Build Table 59 parameters through the installed typed public API."""
        from kmipkit.cryptographic_parameters import CryptographicParameters

        algorithm = case.get("requires", {}).get("algorithm", "AES")
        builder = CryptographicParameters.builder()
        if operation in {"Encrypt", "Decrypt"}:
            # The catalog's AES-CBC recipes use CBC with PKCS5 padding.
            builder.cryptographic_algorithm(3).block_cipher_mode(1).padding_method(3)
        elif operation in {"MAC", "MAC Verify"}:
            builder.cryptographic_algorithm(9)
        elif operation in {"Sign", "Signature Verify"}:
            builder.cryptographic_algorithm({"RSA": 4, "ECDSA": 6}.get(algorithm, 4))
        elif operation == "Hash":
            builder.hashing_algorithm(6)
        return builder.build().into_ttlv()

    @staticmethod
    def _attribute_tag(name: str) -> int | None:
        import json
        from pathlib import Path

        root = Path(__file__).resolve().parents[4]
        catalog = json.loads((root / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8"))
        for element in catalog["elements"]:
            if element.get("kind") != "attribute" or element.get("name") != name:
                continue
            for parent in element.get("parent_element_ids", []):
                match = re.fullmatch(r"KMIPKIT-ELEM-TAG-([0-9A-Fa-f]{6})", parent)
                if match:
                    return int(match.group(1), 16)
        return None

    def _build_item(
        self,
        case: dict[str, Any],
        identifier: Any | None,
        *,
        name: str,
        data: bytes = b"A" * 16,
        auxiliary: bytes | None = None,
        related_identifiers: list[Any] | None = None,
    ) -> Any:
        from kmipkit import extensions, protocol_values, ttlv

        operation = case["operation"]
        extension = self._extensions
        params = (
            self._cryptographic_parameters(case, operation)
            if operation in {"Encrypt", "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify", "Hash"}
            else self._structure([])
        )
        attrs = self._attributes(name)
        try:
            if operation == "Discover Versions":
                return extension.client_batch_item_discover_versions()
            if operation == "Ping":
                return extension.client_batch_item_ping()
            if operation == "Query":
                return extension.client_batch_item_query(
                    [protocol_values.QueryFunction.QUERY_OPERATIONS, protocol_values.QueryFunction.QUERY_OBJECTS]
                )
            if operation == "Get Constraints":
                return extension.client_batch_item_get_constraints()
            if operation == "Login":
                return extension.client_batch_item_login(request_count=1)
            if operation == "Delegated Login":
                rights = ttlv.ttlv_structure_create()
                try:
                    return extension.client_batch_item_delegated_login(rights, request_count=1)
                finally:
                    rights.close()
            if operation == "Log":
                return extension.client_batch_item_log("KMIPKit dedicated interoperability probe")
            if operation == "Interop":
                return extension.client_batch_item_interop(1, name)
            object_type = case.get("requires", {}).get("object_type")
            if operation == "Create" and object_type in {"Private Key", "Public Key"}:
                return extension.client_batch_item_create_key_pair(common_attributes=attrs)
            if operation == "Create" and object_type == "Split Key":
                return extension.client_batch_item_create_split_key(2, 2, 2, 1, attrs)
            if operation == "Create":
                return extension.client_batch_item_create(2, attrs)
            if operation == "Create Key Pair":
                return extension.client_batch_item_create_key_pair(common_attributes=attrs)
            if operation == "Create Split Key":
                return extension.client_batch_item_create_split_key(2, 2, 2, 1, attrs)
            if operation == "Register":
                object_type = case.get("requires", {}).get("object_type", "Opaque Object")
                fixture_root = Path(__file__).resolve().parents[4]
                if object_type == "Certificate":
                    cert = (fixture_root / "interop/rust-kmip-validator/tests/fixtures/certificate.der").read_bytes()
                    attrs.close()
                    attrs = self._structure(
                        [
                            self._item(0x0042_002C, "integer", 0x20),
                            self._item(0x0042_0028, "enumeration", 6),
                            self._item(0x0042_002A, "integer", 256),
                            self._item(0x0042_00AD, "integer", len(cert) * 8),
                        ]
                    )
                    object_item = self._structure_item(
                        0x0042_0013,
                        [self._item(0x0042_001D, "enumeration", 1), self._item(0x0042_001E, "bytes", cert)],
                    )
                    raw_object_type = 1
                elif object_type == "Certificate Request":
                    cert = (fixture_root / "interop/rust-kmip-validator/tests/fixtures/certificate-request.der").read_bytes()
                    attrs.close()
                    attrs = self._attributes(name, 6, 256)
                    object_item = self._structure_item(
                        0x0042_0018,
                        [self._item(0x0042_0019, "enumeration", 1), self._item(0x0042_0140, "bytes", cert)],
                    )
                    raw_object_type = 10
                elif object_type == "Opaque Object":
                    attrs.close()
                    attrs = ttlv.ttlv_structure_create()
                    object_item = self._structure_item(
                        0x0042_005B,
                        [self._item(0x0042_0059, "enumeration", 1), self._item(0x0042_005A, "bytes", b"KMIPKit dedicated opaque fixture")],
                    )
                    raw_object_type = 8
                elif object_type in {"PGP Key", "Secret Data"}:
                    key_material = self._item(0x0042_0043, "bytes", bytes(range(32)))
                    key_value = self._structure_item(0x0042_0045, [key_material])
                    key_block_fields = [
                        self._item(0x0042_0042, "enumeration", 1),
                        key_value,
                    ]
                    if object_type == "PGP Key":
                        key_block_fields.extend(
                            [
                                self._item(0x0042_0028, "enumeration", 3),
                                self._item(0x0042_002A, "integer", 256),
                            ]
                        )
                    key_block = self._structure_item(
                        0x0042_0040,
                        key_block_fields,
                    )
                    if object_type == "PGP Key":
                        object_item = self._structure_item(
                            0x0042_00BD,
                            [self._item(0x0042_00BE, "integer", 4), key_block],
                        )
                        raw_object_type = 9
                    else:
                        attrs.close()
                        attrs = self._structure([self._item(0x0042_002C, "integer", 0x0C)])
                        object_item = self._structure_item(
                            0x0042_0085,
                            [self._item(0x0042_0086, "enumeration", 1), key_block],
                        )
                        raw_object_type = 7
                else:
                    raise LocalRecipeUnavailable("Python Register fixture is unavailable for this object type")
                request = extension.RegisterRequest.create(raw_object_type, attrs, object_item)
                return extension.client_batch_item_register(request)
            if operation == "Import":
                attrs.close()
                attrs = ttlv.ttlv_structure_create()
                opaque = self._structure_item(
                    0x0042_005B,
                    [
                        self._item(0x0042_0059, "enumeration", 1),
                        self._item(0x0042_005A, "bytes", b"KMIPKit dedicated import fixture"),
                    ],
                )
                any_object = self._structure([opaque])
                try:
                    return extension.client_batch_item_import(
                        extensions.UniqueIdentifier.text_string(name),
                        8,
                        attrs,
                        any_object,
                        replace_existing=False,
                    )
                finally:
                    any_object.close()
            if operation == "Get Attributes":
                attribute_name = case.get("requires", {}).get("attribute")
                if not attribute_name:
                    references = [extensions.AttributeReference.tag(0x0042_0053)]
                elif attribute_name == "Certificate Attributes":
                    references = [
                        extensions.AttributeReference.tag(0x0042_001D),
                        extensions.AttributeReference.tag(0x0042_00AD),
                    ]
                elif attribute_name == "Vendor Attribute":
                    references = [extensions.AttributeReference.name("KMIPKit.Interop", "probe")]
                else:
                    raw_tag = self._attribute_tag(attribute_name)
                    if raw_tag is None:
                        raise LocalRecipeUnavailable("standard Attribute Reference tag is unavailable")
                    references = [extensions.AttributeReference.tag(raw_tag)]
                return extension.client_batch_item_get_attributes(
                    identifier, references
                )
            if operation == "Get Attribute List":
                return extension.client_batch_item_get_attribute_list(identifier)
            if operation == "Locate":
                return extension.client_batch_item_locate(attrs, maximum_items=1)
            if operation == "Get":
                return extension.client_batch_item_get(identifier)
            if operation == "Activate":
                return extension.client_batch_item_activate(identifier)
            if operation == "Archive":
                return extension.client_batch_item_archive(identifier)
            if operation == "Destroy":
                return extension.client_batch_item_destroy(identifier)
            if operation == "Recover":
                return extension.client_batch_item_recover(identifier)
            if operation == "Revoke":
                reason = extension.RevocationReason.create(1, None)
                try:
                    request = extension.RevokeRequest.create(identifier, reason, None)
                    return extension.client_batch_item_revoke(request)
                finally:
                    reason.close()
            if operation == "Export":
                return extension.client_batch_item_export(identifier)
            if operation == "Check":
                return extension.client_batch_item_check(identifier, cryptographic_usage_mask=4)
            if operation == "Obtain Lease":
                return extension.client_batch_item_obtain_lease(identifier)
            if operation == "Get Usage Allocation":
                return extension.client_batch_item_get_usage_allocation(identifier, 1)
            if operation == "Set Constraints":
                constraints = ttlv.ttlv_structure_create()
                try:
                    return extension.client_batch_item_set_constraints(constraints)
                finally:
                    constraints.close()
            if operation == "Set Defaults":
                return extension.client_batch_item_set_defaults()
            if operation == "Re-key":
                return extension.client_batch_item_rekey(identifier)
            if operation == "Re-key Key Pair":
                return extension.client_batch_item_rekey_key_pair(identifier)
            if operation == "Derive Key":
                derivation_parameters = self._structure(
                    [
                        self._item(0x0042_0084, "bytes", b"KMIPKit-PBKDF2-Salt"),
                        self._item(0x0042_003C, "integer", 1000),
                    ]
                )
                try:
                    return extension.client_batch_item_derive_key(
                        2,
                        [identifier],
                        1,
                        derivation_parameters,
                        attrs,
                    )
                finally:
                    derivation_parameters.close()
            if operation in {"Add Attribute", "Set Attribute", "Modify Attribute"}:
                comment = self._item(0x0042_00FD, "text", "KMIPKit interop test")
                if operation == "Add Attribute":
                    try:
                        return extension.client_batch_item_add_attribute(identifier, comment)
                    finally:
                        comment.close()
                if operation == "Set Attribute":
                    try:
                        return extension.client_batch_item_set_attribute(identifier, comment)
                    finally:
                        comment.close()
                comment.close()
                tag = 0x0042_00FD
                kind = "text"
                current_value = (
                    "old test value"
                    if case["case_id"] == "test-neg-modify-attribute-current-mismatch"
                    else "KMIPKit interop test"
                )
                current = self._legacy_attribute_item(current_value, tag=tag, kind=kind)
                updated = self._legacy_attribute_item("new test value", tag=tag, kind=kind)
                try:
                    return extension.client_batch_item_modify_attribute(identifier, current, updated)
                finally:
                    current.close()
                    updated.close()
            if operation == "Delete Attribute":
                current = self._legacy_attribute_item("KMIPKit interop test")
                try:
                    return extension.client_batch_item_delete_attribute(
                        identifier,
                        current,
                        extensions.AttributeReference.tag(0x0042_00FD),
                    )
                finally:
                    current.close()
            if operation == "Adjust Attribute":
                return extension.client_batch_item_adjust_attribute(
                    identifier,
                    extensions.AttributeReference.tag(0x0042_0001),
                    1,
                    ttlv.ttlv_value_integer(1),
                )
            if operation in {"Encrypt", "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify"}:
                with ttlv.codec_limits_defaults() as limits:
                    payload = ttlv.ttlv_value_byte_string(data, limits)
                    if operation == "Encrypt":
                        return extension.client_batch_item_encrypt(identifier, params, payload, bytes(16))
                    if operation == "Decrypt":
                        return extension.client_batch_item_decrypt(identifier, params, payload, bytes(16))
                    if operation == "MAC":
                        return extension.client_batch_item_mac(identifier, params, payload)
                    if operation == "MAC Verify":
                        return extension.client_batch_item_mac_verify(identifier, params, payload, auxiliary or bytes(32))
                    if operation == "Sign":
                        return extension.client_batch_item_sign(identifier, params, payload)
                    return extension.client_batch_item_signature_verify(identifier, params, payload, signature_data=auxiliary or bytes(256))
            if operation == "Hash":
                with ttlv.codec_limits_defaults() as limits:
                    payload = ttlv.ttlv_value_byte_string(data, limits)
                    return extension.client_batch_item_hash(params, payload)
            if operation == "RNG Retrieve":
                return extension.client_batch_item_rng_retrieve(32)
            if operation == "RNG Seed":
                return extension.client_batch_item_rng_seed(bytes(32))
            if operation == "Certify":
                return extension.client_batch_item_certify(
                    certificate_request_type=1,
                    certificate_request_value=(Path(__file__).resolve().parents[4] / "interop/rust-kmip-validator/tests/fixtures/certificate-request.der").read_bytes(),
                )
            if operation == "Re-certify":
                return extension.client_batch_item_recertify(
                    identifier,
                    certificate_request_type=1,
                    certificate_request_value=(Path(__file__).resolve().parents[4] / "interop/rust-kmip-validator/tests/fixtures/certificate-request.der").read_bytes(),
                )
            if operation == "Re-Provision":
                csr = (Path(__file__).resolve().parents[4] / "interop/rust-kmip-validator/tests/fixtures/certificate-request.der").read_bytes()
                request = self._structure(
                    [self._item(0x0042_0019, "enumeration", 1), self._item(0x0042_0140, "bytes", csr)]
                )
                return extension.client_batch_item_reprovision(certificate_request=request)
            if operation == "Validate":
                certificate_bytes = (
                    Path(__file__).resolve().parents[4]
                    / "interop/rust-kmip-validator/tests/fixtures/certificate.der"
                ).read_bytes()
                certificate = self._structure(
                    [
                        self._item(0x0042_001D, "enumeration", 1),
                        self._item(0x0042_001E, "bytes", certificate_bytes),
                    ]
                )
                try:
                    return extension.client_batch_item_validate([certificate], [])
                finally:
                    certificate.close()
            if operation == "PKCS#11":
                function = self._structure([self._item(0x0042_015A, "enumeration", 1)])
                try:
                    return extension.client_batch_item_pkcs11(function)
                finally:
                    function.close()
            if operation == "Join Split Key":
                split_identifiers = related_identifiers or [identifier, identifier]
                if len(split_identifiers) < 2:
                    raise LocalRecipeUnavailable("Create Split Key did not return two part identifiers")
                references = [extension.SplitKeyReference(value, 2) for value in split_identifiers[:2]]
                try:
                    return extension.client_batch_item_join_split_key(
                        2,
                        references,
                        attributes=attrs,
                    )
                finally:
                    for reference in references:
                        reference.close()
            raise LocalRecipeUnavailable("operation requires a recipe not available in the Python public API")
        finally:
            attrs.close()
            params.close()

    def _exchange(self, item: Any, *, asynchronous_indicator: int | None = None) -> dict[str, Any]:
        from kmipkit import protocol_values, ttlv

        batch = self._client_module.Batch.create()
        authentication = None
        limits = ttlv.codec_limits_defaults()
        try:
            if asynchronous_indicator is not None:
                batch.set_asynchronous_indicator(asynchronous_indicator)
            batch.add_item(item)
            authentication = self._authentication()
            if authentication is not None:
                batch.set_authentication(authentication)
                authentication = None
            response = self._client.execute(batch, limits)
            try:
                observed = response.item_at(0)
                status_code = int(observed.result_status)
                reason_code = observed.result_reason
                success = status_code == protocol_values.ResultStatus.SUCCESS
                pending = status_code == 2
                unsupported = (
                    status_code == protocol_values.ResultStatus.OPERATION_FAILED
                    and reason_code in (
                        protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                        protocol_values.ResultReason.FEATURE_NOT_SUPPORTED,
                    )
                )
                pending_correlation = observed.pending_correlation_value
                result = {
                    "status": "passed" if success else "pending" if pending else "server_unsupported" if unsupported else "failed",
                    "library_correctness": "passed",
                    "server_compatibility": "supported" if success else "unsupported" if unsupported else "unknown",
                    "parity": "not_compared",
                    "result_status_code": status_code,
                    "result_reason_code": None if reason_code is None else int(reason_code),
                    "_identifiers": [value for value in (
                        observed.unique_identifier,
                        observed.private_key_unique_identifier,
                        observed.public_key_unique_identifier,
                        *(observed.split_key_unique_identifiers or ()),
                    ) if value is not None],
                    "_bytes": self._response_bytes(observed),
                    "_ticket_ttlv": (
                        observed.login_response.ticket_ttlv
                        if observed.login_response is not None
                        else None
                    ),
                    "_pending_correlation": None if pending_correlation is None else bytearray(pending_correlation),
                }
                locate_response = observed.locate_response()
                if locate_response is not None:
                    result["_identifiers"].extend(locate_response.unique_identifiers)
                return result
            finally:
                response.close()
        finally:
            if authentication is not None:
                authentication.close()
            batch.close()
            limits.close()

    @staticmethod
    def _response_bytes(observed: Any) -> bytes | None:
        for name, member in (
            ("encrypt_response", "data"),
            ("decrypt_response", "data"),
            ("mac_response", "mac_data"),
            ("sign_response", "signature_data"),
            ("hash_response", "data"),
        ):
            result = getattr(observed, name, None)
            if result is not None:
                value = getattr(result, member, None)
                return value if isinstance(value, bytes) else None
        return None

    def _async_exchange(self, operation: str, correlation: bytes | bytearray | None = None) -> dict[str, Any]:
        from kmipkit import protocol_values, ttlv

        limits = ttlv.codec_limits_defaults()
        outcome = None
        try:
            if operation == "Query Asynchronous Requests":
                outcome = self._client.execute_query_async_requests(
                    limits,
                    correlation_values=None if correlation is None else [correlation],
                )
            elif operation == "Poll":
                outcome = self._client.execute_poll(correlation or b"", limits)
            elif operation == "Process":
                outcome = self._client.execute_process(
                    correlation or b"", limits, asynchronous_indicator=1
                )
            elif operation == "Cancel":
                outcome = self._client.execute_cancel(correlation or b"", limits)
            else:
                raise ValueError("unsupported asynchronous operation")
            metadata = outcome.metadata_v1
            status_code = int(metadata.result_status)
            reason_code = metadata.result_reason
            unsupported = (
                status_code == protocol_values.ResultStatus.OPERATION_FAILED
                and reason_code in (
                    protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                    protocol_values.ResultReason.FEATURE_NOT_SUPPORTED,
                )
            )
            next_correlation = bytearray()
            has_correlation = outcome.with_asynchronous_correlation_value(
                lambda value: next_correlation.extend(value)
            )
            response = (
                outcome.poll_response()
                if operation == "Poll"
                else outcome.process_response()
                if operation == "Process"
                else outcome.query_async_requests_response()
                if operation == "Query Asynchronous Requests"
                else outcome.cancel_response()
            )
            is_pending = bool(getattr(response, "is_pending", False))
            if not has_correlation:
                next_correlation = None
            return {
                "status": "passed" if status_code in (protocol_values.ResultStatus.SUCCESS, 2) else "server_unsupported" if unsupported else "failed",
                "library_correctness": "passed",
                "server_compatibility": "supported" if status_code in (protocol_values.ResultStatus.SUCCESS, 2) else "unsupported" if unsupported else "unknown",
                "parity": "not_compared",
                "result_status_code": status_code,
                "result_reason_code": None if reason_code is None else int(reason_code),
                "_pending": is_pending,
                "_correlation": next_correlation,
            }
        finally:
            if outcome is not None:
                outcome.close()
            limits.close()

    def _start_async_create(self, name: str) -> dict[str, Any]:
        attributes = self._attributes(name, 3, 128)
        try:
            item = self._extensions.client_batch_item_create(2, attributes)
        finally:
            attributes.close()
        return self._exchange(item, asynchronous_indicator=1)

    def _cleanup_async_fixture(self, name: str, identifiers: list[Any]) -> bool:
        if not identifiers:
            catalog, _ = load_catalog(DEFAULT_CATALOG)
            locate_case = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-locate")
            locate = self._exchange(self._build_item(locate_case, None, name=name))
            if locate["status"] != "passed":
                return False
            identifiers.extend(locate.get("_identifiers", []))
        clean = True
        for identifier in reversed(identifiers):
            try:
                result = self._exchange(self._extensions.client_batch_item_destroy(identifier))
                clean = clean and result["status"] in {"passed", "server_unsupported"}
            except Exception:
                clean = False
        return clean

    def _run_async_case(self, test_case: dict[str, Any], name: str) -> dict[str, Any]:
        operation = test_case["operation"]
        if test_case.get("requires", {}).get("expected_status") == "NotSent":
            from kmipkit import errors, ttlv

            limits = ttlv.codec_limits_defaults()
            outcome = None
            try:
                if operation == "Process":
                    outcome = self._client.execute_process(b"kmipkit-version-boundary", limits)
                else:
                    outcome = self._client.execute_query_async_requests(limits)
                if outcome is not None:
                    outcome.close()
                return {
                    "status": "failed",
                    "library_correctness": "failed",
                    "server_compatibility": "unknown",
                    "parity": "not_compared",
                    "diagnostic": "version-gated request was accepted instead of being rejected before transport",
                }
            except errors.ClientExecutionError as error:
                if error.delivery is errors.DeliveryState.NOT_SENT:
                    return {
                        "status": "passed",
                        "library_correctness": "passed",
                        "server_compatibility": "unknown",
                        "parity": "not_compared",
                        "diagnostic": "KMIPKit rejected the version-boundary request before transport",
                    }
                return {
                    "status": "failed",
                    "library_correctness": "failed",
                    "server_compatibility": "unknown",
                    "parity": "not_compared",
                    "diagnostic": "version-boundary failure occurred after request delivery began",
                }
            finally:
                if outcome is not None:
                    outcome.close()
                limits.close()
        if operation == "Query Asynchronous Requests":
            result = self._async_exchange(operation)
            result.pop("_pending", None)
            result.pop("_correlation", None)
            return result
        setup = self._start_async_create(name)
        identifiers = list(setup.get("_identifiers", []))
        final: dict[str, Any]
        correlation = setup.get("_pending_correlation")
        try:
            if setup["status"] == "server_unsupported" or setup["status"] == "failed":
                final = setup
            elif setup["status"] != "pending" or not correlation:
                final = {
                    "status": "not_run",
                    "library_correctness": "passed",
                    "server_compatibility": "supported",
                    "parity": "not_compared",
                    "diagnostic": "server completed the asynchronous Create inline; no correlation value was available",
                }
            elif test_case.get("requires", {}).get("workflow_type") == "async_poll":
                current = correlation
                final = {
                    "status": "not_run",
                    "library_correctness": "passed",
                    "server_compatibility": "supported",
                    "parity": "not_compared",
                    "diagnostic": "server remained Pending after the bounded Poll sequence",
                }
                for _ in range(16):
                    final = self._async_exchange("Poll", current)
                    if final["status"] != "passed" or not final["_pending"]:
                        final.pop("_pending", None)
                        final.pop("_correlation", None)
                        break
                    next_value = final.get("_correlation")
                    if not next_value:
                        final.update(
                            status="failed",
                            library_correctness="failed",
                            server_compatibility="unknown",
                            diagnostic="Pending Poll response omitted its correlation value",
                        )
                        break
                    if isinstance(current, bytearray):
                        current[:] = bytes(len(current))
                    current = next_value
                else:
                    final = {
                        "status": "not_run",
                        "library_correctness": "passed",
                        "server_compatibility": "supported",
                        "parity": "not_compared",
                        "diagnostic": "server remained Pending after the bounded Poll sequence",
                    }
                if isinstance(current, bytearray):
                    current[:] = bytes(len(current))
            else:
                final = self._async_exchange(operation, correlation)
                final.pop("_pending", None)
                final.pop("_correlation", None)
        finally:
            if isinstance(correlation, bytearray):
                correlation[:] = bytes(len(correlation))
            cleanup_succeeded = self._cleanup_async_fixture(name, identifiers)
        if not cleanup_succeeded:
            final.update(
                status="failed",
                library_correctness="not_checked",
                server_compatibility="unknown",
                diagnostic="asynchronous fixture cleanup failed",
            )
        return final

    def execute_case(self, test_case: dict[str, Any]) -> dict[str, Any]:
        case_id = test_case["case_id"]
        operation = test_case["operation"]
        unique_name = f"kmipkit-python-{case_id}-{uuid.uuid4()}"
        if operation in {"Poll", "Process", "Cancel", "Query Asynchronous Requests"} or test_case.get("requires", {}).get("workflow_type") == "async_poll":
            return self._run_async_case(test_case, unique_name)
        identifiers: list[Any] = []
        target_identifier = None
        setup_required = operation not in self._NO_OBJECT
        algorithm_name = test_case.get("requires", {}).get("algorithm", "AES")
        algorithm = {"AES": 3, "RSA": 4, "ECDSA": 6, "HMAC_SHA256": 9}.get(algorithm_name, 3)
        bits = int(test_case.get("requires", {}).get("key_length", 256))
        outcome: dict[str, Any] | None = None
        try:
            if setup_required:
                attributes = self._attributes(unique_name, algorithm, bits)
                try:
                    if operation == "Join Split Key":
                        setup_item = self._extensions.client_batch_item_create_split_key(2, 2, 2, 1, attributes)
                    elif algorithm in {4, 6}:
                        setup_item = self._extensions.client_batch_item_create_key_pair(common_attributes=attributes)
                    else:
                        setup_item = self._extensions.client_batch_item_create(2, attributes)
                    setup = self._exchange(setup_item)
                finally:
                    attributes.close()
                created_ids = setup.get("_identifiers", [])
                if setup["status"] != "passed":
                    return setup
                if not created_ids:
                    return {
                        "status": "failed", "library_correctness": "failed",
                        "server_compatibility": "unknown", "parity": "not_compared",
                        "diagnostic": "successful Create omitted Unique Identifier",
                    }
                identifiers.extend(created_ids)
                target_identifier = created_ids[0]
                if operation in {"Encrypt", "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify"} and case_id != "test-neg-encrypt-preactive-key":
                    for key_identifier in created_ids:
                        activated = self._exchange(self._extensions.client_batch_item_activate(key_identifier))
                        if activated["status"] != "passed":
                            return activated
                if operation == "Signature Verify" and len(created_ids) > 1:
                    target_identifier = created_ids[1]
            if operation == "Logout":
                login = self._exchange(self._extensions.client_batch_item_login(request_count=2))
                if login["status"] != "passed":
                    return login
                ticket = login.get("_ticket_ttlv")
                if ticket is None:
                    raise LocalRecipeUnavailable("Python Login response has no public Ticket recipe")
                try:
                    result = self._exchange(self._extensions.client_batch_item_logout(ticket))
                finally:
                    ticket.close()
                outcome = result
                return result
            elif operation == "Join Split Key":
                if len(identifiers) < 2:
                    raise LocalRecipeUnavailable("server did not return two identifiers for Create Split Key")
                references = [self._extensions.SplitKeyReference(value, 2) for value in identifiers[:2]]
                try:
                    target = self._extensions.client_batch_item_join_split_key(2, references)
                finally:
                    for reference in references:
                        reference.close()
            elif operation == "Decrypt":
                encrypted = self._exchange(self._build_item(test_case | {"operation": "Encrypt"}, target_identifier, name=unique_name))
                if encrypted["status"] != "passed" or encrypted.get("_bytes") is None:
                    return encrypted
                target = self._build_item(test_case, target_identifier, name=unique_name, data=encrypted["_bytes"], related_identifiers=identifiers)
            elif operation in {"MAC Verify", "Signature Verify"}:
                source_case = test_case | {"operation": "MAC" if operation == "MAC Verify" else "Sign"}
                signature = self._exchange(self._build_item(source_case, target_identifier, name=unique_name))
                if signature["status"] != "passed" or signature.get("_bytes") is None:
                    return signature
                target = self._build_item(test_case, target_identifier, name=unique_name, auxiliary=signature["_bytes"], related_identifiers=identifiers)
            else:
                target = (
                    self._build_high_level_item(test_case, target_identifier, name=unique_name)
                    if self._api_level == "high-level"
                    else self._build_item(test_case, target_identifier, name=unique_name, related_identifiers=identifiers)
                )
            result = self._exchange(target)
            identifiers.extend(result.get("_identifiers", []))
            if operation == "Decrypt" and result["status"] == "passed" and result.get("_bytes") != b"A" * 16:
                result.update(status="failed", library_correctness="failed", server_compatibility="unknown")
            outcome = result
            if operation == "Destroy" and result["status"] == "passed" and target_identifier in identifiers:
                identifiers.remove(target_identifier)
        finally:
            for identifier in reversed(identifiers):
                try:
                    cleanup = self._exchange(self._extensions.client_batch_item_destroy(identifier))
                    if cleanup["status"] not in {"passed", "server_unsupported"} and outcome is not None:
                        outcome.update(
                            status="failed",
                            library_correctness="not_checked",
                            server_compatibility="unknown",
                            diagnostic="test fixture cleanup failed",
                        )
                except Exception:
                    if outcome is not None:
                        outcome.update(
                            status="failed",
                            library_correctness="not_checked",
                            server_compatibility="unknown",
                            diagnostic="test fixture cleanup failed",
                        )
        return outcome or {
            "status": "failed", "library_correctness": "not_checked",
            "server_compatibility": "unknown", "parity": "not_compared",
        }

    def close(self) -> None:
        self._client.close()


@dataclass(frozen=True)
class NormalizedResponse:
    status: str
    result_reason: str = ""
    object_type: str = ""
    sanitized_payload_hex: str = ""


@dataclass(frozen=True)
class EquivalenceResult:
    is_equivalent: bool
    discrepancies: list[str]


class EquivalenceVerifier:
    @staticmethod
    def verify_equivalence(responses: dict[str, NormalizedResponse]) -> EquivalenceResult:
        discrepancies: list[str] = []
        if not responses or len(responses) <= 1:
            return EquivalenceResult(is_equivalent=True, discrepancies=discrepancies)

        items = list(responses.items())
        base_label, base_resp = items[0]
        for label, resp in items[1:]:
            if resp.status != base_resp.status:
                discrepancies.append(
                    f"[{label}] status '{resp.status}' differs from [{base_label}] '{base_resp.status}'"
                )
            if resp.result_reason != base_resp.result_reason:
                discrepancies.append(
                    f"[{label}] result_reason '{resp.result_reason}' differs from [{base_label}] '{base_resp.result_reason}'"
                )
            if resp.object_type != base_resp.object_type:
                discrepancies.append(
                    f"[{label}] object_type '{resp.object_type}' differs from [{base_label}] '{base_resp.object_type}'"
                )
            if resp.sanitized_payload_hex != base_resp.sanitized_payload_hex:
                discrepancies.append(f"[{label}] payload differs from [{base_label}]")

        return EquivalenceResult(is_equivalent=len(discrepancies) == 0, discrepancies=discrepancies)


class RecipeRunner:
    @staticmethod
    def execute_with_guaranteed_teardown(
        setup_fn: Callable[[], bool] | None,
        target_fn: Callable[[], Any] | None,
        teardown_fn: Callable[[], bool] | None,
    ) -> Any:
        if setup_fn and not setup_fn():
            raise RuntimeError("Setup phase failed")
        try:
            return target_fn() if target_fn else None
        finally:
            if teardown_fn:
                teardown_fn()
