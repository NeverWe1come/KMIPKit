from __future__ import annotations

import socket
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest

from _test_support import mtls_transport_config
from kmipkit import (
    ProfileValidationError,
    _handles,
    client,
    errors,
    extensions,
    messages,
    ttlv,
)

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


def _install_profile_validation_error_stub(
    monkeypatch: pytest.MonkeyPatch,
    *,
    error_code: bytes | None = b"profile_error",
    profile_id: bytes | None = None,
    requirement_id: bytes | None = None,
    source_id: bytes | None = None,
    source_locator: bytes | None = None,
    source_kind: int = 0,
    index_present: int = 0,
    batch_item_index: int = 0,
    get_status: int = 0,
    view_status: int = 0,
    null_handle: bool = False,
) -> list[object]:
    released: list[object] = []
    owners: list[object] = []
    handle_type = client.ffi.typeof("kmipkit_profile_validation_error_t *")
    handle = client.ffi.NULL if null_handle else client.ffi.cast(handle_type, 1)

    def get_error(output: object) -> int:
        output[0] = handle  # type: ignore[index]
        return get_status

    def view_error(_handle: object, output: object) -> int:
        if view_status != 0:
            return view_status

        def span(data: bytes | None) -> tuple[object, int]:
            if data is None:
                return client.ffi.NULL, 0
            owner = client.ffi.new("uint8_t[]", data)
            owners.append(owner)
            pointer = client.ffi.cast("const uint8_t *", owner)
            return pointer, len(data)

        for prefix, data in (
            ("error_code", error_code),
            ("profile_id", profile_id),
            ("requirement_id", requirement_id),
            ("source_id", source_id),
            ("source_locator", source_locator),
        ):
            pointer, length = span(data)
            setattr(output[0], f"{prefix}_data", pointer)  # type: ignore[index]
            setattr(output[0], f"{prefix}_length", length)  # type: ignore[index]
        output[0].source_kind = source_kind  # type: ignore[index,attr-defined]
        output[0].batch_item_index_present = index_present  # type: ignore[index,attr-defined]
        output[0].batch_item_index = batch_item_index  # type: ignore[index,attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_last_error_profile_validation_error_get_v1=get_error,
            kmipkit_profile_validation_error_view_get_v1=view_error,
            kmipkit_profile_validation_error_release=lambda error: released.append(error),
        ),
    )
    return released


@pytest.mark.parametrize(
    ("source_kind", "source_id", "source_locator", "expected_clause", "expected_gap"),
    [
        (0, None, None, None, None),
        (1, b"clause-1", None, "clause-1", None),
        (2, b"gap-1", b"docs/gap-1", None, "gap-1"),
    ],
)
def test_profile_validation_error_view_converts_all_source_kinds_and_releases(
    monkeypatch: pytest.MonkeyPatch,
    source_kind: int,
    source_id: bytes | None,
    source_locator: bytes | None,
    expected_clause: str | None,
    expected_gap: str | None,
) -> None:
    released = _install_profile_validation_error_stub(
        monkeypatch,
        profile_id=b"profile-1",
        requirement_id=b"requirement-1",
        source_id=source_id,
        source_locator=source_locator,
        source_kind=source_kind,
        index_present=1,
        batch_item_index=7,
    )

    result = client._read_profile_validation_details()

    assert result == errors.ProfileValidationDetails(
        error_code="profile_error",
        profile_id="profile-1",
        requirement_id="requirement-1",
        source_clause_id=expected_clause,
        source_gap_id=expected_gap,
        source_gap_locator=source_locator.decode() if source_locator else None,
        batch_item_index=7,
    )
    assert len(released) == 1


@pytest.mark.parametrize(
    "record",
    [
        {"source_kind": 0, "source_id": b"unexpected"},
        {"source_kind": 1},
        {"source_kind": 1, "source_id": b"clause", "source_locator": b"unexpected"},
        {"source_kind": 2, "source_locator": b"locator"},
        {"source_kind": 2, "source_id": b"gap"},
        {"source_kind": 3},
        {"index_present": 2},
        {"index_present": 0, "batch_item_index": 1},
        {"error_code": None},
        {"error_code": b"\xff"},
        {"profile_id": b""},
    ],
)
def test_profile_validation_error_view_rejects_malformed_metadata_and_releases(
    monkeypatch: pytest.MonkeyPatch,
    record: dict[str, object],
) -> None:
    released = _install_profile_validation_error_stub(monkeypatch, **record)

    assert client._read_profile_validation_details() is None
    assert len(released) == 1


def test_profile_validation_error_view_handles_absent_error_and_failed_view(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    released = _install_profile_validation_error_stub(monkeypatch, get_status=1)
    assert client._read_profile_validation_details() is None
    assert released == []

    released = _install_profile_validation_error_stub(monkeypatch, null_handle=True)
    assert client._read_profile_validation_details() is None
    assert released == []

    released = _install_profile_validation_error_stub(monkeypatch, view_status=1)
    assert client._read_profile_validation_details() is None
    assert len(released) == 1


def test_profile_transport_values_are_closed_and_match_the_abi() -> None:
    assert client.ProfileTransport.RAW_TTLV_TLS.value == 1
    assert client.ProfileTransport.HTTPS.value == 2
    with pytest.raises(ValueError):
        client.ProfileTransport(3)


def test_profile_validation_types_expose_caller_owned_state() -> None:
    selection = client.ProfileValidationSelection(("baseline_client",))
    state = (
        client.ProfileObjectState("object-7")
        .with_original_creation_date_present(False)
        .with_usage_limits_present(True)
        .with_protection_use_started(False)
        .with_get_usage_allocation_performed(True)
        .with_get_usage_allocation_changed_count(False)
        .with_usage_limits_total(100)
        .with_usage_limits_count(7)
    )
    context = client.ProfileValidationContext().with_object_state(0, state)

    assert selection.profile_ids == ("baseline_client",)
    assert context.object_states == ((0, state),)
    assert "object-7" not in repr(state)


def test_profile_validation_models_reject_invalid_caller_state() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.ProfileValidationSelection(("profile-a", object()))
    with pytest.raises(errors.ResourceLimitError):
        client.ProfileValidationSelection(("x" * 4_097,))
    with pytest.raises(errors.ResourceLimitError):
        client.ProfileValidationSelection(tuple("profile" for _ in range(4_097)))

    with pytest.raises(errors.InvalidInputError):
        client.ProfileObjectState(object())
    with pytest.raises(errors.ResourceLimitError):
        client.ProfileObjectState("é" * 2_049)
    with pytest.raises(errors.InvalidInputError):
        client.ProfileObjectState("object-7").with_usage_limits_present(1)

    with pytest.raises(errors.InvalidInputError):
        client.ProfileValidationContext(object())
    with pytest.raises(errors.InvalidInputError):
        client.ProfileValidationContext(((0, object()),))
    state = client.ProfileObjectState("object-7")
    with pytest.raises(errors.ResourceLimitError):
        client.ProfileValidationContext(((0, state),) * 100_001)
    with pytest.raises(errors.InvalidInputError):
        client.ProfileValidationContext().with_object_state(
            True, state
        )
    with pytest.raises(errors.InvalidInputError):
        client.ProfileValidationContext().with_object_state(0, object())


def test_profile_validation_records_preserve_empty_selection_and_context() -> None:
    selection_record, context_record, options_record, owners, wipe_buffers = (
        client._profile_validation_records(
            client.ProfileValidationSelection(()),
            client.ProfileValidationContext(),
            None,
        )
    )

    assert selection_record.profile_id_count == 0
    assert selection_record.profile_id_data == client.ffi.NULL
    assert selection_record.profile_id_offsets == client.ffi.NULL
    assert selection_record.profile_id_lengths == client.ffi.NULL
    assert context_record != client.ffi.NULL
    assert context_record.object_state_count == 0
    assert context_record.object_states == client.ffi.NULL
    assert options_record == client.ffi.NULL
    assert owners == []
    assert wipe_buffers == []


def test_profile_validation_result_exposes_all_rust_backed_validators() -> None:
    assert callable(client.ProfileValidationSelection.validate)
    result_type = client.ProfileValidationResult
    assert callable(result_type.close)
    assert callable(result_type.validate_transport)
    assert callable(result_type.validate_request)
    assert callable(result_type.validate_request_with_context)
    assert callable(result_type.validate_response)
    assert callable(result_type.validate_response_with_context)
    assert callable(result_type.__enter__)
    assert callable(result_type.__exit__)
    assert hasattr(result_type, "profile_ids")
    assert hasattr(result_type, "requirement_ids")


def test_client_cffi_declares_the_profile_validation_result_abi() -> None:
    client.ffi.typeof("kmipkit_profile_validation_result_t *")
    for name in (
        "kmipkit_profile_validation_selection_validate_v1",
        "kmipkit_profile_validation_result_release",
        "kmipkit_profile_validation_result_profile_id_count_v1",
        "kmipkit_profile_validation_result_profile_id_at_v1",
        "kmipkit_profile_validation_result_requirement_id_count_v1",
        "kmipkit_profile_validation_result_requirement_id_at_v1",
        "kmipkit_profile_validation_result_validate_transport_v1",
        "kmipkit_profile_validation_result_validate_request_v1",
        "kmipkit_profile_validation_result_validate_request_with_context_v1",
        "kmipkit_profile_validation_result_validate_response_v1",
        "kmipkit_profile_validation_result_validate_response_with_context_v1",
    ):
        assert callable(getattr(client.lib, name))


def test_profile_validation_records_copy_tri_state_context_into_one_call() -> None:
    selection = client.ProfileValidationSelection(("baseline_client",))
    state = (
        client.ProfileObjectState("caller-object-id")
        .with_usage_limits_present(True)
        .with_usage_limits_total(100)
        .with_usage_limits_count(7)
    )
    context = client.ProfileValidationContext().with_object_state(3, state)
    selection_record, context_record, options_record, owners, wipe_buffers = (
        client._profile_validation_records(selection, context, None)
    )

    assert selection_record.profile_id_count == 1
    assert bytes(
        client.ffi.buffer(
            selection_record.profile_id_data,
            selection_record.profile_id_data_length,
        )
    ) == b"baseline_client"
    assert context_record.object_state_count == 1
    assert context_record.object_states[0].batch_item_index == 3
    assert context_record.object_states[0].known_fields == 98
    assert context_record.object_states[0].boolean_values == 2
    assert context_record.object_states[0].usage_limits_total == 100
    assert context_record.object_states[0].usage_limits_count == 7
    assert options_record == client.ffi.NULL
    assert owners
    assert len(wipe_buffers) == 1
    for buffer, length in wipe_buffers:
        client.ffi.buffer(buffer, length)[:] = b"\x00" * length


def test_cffi_profile_execution_symbol_clears_outputs_on_invalid_handles() -> None:
    response = client.ffi.new("kmipkit_client_batch_response_t **")
    response[0] = client.ffi.cast("kmipkit_client_batch_response_t *", 1)
    status = client.lib.kmipkit_client_execute_with_profiles(
        client.ffi.NULL,
        client.ffi.NULL,
        client.ffi.NULL,
        client.ffi.NULL,
        response,
    )

    assert status == 4
    assert response[0] == client.ffi.NULL


def test_client_exposes_explicit_profile_exchange_methods() -> None:
    assert callable(client.Client.execute_with_profiles)
    assert callable(client.Client.execute_with_profiles_and_options)
    assert callable(client.Client.execute_with_profiles_and_context)
    assert callable(client.Client.execute_with_profiles_and_context_and_options)


def test_baseline_selection_fails_closed_before_a_cffi_transport_connection() -> None:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(1.0)
    port = listener.getsockname()[1]
    accepted: list[bool] = []

    def observe_connection() -> None:
        try:
            connection, _address = listener.accept()
        except TimeoutError:
            accepted.append(False)
        else:
            accepted.append(True)
            connection.close()
        finally:
            listener.close()

    peer = threading.Thread(target=observe_connection, daemon=True)
    peer.start()
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    selection = client.ProfileValidationSelection(
        ("KMIPKIT-PROFILE-BASELINE-CLIENT",)
    )
    batch = client.Batch.create()
    batch.add_item(extensions.client_batch_item_discover_versions())
    unknown_selection = client.ProfileValidationSelection(
        ("unpublished-profile-selector-sentinel",)
    )
    unknown_batch = client.Batch.create()
    unknown_batch.add_item(extensions.client_batch_item_discover_versions())
    with (
        batch,
        unknown_batch,
        configuration,
        client.Client.create(
            configuration, mtls_transport_config(port, _TLS_FIXTURES)
        ) as kmip,
        ttlv.codec_limits_defaults() as limits,
    ):
        with pytest.raises(errors.ClientExecutionError) as failure:
            kmip.execute_with_profiles(batch, limits, selection)
        assert failure.value.client_category is errors.ClientErrorCategory.VALIDATION
        assert failure.value.delivery is errors.DeliveryState.NOT_SENT
        details = failure.value.profile_validation
        assert details == errors.ProfileValidationDetails(
            error_code="profile_unavailable",
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
            requirement_id="KMIPKIT-REQ-SPEC-14.1-001",
            source_clause_id="KMIPKIT-CLAUSE-SPEC-14.1-001",
            source_gap_id=None,
            source_gap_locator=None,
            batch_item_index=None,
        )
        assert "KMIPKIT-PROFILE-BASELINE-CLIENT" not in str(failure.value)

        with pytest.raises(errors.ClientExecutionError) as unknown_failure:
            kmip.execute_with_profiles(unknown_batch, limits, unknown_selection)
        assert unknown_failure.value.client_category is errors.ClientErrorCategory.VALIDATION
        assert unknown_failure.value.delivery is errors.DeliveryState.NOT_SENT
        unknown_details = unknown_failure.value.profile_validation
        assert unknown_details == errors.ProfileValidationDetails(
            error_code="unknown_profile",
            profile_id=None,
            requirement_id=None,
            source_clause_id=None,
            source_gap_id=None,
            source_gap_locator=None,
            batch_item_index=None,
        )
        assert "unpublished-profile-selector-sentinel" not in str(unknown_failure.value)

    peer.join(timeout=2)
    assert not peer.is_alive(), "the connection observer must finish"
    assert accepted == [False], "incomplete Baseline readiness must make zero TCP connections"


class _ProfileValidationLib:
    """Narrow native-boundary seam for wrapper ownership and marshalling tests."""

    def __init__(self, *, status: int = 0, validator_status: int | None = None) -> None:
        self.status = status
        self.validator_status = validator_status
        self.profile_ids = ("profile-a", "profile-b")
        self.requirement_ids = ("KMIPKIT-REQ-A", "KMIPKIT-REQ-B")
        self.calls: list[tuple[str, tuple[object, ...]]] = []
        self.released: list[object] = []
        self.span_owners: list[object] = []
        self.context_bytes_during_call: bytes | None = None
        self.context_values_during_calls: list[bytes] = []
        self.context_buffer: object | None = None
        self.context_length = 0
        self.last_error = (0, 0, 0, 0)

    def kmipkit_profile_validation_selection_validate_v1(self, selection, out_result):
        self.calls.append(("selection_validate", (selection,)))
        if self.status == 0:
            out_result[0] = self._pointer_for_slot(out_result, 101)
        return self.status

    def kmipkit_profile_validation_result_profile_id_count_v1(self, result, out_count):
        out_count[0] = len(self.profile_ids)
        return 0

    def kmipkit_profile_validation_result_profile_id_at_v1(
        self, result, index, out_span
    ):
        return self._write_span(self.profile_ids, index, out_span)

    def kmipkit_profile_validation_result_requirement_id_count_v1(
        self, result, out_count
    ):
        out_count[0] = len(self.requirement_ids)
        return 0

    def kmipkit_profile_validation_result_requirement_id_at_v1(
        self, result, index, out_span
    ):
        return self._write_span(self.requirement_ids, index, out_span)

    def kmipkit_profile_validation_result_release(self, result):
        self.released.append(result)

    @staticmethod
    def kmipkit_ttlv_structure_release(handle):
        ttlv.lib.kmipkit_ttlv_structure_release(handle)

    def kmipkit_profile_validation_result_validate_transport_v1(
        self, result, transport
    ):
        return self._record_validator("transport", result, transport)

    def kmipkit_profile_validation_result_validate_request_v1(self, result, request):
        return self._record_validator("request", result, request)

    def kmipkit_profile_validation_result_validate_request_with_context_v1(
        self, result, request, context
    ):
        self._capture_context(context)
        return self._record_validator("request_with_context", result, request, context)

    def kmipkit_profile_validation_result_validate_response_v1(
        self, result, request, response
    ):
        return self._record_validator("response", result, request, response)

    def kmipkit_profile_validation_result_validate_response_with_context_v1(
        self, result, request, response, context
    ):
        self._capture_context(context)
        return self._record_validator(
            "response_with_context", result, request, response, context
        )

    def kmipkit_last_error_get(self, status, category, cause, delivery):
        status[0], category[0], cause[0], delivery[0] = self.last_error
        return 0

    @staticmethod
    def _pointer_for_slot(slot, address: int):
        return client.ffi.cast(client.ffi.typeof(slot[0]), address)

    def _write_span(self, values, index, out_span):
        encoded = values[int(index)].encode("utf-8")
        buffer = client.ffi.new("uint8_t[]", encoded)
        self.span_owners.append(buffer)
        out_span.data = buffer
        out_span.length = len(encoded)
        return 0

    def _record_validator(self, name: str, *arguments: object) -> int:
        self.calls.append((name, arguments))
        return self.status if self.validator_status is None else self.validator_status

    def _capture_context(self, context) -> None:
        if context.object_state_count:
            state = context.object_states[0]
            self.context_buffer = state.unique_identifier_data
            self.context_length = int(state.unique_identifier_length)
            self.context_bytes_during_call = client.ffi.buffer(
                self.context_buffer, self.context_length
            )[:]
            self.context_values_during_calls.append(self.context_bytes_during_call)


def _profile_result_from_mock(monkeypatch, native: _ProfileValidationLib):
    try:
        client.ffi.typeof("kmipkit_profile_validation_result_t *")
    except Exception:
        client.ffi.cdef(
            "typedef struct kmipkit_profile_validation_result_t "
            "kmipkit_profile_validation_result_t;"
        )
    monkeypatch.setattr(client, "lib", native)
    monkeypatch.setattr(_handles, "lib", native)
    return client.ProfileValidationSelection(("profile-a", "profile-b")).validate()


def test_profile_selection_resolves_ordered_ids_into_an_immutable_snapshot(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)

    assert result.profile_ids == ("profile-a", "profile-b")
    assert result.requirement_ids == ("KMIPKIT-REQ-A", "KMIPKIT-REQ-B")
    assert isinstance(result.profile_ids, tuple)
    assert isinstance(result.requirement_ids, tuple)
    with pytest.raises(AttributeError):
        result.profile_ids = ("changed",)
    with pytest.raises(AttributeError):
        result.profile_ids.append("changed")
    result.close()
    assert result.profile_ids == ("profile-a", "profile-b")


def test_profile_result_calls_all_validators_with_live_context_buffers(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    request = ttlv.ttlv_structure_create()
    response = ttlv.ttlv_structure_create()
    context = client.ProfileValidationContext().with_object_state(
        0, client.ProfileObjectState("caller-context-secret")
    )

    result.validate_transport(client.ProfileTransport.HTTPS)
    result.validate_request(request)
    result.validate_request_with_context(request, context)
    result.validate_response(request, response)
    result.validate_response_with_context(request, response, context)

    assert [name for name, _ in native.calls[1:]] == [
        "transport",
        "request",
        "request_with_context",
        "response",
        "response_with_context",
    ]
    assert native.calls[1][1][1] == client.ProfileTransport.HTTPS.value
    assert native.context_bytes_during_call == b"caller-context-secret"
    assert native.context_values_during_calls == [
        b"caller-context-secret",
        b"caller-context-secret",
    ]
    assert client.ffi.buffer(native.context_buffer, native.context_length)[:] == bytes(
        native.context_length
    )

    request.close()
    response.close()
    result.close()


def test_profile_context_bytes_are_wiped_after_a_validation_error(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    request = ttlv.ttlv_structure_create()
    secret_identifier = "failed-validation-context"
    context = client.ProfileValidationContext().with_object_state(
        4, client.ProfileObjectState(secret_identifier)
    )

    native.validator_status = 4
    try:
        with pytest.raises(errors.InvalidInputError):
            result.validate_request_with_context(request, context)
        assert native.context_bytes_during_call == secret_identifier.encode()
        assert client.ffi.buffer(native.context_buffer, native.context_length)[:] == bytes(
            native.context_length
        )
    finally:
        request.close()
        result.close()


@pytest.mark.parametrize(
    ("profile_ids", "expected_error"),
    [
        (("profile\x00secret",), errors.InvalidInputError),
        (("",), errors.InvalidInputError),
        (tuple("profile" for _ in range(100_001)), errors.ResourceLimitError),
    ],
)
def test_profile_result_releases_native_handle_when_returned_ids_are_invalid(
    monkeypatch, profile_ids, expected_error
):
    native = _ProfileValidationLib()
    native.profile_ids = profile_ids
    result_pointer = client.ffi.cast("kmipkit_profile_validation_result_t *", 101)

    def stable_result_pointer(slot, _address: int):
        return client.ffi.cast(client.ffi.typeof(slot[0]), result_pointer)

    native._pointer_for_slot = stable_result_pointer
    monkeypatch.setattr(client, "lib", native)
    monkeypatch.setattr(_handles, "lib", native)

    with pytest.raises(expected_error):
        client.ProfileValidationSelection(("profile-a",)).validate()

    assert native.released == [result_pointer]


def test_profile_result_keeps_input_handle_locked_from_pointer_read_to_native_call(
    monkeypatch,
):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    request = ttlv.ttlv_structure_create()
    pointer_read = threading.Event()
    allow_pointer_return = threading.Event()
    close_started = threading.Event()
    close_finished = threading.Event()
    failures: list[BaseException] = []
    original_pointer = messages._as_client_structure

    def pause_after_pointer_read(structure):
        pointer = original_pointer(structure)
        pointer_read.set()
        if not allow_pointer_return.wait(2):
            raise AssertionError("native validation was never released")
        return pointer

    def observe_close() -> None:
        close_started.set()
        request.close()
        close_finished.set()

    monkeypatch.setattr(
        messages, "_as_client_structure", pause_after_pointer_read
    )
    original_validator = native.kmipkit_profile_validation_result_validate_request_v1

    def validate_while_handle_is_live(result_handle, request_handle):
        assert not request.closed
        return original_validator(result_handle, request_handle)

    native.kmipkit_profile_validation_result_validate_request_v1 = (
        validate_while_handle_is_live
    )

    def validate() -> None:
        try:
            result.validate_request(request)
        except BaseException as error:
            failures.append(error)

    validation_thread = threading.Thread(target=validate, daemon=True)
    close_thread = threading.Thread(target=observe_close, daemon=True)
    validation_thread.start()
    try:
        assert pointer_read.wait(2)
        close_thread.start()
        assert close_started.wait(2)
        assert not close_finished.wait(0.05), "close must wait for the borrowed pointer"
    finally:
        allow_pointer_return.set()
        validation_thread.join(timeout=2)
        close_thread.join(timeout=2)
        result.close()

    assert not validation_thread.is_alive()
    assert not close_thread.is_alive()
    assert failures == []
    assert close_finished.is_set()
    assert request.closed


def test_profile_validation_protocol_status_uses_client_error_mapping(monkeypatch):
    native = _ProfileValidationLib()
    native.last_error = (8, int(errors.ClientErrorCategory.PROTOCOL), 4, 0)
    monkeypatch.setattr(client, "lib", native)
    monkeypatch.setattr(client, "_read_profile_validation_details", lambda: None)

    with pytest.raises(errors.ClientExecutionError) as failure:
        client._raise_profile_validation_status(8)

    assert failure.value.status == 8
    assert failure.value.client_category is errors.ClientErrorCategory.PROTOCOL
    assert failure.value.cause is errors.ClientErrorCause.PROTOCOL_OTHER
    assert failure.value.delivery is errors.DeliveryState.NONE


def test_profile_transport_rejects_unknown_values_before_native_call(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    before = len(native.calls)

    with pytest.raises(errors.InvalidInputError):
        result.validate_transport(999)

    assert len(native.calls) == before
    result.close()


def test_profile_result_rejects_non_ttlv_inputs_before_native_call(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    before = len(native.calls)

    with pytest.raises(errors.InvalidInputError):
        result.validate_request(object())

    assert len(native.calls) == before
    result.close()


def test_profile_result_context_manager_and_repeated_close_release_once(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    handle = result._pointer()

    with result as entered:
        assert entered is result
        assert not result.closed
    result.close()

    assert result.closed
    assert native.released.count(handle) == 1


def test_profile_validation_error_keeps_details_and_redacts_caller_values(monkeypatch):
    native = _ProfileValidationLib(validator_status=4)
    details = errors.ProfileValidationDetails(
        error_code="profile_unavailable",
        profile_id=None,
        requirement_id=None,
        source_clause_id=None,
        source_gap_id=None,
        source_gap_locator=None,
        batch_item_index=None,
    )
    monkeypatch.setattr(client, "lib", native)
    monkeypatch.setattr(client, "_read_profile_validation_details", lambda: details)
    caller_context_value = "caller-context-secret"
    raw_body = b"raw-kmip-body-secret"
    result = _profile_result_from_mock(monkeypatch, native)
    request = ttlv.ttlv_structure_create()
    request._owner = raw_body
    context = client.ProfileValidationContext().with_object_state(
        0, client.ProfileObjectState(caller_context_value)
    )

    with pytest.raises(errors.ProfileValidationError) as failure:
        result.validate_request_with_context(request, context)

    assert failure.value.profile_validation is details
    assert failure.value.details is details
    assert isinstance(failure.value, ProfileValidationError)
    rendered = f"{failure.value!r} {failure.value} {details!r}"
    assert caller_context_value not in rendered
    assert raw_body.decode() not in rendered
    request.close()
    result.close()
