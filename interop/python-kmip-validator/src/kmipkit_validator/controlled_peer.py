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
from collections.abc import Callable
from .contract import ROOT

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


def ping_response(request: bytes, status: int, reason: int | None, protocol_minor: int = 1,
                  cancellation_result: int | None = None,
                  asynchronous_correlation_value: bytes | None = None) -> bytes:
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
        response_payload = []
        if asynchronous_correlation_value is not None:
            response_payload.append(item(0x420006, 8, asynchronous_correlation_value))
        if cancellation_result is not None:
            response_payload.append(enumeration(0x420012, cancellation_result))
        payload.append(structure(0x42007C, *response_payload))
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


def discover_versions_response(
    request: bytes,
    status: int,
    reason: int | None,
    protocol_minor: int,
    supported_versions: tuple[tuple[int, int], ...] | None = None,
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


def batch_response(
    request: bytes,
    encoding: str,
    operations: list[str],
    status: int,
    reason: int | None,
    protocol_minor: int,
    response_payloads: list[list[bytes]] | None = None,
    response_extensions: list[bytes] | None = None,
    result_statuses: list[int] | None = None,
    result_reasons: list[int | None] | None = None,
) -> bytes:
    """Return one ordered result per operation in the observed request batch."""
    from kmipkit import protocol_values

    status_values = result_statuses if result_statuses is not None else [status] * len(operations)
    reason_values = result_reasons if result_reasons is not None else [reason] * len(operations)
    if len(status_values) != len(operations) or len(reason_values) != len(operations):
        raise AssertionError("controlled result-code count differed from the batch count")

    request_ids: list[bytes | str | None] = []
    if encoding == "TTLV":
        request_tree = parse_children(request[8:])
        observed = []
        for tag, kind, value in request_tree:
            if tag != 0x42000F or kind != 1:
                continue
            fields = parse_children(value)
            raw_operation = find_value(fields, 0x42005C)
            if raw_operation is None:
                raise AssertionError("batch request item omitted its Operation")
            operation_name = protocol_values.Operation.name(int.from_bytes(raw_operation, "big"))
            if operation_name is None:
                raise AssertionError("batch request item used an unknown Operation")
            observed.append(operation_name)
            batch_item_id = next((field_value for field_tag, _, field_value in fields
                                  if field_tag == 0x420093), None)
            request_ids.append(batch_item_id)
        if [normalized_operation_name(value) for value in observed] != [
                normalized_operation_name(value) for value in operations]:
            raise AssertionError("batch request operation order differed from the controlled scenario")

        operation_codes = {
            "".join(character for character in name.casefold() if character.isalnum()): value
            for name, value in vars(protocol_values.Operation).items()
            if isinstance(value, int)
        }
        if response_payloads is not None and len(response_payloads) != len(observed):
            raise AssertionError("controlled response payload count differed from the batch count")
        if response_extensions is not None and len(response_extensions) != len(observed):
            raise AssertionError("controlled response extension count differed from the batch count")
        response_items = []
        for index, (operation, batch_item_id) in enumerate(
                zip(observed, request_ids, strict=True)):
            key = "".join(character for character in operation.casefold() if character.isalnum())
            fields = [enumeration(0x42005C, operation_codes[key])]
            if batch_item_id is not None:
                fields.append(item(0x420093, 8, batch_item_id))
            fields.append(enumeration(0x42007F, status_values[index]))
            if reason_values[index] is not None:
                fields.append(enumeration(0x42007E, reason_values[index]))
            if response_payloads is not None and response_payloads[index]:
                fields.append(structure(0x420079, *response_payloads[index]))
            if response_extensions is not None:
                fields.append(response_extensions[index])
            response_items.append(structure(0x42000F, *fields))
        version = structure(
            0x420069,
            item(0x42006A, 2, struct.pack(">i", 2)),
            item(0x42006B, 2, struct.pack(">i", protocol_minor)),
        )
        header = structure(
            0x42007A,
            version,
            item(0x420092, 9, struct.pack(">q", 1_794_268_800)),
            item(0x42000D, 2, struct.pack(">i", len(response_items))),
        )
        return structure(0x42007B, header, *response_items)

    if response_payloads is not None or response_extensions is not None:
        raise AssertionError("custom controlled batch results currently require TTLV encoding")
    if encoding == "JSON":
        request_message = json.loads(request)
        request_items = [value for value in request_message["value"]
                         if value.get("tag") == "BatchItem"]
        observed = []
        for request_item in request_items:
            fields = request_item["value"]
            operation = next(value["value"] for value in fields
                             if value.get("tag") == "Operation")
            observed.append(operation)
            batch_item_id = next((value for value in fields
                                  if value.get("tag") == "UniqueBatchItemID"), None)
            request_ids.append(None if batch_item_id is None else batch_item_id["value"])
        if [normalized_operation_name(value) for value in observed] != [
                normalized_operation_name(value) for value in operations]:
            raise AssertionError("batch request operation order differed from the controlled scenario")
    elif encoding == "XML":
        request_message = ET.fromstring(request)
        request_items = request_message.findall("./BatchItem")
        observed = []
        for request_item in request_items:
            operation = request_item.find("./Operation")
            if operation is None:
                raise AssertionError("batch request item omitted its Operation")
            observed.append(operation.attrib.get("value"))
            batch_item_id = request_item.find("./UniqueBatchItemID")
            request_ids.append(None if batch_item_id is None else batch_item_id.attrib.get("value"))
        if [normalized_operation_name(value) for value in observed] != [
                normalized_operation_name(value) for value in operations]:
            raise AssertionError("batch request operation order differed from the controlled scenario")
    response_operations = observed if encoding in ("JSON", "XML") else operations
    documents = [text_response(operation, encoding, status_values[index], reason_values[index], protocol_minor)
                 for index, operation in enumerate(response_operations)]
    if encoding == "JSON":
        messages = [json.loads(document) for document in documents]
        message = messages[0]
        header = next(value for value in message["value"] if value["tag"] == "ResponseHeader")
        batch_count = next(value for value in header["value"] if value["tag"] == "BatchCount")
        batch_count["value"] = len(operations)
        items = [next(value for value in source["value"] if value["tag"] == "BatchItem")
                 for source in messages]
        for batch_item, batch_item_id in zip(items, request_ids, strict=True):
            if batch_item_id is not None:
                batch_item["value"].insert(1, {
                    "tag": "UniqueBatchItemID", "type": "ByteString", "value": batch_item_id,
                })
        message["value"] = [header, *items]
        return json.dumps(message, separators=(",", ":")).encode("utf-8")
    if encoding == "XML":
        message = ET.fromstring(documents[0])
        header = message.find("./ResponseHeader/BatchCount")
        if header is None:
            raise AssertionError("controlled response omitted BatchCount")
        header.set("value", str(len(operations)))
        items = []
        for document, batch_item_id in zip(documents, request_ids, strict=True):
            source = ET.fromstring(document)
            batch_item = source.find("./BatchItem")
            if batch_item is None:
                raise AssertionError("controlled response omitted BatchItem")
            if batch_item_id is not None:
                operation = batch_item.find("./Operation")
                if operation is None:
                    raise AssertionError("controlled response item omitted Operation")
                position = list(batch_item).index(operation) + 1
                batch_item.insert(position, ET.Element(
                    "UniqueBatchItemID", {"type": "ByteString", "value": batch_item_id}
                ))
            items.append(batch_item)
        for batch_item in list(message.findall("./BatchItem")):
            message.remove(batch_item)
        message.extend(items)
        return ET.tostring(message, encoding="utf-8")
    raise AssertionError("controlled batch response has an unsupported encoding")


def request_observation(request: bytes, encoding: str) -> dict[str, object]:
    if encoding == "TTLV":
        tree = parse_children(request[8:])
        operation = int.from_bytes(find_value(tree, 0x42005C), "big")
        batch_count = int.from_bytes(find_value(tree, 0x42000D), "big", signed=True)
        from kmipkit import protocol_values

        batch_operations = []
        for tag, kind, value in tree:
            if tag != 0x42000F or kind != 1:
                continue
            operation_value = find_value(parse_children(value), 0x42005C)
            if operation_value is None:
                raise AssertionError("batch request item omitted its Operation")
            operation_name = protocol_values.Operation.name(int.from_bytes(operation_value, "big"))
            if operation_name is None:
                raise AssertionError("batch request item used an unknown Operation")
            batch_operations.append(operation_name)

        return {"operation": protocol_values.Operation.name(operation), "batch_count": batch_count,
                "batch_operations": batch_operations}
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

        batch_items = [value for value in tree.get("value", [])
                       if value.get("tag") == "BatchItem"]
        batch_operations = [next((field.get("value") for field in batch_item.get("value", [])
                                  if field.get("tag") == "Operation"), None)
                            for batch_item in batch_items]
        if any(value is None for value in batch_operations):
            raise AssertionError("batch request item omitted its Operation")
        return {"operation": find(tree, "Operation"), "batch_count": find(tree, "BatchCount"),
                "batch_operations": batch_operations}
    root = ET.fromstring(request)
    operation = root.find(".//Operation")
    batch_count = root.find(".//BatchCount")
    if operation is None or batch_count is None:
        raise AssertionError("XML request omitted Operation or BatchCount")
    return {
        "operation": operation.attrib.get("value"),
        "batch_count": int(batch_count.attrib["value"]),
        "batch_operations": [value.attrib.get("value")
                             for value in root.findall("./BatchItem/Operation")],
    }


def normalized_operation_name(value: str) -> str:
    return "".join(character for character in value.casefold() if character.isalnum())


def text_response(
    operation: str, encoding: str, status: int, reason: int | None, protocol_minor: int = 1,
    supported_versions: tuple[tuple[int, int], ...] | None = None,
) -> bytes:
    from kmipkit import protocol_values

    status_name = "".join(protocol_values.ResultStatus.name(status).split())
    reason_name = (
        None
        if reason is None
        else "".join(protocol_values.ResultReason.name(reason).split())
    )
    if encoding == "JSON":
        batch_item = [
            {"tag": "Operation", "type": "Enumeration", "value": operation},
            {"tag": "ResultStatus", "type": "Enumeration", "value": status_name},
        ]
        if reason_name is not None:
            batch_item.append({"tag": "ResultReason", "type": "Enumeration", "value": reason_name})
        if status == protocol_values.ResultStatus.SUCCESS:
            versions = [] if supported_versions is None else [
                {"tag": "ProtocolVersion", "value": [
                    {"tag": "ProtocolVersionMajor", "type": "Integer", "value": major},
                    {"tag": "ProtocolVersionMinor", "type": "Integer", "value": minor},
                ]}
                for major, minor in supported_versions
            ]
            batch_item.append({"tag": "ResponsePayload", "value": versions})
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
        cancellation_result: int | None = None,
        asynchronous_correlation_value: bytes | None = None,
        response_builder: Callable[[bytes, str, int], bytes] | None = None,
    ) -> None:
        self.status = status
        self.reason = reason
        self.expect_no_connection = expect_no_connection
        self.encoding = encoding
        self.transport = transport
        self.protocol_minor = int(protocol_version.split(".")[1])
        self.cancellation_result = cancellation_result
        self.asynchronous_correlation_value = asynchronous_correlation_value
        self.response_builder = response_builder
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
                if self.response_builder is not None:
                    response = self.response_builder(request, self.encoding, self.protocol_minor)
                elif self.encoding == "TTLV":
                    if normalized_operation_name(str(self.request_observation["operation"])) == "discoverversions":
                        response = discover_versions_response(
                            request, self.status, self.reason, self.protocol_minor
                        )
                    else:
                        response = ping_response(
                            request, self.status, self.reason, self.protocol_minor,
                            self.cancellation_result, self.asynchronous_correlation_value,
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


