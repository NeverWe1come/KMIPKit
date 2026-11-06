from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions, ttlv


def _opaque(c_type: str) -> tuple[object, object]:
    storage = client.ffi.new("uint8_t[]", 1)
    return client.ffi.cast(f"{c_type} *", storage), storage


def _one_item_structure() -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(0x0054_1234)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_text_string(b"interop-payload", limits)
    item = ttlv.ttlv_item_create(tag, value, limits)
    structure = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), item, limits)
    item.close()
    tag.close()
    raw_tag.close()
    value.close()
    limits.close()
    return structure


@pytest.mark.parametrize(
    ("operation", "constructor_name"),
    (
        ("get_usage_allocation", "kmipkit_client_batch_item_get_usage_allocation"),
        ("import", "kmipkit_client_batch_item_import"),
        ("interop", "kmipkit_client_batch_item_interop"),
        ("log", "kmipkit_client_batch_item_log"),
        ("validate", "kmipkit_client_batch_item_validate"),
    ),
)
def test_operation_specific_python_builder_executes_as_one_client_item(
    monkeypatch: pytest.MonkeyPatch,
    operation: str,
    constructor_name: str,
) -> None:
    builder_arguments: list[tuple[object, ...]] = []
    item_storage: list[object] = []

    def construct_item(
        kind: str,
        symbol: str,
        arguments: tuple[object, ...],
        _out_index: int | None = None,
        _consumed_indices: tuple[int, ...] = (),
    ) -> object:
        assert kind == "kmipkit_client_batch_item_t"
        builder_arguments.append((symbol, *arguments))
        storage = client.ffi.new("uint8_t[]", 1)
        item_storage.append(storage)
        return client.ffi.cast(
            "kmipkit_client_batch_item_t *", storage
        )

    monkeypatch.setattr(
        extensions,
        "_make_handle",
        construct_item,
    )

    attributes: ttlv.Structure | None = None
    any_object: ttlv.Structure | None = None
    item: extensions.ClientBatchItem | None = None
    if operation == "get_usage_allocation":
        item = extensions.client_batch_item_get_usage_allocation(
            extensions.UniqueIdentifier.text_string("usage-object"), 1
        )
    elif operation == "import":
        attributes = ttlv.ttlv_structure_create()
        any_object = _one_item_structure()
        item = extensions.client_batch_item_import(
            extensions.UniqueIdentifier.text_string("import-object"),
            0xF123_4567,
            attributes,
            any_object,
        )
    elif operation == "interop":
        item = extensions.client_batch_item_interop(0xF123_4567, "interop-identifier")
    elif operation == "log":
        item = extensions.client_batch_item_log("direct-helper-log")
    else:
        item = extensions.client_batch_item_validate(
            (), (extensions.UniqueIdentifier.text_string("validation-chain"),)
        )

    assert item is not None
    expected_item_address = int(client.ffi.cast("uintptr_t", item._pointer()))
    client_pointer, client_storage = _opaque("kmipkit_client_t")
    batch_pointer, batch_storage = _opaque("kmipkit_client_batch_t")
    response_pointer, response_storage = _opaque("kmipkit_client_batch_response_t")
    handle = client._wrap_owned_handle(client.Client, client_pointer)
    batch = client._wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    appended_items: list[int] = []
    exchanges: list[int] = []
    result = object()

    def add_item(_batch: object, item_slot: object) -> int:
        native_item = item_slot[0]  # type: ignore[index]
        appended_items.append(int(client.ffi.cast("uintptr_t", native_item)))
        item_slot[0] = client.ffi.NULL  # type: ignore[index]
        return 0

    def execute(
        _client: object,
        batch_slot: object,
        _limits: object,
        output: object,
        error_output: object,
    ) -> int:
        exchanges.append(1)
        assert appended_items == [expected_item_address]
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        assert error_output[0] == client.ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_add_item=add_item,
            kmipkit_client_execute_with_error_ttlv_v1=execute,
            kmipkit_client_release=lambda _pointer: None,
            kmipkit_client_batch_release=lambda _pointer: None,
        ),
    )
    monkeypatch.setattr(client, "_batch_result", lambda _response, _limits: result)

    try:
        assert batch.add_item(item) is batch
        assert item.closed
        assert handle.execute(batch, limits) is result
        assert batch.closed
        assert len(builder_arguments) == 1
        assert builder_arguments[0][0] == constructor_name
        assert len(exchanges) == 1
    finally:
        item.close()
        batch.close()
        handle.close()
        limits.close()
        if attributes is not None:
            attributes.close()
        if any_object is not None:
            any_object.close()

    assert client_storage is not None
    assert batch_storage is not None
    assert response_storage is not None
    assert item_storage
