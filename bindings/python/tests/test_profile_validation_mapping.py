from __future__ import annotations

import socket
import threading
from pathlib import Path

import pytest

from _test_support import mtls_transport_config
from kmipkit import client, errors, extensions, ttlv

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


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
    with (
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
    peer.join(timeout=2)
    assert not peer.is_alive(), "the connection observer must finish"
    assert accepted == [False], "incomplete Baseline readiness must make zero TCP connections"
