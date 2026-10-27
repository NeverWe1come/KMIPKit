from typing import Protocol


class TlsPeer(Protocol):
    def version(self) -> str | None: ...

    def getpeercert(self, binary_form: bool = False) -> bytes | None: ...


def require_tls13_mtls_peer(stream: TlsPeer) -> None:
    if stream.version() != "TLSv1.3":
        raise RuntimeError("test TLS peer did not negotiate TLS 1.3")
    if not stream.getpeercert(binary_form=True):
        raise RuntimeError("test TLS peer did not authenticate a client certificate")
