"""Python facade contract tests for the KMIP 2.1 Encrypt and Decrypt mappings."""

from __future__ import annotations

from dataclasses import fields
from types import SimpleNamespace
from typing import Any

import pytest
from kmipkit import client, errors, extensions, ttlv


def _capture_constructor(
    monkeypatch: pytest.MonkeyPatch,
) -> list[tuple[str, tuple[object, ...]]]:
    calls: list[tuple[str, tuple[object, ...]]] = []
    span_owners: list[object] = []

    def make_handle(
        _c_type: str,
        function_name: str,
        arguments: tuple[object, ...],
        *_args: Any,
        **_kwargs: Any,
    ) -> object:
        copied: list[object] = []
        for argument in arguments:
            try:
                ctype = extensions.ffi.typeof(argument)
            except TypeError:
                copied.append(argument)
                continue
            span_type = ctype.item if ctype.kind == "pointer" else ctype
            if span_type.kind != "struct" or not span_type.cname.endswith(
                "kmipkit_byte_span_v1_t"
            ):
                copied.append(argument)
                continue
            span = argument[0] if ctype.kind == "pointer" else argument
            span_type = extensions.ffi.typeof(span)
            length = int(span.length)
            raw = bytes(extensions.ffi.buffer(span.data, length)) if length else b""
            owner = extensions.ffi.new("uint8_t[]", raw) if raw else extensions.ffi.NULL
            snapshot = extensions.ffi.new(
                "kmipkit_byte_span_v1_t *", {"data": owner, "length": length}
            )
            span_owners.extend((owner, snapshot))
            copied.append(snapshot)
        calls.append((function_name, tuple(copied)))
        return object()

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _type, value: value)
    return calls


def _span_bytes(value: object) -> bytes:
    ctype = extensions.ffi.typeof(value)
    span = value[0] if ctype.kind == "pointer" else value
    return bytes(extensions.ffi.buffer(span.data, int(span.length))) if span.length else b""


@pytest.mark.parametrize(
    ("operation", "optional_bytes", "expected_spans"),
    (
        (
            "encrypt",
            {"authenticated_encryption_additional_data": b"aad-marker"},
            (b"iv-marker", b"correlation-marker", b"aad-marker"),
        ),
        (
            "decrypt",
            {
                "authenticated_encryption_additional_data": b"aad-marker",
                "authenticated_encryption_tag": b"tag-marker",
            },
            (b"iv-marker", b"tag-marker", b"correlation-marker", b"aad-marker"),
        ),
    ),
)
def test_crypto_operation_builders_preserve_typed_fields_and_optional_bytes(
    monkeypatch: pytest.MonkeyPatch,
    operation: str,
    optional_bytes: dict[str, bytes],
    expected_spans: tuple[bytes, ...],
) -> None:
    calls = _capture_constructor(monkeypatch)
    parameters = ttlv.ttlv_structure_create()
    limits = ttlv.codec_limits_defaults()
    data = ttlv.ttlv_value_byte_string(b"operation-data", limits)
    identifier = extensions.UniqueIdentifier.integer(-7)
    try:
        function = getattr(extensions, f"client_batch_item_{operation}", None)
        assert callable(function), f"missing Python operation builder: {operation}"
        function(
            unique_identifier=identifier,
            parameters=parameters,
            data=data,
            iv_counter_nonce=b"iv-marker",
            correlation_value=b"correlation-marker",
            init_indicator=False,
            final_indicator=True,
            **optional_bytes,
        )

        assert len(calls) == 1
        name, arguments = calls[0]
        assert name == f"kmipkit_client_batch_item_{operation}"
        raw_identifier = arguments[0]
        if extensions.ffi.typeof(raw_identifier).kind == "pointer":
            raw_identifier = raw_identifier[0]
        assert raw_identifier.kind == int(extensions.UniqueIdentifierKind.INTEGER)
        assert raw_identifier.integer_value == -7
        assert arguments[1] == parameters._pointer()
        assert arguments[2] == data._pointer()
        span_positions = (3, 4, 7) if operation == "encrypt" else (3, 4, 5, 8)
        assert tuple(_span_bytes(arguments[index]) for index in span_positions) == expected_spans
        indicator_positions = (5, 6) if operation == "encrypt" else (6, 7)
        assert tuple(arguments[index] for index in indicator_positions) == (1, 2)
    finally:
        data.close()
        parameters.close()
        limits.close()


def test_encrypt_decrypt_builders_preserve_absent_and_present_empty_octets(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls = _capture_constructor(monkeypatch)
    parameters = ttlv.ttlv_structure_create()
    try:
        for operation, optional_bytes in (
            (
                "encrypt",
                {"authenticated_encryption_additional_data": b""},
            ),
            (
                "decrypt",
                {
                    "authenticated_encryption_additional_data": None,
                    "authenticated_encryption_tag": b"",
                },
            ),
        ):
            function = getattr(extensions, f"client_batch_item_{operation}", None)
            assert callable(function), f"missing Python operation builder: {operation}"
            function(
                parameters=parameters,
                data=None,
                iv_counter_nonce=None,
                correlation_value=None,
                init_indicator=None,
                final_indicator=None,
                **optional_bytes,
            )

        assert len(calls) == 2
        encrypt_name, encrypt_args = calls[0]
        decrypt_name, decrypt_args = calls[1]
        assert encrypt_name == "kmipkit_client_batch_item_encrypt"
        assert decrypt_name == "kmipkit_client_batch_item_decrypt"
        assert encrypt_args[0] == extensions.ffi.NULL
        assert decrypt_args[0] == extensions.ffi.NULL
        assert encrypt_args[1] == parameters._pointer()
        assert decrypt_args[1] == parameters._pointer()
        assert encrypt_args[2] == extensions.ffi.NULL
        assert decrypt_args[2] == extensions.ffi.NULL
        assert encrypt_args[3] == extensions.ffi.NULL
        assert decrypt_args[3] == extensions.ffi.NULL
        assert encrypt_args[4] == extensions.ffi.NULL
        assert decrypt_args[5] == extensions.ffi.NULL
        assert encrypt_args[5:7] == (0, 0)
        assert decrypt_args[6:8] == (0, 0)
        assert encrypt_args[7] != extensions.ffi.NULL
        assert _span_bytes(encrypt_args[7]) == b""
        assert decrypt_args[4] != extensions.ffi.NULL
        assert _span_bytes(decrypt_args[4]) == b""
        assert decrypt_args[8] == extensions.ffi.NULL
    finally:
        parameters.close()


@pytest.mark.parametrize(
    ("operation", "symbol", "expected"),
    (
        (
            "encrypt",
            "kmipkit_client_batch_item_response_encrypt",
            (b"ciphertext-marker", b"iv-marker", b"correlation-marker", b"tag-marker"),
        ),
        (
            "decrypt",
            "kmipkit_client_batch_item_response_decrypt",
            (b"plaintext-marker", b"correlation-marker"),
        ),
    ),
)
def test_typed_encrypt_and_decrypt_success_responses_copy_and_redact_outputs(
    monkeypatch: pytest.MonkeyPatch,
    operation: str,
    symbol: str,
    expected: tuple[bytes, ...],
) -> None:
    parser = getattr(client, f"_{operation}_response", None)
    assert callable(parser), f"missing Python typed response parser: {operation}"
    assert {"encrypt_response", "decrypt_response"} <= {
        item.name for item in fields(client.BatchItemResult)
    }
    owners: list[object] = []

    def get_response(_item: object, *outputs: object) -> int:
        outputs[0][0] = 1  # type: ignore[index]
        identifier = outputs[1]
        identifier.struct_size = extensions.ffi.sizeof("kmipkit_unique_identifier_v1_t")
        identifier.abi_version = 1
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        identifier_text = extensions.ffi.new("unsigned char[]", b"response-key-secret")
        identifier.text_data = identifier_text
        identifier.text_length = len(b"response-key-secret")
        owners.append(identifier_text)

        spans = expected
        span_start = 2
        for index, raw in enumerate(spans):
            present = outputs[span_start + index * 2]
            span = outputs[span_start + index * 2 + 1]
            storage = extensions.ffi.new("unsigned char[]", raw)
            owners.append(storage)
            present[0] = 1  # type: ignore[index]
            span.data = storage
            span.length = len(raw)
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(**{symbol: get_response}))
    operation_code = 0x1F if operation == "encrypt" else 0x20
    mapped = client._cryptographic_responses(
        SimpleNamespace(_pointer=lambda: object()), operation_code
    )
    selected = 0 if operation == "encrypt" else 1
    assert all(value is None for index, value in enumerate(mapped) if index != selected)
    response = mapped[selected]

    assert response is not None
    assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
        "response-key-secret"
    )
    actual = (
        (response.data, response.iv_counter_nonce, response.correlation_value,
         response.authenticated_encryption_tag)
        if operation == "encrypt"
        else (response.data, response.correlation_value)
    )
    assert actual == expected
    assert "response-key-secret" not in repr(response)
    assert all(raw.decode() not in repr(response) for raw in expected)

    owned_names = (
        ("_data", "_iv_counter_nonce", "_correlation_value", "_authenticated_encryption_tag")
        if operation == "encrypt"
        else ("_data", "_correlation_value")
    )
    owned = [getattr(response, name) for name in owned_names]
    assert all(value is None or isinstance(value, client._SecretBytes) for value in owned)
    buffers = [value.buffer for value in owned if value is not None]
    response.close()
    response.close()
    assert all(not any(buffer) for buffer in buffers)
    public_fields = (
        "unique_identifier", "data", "iv_counter_nonce", "correlation_value",
        "authenticated_encryption_tag",
    )
    for name in public_fields:
        if hasattr(type(response), name):
            with pytest.raises(RuntimeError, match="response is closed"):
                getattr(response, name)


@pytest.mark.parametrize(
    ("operation", "scenario"),
    (
        ("encrypt", "absent"),
        ("encrypt", "absent-with-data"),
        ("encrypt", "invalid-presence"),
        ("encrypt", "present-without-identifier"),
        ("decrypt", "absent"),
        ("decrypt", "absent-with-data"),
        ("decrypt", "invalid-presence"),
        ("decrypt", "present-without-identifier"),
    ),
)
def test_encrypt_and_decrypt_response_parsers_validate_optional_shapes(
    monkeypatch: pytest.MonkeyPatch, operation: str, scenario: str
) -> None:
    def get_response(_item: object, *outputs: object) -> int:
        if scenario == "absent":
            outputs[0][0] = 0  # type: ignore[index]
        elif scenario == "absent-with-data":
            outputs[0][0] = 0  # type: ignore[index]
            outputs[2][0] = 1  # type: ignore[index]
        elif scenario == "invalid-presence":
            outputs[0][0] = 2  # type: ignore[index]
        else:
            outputs[0][0] = 1  # type: ignore[index]
        return 0

    symbol = f"kmipkit_client_batch_item_response_{operation}"
    monkeypatch.setattr(client, "lib", SimpleNamespace(**{symbol: get_response}))
    parser = getattr(client, f"_{operation}_response")
    item = SimpleNamespace(_pointer=lambda: object())

    if scenario == "absent":
        assert parser(item) is None
    else:
        with pytest.raises(errors.InvalidInputError):
            parser(item)


@pytest.mark.parametrize(
    ("malformation", "error_type"),
    (
        ("absent-with-data", errors.InvalidInputError),
        ("invalid-presence", errors.InvalidInputError),
        ("too-large", errors.ResourceLimitError),
        ("null-nonempty", errors.InvalidInputError),
    ),
)
def test_optional_output_secret_span_rejects_malformed_native_shapes(
    malformation: str, error_type: type[Exception]
) -> None:
    present = extensions.ffi.new("uint8_t *")
    span = extensions.ffi.new("kmipkit_byte_span_v1_t *")
    storage: object | None = None

    if malformation == "absent-with-data":
        present[0] = 0
        storage = extensions.ffi.new("uint8_t[]", b"x")
        span.data = storage
        span.length = 1
    elif malformation == "invalid-presence":
        present[0] = 2
    else:
        present[0] = 1
        span.length = client.sys.maxsize + 1 if malformation == "too-large" else 1

    with pytest.raises(error_type):
        client._optional_output_secret_span(present, span)


def test_optional_output_span_rejects_length_above_platform_limit() -> None:
    present = client.ffi.new("uint8_t *")
    span = client.ffi.new("kmipkit_byte_span_v1_t *")
    present[0] = 1
    span.length = client.sys.maxsize + 1

    with pytest.raises(errors.ResourceLimitError):
        client._optional_output_span(present, span)


def test_encrypt_response_copies_caller_bytes_into_wipeable_storage() -> None:
    response = client.EncryptResponse(
        extensions.UniqueIdentifier.text_string("response-id"),
        b"caller-owned-secret",
        None,
        None,
        None,
    )
    owned = response._data

    assert isinstance(owned, client._SecretBytes)
    assert response.data == b"caller-owned-secret"
    response.close()
    assert owned is not None
    assert not any(owned.buffer)
