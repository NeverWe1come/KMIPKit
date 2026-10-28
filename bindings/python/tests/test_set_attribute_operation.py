"""Set Attribute mapping tests from OASIS KMIP 2.1 §6.1.51, Tables 322–324.

These tests implement KMIPKIT-0016-FR-009.
"""

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
_SET_ATTRIBUTE_ID = b"set-attribute-0"
_FAILED_SET_ATTRIBUTE_ID = b"set-attribute-1"
_PING_ID = b"set-attribute-ping"


def _set_attribute_response_message() -> bytes:
    version = ttlv_structure(
        0x420069,
        ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = ttlv_structure(
        0x42007A,
        version,
        ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        ttlv_item(0x42000D, 2, struct.pack(">i", 3)),
    )
    successful_set_attribute = ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, 0x31),
        ttlv_item(0x420093, 8, _SET_ATTRIBUTE_ID),
        ttlv_enumeration(0x42007F, 0),
        ttlv_structure(0x42007C, ttlv_item(0x420094, 7, b"server-object-9")),
    )
    failed_set_attribute = ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, 0x31),
        ttlv_item(0x420093, 8, _FAILED_SET_ATTRIBUTE_ID),
        ttlv_enumeration(0x42007F, 1),
        ttlv_enumeration(0x42007E, 1),
        ttlv_item(0x42007D, 7, b"server rejected the update"),
    )
    successful_ping = ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, 0x3B),
        ttlv_item(0x420093, 8, _PING_ID),
        ttlv_enumeration(0x42007F, 0),
        ttlv_structure(0x42007C),
    )
    return ttlv_structure(
        0x42007B,
        header,
        successful_set_attribute,
        failed_set_attribute,
        successful_ping,
    )


def test_set_attribute_builder_preserves_optional_identifier_and_borrows_new_attribute(
    monkeypatch,
):
    calls: list[tuple[str, tuple[object, ...], tuple[int, ...]]] = []
    attribute_handle = object()
    omitted_identifier = object()
    present_identifier = object()
    attribute = object.__new__(ttlv.TtlvItem)

    monkeypatch.setattr(
        extensions,
        "_client_native_unique_identifier",
        lambda value, _staging: omitted_identifier
        if value is None
        else present_identifier,
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_item",
        lambda _value: attribute_handle,
        raising=False,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments, *, consumed_indices=():
            calls.append((symbol, arguments, consumed_indices)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    omitted = extensions.client_batch_item_set_attribute(None, attribute)
    present = extensions.client_batch_item_set_attribute(
        extensions.UniqueIdentifier.text_string("request-object-3"), attribute
    )

    assert omitted == present == 73
    assert calls == [
        (
            "kmipkit_client_batch_item_set_attribute",
            (omitted_identifier, attribute_handle),
            (),
        ),
        (
            "kmipkit_client_batch_item_set_attribute",
            (present_identifier, attribute_handle),
            (),
        ),
    ]


def test_set_attribute_executes_once_maps_only_successful_results_and_borrows_item():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_set_attribute_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    request_items: list[extensions.ClientBatchItem | None] = []
    batch_result = None
    limits = ttlv.codec_limits_defaults()
    raw_comment_tag = ttlv.raw_tag_from_raw(0x0042_00FD)
    comment_tag = ttlv.raw_tag_try_checked(raw_comment_tag)
    values: list[ttlv.Value] = []
    attributes: list[ttlv.TtlvItem] = []
    try:
        for value in (b"first comment", b"second comment"):
            attribute_value = ttlv.ttlv_value_text_string(value, limits)
            values.append(attribute_value)
            attributes.append(
                ttlv.ttlv_item_create(comment_tag, attribute_value, limits)
            )

        with (
            configuration,
            client.Client.create(
                configuration, mtls_transport_config(port, _TLS_FIXTURES)
            ) as kmip,
        ):
            batch = client.Batch.create()
            request_items.append(
                extensions.client_batch_item_set_attribute(
                    extensions.UniqueIdentifier.text_string("request-object-3"),
                    attributes[0],
                )
            )
            assert attributes[0]._pointer() != extensions.ffi.NULL
            batch.add_item(
                extensions.with_unique_batch_item_id(
                    request_items[-1], _SET_ATTRIBUTE_ID
                )
            )
            request_items[-1] = None
            request_items.append(
                extensions.client_batch_item_set_attribute(None, attributes[1])
            )
            assert attributes[1]._pointer() != extensions.ffi.NULL
            batch.add_item(
                extensions.with_unique_batch_item_id(
                    request_items[-1], _FAILED_SET_ATTRIBUTE_ID
                )
            )
            request_items[-1] = None
            request_items.append(extensions.client_batch_item_ping())
            batch.add_item(
                extensions.with_unique_batch_item_id(request_items[-1], _PING_ID)
            )
            request_items[-1] = None
            batch_result = kmip.execute(batch, limits)

        assert failures.empty(), "the mTLS peer must complete the Set Attribute exchange"
        assert len(received) == 1, "the client must use one exchange for the batch"
        assert received[0].count(ttlv_enumeration(0x42005C, 0x31)) == 2
        assert ttlv_enumeration(0x42005C, 0x3B) in received[0]
        assert ttlv_item(0x420094, 7, b"request-object-3") in received[0]
        assert received[0].count(b"\x42\x00\x94") == 1, (
            "the omitted Set Attribute Unique Identifier must stay absent"
        )
        assert ttlv_structure(
            0x0042_013D, ttlv_item(0x0042_00FD, 7, b"first comment")
        ) in received[0]
        assert ttlv_structure(
            0x0042_013D, ttlv_item(0x0042_00FD, 7, b"second comment")
        ) in received[0]
        assert attributes[0]._pointer() != extensions.ffi.NULL
        assert attributes[1]._pointer() != extensions.ffi.NULL
        assert batch_result is not None and len(batch_result.items) == 3

        successful, failed, other_operation = batch_result.items
        assert successful.operation == 0x31
        response = successful.set_attribute_response
        assert response is not None
        assert response.unique_identifier == "server-object-9"
        assert "server-object-9" not in repr(response)
        assert failed.operation == 0x31
        assert failed.result_status == 1
        assert failed.set_attribute_response is None
        assert other_operation.operation == 0x3B
        assert other_operation.set_attribute_response is None
    finally:
        for request_item in request_items:
            if request_item is not None:
                request_item.close()
        for attribute in attributes:
            attribute.close()
        for value in values:
            value.close()
        if batch_result is not None:
            batch_result.close()
        limits.close()
        comment_tag.close()
        raw_comment_tag.close()
        peer.join(timeout=4)
