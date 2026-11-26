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
ASYNC_NO_RECURSIVE_POLL_CANCEL_CASE = "test-async-no-recursive-async-poll-cancel"
ASYNC_POLL_FINAL_FAILURE_CASE = "test-async-poll-final-operation-failure"
SECRET_DATA_MATERIAL = b"synthetic-secret-material"
BATCH_CRYPTO_AES256_CBC_CIPHERTEXT = bytes.fromhex("78e16b06817a4453abef8a235fa9fa51")


def tlv(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes([item_type]) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def integer(tag: int, value: int, item_type: int = 2) -> bytes:
    return tlv(tag, item_type, struct.pack(">i", value))


def enumeration(tag: int, value: int) -> bytes:
    return tlv(tag, 5, struct.pack(">I", value))


def crypto_output_for_request(operations: tuple[int, ...], request: bytes) -> bytes | None:
    """Choose the deterministic Encrypt output from the request's cipher mode."""
    if (operations in ((31,), (1, 18, 31, 19, 20)) and
            enumeration(0x420011, 1) in request):
        return BATCH_CRYPTO_AES256_CBC_CIPHERTEXT
    return None


def top_level_batch_operations(message: bytes) -> tuple[int, ...]:
    """Read only the Operation fields directly under each top-level Batch Item."""
    if len(message) < 8 or message[3] != 1:
        raise AssertionError("request is not a TTLV Structure")
    message_length = int.from_bytes(message[4:8], "big")
    if 8 + message_length > len(message):
        raise AssertionError("request TTLV Structure is truncated")

    def children(value: bytes):
        offset = 0
        while offset + 8 <= len(value):
            item_tag = int.from_bytes(value[offset:offset + 3], "big")
            item_type = value[offset + 3]
            item_length = int.from_bytes(value[offset + 4:offset + 8], "big")
            end = offset + 8 + item_length
            padded_end = offset + 8 + ((item_length + 7) // 8) * 8
            if end > len(value) or padded_end > len(value):
                raise AssertionError("request contains a truncated TTLV item")
            yield item_tag, item_type, value[offset + 8:end]
            offset = padded_end
        if offset != len(value):
            raise AssertionError("request contains trailing TTLV bytes")

    operations = []
    for item_tag, item_type, item_value in children(message[8:8 + message_length]):
        if item_tag != 0x42000F or item_type != 1:
            continue
        for field_tag, field_type, field_value in children(item_value):
            if field_tag == 0x42005C and field_type == 5:
                if len(field_value) != 4:
                    raise AssertionError("Batch Item Operation is not a 32-bit Enumeration")
                operations.append(int.from_bytes(field_value, "big"))
    return tuple(operations)


def ttlv_contains_tag(message: bytes, target_tag: int) -> bool:
    """Search a validated TTLV message and nested Structures for a tag."""
    if len(message) < 8:
        raise AssertionError("request TTLV is truncated")
    item_tag = int.from_bytes(message[:3], "big")
    item_type = message[3]
    item_length = int.from_bytes(message[4:8], "big")
    padded_end = 8 + ((item_length + 7) // 8) * 8
    if padded_end > len(message):
        raise AssertionError("request TTLV item is truncated")
    if item_tag == target_tag:
        return True
    if item_type != 1:
        return False
    offset = 8
    while offset < 8 + item_length:
        child_length = int.from_bytes(message[offset + 4:offset + 8], "big")
        child_end = offset + 8 + ((child_length + 7) // 8) * 8
        if child_end > 8 + item_length or child_end > len(message):
            raise AssertionError("request contains a truncated nested TTLV item")
        if ttlv_contains_tag(message[offset:child_end], target_tag):
            return True
        offset = child_end
    if offset != 8 + item_length:
        raise AssertionError("request contains trailing nested TTLV bytes")
    return False


def extract_tlvs(message: bytes, target_tag: int, target_type: int) -> list[bytes]:
    """Collect raw values of all nested TTLV items matching tag and type."""
    if len(message) < 8:
        raise AssertionError("request TTLV is truncated")
    item_tag = int.from_bytes(message[:3], "big")
    item_type = message[3]
    item_length = int.from_bytes(message[4:8], "big")
    padded_end = 8 + ((item_length + 7) // 8) * 8
    if padded_end > len(message):
        raise AssertionError("request TTLV item is truncated")
    if item_tag == target_tag and item_type == target_type:
        return [message[8:8 + item_length]]
    if item_type != 1:
        return []
    matches: list[bytes] = []
    offset = 8
    while offset < 8 + item_length:
        child_length = int.from_bytes(message[offset + 4:offset + 8], "big")
        child_end = offset + 8 + ((child_length + 7) // 8) * 8
        if child_end > 8 + item_length or child_end > len(message):
            raise AssertionError("request contains a truncated nested TTLV item")
        matches.extend(extract_tlvs(message[offset:child_end], target_tag, target_type))
        offset = child_end
    return matches


def response(operations: tuple[int, ...], protocol_minor: int,
             selected_attribute_tag: int = 0,
             created_object_type: int = 2,
             selected_attribute_wire: bytes | None = None,
             unique_batch_item_ids: tuple[bytes, ...] | None = None,
             result_statuses: tuple[int, ...] | None = None,
             result_reasons: tuple[int | None, ...] | None = None,
             rng_length: int = 16,
             cancellation_result: int = 1,
             returned_unique_identifier: bytes | None = None,
             returned_unique_identifiers: tuple[bytes, ...] | None = None,
             discover_versions_minors: tuple[int, ...] | None = None,
             crypto_output: bytes | None = None,
             signature_data: bytes | None = None,
             multipart_hash: bool = False,
             final_hash: bool = False,
             hash_correlation_value: bytes | None = None,
             unknown_key_format: bool = False) -> bytes:
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
        if operations == (2, 18):
            if item_index == 0:
                payload_value = tlv(0x420066, 7, b"fixture-private-key-1") + \
                    tlv(0x42006F, 7, b"fixture-public-key-1")
            else:
                payload_value = tlv(
                    0x420094, 7, returned_unique_identifier or b"fixture-private-key-1")
        elif operations == (29,):
            payload_value = tlv(0x420066, 7, b"fixture-rekeyed-private-key-1") + \
                tlv(0x42006F, 7, b"fixture-rekeyed-public-key-1")
        elif operations in ((2, 18, 33, 19, 20, 20), (2, 18, 34, 19, 20, 20)):
            verify = operations[2] == 34
            target_identifier = b"fixture-public-key-1" if verify else b"fixture-private-key-1"
            other_identifier = b"fixture-private-key-1" if verify else b"fixture-public-key-1"
            if item_index == 0:
                payload_value = tlv(0x420066, 7, b"fixture-private-key-1") + \
                    tlv(0x42006F, 7, b"fixture-public-key-1")
            elif item_index == 2 and verify:
                payload_value = tlv(0x420094, 7, target_identifier) + enumeration(0x42009B, 2)
            elif item_index == 2:
                payload_value = tlv(0x420094, 7, target_identifier) + \
                    tlv(0x4200C3, 8, signature_data if signature_data is not None else bytes(256))
            else:
                response_identifier = other_identifier if item_index == 5 else target_identifier
                payload_value = tlv(0x420094, 7, response_identifier)
        elif operation == 2 and len(operations) == 1:
            payload_value = tlv(0x420066, 7, b"fixture-private-key-1") + \
                tlv(0x42006F, 7, b"fixture-public-key-1")
        elif operation == 6 and len(operations) == 1:
            payload_value = tlv(0x420094, 7, b"fixture-certificate-1")
        elif operation == 7 and len(operations) == 1:
            payload_value = tlv(0x420094, 7, b"fixture-recertified-certificate-1")
        elif operation == 33 and len(operations) == 1:
            payload_value = tlv(
                0x420094, 7, returned_unique_identifier or b"fixture-private-key-1"
            ) + tlv(0x4200C3, 8, signature_data if signature_data is not None else bytes(256))
        elif operation == 34 and len(operations) == 1:
            payload_value = tlv(
                0x420094, 7, returned_unique_identifier or b"fixture-public-key-1"
            ) + enumeration(0x42009B, 2)
        elif operations == (2, 20, 20):
            if item_index == 0:
                payload_value = tlv(0x420066, 7, b"fixture-private-key-1") + \
                    tlv(0x42006F, 7, b"fixture-public-key-1")
            else:
                identifier = b"fixture-private-key-1" if item_index == 1 else b"fixture-public-key-1"
                payload_value = tlv(0x420094, 7, identifier)
        elif operations == (2, 18, 6, 19, 20, 20, 20):
            if item_index == 0:
                payload_value = tlv(0x420066, 7, b"fixture-private-key-1") + \
                    tlv(0x42006F, 7, b"fixture-public-key-1")
            elif item_index == 2:
                payload_value = tlv(0x420094, 7, b"fixture-certificate-1")
            else:
                identifier = {
                    1: b"fixture-private-key-1", 3: b"fixture-private-key-1",
                    4: b"fixture-certificate-1", 5: b"fixture-private-key-1",
                    6: b"fixture-public-key-1",
                }[item_index]
                payload_value = tlv(0x420094, 7, identifier)
        elif operations == (2, 18, 6, 7, 19, 20, 20, 20, 20):
            identifiers = {
                1: b'fixture-private-key-1',
                2: b'fixture-certificate-1',
                3: b'fixture-recertified-certificate-1',
                4: b'fixture-private-key-1',
                5: b'fixture-recertified-certificate-1',
                6: b'fixture-certificate-1',
                7: b'fixture-private-key-1',
                8: b'fixture-public-key-1',
            }
            if item_index == 0:
                payload_value = tlv(0x420066, 7, b'fixture-private-key-1') + \
                    tlv(0x42006F, 7, b'fixture-public-key-1')
            else:
                payload_value = tlv(0x420094, 7, identifiers[item_index])
        elif operation == 23:
            payload_value = enumeration(0x42009B, 1)
        elif operation == 30:
            supported_minors = discover_versions_minors or (protocol_minor,)
            payload_value = b"".join(
                tlv(0x420069, 1,
                    integer(0x42006A, 2) + integer(0x42006B, minor))
                for minor in supported_minors
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
            if unknown_key_format:
                key_material = tlv(0x420043, 8, b"\x01\x02\x03")
                key_value = tlv(0x420045, 1, key_material)
                key_block = enumeration(0x420042, 0xF1234567) + key_value
                public_key = tlv(0x42006D, 1, tlv(0x420040, 1, key_block))
                payload_value = (
                    enumeration(0x420057, 3)
                    + tlv(0x420094, 7, b"fixture-unknown-key-format")
                    + public_key
                )
            else:
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
        elif operation == 43:
            key_material = tlv(0x420043, 8, b"KMIPKIT-SECRET!!")
            key_value = tlv(0x420045, 1, key_material)
            key_block = (
                enumeration(0x420042, 1) + key_value + enumeration(0x420028, 3)
            )
            symmetric_key = tlv(0x42008F, 1, tlv(0x420040, 1, key_block))
            attributes = enumeration(0x420028, 3) + integer(0x42002A, 128)
            payload_value = (
                enumeration(0x420057, 2)
                + tlv(0x420094, 7, b"fixture-key-1")
                + tlv(0x420125, 1, attributes)
                + symmetric_key
            )
        elif operation == 39:
            if final_hash:
                payload_value = tlv(0x4200C2, 8, hashlib.sha256(b"KMIP Hash!").digest())
            elif multipart_hash:
                payload_value = (tlv(0x4200D6, 8, hash_correlation_value)
                                 if hash_correlation_value is not None else b"")
            else:
                payload_value = tlv(0x4200C2, 8, hashlib.sha256(b"abc").digest())
        elif operation == 40:
            payload_value = (tlv(0x420094, 7, b"fixture-split-part-1") +
                             tlv(0x420094, 7, b"fixture-split-part-2"))
        elif operation == 41:
            payload_value = tlv(0x420094, 7, b"fixture-joined-key")
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
        elif operation == 51:
            payload_value = (
                tlv(0x42015A, 1, integer(0x42015A, -12345))
                + tlv(0x4200D6, 8, b"pkcs11-response-correlation")
                + tlv(0x42015C, 1, tlv(0x42015C, 8, b"controlled-pkcs11-output"))
                + tlv(0x42015D, 1, enumeration(0x42015D, 0xFFFF1234))
                + integer(0x541234, 7)
            )
        elif operation == 16:
            payload_value = (
                tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1")
                + integer(0x420049, 3600, item_type=10)
                + tlv(0x420048, 9, struct.pack(">q", 1_700_000_000))
                + integer(0x541234, -7)
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
        elif operation in (3, 42):
            identifier = returned_unique_identifier or b"fixture-key-1"
            payload_value = tlv(0x420094, 7, identifier)
        elif operation == 4:
            identifier = (returned_unique_identifiers[item_index]
                          if returned_unique_identifiers is not None else
                          returned_unique_identifier or b"fixture-rekeyed-key")
            payload_value = tlv(0x420094, 7, identifier)
        elif operation == 5:
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-derived-key")
        elif operation == 31:
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1") + tlv(
                0x4200C2, 8, crypto_output if crypto_output is not None else
                bytes.fromhex("69c4e0d86a7b0430d8cdb78070b4c55a"))
        elif operation == 32:
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1") + tlv(
                0x4200C2, 8, bytes.fromhex("00112233445566778899aabbccddeeff"))
        elif operation == 35:
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1") + tlv(
                0x4200C6, 8, bytes(range(32)))
        elif operation == 36:
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1") + enumeration(0x42009B, 1)
        elif operation in (13, 14, 15, 48, 49):
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1")
        elif operation == 11:
            if selected_attribute_wire is not None:
                attributes = selected_attribute_wire
            elif selected_attribute_tag == 0x420028:
                attributes = enumeration(0x420028, 3)
            elif selected_attribute_tag == 0x42002A:
                attributes = integer(0x42002A, 128)
            else:
                attributes = enumeration(0x420028, 3) + integer(0x42002A, 128)
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1") + tlv(0x420125, 1, attributes)
        elif operation == 12:
            payload_value = (
                tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1")
                + enumeration(0x42013B, 0x420028)
                + enumeration(0x42013B, 0x42002A)
            )
        elif operation in (9, 18, 21, 22):
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1")
        elif operation == 20:
            identifier = (returned_unique_identifiers[item_index]
                          if returned_unique_identifiers is not None else
                          returned_unique_identifier or b"fixture-key-1")
            payload_value = tlv(0x420094, 7, identifier)
        elif operation == 17:
            payload_value = tlv(0x420094, 7, returned_unique_identifier or b"fixture-key-1")
        elif operation == 19 and result_status == 0:
            identifier = (returned_unique_identifiers[item_index]
                          if returned_unique_identifiers is not None else
                          returned_unique_identifier or b"fixture-key-1")
            payload_value = tlv(0x420094, 7, identifier)
        elif operation == 56:
            payload_value = tlv(0x420168, 1, b"")
        unique_batch_item_id = (unique_batch_item_ids[item_index]
                                if unique_batch_item_ids is not None else {
                                    3: b"interop-register",
                                    42: b"import-object",
                                    13: b"interop-attributes",
                                    14: b"interop-attributes",
                                    15: b"interop-delete-attribute",
                                    48: b"interop-attributes",
                                    49: b"interop-attributes",
                                    54: b"interop-set-defaults",
                                    55: b"interop-set-constraints",
                                    1: b"interop-create",
                                    11: b"interop-attributes",
                                    12: b"interop-attributes",
                                    20: b"interop-destroy",
                                    10: b"interop-get",
                                    9: b"interop-check",
                                    21: b"interop-archive",
                                    22: b"interop-recover",
                                    18: b"interop-activate",
                                    19: b"interop-revoke",
                                    43: b"interop-export",
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
        if operations == (1, 18, 51, 19, 20):
            unique_batch_item_id = (
                b"pkcs11-create", b"pkcs11-activate", b"pkcs11-target",
                b"pkcs11-revoke", b"pkcs11-destroy"
            )[item_index]
        if operations == (1, 16, 19, 20):
            unique_batch_item_id = (
                b"lease-create", b"lease-target", b"lease-revoke", b"lease-destroy"
            )[item_index]
        if operations == (41, 20, 20, 20):
            unique_batch_item_id = (
                b"join-split-target", b"join-split-destroy-joined",
                b"join-split-destroy-part-1", b"join-split-destroy-part-2"
            )[item_index]
        item_fields = integer(0x42005C, operation, 5)
        if unique_batch_item_id is not None:
            item_fields += tlv(0x420093, 8, unique_batch_item_id)
        item_fields += integer(0x42007F, result_status, 5)
        if result_reason is not None:
            item_fields += enumeration(0x42007E, result_reason)
        if result_status in (0, 3):
            item_fields += tlv(0x42007C, 1, payload_value)
        batch_items.append(tlv(
            0x42000F,
            1,
            item_fields,
        ))
    return tlv(0x42007B, 1, header + b"".join(batch_items))


def pending_create_response(protocol_minor: int, correlation: bytes,
                            unique_batch_item_id: bytes | None = None) -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1,
                 version + tlv(0x420092, 9, struct.pack(">q", int(time.time()))) +
                 integer(0x42000D, 1))
    item_fields = enumeration(0x42005C, 1)
    if unique_batch_item_id is not None:
        item_fields += tlv(0x420093, 8, unique_batch_item_id)
    item_fields += enumeration(0x42007F, 2)
    item = tlv(0x42000F, 1, item_fields +
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


def query_async_requests_response(protocol_minor: int, correlation: bytes) -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
                 + integer(0x42000D, 1))
    payload = tlv(0x42007C, 1, tlv(0x420176, 1, tlv(0x420006, 8, correlation)))
    item = tlv(0x42000F, 1, enumeration(0x42005C, 57) + enumeration(0x42007F, 0) + payload)
    return tlv(0x42007B, 1, header + item)


def completed_poll_create_response(protocol_minor: int,
                                   identifier: bytes = b"fixture-key-1") -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
                 + integer(0x42000D, 1))
    payload = tlv(0x42007C, 1,
                  enumeration(0x420057, 2) + tlv(0x420094, 7, identifier))
    item = tlv(0x42000F, 1, enumeration(0x42005C, 26) + enumeration(0x42007F, 0) + payload)
    return tlv(0x42007B, 1, header + item)


def failed_poll_create_response(protocol_minor: int,
                                message: bytes = b"controlled Create failure") -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
                 + integer(0x42000D, 1))
    item = tlv(0x42000F, 1,
               enumeration(0x42005C, 26) + enumeration(0x42007F, 1) +
               enumeration(0x42007E, 256) + tlv(0x42007D, 7, message))
    return tlv(0x42007B, 1, header + item)


def pending_operation_response(protocol_minor: int, operation: int, correlation: bytes) -> bytes:
    """Build a Pending Get or Create response with its exact correlation bytes."""
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
                 + integer(0x42000D, 1))
    item = tlv(0x42000F, 1, enumeration(0x42005C, operation) + enumeration(0x42007F, 2)
               + tlv(0x420006, 8, correlation) + tlv(0x42007C, 1, b""))
    return tlv(0x42007B, 1, header + item)


def async_query_outstanding_response(protocol_minor: int, correlation: bytes) -> bytes:
    """Build one KMIP 2.1 Asynchronous Request selected by the filters."""
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
                 + integer(0x42000D, 1))
    request = tlv(0x420173, 1, tlv(0x420006, 8, correlation)
                  + enumeration(0x42005C, 1)
                  + tlv(0x420174, 11, struct.pack(">q", int(time.time() * 1_000_000)))
                  + enumeration(0x420175, 2))
    payload = tlv(0x42007C, 1, request)
    item = tlv(0x42000F, 1, enumeration(0x42005C, 57) + enumeration(0x42007F, 0) + payload)
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


def versioned_operation(binary: Path, openssl: str, repository: Path, case_id: str,
                        operation: int, protocol_version: str = "3.0", *, invalid_payload: bool = False, generic_locate: bool = False) -> int:
    with tempfile.TemporaryDirectory(prefix="kmipkit-c-draft-") as raw_directory:
        directory = Path(raw_directory)
        ca, server_cert, server_key, client_cert, client_key = certificate_files(openssl, directory)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_3
        context.load_cert_chain(server_cert, server_key)
        context.load_verify_locations(ca)
        context.verify_mode = ssl.CERT_REQUIRED
        catalog = json.loads((repository / "verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
        case = next(case for case in catalog["test_cases"] if case["case_id"] == case_id)
        case["execution"]["protocol_versions"] = [protocol_version]
        major, minor = map(int, protocol_version.split("."))
        if major == 3:
            case["normative_refs"] = [{"document": "OASIS KMIP 3.0 Working Draft", "section": "6.1"}]
        catalog["test_cases"] = [case]
        catalog["target_versions"] = [protocol_version]
        catalog["statistics"]["total_cases"] = 1
        catalog_path = directory / "catalog.json"
        catalog_path.write_text(json.dumps(catalog))
        request_wires = []
        allow_mutations = case["execution"]["mutation"] == "mutating"
        offered = tlv(0x420069, 1, integer(0x42006a, major) + integer(0x42006b, minor))
        payload = {
            0x1e: offered,
            0x02: b"",
            0x08: integer(0x4200d5, 0),
            0x13: b"",
            0x18: enumeration(0x42005c, 0x18) + enumeration(0x42005c, 0x3b),
            0x25: tlv(0x4200c2, 8, bytes(31 if invalid_payload else 32)),
            0x26: integer(0x4200c4, 33 if invalid_payload else 0),
            0x27: tlv(0x4200c2, 8, hashlib.sha256(b"wrong" if invalid_payload else b"abc").digest()),
            0x3b: b"",
        }[operation]
        if generic_locate:
            payload = integer(0x4200d5, 2) + tlv(0x420094, 7, b"known-id")
            if invalid_payload:
                payload += tlv(0x420094, 7, b"second-id")
        if generic_locate or (major < 3 and operation in (0x18, 0x1e, 0x25, 0x26, 0x27, 0x3b)):
            apis = ("generic-ttlv",)
        else:
            apis = ("typed-protocol",) if operation == 0x18 else ("typed-protocol", "fluent")
        for api in apis:
            with socket.socket() as server:
                server.bind(("127.0.0.1", 0))
                server.listen(1)
                server.settimeout(10)
                connection = {
                    "server_id": "draft-c-peer", "server_product": "controlled peer",
                    "server_version": "fixture", "server_configuration_fingerprint": "b" * 64,
                    "protocol_version": protocol_version, "transport": "raw_tls", "encoding": "TTLV", "api_level": api,
                    "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": server.getsockname()[1]}},
                    "tls": {"ca_cert_path": str(ca), "client_cert_path": str(client_cert),
                            "client_key_path": str(client_key), "server_name_override": "localhost"},
                    "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
                    "execution_policy": {"probe_unsupported": True, "allow_mutations": allow_mutations},
                }
                config = directory / f"{api}.json"
                result = directory / f"{api}-result.json"
                config.write_text(json.dumps(connection))
                command = [str(binary), "run", "--connection", str(config),
                           "--catalog", str(catalog_path), "--output", str(result)]
                if allow_mutations:
                    command.append("--allow-mutations")
                with subprocess.Popen(command, cwd=repository, stdout=subprocess.DEVNULL,
                                      stderr=subprocess.PIPE) as process:
                    active_raw = None
                    active_client = None

                    def accept_next_message():
                        nonlocal active_raw, active_client
                        while True:
                            if active_client is not None:
                                msg = read_message(active_client)
                                if msg is not None:
                                    return msg, active_client
                                active_client.close()
                                active_client = None
                                if active_raw is not None:
                                    active_raw.close()
                                    active_raw = None
                            active_raw, _ = server.accept()
                            active_client = context.wrap_socket(active_raw, server_side=True)
                            active_client.settimeout(5)

                    try:
                        request, client = accept_next_message()
                        if operation == 0x13:
                            assert request and top_level_batch_operations(request) == (1,)
                            assert integer(0x42006a, major) in request and integer(0x42006b, minor) in request
                            client.sendall(response((1,), minor, unique_batch_item_ids=(b"interop-create",)))
                            revoke_req, client = accept_next_message()
                            assert revoke_req and top_level_batch_operations(revoke_req) == (19,)
                            assert tlv(0x420081, 1, enumeration(0x420082, 6)) in revoke_req
                            assert tlv(0x420094, 7, b"fixture-key-1") in revoke_req
                            request_wires.append(revoke_req)
                            client.sendall(response((19,), minor, returned_unique_identifier=b"fixture-key-1",
                                                    unique_batch_item_ids=(b"interop-revoke",)))
                            destroy_req, client = accept_next_message()
                            assert destroy_req and top_level_batch_operations(destroy_req) == (20,)
                            client.sendall(response((20,), minor, returned_unique_identifier=b"fixture-key-1",
                                                    unique_batch_item_ids=(b"interop-destroy",)))
                        elif operation == 0x02:
                            assert request and top_level_batch_operations(request) == (2,)
                            assert integer(0x42006a, major) in request and integer(0x42006b, minor) in request
                            assert enumeration(0x420028, 4) in request
                            assert integer(0x42002A, 2048) in request
                            assert integer(0x42002C, 1) in request and integer(0x42002C, 2) in request
                            client.sendall(response((2,), minor, unique_batch_item_ids=(b"create-pair",)))
                            for destroy_id, destroy_uid in (
                                (b"destroy-private", b"fixture-private-key-1"),
                                (b"destroy-public", b"fixture-public-key-1"),
                            ):
                                destroy_req, client = accept_next_message()
                                assert destroy_req and top_level_batch_operations(destroy_req) == (20,)
                                assert tlv(0x420094, 7, destroy_uid) in destroy_req
                                client.sendall(response((20,), minor, returned_unique_identifier=destroy_uid,
                                                        unique_batch_item_ids=(destroy_id,)))
                        else:
                            assert request and enumeration(0x42005c, operation) in request
                            assert integer(0x42006a, major) in request and integer(0x42006b, minor) in request
                            if major == 3:
                                assert bytes.fromhex("42000d02") not in request
                            if operation == 0x25:
                                assert tlv(0x420079, 1, integer(0x4200c4, 32)) in request
                            if operation == 0x26:
                                assert tlv(0x420079, 1, tlv(0x4200c2, 8, b"A" * 32)) in request
                            if operation == 0x27:
                                assert enumeration(0x420038, 6) in request and tlv(0x4200c2, 8, b"abc") in request
                            if operation == 0x08:
                                assert integer(0x42004f, 1 if major == 3 or generic_locate else 3) in request
                                if major == 3:
                                    assert enumeration(0x420028, 3) in request
                                if generic_locate:
                                    assert tlv(0x420079, 1, integer(0x42004f, 1) +
                                               integer(0x4200d4, 0) + tlv(0x420125, 1, b"")) in request
                            if operation == 0x18:
                                assert tlv(0x420079, 1, enumeration(0x420074, 1)) in request
                            if operation == 0x1e:
                                assert tlv(0x420079, 1, offered) in request
                            if operation == 0x3b:
                                assert tlv(0x420079, 1, b"") in request
                            request_wires.append(request)
                            version = tlv(0x420069, 1, integer(0x42006a, major) + integer(0x42006b, minor))
                            timestamp = tlv(0x420092, 9, (1).to_bytes(8, "big"))
                            batch_count = integer(0x42000d, 1) if major < 3 else b""
                            header = tlv(0x42007a, 1, version + timestamp + batch_count)
                            response_item = tlv(0x42000f, 1, enumeration(0x42005c, operation)
                                                + enumeration(0x42007f, 0) + tlv(0x42007c, 1, payload))
                            client.sendall(tlv(0x42007b, 1, header + response_item))
                    finally:
                        if active_client is not None:
                            active_client.close()
                        elif active_raw is not None:
                            active_raw.close()
                    _, errors = process.communicate(timeout=10)
                    assert process.returncode == (3 if invalid_payload else 0), errors
                report = json.loads(result.read_text())
                assert report["catalog_sha256"] == hashlib.sha256(catalog_path.read_bytes()).hexdigest()
                row, = report["cases"]
                assert row["status"] == ("failed" if invalid_payload else "passed"), row
        if len(request_wires) > 1:
            assert request_wires[0] == request_wires[1]
    return 0


def versioned_owned_key(binary: Path, openssl: str, repository: Path, version: str) -> None:
    minor = int(version[-1])
    source = repository / "verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json"
    catalog = json.loads(source.read_text())
    cases = {row["case_id"]: row for row in catalog["test_cases"]}
    operations = {"create": (1, 20), "get": (1, 10, 20), "destroy": (1, 20), "get-attributes": (1, 11, 20),
                  "get-attribute-list": (1, 12, 20), "activate": (1, 18, 19, 20), "check": (1, 18, 9, 19, 20),
                  "revoke": (1, 18, 19, 20), "archive": (1, 21, 22, 20), "recover": (1, 21, 22, 20)}
    mutations = {"add-attribute": 0x0d, "modify-attribute": 0x0e,
                 "delete-attribute": 0x0f, "set-attribute": 0x31}
    operations.update({name: (1, 11, code, 11, 20) for name, code in mutations.items()})
    initial_description = tlv(0x4200fc, 7, b"kmipkit-attribute-fixture")
    initial_comment = tlv(0x4200fd, 7, b"kmipkit-attribute-fixture")
    changed_description = tlv(0x4200fc, 7, b"kmipkit-changed-description")
    description_reference = enumeration(0x42013b, 0x4200fc)
    comment_reference = enumeration(0x42013b, 0x4200fd)
    with tempfile.TemporaryDirectory(prefix="kmip-c-owned-") as temporary:
        directory = Path(temporary)
        ca, server_cert, server_key, client_cert, client_key = certificate_files(openssl, directory)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_3
        context.load_cert_chain(server_cert, server_key)
        context.load_verify_locations(ca)
        context.verify_mode = ssl.CERT_REQUIRED
        owned_id = tlv(0x420094, 7, b"owned-key")
        object_type = enumeration(0x420057, 2)
        symmetric_key = tlv(0x42008f, 1, tlv(0x420040, 1, enumeration(0x420042, 1)
                            + tlv(0x420045, 1, tlv(0x420043, 8, bytes(32)))
                            + enumeration(0x420028, 3) + integer(0x42002a, 256)))
        header = tlv(0x42007a, 1, tlv(0x420069, 1, integer(0x42006a, 2) + integer(0x42006b, minor))
                     + tlv(0x420092, 9, (1).to_bytes(8, "big")) + integer(0x42000d, 1))
        for target, exchanges in operations.items():
            for valid in ((True, False) if target in {"get", "get-attributes", "get-attribute-list", "activate", "revoke", "check", *mutations} else (True,)):
                description = b"" if target == "add-attribute" else (initial_comment if target == "delete-attribute" else initial_description)
                with socket.socket() as listener:
                    listener.bind(("127.0.0.1", 0))
                    listener.listen()
                    listener.settimeout(0.05)
                    connection = {
                        "server_id": "owned-key-peer", "server_configuration_fingerprint": "b" * 64,
                        "server_product": "controlled peer", "server_version": "fixture",
                        "protocol_version": version, "transport": "raw_tls", "encoding": "TTLV",
                        "api_level": "generic-ttlv",
                        "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": listener.getsockname()[1]}},
                        "tls": {"ca_cert_path": str(ca), "client_cert_path": str(client_cert),
                                "client_key_path": str(client_key), "server_name_override": "localhost"},
                        "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
                        "execution_policy": {"allow_mutations": True, "probe_unsupported": True},
                    }
                    config = directory / "connection.json"
                    selection = directory / "catalog.json"
                    output = directory / f"{target}-{valid}.json"
                    config.write_text(json.dumps(connection))
                    selection.write_text(json.dumps({**catalog, "test_cases": [cases[f"test-op-{target}"]],
                                                      "statistics": {**catalog["statistics"], "total_cases": 1}}))
                    command = [str(binary), "run", "--connection", str(config), "--catalog", str(selection),
                               "--output", str(output), "--allow-mutations"]
                    with subprocess.Popen(command, cwd=repository, stdout=subprocess.DEVNULL,
                                          stderr=subprocess.PIPE) as process:
                        try:
                            for operation in exchanges:
                                raw, _ = listener.accept()
                                with context.wrap_socket(raw, server_side=True) as stream:
                                    stream.settimeout(5)
                                    request = read_message(stream)
                                    assert request and top_level_batch_operations(request) == (operation,)
                                    fields = owned_id
                                    if operation == 1:
                                        assert enumeration(0x420028, 3) in request and integer(0x42002a, 256) in request
                                        if target in mutations:
                                            expected_initial = initial_comment if target == "delete-attribute" else initial_description
                                            assert (expected_initial in request) == (target != "add-attribute")
                                        fields = object_type + owned_id
                                    else:
                                        assert owned_id in request
                                        if operation not in (9, 11, 19, *mutations.values()):
                                            assert tlv(0x420079, 1, owned_id) in request
                                    if operation == 10:
                                        returned_id = owned_id if valid else tlv(0x420094, 7, b"foreign-key")
                                        fields = object_type + returned_id + symmetric_key
                                    if operation == 11 and target in mutations:
                                        expected_ref = comment_reference if target == "delete-attribute" else description_reference
                                        assert tlv(0x420079, 1, owned_id + expected_ref) in request
                                        if description:
                                            fields += tlv(0x420125, 1, description)
                                    elif operation == 11:
                                        assert tlv(0x420079, 1, owned_id + enumeration(0x42013b, 0x42002a)) in request
                                        fields += tlv(0x420125, 1, integer(0x42002a, 256 if valid else 128))
                                    if operation == mutations.get(target):
                                        selector = tlv(0x42013c, 1, initial_description) if target == "modify-attribute" else b""
                                        replacement = (comment_reference if target == "delete-attribute"
                                                       else tlv(0x42013d, 1, changed_description))
                                        assert tlv(0x420079, 1, owned_id + selector + replacement) in request
                                        if valid:
                                            description = b"" if target == "delete-attribute" else changed_description
                                    if operation == 12:
                                        fields += enumeration(0x42013b, 0x420028)
                                        if valid:
                                            fields += enumeration(0x42013b, 0x42002a)
                                    if operation == 9:
                                        assert tlv(0x420079, 1, owned_id + integer(0x42002c, 4)) in request
                                        fields += integer(0x42002c, 4 if valid else 8)
                                    if operation == 19:
                                        assert tlv(0x420079, 1, owned_id + tlv(0x420081, 1, enumeration(0x420082, 6))) in request
                                    if not valid and ((target == "activate" and operation == 18)
                                                      or (target == "revoke" and operation == 19)):
                                        fields = tlv(0x420094, 7, b"foreign-key")
                                    batch = tlv(0x42000f, 1, enumeration(0x42005c, operation)
                                                + enumeration(0x42007f, 0) + tlv(0x42007c, 1, fields))
                                    stream.sendall(tlv(0x42007b, 1, header + batch))
                            _, errors = process.communicate(timeout=5)
                            assert process.returncode == (0 if valid else 3), errors
                        finally:
                            if process.poll() is None:
                                process.kill()
                    row, = json.loads(output.read_text())["cases"]
                    assert row["status"] == ("passed" if valid else "failed"), row


def main() -> int:
    binary, openssl, repository = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    protocol_version = sys.argv[4] if len(sys.argv) > 4 else "2.1"
    if protocol_version == "3.0":
        versioned_operation(binary, openssl, repository, "test-op-ping", 0x3b)
        versioned_operation(binary, openssl, repository, "test-crypto-hash-sha_256-0", 0x27)
        versioned_operation(binary, openssl, repository, "test-op-locate", 0x08)
        return versioned_operation(binary, openssl, repository, "test-op-query", 0x18)
    if protocol_version not in ("2.0", "2.1"):
        raise AssertionError(f"unsupported fixture protocol version {protocol_version!r}")
    versioned_operation(binary, openssl, repository, "test-op-locate", 0x08, protocol_version)
    versioned_operation(binary, openssl, repository, "test-op-create-key-pair", 0x02, protocol_version)
    versioned_operation(binary, openssl, repository, "test-op-revoke", 0x13, protocol_version)
    if os.name == "posix":
        versioned_owned_key(binary, openssl, repository, protocol_version)
        for invalid in (False, True):
            versioned_operation(binary, openssl, repository, "test-crypto-rng-retrieve-rng-32", 0x25,
                                protocol_version, invalid_payload=invalid)
            versioned_operation(binary, openssl, repository, "test-crypto-rng-seed-rng-32", 0x26,
                                protocol_version, invalid_payload=invalid)
            versioned_operation(binary, openssl, repository, "test-op-locate", 0x08, protocol_version,
                                generic_locate=True, invalid_payload=invalid)
        versioned_operation(binary, openssl, repository, "test-op-query", 0x18, protocol_version)
        versioned_operation(binary, openssl, repository, "test-op-discover-versions", 0x1e, protocol_version)
        versioned_operation(binary, openssl, repository, "test-crypto-hash-sha_256-0", 0x27, protocol_version)
        versioned_operation(binary, openssl, repository, "test-crypto-hash-sha_256-0", 0x27, protocol_version, invalid_payload=True)
        if protocol_version == "2.1":
            versioned_operation(binary, openssl, repository, "test-op-ping", 0x3b, protocol_version)
    protocol_minor = int(protocol_version.split(".")[1])
    registered_key_batch = (3, 18, 31, 32, 19, 20)
    stop_on_first_failure_batch = (10, 10)
    registered_key_batch_item_ids = (
        b"crypto-register", b"crypto-activate", b"crypto-encrypt",
        b"crypto-decrypt", b"crypto-revoke", b"crypto-destroy",
    )
    expected_operations = (
        (1, 20, 1, 11, 20, 1, 12, 20, 1, 10, 20, 1, 9, 20, 1, 19, 20,
         1, 20,
         1, 18, 19, 20, 8, 24, 44, 45, 1, 11, 20, 1, 11, 20, 1, 20, 1, 20, 1, 20,
         1, 18, 20, 19, 20)
        if protocol_version == "2.0"
        else (1, 20, 1, 11, 20, 1, 12, 20, 1, 10, 20, 1, 9, 20, 1, 19, 20,
              1, 20, 1, 18, 19, 20, 1, 56, 20, 8, 24, 30, 44, 45, 59, 1, 11, 20,
              1, 11, 20, 1, 20, 1, 20, 1, 20, 1, 18, 20, 19, 20)
    )
    expected_cases = (
        ("test-op-locate", "test-op-query", "test-op-log", "test-op-login", "test-op-logout",
         "test-op-interop")
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
    expected_operations += (1, 43, 20)
    expected_cases += ("test-op-export",)
    expected_operations += (37, 38, 39)
    expected_operations += (39, 37, 38)
    expected_operations += (1, 13, 19, 20)
    expected_operations += (1, 49, 19, 20)
    expected_operations += (1, 17, 20)
    expected_operations += (45, 46)
    if protocol_version == "2.1":
        expected_operations += (55, 54)
    expected_operations += (1, 13, 15, 19, 20)
    expected_operations += (1, 13, 14, 19, 20)
    expected_operations += (1, 13, 14, 19, 20)
    expected_operations += (1, 18, 12, 19, 20)
    expected_operations += (1, 18, 10, 19, 20)
    expected_operations += (1, 18, 19, 20, 1, 18, 19, 20)
    expected_operations += (1, 48, 19, 20)
    expected_operations += (52, 52, 52)
    expected_operations += (26,)
    expected_cases += ("test-async-unknown-correlation-rejected",)
    expected_operations += (10, 12, 10, 12, 10, 12, 10, 12, 10, 12)
    expected_operations += (10,)
    expected_cases += ("test-batch-single-item-default-continuation",)
    expected_operations += registered_key_batch
    expected_operations += (3, 20)
    expected_operations += (3, 20)
    expected_operations += (3, 20)
    expected_operations += (42, 20)
    expected_cases += ("test-op-import",)
    expected_cases += ("test-obj-opaque-object",)
    expected_cases += ("test-obj-pgp-key",)
    expected_cases += ("test-obj-secret-data",)
    expected_operations += (1, 18, 31, 19, 20)
    expected_operations += (1, 18, 31, 19, 20)
    expected_operations += (1, 31, 20)  # test-neg-encrypt-preactive-key
    expected_operations += (1, 18, 32, 19, 20)
    expected_operations += (1, 18, 35, 19, 20)
    expected_operations += (1, 18, 35, 19, 20)
    expected_operations += (1, 18, 35, 36, 19, 20)
    expected_operations += (1, 18, 35, 36, 19, 20)
    expected_operations += (2, 18, 33, 19, 20, 20)
    expected_operations += (2, 18, 33, 19, 20, 20)
    expected_operations += (2, 18, 33, 19, 20, 20)
    expected_operations += (2, 18, 33, 19, 20, 20)
    expected_operations += (2, 18, 34, 19, 20, 20)
    expected_operations += (2, 18, 34, 19, 20, 20)
    expected_operations += (2, 18, 34, 19, 20, 20)
    expected_operations += (2, 20, 20)
    expected_operations += (2, 20, 20)  # test-obj-private-key
    expected_operations += (2, 20, 20)  # test-obj-public-key
    expected_operations += (40, 20, 20)
    expected_operations += (40, 20, 20)
    expected_cases += ("test-obj-split-key",)
    expected_operations += (40, 41, 20, 20, 20)
    expected_operations += (1, 4, 20, 20)
    expected_operations += (1, 18, 5, 20, 19, 20)
    expected_cases += ("test-op-derive-key",)
    expected_operations += (2, 18, 6, 19, 20, 20, 20)
    expected_operations += (2, 18, 6, 7, 19, 20, 20, 20, 20)
    expected_operations += (23,)
    expected_operations += (1, 18, 51, 19, 20)
    expected_cases += ("test-op-pkcs#11",)
    expected_operations += (1, 16, 20)
    expected_cases += ("test-op-obtain-lease",)
    expected_operations += (1, 21, 20)
    expected_cases += ("test-op-archive",)
    expected_operations += (1, 21, 22, 20)
    expected_cases += ("test-op-recover",)
    expected_operations += stop_on_first_failure_batch
    expected_operations += (1, 10)
    expected_cases += ("test-batch-undo-rolls-back-prior-success",)
    expected_operations += (1, 25)
    expected_cases += ("test-op-cancel",)
    expected_operations += (1, 25) * len(ASYNC_CANCEL_RESULT_CASES)
    expected_cases += ASYNC_CANCEL_RESULT_CASES
    expected_operations += (1,)
    expected_cases += ("test-async-indicator-mandatory-pending",)
    expected_operations += (1,)
    expected_cases += ("test-async-indicator-omitted-default",)
    expected_operations += (1,)
    expected_cases += ("test-async-indicator-optional-completes-or-pends",)
    expected_operations += (1,)
    expected_cases += ("test-async-indicator-prohibited-synchronous",)
    expected_operations += (1, 26, 26, 26, 20)
    expected_cases += ("test-async-poll-repeated-pending-to-success",)
    expected_operations += (1, 26, 26, 26, 20)
    expected_cases += ("test-op-poll",)
    expected_operations += (1, 26, 26)
    expected_cases += (ASYNC_POLL_FINAL_FAILURE_CASE,)
    if protocol_version == "2.1":
        expected_operations += (1, 57, 26, 20)
        expected_cases += ("test-async-lifecycle-poll",)
        expected_operations += (1, 57, 19, 20)
        expected_cases += ("test-op-query-asynchronous-requests",)
        expected_operations += (26, 25)
        expected_cases += (ASYNC_NO_RECURSIVE_POLL_CANCEL_CASE,)
        expected_operations += (1, 26, 25)
        expected_cases += (ASYNC_CANCEL_POLL_RACE_CASE,)
        expected_operations += (1, 58, 26, 20)
        expected_cases += ("test-async-process-then-poll-state",)
        expected_operations += (1, 58, 26, 19, 20)
        expected_cases += ("test-op-process",)
        expected_operations += (1, 1, 10, 26, 57, 26, 26, 20, 20)
        expected_cases += ("test-async-query-outstanding-filter",)
    expected_cases += ("test-op-hash", "test-op-rng-retrieve", "test-op-rng-seed")
    expected_cases += (
        "test-crypto-hash-sha_256-0",
        "test-crypto-rng-retrieve-rng-32",
        "test-crypto-rng-seed-rng-32",
    )
    expected_cases += ("test-multipart-hash-sha256",)
    expected_operations += (39, 39, 39)
    expected_cases += ("test-batch-register-encrypt-decrypt-destroy",)
    expected_cases += ("test-op-encrypt",)
    expected_cases += ("test-crypto-encrypt-aes-256-cbc",)
    expected_cases += ("test-neg-encrypt-preactive-key",)
    expected_operations += (1, 18, 32, 19, 20)
    expected_cases += ("test-crypto-decrypt-aes-256-cbc",)
    expected_cases += ("test-op-decrypt",)
    expected_cases += ("test-op-mac",)
    expected_cases += ("test-crypto-mac-hmac_sha256-256",)
    expected_cases += ("test-op-mac-verify",)
    expected_cases += ("test-crypto-mac-verify-hmac_sha256-256",)
    expected_cases += ("test-op-sign", "test-sign-missing-managed-object-cryptographic-parameters",
                       "test-op-signature-verify")
    expected_cases += ("test-crypto-sign-ecdsa-256-p_256",)
    expected_cases += ("test-crypto-sign-rsa-2048-pkcs1_v1_5",)
    expected_cases += ("test-crypto-signature-verify-ecdsa-256-p_256",)
    expected_cases += ("test-crypto-signature-verify-rsa-2048-pkcs1_v1_5",)
    expected_cases += ("test-op-create-key-pair",)
    expected_cases += ("test-op-create-split-key",)
    expected_cases += ("test-op-join-split-key",)
    expected_cases += ("test-op-re-key",)
    expected_operations += (2, 18, 29, 20, 20, 19, 20, 20)
    expected_cases += ("test-op-re-key-key-pair",)
    expected_cases += ("test-op-certify",)
    expected_cases += ("test-op-re-certify", "test-op-validate")
    expected_cases += ("test-batch-stop-on-first-failure",)
    expected_cases += ("test-neg-destroy-active-without-revoke",)
    expected_cases += ("test-op-add-attribute",)
    expected_cases += ("test-op-set-attribute",)
    expected_cases += ("test-op-get-usage-allocation",)
    if protocol_version == "2.1":
        expected_cases += ("test-op-set-constraints", "test-op-set-defaults")
    expected_cases += ("test-op-delete-attribute",)
    expected_cases += ("test-op-modify-attribute",)
    expected_cases += ("test-neg-modify-attribute-current-mismatch",)
    expected_cases += ("test-op-adjust-attribute",)
    expected_cases += ("test-batch-create-activate-destroy",)
    expected_cases += ("test-batch-placeholder-chain-resolves",)
    expected_cases += ("test-batch-placeholder-forward-reference-rejected",)
    expected_cases += (
        "test-batch-order-required-placeholder-resolution",
        "test-batch-order-omitted-defaults-true",
    )
    expected_cases += (
        "test-batch-count-matches-items",
        "test-batch-count-mismatch-rejected",
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
        expected_cases += (
            "test-v20-reject-get-constraints", "test-v20-reject-ping",
            "test-v20-reject-process", "test-v20-reject-set-defaults",
            "test-v20-reject-set-constraints", "test-v20-reject-query-asynchronous-requests",
        )
    shared = repository / "verification/interop/shared"
    catalog = shared / "catalog/kmipkit-2x-matrix-catalog.json"
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
    extra_attr_cases = tuple(
        case["case_id"] for case in version_cases
        if case["case_id"].startswith("test-attr-") and case["case_id"] not in expected_cases
    )
    expected_cases += extra_attr_cases
    expected_operations += (1, 11, 20) * len(extra_attr_cases)
    local_value_cases = sum(
        case["execution"]["mode"] == "local_model"
        and ("enum_type" in case["requires"] or "bitmask_type" in case["requires"])
        for case in version_cases
    )
    with tempfile.TemporaryDirectory(prefix="kmipkit-c-live-") as raw_directory:
        directory = Path(raw_directory)
        ca_cert, server_cert, server_key, client_cert, client_key = certificate_files(openssl, directory)
        # Private and public key object cases now create a key pair and destroy
        # both components instead of creating and destroying one object each.
        expected_operations = list(expected_operations)
        for operation in (1, 1, 20, 20):
            expected_operations.remove(operation)

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
        split_key_created = False
        split_key_destroyed: set[bytes] = set()

        def serve() -> None:
            nonlocal handled
            nonlocal split_key_created
            cancel_result_index = 0
            async_lifecycle_poll = False
            async_request_pending = False
            async_pending_poll_count = 0
            async_poll_mode = "none"
            async_repeated_pending_success_runs = 0
            async_process_completed = False
            async_process_route_finished = False
            async_process_standalone_seed = False
            async_query_route_stage = 0
            multipart_hash_exchange_count = 0
            batch_order_placeholder_seen = 0
            async_query_correlations = (
                b"fixture-query-create-completed",
                b"fixture-query-create-pending",
                b"fixture-query-get-pending",
            )
            try:
                while handled < len(expected_operations):
                    accepted, _ = server.accept()
                    with context.wrap_socket(accepted, server_side=True) as stream:
                        stream.settimeout(10)
                        while handled < len(expected_operations):
                            request = read_message(stream)
                            if request is None:
                                break
                            operations = top_level_batch_operations(request)
                            if not operations:
                                raise AssertionError("request has no KMIP Operation field")
                            if (operations == (1,) and
                                    tlv(0x420093, 8, b"process-operation-seed") in request):
                                async_process_standalone_seed = True
                            if operations == (1, 9, 20):
                                raise AssertionError("Check fixture must revoke the created key before destroying it")
                            split_identifiers = (b"fixture-split-part-1", b"fixture-split-part-2")
                            if operations == (40,):
                                if split_key_created and split_key_destroyed == set(split_identifiers):
                                    split_key_created = False
                                    split_key_destroyed.clear()
                                if (split_key_created or integer(0x42008B, 2) not in request or
                                        integer(0x42008C, 2) not in request or
                                        enumeration(0x42008A, 1) not in request or
                                        enumeration(0x420028, 3) not in request or
                                        integer(0x42002A, 128) not in request):
                                    raise AssertionError("Create Split Key request did not match the 2-of-2 AES fixture")
                                split_key_created = True
                            if operations == (20,) and any(
                                    tlv(0x420094, 7, identifier) in request
                                    for identifier in split_identifiers):
                                identifier = next(identifier for identifier in split_identifiers
                                                  if tlv(0x420094, 7, identifier) in request)
                                if not split_key_created or identifier in split_key_destroyed:
                                    raise AssertionError("Create Split Key cleanup identifier was reused or premature")
                                split_key_destroyed.add(identifier)
                            if operations == (1,) and tlv(0x420093, 8, b"rekey-create") in request:
                                if (enumeration(0x420057, 2) not in request or
                                        enumeration(0x420028, 3) not in request or
                                        integer(0x42002A, 128) not in request or
                                        integer(0x42002C, 4) not in request):
                                    raise AssertionError("Re-key setup must create a typed AES-128 Symmetric Key")
                            if operations == (1,) and tlv(0x420093, 8, b"derive-create") in request:
                                if (enumeration(0x420057, 2) not in request or
                                        enumeration(0x420028, 3) not in request or
                                        integer(0x42002A, 128) not in request or
                                        integer(0x42002C, 512) not in request):
                                    raise AssertionError("Derive Key setup must create an AES source with Derive Key usage")
                            if operations == (5,) and tlv(0x420093, 8, b"derive-target") in request:
                                if (request.count(tlv(0x420094, 7, b"fixture-key-1")) != 1 or
                                        tlv(0x420030, 8, b"synthetic-derivation-data") not in request or
                                        tlv(0x420084, 8, b"synthetic-derivation-salt") not in request or
                                        integer(0x42003C, 1000) not in request or
                                        enumeration(0x420038, 6) not in request or
                                        integer(0x42002C, 4) not in request):
                                    raise AssertionError("Derive Key request fields mismatch")
                            if operations == (4,) and tlv(0x420093, 8, b"rekey-target") in request:
                                if request.count(tlv(0x420094, 7, b"fixture-key-1")) != 1:
                                    raise AssertionError("Re-key must target the created source UID")
                            if operations == (42, 20):
                                imported_object = tlv(
                                    0x42005B, 1,
                                    enumeration(0x420059, 0x80000001) +
                                    tlv(0x42005A, 8, b"controlled-import-object"))
                                if (enumeration(0x420057, 8) not in request or
                                        tlv(0x420094, 7, b"fixture-imported-secret-data") not in request or
                                        tlv(0x420125, 1, b"") not in request or
                                        imported_object not in request or
                                        any(tlv(0x420093, 8, item_id) not in request for item_id in (
                                            b"import-object", b"import-destroy"))):
                                    raise AssertionError("Import request did not match the controlled nonsecret Opaque Object fixture")
                            if (operations == (3,) and
                                    bytes.fromhex("42008501") in request):
                                secret_data = (
                                    enumeration(0x420086, 1) +
                                    tlv(0x420040, 1,
                                        enumeration(0x420042, 1) +
                                        tlv(0x420045, 1,
                                            tlv(0x420043, 8, SECRET_DATA_MATERIAL))))
                                if (enumeration(0x420057, 7) not in request or
                                        tlv(0x420085, 1, secret_data) not in request or
                                        tlv(0x420093, 8, b"interop-register") not in request):
                                    raise AssertionError("Secret Data Register request did not match the synthetic Password fixture")
                            if operations == (41,) and tlv(0x420093, 8, b"join-split-target") in request:
                                if (not split_key_created or
                                        request.count(tlv(0x420094, 7, split_identifiers[0])) != 1 or
                                        request.count(tlv(0x420094, 7, split_identifiers[1])) != 1):
                                    raise AssertionError("Join Split Key must reference both split key parts")
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
                            if operations == (10,):
                                request_has_continuation = ttlv_contains_tag(request, 0x42000E)
                                request_has_order = ttlv_contains_tag(request, 0x420011)
                                request_has_id = tlv(0x420093, 8, b"interop-get") in request
                                if request_has_id and (
                                        request.count(integer(0x42000D, 1)) != 1 or
                                        request_has_continuation or request_has_order):
                                    raise AssertionError(
                                        "single-item Get option mismatch "
                                        f"(continuation={request_has_continuation}, "
                                        f"order={request_has_order}, id={request_has_id})"
                                    )
                            placeholder_chain = operations == (1, 18, 10)
                            batch_order_placeholder = (operations == (1, 18) and
                                                       tlv(0x420093, 8, b"ordered-create") in request)
                            placeholder_cleanup = operations == (19, 20) and \
                                tlv(0x420093, 8, b"placeholder-revoke-cleanup") in request
                            response_operations = operations
                            response_item_ids = None
                            if operations == (52, 52, 52):
                                response_item_ids = (b"interop-begin", b"interop-end", b"interop-reset")
                            if batch_order_placeholder:
                                is_required_option_case = batch_order_placeholder_seen == 0
                                expected_item_ids = (b"ordered-create", b"ordered-activate")
                                batch_order_option = tlv(0x420010, 6, b"\x00" * 7 + b"\x01")
                                if (request.count(enumeration(0x420094, 1)) != 1 or
                                        any(request.count(tlv(0x420093, 8, item_id)) != 1
                                            for item_id in expected_item_ids) or
                                        ((batch_order_option in request) != is_required_option_case)):
                                    raise AssertionError(
                                        "Batch Order placeholder request shape or option mismatch"
                                    )
                                batch_order_placeholder_seen += 1
                                response_item_ids = expected_item_ids
                            if operations == (1, 18) and tlv(0x420093, 8, b"derive-create") in request:
                                response_item_ids = (b"derive-create", b"derive-activate")
                            if operations == (5, 19, 20, 20):
                                response_item_ids = (b"derive-target", b"derive-revoke-source",
                                                     b"derive-destroy-derived", b"derive-destroy-source")
                            if operations == registered_key_batch:
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in registered_key_batch_item_ids):
                                    raise AssertionError(
                                        "registered-key batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = registered_key_batch_item_ids
                            if operations == (1, 18, 31, 19, 20):
                                expected_encrypt_item_ids = (
                                    b"encrypt-create", b"encrypt-activate", b"encrypt-target",
                                    b"encrypt-revoke", b"encrypt-destroy",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_encrypt_item_ids):
                                    raise AssertionError(
                                        "isolated Encrypt batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_encrypt_item_ids
                            if operations == (1, 31, 19, 20):
                                expected_preactive_encrypt_item_ids = (
                                    b"preactive-encrypt-create", b"preactive-encrypt-target",
                                    b"preactive-encrypt-revoke", b"preactive-encrypt-destroy",
                                )
                                if (any(request.count(tlv(0x420093, 8, item_id)) != 1
                                        for item_id in expected_preactive_encrypt_item_ids) or
                                        integer(0x42000D, 4) not in request or
                                        enumeration(0x42000E, 1) not in request):
                                    raise AssertionError(
                                        "Pre-Active Encrypt batch shape or item IDs mismatch"
                                    )
                                response_item_ids = expected_preactive_encrypt_item_ids
                            if operations == (1, 18, 32, 19, 20):
                                expected_decrypt_item_ids = (
                                    b"decrypt-create", b"decrypt-activate", b"decrypt-target",
                                    b"decrypt-revoke", b"decrypt-destroy",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_decrypt_item_ids):
                                    raise AssertionError(
                                        "isolated Decrypt batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_decrypt_item_ids
                            if operations == (1, 18, 35, 19, 20):
                                expected_mac_item_ids = (
                                    b"mac-create", b"mac-activate", b"mac-target",
                                    b"mac-revoke", b"mac-destroy",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_mac_item_ids):
                                    raise AssertionError(
                                        "isolated MAC batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_mac_item_ids
                            if operations == (1, 18, 36, 19, 20):
                                expected_mac_verify_item_ids = (
                                    b"mac-verify-create", b"mac-verify-activate", b"mac-verify-target",
                                    b"mac-verify-revoke", b"mac-verify-destroy",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_mac_verify_item_ids):
                                    raise AssertionError(
                                        "isolated MAC Verify batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_mac_verify_item_ids
                            if operations == (1, 21, 19, 20):
                                expected_archive_item_ids = (
                                    b"archive-create", b"archive-target", b"archive-revoke",
                                    b"archive-destroy",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_archive_item_ids):
                                    raise AssertionError(
                                        "isolated Archive batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_archive_item_ids
                            if operations == (1, 21, 22, 20):
                                expected_recover_item_ids = (
                                    b"recover-create", b"recover-archive",
                                    b"recover-target", b"recover-destroy",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_recover_item_ids):
                                    raise AssertionError(
                                        "isolated Recover batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_recover_item_ids
                            if operations in ((2, 18, 33, 19, 20, 20), (2, 18, 34, 19, 20, 20)):
                                expected_signature_item_ids = (
                                    (b"sign-create-pair", b"sign-activate", b"sign-target",
                                     b"sign-revoke", b"sign-destroy-private", b"sign-destroy-public")
                                    if operations[2] == 33 else
                                    (b"verify-create-pair", b"verify-activate", b"verify-target",
                                     b"verify-revoke", b"verify-destroy-public", b"verify-destroy-private")
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_signature_item_ids):
                                    raise AssertionError(
                                        "signature key-pair batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_signature_item_ids
                            if operations == (2, 20, 20):
                                expected_pair_item_ids = (
                                    b"create-pair", b"destroy-private", b"destroy-public"
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_pair_item_ids):
                                    raise AssertionError(
                                        "Create Key Pair batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_pair_item_ids
                            if operations == (2, 18, 6, 19, 20, 20, 20):
                                expected_certify_item_ids = (
                                    b"certify-create-pair", b"certify-activate-private",
                                    b"certify-public-key", b"certify-revoke-private",
                                    b"certify-destroy-certificate", b"certify-destroy-private",
                                    b"certify-destroy-public",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_certify_item_ids):
                                    raise AssertionError(
                                        "Certify batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_certify_item_ids
                            if operations == (2, 18, 6, 7, 19, 20, 20, 20, 20):
                                expected_recertify_item_ids = (
                                    b"recertify-create-pair", b"recertify-activate-private",
                                    b"recertify-public-key", b"recertify-renew-certificate",
                                    b"recertify-revoke-private", b"recertify-destroy-new-certificate",
                                    b"recertify-destroy-old-certificate",
                                    b"recertify-destroy-private", b"recertify-destroy-public",
                                )
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_recertify_item_ids):
                                    raise AssertionError(
                                        "Re-certify batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_recertify_item_ids
                            if operations == (23,):
                                expected_validate_item_ids = (b"validate-certificate-chain",)
                                if any(request.count(tlv(0x420093, 8, item_id)) != 1
                                       for item_id in expected_validate_item_ids):
                                    raise AssertionError(
                                        "Validate batch omitted or duplicated a unique item ID"
                                    )
                                response_item_ids = expected_validate_item_ids
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
                            if operations == (1, 10):
                                if (request.count(integer(0x42000D, 2)) != 1 or
                                        request.count(tlv(0x420093, 8, b"undo-create")) != 1 or
                                        request.count(tlv(0x420093, 8, b"undo-get")) != 1 or
                                        request.count(enumeration(0x42000E, 3)) != 1 or
                                        request.count(tlv(0x420010, 6, b"\x00" * 7 + b"\x01")) != 1 or
                                        request.count(tlv(0x420094, 7, b"missing-key")) != 1 or
                                        request.count(enumeration(0x420057, 2)) != 1):
                                    raise AssertionError("Undo rollback batch fields mismatch")
                                response_item_ids = (b"undo-create", b"undo-get")
                                response_statuses = (3, 1)
                                response_reasons = (None, 55)
                            if two_read_batch:
                                continue_after_failure = tlv(0x420094, 7, b"missing-key") in request
                                expected_fixture_identifiers = 1 if continue_after_failure else 2
                                if (request.count(bytes((0x42, 0x00, 0x0D))) != 1 or
                                        request.count(integer(0x42000D, 2)) != 1 or
                                        len(operations) != 2 or
                                        request.count(tlv(0x420094, 7, b"fixture-key-1")) !=
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
                            elif operations not in (stop_on_first_failure_batch, (1, 10)):
                                response_statuses = None
                                response_reasons = None
                            if operations == (31,) and tlv(0x420093, 8, b"preactive-encrypt-target") in request:
                                response_statuses = (1,)
                                response_reasons = (10,)
                            if operations == (20,) and tlv(0x420093, 8, b"interop-destroy-active-target") in request:
                                response_statuses = (1,)
                                response_reasons = (67,)
                            if (operations == (33,) and tlv(0x420093, 8, b"sign-target") in request and
                                    not ttlv_contains_tag(request, 0x42002B)):
                                response_statuses = (1,)
                                response_reasons = (36,)
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
                            export_lifecycle = operations == (1, 43, 19, 20)
                            archive_lifecycle = operations == (1, 21, 19, 20)
                            recover_lifecycle = operations == (1, 21, 22, 20)
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
                            if export_lifecycle:
                                lifecycle_target = {43: b"interop-export"}
                            if archive_lifecycle:
                                lifecycle_target = {21: b"archive-target"}
                            if recover_lifecycle:
                                lifecycle_target = {21: b"recover-archive", 22: b"recover-target"}
                            if create_lifecycle:
                                lifecycle_target = {1: b"interop-create"}
                            lifecycle_ids = (b"interop-create", *lifecycle_target.values(),
                                             *((b"interop-revoke",) if (check_lifecycle or activate_lifecycle or
                                                 get_constraints_lifecycle or get_usage_allocation_lifecycle or
                                                 export_lifecycle) else ()),
                                             *((b"interop-revoke-cleanup", b"interop-destroy-cleanup")
                                               if destroy_lifecycle else ()),
                                             *((b"interop-destroy",) if not destroy_lifecycle else ()))
                            if illegal_destroy_lifecycle:
                                lifecycle_ids = (
                                    b"interop-create", b"interop-activate-before-destroy",
                                    b"interop-destroy-active-target", b"interop-revoke-cleanup",
                                    b"interop-destroy-cleanup"
                                )
                            if archive_lifecycle:
                                lifecycle_ids = (
                                    b"archive-create", b"archive-target", b"archive-revoke",
                                    b"archive-destroy"
                                )
                            if recover_lifecycle:
                                lifecycle_ids = (
                                    b"recover-create", b"recover-archive",
                                    b"recover-target", b"recover-destroy"
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
                            attr_ref_tags = [
                                int.from_bytes(val, "big")
                                for val in extract_tlvs(request, 0x42013B, 5)
                                if len(val) == 4
                            ]
                            selected_attribute_tag = next(
                                (tag for tag in (0x420028, 0x42002A) if tag in attr_ref_tags), 0
                            )
                            if create_lifecycle and request.count(enumeration(0x420094, 1)) < 1:
                                raise AssertionError("Create cleanup must Destroy through ID Placeholder")
                            if operations in ((1, 9, 19, 20), (1, 18, 19, 20), (1, 19, 20),
                                              (1, 20, 19, 20), (1, 56, 19, 20), (1, 17, 19, 20),
                                              (1, 43, 19, 20), (1, 10, 20),
                                              (1, 21, 19, 20), (1, 21, 22, 20)) and request.count(
                                    enumeration(0x420094, 1)) < (
                                        3 if (archive_lifecycle or recover_lifecycle) else
                                        3 if (check_lifecycle or activate_lifecycle or destroy_lifecycle or
                                              export_lifecycle) else 2):
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
                            if operations == (48,) and (
                                    request.count(tlv(0x420094, 7, b"fixture-key-1")) != 1 or
                                    request.count(tlv(0x42009D, 7, b"KMIPKit")) != 1 or
                                    request.count(tlv(0x42000A, 7, b"Counter")) != 1 or
                                    enumeration(0x420158, 1) not in request or
                                    integer(0x420162, 5) not in request):
                                raise AssertionError(
                                    "Adjust Attribute request did not target the fixture Counter with Increment 5"
                                )
                            if integer(0x42006A, 2) not in request or \
                                    integer(0x42006B, protocol_minor) not in request:
                                raise AssertionError(
                                    "request header did not preserve the configured KMIP version"
                                )
                            # Dedicated asynchronous C executors have no authentication input;
                            # every ordinary batch in this peer still proves configured credentials.
                            configured_credentials_present = (
                                tlv(0x420099, 7, b"interop-user") in request and
                                tlv(0x4200A1, 7, b"interop-password") in request
                            )
                            if operations == (39,) and not configured_credentials_present:
                                raise AssertionError(
                                    "Hash request is missing configured KMIP credentials"
                                )
                            if operations not in ((25,), (26,), (57,), (58,)) and not configured_credentials_present:
                                raise AssertionError("request is missing configured KMIP credentials")
                            if (async_query_route_stage == 0 and async_process_route_finished and
                                    operations == (1,) and enumeration(0x420007, 1) in request):
                                async_query_route_stage = 1
                                stream.sendall(pending_operation_response(
                                    protocol_minor, 1, async_query_correlations[0]))
                            elif async_query_route_stage == 1:
                                if operations != (1,) or enumeration(0x420007, 1) not in request:
                                    raise AssertionError("Query filter recipe expected its second pending Create")
                                async_query_route_stage = 2
                                stream.sendall(pending_operation_response(
                                    protocol_minor, 1, async_query_correlations[1]))
                            elif async_query_route_stage == 2:
                                if (operations != (10,) or enumeration(0x420007, 1) not in request or
                                        tlv(0x420094, 7, b"fixture-key-1") not in request):
                                    raise AssertionError("Query filter recipe expected its pending Get seed")
                                async_query_route_stage = 3
                                stream.sendall(pending_operation_response(
                                    protocol_minor, 10, async_query_correlations[2]))
                            elif async_query_route_stage == 3:
                                if (operations != (26,) or tlv(0x420006, 8,
                                        async_query_correlations[0]) not in request):
                                    raise AssertionError("Query filter recipe Poll did not target completed Create")
                                async_query_route_stage = 4
                                stream.sendall(completed_poll_create_response(
                                    protocol_minor, b"fixture-key-2"))
                            elif async_query_route_stage == 4:
                                required_filters = tlv(
                                    0x420176, 1,
                                    b"".join(tlv(0x420006, 8, value)
                                             for value in async_query_correlations))
                                operation_filter = tlv(
                                    0x42014F, 1, enumeration(0x42005C, 1))
                                if (operations != (57,) or required_filters not in request or
                                        operation_filter not in request):
                                    raise AssertionError("Query filter recipe omitted either exact filter")
                                async_query_route_stage = 5
                                stream.sendall(async_query_outstanding_response(
                                    protocol_minor, async_query_correlations[1]))
                            elif async_query_route_stage == 5:
                                if (operations != (26,) or tlv(0x420006, 8,
                                        async_query_correlations[1]) not in request):
                                    raise AssertionError("Query filter recipe Poll did not target outstanding Create")
                                async_query_route_stage = 6
                                stream.sendall(completed_poll_create_response(
                                    protocol_minor, b"fixture-key-3"))
                            elif async_query_route_stage == 6:
                                if (operations != (26,) or tlv(0x420006, 8,
                                        async_query_correlations[2]) not in request):
                                    raise AssertionError("Query filter recipe Poll did not target Get")
                                async_query_route_stage = 7
                                stream.sendall(failed_poll_create_response(
                                    protocol_minor, b"controlled Get failure"))
                            elif async_query_route_stage in (7, 8):
                                identifier = b"fixture-key-2" if async_query_route_stage == 7 else b"fixture-key-3"
                                if operations != (20,) or tlv(0x420094, 7, identifier) not in request:
                                    raise AssertionError("Query filter recipe Destroy missed its Create result")
                                response_item_ids = (None,)
                                async_query_route_stage += 1
                                destroy_response = response(
                                    (20,), protocol_minor,
                                    unique_batch_item_ids=response_item_ids).replace(
                                        tlv(0x420094, 7, b"fixture-key-1"),
                                        tlv(0x420094, 7, identifier), 1)
                                stream.sendall(destroy_response)
                                if async_query_route_stage == 9:
                                    async_query_route_stage = 0
                            elif operations == (1,) and enumeration(0x420007, 1) in request:
                                cancel_correlation = b"fixture-cancel-correlation"
                                async_request_pending = True
                                async_pending_poll_count = 0
                                async_poll_mode = (
                                    "repeated_pending_success"
                                    if async_repeated_pending_success_runs < 2 else
                                    "final_failure")
                                stream.sendall(pending_create_response(
                                    protocol_minor, cancel_correlation,
                                    b"process-operation-seed" if async_process_standalone_seed else None))
                            elif operations == (57,):
                                cancel_correlation = b"fixture-cancel-correlation"
                                if request.count(tlv(0x420006, 8, cancel_correlation)) == 1:
                                    async_lifecycle_poll = True
                                    stream.sendall(query_async_requests_response(
                                        protocol_minor, cancel_correlation))
                                else:
                                    if (bytes.fromhex("420176") in request or
                                            bytes.fromhex("42014F") in request or
                                            bytes.fromhex("420006") in request):
                                        raise AssertionError(
                                            "empty Query Async Requests request included a filter"
                                        )
                                    stream.sendall(response((57,), protocol_minor))
                            elif operations == (58,):
                                cancel_correlation = b"fixture-cancel-correlation"
                                if (request.count(tlv(0x420006, 8, cancel_correlation)) != 1 or
                                        enumeration(0x420007, 3) not in request):
                                    raise AssertionError(
                                        "Process must preserve the pending correlation and use the prohibited indicator"
                                    )
                                async_process_completed = True
                                stream.sendall(response((58,), protocol_minor))
                            elif operations == (26,):
                                unknown_correlation = b"unknown-unissued-value"
                                if tlv(0x420006, 8, unknown_correlation) in request:
                                    if request.count(tlv(0x420006, 8, unknown_correlation)) != 1:
                                        raise AssertionError(
                                            "unknown-correlation Poll must carry one exact correlation value"
                                        )
                                    stream.sendall(response(
                                        (26,), protocol_minor,
                                        result_statuses=(1,), result_reasons=(43,)))
                                    handled += len(operations)
                                    handled_operations.extend(operations)
                                    handled_batches.append(operations)
                                    continue
                                cancel_correlation = b"fixture-cancel-correlation"
                                if request.count(tlv(0x420006, 8, cancel_correlation)) != 1:
                                    raise AssertionError("Poll request omitted its exact pending correlation value")
                                if async_process_completed:
                                    async_request_pending = False
                                    async_poll_mode = "none"
                                    stream.sendall(completed_poll_create_response(protocol_minor))
                                elif async_lifecycle_poll:
                                    stream.sendall(completed_poll_create_response(protocol_minor))
                                elif async_request_pending:
                                    async_pending_poll_count += 1
                                    if (async_poll_mode == "repeated_pending_success" and
                                            async_pending_poll_count == 3):
                                        stream.sendall(completed_poll_create_response(protocol_minor))
                                        async_request_pending = False
                                        async_repeated_pending_success_runs += 1
                                    elif async_pending_poll_count == 1 or (
                                            async_poll_mode == "repeated_pending_success" and
                                            async_pending_poll_count == 2):
                                        stream.sendall(pending_poll_response(protocol_minor, cancel_correlation))
                                    else:
                                        stream.sendall(failed_poll_create_response(protocol_minor))
                                        async_request_pending = False
                                else:
                                    stream.sendall(completed_poll_create_response(protocol_minor))
                            elif (operations == (19, 20) and async_process_completed and
                                  async_process_standalone_seed):
                                async_request_pending = False
                                async_poll_mode = "none"
                                async_process_completed = False
                                async_process_standalone_seed = False
                                stream.sendall(response(
                                    (19, 20), protocol_minor,
                                    unique_batch_item_ids=(None, None)))
                            elif (operations == (19,) and async_process_completed and
                                  async_process_standalone_seed):
                                stream.sendall(response(
                                    (19,), protocol_minor,
                                    unique_batch_item_ids=(b"process-operation-revoke",)))
                            else:
                                req_ids = extract_tlvs(request, 0x420093, 8)
                                req_uids = extract_tlvs(request, 0x420094, 7)
                                if operations == (20,) and async_lifecycle_poll:
                                    response_item_ids = (None,)
                                    async_lifecycle_poll = False
                                elif operations == (20,) and async_poll_mode == "repeated_pending_success":
                                    response_item_ids = (None,)
                                elif operations == (20,) and async_process_completed:
                                    response_item_ids = (
                                        b"process-operation-destroy" if async_process_standalone_seed else None,)
                                    if not async_process_standalone_seed:
                                        async_process_route_finished = True
                                    async_process_completed = False
                                    async_process_standalone_seed = False
                                elif operations == (42, 20):
                                    response_item_ids = (b"import-object", b"import-destroy")
                                elif response_item_ids is None and len(operations) == 1:
                                    response_item_ids = (req_ids[0],) if req_ids else (None,)
                                cancellation_result = 1
                                if operations == (25,) and async_request_pending:
                                    if cancel_result_index >= len(cancel_result_values):
                                        raise AssertionError("peer received an unplanned Cancel request")
                                    cancellation_result = cancel_result_values[cancel_result_index]
                                    cancel_result_index += 1
                                if operations == (25,) and request.count(
                                        tlv(0x420006, 8, b"fixture-cancel-correlation")) != 1:
                                    raise AssertionError("Cancel request omitted its exact pending correlation value")
                                multipart_hash = operations == (39,) and any(
                                    marker in request for marker in (
                                        tlv(0x4200D6, 8, b"fixture-hash-correlation"),
                                        tlv(0x4200D7, 6, b"\x00" * 7 + b"\x01"),
                                        tlv(0x4200D8, 6, b"\x00" * 7 + b"\x01"),
                                    )
                                )
                                hash_correlation_value = (
                                    b"fixture-hash-correlation"
                                    if multipart_hash and multipart_hash_exchange_count == 0 else None
                                )
                                returned_uid = (
                                    b"fixture-imported-secret-data" if operations == (42, 20)
                                    else req_uids[0] if (
                                        len(operations) == 1 and
                                        operations[0] not in (4, 5, 6, 7, 41, 42) and
                                        req_uids
                                    ) else None
                                )
                                stream.sendall(response(
                                    response_operations, protocol_minor, selected_attribute_tag,
                                    4 if private_key_create else 3 if public_key_create else 2,
                                    unique_batch_item_ids=response_item_ids,
                                    result_statuses=response_statuses,
                                    result_reasons=response_reasons,
                                    rng_length=32 if integer(0x4200C4, 32) in request or
                                    tlv(0x4200C2, 8, bytes(range(32))) in request else 16,
                                    cancellation_result=cancellation_result,
                                    returned_unique_identifier=returned_uid,
                                    crypto_output=crypto_output_for_request(operations, request),
                                    multipart_hash=multipart_hash,
                                    final_hash=multipart_hash and tlv(0x4200D8, 6, b"\x00" * 7 + b"\x01") in request,
                                    hash_correlation_value=hash_correlation_value))
                                if multipart_hash:
                                    multipart_hash_exchange_count += 1
                                if operations == (25,):
                                    async_request_pending = False
                                    async_poll_mode = "none"
                                elif operations == (20,) and async_lifecycle_poll:
                                    async_request_pending = False
                                elif operations == (20,) and async_poll_mode == "repeated_pending_success":
                                    async_poll_mode = "none"
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
        if run.returncode != 0:
            result = json.loads(result_path.read_text(encoding="utf-8")) if result_path.is_file() else {}
            failed_cases = [
                (case.get("case_id"), case.get("status"), case.get("diagnostic"))
                for case in result.get("cases", []) if case.get("status") == "failed"
            ]
            raise AssertionError(
                f"C validator exited {run.returncode}; failed_cases={failed_cases!r}; "
                f"batches={handled_batches!r}; stdout={run.stdout[:1200]!r}; stderr={run.stderr[-1200:]!r}"
            )
        if (handled != len(expected_operations) or sorted(handled_operations) != sorted(expected_operations)
                or (11,) not in handled_batches
                or (12,) not in handled_batches
                or (10,) not in handled_batches
                or registered_key_batch not in handled_batches
                or not split_key_created or split_key_destroyed != {b"fixture-split-part-1", b"fixture-split-part-2"}):
            raise AssertionError(
                f"controlled peer observed {handled} operation calls (expected {len(expected_operations)}); "
                f"Hash calls={handled_operations.count(39)} (expected {expected_operations.count(39)}); "
                f"batch_count={len(handled_batches)}"
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
            process_boundary = next(
                case for case in result["cases"]
                if case["case_id"] == "test-v20-reject-process"
            )
            if process_boundary["status"] != "passed" or \
                    process_boundary["server_compatibility"] != "unknown" or \
                    process_boundary["library_correctness"] != "passed":
                raise AssertionError(
                    "KMIP 2.1-only Process must be rejected before transport under KMIP 2.0: "
                    f"{process_boundary!r}"
                )
            set_defaults_boundary = next(
                case for case in result["cases"]
                if case["case_id"] == "test-v20-reject-set-defaults"
            )
            if set_defaults_boundary["status"] != "passed" or \
                    set_defaults_boundary["server_compatibility"] != "unknown" or \
                    set_defaults_boundary["library_correctness"] != "passed":
                raise AssertionError(
                    "KMIP 2.1-only Set Defaults must be rejected before transport under KMIP 2.0: "
                    f"{set_defaults_boundary!r}"
                )
            set_constraints_boundary = next(
                case for case in result["cases"]
                if case["case_id"] == "test-v20-reject-set-constraints"
            )
            if set_constraints_boundary["status"] != "passed" or \
                    set_constraints_boundary["server_compatibility"] != "unknown" or \
                    set_constraints_boundary["library_correctness"] != "passed":
                raise AssertionError(
                    "KMIP 2.1-only Set Constraints must be rejected before transport under KMIP 2.0: "
                    f"{set_constraints_boundary!r}"
                )
            query_async_boundary = next(
                case for case in result["cases"]
                if case["case_id"] == "test-v20-reject-query-asynchronous-requests"
            )
            if query_async_boundary["status"] != "passed" or \
                    query_async_boundary["server_compatibility"] != "unknown" or \
                    query_async_boundary["library_correctness"] != "passed":
                raise AssertionError(
                    "KMIP 2.1-only Query Asynchronous Requests must be rejected before transport under KMIP 2.0: "
                    f"{query_async_boundary!r}"
                )
            discovery = next(case for case in result["cases"]
                             if case["case_id"] == "test-op-discover-versions")
            if discovery["status"] != "not_run" or "cannot construct a KMIP 2.0 offer" not in discovery.get("diagnostic", ""):
                raise AssertionError(
                    "KMIP 2.0 Discover Versions must report the documented C API offer limitation"
                )
            login = next(case for case in result["cases"] if case["case_id"] == "test-op-login")
            if login["status"] != "passed":
                raise AssertionError("KMIP 2.0 Login must execute its typed response route")
            logout = next(case for case in result["cases"] if case["case_id"] == "test-op-logout")
            if logout["status"] != "passed":
                raise AssertionError("KMIP 2.0 Logout must use its server-issued Ticket")
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
                f"actual={result['summary']!r}; expected={expected_summary!r}"
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
                f"actual={result['summary']!r}; expected={expected_kmip20_summary!r}"
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
    verify_extended_operations(binary, openssl, repository, "generic-ttlv")
    verify_extended_operations(binary, openssl, repository, "typed-protocol")
    return 0


def verify_extended_operations(binary, openssl, repository, api_level):
    extended_cases = [
        "test-op-create-key-pair", "test-op-register", "test-op-re-key", "test-op-derive-key",
        "test-op-certify", "test-op-re-certify", "test-op-get-usage-allocation", "test-op-validate",
        "test-op-cancel", "test-op-poll", "test-op-re-key-key-pair", "test-crypto-encrypt-aes-256-cbc",
        "test-crypto-decrypt-aes-256-cbc", "test-crypto-sign-rsa-2048-pkcs1_v1_5", "test-crypto-signature-verify-rsa-2048-pkcs1_v1_5", "test-crypto-mac-hmac_sha256-256",
        "test-crypto-mac-verify-hmac_sha256-256", "test-op-create-split-key", "test-op-join-split-key",
        "test-op-import", "test-op-export",
    ]
    if api_level == "typed-protocol":
        extended_cases = ["test-op-poll", "test-op-cancel"]
    catalog = json.loads((repository / "verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
    cases_by_id = {c["case_id"]: c for c in catalog["test_cases"]}
    sym_key = tlv(0x42008f, 1, tlv(0x420040, 1, enumeration(0x420042, 1)
                        + tlv(0x420045, 1, tlv(0x420043, 8, bytes(16)))))
    for version in ("2.0", "2.1"):
        minor = int(version[-1])
        header = tlv(0x42007a, 1, tlv(0x420069, 1, integer(0x42006a, 2) + integer(0x42006b, minor))
                     + tlv(0x420092, 9, (1).to_bytes(8, "big")) + integer(0x42000d, 1))
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            ca, server_cert, server_key, client_cert, client_key = certificate_files(openssl, directory)
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(server_cert, server_key)
            context.load_verify_locations(ca)
            context.verify_mode = ssl.CERT_REQUIRED
            for case_id in extended_cases:
                with socket.socket() as listener:
                    listener.bind(("127.0.0.1", 0))
                    listener.listen()
                    listener.settimeout(0.05)
                    connection = {
                        "server_id": "extended-generic-peer", "server_configuration_fingerprint": "b" * 64,
                        "server_product": "controlled peer", "server_version": "fixture",
                        "protocol_version": version, "transport": "raw_tls", "encoding": "TTLV",
                        "api_level": api_level,
                        "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": listener.getsockname()[1]}},
                        "tls": {"ca_cert_path": str(ca), "client_cert_path": str(client_cert),
                                "client_key_path": str(client_key), "server_name_override": "localhost"},
                        "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
                        "execution_policy": {"allow_mutations": True, "probe_unsupported": True},
                    }
                    config = directory / "connection.json"
                    selection = directory / "catalog.json"
                    output = directory / f"{case_id}.json"
                    config.write_text(json.dumps(connection))
                    selection.write_text(json.dumps({**catalog, "test_cases": [cases_by_id[case_id]],
                                                      "statistics": {**catalog["statistics"], "total_cases": 1}}))
                    command = [str(binary), "run", "--connection", str(config), "--catalog", str(selection),
                               "--output", str(output), "--allow-mutations"]
                    observed_operations = []
                    initial = b"c-peer-initial-token"
                    updated = b"c-peer-updated-token"
                    asynchronous = case_id in {"test-op-poll", "test-op-cancel"}
                    with subprocess.Popen(command, cwd=repository, stdout=subprocess.DEVNULL,
                                          stderr=subprocess.PIPE) as process:
                        try:
                            while process.poll() is None:
                                try:
                                    raw, _ = listener.accept()
                                except socket.timeout:
                                    continue
                                with context.wrap_socket(raw, server_side=True) as stream:
                                    stream.settimeout(5)
                                    request = read_message(stream)
                                    op = top_level_batch_operations(request)[0]
                                    observed_operations.append(op)
                                    if asynchronous and op == 1:
                                        assert enumeration(0x420007, 1) in request
                                        batch = tlv(0x42000f, 1, enumeration(0x42005c, op)
                                                    + enumeration(0x42007f, 2) + tlv(0x420006, 8, initial)
                                                    + tlv(0x42007c, 1, b""))
                                        stream.sendall(tlv(0x42007b, 1, header + batch))
                                        continue
                                    if op == 1:
                                        fields = enumeration(0x420057, 2) + tlv(0x420094, 7, b"owned-key")
                                    elif op == 2:
                                        fields = tlv(0x420066, 7, b"priv-key") + tlv(0x42006f, 7, b"pub-key")
                                    elif op in (3, 0x2a):
                                        fields = tlv(0x420094, 7, b"reg-key")
                                    elif op == 4:
                                        fields = tlv(0x420094, 7, b"rekey-key")
                                    elif op == 5:
                                        fields = tlv(0x420094, 7, b"derived-key")
                                    elif op == 6:
                                        assert tlv(0x420079, 1, tlv(0x420094, 7, b"pub-key")
                                                   + tlv(0x420125, 1, enumeration(0x42001d, 1))) in request
                                        fields = tlv(0x420094, 7, b"cert-id")
                                    elif op == 7:
                                        fields = tlv(0x420094, 7, b"recert-id")
                                    elif op in (0x17, 0x22, 0x24):
                                        fields = enumeration(0x42009b, 1)
                                    elif op == 0x19:
                                        assert tlv(0x420006, 8, initial) in request
                                        fields = tlv(0x420006, 8, initial) + enumeration(0x420012, 1)
                                    elif op == 0x1a:
                                        if tlv(0x420006, 8, initial) in request:
                                            batch = tlv(0x42000f, 1, enumeration(0x42005c, op)
                                                        + enumeration(0x42007f, 2) + tlv(0x420006, 8, updated))
                                            stream.sendall(tlv(0x42007b, 1, header + batch))
                                            continue
                                        assert tlv(0x420006, 8, updated) in request
                                        fields = enumeration(0x420057, 2) + tlv(0x420094, 7, b"owned-key")
                                    elif op == 0x1d:
                                        fields = tlv(0x420066, 7, b"new-priv") + tlv(0x42006f, 7, b"new-pub")
                                    elif op == 0x1f:
                                        fields = tlv(0x420094, 7, b"owned-key") + tlv(0x4200c2, 8, b"cipher")
                                    elif op == 0x20:
                                        fields = tlv(0x420094, 7, b"owned-key") + tlv(0x4200c2, 8, b"0123456789abcdef")
                                    elif op == 0x21:
                                        fields = tlv(0x420094, 7, b"priv-key") + tlv(0x4200c3, 8, b"sig")
                                    elif op == 0x23:
                                        fields = tlv(0x420094, 7, b"owned-key") + tlv(0x4200c6, 8, b"mac")
                                    elif op == 0x28:
                                        fields = tlv(0x420094, 7, b"split-1") + tlv(0x420094, 7, b"split-2")
                                    elif op == 0x29:
                                        fields = tlv(0x420094, 7, b"joined-key")
                                    elif op == 0x2b:
                                        fields = enumeration(0x420057, 2) + tlv(0x420094, 7, b"owned-key") + sym_key
                                    else:
                                        fields = tlv(0x420094, 7, b"owned-key")
                                    batch = tlv(0x42000f, 1, enumeration(0x42005c, op)
                                                + enumeration(0x42007f, 0) + tlv(0x42007c, 1, fields))
                                    stream.sendall(tlv(0x42007b, 1, header + batch))
                        finally:
                            _, stderr = process.communicate(timeout=10)
                        if process.returncode != 0:
                            raise AssertionError(f"C {api_level} extended {version} {case_id} failed: {stderr.decode()}")
                    report = json.loads(output.read_text())
                    if case_id == "test-op-cancel":
                        assert observed_operations == [1, 0x19], observed_operations
                    if case_id == "test-op-poll":
                        assert observed_operations == [1, 0x1a, 0x1a, 0x14], observed_operations
                    if report["cases"][0]["status"] != "passed":
                        raise AssertionError(f"C {api_level} extended {version} {case_id} status={report['cases'][0]}")


if __name__ == "__main__":
    raise SystemExit(main())
