from __future__ import annotations

import struct
from pathlib import Path

import pytest

from _test_support import mtls_transport_config, read_ttlv_frame, start_tls13_mtls_peer
from kmipkit import client, errors, extensions, fluent, ttlv

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


def test_legacy_extensions_remain_available() -> None:
    assert extensions.UniqueIdentifier.text_string("key-1").text == "key-1"
