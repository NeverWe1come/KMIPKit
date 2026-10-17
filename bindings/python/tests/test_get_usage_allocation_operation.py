from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi


def test_get_usage_allocation_builder_preserves_identifier_and_exact_signed_count(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    handle = extensions.client_batch_item_get_usage_allocation(
        extensions.UniqueIdentifier.text_string("allocation-object"), 9_223_372_036_854_775_807
    )

    assert handle == 23
    assert calls[0][0] == "kmipkit_client_batch_item_get_usage_allocation"
    assert calls[0][1][1] == 9_223_372_036_854_775_807


def test_get_usage_allocation_builder_allows_placeholder_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_get_usage_allocation(None, 0) == 23
    assert calls[0][1][0] == ffi.NULL
    assert calls[0][1][1] == 0


def test_get_usage_allocation_response_maps_identifier_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_copy = _usage_allocation_response_ttlv()
    native_handle, owner = native_copy._take()
    assert owner is None
    storage_refs: list[object] = []

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        encoded = b"allocation-object"
        storage = ffi.new("uint8_t[]", encoded)
        storage_refs.append(storage)
        identifier.text_data = storage  # type: ignore[attr-defined]
        identifier.text_length = len(encoded)  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ttlv_ffi.cast("uintptr_t", native_handle)),
        )  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_usage_allocation=response,
            kmipkit_client_batch_item_response_get_usage_allocation_ttlv=response_ttlv,
        ),
    )
    result = client._get_usage_allocation_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.GetUsageAllocationResponse)
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("allocation-object")
    assert "allocation-object" not in repr(result)
    result.close()


def test_get_usage_allocation_response_allows_absent_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def response(_item: object, present: object, _identifier: object) -> int:
        present[0] = 0  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_get_usage_allocation=response),
    )

    result = client._get_usage_allocation_response(SimpleNamespace(_pointer=lambda: object()))

    assert result is None


def test_get_usage_allocation_exposes_owned_complete_response_ttlv(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_copy = _usage_allocation_response_ttlv()
    native_handle, owner = native_copy._take()
    assert owner is None
    storage_refs: list[object] = []
    released_items: list[object] = []

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        encoded = b"allocation-object"
        storage = ffi.new("uint8_t[]", encoded)
        storage_refs.append(storage)
        identifier.text_data = storage  # type: ignore[attr-defined]
        identifier.text_length = len(encoded)  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ttlv_ffi.cast("uintptr_t", native_handle)),
        )  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_item_response_get_usage_allocation=response,
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv=response_ttlv,
        kmipkit_client_batch_item_response_release=lambda item: released_items.append(item),
    )
    monkeypatch.setattr(client, "lib", fake)
    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 2)
    item = client._wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)

    result = client._get_usage_allocation_response(item)
    item.close()

    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("allocation-object")
    assert released_items == [item_pointer]
    assert result.response_ttlv.closed is False
    with ttlv.ttlv_structure_view(result.response_ttlv) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 4
        assert [_tag_at(view, index) for index in range(4)] == [
            0x0042_005C,
            0x0042_0093,
            0x0042_007F,
            0x0042_007C,
        ]
        payload_item = ttlv.ttlv_structure_view_item_at(view, 3)
        payload_value = ttlv.ttlv_item_view_value(payload_item)
        payload = ttlv.ttlv_value_view_structure(payload_value)
        try:
            assert [_tag_at(payload, index) for index in range(2)] == [
                0x0042_0094,
                0x0054_0001,
            ]
        finally:
            payload.close()
            payload_value.close()
            payload_item.close()

    result.close()
    assert result.response_ttlv.closed


def _usage_allocation_response_ttlv() -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()

    def item(tag_value: int, value: ttlv.Value) -> ttlv.TtlvItem:
        raw_tag = ttlv.raw_tag_from_raw(tag_value)
        tag = ttlv.raw_tag_try_checked(raw_tag)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            raw_tag.close()

    def enumeration_item(tag_value: int, value: int) -> ttlv.TtlvItem:
        return item(tag_value, ttlv.ttlv_value_enumeration(value))

    def bytes_item(tag_value: int, value: bytes) -> ttlv.TtlvItem:
        return item(tag_value, ttlv.ttlv_value_byte_string(value, limits))

    def text_item(tag_value: int, value: str) -> ttlv.TtlvItem:
        return item(tag_value, ttlv.ttlv_value_text_string(value.encode(), limits))

    def structure_item(tag_value: int, fields: tuple[ttlv.TtlvItem, ...]) -> ttlv.TtlvItem:
        structure = ttlv.ttlv_structure_create()
        for field in fields:
            structure = ttlv.ttlv_structure_with_item(structure, field, limits)
        return item(tag_value, ttlv.ttlv_value_structure(structure, limits))

    try:
        payload = structure_item(
            0x0042_007C,
            (
                text_item(0x0042_0094, "allocation-object"),
                bytes_item(0x0054_0001, b"\xde\xad\x00\xff"),
            ),
        )
        response = ttlv.ttlv_structure_create()
        for field in (
            enumeration_item(0x0042_005C, 0x11),
            bytes_item(0x0042_0093, b"item-0"),
            enumeration_item(0x0042_007F, 0),
            payload,
        ):
            response = ttlv.ttlv_structure_with_item(response, field, limits)
        return response
    finally:
        limits.close()


def _tag_at(view: ttlv.TtlvStructureView, index: int) -> int:
    item = ttlv.ttlv_structure_view_item_at(view, index)
    tag = ttlv.ttlv_item_view_tag(item)
    try:
        return ttlv.tag_value(tag)
    finally:
        tag.close()
        item.close()


@pytest.mark.parametrize(("status", "expects_response"), ((0, True), (1, False)))
def test_batch_result_maps_get_usage_allocation_only_on_success(
    monkeypatch: pytest.MonkeyPatch, status: int, expects_response: bool
) -> None:
    response_pointer = ffi.cast("kmipkit_client_batch_response_t *", 1)
    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 2)
    calls: list[str] = []
    native_handle = ffi.NULL
    if expects_response:
        native_copy = _usage_allocation_response_ttlv()
        native_handle, owner = native_copy._take()
        assert owner is None

    def metadata(_item: object, output: object) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")  # type: ignore[attr-defined]
        output.abi_version = 2  # type: ignore[attr-defined]
        output.reserved = 0  # type: ignore[attr-defined]
        output.operation = 0x11  # type: ignore[attr-defined]
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        output.result_status = status  # type: ignore[attr-defined]
        output.has_result_reason = 0  # type: ignore[attr-defined]
        output.result_reason = 0  # type: ignore[attr-defined]
        output.has_result_message = 0  # type: ignore[attr-defined]
        output.result_message_data = ffi.NULL  # type: ignore[attr-defined]
        output.result_message_length = 0  # type: ignore[attr-defined]
        return 0

    def allocation_response(_item: object, present: object, identifier: object) -> int:
        calls.append("allocation")
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.INTEGER)  # type: ignore[attr-defined]
        identifier.integer_value = -17  # type: ignore[attr-defined]
        return 0

    def allocation_ttlv_response(_item: object, present: object, output: object) -> int:
        calls.append("allocation-ttlv")
        present[0] = 1  # type: ignore[index]
        output[0] = ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ttlv_ffi.cast("uintptr_t", native_handle)),
        )  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: None,
        kmipkit_client_batch_item_response_release=lambda _ptr: None,
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, output: output.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda _item, present, span: (
            present.__setitem__(0, 0) or setattr(span, "data", ffi.NULL) or setattr(span, "length", 0) or 0
        ),
        kmipkit_client_batch_item_response_pending_correlation_value=lambda _item, present, span: (
            present.__setitem__(0, 0) or setattr(span, "data", ffi.NULL) or setattr(span, "length", 0) or 0
        ),
        kmipkit_client_batch_item_response_discover_versions_supported_versions=lambda _item, present, _values, _capacity, count: (
            present.__setitem__(0, 0) or count.__setitem__(0, 0) or 0
        ),
        kmipkit_client_batch_item_response_get_usage_allocation=allocation_response,
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv=allocation_ttlv_response,
        **{
            "kmipkit_client_batch_item_response_create": lambda _item, present, object_type, _identifier: (
                present.__setitem__(0, 0) or object_type.__setitem__(0, 0) or 0
            ),
            "kmipkit_client_batch_item_response_create_key_pair": lambda _item, present, _private, _public: (
                present.__setitem__(0, 0) or 0
            ),
            "kmipkit_client_batch_item_response_create_split_key_count": lambda _item, present, count: (
                present.__setitem__(0, 0) or count.__setitem__(0, 0) or 0
            ),
        },
    )
    monkeypatch.setattr(client, "lib", fake)

    result = client._batch_result(response_pointer)

    allocation = result.items[0].get_usage_allocation_response
    assert (allocation is not None) is expects_response
    if expects_response:
        assert allocation.unique_identifier == extensions.UniqueIdentifier.integer(-17)
        assert "-17" not in repr(result.items[0])
        allocation.close()
    assert calls == (["allocation", "allocation-ttlv"] if expects_response else [])
    result.close()
