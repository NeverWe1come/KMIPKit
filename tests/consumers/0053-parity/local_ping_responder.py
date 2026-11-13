"""Loopback-only TLS 1.3/mTLS fixture for the four external Ping consumers."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import socket
import ssl
import struct


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _response(operation: int, pending: bool = False, failed: bool = False) -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 1)),
    )
    if pending:
        result_fields = [
            _item(0x42005C, 5, struct.pack(">I", operation)),
            _item(0x42007F, 5, struct.pack(">I", 2)),
            _item(0x420006, 8, b"destroy-pending-id"),
            _structure(0x42007C),
        ]
        response_item = _structure(0x42000F, *result_fields)
        return _structure(0x42007B, header, response_item)
    if failed:
        response_item = _structure(
            0x42000F,
            _item(0x42005C, 5, struct.pack(">I", operation)),
            _item(0x42007F, 5, struct.pack(">I", 1)),
            _item(0x42007E, 5, struct.pack(">I", 1)),
            _item(0x42007D, 7, b"fixture rejected destroy"),
        )
        return _structure(0x42007B, header, response_item)
    if operation == 1:
        payload = _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, b"fluent-key"),
        )
    elif operation == 10:
        payload = _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, b"fluent-key"),
            _structure(0x42008F),
        )
    elif operation == 20:
        payload = _structure(0x42007C, _item(0x420094, 7, b"fluent-key"))
    elif operation == 0x3B:
        payload = _structure(0x42007C)
    else:
        raise RuntimeError(f"fixture received unsupported KMIP operation {operation:#x}")
    response_item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", operation)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        payload,
    )
    return _structure(0x42007B, header, response_item)


def _read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    data = bytearray()
    while len(data) < length:
        chunk = stream.recv(length - len(data))
        if not chunk:
            raise RuntimeError("consumer closed before sending a complete TTLV item")
        data.extend(chunk)
    return bytes(data)


def _read_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    body_length = int.from_bytes(header[4:8], "big")
    if body_length > 16 * 1024 * 1024 or body_length % 8:
        raise RuntimeError("consumer sent an invalid or oversized TTLV frame")
    return header + _read_exact(stream, body_length)


def _children(value: bytes) -> list[tuple[int, int, bytes]]:
    children: list[tuple[int, int, bytes]] = []
    offset = 0
    while offset < len(value):
        if len(value) - offset < 8:
            raise RuntimeError("consumer sent a truncated TTLV child header")
        tag = int.from_bytes(value[offset : offset + 3], "big")
        item_type = value[offset + 3]
        length = int.from_bytes(value[offset + 4 : offset + 8], "big")
        value_start = offset + 8
        value_end = value_start + length
        padded_end = value_end + (-length) % 8
        if padded_end > len(value) or any(value[value_end:padded_end]):
            raise RuntimeError("consumer sent a truncated TTLV child or nonzero padding")
        children.append((tag, item_type, value[value_start:value_end]))
        offset = padded_end
    return children


def _flatten(items: list[tuple[int, int, bytes]]) -> list[tuple[int, int, bytes]]:
    flattened = list(items)
    for _tag, kind, value in items:
        if kind == 1:
            flattened.extend(_flatten(_children(value)))
    return flattened


def _request_operation(
    frame: bytes,
) -> tuple[int, list[tuple[int, int, bytes]], list[tuple[int, int, bytes]]]:
    if frame[:4] != bytes.fromhex("42007801"):
        raise RuntimeError("consumer sent an invalid Request Message")
    body_length = int.from_bytes(frame[4:8], "big")
    if body_length != len(frame) - 8:
        raise RuntimeError("consumer sent a malformed request frame")
    top_level = _children(frame[8:])
    batch_items = [
        value for tag, item_type, value in top_level if tag == 0x42000F and item_type == 1
    ]
    headers = [
        value for tag, item_type, value in top_level if tag == 0x420077 and item_type == 1
    ]
    if len(batch_items) != 1:
        raise RuntimeError("fixture expects one typed Batch Item per exchange")
    if len(headers) != 1:
        raise RuntimeError("fixture expects one Request Header")
    fields = _children(batch_items[0])
    operations = [
        value
        for tag, item_type, value in fields
        if tag == 0x42005C and item_type == 5
    ]
    if len(operations) != 1 or len(operations[0]) != 4:
        raise RuntimeError("fixture could not read the typed KMIP operation")
    return int.from_bytes(operations[0], "big"), fields, _children(headers[0])


def _validate_operation(
    operation: int,
    fields: list[tuple[int, int, bytes]],
    header_fields: list[tuple[int, int, bytes]],
) -> None:
    payloads = [value for tag, kind, value in fields if tag == 0x420079 and kind == 1]
    if operation in (1, 10, 20) and len(payloads) != 1:
        raise RuntimeError(
            "lifecycle request must contain one Request Payload; "
            f"operation={operation} tags={[hex(tag) for tag, _kind, _value in fields]}"
        )
    if operation == 1:
        payload = _children(payloads[0])
        flattened_payload = _flatten(payload)
        values = {tag: (kind, value) for tag, kind, value in flattened_payload}
        if values.get(0x420057) != (5, struct.pack(">I", 2)):
            raise RuntimeError("Create did not select Symmetric Key")
        if values.get(0x420028) != (5, struct.pack(">I", 3)):
            raise RuntimeError(
                "Create Cryptographic Algorithm was not AES; "
                f"payload tags={[hex(tag) for tag, _kind, _value in flattened_payload]}"
            )
        if values.get(0x42002A) != (2, struct.pack(">i", 256)):
            raise RuntimeError("Create Cryptographic Length was not 256 bits")
        if values.get(0x42002C) != (2, struct.pack(">i", 12)):
            raise RuntimeError("Create Cryptographic Usage Mask was not explicit")
    elif operation in (10, 20):
        payload = _children(payloads[0])
        identifiers = [value for tag, kind, value in payload if tag == 0x420094 and kind == 7]
        if identifiers != [b"fluent-key"]:
            raise RuntimeError("Get or Destroy did not use the returned Unique Identifier")
    if operation in (10, 20):
        indicators = [value for tag, kind, value in header_fields if tag == 0x420007 and kind == 5]
        if operation == 10 and indicators != [struct.pack(">I", 1)]:
            raise RuntimeError("Get omitted the explicit Request Header Asynchronous Indicator")
        if operation == 20 and indicators not in ([], [struct.pack(">I", 1)]):
            raise RuntimeError("Destroy contains an invalid Request Header Asynchronous Indicator")


def _has_async_indicator(header_fields: list[tuple[int, int, bytes]]) -> bool:
    return any(tag == 0x420007 and kind == 5 for tag, kind, _value in header_fields)



def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ready-file", type=Path, required=True)
    parser.add_argument("--pid-file", type=Path, required=True)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--connections", type=int, default=8)
    args = parser.parse_args()

    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.maximum_version = ssl.TLSVersion.TLSv1_3
    context.load_cert_chain(args.fixture_dir / "server-cert.pem", args.fixture_dir / "server-key.pem")
    context.load_verify_locations(cafile=args.fixture_dir / "client-cert.pem")
    context.verify_mode = ssl.CERT_REQUIRED

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", 0))
        listener.listen(args.connections)
        # Native/JNI builds run before Java connects in the consumer runner.
        listener.settimeout(180)
        args.pid_file.write_text(str(os.getpid()), encoding="ascii")
        args.ready_file.write_text(str(listener.getsockname()[1]), encoding="ascii")
        try:
            async_destroy_count = 0
            for connection_index in range(args.connections):
                raw, _address = listener.accept()
                raw.settimeout(10)
                with raw, context.wrap_socket(raw, server_side=True) as stream:
                    peer_cert = stream.getpeercert(binary_form=True)
                    if stream.version() != "TLSv1.3" or not peer_cert:
                        raise RuntimeError("fixture requires an authenticated TLS 1.3 client")
                    request = _read_frame(stream)
                    operation, fields, header_fields = _request_operation(request)
                    _validate_operation(operation, fields, header_fields)
                    asynchronous_destroy = operation == 20 and _has_async_indicator(header_fields)
                    pending = asynchronous_destroy and async_destroy_count == 0
                    failed = asynchronous_destroy and async_destroy_count == 1
                    if asynchronous_destroy:
                        async_destroy_count += 1
                    stream.sendall(_response(operation, pending, failed))
        finally:
            args.pid_file.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
