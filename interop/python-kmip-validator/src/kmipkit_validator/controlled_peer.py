"""Controlled TLS 1.3/mTLS fixtures used by the independent consumer.

Fixture bytes stay in memory; evidence contains normalized tree hashes only.
"""
from __future__ import annotations
import json
from pathlib import Path
import queue
import socket
import ssl
import struct
import threading
import xml.etree.ElementTree as ET
from .contract import ROOT

PKI = ROOT / "bindings/python/tests/fixtures/login-mtls"

def item(tag: int, item_type: int, value: bytes) -> bytes:
    return tag.to_bytes(3, "big") + bytes([item_type]) + len(value).to_bytes(4, "big") + value + bytes((-len(value)) % 8)


def structure(tag: int, *children: bytes) -> bytes:
    return item(tag, 1, b"".join(children))


def enumeration(tag: int, value: int) -> bytes:
    return item(tag, 5, struct.pack(">I", value))


def response_message_extension() -> bytes:
    """Create a standards-shaped extension with a synthetic, non-secret payload."""
    return structure(
        0x420051,
        item(0x42009D, 7, b"KMIPKit.Interop"),
        item(0x420026, 6, struct.pack(">Q", 0)),
        structure(0x42009C, item(0x540001, 8, b"fixture")),
    )


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


def ping_response(request: bytes, status: int, reason: int | None, protocol_minor: int = 1,
                  cancellation_result: int | None = None,
                  asynchronous_correlation_value: bytes | None = None,
                  include_response_extension: bool = False) -> bytes:
    tree = parse_children(request[8:])
    request_batch = [parse_children(value) for tag, kind, value in tree
                     if tag == 0x42000F and kind == 1]
    if not request_batch:
        raise AssertionError("request had no Batch Item")
    response_batch = []
    for fields in request_batch:
        operation_bytes = find_value(fields, 0x42005C)
        batch_id = find_value(fields, 0x420093)
        if operation_bytes is None:
            raise AssertionError("request Batch Item had no Operation")
        payload = [enumeration(0x42005C, int.from_bytes(operation_bytes, "big"))]
        if batch_id is not None:
            payload.append(item(0x420093, 8, batch_id))
        payload.append(enumeration(0x42007F, status))
        if reason is not None:
            payload.append(enumeration(0x42007E, reason))
        if status in (0, 2):
            response_payload = []
            if asynchronous_correlation_value is not None:
                response_payload.append(item(0x420006, 8, asynchronous_correlation_value))
            if cancellation_result is not None:
                response_payload.append(enumeration(0x420012, cancellation_result))
            payload.append(structure(0x42007C, *response_payload))
        if include_response_extension:
            payload.append(response_message_extension())
        response_batch.append(structure(0x42000F, *payload))
    version = structure(
        0x420069,
        item(0x42006A, 2, struct.pack(">i", 2)),
        item(0x42006B, 2, struct.pack(">i", protocol_minor)),
    )
    header = structure(
        0x42007A,
        version,
        item(0x420092, 9, struct.pack(">q", 1_794_268_800)),
        item(0x42000D, 2, struct.pack(">i", len(response_batch))),
    )
    return structure(0x42007B, header, *response_batch)


def discover_versions_response(
    request: bytes,
    status: int,
    reason: int | None,
    protocol_minor: int,
    supported_versions: tuple[tuple[int, int], ...] | None = None,
    include_response_extension: bool = False,
) -> bytes:
    """Return a deterministic, operation-shaped Discover Versions response."""
    tree = parse_children(request[8:])
    operation_bytes = find_value(tree, 0x42005C)
    batch_id = find_value(tree, 0x420093)
    if operation_bytes is None or int.from_bytes(operation_bytes, "big") != 0x1E:
        raise AssertionError("request was not Discover Versions")
    payload = [enumeration(0x42005C, 0x1E)]
    if batch_id is not None:
        payload.append(item(0x420093, 8, batch_id))
    payload.append(enumeration(0x42007F, status))
    if reason is not None:
        payload.append(enumeration(0x42007E, reason))
    if status == 0:
        # DiscoverVersionsRequest::new advertises KMIP 2.1 regardless of the
        # negotiated message version used for this request.
        versions = supported_versions if supported_versions is not None else ((2, 1),)
        version_items = [
            structure(
                0x420069,
                item(0x42006A, 2, struct.pack(">i", major)),
                item(0x42006B, 2, struct.pack(">i", minor)),
            )
            for major, minor in versions
        ]
        payload.append(structure(0x42007C, *version_items))
    if include_response_extension:
        payload.append(response_message_extension())
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
        batch_items = [parse_children(value) for tag, kind, value in tree
                       if tag == 0x42000F and kind == 1]
        operations = [int.from_bytes(value, "big") for batch_item in batch_items
                      if (value := find_value(batch_item, 0x42005C)) is not None]
        batch_count = int.from_bytes(find_value(tree, 0x42000D), "big", signed=True)
        from kmipkit import protocol_values

        if not operations:
            raise AssertionError("request had no Batch Item Operation")
        names = [protocol_values.Operation.name(operation) for operation in operations]
        return {"operation": names[0], "operations": names, "batch_count": batch_count}
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

        def batch_nodes(node):
            if isinstance(node, dict):
                if node.get("tag") == "BatchItem":
                    return [node]
                return batch_nodes(node.get("value"))
            if isinstance(node, list):
                return [item for child in node for item in batch_nodes(child)]
            return []

        operations = [find(node.get("value"), "Operation") for node in batch_nodes(tree)]
        if not operations or any(operation is None for operation in operations):
            raise AssertionError("request had no Batch Item Operation")
        identifiers = [
            next((field.get("value") for field in node.get("value", [])
                  if isinstance(field, dict) and field.get("tag") == "UniqueBatchItemID"), None)
            for node in batch_nodes(tree)
        ]
        return {"operation": operations[0], "operations": operations,
                "batch_count": find(tree, "BatchCount"), "batch_ids": identifiers}
    root = ET.fromstring(request)
    batch_items = root.findall(".//BatchItem")
    operations = [node.find("./Operation") for node in batch_items]
    batch_count = root.find(".//BatchCount")
    if not operations or any(operation is None for operation in operations) or batch_count is None:
        raise AssertionError("XML request omitted Operation or BatchCount")
    names = [operation.attrib.get("value") for operation in operations]
    identifiers = [
        (identifier.attrib.get("value") if identifier is not None else None)
        for node in batch_items
        for identifier in [node.find("./UniqueBatchItemID")]
    ]
    return {
        "operation": names[0],
        "operations": names,
        "batch_count": int(batch_count.attrib["value"]),
        "batch_ids": identifiers,
    }


def normalized_operation_name(value: str) -> str:
    return "".join(character for character in value.casefold() if character.isalnum())


def text_response(
    operation: str, encoding: str, status: int, reason: int | None, protocol_minor: int = 1,
    supported_versions: tuple[tuple[int, int], ...] | None = None,
    operations: list[str] | None = None,
    batch_ids: list[str | None] | None = None,
) -> bytes:
    from kmipkit import protocol_values

    status_name = "".join(protocol_values.ResultStatus.name(status).split())
    reason_name = (
        None
        if reason is None
        else "".join(protocol_values.ResultReason.name(reason).split())
    )
    operations = operations or [operation]
    batch_ids = [None] * len(operations) if batch_ids is None else batch_ids
    if len(batch_ids) != len(operations):
        raise ValueError("controlled peer batch identifier count did not match operations")

    def response_item(item_operation: str, identifier: str | None) -> list[dict]:
        fields = [
            {"tag": "Operation", "type": "Enumeration", "value": item_operation},
        ]
        if identifier is not None:
            fields.append({"tag": "UniqueBatchItemID", "type": "ByteString", "value": identifier})
        fields.append({"tag": "ResultStatus", "type": "Enumeration", "value": status_name})
        if reason_name is not None:
            fields.append({"tag": "ResultReason", "type": "Enumeration", "value": reason_name})
        if status == protocol_values.ResultStatus.SUCCESS:
            versions = [] if supported_versions is None else [
                {"tag": "ProtocolVersion", "value": [
                    {"tag": "ProtocolVersionMajor", "type": "Integer", "value": major},
                    {"tag": "ProtocolVersionMinor", "type": "Integer", "value": minor},
                ]}
                for major, minor in supported_versions
            ]
            fields.append({"tag": "ResponsePayload", "value": versions})
        return fields

    if encoding == "JSON":
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
                        {"tag": "BatchCount", "type": "Integer", "value": len(operations)},
                    ],
                },
                *[{"tag": "BatchItem", "value": response_item(item_operation, batch_ids[index])}
                  for index, item_operation in enumerate(operations)],
            ],
        }
        return json.dumps(response, separators=(",", ":")).encode("utf-8")

    def xml_response_item(item_operation: str, identifier: str | None) -> str:
        fields = [
            f'<Operation type="Enumeration" value="{item_operation}"/>',
        ]
        if identifier is not None:
            fields.append(f'<UniqueBatchItemID type="ByteString" value="{identifier}"/>')
        fields.append(f'<ResultStatus type="Enumeration" value="{status_name}"/>')
        if reason_name is not None:
            fields.append(f'<ResultReason type="Enumeration" value="{reason_name}"/>')
        if status == protocol_values.ResultStatus.SUCCESS:
            if supported_versions is None:
                fields.append("<ResponsePayload/>")
            else:
                versions = "".join(
                    '<ProtocolVersion><ProtocolVersionMajor type="Integer" value="'
                    + str(major)
                    + '"/><ProtocolVersionMinor type="Integer" value="'
                    + str(minor)
                    + '"/></ProtocolVersion>'
                    for major, minor in supported_versions
                )
                fields.append("<ResponsePayload>" + versions + "</ResponsePayload>")
        return "<BatchItem>" + "".join(fields) + "</BatchItem>"

    document = (
        "<ResponseMessage><ResponseHeader><ProtocolVersion>"
        '<ProtocolVersionMajor type="Integer" value="2"/>'
        f'<ProtocolVersionMinor type="Integer" value="{protocol_minor}"/>'
        "</ProtocolVersion>"
        '<TimeStamp type="DateTime" value="1970-01-01T00:00:01Z"/>'
        f'<BatchCount type="Integer" value="{len(operations)}"/>'
        "</ResponseHeader>"
        + "".join(xml_response_item(item_operation, batch_ids[index])
                  for index, item_operation in enumerate(operations))
        + "</ResponseMessage>"
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
        cancellation_result: int | None = None,
        asynchronous_correlation_value: bytes | None = None,
        include_response_extension: bool = False,
    ) -> None:
        self.status = status
        self.reason = reason
        self.expect_no_connection = expect_no_connection
        self.encoding = encoding
        self.transport = transport
        self.protocol_minor = int(protocol_version.split(".")[1])
        self.cancellation_result = cancellation_result
        self.asynchronous_correlation_value = asynchronous_correlation_value
        self.include_response_extension = include_response_extension
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
                    if normalized_operation_name(str(self.request_observation["operation"])) == "discoverversions":
                        response = discover_versions_response(
                            request, self.status, self.reason, self.protocol_minor,
                            include_response_extension=self.include_response_extension,
                        )
                    else:
                        response = ping_response(
                            request, self.status, self.reason, self.protocol_minor,
                            self.cancellation_result, self.asynchronous_correlation_value,
                            self.include_response_extension,
                        )
                else:
                    response = text_response(
                        str(self.request_observation["operation"]),
                        self.encoding,
                        self.status,
                        self.reason,
                        self.protocol_minor,
                        supported_versions=((2, 1),)
                        if normalized_operation_name(str(self.request_observation["operation"])) == "discoverversions"
                        else None,
                        operations=[str(value) for value in self.request_observation["operations"]],
                        batch_ids=[None if value is None else str(value)
                                   for value in self.request_observation.get("batch_ids", [])],
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


