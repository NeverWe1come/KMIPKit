from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path
from typing import NamedTuple

import pytest
from kmipkit import client, errors, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_BATCH_IDS = (b"create-0", b"pair-0", b"pair-1", b"pair-2")
_CORRELATION = b"\x00\xa1\x7fpair"


class _TtlvSpan(NamedTuple):
    tag: int
    item_type: int
    value_start: int
    value_length: int
    end: int


def _item(tag: int, kind: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((kind,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _enum(tag: int, value: int) -> bytes:
    return _item(tag, 5, struct.pack(">I", value))


def _response() -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 4)),
    )
    payload = _structure(
        0x42007C,
        _item(0x420066, 7, b"replacement-private"),
        _item(0x42006F, 7, b"replacement-public"),
        _item(0x541234, 8, b"\x01\x00\xff"),
    )
    items = [
        _structure(
            0x42000F,
            _enum(0x42005C, 1),
            _item(0x420093, 8, _BATCH_IDS[0]),
            _enum(0x42007F, 0),
            _structure(
                0x42007C, _enum(0x420057, 4), _item(0x420094, 7, b"existing-private")
            ),
        ),
        _structure(
            0x42000F,
            _enum(0x42005C, 0x1D),
            _item(0x420093, 8, _BATCH_IDS[1]),
            _enum(0x42007F, 0),
            payload,
        ),
        _structure(
            0x42000F,
            _enum(0x42005C, 0x1D),
            _item(0x420093, 8, _BATCH_IDS[2]),
            _enum(0x42007F, 1),
            _enum(0x42007E, 5),
        ),
        _structure(
            0x42000F,
            _enum(0x42005C, 0x1D),
            _item(0x420093, 8, _BATCH_IDS[3]),
            _enum(0x42007F, 2),
            _item(0x420006, 8, _CORRELATION),
            _structure(0x42007C),
        ),
    ]
    return _structure(0x42007B, header, *items)


def _read_exact(stream: ssl.SSLSocket, size: int) -> bytes:
    out = bytearray()
    while len(out) < size:
        chunk = stream.recv(size - len(out))
        if not chunk:
            raise AssertionError("client closed before full request")
        out.extend(chunk)
    return bytes(out)


def _read_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    return header + _read_exact(stream, int.from_bytes(header[4:8], "big"))


def _start_mtls_peer(
    response: bytes | None,
) -> tuple[int, list[bytes], queue.Queue[Exception], threading.Thread]:
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    received: list[bytes] = []
    failures: queue.Queue[Exception] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _ = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                assert stream.version() == "TLSv1.3"
                assert stream.getpeercert(binary_form=True)
                received.append(_read_frame(stream))
                if response is not None:
                    stream.sendall(response)
        except Exception as error:  # noqa: BLE001 - propagate peer errors to the test thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return port, received, failures, peer


def _transport(port: int) -> client.TransportConfig:
    return client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(
            client.TransportInput.file(
                _FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
            ),
        ),
        client_private_key=client.TransportInput.file(
            _FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.certificate_authorities(
            (
                client.TransportInput.file(
                    _FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
                ),
            )
        ),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(8),
    )


def _tags(value: ttlv.Structure) -> tuple[int, ...]:
    with ttlv.ttlv_structure_view(value) as view:
        return _view_tags(view)


def _view_tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    tags = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        with (
            ttlv.ttlv_structure_view_item_at(view, index) as item,
            ttlv.ttlv_item_view_tag(item) as tag,
        ):
            tags.append(ttlv.tag_value(tag))
    return tuple(tags)


def _sentinel_structure(
    tag_value: int, marker: bytes, limits: ttlv.CodecLimits
) -> ttlv.Structure:
    with (
        ttlv.raw_tag_from_raw(tag_value) as raw_tag,
        ttlv.raw_tag_try_checked(raw_tag) as tag,
        ttlv.ttlv_value_byte_string(marker, limits) as value,
        ttlv.ttlv_item_create(tag, value, limits) as item,
    ):
        structure = ttlv.ttlv_structure_create()
        try:
            return ttlv.ttlv_structure_with_item(structure, item, limits)
        finally:
            structure.close()


def _parse_item(encoded: bytes, offset: int, parent_end: int) -> _TtlvSpan:
    assert 0 <= offset <= parent_end - 8
    tag_value = int.from_bytes(encoded[offset : offset + 3], "big")
    item_type = encoded[offset + 3]
    value_length = int.from_bytes(encoded[offset + 4 : offset + 8], "big")
    value_start = offset + 8
    padded_length = value_length + (-value_length % 8)
    item_end = value_start + padded_length
    assert item_end <= parent_end
    return _TtlvSpan(tag_value, item_type, value_start, value_length, item_end)


def _children(encoded: bytes, parent: _TtlvSpan) -> list[_TtlvSpan]:
    assert parent.item_type == 1
    value_end = parent.value_start + parent.value_length
    children = []
    offset = parent.value_start
    while offset < value_end:
        child = _parse_item(encoded, offset, value_end)
        children.append(child)
        offset = child[4]
    assert offset == value_end
    return children


def _assert_request_fields(encoded: bytes) -> None:
    message = _parse_item(encoded, 0, len(encoded))
    assert message.tag == 0x420078 and message.item_type == 1
    assert message.end == len(encoded)
    request_payload = None
    for batch_item in _children(encoded, message):
        if batch_item.tag != 0x42000F or batch_item.item_type != 1:
            continue
        is_pair = False
        candidate_payload = None
        for field in _children(encoded, batch_item):
            if (
                field.tag == 0x42005C
                and field.item_type == 5
                and field.value_length == 4
            ):
                is_pair = (
                    int.from_bytes(
                        encoded[
                            field.value_start : field.value_start + field.value_length
                        ],
                        "big",
                    )
                    == 0x1D
                )
            elif field.tag == 0x420079 and field.item_type == 1:
                candidate_payload = field
        if is_pair:
            request_payload = candidate_payload
            break
    assert request_payload is not None

    fields = _children(encoded, request_payload)
    expected_tags = (
        0x420058,
        0x420126,
        0x420127,
        0x420128,
        0x420163,
        0x420164,
        0x420165,
    )
    assert tuple(field.tag for field in fields) == expected_tags
    offset = fields[0]
    assert offset.item_type == 10 and offset.value_length == 4
    assert (
        int.from_bytes(
            encoded[offset.value_start : offset.value_start + offset.value_length],
            "big",
        )
        == 86_400
    )

    marker_tags = (0x541101, 0x541102, 0x541103, 0x541104, 0x541105, 0x541106)
    markers = (
        b"common-attributes",
        b"private-attributes",
        b"public-attributes",
        b"common-masks",
        b"private-masks",
        b"public-masks",
    )
    for group, marker_tag, marker_value in zip(
        fields[1:], marker_tags, markers, strict=True
    ):
        children = _children(encoded, group)
        assert len(children) == 1
        marker = children[0]
        assert marker.tag == marker_tag and marker.item_type == 8
        assert (
            encoded[marker.value_start : marker.value_start + marker.value_length]
            == marker_value
        )


def _byte_string_at(
    structure: ttlv.TtlvStructureView, index: int, expected_tag: int
) -> bytes:
    with ttlv.ttlv_structure_view_item_at(structure, index) as item:
        with ttlv.ttlv_item_view_tag(item) as tag:
            assert ttlv.tag_value(tag) == expected_tag
        assert ttlv.ttlv_item_view_type(item) == ttlv.ItemType.ByteString
        with ttlv.ttlv_item_view_value(item) as value:
            length = ttlv.ttlv_value_view_byte_length(value)
            return bytes(
                ttlv.ttlv_value_view_byte_at(value, byte) for byte in range(length)
            )


def test_pair_rekey_cffi_preserves_both_ids_failure_pending_and_owned_response() -> (
    None
):
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    received: list[bytes] = []
    failures: queue.Queue[Exception] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _ = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                received.append(_read_frame(stream))
                stream.sendall(_response())
        except Exception as error:  # noqa: BLE001 - propagate background peer failures to the test thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    config = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    transport = client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(
            client.TransportInput.file(
                _FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
            ),
        ),
        client_private_key=client.TransportInput.file(
            _FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.certificate_authorities(
            (
                client.TransportInput.file(
                    _FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
                ),
            )
        ),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(8),
    )
    result = None
    pair = None
    try:
        with (
            config,
            client.Client.create(config, transport) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            batch.set_asynchronous_indicator(1)
            with ttlv.ttlv_structure_create() as attributes:
                create = extensions.client_batch_item_create(4, attributes)
            batch.add_item(extensions.with_unique_batch_item_id(create, _BATCH_IDS[0]))
            for index, batch_id in enumerate(_BATCH_IDS[1:]):
                if index == 0:
                    with (
                        ttlv.codec_limits_defaults() as group_limits,
                        _sentinel_structure(
                            0x541101, b"common-attributes", group_limits
                        ) as common_attributes,
                        _sentinel_structure(
                            0x541102, b"private-attributes", group_limits
                        ) as private_attributes,
                        _sentinel_structure(
                            0x541103, b"public-attributes", group_limits
                        ) as public_attributes,
                        _sentinel_structure(
                            0x541104, b"common-masks", group_limits
                        ) as common_masks,
                        _sentinel_structure(
                            0x541105, b"private-masks", group_limits
                        ) as private_masks,
                        _sentinel_structure(
                            0x541106, b"public-masks", group_limits
                        ) as public_masks,
                    ):
                        item = extensions.client_batch_item_rekey_key_pair(
                            offset=86_400,
                            common_attributes=common_attributes,
                            private_key_attributes=private_attributes,
                            public_key_attributes=public_attributes,
                            common_protection_storage_masks=common_masks,
                            private_protection_storage_masks=private_masks,
                            public_protection_storage_masks=public_masks,
                        )
                else:
                    item = extensions.client_batch_item_rekey_key_pair()
                batch.add_item(extensions.with_unique_batch_item_id(item, batch_id))
            result = kmip.execute(batch, limits)
        assert len(received) == 1
        _assert_request_fields(received[0])
        assert (
            received[0].count(b"\x42\x00\x5c\x05\x00\x00\x00\x04\x00\x00\x00\x1d") == 3
        )
        assert (
            b"\x42\x00\x66" not in received[0]
        )  # no local ID substituted for the server placeholder
        assert failures.empty() and result is not None
        created, success, failure, pending = result.items
        assert created.operation == 1 and created.result_status == 0
        assert success.outcome is client.BatchOutcome.COMPLETED
        assert failure.outcome is client.BatchOutcome.COMPLETED
        assert pending.outcome is client.BatchOutcome.PENDING
        pair = success.rekey_key_pair_response
        assert pair is not None
        assert (
            pair.private_key_unique_identifier
            == extensions.UniqueIdentifier.text_string("replacement-private")
        )
        assert (
            pair.public_key_unique_identifier
            == extensions.UniqueIdentifier.text_string("replacement-public")
        )
        assert (
            failure.rekey_key_pair_response is None
            and pending.rekey_key_pair_response is None
        )
        assert (
            pending.pending_correlation_value == _CORRELATION
            and pending.result_status == 2
        )
        result.close()
        assert not pair.response_ttlv.closed
        with ttlv.ttlv_structure_view(pair.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
            assert _view_tags(view) == (0x42005C, 0x420093, 0x42007F, 0x42007C)
            with (
                ttlv.ttlv_structure_view_item_at(view, 3) as payload_item,
                ttlv.ttlv_item_view_value(payload_item) as payload_value,
                ttlv.ttlv_value_view_structure(payload_value) as payload,
            ):
                assert _view_tags(payload) == (0x420066, 0x42006F, 0x541234)
                assert _byte_string_at(payload, 2, 0x541234) == b"\x01\x00\xff"
        assert "replacement-private" not in repr(
            pair
        ) and "replacement-public" not in repr(pair)
    finally:
        if pair is not None:
            pair.close()
        if result is not None:
            result.close()
        listener.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("mTLS peer did not finish")
        if not failures.empty():
            raise failures.get()


def test_pair_rekey_transport_error_preserves_delivery_state() -> None:
    port, received, failures, peer = _start_mtls_peer(None)
    config = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    try:
        with (
            config,
            client.Client.create(config, _transport(port)) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            item = extensions.client_batch_item_rekey_key_pair(
                extensions.UniqueIdentifier.text_string("delivery-key")
            )
            batch.add_item(extensions.with_unique_batch_item_id(item, _BATCH_IDS[1]))
            with pytest.raises(errors.ClientExecutionError) as failure:
                kmip.execute(batch, limits)

        assert failure.value.delivery is errors.DeliveryState.POSSIBLY_SENT
        assert failure.value.response_ttlv is None
        assert len(received) == 1
        assert _item(0x42005C, 5, struct.pack(">I", 0x1D)) in received[0]
        assert _item(0x420066, 7, b"delivery-key") in received[0]
    finally:
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("mTLS peer did not finish")
        if not failures.empty():
            raise failures.get()
