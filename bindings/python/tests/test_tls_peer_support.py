import pytest
from _test_support import require_tls13_mtls_peer


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
