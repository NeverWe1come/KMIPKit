#!/usr/bin/env python3
"""Exercise the installed C client against a local KMIP TTLV/TLS peer."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import socket
import ssl
import struct
import subprocess
import sys
import tempfile
import threading
import time

ASYNC_CANCEL_RESULT_CASES = (
    "test-async-cancel-result-canceled",
    "test-async-cancel-result-unable-to-cancel",
    "test-async-cancel-result-completed",
    "test-async-cancel-result-failed",
    "test-async-cancel-result-unavailable",
)
ASYNC_CANCEL_POLL_RACE_CASE = "test-async-cancel-vs-poll-race"


def tlv(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes([item_type]) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def integer(tag: int, value: int, item_type: int = 2) -> bytes:
    return tlv(tag, item_type, struct.pack(">i", value))


def enumeration(tag: int, value: int) -> bytes:
    return tlv(tag, 5, struct.pack(">I", value))


def response(operations: tuple[int, ...], protocol_minor: int,
             selected_attribute_tag: int = 0,
             created_object_type: int = 2,
             selected_attribute_wire: bytes | None = None,
             unique_batch_item_ids: tuple[bytes, ...] | None = None,
             result_statuses: tuple[int, ...] | None = None,
             result_reasons: tuple[int | None, ...] | None = None,
             rng_length: int = 16,
             cancellation_result: int = 1) -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(
        0x42007A,
        1,
        version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
        + integer(0x42000D, len(operations)),
    )
    batch_items = []
    for item_index, operation in enumerate(operations):
        payload_value = b""
        result_status = 0
        result_reason = None
        if operations == (1, 20, 19, 20) and item_index >= 2:
            result_status = 1
            result_reason = 55
        if operations == (1, 18, 20, 19, 20) and item_index == 2:
            result_status = 1
            result_reason = 67
        if result_statuses is not None:
            result_status = result_statuses[item_index]
            result_reason = result_reasons[item_index] if result_reasons is not None else None
        if operation == 30:
            payload_value = tlv(
            0x420069, 1,
            integer(0x42006A, 2) + integer(0x42006B, protocol_minor),
            )
        elif operation == 24:
            query_operations = (24, 30, 44) if protocol_minor == 0 else (24, 30, 44, 59)
            payload_value = b"".join(integer(0x42005C, value, 5) for value in query_operations)
            payload_value += tlv(0x42015F, 1, b"")
        elif operation == 8:
            if operations == (8, 17, 17):
                payload_value = integer(0x4200D5, 1)
                payload_value += tlv(0x420094, 7, b"fixture-key-1")
            else:
                payload_value = integer(0x4200D5, 3)
                payload_value += tlv(0x420094, 7, b"fixture-locate-1")
                payload_value += enumeration(0x420094, 0xF1234567)
                payload_value += integer(0x420094, -12345)
        elif operation == 10:
            key_material = tlv(0x420043, 8, b"KMIPKIT-SECRET!!")
            key_value = tlv(0x420045, 1, key_material)
            key_block = (
                enumeration(0x420042, 1) + key_value + enumeration(0x420028, 3)
            )
            symmetric_key = tlv(0x42008F, 1, tlv(0x420040, 1, key_block))
            payload_value = (
                enumeration(0x420057, 2)
                + tlv(0x420094, 7, b"fixture-key-1")
                + symmetric_key
            )
        elif operation == 39:
            payload_value = tlv(0x4200C2, 8, hashlib.sha256(b"abc").digest())
        elif operation == 37:
            payload_value = tlv(0x4200C2, 8, bytes(range(rng_length)))
        elif operation == 38:
            payload_value = integer(0x4200C4, rng_length)
        elif operation == 45:
            payload_value = tlv(
                0x420149,
                1,
                enumeration(0x42014A, 1) + tlv(0x42014B, 8, b"controlled-login-ticket"),
            )
        elif operation == 47:
            payload_value = tlv(
                0x420149,
                1,
                enumeration(0x42014A, 1) + tlv(0x42014B, 8, b"controlled-delegated-login-ticket"),
            )
        elif operation == 25:
            payload_value = tlv(
                0x420006, 8, b"fixture-cancel-correlation"
            ) + enumeration(0x420012, cancellation_result)
        elif operation == 1:
            payload_value = enumeration(0x420057, created_object_type) + \
                tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 3:
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 31:
            payload_value = tlv(0x420094, 7, b"fixture-key-1") + tlv(
                0x4200C2, 8, bytes.fromhex("69c4e0d86a7b0430d8cdb78070b4c55a"))
        elif operation == 32:
            payload_value = tlv(0x420094, 7, b"fixture-key-1") + tlv(
                0x4200C2, 8, bytes.fromhex("00112233445566778899aabbccddeeff"))
        elif operation in (13, 14, 15, 49):
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 11:
            if selected_attribute_wire is not None:
                attributes = selected_attribute_wire
            elif selected_attribute_tag == 0x420028:
                attributes = enumeration(0x420028, 3)
            elif selected_attribute_tag == 0x42002A:
                attributes = integer(0x42002A, 128)
            else:
                attributes = enumeration(0x420028, 3) + integer(0x42002A, 128)
            payload_value = tlv(0x420094, 7, b"fixture-key-1") + tlv(0x420125, 1, attributes)
        elif operation == 12:
            payload_value = (
                tlv(0x420094, 7, b"fixture-key-1")
                + enumeration(0x42013B, 0x420028)
                + enumeration(0x42013B, 0x42002A)
            )
        elif operation in (9, 18):
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 20:
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 17:
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 19 and result_status == 0:
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 56:
            payload_value = tlv(0x420168, 1, b"")
        unique_batch_item_id = (unique_batch_item_ids[item_index]
                                if unique_batch_item_ids is not None else {
                                    3: b"interop-register",
                                    13: b"interop-attributes",
                                    14: b"interop-attributes",
                                    15: b"interop-delete-attribute",
                                    49: b"interop-attributes",
                                    54: b"interop-set-defaults",
                                    55: b"interop-set-constraints",
                                    1: b"interop-create",
                                    11: b"interop-attributes",
                                    12: b"interop-attributes",
                                    20: b"interop-destroy",
                                    10: b"interop-get",
                                    9: b"interop-check",
                                    18: b"interop-activate",
                                    19: b"interop-revoke",
                                    56: b"interop-get-constraints",
                                    17: b"interop-get-usage-allocation",
                                }.get(operation))
        if operations == (1, 20, 19, 20):
            unique_batch_item_id = (
                b"interop-create", b"interop-destroy-target",
                b"interop-revoke-cleanup", b"interop-destroy-cleanup"
            )[item_index]
        if operations == (1, 18, 20, 19, 20):
            unique_batch_item_id = (
                b"interop-create", b"interop-activate-before-destroy",
                b"interop-destroy-active-target", b"interop-revoke-cleanup",
                b"interop-destroy-cleanup"
            )[item_index]
        item_fields = integer(0x42005C, operation, 5)
        if unique_batch_item_id is not None:
            item_fields += tlv(0x420093, 8, unique_batch_item_id)
        item_fields += integer(0x42007F, result_status, 5)
        if result_reason is not None:
            item_fields += enumeration(0x42007E, result_reason)
        if result_status == 0:
            item_fields += tlv(0x42007C, 1, payload_value)
        batch_items.append(tlv(
            0x42000F,
            1,
            item_fields,
        ))
    return tlv(0x42007B, 1, header + b"".join(batch_items))


def pending_create_response(protocol_minor: int, correlation: bytes) -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1,
                 version + tlv(0x420092, 9, struct.pack(">q", int(time.time()))) +
                 integer(0x42000D, 1))
    item = tlv(0x42000F, 1,
               enumeration(0x42005C, 1) + enumeration(0x42007F, 2) +
               tlv(0x420006, 8, correlation) + tlv(0x42007C, 1, b""))
    return tlv(0x42007B, 1, header + item)


def pending_poll_response(protocol_minor: int, correlation: bytes) -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
                 + integer(0x42000D, 1))
    item = tlv(0x42000F, 1,
               integer(0x42005C, 26, 5) + integer(0x42007F, 2, 5)
               + tlv(0x420006, 8, correlation))
    return tlv(0x42007B, 1, header + item)


def include_delegated_login(operations: tuple[int, ...],
                            cases: tuple[str, ...]) -> tuple[tuple[int, ...], tuple[str, ...]]:
    """Add the catalog's first-run Delegated Login to the controlled peer plan."""
    return (47, *operations), ("test-op-delegated-login", *cases)


def read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    chunks = bytearray()
    while len(chunks) < length:
        part = stream.recv(length - len(chunks))
        if not part:
            raise EOFError("peer closed before the complete TTLV message arrived")
        chunks.extend(part)
    return bytes(chunks)


def read_message(stream: ssl.SSLSocket) -> bytes | None:
    try:
        header = read_exact(stream, 8)
    except EOFError:
        return None
    body_length = int.from_bytes(header[4:8], "big")
    body = read_exact(stream, ((body_length + 7) // 8) * 8)
    return header + body


def certificate_files(openssl: str, directory: Path) -> tuple[Path, Path, Path, Path, Path]:
    ca_key = directory / "ca-key.pem"
    ca_cert = directory / "ca-cert.pem"
    subprocess.run(
        [openssl, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-sha256", "-days", "2",
         "-subj", "/CN=KMIPKit interop test CA", "-keyout", str(ca_key), "-out", str(ca_cert)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    server_ext = directory / "server-ext.cnf"
    client_ext = directory / "client-ext.cnf"
    server_ext.write_text(
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n"
        "extendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n",
        encoding="ascii",
    )
    client_ext.write_text(
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n"
        "extendedKeyUsage=clientAuth\n",
        encoding="ascii",
    )
    for name, ext in (("server", server_ext), ("client", client_ext)):
        key = directory / f"{name}-key.pem"
        csr = directory / f"{name}.csr"
        cert = directory / f"{name}-cert.pem"
        subprocess.run(
            [openssl, "req", "-new", "-newkey", "rsa:2048", "-nodes", "-sha256",
             "-subj", f"/CN=KMIPKit {name}", "-keyout", str(key), "-out", str(csr)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        subprocess.run(
            [openssl, "x509", "-req", "-in", str(csr), "-CA", str(ca_cert), "-CAkey", str(ca_key),
             "-CAcreateserial", "-out", str(cert), "-days", "2", "-sha256", "-extfile", str(ext)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    return (ca_cert, directory / "server-cert.pem", directory / "server-key.pem",
            directory / "client-cert.pem", directory / "client-key.pem")


def main() -> int:
    binary, openssl, repository = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    protocol_version = sys.argv[4] if len(sys.argv) > 4 else "2.1"
    if protocol_version not in ("2.0", "2.1"):
        raise AssertionError(f"unsupported fixture protocol version {protocol_version!r}")
    protocol_minor = int(protocol_version.split(".")[1])
    registered_key_batch = (3, 18, 31, 32, 19, 20)
    stop_on_first_failure_batch = (10, 10)
    registered_key_batch_item_ids = (
        b"crypto-register", b"crypto-activate", b"crypto-encrypt",
        b"crypto-decrypt", b"crypto-revoke", b"crypto-destroy",
    )
    expected_operations = (
        (1, 20, 1, 11, 20, 1, 12, 20, 1, 10, 20, 1, 9, 19, 20, 1, 19, 20,
         1, 20, 19, 20,
         1, 18, 19, 20, 8, 24, 44, 1, 11, 20, 1, 11, 20, 1, 20, 1, 20, 1, 20,
         1, 18, 20, 19, 20)
        if protocol_version == "2.0"
        else (1, 20, 1, 11, 20, 1, 12, 20, 1, 10, 20, 1, 9, 19, 20, 1, 19, 20,
              1, 20, 19, 20, 1, 18, 19, 20, 1, 56, 19, 20, 8, 24, 30, 44, 45, 59, 1, 11, 20,
              1, 11, 20, 1, 20, 1, 20, 1, 20, 1, 18, 20, 19, 20)
    )
    expected_cases = (
        ("test-op-locate", "test-op-query", "test-op-log", "test-op-interop")
        if protocol_version == "2.0"
        else ("test-op-discover-versions", "test-op-ping", "test-op-query", "test-op-log",
              "test-op-locate", "test-op-login", "test-op-logout", "test-op-interop", "test-op-get", "test-op-get-attributes",
              "test-op-get-attribute-list", "test-op-check", "test-op-create", "test-op-activate",
              "test-op-revoke", "test-op-destroy", "test-op-get-constraints",
              "test-attr-cryptographic-algorithm", "test-attr-cryptographic-length",
              "test-obj-symmetric-key", "test-obj-private-key", "test-obj-public-key")
    )
    expected_operations, expected_cases = include_delegated_login(
        expected_operations, expected_cases)
    expected_operations += (37, 38, 39)
    expected_operations += (39, 37, 38)
    expected_operations += (1, 13, 19, 20)
    expected_operations += (1, 49, 19, 20)
    expected_operations += (1, 17, 19, 20)
    if protocol_version == "2.1":
        expected_operations += (45, 46)
        expected_operations += (55, 54)
    expected_operations += (1, 13, 15, 19, 20)
    expected_operations += (1, 13, 14, 19, 20)
    expected_operations += (1, 18, 12, 19, 20)
    expected_operations += (1, 18, 10, 19, 20)
    expected_operations += (52, 52, 52)
    expected_operations += (10, 12, 10, 12, 10, 12, 10, 12)
    expected_operations += registered_key_batch
    expected_operations += stop_on_first_failure_batch
    expected_operations += (1, 25)
    expected_cases += ("test-op-cancel",)
    expected_operations += (1, 25) * len(ASYNC_CANCEL_RESULT_CASES)
    expected_cases += ASYNC_CANCEL_RESULT_CASES
    if protocol_version == "2.1":
        expected_operations += (1, 26, 25)
        expected_cases += (ASYNC_CANCEL_POLL_RACE_CASE,)
    expected_cases += ("test-op-hash", "test-op-rng-retrieve", "test-op-rng-seed")
    expected_cases += (
        "test-crypto-hash-sha_256-0",
        "test-crypto-rng-retrieve-rng-32",
        "test-crypto-rng-seed-rng-32",
    )
    expected_cases += ("test-batch-register-encrypt-decrypt-destroy",)
    expected_cases += ("test-batch-stop-on-first-failure",)
    expected_cases += ("test-neg-destroy-active-without-revoke",)
    expected_cases += ("test-op-add-attribute",)
    expected_cases += ("test-op-set-attribute",)
    expected_cases += ("test-op-get-usage-allocation",)
    if protocol_version == "2.1":
        expected_cases += ("test-op-set-constraints", "test-op-set-defaults")
    expected_cases += ("test-op-delete-attribute",)
    expected_cases += ("test-op-modify-attribute",)
    expected_cases += ("test-batch-create-activate-destroy",)
    expected_cases += ("test-batch-placeholder-chain-resolves",)
    expected_cases += (
        "test-batch-count-matches-items",
        "test-batch-unique-item-id-response-correlation",
        "test-batch-order-false-correlates-response-items",
        "test-batch-continue-after-failure",
    )
    if protocol_version == "2.0":
        expected_cases += (
            "test-op-get", "test-op-get-attributes", "test-op-get-attribute-list", "test-op-check",
            "test-op-create", "test-op-activate", "test-op-revoke", "test-op-destroy",
            "test-attr-cryptographic-algorithm", "test-attr-cryptographic-length",
            "test-obj-symmetric-key", "test-obj-private-key", "test-obj-public-key"
        )
        expected_cases += ("test-v20-reject-get-constraints", "test-v20-reject-ping")
    shared = repository / "interop/shared"
    catalog = shared / "catalog/kmipkit-1.0-test-catalog.json"
    catalog_data = json.loads(catalog.read_text(encoding="utf-8"))
    catalog_cases = catalog_data["test_cases"]
    cancellation_targets = {
        target["name"]: int(target["wire_value"], 16)
        for target in catalog_data["normative_targets"]
        if target["kind"] == "enumeration_value"
        and "KMIPKIT-ELEM-ENUMERATION-CANCELLATION-RESULT" in target.get("parent_element_ids", [])
        and re.fullmatch(r"[0-9A-Fa-f]{8}", target.get("wire_value", ""))
        and protocol_version in target["protocol_versions"]
    }
    cancel_result_values = [cancellation_targets["Canceled"]] + [
        cancellation_targets[case["requires"]["protocol_scenario"]["parameters"]["cancel_result"]]
        for case in catalog_cases if case["case_id"] in ASYNC_CANCEL_RESULT_CASES
    ]
    if protocol_version == "2.1":
        cancel_result_values.append(cancellation_targets["Canceled"])
    version_cases = [
        case for case in catalog_cases
        if protocol_version in case["execution"]["protocol_versions"]
    ]
    local_value_cases = sum(
        case["execution"]["mode"] == "local_model"
        and ("enum_type" in case["requires"] or "bitmask_type" in case["requires"])
        for case in version_cases
    )
    with tempfile.TemporaryDirectory(prefix="kmipkit-c-live-") as raw_directory:
        directory = Path(raw_directory)
        ca_cert, server_cert, server_key, client_cert, client_key = certificate_files(openssl, directory)
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(4)
        server.settimeout(10)
        port = server.getsockname()[1]
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_3
        context.load_cert_chain(str(server_cert), str(server_key))
        context.load_verify_locations(str(ca_cert))
        context.verify_mode = ssl.CERT_REQUIRED
        server_errors: list[BaseException] = []
        handled = 0
        handled_operations: list[int] = []
        handled_batches: list[tuple[int, ...]] = []

        def serve() -> None:
            nonlocal handled
            cancel_result_index = 0
            try:
                while handled < len(expected_operations):
                    accepted, _ = server.accept()
                    with context.wrap_socket(accepted, server_side=True) as stream:
                        stream.settimeout(10)
                        while handled < len(expected_operations):
                            request = read_message(stream)
                            if request is None:
                                break
                            operation_marker = bytes.fromhex("42005c0500000004")
                            offsets = []
                            offset = request.find(operation_marker)
                            while offset >= 0:
                                offsets.append(offset)
                                offset = request.find(operation_marker, offset + len(operation_marker) + 4)
                            if not offsets:
                                raise AssertionError("request has no KMIP Operation field")
                            operations = tuple(int.from_bytes(
                                request[offset + len(operation_marker):offset + len(operation_marker) + 4], "big"
                            ) for offset in offsets)
                            if operations == (1, 9, 20):
                                raise AssertionError("Check fixture must revoke the created key before destroying it")
                            if any(operation not in expected_operations for operation in operations):
                                raise AssertionError(f"unexpected operation batch {operations!r} for {protocol_version}")
                            if operations == (25,) and request.count(
                                    tlv(0x420006, 8, b"fixture-cancel-correlation")) != 1:
                                raise AssertionError("Cancel request omitted its exact fixture correlation value")
                            if operations == (52, 52, 52):
                                interop_identifier = b"KMIPKit controlled interoperability fixture"
                                function_offsets = [request.find(enumeration(0x420160, function))
                                                    for function in (1, 2, 3)]
                                if (any(offset < 0 for offset in function_offsets) or
                                        function_offsets != sorted(function_offsets) or
                                        request.count(tlv(0x420161, 7, interop_identifier)) != 3 or
                                        any(request.count(tlv(0x420093, 8, item_id)) != 1
                                            for item_id in (b"interop-begin", b"interop-end",
                                                            b"interop-reset"))):
                                    raise AssertionError("Interop function order or shared identifier mismatch")
                            two_read_batch = operations == (10, 12)
                            placeholder_chain = operations == (1, 18, 10)
                            placeholder_cleanup = operations == (19, 20) and \
                                tlv(0x420093, 8, b"placeholder-revoke-cleanup") in request
                            response_operations = operations
                            response_item_ids = None
                            if operations == (52, 52, 52):
                                response_item_ids = (b"interop-begin", b"interop-end", b"interop-reset")
                            if operations == registered_key_batch:
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in registered_key_batch_item_ids):
                                    raise AssertionError(
                                        "registered-key batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = registered_key_batch_item_ids
                            if operations == stop_on_first_failure_batch:
                                if (request.count(integer(0x42000D, 2)) != 1 or
                                        request.count(tlv(0x420094, 7, b"missing-key")) != 1 or
                                        request.count(tlv(0x420094, 7, b"fixture-key-1")) != 1 or
                                        request.count(tlv(0x420093, 8, b"item-a")) != 1 or
                                        request.count(tlv(0x420093, 8, b"item-b")) != 1 or
                                        enumeration(0x42000E, 2) not in request or
                                        enumeration(0x42000E, 1) in request):
                                    raise AssertionError("Stop-on-first-failure batch fields mismatch")
                                response_operations = (10,)
                                response_item_ids = (b"item-a",)
                                response_statuses = (1,)
                                response_reasons = (55,)
                            if two_read_batch:
                                continue_after_failure = tlv(0x420094, 7, b"missing-key") in request
                                expected_fixture_identifiers = 1 if continue_after_failure else 2
                                if (request.count(tlv(0x420094, 7, b"fixture-key-1")) !=
                                        expected_fixture_identifiers or
                                        (continue_after_failure and
                                         request.count(tlv(0x420094, 7, b"missing-key")) != 1) or
                                        request.count(tlv(0x420093, 8, b"item-a")) != 1 or
                                        request.count(tlv(0x420093, 8, b"item-b")) != 1):
                                    raise AssertionError("two-read batch omitted fixture or unique item IDs")
                                response_item_ids = (b"item-a", b"item-b")
                                if continue_after_failure:
                                    if enumeration(0x42000E, 1) not in request:
                                        raise AssertionError("Continue-after-failure batch omitted Batch Error Continuation Option")
                                    response_statuses = (1, 0)
                                    response_reasons = (55, None)
                                else:
                                    response_statuses = None
                                    response_reasons = None
                                if tlv(0x420010, 6, b"\x00") in request:
                                    response_operations = tuple(reversed(operations))
                                    response_item_ids = (b"item-b", b"item-a")
                                    response_statuses = None
                                    response_reasons = None
                            elif operations != stop_on_first_failure_batch:
                                response_statuses = None
                                response_reasons = None
                            if placeholder_chain:
                                expected_item_ids = (
                                    b"placeholder-create", b"placeholder-activate", b"placeholder-get"
                                )
                                if (request.count(enumeration(0x420094, 1)) != 2 or
                                        any(tlv(0x420093, 8, item_id) not in request
                                            for item_id in expected_item_ids) or
                                        enumeration(0x42000E, 1) not in request):
                                    raise AssertionError("placeholder chain did not send two IDPlaceholder references")
                                response_item_ids = expected_item_ids
                            elif placeholder_cleanup:
                                expected_cleanup_ids = (
                                    b"placeholder-revoke-cleanup", b"placeholder-destroy-cleanup"
                                )
                                if (request.count(tlv(0x420094, 7, b"fixture-key-1")) != 2 or
                                        any(tlv(0x420093, 8, item_id) not in request
                                            for item_id in expected_cleanup_ids) or
                                        operations != (19, 20)):
                                    raise AssertionError("placeholder chain teardown did not use the returned identifier")
                                response_item_ids = expected_cleanup_ids
                            create_lifecycle = operations == (1, 20)
                            check_lifecycle = operations == (1, 9, 19, 20)
                            activate_lifecycle = operations == (1, 18, 19, 20)
                            revoke_lifecycle = operations == (1, 19, 20)
                            destroy_lifecycle = operations == (1, 20, 19, 20)
                            illegal_destroy_lifecycle = operations == (1, 18, 20, 19, 20)
                            get_constraints_lifecycle = operations == (1, 56, 19, 20)
                            get_usage_allocation_lifecycle = operations == (1, 17, 19, 20)
                            lifecycle_target = {operations[1]: {
                                9: b"interop-check", 10: b"interop-get",
                                11: b"interop-attributes", 12: b"interop-attributes"
                            }.get(operations[1])} if len(operations) == 3 and operations[0] == 1 and operations[2] == 20 else {}
                            if check_lifecycle:
                                lifecycle_target = {9: b"interop-check"}
                            if activate_lifecycle:
                                lifecycle_target = {18: b"interop-activate"}
                            if revoke_lifecycle:
                                lifecycle_target = {19: b"interop-revoke"}
                            if destroy_lifecycle:
                                lifecycle_target = {20: b"interop-destroy-target"}
                            if illegal_destroy_lifecycle:
                                lifecycle_target = {20: b"interop-destroy-active-target"}
                            if get_constraints_lifecycle:
                                lifecycle_target = {56: b"interop-get-constraints"}
                            if get_usage_allocation_lifecycle:
                                lifecycle_target = {17: b"interop-get-usage-allocation"}
                            if create_lifecycle:
                                lifecycle_target = {1: b"interop-create"}
                            lifecycle_ids = (b"interop-create", *lifecycle_target.values(),
                                             *((b"interop-revoke",) if (check_lifecycle or activate_lifecycle or
                                                 get_constraints_lifecycle or
                                                 get_usage_allocation_lifecycle) else ()),
                                             *((b"interop-revoke-cleanup", b"interop-destroy-cleanup")
                                               if destroy_lifecycle else ()),
                                             *((b"interop-destroy",) if not destroy_lifecycle else ()))
                            if illegal_destroy_lifecycle:
                                lifecycle_ids = (
                                    b"interop-create", b"interop-activate-before-destroy",
                                    b"interop-destroy-active-target", b"interop-revoke-cleanup",
                                    b"interop-destroy-cleanup"
                                )
                            if lifecycle_target and any(
                                    tlv(0x420093, 8, item_id) not in request for item_id in lifecycle_ids):
                                raise AssertionError("lifecycle batch omitted a unique batch item ID")
                            if lifecycle_target and enumeration(0x42000E, 1) not in request:
                                raise AssertionError("lifecycle batch must continue to Destroy after a failed item")
                            private_key_create = enumeration(0x420057, 4) in request
                            public_key_create = enumeration(0x420057, 3) in request
                            if lifecycle_target:
                                expected_algorithm = 4 if private_key_create or public_key_create else 3
                                expected_length = 2048 if private_key_create or public_key_create else 128
                                if enumeration(0x420028, expected_algorithm) not in request or \
                                        integer(0x42002A, expected_length) not in request:
                                    raise AssertionError(
                                        "lifecycle Create omitted the expected key algorithm or length"
                                    )
                            selected_attribute_tag = next((tag for tag in (0x420028, 0x42002A)
                                                           if enumeration(0x42013B, tag) in request), 0)
                            prior_attribute_requests = sum(1 for prior in handled_batches
                                                           if prior == (1, 11, 20))
                            if operations == (1, 11, 20) and prior_attribute_requests > 0:
                                expected_tag = 0x420028 if prior_attribute_requests == 1 else 0x42002A
                                if selected_attribute_tag != expected_tag:
                                    raise AssertionError(
                                        "attribute-specific Get Attributes used the wrong Attribute Reference"
                                    )
                            if create_lifecycle and request.count(enumeration(0x420094, 1)) < 1:
                                raise AssertionError("Create cleanup must Destroy through ID Placeholder")
                            if operations in ((1, 9, 19, 20), (1, 18, 19, 20), (1, 19, 20),
                                              (1, 20, 19, 20), (1, 56, 19, 20), (1, 10, 20)) and request.count(
                                    enumeration(0x420094, 1)) < (
                                        3 if (check_lifecycle or activate_lifecycle or destroy_lifecycle) else 2):
                                raise AssertionError("target operation and cleanup must consume the created ID Placeholder")
                            if illegal_destroy_lifecycle and request.count(enumeration(0x420094, 1)) < 4:
                                raise AssertionError(
                                    "illegal Destroy lifecycle must target and clean the same created object"
                                )
                            if 8 in operations:
                                if bytes.fromhex("4201250100000000") not in request:
                                    raise AssertionError("Locate request omitted its required Attributes structure")
                                if integer(0x42004F, 3) not in request:
                                    raise AssertionError("Locate request omitted its bounded Maximum Items value")
                            if integer(0x42006A, 2) not in request or \
                                    integer(0x42006B, protocol_minor) not in request:
                                raise AssertionError(
                                    "request header did not preserve the configured KMIP version"
                                )
                            # The dedicated asynchronous C executor has no authentication input;
                            # every ordinary batch in this peer still proves configured credentials.
                            if operations not in ((25,), (26,)) and (
                                    tlv(0x420099, 7, b"interop-user") not in request or
                                    tlv(0x4200A1, 7, b"interop-password") not in request):
                                raise AssertionError("request is missing configured KMIP credentials")
                            if operations == (1,) and enumeration(0x420007, 1) in request:
                                cancel_correlation = b"fixture-cancel-correlation"
                                stream.sendall(pending_create_response(protocol_minor, cancel_correlation))
                            elif operations == (26,):
                                cancel_correlation = b"fixture-cancel-correlation"
                                if request.count(tlv(0x420006, 8, cancel_correlation)) != 1:
                                    raise AssertionError("Poll request omitted its exact pending correlation value")
                                stream.sendall(pending_poll_response(protocol_minor, cancel_correlation))
                            else:
                                cancellation_result = 1
                                if operations == (25,):
                                    if cancel_result_index >= len(cancel_result_values):
                                        raise AssertionError("peer received an unplanned Cancel request")
                                    cancellation_result = cancel_result_values[cancel_result_index]
                                    cancel_result_index += 1
                                if operations == (25,) and request.count(
                                        tlv(0x420006, 8, b"fixture-cancel-correlation")) != 1:
                                    raise AssertionError("Cancel request omitted its exact pending correlation value")
                                stream.sendall(response(
                                    response_operations, protocol_minor, selected_attribute_tag,
                                    4 if private_key_create else 3 if public_key_create else 2,
                                    unique_batch_item_ids=response_item_ids,
                                    result_statuses=response_statuses,
                                    result_reasons=response_reasons,
                                    rng_length=32 if integer(0x4200C4, 32) in request or
                                    tlv(0x4200C2, 8, bytes(range(32))) in request else 16,
                                    cancellation_result=cancellation_result))
                            handled += len(operations)
                            handled_operations.extend(operations)
                            handled_batches.append(operations)
            except BaseException as error:  # surfaced in the main test thread below
                server_errors.append(error)
            finally:
                server.close()

        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        connection = {
            "server_id": "controlled-c-peer",
            "server_vendor": "",
            "server_product": "KMIPKit controlled TLS test peer",
            "server_version": "fixture-1",
            "server_configuration_fingerprint": hashlib.sha256(b"controlled-c-peer-v1").hexdigest(),
            "protocol_version": protocol_version,
            "transport": "raw_tls",
            "encoding": "TTLV",
            "api_level": "typed-protocol",
            "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": port}},
            "tls": {
                "ca_cert_path": str(ca_cert),
                "client_cert_path": str(client_cert),
                "client_key_path": str(client_key),
                "server_name_override": "localhost",
            },
            "authentication": {
                "username_password": {
                    "username_env": "KMIPKIT_INTEROP_TEST_USERNAME",
                    "password_env": "KMIPKIT_INTEROP_TEST_PASSWORD",
                }
            },
            "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
            "execution_policy": {"probe_unsupported": True, "allow_mutations": True},
        }
        connection_path = directory / "connection.json"
        connection_path.write_text(json.dumps(connection), encoding="utf-8")
        result_path = directory / "validation-results.json"
        markdown_path = directory / "validation-results.md"
        html_path = directory / "validation-results.html"
        run = subprocess.run(
            [str(binary), "run", "--connection", str(connection_path), "--catalog", str(catalog),
             "--allow-mutations",
             "--output", str(result_path), "--output-markdown", str(markdown_path),
             "--output-html", str(html_path)],
            cwd=repository,
            capture_output=True,
            text=True,
            env={
                **os.environ,
                "KMIPKIT_INTEROP_TEST_USERNAME": "interop-user",
                "KMIPKIT_INTEROP_TEST_PASSWORD": "interop-password",
            },
            timeout=30,
            check=False,
        )
        thread.join(timeout=10)
        if server_errors:
            partial_result = json.loads(result_path.read_text(encoding="utf-8")) if result_path.is_file() else {}
            failing_cases = [
                {"case_id": case.get("case_id"), "diagnostic": case.get("diagnostic")}
                for case in partial_result.get("cases", []) if case.get("status") == "failed"
            ]
            raise AssertionError(
                f"controlled TLS peer failed; validator exit={run.returncode}; "
                f"batches={handled_batches!r}; cases={failing_cases!r}; "
                f"stdout={run.stdout[:1200]!r}; stderr={run.stderr[-1200:]!r}"
            ) from server_errors[0]
        if (handled != len(expected_operations) or sorted(handled_operations) != sorted(expected_operations)
                or (1, 11, 20) not in handled_batches
                or (1, 12, 20) not in handled_batches
                or (1, 10, 20) not in handled_batches
                or registered_key_batch not in handled_batches):
            raise AssertionError(
                f"controlled peer observed batches {handled_batches!r}, "
                f"expected {expected_operations!r}"
            )
        result = json.loads(result_path.read_text(encoding="utf-8"))
        try:
            import jsonschema
        except ImportError:
            jsonschema = None
        if jsonschema is not None:
            schema = json.loads((shared / "schemas/validation-results.schema.json").read_text(encoding="utf-8"))
            jsonschema.validate(result, schema)
        if result["schema_version"] != "2.0" or result["protocol_version"] != protocol_version:
            raise AssertionError("C result metadata does not match the shared evidence schema")
        if result["server"]["id"] != "controlled-c-peer":
            raise AssertionError("C result identifies the wrong test server")
        statuses = {case["case_id"]: case["status"] for case in result["cases"]}
        if any(statuses.get(case_id) != "passed" for case_id in expected_cases):
            details = {case["case_id"]: case.get("diagnostic", "") for case in result["cases"]
                       if case["case_id"] in expected_cases and case["status"] != "passed"}
            raise AssertionError(f"controlled operation response checks failed: {details!r}")
        if protocol_version == "2.0":
            constraints_boundary = next(
                case for case in result["cases"]
                if case["case_id"] == "test-v20-reject-get-constraints"
            )
            if constraints_boundary["status"] != "passed" or \
                    constraints_boundary["server_compatibility"] != "unknown":
                raise AssertionError(
                    "KMIP 2.1-only Get Constraints must be rejected before transport under KMIP 2.0: "
                    f"{constraints_boundary!r}"
                )
            ping_boundary = next(
                case for case in result["cases"]
                if case["case_id"] == "test-v20-reject-ping"
            )
            if ping_boundary["status"] != "passed" or \
                    ping_boundary["server_compatibility"] != "unknown":
                raise AssertionError(
                    "KMIP 2.1-only Ping must be rejected before transport under KMIP 2.0: "
                    f"{ping_boundary!r}"
                )
            discovery = next(case for case in result["cases"]
                             if case["case_id"] == "test-op-discover-versions")
            if discovery["status"] != "not_run" or "cannot construct a KMIP 2.0 offer" not in discovery.get("diagnostic", ""):
                raise AssertionError(
                    "KMIP 2.0 Discover Versions must report the documented C API offer limitation"
                )
            login = next(case for case in result["cases"] if case["case_id"] == "test-op-login")
            if login["status"] != "not_run" or "Login builder only supports KMIP 2.1" not in login.get("diagnostic", ""):
                raise AssertionError("KMIP 2.0 Login must report the documented public C API limitation")
            logout = next(case for case in result["cases"] if case["case_id"] == "test-op-logout")
            if logout["status"] != "not_run" or "cannot obtain a server-issued Ticket" not in logout.get("diagnostic", ""):
                raise AssertionError("KMIP 2.0 Logout must report the missing Login Ticket prerequisite")
        if run.returncode != 0:
            failed_cases = [
                (case["case_id"], case["status"])
                for case in result["cases"] if case["status"] == "failed"
            ]
            raise AssertionError(
                f"C validator exited {run.returncode}; stderr={run.stderr[-400:]!r}; "
                f"failed_cases={failed_cases!r}; summary={run.stdout[:1800]!r}"
            )
        correctness = {case["case_id"]: case["library_correctness"] for case in result["cases"]}
        if any(correctness.get(case_id) != "passed" for case_id in expected_cases):
            raise AssertionError("successful exchanges must report the client library as validated")
        if result["summary"]["policy_restricted"] != 0 or result["summary"]["not_run"] == 0:
            raise AssertionError("opted-in controlled run must preserve uncovered-case classification")
        expected_summary = {
            "total": len(version_cases),
            "passed": len(expected_cases) + local_value_cases,
            "failed": 0,
            "unsupported": 0,
            "policy_restricted": 0,
            "not_run": len(version_cases) - len(expected_cases) - local_value_cases,
        }
        if protocol_version == "2.1" and result["summary"] != expected_summary:
            raise AssertionError(
                f"controlled run must execute only the mapped recipes and classify all other cases: "
                f"{result['summary']!r}"
            )
        expected_kmip20_summary = {
            "total": len(version_cases),
            "passed": len(expected_cases) + local_value_cases,
            "failed": 0,
            "unsupported": 0,
            "policy_restricted": 0,
            "not_run": len(version_cases) - len(expected_cases) - local_value_cases,
        }
        if protocol_version == "2.0" and result["summary"] != expected_kmip20_summary:
            raise AssertionError(
                f"controlled KMIP 2.0 run did not preserve complete case classification: "
                f"{result['summary']!r}"
            )
        if not markdown_path.is_file() or not html_path.is_file():
            raise AssertionError("requested human-readable reports were not written")
        secret_values = (
            b"interop-user", b"interop-password", b"controlled-login-ticket", b"KMIPKIT-SECRET!!"
        )
        for report_path in (result_path, markdown_path, html_path):
            report_bytes = report_path.read_bytes()
            if any(secret in report_bytes for secret in secret_values):
                raise AssertionError("a report contains configured authentication material")
        if any(secret.decode("ascii") in run.stdout or secret.decode("ascii") in run.stderr
               for secret in secret_values):
            raise AssertionError("console output contains configured authentication material")
        missing_auth_environment = os.environ.copy()
        missing_auth_environment.pop("KMIPKIT_INTEROP_TEST_USERNAME", None)
        missing_auth_environment.pop("KMIPKIT_INTEROP_TEST_PASSWORD", None)
        missing_auth = subprocess.run(
            [str(binary), "run", "--connection", str(connection_path), "--catalog", str(catalog),
             "--output", str(directory / "missing-auth-results.json")],
            cwd=repository,
            capture_output=True,
            text=True,
            env=missing_auth_environment,
            timeout=10,
            check=False,
        )
        if missing_auth.returncode != 2 or "environment variables are unavailable" not in missing_auth.stderr:
            raise AssertionError("missing authentication variables must stop before any network exchange")
        if any(secret.decode("ascii") in missing_auth.stdout or secret.decode("ascii") in missing_auth.stderr
               for secret in secret_values):
            raise AssertionError("credential diagnostics exposed authentication material")
        preflight_without_auth = subprocess.run(
            [str(binary), "preflight", "--connection", str(connection_path)],
            cwd=repository,
            capture_output=True,
            text=True,
            env=missing_auth_environment,
            timeout=10,
            check=False,
        )
        if preflight_without_auth.returncode != 2 or \
                "environment variables are unavailable" not in preflight_without_auth.stderr:
            raise AssertionError("preflight must detect missing authentication without opening a socket")
        for case in result["cases"]:
            if case["status"] in ("not_run", "policy_restricted") and not case.get("diagnostic"):
                raise AssertionError("unexecuted cases must include a sanitized reason")
        if run.stderr:
            print(run.stderr, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
