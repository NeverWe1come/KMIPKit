from __future__ import annotations

import socket
import threading
from pathlib import Path

import pytest

from _test_support import mtls_transport_config
from kmipkit import _handles, client, errors, extensions, messages, ttlv
from kmipkit._handles import _wrap_owned_handle

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


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


def test_profile_validation_records_copy_tri_state_context_into_one_call() -> None:
    selection = client.ProfileValidationSelection(("baseline_client",))
    state = client.ProfileObjectState("caller-object-id").with_usage_limits_present(True)
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
    assert context_record.object_states[0].known_fields == 2
    assert context_record.object_states[0].boolean_values == 2
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
        self.context_buffer: object | None = None
        self.context_length = 0

    def kmipkit_profile_validation_selection_validate_v1(self, selection, out_result):
        self.calls.append(("selection_validate", (selection,)))
        if self.status == 0:
            out_result[0] = self._pointer_for_slot(out_result, 101)
        return self.status

    def kmipkit_profile_validation_result_profile_id_count_v1(self, result, out_count):
        out_count[0] = len(self.profile_ids)
        return 0

    def kmipkit_profile_validation_result_profile_id_at_v1(self, result, index, out_span):
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

    def kmipkit_request_message_release(self, handle):
        self.released.append(handle)

    def kmipkit_response_message_release(self, handle):
        self.released.append(handle)

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
    monkeypatch.setattr(messages, "_native_lib", native)
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


def test_profile_result_calls_all_five_validators_and_wipes_context_buffers(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    request = _wrap_owned_handle(
        messages.RequestMessage,
        client.ffi.cast("kmipkit_request_message_t *", 102),
    )
    response = _wrap_owned_handle(
        messages.ResponseMessage,
        client.ffi.cast("kmipkit_response_message_t *", 103),
    )
    context = client.ProfileValidationContext().with_object_state(
        0, client.ProfileObjectState("caller-context-secret")
    )

    result.validate_transport(client.ProfileTransport.HTTPS)
    result.validate_request(request)
    result.validate_request_with_context(request, context)
    request_context = native.context_buffer
    request_context_length = native.context_length
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
    assert (
        client.ffi.buffer(request_context, request_context_length)[:]
        == b"\x00" * request_context_length
    )
    assert (
        client.ffi.buffer(native.context_buffer, native.context_length)[:]
        == b"\x00" * native.context_length
    )

    request.close()
    response.close()
    result.close()


def test_profile_transport_rejects_unknown_values_before_native_call(monkeypatch):
    native = _ProfileValidationLib()
    result = _profile_result_from_mock(monkeypatch, native)
    before = len(native.calls)

    with pytest.raises(errors.InvalidInputError):
        result.validate_transport(999)

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


def test_profile_validation_error_keeps_safe_details_and_redacts_caller_values(monkeypatch):
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
    request = _wrap_owned_handle(
        messages.RequestMessage,
        client.ffi.cast("kmipkit_request_message_t *", 104),
    )
    context = client.ProfileValidationContext().with_object_state(
        0, client.ProfileObjectState(caller_context_value)
    )

    with pytest.raises(errors.ProfileValidationError) as failure:
        result.validate_request_with_context(request, context)

    assert failure.value.profile_validation is details
    assert failure.value.details is details
    rendered = f"{failure.value!r} {failure.value} {details!r}"
    assert caller_context_value not in rendered
    assert raw_body.decode() not in rendered
    request.close()
    result.close()
