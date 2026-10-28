import queue
import socket
import ssl
import threading
from pathlib import Path

import _test_support
import pytest
from _test_support import (
    capture_peer_failures,
    operation_response_message,
    require_tls13_mtls_peer,
    start_tls13_mtls_peer,
)

from kmipkit import client


class FakeTlsPeer:
    def __init__(self, version: str | None, certificate: bytes | None) -> None:
        self._version = version
        self._certificate = certificate

    def version(self) -> str | None:
        return self._version

    def getpeercert(self, binary_form: bool = False) -> bytes | None:
        if not binary_form:
            return None
        return self._certificate


@pytest.mark.parametrize(
    ("version", "certificate"),
    [("TLSv1.2", b"client-certificate"), ("TLSv1.3", None)],
)
def test_rejects_peers_without_tls13_mutual_authentication(
    version: str | None, certificate: bytes | None
) -> None:
    with pytest.raises(RuntimeError):
        require_tls13_mtls_peer(FakeTlsPeer(version, certificate))


def test_accepts_tls13_mutual_authentication_peer() -> None:
    require_tls13_mtls_peer(FakeTlsPeer("TLSv1.3", b"client-certificate"))


def test_mtls_transport_factory_preserves_shared_client_inputs() -> None:
    factory = getattr(_test_support, "mtls_transport_config", None)
    assert callable(factory), "shared mTLS transport factory is required"

    fixtures = Path(__file__).parent / "fixtures" / "login-mtls"
    transport = factory(43123, fixtures)

    assert transport.endpoint == client.Endpoint.raw_tls("127.0.0.1", 43123)
    assert transport.client_certificates[0].source == fixtures / "client-cert.pem"
    assert transport.client_private_key.source == fixtures / "client-key.pem"
    assert transport.trust.kind is client.TrustKind.CERTIFICATES
    assert transport.trust.certificates[0].source == fixtures / "server-cert.pem"
    assert transport.connect_timeout == client.TimeoutLimit.bounded(5)
    assert transport.write_timeout == client.TimeoutLimit.bounded(5)
    assert transport.read_timeout == client.TimeoutLimit.bounded(5)
    assert transport.total_timeout == client.TimeoutLimit.bounded(8)
    extended = factory(43123, fixtures, total_timeout_seconds=10)
    assert extended.total_timeout == client.TimeoutLimit.bounded(10)


def test_captures_runtime_tls_peer_validation_failures() -> None:
    peer_failures: queue.Queue[BaseException] = queue.Queue()

    with capture_peer_failures(peer_failures):
        require_tls13_mtls_peer(FakeTlsPeer("TLSv1.2", b"client-certificate"))

    failure = peer_failures.get_nowait()
    assert type(failure) is RuntimeError
    assert str(failure) == "test TLS peer did not negotiate TLS 1.3"


@pytest.mark.parametrize(
    ("operation", "batch_id", "expected"),
    [
        (
            0x36,
            b"set-defaults-0",
            "42007b010000009842007a0100000048420069010000002042006a0200000004000000020000000042006b020000000400000001000000004200920900000008000000000000000142000d0200000004000000010000000042000f010000004042005c05000000040000003600000000420093080000000e7365742d64656661756c74732d30000042007f0500000004000000000000000042007c0100000000",
        ),
        (
            0x37,
            b"set-constraints-0",
            "42007b01000000a042007a0100000048420069010000002042006a0200000004000000020000000042006b020000000400000001000000004200920900000008000000000000000142000d0200000004000000010000000042000f010000004842005c0500000004000000370000000042009308000000117365742d636f6e73747261696e74732d300000000000000042007f0500000004000000000000000042007c0100000000",
        ),
    ],
)
def test_operation_response_fixture_builder_preserves_expected_wire_bytes(
    operation: int, batch_id: bytes, expected: str
) -> None:
    builder = getattr(_test_support, "operation_response_message", None)
    assert callable(builder), "shared operation response fixture builder is required"

    assert builder(operation, batch_id).hex() == expected


def test_shared_tls_peer_reports_handler_failures() -> None:
    fixtures = Path(__file__).parent / "fixtures" / "login-mtls"
    expected_failure = RuntimeError("peer handler failed")
    handler_finished = threading.Event()

    def fail_after_handshake(stream: ssl.SSLSocket) -> None:
        try:
            assert stream.recv(1) == b"x"
            raise expected_failure
        finally:
            handler_finished.set()

    port, failures, peer = start_tls13_mtls_peer(fixtures, fail_after_handshake)
    context = ssl.create_default_context(
        ssl.Purpose.SERVER_AUTH, cafile=str(fixtures / "server-cert.pem")
    )
    context.load_cert_chain(fixtures / "client-cert.pem", fixtures / "client-key.pem")
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.maximum_version = ssl.TLSVersion.TLSv1_3
    with socket.create_connection(("127.0.0.1", port), timeout=5) as raw:
        with context.wrap_socket(raw, server_hostname="127.0.0.1") as stream:
            stream.sendall(b"x")
            assert handler_finished.wait(timeout=5)

    peer.join(timeout=5)
    assert not peer.is_alive()
    assert failures.get_nowait() is expected_failure
