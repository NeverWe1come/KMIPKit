from __future__ import annotations

import ssl
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

from kmipkit import extensions, ttlv
from kmipkit import client

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_ADD_ATTRIBUTE_ID = b"add-attribute-0"
_ADJUST_ATTRIBUTE_ID = b"adjust-attribute-0"


def _add_attribute_response_message() -> bytes:
    return operation_response_message(
        0x0D,
        _ADD_ATTRIBUTE_ID,
        payload_items=(ttlv_item(0x0042_0094, 7, b"object-7"),),
    )


def _adjust_attribute_response_message() -> bytes:
    return operation_response_message(
        0x30,
        _ADJUST_ATTRIBUTE_ID,
        payload_items=(ttlv_item(0x0042_0094, 7, b"adjusted-object-1"),),
    )


def test_add_attribute_builder_preserves_identifier_and_borrows_new_attribute(
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

    omitted = extensions.client_batch_item_add_attribute(None, attribute)
    present = extensions.client_batch_item_add_attribute(
        extensions.UniqueIdentifier.text_string("object-7"), attribute
    )

    assert omitted == present == 73
    assert calls == [
        (
            "kmipkit_client_batch_item_add_attribute",
            (omitted_identifier, attribute_handle),
            (),
        ),
        (
            "kmipkit_client_batch_item_add_attribute",
            (present_identifier, attribute_handle),
            (),
        ),
    ]


def test_add_attribute_executes_once_and_preserves_the_new_attribute_item():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_add_attribute_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    request_item = None
    batch_result = None
    limits = ttlv.codec_limits_defaults()
    raw_name_tag = ttlv.raw_tag_from_raw(0x0042_000A)
    name_tag = ttlv.raw_tag_try_checked(raw_name_tag)
    raw_value_tag = ttlv.raw_tag_from_raw(0x0042_00A5)
    value_tag = ttlv.raw_tag_try_checked(raw_value_tag)
    raw_type_tag = ttlv.raw_tag_from_raw(0x0042_00A6)
    type_tag = ttlv.raw_tag_try_checked(raw_type_tag)
    new_attribute = None
    try:
        name_value = ttlv.ttlv_item_create(
            value_tag, ttlv.ttlv_value_text_string(b"new name", limits), limits
        )
        name_type = ttlv.ttlv_item_create(
            type_tag, ttlv.ttlv_value_enumeration(1), limits
        )
        name_value_structure = ttlv.ttlv_structure_with_item(
            ttlv.ttlv_structure_create(), name_value, limits
        )
        name_value_structure = ttlv.ttlv_structure_with_item(
            name_value_structure, name_type, limits
        )
        new_attribute = ttlv.ttlv_item_create(
            name_tag, ttlv.ttlv_value_structure(name_value_structure, limits), limits
        )
        with (
            configuration,
            client.Client.create(configuration, mtls_transport_config(port, _TLS_FIXTURES)) as kmip,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_add_attribute(None, new_attribute)
            assert new_attribute._pointer() != extensions.ffi.NULL
            batch.add_item(
                extensions.with_unique_batch_item_id(request_item, _ADD_ATTRIBUTE_ID)
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        expected_new_attribute = ttlv_structure(
            0x0042_013D,
            ttlv_structure(
                0x0042_000A,
                ttlv_item(0x0042_00A5, 7, b"new name"),
                ttlv_enumeration(0x0042_00A6, 1),
            ),
        )
        assert failures.empty(), "the mTLS peer must complete one Add Attribute exchange"
        assert len(received) == 1
        assert ttlv_enumeration(0x42005C, 0x0D) in received[0]
        assert expected_new_attribute in received[0]
        assert b"\x42\x00\x94" not in received[0], "omitted Unique Identifier must stay absent"
        assert batch_result is not None and len(batch_result.items) == 1
        response = batch_result.items[0].add_attribute_response
        assert response is not None
        assert response.unique_identifier == "object-7"
    finally:
        if request_item is not None:
            request_item.close()
        if new_attribute is not None:
            new_attribute.close()
        if batch_result is not None:
            batch_result.close()
        limits.close()
        for handle in (type_tag, raw_type_tag, value_tag, raw_value_tag, name_tag, raw_name_tag):
            handle.close()
        peer.join(timeout=4)


def test_adjust_attribute_preserves_reference_value_and_maps_typed_response():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_adjust_attribute_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    request_item = None
    batch_result = None
    adjustment_value = ttlv.ttlv_value_integer(5)
    limits = ttlv.codec_limits_defaults()
    try:
        with (
            configuration,
            client.Client.create(configuration, mtls_transport_config(port, _TLS_FIXTURES)) as kmip,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_adjust_attribute(
                None,
                extensions.AttributeReference.name("KMIPKit", "Counter"),
                1,
                adjustment_value,
            )
            adjustment_value.close()
            batch.add_item(
                extensions.with_unique_batch_item_id(request_item, _ADJUST_ATTRIBUTE_ID)
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        assert failures.empty(), "the mTLS peer must complete one Adjust Attribute exchange"
        assert len(received) == 1
        assert ttlv_enumeration(0x42005C, 0x30) in received[0]
        assert ttlv_structure(
            0x0042_013B,
            ttlv_item(0x0042_009D, 7, b"KMIPKit"),
            ttlv_item(0x0042_000A, 7, b"Counter"),
        ) in received[0]
        assert ttlv_enumeration(0x0042_0158, 1) in received[0]
        assert ttlv_item(0x0042_0162, 2, b"\x00\x00\x00\x05") in received[0]
        assert b"\x42\x00\x94" not in received[0], "omitted Unique Identifier must stay absent"
        assert batch_result is not None and len(batch_result.items) == 1
        response = batch_result.items[0].adjust_attribute_response
        assert response is not None
        assert response.unique_identifier == "adjusted-object-1"
    finally:
        if request_item is not None:
            request_item.close()
        if not adjustment_value.closed:
            adjustment_value.close()
        if batch_result is not None:
            batch_result.close()
        limits.close()
        peer.join(timeout=4)
