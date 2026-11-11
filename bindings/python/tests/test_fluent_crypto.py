"""Consumer contracts for operation-specific Python crypto builders."""

from __future__ import annotations

import struct
from collections.abc import Callable
from pathlib import Path
from typing import Any

import pytest
from _test_support import (
    mtls_transport_config,
    read_ttlv_frame,
    start_tls13_mtls_peer,
    ttlv_enumeration,
    ttlv_item,
    ttlv_structure,
)
from kmipkit import (
    client,
    cryptographic_parameters,
    errors,
    extensions,
    fluent,
    protocol_values,
    ttlv,
)

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


def _aes_parameters() -> cryptographic_parameters.CryptographicParameters:
    builder = cryptographic_parameters.CryptographicParameters.builder()
    try:
        return builder.cryptographic_algorithm(
            protocol_values.CryptographicAlgorithm.AES
        ).build()
    finally:
        builder.close()


def _hash_parameters() -> cryptographic_parameters.CryptographicParameters:
    builder = cryptographic_parameters.CryptographicParameters.builder()
    try:
        return builder.hashing_algorithm(protocol_values.HashingAlgorithm.SHA_256).build()
    finally:
        builder.close()


def _items(encoded_structure_value: bytes) -> list[tuple[int, int, bytes]]:
    items: list[tuple[int, int, bytes]] = []
    offset = 0
    while offset < len(encoded_structure_value):
        tag = int.from_bytes(encoded_structure_value[offset : offset + 3], "big")
        item_type = encoded_structure_value[offset + 3]
        length = int.from_bytes(encoded_structure_value[offset + 4 : offset + 8], "big")
        value_start = offset + 8
        value_end = value_start + length
        padded_end = value_end + (-length % 8)
        assert padded_end <= len(encoded_structure_value)
        items.append((tag, item_type, encoded_structure_value[value_start:value_end]))
        offset = padded_end
    assert offset == len(encoded_structure_value)
    return items


def _find_item(encoded_structure_value: bytes, wanted_tag: int) -> tuple[int, int, bytes] | None:
    for tag, item_type, value in _items(encoded_structure_value):
        if tag == wanted_tag:
            return tag, item_type, value
        if item_type == 1:
            found = _find_item(value, wanted_tag)
            if found is not None:
                return found
    return None


def _crypto_failure_response(operation: int) -> bytes:
    version = ttlv_structure(
        0x420069,
        ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = ttlv_structure(
        0x42007A,
        version,
        ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    response_item = ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, operation),
        ttlv_enumeration(0x42007F, 1),
        ttlv_enumeration(0x42007E, 1),
    )
    return ttlv_structure(0x42007B, header, response_item)


def test_request_namespace_exposes_seven_operation_specific_crypto_builders() -> None:
    expected = {
        "encrypt": "EncryptBuilder",
        "decrypt": "DecryptBuilder",
        "sign": "SignBuilder",
        "signature_verify": "SignatureVerifyBuilder",
        "mac": "MacBuilder",
        "mac_verify": "MacVerifyBuilder",
        "hash": "HashBuilder",
    }

    for factory_name, builder_name in expected.items():
        assert callable(getattr(fluent.Request, factory_name, None))
        assert builder_name in fluent.__all__
        assert getattr(fluent, builder_name, None) is not None


def test_crypto_builders_expose_only_the_fields_of_their_typed_operation() -> None:
    common = {
        "cryptographic_parameters",
        "cryptographic_parameters_structure",
        "data",
        "data_value",
        "correlation_value",
        "init_indicator",
        "final_indicator",
        "with_options",
        "asynchronous_indicator",
        "build",
        "validate",
        "to_batch_item",
        "execute",
        "close",
    }
    operation_fields = {
        "EncryptBuilder": {"unique_identifier", "iv_counter_nonce", "authenticated_encryption_additional_data"},
        "DecryptBuilder": {"unique_identifier", "iv_counter_nonce", "authenticated_encryption_additional_data", "authenticated_encryption_tag"},
        "SignBuilder": {"unique_identifier", "digested_data"},
        "SignatureVerifyBuilder": {"unique_identifier", "digested_data", "signature_data"},
        "MacBuilder": {"unique_identifier"},
        "MacVerifyBuilder": {"unique_identifier", "mac_data"},
        "HashBuilder": set(),
    }

    for builder_name, specific_fields in operation_fields.items():
        builder_type = getattr(fluent, builder_name)
        assert common <= set(dir(builder_type))
        assert specific_fields <= set(dir(builder_type))
        for other_builder, other_fields in operation_fields.items():
            if other_builder == builder_name:
                continue
            forbidden = other_fields - specific_fields
            assert not (forbidden & set(dir(builder_type))), (
                f"{builder_name} exposes fields exclusive to {other_builder}"
            )


def test_crypto_builders_reject_missing_request_data_and_use_after_close() -> None:
    builder_names = (
        "EncryptBuilder",
        "DecryptBuilder",
        "SignBuilder",
        "SignatureVerifyBuilder",
        "MacBuilder",
        "MacVerifyBuilder",
        "HashBuilder",
    )

    for builder_name in builder_names:
        builder = getattr(fluent, builder_name)()
        try:
            with pytest.raises(errors.KmipKitError):
                builder.build()
        finally:
            builder.close()
        with pytest.raises(RuntimeError, match="closed"):
            builder.build()


def test_crypto_builder_asynchronous_indicator_requires_uint32_not_bool() -> None:
    builder = fluent.EncryptBuilder()
    try:
        for invalid in (-1, 9, 1 << 32, True):
            with pytest.raises(errors.KmipKitError):
                builder.asynchronous_indicator(invalid)
    finally:
        builder.close()


def test_crypto_builder_accepts_typed_parameters_and_explicit_ttlv_data() -> None:
    with ttlv.codec_limits_defaults() as limits:
        parameters = _aes_parameters()
        data = ttlv.ttlv_value_byte_string(b"expert-data", limits)
        try:
            with fluent.EncryptBuilder() as builder:
                builder.unique_identifier(
                    extensions.UniqueIdentifier.text_string("typed-parameters-key")
                )
                builder.cryptographic_parameters(parameters)
                builder.data_value(data)
                item = builder.to_batch_item()
                try:
                    assert item.operation == protocol_values.Operation.ENCRYPT
                finally:
                    item.close()
        finally:
            data.close()
            parameters.close()


def test_crypto_builder_wipes_staging_when_span_allocation_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """KMIPKIT-0105-FR-004: cleanup covers failures before the native setter."""
    original_ffi = fluent.ffi
    stages: list[bytearray] = []
    allocation_error = MemoryError("span allocation failed")

    class FailingSpanAllocator:
        def __getattr__(self, name: str) -> Any:
            return getattr(original_ffi, name)

        def from_buffer(self, c_type: str, staged: bytearray) -> Any:
            stages.append(staged)
            return original_ffi.from_buffer(c_type, staged)

        def new(self, c_type: str) -> Any:
            if c_type == "kmipkit_byte_span_v1_t *":
                raise allocation_error
            return original_ffi.new(c_type)

    caller_buffer = bytearray(b"caller-owned-secret")
    with fluent.EncryptBuilder() as builder:
        monkeypatch.setattr(fluent, "ffi", FailingSpanAllocator())
        with pytest.raises(MemoryError) as failure:
            builder.data(caller_buffer)

    assert failure.value is allocation_error
    assert caller_buffer == b"caller-owned-secret"
    assert len(stages) == 1
    assert stages[0] == bytearray(len(caller_buffer))


@pytest.mark.parametrize(
    ("setter", "invalid"),
    (
        ("cryptographic_parameters", object()),
        ("cryptographic_parameters_structure", object()),
        ("data_value", object()),
        ("init_indicator", 1),
        ("final_indicator", 0),
        ("with_options", object()),
    ),
)
def test_crypto_builder_rejects_wrong_setter_types_locally(
    setter: str, invalid: object,
) -> None:
    """KMIPKIT-0105-FR-002/003/005: invalid Python inputs fail before execution."""
    with fluent.EncryptBuilder() as builder:
        with pytest.raises(errors.InvalidInputError):
            getattr(builder, setter)(invalid)


@pytest.mark.parametrize("invalid", ("secret", None, 1))
def test_crypto_builder_rejects_non_buffer_data(invalid: object) -> None:
    with fluent.EncryptBuilder() as builder:
        with pytest.raises(errors.InvalidInputError):
            builder.data(invalid)  # type: ignore[arg-type]


def test_crypto_builder_rejects_released_memoryview() -> None:
    released = memoryview(b"secret")
    released.release()

    with fluent.EncryptBuilder() as builder:
        with pytest.raises(errors.InvalidInputError):
            builder.data(released)


def test_crypto_builder_rejects_non_contiguous_memoryview() -> None:
    caller_buffer = bytearray(b"caller-owned-secret")
    with memoryview(caller_buffer)[::2] as view, fluent.EncryptBuilder() as builder:
        with pytest.raises(errors.InvalidInputError):
            builder.data(view)

    assert caller_buffer == b"caller-owned-secret"


def test_crypto_builder_rejects_oversized_data_before_copying() -> None:
    with fluent.EncryptBuilder() as builder:
        with pytest.raises(errors.ResourceLimitError):
            builder.data(bytes(16 * 1024 * 1024 + 1))


def test_crypto_builder_validation_preserves_reusable_builder() -> None:
    """KMIPKIT-0105-FR-005: local validation does not consume the request."""
    with fluent.HashBuilder() as builder, _hash_parameters() as parameters:
        builder.cryptographic_parameters(parameters).data(b"hash input")

        builder.validate()

        with builder.to_batch_item() as item:
            assert item.operation == protocol_values.Operation.HASH


def test_crypto_builder_execute_requires_a_client() -> None:
    with fluent.EncryptBuilder() as builder, ttlv.codec_limits_defaults() as limits:
        with pytest.raises(errors.InvalidInputError):
            builder.execute(limits)


def test_crypto_builder_context_closes_on_exception() -> None:
    builder = fluent.EncryptBuilder()
    with pytest.raises(ValueError, match="consumer failure"):
        with builder:
            raise ValueError("consumer failure")

    with pytest.raises(RuntimeError, match="closed"):
        builder.data(b"secret")
    with pytest.raises(RuntimeError, match="closed"):
        with builder:
            pytest.fail("closed builder must reject context entry")


@pytest.mark.parametrize(
    ("factory_name", "operation", "extra_fields", "expected_wire_fields"),
    (
        (
            "encrypt",
            protocol_values.Operation.ENCRYPT,
            lambda builder: builder.iv_counter_nonce(b"nonce")
            .authenticated_encryption_additional_data(b"aad"),
            ((0x42003D, b"nonce"), (0x4200FE, b"aad")),
        ),
        (
            "decrypt",
            protocol_values.Operation.DECRYPT,
            lambda builder: builder.iv_counter_nonce(b"nonce")
            .authenticated_encryption_additional_data(b"aad")
            .authenticated_encryption_tag(b"tag"),
            ((0x42003D, b"nonce"), (0x4200FE, b"aad"), (0x4200FF, b"tag")),
        ),
        (
            "sign",
            protocol_values.Operation.SIGN,
            lambda builder: builder.digested_data(b"digest"),
            ((0x420107, b"digest"),),
        ),
        (
            "signature_verify",
            protocol_values.Operation.SIGNATURE_VERIFY,
            lambda builder: builder.digested_data(b"digest").signature_data(b"signature"),
            ((0x420107, b"digest"), (0x4200C3, b"signature")),
        ),
        ("mac", protocol_values.Operation.MAC, lambda builder: builder, ()),
        (
            "mac_verify",
            protocol_values.Operation.MAC_VERIFY,
            lambda builder: builder.mac_data(b"mac"),
            ((0x4200C6, b"mac"),),
        ),
        ("hash", protocol_values.Operation.HASH, lambda builder: builder, ()),
    ),
)
def test_crypto_builder_executes_one_mtls_exchange_and_copies_input(
    factory_name: str,
    operation: int,
    extra_fields: Callable[[Any], Any],
    expected_wire_fields: tuple[tuple[int, bytes], ...],
) -> None:
    assert callable(getattr(fluent.Request, factory_name, None))
    received: list[bytes] = []

    def handler(stream: object) -> None:
        request = read_ttlv_frame(stream)  # type: ignore[arg-type]
        received.append(request)
        stream.sendall(_crypto_failure_response(operation))  # type: ignore[attr-defined]

    port, peer_failures, peer = start_tls13_mtls_peer(_FIXTURES, handler)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result: client.BatchResult | None = None
    caller_buffer = bytearray(b"original-data")
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                request = getattr(kmip_client.request(), factory_name)()
                with request as builder:
                    parameters = None
                    parameters_structure = None
                    try:
                        builder.with_options(
                            client.RequestOptions(
                                total_timeout=client.TimeoutLimit.bounded(8)
                            )
                        )
                        builder.asynchronous_indicator(1)
                        if factory_name in {"encrypt", "decrypt"}:
                            builder.init_indicator(True).final_indicator(True)
                        else:
                            builder.correlation_value(b"correlation")
                            builder.init_indicator(False).final_indicator(False)
                        builder.data(caller_buffer)
                        if factory_name in {"encrypt", "decrypt", "hash"}:
                            parameters = (
                                _hash_parameters()
                                if factory_name == "hash"
                                else _aes_parameters()
                            )
                            if factory_name == "encrypt":
                                builder.cryptographic_parameters(parameters)
                            elif factory_name == "decrypt":
                                parameters_structure = parameters.into_ttlv()
                                builder.cryptographic_parameters_structure(parameters_structure)
                            else:
                                builder.cryptographic_parameters(parameters)
                        if factory_name != "hash":
                            builder.unique_identifier(
                                extensions.UniqueIdentifier.text_string("crypto-key")
                            )
                        extra_fields(builder)
                        caller_buffer[:] = b"modified-data"
                        result = builder.execute(limits)
                    finally:
                        if parameters_structure is not None:
                            parameters_structure.close()
                        if parameters is not None:
                            parameters.close()
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert len(received) == 1
        request = received[0]
        root_items = _items(request[8:])
        assert root_items[0][0] == 0x420077
        operation_item = _find_item(request[8:], 0x42005C)
        assert operation_item is not None
        assert int.from_bytes(operation_item[2], "big") == operation
        data_item = _find_item(request[8:], 0x4200C2)
        assert data_item is not None
        assert data_item[1] == 8
        assert data_item[2] == b"original-data"
        init_indicator = _find_item(request[8:], 0x4200D7)
        final_indicator = _find_item(request[8:], 0x4200D8)
        async_indicator = _find_item(request[8:], 0x420007)
        expected_indicator = struct.pack(">q", int(factory_name in {"encrypt", "decrypt"}))
        assert init_indicator is not None and init_indicator[2] == expected_indicator
        assert final_indicator is not None and final_indicator[2] == expected_indicator
        assert async_indicator is not None
        assert int.from_bytes(async_indicator[2], "big") == 1
        correlation = _find_item(request[8:], 0x4200D6)
        if factory_name in {"encrypt", "decrypt"}:
            assert correlation is None
        else:
            assert correlation is not None and correlation[2] == b"correlation"
        for tag, expected_bytes in expected_wire_fields:
            field = _find_item(request[8:], tag)
            assert field is not None
            assert field[2] == expected_bytes
        if factory_name != "hash":
            identifier = _find_item(request[8:], 0x420094)
            assert identifier is not None
            assert identifier[2] == b"crypto-key"
        assert result is not None
        assert result.items[0].operation == operation
        assert result.items[0].result_status == 1
        if factory_name in {"encrypt", "decrypt", "hash"}:
            assert _find_item(request[8:], 0x42002B) is not None
    finally:
        if result is not None:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()
