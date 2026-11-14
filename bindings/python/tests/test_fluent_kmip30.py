"""Public Python fluent surface and execution contract for KMIP 3.0 routes."""

from __future__ import annotations

import inspect
import queue
import socket
import ssl
import struct
import threading
from collections.abc import Callable
from pathlib import Path

import pytest
from _test_support import mtls_transport_config, read_ttlv_frame, require_tls13_mtls_peer
from kmipkit import (
    attribute_operations,
    client,
    cryptographic_parameters,
    errors,
    extensions,
    fluent,
    ttlv,
)
from kmipkit.credentials import SecretBytes, SecretText

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_CREATE_GROUP = 0x3C
_CORRELATION = b"pending-correlation"
_FAILURE_MESSAGE = "private-response-diagnostic"
_DECAPSULATE_SENTINEL = b"server-managed-shared-secret"


def test_request_exposes_every_kmip30_operation_builder() -> None:
    expected = (
        "create_credential",
        "create_group",
        "create_user",
        "deactivate",
        "decapsulate",
        "encapsulate",
        "obliterate",
    )
    for name in expected:
        assert callable(getattr(fluent.Request, name, None)), name


def test_create_credential_infers_type_from_closed_object_alternative() -> None:
    parameters = tuple(inspect.signature(fluent.Request.create_credential).parameters)
    assert parameters == ("self", "attributes", "credential_object")
    assert not hasattr(fluent.CreateCredentialObject, "credential_type")


def test_request_rejects_invalid_client_before_creating_operation_handles() -> None:
    with pytest.raises(errors.InvalidInputError):
        fluent.Request(None)  # type: ignore[arg-type]


def test_hashed_password_wipes_first_copy_if_second_copy_fails(monkeypatch: pytest.MonkeyPatch) -> None:
    first = bytearray(b"first-secret")
    wiped: list[bytearray] = []
    copies = 0

    def copy_secret(_value: object) -> bytearray:
        nonlocal copies
        copies += 1
        if copies == 1:
            return first
        raise RuntimeError("controlled second-copy failure")

    def wipe_secret(value: bytearray) -> None:
        value[:] = b"\0" * len(value)
        wiped.append(value)

    monkeypatch.setattr(fluent, "_secret_bytes_copy", copy_secret)
    monkeypatch.setattr(fluent, "_wipe_crypto_stage", wipe_secret)
    with pytest.raises(RuntimeError, match="controlled second-copy failure"):
        fluent.CreateCredentialObject.hashed_password(object(), object())  # type: ignore[arg-type]
    assert first == bytearray(len(first))
    assert wiped == [first]


def test_every_assigned_route_validates_and_builds_its_typed_operation() -> None:
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    attributes = _empty_attributes()
    password = SecretText.from_text("credential-secret")
    credential = fluent.CreateCredentialObject.password(password=password)
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(1, _FIXTURES)
        ) as kmip_client:
            identifier = extensions.UniqueIdentifier.text_string("kmip30-target")
            builders = [
                kmip_client.request().create_credential(attributes, credential),
                kmip_client.request().create_group(attributes),
                kmip_client.request().create_user(attributes),
                kmip_client.request().deactivate(identifier),
                kmip_client.request().decapsulate(identifier, b"ciphertext"),
                kmip_client.request().encapsulate(identifier),
                kmip_client.request().obliterate(identifier),
            ]
            try:
                expected = (0x3F, 0x3C, 0x3E, 0x40, 0x42, 0x41, 0x3D)
                for builder, operation in zip(builders, expected, strict=True):
                    builder.validate()
                    with builder.to_batch_item() as item:
                        assert item.operation == operation
            finally:
                for builder in builders:
                    builder.close()
        assert "credential-secret" not in repr(credential)
    finally:
        credential.close()
        password.close()
        attributes.close()


def test_credential_alternatives_and_optional_kmip30_inputs_reach_typed_abi() -> None:
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    attributes = _empty_attributes()
    first_secret = SecretBytes.from_bytes(b"first-secret")
    second_secret = SecretBytes.from_bytes(b"second-secret")
    otp_seed = SecretBytes.from_bytes(b"otp-seed")
    password_text = SecretText.from_text("credential-password")
    certificate_item = _certificate_item()
    objects = [
        fluent.CreateCredentialObject.password(password=password_text),
        fluent.CreateCredentialObject.certificate(certificate_item),
        fluent.CreateCredentialObject.device(
            serial_number="serial",
            device_identifier="device-id",
            network_identifier="network-id",
            media_identifier="media-id",
        ),
        fluent.CreateCredentialObject.hashed_password(
            first_secret, second_secret, hashing_algorithm=1
        ),
        fluent.CreateCredentialObject.one_time_password(
            1, digest=1, serial="otp-serial", seed=otp_seed, interval=30, digits=6
        ),
    ]
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(1, _FIXTURES)
        ) as kmip_client:
            for credential in objects:
                builder = kmip_client.request().create_credential(attributes, credential)
                try:
                    builder.validate()
                    with builder.to_batch_item() as item:
                        assert item.operation == 0x3F
                finally:
                    builder.close()

            identifier = extensions.UniqueIdentifier.text_string("kmip30-target")
            reason = ttlv.ttlv_structure_create()
            parameters = (
                cryptographic_parameters.CryptographicParameters.builder()
                .cryptographic_algorithm(1)
                .build()
            )
            deactivate = kmip_client.request().deactivate(identifier)
            encapsulate = kmip_client.request().encapsulate(identifier)
            try:
                deactivate.deactivation_reason(reason).deactivation_date(1_700_000_000)
                encapsulate.attributes(attributes).cryptographic_parameters(parameters)
                deactivate.validate()
                encapsulate.validate()
                with deactivate.to_batch_item() as deactivate_item:
                    assert deactivate_item.operation == 0x40
                with encapsulate.to_batch_item() as encapsulate_item:
                    assert encapsulate_item.operation == 0x41
            finally:
                encapsulate.close()
                deactivate.close()
                parameters.close()
                reason.close()
    finally:
        for credential in objects:
            credential.close()
        certificate_item.close()
        password_text.close()
        otp_seed.close()
        second_secret.close()
        first_secret.close()
        attributes.close()


def test_kmip30_fluent_rejects_wrong_credential_and_optional_field_types() -> None:
    _assert_invalid_input(lambda: fluent.CreateCredentialObject.certificate(object()))
    secret = SecretBytes.from_bytes(b"hashed-password")
    try:
        _assert_invalid_input(
            lambda: fluent.CreateCredentialObject.hashed_password(secret, None)
        )
    finally:
        secret.close()

    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    attributes = _empty_attributes()
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(1, _FIXTURES)
        ) as kmip_client:
            _assert_invalid_input(
                lambda: kmip_client.request().create_credential(attributes, object())
            )
            identifier = extensions.UniqueIdentifier.text_string("kmip30-invalid-input")
            with kmip_client.request().deactivate(identifier) as deactivate:
                _assert_invalid_input(
                    lambda: deactivate.deactivation_reason(object())
                )
                _assert_invalid_input(lambda: deactivate.deactivation_date(True))
                _assert_invalid_input(lambda: deactivate.deactivation_date(1 << 63))
            with kmip_client.request().encapsulate(identifier) as encapsulate:
                _assert_invalid_input(
                    lambda: encapsulate.cryptographic_parameters(object())
                )
    finally:
        attributes.close()
        registry.close()


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((item_type,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def _assert_invalid_input(action: Callable[[], object]) -> None:
    with pytest.raises(errors.InvalidInputError):
        action()


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _integer(tag: int, value: int) -> bytes:
    return _item(tag, 2, struct.pack(">i", value))


def _enumeration(tag: int, value: int) -> bytes:
    return _item(tag, 5, struct.pack(">I", value))


def _response(
    status: int,
    *, operation: int = _CREATE_GROUP,
    reason: int | None = None,
    message: str | None = None,
) -> bytes:
    version = _structure(0x420069, _integer(0x42006A, 3), _integer(0x42006B, 0))
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _integer(0x42000D, 1),
    )
    response_fields = [
        _enumeration(0x42005C, operation),
        _enumeration(0x42007F, status),
    ]
    if status == 2:
        response_fields.append(_item(0x420006, 8, _CORRELATION))
    if reason is not None:
        response_fields.append(_enumeration(0x42007E, reason))
    if message is not None:
        response_fields.append(_item(0x42007D, 7, message.encode()))
    if status in (0, 2):
        payload_fields = []
        if operation != 0x3D:
            payload_fields.append(_item(0x420094, 7, b"created-group-id"))
        if operation == 0x41:
            payload_fields.append(_item(0x4200C2, 8, b"public-encapsulated-data"))
        if operation == 0x42:
            payload_fields.append(_item(0x4200C2, 8, _DECAPSULATE_SENTINEL))
        response_fields.append(_structure(0x42007C, *payload_fields))
    return _structure(0x42007B, header, _structure(0x42000F, *response_fields))


def _start_peer(
    responses: tuple[bytes, ...],
) -> tuple[int, list[bytes], queue.Queue[BaseException], threading.Thread]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(len(responses))
    listener.settimeout(12)
    port = listener.getsockname()[1]
    requests: list[bytes] = []
    failures: queue.Queue[BaseException] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            for response in responses:
                raw, _ = listener.accept()
                with raw, context.wrap_socket(raw, server_side=True) as stream:
                    require_tls13_mtls_peer(stream)
                    request = read_ttlv_frame(stream)
                    requests.append(request)
                    stream.sendall(response)
        except BaseException as error:  # noqa: BLE001 - make fixture failures visible to the test.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, name="kmipkit-python-kmip30-fixture", daemon=True)
    peer.start()
    return port, requests, failures, peer


def _empty_attributes() -> attribute_operations.AttributeSet:
    structure = ttlv.ttlv_structure_create()
    try:
        return attribute_operations.AttributeSet.try_new(structure)
    finally:
        structure.close()


def _certificate_item() -> ttlv.TtlvItem:
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    try:
        structure = ttlv.ttlv_structure_with_item(
            structure,
            _ttlv_item(0x0042001D, ttlv.ttlv_value_enumeration(1), limits),
            limits,
        )
        structure = ttlv.ttlv_structure_with_item(
            structure,
            _ttlv_item(0x0042001E, ttlv.ttlv_value_byte_string(b"cert", limits), limits),
            limits,
        )
        value = ttlv.ttlv_value_structure(structure, limits)
        structure.close()
        return _ttlv_item(0x00420013, value, limits)
    finally:
        structure.close()
        limits.close()


def _ttlv_item(raw_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    raw = ttlv.raw_tag_from_raw(raw_tag)
    tag = ttlv.raw_tag_try_checked(raw)
    try:
        return ttlv.ttlv_item_create(tag, value, limits)
    finally:
        raw.close()
        tag.close()
        value.close()


def _request_operation(request: bytes) -> int:
    offset = 8
    while offset < len(request):
        tag = int.from_bytes(request[offset : offset + 3], "big")
        length = int.from_bytes(request[offset + 4 : offset + 8], "big")
        end = offset + 8 + length + (-length % 8)
        if tag == 0x42000F:
            child = offset + 8
            while child < offset + 8 + length:
                child_tag = int.from_bytes(request[child : child + 3], "big")
                child_length = int.from_bytes(request[child + 4 : child + 8], "big")
                if child_tag == 0x42005C:
                    return int.from_bytes(request[child + 8 : child + 8 + child_length], "big")
                child += 8 + child_length + (-child_length % 8)
        offset = end
    raise AssertionError("request omitted the Batch Item operation")


def test_default_protocol_version_rejects_kmip30_before_transport() -> None:
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        unused_port = reservation.getsockname()[1]
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    attributes = _empty_attributes()
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(unused_port, _FIXTURES)
        ) as kmip_client, ttlv.codec_limits_defaults() as limits:
            with pytest.raises(errors.ClientExecutionError) as caught:
                kmip_client.request().create_group(attributes).execute(limits)
            assert caught.value.delivery is errors.DeliveryState.NOT_SENT
    finally:
        attributes.close()


def test_execute_one_exchange_preserves_outcomes_options_and_async_metadata() -> None:
    port, requests, failures, peer = _start_peer(
        tuple(
            [_response(0, operation=operation) for operation in (0x3F, 0x3C, 0x3E, 0x40)]
            + [_response(0, operation=0x42), _response(0, operation=0x41), _response(0, operation=0x3D)]
            + [_response(2), _response(1, reason=1, message=_FAILURE_MESSAGE)]
        )
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry).with_protocol_version(
        client.ProtocolVersion(3, 0)
    )
    attributes = _empty_attributes()
    results: list[client.BatchResult] = []
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client, ttlv.codec_limits_defaults() as limits:
            identifier = extensions.UniqueIdentifier.text_string("kmip30-target")
            credential_password = SecretText.from_text("credential-secret")
            credential = fluent.CreateCredentialObject.password(password=credential_password)
            create_credential = kmip_client.request().create_credential(attributes, credential)
            credential.close()
            credential_password.close()
            try:
                results.append(create_credential.execute(limits))
            finally:
                create_credential.close()
            results.append(kmip_client.request().create_group(attributes)
                           .with_options(client.RequestOptions()).execute(limits))
            results.append(kmip_client.request().create_user(attributes).execute(limits))
            results.append(kmip_client.request().deactivate(identifier).execute(limits))
            with pytest.raises(errors.ClientExecutionError) as decapsulate_error:
                kmip_client.request().decapsulate(identifier, b"ciphertext").execute(limits)
            assert decapsulate_error.value.delivery is errors.DeliveryState.RESPONSE_STARTED
            assert _DECAPSULATE_SENTINEL.decode() not in str(decapsulate_error.value)
            results.append(kmip_client.request().encapsulate(identifier).execute(limits))
            results.append(kmip_client.request().obliterate(identifier).execute(limits))
            results.append(kmip_client.request().create_group(attributes)
                           .asynchronous_indicator(1).execute(limits))
            results.append(kmip_client.request().create_group(attributes).execute(limits))
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert failures.empty()
        operations = (0x3F, 0x3C, 0x3E, 0x40, 0x42, 0x41, 0x3D, _CREATE_GROUP, _CREATE_GROUP)
        assert len(requests) == len(operations), "each execute must make exactly one transport exchange"
        assert [_request_operation(request) for request in requests] == list(operations)
        assert b"\x42\x00\x07\x05\x00\x00\x00\x04\x00\x00\x00\x01" in requests[7]
        success, pending, failed = results[1].items[0], results[6].items[0], results[7].items[0]
        for index, operation in enumerate(operations[:4]):
            result = results[index].items[0]
            assert result.operation == operation and result.result_status == 0
            assert result.kmip30_identifier == extensions.UniqueIdentifier.text_string("created-group-id")
        encapsulated = results[4].items[0]
        assert encapsulated.kmip30_identifier == extensions.UniqueIdentifier.text_string("created-group-id")
        assert encapsulated.encapsulate_data == b"public-encapsulated-data"
        assert results[5].items[0].kmip30_identifier is None
        assert (success.outcome, success.result_status, success.operation) == (
            client.BatchOutcome.COMPLETED, 0, _CREATE_GROUP
        )
        assert pending.outcome is client.BatchOutcome.PENDING
        assert pending.result_status == 2
        assert pending.pending_correlation_value == _CORRELATION
        assert failed.outcome is client.BatchOutcome.COMPLETED
        assert failed.result_status == 1
        assert failed.result_reason == 1
        assert failed.result_message == _FAILURE_MESSAGE
        assert _FAILURE_MESSAGE not in repr(failed)
    finally:
        for result in results:
            result.close()
        attributes.close()
        peer.join(timeout=12)
        if not failures.empty():
            raise failures.get()


def test_credential_builder_keeps_owned_secret_copy_and_redacts_it() -> None:
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    attributes = _empty_attributes()
    password = SecretText.from_text("credential-secret")
    salt = SecretBytes.from_bytes(b"\x09\x08\x07")
    credential = fluent.CreateCredentialObject.password(
        password=password, password_salt=salt
    )
    try:
        assert "credential-secret" not in repr(credential)
        with configuration, client.Client.create(
            configuration, mtls_transport_config(1, _FIXTURES)
        ) as kmip_client:
            builder = kmip_client.request().create_credential(attributes, credential)
            credential.close()
            password.close()
            salt.close()
            try:
                with builder.to_batch_item() as item:
                    assert item.operation == 0x3F
            finally:
                builder.close()
    finally:
        credential.close()
        password.close()
        salt.close()
        attributes.close()
