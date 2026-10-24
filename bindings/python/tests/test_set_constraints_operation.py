from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_SET_CONSTRAINTS_ID = b"set-constraints-0"


def _tlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((item_type,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def _tlv_structure(tag: int, *children: bytes) -> bytes:
    return _tlv_item(tag, 1, b"".join(children))


def _tlv_enumeration(tag: int, value: int) -> bytes:
    return _tlv_item(tag, 5, struct.pack(">I", value))


def _set_constraints_response_message(
    *,
    result_status: int = 0,
    result_reason: int | None = None,
    result_message: str | None = None,
    pending_correlation_value: bytes | None = None,
) -> bytes:
    version = _tlv_structure(
        0x420069,
        _tlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _tlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _tlv_structure(
        0x42007A,
        version,
        _tlv_item(0x420092, 9, struct.pack(">q", 1)),
        _tlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    response_fields = [
        _tlv_enumeration(0x42005C, 0x37),
        _tlv_item(0x420093, 8, _SET_CONSTRAINTS_ID),
        _tlv_enumeration(0x42007F, result_status),
    ]
    if result_reason is not None:
        response_fields.append(_tlv_enumeration(0x42007E, result_reason))
    if result_message is not None:
        response_fields.append(_tlv_item(0x42007D, 7, result_message.encode()))
    if pending_correlation_value is not None:
        response_fields.append(_tlv_item(0x420006, 8, pending_correlation_value))
    if result_status in (0, 2):
        response_fields.append(_tlv_structure(0x42007C))
    response_item = _tlv_structure(0x42000F, *response_fields)
    return _tlv_structure(0x42007B, header, response_item)


def _read_exact(stream: ssl.SSLSocket, size: int) -> bytes:
    output = bytearray()
    while len(output) < size:
        chunk = stream.recv(size - len(output))
        if not chunk:
            raise AssertionError("client closed before sending a complete request")
        output.extend(chunk)
    return bytes(output)


def _start_set_constraints_mtls_peer(
    response_message: bytes | None = None,
) -> tuple[int, list[bytes], queue.Queue[Exception], threading.Thread]:
    listener = socket.socket()
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    received: list[bytes] = []
    failures: queue.Queue[Exception] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _TLS_FIXTURES / "server-cert.pem", _TLS_FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_TLS_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _ = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                assert stream.version() == "TLSv1.3"
                assert stream.getpeercert(binary_form=True)
                prefix = _read_exact(stream, 8)
                message_length = int.from_bytes(prefix[4:8], "big")
                received.append(prefix + _read_exact(stream, message_length))
                stream.sendall(
                    _set_constraints_response_message()
                    if response_message is None
                    else response_message
                )
        except Exception as error:  # noqa: BLE001 - report peer failures from the fixture thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return listener.getsockname()[1], received, failures, peer


def _set_constraints_transport(port: int) -> client.TransportConfig:
    return client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(
            client.TransportInput.file(
                _TLS_FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
            ),
        ),
        client_private_key=client.TransportInput.file(
            _TLS_FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.certificate_authorities(
            (
                client.TransportInput.file(
                    _TLS_FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
                ),
            )
        ),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(8),
    )


def test_set_constraints_builder_passes_the_caller_structure_to_native_abi(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure_handle = object()
    constraints = object()
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_client_native_structure",
        lambda value, *, required: structure_handle,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 41,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_set_constraints(constraints) == 41
    assert calls == [("kmipkit_client_batch_item_set_constraints", (structure_handle,))]


def test_set_constraints_response_owns_generic_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_set_constraints_ttlv=response_ttlv
        ),
    )
    monkeypatch.setattr(
        ttlv, "_structure_from_native_address", lambda _address: structure
    )

    result = client._set_constraints_response(
        SimpleNamespace(_pointer=lambda: object())
    )

    assert isinstance(result, client.SetConstraintsResponse)
    assert result.response_ttlv is structure
    assert "123" not in repr(result)
    result.close()
    assert structure.closed


def test_set_constraints_response_releases_unexpected_native_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_set_constraints_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._set_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [native_handle]


def test_set_constraints_native_mtls_response_is_owned_and_preserves_empty_success_payload() -> (
    None
):
    port, received, failures, peer = _start_set_constraints_mtls_peer()
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    result = None
    response = None
    constraints = ttlv.ttlv_structure_create()
    try:
        with (
            configuration,
            client.Client.create(
                configuration, _set_constraints_transport(port)
            ) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            item = extensions.client_batch_item_set_constraints(constraints)
            constraints.close()
            batch.add_item(
                extensions.with_unique_batch_item_id(item, _SET_CONSTRAINTS_ID)
            )
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete its response"
        assert len(received) == 1
        assert _tlv_enumeration(0x42005C, 0x37) in received[0]
        assert _tlv_structure(0x420168) in received[0]
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        assert item_result.operation == 0x37 and item_result.result_status == 0
        response = item_result.set_constraints_response
        assert isinstance(response, client.SetConstraintsResponse)
        assert "response_ttlv" not in repr(response)
        with (
            ttlv.ttlv_structure_view(response.response_ttlv) as view,
            ttlv.ttlv_structure_view_item_at(view, 3) as payload,
            ttlv.ttlv_item_view_value(payload) as payload_value,
            ttlv.ttlv_value_view_structure(payload_value) as payload_view,
        ):
            assert ttlv.ttlv_structure_view_item_count(view) == 4
            assert ttlv.ttlv_structure_view_item_count(payload_view) == 0
        result.close()
        with ttlv.ttlv_structure_view(response.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 4
    finally:
        constraints.close()
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)


@pytest.mark.parametrize(
    ("status", "reason", "message", "correlation", "outcome"),
    (
        pytest.param(
            1, 7, "constraint denied", None, client.BatchOutcome.COMPLETED, id="failure"
        ),
        pytest.param(
            2,
            None,
            None,
            b"set-constraints-pending-correlation",
            client.BatchOutcome.PENDING,
            id="pending",
        ),
    ),
)
def test_set_constraints_native_mtls_failure_and_pending_keep_common_metadata_only(
    status: int,
    reason: int | None,
    message: str | None,
    correlation: bytes | None,
    outcome: client.BatchOutcome,
) -> None:
    response_message = _set_constraints_response_message(
        result_status=status,
        result_reason=reason,
        result_message=message,
        pending_correlation_value=correlation,
    )
    port, received, failures, peer = _start_set_constraints_mtls_peer(response_message)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    result = None
    constraints = ttlv.ttlv_structure_create()
    try:
        with (
            configuration,
            client.Client.create(
                configuration, _set_constraints_transport(port)
            ) as kmip,
            ttlv.codec_limits_defaults() as limits,
        ):
            batch = client.Batch.create()
            if status == 2:
                batch.set_asynchronous_indicator(1)
            item = extensions.client_batch_item_set_constraints(constraints)
            constraints.close()
            batch.add_item(
                extensions.with_unique_batch_item_id(item, _SET_CONSTRAINTS_ID)
            )
            result = kmip.execute(batch, limits)

        assert failures.empty(), "the loopback mTLS peer must complete its response"
        assert len(received) == 1
        assert result is not None and len(result.items) == 1
        item_result = result.items[0]
        assert item_result.operation == 0x37
        assert item_result.outcome == outcome
        assert item_result.result_status == status
        assert item_result.result_reason == reason
        assert item_result.result_message == message
        assert item_result.pending_correlation_value == correlation
        assert item_result.set_constraints_response is None
    finally:
        constraints.close()
        if result is not None:
            result.close()
        peer.join(timeout=4)
