"""Native CFFI RNG mapping tests derived from KMIP 2.1 §§6.1.49–6.1.50."""

from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

from _test_support import (
    mtls_transport_config,
    read_ttlv_frame,
    require_tls13_mtls_peer,
)
from kmipkit import client, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_BATCH_IDS = (b"item-0", b"item-1")
_ASYNC_BATCH_IDS = (b"item-0", b"item-1", b"item-2", b"item-3")
_SEED = b"\x00\x81\xff"
_RETRIEVED = b"KMIP_RNG_RETRIEVE_SECRET_SENTINEL"
_CORRELATION = b"\x00\xa1\x7f"


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _enumeration(tag: int, value: int) -> bytes:
    return _ttlv_item(tag, 5, struct.pack(">I", value))


def _rng_response_message() -> bytes:
    version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _ttlv_structure(
        0x42007A,
        version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 2)),
    )
    retrieve = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x25),
        _ttlv_item(0x420093, 8, _BATCH_IDS[0]),
        _enumeration(0x42007F, 0),
        _ttlv_structure(0x42007C, _ttlv_item(0x4200C2, 8, _RETRIEVED)),
    )
    seed = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x26),
        _ttlv_item(0x420093, 8, _BATCH_IDS[1]),
        _enumeration(0x42007F, 0),
        _ttlv_structure(0x42007C, _ttlv_item(0x4200D5, 2, struct.pack(">i", 0))),
    )
    return _ttlv_structure(0x42007B, header, retrieve, seed)


def _rng_failure_pending_response_message() -> bytes:
    version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _ttlv_structure(
        0x42007A,
        version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 4)),
    )
    retrieve_failure = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x25),
        _ttlv_item(0x420093, 8, _ASYNC_BATCH_IDS[0]),
        _enumeration(0x42007F, 1),
        _enumeration(0x42007E, 5),
    )
    retrieve_pending = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x25),
        _ttlv_item(0x420093, 8, _ASYNC_BATCH_IDS[1]),
        _enumeration(0x42007F, 2),
        _ttlv_item(0x420006, 8, _CORRELATION),
        _ttlv_structure(0x42007C),
    )
    seed_failure = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x26),
        _ttlv_item(0x420093, 8, _ASYNC_BATCH_IDS[2]),
        _enumeration(0x42007F, 1),
        _enumeration(0x42007E, 5),
    )
    seed_pending = _ttlv_structure(
        0x42000F,
        _enumeration(0x42005C, 0x26),
        _ttlv_item(0x420093, 8, _ASYNC_BATCH_IDS[3]),
        _enumeration(0x42007F, 2),
        _ttlv_item(0x420006, 8, _CORRELATION),
        _ttlv_structure(0x42007C),
    )
    return _ttlv_structure(
        0x42007B, header, retrieve_failure, retrieve_pending, seed_failure, seed_pending
    )


def _item_tags(structure: ttlv.Structure) -> tuple[int, ...]:
    with ttlv.ttlv_structure_view(structure) as view:
        tags: list[int] = []
        for index in range(ttlv.ttlv_structure_view_item_count(view)):
            with ttlv.ttlv_structure_view_item_at(view, index) as item:
                with ttlv.ttlv_item_view_tag(item) as tag:
                    tags.append(ttlv.tag_value(tag))
        return tuple(tags)


def _contains_bytes(actual: bytes, expected: bytes) -> bool:
    return any(
        actual[offset : offset + len(expected)] == expected
        for offset in range(len(actual) - len(expected) + 1)
    )


def _start_mtls_peer(
    response_message: bytes | None = None,
) -> tuple[int, list[bytes], queue.Queue[BaseException], threading.Thread, socket.socket]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[BaseException] = queue.Queue()
    received_requests: list[bytes] = []

    def serve_one_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                require_tls13_mtls_peer(stream)
                received_requests.append(
                    read_ttlv_frame(
                        stream,
                        eof_message="client closed before sending a full TTLV frame",
                    )
                )
                stream.sendall(response_message or _rng_response_message())
        except OSError as failure:
            if listener.fileno() != -1:
                peer_failures.put(failure)
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_one_response, daemon=True)
    peer.start()
    return port, received_requests, peer_failures, peer, listener


def _transport(port: int) -> client.TransportConfig:
    return mtls_transport_config(port, _FIXTURES)


def test_rng_operations_cross_native_cffi_over_mutual_tls() -> None:
    port, received_requests, peer_failures, peer, listener = _start_mtls_peer()
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    batch_result: client.BatchResult | None = None
    retrieve_response: client.RngRetrieveResponse | None = None
    seed_response: client.RngSeedResponse | None = None
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            batch = client.Batch.create()
            batch.set_asynchronous_indicator(1)
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_rng_retrieve(-17), _BATCH_IDS[0]
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_rng_seed(_SEED), _BATCH_IDS[1]
            ))
            with ttlv.codec_limits_defaults() as limits:
                batch_result = kmip_client.execute(batch, limits)

        assert len(received_requests) == 1
        request = received_requests[0]
        assert _contains_bytes(request, _enumeration(0x42005C, 0x25))
        assert _contains_bytes(request, _ttlv_item(0x4200D5, 2, struct.pack(">i", -17)))
        assert _contains_bytes(request, _enumeration(0x42005C, 0x26))
        assert _contains_bytes(request, _ttlv_item(0x4200C2, 8, _SEED)), (
            "RNG Seed must preserve caller-supplied bytes"
        )
        assert peer_failures.empty()
        assert batch_result is not None and len(batch_result.items) == 2
        retrieve, seed = batch_result.items
        assert retrieve.operation == 0x25 and retrieve.result_status == 0
        assert seed.operation == 0x26 and seed.result_status == 0
        retrieve_response = retrieve.rng_retrieve_response()
        seed_response = seed.rng_seed_response()
        assert retrieve_response is not None
        assert seed_response is not None
        caller_copy = retrieve_response.data
        assert caller_copy == _RETRIEVED, "RNG Retrieve must preserve returned bytes"
        assert _RETRIEVED.decode("ascii") not in repr(retrieve_response)
        assert seed_response.data_length == 0

        batch_result.close()
        assert not retrieve_response.response_ttlv.closed
        assert not seed_response.response_ttlv.closed
        assert _item_tags(retrieve_response.response_ttlv) == (
            0x42005C,
            0x420093,
            0x42007F,
            0x42007C,
        )
        assert _item_tags(seed_response.response_ttlv) == (
            0x42005C,
            0x420093,
            0x42007F,
            0x42007C,
        )
        owned_data = retrieve_response._data
        retrieve_response.close()
        seed_response.close()
        assert owned_data is not None
        assert owned_data.buffer == bytearray(len(_RETRIEVED))
    finally:
        if retrieve_response is not None:
            retrieve_response.close()
        if seed_response is not None:
            seed_response.close()
        if batch_result is not None:
            batch_result.close()
        listener.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()


def test_rng_failure_and_pending_cross_native_cffi_without_success_values() -> None:
    port, received_requests, peer_failures, peer, listener = _start_mtls_peer(
        _rng_failure_pending_response_message()
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    batch_result: client.BatchResult | None = None
    try:
        with configuration, client.Client.create(configuration, _transport(port)) as kmip_client:
            batch = client.Batch.create()
            batch.set_asynchronous_indicator(1)
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_rng_retrieve(8), _ASYNC_BATCH_IDS[0]
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_rng_retrieve(8), _ASYNC_BATCH_IDS[1]
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_rng_seed(b"\x00\xff"), _ASYNC_BATCH_IDS[2]
            ))
            batch.add_item(extensions.with_unique_batch_item_id(
                extensions.client_batch_item_rng_seed(b"\x01"), _ASYNC_BATCH_IDS[3]
            ))
            with ttlv.codec_limits_defaults() as limits:
                batch_result = kmip_client.execute(batch, limits)

        assert len(received_requests) == 1
        assert peer_failures.empty()
        assert batch_result is not None and len(batch_result.items) == 4
        retrieve_failure, retrieve_pending, seed_failure, seed_pending = batch_result.items
        assert retrieve_failure.operation == 0x25 and retrieve_failure.result_status == 1
        assert retrieve_failure.result_reason == 5
        assert retrieve_failure.rng_retrieve_response() is None
        assert retrieve_pending.operation == 0x25 and retrieve_pending.result_status == 2
        assert retrieve_pending.pending_correlation_value == _CORRELATION
        assert retrieve_pending.rng_retrieve_response() is None
        assert seed_failure.operation == 0x26 and seed_failure.result_status == 1
        assert seed_failure.result_reason == 5
        assert seed_failure.rng_seed_response() is None
        assert seed_pending.operation == 0x26 and seed_pending.result_status == 2
        assert seed_pending.pending_correlation_value == _CORRELATION
        assert seed_pending.rng_seed_response() is None
    finally:
        if batch_result is not None:
            batch_result.close()
        listener.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()
