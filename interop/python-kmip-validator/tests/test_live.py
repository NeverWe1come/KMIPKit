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
import xml.etree.ElementTree as ET
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


def ping_response(request: bytes, status: int, reason: int | None, protocol_minor: int = 1) -> bytes:
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
        item(0x42006B, 2, struct.pack(">i", protocol_minor)),
    )
    header = structure(
        0x42007A,
        version,
        item(0x420092, 9, struct.pack(">q", 1_794_268_800)),
        item(0x42000D, 2, struct.pack(">i", 1)),
    )
    return structure(0x42007B, header, structure(0x42000F, *payload))


def request_observation(request: bytes, encoding: str) -> dict[str, object]:
    if encoding == "TTLV":
        tree = parse_children(request[8:])
        operation = int.from_bytes(find_value(tree, 0x42005C), "big")
        batch_count = int.from_bytes(find_value(tree, 0x42000D), "big", signed=True)
        from kmipkit import protocol_values

        return {"operation": protocol_values.Operation.name(operation), "batch_count": batch_count}
    if encoding == "JSON":
        tree = json.loads(request)

        def find(node, tag):
            if isinstance(node, dict):
                if node.get("tag") == tag:
                    return node.get("value")
                return find(node.get("value"), tag)
            if isinstance(node, list):
                for child in node:
                    value = find(child, tag)
                    if value is not None:
                        return value
            return None

        return {"operation": find(tree, "Operation"), "batch_count": find(tree, "BatchCount")}
    root = ET.fromstring(request)
    operation = root.find(".//Operation")
    batch_count = root.find(".//BatchCount")
    if operation is None or batch_count is None:
        raise AssertionError("XML request omitted Operation or BatchCount")
    return {
        "operation": operation.attrib.get("value"),
        "batch_count": int(batch_count.attrib["value"]),
    }


def text_response(
    operation: str, encoding: str, status: int, reason: int | None, protocol_minor: int = 1
) -> bytes:
    from kmipkit import protocol_values

    status_name = protocol_values.ResultStatus.name(status)
    reason_name = None if reason is None else protocol_values.ResultReason.name(reason)
    if encoding == "JSON":
        batch_item = [
            {"tag": "Operation", "type": "Enumeration", "value": operation},
            {"tag": "ResultStatus", "type": "Enumeration", "value": status_name},
        ]
        if reason_name is not None:
            batch_item.append({"tag": "ResultReason", "type": "Enumeration", "value": reason_name})
        if status == protocol_values.ResultStatus.SUCCESS:
            batch_item.append({"tag": "ResponsePayload", "value": []})
        response = {
            "tag": "ResponseMessage",
            "value": [
                {
                    "tag": "ResponseHeader",
                    "value": [
                        {
                            "tag": "ProtocolVersion",
                            "value": [
                                {"tag": "ProtocolVersionMajor", "type": "Integer", "value": 2},
                                {"tag": "ProtocolVersionMinor", "type": "Integer", "value": protocol_minor},
                            ],
                        },
                        {"tag": "TimeStamp", "type": "DateTime", "value": "1970-01-01T00:00:01Z"},
                        {"tag": "BatchCount", "type": "Integer", "value": 1},
                    ],
                },
                {"tag": "BatchItem", "value": batch_item},
            ],
        }
        return json.dumps(response, separators=(",", ":")).encode("utf-8")
    fields = [
        f'<Operation type="Enumeration" value="{operation}"/>',
        f'<ResultStatus type="Enumeration" value="{status_name}"/>',
    ]
    if reason_name is not None:
        fields.append(f'<ResultReason type="Enumeration" value="{reason_name}"/>')
    if status == protocol_values.ResultStatus.SUCCESS:
        fields.append("<ResponsePayload/>")
    document = (
        "<ResponseMessage><ResponseHeader><ProtocolVersion>"
        '<ProtocolVersionMajor type="Integer" value="2"/>'
        f'<ProtocolVersionMinor type="Integer" value="{protocol_minor}"/>'
        "</ProtocolVersion>"
        '<TimeStamp type="DateTime" value="1970-01-01T00:00:01Z"/>'
        '<BatchCount type="Integer" value="1"/>'
        "</ResponseHeader><BatchItem>"
        + "".join(fields)
        + "</BatchItem></ResponseMessage>"
    )
    return document.encode("utf-8")


def read_raw_document(stream: ssl.SSLSocket, encoding: str) -> bytes:
    received = bytearray()
    while True:
        chunk = stream.recv(4096)
        if not chunk:
            raise AssertionError("client closed before a complete KMIP document")
        received.extend(chunk)
        if encoding == "JSON":
            try:
                _, end = json.JSONDecoder().raw_decode(received.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError):
                continue
            return bytes(received[:end])
        closing = b"</RequestMessage>"
        if encoding == "XML" and closing in received:
            end = received.index(closing) + len(closing)
            return bytes(received[:end])


def read_https_request(stream: ssl.SSLSocket) -> tuple[bytes, dict[str, str]]:
    received = bytearray()
    delimiter = b"\r\n\r\n"
    while delimiter not in received:
        chunk = stream.recv(4096)
        if not chunk:
            raise AssertionError("client closed before completing HTTP headers")
        received.extend(chunk)
    header_end = received.index(delimiter) + len(delimiter)
    lines = bytes(received[: header_end - len(delimiter)]).decode("ascii").split("\r\n")
    headers = {
        name.strip().lower(): value.strip()
        for line in lines[1:]
        if ":" in line
        for name, value in [line.split(":", 1)]
    }
    body_size = int(headers.get("content-length", "-1"))
    if body_size < 0 or body_size > 16 * 1024 * 1024:
        raise AssertionError("HTTPS request has no safe Content-Length")
    while len(received) < header_end + body_size:
        chunk = stream.recv(min(4096, header_end + body_size - len(received)))
        if not chunk:
            raise AssertionError("client closed before completing the HTTPS body")
        received.extend(chunk)
    return bytes(received[header_end : header_end + body_size]), headers


class ControlledTlsPeer:
    def __init__(
        self,
        status: int = 0,
        reason: int | None = None,
        *,
        expect_no_connection: bool = False,
        encoding: str = "TTLV",
        transport: str = "raw_tls",
        protocol_version: str = "2.1",
    ) -> None:
        self.status = status
        self.reason = reason
        self.expect_no_connection = expect_no_connection
        self.encoding = encoding
        self.transport = transport
        self.protocol_minor = int(protocol_version.split(".")[1])
        self.listener = socket.socket()
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen(1)
        self.port = self.listener.getsockname()[1]
        self.error: queue.Queue[BaseException] = queue.Queue()
        self.requests: list[bytes] = []
        self.responses: list[bytes] = []
        self.request_observation: dict[str, object] = {}
        self.http_headers: dict[str, str] = {}
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
                if self.transport == "https":
                    request, self.http_headers = read_https_request(stream)
                elif self.encoding == "TTLV":
                    request = frame(stream)
                else:
                    request = read_raw_document(stream, self.encoding)
                self.requests.append(request)
                self.request_observation = request_observation(request, self.encoding)
                if self.encoding == "TTLV":
                    response = ping_response(
                        request, self.status, self.reason, self.protocol_minor
                    )
                else:
                    response = text_response(
                        str(self.request_observation["operation"]),
                        self.encoding,
                        self.status,
                        self.reason,
                        self.protocol_minor,
                    )
                self.responses.append(response)
                if self.transport == "https":
                    content_type = {
                        "TTLV": "application/octet-stream",
                        "JSON": "application/json",
                        "XML": "text/xml",
                    }[self.encoding]
                    stream.sendall(
                        f"HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n".encode("ascii")
                        + f"Content-Length: {len(response)}\r\nConnection: close\r\n\r\n".encode("ascii")
                        + response
                    )
                else:
                    stream.sendall(response)
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
    def test_every_applicable_catalog_operation_is_sent_to_a_controlled_kmip_peer(self):
        from kmipkit import extensions, protocol_values

        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        async_operations = {
            "Poll", "Process", "Cancel", "Query Asynchronous Requests"
        }
        for protocol_version in ("2.0", "2.1"):
            cases = [
                case
                for case in catalog["test_cases"]
                if case["execution"]["mode"] == "server_exchange"
                and protocol_version in case["execution"]["protocol_versions"]
                and case.get("requires", {}).get("workflow_type") != "server_to_client"
                and PythonKmipDriver.supports(case, "typed-protocol")
            ]
            self.assertGreater(len(cases), 140)
            for case in cases:
                with self.subTest(protocol_version=protocol_version, case_id=case["case_id"]):
                    peer = ControlledTlsPeer(
                        status=protocol_values.ResultStatus.OPERATION_FAILED,
                        reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                        protocol_version=protocol_version,
                    )
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
                    driver = PythonKmipDriver(connection)
                    item = None
                    try:
                        if case["operation"] in async_operations:
                            correlation = (
                                None
                                if case["operation"] == "Query Asynchronous Requests"
                                else b"fixture-correlation"
                            )
                            observed = driver._async_exchange(case["operation"], correlation)
                        elif case["operation"] == "Logout":
                            ticket_type = driver._item(0x0042_014A, "enumeration", 1)
                            ticket_value = driver._item(0x0042_014B, "bytes", b"fixture-ticket")
                            ticket = driver._structure([ticket_type, ticket_value])
                            try:
                                item = extensions.client_batch_item_logout(ticket)
                                observed = driver._exchange(item)
                            finally:
                                ticket.close()
                                ticket_type.close()
                                ticket_value.close()
                        else:
                            item = driver._build_item(
                                case,
                                extensions.UniqueIdentifier.text_string("catalog-operation-fixture"),
                                name="catalog-operation-fixture",
                            )
                            observed = driver._exchange(item)
                        self.assertEqual("server_unsupported", observed["status"])
                        self.assertEqual(case["operation"], peer.request_observation["operation"])
                        self.assertEqual(1, len(peer.requests))
                    finally:
                        if item is not None:
                            item.close()
                        driver.close()
                        peer.close()

    def _execute_attribute_route(
        self, peer: ControlledTlsPeer, *, high_level: bool, case_id: str
    ) -> dict:
        from kmipkit import extensions

        connection = {
            "server_id": "loopback",
            "protocol_version": "2.1",
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
        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        case = next(entry for entry in catalog["test_cases"] if entry["case_id"] == case_id)
        driver = PythonKmipDriver(connection)
        identifier = extensions.UniqueIdentifier.text_string("same-request-fixture")
        item = None
        try:
            item = (
                driver._build_high_level_item(case, identifier, name="same-request")
                if high_level
                else driver._build_item(case, identifier, name="same-request")
            )
            return driver._exchange(item)
        finally:
            if item is not None:
                item.close()
            driver.close()
            peer.close()

    def test_high_level_and_typed_attribute_routes_emit_identical_requests_and_responses(self):
        from kmipkit import protocol_values

        for case_id in (
            "test-op-add-attribute",
            "test-op-adjust-attribute",
            "test-op-delete-attribute",
            "test-op-get-attributes",
            "test-op-get-attribute-list",
            "test-op-modify-attribute",
            "test-attr-comment",
        ):
            sent = []
            received = []
            outcomes = []
            for high_level in (False, True):
                peer = ControlledTlsPeer(
                    status=protocol_values.ResultStatus.OPERATION_FAILED,
                    reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                )
                outcomes.append(
                    self._execute_attribute_route(peer, high_level=high_level, case_id=case_id)
                )
                sent.append(peer.requests[0])
                received.append(peer.responses[0])
            with self.subTest(case_id=case_id):
                self.assertEqual(sent[0], sent[1])
                self.assertEqual(received[0], received[1])
                self.assertEqual(
                    ["server_unsupported", "server_unsupported"],
                    [result["status"] for result in outcomes],
                )

    def _execute_ping(
        self,
        peer: ControlledTlsPeer,
        authentication: dict | None = None,
        case_id: str = "test-op-ping",
        protocol_version: str = "2.1",
        encoding: str = "TTLV",
        transport: str = "raw_tls",
    ) -> dict:
        connection = {
            "server_id": "loopback",
            "protocol_version": protocol_version,
            "transport": transport,
            "encoding": encoding,
            "api_level": "typed-protocol",
            "endpoints": {
                "raw_tls": {"host": "127.0.0.1", "port": peer.port},
                "https": {"url": f"https://127.0.0.1:{peer.port}/kmip"},
            },
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

    def test_ping_round_trips_all_encodings_over_raw_tls_and_https(self):
        for encoding in ("TTLV", "JSON", "XML"):
            for transport in ("raw_tls", "https"):
                with self.subTest(encoding=encoding, transport=transport):
                    peer = ControlledTlsPeer(encoding=encoding, transport=transport)
                    observed = self._execute_ping(peer, encoding=encoding, transport=transport)
                    self.assertEqual("passed", observed["status"])
                    self.assertEqual("passed", observed["library_correctness"])
                    self.assertEqual("Ping", peer.request_observation["operation"])
                    self.assertEqual(1, peer.request_observation["batch_count"])

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
