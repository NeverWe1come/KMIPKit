"""Binding checks for OASIS KMIP v2.1 §§6.1.19 (Get) and 6.1.28 (Locate)."""

from __future__ import annotations

from types import SimpleNamespace

from kmipkit._client_ffi import ffi
from kmipkit import client, extensions, ttlv


def _fill_text_identifier(output, value: bytes, storage: list[object]) -> None:
    text = ffi.new("uint8_t[]", value)
    storage.append(text)
    output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
    output.text_data = text
    output.text_length = len(value)
    output.enumeration_value = 0
    output.integer_value = 0


def test_get_builder_preserves_optional_raw_table_220_selectors(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    with ttlv.ttlv_structure_create() as wrapping_specification:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is wrapping_specification else 0,
        )
        result = extensions.client_batch_item_get(
            extensions.UniqueIdentifier.text_string("get-key"),
            0xF1234567,
            0xF1234568,
            0xF1234569,
            wrapping_specification,
        )

    assert result == 23
    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_get"
    assert arguments[1:] == (1, 0xF1234567, 1, 0xF1234568, 1, 0xF1234569, 17)


def test_locate_builder_preserves_zero_unknown_bits_and_required_empty_attributes(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    with ttlv.ttlv_structure_create() as attributes:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is attributes else 0,
        )
        result = extensions.client_batch_item_locate(
            attributes,
            maximum_items=-4,
            offset_items=0,
            storage_status_mask=0x80000001,
            object_group_member=0xF1234567,
        )

    assert result == 23
    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_locate"
    assert arguments == (1, -4, 1, 0, 1, 0x80000001, 1, 0xF1234567, 17)


def test_get_response_owns_object_and_redacts_its_formatting(monkeypatch) -> None:
    object_handle = ffi.cast("kmipkit_ttlv_structure_t *", 51)
    identifier_bytes = ffi.new("uint8_t[]", b"get-secret-id")

    def response(_item, present, payload_present, object_type, identifier, any_object):
        present[0] = 1
        payload_present[0] = 1
        object_type[0] = 0xF1234567
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        identifier.text_data = identifier_bytes
        identifier.text_length = len(b"get-secret-id")
        any_object[0] = object_handle
        return 0

    wrapped: list[object] = []

    class OwnedStructure:
        def __init__(self, address):
            self.address = address
            self.closed = False
            wrapped.append(self)

        def close(self):
            self.closed = True

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get=response,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", OwnedStructure)

    result = client._get_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.GetResponse)
    assert result.object_type_raw == 0xF1234567
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("get-secret-id")
    assert result.object.address == 51
    assert "get-secret-id" not in repr(result)
    assert "get-secret-id" not in repr(result.object)
    result.close()
    assert all(value.closed for value in wrapped)


def test_locate_response_preserves_repeated_identifier_order(monkeypatch) -> None:
    identifiers = [b"first", b"second", b"first"]
    storage: list[object] = []
    calls: list[int] = []

    def response(_item, present, payload_present, located_items_present, located_items, count):
        present[0] = 1
        payload_present[0] = 1
        located_items_present[0] = 1
        located_items[0] = 3
        count[0] = len(identifiers)
        return 0

    def identifier_at(_item, index, output):
        calls.append(int(index))
        _fill_text_identifier(output[0], identifiers[int(index)], storage)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_locate=response,
            kmipkit_client_batch_item_response_locate_unique_identifier_at=identifier_at,
        ),
    )

    result = client._locate_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.LocateResponse)
    assert result.located_items == 3
    assert [identifier.text for identifier in result.unique_identifiers] == [
        "first",
        "second",
        "first",
    ]
    assert calls == [0, 1, 2]
