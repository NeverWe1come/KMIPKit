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

from kmipkit import client, extensions, ttlv

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_MODIFY_ATTRIBUTE_ID = b"modify-attribute-0"


def test_modify_attribute_builder_preserves_optional_current_and_required_new(
    monkeypatch,
):
    calls: list[tuple[str, tuple[object, ...], tuple[int, ...]]] = []
    identifier = object()
    current_handle = object()
    new_handle = object()
    current_attribute = object.__new__(ttlv.TtlvItem)
    new_attribute = object.__new__(ttlv.TtlvItem)

    monkeypatch.setattr(
        extensions,
        "_client_native_unique_identifier",
        lambda value, _staging: identifier if value is not None else None,
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_item",
        lambda value: current_handle if value is current_attribute else new_handle,
        raising=False,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments, *, consumed_indices=():
            calls.append((symbol, arguments, consumed_indices)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    omitted = extensions.client_batch_item_modify_attribute(
        None, None, new_attribute
    )
    selected = extensions.client_batch_item_modify_attribute(
        extensions.UniqueIdentifier.text_string("object-7"),
        current_attribute,
        new_attribute,
    )

    assert omitted == selected == 73
    assert calls == [
        (
            "kmipkit_client_batch_item_modify_attribute",
            (None, None, new_handle),
            (),
        ),
        (
            "kmipkit_client_batch_item_modify_attribute",
            (identifier, current_handle, new_handle),
            (),
        ),
    ]


def test_modify_attribute_executes_once_and_maps_typed_response_over_mtls():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(
            operation_response_message(
                0x0E,
                _MODIFY_ATTRIBUTE_ID,
                payload_items=(ttlv_item(0x0042_0094, 7, b"object-7"),),
            )
        )

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    request_item = None
    batch_result = None
    limits = ttlv.codec_limits_defaults()
    raw_value_tag = ttlv.raw_tag_from_raw(0x0042_00C0)
    value_tag = ttlv.raw_tag_try_checked(raw_value_tag)
    raw_type_tag = ttlv.raw_tag_from_raw(0x0042_00C1)
    type_tag = ttlv.raw_tag_try_checked(raw_type_tag)
    raw_attribute_tag = ttlv.raw_tag_from_raw(0x0042_00BF)
    attribute_tag = ttlv.raw_tag_try_checked(raw_attribute_tag)
    attributes: list[ttlv.TtlvItem] = []
    try:
        for value in (b"old value", b"new value"):
            attribute_value = ttlv.ttlv_item_create(
                value_tag, ttlv.ttlv_value_text_string(value, limits), limits
            )
            attribute_type = ttlv.ttlv_item_create(
                type_tag, ttlv.ttlv_value_enumeration(1), limits
            )
            value_structure = ttlv.ttlv_structure_with_item(
                ttlv.ttlv_structure_create(), attribute_value, limits
            )
            value_structure = ttlv.ttlv_structure_with_item(
                value_structure, attribute_type, limits
            )
            attributes.append(
                ttlv.ttlv_item_create(
                    attribute_tag,
                    ttlv.ttlv_value_structure(value_structure, limits),
                    limits,
                )
            )

        current_attribute, new_attribute = attributes
        with (
            configuration,
            client.Client.create(
                configuration, mtls_transport_config(port, _TLS_FIXTURES)
            ) as kmip,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_modify_attribute(
                extensions.UniqueIdentifier.text_string("object-7"),
                current_attribute,
                new_attribute,
            )
            current_attribute.close()
            new_attribute.close()
            batch.add_item(
                extensions.with_unique_batch_item_id(
                    request_item, _MODIFY_ATTRIBUTE_ID
                )
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        assert failures.empty(), "the mTLS peer must complete one Modify Attribute exchange"
        assert len(received) == 1
        assert ttlv_enumeration(0x42005C, 0x0E) in received[0]
        assert ttlv_structure(
            0x0042_013C,
            ttlv_structure(
                0x0042_00BF,
                ttlv_item(0x0042_00C0, 7, b"old value"),
                ttlv_enumeration(0x0042_00C1, 1),
            ),
        ) in received[0]
        assert ttlv_structure(
            0x0042_013D,
            ttlv_structure(
                0x0042_00BF,
                ttlv_item(0x0042_00C0, 7, b"new value"),
                ttlv_enumeration(0x0042_00C1, 1),
            ),
        ) in received[0]
        assert batch_result is not None and len(batch_result.items) == 1
        response = batch_result.items[0].modify_attribute_response
        assert response is not None
        assert response.unique_identifier == "object-7"
        assert "object-7" not in repr(response)
    finally:
        if request_item is not None:
            request_item.close()
        for attribute in attributes:
            attribute.close()
        if batch_result is not None:
            batch_result.close()
        limits.close()
        for handle in (
            attribute_tag,
            raw_attribute_tag,
            type_tag,
            raw_type_tag,
            value_tag,
            raw_value_tag,
        ):
            handle.close()
        peer.join(timeout=4)
