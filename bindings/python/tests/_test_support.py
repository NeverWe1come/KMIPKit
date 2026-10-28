import queue
import ssl
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Protocol

from kmipkit import client

_PEER_FAILURE_TYPES = (AssertionError, OSError, RuntimeError, ssl.SSLError)


class _TlsPeer(Protocol):
    def version(self) -> str | None: ...

    def getpeercert(self, binary_form: bool = False) -> bytes | None: ...


@contextmanager
def capture_peer_failures(
    peer_failures: queue.Queue[BaseException],
) -> Iterator[None]:
    """Queue expected TLS peer errors so the owning test can report them."""
    try:
        yield
    except _PEER_FAILURE_TYPES as failure:
        peer_failures.put(failure)


def mtls_transport_config(
    port: int, fixtures: Path, *, total_timeout_seconds: int = 8
) -> client.TransportConfig:
    return client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(
            client.TransportInput.file(
                fixtures / "client-cert.pem", client.CredentialEncoding.PEM
            ),
        ),
        client_private_key=client.TransportInput.file(
            fixtures / "client-key.pem", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.certificate_authorities(
            (
                client.TransportInput.file(
                    fixtures / "server-cert.pem", client.CredentialEncoding.PEM
                ),
            )
        ),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(total_timeout_seconds),
    )


def require_tls13_mtls_peer(stream: _TlsPeer) -> None:
    version = stream.version()
    if version != "TLSv1.3":
        raise RuntimeError("test TLS peer did not negotiate TLS 1.3")
    peer_certificate = stream.getpeercert(binary_form=True)
    if not peer_certificate:
        raise RuntimeError("test TLS peer did not authenticate a client certificate")
