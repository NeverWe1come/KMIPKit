"""Controlled TLS 1.3 mTLS exchange tests; never evidence of vendor support."""

import json
import os
from pathlib import Path
import queue
import socket
import ssl
import struct
import threading
import unittest
from unittest.mock import patch

from kmipkit_validator.contract import DEFAULT_CATALOG
from kmipkit_validator.driver import PythonKmipDriver


ROOT = Path(__file__).resolve().parents[3]
PKI = ROOT / "bindings/python/tests/fixtures/login-mtls"


def item(tag: int, item_type: int, value: bytes) -> bytes:
    return tag.to_bytes(3, "big") + bytes([item_type]) + len(value).to_bytes(4, "big") + value + bytes((-len(value)) % 8)


def structure(tag: int, *children: bytes) -> bytes:
    return item(tag, 1, b"".join(children))


def enumeration(tag: int, value: int) -> bytes:
    return item(tag, 5, struct.pack(">I", value))


def exact(stream: ssl.SSLSocket, size: int) -> bytes:
    buffer = bytearray()
    while len(buffer) < size:
        chunk = stream.recv(size - len(buffer))
        if not chunk:
            raise AssertionError("client closed before a complete TTLV frame")
        buffer.extend(chunk)
    return bytes(buffer)


def frame(stream: ssl.SSLSocket) -> bytes:
    header = exact(stream, 8)
    length = int.from_bytes(header[4:8], "big")
    if length > 16 * 1024 * 1024:
        raise AssertionError("request exceeded configured TTLV limit")
    return header + exact(stream, length + (-length % 8))


def parse_children(data: bytes) -> list[tuple[int, int, bytes]]:
    found = []
    offset = 0
    while offset < len(data):
        tag = int.from_bytes(data[offset : offset + 3], "big")
        kind = data[offset + 3]
        length = int.from_bytes(data[offset + 4 : offset + 8], "big")
        value = data[offset + 8 : offset + 8 + length]
        found.append((tag, kind, value))
        offset += 8 + length + (-length % 8)
    return found


def find_value(tree: list[tuple[int, int, bytes]], tag: int) -> bytes | None:
    for current_tag, kind, value in tree:
        if current_tag == tag:
            return value
        if kind == 1:
            nested = find_value(parse_children(value), tag)
            if nested is not None:
                return nested
    return None


def ping_response(request: bytes, status: int, reason: int | None) -> bytes:
    tree = parse_children(request[8:])
    operation_bytes = find_value(tree, 0x42005C)
    batch_id = find_value(tree, 0x420093)
    if operation_bytes is None:
        raise AssertionError("request had no Operation")
    operation = int.from_bytes(operation_bytes, "big")
    payload = [enumeration(0x42005C, operation)]
    if batch_id is not None:
        payload.append(item(0x420093, 8, batch_id))
    payload.append(enumeration(0x42007F, status))
    if reason is not None:
        payload.append(enumeration(0x42007E, reason))
    if status in (0, 2):
        payload.append(structure(0x42007C))
    version = structure(
        0x420069,
        item(0x42006A, 2, struct.pack(">i", 2)),
        item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = structure(
        0x42007A,
        version,
        item(0x420092, 9, struct.pack(">q", 1_794_268_800)),
        item(0x42000D, 2, struct.pack(">i", 1)),
    )
    return structure(0x42007B, header, structure(0x42000F, *payload))


class ControlledTlsPeer:
    def __init__(self, status: int = 0, reason: int | None = None, *, expect_no_connection: bool = False) -> None:
        self.status = status
        self.reason = reason
        self.expect_no_connection = expect_no_connection
        self.listener = socket.socket()
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen(1)
        self.port = self.listener.getsockname()[1]
        self.error: queue.Queue[BaseException] = queue.Queue()
        self.requests: list[bytes] = []
        self.thread = threading.Thread(target=self._serve, daemon=True)
        self.thread.start()

    def _serve(self) -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
            context.load_verify_locations(cafile=PKI / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            self.listener.settimeout(2 if self.expect_no_connection else 10)
            try:
                raw, _ = self.listener.accept()
            except TimeoutError as error:
                if not self.expect_no_connection:
                    raise error
                return
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                if stream.version() != "TLSv1.3" or not stream.getpeercert(binary_form=True):
                    raise AssertionError("TLS 1.3 mutual authentication was not negotiated")
                request = frame(stream)
                self.requests.append(request)
                stream.sendall(ping_response(request, self.status, self.reason))
        except BaseException as error:  # Captured and surfaced by the owning test.
            self.error.put(error)
        finally:
            self.listener.close()

    def close(self) -> None:
        self.thread.join(timeout=12)
        if self.thread.is_alive():
            raise AssertionError("controlled TLS peer did not terminate")
        if not self.error.empty():
            raise self.error.get()


class ControlledExchangeTests(unittest.TestCase):
    def _execute_ping(
        self,
        peer: ControlledTlsPeer,
        authentication: dict | None = None,
        case_id: str = "test-op-ping",
        protocol_version: str = "2.1",
    ) -> dict:
        connection = {
            "server_id": "loopback",
            "protocol_version": protocol_version,
            "transport": "raw_tls",
            "encoding": "TTLV",
            "api_level": "typed-protocol",
            "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": peer.port}},
            "tls": {
                "ca_cert_path": str(PKI / "server-cert.pem"),
                "client_cert_path": str(PKI / "client-cert.pem"),
                "client_key_path": str(PKI / "client-key.pem"),
                "server_name_override": "localhost",
                "connect_timeout_seconds": 5,
                "read_timeout_seconds": 5,
            },
        }
        if authentication is not None:
            connection["authentication"] = authentication
        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        case = next(entry for entry in catalog["test_cases"] if entry["case_id"] == case_id)
        driver = None
        try:
            driver = PythonKmipDriver(connection)
            return driver.execute_case(case)
        finally:
            if driver is not None:
                driver.close()
            peer.close()

    def test_ping_uses_installed_client_and_verifies_a_controlled_tls_response(self):
        from kmipkit import protocol_values

        peer = ControlledTlsPeer()
        observed = self._execute_ping(peer)
        self.assertEqual("passed", observed["status"])
        self.assertEqual(0, observed["result_status_code"])
        self.assertEqual(1, len(peer.requests))

    def test_normative_not_supported_response_is_separate_from_library_failure(self):
        from kmipkit import protocol_values

        peer = ControlledTlsPeer(
            status=protocol_values.ResultStatus.OPERATION_FAILED,
            reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
        )
        observed = self._execute_ping(peer)
        self.assertEqual("server_unsupported", observed["status"])
        self.assertEqual("passed", observed["library_correctness"])
        self.assertEqual("unsupported", observed["server_compatibility"])

    def test_username_password_authentication_is_sent_but_never_reported(self):
        peer = ControlledTlsPeer()
        authentication = {
            "username_password": {
                "username_env": "KMIPKIT_INTEROP_TEST_USER",
                "password_env": "KMIPKIT_INTEROP_TEST_PASSWORD",
            }
        }
        with patch.dict(
            os.environ,
            {
                "KMIPKIT_INTEROP_TEST_USER": "interop-user-sentinel",
                "KMIPKIT_INTEROP_TEST_PASSWORD": "interop-password-sentinel",
            },
        ):
            observed = self._execute_ping(peer, authentication)
        request = peer.requests[0]
        self.assertIn(bytes.fromhex("420023"), request)
        self.assertIn(b"interop-user-sentinel", request)
        self.assertIn(b"interop-password-sentinel", request)
        self.assertNotIn("interop-user-sentinel", json.dumps(observed))
        self.assertNotIn("interop-password-sentinel", json.dumps(observed))

    def test_query_async_requests_uses_installed_explicit_async_api(self):
        from kmipkit import protocol_values

        peer = ControlledTlsPeer(
            status=protocol_values.ResultStatus.OPERATION_FAILED,
            reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
        )
        observed = self._execute_ping(peer, case_id="test-op-query-asynchronous-requests")
        self.assertEqual("server_unsupported", observed["status"])
        self.assertEqual(1, observed["result_status_code"])
        self.assertEqual(1, len(peer.requests))
        self.assertEqual(
            0x39,
            int.from_bytes(find_value(parse_children(peer.requests[0][8:]), 0x42005C), "big"),
        )

    def test_kmip20_process_is_rejected_before_opening_a_connection(self):
        peer = ControlledTlsPeer(expect_no_connection=True)
        observed = self._execute_ping(
            peer,
            case_id="test-v20-reject-process",
            protocol_version="2.0",
        )
        self.assertEqual("passed", observed["status"])
        self.assertEqual("passed", observed["library_correctness"])
        self.assertEqual([], peer.requests)

    def test_kmip20_query_async_requests_is_rejected_before_opening_a_connection(self):
        peer = ControlledTlsPeer(expect_no_connection=True)
        observed = self._execute_ping(
            peer,
            case_id="test-v20-reject-query-asynchronous-requests",
            protocol_version="2.0",
        )
        self.assertEqual("passed", observed["status"])
        self.assertEqual("passed", observed["library_correctness"])
        self.assertEqual([], peer.requests)


if __name__ == "__main__":
    unittest.main()
