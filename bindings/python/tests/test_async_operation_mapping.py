"""Explicit asynchronous operation mapping through the Python client facade."""

from __future__ import annotations

from contextlib import ExitStack
from threading import Event, Thread
from types import SimpleNamespace

import pytest

from kmipkit import _handles, client, errors, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._handles import _wrap_owned_handle


def _released_correlation_view() -> memoryview:
    value = memoryview(b"correlation")
    value.release()
    return value


@pytest.mark.parametrize(
    "source",
    [
        b"\x00\xffcorrelation",
        bytearray(b"\x00\xffcorrelation"),
        memoryview(b"\x00\xffcorrelation"),
        memoryview(b"a-b-c-d")[::2],
    ],
    ids=("bytes", "bytearray", "memoryview", "strided-memoryview"),
)
def test_correlation_staging_is_wipeable_for_all_buffer_inputs(source) -> None:
    expected = memoryview(source).tobytes()
    raw, staged = client._stage_correlation_value(source)
    try:
        assert bytes(ffi.buffer(staged, len(expected))) == expected
    finally:
        client._wipe_staging_buffer(staged)

    assert isinstance(raw, bytearray)
    assert raw == bytearray(len(expected))


@pytest.mark.parametrize(
    "source_factory",
    [
        pytest.param(lambda: object(), id="not-a-buffer"),
        pytest.param(_released_correlation_view, id="released-memoryview"),
    ],
)
def test_poll_rejects_invalid_correlation_before_native_call(
    monkeypatch, source_factory
) -> None:
    native_calls: list[object] = []
    client_handle = object.__new__(client.Client)
    limits = ttlv.codec_limits_defaults()

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_poll=lambda *args: native_calls.append(args)
        ),
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            client_handle.execute_poll(source_factory(), limits)
    finally:
        limits.close()

    assert native_calls == []


def test_poll_wipes_staged_correlation_if_native_buffer_creation_fails(monkeypatch):
    staged_values: list[bytearray] = []

    class FailingBufferBridge:
        NULL = ffi.NULL

        @staticmethod
        def from_buffer(_ctype, staged: bytearray):
            staged_values.append(staged)
            raise BufferError("buffer bridge unavailable")

    client_handle = object.__new__(client.Client)
    limits = ttlv.codec_limits_defaults()
    monkeypatch.setattr(client, "ffi", FailingBufferBridge)
    try:
        with pytest.raises(BufferError, match="buffer bridge unavailable"):
            client_handle.execute_poll(b"correlation-secret", limits)
    finally:
        limits.close()

    assert staged_values == [bytearray(len(b"correlation-secret"))]


def test_poll_preserves_binary_correlation_and_exposes_pending_outcome(
    monkeypatch,
) -> None:
    expected_request_correlation = b"\x00\xff\x80\x13\x00"
    expected_response_correlation = b"\x00\x81\xfe\x00"
    native_correlation = ffi.new("uint8_t[]", expected_response_correlation)
    outcome_handle = ffi.cast("kmipkit_client_async_operation_outcome_t *", 42)
    released: list[object] = []
    observed_request: list[bytes] = []

    def execute_poll(_client, data, length, _limits, output) -> int:
        observed_request.append(bytes(ffi.buffer(data, int(length))))
        output[0] = outcome_handle
        return 0

    def metadata(_outcome, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_async_operation_metadata_v1_t")
        output.abi_version = 1
        output.operation = 0x1A
        output.outcome_kind = int(client.BatchOutcome.PENDING)
        output.result_status = 2
        output.has_result_reason = 0
        output.result_reason = 0
        output.has_cancellation_result = 0
        output.cancellation_result = 0
        output.has_result_message = 0
        output.reserved = 0
        output.result_message_data = ffi.NULL
        output.result_message_length = 0
        return 0

    def correlation(_outcome, present, span) -> int:
        present[0] = 1
        span.data = native_correlation
        span.length = len(expected_response_correlation)
        return 0

    def response_payload(_outcome, present, output) -> int:
        present[0] = 0
        output[0] = ffi.NULL
        return 0

    fake = SimpleNamespace(
        kmipkit_client_execute_poll=execute_poll,
        kmipkit_client_async_operation_outcome_release=lambda value: released.append(
            value
        ),
        kmipkit_client_async_operation_outcome_metadata_v1=metadata,
        kmipkit_client_async_operation_outcome_correlation_value=correlation,
        kmipkit_client_async_operation_outcome_response_payload_ttlv=response_payload,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    client_handle = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_poll(
            client_handle, expected_request_correlation, limits
        )
        try:
            assert isinstance(outcome, client.AsyncOperationOutcome)
            assert observed_request == [expected_request_correlation]
            assert outcome.metadata_v1.operation == 0x1A
            assert outcome.metadata_v1.outcome == client.BatchOutcome.PENDING
            assert outcome.metadata_v1.result_status == 2
            borrowed: list[memoryview] = []

            def inspect_correlation(value: memoryview) -> None:
                assert value.readonly
                assert bytes(value) == expected_response_correlation
                borrowed.append(value[:])

            assert outcome.with_asynchronous_correlation_value(inspect_correlation)
            assert bytes(borrowed[0]) == bytes(len(expected_response_correlation))
            assert outcome.response_payload_ttlv is None
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [outcome_handle]


def test_async_outcome_exposes_owned_generic_response_extension_copy(
    monkeypatch,
) -> None:
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released)
    extension_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 43)
    expected_extension = _async_response_extension()
    calls: list[int] = []

    def extension_count(_outcome, output) -> int:
        output[0] = 1
        return 0

    def extension_at(_outcome, index, present, output) -> int:
        calls.append(int(index))
        present[0] = int(index == 0)
        output[0] = extension_pointer if index == 0 else ffi.NULL
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            **{
                **stubs.__dict__,
                "kmipkit_client_async_operation_outcome_extension_count": extension_count,
                "kmipkit_client_async_operation_outcome_extension_ttlv_at": extension_at,
            }
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: expected_extension if address == 43 else None,
    )
    outcome = client._wrap_owned_handle(client.AsyncOperationOutcome, stubs.outcome_handle)
    try:
        assert outcome.extension_count == 1
        assert outcome.extension_ttlv_at(0) is expected_extension
        _assert_async_extension_payload(expected_extension)
        assert calls == [0]
        with pytest.raises(errors.InvalidInputError):
            outcome.extension_ttlv_at(1)
        assert calls == [0]
    finally:
        outcome.close()
        expected_extension.close()

    assert released == [stubs.outcome_handle]


def _async_response_extension() -> ttlv.Structure:
    with ttlv.codec_limits_defaults() as limits:
        vendor_extension = ttlv.ttlv_structure_create()
        vendor_extension = _append_ttlv_item(
            vendor_extension,
            0x00540173,
            ttlv.ttlv_value_text_string(b"opaque-v1", limits),
            limits,
        )
        vendor_extension = _append_ttlv_item(
            vendor_extension,
            0x00540174,
            ttlv.ttlv_value_enumeration(0xFFFFFFFF),
            limits,
        )

        extension = ttlv.ttlv_structure_create()
        extension = _append_ttlv_item(
            extension,
            0x0042009D,
            ttlv.ttlv_value_text_string(b"OpaqueVendor", limits),
            limits,
        )
        extension = _append_ttlv_item(
            extension,
            0x00420026,
            ttlv.ttlv_value_boolean(False),
            limits,
        )
        extension = _append_ttlv_item(
            extension,
            0x0042009C,
            ttlv.ttlv_value_structure(vendor_extension, limits),
            limits,
        )
        return extension


def _append_ttlv_item(
    structure: ttlv.Structure,
    raw_tag: int,
    value: ttlv.Value,
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
    with ttlv.raw_tag_from_raw(raw_tag) as raw:
        tag = ttlv.raw_tag_try_checked(raw)
    try:
        item = ttlv.ttlv_item_create(tag, value, limits)
    finally:
        tag.close()
    return ttlv.ttlv_structure_with_item(structure, item, limits)


def _assert_async_extension_payload(extension: ttlv.Structure) -> None:
    with ttlv.ttlv_structure_view(extension) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 3
        _assert_text_value(view, 0, 0x0042009D, b"OpaqueVendor")
        _assert_boolean_value(view, 1, 0x00420026, False)
        with ttlv.ttlv_structure_view_item_at(view, 2) as item:
            with ttlv.ttlv_item_view_tag(item) as tag:
                assert ttlv.tag_value(tag) == 0x0042009C
            with ttlv.ttlv_item_view_value(item) as value:
                with ttlv.ttlv_value_view_structure(value) as vendor_view:
                    assert ttlv.ttlv_structure_view_item_count(vendor_view) == 2
                    _assert_text_value(vendor_view, 0, 0x00540173, b"opaque-v1")
                    _assert_enumeration_value(vendor_view, 1, 0x00540174, 0xFFFFFFFF)


def _assert_text_value(
    view: ttlv.TtlvStructureView, index: int, raw_tag: int, expected: bytes
) -> None:
    with ttlv.ttlv_structure_view_item_at(view, index) as item:
        assert ttlv.ttlv_item_view_type(item) is ttlv.ItemType.TextString
        with ttlv.ttlv_item_view_tag(item) as tag:
            assert ttlv.tag_value(tag) == raw_tag
        with ttlv.ttlv_item_view_value(item) as value:
            actual = bytes(
                ttlv.ttlv_value_view_byte_at(value, offset)
                for offset in range(ttlv.ttlv_value_view_byte_length(value))
            )
        assert actual == expected


def _assert_boolean_value(
    view: ttlv.TtlvStructureView, index: int, raw_tag: int, expected: bool
) -> None:
    with ttlv.ttlv_structure_view_item_at(view, index) as item:
        assert ttlv.ttlv_item_view_type(item) is ttlv.ItemType.Boolean
        with ttlv.ttlv_item_view_tag(item) as tag:
            assert ttlv.tag_value(tag) == raw_tag
        with ttlv.ttlv_item_view_value(item) as value:
            assert ttlv.ttlv_value_view_boolean(value) is expected


def _assert_enumeration_value(
    view: ttlv.TtlvStructureView, index: int, raw_tag: int, expected: int
) -> None:
    with ttlv.ttlv_structure_view_item_at(view, index) as item:
        assert ttlv.ttlv_item_view_type(item) is ttlv.ItemType.Enumeration
        with ttlv.ttlv_item_view_tag(item) as tag:
            assert ttlv.tag_value(tag) == raw_tag
        with ttlv.ttlv_item_view_value(item) as value:
            assert ttlv.ttlv_value_view_enumeration(value) == expected


def _outcome_stubs(
    operation: int,
    released: list[object],
    *,
    response_correlation: bytes | None = None,
    cancellation_result: int | None = None,
    outcome_kind: int = int(client.BatchOutcome.COMPLETED),
    result_status: int = 0,
    response_payload_structure: ttlv.Structure | None = None,
) -> SimpleNamespace:
    outcome_handle = ffi.cast("kmipkit_client_async_operation_outcome_t *", 42)
    native_correlation = (
        ffi.new("uint8_t[]", response_correlation)
        if response_correlation is not None
        else ffi.NULL
    )

    def metadata(_outcome, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_async_operation_metadata_v1_t")
        output.abi_version = 1
        output.operation = operation
        output.outcome_kind = outcome_kind
        output.result_status = result_status
        output.has_result_reason = 0
        output.result_reason = 0
        output.has_cancellation_result = int(cancellation_result is not None)
        output.cancellation_result = cancellation_result or 0
        output.has_result_message = 0
        output.reserved = 0
        output.result_message_data = ffi.NULL
        output.result_message_length = 0
        return 0

    def correlation(_outcome, present, span) -> int:
        present[0] = int(response_correlation is not None)
        span.data = native_correlation
        span.length = len(response_correlation or b"")
        return 0

    def response_payload(_outcome, present, output) -> int:
        if response_payload_structure is None:
            present[0] = 0
            output[0] = ffi.NULL
        else:
            present[0] = 1
            native_payload, _owner = response_payload_structure._take()
            address = int(ttlv.ffi.cast("uintptr_t", native_payload))
            output[0] = ffi.cast("kmipkit_ttlv_structure_t *", address)
        return 0

    return SimpleNamespace(
        outcome_handle=outcome_handle,
        kmipkit_client_async_operation_outcome_release=lambda value: released.append(
            value
        ),
        kmipkit_client_async_operation_outcome_metadata_v1=metadata,
        kmipkit_client_async_operation_outcome_correlation_value=correlation,
        kmipkit_client_async_operation_outcome_response_payload_ttlv=response_payload,
    )


def _typed_outcome_stubs(operation: int, *, is_pending: bool = False,
                         payload_member_count: int | None = None,
                         has_payload: bool = False,
                         correlation: bytes | None = None,
                         payload_structure: ttlv.Structure | None = None) -> SimpleNamespace:
    """Build typed-reader ABI stubs without allocating another native owner."""
    stubs = _outcome_stubs(
        operation,
        [],
        response_correlation=correlation,
        cancellation_result=1 if operation == 0x19 else None,
        outcome_kind=(
            int(client.BatchOutcome.PENDING)
            if is_pending
            else int(client.BatchOutcome.COMPLETED)
        ),
        result_status=2 if is_pending else 0,
        response_payload_structure=payload_structure,
    )
    native_correlation = (
        ffi.new("uint8_t[]", correlation) if correlation is not None else ffi.NULL
    )

    def read(expected: int):
        def invoke(_outcome, present, output) -> int:
            present[0] = int(operation == expected)
            if present[0]:
                output.struct_size = ffi.sizeof(
                    "kmipkit_client_async_operation_response_v1_t"
                )
                output.abi_version = 1
                output.operation = expected
                output.result_status = 2 if is_pending else 0
                output.is_pending = int(is_pending)
                output.has_cancellation_result = int(expected == 0x19)
                output.cancellation_result = 1 if expected == 0x19 else 0
                output.has_asynchronous_correlation_value = int(
                    correlation is not None
                )
                output.asynchronous_correlation_value.data = native_correlation
                output.asynchronous_correlation_value.length = len(correlation or b"")
                output.has_response_payload = int(has_payload)
                output.reserved = 0
                output.response_payload_member_count = payload_member_count or 0
            return 0

        return invoke

    stubs.kmipkit_client_async_operation_outcome_cancel_response_v1 = read(
        0x19
    )
    stubs.kmipkit_client_async_operation_outcome_poll_response_v1 = read(0x1A)
    stubs.kmipkit_client_async_operation_outcome_process_response_v1 = read(0x3A)
    stubs.kmipkit_client_async_operation_outcome_query_async_requests_response_v1 = read(
        0x39
    )
    return stubs


@pytest.mark.parametrize(
    ("operation", "reader"),
    [
        (0x19, "cancel_response"),
        (0x1A, "poll_response"),
        (0x3A, "process_response"),
        (0x39, "query_async_requests_response"),
    ],
)
def test_async_typed_response_reader_returns_an_owner_backed_typed_view(
    monkeypatch, operation: int, reader: str
) -> None:
    stubs = _typed_outcome_stubs(
        operation,
        is_pending=operation in (0x1A, 0x3A, 0x39),
        payload_member_count=0,
        has_payload=operation in (0x3A, 0x39),
        correlation=(b"typed-correlation" if operation in (0x19, 0x1A, 0x3A) else None),
        payload_structure=(
            ttlv.ttlv_structure_create()
            if operation in (0x3A, 0x39)
            else None
        ),
    )
    monkeypatch.setattr(client, "lib", stubs)
    outcome = _wrap_owned_handle(client.AsyncOperationOutcome, stubs.outcome_handle)
    try:
        typed = getattr(outcome, reader)()
        assert typed is not None
        assert typed.result.operation == operation
        if operation in (0x3A, 0x39):
            raw = typed.raw
            assert raw is not None
            try:
                with ttlv.ttlv_structure_view(raw) as view:
                    assert ttlv.ttlv_structure_view_item_count(view) == 0
            finally:
                raw.close()
        else:
            assert typed.raw is None
        if operation == 0x19:
            assert typed.cancellation_result == 1
            observed: list[bytes] = []

            def inspect_echo(value: memoryview) -> None:
                assert value.readonly
                assert bytes(value) == b"typed-correlation"
                observed.append(bytes(value))

            assert typed.with_echoed_correlation_value(inspect_echo)
            assert observed == [b"typed-correlation"]
        if operation in (0x1A, 0x3A, 0x39):
            assert typed.is_pending
        if operation == 0x3A:
            assert typed.payload_member_count == 0
    finally:
        outcome.close()


def test_async_typed_response_readers_return_none_for_other_operations(
    monkeypatch,
) -> None:
    stubs = _typed_outcome_stubs(0x1A, is_pending=True, has_payload=False)
    monkeypatch.setattr(client, "lib", stubs)
    outcome = _wrap_owned_handle(client.AsyncOperationOutcome, stubs.outcome_handle)
    try:
        assert outcome.poll_response() is not None
        assert outcome.cancel_response() is None
        assert outcome.process_response() is None
        assert outcome.query_async_requests_response() is None
    finally:
        outcome.close()


def test_async_response_view_uses_the_outcome_lifetime(monkeypatch) -> None:
    stubs = _typed_outcome_stubs(0x1A, is_pending=True)
    monkeypatch.setattr(client, "lib", stubs)
    outcome = _wrap_owned_handle(client.AsyncOperationOutcome, stubs.outcome_handle)
    view = outcome.poll_response()
    assert view is not None
    outcome.close()
    with pytest.raises(errors.InvalidInputError):
        _ = view.result
    with pytest.raises(errors.InvalidInputError):
        _ = view.is_pending


def test_typed_correlation_span_is_copied_before_concurrent_close(monkeypatch) -> None:
    expected = b"borrowed-correlation"
    copy_started = Event()
    allow_copy = Event()
    close_finished = Event()
    callback_values: list[bytes] = []
    failures: list[BaseException] = []
    response = SimpleNamespace(
        has_asynchronous_correlation_value=1,
        asynchronous_correlation_value=SimpleNamespace(
            data=object(), length=len(expected)
        ),
    )
    outcome_handle = ffi.cast("kmipkit_client_async_operation_outcome_t *", 47)
    stubs = SimpleNamespace(
        kmipkit_client_async_operation_outcome_release=lambda _handle: None
    )

    def delayed_buffer(_data, length):
        copy_started.set()
        assert client._NATIVE_CALL_LOCK._is_owned()
        assert length == len(expected)
        assert allow_copy.wait(timeout=5)
        return expected

    monkeypatch.setattr(client, "ffi", SimpleNamespace(buffer=delayed_buffer))
    monkeypatch.setattr(client, "lib", stubs)
    monkeypatch.setattr(_handles, "lib", stubs)
    monkeypatch.setattr(client.AsyncOperationOutcome, "_typed_response", lambda *_args: response)
    outcome = _wrap_owned_handle(client.AsyncOperationOutcome, outcome_handle)

    def read_correlation() -> None:
        try:
            assert outcome._with_typed_correlation_value(
                "kmipkit_client_async_operation_outcome_cancel_response_v1",
                lambda view: callback_values.append(bytes(view)),
            )
        except BaseException as error:
            failures.append(error)

    def close_outcome() -> None:
        outcome.close()
        close_finished.set()

    reader = Thread(target=read_correlation)
    reader.start()
    assert copy_started.wait(timeout=5)
    closer = Thread(target=close_outcome)
    closer.start()
    try:
        assert not close_finished.wait(timeout=0.05)
    finally:
        allow_copy.set()
        reader.join(timeout=5)
        closer.join(timeout=5)

    assert not reader.is_alive()
    assert not closer.is_alive()
    assert failures == []
    assert close_finished.is_set()
    assert callback_values == [expected]


def _cancel_response_payload(
    correlation: bytes, cancellation_result: int, limits: ttlv.CodecLimits
) -> ttlv.Structure:
    with ExitStack() as owners:
        correlation_raw_tag = owners.enter_context(ttlv.raw_tag_from_raw(0x0042_0006))
        correlation_tag = owners.enter_context(ttlv.raw_tag_try_checked(correlation_raw_tag))
        correlation_value = owners.enter_context(
            ttlv.ttlv_value_byte_string(correlation, limits)
        )
        correlation_item = owners.enter_context(
            ttlv.ttlv_item_create(correlation_tag, correlation_value, limits)
        )
        structure = owners.enter_context(ttlv.ttlv_structure_create())
        structure = owners.enter_context(
            ttlv.ttlv_structure_with_item(structure, correlation_item, limits)
        )

        result_raw_tag = owners.enter_context(ttlv.raw_tag_from_raw(0x0042_0012))
        result_tag = owners.enter_context(ttlv.raw_tag_try_checked(result_raw_tag))
        result_value = owners.enter_context(ttlv.ttlv_value_enumeration(cancellation_result))
        result_item = owners.enter_context(
            ttlv.ttlv_item_create(result_tag, result_value, limits)
        )
        return ttlv.ttlv_structure_with_item(structure, result_item, limits)


def test_cancel_preserves_exact_echo_in_response_payload(monkeypatch) -> None:
    expected = b"\x00\xff\x80\x12\x00"
    cancellation_result = 1
    observed: list[bytes] = []
    released: list[object] = []
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)
    payload_structure = _cancel_response_payload(expected, cancellation_result, limits)
    # KMIP 2.1 section 11.36, Table 470 assigns Cancel the Operation value 0x19.
    stubs = _outcome_stubs(
        0x19,
        released,
        response_correlation=None,
        cancellation_result=cancellation_result,
        response_payload_structure=payload_structure,
    )

    def execute_cancel(_client, data, length, _limits, output) -> int:
        observed.append(bytes(ffi.buffer(data, int(length))))
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_cancel": execute_cancel,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    payload: ttlv.Structure | None = None

    try:
        outcome = client.Client.execute_cancel(instance, expected, limits)
        try:
            assert observed == [expected]
            assert outcome.metadata_v1.operation == 0x19
            assert outcome.metadata_v1.cancellation_result == cancellation_result
            assert not outcome.with_asynchronous_correlation_value(
                lambda _value: pytest.fail("Cancel echo is in the Response Payload")
            )
            payload = outcome.response_payload_ttlv
            assert payload is not None
            with ttlv.ttlv_structure_view(payload) as payload_view:
                assert ttlv.ttlv_structure_view_item_count(payload_view) == 2
                with ttlv.ttlv_structure_view_item_at(payload_view, 0) as item_view:
                    with ttlv.ttlv_item_view_tag(item_view) as tag:
                        assert ttlv.tag_value(tag) == 0x0042_0006
                    assert (
                        ttlv.ttlv_item_view_type(item_view)
                        == ttlv.ItemType.ByteString
                    )
                    with ttlv.ttlv_item_view_value(item_view) as value_view:
                        length = ttlv.ttlv_value_view_byte_length(value_view)
                        assert length == len(expected)
                        for index in range(length):
                            assert (
                                ttlv.ttlv_value_view_byte_at(value_view, index)
                                == expected[index]
                            )
                with ttlv.ttlv_structure_view_item_at(payload_view, 1) as item_view:
                    with ttlv.ttlv_item_view_tag(item_view) as tag:
                        assert ttlv.tag_value(tag) == 0x0042_0012
                    assert (
                        ttlv.ttlv_item_view_type(item_view)
                        == ttlv.ItemType.Enumeration
                    )
                    with ttlv.ttlv_item_view_value(item_view) as value_view:
                        assert (
                            ttlv.ttlv_value_view_enumeration(value_view)
                            == cancellation_result
                        )
        finally:
            outcome.close()
    finally:
        if payload is not None:
            payload.close()
        payload_structure.close()
        limits.close()

    assert released == [stubs.outcome_handle]


def test_async_correlation_callback_wipes_borrowed_snapshot_when_callback_raises(
    monkeypatch,
) -> None:
    expected = b"\x00\xff\x81"
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released, response_correlation=expected)
    fake = SimpleNamespace(**stubs.__dict__)
    monkeypatch.setattr(client, "lib", fake)
    outcome = client._wrap_owned_handle(
        client.AsyncOperationOutcome, stubs.outcome_handle
    )
    retained: list[memoryview] = []

    def fail_after_read(value: memoryview) -> None:
        retained.append(value[:])
        assert bytes(value) == expected
        raise RuntimeError("callback failure")

    with pytest.raises(RuntimeError, match="callback failure"):
        outcome.with_asynchronous_correlation_value(fail_after_read)

    assert bytes(retained[0]) == bytes(len(expected))
    outcome.close()
    assert released == [stubs.outcome_handle]


def test_async_correlation_buffer_is_wiped_if_readonly_view_creation_fails(
    monkeypatch,
) -> None:
    expected = b"\x00\xff\x81"
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released, response_correlation=expected)
    monkeypatch.setattr(client, "lib", SimpleNamespace(**stubs.__dict__))
    outcome = client._wrap_owned_handle(
        client.AsyncOperationOutcome, stubs.outcome_handle
    )
    buffers: list[bytearray] = []

    def tracked_bytearray(value=b"") -> bytearray:
        buffer = bytearray(value)
        buffers.append(buffer)
        return buffer

    def fail_view(_buffer):
        raise MemoryError("readonly view allocation failed")

    monkeypatch.setattr(client, "bytearray", tracked_bytearray, raising=False)
    monkeypatch.setattr(client, "memoryview", fail_view, raising=False)

    with pytest.raises(MemoryError, match="readonly view allocation failed"):
        outcome.with_asynchronous_correlation_value(lambda _value: None)

    assert buffers == [bytearray(len(expected))]
    outcome.close()
    assert released == [stubs.outcome_handle]


def test_async_correlation_wipe_does_not_allocate_a_zero_fill_copy(monkeypatch) -> None:
    expected = b"\x00\xff\x81"
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released, response_correlation=expected)
    monkeypatch.setattr(client, "lib", SimpleNamespace(**stubs.__dict__))
    outcome = client._wrap_owned_handle(
        client.AsyncOperationOutcome, stubs.outcome_handle
    )
    retained: list[memoryview] = []
    builtin_bytes = bytes

    def reject_sized_bytes(value=b""):
        if isinstance(value, int):
            raise MemoryError("zero-fill allocation attempted")
        return builtin_bytes(value)

    monkeypatch.setattr(client, "bytes", reject_sized_bytes, raising=False)
    assert outcome.with_asynchronous_correlation_value(
        lambda value: retained.append(value[:])
    )

    assert builtin_bytes(retained[0]) == builtin_bytes(len(expected))
    outcome.close()
    assert released == [stubs.outcome_handle]


def test_process_forwards_an_explicit_indicator_once(monkeypatch) -> None:
    expected = b"\x80\x00\xfe"
    indicator = 1
    observed: list[tuple[bytes, int, int]] = []
    released: list[object] = []
    stubs = _outcome_stubs(0x28, released)

    def execute_process(
        _client, data, length, has_indicator, indicator_value, _limits, output
    ) -> int:
        observed.append(
            (
                bytes(ffi.buffer(data, int(length))),
                int(has_indicator),
                int(indicator_value),
            )
        )
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_process": execute_process,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_process(
            instance, expected, limits, asynchronous_indicator=indicator
        )
        try:
            assert observed == [(expected, 1, indicator)]
            assert outcome.metadata_v1.operation == 0x28
            assert not outcome.with_asynchronous_correlation_value(
                lambda _value: pytest.fail("unexpected correlation callback")
            )
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle]


def test_query_async_requests_preserves_present_empty_and_binary_filters(
    monkeypatch,
) -> None:
    expected_correlations = (b"\x00\xff\x10", b"\x80\x00\xfe")
    expected_operations = (0x0000_0005, 0xF000_0001, 0x0000_0005)
    observed: list[tuple[tuple[bytes, ...], tuple[int, ...], tuple[int, ...]]] = []
    released: list[object] = []
    stubs = _outcome_stubs(0x39, released)

    def execute_query(
        _client,
        has_correlations,
        spans,
        correlation_count,
        has_operations,
        operations,
        operation_count,
        has_indicator,
        indicator,
        _limits,
        output,
    ) -> int:
        correlation_values = tuple(
            bytes(ffi.buffer(spans[index].data, int(spans[index].length)))
            for index in range(int(correlation_count))
        )
        operation_values = tuple(
            int(operations[index]) for index in range(int(operation_count))
        )
        observed.append(
            (
                correlation_values,
                operation_values,
                (
                    int(has_correlations),
                    int(has_operations),
                    int(has_indicator),
                    int(indicator),
                ),
            )
        )
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_query_async_requests": execute_query,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        outcome = client.Client.execute_query_async_requests(
            instance,
            limits,
            correlation_values=expected_correlations,
            operations=expected_operations,
        )
        try:
            assert observed == [
                (expected_correlations, expected_operations, (1, 1, 0, 0))
            ]
            assert outcome.metadata_v1.operation == 0x39
        finally:
            outcome.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle]


def test_query_async_requests_distinguishes_absent_from_present_empty_filters(
    monkeypatch,
) -> None:
    observed: list[tuple[int, int, int, int]] = []
    released: list[object] = []
    stubs = _outcome_stubs(0x39, released)

    def execute_query(
        _client,
        has_correlations,
        _spans,
        _correlation_count,
        has_operations,
        _operations,
        _operation_count,
        has_indicator,
        indicator,
        _limits,
        output,
    ) -> int:
        observed.append(
            (
                int(has_correlations),
                int(has_operations),
                int(has_indicator),
                int(indicator),
            )
        )
        output[0] = stubs.outcome_handle
        return 0

    fake = SimpleNamespace(
        **{
            **stubs.__dict__,
            "kmipkit_client_execute_query_async_requests": execute_query,
        }
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    instance = object.__new__(client.Client)
    limits = ttlv.codec_limits_create(16 * 1024 * 1024, 64, 100_000)

    try:
        absent = client.Client.execute_query_async_requests(instance, limits)
        empty = client.Client.execute_query_async_requests(
            instance, limits, correlation_values=(), operations=()
        )
        try:
            assert observed == [(0, 0, 0, 0), (1, 1, 0, 0)]
        finally:
            absent.close()
            empty.close()
    finally:
        limits.close()

    assert released == [stubs.outcome_handle, stubs.outcome_handle]


def test_query_async_request_error_wipes_every_staged_correlation(monkeypatch) -> None:
    expected = (b"first\x00\xff", b"second\x80\x00")
    observed: list[tuple[bytes, ...]] = []
    borrowed_spans: list[object] = []

    def execute_query(
        _client,
        has_correlations,
        spans,
        correlation_count,
        _has_operations,
        _operations,
        _operation_count,
        _has_indicator,
        _indicator,
        _limits,
        _output,
    ) -> int:
        assert int(has_correlations) == 1
        values = tuple(
            bytes(ffi.buffer(spans[index].data, int(spans[index].length)))
            for index in range(int(correlation_count))
        )
        observed.append(values)
        borrowed_spans.extend(spans[index].data for index in range(int(correlation_count)))
        return 4

    def last_error(status, category, cause, delivery) -> int:
        status[0] = 4
        category[0] = int(errors.ClientErrorCategory.VALIDATION)
        cause[0] = int(errors.ClientErrorCause.INVALID_INPUT)
        delivery[0] = int(errors.DeliveryState.NOT_SENT)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_query_async_requests=execute_query,
            kmipkit_last_error_get=last_error,
        ),
    )
    monkeypatch.setattr(client.Client, "_pointer", lambda _self: object())
    limits = ttlv.codec_limits_create(1024, 32, 16)
    instance = object.__new__(client.Client)

    try:
        with pytest.raises(errors.ClientExecutionError) as failure:
            client.Client.execute_query_async_requests(
                instance, limits, correlation_values=expected
            )
        assert failure.value.delivery is errors.DeliveryState.NOT_SENT
    finally:
        limits.close()

    assert observed == [expected]
    assert [bytes(ffi.buffer(pointer, len(value))) for pointer, value in zip(borrowed_spans, expected)] == [
        bytes(len(value)) for value in expected
    ]


def test_async_outcome_preserves_present_optional_metadata_and_unknown_outcome(
    monkeypatch,
) -> None:
    released: list[object] = []
    stubs = _outcome_stubs(0x39, released, cancellation_result=0)
    message = b"server result message"
    message_storage = ffi.new("uint8_t[]", message)

    def metadata(_outcome, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_async_operation_metadata_v1_t")
        output.abi_version = 1
        output.operation = 0x39
        output.outcome_kind = 0xF000_0003
        output.result_status = 1
        output.has_result_reason = 1
        output.result_reason = 0x8000_0001
        output.has_cancellation_result = 1
        output.cancellation_result = 0
        output.has_result_message = 1
        output.reserved = 0
        output.result_message_data = message_storage
        output.result_message_length = len(message)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            **{
                **stubs.__dict__,
                "kmipkit_client_async_operation_outcome_metadata_v1": metadata,
            }
        ),
    )
    outcome = client._wrap_owned_handle(client.AsyncOperationOutcome, stubs.outcome_handle)
    try:
        copied = outcome.metadata_v1
        assert copied.operation == 0x39
        assert copied.outcome == 0xF000_0003
        assert copied.result_status == 1
        assert copied.result_reason == 0x8000_0001
        assert copied.cancellation_result == 0
        assert copied.result_message == message.decode()
        assert message.decode() not in repr(copied)
    finally:
        outcome.close()

    assert released == [stubs.outcome_handle]


@pytest.mark.parametrize(
    ("field", "invalid_value"),
    [
        pytest.param("has_result_reason", 2, id="reason-presence"),
        pytest.param("has_cancellation_result", 2, id="cancellation-presence"),
        pytest.param("has_result_message", 2, id="message-presence"),
    ],
)
def test_async_outcome_rejects_invalid_optional_metadata_presence_flags(
    monkeypatch, field: str, invalid_value: int
) -> None:
    released: list[object] = []
    stubs = _outcome_stubs(0x1A, released)
    original_metadata = stubs.kmipkit_client_async_operation_outcome_metadata_v1

    def malformed_metadata(outcome, output) -> int:
        original_metadata(outcome, output)
        setattr(output, field, invalid_value)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            **{
                **stubs.__dict__,
                "kmipkit_client_async_operation_outcome_metadata_v1": malformed_metadata,
            }
        ),
    )
    outcome = client._wrap_owned_handle(client.AsyncOperationOutcome, stubs.outcome_handle)
    try:
        with pytest.raises(errors.InvalidInputError):
            _ = outcome.metadata_v1
    finally:
        outcome.close()
    assert released == [stubs.outcome_handle]
