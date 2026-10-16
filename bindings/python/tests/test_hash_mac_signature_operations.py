"""Python CFFI parity tests for KMIPKIT-0043 T033.

The extension facade forwards typed TTLV values and opaque operation bytes to
the C ABI. These tests stub only the batch-item constructor boundary; no
network request or local cryptographic operation is performed.
"""

from __future__ import annotations

from dataclasses import fields
from types import SimpleNamespace

import pytest

from kmipkit import client, extensions, ttlv


_OPERATIONS = (
    "hash",
    "mac",
    "mac_verify",
    "sign",
    "signature_verify",
)


def _capture_constructor(
    monkeypatch: pytest.MonkeyPatch,
) -> list[tuple[str, tuple[object, ...]]]:
    calls: list[tuple[str, tuple[object, ...]]] = []

    def make_handle(
        _c_type: str,
        function_name: str,
        arguments: tuple[object, ...],
        *_args: object,
        **_kwargs: object,
    ) -> object:
        calls.append((function_name, arguments))
        return object()

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(
        extensions,
        "_wrap_owned_handle",
        lambda _wrapper, handle, *_args, **_kwargs: handle,
    )
    return calls


def _call_operation(
    operation: str,
    *,
    parameters: ttlv.Structure,
    data: ttlv.Value | None,
    correlation_value: bytes | None = None,
    opaque_bytes: bytes | None = None,
    opaque_bytes_2: bytes | None = None,
    init_indicator: bool | None = None,
    final_indicator: bool | None = None,
) -> object:
    function = getattr(extensions, f"client_batch_item_{operation}", None)
    assert callable(function), f"missing Python operation builder: {operation}"
    arguments: dict[str, object] = {
        "parameters": parameters,
        "data": data,
        "correlation_value": correlation_value,
        "init_indicator": init_indicator,
        "final_indicator": final_indicator,
    }
    if operation != "hash":
        arguments["unique_identifier"] = None
    if operation == "mac_verify":
        arguments["mac_data"] = opaque_bytes
    elif operation == "sign":
        arguments["digested_data"] = opaque_bytes
    elif operation == "signature_verify":
        arguments["digested_data"] = opaque_bytes
        arguments["signature_data"] = opaque_bytes_2
    return function(**arguments)


def _span_fields(value: object) -> tuple[object, int]:
    """Read a byte-span whether the CFFI boundary passes it by value or pointer."""
    ctype = extensions.ffi.typeof(value)
    span = value[0] if ctype.kind == "pointer" else value
    return span.data, int(span.length)


def _is_span(value: object) -> bool:
    ctype = extensions.ffi.typeof(value)
    if ctype.kind == "pointer":
        ctype = ctype.item
    return ctype.kind == "struct" and ctype.cname.endswith(
        "kmipkit_byte_span_v1_t"
    )


def _span_arguments(arguments: tuple[object, ...]) -> tuple[object, ...]:
    return tuple(argument for argument in arguments if _is_span(argument))


def _span_bytes(value: object) -> bytes:
    pointer, length = _span_fields(value)
    return bytes(extensions.ffi.buffer(pointer, length)) if length else b""


def _data_value(item_type: ttlv.ItemType, limits: ttlv.CodecLimits) -> ttlv.Value:
    if item_type is ttlv.ItemType.ByteString:
        return ttlv.ttlv_value_byte_string(b"operation-data", limits)
    if item_type is ttlv.ItemType.Enumeration:
        return ttlv.ttlv_value_enumeration(0xE102_0304)
    if item_type is ttlv.ItemType.Integer:
        return ttlv.ttlv_value_integer(-37)
    raise AssertionError(f"unsupported test Data type: {item_type}")


def _opaque_client_handle(c_type: str) -> tuple[object, object]:
    storage = client.ffi.new("uint8_t[]", 1)
    return client.ffi.cast(f"{c_type} *", storage), storage


def _initialize_metadata(out: object, operation: int, outcome: int, status: int) -> None:
    out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")  # type: ignore[attr-defined]
    out.abi_version = 2  # type: ignore[attr-defined]
    out.operation = operation  # type: ignore[attr-defined]
    out.outcome_kind = outcome  # type: ignore[attr-defined]
    out.result_status = status  # type: ignore[attr-defined]
    out.has_result_reason = 0  # type: ignore[attr-defined]
    out.result_reason = 0  # type: ignore[attr-defined]
    out.has_result_message = 0  # type: ignore[attr-defined]
    out.reserved = 0  # type: ignore[attr-defined]
    out.result_message_data = client.ffi.NULL  # type: ignore[attr-defined]
    out.result_message_length = 0  # type: ignore[attr-defined]


def _set_output_span(present: object, out_span: object, value: bytes | None, owners: list[object]) -> None:
    if value is None:
        present[0] = 0  # type: ignore[index]
        out_span.data = client.ffi.NULL  # type: ignore[attr-defined]
        out_span.length = 0  # type: ignore[attr-defined]
        return
    present[0] = 1  # type: ignore[index]
    if value:
        owner = client.ffi.new("uint8_t[]", value)
        owners.append(owner)
        out_span.data = owner  # type: ignore[attr-defined]
    else:
        out_span.data = client.ffi.NULL  # type: ignore[attr-defined]
    out_span.length = len(value)  # type: ignore[attr-defined]


def _set_unique_identifier(out: object, text: str, owners: list[object]) -> None:
    raw = text.encode("utf-8")
    owner = client.ffi.new("uint8_t[]", raw)
    owners.append(owner)
    out.struct_size = client.ffi.sizeof("kmipkit_unique_identifier_v1_t")  # type: ignore[attr-defined]
    out.abi_version = 1  # type: ignore[attr-defined]
    out.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
    out.reserved = 0  # type: ignore[attr-defined]
    out.text_data = owner  # type: ignore[attr-defined]
    out.text_length = len(raw)  # type: ignore[attr-defined]
    out.enumeration_value = 0  # type: ignore[attr-defined]
    out.integer_value = 0  # type: ignore[attr-defined]


def _common_result_symbols(
    item_pointer: object,
    *,
    operation: int,
    outcome: int,
    status: int,
    owners: list[object],
) -> dict[str, object]:
    def metadata(_item: object, out: object) -> int:
        _initialize_metadata(out, operation, outcome, status)
        return 0

    def absent_span(_item: object, present: object, out_span: object) -> int:
        _set_output_span(present, out_span, None, owners)
        return 0

    def absent_versions(
        _item: object,
        present: object,
        _versions: object,
        _capacity: int,
        count: object,
    ) -> int:
        present[0] = 0  # type: ignore[index]
        count[0] = 0  # type: ignore[index]
        return 0

    def absent_create(
        _item: object, present: object, object_type: object, _identifier: object
    ) -> int:
        present[0] = 0  # type: ignore[index]
        object_type[0] = 0  # type: ignore[index]
        return 0

    def absent_pair(
        _item: object, present: object, _private: object, _public: object
    ) -> int:
        present[0] = 0  # type: ignore[index]
        return 0

    def absent_split(_item: object, present: object, count: object) -> int:
        present[0] = 0  # type: ignore[index]
        count[0] = 0  # type: ignore[index]
        return 0

    return {
        "kmipkit_client_batch_response_release": lambda _handle: None,
        "kmipkit_client_batch_item_response_release": lambda _handle: None,
        "kmipkit_client_batch_response_item_count": lambda _response, count: count.__setitem__(0, 1) or 0,
        "kmipkit_client_batch_response_item_at": lambda _response, _index, out: out.__setitem__(0, item_pointer) or 0,
        "kmipkit_client_batch_item_response_metadata_v2": metadata,
        "kmipkit_client_batch_item_response_unique_batch_item_id": absent_span,
        "kmipkit_client_batch_item_response_pending_correlation_value": absent_span,
        "kmipkit_client_batch_item_response_discover_versions_supported_versions": absent_versions,
        "kmipkit_client_batch_item_response_create": absent_create,
        "kmipkit_client_batch_item_response_create_key_pair": absent_pair,
        "kmipkit_client_batch_item_response_create_split_key_count": absent_split,
        "kmipkit_client_batch_item_response_create_split_key_unique_identifier_at": lambda *_args: 4,
    }


def _operation_getter_symbols(
    owners: list[object],
    calls: list[str],
) -> dict[str, object]:
    def hash_response(
        _item: object,
        present: object,
        data_present: object,
        data_span: object,
        correlation_present: object,
        correlation_span: object,
    ) -> int:
        calls.append("hash")
        present[0] = 1  # type: ignore[index]
        _set_output_span(data_present, data_span, b"hash-output-marker", owners)
        _set_output_span(correlation_present, correlation_span, b"hash-correlation", owners)
        return 0

    def with_uid(out_identifier: object) -> None:
        _set_unique_identifier(out_identifier, "server-key-marker", owners)

    def mac_response(
        _item: object,
        present: object,
        identifier: object,
        mac_present: object,
        mac_span: object,
        correlation_present: object,
        correlation_span: object,
    ) -> int:
        calls.append("mac")
        present[0] = 1  # type: ignore[index]
        with_uid(identifier)
        _set_output_span(mac_present, mac_span, b"mac-output-marker", owners)
        _set_output_span(correlation_present, correlation_span, b"mac-correlation", owners)
        return 0

    def mac_verify_response(
        _item: object,
        present: object,
        identifier: object,
        validity_present: object,
        validity_raw: object,
        correlation_present: object,
        correlation_span: object,
    ) -> int:
        calls.append("mac_verify")
        present[0] = 1  # type: ignore[index]
        with_uid(identifier)
        validity_present[0] = 1  # type: ignore[index]
        validity_raw[0] = 0xE102_0304  # type: ignore[index]
        _set_output_span(correlation_present, correlation_span, b"verify-correlation", owners)
        return 0

    def sign_response(
        _item: object,
        present: object,
        identifier: object,
        signature_present: object,
        signature_span: object,
        correlation_present: object,
        correlation_span: object,
    ) -> int:
        calls.append("sign")
        present[0] = 1  # type: ignore[index]
        with_uid(identifier)
        _set_output_span(signature_present, signature_span, b"signature-output-marker", owners)
        _set_output_span(correlation_present, correlation_span, b"sign-correlation", owners)
        return 0

    def signature_verify_response(
        _item: object,
        present: object,
        identifier: object,
        validity_present: object,
        validity_raw: object,
        recovered_present: object,
        recovered_span: object,
        correlation_present: object,
        correlation_span: object,
    ) -> int:
        calls.append("signature_verify")
        present[0] = 1  # type: ignore[index]
        with_uid(identifier)
        validity_present[0] = 1  # type: ignore[index]
        validity_raw[0] = 0xF123_4567  # type: ignore[index]
        _set_output_span(recovered_present, recovered_span, b"recovered-output-marker", owners)
        _set_output_span(correlation_present, correlation_span, b"signature-verify-correlation", owners)
        return 0

    return {
        "kmipkit_client_batch_item_response_hash": hash_response,
        "kmipkit_client_batch_item_response_mac": mac_response,
        "kmipkit_client_batch_item_response_mac_verify": mac_verify_response,
        "kmipkit_client_batch_item_response_sign": sign_response,
        "kmipkit_client_batch_item_response_signature_verify": signature_verify_response,
    }


def test_python_facade_declares_all_typed_operation_response_slots() -> None:
    missing_operations = tuple(
        name
        for name in _OPERATIONS
        if not callable(getattr(extensions, f"client_batch_item_{name}", None))
    )
    assert missing_operations == ()

    result_fields = {item.name for item in fields(client.BatchItemResult)}
    assert {
        "hash_response",
        "mac_response",
        "mac_verify_response",
        "sign_response",
        "signature_verify_response",
    } <= result_fields


@pytest.mark.parametrize(
    "item_type",
    (ttlv.ItemType.ByteString, ttlv.ItemType.Enumeration, ttlv.ItemType.Integer),
)
def test_all_operation_builders_forward_each_allowed_data_encoding(
    monkeypatch: pytest.MonkeyPatch, item_type: ttlv.ItemType
) -> None:
    calls = _capture_constructor(monkeypatch)
    limits = ttlv.codec_limits_defaults()
    parameters = ttlv.ttlv_structure_create()
    data = _data_value(item_type, limits)
    view = ttlv.ttlv_value_view(data)
    try:
        assert ttlv.ttlv_value_view_type(view) is item_type
        for operation in _OPERATIONS:
            assert callable(getattr(extensions, f"client_batch_item_{operation}", None))

        data_positions = {"hash": 1, "mac": 2, "mac_verify": 2, "sign": 2, "signature_verify": 2}
        parameter_positions = {"hash": 0, "mac": 1, "mac_verify": 1, "sign": 1, "signature_verify": 1}
        expected_spans = {
            "hash": (b"correlation-marker",),
            "mac": (b"correlation-marker",),
            "mac_verify": (b"opaque-operation-input", b"correlation-marker"),
            "sign": (b"opaque-operation-input", b"correlation-marker"),
            "signature_verify": (
                b"opaque-operation-input",
                b"opaque-signature-input",
                b"correlation-marker",
            ),
        }
        # Build requests with distinct byte markers so every operation-specific
        # span can be checked at its own C ABI position.
        calls.clear()
        for operation in _OPERATIONS:
            _call_operation(
                operation,
                parameters=parameters,
                data=data,
                correlation_value=b"correlation-marker",
                opaque_bytes=b"opaque-operation-input",
                opaque_bytes_2=b"opaque-signature-input",
            )
        assert tuple(name for name, _arguments in calls) == tuple(
            f"kmipkit_client_batch_item_{operation}" for operation in _OPERATIONS
        )
        for operation, (_name, arguments) in zip(_OPERATIONS, calls, strict=True):
            assert arguments[data_positions[operation]] == data._pointer()
            assert arguments[parameter_positions[operation]] == parameters._pointer()
            assert tuple(map(_span_bytes, _span_arguments(arguments))) == expected_spans[operation]
            if operation != "hash":
                assert arguments[0] == extensions.ffi.NULL
    finally:
        view.close()
        data.close()
        parameters.close()
        limits.close()


def test_request_builders_preserve_optional_octet_span_presence_and_bool_codes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls = _capture_constructor(monkeypatch)
    parameters = ttlv.ttlv_structure_create()
    limits = ttlv.codec_limits_defaults()
    data = ttlv.ttlv_value_byte_string(b"", limits)
    try:
        for operation in _OPERATIONS:
            _call_operation(
                operation,
                parameters=parameters,
                data=data,
                correlation_value=b"",
                opaque_bytes=b"",
                opaque_bytes_2=b"",
                init_indicator=False,
                final_indicator=True,
            )

        assert len(calls) == len(_OPERATIONS)
        for operation, (_name, arguments) in zip(_OPERATIONS, calls, strict=True):
            assert _name == f"kmipkit_client_batch_item_{operation}"
            # The C ABI uses a non-null span pointer for a present empty value;
            # an omitted value is represented by a null span pointer.
            spans = [_span_fields(argument) for argument in _span_arguments(arguments)]
            assert spans
            assert all(pointer != extensions.ffi.NULL and length == 0 for pointer, length in spans)
            assert arguments[-2:] == (1, 2)
            data_position = 1 if operation == "hash" else 2
            assert arguments[data_position] == data._pointer()
    finally:
        data.close()
        parameters.close()
        limits.close()


def test_omitted_optional_octet_spans_remain_null(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    # The byte-span C ABI distinguishes None from b"" even though both have a
    # zero byte length.
    missing = tuple(
        name
        for name in _OPERATIONS
        if not callable(getattr(extensions, f"client_batch_item_{name}", None))
    )
    assert missing == ()
    calls = _capture_constructor(monkeypatch)
    parameters = ttlv.ttlv_structure_create()
    try:
        for operation in _OPERATIONS:
            _call_operation(
                operation,
                parameters=parameters,
                data=None,
                init_indicator=True,
                final_indicator=False,
            )

        assert len(calls) == len(_OPERATIONS)
        data_positions = {"hash": 1, "mac": 2, "mac_verify": 2, "sign": 2, "signature_verify": 2}
        for operation, (_name, arguments) in zip(_OPERATIONS, calls, strict=True):
            spans = [_span_fields(argument) for argument in _span_arguments(arguments)]
            assert spans
            assert all(pointer == extensions.ffi.NULL and length == 0 for pointer, length in spans)
            assert arguments[data_positions[operation]] == extensions.ffi.NULL
            assert arguments[-2:] == (2, 1)
    finally:
        parameters.close()


def test_keyed_operation_builder_preserves_unique_identifier_wire_type(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls = _capture_constructor(monkeypatch)
    parameters = ttlv.ttlv_structure_create()
    limits = ttlv.codec_limits_defaults()
    data = ttlv.ttlv_value_byte_string(b"signed-data", limits)
    identifier = extensions.UniqueIdentifier.integer(-7)
    try:
        function = getattr(extensions, "client_batch_item_mac", None)
        assert callable(function)
        function(
            unique_identifier=identifier,
            parameters=parameters,
            data=data,
        )

        assert len(calls) == 1
        name, arguments = calls[0]
        assert name == "kmipkit_client_batch_item_mac"
        raw_identifier = arguments[0]
        if extensions.ffi.typeof(raw_identifier).kind == "pointer":
            raw_identifier = raw_identifier[0]
        assert raw_identifier.kind == int(extensions.UniqueIdentifierKind.INTEGER)
        assert raw_identifier.integer_value == -7
        assert raw_identifier.text_data == extensions.ffi.NULL
        assert arguments[1] == parameters._pointer()
        assert arguments[2] == data._pointer()
    finally:
        data.close()
        parameters.close()
        limits.close()


@pytest.mark.parametrize(
    ("operation", "operation_code"),
    (
        ("hash", 0x27),
        ("mac", 0x23),
        ("mac_verify", 0x24),
        ("sign", 0x21),
        ("signature_verify", 0x22),
    ),
)
def test_successful_batch_item_maps_only_its_typed_operation_response(
    monkeypatch: pytest.MonkeyPatch, operation: str, operation_code: int
) -> None:
    response_pointer, _response_storage = _opaque_client_handle(
        "kmipkit_client_batch_response_t"
    )
    item_pointer, _item_storage = _opaque_client_handle(
        "kmipkit_client_batch_item_response_t"
    )
    owners: list[object] = []
    getter_calls: list[str] = []
    fake = _common_result_symbols(
        item_pointer,
        operation=operation_code,
        outcome=int(client.BatchOutcome.COMPLETED),
        status=0,
        owners=owners,
    )
    fake.update(_operation_getter_symbols(owners, getter_calls))
    monkeypatch.setattr(client, "lib", SimpleNamespace(**fake))

    response = client._batch_result(response_pointer)
    item = response.items[0]
    assert getter_calls == [operation]
    response_value = getattr(item, f"{operation}_response")
    response_types = {
        "hash": client.HashResponse,
        "mac": client.MacResponse,
        "mac_verify": client.MacVerifyResponse,
        "sign": client.SignResponse,
        "signature_verify": client.SignatureVerifyResponse,
    }
    assert isinstance(response_value, response_types[operation])
    for other in _OPERATIONS:
        if other != operation:
            assert getattr(item, f"{other}_response") is None

    if operation == "hash":
        assert item.hash_response.data == b"hash-output-marker"
        assert item.hash_response.correlation_value == b"hash-correlation"
    elif operation == "mac":
        assert item.mac_response.unique_identifier == extensions.UniqueIdentifier.text_string(
            "server-key-marker"
        )
        assert item.mac_response.mac_data == b"mac-output-marker"
        assert item.mac_response.correlation_value == b"mac-correlation"
    elif operation == "mac_verify":
        assert item.mac_verify_response.validity_indicator_raw == 0xE102_0304
        assert item.mac_verify_response.correlation_value == b"verify-correlation"
    elif operation == "sign":
        assert item.sign_response.signature_data == b"signature-output-marker"
        assert item.sign_response.correlation_value == b"sign-correlation"
    else:
        assert item.signature_verify_response.validity_indicator_raw == 0xF123_4567
        assert item.signature_verify_response.recovered_data == b"recovered-output-marker"
        assert item.signature_verify_response.correlation_value == (
            b"signature-verify-correlation"
        )

    rendered = repr(item)
    assert "output-marker" not in repr(response_value)
    for marker in (
        "hash-output-marker",
        "mac-output-marker",
        "signature-output-marker",
        "recovered-output-marker",
        "server-key-marker",
        "hash-correlation",
        "signature-verify-correlation",
    ):
        assert marker not in rendered
    response.close()


@pytest.mark.parametrize(
    ("outcome", "status"),
    ((client.BatchOutcome.PENDING, 0), (client.BatchOutcome.COMPLETED, 1)),
)
def test_pending_or_failed_batch_item_never_calls_success_payload_getters(
    monkeypatch: pytest.MonkeyPatch,
    outcome: client.BatchOutcome,
    status: int,
) -> None:
    response_pointer, _response_storage = _opaque_client_handle(
        "kmipkit_client_batch_response_t"
    )
    item_pointer, _item_storage = _opaque_client_handle(
        "kmipkit_client_batch_item_response_t"
    )
    owners: list[object] = []
    getter_calls: list[str] = []
    fake = _common_result_symbols(
        item_pointer,
        operation=0x27,
        outcome=int(outcome),
        status=status,
        owners=owners,
    )
    fake.update(_operation_getter_symbols(owners, getter_calls))
    monkeypatch.setattr(client, "lib", SimpleNamespace(**fake))

    response = client._batch_result(response_pointer)
    item = response.items[0]

    assert getter_calls == []
    result_field_names = {field.name for field in fields(client.BatchItemResult)}
    assert {
        "hash_response",
        "mac_response",
        "mac_verify_response",
        "sign_response",
        "signature_verify_response",
    } <= result_field_names
    for operation in _OPERATIONS:
        assert getattr(item, f"{operation}_response") is None
    response.close()


