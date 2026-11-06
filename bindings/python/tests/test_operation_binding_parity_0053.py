"""Executable Python request-entry coverage for the KMIP 2.1 client operations."""

from __future__ import annotations

import json
from pathlib import Path
from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit.protocol_values import Operation
from kmipkit._client_ffi import ffi

_ROOT = Path(__file__).resolve().parents[3]
_MANIFEST = _ROOT / "specification" / "api" / "public-api.json"
_UNIQUE_IDENTIFIER = extensions.UniqueIdentifier.text_string("operation-parity")


def _structure() -> ttlv.Structure:
    return object.__new__(ttlv.Structure)


def _item() -> ttlv.TtlvItem:
    return object.__new__(ttlv.TtlvItem)


def _register_request() -> extensions.RegisterRequest:
    return object.__new__(extensions.RegisterRequest)


def _revoke_request() -> extensions.RevokeRequest:
    return object.__new__(extensions.RevokeRequest)


def _split_key_reference() -> object:
    return _FakeSplitKeyReference()


def test_real_batch_item_reports_its_kmip_operation_value() -> None:
    with extensions.client_batch_item_ping() as item:
        assert item.operation == Operation.PING


class _FakeSplitKeyReference:
    split_key_threshold = 1

    @staticmethod
    def _pointer() -> object:
        return extensions.ffi.cast("kmipkit_split_key_reference_t *", 1)


# Each case calls a public request builder. The argument factories provide only
# the minimum typed inputs needed to reach that operation's native ABI route.
_BATCH_OPERATION_CASES = {
    "Activate": (
        "client_batch_item_activate",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Add Attribute": (
        "client_batch_item_add_attribute",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER, "new_attribute": _item()},
    ),
    "Adjust Attribute": (
        "client_batch_item_adjust_attribute",
        lambda: {
            "unique_identifier": _UNIQUE_IDENTIFIER,
            "attribute_reference": extensions.AttributeReference.tag(0x420094),
            "adjustment_type": 0,
        },
    ),
    "Archive": (
        "client_batch_item_archive",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Certify": (
        "client_batch_item_certify",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Check": (
        "client_batch_item_check",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Create": (
        "client_batch_item_create",
        lambda: {"object_type": 0, "attributes": _structure()},
    ),
    "Create Key Pair": ("client_batch_item_create_key_pair", dict),
    "Create Split Key": (
        "client_batch_item_create_split_key",
        lambda: {
            "object_type": 0,
            "split_key_parts": 1,
            "split_key_threshold": 1,
            "split_key_method": 0,
            "attributes": _structure(),
        },
    ),
    "Decrypt": ("client_batch_item_decrypt", dict),
    "Delegated Login": (
        "client_batch_item_delegated_login",
        lambda: {"rights": _structure()},
    ),
    "Delete Attribute": (
        "client_batch_item_delete_attribute",
        lambda: {
            "unique_identifier": _UNIQUE_IDENTIFIER,
            "current_attribute": None,
            "attribute_reference": None,
        },
    ),
    "Derive Key": (
        "client_batch_item_derive_key",
        lambda: {
            "object_type": 0,
            "unique_identifiers": (_UNIQUE_IDENTIFIER,),
            "derivation_method": 0,
            "derivation_parameters": _structure(),
            "attributes": _structure(),
        },
    ),
    "Destroy": (
        "client_batch_item_destroy",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Discover Versions": ("client_batch_item_discover_versions", dict),
    "Encrypt": ("client_batch_item_encrypt", dict),
    "Export": (
        "client_batch_item_export",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Get": ("client_batch_item_get", lambda: {"unique_identifier": _UNIQUE_IDENTIFIER}),
    "Get Attributes": (
        "client_batch_item_get_attributes",
        lambda: {
            "unique_identifier": _UNIQUE_IDENTIFIER,
            "attribute_references": (extensions.AttributeReference.tag(0x420094),),
        },
    ),
    "Get Attribute List": (
        "client_batch_item_get_attribute_list",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Get Constraints": ("client_batch_item_get_constraints", dict),
    "Get Usage Allocation": (
        "client_batch_item_get_usage_allocation",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER, "usage_limits_count": 1},
    ),
    "Hash": ("client_batch_item_hash", lambda: {"parameters": _structure()}),
    "Import": (
        "client_batch_item_import",
        lambda: {
            "unique_identifier": _UNIQUE_IDENTIFIER,
            "object_type": 0,
            "attributes": _structure(),
            "any_object": _structure(),
        },
    ),
    "Interop": (
        "client_batch_item_interop",
        lambda: {"interop_function": 0, "interop_identifier": "operation-parity"},
    ),
    "Join Split Key": (
        "client_batch_item_join_split_key",
        lambda: {"object_type": 0, "references": (_split_key_reference(),)},
    ),
    "Locate": ("client_batch_item_locate", lambda: {"attributes": _structure()}),
    "Log": ("client_batch_item_log", lambda: {"log_message": "operation-parity"}),
    "Login": ("client_batch_item_login", dict),
    "Logout": ("client_batch_item_logout", lambda: {"ticket": _structure()}),
    "MAC": ("client_batch_item_mac", dict),
    "MAC Verify": ("client_batch_item_mac_verify", dict),
    "Modify Attribute": (
        "client_batch_item_modify_attribute",
        lambda: {
            "unique_identifier": _UNIQUE_IDENTIFIER,
            "current_attribute": None,
            "new_attribute": _item(),
        },
    ),
    "Obtain Lease": (
        "client_batch_item_obtain_lease",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Ping": ("client_batch_item_ping", dict),
    "PKCS#11": ("client_batch_item_pkcs11", lambda: {"function": _structure()}),
    "Query": ("client_batch_item_query", lambda: {"functions": (0,)}),
    "Recover": (
        "client_batch_item_recover",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Register": (
        "client_batch_item_register",
        lambda: {"request": _register_request()},
    ),
    "Revoke": ("client_batch_item_revoke", lambda: {"request": _revoke_request()}),
    "Re-certify": (
        "client_batch_item_recertify",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Re-key": (
        "client_batch_item_rekey",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Re-key Key Pair": (
        "client_batch_item_rekey_key_pair",
        lambda: {"private_key_unique_identifier": _UNIQUE_IDENTIFIER},
    ),
    "Re-Provision": ("client_batch_item_reprovision", dict),
    "RNG Retrieve": ("client_batch_item_rng_retrieve", lambda: {"data_length": 1}),
    "RNG Seed": ("client_batch_item_rng_seed", lambda: {"data": b"seed"}),
    "Set Attribute": (
        "client_batch_item_set_attribute",
        lambda: {"unique_identifier": _UNIQUE_IDENTIFIER, "new_attribute": _item()},
    ),
    "Set Constraints": (
        "client_batch_item_set_constraints",
        lambda: {"constraints": _structure()},
    ),
    "Set Defaults": ("client_batch_item_set_defaults", dict),
    "Sign": ("client_batch_item_sign", dict),
    "Signature Verify": ("client_batch_item_signature_verify", dict),
    "Validate": (
        "client_batch_item_validate",
        lambda: {"certificates": (), "unique_identifiers": (_UNIQUE_IDENTIFIER,)},
    ),
}

_ASYNC_OPERATION_CASES = {
    "Cancel": (
        "execute_cancel",
        "kmipkit_client_execute_cancel",
        (b"cancel-correlation",),
    ),
    "Poll": ("execute_poll", "kmipkit_client_execute_poll", (b"poll-correlation",)),
    "Process": (
        "execute_process",
        "kmipkit_client_execute_process",
        (b"process-correlation",),
    ),
    "Query Asynchronous Requests": (
        "execute_query_async_requests",
        "kmipkit_client_execute_query_async_requests",
        (),
    ),
}


def _client_operations() -> list[dict[str, object]]:
    manifest = json.loads(_MANIFEST.read_text(encoding="utf-8"))
    return [
        operation
        for operation in manifest["operations"]
        if operation.get("scope") == "client_1_0"
    ]


def test_request_entrypoint_matrix_accounts_for_all_56_operations() -> None:
    operations = _client_operations()
    operation_names = {str(operation["name"]) for operation in operations}

    assert len(operations) == 56
    assert len({operation["operationId"] for operation in operations}) == 56
    assert set(_BATCH_OPERATION_CASES) | set(_ASYNC_OPERATION_CASES) == operation_names
    assert not (set(_BATCH_OPERATION_CASES) & set(_ASYNC_OPERATION_CASES))


@pytest.mark.parametrize(
    ("operation_name", "builder_name", "arguments_factory"),
    [
        (name, builder, arguments)
        for name, (builder, arguments) in _BATCH_OPERATION_CASES.items()
    ],
    ids=list(_BATCH_OPERATION_CASES),
)
def test_batch_operation_builder_invokes_its_native_request_entrypoint(
    monkeypatch: pytest.MonkeyPatch,
    operation_name: str,
    builder_name: str,
    arguments_factory,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    wrapped = object()
    expected_symbol = (
        f"kmipkit_client_batch_item_{builder_name.removeprefix('client_batch_item_')}"
    )
    if builder_name == "client_batch_item_pkcs11":
        expected_symbol += "_with_extensions_v1"
    uses_client_ffi = expected_symbol in extensions._CLIENT_ABI_HANDLE_FUNCTIONS
    native_ffi = extensions._client_ffi if uses_client_ffi else extensions.ffi

    def invoke_native(*arguments: object) -> int:
        calls.append((expected_symbol, arguments[:-1]))
        arguments[-1][0] = native_ffi.cast("kmipkit_client_batch_item_t *", 43)  # type: ignore[index]
        return 0

    symbols = {expected_symbol: invoke_native}
    monkeypatch.setattr(extensions, "lib", SimpleNamespace(**symbols))
    monkeypatch.setattr(extensions, "_client_lib", SimpleNamespace(**symbols))
    monkeypatch.setattr(
        extensions,
        "_invoke_consuming",
        lambda function, arguments, _indices: function(*arguments),
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: wrapped)
    monkeypatch.setattr(
        extensions, "_native_structure", lambda _value, **_kwargs: ffi.NULL
    )
    monkeypatch.setattr(
        extensions, "_client_native_structure", lambda _value, **_kwargs: ffi.NULL
    )
    monkeypatch.setattr(extensions, "_client_native_item", lambda _value: ffi.NULL)
    monkeypatch.setattr(extensions, "_native_value", lambda _value: ffi.NULL)
    monkeypatch.setattr(extensions, "_client_native_value", lambda _value: ffi.NULL)
    monkeypatch.setattr(
        extensions,
        "_client_native_attribute_reference",
        lambda _value, _staging: ffi.NULL,
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_attribute_reference_array",
        lambda _values, _staging: (ffi.NULL, 1),
    )
    monkeypatch.setattr(extensions, "SplitKeyReference", _FakeSplitKeyReference)

    builder = getattr(extensions, builder_name)
    result = builder(**arguments_factory())

    assert result is wrapped, operation_name
    assert len(calls) == 1, operation_name
    assert calls[0][0] == expected_symbol, operation_name


@pytest.mark.parametrize(
    ("operation_name", "method_name", "native_symbol", "method_arguments"),
    [
        (name, method, symbol, arguments)
        for name, (method, symbol, arguments) in _ASYNC_OPERATION_CASES.items()
    ],
    ids=list(_ASYNC_OPERATION_CASES),
)
def test_async_operation_entrypoint_calls_native_method(
    monkeypatch: pytest.MonkeyPatch,
    operation_name: str,
    method_name: str,
    native_symbol: str,
    method_arguments: tuple[bytes, ...],
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []

    def reject_request(*arguments: object) -> int:
        calls.append((native_symbol, arguments))
        return 4

    def last_error(
        status: object, category: object, cause: object, delivery: object
    ) -> int:
        status[0] = 4  # type: ignore[index]
        category[0] = int(errors.ClientErrorCategory.VALIDATION)  # type: ignore[index]
        cause[0] = int(errors.ClientErrorCause.INVALID_INPUT)  # type: ignore[index]
        delivery[0] = int(errors.DeliveryState.NOT_SENT)  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            **{
                native_symbol: reject_request,
                "kmipkit_last_error_get": last_error,
            }
        ),
    )
    monkeypatch.setattr(
        client.Client,
        "_pointer",
        lambda _self: ffi.cast("kmipkit_client_t *", 41),
    )
    limits = object.__new__(ttlv.CodecLimits)
    monkeypatch.setattr(
        ttlv.CodecLimits,
        "_pointer",
        lambda _self: ffi.cast("kmipkit_codec_limits_t *", 42),
    )
    method = getattr(client.Client, method_name)

    with pytest.raises(errors.ClientExecutionError) as failure:
        if method_name == "execute_query_async_requests":
            method(object.__new__(client.Client), limits)
        else:
            method(object.__new__(client.Client), *method_arguments, limits)

    assert len(calls) == 1, operation_name
    assert calls[0][0] == native_symbol, operation_name
    assert failure.value.delivery is errors.DeliveryState.NOT_SENT
