from __future__ import annotations

import ssl
from pathlib import Path
import threading
from types import SimpleNamespace

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
from kmipkit._client_ffi import ffi

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_GET_ATTRIBUTE_LIST_ID = b"get-attribute-list-0"


def _get_attribute_list_response_message() -> bytes:
    return operation_response_message(
        0x0C,
        _GET_ATTRIBUTE_LIST_ID,
        payload_items=(
            ttlv_item(0x0042_0094, 7, b"object-7"),
            ttlv_enumeration(0x0042_013B, 0x0054_0001),
            ttlv_structure(
                0x0042_013B,
                ttlv_item(0x0042_009D, 7, b"vendor"),
                ttlv_item(0x0042_000A, 7, b"Opaque.Name"),
            ),
            ttlv_structure(
                0x0042_013B,
                ttlv_item(0x0042_009D, 7, b"vendor"),
                ttlv_item(0x0042_000A, 7, b"Opaque.Name"),
            ),
        ),
    )


def test_get_attribute_list_builder_preserves_optional_unique_identifier(monkeypatch):
    calls: list[tuple[str, tuple[object, ...]]] = []
    identifier_handle = object()
    monkeypatch.setattr(
        extensions,
        "_native_unique_identifier",
        lambda value, _staging: (ffi.NULL if value is None else identifier_handle, None),
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    omitted_result = extensions.client_batch_item_get_attribute_list(None)
    present_result = extensions.client_batch_item_get_attribute_list(object())

    assert omitted_result == present_result == 73
    assert calls == [
        ("kmipkit_client_batch_item_get_attribute_list", (ffi.NULL,)),
        ("kmipkit_client_batch_item_get_attribute_list", (identifier_handle,)),
    ]


def test_get_attribute_list_response_owns_generic_payload_and_redacts_repr(monkeypatch):
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)
    identifier_bytes = ffi.new("uint8_t[]", b"object-7")
    vendor_bytes = ffi.new("uint8_t[]", b"vendor")
    name_bytes = ffi.new("uint8_t[]", b"Opaque.Name")

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()

    def response_ttlv(_item, present, output):
        present[0] = 1
        output[0] = native_handle
        return 0

    def typed_response(_item, present, identifier, references, capacity, count):
        present[0] = 1
        identifier.kind = 1
        identifier.text_data = identifier_bytes
        identifier.text_length = 8
        count[0] = 3
        if references != ffi.NULL:
            assert capacity == 3
            references[0].kind = 1
            references[0].tag_value = 0x0054_0001
            for index in (1, 2):
                references[index].kind = 2
                references[index].vendor_identification_data = vendor_bytes
                references[index].vendor_identification_length = 6
                references[index].attribute_name_data = name_bytes
                references[index].attribute_name_length = 11
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_attribute_list=typed_response,
            kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv=response_ttlv
        ),
    )
    monkeypatch.setattr(
        ttlv, "_structure_from_native_address", lambda _address: structure
    )

    result = client._get_attribute_list_response(
        SimpleNamespace(_pointer=lambda: object())
    )

    assert isinstance(result, client.GetAttributeListResponse)
    assert result.response_payload_ttlv is structure
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("object-7")
    assert result.attribute_references == (
        extensions.AttributeReference.tag(0x0054_0001),
        extensions.AttributeReference.name("vendor", "Opaque.Name"),
        extensions.AttributeReference.name("vendor", "Opaque.Name"),
    )
    assert "123" not in repr(result)
    result.close()
    assert structure.closed


def test_get_attribute_list_executes_once_over_native_mtls_client():
    received: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        received.append(read_ttlv_frame(stream))
        stream.sendall(_get_attribute_list_response_message())

    port, failures, peer = start_tls13_mtls_peer(_TLS_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    response = None
    request_item = None
    batch_result = None
    try:
        with (
            configuration,
            client.Client.create(configuration, mtls_transport_config(port, _TLS_FIXTURES)) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            request_item = extensions.client_batch_item_get_attribute_list(
                extensions.UniqueIdentifier.text_string("object-7")
            )
            batch.add_item(
                extensions.with_unique_batch_item_id(request_item, _GET_ATTRIBUTE_LIST_ID)
            )
            request_item = None
            batch_result = kmip.execute(batch, limits)

        assert failures.empty(), "the mTLS peer must complete one Get Attribute List exchange"
        assert len(received) == 1
        assert ttlv_enumeration(0x42005C, 0x0C) in received[0]
        assert batch_result is not None and len(batch_result.items) == 1
        response = batch_result.items[0].get_attribute_list_response
        assert isinstance(response, client.GetAttributeListResponse)
        assert response.unique_identifier == extensions.UniqueIdentifier.text_string("object-7")
        assert len(response.attribute_references) == 3
        assert response.attribute_references[0] == extensions.AttributeReference.tag(0x0054_0001)
        assert response.attribute_references[1:] == (
            extensions.AttributeReference.name("vendor", "Opaque.Name"),
            extensions.AttributeReference.name("vendor", "Opaque.Name"),
        )
        with ttlv.ttlv_structure_view(response.response_payload_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
        batch_result.close()
        assert response.attribute_references[-1].attribute_name == "Opaque.Name"
    finally:
        if request_item is not None:
            request_item.close()
        if response is not None:
            response.close()
        if batch_result is not None:
            batch_result.close()
        peer.join(timeout=4)
