"""Java/Python facade mapping for the approved KMIP 2.1 Query and Ping API."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions
from kmipkit._client_ffi import ffi


def test_ping_builder_calls_the_native_ping_constructor(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_ping() == 23
    assert calls == [("kmipkit_client_batch_item_ping", ())]


def test_query_builder_preserves_raw_function_and_group_order_and_presence(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    functions = [1, 0xF123_4567, 1]
    groups = ["group-a", "gr\u00f8up-b", "group-a"]
    assert extensions.client_batch_item_query(functions, groups) == 23

    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_query"
    function_values = ffi.cast("uint32_t *", arguments[0])
    assert [int(function_values[index]) for index in range(int(arguments[1]))] == functions
    assert arguments[2] == 1
    spans = ffi.cast("kmipkit_byte_span_v1_t *", arguments[3])
    decoded_groups = [
        bytes(ffi.buffer(spans[index].data, int(spans[index].length))).decode("utf-8")
        for index in range(int(arguments[4]))
    ]
    assert decoded_groups == groups

    extensions.client_batch_item_query([14], None)
    absent_arguments = calls[1][1]
    assert absent_arguments[2] == 0
    assert absent_arguments[3] == ffi.NULL
    assert absent_arguments[4] == 0

    extensions.client_batch_item_query([14], [])
    empty_arguments = calls[2][1]
    assert empty_arguments[2] == 1
    assert empty_arguments[4] == 0


def test_query_builder_requires_a_non_empty_function_sequence() -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_query([], None)


@pytest.mark.parametrize(
    ("operation", "mapper_name", "result_field"),
    [
        pytest.param(0x3B, "_ping_response", "ping_response", id="ping"),
        pytest.param(0x18, "_query_response", "query_response", id="query"),
    ],
)
def test_batch_result_dispatches_ping_and_query_payload_mappers(
    monkeypatch: pytest.MonkeyPatch,
    operation: int,
    mapper_name: str,
    result_field: str,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    mapped_response = True if operation == 0x3B else object()
    dispatched: list[object] = []

    def metadata(_item: object, out: object) -> int:
        out.struct_size = ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")  # type: ignore[attr-defined]
        out.abi_version = 2  # type: ignore[attr-defined]
        out.operation = operation  # type: ignore[attr-defined]
        out.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        out.result_status = 0  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: None,
        kmipkit_client_batch_item_response_release=lambda _ptr: None,
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda *_args: 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda *_args: 0,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client, "_response_span", lambda _item, _function: None)
    monkeypatch.setattr(client, "_discover_versions_supported_versions", lambda _item: None)
    monkeypatch.setattr(client, "_create_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_key_pair_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_split_key_response", lambda _item: None)
    monkeypatch.setattr(client, "_join_split_key_response_for_result", lambda **_kwargs: None)
    monkeypatch.setattr(client, "_cryptographic_responses", lambda *_args: (None,) * 7)

    def mapper(item: object) -> object:
        dispatched.append(item._pointer())  # type: ignore[attr-defined]
        return mapped_response

    monkeypatch.setattr(client, mapper_name, mapper)

    result = client._batch_result(response_pointer)
    try:
        assert result.items[0].operation == operation
        assert getattr(result.items[0], result_field) is mapped_response
        assert dispatched == [item_pointer]
    finally:
        result.close()


def _opaque(type_name: str) -> tuple[object, object]:
    storage = ffi.new("uint8_t[]", b"\x00")
    return ffi.cast(f"{type_name} *", storage), storage
