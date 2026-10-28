"""Java/Python facade mapping for the approved KMIP 2.1 Query and Ping API."""

from __future__ import annotations

from dataclasses import fields
from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def test_ping_query_result_fields_follow_existing_positional_fields() -> None:
    result_fields = tuple(field.name for field in fields(client.BatchItemResult))

    start = result_fields.index("encrypt_response")
    assert result_fields[start : start + 16] == (
        "encrypt_response",
        "decrypt_response",
        "ping_response",
        "query_response",
        "pkcs11_response",
        "recertify_response",
        "rekey_response",
        "rekey_key_pair_response",
        "reprovision_response",
        "register_response",
        "revoke_response",
        "set_defaults_response",
        "activate_response",
        "archive_response",
        "destroy_response",
        "recover_response",
    )


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

    def make_handle(
        _kind: str, symbol: str, arguments: tuple[object, ...]
    ) -> int:
        calls.append((symbol, arguments))
        if symbol == "kmipkit_client_batch_item_query":
            function_values = ffi.cast("uint32_t *", arguments[0])
            functions_seen = [
                int(function_values[index]) for index in range(int(arguments[1]))
            ]
            spans = ffi.cast("kmipkit_byte_span_v1_t *", arguments[3])
            groups_seen = [
                bytes(ffi.buffer(spans[index].data, int(spans[index].length))).decode(
                    "utf-8"
                )
                for index in range(int(arguments[4]))
            ]
            captured_query_inputs.append(
                (functions_seen, int(arguments[2]), groups_seen)
            )
        return 23

    captured_query_inputs: list[tuple[list[int], int, list[str]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        make_handle,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    functions = [1, 0xF123_4567, 1]
    groups = ["group-a", "gr\u00f8up-b", "group-a"]
    assert extensions.client_batch_item_query(functions, groups) == 23

    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_query"
    assert arguments[2] == 1
    assert captured_query_inputs[0] == (functions, 1, groups)

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
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_query([1], [None])  # type: ignore[list-item]
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_query([1], ["\ud800"])


@pytest.mark.parametrize("invalid_argument", ("limits", "functions", "groups"))
def test_query_builder_rejects_invalid_sequence_and_limit_types(
    invalid_argument: str,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        if invalid_argument == "limits":
            extensions.client_batch_item_query_with_limits(
                [1], [], object()  # type: ignore[arg-type]
            )
        elif invalid_argument == "functions":
            extensions.client_batch_item_query(1, None)  # type: ignore[arg-type]
        else:
            extensions.client_batch_item_query([1], object())  # type: ignore[arg-type]


def test_query_snapshot_uses_tuple_backing_and_string_contents(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class HostileTuple(tuple):
        def __len__(self) -> int:
            raise AssertionError("tuple subclass length override must be ignored")

        def __iter__(self):
            raise AssertionError("tuple subclass iterator override must be ignored")

    class HostileString(str):
        def __len__(self) -> int:
            raise AssertionError("string subclass length override must be ignored")

        def __getitem__(self, index: int) -> str:
            raise AssertionError("string subclass indexing override must be ignored")

        def encode(self, *_args: object, **_kwargs: object) -> bytes:
            raise AssertionError("string subclass encoding override must be ignored")

    observed: list[tuple[list[int], list[str]]] = []

    def make_handle(_kind: str, _symbol: str, arguments: tuple[object, ...]) -> int:
        function_values = ffi.cast("uint32_t *", arguments[0])
        functions = [int(function_values[index]) for index in range(int(arguments[1]))]
        spans = ffi.cast("kmipkit_byte_span_v1_t *", arguments[3])
        groups = [
            bytes(ffi.buffer(spans[index].data, int(spans[index].length))).decode(
                "utf-8"
            )
            for index in range(int(arguments[4]))
        ]
        observed.append((functions, groups))
        return 23

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    functions = HostileTuple((1, 0xF123_4567))
    groups = HostileTuple((HostileString("backing-value"),))
    assert extensions.client_batch_item_query(functions, groups) == 23  # type: ignore[arg-type]
    assert observed == [([1, 0xF123_4567], ["backing-value"])]


def test_bounded_snapshot_maps_list_allocation_failure_to_resource_limit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class AllocationFailureTuple(tuple):
        def __new__(cls, _values=()):
            raise MemoryError

    monkeypatch.setattr(extensions, "tuple", AllocationFailureTuple, raising=False)

    with pytest.raises(errors.ResourceLimitError):
        extensions._bounded_snapshot(["group"], 10)


def test_bounded_snapshot_maps_tuple_allocation_failure_to_resource_limit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class AllocationFailureTuple(tuple):
        fail = False

        def __new__(cls, values=()):
            if cls.fail:
                raise MemoryError
            return super().__new__(cls, values)

    values = AllocationFailureTuple(("group",))
    AllocationFailureTuple.fail = True
    monkeypatch.setattr(extensions, "tuple", AllocationFailureTuple, raising=False)

    with pytest.raises(errors.ResourceLimitError):
        extensions._bounded_snapshot(values, 10)


@pytest.mark.parametrize(
    ("failure", "error_type"),
    [
        (MemoryError, errors.ResourceLimitError),
        (ValueError, errors.InvalidInputError),
    ],
)
def test_validate_iterable_snapshot_maps_iterator_creation_failures(
    failure: type[Exception], error_type: type[Exception]
) -> None:
    class FailingIterable:
        def __iter__(self):
            raise failure("iterator creation failed")

    with pytest.raises(error_type):
        extensions._bounded_iterable_snapshot(FailingIterable(), 4)


@pytest.mark.parametrize(
    ("failure", "error_type"),
    [
        (MemoryError, errors.ResourceLimitError),
        (ValueError, errors.InvalidInputError),
    ],
)
def test_validate_iterable_snapshot_maps_iteration_failures(
    failure: type[Exception], error_type: type[Exception]
) -> None:
    class FailingIterator:
        def __iter__(self):
            return self

        def __next__(self):
            raise failure("iteration failed")

    with pytest.raises(error_type):
        extensions._bounded_iterable_snapshot(FailingIterator(), 4)


def test_validate_iterable_snapshot_enforces_remaining_budget_for_underreported_size() -> None:
    class UnderreportedIterable:
        def __len__(self) -> int:
            return 0

        def __iter__(self):
            return iter(("first",))

    with pytest.raises(errors.ResourceLimitError):
        extensions._bounded_iterable_snapshots(
            (["already counted"], UnderreportedIterable()), 1
        )


@pytest.mark.parametrize(
    ("failure", "error_type"),
    [
        (MemoryError, errors.ResourceLimitError),
        (ValueError, errors.InvalidInputError),
    ],
)
def test_validate_iterable_snapshots_maps_sized_length_failures(
    failure: type[Exception], error_type: type[Exception]
) -> None:
    class FailingSizedIterable:
        def __len__(self) -> int:
            raise failure("length lookup failed")

        def __iter__(self):
            return iter(())

    with pytest.raises(error_type):
        extensions._bounded_iterable_snapshots((FailingSizedIterable(),), 4)


def test_validate_utf8_length_rejects_invalid_and_over_limit_values() -> None:
    assert extensions._bounded_utf8_length("🔐", 4) == 4
    with pytest.raises(errors.InvalidInputError):
        extensions._bounded_utf8_length(object(), 4)  # type: ignore[arg-type]
    with pytest.raises(errors.ResourceLimitError):
        extensions._bounded_utf8_length("long", 3)
    with pytest.raises(errors.InvalidInputError):
        extensions._bounded_utf8_length("\ud800", 4)
    with pytest.raises(errors.ResourceLimitError):
        extensions._bounded_utf8_length("é", 1)


def test_query_preflight_rejects_object_group_span_count_overflow(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(extensions.sys, "maxsize", 4)
    monkeypatch.setattr(
        extensions,
        "_client_ffi",
        SimpleNamespace(
            sizeof=lambda name: 1 if name == "uint32_t" else 8,
        ),
    )

    with pytest.raises(errors.ResourceLimitError):
        extensions.client_batch_item_query([1], ["group"])


def test_query_preflight_rejects_function_count_overflow(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(extensions.sys, "maxsize", 0)
    monkeypatch.setattr(
        extensions,
        "_client_ffi",
        SimpleNamespace(sizeof=lambda _name: 1),
    )

    with pytest.raises(errors.ResourceLimitError):
        extensions.client_batch_item_query([1], None)


def test_query_rechecks_group_span_count_after_snapshot(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(extensions.sys, "maxsize", 4)
    monkeypatch.setattr(
        extensions,
        "_client_ffi",
        SimpleNamespace(sizeof=lambda name: 1 if name == "uint32_t" else 8),
    )
    groups: list[str] = []

    def changed_snapshot(values: object, _maximum: int) -> tuple[object, ...]:
        return ("group",) if values is groups else tuple(values)  # type: ignore[arg-type]

    monkeypatch.setattr(extensions, "_bounded_snapshot", changed_snapshot)

    with pytest.raises(errors.ResourceLimitError):
        extensions.client_batch_item_query([1], groups)


def test_query_rejects_encoded_byte_count_overflow_before_native_staging(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class MinimalFFI:
        NULL = None

        @staticmethod
        def sizeof(_name: str) -> int:
            return 1

        @staticmethod
        def new(c_type: str, value: object) -> object:
            if c_type == "kmipkit_byte_span_v1_t[]":
                return [SimpleNamespace(data=None, length=0) for _ in range(int(value))]
            return object()

    monkeypatch.setattr(extensions.sys, "maxsize", 3)
    monkeypatch.setattr(extensions, "_client_ffi", MinimalFFI())

    with pytest.raises(errors.ResourceLimitError):
        extensions.client_batch_item_query([1], ["four"])


def test_query_rechecks_element_limit_after_snapshot(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    limits = ttlv.codec_limits_create(4096, 64, 3)
    groups: list[str] = []

    def changed_snapshot(values: object, _maximum: int) -> tuple[object, ...]:
        return ("group-a", "group-b") if values is groups else tuple(values)  # type: ignore[arg-type]

    monkeypatch.setattr(extensions, "_bounded_snapshot", changed_snapshot)
    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_query_with_limits([1], groups, limits)
    finally:
        limits.close()


def test_query_builder_rejects_group_text_when_character_count_exceeds_budget() -> None:
    limits = ttlv.codec_limits_create(32, 64, 100)
    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_query_with_limits([1], ["x"], limits)
    finally:
        limits.close()


def test_query_builder_rejects_ttlv_padding_over_message_budget() -> None:
    limits = ttlv.codec_limits_create(35, 64, 100)
    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_query_with_limits([1], ["x"], limits)
    finally:
        limits.close()


def test_query_builder_counts_utf8_bytes_and_preserves_supplementary_text(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    observed: list[str] = []

    def make_handle(_kind: str, _symbol: str, arguments: tuple[object, ...]) -> int:
        spans = ffi.cast("kmipkit_byte_span_v1_t *", arguments[3])
        observed.append(
            bytes(ffi.buffer(spans[0].data, int(spans[0].length))).decode("utf-8")
        )
        return 23

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    group = "\u07ff\u0800\U0001f642"
    assert extensions.client_batch_item_query([1], [group]) == 23
    assert observed == [group]


def test_query_builder_rejects_default_limit_excess_before_snapshot(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(
        extensions,
        "_bounded_snapshot",
        lambda *_args: pytest.fail("query inputs must be preflighted before snapshotting"),
    )
    with pytest.raises(errors.ResourceLimitError):
        extensions.client_batch_item_query([1] * 100_001, [])


def test_query_with_limits_rejects_insufficient_structure_depth_before_snapshot(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    limits = ttlv.codec_limits_create(1024, 3, 100)
    monkeypatch.setattr(
        extensions,
        "_bounded_snapshot",
        lambda *_args: pytest.fail("query inputs must be preflighted before snapshotting"),
    )
    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_query_with_limits([1], ["group"], limits)
        shallow_limits = ttlv.codec_limits_create(1024, 2, 100)
        try:
            with pytest.raises(errors.ResourceLimitError):
                extensions.client_batch_item_query_with_limits([1], None, shallow_limits)
        finally:
            shallow_limits.close()
    finally:
        limits.close()


def test_native_ping_and_query_builders_return_owned_items() -> None:
    ping = extensions.client_batch_item_ping()
    query = extensions.client_batch_item_query(
        [1, 0xF123_4567, 1], ["group-a", "gr\u00f8up-b", "group-a"]
    )
    absent_groups = extensions.client_batch_item_query([14], None)
    empty_groups = extensions.client_batch_item_query([14], [])
    try:
        assert not ping.closed
        assert not query.closed
        assert not absent_groups.closed
        assert not empty_groups.closed
    finally:
        ping.close()
        query.close()
        absent_groups.close()
        empty_groups.close()


def test_native_query_builder_accepts_more_than_one_hundred_thousand_values() -> None:
    functions = [1] * 100_001
    groups = ["group"] * 100_001
    limits = ttlv.codec_limits_create(8 * 1024 * 1024, 64, 400_000)

    item = extensions.client_batch_item_query_with_limits(functions, groups, limits)
    try:
        assert not item.closed
    finally:
        item.close()
        limits.close()


def test_native_query_builder_accepts_object_group_larger_than_default_limit() -> None:
    object_group = "x" * (16 * 1024 * 1024 + 1)
    limits = ttlv.codec_limits_create(32 * 1024 * 1024, 64, 100_000)

    item = extensions.client_batch_item_query_with_limits(
        [1], [object_group], limits
    )
    try:
        assert not item.closed
    finally:
        item.close()
        limits.close()


@pytest.mark.parametrize(
    ("operation", "mapper_name", "result_field"),
    [
        pytest.param(0x3B, "_ping_response", "ping_response", id="ping"),
        pytest.param(0x18, "_query_response", "query_response", id="query"),
        pytest.param(
            0x10,
            "_obtain_lease_response",
            "obtain_lease_response",
            id="obtain-lease",
        ),
    ],
)
def test_batch_result_dispatches_operation_specific_payload_mappers(
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
    monkeypatch.setattr(
        client, "_join_split_key_response_for_result", lambda *_args, **_kwargs: None
    )
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


def test_ping_response_maps_presence_and_rejects_invalid_native_values(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    present = [1]
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_ping=lambda _item, output: output.__setitem__(
                0, present[0]
            )
            or 0
        ),
    )
    item = SimpleNamespace(_pointer=lambda: object())

    assert client._ping_response(item) is True
    present[0] = 0
    assert client._ping_response(item) is None
    present[0] = 2
    with pytest.raises(errors.InvalidInputError):
        client._ping_response(item)


def test_query_response_adopts_and_releases_owned_ttlv_handles(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)
    present = [1]
    released: list[object] = []
    payload = object()

    def response_ttlv(_item: object, out_present: object, output: object) -> int:
        out_present[0] = present[0]  # type: ignore[index]
        output[0] = handle if present[0] == 1 else ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_query_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(
        client.ttlv, "_structure_from_native_address", lambda _address: payload
    )
    item = SimpleNamespace(_pointer=lambda: object())

    assert client._query_response(item) is payload
    assert released == []
    present[0] = 0
    assert client._query_response(item) is None

    def absent_with_handle(_item: object, out_present: object, output: object) -> int:
        out_present[0] = 0  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_query_ttlv=absent_with_handle,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._query_response(item)
    assert released == [handle]

    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("test wrapper failure")),
    )
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_query_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    present[0] = 1
    with pytest.raises(RuntimeError, match="test wrapper failure"):
        client._query_response(item)
    assert released == [handle, handle]

    def failed_response(_item: object, out_present: object, output: object) -> int:
        out_present[0] = 1  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 4

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_query_ttlv=failed_response,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._query_response(item)
    assert released == [handle, handle, handle]


def _opaque(type_name: str) -> tuple[object, object]:
    storage = ffi.new("uint8_t[]", b"\x00")
    return ffi.cast(f"{type_name} *", storage), storage
