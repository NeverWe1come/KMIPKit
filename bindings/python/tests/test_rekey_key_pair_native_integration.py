from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

from kmipkit import client, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_BATCH_IDS = (b"create-0", b"pair-0", b"pair-1", b"pair-2")
_CORRELATION = b"\x00\xa1\x7fpair"


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


def _tags(value: ttlv.Structure) -> tuple[int, ...]:
    with ttlv.ttlv_structure_view(value) as view:
        return tuple(
            ttlv.tag_value(
                ttlv.ttlv_item_view_tag(ttlv.ttlv_structure_view_item_at(view, i))
            )
            for i in range(ttlv.ttlv_structure_view_item_count(view))
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
                        ttlv.ttlv_structure_create() as common_attributes,
                        ttlv.ttlv_structure_create() as private_attributes,
                        ttlv.ttlv_structure_create() as public_attributes,
                        ttlv.ttlv_structure_create() as common_masks,
                        ttlv.ttlv_structure_create() as private_masks,
                        ttlv.ttlv_structure_create() as public_masks,
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
        assert (
            received[0].count(b"\x42\x00\x5c\x05\x00\x00\x00\x04\x00\x00\x00\x1d") == 3
        )
        assert (
            b"\x42\x00\x66" not in received[0]
        )  # no local ID substituted for the server placeholder
        assert failures.empty() and result is not None
        created, success, failure, pending = result.items
        assert created.operation == 1 and created.result_status == 0
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
