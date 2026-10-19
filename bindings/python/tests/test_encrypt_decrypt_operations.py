"""Python facade contract tests for the KMIP 2.1 Encrypt and Decrypt mappings."""

from __future__ import annotations

from typing import Any

import pytest

from kmipkit import extensions, ttlv


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
