"""KMIP 2.1 Request Dispatcher with Profile Enforcement."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .state import (
    IllegalLifecycleTransitionError,
    KmsState,
    NameCollisionError,
    ObjectNotFoundError,
)


class MockKmsDispatcher:
    """Dispatches KMIP requests against in-memory state according to a loaded profile."""

    def __init__(self, profile_data: dict[str, Any] | None = None) -> None:
        self.profile = profile_data or self._default_profile()
        self.state = KmsState()
        self.supported_operations: dict[str, bool] = {
            k: v.get("supported", False) for k, v in self.profile.get("operations", {}).items()
        }
        self.supported_objects: dict[str, bool] = {
            k: v.get("supported", False) for k, v in self.profile.get("managed_objects", {}).items()
        }

    @classmethod
    def from_profile_file(cls, profile_path: Path) -> MockKmsDispatcher:
        p = Path(profile_path).resolve()
        with open(p, "r", encoding="utf-8") as f:  # NOSONAR
            data = json.load(f)
        return cls(data)

    def dispatch(self, request: dict[str, Any]) -> dict[str, Any]:
        """Dispatch a single KMIP request payload."""
        op = request.get("operation")
        if not op:
            return {
                "result_status": "OperationFailed",
                "result_reason": "InvalidMessage",
                "message": "Missing 'operation' in request",
            }

        # 1. Profile enforcement: Check if operation is supported
        # Normalize operation string (e.g. "Discover Versions" or "DiscoverVersions")
        canonical_op = self._canonical_operation_name(op)
        if canonical_op in self.supported_operations and not self.supported_operations[canonical_op]:
            return {
                "result_status": "OperationFailed",
                "result_reason": "OperationNotSupported",
                "message": f"Operation '{op}' is not supported by {self.profile.get('server', {}).get('product', 'KMS')}",
            }

        # 2. Route to operation handlers
        handler_name = f"_handle_{canonical_op.lower().replace(' ', '_').replace('-', '_')}"
        handler = getattr(self, handler_name, None)
        if handler:
            return handler(request)

        # Fallback generic success for supported but unmodeled operations
        return {
            "result_status": "Success",
            "operation": op,
            "message": f"Simulated success for {op}",
        }

    def _canonical_operation_name(self, op: str) -> str:
        # Map common variations
        mapping = {
            "discoverversions": "Discover Versions",
            "discover_versions": "Discover Versions",
            "createkeypair": "Create Key Pair",
            "create_key_pair": "Create Key Pair",
            "createsplitkey": "Create Split Key",
            "create_split_key": "Create Split Key",
            "joinsplitkey": "Join Split Key",
            "join_split_key": "Join Split Key",
            "getattributes": "Get Attributes",
            "get_attributes": "Get Attributes",
            "getattributelist": "Get Attribute List",
            "get_attribute_list": "Get Attribute List",
            "addattribute": "Add Attribute",
            "add_attribute": "Add Attribute",
            "modifyattribute": "Modify Attribute",
            "modify_attribute": "Modify Attribute",
            "deleteattribute": "Delete Attribute",
            "delete_attribute": "Delete Attribute",
            "signatureverify": "Signature Verify",
            "signature_verify": "Signature Verify",
            "macverify": "MAC Verify",
            "mac_verify": "MAC Verify",
            "rngretrieve": "RNG Retrieve",
            "rng_retrieve": "RNG Retrieve",
        }
        clean = op.strip().lower().replace(" ", "").replace("_", "")
        return mapping.get(clean, op.strip())

    def _handle_discover_versions(self, request: dict[str, Any]) -> dict[str, Any]:
        versions = self.profile.get("server", {}).get("kmip_versions", ["2.1"])
        return {
            "result_status": "Success",
            "supported_versions": versions,
        }

    def _handle_query(self, request: dict[str, Any]) -> dict[str, Any]:
        server_info = self.profile.get("server", {})
        ops = [k for k, v in self.supported_operations.items() if v]
        objs = [k for k, v in self.supported_objects.items() if v]
        return {
            "result_status": "Success",
            "server_information": {
                "vendor": server_info.get("vendor", "Generic KMIP Vendor"),
                "product": server_info.get("product", "Mock KMS"),
                "version": server_info.get("version", "2.1.0"),
            },
            "operations": ops,
            "object_types": objs,
        }

    def _handle_create(self, request: dict[str, Any]) -> dict[str, Any]:
        name = request.get("name", "")
        obj_type = request.get("object_type", "SymmetricKey")
        alg = request.get("algorithm") or request.get("cryptographic_algorithm")
        length = request.get("length") or request.get("cryptographic_length")
        attrs = request.get("attributes", {})

        try:
            obj = self.state.create_object(
                name=name,
                object_type=obj_type,
                algorithm=alg,
                length=length,
                initial_state="Active",
                attributes=attrs,
            )
            return {
                "result_status": "Success",
                "unique_identifier": obj.unique_identifier,
                "object_type": obj.object_type,
                "name": obj.name,
            }
        except NameCollisionError as e:
            return {
                "result_status": "OperationFailed",
                "result_reason": "IndexAlreadyExists",
                "message": str(e),
            }

    def _handle_create_key_pair(self, request: dict[str, Any]) -> dict[str, Any]:
        base_name = request.get("name", "")
        priv_name = f"{base_name}-private" if base_name else ""
        pub_name = f"{base_name}-public" if base_name else ""

        try:
            priv_obj = self.state.create_object(name=priv_name, object_type="PrivateKey")
            pub_obj = self.state.create_object(name=pub_name, object_type="PublicKey")
            return {
                "result_status": "Success",
                "private_key_unique_identifier": priv_obj.unique_identifier,
                "public_key_unique_identifier": pub_obj.unique_identifier,
            }
        except NameCollisionError as e:
            return {
                "result_status": "OperationFailed",
                "result_reason": "IndexAlreadyExists",
                "message": str(e),
            }

    def _handle_register(self, request: dict[str, Any]) -> dict[str, Any]:
        return self._handle_create(request)

    def _handle_get(self, request: dict[str, Any]) -> dict[str, Any]:
        uid = request.get("unique_identifier")
        if not uid:
            return {"result_status": "OperationFailed", "result_reason": "InvalidField", "message": "Missing unique_identifier"}
        try:
            obj = self.state.get_object(uid)
            return {
                "result_status": "Success",
                "object": obj.to_dict(),
            }
        except ObjectNotFoundError as e:
            return {
                "result_status": "OperationFailed",
                "result_reason": "ItemNotFound",
                "message": str(e),
            }

    def _handle_locate(self, request: dict[str, Any]) -> dict[str, Any]:
        name_prefix = request.get("name_prefix")
        name_exact = request.get("name")
        obj_type = request.get("object_type")
        state = request.get("state")

        uids = self.state.locate_objects(
            name_prefix=name_prefix,
            name_exact=name_exact,
            object_type=obj_type,
            state=state,
        )
        return {
            "result_status": "Success",
            "unique_identifiers": uids,
            "count": len(uids),
        }

    def _handle_activate(self, request: dict[str, Any]) -> dict[str, Any]:
        uid = request.get("unique_identifier")
        try:
            obj = self.state.activate_object(uid)
            return {"result_status": "Success", "unique_identifier": obj.unique_identifier, "state": obj.state}
        except ObjectNotFoundError as e:
            return {"result_status": "OperationFailed", "result_reason": "ItemNotFound", "message": str(e)}

    def _handle_revoke(self, request: dict[str, Any]) -> dict[str, Any]:
        uid = request.get("unique_identifier")
        reason = request.get("revocation_reason", "CessationOfOperation")
        try:
            obj = self.state.revoke_object(uid, reason=reason)
            return {"result_status": "Success", "unique_identifier": obj.unique_identifier, "state": obj.state}
        except ObjectNotFoundError as e:
            return {"result_status": "OperationFailed", "result_reason": "ItemNotFound", "message": str(e)}

    def _handle_destroy(self, request: dict[str, Any]) -> dict[str, Any]:
        uid = request.get("unique_identifier")
        allow_active = request.get("allow_active_destroy", False)
        try:
            obj = self.state.destroy_object(uid, allow_active=allow_active)
            return {"result_status": "Success", "unique_identifier": obj.unique_identifier, "state": "Destroyed"}
        except ObjectNotFoundError as e:
            return {"result_status": "OperationFailed", "result_reason": "ItemNotFound", "message": str(e)}
        except IllegalLifecycleTransitionError as e:
            return {"result_status": "OperationFailed", "result_reason": "IllegalOperation", "message": str(e)}

    def _handle_encrypt(self, request: dict[str, Any]) -> dict[str, Any]:
        return {
            "result_status": "Success",
            "data": "73696d756c617465642d63697068657274657874",  # hex
            "iv_counter_nonce": "000102030405060708090a0b0c0d0e0f",
        }

    def _handle_decrypt(self, request: dict[str, Any]) -> dict[str, Any]:
        return {
            "result_status": "Success",
            "data": "73696d756c617465642d706c61696e74657874",  # hex
        }

    def _handle_sign(self, request: dict[str, Any]) -> dict[str, Any]:
        return {
            "result_status": "Success",
            "signature": "73696d756c617465642d7369676e6174757265",
        }

    def _handle_signature_verify(self, request: dict[str, Any]) -> dict[str, Any]:
        return {
            "result_status": "Success",
            "validity_indicator": "Valid",
        }

    def _default_profile(self) -> dict[str, Any]:
        return {
            "schema_version": "1.0",
            "server": {"id": "mock-kms-default", "vendor": "KMIPKit", "product": "Mock KMS", "version": "2.1.0", "kmip_versions": ["2.1"]},
            "operations": {
                "Discover Versions": {"supported": True},
                "Query": {"supported": True},
                "Create": {"supported": True},
                "Create Key Pair": {"supported": True},
                "Register": {"supported": True},
                "Get": {"supported": True},
                "Locate": {"supported": True},
                "Activate": {"supported": True},
                "Revoke": {"supported": True},
                "Destroy": {"supported": True},
                "Archive": {"supported": False},
                "Recover": {"supported": False},
            },
        }
