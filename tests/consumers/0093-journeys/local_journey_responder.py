"""Loopback-only TLS 1.3/mTLS fixture for the four fluent journey consumers."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import socket
import ssl
import struct


_AES_GCM_JOURNEY_OPERATIONS = (1, 10, 20, 1, 0x1F, 0x20, 20)
_KEY_PAIR_JOURNEY_OPERATIONS = (2, 0x21, 0x22)
_EXPERT_ATTRIBUTE_JOURNEY_OPERATIONS = (1, 20)
_CONSUMER_JOURNEY_OPERATIONS = (
    _AES_GCM_JOURNEY_OPERATIONS
    + _KEY_PAIR_JOURNEY_OPERATIONS
    + _EXPERT_ATTRIBUTE_JOURNEY_OPERATIONS
)
EXPECTED_CONSUMER_OPERATIONS = _CONSUMER_JOURNEY_OPERATIONS * 4
_EXPECTED_PER_CONSUMER_REQUESTS = tuple(
    (operation,) for operation in _CONSUMER_JOURNEY_OPERATIONS
) + ((1, 10),)
EXPECTED_CONSUMER_REQUESTS = _EXPECTED_PER_CONSUMER_REQUESTS * 4
LIFECYCLE_IDENTIFIER = b"fluent-key"
AES_GCM_IDENTIFIER = b"fluent-aes-gcm-key"
PRIVATE_KEY_IDENTIFIER = b"fluent-signing-private"
PUBLIC_KEY_IDENTIFIER = b"fluent-verification-public"
EXPERT_ATTRIBUTE_IDENTIFIER = b"fluent-expert-attributes"
BATCH_PLACEHOLDER_NAME = b"fluent-batch-placeholder"
BATCH_PLACEHOLDER_IDENTIFIER = b"server-created-batch-key"
AES_GCM_PLAINTEXT = bytes.fromhex("4b4d49504b6974204145532d47434d206a6f75726e6579")
AES_GCM_CIPHERTEXT = bytes.fromhex("0c4f9f4b8e8cb63bcc04c4a6f6aa354de9b9f2469e1e26")
AES_GCM_IV = bytes.fromhex("000102030405060708090a0b")
AES_GCM_AAD = b"kmipkit-gcm-aad"
AES_GCM_TAG = bytes.fromhex("b516ad2a79bf55d147482f13feed6ea6")
SIGNATURE_DATA = bytes.fromhex("3045022100aabbccddeeff0011223344556677889902210033445566778899aabbccddeeff0011")
SIGNATURE_PLAINTEXT = b"KMIPKit ECDSA signature journey"


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _response(operation: int, identifier: bytes) -> bytes:
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
    if operation == 1:
        payload = _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, identifier),
        )
    elif operation == 10:
        payload = _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, identifier),
            _structure(0x42008F),
        )
    elif operation == 20:
        payload = _structure(0x42007C, _item(0x420094, 7, identifier))
    elif operation == 0x1F:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, identifier),
            _item(0x4200C2, 8, AES_GCM_CIPHERTEXT),
            _item(0x42003D, 8, AES_GCM_IV),
            _item(0x4200FF, 8, AES_GCM_TAG),
        )
    elif operation == 0x20:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, identifier),
            _item(0x4200C2, 8, AES_GCM_PLAINTEXT),
        )
    elif operation == 2:
        payload = _structure(
            0x42007C,
            _item(0x420066, 7, PRIVATE_KEY_IDENTIFIER),
            _item(0x42006F, 7, PUBLIC_KEY_IDENTIFIER),
        )
    elif operation == 0x21:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, PRIVATE_KEY_IDENTIFIER),
            _item(0x4200C3, 8, SIGNATURE_DATA),
        )
    elif operation == 0x22:
        payload = _structure(
            0x42007C,
            _item(0x420094, 7, PUBLIC_KEY_IDENTIFIER),
            _item(0x42009B, 5, struct.pack(">I", 1)),
        )
    else:
        raise RuntimeError(f"fixture received unsupported KMIP operation {operation:#x}")
    response_item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", operation)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        payload,
    )
    return _structure(0x42007B, header, response_item)


def _response_batch(
    items: tuple[tuple[int, bytes], ...], identifier: bytes
) -> bytes:
    if not items:
        raise RuntimeError("fixture cannot return an empty response batch")
    first_response = _children(_response(items[0][0], identifier)[8:])
    original_header = next(
        value for tag, kind, value in first_response if tag == 0x42007A and kind == 1
    )
    header_fields = [
        (tag, kind, value)
        for tag, kind, value in _children(original_header)
        if tag != 0x42000D
    ]
    header = _structure(
        0x42007A,
        *(_item(tag, kind, value) for tag, kind, value in header_fields),
        _item(0x42000D, 2, struct.pack(">i", len(items))),
    )
    response_items = []
    for operation, unique_batch_item_id in items:
        fields = _children(
            next(
                value
                for tag, kind, value in _children(_response(operation, identifier)[8:])
                if tag == 0x42000F and kind == 1
            )
        )
        if not fields or fields[0][:2] != (0x42005C, 5):
            raise RuntimeError("fixture response batch item must begin with Operation")
        fields.insert(1, (0x420093, 8, unique_batch_item_id))
        response_items.append(
            _structure(
                0x42000F,
                *(_item(tag, kind, value) for tag, kind, value in fields),
            )
        )
    return _structure(0x42007B, header, *response_items)


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


def _request_batch(
    frame: bytes,
) -> tuple[
    list[tuple[int, list[tuple[int, int, bytes]]]],
    list[tuple[int, int, bytes]],
]:
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
    header_fields = _children(headers[0])
    declared_counts = [
        value for tag, kind, value in header_fields if tag == 0x42000D and kind == 2
    ]
    if declared_counts != [struct.pack(">i", len(batch_items))]:
        raise RuntimeError("Request Header Batch Count does not match the received batch items")

    items: list[tuple[int, list[tuple[int, int, bytes]]]] = []
    for encoded_item in batch_items:
        fields = _children(encoded_item)
        operations = [
            value
            for tag, item_type, value in fields
            if tag == 0x42005C and item_type == 5
        ]
        if len(operations) != 1 or len(operations[0]) != 4:
            raise RuntimeError("fixture could not read a typed KMIP operation")
        items.append((int.from_bytes(operations[0], "big"), fields))
    return items, header_fields


def _request_operation(
    frame: bytes,
) -> tuple[int, list[tuple[int, int, bytes]], list[tuple[int, int, bytes]]]:
    items, header_fields = _request_batch(frame)
    if len(items) != 1:
        raise RuntimeError("fixture expects one typed Batch Item for this journey exchange")
    operation, fields = items[0]
    return operation, fields, header_fields


def _validate_placeholder_batch(
    items: list[tuple[int, list[tuple[int, int, bytes]]]],
    header_fields: list[tuple[int, int, bytes]],
) -> None:
    if len(items) != 2 or [operation for operation, _fields in items] != [1, 10]:
        raise RuntimeError("ID Placeholder batch must order Create before Get")
    unique_batch_item_ids = []
    for _operation, fields in items:
        values = [
            value for tag, kind, value in fields if tag == 0x420093 and kind == 8
        ]
        if len(values) != 1 or not values[0]:
            raise RuntimeError("each batch item requires a Unique Batch Item ID")
        unique_batch_item_ids.append(values[0])
    if unique_batch_item_ids != [
        b"create-placeholder-source",
        b"get-placeholder-target",
    ]:
        raise RuntimeError("Unique Batch Item IDs must be distinct and ordered with their requests")
    ordered = [value for tag, kind, value in header_fields if tag == 0x420010 and kind == 6]
    if ordered != [struct.pack(">Q", 1)]:
        raise RuntimeError("ID Placeholder batch requires Batch Order Option true")

    create_payloads = [
        value for tag, kind, value in items[0][1] if tag == 0x420079 and kind == 1
    ]
    get_payloads = [
        value for tag, kind, value in items[1][1] if tag == 0x420079 and kind == 1
    ]
    if len(create_payloads) != 1 or len(get_payloads) != 1:
        raise RuntimeError("Create and Get must each carry one Request Payload")

    create_values = _flatten(_children(create_payloads[0]))
    if any(tag == 0x420094 for tag, _kind, _value in create_values):
        raise RuntimeError("Create must not supply a guessed Unique Identifier")
    values = {tag: (kind, value) for tag, kind, value in create_values}
    if values.get(0x420057) != (5, struct.pack(">I", 2)):
        raise RuntimeError("ID Placeholder producer must Create a Symmetric Key")
    if values.get(0x420098) != (7, BATCH_PLACEHOLDER_NAME):
        raise RuntimeError("ID Placeholder Create name did not match the fixture journey")
    if values.get(0x420028) != (5, struct.pack(">I", 3)):
        raise RuntimeError("ID Placeholder Create must explicitly select AES")
    if values.get(0x42002A) != (2, struct.pack(">i", 256)):
        raise RuntimeError("ID Placeholder Create must explicitly select 256-bit AES")

    get_values = [
        (kind, value)
        for tag, kind, value in _flatten(_children(get_payloads[0]))
        if tag == 0x420094
    ]
    if get_values != [(5, struct.pack(">I", 1))]:
        raise RuntimeError("Get must use the KMIP ID Placeholder enumeration")


def _validate_operation(
    operation: int,
    fields: list[tuple[int, int, bytes]],
    header_fields: list[tuple[int, int, bytes]],
    expected_identifier: bytes,
) -> None:
    payloads = [value for tag, kind, value in fields if tag == 0x420079 and kind == 1]
    if operation in (1, 2, 10, 20, 0x1F, 0x20, 0x21, 0x22) and len(payloads) != 1:
        raise RuntimeError(
            "journey request must contain one Request Payload; "
            f"operation={operation} tags={[hex(tag) for tag, _kind, _value in fields]}"
        )
    if operation == 1:
        payload = _children(payloads[0])
        flattened_payload = _flatten(payload)
        values = {tag: (kind, value) for tag, kind, value in flattened_payload}
        names = [
            value for tag, kind, value in flattened_payload if tag == 0x420098 and kind == 7
        ]
        if names != [expected_identifier]:
            raise RuntimeError("Create did not use the expected journey key name")
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
        if expected_identifier == EXPERT_ATTRIBUTE_IDENTIFIER:
            object_groups = [
                (kind, value)
                for tag, kind, value in flattened_payload
                if tag == 0x420056
            ]
            if object_groups != [(7, b"fluent-expert-group")]:
                raise RuntimeError(
                    "Expert attribute journey must send the catalogued Object Group Text String"
                )
    elif operation in (10, 20):
        payload = _children(payloads[0])
        identifiers = [value for tag, kind, value in payload if tag == 0x420094 and kind == 7]
        if identifiers != [expected_identifier]:
            raise RuntimeError("Get or Destroy did not use the returned Unique Identifier")
    elif operation in (0x1F, 0x20):
        payload = _children(payloads[0])
        flattened_payload = _flatten(payload)

        def values_for(tag: int) -> list[tuple[int, bytes]]:
            return [(kind, value) for field_tag, kind, value in flattened_payload if field_tag == tag]

        if values_for(0x420094) != [(7, expected_identifier)]:
            raise RuntimeError("AES-GCM request did not use the Create response identifier")
        if values_for(0x4200C2) != [
            (8, AES_GCM_PLAINTEXT if operation == 0x1F else AES_GCM_CIPHERTEXT)
        ]:
            raise RuntimeError("AES-GCM request Data did not match the known fixture vector")
        if values_for(0x42003D) != [(8, AES_GCM_IV)]:
            raise RuntimeError("AES-GCM request omitted or changed the explicit IV")
        if values_for(0x4200FE) != [(8, AES_GCM_AAD)]:
            raise RuntimeError("AES-GCM request omitted or changed explicit AAD")
        if operation == 0x20 and values_for(0x4200FF) != [(8, AES_GCM_TAG)]:
            raise RuntimeError("Decrypt did not use the Encrypt response authentication tag")
        parameters = [
            value for tag, kind, value in payload if tag == 0x42002B and kind == 1
        ]
        if len(parameters) != 1:
            raise RuntimeError("AES-GCM request must carry one Cryptographic Parameters structure")
        parameter_values = {tag: (kind, value) for tag, kind, value in _flatten(_children(parameters[0]))}
        if parameter_values.get(0x420011) != (5, struct.pack(">I", 9)):
            raise RuntimeError("AES-GCM Block Cipher Mode was not explicitly GCM")
        if parameter_values.get(0x420028) != (5, struct.pack(">I", 3)):
            raise RuntimeError("AES-GCM Cryptographic Algorithm was not explicitly AES")
        if parameter_values.get(0x4200CD) != (2, struct.pack(">i", 96)):
            raise RuntimeError("AES-GCM IV Length was not explicitly 96 bits")
        if parameter_values.get(0x4200CE) != (2, struct.pack(">i", 128)):
            raise RuntimeError("AES-GCM Tag Length was not explicitly 128 bits")
    elif operation == 2:
        payload = _children(payloads[0])
        groups = {tag: (kind, value) for tag, kind, value in payload}
        expected_attributes = {
            0x420126: {
                0x420028: (5, struct.pack(">I", 6)),
                0x42002A: (2, struct.pack(">i", 256)),
                # KMIP 2.1 sections 4.14 and 11.45: explicitly choose P-256.
                0x420029: (1, _item(0x420075, 5, struct.pack(">I", 7))),
            },
            0x420127: {0x42002C: (2, struct.pack(">i", 1))},
            0x420128: {0x42002C: (2, struct.pack(">i", 2))},
        }
        if set(groups) != set(expected_attributes) or any(kind != 1 for kind, _value in groups.values()):
            raise RuntimeError("Create Key Pair must use separate common/private/public attribute groups")
        for group_tag, expected in expected_attributes.items():
            actual = {tag: (kind, value) for tag, kind, value in _children(groups[group_tag][1])}
            if actual != expected:
                raise RuntimeError("Create Key Pair algorithm, length, P-256 curve, or usage masks were not explicit")
    elif operation in (0x21, 0x22):
        payload = _children(payloads[0])
        flattened_payload = _flatten(payload)
        values = {tag: (kind, value) for tag, kind, value in flattened_payload}
        expected_key = PRIVATE_KEY_IDENTIFIER if operation == 0x21 else PUBLIC_KEY_IDENTIFIER
        if values.get(0x420094) != (7, expected_key):
            raise RuntimeError("Sign or Signature Verify did not use the matching key-pair identifier")
        if values.get(0x4200C2) != (8, SIGNATURE_PLAINTEXT):
            raise RuntimeError("Sign or Signature Verify did not use the known message bytes")
        if operation == 0x22 and values.get(0x4200C3) != (8, SIGNATURE_DATA):
            raise RuntimeError("Signature Verify did not use the returned signature bytes")
        parameters = [value for tag, kind, value in payload if tag == 0x42002B and kind == 1]
        if len(parameters) != 1:
            raise RuntimeError("Sign and Signature Verify must carry explicit Cryptographic Parameters")
        parameter_values = {tag: (kind, value) for tag, kind, value in _flatten(_children(parameters[0]))}
        if parameter_values.get(0x420038) != (5, struct.pack(">I", 6)):
            raise RuntimeError("Sign and Signature Verify must explicitly select SHA-256")
        if parameter_values.get(0x4200AE) != (5, struct.pack(">I", 14)):
            raise RuntimeError("Sign and Signature Verify must explicitly select ECDSA with SHA-256")
    if operation in (10, 20):
        indicators = [value for tag, kind, value in header_fields if tag == 0x420007 and kind == 5]
        if operation == 10 and indicators not in ([], [struct.pack(">I", 1)]):
            raise RuntimeError("Get contains an invalid Request Header Asynchronous Indicator")
        if operation == 20 and indicators not in ([], [struct.pack(">I", 1)]):
            raise RuntimeError("Destroy contains an invalid Request Header Asynchronous Indicator")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ready-file", type=Path, required=True)
    parser.add_argument("--pid-file", type=Path, required=True)
    parser.add_argument("--fixture-dir", type=Path, required=True)
    parser.add_argument("--connections", type=int, default=len(EXPECTED_CONSUMER_REQUESTS))
    args = parser.parse_args()
    if args.connections != len(EXPECTED_CONSUMER_REQUESTS):
        raise RuntimeError("the parity fixture requires the complete four-language journey sequence")

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
            for connection_index, expected_operations in enumerate(EXPECTED_CONSUMER_REQUESTS):
                raw, _address = listener.accept()
                raw.settimeout(10)
                with raw, context.wrap_socket(raw, server_side=True) as stream:
                    peer_cert = stream.getpeercert(binary_form=True)
                    if stream.version() != "TLSv1.3" or not peer_cert:
                        raise RuntimeError("fixture requires an authenticated TLS 1.3 client")
                    request = _read_frame(stream)
                    items, header_fields = _request_batch(request)
                    operations = tuple(operation for operation, _fields in items)
                    if operations != expected_operations:
                        raise RuntimeError(
                            "consumer journey operation order mismatch; "
                            f"index={connection_index} expected={expected_operations!r} "
                            f"received={operations!r}"
                        )
                    if operations == (1, 10):
                        _validate_placeholder_batch(items, header_fields)
                        response_items = tuple(
                            (
                                operation,
                                next(
                                    value
                                    for tag, kind, value in fields
                                    if tag == 0x420093 and kind == 8
                                ),
                            )
                            for operation, fields in items
                        )
                        stream.sendall(
                            _response_batch(response_items, BATCH_PLACEHOLDER_IDENTIFIER)
                        )
                        continue

                    operation, fields = items[0]
                    journey_offset = connection_index % len(_EXPECTED_PER_CONSUMER_REQUESTS)
                    expected_identifier = (
                        AES_GCM_IDENTIFIER if 3 <= journey_offset < 7
                        else EXPERT_ATTRIBUTE_IDENTIFIER if journey_offset >= 10
                        else LIFECYCLE_IDENTIFIER if journey_offset < 3
                        else b""
                    )
                    _validate_operation(operation, fields, header_fields, expected_identifier)
                    stream.sendall(_response(operation, expected_identifier))
        finally:
            args.pid_file.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
