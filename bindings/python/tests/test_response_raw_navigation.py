"""KMIP v2.1 §6.1.19/§6.1.40 response TTLV navigation through CFFI."""

from __future__ import annotations

import ssl
import struct
from pathlib import Path

from _test_support import (
    mtls_transport_config,
    read_ttlv_frame,
    start_tls13_mtls_peer,
    ttlv_enumeration,
    ttlv_item,
    ttlv_structure,
)
from kmipkit import client, extensions, ttlv

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_QUERY_ID = b"raw-query"
_GET_ID = b"raw-get"


def _response_message() -> bytes:
    version = ttlv_structure(
        0x420069,
        ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = ttlv_structure(
        0x42007A,
        version,
        ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        ttlv_item(0x42000D, 2, struct.pack(">i", 2)),
    )
    server_information = ttlv_structure(
        0x420088,
        ttlv_structure(
            0x540001,
            ttlv_item(0x540002, 2, struct.pack(">i", 0x1234)),
        ),
    )
    query_payload = ttlv_structure(0x42007C, server_information, ttlv_structure(0x42015F))
    query_item = ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, 0x18),
        ttlv_item(0x420093, 8, _QUERY_ID),
        ttlv_enumeration(0x42007F, 0),
        query_payload,
    )
    key_block = ttlv_structure(
        0x420040,
        ttlv_enumeration(0x420042, 1),
        ttlv_structure(0x420045, ttlv_item(0x420043, 8, b"opaque-key-material")),
        ttlv_structure(
            0x540001,
            ttlv_item(0x540002, 2, struct.pack(">i", 0x5678)),
        ),
    )
    get_payload = ttlv_structure(
        0x42007C,
        ttlv_enumeration(0x420057, 2),
        ttlv_item(0x420094, 7, b"response-object"),
        ttlv_structure(0x42008F, key_block),
    )
    get_item = ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, 0x0A),
        ttlv_item(0x420093, 8, _GET_ID),
        ttlv_enumeration(0x42007F, 0),
        get_payload,
    )
    return ttlv_structure(0x42007B, header, query_item, get_item)


def _integer_at_path(root: ttlv.Structure, path: tuple[int, ...]) -> int:
    def descend(view: ttlv.TtlvStructureView, offset: int) -> int:
        for index in range(ttlv.ttlv_structure_view_item_count(view)):
            with ttlv.ttlv_structure_view_item_at(view, index) as item:
                with ttlv.ttlv_item_view_tag(item) as tag:
                    if ttlv.tag_value(tag) != path[offset]:
                        continue
                with ttlv.ttlv_item_view_value(item) as value:
                    if offset + 1 == len(path):
                        return ttlv.ttlv_value_view_integer(value)
                    with ttlv.ttlv_value_view_structure(value) as nested:
                        try:
                            return descend(nested, offset + 1)
                        except AssertionError:
                            continue
        raise AssertionError(f"TTLV path component {path[offset]:#x} was not found")

    with ttlv.ttlv_structure_view(root) as view:
        return descend(view, 0)


def test_raw_response_copies_navigate_vendor_extensions_and_outlive_result_close() -> None:
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    batch_result = None
    raw_query = None
    raw_get = None
    limits = ttlv.codec_limits_defaults()
    try:
        with (
            configuration,
            client.Client.create(configuration, mtls_transport_config(port, _TLS_FIXTURES)) as kmip,
            client.Batch.create() as batch,
        ):
            query_item = extensions.client_batch_item_query([3])
            batch.add_item(extensions.with_unique_batch_item_id(query_item, _QUERY_ID))
            get_item = extensions.client_batch_item_get(
                extensions.UniqueIdentifier.text_string("requested-object")
            )
            batch.add_item(extensions.with_unique_batch_item_id(get_item, _GET_ID))
            batch_result = kmip.execute(batch, limits)

        assert len(batch_result.items) == 2
        raw_query = batch_result.items[0].raw()
        raw_get = batch_result.items[1].raw()
        assert raw_query is not None and raw_get is not None
        assert "opaque-key-material" not in repr(batch_result.items[1])
        batch_result.close()

        assert _integer_at_path(raw_query, (0x420088, 0x540001, 0x540002)) == 0x1234
        assert _integer_at_path(raw_get, (0x42008F, 0x420040, 0x540001, 0x540002)) == 0x5678
        assert failures.empty(), "the TLS 1.3 mTLS peer must complete the response exchange"
        assert len(received) == 1
    finally:
        if raw_get is not None:
            raw_get.close()
        if raw_query is not None:
            raw_query.close()
        if batch_result is not None:
            batch_result.close()
        limits.close()
        peer.join(timeout=4)
