import queue
import socket
import ssl
import struct
import threading
from collections.abc import Callable, Iterator
from contextlib import contextmanager
from pathlib import Path
from types import SimpleNamespace
from typing import Protocol

from kmipkit import client
from kmipkit._client_ffi import ffi

_PEER_FAILURE_TYPES = (AssertionError, OSError, RuntimeError, ssl.SSLError)


class _TlsPeer(Protocol):
    def version(self) -> str | None: ...

    def getpeercert(self, binary_form: bool = False) -> bytes | None: ...

    def recv(self, size: int) -> bytes: ...


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


def start_tls13_mtls_peer(
    fixtures: Path,
    handler: Callable[[ssl.SSLSocket], None],
    *,
    backlog: int = 1,
    timeout_seconds: float = 10,
) -> tuple[int, queue.Queue[Exception], threading.Thread]:
    """Start one TLS 1.3 mutual-authentication peer for a test callback."""
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(backlog)
    listener.settimeout(timeout_seconds)
    port = listener.getsockname()[1]
    failures: queue.Queue[Exception] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                fixtures / "server-cert.pem", fixtures / "server-key.pem"
            )
            context.load_verify_locations(cafile=fixtures / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                try:
                    require_tls13_mtls_peer(stream)
                    handler(stream)
                except Exception as error:  # noqa: BLE001 - preserve callback failures before TLS close.
                    failures.put(error)
        except Exception as error:  # noqa: BLE001 - report peer failures to the test thread.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return port, failures, peer


def read_exact(
    stream: _TlsPeer,
    size: int,
    *,
    eof_message: str = "client closed before sending a complete request",
) -> bytes:
    output = bytearray()
    while len(output) < size:
        chunk = stream.recv(size - len(output))
        if not chunk:
            raise AssertionError(eof_message)
        output.extend(chunk)
    return bytes(output)


def read_ttlv_frame(
    stream: _TlsPeer,
    *,
    eof_message: str = "client closed before sending a complete request",
) -> bytes:
    header = read_exact(stream, 8, eof_message=eof_message)
    return header + read_exact(
        stream, int.from_bytes(header[4:8], "big"), eof_message=eof_message
    )


def ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    return (
        tag.to_bytes(3, "big")
        + bytes((item_type,))
        + len(value).to_bytes(4, "big")
        + value
        + bytes((-len(value)) % 8)
    )


def ttlv_structure(tag: int, *children: bytes) -> bytes:
    return ttlv_item(tag, 1, b"".join(children))


def ttlv_enumeration(tag: int, value: int) -> bytes:
    return ttlv_item(tag, 5, struct.pack(">I", value))


def operation_response_message(
    operation: int,
    batch_id: bytes,
    *,
    result_status: int = 0,
    result_reason: int | None = None,
    result_message: str | None = None,
    pending_correlation_value: bytes | None = None,
    payload_items: tuple[bytes, ...] = (),
) -> bytes:
    version = ttlv_structure(
        0x420069,
        ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = ttlv_structure(
        0x42007A,
        version,
        ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    response_fields = [
        ttlv_enumeration(0x42005C, operation),
        ttlv_item(0x420093, 8, batch_id),
        ttlv_enumeration(0x42007F, result_status),
    ]
    if result_reason is not None:
        response_fields.append(ttlv_enumeration(0x42007E, result_reason))
    if result_message is not None:
        response_fields.append(ttlv_item(0x42007D, 7, result_message.encode()))
    if pending_correlation_value is not None:
        response_fields.append(ttlv_item(0x420006, 8, pending_correlation_value))
    if result_status in (0, 2):
        response_fields.append(ttlv_structure(0x42007C, *payload_items))
    response_item = ttlv_structure(0x42000F, *response_fields)
    return ttlv_structure(0x42007B, header, response_item)


def install_ticket_native_response(
    monkeypatch: object,
    operation: str,
    *,
    present: int = 1,
    ticket_present: int = 1,
    ticket_type: int = 7,
    ticket_value: bytes = b"ticket-secret",
    span_length: int | None = None,
    ttlv_present: int = 1,
    ttlv_pointer: int = 24,
    released: list[int] | None = None,
) -> None:
    """Install a fake Login or Delegated Login response ABI for binding tests."""
    native_value = ffi.new("uint8_t[]", ticket_value) if ticket_value else ffi.NULL

    def response(_item: object, out_present: object, out_ticket_present: object, out_ticket_type: object, out_value: object) -> int:
        out_present[0] = present
        out_ticket_present[0] = ticket_present
        out_ticket_type[0] = ticket_type
        out_value.data = native_value
        out_value.length = len(ticket_value) if span_length is None else span_length
        return 0

    def response_ttlv(_item: object, out_present: object, output: object) -> int:
        out_present[0] = ttlv_present
        output[0] = ffi.cast("kmipkit_ttlv_structure_t *", ttlv_pointer)
        return 0

    def release(pointer: object) -> None:
        if released is not None:
            released.append(int(ffi.cast("uintptr_t", pointer)))

    response_symbol = f"kmipkit_client_batch_item_response_{operation}"
    response_ttlv_symbol = f"{response_symbol}_ttlv"
    library = SimpleNamespace(
        **{
            response_symbol: response,
            response_ttlv_symbol: response_ttlv,
            "kmipkit_ttlv_structure_release": release,
        }
    )
    monkeypatch.setattr(client, "lib", library)


def batch_result_test_symbols(
    ffi_module: object,
    item_pointer: object,
    metadata: Callable[..., int],
    *,
    response_release: Callable[[object], None] | None = None,
    item_release: Callable[[object], None] | None = None,
) -> dict[str, object]:
    """Return common fake ABI symbols for one-item batch response tests."""
    def absent_span(_item: object, present: object, span: object) -> int:
        present[0] = 0
        span.data = ffi_module.NULL
        span.length = 0
        return 0

    def absent_versions(
        _item: object,
        present: object,
        _values: object,
        _capacity: object,
        count: object,
    ) -> int:
        present[0] = 0
        count[0] = 0
        return 0

    def absent_create(
        _item: object, present: object, object_type: object, _identifier: object
    ) -> int:
        present[0] = 0
        object_type[0] = 0
        return 0

    def absent_create_key_pair(
        _item: object, present: object, _private: object, _public: object
    ) -> int:
        present[0] = 0
        return 0

    def absent_split_key_count(_item: object, present: object, count: object) -> int:
        present[0] = 0
        count[0] = 0
        return 0

    return {
        "kmipkit_client_batch_response_release": response_release
        or (lambda _pointer: None),
        "kmipkit_client_batch_item_response_release": item_release
        or (lambda _pointer: None),
        "kmipkit_client_batch_response_item_count": lambda _pointer, count: (
            count.__setitem__(0, 1) or 0
        ),
        "kmipkit_client_batch_response_item_at": lambda _pointer, _index, output: (
            output.__setitem__(0, item_pointer) or 0
        ),
        "kmipkit_client_batch_item_response_metadata_v2": metadata,
        "kmipkit_client_batch_item_response_unique_batch_item_id": absent_span,
        "kmipkit_client_batch_item_response_pending_correlation_value": absent_span,
        "kmipkit_client_batch_item_response_discover_versions_supported_versions": absent_versions,
        "kmipkit_client_batch_item_response_create": absent_create,
        "kmipkit_client_batch_item_response_create_key_pair": absent_create_key_pair,
        "kmipkit_client_batch_item_response_create_split_key_count": absent_split_key_count,
    }
