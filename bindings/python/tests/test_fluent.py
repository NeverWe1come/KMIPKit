from __future__ import annotations

import struct
import queue
import socket
import ssl
import threading
from pathlib import Path

import pytest

from _test_support import (
    mtls_transport_config,
    read_ttlv_frame,
    require_tls13_mtls_peer,
    start_tls13_mtls_peer,
)
from kmipkit import client, errors, extensions, fluent, ttlv
from kmipkit.typed_attributes import StandardAttribute

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    return tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big") + value + bytes((-len(value)) % 8)


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _parse_item(data: bytes, offset: int, parent_end: int) -> tuple[int, int, int, int]:
    tag = int.from_bytes(data[offset : offset + 3], "big")
    item_type = data[offset + 3]
    length = int.from_bytes(data[offset + 4 : offset + 8], "big")
    value_start = offset + 8
    end = value_start + length + (-length % 8)
    assert end <= parent_end
    return tag, item_type, value_start, end


def _children(data: bytes, parent: tuple[int, int, int, int]) -> list[tuple[int, int, int, int]]:
    _, item_type, value_start, padded_end = parent
    assert item_type == 1
    value_end = value_start + int.from_bytes(data[value_start - 4 : value_start], "big")
    children = []
    offset = value_start
    while offset < value_end:
        child = _parse_item(data, offset, value_end)
        children.append(child)
        offset = child[3]
    assert offset == value_end and padded_end <= len(data)
    return children


def _create_response() -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 1)),
    )
    item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", 1)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, b"fluent-key"),
        ),
    )
    return _structure(0x42007B, header, item)


def _request_header_indicator(request: bytes) -> int:
    message = _parse_item(request, 0, len(request))
    header = next(child for child in _children(request, message) if child[0] == 0x420077)
    indicators = [child for child in _children(request, header) if child[0] == 0x420007]
    assert len(indicators) == 1
    indicator = indicators[0]
    encoded_length = int.from_bytes(request[indicator[2] - 4 : indicator[2]], "big")
    assert indicator[1] == 5 and encoded_length == 4
    return int.from_bytes(request[indicator[2] : indicator[2] + 4], "big")


def _request_operation_and_identifier(request: bytes) -> tuple[int, bytes | None]:
    root = _parse_item(request, 0, len(request))
    batch_item = next(child for child in _children(request, root) if child[0] == 0x42000F)
    fields = _children(request, batch_item)
    operation = next(child for child in fields if child[0] == 0x42005C)
    operation_raw = int.from_bytes(request[operation[2] : operation[2] + 4], "big")
    payload = next((child for child in fields if child[0] == 0x420079), None)
    if payload is None:
        return operation_raw, None
    identifier = next((child for child in _children(request, payload) if child[0] == 0x420094), None)
    if identifier is None:
        return operation_raw, None
    assert identifier[1] == 7
    length = int.from_bytes(request[identifier[2] - 4 : identifier[2]], "big")
    return operation_raw, request[identifier[2] : identifier[2] + length]


def _lifecycle_response(operation: int, identifier: bytes) -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 1)),
    )
    if operation == 1:
        payload_fields = [
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, identifier),
        ]
    elif operation == 10:
        key_block = _structure(
            0x420040,
            _item(0x420042, 5, struct.pack(">I", 1)),
            _structure(
                0x420045,
                _item(0x420043, 8, bytes(range(32))),
            ),
        )
        payload_fields = [
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, identifier),
            _structure(0x42008F, key_block),
        ]
    elif operation == 20:
        payload_fields = [_item(0x420094, 7, identifier)]
    else:
        raise AssertionError(f"unexpected fluent operation {operation}")
    response_item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", operation)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        _structure(0x42007C, *payload_fields),
    )
    return _structure(0x42007B, header, response_item)


def _start_lifecycle_peer(
    expected_operations: tuple[int, ...],
    *,
    expected_asynchronous_indicator: int | None = None,
) -> tuple[int, list[tuple[int, bytes | None]], queue.Queue[BaseException], threading.Thread]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(3)
    listener.settimeout(12)
    port = listener.getsockname()[1]
    observations: list[tuple[int, bytes | None]] = []
    failures: queue.Queue[BaseException] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            for expected_operation in expected_operations:
                raw, _ = listener.accept()
                with raw, context.wrap_socket(raw, server_side=True) as stream:
                    require_tls13_mtls_peer(stream)
                    request = read_ttlv_frame(stream)
                    if expected_asynchronous_indicator is not None:
                        actual_indicator = _request_header_indicator(request)
                        if actual_indicator != expected_asynchronous_indicator:
                            raise AssertionError(
                                "expected asynchronous indicator "
                                f"{expected_asynchronous_indicator}, got {actual_indicator}"
                            )
                    operation, request_identifier = _request_operation_and_identifier(request)
                    if operation != expected_operation:
                        raise AssertionError(f"expected operation {expected_operation}, got {operation}")
                    observations.append((operation, request_identifier))
                    response_identifier = request_identifier or b"fluent-created-id"
                    stream.sendall(_lifecycle_response(operation, response_identifier))
        except BaseException as error:  # noqa: BLE001 - make peer failures visible in the test.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return port, observations, failures, peer


def test_fluent_symmetric_builder_requires_explicit_key_properties() -> None:
    builder = fluent.SymmetricKeyBuilder()
    try:
        builder.aes_256()
        with pytest.raises(errors.KmipKitError):
            builder.build()
    finally:
        builder.close()


def test_client_request_namespace_and_named_identifier_validation() -> None:
    assert hasattr(client.Client, "request")
    assert hasattr(client.Client, "create_symmetric_key")
    assert hasattr(client.Client, "get_by_id")
    assert hasattr(client.Client, "destroy_by_id")
    with pytest.raises(errors.KmipKitError):
        fluent._identifier("bad")


def test_operation_builds_typed_batch_item() -> None:
    operation = fluent.Operation(
        object(), "get", extensions.UniqueIdentifier.text_string("key-1")
    )
    with operation.build() as item:
        assert item.operation == 10


def test_fluent_operation_and_symmetric_builder_execute_with_and_without_options() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((10, 20, 1, 1))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    identifier = extensions.UniqueIdentifier.text_string("fluent-execute-id")
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                results.append(kmip_client.request().get(identifier).execute(limits))
                results.append(kmip_client.request().destroy(identifier).with_options(
                    client.RequestOptions()
                ).execute(limits))
                results.append(kmip_client.request().create_symmetric_key()
                               .aes_256().usage_mask(12).execute(limits))
                results.append(kmip_client.request().create_symmetric_key()
                               .aes_256().usage_mask(12)
                               .with_options(client.RequestOptions()).execute(limits))
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [result.items[0].operation for result in results] == [10, 20, 1, 1]
        assert [operation for operation, _ in observations] == [10, 20, 1, 1]
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_destroy_operation_builds_typed_batch_item() -> None:
    operation = fluent.Operation(
        object(), "destroy", extensions.UniqueIdentifier.text_string("key-1")
    )
    with operation.to_batch_item() as item:
        assert item.operation == 20


def test_operation_sets_batch_level_asynchronous_indicator() -> None:
    operation = fluent.Operation(
        object(), "get", extensions.UniqueIdentifier.text_string("key-1")
    )
    assert operation.asynchronous_indicator(2) is operation
    assert operation._asynchronous_indicator == 2
    with pytest.raises(errors.KmipKitError):
        operation.asynchronous_indicator(True)


def test_operation_async_indicator_is_encoded_in_request_header() -> None:
    port, _, peer_failures, peer = _start_lifecycle_peer(
        (10,), expected_asynchronous_indicator=2
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result: client.BatchResult | None = None
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                result = (
                    kmip_client.request()
                    .get(extensions.UniqueIdentifier.text_string("key-1"))
                    .asynchronous_indicator(2)
                    .execute(limits)
                )
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert result is not None and result.items[0].operation == 10
        assert result.items[0].outcome is client.BatchOutcome.COMPLETED
    finally:
        if result is not None:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_builder_builds_existing_typed_batch_item() -> None:
    with fluent.SymmetricKeyBuilder() as builder:
        item = builder.aes_256().usage_mask(12).name("test-key").build()
    batch = client.Batch.create()
    try:
        batch.add_item(item)
        assert batch.item_count == 1
    finally:
        batch.close()


def test_builder_context_manager_releases_native_handle() -> None:
    builder = fluent.SymmetricKeyBuilder()
    with builder:
        assert builder._handle != fluent.ffi.NULL
    assert builder._handle == fluent.ffi.NULL
    with pytest.raises(RuntimeError, match="closed"):
        builder.name("too-late")


def test_asynchronous_indicator_is_batch_level_and_not_a_request_option() -> None:
    with fluent.SymmetricKeyBuilder() as builder:
        builder.asynchronous_indicator(1)
        assert builder._asynchronous_indicator == 1
        assert builder._options is None
        with pytest.raises(errors.KmipKitError):
            builder.asynchronous_indicator(True)


def test_fluent_async_indicator_is_encoded_in_request_header() -> None:
    observed: list[bytes] = []

    def handler(stream: object) -> None:
        request = read_ttlv_frame(stream)  # type: ignore[arg-type]
        observed.append(request)
        root = _parse_item(request, 0, len(request))
        batch_item = next(child for child in _children(request, root) if child[0] == 0x42000F)
        assert batch_item[0] == 0x42000F
        stream.sendall(_create_response())  # type: ignore[attr-defined]

    port, peer_failures, peer = start_tls13_mtls_peer(_FIXTURES, handler)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result: client.BatchResult | None = None
    try:
        with configuration, client.Client.create(configuration, mtls_transport_config(port, _FIXTURES)) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                result = (
                    kmip_client.request()
                    .create_symmetric_key()
                    .aes_256()
                    .usage_mask(12)
                    .asynchronous_indicator(2)
                    .execute(limits)
                )
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert len(observed) == 1
        assert _request_header_indicator(observed[0]) == 2
        assert len(result.items) == 1
        assert result.items[0].outcome is client.BatchOutcome.COMPLETED
    finally:
        if result is not None:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_fluent_create_get_destroy_uses_server_created_identifier() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((1, 10, 20))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    try:
        with configuration, client.Client.create(configuration, mtls_transport_config(port, _FIXTURES)) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                create_result = (
                    kmip_client.request()
                    .create_symmetric_key()
                    .aes_256()
                    .usage_mask(12)
                    .execute(limits)
                )
                results.append(create_result)
                created_item = create_result.items[0]
                assert created_item.operation == 1
                assert created_item.outcome is client.BatchOutcome.COMPLETED
                assert created_item.result_status == 0
                assert created_item.create_object_type == 2
                assert created_item.unique_identifier == extensions.UniqueIdentifier.text_string(
                    "fluent-created-id"
                )

                created_identifier = created_item.unique_identifier
                assert created_identifier is not None
                get_result = kmip_client.request().get(created_identifier).execute(limits)
                results.append(get_result)
                get_item = get_result.items[0]
                assert get_item.operation == 10
                assert get_item.outcome is client.BatchOutcome.COMPLETED
                assert get_item.result_status == 0
                get_response = get_item.get_response()
                assert get_response is not None
                assert get_response.object_type_raw == 2
                assert get_response.unique_identifier == created_identifier
                assert get_response.object is not None

                destroy_result = kmip_client.request().destroy(created_identifier).execute(limits)
                results.append(destroy_result)
                destroy_item = destroy_result.items[0]
                assert destroy_item.operation == 20
                assert destroy_item.outcome is client.BatchOutcome.COMPLETED
                assert destroy_item.result_status == 0
                assert destroy_item.destroy_response is not None
                assert destroy_item.destroy_response.unique_identifier == created_identifier

        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert observations == [
            (1, None),
            (10, b"fluent-created-id"),
            (20, b"fluent-created-id"),
        ]
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_direct_client_helpers_create_get_and_destroy_with_exchange_options() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((1, 10, 20))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    identifier = extensions.UniqueIdentifier.text_string("fluent-created-id")
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                results.append(kmip_client.create_symmetric_key(
                    limits,
                    algorithm=3,
                    length=256,
                    usage_mask=12,
                    name="direct-key",
                    options=client.RequestOptions(),
                ))
                results.append(kmip_client.get_by_id(
                    identifier, limits, options=client.RequestOptions()
                ))
                results.append(kmip_client.destroy_by_id(
                    identifier, limits, options=client.RequestOptions()
                ))
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [result.items[0].operation for result in results] == [1, 10, 20]
        assert [operation for operation, _ in observations] == [1, 10, 20]
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_symmetric_builder_validates_catalogued_and_explicit_attributes() -> None:
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.ttlv_value_text_string(b"fluently managed", limits) as description:
            with fluent.SymmetricKeyBuilder() as builder:
                builder.aes_256().usage_mask(12).name("expert-key")
                builder.add_attribute(StandardAttribute.DESCRIPTION, description)
                for tag_value, text in ((0x4200FE, b"expert"), (0x42005C, b"vendor-value")):
                    raw = ttlv.raw_tag_from_raw(tag_value)
                    try:
                        tag = ttlv.raw_tag_try_checked(raw)
                        try:
                            value = ttlv.ttlv_value_text_string(text, limits)
                            item = ttlv.ttlv_item_create(tag, value, limits)
                            try:
                                builder.add_attribute_item(item)
                                if tag_value == 0x42005C:
                                    builder.add_vendor_attribute("Vendor_Name.1", "region", item)
                            finally:
                                item.close()
                        finally:
                            tag.close()
                    finally:
                        raw.close()
                with builder.build() as item:
                    assert item.operation == 1


def test_fluent_builder_and_operation_reject_invalid_options_and_closed_handles() -> None:
    builder = fluent.SymmetricKeyBuilder()
    with builder:
        with pytest.raises(errors.KmipKitError):
            builder.name(None)  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.with_options(object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.add_attribute("description", object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.add_attribute_item(object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.add_vendor_attribute("vendor", "name", object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.asynchronous_indicator(0x1_0000_0000)
        with pytest.raises(errors.KmipKitError):
            builder.execute(object())  # type: ignore[arg-type]
    with pytest.raises(RuntimeError, match="closed"):
        builder.build()

    operation = fluent.Operation(
        object(), "destroy", extensions.UniqueIdentifier.integer(-17)
    )
    assert operation.with_options(client.RequestOptions()) is operation
    with pytest.raises(errors.KmipKitError):
        operation.with_options(object())  # type: ignore[arg-type]
    record, owner = fluent._identifier(extensions.UniqueIdentifier.enumeration(9))
    assert owner is None
    assert record.enumeration_value == 9
    assert record.kind == int(extensions.UniqueIdentifierKind.ENUMERATION)


def test_fluent_abi_pointer_bridge_and_direct_entrypoint_argument_checks() -> None:
    foreign_pointer = fluent._base_ffi.cast("kmipkit_client_t *", 0x1234)
    bridged = fluent._client_abi_pointer("kmipkit_client_t *", foreign_pointer)
    assert int(fluent.ffi.cast("uintptr_t", bridged)) == 0x1234

    with ttlv.codec_limits_defaults() as limits:
        with pytest.raises(errors.KmipKitError):
            fluent._execute_direct(object(), object(), None, limits, None)  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            fluent._execute_direct_create(object(), object(), limits, None)  # type: ignore[arg-type]


def test_fluent_finalizer_swallows_shutdown_cleanup_errors(monkeypatch: pytest.MonkeyPatch) -> None:
    builder = fluent.SymmetricKeyBuilder()

    def fail_close() -> None:
        raise RuntimeError("native runtime already shut down")

    monkeypatch.setattr(builder, "close", fail_close)
    builder.__del__()


def test_legacy_extensions_remain_available() -> None:
    assert extensions.UniqueIdentifier.text_string("key-1").text == "key-1"
