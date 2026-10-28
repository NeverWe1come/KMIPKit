import queue
import socket
import ssl
import threading
from pathlib import Path
from types import SimpleNamespace

import _test_support
import pytest
from _test_support import (
    capture_peer_failures,
    operation_response_message,
    require_tls13_mtls_peer,
    start_tls13_mtls_peer,
)
from kmipkit import client
from kmipkit._client_ffi import ffi


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


def test_operation_response_fixture_builder_preserves_success_payload_fields() -> None:
    builder = getattr(_test_support, "operation_response_message", None)
    assert callable(builder), "shared operation response fixture builder is required"
    payload_items = (
        _test_support.ttlv_item(0x420094, 7, b"registered-server-id"),
        _test_support.ttlv_item(0x541234, 8, b"\x01\x00\xff"),
    )

    assert builder(0x03, b"register-0", payload_items=payload_items).hex() == (
        "42007b01000000c842007a0100000048420069010000002042006a0200000004000000020000000042006b020000000400000001000000004200920900000008000000000000000142000d0200000004000000010000000042000f010000007042005c05000000040000000300000000420093080000000a72656769737465722d3000000000000042007f0500000004000000000000000042007c01000000304200940700000014726567697374657265642d7365727665722d69640000000054123408000000030100ff0000000000"
    )


def test_unique_identifier_ttlv_fixture_preserves_native_callback_contract() -> None:
    factory = getattr(_test_support, "unique_identifier_ttlv_response_test_fixture", None)
    assert callable(factory), "shared unique-identifier TTLV fixture is required"
    released: list[object] = []
    fixture = factory("revoke", b"revoked-object", 42, released.append)

    expected_symbols = {
        "kmipkit_client_batch_item_response_revoke",
        "kmipkit_client_batch_item_response_revoke_ttlv",
        "kmipkit_ttlv_structure_release",
    }
    assert set(fixture.response_symbols) == expected_symbols

    identifier_fields = SimpleNamespace(
        kind=0,
        text_data=ffi.NULL,
        text_length=0,
    )
    present = [0]
    assert (
        fixture.response_symbols["kmipkit_client_batch_item_response_revoke"](
            object(), present, identifier_fields
        )
        == 0
    )
    assert present == [1]
    assert identifier_fields.kind == int(
        client.extensions.UniqueIdentifierKind.TEXT_STRING
    )
    assert bytes(
        ffi.buffer(identifier_fields.text_data, identifier_fields.text_length)
    ) == b"revoked-object"

    ttlv_present = [0]
    ttlv_output = [ffi.NULL]
    assert (
        fixture.response_symbols[
            "kmipkit_client_batch_item_response_revoke_ttlv"
        ](object(), ttlv_present, ttlv_output)
        == 0
    )
    assert ttlv_present == [1]
    assert ttlv_output[0] == fixture.native_ttlv
    release_value = object()
    fixture.response_symbols["kmipkit_ttlv_structure_release"](release_value)
    assert released == [release_value]
    assert fixture.native_ttlv_address == 42

    assert not fixture.owned_ttlv.closed
    fixture.owned_ttlv.close()
    assert fixture.owned_ttlv.closed


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


def test_ttlv_frame_reader_preserves_operation_specific_eof_message() -> None:
    class ClosedStream:
        def recv(self, _size: int) -> bytes:
            return b""

    reader = getattr(_test_support, "read_ttlv_frame", None)
    assert callable(reader), "shared TTLV frame reader is required"

    with pytest.raises(AssertionError) as failure:
        reader(ClosedStream(), eof_message="client closed before full request")

    assert str(failure.value) == "client closed before full request"


@pytest.mark.parametrize(
    ("operation", "response_symbol", "ttlv_symbol"),
    [
        (
            "login",
            "kmipkit_client_batch_item_response_login",
            "kmipkit_client_batch_item_response_login_ttlv",
        ),
        (
            "delegated_login",
            "kmipkit_client_batch_item_response_delegated_login",
            "kmipkit_client_batch_item_response_delegated_login_ttlv",
        ),
    ],
)
def test_ticket_response_fixture_installs_operation_specific_native_symbols(
    monkeypatch: pytest.MonkeyPatch,
    operation: str,
    response_symbol: str,
    ttlv_symbol: str,
) -> None:
    installer = getattr(_test_support, "install_ticket_native_response", None)
    assert callable(installer), "shared ticket response fixture is required"
    installer(
        monkeypatch,
        operation,
        ticket_type=0xFEED_BEEF,
        ticket_value=b"ticket-fixture",
    )

    present = ffi.new("uint8_t *")
    ticket_present = ffi.new("uint8_t *")
    ticket_type = ffi.new("uint32_t *")
    ticket_value = ffi.new("kmipkit_byte_span_v1_t *")
    assert getattr(client.lib, response_symbol)(
        ffi.NULL, present, ticket_present, ticket_type, ticket_value
    ) == 0
    assert present[0] == ticket_present[0] == 1
    assert ticket_type[0] == 0xFEED_BEEF
    assert ffi.buffer(ticket_value.data, int(ticket_value.length))[:] == b"ticket-fixture"

    ttlv_present = ffi.new("uint8_t *")
    ttlv_output = ffi.new("kmipkit_ttlv_structure_t **")
    assert getattr(client.lib, ttlv_symbol)(ffi.NULL, ttlv_present, ttlv_output) == 0
    assert ttlv_present[0] == 1
    assert ffi.cast("uintptr_t", ttlv_output[0]) == 24


def test_batch_result_fixture_supplies_common_absent_response_symbols() -> None:
    factory = getattr(_test_support, "batch_result_test_symbols", None)
    assert callable(factory), "shared batch-result ABI fixture is required"

    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 2)
    symbols = factory(ffi, item_pointer, lambda _item, _output: 0)
    count = ffi.new("size_t *")
    assert symbols["kmipkit_client_batch_response_item_count"](ffi.NULL, count) == 0
    assert count[0] == 1

    present = ffi.new("uint8_t *")
    span = ffi.new("kmipkit_byte_span_v1_t *")
    assert symbols["kmipkit_client_batch_item_response_unique_batch_item_id"](
        item_pointer, present, span
    ) == 0
    assert present[0] == 0
    assert span.data == ffi.NULL and span.length == 0
