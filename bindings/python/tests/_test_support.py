from typing import Protocol


class TlsPeer(Protocol):
    def version(self) -> str | None: ...

    def getpeercert(self, binary_form: bool = False) -> bytes | None: ...


def require_tls13_mtls_peer(stream: TlsPeer) -> None:
    assert stream.version() == "TLSv1.3"
    assert stream.getpeercert(binary_form=True)
