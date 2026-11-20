from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit._client_ffi import ffi
from kmipkit import client, extensions, ttlv


def test_export_builder_preserves_optional_raw_table_217_selectors(
    monkeypatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    identifier_snapshots: list[bytes] = []

    def make_handle(_kind, symbol, arguments):
        identifier = arguments[0]
        identifier_snapshots.append(
            bytes(ffi.buffer(identifier.text_data, identifier.text_length))
        )
        calls.append((symbol, arguments))
        return 23

    monkeypatch.setattr(
        extensions,
        "_make_handle",
        make_handle,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as wrapping_specification:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is wrapping_specification else 0,
        )
        result = extensions.client_batch_item_export(
            extensions.UniqueIdentifier.text_string("exported-key"),
            0xF123_4567,
            0xF123_4568,
            0xF123_4569,
            wrapping_specification,
        )

    assert result == 23
    assert len(calls) == 1
    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_export"
    assert arguments[1:] == (1, 0xF123_4567, 1, 0xF123_4568, 1, 0xF123_4569, 17)
    identifier = arguments[0]
    assert identifier.kind == int(extensions.UniqueIdentifierKind.TEXT_STRING)
    assert identifier_snapshots == [b"exported-key"]


def test_export_builder_keeps_every_table_217_option_omitted(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    monkeypatch.setattr(extensions, "_native_structure", lambda *_args, **_kwargs: 0)

    assert extensions.client_batch_item_export(None, None, None, None, None) == 23

    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_export"
    assert arguments[0] == ffi.NULL
    assert arguments[1:] == (0, 0, 0, 0, 0, 0, 0)


def test_export_response_repr_redacts_identifiers_and_sensitive_ttlv() -> None:
    response_ttlv = ttlv.ttlv_structure_create()
    attributes = ttlv.ttlv_structure_create()
    any_object = ttlv.ttlv_structure_create()
    response = client.ExportResponse(
        object_type_raw=0xF123_4567,
        unique_identifier=extensions.UniqueIdentifier.text_string("export-secret-id"),
        attributes=attributes,
        any_object=any_object,
        response_ttlv=response_ttlv,
    )

    assert response.object_type_raw == 0xF123_4567
    assert "export-secret-id" not in repr(response)
    assert "ExportResponse" in repr(response)
    response.close()
    assert response_ttlv.closed
    assert attributes.closed
    assert any_object.closed


def test_export_failure_remains_a_typed_result_with_owned_ordered_ttlv(monkeypatch) -> None:
    response_handle = ffi.cast("kmipkit_ttlv_structure_t *", 41)
    wrapped: list[object] = []
    released: list[object] = []

    def response(_item, present, payload_present, object_type, identifier, attributes, any_object):
        present[0] = 1
        payload_present[0] = 0
        object_type[0] = 0
        identifier.kind = 0
        identifier.text_data = ffi.NULL
        identifier.text_length = 0
        identifier.enumeration_value = 0
        identifier.integer_value = 0
        attributes[0] = ffi.NULL
        any_object[0] = ffi.NULL
        return 0

    def response_ttlv(_item, present, output):
        present[0] = 1
        output[0] = response_handle
        return 0

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
            kmipkit_client_batch_item_response_export=response,
            kmipkit_client_batch_item_response_export_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", OwnedStructure)

    result = client._export_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.ExportResponse)
    assert result.object_type_raw is None
    assert result.unique_identifier is None
    assert result.attributes is None
    assert result.any_object is None
    assert result.response_ttlv.address > 0
    assert len(wrapped) == 1
    assert "ExportResponse" in repr(result)
    result.close()
    assert result.response_ttlv.closed
    assert released == []


def test_export_success_maps_raw_fields_and_transfers_all_owned_ttlv(monkeypatch) -> None:
    attributes_handle = ffi.cast("kmipkit_ttlv_structure_t *", 51)
    any_object_handle = ffi.cast("kmipkit_ttlv_structure_t *", 52)
    response_handle = ffi.cast("kmipkit_ttlv_structure_t *", 53)
    identifier_bytes = ffi.new("uint8_t[]", b"exported-key")

    def response(_item, present, payload_present, object_type, identifier, attributes, any_object):
        present[0] = 1
        payload_present[0] = 1
        object_type[0] = 0xF123_4567
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        identifier.text_data = identifier_bytes
        identifier.text_length = len(b"exported-key")
        attributes[0] = attributes_handle
        any_object[0] = any_object_handle
        return 0

    def response_ttlv(_item, present, output):
        present[0] = 1
        output[0] = response_handle
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
            kmipkit_client_batch_item_response_export=response,
            kmipkit_client_batch_item_response_export_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", OwnedStructure)

    result = client._export_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.ExportResponse)
    assert result.object_type_raw == 0xF123_4567
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("exported-key")
    assert result.attributes.address == 51
    assert result.any_object.address == 52
    assert result.response_ttlv.address == 53
    assert len(wrapped) == 3
    result.close()
    assert all(value.closed for value in wrapped)


def test_export_absent_response_returns_none_without_native_handles(monkeypatch) -> None:
    def response(_item, present, payload_present, object_type, identifier, attributes, any_object):
        present[0] = 0
        payload_present[0] = 0
        object_type[0] = 0
        identifier.kind = 0
        identifier.text_data = ffi.NULL
        identifier.text_length = 0
        identifier.enumeration_value = 0
        identifier.integer_value = 0
        attributes[0] = ffi.NULL
        any_object[0] = ffi.NULL
        return 0

    def response_ttlv(_item, present, output):
        present[0] = 0
        output[0] = ffi.NULL
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_export=response,
            kmipkit_client_batch_item_response_export_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )

    assert client._export_response(SimpleNamespace(_pointer=lambda: object())) is None


def test_export_inconsistent_response_ttlv_releases_native_payload_handles(monkeypatch) -> None:
    attributes_handle = ffi.cast("kmipkit_ttlv_structure_t *", 61)
    any_object_handle = ffi.cast("kmipkit_ttlv_structure_t *", 62)
    released: list[object] = []

    def response(_item, present, payload_present, object_type, identifier, attributes, any_object):
        present[0] = 1
        payload_present[0] = 1
        object_type[0] = 7
        identifier.kind = int(extensions.UniqueIdentifierKind.INTEGER)
        identifier.integer_value = -17
        attributes[0] = attributes_handle
        any_object[0] = any_object_handle
        return 0

    def response_ttlv(_item, present, output):
        present[0] = 0
        output[0] = ffi.NULL
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_export=response,
            kmipkit_client_batch_item_response_export_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(client.errors.InvalidInputError):
        client._export_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [attributes_handle, any_object_handle]


@pytest.mark.parametrize(
    (
        "present_value",
        "payload_present_value",
        "response_ttlv_present_value",
        "object_type_value",
        "identifier_kind",
        "attributes_handle",
        "any_object_handle",
        "response_ttlv_handle",
    ),
    [
        (2, 0, 0, 0, 0, ffi.NULL, ffi.NULL, ffi.NULL),
        (0, 1, 0, 0, 0, ffi.NULL, ffi.NULL, ffi.NULL),
        (1, 0, 1, 0, 0, ffi.NULL, ffi.NULL, ffi.NULL),
        (1, 0, 1, 7, 0, ffi.NULL, ffi.NULL, 71),
        (1, 1, 1, 7, 1, ffi.NULL, 72, 73),
        (1, 1, 1, 7, 0, 74, 75, 76),
    ],
    ids=(
        "invalid-presence-flag",
        "absent-response-with-payload",
        "success-without-response-ttlv",
        "failure-with-object-type",
        "success-without-attributes",
        "success-without-identifier",
    ),
)
def test_export_response_rejects_inconsistent_native_payload_shapes(
    monkeypatch,
    present_value,
    payload_present_value,
    response_ttlv_present_value,
    object_type_value,
    identifier_kind,
    attributes_handle,
    any_object_handle,
    response_ttlv_handle,
) -> None:
    released: list[object] = []
    identifier_text = ffi.new("uint8_t[]", b"exported-key")

    def native_handle(value):
        return ffi.cast("kmipkit_ttlv_structure_t *", value) if isinstance(value, int) else value

    def response(_item, present, payload_present, object_type, identifier, attributes, any_object):
        present[0] = present_value
        payload_present[0] = payload_present_value
        object_type[0] = object_type_value
        identifier.kind = identifier_kind
        identifier.text_data = identifier_text if identifier_kind == 1 else ffi.NULL
        identifier.text_length = len(b"exported-key") if identifier_kind == 1 else 0
        identifier.enumeration_value = 0
        identifier.integer_value = 0
        attributes[0] = native_handle(attributes_handle)
        any_object[0] = native_handle(any_object_handle)
        return 0

    def response_ttlv(_item, present, output):
        present[0] = response_ttlv_present_value
        output[0] = native_handle(response_ttlv_handle)
        return 0

    def release(handle):
        released.append(handle)

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_export=response,
            kmipkit_client_batch_item_response_export_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=release,
        ),
    )

    with pytest.raises(client.errors.InvalidInputError):
        client._export_response(SimpleNamespace(_pointer=lambda: object()))

    expected_releases = [
        native_handle(handle)
        for handle in (attributes_handle, any_object_handle, response_ttlv_handle)
        if native_handle(handle) != ffi.NULL
    ]
    assert released == expected_releases


def test_export_response_closes_transferred_wrapper_when_later_conversion_fails(
    monkeypatch,
) -> None:
    attributes_handle = ffi.cast("kmipkit_ttlv_structure_t *", 81)
    any_object_handle = ffi.cast("kmipkit_ttlv_structure_t *", 82)
    response_handle = ffi.cast("kmipkit_ttlv_structure_t *", 83)
    identifier_text = ffi.new("uint8_t[]", b"exported-key")
    wrapped: list[object] = []
    released: list[object] = []
    conversions = 0

    def response(_item, present, payload_present, object_type, identifier, attributes, any_object):
        present[0] = 1
        payload_present[0] = 1
        object_type[0] = 7
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        identifier.text_data = identifier_text
        identifier.text_length = len(b"exported-key")
        attributes[0] = attributes_handle
        any_object[0] = any_object_handle
        return 0

    def response_ttlv(_item, present, output):
        present[0] = 1
        output[0] = response_handle
        return 0

    class OwnedStructure:
        def __init__(self, address):
            self.address = address
            self.closed = False
            wrapped.append(self)

        def close(self):
            self.closed = True

    def wrap_native(address):
        nonlocal conversions
        conversions += 1
        if conversions == 2:
            raise RuntimeError("injected wrapper conversion failure")
        return OwnedStructure(address)

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_export=response,
            kmipkit_client_batch_item_response_export_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", wrap_native)

    with pytest.raises(RuntimeError, match="injected wrapper conversion failure"):
        client._export_response(SimpleNamespace(_pointer=lambda: object()))

    assert len(wrapped) == 1
    assert wrapped[0].closed
    assert released == [any_object_handle, response_handle]
