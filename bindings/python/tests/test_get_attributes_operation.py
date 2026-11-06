from __future__ import annotations

import ssl
import struct
from pathlib import Path

from _test_support import (
    mtls_transport_config,
    operation_response_message,
    read_ttlv_frame,
    start_tls13_mtls_peer,
    ttlv_enumeration,
    ttlv_item,
    ttlv_structure,
)

from kmipkit import client, extensions, ttlv

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_GET_ATTRIBUTES_ID = b"get-attributes-0"


def _get_attributes_response_message() -> bytes:
    return operation_response_message(
        0x0B,
        _GET_ATTRIBUTES_ID,
        payload_items=(
            ttlv_item(0x0042_0094, 7, b"object-7"),
            ttlv_structure(
                0x0042_0125,
                ttlv_item(0x0042_002F, 9, struct.pack(">q", 19)),
                ttlv_item(0x0042_00FD, 7, b"KMIPKit comment"),
            ),
        ),
    )


def test_get_attributes_builder_preserves_identifier_and_reference_order(monkeypatch):
    calls: list[tuple[str, tuple[object, ...]]] = []
    identifier = object()
    references = object()
    monkeypatch.setattr(
        extensions,
        "_client_native_unique_identifier",
        lambda value, _staging: identifier if value is not None else None,
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_attribute_reference_array",
        lambda values, _staging: (references, len(values)),
        raising=False,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    attribute_references = (
        extensions.AttributeReference.tag(0x0042_002F),
        extensions.AttributeReference.name("KMIPKit", "Comment"),
    )
    omitted = extensions.client_batch_item_get_attributes(None, ())
    selected = extensions.client_batch_item_get_attributes(
        extensions.UniqueIdentifier.text_string("object-7"), attribute_references
    )

    assert omitted == selected == 73
    assert calls == [
        ("kmipkit_client_batch_item_get_attributes", (None, None, 0)),
        (
            "kmipkit_client_batch_item_get_attributes",
            (identifier, references, 2),
        ),
    ]


def test_get_attributes_executes_once_and_preserves_requested_attributes():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_get_attributes_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    request_item = None
    batch_result = None
    references = (
        extensions.AttributeReference.tag(0x0042_002F),
        extensions.AttributeReference.name("KMIPKit", "Comment"),
    )
    try:
        with (
            configuration,
            client.Client.create(
                configuration, mtls_transport_config(port, _TLS_FIXTURES)
            ) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_get_attributes(
                extensions.UniqueIdentifier.text_string("object-7"), references
            )
            batch.add_item(
                extensions.with_unique_batch_item_id(request_item, _GET_ATTRIBUTES_ID)
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        request = received[0]
        uid = ttlv_item(0x0042_0094, 7, b"object-7")
        tag_reference = ttlv_enumeration(0x0042_013B, 0x0042_002F)
        name_reference = ttlv_structure(
            0x0042_013B,
            ttlv_item(0x0042_009D, 7, b"KMIPKit"),
            ttlv_item(0x0042_000A, 7, b"Comment"),
        )
        assert failures.empty(), "the mTLS peer must complete one Get Attributes exchange"
        assert len(received) == 1
        assert ttlv_enumeration(0x0042_005C, 0x0B) in request
        assert uid in request and tag_reference in request and name_reference in request
        assert request.index(uid) < request.index(tag_reference) < request.index(name_reference)
        assert batch_result is not None and len(batch_result.items) == 1
        response = batch_result.items[0].get_attributes_response
        assert response is not None
        assert response.unique_identifier == "object-7"
        assert response.attributes is not None
        with ttlv.ttlv_structure_view(response.attributes) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 2
    finally:
        if request_item is not None:
            request_item.close()
        if batch_result is not None:
            batch_result.close()
        peer.join(timeout=4)


def _assert_get_attributes_response_shape(
    payload_items: tuple[bytes, ...], *, attributes_present: bool
) -> None:
    received: list[bytes] = []
    response = operation_response_message(
        0x0B,
        _GET_ATTRIBUTES_ID,
        payload_items=payload_items,
    )

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(response)

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    batch_result = None
    request_item = None
    try:
        with (
            configuration,
            client.Client.create(
                configuration, mtls_transport_config(port, _TLS_FIXTURES)
            ) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_get_attributes(
                extensions.UniqueIdentifier.text_string("object-7"),
                (extensions.AttributeReference.tag(0x0042_002F),),
            )
            batch.add_item(
                extensions.with_unique_batch_item_id(request_item, _GET_ATTRIBUTES_ID)
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        assert failures.empty(), "the mTLS peer must complete one Get Attributes exchange"
        assert len(received) == 1
        assert batch_result is not None and len(batch_result.items) == 1
        result = batch_result.items[0].get_attributes_response
        assert result is not None
        assert result.unique_identifier == "object-7"
        assert (result.attributes is not None) is attributes_present
        if attributes_present:
            assert result.attributes is not None
            with ttlv.ttlv_structure_view(result.attributes) as view:
                assert ttlv.ttlv_structure_view_item_count(view) == 0
    finally:
        if request_item is not None:
            request_item.close()
        if batch_result is not None:
            batch_result.close()
        peer.join(timeout=4)


def test_get_attributes_accepts_successful_no_match_response_with_identifier_only():
    _assert_get_attributes_response_shape(
        (ttlv_item(0x0042_0094, 7, b"object-7"),), attributes_present=False
    )


def test_get_attributes_preserves_present_empty_attributes_structure():
    _assert_get_attributes_response_shape(
        (
            ttlv_item(0x0042_0094, 7, b"object-7"),
            ttlv_structure(0x0042_0125),
        ),
        attributes_present=True,
    )
