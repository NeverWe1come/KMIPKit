"""Loopback-only TLS 1.3/mTLS fixture for four external fluent API consumers."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import socket
import ssl
import struct

_FLUENT_KEY_ID = b"fluent-key"
_PLAINTEXT = b"fluent plaintext"
_CIPHERTEXT = b"fixture ciphertext"
_NONCE = b"nonce-12byte"
_AAD = b"fluent context"
_AUTH_TAG = b"fixture-auth-tag"
_PAIR_PRIVATE_ID = b"fluent-private-key"
_PAIR_PUBLIC_ID = b"fluent-public-key"
_EXPERT_KEY_NAME = b"fluent-expert-key"
_EXPERT_KEY_ID = b"fluent-expert-key"
_EXPERT_VENDOR = b"KMIPKit.Example"
_EXPERT_ATTRIBUTE = b"purpose"
_EXPERT_VALUE = b"fluent expert journey"
_SIGN_INPUT = b"fluent sign input"
_SIGNATURE = b"fixture signature"


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _response(
    operation: int,
    pending: bool = False,
    failed: bool = False,
    unique_identifier: bytes | None = None,
    unique_batch_item_id: bytes | None = None,
) -> bytes:
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
            _item(0x420094, 7, unique_identifier or b"fluent-key"),
        )
    elif operation == 2:
        payload = _structure(
            0x42007C,
            _item(0x420066, 7, _PAIR_PRIVATE_ID),
            _item(0x42006F, 7, _PAIR_PUBLIC_ID),
        )
    elif operation == 10:
        payload = _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, unique_identifier or b"fluent-key"),
            _structure(0x42008F),
        )
    elif operation == 20:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, unique_identifier or b"fluent-key"),
        )
    elif operation == 0x1F:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, _FLUENT_KEY_ID),
            _item(0x4200C2, 8, _CIPHERTEXT),
            _item(0x42003D, 8, _NONCE),
            _item(0x4200FF, 8, _AUTH_TAG),
        )
    elif operation == 0x20:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, _FLUENT_KEY_ID),
            _item(0x4200C2, 8, _PLAINTEXT),
        )
    elif operation == 0x21:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, _PAIR_PRIVATE_ID),
            _item(0x4200C3, 8, _SIGNATURE),
        )
    elif operation == 0x3B:
        payload = _structure(0x42007C)
    else:
        raise RuntimeError(f"fixture received unsupported KMIP operation {operation:#x}")
    response_fields = [_item(0x42005C, 5, struct.pack(">I", operation))]
    if unique_batch_item_id is not None:
        response_fields.append(_item(0x420093, 8, unique_batch_item_id))
    response_fields.extend([
        _item(0x42007F, 5, struct.pack(">I", 0)),
        payload,
    ])
    response_item = _structure(0x42000F, *response_fields)
    return _structure(0x42007B, header, response_item)


def _batch_response(items: list[tuple[int, bytes | None, bytes]]) -> bytes:
    responses = []
    header_fields = None
    for operation, unique_identifier, unique_batch_item_id in items:
        encoded = _response(
            operation,
            unique_identifier=unique_identifier,
            unique_batch_item_id=unique_batch_item_id,
        )
        message_fields = _children(encoded[8:])
        header = next(value for tag, kind, value in message_fields if tag == 0x42007A and kind == 1)
        if header_fields is None:
            header_fields = _children(header)
        response_value = next(
            value for tag, kind, value in message_fields if tag == 0x42000F and kind == 1
        )
        responses.append(
            _structure(
                0x42000F,
                *(_item(tag, kind, value) for tag, kind, value in _children(response_value)),
            )
        )
    if not responses or header_fields is None:
        raise RuntimeError("fixture batch response requires at least one response item")
    encoded_header_fields = [
        _item(tag, kind, value)
        for tag, kind, value in header_fields
        if tag != 0x42000D
    ]
    encoded_header_fields.append(_item(0x42000D, 2, struct.pack(">i", len(responses))))
    return _structure(
        0x42007B,
        _structure(0x42007A, *encoded_header_fields),
        *responses,
    )


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


def _request_message(
    frame: bytes,
) -> tuple[list[tuple[int, list[tuple[int, int, bytes]]]], list[tuple[int, int, bytes]]]:
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
    if not batch_items:
        raise RuntimeError("fixture expects at least one typed Batch Item per exchange")
    if len(headers) != 1:
        raise RuntimeError("fixture expects one Request Header")
    request_items = []
    for batch_item in batch_items:
        fields = _children(batch_item)
        operations = [
            value
            for tag, item_type, value in fields
            if tag == 0x42005C and item_type == 5
        ]
        if len(operations) != 1 or len(operations[0]) != 4:
            raise RuntimeError("fixture could not read the typed KMIP operation")
        request_items.append((int.from_bytes(operations[0], "big"), fields))
    return request_items, _children(headers[0])


def _request_operation(
    frame: bytes,
) -> tuple[int, list[tuple[int, int, bytes]], list[tuple[int, int, bytes]]]:
    request_items, header_fields = _request_message(frame)
    if len(request_items) != 1:
        raise RuntimeError("fixture expected one operation for this single-item check")
    operation, fields = request_items[0]
    return operation, fields, header_fields


def _requested_identifier(
    operation: int,
    fields: list[tuple[int, int, bytes]],
) -> bytes | None:
    if operation not in (10, 20):
        return None
    payloads = [value for tag, kind, value in fields if tag == 0x420079 and kind == 1]
    if len(payloads) != 1:
        raise RuntimeError("Get or Destroy request must contain one Request Payload")
    identifiers = [
        value for tag, kind, value in _children(payloads[0]) if tag == 0x420094 and kind == 7
    ]
    if len(identifiers) != 1:
        raise RuntimeError("Get or Destroy request must contain one Unique Identifier")
    return identifiers[0]


def _requested_name(
    operation: int,
    fields: list[tuple[int, int, bytes]],
) -> bytes | None:
    if operation != 1:
        return None
    payloads = [value for tag, kind, value in fields if tag == 0x420079 and kind == 1]
    if len(payloads) != 1:
        return None
    names = [
        value
        for tag, kind, value in _flatten(_children(payloads[0]))
        if tag in (0x420055, 0x420098) and kind == 7
    ]
    return names[0] if len(names) == 1 else None


def _uses_id_placeholder(
    operation: int,
    fields: list[tuple[int, int, bytes]],
) -> bool:
    if operation not in (10, 20):
        return False
    payloads = [value for tag, kind, value in fields if tag == 0x420079 and kind == 1]
    if len(payloads) != 1:
        return False
    identifiers = [tag for tag, _kind, _value in _children(payloads[0]) if tag == 0x420094]
    return not identifiers


def _requested_batch_item_id(fields: list[tuple[int, int, bytes]]) -> bytes:
    identifiers = [
        value for tag, kind, value in fields if tag == 0x420093 and kind == 8
    ]
    if len(identifiers) != 1 or not identifiers[0]:
        raise RuntimeError("each batch request item must contain one non-empty Unique Batch Item ID")
    return identifiers[0]


def _validate_id_placeholder_batch(
    request_items: list[tuple[int, list[tuple[int, int, bytes]]]],
    header_fields: list[tuple[int, int, bytes]],
) -> None:
    operations = [operation for operation, _fields in request_items]
    if operations != [1, 10]:
        raise RuntimeError("fixture batch must contain Create followed by Get")
    order_options = [
        value for tag, kind, value in header_fields if tag == 0x420010 and kind == 6
    ]
    if order_options != [struct.pack(">Q", 1)]:
        raise RuntimeError("fixture ID Placeholder batch requires Batch Order Option true")
    batch_item_ids = [_requested_batch_item_id(fields) for _operation, fields in request_items]
    if len(set(batch_item_ids)) != len(batch_item_ids):
        raise RuntimeError("fixture ID Placeholder batch requires distinct Unique Batch Item IDs")
    if not _requested_name(1, request_items[0][1]):
        raise RuntimeError("fixture ID Placeholder producer must name the created key")
    if not _uses_id_placeholder(10, request_items[1][1]):
        raise RuntimeError("fixture ID Placeholder consumer must use Enumeration 1")


def _validate_operation(
    operation: int,
    fields: list[tuple[int, int, bytes]],
    header_fields: list[tuple[int, int, bytes]],
) -> None:
    payloads = [value for tag, kind, value in fields if tag == 0x420079 and kind == 1]
    if operation in (1, 2, 10, 20, 0x21) and len(payloads) != 1:
        raise RuntimeError(
            "key lifecycle or signing request must contain one Request Payload; "
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
        if any(
            tag in (0x420055, 0x420098) and kind == 7 and value == _EXPERT_KEY_NAME
            for tag, kind, value in flattened_payload
        ):
            vendor_attributes = [
                value
                for tag, kind, value in flattened_payload
                if tag == 0x420008 and kind == 1
            ]
            if len(vendor_attributes) != 1:
                raise RuntimeError("expert Create omitted the expected Vendor Attribute")
            vendor_fields = _children(vendor_attributes[0])
            if vendor_fields != [
                (0x42009D, 7, _EXPERT_VENDOR),
                (0x42000A, 7, _EXPERT_ATTRIBUTE),
                (0x42000B, 7, _EXPERT_VALUE),
            ]:
                raise RuntimeError("expert Create encoded an unexpected Vendor Attribute")
    elif operation == 2:
        payload = _children(payloads[0])
        groups = {
            tag: _children(value)
            for tag, kind, value in payload
            if kind == 1 and tag in (0x420126, 0x420127, 0x420128)
        }
        common = {tag: (kind, value) for tag, kind, value in groups.get(0x420126, [])}
        private = {tag: (kind, value) for tag, kind, value in groups.get(0x420127, [])}
        public = {tag: (kind, value) for tag, kind, value in groups.get(0x420128, [])}
        if common.get(0x420028) != (5, struct.pack(">I", 6)):
            raise RuntimeError("Create Key Pair did not select ECDSA")
        if common.get(0x42002A) != (2, struct.pack(">i", 256)):
            raise RuntimeError("Create Key Pair did not select a 256-bit key")
        if private.get(0x42002C) != (2, struct.pack(">i", 1)):
            raise RuntimeError("Create Key Pair did not select private Sign usage")
        if public.get(0x42002C) != (2, struct.pack(">i", 2)):
            raise RuntimeError("Create Key Pair did not select public Verify usage")
    elif operation in (10, 20):
        payload = _children(payloads[0])
        identifiers = [value for tag, kind, value in payload if tag == 0x420094 and kind == 7]
        if operation == 10 and identifiers not in ([b"fluent-key"], [_EXPERT_KEY_ID]):
            if not _uses_id_placeholder(operation, fields):
                raise RuntimeError("Get did not use a returned identifier or ID Placeholder")
        if operation == 20 and identifiers not in (
            [b"fluent-key"], [_PAIR_PRIVATE_ID], [_PAIR_PUBLIC_ID], [_EXPERT_KEY_ID]
        ):
            raise RuntimeError("Destroy did not use a returned Unique Identifier")
    elif operation in (0x1F, 0x20):
        payload = _children(payloads[0])
        identifiers = [value for tag, kind, value in payload if tag == 0x420094 and kind == 7]
        if identifiers != [_FLUENT_KEY_ID]:
            raise RuntimeError("Encrypt or Decrypt did not use the returned Unique Identifier")
        data = [value for tag, kind, value in payload if tag == 0x4200C2 and kind == 8]
        expected_data = _PLAINTEXT if operation == 0x1F else _CIPHERTEXT
        if data != [expected_data]:
            raise RuntimeError("Encrypt or Decrypt used unexpected input Data")
        nonces = [value for tag, kind, value in payload if tag == 0x42003D and kind == 8]
        additional_data = [
            value for tag, kind, value in payload if tag == 0x4200FE and kind == 8
        ]
        if nonces != [_NONCE] or additional_data != [_AAD]:
            raise RuntimeError("Encrypt or Decrypt omitted the selected GCM nonce or AAD")
        parameter_structures = [
            value for tag, kind, value in payload if tag == 0x42002B and kind == 1
        ]
        if len(parameter_structures) != 1:
            raise RuntimeError("Encrypt or Decrypt omitted Cryptographic Parameters")
        parameters = {tag: (kind, value) for tag, kind, value in _children(parameter_structures[0])}
        expected_parameters = {
            0x420011: (5, struct.pack(">I", 9)),
            0x420028: (5, struct.pack(">I", 3)),
            0x4200CD: (2, struct.pack(">i", 96)),
            0x4200CE: (2, struct.pack(">i", 16)),
        }
        if any(parameters.get(tag) != value for tag, value in expected_parameters.items()):
            raise RuntimeError("Encrypt or Decrypt did not select explicit AES-GCM parameters")
        tags = [value for tag, kind, value in payload if tag == 0x4200FF and kind == 8]
        if (operation == 0x1F and tags) or (operation == 0x20 and tags != [_AUTH_TAG]):
            raise RuntimeError("Encrypt or Decrypt has an invalid authenticated encryption tag")
    elif operation == 0x21:
        payload = _children(payloads[0])
        identifiers = [value for tag, kind, value in payload if tag == 0x420094 and kind == 7]
        data = [value for tag, kind, value in payload if tag == 0x4200C2 and kind == 8]
        parameter_structures = [
            value for tag, kind, value in payload if tag == 0x42002B and kind == 1
        ]
        if identifiers != [_PAIR_PRIVATE_ID] or data != [_SIGN_INPUT]:
            raise RuntimeError("Sign did not use the returned private key and example input")
        if len(parameter_structures) != 1:
            raise RuntimeError("Sign omitted explicit Cryptographic Parameters")
        parameters = {
            tag: (kind, value) for tag, kind, value in _children(parameter_structures[0])
        }
        if parameters.get(0x420038) != (5, struct.pack(">I", 6)) or parameters.get(
            0x4200AE
        ) != (5, struct.pack(">I", 0x0E)):
            raise RuntimeError("Sign did not select ECDSA with SHA-256")
    if operation in (10, 20):
        indicators = [value for tag, kind, value in header_fields if tag == 0x420007 and kind == 5]
        if operation == 10 and indicators not in ([], [struct.pack(">I", 1)]):
            raise RuntimeError("Get contains an invalid Request Header Asynchronous Indicator")
        if operation == 20 and indicators not in ([], [struct.pack(">I", 1)]):
            raise RuntimeError("Destroy contains an invalid Request Header Asynchronous Indicator")


def _has_async_indicator(header_fields: list[tuple[int, int, bytes]]) -> bool:
    return any(tag == 0x420007 and kind == 5 for tag, kind, _value in header_fields)


def _parse_operations(value: str) -> list[list[int]]:
    try:
        operations = [
            [int(token, 0) for token in group.split("+")]
            for group in value.split(";")
        ]
    except ValueError as error:
        raise argparse.ArgumentTypeError(
            "expected operation groups must use semicolons and plus-separated batch items"
        ) from error
    if not operations or any(not group for group in operations):
        raise argparse.ArgumentTypeError("at least one expected operation is required")
    return operations



def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ready-file", type=Path, required=True)
    parser.add_argument("--pid-file", type=Path, required=True)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--connections", type=int, default=8)
    parser.add_argument("--expected-operations", type=_parse_operations)
    args = parser.parse_args()
    if args.expected_operations is not None and len(args.expected_operations) != args.connections:
        parser.error("--expected-operations must contain one operation group for each connection")

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
                    request_items, header_fields = _request_message(request)
                    operations = [operation for operation, _fields in request_items]
                    if len(request_items) > 1:
                        _validate_id_placeholder_batch(request_items, header_fields)
                    if (
                        args.expected_operations is not None
                        and operations != args.expected_operations[connection_index]
                    ):
                        expected = args.expected_operations[connection_index]
                        raise RuntimeError(
                            "fixture received an unexpected operation at connection "
                            f"{connection_index + 1}: expected {expected}, got {operations}"
                        )
                    response_items = []
                    placeholder_identifier = None
                    for operation, fields in request_items:
                        _validate_operation(operation, fields, header_fields)
                        response_identifier = None
                        if operation == 1:
                            response_identifier = _requested_name(operation, fields)
                            if response_identifier is not None:
                                placeholder_identifier = response_identifier
                        elif _uses_id_placeholder(operation, fields):
                            if placeholder_identifier is None:
                                raise RuntimeError("ID Placeholder consumer has no preceding producer")
                            response_identifier = placeholder_identifier
                        else:
                            response_identifier = _requested_identifier(operation, fields)
                        batch_item_id = (
                            _requested_batch_item_id(fields) if len(request_items) > 1 else None
                        )
                        response_items.append((operation, response_identifier, batch_item_id))
                    asynchronous_destroy = any(
                        operation == 20 for operation in operations
                    ) and _has_async_indicator(header_fields)
                    pending = asynchronous_destroy and async_destroy_count == 0
                    failed = asynchronous_destroy and async_destroy_count == 1
                    if asynchronous_destroy:
                        async_destroy_count += 1
                    if len(response_items) == 1:
                        operation, response_identifier, _batch_item_id = response_items[0]
                        stream.sendall(_response(operation, pending, failed, response_identifier))
                    else:
                        stream.sendall(_batch_response(response_items))
        finally:
            args.pid_file.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
