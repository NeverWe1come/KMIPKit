import queue
import ssl
from collections.abc import Iterator
from contextlib import contextmanager
from typing import Protocol

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


def require_tls13_mtls_peer(stream: _TlsPeer) -> None:
    version = stream.version()
    if version != "TLSv1.3":
        raise RuntimeError("test TLS peer did not negotiate TLS 1.3")
    peer_certificate = stream.getpeercert(binary_form=True)
    if not peer_certificate:
        raise RuntimeError("test TLS peer did not authenticate a client certificate")
