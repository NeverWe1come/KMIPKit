from __future__ import annotations

import ssl
from pathlib import Path

from _test_support import (
    mtls_transport_config,
    operation_response_message,
    read_ttlv_frame,
    start_tls13_mtls_peer,
    ttlv_item,
    ttlv_structure,
)

from kmipkit import client, extensions, ttlv

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_DELETE_ATTRIBUTE_ID = b"delete-attribute-0"


def _delete_attribute_response_message() -> bytes:
    return operation_response_message(
        0x0F,
        _DELETE_ATTRIBUTE_ID,
        payload_items=(ttlv_item(0x0042_0094, 7, b"object-7"),),
    )


def test_delete_attribute_builder_preserves_each_optional_selector(monkeypatch):
    calls: list[tuple[str, tuple[object, ...], tuple[int, ...]]] = []
    omitted_identifier = object()
    present_identifier = object()
    current_attribute = object()
    current_handle = object()
    reference = object()
    reference_handle = object()

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
        lambda value: None if value is None else current_handle,
        raising=False,
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_attribute_reference",
        lambda value, _staging: None if value is None else reference_handle,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments, *, consumed_indices=():
            calls.append((symbol, arguments, consumed_indices)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    omitted = extensions.client_batch_item_delete_attribute(None, None, None)
    present = extensions.client_batch_item_delete_attribute(
        extensions.UniqueIdentifier.text_string("object-7"),
        current_attribute,
        reference,
    )

    assert omitted == present == 73
    assert calls == [
        (
            "kmipkit_client_batch_item_delete_attribute",
            (omitted_identifier, None, None),
            (),
        ),
        (
            "kmipkit_client_batch_item_delete_attribute",
            (present_identifier, current_handle, reference_handle),
            (),
        ),
    ]


def test_delete_attribute_executes_once_and_maps_typed_response():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_delete_attribute_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    limits = ttlv.codec_limits_defaults()
    current_attribute = None
    request_item = None
    batch_result = None
    raw_tag = ttlv.raw_tag_from_raw(0x0042_00FD)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_text_string(b"old comment", limits)
    try:
        current_attribute = ttlv.ttlv_item_create(tag, value, limits)
        value.close()
        with (
            configuration,
            client.Client.create(
                configuration, mtls_transport_config(port, _TLS_FIXTURES)
            ) as kmip,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_delete_attribute(
                extensions.UniqueIdentifier.text_string("object-7"),
                current_attribute,
                extensions.AttributeReference.name("KMIPKit", "Comment"),
            )
            current_attribute.close()
            current_attribute = None
            batch.add_item(
                extensions.with_unique_batch_item_id(
                    request_item, _DELETE_ATTRIBUTE_ID
                )
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        request = received[0]
        uid = ttlv_item(0x0042_0094, 7, b"object-7")
        current = ttlv_structure(
            0x0042_013C, ttlv_item(0x0042_00FD, 7, b"old comment")
        )
        reference = ttlv_structure(
            0x0042_013B,
            ttlv_item(0x0042_009D, 7, b"KMIPKit"),
            ttlv_item(0x0042_000A, 7, b"Comment"),
        )
        assert failures.empty(), "the mTLS peer must complete one Delete Attribute exchange"
        assert len(received) == 1
        assert ttlv_item(0x0042_005C, 0x05, b"\x00\x00\x00\x0f") in request
        assert uid in request and current in request and reference in request
        assert request.index(uid) < request.index(current) < request.index(reference)
        assert batch_result is not None and len(batch_result.items) == 1
        response = batch_result.items[0].delete_attribute_response
        assert response is not None
        assert response.unique_identifier == "object-7"
    finally:
        if current_attribute is not None:
            current_attribute.close()
        if request_item is not None:
            request_item.close()
        if batch_result is not None:
            batch_result.close()
        limits.close()
        tag.close()
        raw_tag.close()
        peer.join(timeout=4)
