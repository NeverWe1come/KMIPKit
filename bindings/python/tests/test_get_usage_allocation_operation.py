from __future__ import annotations

from types import SimpleNamespace

import pytest
from _test_support import batch_result_test_symbols
from kmipkit import client, errors, extensions, ttlv
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


def test_usage_allocation_context_cannot_be_created_from_a_copied_response_dto() -> None:
    response = client.GetUsageAllocationResponse(
        extensions.UniqueIdentifier.text_string("allocation-object"),
        _usage_allocation_response_ttlv(),
    )
    result = client.BatchItemResult(
        client.BatchOutcome.COMPLETED, 0, None, None, None,
        operation=0x11, get_usage_allocation_response=response,
    )

    with pytest.raises(errors.InvalidInputError):
        result.create_usage_allocation_context(5, 1)

    response.close()


def test_usage_allocation_context_uses_live_native_result_and_owns_its_state(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 22)
    context_pointer = ffi.cast("kmipkit_usage_allocation_context_t *", 23)
    result = client.BatchItemResult(
        client.BatchOutcome.COMPLETED, 0, None, None, None, operation=0x11
    )
    native_response = client._wrap_owned_handle(
        client._ClientBatchItemResponse, response_pointer
    )
    object.__setattr__(result, "_native_usage_allocation_response", native_response)
    batch_response_pointer = ffi.cast("kmipkit_client_batch_response_t *", 24)
    batch_result = client._wrap_owned_handle(
        client.BatchResult, batch_response_pointer, (result,)
    )
    storage = ffi.new("uint8_t[]", b"allocation-object")
    calls: list[tuple[object, int, int]] = []
    released: list[object] = []

    def create(item: object, requested: int, unit: int, output: object) -> int:
        calls.append((item, requested, unit))
        output[0] = context_pointer  # type: ignore[index]
        return 0

    def identifier(_context: object, output: object) -> int:
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        output.text_data = storage  # type: ignore[attr-defined]
        output.text_length = len(b"allocation-object")  # type: ignore[attr-defined]
        return 0

    def scalar(value: int):
        def read(_context: object, output: object) -> int:
            output[0] = value  # type: ignore[index]
            return 0

        return read

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_usage_allocation_context_create=create,
            kmipkit_usage_allocation_context_usage_limits_unit=scalar(0xFEDC_BA98),
            kmipkit_usage_allocation_context_allocated_units=scalar(9),
            kmipkit_usage_allocation_context_available_units=scalar(9),
            kmipkit_usage_allocation_context_reserved_units=scalar(0),
            kmipkit_usage_allocation_context_unique_identifier=identifier,
            kmipkit_usage_allocation_context_release=lambda value: released.append(value),
            kmipkit_client_batch_item_response_release=lambda value: released.append(value),
            kmipkit_client_batch_response_release=lambda value: released.append(value),
        ),
    )

    context = result.create_usage_allocation_context(9, 0xFEDC_BA98)
    assert calls == [(response_pointer, 9, 0xFEDC_BA98)]
    assert context.usage_limits_unit == 0xFEDC_BA98
    assert context.allocated_units == 9
    assert context.available_units == 9
    assert context.reserved_units == 0
    assert context.unique_identifier == extensions.UniqueIdentifier.text_string("allocation-object")
    batch_result.close()
    assert released == [response_pointer, batch_response_pointer]
    assert context.allocated_units == 9
    context.close()
    assert released == [response_pointer, batch_response_pointer, context_pointer]


def test_usage_allocation_context_rejects_null_handle_after_successful_create(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 27)
    native_response = client._wrap_owned_handle(
        client._ClientBatchItemResponse, response_pointer
    )
    result = client.BatchItemResult(
        client.BatchOutcome.COMPLETED, 0, None, None, None, operation=0x11
    )
    object.__setattr__(result, "_native_usage_allocation_response", native_response)
    released: list[object] = []
    calls: list[tuple[object, int, int]] = []

    def create(item: object, requested: int, unit: int, _output: object) -> int:
        calls.append((item, requested, unit))
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_usage_allocation_context_create=create,
            kmipkit_client_batch_item_response_release=lambda value: released.append(
                value
            ),
        ),
    )

    with pytest.raises(errors.KmipKitError):
        result.create_usage_allocation_context(5, 1)

    native_response.close()
    assert calls == [(response_pointer, 5, 1)]
    assert released == [response_pointer]


def test_batch_result_releases_retained_allocation_if_later_mapper_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer = client.ffi.cast("kmipkit_client_batch_response_t *", 31)
    item_pointers = (
        client.ffi.cast("kmipkit_client_batch_item_response_t *", 32),
        client.ffi.cast("kmipkit_client_batch_item_response_t *", 33),
    )
    addresses = tuple(
        int(client.ffi.cast("uintptr_t", pointer)) for pointer in item_pointers
    )
    released: list[tuple[str, int]] = []

    def metadata(item: object, output: object) -> int:
        item_address = int(client.ffi.cast("uintptr_t", item))
        output.struct_size = client.ffi.sizeof(  # type: ignore[attr-defined]
            "kmipkit_client_batch_item_metadata_v2_t"
        )
        output.abi_version = 2  # type: ignore[attr-defined]
        output.reserved = 0  # type: ignore[attr-defined]
        output.operation = 0x11 if item_address == addresses[0] else 0x18  # type: ignore[attr-defined]
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        output.result_status = 0  # type: ignore[attr-defined]
        output.has_result_reason = 0  # type: ignore[attr-defined]
        output.result_reason = 0  # type: ignore[attr-defined]
        output.has_result_message = 0  # type: ignore[attr-defined]
        output.result_message_data = client.ffi.NULL  # type: ignore[attr-defined]
        output.result_message_length = 0  # type: ignore[attr-defined]
        return 0

    def item_at(_batch: object, index: int, output: object) -> int:
        output[0] = item_pointers[index]  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_response_release=lambda value: released.append(
                ("batch", int(client.ffi.cast("uintptr_t", value)))
            ),
            kmipkit_client_batch_item_response_release=lambda value: released.append(
                ("item", int(client.ffi.cast("uintptr_t", value)))
            ),
            kmipkit_client_batch_response_item_count=lambda _batch, count: count.__setitem__(
                0, 2
            ) or 0,
            kmipkit_client_batch_response_item_at=item_at,
            kmipkit_client_batch_item_response_metadata_v2=metadata,
            kmipkit_client_batch_item_response_unique_batch_item_id=lambda *_args: 0,
            kmipkit_client_batch_item_response_pending_correlation_value=lambda *_args: 0,
        ),
    )
    monkeypatch.setattr(client, "_response_span", lambda *_args: None)
    monkeypatch.setattr(client, "_discover_versions_supported_versions", lambda _item: None)
    monkeypatch.setattr(client, "_create_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_key_pair_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_split_key_response", lambda _item: None)
    monkeypatch.setattr(client, "_get_usage_allocation_response", lambda _item: object())
    monkeypatch.setattr(
        client, "_join_split_key_response_for_result", lambda *_args, **_kwargs: None
    )
    monkeypatch.setattr(client, "_cryptographic_responses", lambda *_args: (None,) * 7)

    def fail_query_mapper(_item: object) -> None:
        raise RuntimeError("later query mapper failed")

    monkeypatch.setattr(client, "_query_response", fail_query_mapper)

    with pytest.raises(RuntimeError, match="later query mapper failed"):
        client._batch_result(batch_pointer)

    assert released == [
        ("item", addresses[1]),
        ("item", addresses[0]),
        ("batch", int(client.ffi.cast("uintptr_t", batch_pointer))),
    ]


@pytest.mark.parametrize(
    ("kind", "field", "value", "expected"),
    (
        pytest.param(
            extensions.UniqueIdentifierKind.ENUMERATION,
            "enumeration_value",
            7,
            extensions.UniqueIdentifier.enumeration(7),
            id="enumeration",
        ),
        pytest.param(
            extensions.UniqueIdentifierKind.INTEGER,
            "integer_value",
            -17,
            extensions.UniqueIdentifier.integer(-17),
            id="integer",
        ),
    ),
)
def test_usage_allocation_context_preserves_native_identifier_kind(
    monkeypatch: pytest.MonkeyPatch,
    kind: extensions.UniqueIdentifierKind,
    field: str,
    value: int,
    expected: extensions.UniqueIdentifier,
) -> None:
    pointer = ffi.cast("kmipkit_usage_allocation_context_t *", 25)
    context = client._wrap_owned_handle(client.UsageAllocationContext, pointer)
    released: list[object] = []

    def identifier(_context: object, output: object) -> int:
        output.kind = int(kind)  # type: ignore[attr-defined]
        setattr(output, field, value)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_usage_allocation_context_unique_identifier=identifier,
            kmipkit_usage_allocation_context_release=lambda handle: released.append(handle),
        ),
    )

    assert context.unique_identifier == expected
    context.close()
    assert released == [pointer]


@pytest.mark.parametrize(
    "malformation",
    ("text-too-large", "text-null-data", "text-invalid-utf8", "unknown-kind"),
)
def test_usage_allocation_context_rejects_malformed_native_identifiers(
    monkeypatch: pytest.MonkeyPatch, malformation: str
) -> None:
    pointer = ffi.cast("kmipkit_usage_allocation_context_t *", 26)
    context = client._wrap_owned_handle(client.UsageAllocationContext, pointer)
    storage_refs: list[object] = []

    def identifier(_context: object, output: object) -> int:
        if malformation == "unknown-kind":
            output.kind = 99  # type: ignore[attr-defined]
        else:
            output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
            if malformation == "text-too-large":
                output.text_length = client._MAX_INPUT_BYTES + 1  # type: ignore[attr-defined]
            elif malformation == "text-null-data":
                output.text_length = 1  # type: ignore[attr-defined]
            else:
                storage = ffi.new("uint8_t[]", b"\xff")
                storage_refs.append(storage)
                output.text_data = storage  # type: ignore[attr-defined]
                output.text_length = 1  # type: ignore[attr-defined]
        return 0

    released: list[object] = []
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_usage_allocation_context_unique_identifier=identifier,
            kmipkit_usage_allocation_context_release=lambda handle: released.append(handle),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        context.unique_identifier

    context.close()
    assert released == [pointer]


def test_native_unique_identifier_rejects_length_above_platform_limit() -> None:
    identifier = client._initialize_unique_identifier_output()
    identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
    identifier.text_length = client.sys.maxsize + 1

    with pytest.raises(errors.ResourceLimitError):
        client._read_unique_identifier(identifier)


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


@pytest.mark.parametrize(
    ("present", "identifier_kind"), ((0, 3), (2, 0), (1, 0))
)
def test_get_usage_allocation_rejects_inconsistent_response_fields(
    monkeypatch: pytest.MonkeyPatch, present: int, identifier_kind: int
) -> None:
    def response(_item: object, out_present: object, identifier: object) -> int:
        out_present[0] = present  # type: ignore[index]
        identifier.kind = identifier_kind  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_get_usage_allocation=response),
    )

    with pytest.raises(errors.InvalidInputError):
        client._get_usage_allocation_response(SimpleNamespace(_pointer=lambda: object()))


def test_get_usage_allocation_releases_inconsistent_ttlv_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)
    released: list[object] = []

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.INTEGER)  # type: ignore[attr-defined]
        identifier.integer_value = -17  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_usage_allocation=response,
            kmipkit_client_batch_item_response_get_usage_allocation_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._get_usage_allocation_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [pointer]


def test_get_usage_allocation_releases_ttlv_when_conversion_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)
    released: list[object] = []

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.INTEGER)  # type: ignore[attr-defined]
        identifier.integer_value = -17  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = pointer  # type: ignore[index]
        return 0

    def fail_conversion(_address: int) -> object:
        raise RuntimeError("conversion failed")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_usage_allocation=response,
            kmipkit_client_batch_item_response_get_usage_allocation_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", fail_conversion)

    with pytest.raises(RuntimeError, match="conversion failed"):
        client._get_usage_allocation_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [pointer]


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
        **batch_result_test_symbols(ffi, item_pointer, metadata),
        kmipkit_client_batch_item_response_get_usage_allocation=allocation_response,
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv=allocation_ttlv_response,
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
