import queue
from pathlib import Path

import _test_support
import pytest
from _test_support import capture_peer_failures, require_tls13_mtls_peer
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
