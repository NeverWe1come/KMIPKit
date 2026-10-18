"""Binding checks for OASIS KMIP v2.1 §§6.1.19 (Get) and 6.1.28 (Locate)."""

from __future__ import annotations

import sys
from types import SimpleNamespace

import pytest

from kmipkit._client_ffi import ffi
from kmipkit import client, extensions, ttlv
from kmipkit.errors import InvalidInputError, ResourceLimitError


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


@pytest.mark.parametrize(
    ("present", "payload_present", "object_type"),
    [
        pytest.param(0, 0, 0, id="absent"),
        pytest.param(1, 0, 0, id="failed"),
    ],
)
def test_get_response_accepts_absent_and_payloadless_outcomes(
    monkeypatch, present, payload_present, object_type
) -> None:
    def response(_item, out_present, out_payload, out_type, _identifier, out_object):
        out_present[0] = present
        out_payload[0] = payload_present
        out_type[0] = object_type
        out_object[0] = ffi.NULL
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get=response,
            kmipkit_ttlv_structure_release=lambda _handle: None,
        ),
    )
    result = client._get_response(SimpleNamespace(_pointer=lambda: object()))
    if present == 0:
        assert result is None
    else:
        assert result is not None
        assert result.object is None


@pytest.mark.parametrize(
    ("present", "payload_present", "object_type", "identifier", "handle"),
    [
        pytest.param(2, 0, 0, None, True, id="invalid-present-flag-releases-owned-object"),
        pytest.param(1, 2, 0, None, False, id="invalid-payload-flag"),
        pytest.param(1, 2, 0, None, True, id="invalid-payload-flag-releases-owned-object"),
        pytest.param(0, 0, 7, None, False, id="absent-with-object-type"),
        pytest.param(0, 0, 0, None, True, id="absent-with-owned-object"),
        pytest.param(1, 0, 0, b"unexpected", False, id="failed-with-identifier"),
        pytest.param(1, 0, 0, None, True, id="failed-with-owned-object"),
        pytest.param(1, 1, 7, b"identifier", False, id="payload-without-object"),
        pytest.param(1, 1, 7, None, True, id="payload-without-identifier"),
    ],
)
def test_get_response_rejects_inconsistent_native_fields_and_releases_owned_object(
    monkeypatch, present, payload_present, object_type, identifier, handle
) -> None:
    owned = ffi.cast("kmipkit_ttlv_structure_t *", 61) if handle else ffi.NULL
    released: list[object] = []
    identifier_storage: list[object] = []

    def response(_item, out_present, out_payload, out_type, out_identifier, out_object):
        out_present[0] = present
        out_payload[0] = payload_present
        out_type[0] = object_type
        if identifier is not None:
            _fill_text_identifier(out_identifier, identifier, identifier_storage)
        out_object[0] = owned
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get=response,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    with pytest.raises(InvalidInputError):
        client._get_response(SimpleNamespace(_pointer=lambda: object()))
    assert released == ([owned] if handle else [])


def test_get_response_releases_object_when_wrapper_creation_fails(monkeypatch) -> None:
    owned = ffi.cast("kmipkit_ttlv_structure_t *", 61)
    released: list[object] = []
    identifier_bytes = ffi.new("uint8_t[]", b"object-id")

    def response(_item, present, payload, object_type, identifier, object_output):
        present[0] = 1
        payload[0] = 1
        object_type[0] = 1
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        identifier.text_data = identifier_bytes
        identifier.text_length = len(b"object-id")
        object_output[0] = owned
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get=response,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("wrapper rejected handle")),
    )
    with pytest.raises(RuntimeError, match="wrapper rejected handle"):
        client._get_response(SimpleNamespace(_pointer=lambda: object()))
    assert released == [owned]


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
    formatted = repr(result)
    assert "first" not in formatted
    assert "second" not in formatted
    assert formatted.count("value=redacted") == 3
    assert calls == [0, 1, 2]


@pytest.mark.parametrize(
    ("present", "payload_present", "located_present", "located_items", "count", "expected"),
    [
        pytest.param(0, 0, 0, 0, 0, None, id="absent"),
        pytest.param(1, 0, 0, 0, 0, (), id="failed"),
        pytest.param(1, 1, 0, 0, 0, (), id="empty-success"),
    ],
)
def test_locate_response_accepts_absent_failed_and_empty_payloads(
    monkeypatch, present, payload_present, located_present, located_items, count, expected
) -> None:
    def response(_item, out_present, out_payload, out_located_present, out_located, out_count):
        out_present[0] = present
        out_payload[0] = payload_present
        out_located_present[0] = located_present
        out_located[0] = located_items
        out_count[0] = count
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_locate=response),
    )
    result = client._locate_response(SimpleNamespace(_pointer=lambda: object()))
    if expected is None:
        assert result is None
    else:
        assert result.unique_identifiers == expected


@pytest.mark.parametrize(
    ("present", "payload_present", "located_present", "located_items", "count"),
    [
        pytest.param(2, 0, 0, 0, 0, id="invalid-present-flag"),
        pytest.param(1, 2, 0, 0, 0, id="invalid-payload-flag"),
        pytest.param(1, 1, 0, 4, 0, id="omitted-located-items-with-value"),
        pytest.param(0, 0, 1, 0, 0, id="absent-with-located-items"),
        pytest.param(1, 0, 0, 0, 1, id="failed-with-identifiers"),
    ],
)
def test_locate_response_rejects_inconsistent_native_fields(
    monkeypatch, present, payload_present, located_present, located_items, count
) -> None:
    def response(_item, out_present, out_payload, out_located_present, out_located, out_count):
        out_present[0] = present
        out_payload[0] = payload_present
        out_located_present[0] = located_present
        out_located[0] = located_items
        out_count[0] = count
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_locate=response),
    )
    with pytest.raises(InvalidInputError):
        client._locate_response(SimpleNamespace(_pointer=lambda: object()))


def test_locate_response_rejects_counts_outside_python_sequence_range(monkeypatch) -> None:
    def response(_item, present, payload, located_present, located_items, count):
        present[0] = 1
        payload[0] = 1
        located_present[0] = 0
        located_items[0] = 0
        count[0] = sys.maxsize + 1
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_locate=response),
    )
    with pytest.raises(ResourceLimitError):
        client._locate_response(SimpleNamespace(_pointer=lambda: object()))


def test_locate_response_rejects_missing_identifier_values(monkeypatch) -> None:
    def response(_item, present, payload, located_present, located_items, count):
        present[0] = 1
        payload[0] = 1
        located_present[0] = 0
        located_items[0] = 0
        count[0] = 1
        return 0

    def identifier_at(_item, _index, output):
        output.kind = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_locate=response,
            kmipkit_client_batch_item_response_locate_unique_identifier_at=identifier_at,
        ),
    )
    with pytest.raises(InvalidInputError):
        client._locate_response(SimpleNamespace(_pointer=lambda: object()))


def test_locate_response_preserves_identifiers_beyond_default_decoder_limit(monkeypatch) -> None:
    identifier_count = 100_001
    identifier_bytes = ffi.new("uint8_t[]", b"repeated")
    call_count = 0

    def response(_item, present, payload_present, located_items_present, located_items, count):
        present[0] = 1
        payload_present[0] = 1
        located_items_present[0] = 1
        located_items[0] = identifier_count
        count[0] = identifier_count
        return 0

    def identifier_at(_item, _index, output):
        nonlocal call_count
        call_count += 1
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = identifier_bytes
        output.text_length = len(b"repeated")
        output.enumeration_value = 0
        output.integer_value = 0
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
    assert len(result.unique_identifiers) == identifier_count
    assert result.unique_identifiers[0].text == "repeated"
    assert result.unique_identifiers[-1].text == "repeated"
    assert call_count == identifier_count
