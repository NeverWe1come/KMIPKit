"""Controlled C fixtures for each supported wire encoding and transport.

This peer parses KMIP trees and checks recipe fields before responding. It
shares response fixtures with the established C ABI tests and never persists
request bodies, credentials, tickets, or key bytes.
"""
from __future__ import annotations

import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import select
import socket
import ssl
import subprocess
import sys
import tempfile
import threading
import xml.etree.ElementTree as ET

from artifact_support import INSTALL, PIN, ROOT, sha256
from route_evidence import cell_fields
from route_attributes import AttributeCases
sys.path.insert(0, str(ROOT / 'interop/c-kmip-validator/tests'))
from controlled_kms_test import certificate_files, response, tlv
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import _key

TYPES = ('', 'Structure', 'Integer', 'LongInteger', 'BigInteger', 'Enumeration',
         'Boolean', 'TextString', 'ByteString', 'DateTime', 'Interval', 'DateTimeExtended')

BATCH_ITEM_TAG = 0x42000F
REQUEST_PAYLOAD_TAG = 0x420079
RESPONSE_PAYLOAD_TAG = 0x42007C
UNIQUE_IDENTIFIER_TAG = 0x420094
ENUMERATION_TTLV_TYPE = 5
ID_PLACEHOLDER_ENUMERATION = 1
PLACEHOLDER_CHAIN_OPERATIONS = (1, 18, 10)
PLACEHOLDER_CLEANUP_OPERATIONS = (19, 20)
BATCH_ORDER_PLACEHOLDER_ITEM_IDS = (b'ordered-create', b'ordered-activate')
BATCH_CRYPTO_OPERATIONS = (3, 18, 31, 32, 19, 20)
BATCH_CRYPTO_ITEM_IDS = (b'crypto-register', b'crypto-activate', b'crypto-encrypt',
                         b'crypto-decrypt', b'crypto-revoke', b'crypto-destroy')
BATCH_CRYPTO_KEY = bytes(range(16))
BATCH_CRYPTO_PLAINTEXT = bytes.fromhex('00112233445566778899aabbccddeeff')
BATCH_CRYPTO_CIPHERTEXT = bytes.fromhex('69c4e0d86a7b0430d8cdb78070b4c55a')
BATCH_CONTINUATION_CONSTRUCTOR_CASES = frozenset({
    'test-batch-continue-after-failure',
    'test-batch-stop-on-first-failure',
})
ENCRYPT_ITEM_IDS = (b'encrypt-create', b'encrypt-activate', b'encrypt-target',
                    b'encrypt-revoke', b'encrypt-destroy')
DECRYPT_ITEM_IDS = (b'decrypt-create', b'decrypt-activate', b'decrypt-target',
                    b'decrypt-revoke', b'decrypt-destroy')
MAC_ITEM_IDS = (b'mac-create', b'mac-activate', b'mac-target', b'mac-revoke', b'mac-destroy')
MAC_VERIFY_ITEM_IDS = (b'mac-verify-create', b'mac-verify-activate', b'mac-verify-target',
                       b'mac-verify-revoke', b'mac-verify-destroy')
SIGN_ITEM_IDS = (b'sign-create-pair', b'sign-activate', b'sign-target', b'sign-revoke',
                 b'sign-destroy-private', b'sign-destroy-public')
SIGNATURE_VERIFY_ITEM_IDS = (b'verify-create-pair', b'verify-activate', b'verify-target',
                             b'verify-revoke', b'verify-destroy-public', b'verify-destroy-private')
PKCS11_ITEM_IDS = (b'pkcs11-create', b'pkcs11-activate', b'pkcs11-target',
                   b'pkcs11-revoke', b'pkcs11-destroy')
OBTAIN_LEASE_ITEM_IDS = (b'lease-create', b'lease-target', b'lease-revoke', b'lease-destroy')
PKCS11_REQUEST_CORRELATION = b'pkcs11-request-correlation'
PKCS11_RESPONSE_CORRELATION = b'pkcs11-response-correlation'
PKCS11_INPUT = b'controlled-pkcs11-input'
PKCS11_OUTPUT = b'controlled-pkcs11-output'
CREATE_KEY_PAIR_ITEM_IDS = (b'create-pair', b'destroy-private', b'destroy-public')
CREATE_SPLIT_KEY_ITEM_IDS = (b'split-create', b'split-destroy-part-1', b'split-destroy-part-2')
CREATE_SPLIT_KEY_IDENTIFIERS = (b'fixture-split-part-1', b'fixture-split-part-2')
JOIN_SPLIT_KEY_ITEM_IDS = (b'join-split-create', b'join-split-target',
                           b'join-split-destroy-joined', b'join-split-destroy-part-1',
                           b'join-split-destroy-part-2')
REKEY_ITEM_IDS = (b'rekey-create', b'rekey-target', b'rekey-revoke-replacement',
                  b'rekey-destroy-replacement', b'rekey-revoke-original', b'rekey-destroy-original')
REKEY_ORIGINAL_IDENTIFIER = b'fixture-key-1'
REKEY_REPLACEMENT_IDENTIFIER = b'fixture-rekeyed-key'
DERIVE_KEY_ITEM_IDS = (b'derive-create', b'derive-activate', b'derive-target',
                       b'derive-revoke-source', b'derive-destroy-derived', b'derive-destroy-source')
DERIVE_KEY_SOURCE_IDENTIFIER = b'fixture-key-1'
DERIVE_KEY_RESULT_IDENTIFIER = b'fixture-derived-key'
DERIVE_KEY_DATA = b'synthetic-derivation-data'
DERIVE_KEY_SALT = b'synthetic-derivation-salt'
SECRET_DATA_MATERIAL = b'synthetic-secret-material'
SECRET_DATA_ITEM_IDS = (b'interop-register', b'interop-destroy')
IMPORT_ITEM_IDS = (b'import-object', b'import-destroy')
IMPORT_IDENTIFIER = b'fixture-imported-secret-data'
IMPORT_OBJECT_DATA = b'controlled-import-object'
CERTIFY_ITEM_IDS = (b'certify-create-pair', b'certify-activate-private', b'certify-public-key',
                    b'certify-revoke-private', b'certify-destroy-certificate',
                    b'certify-destroy-private', b'certify-destroy-public')
MAC_TEST_OUTPUT = bytes(range(32))
CANCEL_CORRELATION_VALUE = b'fixture-cancel-correlation'
ASYNC_QUERY_CORRELATION_CREATE_COMPLETED = b'fixture-query-create-completed'
ASYNC_QUERY_CORRELATION_CREATE_PENDING = b'fixture-query-create-pending'
ASYNC_QUERY_CORRELATION_GET_PENDING = b'fixture-query-get-pending'
CANCEL_RESULT_CANCELED = 1

CERTIFICATE_ATTRIBUTE_FIELDS = (
    'Certificate Subject CN', 'Certificate Subject O', 'Certificate Subject OU',
    'Certificate Subject Email', 'Certificate Subject C', 'Certificate Subject ST',
    'Certificate Subject L', 'Certificate Subject UID', 'Certificate Subject Serial Number',
    'Certificate Subject Title', 'Certificate Subject DC', 'Certificate Subject DN Qualifier',
    'Certificate Issuer CN', 'Certificate Issuer O', 'Certificate Issuer OU',
    'Certificate Issuer Email', 'Certificate Issuer C', 'Certificate Issuer ST',
    'Certificate Issuer L', 'Certificate Issuer UID', 'Certificate Issuer Serial Number',
    'Certificate Issuer Title', 'Certificate Issuer DC', 'Certificate Issuer DN Qualifier',
)

def normalize(name: str) -> str:
    return re.sub('[^a-z0-9]', '', name.lower())


class Codec:
    def __init__(self, catalog):
        self.tags = {normalize(t['name']): int(t['wire_value'], 16)
                     for t in catalog['normative_targets'] if t['kind'] == 'tag'}
        self.values = {}
        self.bitmask_tags = set()
        for target in catalog['normative_targets']:
            if target['kind'] not in ('enumeration_value', 'bitmask_value'):
                continue
            if not re.fullmatch('[0-9A-Fa-f]{8}', target['wire_value']):
                continue
            parent = target['parent_element_ids'][0]
            name = re.sub('^KMIPKIT-ELEM-(ENUMERATION|BITMASK)-', '', parent)
            tag = self.tags.get(normalize(name))
            self.values[(tag, normalize(target['name']))] = int(target['wire_value'], 16)
            if target['kind'] == 'bitmask_value' and tag is not None:
                self.bitmask_tags.add(tag)

    def tag(self, value):
        return int(value, 16) if value.startswith('0x') else self.tags[normalize(value)]

    def _decode_named_bitmask(self, tag, text):
        remaining = normalize(text)
        value = 0
        names = sorted(((name, bit) for (bitmask_tag, name), bit in self.values.items()
                        if bitmask_tag == tag), key=lambda entry: len(entry[0]), reverse=True)
        while remaining:
            for name, bit in names:
                if remaining.startswith(name):
                    value |= bit
                    remaining = remaining[len(name):]
                    break
            else:
                raise ValueError('fixture_invalid_named_bitmask')
        return value

    def scalar(self, tag, kind, value):
        if kind == 7:
            return str(value).encode()
        if kind in (4, 8):
            return bytes.fromhex(str(value).removeprefix('0x'))
        if kind == 6:
            value = int(value) if not isinstance(value, str) else int(value == 'true')
        elif kind == 9:
            value = int(dt.datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp())
        elif isinstance(value, str) and not re.fullmatch('-?[0-9]+', value):
            if value.startswith('0x'):
                value = int(value, 16)
            elif (tag, normalize(value)) in self.values:
                value = self.values[(tag, normalize(value))]
            elif tag in self.bitmask_tags:
                value = self._decode_named_bitmask(tag, value)
            else:
                value = sum(self.values[(tag, normalize(part))] for part in value.split())
        size = 8 if kind in (3, 6, 9, 11) else 4
        return int(value).to_bytes(size, 'big', signed=int(value) < 0)

    def from_object(self, value):
        tag = self.tag(value['tag'])
        kind = TYPES.index(value.get('type', 'Structure'))
        body = b''.join(self.from_object(child) for child in value['value']) if kind == 1 else self.scalar(tag, kind, value['value'])
        return tlv(tag, kind, body)

    def from_xml(self, element):
        tag = self.tag(element.attrib.get('tag', element.tag))
        kind = TYPES.index(element.attrib.get('type', 'Structure'))
        body = b''.join(self.from_xml(child) for child in element) if kind == 1 else self.scalar(tag, kind, element.attrib['value'])
        return tlv(tag, kind, body)

    def decode(self, wire, encoding):
        if encoding == 'TTLV':
            return parse(wire)
        if encoding == 'JSON':
            return parse(self.from_object(json.loads(wire)))
        return parse(self.from_xml(ET.fromstring(wire)))

    def to_object(self, node):
        tag, kind, value = node
        if kind == 1:
            body = [self.to_object(child) for child in value]
        elif kind == 7:
            body = value.decode()
        elif kind in (4, 8):
            body = value.hex().upper()
            if kind == 4:
                body = '0x' + body
        elif kind == 9:
            body = dt.datetime.fromtimestamp(int.from_bytes(value, 'big', signed=True), dt.timezone.utc).isoformat().replace('+00:00', 'Z')
        elif kind == 6:
            body = bool(int.from_bytes(value, 'big'))
        else:
            body = int.from_bytes(value, 'big', signed=kind in (2, 3, 11))
        return {'tag': f'0x{tag:06X}', 'type': TYPES[kind], 'value': body}

    def to_xml(self, node):
        obj = self.to_object(node)
        element = ET.Element('TTLV', {'tag': obj['tag']})
        if obj['type'] == 'Structure':
            for child in node[2]:
                element.append(self.to_xml(child))
        else:
            value = obj['value']
            if isinstance(value, bool):
                value = 'true' if value else 'false'
            elif obj['type'] == 'Enumeration':
                value = f'0x{value:08X}'
            element.attrib.update(type=obj['type'], value=str(value))
        return element

    def encode(self, wire, encoding):
        if encoding == 'TTLV':
            return wire
        if encoding == 'JSON':
            return json.dumps(self.to_object(parse(wire)), separators=(',', ':')).encode()
        return ET.tostring(self.to_xml(parse(wire)))


def catalog_enum_value(catalog: dict, enum_name: str, value_name: str,
                       protocol_version: str) -> int | None:
    parent_id = f"KMIPKIT-ELEM-ENUMERATION-{enum_name.upper().replace(' ', '-')}"
    matches = [target for target in catalog['normative_targets']
               if target['kind'] == 'enumeration_value'
               and target['name'] == value_name
               and parent_id in target.get('parent_element_ids', [])
               and protocol_version in target['protocol_versions']]
    if len(matches) != 1:
        return None
    return int(matches[0]['wire_value'], 16)


def parse(data):
    if len(data) < 8:
        raise ValueError('fixture_truncated_header')
    tag, kind, length = int.from_bytes(data[:3], 'big'), data[3], int.from_bytes(data[4:8], 'big')
    if len(data) != 8 + ((length + 7) // 8) * 8:
        raise ValueError('fixture_invalid_length')
    value = data[8:8 + length]
    if kind == 1:
        children, offset = [], 0
        while offset < len(value):
            size = 8 + ((int.from_bytes(value[offset + 4:offset + 8], 'big') + 7) // 8) * 8
            children.append(parse(value[offset:offset + size]))
            offset += size
        value = children
    return tag, kind, value


def descendants(node, tag):
    if node[0] == tag:
        yield node
    if node[1] == 1:
        for child in node[2]:
            yield from descendants(child, tag)


def numbers(node, tag):
    return [int.from_bytes(item[2], 'big') for item in descendants(node, tag)
            if item[1] != 1 and isinstance(item[2], bytes)]


def direct_numbers(node, tag):
    return [int.from_bytes(item[2], 'big') for item in node[2]
            if item[0] == tag and item[1] != 1 and isinstance(item[2], bytes)]


def text_values(node, tag):
    return [item[2].decode('utf-8') for item in descendants(node, tag) if item[1] == 7]


def byte_string_values(node, tag):
    return [item[2] for item in descendants(node, tag) if item[1] == 8]


def has_single_empty_response_payload(batch_item):
    """Check the generic response shape for a Pending Create result."""
    payloads = [child for child in batch_item[2] if child[0] == RESPONSE_PAYLOAD_TAG]
    return len(payloads) == 1 and payloads[0][1] == 1 and not payloads[0][2]


def response_has_single_empty_payload(response_tree):
    """Check that one response batch item carries exactly one empty payload."""
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    return len(items) == 1 and has_single_empty_response_payload(items[0])


def payload_field_present(target, batch_item, targets, codec, *, role, payload_tag):
    """Check a payload field under the exact operation and message direction."""
    if (target.get('kind') != 'operation_payload_field'
            or target.get('role') != role
            or not isinstance(target.get('operation_id'), str)
            or batch_item[0] != BATCH_ITEM_TAG or batch_item[1] != 1):
        return False
    operation = targets.get(target['operation_id'])
    if operation is None or operation.get('kind') != 'operation':
        return False
    operation_value = codec.values.get((codec.tag('Operation'), normalize(operation['name'])))
    field_tag = codec.tags.get(normalize(target.get('name', '')))
    if operation_value is None or field_tag is None:
        return False
    operation_fields = [child for child in batch_item[2]
                        if child[0] == codec.tag('Operation') and child[1] == 5]
    if len(operation_fields) != 1 or int.from_bytes(operation_fields[0][2], 'big') != operation_value:
        return False
    payloads = [child for child in batch_item[2]
                if child[0] == payload_tag and child[1] == 1]
    return len(payloads) == 1 and any(child[0] == field_tag for child in payloads[0][2])


def request_payload_field_present(target, batch_item, targets, codec):
    """Check a request payload field under its exact operation's payload."""
    return payload_field_present(target, batch_item, targets, codec,
                                 role='request', payload_tag=REQUEST_PAYLOAD_TAG)


def response_payload_field_present(target, batch_item, targets, codec):
    """Check a response payload field under its exact operation's batch item."""
    if target.get('name') == 'Any Object (Section 2)':
        operation = targets.get(target.get('operation_id'))
        object_tag = codec.tags.get(normalize('Symmetric Key'))
        payloads = [child for child in batch_item[2]
                    if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
        operation_value = (codec.values.get((codec.tag('Operation'), normalize(operation['name'])))
                          if operation is not None else None)
        return (target.get('role') == 'response' and operation is not None and
                operation.get('kind') == 'operation' and object_tag is not None and
                operation_value is not None and numbers(batch_item, codec.tag('Operation')) == [operation_value] and
                len(payloads) == 1 and
                any(child[0] == object_tag and child[1] == 1 for child in payloads[0][2]))
    return payload_field_present(target, batch_item, targets, codec,
                                 role='response', payload_tag=RESPONSE_PAYLOAD_TAG)


def tree_digest(node):
    # Redact nondeterministic and sensitive fields before hashing. Preserve
    # numeric result codes, tag/type distinctions, and ordered batch structure.
    sensitive = {0x420094, 0x420092, 0x420099, 0x4200A1, 0x420043, 0x42014B,
                 0x4200D6, 0x42015B, 0x42015C}
    def clean(item):
        tag, kind, value = item
        if tag in sensitive:
            value = '<redacted>'
        elif kind == 1:
            value = [clean(child) for child in value]
        else:
            value = value.hex()
        return [tag, kind, value]
    return hashlib.sha256(json.dumps(clean(node), separators=(',', ':')).encode()).hexdigest()


RECIPES = {
    'test-async-indicator-mandatory-pending': ((1,), 'async_create_pending'),
    'test-async-indicator-omitted-default': ((1,), 'async_create_sync_default'),
    'test-async-indicator-optional-completes-or-pends': ((1,), 'async_create_optional_complete'),
    'test-async-indicator-prohibited-synchronous': ((1,), 'async_create_prohibited_sync'),
    'test-async-lifecycle-poll': ((1, 57, 26, 20), 'async_create_poll_lifecycle'),
    'test-async-no-recursive-async-poll-cancel': ((26, 25), 'async_poll_cancel_no_recursion'),
    'test-async-poll-final-operation-failure': ((1, 26), 'async_poll_final_failure'),
    'test-async-poll-repeated-pending-to-success': ((1, 26), 'async_poll_repeated_pending_to_success'),
    'test-async-process-then-poll-state': ((1, 58, 26), 'async_process_then_poll_state'),
    'test-op-query-asynchronous-requests': ((1, 57, 19, 20), 'query_async_empty'),
    'test-async-query-outstanding-filter': ((1, 57), 'async_query_outstanding_filter'),
    'test-async-unknown-correlation-rejected': ((26,), 'async_unknown_correlation_rejected'),
    'test-op-cancel': ((25,), 'cancel_async'),
    'test-async-cancel-result-canceled': ((25,), 'cancel_async'),
    'test-async-cancel-result-unable-to-cancel': ((25,), 'cancel_async'),
    'test-async-cancel-result-completed': ((25,), 'cancel_async'),
    'test-async-cancel-result-failed': ((25,), 'cancel_async'),
    'test-async-cancel-result-unavailable': ((25,), 'cancel_async'),
    'test-async-cancel-vs-poll-race': ((25,), 'cancel_poll_race'),
    'test-op-hash': ((39,), 'hash'),
    'test-crypto-hash-sha_256-0': ((39,), 'hash'),
    'test-op-rng-retrieve': ((37,), 'rng_retrieve'),
    'test-crypto-rng-retrieve-rng-32': ((37,), 'rng_retrieve'),
    'test-op-rng-seed': ((38,), 'rng_seed'),
    'test-crypto-rng-seed-rng-32': ((38,), 'rng_seed'),
    'test-v20-reject-get-constraints': ((56,), 'get_constraints'),
    'test-v20-reject-ping': ((59,), 'ping'),
    'test-op-create': ((1, 20), 'create'),
    'test-op-archive': ((1, 21, 19, 20), 'archive'),
    'test-op-export': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-20-001': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-20-002': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-20-003': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-20-004': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-20-005': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-21-001': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-21-002': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-21-003': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-21-004': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-request-21-005': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-20-001': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-20-002': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-20-003': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-20-004': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-21-001': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-21-002': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-21-003': ((1, 43, 19, 20), 'export'),
    'test-norm-payload-client-export-response-21-004': ((1, 43, 19, 20), 'export'),
    'test-op-register': ((3, 20), 'register'),
    'test-op-import': ((42, 20), 'import_secret_data'),
    'test-op-encrypt': ((1, 18, 31, 19, 20), 'encrypt'),
    'test-op-decrypt': ((1, 18, 32, 19, 20), 'decrypt'),
    'test-op-mac': ((1, 18, 35, 19, 20), 'mac'),
    'test-op-mac-verify': ((1, 18, 36, 19, 20), 'mac_verify'),
    'test-op-sign': ((2, 18, 33, 19, 20, 20), 'sign'),
    'test-op-signature-verify': ((2, 18, 34, 19, 20, 20), 'signature_verify'),
    'test-op-create-key-pair': ((2, 20, 20), 'create_key_pair'),
    'test-op-create-split-key': ((40, 20, 20), 'create_split_key'),
    'test-op-join-split-key': ((40, 41, 20, 20, 20), 'join_split_key'),
    'test-op-re-key': ((1, 4, 19, 20, 19, 20), 'rekey'),
    'test-op-derive-key': ((1, 18, 5, 19, 20, 20), 'derive_key'),
    'test-op-certify': ((2, 18, 6, 19, 20, 20, 20), 'certify'),
    'test-op-pkcs#11': ((1, 18, 51, 19, 20), 'pkcs11'),
    'test-op-obtain-lease': ((1, 16, 19, 20), 'obtain_lease'),
    'test-op-add-attribute': ((1, 13, 19, 20), 'add_attribute'),
    'test-norm-payload-client-add-attribute-request-20-002': ((1, 13, 19, 20), 'add_attribute'),
    'test-norm-payload-client-add-attribute-request-21-002': ((1, 13, 19, 20), 'add_attribute'),
    'test-op-modify-attribute': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-norm-payload-client-modify-attribute-request-20-002': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-norm-payload-client-modify-attribute-request-20-003': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-norm-payload-client-modify-attribute-response-20-001': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-norm-payload-client-modify-attribute-request-21-002': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-norm-payload-client-modify-attribute-request-21-003': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-norm-payload-client-modify-attribute-response-21-001': ((1, 13, 14, 19, 20), 'modify_attribute'),
    'test-op-set-attribute': ((1, 49, 19, 20), 'set_attribute'),
    'test-op-adjust-attribute': ((1, 48, 19, 20), 'adjust_attribute'),
    'test-op-set-constraints': ((55,), 'set_constraints'),
    'test-op-set-defaults': ((54,), 'set_defaults'),
    'test-op-delete-attribute': ((1, 13, 15, 19, 20), 'delete_attribute'),
    'test-obj-opaque-object': ((3, 20), 'register_opaque_object'),
    'test-obj-pgp-key': ((3, 20), 'register_pgp_key'),
    'test-obj-secret-data': ((3, 20), 'register_secret_data'),
    'test-obj-certificate': ((3, 20), 'register'),
    'test-obj-certificate-request': ((3, 20), 'register_certificate_request'),
    'test-batch-create-activate-destroy': ((1, 18, 12, 19, 20), 'create_activate_attribute_list_revoke_destroy'),
    'test-batch-register-encrypt-decrypt-destroy': (BATCH_CRYPTO_OPERATIONS, 'batch_register_crypto'),
    'test-batch-placeholder-chain-resolves': (PLACEHOLDER_CHAIN_OPERATIONS, 'batch_placeholder_chain'),
    'test-batch-order-required-placeholder-resolution':
        ((1, 18), 'batch_order_placeholder_two_item'),
    'test-batch-order-omitted-defaults-true':
        ((1, 18), 'batch_order_placeholder_two_item'),
    'test-batch-single-item-options-omitted': ((10,), 'batch_single_item_options_omitted'),
    'test-batch-single-item-default-continuation':
        ((10,), 'batch_single_item_default_continuation'),
    'test-batch-placeholder-without-producer-rejected':
        ((18,), 'batch_placeholder_without_producer'),
    'test-batch-count-matches-items': ((10, 12), 'batch_count_two_read_items'),
    'test-batch-count-mismatch-rejected': ((10, 12), 'batch_count_two_read_items'),
    'test-batch-unique-item-id-response-correlation': ((10, 12), 'batch_count_two_read_items'),
    'test-batch-order-false-correlates-response-items': ((10, 12), 'batch_order_false_two_read_items'),
    'test-batch-continue-after-failure': ((10, 12), 'batch_continue_after_failure'),
    'test-batch-stop-on-first-failure': ((10, 10), 'batch_stop_on_first_failure'),
    'test-norm-payload-client-create-request-20-001': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-20-002': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-21-001': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-21-002': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-21-003': ((1, 20), 'create'),
    'test-op-get': ((1, 10, 20), 'get'),
    'test-norm-payload-client-get-request-20-001': ((1, 10, 20), 'get'),
    'test-norm-payload-client-get-request-21-001': ((1, 10, 20), 'get'),
    'test-op-get-attributes': ((1, 11, 20), 'get_attributes'),
    'test-op-get-attribute-list': ((1, 12, 20), 'get_attribute_list'),
    'test-norm-payload-client-get-attribute-list-request-20-001': ((1, 12, 20), 'get_attribute_list'),
    'test-norm-payload-client-get-attribute-list-request-21-001': ((1, 12, 20), 'get_attribute_list'),
    'test-op-check': ((1, 9, 19, 20), 'check'),
    'test-op-activate': ((1, 18, 19, 20), 'activate'),
    'test-op-revoke': ((1, 19, 20), 'revoke'),
    'test-op-destroy': ((1, 20, 19, 20), 'destroy'),
    'test-op-get-constraints': ((1, 56, 19, 20), 'get_constraints'),
    'test-op-get-usage-allocation': ((1, 17, 19, 20), 'get_usage_allocation'),
    'test-norm-payload-client-get-usage-allocation-request-20-001':
        ((8, 17, 17), 'get_usage_allocation_uid_variants'),
    'test-norm-payload-client-get-usage-allocation-request-21-001':
        ((8, 17, 17), 'get_usage_allocation_uid_variants'),
    'test-op-locate': ((8,), 'locate'), 'test-op-query': ((24,), 'query'),
    'test-norm-payload-client-query-request-20-001': ((24,), 'query'),
    'test-norm-payload-client-query-request-21-001': ((24,), 'query'),
    'test-norm-payload-client-query-request-21-002': ((24,), 'query'),
    'test-norm-payload-client-query-response-20-001': ((24,), 'query'),
    'test-norm-payload-client-query-response-21-001': ((24,), 'query'),
    'test-op-discover-versions': ((30,), 'discover_versions'),
    'test-op-log': ((44,), 'log'), 'test-op-login': ((45,), 'login'),
    'test-op-logout': ((45, 46), 'login_logout'),
    'test-op-interop': ((52, 52, 52), 'interop'),
    'test-op-delegated-login': ((47,), 'delegated_login'),
    'test-op-ping': ((59,), 'ping'),
    'test-attr-cryptographic-algorithm': ((1, 11, 20), 'get_attributes'),
    'test-attr-cryptographic-length': ((1, 11, 20), 'get_attributes'),
    'test-obj-symmetric-key': ((1, 20), 'create'),
    'test-obj-private-key': ((1, 20), 'create'),
    'test-obj-public-key': ((1, 20), 'create'),
    'test-neg-destroy-active-without-revoke': ((1, 18, 20, 19, 20), 'destroy'),
}

CONTROLLED_PEER_LOCAL_MODEL_CASES = frozenset({
    'test-batch-single-item-options-omitted',
    'test-batch-placeholder-without-producer-rejected',
    'test-norm-payload-client-get-request-20-001',
    'test-norm-payload-client-get-request-21-001',
    'test-norm-payload-client-get-attribute-list-request-20-001',
    'test-norm-payload-client-get-attribute-list-request-21-001',
    'test-norm-payload-client-add-attribute-request-20-002',
    'test-norm-payload-client-add-attribute-request-21-002',
    'test-norm-payload-client-modify-attribute-request-20-002',
    'test-norm-payload-client-modify-attribute-request-20-003',
    'test-norm-payload-client-modify-attribute-response-20-001',
    'test-norm-payload-client-modify-attribute-request-21-002',
    'test-norm-payload-client-modify-attribute-request-21-003',
    'test-norm-payload-client-modify-attribute-response-21-001',
    'test-norm-payload-client-get-usage-allocation-request-20-001',
    'test-norm-payload-client-get-usage-allocation-request-21-001',
    'test-norm-payload-client-query-request-20-001',
    'test-norm-payload-client-query-request-21-001',
    'test-norm-payload-client-query-request-21-002',
    'test-norm-payload-client-query-response-20-001',
    'test-norm-payload-client-query-response-21-001',
    'test-norm-payload-client-create-request-20-001',
    'test-norm-payload-client-create-request-20-002',
    'test-norm-payload-client-create-request-21-001',
    'test-norm-payload-client-create-request-21-002',
    'test-norm-payload-client-create-request-21-003',
    'test-norm-payload-client-get-usage-allocation-request-20-001',
    'test-norm-payload-client-get-usage-allocation-request-21-001',
    'test-norm-payload-client-export-request-20-001',
    'test-norm-payload-client-export-request-20-002',
    'test-norm-payload-client-export-request-20-003',
    'test-norm-payload-client-export-request-20-004',
    'test-norm-payload-client-export-request-20-005',
    'test-norm-payload-client-export-request-21-001',
    'test-norm-payload-client-export-request-21-002',
    'test-norm-payload-client-export-request-21-003',
    'test-norm-payload-client-export-request-21-004',
    'test-norm-payload-client-export-request-21-005',
    'test-norm-payload-client-export-response-20-001',
    'test-norm-payload-client-export-response-20-002',
    'test-norm-payload-client-export-response-20-003',
    'test-norm-payload-client-export-response-20-004',
    'test-norm-payload-client-export-response-21-001',
    'test-norm-payload-client-export-response-21-002',
    'test-norm-payload-client-export-response-21-003',
    'test-norm-payload-client-export-response-21-004',
})


def assert_registered_crypto_batch(node):
    """Check KMIP 2.0/2.1 Register/crypto fields and the complete ordered batch."""
    def enumeration(tag, value):
        return (tag, 5, value.to_bytes(4, 'big'))

    def integer(tag, value):
        return (tag, 2, value.to_bytes(4, 'big'))

    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 6 or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[value] for value in BATCH_CRYPTO_ITEM_IDS] or
            numbers(node, 0x420010) != [1] or numbers(node, 0x42000E) != [1]):
        raise ValueError('fixture_registered_crypto_batch_shape_mismatch')
    payloads = [[child for child in item[2] if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
                for item in items]
    if any(len(payload) != 1 for payload in payloads):
        raise ValueError('fixture_registered_crypto_payload_missing')
    identifier = enumeration(UNIQUE_IDENTIFIER_TAG, ID_PLACEHOLDER_ENUMERATION)
    attributes = (0x420125, 1, [enumeration(0x420028, 3), integer(0x42002A, 128),
                                integer(0x42002C, 12)])
    symmetric_key = (0x42008F, 1, [(0x420040, 1, [
        enumeration(0x420042, 1),
        (0x420045, 1, [(0x420043, 8, BATCH_CRYPTO_KEY)]),
        enumeration(0x420028, 3), integer(0x42002A, 128)])])
    parameters = (0x42002B, 1, [enumeration(0x420011, 2), enumeration(0x42005F, 1),
                                enumeration(0x420028, 3)])
    expected = [
        [enumeration(0x420057, 2), attributes, symmetric_key],
        [identifier],
        [identifier, parameters, (0x4200C2, 8, BATCH_CRYPTO_PLAINTEXT)],
        [identifier, parameters, (0x4200C2, 8, BATCH_CRYPTO_CIPHERTEXT)],
        [identifier, (0x420081, 1, [enumeration(0x420082, 6)])],
        [identifier],
    ]
    if [payload[0][2] for payload in payloads] != expected:
        raise ValueError('fixture_registered_crypto_request_fields_mismatch')


def assert_login_logout_exchange(node, version, exchange_index):
    """Verify Login obtains the Ticket that the following Logout returns (KMIP 2.x §6.1.30/31)."""
    expected_operation = (45, 46)[exchange_index]
    header = next(descendants(node, 0x420077), node)
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (numbers(header, 0x42006A) != [2] or
            numbers(header, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [expected_operation]):
        raise ValueError('fixture_login_logout_operation_mismatch')
    if exchange_index == 0:
        payloads = [child for child in items[0][2]
                    if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
        request_counts = [child for payload in payloads for child in payload[2]
                          if child[0] == 0x42014C and child[1] == 2]
        if (len(payloads) != 1 or len(request_counts) != 1 or
                int.from_bytes(request_counts[0][2], 'big', signed=True) != 2):
            raise ValueError('fixture_logout_login_request_count_mismatch')
        return
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    tickets = [child for payload in payloads for child in payload[2]
               if child[0] == 0x420149 and child[1] == 1]
    ticket_types = [child for ticket in tickets for child in ticket[2]
                    if child[0] == 0x42014A and child[1] == 5]
    ticket_values = [child for ticket in tickets for child in ticket[2]
                     if child[0] == 0x42014B and child[1] == 8]
    if (len(payloads) != 1 or len(tickets) != 1 or len(ticket_types) != 1 or
            ticket_types[0][2] != (1).to_bytes(4, 'big') or len(ticket_values) != 1 or
            ticket_values[0][2] != b'controlled-login-ticket'):
        raise ValueError('fixture_logout_ticket_mismatch')


def verify_login_logout_response(node, exchange_index):
    """Check both operation responses and keep the server-issued Ticket out of evidence."""
    expected_operation = (45, 46)[exchange_index]
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [expected_operation] or
            numbers(items[0], 0x42007F) != [0]):
        raise ValueError('fixture_login_logout_response_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == 0x42007C and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_login_logout_response_payload_mismatch')
    if exchange_index == 0:
        tickets = [child for child in payloads[0][2]
                   if child[0] == 0x420149 and child[1] == 1]
        ticket_types = [child for ticket in tickets for child in ticket[2]
                        if child[0] == 0x42014A and child[1] == 5]
        ticket_values = [child for ticket in tickets for child in ticket[2]
                         if child[0] == 0x42014B and child[1] == 8]
        if (len(tickets) != 1 or len(ticket_types) != 1 or
                ticket_types[0][2] != (1).to_bytes(4, 'big') or
                len(ticket_values) != 1 or ticket_values[0][2] != b'controlled-login-ticket'):
            raise ValueError('fixture_login_response_ticket_mismatch')
    elif payloads[0][2]:
        raise ValueError('fixture_logout_response_payload_not_empty')


def verify_interop_response(node):
    """Check one successful, empty-payload response per ordered Interop item."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[52], [52], [52]] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0], [0]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[b'interop-begin'], [b'interop-end'], [b'interop-reset']]):
        raise ValueError('fixture_interop_response_status_mismatch')
    payloads = [[child for child in item[2]
                 if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
                for item in items]
    if len(items) != 3 or any(len(payload) != 1 or payload[0][2] for payload in payloads):
        raise ValueError('fixture_interop_response_payload_mismatch')


def assert_interop_request(node, version):
    """Check the ordered Begin/End/Reset requests and shared test identifier."""
    header = next(descendants(node, 0x420077), node)
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (numbers(header, 0x42006A) != [2] or
            numbers(header, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [3] or len(items) != 3 or
            [numbers(item, 0x42005C) for item in items] != [[52], [52], [52]]):
        raise ValueError('fixture_interop_request_shape_mismatch')
    functions, identifiers = [], []
    item_identifiers = []
    for item in items:
        payloads = [child for child in item[2]
                    if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
        if len(payloads) != 1:
            raise ValueError('fixture_interop_payload_missing')
        functions.append(numbers(payloads[0], 0x420160))
        identifiers.append(text_values(payloads[0], 0x420161))
        item_identifiers.append(byte_string_values(item, 0x420093))
    if functions != [[1], [2], [3]]:
        raise ValueError('interop_function_sequence_mismatch')
    if (any(len(values) != 1 or not values[0] for values in identifiers) or
            len({values[0] for values in identifiers}) != 1):
        raise ValueError('fixture_interop_identifier_mismatch')
    if item_identifiers != [[b'interop-begin'], [b'interop-end'], [b'interop-reset']]:
        raise ValueError('fixture_interop_batch_item_id_mismatch')


def request_add_attribute_has_new_attribute(node):
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    mutation_items = [item for item in items if numbers(item, 0x42005C) == [13]]
    if len(mutation_items) != 1:
        return False
    payloads = [child for child in mutation_items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    return len(payloads) == 1 and sum(
        child[0] == 0x42013D and child[1] == 1 for child in payloads[0][2]) == 1


def modify_attribute_value(attribute_field):
    comments = [child for child in attribute_field[2]
                if child[0] == 0x4200FD and child[1] == 7]
    if attribute_field[0] not in (0x42013C, 0x42013D) or len(comments) != 1:
        raise ValueError('fixture_modify_attribute_structure_mismatch')
    return 'Comment', comments[0][2]


def request_modify_attribute_fields(batch_item):
    payloads = [child for child in batch_item[2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_modify_attribute_payload_missing')
    current = [child for child in payloads[0][2]
               if child[0] == 0x42013C and child[1] == 1]
    new = [child for child in payloads[0][2]
           if child[0] == 0x42013D and child[1] == 1]
    if len(current) > 1 or len(new) != 1:
        raise ValueError('fixture_modify_attribute_field_cardinality_mismatch')
    return (modify_attribute_value(current[0]) if current else None,
            modify_attribute_value(new[0]))


def request_adjust_attribute_fields(batch_item):
    payloads = [child for child in batch_item[2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_adjust_attribute_payload_missing')
    references = [child for child in payloads[0][2]
                  if child[0] == 0x42013B and child[1] == 1]
    vendors = [child for child in references[0][2]
               if child[0] == 0x42009D and child[1] == 7] if len(references) == 1 else []
    names = [child for child in references[0][2]
             if child[0] == 0x42000A and child[1] == 7] if len(references) == 1 else []
    types = [child for child in payloads[0][2]
             if child[0] == 0x420158 and child[1] == 5]
    values = [child for child in payloads[0][2]
              if child[0] == 0x420162 and child[1] == 2]
    if len(vendors) != 1 or len(names) != 1 or len(types) != 1 or len(values) > 1:
        raise ValueError('fixture_adjust_attribute_field_cardinality_mismatch')
    adjustment_value = (int.from_bytes(values[0][2], 'big', signed=True)
                        if values else 1)
    return (vendors[0][2].decode('utf-8'), names[0][2].decode('utf-8'),
            int.from_bytes(types[0][2], 'big'), adjustment_value)


def assert_crypto_request(node, decrypt):
    """Check an isolated Create/Activate/crypto/Revoke/Destroy request shape."""
    operation = 32 if decrypt else 31
    item_ids = DECRYPT_ITEM_IDS if decrypt else ENCRYPT_ITEM_IDS
    usage_mask = 8 if decrypt else 4
    input_data = BATCH_CRYPTO_CIPHERTEXT if decrypt else BATCH_CRYPTO_PLAINTEXT
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    operations = [numbers(item, 0x42005C) for item in items]
    observed_item_ids = [byte_string_values(item, 0x420093) for item in items]
    if (operations != [[1], [18], [operation], [19], [20]] or
            observed_item_ids != [[item_id] for item_id in item_ids] or
            numbers(items[0], 0x420057) != [2] or
            numbers(items[0], 0x420028) != [3] or
            numbers(items[0], 0x42002A) != [128] or
            numbers(items[0], 0x42002C) != [usage_mask] or
            any(not is_id_placeholder_identifier(identifier)
                for item in items[1:]
                for identifier in descendants(item, UNIQUE_IDENTIFIER_TAG))):
        raise ValueError('fixture_crypto_batch_lifecycle_mismatch')
    encrypt_payloads = [child for child in items[2][2]
                        if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(encrypt_payloads) != 1:
        raise ValueError('fixture_encrypt_payload_missing')
    fields = encrypt_payloads[0][2]
    parameters = [field for field in fields if field[0] == 0x42002B and field[1] == 1]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG]
    data = [field for field in fields if field[0] == 0x4200C2 and field[1] == 8]
    if ([field[0] for field in fields] != [UNIQUE_IDENTIFIER_TAG, 0x42002B, 0x4200C2] or
            len(identifiers) != 1 or not is_id_placeholder_identifier(identifiers[0]) or
            len(parameters) != 1 or len(data) != 1 or
            data[0][2] != input_data):
        raise ValueError('fixture_crypto_request_fields_mismatch')
    parameter_fields = parameters[0][2]
    expected_parameters = [(0x420011, 5, (2).to_bytes(4, 'big')),
                           (0x42005F, 5, (1).to_bytes(4, 'big')),
                           (0x420028, 5, (3).to_bytes(4, 'big'))]
    if [(field[0], field[1], field[2]) for field in parameter_fields] != expected_parameters:
        raise ValueError('fixture_crypto_parameters_mismatch')


def verify_crypto_response(response_tree, decrypt):
    """Check all lifecycle outcomes and typed Encrypt or Decrypt fields."""
    case_id = 'test-op-decrypt' if decrypt else 'test-op-encrypt'
    item_ids = DECRYPT_ITEM_IDS if decrypt else ENCRYPT_ITEM_IDS
    expected_data = BATCH_CRYPTO_PLAINTEXT if decrypt else BATCH_CRYPTO_CIPHERTEXT
    verify_lifecycle(RECIPES[case_id][0], response_tree)
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if [byte_string_values(item, 0x420093) for item in items] != [[item_id] for item_id in item_ids]:
        raise ValueError('fixture_crypto_response_item_ids_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_crypto_response_payload_missing')
    fields = payloads[0][2]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7]
    data = [field for field in fields if field[0] == 0x4200C2 and field[1] == 8]
    if (len(fields) != 2 or len(identifiers) != 1 or identifiers[0][2] != b'fixture-key-1' or
            len(data) != 1 or data[0][2] != expected_data):
        raise ValueError('fixture_crypto_response_fields_mismatch')


def assert_mac_request(node):
    """Check the isolated HMAC-SHA256 Create/Activate/MAC/Revoke/Destroy request."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[1], [18], [35], [19], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in MAC_ITEM_IDS] or
            numbers(items[0], 0x420057) != [2] or numbers(items[0], 0x420028) != [9] or
            numbers(items[0], 0x42002A) != [256] or numbers(items[0], 0x42002C) != [128] or
            any(not is_id_placeholder_identifier(identifier)
                for item in items[1:]
                for identifier in descendants(item, UNIQUE_IDENTIFIER_TAG))):
        raise ValueError('fixture_mac_batch_lifecycle_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_mac_request_payload_missing')
    fields = payloads[0][2]
    parameters = [field for field in fields if field[0] == 0x42002B and field[1] == 1]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG]
    data = [field for field in fields if field[0] == 0x4200C2 and field[1] == 8]
    if ([field[0] for field in fields] != [UNIQUE_IDENTIFIER_TAG, 0x42002B, 0x4200C2] or
            len(identifiers) != 1 or not is_id_placeholder_identifier(identifiers[0]) or
            len(parameters) != 1 or len(data) != 1 or data[0][2] != BATCH_CRYPTO_PLAINTEXT or
            [field[0] for field in parameters[0][2]] != [0x420028] or
            numbers(parameters[0], 0x420028) != [9]):
        raise ValueError('fixture_mac_request_fields_mismatch')


def verify_mac_response(response_tree):
    """Check the complete MAC response fields and ordered lifecycle."""
    verify_lifecycle(RECIPES['test-op-mac'][0], response_tree)
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if [byte_string_values(item, 0x420093) for item in items] != [[item_id] for item_id in MAC_ITEM_IDS]:
        raise ValueError('fixture_mac_response_item_ids_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_mac_response_payload_missing')
    fields = payloads[0][2]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7]
    mac_data = [field for field in fields if field[0] == 0x4200C6 and field[1] == 8]
    if (len(fields) != 2 or len(identifiers) != 1 or identifiers[0][2] != b'fixture-key-1' or
            len(mac_data) != 1 or mac_data[0][2] != MAC_TEST_OUTPUT):
        raise ValueError('fixture_mac_response_fields_mismatch')


def assert_mac_verify_request(node):
    """Check an isolated HMAC-SHA256 MAC Verify request and cleanup."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[1], [18], [36], [19], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in MAC_VERIFY_ITEM_IDS] or
            numbers(items[0], 0x420057) != [2] or numbers(items[0], 0x420028) != [9] or
            numbers(items[0], 0x42002A) != [256] or numbers(items[0], 0x42002C) != [256] or
            any(not is_id_placeholder_identifier(identifier)
                for item in items[1:]
                for identifier in descendants(item, UNIQUE_IDENTIFIER_TAG))):
        raise ValueError('fixture_mac_verify_batch_lifecycle_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_mac_verify_request_payload_missing')
    fields = payloads[0][2]
    parameters = [field for field in fields if field[0] == 0x42002B and field[1] == 1]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG]
    data = [field for field in fields if field[0] == 0x4200C2 and field[1] == 8]
    mac_data = [field for field in fields if field[0] == 0x4200C6 and field[1] == 8]
    if ([field[0] for field in fields] !=
            [UNIQUE_IDENTIFIER_TAG, 0x42002B, 0x4200C2, 0x4200C6] or
            len(identifiers) != 1 or not is_id_placeholder_identifier(identifiers[0]) or
            len(parameters) != 1 or len(data) != 1 or data[0][2] != BATCH_CRYPTO_PLAINTEXT or
            len(mac_data) != 1 or mac_data[0][2] != MAC_TEST_OUTPUT or
            [field[0] for field in parameters[0][2]] != [0x420028] or
            numbers(parameters[0], 0x420028) != [9]):
        raise ValueError('fixture_mac_verify_request_fields_mismatch')


def verify_mac_verify_response(response_tree):
    """Check the Valid MAC Verify fixture result and full ordered lifecycle."""
    verify_lifecycle(RECIPES['test-op-mac-verify'][0], response_tree)
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if [byte_string_values(item, 0x420093) for item in items] != [[item_id] for item_id in MAC_VERIFY_ITEM_IDS]:
        raise ValueError('fixture_mac_verify_response_item_ids_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_mac_verify_response_payload_missing')
    fields = payloads[0][2]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7]
    validity = [field for field in fields if field[0] == 0x42009B and field[1] == 5]
    if (len(fields) != 2 or len(identifiers) != 1 or identifiers[0][2] != b'fixture-key-1' or
            len(validity) != 1 or int.from_bytes(validity[0][2], 'big') != 1):
        raise ValueError('fixture_mac_verify_response_fields_mismatch')


def assert_signature_request(node, verify):
    """Check a key-pair-backed RSA Sign or Signature Verify batch."""
    operation = 34 if verify else 33
    item_ids = SIGNATURE_VERIFY_ITEM_IDS if verify else SIGN_ITEM_IDS
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[2], [18], [operation], [19], [20], [20]] or
            [byte_string_values(item, 0x420093) for item in items] != [[item_id] for item_id in item_ids]):
        raise ValueError('fixture_signature_batch_lifecycle_mismatch')
    create_payloads = [child for child in items[0][2]
                       if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(create_payloads) != 1:
        raise ValueError('fixture_signature_create_pair_payload_missing')
    fields = create_payloads[0][2]
    if [field[0] for field in fields] != [0x420126, 0x420127, 0x420128]:
        raise ValueError('fixture_signature_create_pair_fields_mismatch')
    common, private, public = fields
    if (numbers(common, 0x420028) != [4] or numbers(common, 0x42002A) != [2048] or
            numbers(private, 0x42002C) != [1] or numbers(public, 0x42002C) != [2]):
        raise ValueError('fixture_signature_key_usage_mismatch')
    target_uid = 6 if verify else 5
    other_uid = 5 if verify else 6
    if ([numbers(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] !=
            [[target_uid], [target_uid], [target_uid], [target_uid], [other_uid]]):
        raise ValueError('fixture_signature_unique_identifier_mapping_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_signature_request_payload_missing')
    fields = payloads[0][2]
    expected_tags = [UNIQUE_IDENTIFIER_TAG, 0x42002B, 0x4200C2]
    if verify:
        expected_tags.append(0x4200C3)
    if ([field[0] for field in fields] != expected_tags or
            len([field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 5]) != 1 or
            [field[0] for field in fields[1][2]] != [0x42004E] or
            numbers(fields[1], 0x42004E) != [5] or
            len([field for field in fields if field[0] == 0x4200C2 and field[1] == 8]) != 1 or
            [field[2] for field in fields if field[0] == 0x4200C2] != [BATCH_CRYPTO_PLAINTEXT]):
        raise ValueError('fixture_signature_request_fields_mismatch')
    if verify:
        signature = [field for field in fields if field[0] == 0x4200C3 and field[1] == 8]
        if len(signature) != 1 or signature[0][2] != bytes(256):
            raise ValueError('fixture_signature_verify_input_mismatch')


def verify_signature_response(response_tree, verify):
    """Check typed key-pair outputs and a shape-only signature result."""
    operation = 34 if verify else 33
    item_ids = SIGNATURE_VERIFY_ITEM_IDS if verify else SIGN_ITEM_IDS
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[2], [18], [operation], [19], [20], [20]] or
            [byte_string_values(item, 0x420093) for item in items] != [[item_id] for item_id in item_ids] or
            [numbers(item, 0x42007F) for item in items] != [[0]] * 6):
        raise ValueError('fixture_signature_response_lifecycle_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_signature_key_pair_response_missing')
    private_ids = [field for field in payloads[0][2] if field[0] == 0x420066 and field[1] == 7]
    public_ids = [field for field in payloads[0][2] if field[0] == 0x42006F and field[1] == 7]
    if (len(payloads[0][2]) != 2 or len(private_ids) != 1 or len(public_ids) != 1 or
            private_ids[0][2] != b'fixture-private-key-1' or
            public_ids[0][2] != b'fixture-public-key-1'):
        raise ValueError('fixture_signature_key_pair_response_mismatch')
    target_identifier = b'fixture-public-key-1' if verify else b'fixture-private-key-1'
    other_identifier = b'fixture-private-key-1' if verify else b'fixture-public-key-1'
    expected_lifecycle_identifiers = [
        target_identifier.decode('ascii'), target_identifier.decode('ascii'),
        target_identifier.decode('ascii'), target_identifier.decode('ascii'),
        other_identifier.decode('ascii'),
    ]
    if [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] != [
            [identifier] for identifier in expected_lifecycle_identifiers]:
        raise ValueError('fixture_signature_lifecycle_identifiers_mismatch')
    target_payloads = [child for child in items[2][2]
                       if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(target_payloads) != 1:
        raise ValueError('fixture_signature_operation_response_missing')
    fields = target_payloads[0][2]
    if verify:
        if (len(fields) != 2 or text_values(items[2], UNIQUE_IDENTIFIER_TAG) != [target_identifier.decode()] or
                numbers(items[2], 0x42009B) != [1]):
            raise ValueError('fixture_signature_verify_response_fields_mismatch')
    else:
        signature = [field for field in fields if field[0] == 0x4200C3 and field[1] == 8]
        if (len(fields) != 2 or text_values(items[2], UNIQUE_IDENTIFIER_TAG) != [target_identifier.decode()] or
                len(signature) != 1 or len(signature[0][2]) != 256):
            raise ValueError('fixture_sign_response_fields_mismatch')


def assert_create_key_pair_request(node):
    """Check a Create Key Pair request and cleanup of both returned objects."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[2], [20], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in CREATE_KEY_PAIR_ITEM_IDS]):
        raise ValueError('fixture_create_key_pair_batch_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_create_key_pair_payload_missing')
    fields = payloads[0][2]
    if [field[0] for field in fields] != [0x420126, 0x420127, 0x420128]:
        raise ValueError('fixture_create_key_pair_attribute_fields_mismatch')
    common, private, public = fields
    if (numbers(common, 0x420028) != [4] or numbers(common, 0x42002A) != [2048] or
            numbers(private, 0x42002C) != [1] or numbers(public, 0x42002C) != [2]):
        raise ValueError('fixture_create_key_pair_attributes_mismatch')
    if [numbers(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] != [[5], [6]]:
        raise ValueError('fixture_create_key_pair_cleanup_identifiers_mismatch')


def verify_create_key_pair_response(response_tree):
    """Check distinct response IDs and exact cleanup identifier mapping."""
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[2], [20], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in CREATE_KEY_PAIR_ITEM_IDS] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0], [0]]):
        raise ValueError('fixture_create_key_pair_response_lifecycle_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_create_key_pair_response_payload_missing')
    fields = payloads[0][2]
    private_ids = [field for field in fields if field[0] == 0x420066 and field[1] == 7]
    public_ids = [field for field in fields if field[0] == 0x42006F and field[1] == 7]
    if (len(fields) != 2 or len(private_ids) != 1 or len(public_ids) != 1 or
            private_ids[0][2] != b'fixture-private-key-1' or
            public_ids[0][2] != b'fixture-public-key-1'):
        raise ValueError('fixture_create_key_pair_response_identifiers_mismatch')
    if [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] != [
            ['fixture-private-key-1'], ['fixture-public-key-1']]:
        raise ValueError('fixture_create_key_pair_cleanup_mapping_mismatch')


def assert_create_split_key_request(node, item_id=CREATE_SPLIT_KEY_ITEM_IDS[0]):
    """Check a 2-of-2 AES Create Split Key request and its item identity."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [40] or
            byte_string_values(items[0], 0x420093) != [item_id]):
        raise ValueError('fixture_create_split_key_batch_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_create_split_key_payload_missing')
    fields = payloads[0][2]
    object_type = [field for field in fields if field[0] == 0x420057]
    part_counts = [field for field in fields if field[0] == 0x42008B]
    thresholds = [field for field in fields if field[0] == 0x42008C]
    methods = [field for field in fields if field[0] == 0x42008A]
    attributes = [field for field in fields if field[0] == 0x420125 and field[1] == 1]
    if (len(object_type) != 1 or object_type[0][1] != 5 or
            int.from_bytes(object_type[0][2], 'big') != 2 or
            len(part_counts) != 1 or part_counts[0][1] != 2 or
            int.from_bytes(part_counts[0][2], 'big', signed=True) != 2 or
            len(thresholds) != 1 or thresholds[0][1] != 2 or
            int.from_bytes(thresholds[0][2], 'big', signed=True) != 2 or
            len(methods) != 1 or methods[0][1] != 5 or int.from_bytes(methods[0][2], 'big') != 1 or
            len(attributes) != 1):
        raise ValueError('fixture_create_split_key_parameters_mismatch')
    attribute_fields = attributes[0][2]
    if ([field[0] for field in attribute_fields] != [0x420028, 0x42002A, 0x42002C] or
            int.from_bytes(attribute_fields[0][2], 'big') != 3 or
            int.from_bytes(attribute_fields[1][2], 'big', signed=True) != 128 or
            int.from_bytes(attribute_fields[2][2], 'big', signed=True) != 4):
        raise ValueError('fixture_create_split_key_attributes_mismatch')


def verify_create_split_key_response(response_tree, item_id=CREATE_SPLIT_KEY_ITEM_IDS[0]):
    """Check both repeated split-part IDs and the successful operation result."""
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [40] or
            byte_string_values(items[0], 0x420093) != [item_id] or
            numbers(items[0], 0x42007F) != [0]):
        raise ValueError('fixture_create_split_key_response_lifecycle_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    identifiers = [child[2] for child in payloads[0][2]
                   if child[0] == UNIQUE_IDENTIFIER_TAG and child[1] == 7] if len(payloads) == 1 else []
    if identifiers != list(CREATE_SPLIT_KEY_IDENTIFIERS):
        raise ValueError('fixture_create_split_key_response_identifiers_mismatch')


def assert_split_key_destroy_request(node, exchange_index):
    expected_identifier = CREATE_SPLIT_KEY_IDENTIFIERS[exchange_index - 1]
    expected_item_id = CREATE_SPLIT_KEY_ITEM_IDS[exchange_index]
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [20] or
            byte_string_values(items[0], 0x420093) != [expected_item_id] or
            text_values(items[0], UNIQUE_IDENTIFIER_TAG) != [expected_identifier.decode('ascii')]):
        raise ValueError('fixture_create_split_key_cleanup_request_mismatch')


def verify_split_key_destroy_response(response_tree, exchange_index):
    expected_identifier = CREATE_SPLIT_KEY_IDENTIFIERS[exchange_index - 1]
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [20] or
            byte_string_values(items[0], 0x420093) !=
            [CREATE_SPLIT_KEY_ITEM_IDS[exchange_index]] or
            numbers(items[0], 0x42007F) != [0] or
            text_values(items[0], UNIQUE_IDENTIFIER_TAG) != [expected_identifier.decode('ascii')]):
        raise ValueError('fixture_create_split_key_cleanup_response_mismatch')


def assert_join_split_key_request(node):
    """Check Join Split Key references both created parts and cleans all three objects."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    operations = [[41], [20], [20], [20]]
    if ([numbers(item, 0x42005C) for item in items] != operations or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in JOIN_SPLIT_KEY_ITEM_IDS[1:]]):
        raise ValueError('fixture_join_split_key_batch_mismatch')
    join_payloads = [child for child in items[0][2]
                     if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(join_payloads) != 1:
        raise ValueError('fixture_join_split_key_payload_missing')
    fields = join_payloads[0][2]
    identifiers = [field[2] for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7]
    if (identifiers != list(CREATE_SPLIT_KEY_IDENTIFIERS) or
            numbers(items[0], 0x420057) != [2] or
            numbers(items[1], UNIQUE_IDENTIFIER_TAG) != [ID_PLACEHOLDER_ENUMERATION] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[2:]] !=
            [[identifier.decode('ascii')] for identifier in CREATE_SPLIT_KEY_IDENTIFIERS]):
        raise ValueError('fixture_join_split_key_identifiers_mismatch')


def verify_join_split_key_response(response_tree):
    """Check the joined identifier and exact joined/source object cleanup responses."""
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[41], [20], [20], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in JOIN_SPLIT_KEY_ITEM_IDS[1:]] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0], [0], [0]]):
        raise ValueError('fixture_join_split_key_response_lifecycle_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    joined_ids = [field[2] for field in payloads[0][2]
                  if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7] if len(payloads) == 1 else []
    if joined_ids != [b'fixture-joined-key']:
        raise ValueError('fixture_join_split_key_result_identifier_mismatch')
    expected_cleanup = (b'fixture-joined-key', *CREATE_SPLIT_KEY_IDENTIFIERS)
    if [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] != [
            [identifier.decode('ascii')] for identifier in expected_cleanup]:
        raise ValueError('fixture_join_split_key_cleanup_identifiers_mismatch')


def assert_rekey_create_request(node):
    """Check the isolated AES key setup used by the Re-key fixture."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if (len(items) != 1 or numbers(node, 0x42000D) != [1] or
            numbers(items[0], 0x42005C) != [1] or
            byte_string_values(items[0], 0x420093) != [REKEY_ITEM_IDS[0]] or
            numbers(items[0], 0x420057) != [2] or numbers(items[0], 0x420028) != [3] or
            numbers(items[0], 0x42002A) != [128] or numbers(items[0], 0x42002C) != [4]):
        raise ValueError('fixture_rekey_create_request_mismatch')


def assert_rekey_request(node):
    """Check Re-key targets the created key and both results are torn down."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[4], [19], [20], [19], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in REKEY_ITEM_IDS[1:]] or
            numbers(node, 0x42000D) != [5] or
            [(field[1], int.from_bytes(field[2], 'big')) for field in descendants(node, 0x420010)] !=
            [(6, 1)]):
        raise ValueError('fixture_rekey_batch_mismatch')
    rekey_payloads = [child for child in items[0][2]
                      if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if (len(rekey_payloads) != 1 or
            [(field[0], field[1], field[2]) for field in rekey_payloads[0][2]] !=
            [(UNIQUE_IDENTIFIER_TAG, 7, REKEY_ORIGINAL_IDENTIFIER)] or
            [numbers(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:3]] != [[1], [1]] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[3:]] !=
            [[REKEY_ORIGINAL_IDENTIFIER.decode('ascii')],
             [REKEY_ORIGINAL_IDENTIFIER.decode('ascii')]]):
        raise ValueError('fixture_rekey_identifier_or_placeholder_mismatch')


def verify_rekey_create_response(response_tree):
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    identifiers = [field for field in payloads[0][2]
                   if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7] if len(payloads) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [1] or
            numbers(items[0], 0x42007F) != [0] or len(identifiers) != 1 or
            identifiers[0][2] != REKEY_ORIGINAL_IDENTIFIER or
            byte_string_values(items[0], 0x420093) != [REKEY_ITEM_IDS[0]]):
        raise ValueError('fixture_rekey_create_response_identifier_mismatch')


def verify_rekey_response(response_tree):
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[4], [19], [20], [19], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in REKEY_ITEM_IDS[1:]] or
            [numbers(item, 0x42007F) for item in items] != [[0]] * 5):
        raise ValueError('fixture_rekey_response_lifecycle_mismatch')
    payload_values = []
    for item in items:
        payloads = [child for child in item[2]
                    if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
        if len(payloads) != 1:
            raise ValueError('fixture_rekey_response_payload_missing')
        identifiers = [field for field in payloads[0][2]
                       if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7]
        if len(identifiers) != 1:
            raise ValueError('fixture_rekey_response_identifier_missing')
        payload_values.append(identifiers[0][2])
    if payload_values != [REKEY_REPLACEMENT_IDENTIFIER] * 3 + [REKEY_ORIGINAL_IDENTIFIER] * 2:
        raise ValueError('fixture_rekey_response_identifiers_mismatch')


def assert_derive_key_create_request(node, version):
    """Check the catalog's isolated AES source key and required activation."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    header = next(descendants(node, 0x420077), node)
    if (numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [2] or
            [numbers(item, 0x42005C) for item in items] != [[1], [18]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[DERIVE_KEY_ITEM_IDS[0]], [DERIVE_KEY_ITEM_IDS[1]]] or
            numbers(items[0], 0x420057) != [2] or numbers(items[0], 0x420028) != [3] or
            numbers(items[0], 0x42002A) != [128] or numbers(items[0], 0x42002C) != [512] or
            numbers(items[1], UNIQUE_IDENTIFIER_TAG) != [ID_PLACEHOLDER_ENUMERATION]):
        raise ValueError('fixture_derive_key_create_request_mismatch')


def assert_derive_key_request(node, version):
    """Check explicit PBKDF2 parameters, output attributes, and placeholder cleanup."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    header = next(descendants(node, 0x420077), node)
    if (numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])]):
        raise ValueError('fixture_derive_key_version_mismatch')
    if numbers(node, 0x42000D) != [4]:
        raise ValueError('fixture_derive_key_batch_count_mismatch')
    if numbers(node, 0x42000E) != [1] or numbers(node, 0x420010) != [1]:
        raise ValueError('fixture_derive_key_batch_option_mismatch')
    if [numbers(item, 0x42005C) for item in items] != [[5], [19], [20], [20]]:
        raise ValueError('fixture_derive_key_operations_mismatch')
    if [byte_string_values(item, 0x420093) for item in items] != [
            [item_id] for item_id in DERIVE_KEY_ITEM_IDS[2:]]:
        raise ValueError('fixture_derive_key_batch_item_id_mismatch')
    if text_values(items[0], UNIQUE_IDENTIFIER_TAG) != [DERIVE_KEY_SOURCE_IDENTIFIER.decode()]:
        raise ValueError('fixture_derive_key_source_identifier_mismatch')
    if text_values(items[1], UNIQUE_IDENTIFIER_TAG) != [DERIVE_KEY_SOURCE_IDENTIFIER.decode()]:
        raise ValueError('fixture_derive_key_revoke_identifier_mismatch')
    if numbers(items[2], UNIQUE_IDENTIFIER_TAG) != [ID_PLACEHOLDER_ENUMERATION]:
        raise ValueError('fixture_derive_key_destroy_placeholder_mismatch')
    if text_values(items[3], UNIQUE_IDENTIFIER_TAG) != [DERIVE_KEY_SOURCE_IDENTIFIER.decode()]:
        raise ValueError('fixture_derive_key_source_cleanup_identifier_mismatch')
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_derive_key_payload_missing')
    payload = payloads[0]
    parameters = [child for child in payload[2]
                  if child[0] == 0x420032 and child[1] == 1]
    attrs = [child for child in payload[2]
             if child[0] == 0x420125 and child[1] == 1]
    parameter = parameters[0] if len(parameters) == 1 else None
    crypto_parameters = [child for child in parameter[2]
                         if child[0] == 0x42002B and child[1] == 1] if parameter else []
    crypto_parameters = crypto_parameters[0] if len(crypto_parameters) == 1 else None
    fields = parameter[2] if parameter else []
    if (numbers(payload, 0x420057) != [2] or numbers(payload, 0x420031) != [1] or
            len(attrs) != 1 or numbers(attrs[0], 0x420028) != [3] or
            numbers(attrs[0], 0x42002A) != [128] or
            numbers(attrs[0], 0x42002C) != [4] or
            byte_string_values(parameter, 0x420030) != [DERIVE_KEY_DATA] or
            byte_string_values(parameter, 0x420084) != [DERIVE_KEY_SALT] or
            numbers(parameter, 0x42003C) != [1000] or
            numbers(crypto_parameters, 0x420038) != [6]):
        raise ValueError('fixture_derive_key_parameters_mismatch')


def verify_derive_key_create_response(response_tree):
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[1], [18]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[DERIVE_KEY_ITEM_IDS[0]], [DERIVE_KEY_ITEM_IDS[1]]] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0]] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items] !=
            [[DERIVE_KEY_SOURCE_IDENTIFIER.decode()], [DERIVE_KEY_SOURCE_IDENTIFIER.decode()]]):
        raise ValueError('fixture_derive_key_create_response_mismatch')


def verify_derive_key_response(response_tree):
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if [numbers(item, 0x42005C) for item in items] != [[5], [19], [20], [20]]:
        raise ValueError('fixture_derive_key_response_operations_mismatch')
    if [byte_string_values(item, 0x420093) for item in items] != [
            [item_id] for item_id in DERIVE_KEY_ITEM_IDS[2:]]:
        raise ValueError('fixture_derive_key_response_item_ids_mismatch')
    if [numbers(item, 0x42007F) for item in items] != [[0]] * 4:
        raise ValueError('fixture_derive_key_response_status_mismatch')
    if [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items] != [
            [DERIVE_KEY_RESULT_IDENTIFIER.decode()], [DERIVE_KEY_SOURCE_IDENTIFIER.decode()],
            [DERIVE_KEY_RESULT_IDENTIFIER.decode()], [DERIVE_KEY_SOURCE_IDENTIFIER.decode()]]:
        raise ValueError('fixture_derive_key_response_identifiers_mismatch')


def verify_secret_data_response(response_tree):
    """Check typed Register response and exact-ID Destroy cleanup without emitting the secret."""
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[3], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in SECRET_DATA_ITEM_IDS] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0]] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items] !=
            [['fixture-key-1'], ['fixture-key-1']]):
        raise ValueError('fixture_register_secret_data_response_mismatch')


def verify_import_response(response_tree):
    """Check the typed Import identifier and the exact-ID Destroy cleanup."""
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[42], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in IMPORT_ITEM_IDS] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0]] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items] !=
            [[IMPORT_IDENTIFIER.decode()], [IMPORT_IDENTIFIER.decode()]]):
        raise ValueError('fixture_import_response_mismatch')


def assert_certify_request(node):
    """Check Certify of the Public Key and Certificate Sign capability."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    operations = [[2], [18], [6], [19], [20], [20], [20]]
    if ([numbers(item, 0x42005C) for item in items] != operations or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in CERTIFY_ITEM_IDS]):
        raise ValueError('fixture_certify_batch_mismatch')
    create_payloads = [child for child in items[0][2]
                       if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(create_payloads) != 1:
        raise ValueError('fixture_certify_create_pair_payload_missing')
    create_fields = create_payloads[0][2]
    if [field[0] for field in create_fields] != [0x420126, 0x420127, 0x420128]:
        raise ValueError('fixture_certify_create_pair_attribute_fields_mismatch')
    common, private, public = create_fields
    if (numbers(common, 0x420028) != [4] or numbers(common, 0x42002A) != [2048] or
            numbers(private, 0x42002C) != [4096] or numbers(public, 0x42002C) != [2]):
        raise ValueError('fixture_certify_key_pair_attributes_mismatch')
    if ([numbers(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] !=
            [[5], [6], [5], [ID_PLACEHOLDER_ENUMERATION], [5], [6]]):
        raise ValueError('fixture_certify_unique_identifier_mapping_mismatch')
    certify_payloads = [child for child in items[2][2]
                        if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(certify_payloads) != 1:
        raise ValueError('fixture_certify_payload_missing')
    fields = certify_payloads[0][2]
    identifier_fields = [field for field in fields
                         if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 5]
    attribute_fields = [field for field in fields
                        if field[0] == 0x420125 and field[1] == 1]
    if (len(fields) != 2 or len(identifier_fields) != 1 or
            int.from_bytes(identifier_fields[0][2], 'big') != 6 or len(attribute_fields) != 1):
        raise ValueError('fixture_certify_public_key_selector_mismatch')
    attributes = attribute_fields[0][2]
    certificate_types = [field for field in attributes
                         if field[0] == 0x42001D and field[1] == 5]
    if (len(attributes) != 1 or len(certificate_types) != 1 or
            int.from_bytes(certificate_types[0][2], 'big') != 1):
        raise ValueError('fixture_certify_certificate_type_attribute_mismatch')


def verify_certify_response(response_tree):
    """Check the generated certificate ID and all three cleanup results."""
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    operations = [[2], [18], [6], [19], [20], [20], [20]]
    if ([numbers(item, 0x42005C) for item in items] != operations or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in CERTIFY_ITEM_IDS] or
            [numbers(item, 0x42007F) for item in items] != [[0]] * 7):
        raise ValueError('fixture_certify_response_lifecycle_mismatch')
    create_payloads = [child for child in items[0][2]
                       if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(create_payloads) != 1:
        raise ValueError('fixture_certify_response_create_pair_payload_missing')
    pair_fields = create_payloads[0][2]
    private_ids = [field for field in pair_fields if field[0] == 0x420066 and field[1] == 7]
    public_ids = [field for field in pair_fields if field[0] == 0x42006F and field[1] == 7]
    if (len(pair_fields) != 2 or len(private_ids) != 1 or len(public_ids) != 1 or
            private_ids[0][2] != b'fixture-private-key-1' or
            public_ids[0][2] != b'fixture-public-key-1'):
        raise ValueError('fixture_certify_pair_response_identifiers_mismatch')
    certify_payloads = [child for child in items[2][2]
                        if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    certificate_ids = [field for field in certify_payloads[0][2]
                       if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7] \
        if len(certify_payloads) == 1 else []
    if (len(certify_payloads) != 1 or len(certify_payloads[0][2]) != 1 or
            len(certificate_ids) != 1 or certificate_ids[0][2] != b'fixture-certificate-1'):
        raise ValueError('fixture_certify_response_certificate_identifier_mismatch')
    expected_identifiers = [
        ['fixture-private-key-1'], ['fixture-certificate-1'],
        ['fixture-private-key-1'], ['fixture-certificate-1'],
        ['fixture-private-key-1'], ['fixture-public-key-1'],
    ]
    if [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] != expected_identifiers:
        raise ValueError('fixture_certify_response_cleanup_identifiers_mismatch')


def assert_pkcs11_request(node):
    """Check the isolated Create/Activate/PKCS#11/Revoke/Destroy request."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[1], [18], [51], [19], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in PKCS11_ITEM_IDS] or
            numbers(items[0], 0x420057) != [2] or numbers(items[0], 0x420028) != [3] or
            numbers(items[0], 0x42002A) != [128] or numbers(items[0], 0x42002C) != [4] or
            any(not is_id_placeholder_identifier(identifier)
                for item in items[1:]
                for identifier in descendants(item, UNIQUE_IDENTIFIER_TAG))):
        raise ValueError('fixture_pkcs11_batch_lifecycle_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_pkcs11_request_payload_missing')
    fields = payloads[0][2]
    functions = [field for field in fields if field[0] == 0x42015A and field[1] == 1]
    interfaces = [field for field in fields if field[0] == 0x420159 and field[1] == 1]
    correlations = [field for field in fields if field[0] == 0x4200D6 and field[1] == 8]
    inputs = [field for field in fields if field[0] == 0x42015B and field[1] == 1]
    expected_function = [(0x42015A, 2, (-12345).to_bytes(4, 'big', signed=True))]
    expected_interface = [(0x420159, 7, b'test-interface')]
    expected_input = [(0x42015B, 8, PKCS11_INPUT)]
    if ([field[0] for field in fields] != [0x420159, 0x42015A, 0x4200D6, 0x42015B] or
            len(functions) != 1 or functions[0][2] != expected_function or
            len(interfaces) != 1 or interfaces[0][2] != expected_interface or
            [(field[1], field[2]) for field in correlations] !=
            [(8, PKCS11_REQUEST_CORRELATION)] or
            len(inputs) != 1 or inputs[0][2] != expected_input):
        raise ValueError('fixture_pkcs11_request_fields_mismatch')


def verify_pkcs11_response(response_tree):
    """Check opaque typed fields and unknown values from the controlled response."""
    verify_lifecycle(RECIPES['test-op-pkcs#11'][0], response_tree)
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if [byte_string_values(item, 0x420093) for item in items] != [
            [item_id] for item_id in PKCS11_ITEM_IDS]:
        raise ValueError('fixture_pkcs11_response_item_ids_mismatch')
    payloads = [child for child in items[2][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_pkcs11_response_payload_missing')
    fields = payloads[0][2]
    functions = [field for field in fields if field[0] == 0x42015A and field[1] == 1]
    correlations = [field for field in fields if field[0] == 0x4200D6 and field[1] == 8]
    outputs = [field for field in fields if field[0] == 0x42015C and field[1] == 1]
    returns = [field for field in fields if field[0] == 0x42015D and field[1] == 1]
    extensions = [field for field in fields if field[0] == 0x541234 and field[1] == 2]
    expected_function = [(0x42015A, 2, (-12345).to_bytes(4, 'big', signed=True))]
    expected_output = [(0x42015C, 8, PKCS11_OUTPUT)]
    expected_return = [(0x42015D, 5, (0xFFFF1234).to_bytes(4, 'big'))]
    if ([field[0] for field in fields] != [0x42015A, 0x4200D6, 0x42015C, 0x42015D, 0x541234] or
            len(functions) != 1 or functions[0][2] != expected_function or
            [(field[1], field[2]) for field in correlations] !=
            [(8, PKCS11_RESPONSE_CORRELATION)] or
            len(outputs) != 1 or outputs[0][2] != expected_output or
            len(returns) != 1 or returns[0][2] != expected_return or
            [(field[1], int.from_bytes(field[2], 'big', signed=True)) for field in extensions] !=
            [(2, 7)]):
        raise ValueError('fixture_pkcs11_response_fields_mismatch')


def assert_obtain_lease_request(node):
    """Check the catalogued Create/Obtain Lease/Revoke/Destroy batch."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[1], [16], [19], [20]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[item_id] for item_id in OBTAIN_LEASE_ITEM_IDS] or
            numbers(items[0], 0x420057) != [2] or numbers(items[0], 0x420028) != [3] or
            numbers(items[0], 0x42002A) != [128] or numbers(items[0], 0x42002C) != [4]):
        raise ValueError('fixture_obtain_lease_batch_mismatch')
    payloads = [child for child in items[1][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1 or payloads[0][2]:
        raise ValueError('fixture_obtain_lease_identifier_mismatch')


def verify_obtain_lease_response(response_tree):
    """Check Table 269 response fields, their values, and lifecycle teardown."""
    verify_lifecycle(RECIPES['test-op-obtain-lease'][0], response_tree)
    items = [child for child in response_tree[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if [byte_string_values(item, 0x420093) for item in items] != [
            [item_id] for item_id in OBTAIN_LEASE_ITEM_IDS]:
        raise ValueError('fixture_obtain_lease_response_item_ids_mismatch')
    payloads = [child for child in items[1][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if len(payloads) != 1:
        raise ValueError('fixture_obtain_lease_response_payload_missing')
    fields = payloads[0][2]
    identifiers = [field for field in fields if field[0] == UNIQUE_IDENTIFIER_TAG and field[1] == 7]
    lease_times = [field for field in fields if field[0] == 0x420049 and field[1] == 10]
    last_changes = [field for field in fields if field[0] == 0x420048 and field[1] == 9]
    extensions = [field for field in fields if field[0] == 0x541234 and field[1] == 2]
    if ([field[0] for field in fields] != [0x420094, 0x420049, 0x420048, 0x541234] or
            len(identifiers) != 1 or identifiers[0][2] != b'fixture-key-1' or
            len(lease_times) != 1 or lease_times[0][2] != (3600).to_bytes(4, 'big', signed=True) or
            len(last_changes) != 1 or last_changes[0][2] !=
            (1_700_000_000).to_bytes(8, 'big', signed=True) or
            len(extensions) != 1 or int.from_bytes(extensions[0][2], 'big', signed=True) != -7):
        raise ValueError('fixture_obtain_lease_response_fields_mismatch')


def assert_recipe(node, case_id, version, recipe=None, attribute_fixture=None,
                  constructor_variant=None, export_field=None):
    selected_recipe = recipe or RECIPES[case_id]
    expected, _ = selected_recipe
    observed_operations = tuple(numbers(node, 0x42005C))
    if observed_operations != expected:
        raise ValueError(
            f'fixture_operation_order_mismatch:expected={expected}:observed={observed_operations}')
    assert_placeholder_usage(node, case_id)
    header = next(descendants(node, 0x420077), node)
    if numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])]:
        raise ValueError('fixture_protocol_version_mismatch')
    if numbers(node, 0x42000D) != [len(expected)]:
        raise ValueError('fixture_batch_count_mismatch')
    if selected_recipe[1] in ('cancel_async', 'cancel_poll_race'):
        assert_cancel_request(node)
    if selected_recipe[1] == 'archive':
        archive_items = [item for item in node[2]
                         if item[0] == BATCH_ITEM_TAG and numbers(item, 0x42005C) == [21]]
        payloads = [child for child in archive_items[0][2]
                    if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] \
            if len(archive_items) == 1 else []
        identifiers = [child for child in payloads[0][2]
                       if child[0] == UNIQUE_IDENTIFIER_TAG] if len(payloads) == 1 else []
        if len(identifiers) != 1 or not is_id_placeholder_identifier(identifiers[0]):
            raise ValueError('fixture_archive_identifier_mismatch')
    if selected_recipe[1] == 'encrypt':
        assert_crypto_request(node, False)
    if selected_recipe[1] == 'decrypt':
        assert_crypto_request(node, True)
    if selected_recipe[1] == 'mac':
        assert_mac_request(node)
    if selected_recipe[1] == 'mac_verify':
        assert_mac_verify_request(node)
    if selected_recipe[1] in ('sign', 'signature_verify'):
        assert_signature_request(node, selected_recipe[1] == 'signature_verify')
    if selected_recipe[1] == 'create_key_pair':
        assert_create_key_pair_request(node)
    if selected_recipe[1] == 'certify':
        assert_certify_request(node)
    if selected_recipe[1] == 'pkcs11':
        assert_pkcs11_request(node)
    if selected_recipe[1] == 'obtain_lease':
        assert_obtain_lease_request(node)
    if (case_id in {
            'test-norm-payload-client-add-attribute-request-20-002',
            'test-norm-payload-client-add-attribute-request-21-002'} and
            not request_add_attribute_has_new_attribute(node)):
        raise ValueError('fixture_add_attribute_new_attribute_missing')
    if selected_recipe[1] == 'modify_attribute' and case_id.startswith(
            'test-norm-payload-client-modify-attribute-request-'):
        current, new = request_modify_attribute_fields(
            next(item for item in node[2] if item[0] == BATCH_ITEM_TAG and
                 item[1] == 1 and numbers(item, 0x42005C) == [14]))
        current_expected = case_id.endswith('-002')
        if ((current is not None) != current_expected or new[0] != 'Comment' or
                new[1] != b'KMIPKit modified interoperability comment' or
                (current_expected and current != ('Comment', b'KMIPKit interoperability comment'))):
            raise ValueError('fixture_modify_attribute_request_fields_mismatch')
    if selected_recipe[1] == 'adjust_attribute':
        adjust_item = next((item for item in node[2]
                            if item[0] == BATCH_ITEM_TAG and item[1] == 1 and
                            numbers(item, 0x42005C) == [48]), None)
        if (adjust_item is None or
                text_values(adjust_item, UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1'] or
                request_adjust_attribute_fields(adjust_item) != ('KMIPKit', 'Counter', 1, 5)):
            raise ValueError('fixture_adjust_attribute_request_fields_mismatch')
    if selected_recipe[1] == 'export':
        export_item = next((item for item in node[2]
                            if item[0] == BATCH_ITEM_TAG and item[1] == 1 and
                            numbers(item, 0x42005C) == [43]), None)
        payloads = [child for child in export_item[2]
                    if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if export_item else []
        fields = payloads[0][2] if len(payloads) == 1 else []
        identifiers = [child for child in fields if child[0] == UNIQUE_IDENTIFIER_TAG]
        expected_field_tag = {
            'Key Format Type': 0x420042,
            'Key Wrap Type': 0x4200F8,
            'Key Compression Type': 0x420041,
            'Key Wrapping Specification': 0x420047,
        }.get(export_field)
        field_is_present = constructor_variant == 'present'
        expected_tags = ([] if export_field == 'Unique Identifier' and not field_is_present else
                         [UNIQUE_IDENTIFIER_TAG])
        if expected_field_tag is not None and field_is_present:
            expected_tags.append(expected_field_tag)
        if (len(payloads) != 1 or [field[0] for field in fields] != expected_tags or
                (identifiers and not is_id_placeholder_identifier(identifiers[0])) or
                (expected_tags and expected_tags[0] == UNIQUE_IDENTIFIER_TAG and len(identifiers) != 1)):
            raise ValueError('fixture_export_request_fields_mismatch')
        if expected_field_tag is not None and field_is_present:
            selected = next(field for field in fields if field[0] == expected_field_tag)
            if export_field == 'Key Format Type' and (selected[1] != 5 or selected[2] != (1).to_bytes(4, 'big')):
                raise ValueError('fixture_export_key_format_type_mismatch')
            if export_field == 'Key Wrap Type' and (selected[1] != 5 or selected[2] != (1).to_bytes(4, 'big')):
                raise ValueError('fixture_export_key_wrap_type_mismatch')
            if export_field == 'Key Compression Type' and (selected[1] != 5 or selected[2] != (1).to_bytes(4, 'big')):
                raise ValueError('fixture_export_key_compression_type_mismatch')
            if export_field == 'Key Wrapping Specification':
                spec_fields = selected[2] if selected[1] == 1 else []
                method = [field for field in spec_fields if field[0] == 0x42009E and field[1] == 5]
                key_info = [field for field in spec_fields if field[0] == 0x420036 and field[1] == 1]
                identifiers = [field for field in key_info[0][2] if field[0] == UNIQUE_IDENTIFIER_TAG] if len(key_info) == 1 else []
                if (len(spec_fields) != 2 or len(method) != 1 or method[0][2] != (1).to_bytes(4, 'big') or
                        len(key_info) != 1 or len(identifiers) != 1 or
                        identifiers[0][2] != b'fixture-wrapping-key'):
                    raise ValueError('fixture_export_key_wrapping_specification_mismatch')
    if expected[0] == 1:
        object_type = 4 if case_id == 'test-obj-private-key' else 3 if case_id == 'test-obj-public-key' else 2
        if numbers(node, 0x420057) != [object_type]:
            raise ValueError('fixture_object_type_mismatch')
        expected_algorithms = ([3, 3] if selected_recipe[1] in ('encrypt', 'decrypt') else
                               [9, 9] if selected_recipe[1] in ('mac', 'mac_verify') else
                               [4 if object_type in (3, 4) else 3])
        if numbers(node, 0x420028) != expected_algorithms:
            raise ValueError('fixture_algorithm_mismatch')
        expected_key_length = 256 if selected_recipe[1] in ('mac', 'mac_verify') else (
            2048 if object_type in (3, 4) else 128)
        if numbers(node, 0x42002A) != [expected_key_length]:
            raise ValueError('fixture_key_length_mismatch')
        if numbers(node, 0x42000E) != [1]:
            raise ValueError('fixture_cleanup_continuation_missing')
        implicit_target = case_id in ('test-op-get-attributes', 'test-op-get-attribute-list',
            'test-norm-payload-client-get-attribute-list-request-20-001',
            'test-norm-payload-client-get-attribute-list-request-21-001',
            'test-batch-create-activate-destroy', 'test-attr-cryptographic-algorithm',
            'test-attr-cryptographic-length', 'test-op-get-constraints',
            'test-norm-payload-client-get-usage-allocation-request-20-001',
            'test-norm-payload-client-get-usage-allocation-request-21-001',
            'test-op-pkcs#11', 'test-op-obtain-lease') or attribute_fixture is not None
        implicit_target = implicit_target or (
            selected_recipe[1] == 'export' and export_field == 'Unique Identifier' and
            constructor_variant == 'omitted')
        minimum_unique_identifiers = 0 if case_id == 'test-batch-placeholder-chain-resolves' else (
            len(expected) - 1 - int(implicit_target))
        if len(numbers(node, 0x420094)) < minimum_unique_identifiers:
            raise ValueError('fixture_id_placeholder_missing')
        if len(list(descendants(node, 0x420093))) != len(expected):
            raise ValueError('fixture_batch_item_identity_missing')
        if case_id in {
                'test-norm-payload-client-get-attribute-list-request-20-001',
                'test-norm-payload-client-get-attribute-list-request-21-001'}:
            attribute_list_items = [item for item in node[2]
                                    if item[0] == BATCH_ITEM_TAG and item[1] == 1 and
                                    numbers(item, 0x42005C) == [12]]
            if (len(attribute_list_items) != 1 or
                    text_values(attribute_list_items[0], UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']):
                raise ValueError('fixture_get_attribute_list_identifier_mismatch')
        if selected_recipe[1] == 'batch_placeholder_chain':
            items = [child for child in node[2] if child[0] == 0x42000F and child[1] == 1]
            if ([numbers(item, 0x42005C) for item in items] != [[1], [18], [10]] or
                    [byte_string_values(item, 0x420093) for item in items] !=
                    [[b'placeholder-create'], [b'placeholder-activate'], [b'placeholder-get']]):
                raise ValueError('fixture_placeholder_chain_shape_mismatch')
            if numbers(node, 0x42000E) != [1]:
                raise ValueError('fixture_placeholder_chain_continue_option_missing')
    if selected_recipe[1] == 'batch_order_placeholder_two_item':
        items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
        is_required = case_id == 'test-batch-order-required-placeholder-resolution'
        if ([numbers(item, 0x42005C) for item in items] != [[1], [18]] or
                [byte_string_values(item, 0x420093) for item in items] !=
                [[BATCH_ORDER_PLACEHOLDER_ITEM_IDS[0]], [BATCH_ORDER_PLACEHOLDER_ITEM_IDS[1]]] or
                numbers(node, 0x420010) != ([1] if is_required else [])):
            raise ValueError('fixture_batch_order_placeholder_shape_mismatch')
    if selected_recipe[1] == 'get_usage_allocation_uid_variants':
        items = [child for child in node[2]
                 if child[0] == BATCH_ITEM_TAG and child[1] == 1]
        identifiers = []
        item_ids = []
        counts = []
        for item in items[1:]:
            payloads = [child for child in item[2]
                        if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
            if len(payloads) != 1:
                raise ValueError('fixture_usage_allocation_identifier_variants_mismatch')
            identifiers.append([child[2].decode('utf-8') for child in payloads[0][2]
                                if child[0] == UNIQUE_IDENTIFIER_TAG and child[1] == 7])
            item_ids.append(byte_string_values(item, 0x420093))
            counts.append([child for child in payloads[0][2] if child[0] == 0x420096])
        locate_payloads = [child for child in items[0][2]
                           if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if items else []
        if (len(items) != 3 or [numbers(item, 0x42005C) for item in items] !=
                [[8], [17], [17]] or
                [byte_string_values(item, 0x420093) for item in items] !=
                [[b'usage-locate'], [b'usage-explicit'], [b'usage-placeholder']] or
                identifiers != [['fixture-key-1'], []] or
                len(locate_payloads) != 1 or numbers(locate_payloads[0], 0x42004F) != [3] or
                item_ids != [[b'usage-explicit'], [b'usage-placeholder']] or
                any(len(count) != 1 or count[0][1] != 3 or len(count[0][2]) != 8 or
                    int.from_bytes(count[0][2], 'big', signed=True) != 1 for count in counts)):
            raise ValueError('fixture_usage_allocation_identifier_variants_mismatch')
    if selected_recipe[1] == 'import_secret_data':
        items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
        payloads = [child for child in items[0][2]
                    if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if items else []
        payload = payloads[0] if len(payloads) == 1 else None
        opaque_object = next((child for child in payload[2]
                              if child[0] == 0x42005B and child[1] == 1), None) if payload else None
        attributes = next((child for child in payload[2]
                           if child[0] == 0x420125 and child[1] == 1), None) if payload else None
        if ([numbers(item, 0x42005C) for item in items] != [[42], [20]] or
                [byte_string_values(item, 0x420093) for item in items] !=
                [[item_id] for item_id in IMPORT_ITEM_IDS] or
                payload is None or
                text_values(payload, UNIQUE_IDENTIFIER_TAG) != [IMPORT_IDENTIFIER.decode()] or
                numbers(payload, 0x420057) != [8] or
                [child[0] for child in payload[2]] != [0x420094, 0x420057, 0x420125, 0x42005B] or
                attributes is None or attributes[2] or
                opaque_object is None or numbers(opaque_object, 0x420059) != [0x80000001] or
                byte_string_values(opaque_object, 0x42005A) != [IMPORT_OBJECT_DATA]):
            raise ValueError('fixture_import_opaque_object_shape_mismatch')
    if expected[0] == 3:
        register_item = next(item for item in node[2]
                             if item[0] == 0x42000F and numbers(item, 0x42005C) == [3])
        request_payload = next((item for item in register_item[2]
                                if item[0] == 0x420079 and item[1] == 1), None)
        attributes = next((item for item in request_payload[2]
                           if item[0] == 0x420125 and item[1] == 1), None) if request_payload else None
        if selected_recipe[1] == 'batch_register_crypto':
            assert_registered_crypto_batch(node)
        elif selected_recipe[1] == 'register_opaque_object':
            opaque_object = next(descendants(register_item, 0x42005B), None)
            opaque_data_type = next(descendants(opaque_object, 0x420059), None) \
                if opaque_object is not None else None
            opaque_data_value = next(descendants(opaque_object, 0x42005A), None) \
                if opaque_object is not None else None
            if (numbers(node, 0x420057) != [8] or opaque_object is None or
                    opaque_object[1] != 1 or
                    [field[0] for field in opaque_object[2]] != [0x420059, 0x42005A] or
                    opaque_data_type is None or opaque_data_type[1] != 5 or
                    numbers(opaque_object, 0x420059) != [0x80000001] or
                    opaque_data_value is None or opaque_data_value[1] != 8 or
                    opaque_data_value[2] != b'opaque-register-data'):
                raise ValueError('fixture_register_opaque_object_shape_mismatch')
        elif selected_recipe[1] == 'register_pgp_key':
            pgp_key = next(descendants(register_item, 0x4200BD), None)
            version = next(descendants(pgp_key, 0x4200BE), None) \
                if pgp_key is not None else None
            key_block = next(descendants(pgp_key, 0x420040), None) \
                if pgp_key is not None else None
            key_value = next(descendants(key_block, 0x420045), None) \
                if key_block is not None else None
            key_format = next(descendants(key_block, 0x420042), None) \
                if key_block is not None else None
            key_material = next(descendants(key_value, 0x420043), None) \
                if key_value is not None else None
            if (numbers(node, 0x420057) != [9] or pgp_key is None or
                    pgp_key[1] != 1 or [field[0] for field in pgp_key[2]] != [0x4200BE, 0x420040] or
                    version is None or version[1] != 2 or version[2] != b'\x00\x00\x00\x04' or
                    key_block is None or key_block[1] != 1 or key_format is None or
                    key_format[1] != 5 or numbers(key_block, 0x420042) != [1] or
                    key_value is None or key_value[1] != 1 or key_material is None or
                    key_material[1] != 8 or key_material[2] != b'\xa1' or
                    attributes is None or attributes[1] != 1 or
                    numbers(attributes, 0x420028) != [4] or
                    numbers(attributes, 0x42002A) != [2048] or
                    numbers(attributes, 0x42002C) != [1]):
                raise ValueError('fixture_register_pgp_key_shape_mismatch')
        elif selected_recipe[1] == 'register_secret_data':
            attributes = next(descendants(register_item, 0x420125), None)
            secret_data = next(descendants(register_item, 0x420085), None)
            secret_data_type = next(descendants(secret_data, 0x420086), None) \
                if secret_data is not None else None
            key_block = next(descendants(secret_data, 0x420040), None) \
                if secret_data is not None else None
            key_format = next(descendants(key_block, 0x420042), None) \
                if key_block is not None else None
            key_value = next(descendants(key_block, 0x420045), None) \
                if key_block is not None else None
            key_material = next(descendants(key_value, 0x420043), None) \
                if key_value is not None else None
            if (numbers(node, 0x420057) != [7] or secret_data is None or
                    attributes is None or attributes[1] != 1 or
                    numbers(attributes, 0x42002C) != [512] or
                    secret_data[1] != 1 or
                    [field[0] for field in secret_data[2]] != [0x420086, 0x420040] or
                    secret_data_type is None or secret_data_type[1] != 5 or
                    numbers(secret_data, 0x420086) != [1] or
                    key_block is None or key_block[1] != 1 or key_format is None or
                    key_format[1] != 5 or numbers(key_block, 0x420042) != [1] or
                    key_value is None or key_value[1] != 1 or key_material is None or
                    key_material[1] != 8 or key_material[2] != SECRET_DATA_MATERIAL):
                raise ValueError('fixture_register_secret_data_shape_mismatch')
        elif selected_recipe[1] == 'register_certificate_request':
            certificate_request = next(descendants(register_item, 0x420018), None)
            request_value = next(descendants(register_item, 0x420140), None)
            if (numbers(node, 0x420057) != [10] or certificate_request is None or
                    certificate_request[1] != 1 or numbers(certificate_request, 0x420019) != [2]):
                raise ValueError('fixture_register_certificate_request_shape_mismatch')
            if request_value is None or request_value[1] != 8 or not request_value[2]:
                raise ValueError('fixture_register_certificate_request_value_mismatch')
        else:
            certificate = next(descendants(register_item, 0x420013), None)
            certificate_value = next(descendants(register_item, 0x42001E), None)
            if certificate is None or certificate[1] != 1 or numbers(certificate, 0x42001D) != [1]:
                raise ValueError('fixture_register_certificate_shape_mismatch')
            if certificate_value is None or certificate_value[1] != 8:
                raise ValueError('fixture_register_certificate_value_mismatch')
            if attributes is None or attributes[1] != 1:
                raise ValueError('fixture_register_certificate_attributes_missing')
            certificate_attribute_tags = {item[0] for item in attributes[2]}
            required_certificate_attributes = {0x42001D, 0x420028, 0x42002A, 0x42002C, 0x4200AD}
            certificate_length = numbers(attributes, 0x4200AD)
            if not required_certificate_attributes <= certificate_attribute_tags:
                raise ValueError('fixture_register_certificate_attribute_tags_mismatch')
            if (numbers(attributes, 0x42001D) != [1] or numbers(attributes, 0x420028) != [4] or
                    numbers(attributes, 0x42002A) != [2048] or numbers(attributes, 0x42002C) != [3]):
                raise ValueError('fixture_register_certificate_attribute_values_mismatch')
            if len(certificate_length) != 1 or certificate_length[0] != len(certificate_value[2]):
                raise ValueError('fixture_register_certificate_length_mismatch')
    if case_id == 'test-attr-cryptographic-algorithm' and numbers(node, 0x42013B) != [0x420028]:
        raise ValueError('fixture_attribute_reference_mismatch')
    if case_id == 'test-attr-cryptographic-length' and numbers(node, 0x42013B) != [0x42002A]:
        raise ValueError('fixture_attribute_reference_mismatch')
    if selected_recipe[1] in ('add_attribute', 'set_attribute'):
        operation = 13 if selected_recipe[1] == 'add_attribute' else 49
        items = [item for item in node[2]
                 if item[0] == BATCH_ITEM_TAG and numbers(item, 0x42005C) == [operation]]
        if len(items) != 1:
            raise ValueError('fixture_attribute_mutation_item_missing')
        payloads = [item for item in items[0][2]
                    if item[0] == REQUEST_PAYLOAD_TAG and item[1] == 1]
        comments = [item for item in descendants(payloads[0], 0x4200FD) if item[1] == 7] \
            if len(payloads) == 1 else []
        if len(comments) != 1 or comments[0][2] != b'KMIPKit interoperability comment':
            observed = [(f'{item[0]:06X}', item[1])
                        for item in descendants(payloads[0], 0x4200FD)] if len(payloads) == 1 else []
            summary = [(len(item[2]), hashlib.sha256(item[2]).hexdigest()[:12])
                       for item in comments]
            raise ValueError(f'fixture_attribute_mutation_comment_mismatch:{observed}:{summary}')
    if selected_recipe[1] == 'set_constraints':
        items = [item for item in node[2]
                 if item[0] == BATCH_ITEM_TAG and numbers(item, 0x42005C) == [55]]
        payloads = [item for item in items[0][2]
                    if item[0] == REQUEST_PAYLOAD_TAG and item[1] == 1] if len(items) == 1 else []
        constraints = [item for item in payloads[0][2]
                       if item[0] == 0x420168 and item[1] == 1] if len(payloads) == 1 else []
        if len(constraints) != 1 or constraints[0][2]:
            raise ValueError('fixture_set_constraints_shape_mismatch')
    if selected_recipe[1] == 'set_defaults':
        items = [item for item in node[2]
                 if item[0] == BATCH_ITEM_TAG and numbers(item, 0x42005C) == [54]]
        payloads = [item for item in items[0][2]
                    if item[0] == REQUEST_PAYLOAD_TAG and item[1] == 1] if len(items) == 1 else []
        if len(payloads) != 1 or list(descendants(payloads[0], 0x420152)):
            raise ValueError('fixture_set_defaults_omission_mismatch')
    if selected_recipe[1] == 'delete_attribute':
        items = [item for item in node[2]
                 if item[0] == BATCH_ITEM_TAG and numbers(item, 0x42005C) == [15]]
        if (len(items) != 1 or numbers(items[0], 0x42013B) != [0x4200FD] or
                text_values(items[0], UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']):
            raise ValueError('fixture_delete_attribute_reference_mismatch')
    if attribute_fixture is not None:
        target_item = next(item for item in node[2]
                           if item[0] == 0x42000F and numbers(item, 0x42005C) == [11])
        if attribute_fixture['attribute_reference_kind'] == 'tag':
            matches = numbers(target_item, 0x42013B) == [
                int(attribute_fixture['attribute_reference_tag'], 16)]
        elif attribute_fixture['attribute_reference_kind'] == 'name':
            matches = (text_values(target_item, 0x42000A) == [
                attribute_fixture['attribute_reference_name']] and
                text_values(target_item, 0x42009D) == [
                    attribute_fixture['attribute_vendor_identification']])
        else:
            matches = (not list(descendants(target_item, 0x42013B)) and
                       not text_values(target_item, 0x42000A) and
                       not text_values(target_item, 0x42009D))
        if not matches:
            raise ValueError('fixture_attribute_reference_mismatch')
    if selected_recipe[1] in ('batch_single_item_options_omitted',
                              'batch_single_item_default_continuation'):
        items = [child for child in node[2]
                 if child[0] == 0x42000F and child[1] == 1]
        if (numbers(node, 0x42000D) != [1] or len(items) != 1 or
                list(descendants(node, 0x420011)) or list(descendants(node, 0x42000E))):
            raise ValueError('fixture_single_item_batch_options_mismatch')
        if numbers(items[0], 0x42005C) != [10]:
            raise ValueError('fixture_single_item_batch_operation_mismatch')
        if text_values(items[0], 0x420094) != ['fixture-key-1']:
            raise ValueError('fixture_single_item_batch_identifier_mismatch')
        if byte_string_values(items[0], 0x420093) != [b'interop-get']:
            raise ValueError('fixture_single_item_batch_item_id_mismatch')
    if selected_recipe[1] in ('batch_count_two_read_items', 'batch_order_false_two_read_items',
                              'batch_continue_after_failure', 'batch_stop_on_first_failure'):
        items = [child for child in node[2]
                 if child[0] == 0x42000F and child[1] == 1]
        failing_batch = selected_recipe[1] in ('batch_continue_after_failure',
                                                'batch_stop_on_first_failure')
        first_id = 'missing-key' if failing_batch else 'fixture-key-1'
        expected_operations = [[10], [10]] if selected_recipe[1] == 'batch_stop_on_first_failure' else [[10], [12]]
        if (numbers(node, 0x42000D) != [2] or
                [numbers(item, 0x42005C) for item in items] != expected_operations or
                [text_values(item, 0x420094) for item in items] != [[first_id], ['fixture-key-1']] or
                [byte_string_values(item, 0x420093) for item in items] !=
                [[b'item-a'], [b'item-b']]):
            raise ValueError('fixture_two_read_batch_shape_mismatch')
        if selected_recipe[1] == 'batch_order_false_two_read_items':
            if numbers(node, 0x420010) != [0]:
                raise ValueError('fixture_batch_order_option_false_missing')
        elif list(descendants(node, 0x420010)):
            raise ValueError('fixture_unrequested_batch_order_option_present')
        if selected_recipe[1] == 'batch_continue_after_failure':
            if numbers(node, 0x42000E) != [1]:
                raise ValueError('fixture_batch_continue_option_missing')
        elif selected_recipe[1] == 'batch_stop_on_first_failure':
            if numbers(node, 0x42000E) != [2]:
                raise ValueError('fixture_batch_stop_option_missing')
    if selected_recipe[1] == 'query' and numbers(node, 0x420074) != [1]:
        raise ValueError('fixture_query_function_mismatch')
    if case_id == 'test-norm-payload-client-create-request-21-003':
        masks = list(descendants(node, 0x42015F))
        if len(masks) != 1 or masks[0][1] != 1 or masks[0][2]:
            raise ValueError('fixture_create_storage_masks_mismatch')
    if case_id == 'test-norm-payload-client-query-request-21-002':
        groups = list(descendants(node, 0x420166))
        if (len(groups) != 1 or groups[0][1] != 1
                or [item[2] for item in groups[0][2] if item[0] == 0x420056] != [b'KMIPKit-Test-Group']):
            raise ValueError('fixture_query_object_groups_mismatch')
    if case_id == 'test-op-locate' and numbers(node, 0x42004F) != [3]:
        raise ValueError('fixture_locate_bound_missing')
    if case_id == 'test-op-log' and not list(descendants(node, 0x420141)):
        raise ValueError('fixture_log_message_missing')
    if selected_recipe[1] == 'hash':
        if numbers(node, 0x420038) != [6] or [item[2] for item in descendants(node, 0x4200C2)] != [b'abc']:
            raise ValueError('fixture_hash_input_mismatch')
    expected_rng_length = 32 if case_id.endswith('-rng-32') else 16
    if selected_recipe[1] == 'rng_retrieve' and numbers(node, 0x4200C4) != [expected_rng_length]:
        raise ValueError('fixture_rng_length_mismatch')
    if selected_recipe[1] == 'rng_seed' and [item[2] for item in descendants(node, 0x4200C2)] != [bytes(range(expected_rng_length))]:
        raise ValueError('fixture_rng_seed_mismatch')


def assert_add_attribute_setup_request(node, version):
    header = next(descendants(node, 0x420077), node)
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [1] or
            [numbers(item, 0x42005C) for item in items] != [[1]] or
            numbers(node, 0x420057) != [2] or numbers(node, 0x420028) != [3] or
            numbers(node, 0x42002A) != [128]):
        raise ValueError('fixture_add_attribute_setup_request_mismatch')


def assert_add_attribute_cleanup_request(node, version):
    header = next(descendants(node, 0x420077), node)
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [2] or
            [numbers(item, 0x42005C) for item in items] != [[19], [20]] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items] !=
            [['fixture-key-1'], ['fixture-key-1']] or numbers(node, 0x42000E) != [1] or
            numbers(node, 0x420010) != [1]):
        raise ValueError('fixture_add_attribute_cleanup_request_mismatch')


def assert_placeholder_cleanup_request(node, version):
    items = [child for child in node[2] if child[0] == 0x42000F and child[1] == 1]
    if numbers(node, 0x42000D) != [2] or [numbers(item, 0x42005C) for item in items] != [[19], [20]]:
        raise ValueError('fixture_placeholder_chain_cleanup_operation_mismatch')
    header = next(descendants(node, 0x420077), node)
    if numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])]:
        raise ValueError('fixture_placeholder_chain_cleanup_version_mismatch')
    if numbers(node, 0x42000E) != [1] or numbers(node, 0x420010) != [1]:
        raise ValueError('fixture_placeholder_chain_cleanup_option_mismatch')
    if [text_values(item, 0x420094) for item in items] != [['fixture-key-1'], ['fixture-key-1']]:
        raise ValueError('fixture_placeholder_chain_cleanup_identifier_mismatch')
    if [byte_string_values(item, 0x420093) for item in items] != [
            [b'placeholder-revoke-cleanup'], [b'placeholder-destroy-cleanup']]:
        raise ValueError('fixture_placeholder_chain_cleanup_item_id_mismatch')


def verify_placeholder_chain_response(node, operations):
    items = [child for child in node[2] if child[0] == 0x42000F and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[value] for value in operations] or
            [numbers(item, 0x42007F) for item in items] != [[0] for _ in operations] or
            [text_values(item, 0x420094) for item in items] !=
            [['fixture-key-1'] for _ in operations]):
        raise ValueError('fixture_placeholder_chain_response_mismatch')
    if tuple(operations) == PLACEHOLDER_CHAIN_OPERATIONS:
        get_item = items[2]
        if not list(descendants(get_item, 0x42008F)):
            raise ValueError('fixture_placeholder_chain_get_object_missing')


def verify_batch_order_placeholder_response(node):
    """Verify Create then Activate responses remain correlated and target one object."""
    items = [child for child in node[2] if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if ([numbers(item, 0x42005C) for item in items] != [[1], [18]] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0]] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[BATCH_ORDER_PLACEHOLDER_ITEM_IDS[0]], [BATCH_ORDER_PLACEHOLDER_ITEM_IDS[1]]] or
            [text_values(item, 0x420094) for item in items] !=
            [['fixture-key-1'], ['fixture-key-1']]):
        raise ValueError('fixture_batch_order_placeholder_response_mismatch')
    create_payloads = [child for child in items[0][2]
                       if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1]
    if (len(create_payloads) != 1 or
            numbers(create_payloads[0], 0x420057) != [2]):
        raise ValueError('fixture_batch_order_placeholder_create_payload_mismatch')


def peer_cleanup_evidence(observations, include_state_hash=False):
    cleanup = observations[-1]
    state_after = cleanup[2]['peer_observed_state_after_sha256']
    evidence = {
        'peer_cleanup_operations': [
            numbers(item, 0x42005C)[0]
            for item in cleanup[0][2]
            if item[0] == 0x42000F],
        'peer_final_state_matches_setup':
            observations[0][2]['peer_observed_state_before_sha256'] == state_after,
    }
    if include_state_hash:
        evidence['peer_cleanup_state_after_sha256'] = state_after
    return evidence


def assert_placeholder_usage(node, case_id):
    """Accept omitted or explicit ID Placeholder identifiers after Create."""
    if case_id not in ('test-batch-create-activate-destroy',
                       'test-batch-register-encrypt-decrypt-destroy',
                       'test-batch-placeholder-chain-resolves',
                       'test-batch-order-required-placeholder-resolution',
                       'test-batch-order-omitted-defaults-true', 'test-op-encrypt',
                       'test-op-decrypt', 'test-op-mac', 'test-op-mac-verify',
                       'test-op-archive', 'test-op-pkcs#11', 'test-op-obtain-lease'):
        return
    batch_items = [child for child in node[2]
                   if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    if len(batch_items) < 2:
        raise ValueError('fixture_id_placeholder_batch_shape_mismatch')
    for batch_item in batch_items[1:]:
        payloads = [child for child in batch_item[2]
                    if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
        if len(payloads) != 1:
            raise ValueError('fixture_id_placeholder_payload_mismatch')
        identifiers = [child for child in payloads[0][2]
                       if child[0] == UNIQUE_IDENTIFIER_TAG]
        if len(identifiers) > 1:
            raise ValueError('fixture_id_placeholder_identifier_mismatch')
        if identifiers and not is_id_placeholder_identifier(identifiers[0]):
            raise ValueError('fixture_id_placeholder_identifier_mismatch')


def is_id_placeholder_identifier(node):
    """Check the TTLV type and enumeration value for ID Placeholder."""
    return (node[1] == ENUMERATION_TTLV_TYPE and len(node[2]) == 4 and
            int.from_bytes(node[2], 'big') == ID_PLACEHOLDER_ENUMERATION)


def fixture_wire(node):
    tag, kind, value = node
    body = b''.join(fixture_wire(child) for child in value) if kind == 1 else value
    return tlv(tag, kind, body)


def cancel_response(correlation: bytes, cancellation_result: int, protocol_minor: int) -> bytes:
    """Build one successful Cancel response using KMIP §6.1.5 Table 177 fields."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    payload = tlv(0x42007C, 1,
                  tlv(0x420006, 8, correlation) +
                  tlv(0x420012, 5, cancellation_result.to_bytes(4, 'big')))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (25).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (0).to_bytes(4, 'big')) + payload)
    return tlv(0x42007B, 1, header + item)


def terminal_poll_response_without_correlation(protocol_minor: int) -> bytes:
    """Build a terminal Poll response without recursively returning a correlation value."""
    # A terminal Poll Success must carry the original operation's Response Payload.
    return completed_poll_create_response(protocol_minor)


def terminal_cancel_response_with_correlation(protocol_minor: int) -> bytes:
    """Build a terminal Cancel response with the request correlation and Cancellation Result."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    payload = tlv(0x42007C, 1,
                  tlv(0x420006, 8, CANCEL_CORRELATION_VALUE) +
                  tlv(0x420012, 5, (1).to_bytes(4, 'big')))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (25).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (0).to_bytes(4, 'big')) + payload)
    return tlv(0x42007B, 1, header + item)


def pending_create_response(correlation: bytes, protocol_minor: int) -> bytes:
    """Build a Pending Create response with its required empty payload and correlation value."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (1).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (2).to_bytes(4, 'big')) +
               tlv(0x420006, 8, correlation) + tlv(0x42007C, 1, b''))
    return tlv(0x42007B, 1, header + item)


def pending_poll_response(correlation: bytes, protocol_minor: int) -> bytes:
    """Build one Pending Poll response with its required correlation and no payload."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (26).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (2).to_bytes(4, 'big')) +
               tlv(0x420006, 8, correlation))
    return tlv(0x42007B, 1, header + item)


def failed_poll_response(protocol_minor: int, message: str = 'controlled Create failure') -> bytes:
    """Build a terminal Poll carrying the failed original operation result."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(
        8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (26).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (1).to_bytes(4, 'big')) +
               tlv(0x42007E, 5, (256).to_bytes(4, 'big')) +
               tlv(0x42007D, 7, message.encode('ascii')))
    return tlv(0x42007B, 1, header + item)


def invalid_async_correlation_poll_response(protocol_minor: int) -> bytes:
    """Build Operation Failed for an unissued asynchronous correlation value."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(
        8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (26).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (1).to_bytes(4, 'big')) +
               tlv(0x42007E, 5, (43).to_bytes(4, 'big')))
    return tlv(0x42007B, 1, header + item)


def query_async_requests_response(correlation: bytes, protocol_minor: int) -> bytes:
    """Build one successful Query Async Requests response with its correlation filter result."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    payload = tlv(0x42007C, 1,
                  tlv(0x420176, 1, tlv(0x420006, 8, correlation)))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (57).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (0).to_bytes(4, 'big')) + payload)
    return tlv(0x42007B, 1, header + item)


def completed_poll_create_response(protocol_minor: int) -> bytes:
    """Build one successful Poll response carrying the original Create result."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    payload = tlv(0x42007C, 1,
                  tlv(0x420057, 5, (2).to_bytes(4, 'big')) +
                  tlv(0x420094, 7, b'fixture-key-1'))
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (26).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (0).to_bytes(4, 'big')) + payload)
    return tlv(0x42007B, 1, header + item)


def async_query_outstanding_response(protocol_minor: int) -> bytes:
    """Build the one outstanding Create selected by both Query filters."""
    version = tlv(0x420069, 1,
                  tlv(0x42006A, 2, (2).to_bytes(4, 'big')) +
                  tlv(0x42006B, 2, protocol_minor.to_bytes(4, 'big')))
    timestamp = int(dt.datetime.now(dt.timezone.utc).timestamp()).to_bytes(8, 'big', signed=True)
    header = tlv(0x42007A, 1, version + tlv(0x420092, 9, timestamp) +
                 tlv(0x42000D, 2, (1).to_bytes(4, 'big')))
    submission_date = int(dt.datetime.now(dt.timezone.utc).timestamp() * 1_000_000)
    asynchronous_request = tlv(
        0x420173, 1,
        tlv(0x420006, 8, ASYNC_QUERY_CORRELATION_CREATE_PENDING) +
        tlv(0x42005C, 5, (1).to_bytes(4, 'big')) +
        tlv(0x420174, 11, submission_date.to_bytes(8, 'big', signed=True)) +
        tlv(0x420175, 5, (2).to_bytes(4, 'big')))
    payload = tlv(0x42007C, 1, asynchronous_request)
    item = tlv(0x42000F, 1,
               tlv(0x42005C, 5, (57).to_bytes(4, 'big')) +
               tlv(0x42007F, 5, (0).to_bytes(4, 'big')) + payload)
    return tlv(0x42007B, 1, header + item)


def assert_query_async_empty_request(request_tree, version):
    """Require the no-filter Query Asynchronous Requests request in KMIP 2.1."""
    header = next(descendants(request_tree, 0x420077), request_tree)
    items = [child for child in request_tree[2]
             if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (version != '2.1' or numbers(header, 0x42006A) != [2] or
            numbers(header, 0x42006B) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [57] or len(payloads) != 1 or payloads[0][2] or
            numbers(header, 0x420007)):
        raise ValueError('fixture_query_async_empty_request_mismatch')


def verify_query_async_empty_response(response_tree):
    """Require a successful Query response with a present, empty payload."""
    items = [child for child in response_tree[2]
             if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [57] or
            numbers(items[0], 0x42007F) != [0] or len(payloads) != 1 or payloads[0][2]):
        raise ValueError('fixture_query_async_empty_response_mismatch')


def query_async_empty_peer_check(observation):
    """Record the empty query contract and isolated fixture lifecycle."""
    query_request, query_response, _ = observation[1]
    verify_query_async_empty_response(query_response)
    return {
        'peer_async_query_has_no_filters':
            not any(descendants(query_request, 0x420176)) and
            not any(descendants(query_request, 0x42014F)),
        'peer_async_query_result_is_empty':
            not any(descendants(query_response, 0x420173)),
        'peer_operation_sequence': [
            operation for request, _, _ in observation
            for item in request[2] if item[0] == BATCH_ITEM_TAG
            for operation in numbers(item, 0x42005C)],
        'peer_effects': [effect for _, _, evidence in observation
                         for effect in evidence['effects']],
        'peer_final_state_matches_setup':
            observation[0][2]['peer_observed_state_before_sha256'] ==
            observation[-1][2]['peer_observed_state_after_sha256'],
    }


def batch_single_item_default_continuation_peer_check(observation):
    """Record a successful one-item Get with the default continuation option omitted."""
    request_tree, response_tree, _ = observation[0]
    request_items = [item for item in request_tree[2]
                     if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    response_items = [item for item in response_tree[2]
                      if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [item for item in response_items[0][2]
                if item[0] == RESPONSE_PAYLOAD_TAG and item[1] == 1] \
        if len(response_items) == 1 else []
    return {
        'peer_single_get': len(request_items) == 1 and
            numbers(request_items[0], 0x42005C) == [10] and
            text_values(request_items[0], UNIQUE_IDENTIFIER_TAG) == ['fixture-key-1'],
        'peer_batch_count_is_one': numbers(request_tree, 0x42000D) == [1],
        'peer_continuation_option_is_omitted': not any(descendants(request_tree, 0x42000E)),
        'peer_order_option_is_omitted': not any(descendants(request_tree, 0x420011)),
        'peer_single_success_response': len(response_items) == 1 and
            numbers(response_tree, 0x42000D) == [1] and
            numbers(response_items[0], 0x42005C) == [10] and
            numbers(response_items[0], 0x42007F) == [0] and
            byte_string_values(response_items[0], 0x420093) == [b'interop-get'] and
            len(payloads) == 1 and bool(payloads[0][2]),
    }


def completed_poll_create_response_for_identifier(protocol_minor: int, identifier: str) -> bytes:
    """Build a successful Poll response for a Create with its issued identifier."""
    wire = completed_poll_create_response(protocol_minor)
    old = tlv(0x420094, 7, b'fixture-key-1')
    new = tlv(0x420094, 7, identifier.encode('ascii'))
    return wire.replace(old, new)


def assert_poll_request(node, version):
    """Require one KMIP 2.1 Poll request carrying the pending operation correlation."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    correlations = [child for child in payloads[0][2]
                    if child[0] == 0x420006 and child[1] == 8] if len(payloads) == 1 else []
    if (version != '2.1' or numbers(node, 0x42006A) != [2] or numbers(node, 0x42006B) != [1] or
            numbers(node, 0x42000D) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [26] or len(payloads) != 1 or
            len(payloads[0][2]) != 1 or len(correlations) != 1 or
            correlations[0][2] != CANCEL_CORRELATION_VALUE):
        raise ValueError('fixture_poll_correlation_mismatch')


def assert_async_failure_poll_request(node, version):
    """Require Poll with the exact pending Create correlation under KMIP 2.0 or 2.1."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    correlations = [child for child in payloads[0][2]
                    if child[0] == 0x420006 and child[1] == 8] if len(payloads) == 1 else []
    if (numbers(node, 0x42006A) != [2] or numbers(node, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [26] or len(payloads) != 1 or
            len(payloads[0][2]) != 1 or len(correlations) != 1 or
            correlations[0][2] != CANCEL_CORRELATION_VALUE):
        raise ValueError('fixture_async_failure_poll_correlation_mismatch')


def assert_async_unknown_correlation_poll_request(node, version):
    """Require the catalog's literal, unissued correlation on a standalone Poll."""
    header = next(descendants(node, 0x420077), node)
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (numbers(header, 0x42006A) != [2] or
            numbers(header, 0x42006B) != [int(version[-1])] or
            numbers(node, 0x42000D) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [26] or
            byte_string_values(items[0], 0x420006) != [b'unknown-unissued-value'] or
            numbers(header, 0x420007)):
        raise ValueError('fixture_async_unknown_correlation_poll_mismatch')


def verify_invalid_async_correlation_poll_response(response_tree):
    """Require the normative terminal error without a correlation or payload."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [26] or
            numbers(items[0], 0x42007F) != [1] or
            numbers(items[0], 0x42007E) != [43] or
            byte_string_values(items[0], 0x420006) or
            any(child[0] == RESPONSE_PAYLOAD_TAG for child in items[0][2])):
        raise ValueError('fixture_async_unknown_correlation_response_mismatch')


def async_unknown_correlation_peer_check(observation):
    """Report the exact unissued request and unchanged state without exposing its value."""
    request, response_tree, state = observation[0]
    return {
        'peer_async_unknown_correlation_request':
            numbers(request, 0x42005C) == [26] and
            byte_string_values(request, 0x420006) == [b'unknown-unissued-value'],
        'peer_async_unknown_correlation_failure':
            numbers(response_tree, 0x42007F) == [1] and
            numbers(response_tree, 0x42007E) == [43],
        'peer_async_unknown_correlation_not_remapped':
            not byte_string_values(response_tree, 0x420006) and
            not any(child[0] == RESPONSE_PAYLOAD_TAG
                    for item in response_tree[2] if item[0] == BATCH_ITEM_TAG
                    for child in item[2]),
        'peer_final_state_matches_setup':
            state['peer_observed_state_before_sha256'] ==
            state['peer_observed_state_after_sha256'],
    }


def assert_async_process_request(node, version):
    """Require Process with the exact pending correlation and a synchronous indicator."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    correlations = [child for child in payloads[0][2]
                    if child[0] == 0x420006 and child[1] == 8] if len(payloads) == 1 else []
    header = next(descendants(node, 0x420077), node)
    if (version != '2.1' or numbers(node, 0x42006A) != [2] or
            numbers(node, 0x42006B) != [1] or numbers(node, 0x42000D) != [1] or
            numbers(header, 0x420007) != [3] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [58] or len(payloads) != 1 or
            len(payloads[0][2]) != 1 or len(correlations) != 1 or
            correlations[0][2] != CANCEL_CORRELATION_VALUE):
        raise ValueError('fixture_async_process_request_mismatch')


def verify_process_success_response(response_tree):
    """Require synchronous Process Success and the defined empty response payload."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [58] or
            numbers(items[0], 0x42007F) != [0] or
            byte_string_values(items[0], 0x420006) or len(payloads) != 1 or
            payloads[0][2]):
        raise ValueError('fixture_async_process_response_mismatch')


def verify_failed_poll_response(response_tree, message='controlled Create failure'):
    """Require the terminal Poll to preserve the original failed Create result."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [26] or
            numbers(items[0], 0x42007F) != [1] or
            numbers(items[0], 0x42007E) != [256] or
            text_values(items[0], 0x42007D) != [message] or
            byte_string_values(items[0], 0x420006) or
            any(child[0] == RESPONSE_PAYLOAD_TAG for child in items[0][2])):
        raise ValueError('fixture_async_final_operation_failure_mismatch')


def async_failure_poll_peer_check(observation):
    """Report complete request correlation, terminal failure, and unchanged key state."""
    return {
        **async_interleaving_peer_evidence(observation),
        'peer_async_create_indicator_mandatory':
            numbers(next(descendants(observation[0][0], 0x420077), observation[0][0]),
                    0x420007) == [1],
        'peer_async_poll_correlations_match': all(
            byte_string_values(request, 0x420006) == [CANCEL_CORRELATION_VALUE]
            for request, _, _ in observation[1:]),
        'peer_async_final_failure_shape':
            numbers(observation[2][1], 0x42007E) == [256] and
            text_values(observation[2][1], 0x42007D) == ['controlled Create failure'] and
            not byte_string_values(observation[2][1], 0x420006) and
            not any(child[0] == RESPONSE_PAYLOAD_TAG
                    for item in observation[2][1][2] if item[0] == BATCH_ITEM_TAG
                    for child in item[2]),
        'peer_final_state_matches_setup':
            observation[0][2]['peer_observed_state_before_sha256'] ==
            observation[-1][2]['peer_observed_state_after_sha256'],
    }


def async_repeated_pending_poll_peer_check(observation):
    """Report stable correlation, terminal Create result, and completed cleanup."""
    return {
        **async_interleaving_peer_evidence(observation),
        'peer_async_create_indicator_mandatory':
            numbers(next(descendants(observation[0][0], 0x420077), observation[0][0]),
                    0x420007) == [1],
        'peer_async_poll_correlations_match': all(
            byte_string_values(request, 0x420006) == [CANCEL_CORRELATION_VALUE]
            for request, _, _ in observation[1:4]),
        'peer_async_final_poll_create_shape':
            numbers(observation[3][1], 0x42005C) == [26] and
            numbers(observation[3][1], 0x42007F) == [0] and
            not byte_string_values(observation[3][1], 0x420006) and
            len([child for item in observation[3][1][2]
                 if item[0] == BATCH_ITEM_TAG and item[1] == 1
                 for child in item[2] if child[0] == RESPONSE_PAYLOAD_TAG]) == 1,
        'peer_final_state_matches_setup':
            observation[0][2]['peer_observed_state_before_sha256'] ==
            observation[-1][2]['peer_observed_state_after_sha256'],
    }


def async_process_then_poll_peer_check(observation):
    """Report shared Process/Poll correlation, synchronous Process, and cleanup."""
    return {
        **async_interleaving_peer_evidence(observation),
        'peer_async_process_correlation_match':
            byte_string_values(observation[1][0], 0x420006) == [CANCEL_CORRELATION_VALUE],
        'peer_async_poll_correlation_match':
            byte_string_values(observation[2][0], 0x420006) == [CANCEL_CORRELATION_VALUE],
        'peer_async_process_is_synchronous':
            numbers(observation[1][1], 0x42007F) == [0] and
            not byte_string_values(observation[1][1], 0x420006) and
            response_has_single_empty_payload(observation[1][1]),
        'peer_async_final_poll_create_shape':
            numbers(observation[2][1], 0x42005C) == [26] and
            numbers(observation[2][1], 0x42007F) == [0] and
            not byte_string_values(observation[2][1], 0x420006) and
            any(child[0] == RESPONSE_PAYLOAD_TAG
                for item in observation[2][1][2] if item[0] == BATCH_ITEM_TAG
                for child in item[2]),
        'peer_final_state_matches_setup':
            observation[0][2]['peer_observed_state_before_sha256'] ==
            observation[-1][2]['peer_observed_state_after_sha256'],
    }


def async_query_outstanding_peer_check(observation):
    """Report filter intersection, completed exclusion, and unrelated-operation exclusion."""
    query_request, query_response, _ = observation[4]
    returned = [child for item in query_response[2]
                if item[0] == BATCH_ITEM_TAG and item[1] == 1
                for payload in item[2]
                if payload[0] == RESPONSE_PAYLOAD_TAG and payload[1] == 1
                for child in payload[2]
                if child[0] == 0x420173 and child[1] == 1]
    request_payload = next(child for item in query_request[2]
                           if item[0] == BATCH_ITEM_TAG and item[1] == 1
                           for child in item[2]
                           if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1)
    correlation_container = next(child for child in request_payload[2]
                                  if child[0] == 0x420176 and child[1] == 1)
    correlation_values = [child[2] for child in correlation_container[2]
                          if child[0] == 0x420006 and child[1] == 8]
    operation_container = next((child for child in request_payload[2]
                                if child[0] == 0x42014F and child[1] == 1), None)
    operations = [int.from_bytes(child[2], 'big') for child in operation_container[2]
                  if child[0] == 0x42005C and child[1] == 5] if operation_container else []
    returned_correlation = byte_string_values(returned[0], 0x420006) if len(returned) == 1 else []
    return {
        **async_interleaving_peer_evidence(observation),
        'peer_async_query_operation_filter': operations == [1],
        'peer_async_query_correlation_filter': correlation_values == [
            ASYNC_QUERY_CORRELATION_CREATE_COMPLETED,
            ASYNC_QUERY_CORRELATION_CREATE_PENDING,
            ASYNC_QUERY_CORRELATION_GET_PENDING],
        'peer_async_query_filters_intersect': len(returned) == 1 and
            returned_correlation == [ASYNC_QUERY_CORRELATION_CREATE_PENDING] and
            numbers(returned[0], 0x42005C) == [1],
        'peer_async_query_excludes_completed':
            ASYNC_QUERY_CORRELATION_CREATE_COMPLETED not in returned_correlation,
        'peer_async_query_excludes_other_operation':
            ASYNC_QUERY_CORRELATION_GET_PENDING not in returned_correlation,
        'peer_async_query_result_is_exact': len(returned) == 1 and
            numbers(returned[0], 0x420175) == [2] and
            len([child for returned_item in returned for child in returned_item[2]
                 if child[0] == 0x420174 and child[1] == 11]) == 1,
        'peer_final_state_matches_setup':
            observation[0][2]['peer_observed_state_before_sha256'] ==
            observation[-1][2]['peer_observed_state_after_sha256'],
    }


def assert_cancel_request(node):
    """Require the request's single Cancel payload to preserve the fixture correlation bytes."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    correlations = [child for child in payloads[0][2]
                    if child[0] == 0x420006 and child[1] == 8] if len(payloads) == 1 else []
    if len(correlations) != 1 or correlations[0][2] != CANCEL_CORRELATION_VALUE:
        raise ValueError('fixture_cancel_correlation_mismatch')


def verify_cancel_response(response_tree, correlation, cancellation_result):
    """Require one successful synchronous Cancel response with its payload echo."""
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [25] or
            numbers(items[0], 0x42007F) != [0] or len(payloads) != 1 or
            byte_string_values(payloads[0], 0x420006) != [correlation] or
            numbers(payloads[0], 0x420012) != [cancellation_result]):
        raise ValueError('fixture_cancel_response_mismatch')


def verify_terminal_poll_response_without_correlation(response_tree):
    """Require a terminal Poll Success without an async correlation value."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [26] or
            numbers(items[0], 0x42007F) != [0] or
            byte_string_values(response_tree, 0x420006)):
        raise ValueError('fixture_nonrecursive_poll_response_mismatch')


def verify_terminal_cancel_response_with_correlation(response_tree):
    """Require a terminal Cancel response that echoes the requested correlation."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [25] or
            numbers(items[0], 0x42007F) != [0] or len(payloads) != 1 or
            numbers(payloads[0], 0x420012) != [1] or
            byte_string_values(payloads[0], 0x420006) != [CANCEL_CORRELATION_VALUE]):
        raise ValueError('fixture_nonrecursive_cancel_response_mismatch')


def async_interleaving_peer_evidence(observation):
    """Return the shared operation order and response-status evidence."""
    return {
        'peer_async_interleaving_operations': [
            numbers(request[0], 0x42005C)[0] for request in observation],
        'peer_async_response_statuses': [
            numbers(item, 0x42007F)[0]
            for _, response_tree, _ in observation
            for item in response_tree[2]
            if item[0] == BATCH_ITEM_TAG],
    }


def nonrecursive_poll_cancel_peer_check(observation):
    """Summarize the terminal Poll and Cancel exchanges and correlation handling."""
    return {
        **async_interleaving_peer_evidence(observation),
        'peer_async_poll_no_correlation':
            not byte_string_values(observation[0][1], 0x420006),
        'peer_async_cancel_echo_matches':
            byte_string_values(observation[1][1], 0x420006) == [CANCEL_CORRELATION_VALUE],
        'peer_async_indicator_absent': all(
            not numbers(next(descendants(request[0], 0x420077), request[0]), 0x420007)
            for request in observation),
    }


def verify_pending_poll_response(response_tree, correlation):
    """Require one Pending Poll result, the exact correlation, and no response payload."""
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [26] or
            numbers(items[0], 0x42007F) != [2] or
            byte_string_values(items[0], 0x420006) != [correlation] or
            any(child[0] == RESPONSE_PAYLOAD_TAG for child in items[0][2])):
        raise ValueError('fixture_poll_pending_response_mismatch')


def verify_pending_create_response(response_tree):
    """Require one Pending Create result with a correlation and empty payload."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [1] or
            numbers(items[0], 0x42007F) != [2] or
            byte_string_values(items[0], 0x420006) != [CANCEL_CORRELATION_VALUE] or
            not has_single_empty_response_payload(items[0])):
        raise ValueError('fixture_pending_create_response_mismatch')


def verify_pending_async_response(response_tree, operation, correlation):
    """Require a Pending response with the requested operation and opaque correlation."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [operation] or
            numbers(items[0], 0x42007F) != [2] or
            byte_string_values(items[0], 0x420006) != [correlation] or
            not has_single_empty_response_payload(items[0])):
        raise ValueError('fixture_async_query_pending_response_mismatch')


def verify_completed_async_query_create(response_tree, identifier):
    """Require a successful Poll carrying the exact completed Create result."""
    items = [item for item in response_tree[2]
             if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [26] or
            numbers(items[0], 0x42007F) != [0] or byte_string_values(items[0], 0x420006) or
            len(payloads) != 1 or numbers(payloads[0], 0x420057) != [2] or
            text_values(payloads[0], UNIQUE_IDENTIFIER_TAG) != [identifier]):
        raise ValueError('fixture_async_query_poll_create_result_mismatch')


def assert_query_async_requests_request(node, version):
    """Require a KMIP 2.1 Query Async Requests filtered by the Create correlation."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    filters = [child for child in payloads[0][2]
               if child[0] == 0x420176 and child[1] == 1] if len(payloads) == 1 else []
    values = [child for child in filters[0][2]
              if child[0] == 0x420006 and child[1] == 8] if len(filters) == 1 else []
    header = next(descendants(node, 0x420077), node)
    if (version != '2.1' or numbers(node, 0x42006A) != [2] or
            numbers(node, 0x42006B) != [1] or numbers(node, 0x42000D) != [1] or
            len(items) != 1 or numbers(items[0], 0x42005C) != [57] or
            numbers(header, 0x420007) or len(payloads) != 1 or len(filters) != 1 or
            len(values) != 1 or values[0][2] != CANCEL_CORRELATION_VALUE or
            list(descendants(node, 0x42014F))):
        raise ValueError('fixture_async_query_request_mismatch')


def verify_query_async_requests_response(response_tree):
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    filters = [child for child in payloads[0][2]
               if child[0] == 0x420176 and child[1] == 1] if len(payloads) == 1 else []
    values = [child for child in filters[0][2]
              if child[0] == 0x420006 and child[1] == 8] if len(filters) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [57] or
            numbers(items[0], 0x42007F) != [0] or len(payloads) != 1 or
            len(filters) != 1 or len(values) != 1 or
            values[0][2] != CANCEL_CORRELATION_VALUE):
        raise ValueError('fixture_async_query_response_mismatch')


def assert_async_query_outstanding_request(node, version):
    """Require Query to carry both specified filters with exact correlation bytes."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    payload_fields = payloads[0][2] if len(payloads) == 1 else []
    operations = [child for child in payload_fields
                  if child[0] == 0x42014F and child[1] == 1]
    operation_values = [child for child in operations[0][2]
                        if child[0] == 0x42005C and child[1] == 5] if len(operations) == 1 else []
    request_operations = [int.from_bytes(child[2], 'big') for child in items[0][2]
                          if child[0] == 0x42005C and child[1] == 5] if len(items) == 1 else []
    correlation_sets = [child for child in payload_fields
                        if child[0] == 0x420176 and child[1] == 1]
    correlations = [child[2] for child in correlation_sets[0][2]
                    if child[0] == 0x420006 and child[1] == 8] if len(correlation_sets) == 1 else []
    header = next(descendants(node, 0x420077), node)
    expected_correlations = [ASYNC_QUERY_CORRELATION_CREATE_COMPLETED,
                             ASYNC_QUERY_CORRELATION_CREATE_PENDING,
                             ASYNC_QUERY_CORRELATION_GET_PENDING]
    if (version != '2.1' or numbers(node, 0x42006A) != [2] or
            numbers(node, 0x42006B) != [1] or numbers(node, 0x42000D) != [1] or
            len(items) != 1 or request_operations != [57] or
            numbers(header, 0x420007) or len(payloads) != 1 or
            len(payload_fields) != 2 or len(operations) != 1 or
            len(operations[0][2]) != 1 or len(operation_values) != 1 or
            int.from_bytes(operation_values[0][2], 'big') != 1 or
            len(correlation_sets) != 1 or correlations != expected_correlations):
        safe_shape = {
            'version': version,
            'major': numbers(node, 0x42006A),
            'minor': numbers(node, 0x42006B),
            'batch_count': numbers(node, 0x42000D),
            'item_count': len(items),
            'item_operation': request_operations,
            'async_indicator': numbers(header, 0x420007),
            'payload_count': len(payloads),
            'payload_fields': [(field[0], field[1]) for field in payload_fields],
            'operation_fields': len(operations),
            'operation_member_count': len(operations[0][2]) if operations else 0,
            'operation_values': [int.from_bytes(field[2], 'big') for field in operation_values],
            'correlation_containers': len(correlation_sets),
            'correlation_count': len(correlations),
            'correlation_filter_matches': correlations == expected_correlations,
        }
        raise ValueError(f'fixture_async_query_outstanding_filters_mismatch:{safe_shape}')


def assert_async_query_get_seed_request(node, version):
    """Require an asynchronous Get for the pre-existing controlled object."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    header = next(descendants(node, 0x420077), node)
    if (version != '2.1' or numbers(node, 0x42006A) != [2] or
            numbers(node, 0x42006B) != [1] or numbers(node, 0x42000D) != [1] or
            numbers(header, 0x420007) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [10] or len(payloads) != 1 or
            text_values(payloads[0], UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']):
        raise ValueError('fixture_async_query_get_seed_mismatch')


def assert_async_query_poll_request(node, correlation):
    """Require one Poll carrying the selected request's exact correlation."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    values = [child for child in payloads[0][2]
              if child[0] == 0x420006 and child[1] == 8] if len(payloads) == 1 else []
    if (numbers(node, 0x42006A) != [2] or numbers(node, 0x42006B) != [1] or
            numbers(node, 0x42000D) != [1] or len(items) != 1 or
            numbers(items[0], 0x42005C) != [26] or len(payloads) != 1 or
            len(values) != 1 or values[0][2] != correlation):
        raise ValueError('fixture_async_query_poll_correlation_mismatch')


def assert_async_query_cleanup_request(node, identifier):
    """Require one Destroy for the exact asynchronous Create result."""
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [20] or
            len(payloads) != 1 or text_values(payloads[0], UNIQUE_IDENTIFIER_TAG) != [identifier]):
        raise ValueError('fixture_async_query_cleanup_identifier_mismatch')


def verify_async_query_outstanding_response(response_tree):
    """Require exactly the pending Create selected by the filter intersection."""
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    requests = [child for child in payloads[0][2]
                if child[0] == 0x420173 and child[1] == 1] if len(payloads) == 1 else []
    response_operations = [int.from_bytes(child[2], 'big') for child in items[0][2]
                           if child[0] == 0x42005C and child[1] == 5] if len(items) == 1 else []
    request = requests[0] if len(requests) == 1 else None
    if (len(items) != 1 or response_operations != [57] or
            numbers(items[0], 0x42007F) != [0] or len(payloads) != 1 or
            len(payloads[0][2]) != 1 or len(requests) != 1 or
            byte_string_values(request, 0x420006) != [ASYNC_QUERY_CORRELATION_CREATE_PENDING] or
            numbers(request, 0x42005C) != [1] or numbers(request, 0x420175) != [2] or
            len([child for child in request[2] if child[0] == 0x420174 and child[1] == 11]) != 1):
        safe_shape = {
            'item_count': len(items),
            'item_operations': [
                [int.from_bytes(child[2], 'big') for child in item[2]
                 if child[0] == 0x42005C and child[1] == 5] for item in items],
            'item_statuses': [numbers(item, 0x42007F) for item in items],
            'payload_count': len(payloads),
            'payload_members': [(child[0], child[1]) for child in payloads[0][2]] if payloads else [],
            'request_count': len(requests),
            'request_members': [(child[0], child[1]) for child in request[2]] if request else [],
            'correlation_matches': bool(request) and
                byte_string_values(request, 0x420006) == [ASYNC_QUERY_CORRELATION_CREATE_PENDING],
            'operation_values': numbers(request, 0x42005C) if request else [],
            'processing_stage_values': numbers(request, 0x420175) if request else [],
        }
        raise ValueError(f'fixture_async_query_outstanding_response_mismatch:{safe_shape}')


def verify_completed_poll_create_response(response_tree):
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [26] or
            numbers(items[0], 0x42007F) != [0] or byte_string_values(items[0], 0x420006) or
            len(payloads) != 1 or numbers(payloads[0], 0x420057) != [2] or
            text_values(payloads[0], UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']):
        raise ValueError('fixture_async_poll_completed_response_mismatch')


def assert_async_create_cleanup_request(node, version):
    items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (version not in ('2.0', '2.1') or len(items) != 1 or numbers(items[0], 0x42005C) != [20] or
            len(payloads) != 1 or text_values(payloads[0], UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']):
        raise ValueError('fixture_async_create_cleanup_mismatch')


def assert_async_lifecycle_exchange(node, version, exchange_index):
    """Validate one ordered request and describe the matching peer response."""
    if exchange_index == 0:
        assert_async_create_seed_request(node, version)
        return (1,), None
    if exchange_index == 1:
        assert_query_async_requests_request(node, version)
        return (57,), None
    if exchange_index == 2:
        assert_poll_request(node, version)
        return (26,), None
    if exchange_index == 3:
        assert_async_create_cleanup_request(node, version)
        return (20,), (None,)
    raise ValueError('fixture_async_lifecycle_exchange_unexpected')


def async_lifecycle_peer_check(observation):
    """Summarize the controlled Create, Query, Poll, and cleanup sequence."""
    return {
        **async_interleaving_peer_evidence(observation),
        'peer_async_query_correlation_match':
            byte_string_values(next(descendants(observation[1][1], 0x420176)),
                               0x420006) == [CANCEL_CORRELATION_VALUE],
        'peer_async_poll_correlation_match':
            byte_string_values(observation[2][0], 0x420006) == [CANCEL_CORRELATION_VALUE],
        'peer_async_poll_completed':
            numbers(observation[2][1], 0x42007F) == [0] and
            text_values(observation[2][1], UNIQUE_IDENTIFIER_TAG) == ['fixture-key-1'],
        **peer_cleanup_evidence(observation),
    }


def assert_async_create_seed_request(node, version):
    """Require a one-item Create with the mandatory asynchronous indicator."""
    batch_items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    header = next(descendants(node, 0x420077), node)
    if len(batch_items) != 1 or numbers(batch_items[0], 0x42005C) != [1]:
        raise ValueError('fixture_cancel_seed_operation_mismatch')
    if numbers(node, 0x42006A) != [2] or numbers(node, 0x42006B) != [int(version[-1])]:
        raise ValueError('fixture_cancel_seed_version_mismatch')
    if numbers(node, 0x42000D) != [1]:
        raise ValueError('fixture_cancel_seed_batch_count_mismatch')
    if numbers(header, 0x420007) != [1]:
        raise ValueError('fixture_cancel_seed_missing_async_indicator')
    if (numbers(node, 0x420057) != [2] or numbers(node, 0x420028) != [3] or
            numbers(node, 0x42002A) != [128]):
        raise ValueError('fixture_cancel_seed_key_attributes_mismatch')


def assert_sync_default_create_request(node, version):
    """Require a Create request whose omitted indicator defaults to Prohibited."""
    assert_sync_create_request(node, version, None)


def assert_sync_create_request(node, version, expected_indicator):
    """Require a Create request with the selected optional indicator value."""
    batch_items = [item for item in node[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    header = next(descendants(node, 0x420077), node)
    if len(batch_items) != 1 or numbers(batch_items[0], 0x42005C) != [1]:
        raise ValueError('fixture_sync_create_operation_mismatch')
    if numbers(node, 0x42006A) != [2] or numbers(node, 0x42006B) != [int(version[-1])]:
        raise ValueError('fixture_sync_create_version_mismatch')
    indicators = numbers(header, 0x420007)
    if (numbers(node, 0x42000D) != [1] or
            indicators != ([] if expected_indicator is None else [expected_indicator])):
        raise ValueError('fixture_sync_create_async_indicator_mismatch')
    if (numbers(node, 0x420057) != [2] or numbers(node, 0x420028) != [3] or
            numbers(node, 0x42002A) != [128]):
        raise ValueError('fixture_sync_create_key_attributes_mismatch')


def verify_sync_create_response(response_tree):
    """Require one completed Create response without an asynchronous correlation."""
    items = [item for item in response_tree[2] if item[0] == BATCH_ITEM_TAG and item[1] == 1]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    if (len(items) != 1 or numbers(items[0], 0x42005C) != [1] or
            numbers(items[0], 0x42007F) != [0] or
            byte_string_values(items[0], 0x420006) or len(payloads) != 1 or
            numbers(payloads[0], 0x420057) != [2] or
            text_values(payloads[0], UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']):
        raise ValueError('fixture_sync_create_response_mismatch')


def sync_create_peer_check(observation, indicator_value, indicator_name):
    """Summarize peer evidence for a successful synchronous Create exchange."""
    request_tree, response_tree, _ = observation[0]
    header = next(descendants(request_tree, 0x420077), request_tree)
    return {
        'peer_async_response_statuses': [
            numbers(item, 0x42007F)[0]
            for item in response_tree[2]
            if item[0] == BATCH_ITEM_TAG],
        f'peer_async_indicator_{indicator_name}':
            numbers(header, 0x420007) == [indicator_value],
        'peer_async_correlation_absent':
            not byte_string_values(response_tree, 0x420006),
        'peer_async_no_poll_after_completion': len(observation) == 1,
    }


def read_back_attribute_fixture(case, catalog, version, attributes):
    policy = case.get('requires', {}).get('attribute_policy', {})
    if 'read_back_attribute' not in policy.get('branch_classes', []):
        return None
    target_id = case.get('requires', {}).get('normative_target_id')
    target = next((row for row in catalog['normative_targets']
                   if row.get('target_id') == target_id), None)
    if target is None:
        return None
    if target.get('name') == 'Certificate Attributes':
        trees = [attributes.scalar(name, 7, 'fixture-certificate-field')
                 for name in CERTIFICATE_ATTRIBUTE_FIELDS]
        fields = ','.join(f'{tag:06X}:{item_type}' for tag, item_type, _ in trees)
        return {
            'attribute_reference_kind': 'all',
            'attribute_response_fields': fields,
            'attribute_response_tags': [f'{tag:06X}' for tag, _, _ in trees],
            'attribute_response_wire': b''.join(fixture_wire(tree) for tree in trees),
            'register_certificate': True,
        }
    fixtures = attributes.fixtures(target, version)
    if not fixtures:
        return None
    tag, item_type, _ = fixtures[0]
    if target.get('name') == 'Vendor Attribute':
        if tag != attributes.tags.get(normalize('Attribute')) or item_type != 1:
            return None
        vendor_tag = attributes.tags[normalize('Vendor Identification')]
        name_tag = attributes.tags[normalize('Attribute Name')]
        fields = {field_tag: value.decode('utf-8') for field_tag, kind, value in fixtures[0][2]
                  if kind == 7 and field_tag in (vendor_tag, name_tag)}
        vendor, name = fields.get(vendor_tag), fields.get(name_tag)
        if not vendor or not name:
            return None
        reference = {
            'attribute_reference_kind': 'name',
            'attribute_reference_name': name,
            'attribute_vendor_identification': vendor,
        }
    else:
        reference_tag = attributes.tags.get(normalize(case['requires']['attribute']))
        if reference_tag is None or reference_tag != tag:
            return None
        reference = {
            'attribute_reference_kind': 'tag',
            'attribute_reference_tag': f'{reference_tag:06X}',
        }
    return {
        **reference,
        'attribute_response_tag': f'{tag:06X}',
        'attribute_response_type': str(item_type),
        'attribute_response_wire': fixture_wire(fixtures[0]),
    }


def target_operation_exchange_index(recipe):
    """Select the request/response that belongs to the catalog operation under test."""
    if recipe is not None and recipe[1] == 'cancel_poll_race':
        return 2
    if recipe is not None and recipe[1] == 'async_poll_cancel_no_recursion':
        return 1
    if recipe is not None and recipe[1] == 'async_create_poll_lifecycle':
        return 0
    if recipe is not None and recipe[1] == 'async_query_outstanding_filter':
        return 4
    if recipe is not None and recipe[1] == 'query_async_empty':
        return 1
    if recipe is not None and recipe[1] == 'async_poll_final_failure':
        return 2
    if recipe is not None and recipe[1] == 'async_poll_repeated_pending_to_success':
        return 3
    if recipe is not None and recipe[1] == 'async_process_then_poll_state':
        return 0
    if recipe is not None and recipe[1] in ('join_split_key', 'rekey', 'derive_key'):
        return 1
    return 1 if recipe is not None and recipe[1] in ('login_logout', 'cancel_async') else 0


def observation_for_cell(observations, cell, recipe, fallback_index):
    """Select the peer exchange described by a workflow matrix cell."""
    index = fallback_index
    workflow_step_index = cell.get('workflow_step_index')
    if recipe[1] == 'cancel_async' and workflow_step_index is not None:
        index = workflow_step_index
    elif recipe[1] == 'cancel_poll_race':
        index = {'Create': 0, 'Poll': 1, 'Cancel': 2}.get(
            cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'async_create_poll_lifecycle':
        index = {'Create': 0, 'Query Asynchronous Requests': 1, 'Poll': 2}.get(
            cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'async_query_outstanding_filter':
        index = {'Create': 0, 'Get': 2, 'Poll': 3,
                 'Query Asynchronous Requests': 4, 'Destroy': 7}.get(
                     cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'query_async_empty':
        index = {'Create': 0, 'Query Asynchronous Requests': 1,
                 'Revoke': 2, 'Destroy': 2}.get(
                     cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'async_poll_cancel_no_recursion':
        index = {'Poll': 0, 'Cancel': 1}.get(cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'async_poll_final_failure':
        index = {'Create': 0, 'Poll': 2}.get(cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'async_poll_repeated_pending_to_success':
        index = {'Create': 0, 'Poll': 3}.get(cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'async_process_then_poll_state':
        index = {'Create': 0, 'Process': 1, 'Poll': 2}.get(
            cell.get('workflow_operation'), fallback_index)
    elif recipe[1] == 'rekey':
        index = 0 if cell.get('workflow_operation') == 'Create' else 1
    elif recipe[1] == 'derive_key':
        index = 0 if cell.get('workflow_operation') in ('Create', 'Activate') else 1
    if index < 0 or index >= len(observations):
        return None
    return observations[index]


def recipe_for_case(case, attribute_fixture):
    recipe = RECIPES.get(case['case_id'])
    if recipe is not None:
        return recipe
    if attribute_fixture is not None:
        if attribute_fixture.get('register_certificate'):
            return ((3, 11, 20), 'get_attributes')
        return ((1, 11, 20), 'get_attributes')
    return None


def attribute_item_value(attribute_field):
    direct_comments = [child for child in attribute_field[2]
                       if child[0] == 0x4200FD and child[1] == 7]
    if len(direct_comments) == 1:
        return 0x4200FD, direct_comments[0][2]
    name, value = modify_attribute_value(attribute_field)
    if name != 'Comment':
        raise ValueError('fixture_attribute_name_not_supported')
    return 0x4200FD, value



class PeerState:
    """Maintain the public object lifecycle represented by the controlled peer."""

    def __init__(self, *, existing_object=False, usage_allocation_ready=False):
        self.objects = {
            'fixture-key-1': 'active' if usage_allocation_ready else 'pre_active'
        } if existing_object else {}
        self.attributes = {'fixture-key-1': set()} if existing_object else {}
        self.attribute_values = {'fixture-key-1': {}} if existing_object else {}
        self.adjustable_attribute_values = {'fixture-key-1': {}} if existing_object else {}
        self.usage_allocation_ready = usage_allocation_ready
        self.usage_allocation_totals = {'fixture-key-1': 1000} if usage_allocation_ready else {}
        self.usage_allocation_counts = {'fixture-key-1': 0} if usage_allocation_ready else {}
        self.leases = {}
        self.interop_sessions = {}
        self.key_pair_identifiers = {}
        self.archive_requests = []

    def digest(self):
        state_value = {'objects': self.objects}
        if any(self.attributes.values()):
            state_value['attributes'] = {
                key: sorted(value) for key, value in self.attributes.items() if value
            }
        if any(self.attribute_values.values()):
            state_value['attribute_values_sha256'] = {
                key: {f'{tag:06X}': digest for tag, digest in sorted(values.items())}
                for key, values in self.attribute_values.items() if values
            }
        if any(self.adjustable_attribute_values.values()):
            state_value['adjustable_attribute_values'] = {
                key: values for key, values in self.adjustable_attribute_values.items() if values
            }
        if self.usage_allocation_counts:
            state_value['usage_allocation_counts'] = self.usage_allocation_counts
        if self.leases:
            state_value['leases'] = self.leases
        if self.interop_sessions:
            state_value['interop_sessions'] = self.interop_sessions
        if self.archive_requests:
            state_value['archive_requests'] = self.archive_requests
        state = json.dumps(state_value, sort_keys=True, separators=(',', ':'))
        return hashlib.sha256(state.encode()).hexdigest()

    def execute(self, request_tree, *, continued_batch=False, stop_after_failure=False,
                rejected_operation=None,
                rejection_status=1, rejection_reason=5):
        before = self.digest()
        effects, statuses, reasons = [], [], []
        placeholder = None
        labels = {9: 'checked', 10: 'object_retrieved', 11: 'attributes_retrieved',
                  12: 'attribute_list_retrieved', 13: 'attribute_added',
                  15: 'attribute_deleted',
                  16: 'lease_obtained',
                  48: 'attribute_adjusted',
                  49: 'attribute_set',
                  43: 'exported',
                  17: 'usage_allocation_returned',
                  26: 'async_operation_polled',
                  51: 'pkcs11_executed',
                  56: 'constraints_retrieved',
                  57: 'async_requests_queried'}
        items = [item for item in request_tree[2] if item[0] == BATCH_ITEM_TAG]
        for index, item in enumerate(items):
            operation = numbers(item, 0x42005C)[0]
            request_payloads = [child for child in item[2]
                                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
            identifier_fields = [child for child in request_payloads[0][2]
                                 if child[0] == UNIQUE_IDENTIFIER_TAG] \
                if len(request_payloads) == 1 else []
            text_identifiers = [child[2].decode('utf-8') for child in identifier_fields
                                if child[1] == 7]
            enum_identifiers = [int.from_bytes(child[2], 'big') for child in identifier_fields
                                if child[1] == 5]
            if text_identifiers:
                identifier = text_identifiers[0]
            elif enum_identifiers and enum_identifiers[0] in self.key_pair_identifiers:
                identifier = self.key_pair_identifiers[enum_identifiers[0]]
            elif enum_identifiers and enum_identifiers[0] == ID_PLACEHOLDER_ENUMERATION:
                identifier = placeholder
            else:
                identifier = placeholder
            status, reason = 0, 0
            if operation == rejected_operation:
                status, reason, effect = rejection_status, rejection_reason, 'operation_rejected'
            elif operation == 57:
                effect = labels[operation]
            elif operation == 52:
                payloads = [child for child in item[2]
                            if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
                function_values = [child for payload in payloads for child in payload[2]
                                   if child[0] == 0x420160 and child[1] == 5]
                identifier_values = [child for payload in payloads for child in payload[2]
                                     if child[0] == 0x420161 and child[1] == 7]
                if (len(payloads) != 1 or len(function_values) != 1 or
                        len(identifier_values) != 1 or not identifier_values[0][2]):
                    raise ValueError('fixture_interop_request_fields_mismatch')
                function = int.from_bytes(function_values[0][2], 'big')
                identifier_key = hashlib.sha256(identifier_values[0][2]).hexdigest()
                if function == 1:
                    self.interop_sessions[identifier_key] = 'in_progress'
                    effect = 'interop_begun'
                elif function == 2:
                    self.interop_sessions[identifier_key] = 'completed'
                    effect = 'interop_ended'
                elif function == 3:
                    self.interop_sessions.pop(identifier_key, None)
                    effect = 'interop_reset'
                else:
                    raise ValueError('fixture_interop_function_unknown')
            elif operation == 2:
                private_identifier = 'fixture-private-key-1'
                public_identifier = 'fixture-public-key-1'
                if private_identifier in self.objects or public_identifier in self.objects:
                    raise ValueError('fixture_duplicate_create_key_pair')
                self.objects[private_identifier] = 'pre_active'
                self.objects[public_identifier] = 'pre_active'
                self.attributes[private_identifier] = set()
                self.attributes[public_identifier] = set()
                self.attribute_values[private_identifier] = {}
                self.attribute_values[public_identifier] = {}
                self.adjustable_attribute_values[private_identifier] = {}
                self.adjustable_attribute_values[public_identifier] = {}
                self.key_pair_identifiers = {5: private_identifier, 6: public_identifier}
                effect = 'key_pair_created'
            elif operation == 40:
                split_identifiers = ('fixture-split-part-1', 'fixture-split-part-2')
                if any(split_identifier in self.objects for split_identifier in split_identifiers):
                    raise ValueError('fixture_duplicate_create_split_key')
                for split_identifier in split_identifiers:
                    self.objects[split_identifier] = 'pre_active'
                    self.attributes[split_identifier] = set()
                    self.attribute_values[split_identifier] = {}
                    self.adjustable_attribute_values[split_identifier] = {}
                placeholder = split_identifiers[0]
                effect = 'split_key_created'
            elif operation == 41:
                split_identifiers = ('fixture-split-part-1', 'fixture-split-part-2')
                joined_identifier = 'fixture-joined-key'
                references = text_identifiers
                if (references != list(split_identifiers) or
                        any(self.objects.get(split_identifier) != 'pre_active'
                            for split_identifier in split_identifiers) or
                        joined_identifier in self.objects):
                    raise ValueError('fixture_join_split_key_parts_mismatch')
                self.objects[joined_identifier] = 'pre_active'
                self.attributes[joined_identifier] = set()
                self.attribute_values[joined_identifier] = {}
                self.adjustable_attribute_values[joined_identifier] = {}
                placeholder = joined_identifier
                effect = 'split_key_joined'
            elif operation == 4:
                replacement_identifier = REKEY_REPLACEMENT_IDENTIFIER.decode('ascii')
                if (identifier not in self.objects or replacement_identifier in self.objects):
                    raise ValueError('fixture_rekey_source_or_replacement_mismatch')
                self.objects[replacement_identifier] = 'pre_active'
                self.attributes[replacement_identifier] = set()
                self.attribute_values[replacement_identifier] = {}
                self.adjustable_attribute_values[replacement_identifier] = {}
                placeholder = replacement_identifier
                effect = 'rekeyed'
            elif operation == 5:
                derived_identifier = DERIVE_KEY_RESULT_IDENTIFIER.decode('ascii')
                if (identifier not in self.objects or self.objects[identifier] != 'active' or
                        derived_identifier in self.objects):
                    raise ValueError('fixture_derive_key_source_or_result_mismatch')
                self.objects[derived_identifier] = 'pre_active'
                self.attributes[derived_identifier] = set()
                self.attribute_values[derived_identifier] = {}
                self.adjustable_attribute_values[derived_identifier] = {}
                placeholder = derived_identifier
                effect = 'derived_key_created'
            elif operation == 6:
                private_identifier = self.key_pair_identifiers.get(5)
                public_identifier = self.key_pair_identifiers.get(6)
                certificate_identifier = 'fixture-certificate-1'
                if (identifier != public_identifier or public_identifier not in self.objects or
                        private_identifier not in self.objects or
                        self.objects[private_identifier] != 'active' or
                        self.objects[public_identifier] != 'pre_active' or
                        certificate_identifier in self.objects):
                    raise ValueError('fixture_certify_key_pair_state_mismatch')
                self.objects[certificate_identifier] = 'pre_active'
                self.attributes[certificate_identifier] = set()
                self.attribute_values[certificate_identifier] = {}
                self.adjustable_attribute_values[certificate_identifier] = {}
                placeholder = certificate_identifier
                effect = 'certified'
            elif operation == 42:
                if not identifier or identifier in self.objects:
                    raise ValueError('fixture_duplicate_import')
                self.objects[identifier] = 'pre_active'
                self.attributes[identifier] = set()
                self.attribute_values[identifier] = {}
                self.adjustable_attribute_values[identifier] = {}
                placeholder = identifier
                effect = 'imported'
            elif operation in (1, 3):
                identifier = 'fixture-key-1'
                if identifier in self.objects:
                    raise ValueError('fixture_duplicate_create')
                self.objects[identifier] = 'pre_active'
                self.attributes[identifier] = set()
                self.attribute_values[identifier] = {}
                self.adjustable_attribute_values[identifier] = {}
                placeholder = identifier
                effect = 'created'
            elif operation == 8:
                matching_identifiers = list(self.objects)
                if len(matching_identifiers) != 1:
                    raise ValueError('fixture_usage_allocation_locate_not_unique')
                identifier = matching_identifiers[0]
                placeholder = identifier
                effect = 'located'
            elif operation in (9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 31, 32, 33, 34, 35, 36, 43, 48, 49, 51, 56):
                state = self.objects.get(identifier)
                if state is None:
                    status, reason, effect = 1, 55, 'object_not_found'
                elif operation == 21:
                    # KMIP §6.1.4 defines Archive as a client's indication;
                    # the KMS policy decides whether and when archival occurs.
                    self.archive_requests.append(identifier)
                    effect = 'archive_requested'
                elif operation == 17 and self.usage_allocation_ready:
                    payloads = [child for child in item[2]
                                if child[0] == REQUEST_PAYLOAD_TAG and child[1] == 1]
                    count_items = [child for payload in payloads for child in payload[2]
                                   if child[0] == 0x420096 and child[1] == 3]
                    if (state != 'active' or identifier not in self.usage_allocation_totals or
                            len(payloads) != 1 or len(count_items) != 1 or
                            len(count_items[0][2]) != 8):
                        raise ValueError('fixture_usage_allocation_precondition_mismatch')
                    requested = int.from_bytes(count_items[0][2], 'big', signed=True)
                    allocated = self.usage_allocation_counts[identifier]
                    if (requested <= 0 or
                            allocated + requested > self.usage_allocation_totals[identifier]):
                        raise ValueError('fixture_usage_allocation_limit_mismatch')
                    self.usage_allocation_counts[identifier] = allocated + requested
                    effect = 'usage_allocation_returned'
                elif operation == 16:
                    self.leases[identifier] = {'lease_time': 3600, 'last_change_date': 1_700_000_000}
                    effect = 'lease_obtained'
                elif operation == 18:
                    if state != 'pre_active':
                        raise ValueError('fixture_activate_state_mismatch')
                    self.objects[identifier] = 'active'
                    effect = 'activated'
                elif operation in (31, 32, 33, 34, 35, 36):
                    if state != 'active':
                        raise ValueError('fixture_registered_crypto_key_not_active')
                    effect = {31: 'encrypted', 32: 'decrypted', 33: 'signed',
                              34: 'signature_verified', 35: 'mac_generated',
                              36: 'mac_verified'}[operation]
                elif operation == 51:
                    if state != 'active':
                        raise ValueError('fixture_pkcs11_key_not_active')
                    effect = 'pkcs11_executed'
                elif operation == 19:
                    self.objects[identifier] = 'revoked'
                    effect = 'revoked'
                elif operation == 20:
                    if state == 'active':
                        status, reason, effect = 1, 67, 'illegal_destroy_rejected'
                    else:
                        del self.objects[identifier]
                        self.leases.pop(identifier, None)
                        self.attributes.pop(identifier, None)
                        self.attribute_values.pop(identifier, None)
                        self.adjustable_attribute_values.pop(identifier, None)
                        effect = 'destroyed'
                elif operation in (13, 49):
                    new_attribute = next((child for child in item[2]
                                          if child[0] == REQUEST_PAYLOAD_TAG), None)
                    new_attribute_value = next((child for child in new_attribute[2]
                                                if child[0] == 0x42013D), None) \
                        if new_attribute is not None and new_attribute[1] == 1 else None
                    if new_attribute_value is None:
                        raise ValueError('fixture_add_attribute_tag_missing')
                    attribute_tag, attribute_value = attribute_item_value(new_attribute_value)
                    self.attributes.setdefault(identifier, set()).add(attribute_tag)
                    self.attribute_values.setdefault(identifier, {})[attribute_tag] = hashlib.sha256(
                        attribute_value).hexdigest()
                    effect = 'attribute_added' if operation == 13 else 'attribute_set'
                elif operation == 14:
                    current, new = request_modify_attribute_fields(item)
                    attribute_tag = 0x4200FD
                    if attribute_tag not in self.attributes.setdefault(identifier, set()):
                        raise ValueError('fixture_modify_attribute_target_missing')
                    values = self.attribute_values.setdefault(identifier, {})
                    old_digest = values.get(attribute_tag)
                    if old_digest is None:
                        raise ValueError('fixture_modify_attribute_value_missing')
                    if current is not None and hashlib.sha256(current[1]).hexdigest() != old_digest:
                        raise ValueError('fixture_modify_attribute_current_value_mismatch')
                    if new[0] != 'Comment':
                        raise ValueError('fixture_modify_attribute_name_mismatch')
                    values[attribute_tag] = hashlib.sha256(new[1]).hexdigest()
                    effect = 'attribute_modified'
                elif operation == 48:
                    vendor, name, adjustment_type, adjustment_value = \
                        request_adjust_attribute_fields(item)
                    values = self.adjustable_attribute_values.setdefault(identifier, {})
                    key = f'{vendor}:{name}'
                    previous = values.get(key, 0)
                    if adjustment_type == 1:
                        adjusted = previous + adjustment_value
                    elif adjustment_type == 2:
                        adjusted = previous - adjustment_value
                    elif adjustment_type == 3:
                        adjusted = -previous
                    else:
                        raise ValueError('fixture_adjustment_type_unknown')
                    values[key] = adjusted
                    effect = 'attribute_adjusted'
                elif operation == 15:
                    references = numbers(item, 0x42013B)
                    if references != [0x4200FD]:
                        raise ValueError('fixture_delete_attribute_reference_mismatch')
                    attributes = self.attributes.setdefault(identifier, set())
                    if 0x4200FD not in attributes:
                        raise ValueError('fixture_delete_attribute_target_missing')
                    else:
                        attributes.remove(0x4200FD)
                        self.attribute_values.setdefault(identifier, {}).pop(0x4200FD, None)
                        effect = 'attribute_deleted'
                else:
                    effect = labels[operation]
            else:
                effect = 'operation_executed'
            if continued_batch:
                effect = ('first_item_failed' if index == 0 and status != 0 else
                          'second_item_executed' if index == 1 and status == 0 else effect)
            effects.append(effect)
            statuses.append(status)
            reasons.append(reason)
            if stop_after_failure and status != 0:
                break
        return {'effects': effects, 'statuses': statuses, 'reasons': reasons,
                'peer_observed_state_before_sha256': before,
                'peer_observed_state_after_sha256': self.digest()}


def verify_lifecycle(operations, response_tree):
    """Check the fixture state machine against every ordered response item."""
    if not operations or operations[0] not in (1, 3):
        return []
    state = None
    effects, expected_statuses, expected_reasons = [], [], []
    labels = {9: 'checked', 10: 'object_retrieved', 11: 'attributes_retrieved',
              12: 'attribute_list_retrieved', 13: 'attribute_added',
              48: 'attribute_adjusted',
              16: 'lease_obtained',
              43: 'exported',
              17: 'usage_allocation_returned',
              51: 'pkcs11_executed',
              56: 'constraints_retrieved'}
    for operation in operations:
        status, reason = 0, 0
        if operation in (1, 3):
            if state is not None:
                raise ValueError('fixture_duplicate_create')
            state = 'pre_active'
            effects.append('created')
        elif state is None:
            status, reason = 1, 55
            effects.append('object_not_found')
        elif operation == 18:
            if state != 'pre_active':
                raise ValueError('fixture_activate_state_mismatch')
            state = 'active'
            effects.append('activated')
        elif operation == 21:
            # KMIP §6.1.4 leaves whether and when archival occurs to KMS policy.
            effects.append('archive_requested')
        elif operation in (31, 32, 35, 36):
            if state != 'active':
                raise ValueError('fixture_registered_crypto_key_not_active')
            effects.append({31: 'encrypted', 32: 'decrypted', 35: 'mac_generated',
                             36: 'mac_verified'}[operation])
        elif operation == 51:
            if state != 'active':
                raise ValueError('fixture_pkcs11_key_not_active')
            effects.append('pkcs11_executed')
        elif operation == 19:
            state = 'revoked'
            effects.append('revoked')
        elif operation == 20:
            if state == 'active':
                status, reason = 1, 67
                effects.append('illegal_destroy_rejected')
            else:
                state = None
                effects.append('destroyed')
        elif operation in labels:
            effects.append(labels[operation])
        else:
            raise ValueError('fixture_lifecycle_operation_unmapped')
        expected_statuses.append(status)
        expected_reasons.append(reason)
    if state is not None:
        raise ValueError('fixture_cleanup_missing')
    items = [child for child in response_tree[2] if child[0] == 0x42000F]
    if len(items) != len(operations):
        raise ValueError('fixture_response_count_mismatch')
    if [numbers(item, 0x42005C)[0] for item in items] != list(operations):
        raise ValueError('fixture_response_operation_order_mismatch')
    if [numbers(item, 0x42007F)[0] for item in items] != expected_statuses:
        raise ValueError('fixture_state_result_status_mismatch')
    if [(numbers(item, 0x42007E) or [0])[0] for item in items] != expected_reasons:
        raise ValueError('fixture_state_result_reason_mismatch')
    return effects


def verify_export_response(response_tree):
    """Check the required KMIP v2.0/v2.1 §6.1.18 Export payload fields."""
    items = [child for child in response_tree[2]
             if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    export_items = [item for item in items if numbers(item, 0x42005C) == [43]]
    payloads = [child for child in export_items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(export_items) == 1 else []
    fields = payloads[0][2] if len(payloads) == 1 else []
    required_tags = [0x420057, UNIQUE_IDENTIFIER_TAG, 0x420125, 0x42008F]
    if ([field[0] for field in fields] != required_tags or
            fields[0][1] != 5 or int.from_bytes(fields[0][2], 'big') != 2 or
            fields[1][1] != 7 or fields[1][2] != b'fixture-key-1' or
            fields[2][1] != 1 or fields[3][1] != 1):
        raise ValueError('fixture_export_response_fields_mismatch')
    attribute_fields = fields[2][2]
    if ([field[0] for field in attribute_fields] != [0x420028, 0x42002A] or
            int.from_bytes(attribute_fields[0][2], 'big') != 3 or
            int.from_bytes(attribute_fields[1][2], 'big', signed=True) != 128):
        raise ValueError('fixture_export_response_attributes_mismatch')
    if not any(child[0] == 0x420040 for child in fields[3][2]):
        raise ValueError('fixture_export_response_any_object_mismatch')


def verify_archive_response(response_tree):
    """Check the required KMIP v2.0/v2.1 §6.1.4 Archive response identifier."""
    items = [child for child in response_tree[2]
             if child[0] == BATCH_ITEM_TAG and child[1] == 1 and
             numbers(child, 0x42005C) == [21]]
    payloads = [child for child in items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(items) == 1 else []
    identifiers = [child for child in payloads[0][2]
                   if child[0] == UNIQUE_IDENTIFIER_TAG] if len(payloads) == 1 else []
    if (len(identifiers) != 1 or identifiers[0][1] != 7 or
            identifiers[0][2] != b'fixture-key-1'):
        raise ValueError('fixture_archive_response_identifier_mismatch')



def export_response_field_tag(field_name):
    """Resolve the fixture's exact tag for one Export response field."""
    tags = {'Object Type': 0x420057, 'Unique Identifier': UNIQUE_IDENTIFIER_TAG,
            'Attributes': 0x420125, 'Any Object (Section 2)': 0x42008F}
    tag = tags.get(field_name)
    if tag is None:
        raise ValueError('fixture_export_response_field_unmapped')
    return tag


def omit_export_response_field(response_wire, field_name):
    """Build a response fixture with one required Export field removed."""
    response_tree = parse(response_wire)
    export_items = [item for item in response_tree[2]
                    if item[0] == BATCH_ITEM_TAG and item[1] == 1 and
                    numbers(item, 0x42005C) == [43]]
    payloads = [child for child in export_items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(export_items) == 1 else []
    if len(payloads) != 1:
        raise ValueError('fixture_export_response_payload_missing')
    fields = payloads[0][2]
    tag = export_response_field_tag(field_name)
    matching = [field for field in fields if field[0] == tag]
    if len(matching) != 1:
        raise ValueError('fixture_export_response_field_cardinality_mismatch')
    fields.remove(matching[0])
    return fixture_wire(response_tree)


def verify_export_response_missing_field(response_tree, field_name):
    """Confirm one required successful Export field is absent in the negative fixture."""
    export_items = [item for item in response_tree[2]
                    if item[0] == BATCH_ITEM_TAG and item[1] == 1 and
                    numbers(item, 0x42005C) == [43]]
    payloads = [child for child in export_items[0][2]
                if child[0] == RESPONSE_PAYLOAD_TAG and child[1] == 1] if len(export_items) == 1 else []
    if len(payloads) != 1:
        raise ValueError('fixture_export_response_payload_missing')
    expected_tags = [0x420057, UNIQUE_IDENTIFIER_TAG, 0x420125, 0x42008F]
    expected_tags.remove(export_response_field_tag(field_name))
    if [field[0] for field in payloads[0][2]] != expected_tags:
        raise ValueError('fixture_export_response_missing_field_mismatch')


def verify_usage_allocation_identifier_variants(response_tree):
    items = [child for child in response_tree[2]
             if child[0] == BATCH_ITEM_TAG and child[1] == 1]
    locate_identifiers = text_values(items[0], UNIQUE_IDENTIFIER_TAG) if items else []
    if (len(items) != 3 or
            [numbers(item, 0x42005C) for item in items] != [[8], [17], [17]] or
            [numbers(item, 0x42007F) for item in items] != [[0], [0], [0]] or
            locate_identifiers != ['fixture-key-1'] or
            [byte_string_values(item, 0x420093) for item in items] !=
            [[b'usage-locate'], [b'usage-explicit'], [b'usage-placeholder']] or
            [text_values(item, UNIQUE_IDENTIFIER_TAG) for item in items[1:]] !=
            [['fixture-key-1'], ['fixture-key-1']]):
        raise ValueError('fixture_usage_allocation_identifier_response_mismatch')
    return ['fixture-key-1', 'fixture-key-1']


def read_request(stream, encoding, transport, initial=b''):
    data = bytearray(initial)
    if transport == 'https':
        while not data.endswith(b'\r\n\r\n'):
            data.extend(stream.recv(1))
            if len(data) > 16384:
                raise ValueError('fixture_http_header_limit')
        header = data.decode('ascii')
        if not header.startswith('POST /kmip HTTP/1.1\r\n'):
            raise ValueError('fixture_http_method_mismatch')
        length = int(re.search(r'(?im)^content-length:\s*(\d+)', header)[1])
        data.clear()
        while len(data) < length:
            part = stream.recv(length - len(data))
            if not part:
                raise EOFError('fixture_connection_closed')
            data.extend(part)
        return bytes(data)
    while len(data) < 16 * 1024 * 1024:
        part = stream.recv(4096)
        if not part:
            raise EOFError('fixture_connection_closed')
        data.extend(part)
        if encoding == 'TTLV' and len(data) >= 8 and len(data) == 8 + ((int.from_bytes(data[4:8], 'big') + 7) // 8) * 8:
            return bytes(data)
        if encoding == 'JSON':
            try:
                json.loads(data)
                return bytes(data)
            except ValueError:
                pass
        if encoding == 'XML':
            try:
                ET.fromstring(data)
                return bytes(data)
            except ET.ParseError:
                pass
    raise ValueError('fixture_message_limit')



def verify_consumer_provenance(report, artifact_sha256, catalog_sha256):
    execution = report.get('execution', {})
    if (execution.get('library_commit') != PIN or execution.get('artifact_sha256') != artifact_sha256
            or report.get('catalog_sha256') != catalog_sha256):
        raise ValueError('consumer_artifact_provenance_mismatch')


def case_for_controlled_exchange(case, attribute_fixture=None, register_object_path=None):
    """Run opaque request constructors through the controlled peer."""
    if (case['case_id'] in CONTROLLED_PEER_LOCAL_MODEL_CASES
            and case.get('execution', {}).get('mode') == 'local_model'):
        case = {**case, 'execution': {**case['execution'], 'mode': 'server_exchange'}}
    if attribute_fixture is not None or register_object_path is not None:
        requires = dict(case.get('requires', {}))
        if attribute_fixture is not None:
            requires.update({
                'attribute_reference_kind': attribute_fixture['attribute_reference_kind'],
                'attribute_reference_tag': attribute_fixture.get('attribute_reference_tag', ''),
                'attribute_reference_name': attribute_fixture.get('attribute_reference_name', ''),
                'attribute_vendor_identification': attribute_fixture.get(
                    'attribute_vendor_identification', ''),
                'attribute_response_tag': attribute_fixture.get('attribute_response_tag', ''),
                'attribute_response_type': attribute_fixture.get('attribute_response_type', ''),
                'attribute_response_fields': attribute_fixture.get('attribute_response_fields', ''),
            })
        requires['register_object_path'] = register_object_path or ''
        if attribute_fixture and attribute_fixture.get('register_certificate'):
            requires['certificate_object_path'] = register_object_path or ''
        case = {**case, 'requires': {
            **requires,
        }}
    return case


def execute_network_cells(binary, catalog, manifest, *, protocol_versions=('2.0', '2.1'),
                          network_combinations=(('TTLV', 'raw_tls'), ('TTLV', 'https'),
                                                ('JSON', 'raw_tls'), ('JSON', 'https'),
                                                ('XML', 'raw_tls'), ('XML', 'https')),
                          peer_rejection=None, covered_keys=()):
    """Execute peer cells, reusing exact-cell receipts from an earlier local pass.

    The returned key set includes both previously covered cells and network
    cells observed here, allowing the matrix runner to clear pending cells
    without emitting duplicate receipts.
    """
    codec = Codec(catalog)
    targets_by_id = {target['target_id']: target for target in catalog['normative_targets']}
    events, keys, checks = [], set(covered_keys), []
    all_cells = list(iter_cells(catalog, manifest, languages=('c',)))
    artifact_sha256 = sha256(INSTALL / 'lib/libkmipkit_ffi.so')
    catalog_sha256 = sha256(ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json')
    attribute_executor = AttributeCases(INSTALL / 'lib/libkmipkit_ffi.so', catalog)
    attribute_case_ids = {
        case['case_id'] for case in catalog['test_cases']
        if case['case_id'].startswith('test-attr-') and any(
            read_back_attribute_fixture(case, catalog, version, attribute_executor) is not None
            for version in case.get('execution', {}).get('protocol_versions', []))
    }
    case_variants = [(case, variant) for case in catalog['test_cases'] for variant in
        (('usage-allocation-explicit', 'usage-allocation-omitted')
         if case['case_id'] in {
             'test-norm-payload-client-get-usage-allocation-request-20-001',
             'test-norm-payload-client-get-usage-allocation-request-21-001'} else
         ('present', 'missing_rejected') if case['case_id'].startswith(
             'test-norm-payload-client-export-response-') else
         ('present', 'omitted') if case['case_id'].startswith(
             'test-norm-payload-client-export-request-') else
         ('direct', 'standalone') if case['case_id'] in {'test-op-get-attributes', 'test-op-get-attribute-list',
                                                         'test-batch-continue-after-failure',
                                                         'test-batch-count-matches-items',
                                                         'test-batch-count-mismatch-rejected',
                                                         'test-batch-order-false-correlates-response-items',
                                                         'test-norm-payload-client-get-attribute-list-request-20-001',
                                                         'test-norm-payload-client-get-attribute-list-request-21-001',
                                                         'test-norm-payload-client-add-attribute-request-20-002',
                                                         'test-norm-payload-client-add-attribute-request-21-002',
                                                         'test-op-add-attribute',
                                                         'test-norm-payload-client-modify-attribute-request-20-002',
                                                         'test-norm-payload-client-modify-attribute-request-20-003',
                                                         'test-norm-payload-client-modify-attribute-response-20-001',
                                                         'test-norm-payload-client-modify-attribute-request-21-002',
                                                         'test-norm-payload-client-modify-attribute-request-21-003',
                                                         'test-norm-payload-client-modify-attribute-response-21-001',
                                                         'test-op-modify-attribute', 'test-op-set-attribute',
                                                         'test-op-delete-attribute', 'test-op-adjust-attribute'}
         or case['case_id'] in attribute_case_ids else ('direct',))]
    cells_by_case = {}
    for cell in all_cells:
        cells_by_case.setdefault(cell['case_id'], []).append(cell)
    with tempfile.TemporaryDirectory(prefix='kmipkit-c-route-peer-') as raw:
        directory = Path(raw)
        openssl = shutil.which('openssl') or 'openssl'
        ca, cert, key, client_cert, client_key = certificate_files(openssl, directory)
        certificate_der = directory / 'server-certificate.der'
        subprocess.run([openssl, 'x509', '-in', str(cert), '-outform', 'DER',
                        '-out', str(certificate_der)], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        certificate_request_der = directory / 'certificate-request.der'
        subprocess.run([openssl, 'req', '-new', '-key', str(key), '-subj',
                        '/CN=KMIPKit C interop fixture', '-outform', 'DER',
                        '-out', str(certificate_request_der)], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_3
        context.load_cert_chain(cert, key)
        context.load_verify_locations(ca)
        context.verify_mode = ssl.CERT_REQUIRED
        for version in protocol_versions:
            for encoding, transport in network_combinations:
                    for case, variant in case_variants:
                        case_id = case['case_id']
                        if version not in case['execution']['protocol_versions']:
                            continue
                        attribute_fixture = read_back_attribute_fixture(
                            case, catalog, version, attribute_executor)
                        recipe = recipe_for_case(case, attribute_fixture)
                        if recipe is None:
                            continue
                        cancel_result_name = case.get('requires', {}).get(
                            'protocol_scenario', {}).get('parameters', {}).get('cancel_result')
                        cancel_result = (catalog_enum_value(
                            catalog, 'Cancellation Result', cancel_result_name, version)
                            if cancel_result_name else CANCEL_RESULT_CANCELED)
                        if cancel_result is None:
                            raise ValueError('fixture_cancel_result_unavailable')
                        placeholder_chain_recipe = recipe[1] == 'batch_placeholder_chain'
                        batch_order_placeholder_recipe = recipe[1] == 'batch_order_placeholder_two_item'
                        attribute_mutation_recipe = recipe[1] in (
                            'add_attribute', 'modify_attribute', 'set_attribute', 'delete_attribute',
                            'adjust_attribute')
                        delete_attribute_recipe = recipe[1] == 'delete_attribute'
                        modify_attribute_recipe = recipe[1] == 'modify_attribute'
                        attribute_mutation_operation = {
                            'add_attribute': 13, 'modify_attribute': 14,
                            'set_attribute': 49, 'delete_attribute': 15, 'adjust_attribute': 48,
                        }.get(recipe[1])
                        seeded_attribute_recipe = delete_attribute_recipe or modify_attribute_recipe
                        mutation_exchange_index = 2 if seeded_attribute_recipe else 1
                        attribute_cleanup_exchange_index = 3 if seeded_attribute_recipe else 2
                        reject_case = peer_rejection is not None and peer_rejection.get('case_id') == case_id
                        if version == '2.0' and case_id in ('test-op-discover-versions', 'test-op-login', 'test-op-logout', 'test-op-get-constraints', 'test-op-ping'):
                            continue
                        observation, errors = [], []
                        peer_state = PeerState(
                            existing_object=(recipe[1] == 'async_query_outstanding_filter' or
                                            (recipe[1] not in (
                                                'cancel_async', 'cancel_poll_race',
                                                'async_poll_cancel_no_recursion',
                                                'async_poll_final_failure',
                                                'async_poll_repeated_pending_to_success',
                                                'async_process_then_poll_state',
                                                'async_query_outstanding_filter',
                                                'async_unknown_correlation_rejected',
                                                'create_split_key', 'join_split_key') and
                                            recipe[0][0] not in (1, 2, 3))),
                            usage_allocation_ready=recipe[1] == 'get_usage_allocation_uid_variants')
                        local_placeholder_rejection = (
                            case_id == 'test-batch-placeholder-without-producer-rejected')
                        boundary = local_placeholder_rejection or case_id in (
                            'test-v20-reject-get-constraints', 'test-v20-reject-ping')
                        with socket.socket() as server:
                            server.bind(('127.0.0.1', 0))
                            server.listen(1)
                            server.settimeout(5)
                            port = server.getsockname()[1]
                            def serve():
                                stream = None
                                exchange_index = -1
                                try:
                                    exchange_count = (
                                        9 if recipe[1] == 'async_query_outstanding_filter' else
                                        3 if recipe[1] == 'query_async_empty' else
                                        4 if seeded_attribute_recipe or
                                        recipe[1] == 'async_create_poll_lifecycle' else
                                        3 if attribute_mutation_recipe or
                                        recipe[1] in ('cancel_poll_race', 'async_poll_final_failure') else
                                        3 if recipe[1] == 'create_split_key' else
                                        2 if recipe[1] in ('join_split_key', 'rekey', 'derive_key') else
                                        5 if recipe[1] == 'async_poll_repeated_pending_to_success' else
                                        4 if recipe[1] == 'async_process_then_poll_state' else
                                        2 if placeholder_chain_recipe or batch_order_placeholder_recipe or
                                        recipe[1] in ('login_logout', 'cancel_async',
                                                       'async_poll_cancel_no_recursion') else 1)
                                    seed_rejected = False
                                    accepted, _ = server.accept()
                                    stream = context.wrap_socket(accepted, server_side=True)
                                    for exchange_index in range(exchange_count):
                                        initial = b''
                                        if exchange_index > 0:
                                            ready, _, _ = select.select([stream, server], [], [], 5)
                                            if not ready:
                                                raise TimeoutError()
                                            if server in ready:
                                                stream.close()
                                                accepted, _ = server.accept()
                                                stream = context.wrap_socket(accepted, server_side=True)
                                            else:
                                                initial = stream.recv(1)
                                                if not initial:
                                                    stream.close()
                                                    accepted, _ = server.accept()
                                                    stream = context.wrap_socket(accepted, server_side=True)
                                                    initial = b''
                                        stream.settimeout(5)
                                        wire = read_request(stream, encoding, transport, initial)
                                        request_tree = codec.decode(wire, encoding)
                                        reject_this_exchange = reject_case and exchange_index == peer_rejection.get(
                                            'exchange_index')
                                        cleanup_exchange = (
                                            (placeholder_chain_recipe or batch_order_placeholder_recipe) and
                                            exchange_index == 1) or (
                                            attribute_mutation_recipe and
                                            (exchange_index == attribute_cleanup_exchange_index or
                                             (delete_attribute_recipe and seed_rejected and
                                              exchange_index == attribute_cleanup_exchange_index - 1)))
                                        response_item_ids = None
                                        returned_unique_identifier = None
                                        returned_unique_identifiers = None
                                        logout_recipe = recipe[1] == 'login_logout'
                                        if logout_recipe:
                                            assert_login_logout_exchange(request_tree, version, exchange_index)
                                            response_operations = ((45,) if exchange_index == 0 else (46,))
                                        elif recipe[1] == 'query_async_empty':
                                            if exchange_index == 0:
                                                items = [child for child in request_tree[2]
                                                         if child[0] == BATCH_ITEM_TAG and child[1] == 1]
                                                if (len(items) != 1 or numbers(items[0], 0x42005C) != [1] or
                                                        numbers(items[0], 0x420057) != [2]):
                                                    raise ValueError('fixture_query_async_create_request_mismatch')
                                                response_operations = (1,)
                                            elif exchange_index == 1:
                                                assert_query_async_empty_request(request_tree, version)
                                                response_operations = (57,)
                                            else:
                                                items = [child for child in request_tree[2]
                                                         if child[0] == BATCH_ITEM_TAG and child[1] == 1]
                                                operation_values = [numbers(item, 0x42005C) for item in items]
                                                identifiers = [text_values(item, UNIQUE_IDENTIFIER_TAG)
                                                               for item in items]
                                                if (operation_values != [[19], [20]] or
                                                        identifiers != [['fixture-key-1'], ['fixture-key-1']]):
                                                    raise ValueError('fixture_query_async_cleanup_request_mismatch')
                                                response_operations = (19, 20)
                                        elif recipe[1] == 'async_create_poll_lifecycle':
                                            response_operations, response_item_ids = \
                                                assert_async_lifecycle_exchange(
                                                    request_tree, version, exchange_index)
                                        elif recipe[1] == 'async_query_outstanding_filter':
                                            if exchange_index < 2:
                                                assert_async_create_seed_request(request_tree, version)
                                                response_operations = (1,)
                                            elif exchange_index == 2:
                                                assert_async_query_get_seed_request(request_tree, version)
                                                response_operations = (10,)
                                            elif exchange_index in (3, 5, 6):
                                                correlation = (
                                                    ASYNC_QUERY_CORRELATION_CREATE_COMPLETED
                                                    if exchange_index == 3 else
                                                    ASYNC_QUERY_CORRELATION_CREATE_PENDING
                                                    if exchange_index == 5 else
                                                    ASYNC_QUERY_CORRELATION_GET_PENDING)
                                                assert_async_query_poll_request(request_tree, correlation)
                                                response_operations = (26,)
                                            elif exchange_index == 4:
                                                assert_async_query_outstanding_request(request_tree, version)
                                                response_operations = (57,)
                                            else:
                                                identifier = 'fixture-key-2' if exchange_index == 7 else 'fixture-key-3'
                                                assert_async_query_cleanup_request(request_tree, identifier)
                                                response_operations = (20,)
                                                response_item_ids = (None,)
                                        elif recipe[1] == 'async_poll_cancel_no_recursion':
                                            if exchange_index == 0:
                                                assert_poll_request(request_tree, version)
                                                response_operations = (26,)
                                            else:
                                                assert_cancel_request(request_tree)
                                                header = next(descendants(request_tree, 0x420077), request_tree)
                                                items = [item for item in request_tree[2]
                                                         if item[0] == BATCH_ITEM_TAG and item[1] == 1]
                                                if (version != '2.1' or numbers(header, 0x42006A) != [2] or
                                                        numbers(header, 0x42006B) != [1] or
                                                        numbers(request_tree, 0x42000D) != [1] or
                                                        len(items) != 1 or numbers(items[0], 0x42005C) != [25]):
                                                    raise ValueError('fixture_nonrecursive_cancel_request_mismatch')
                                                response_operations = (25,)
                                            header = next(descendants(request_tree, 0x420077), request_tree)
                                            if numbers(header, 0x420007):
                                                raise ValueError('fixture_nonrecursive_async_indicator_present')
                                        elif recipe[1] == 'async_poll_final_failure':
                                            if exchange_index == 0:
                                                assert_async_create_seed_request(request_tree, version)
                                                response_operations = (1,)
                                            else:
                                                assert_async_failure_poll_request(request_tree, version)
                                                response_operations = (26,)
                                        elif recipe[1] == 'async_unknown_correlation_rejected':
                                            assert_async_unknown_correlation_poll_request(request_tree, version)
                                            response_operations = (26,)
                                        elif recipe[1] == 'async_poll_repeated_pending_to_success':
                                            if exchange_index == 0:
                                                assert_async_create_seed_request(request_tree, version)
                                                response_operations = (1,)
                                            elif exchange_index < 4:
                                                assert_async_failure_poll_request(request_tree, version)
                                                response_operations = (26,)
                                            else:
                                                assert_async_create_cleanup_request(request_tree, version)
                                                response_operations = (20,)
                                        elif recipe[1] == 'async_process_then_poll_state':
                                            if exchange_index == 0:
                                                assert_async_create_seed_request(request_tree, version)
                                                response_operations = (1,)
                                            elif exchange_index == 1:
                                                assert_async_process_request(request_tree, version)
                                                response_operations = (58,)
                                            elif exchange_index == 2:
                                                assert_async_failure_poll_request(request_tree, version)
                                                response_operations = (26,)
                                            else:
                                                assert_async_create_cleanup_request(request_tree, version)
                                                response_operations = (20,)
                                        elif recipe[1] == 'async_create_pending' and exchange_index == 0:
                                            assert_async_create_seed_request(request_tree, version)
                                            response_operations = (1,)
                                        elif recipe[1] == 'async_create_sync_default' and exchange_index == 0:
                                            assert_sync_default_create_request(request_tree, version)
                                            response_operations = (1,)
                                            response_item_ids = (None,)
                                        elif recipe[1] in ('async_create_optional_complete',
                                                           'async_create_prohibited_sync') and exchange_index == 0:
                                            indicator_value = 2 if recipe[1] == 'async_create_optional_complete' else 3
                                            assert_sync_create_request(request_tree, version, indicator_value)
                                            response_operations = (1,)
                                            response_item_ids = (None,)
                                        elif recipe[1] in ('cancel_async', 'cancel_poll_race') and exchange_index == 0:
                                            assert_async_create_seed_request(request_tree, version)
                                            response_operations = (1,)
                                        elif recipe[1] == 'cancel_poll_race' and exchange_index == 1:
                                            assert_poll_request(request_tree, version)
                                            response_operations = (26,)
                                        elif recipe[1] == 'interop':
                                            assert_interop_request(request_tree, version)
                                            response_operations = recipe[0]
                                            response_item_ids = (b'interop-begin', b'interop-end',
                                                                 b'interop-reset')
                                        elif recipe[1] == 'create_split_key':
                                            if exchange_index == 0:
                                                assert_create_split_key_request(request_tree)
                                                response_operations = (40,)
                                            else:
                                                assert_split_key_destroy_request(request_tree, exchange_index)
                                                response_operations = (20,)
                                                returned_unique_identifier = \
                                                    CREATE_SPLIT_KEY_IDENTIFIERS[exchange_index - 1]
                                        elif recipe[1] == 'join_split_key':
                                            if exchange_index == 0:
                                                assert_create_split_key_request(
                                                    request_tree, JOIN_SPLIT_KEY_ITEM_IDS[0])
                                                response_operations = (40,)
                                            else:
                                                assert_join_split_key_request(request_tree)
                                                response_operations = (41, 20, 20, 20)
                                                returned_unique_identifiers = (
                                                    b'fixture-joined-key', b'fixture-joined-key',
                                                    *CREATE_SPLIT_KEY_IDENTIFIERS)
                                        elif recipe[1] == 'rekey':
                                            if exchange_index == 0:
                                                assert_rekey_create_request(request_tree)
                                                response_operations = (1,)
                                                response_item_ids = (REKEY_ITEM_IDS[0],)
                                            else:
                                                assert_rekey_request(request_tree)
                                                response_operations = (4, 19, 20, 19, 20)
                                                response_item_ids = REKEY_ITEM_IDS[1:]
                                                returned_unique_identifiers = (
                                                    REKEY_REPLACEMENT_IDENTIFIER,
                                                    REKEY_REPLACEMENT_IDENTIFIER,
                                                    REKEY_REPLACEMENT_IDENTIFIER,
                                                    REKEY_ORIGINAL_IDENTIFIER,
                                                    REKEY_ORIGINAL_IDENTIFIER)
                                        elif recipe[1] == 'derive_key':
                                            if exchange_index == 0:
                                                assert_derive_key_create_request(request_tree, version)
                                                response_operations = (1, 18)
                                                response_item_ids = DERIVE_KEY_ITEM_IDS[:2]
                                            else:
                                                assert_derive_key_request(request_tree, version)
                                                response_operations = (5, 19, 20, 20)
                                                response_item_ids = DERIVE_KEY_ITEM_IDS[2:]
                                                returned_unique_identifier = DERIVE_KEY_RESULT_IDENTIFIER
                                                returned_unique_identifiers = (
                                                    DERIVE_KEY_RESULT_IDENTIFIER,
                                                    DERIVE_KEY_SOURCE_IDENTIFIER,
                                                    DERIVE_KEY_RESULT_IDENTIFIER,
                                                    DERIVE_KEY_SOURCE_IDENTIFIER)
                                        elif cleanup_exchange:
                                            if placeholder_chain_recipe or batch_order_placeholder_recipe:
                                                assert_placeholder_cleanup_request(request_tree, version)
                                            else:
                                                assert_add_attribute_cleanup_request(request_tree, version)
                                            response_operations = PLACEHOLDER_CLEANUP_OPERATIONS
                                        elif attribute_mutation_recipe and exchange_index == 0:
                                            assert_add_attribute_setup_request(request_tree, version)
                                            response_operations = (1,)
                                        elif seeded_attribute_recipe and exchange_index == 1:
                                            assert_recipe(request_tree, case_id, version,
                                                          ((13,), 'add_attribute'))
                                            if text_values(request_tree, UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']:
                                                raise ValueError('fixture_attribute_seed_identifier_mismatch')
                                            response_operations = (13,)
                                        elif attribute_mutation_recipe and exchange_index == mutation_exchange_index:
                                            assert_recipe(request_tree, case_id, version,
                                                          ((attribute_mutation_operation,), recipe[1]))
                                            if text_values(request_tree, UNIQUE_IDENTIFIER_TAG) != ['fixture-key-1']:
                                                raise ValueError('fixture_attribute_mutation_identifier_mismatch')
                                            response_operations = (attribute_mutation_operation,)
                                        elif recipe[1] in ('set_constraints', 'set_defaults'):
                                            assert_recipe(request_tree, case_id, version, recipe,
                                                          attribute_fixture, variant,
                                                          case.get('requires', {}).get('field_name')
                                                          if case_id.startswith(
                                                              'test-norm-payload-client-export-request-')
                                                          else None)
                                            response_operations = recipe[0]
                                        else:
                                            assert_recipe(request_tree, case_id, version, recipe,
                                                          attribute_fixture, variant,
                                                          case.get('requires', {}).get('field_name')
                                                          if case_id.startswith(
                                                              'test-norm-payload-client-export-request-')
                                                          else None)
                                            response_operations = recipe[0]
                                        selected_tag = {'test-attr-cryptographic-algorithm': 0x420028,
                                                        'test-attr-cryptographic-length': 0x42002A}.get(
                                                            case_id, int(attribute_fixture.get(
                                                                'attribute_response_tag', '000000'), 16)
                                                            if attribute_fixture else 0)
                                        object_type = 4 if case_id == 'test-obj-private-key' else 3 if case_id == 'test-obj-public-key' else 2
                                        if recipe[1] == 'get_usage_allocation_uid_variant':
                                            response_item_ids = (
                                                b'interop-create',
                                                b'usage-allocation-explicit'
                                                if variant == 'usage-allocation-explicit' else
                                                b'usage-allocation-omitted',
                                                b'interop-revoke', b'interop-destroy')
                                        elif recipe[1] == 'batch_register_crypto':
                                            response_item_ids = BATCH_CRYPTO_ITEM_IDS
                                        elif recipe[1] == 'encrypt':
                                            response_item_ids = ENCRYPT_ITEM_IDS
                                        elif recipe[1] == 'decrypt':
                                            response_item_ids = DECRYPT_ITEM_IDS
                                        elif recipe[1] == 'mac':
                                            response_item_ids = MAC_ITEM_IDS
                                        elif recipe[1] == 'mac_verify':
                                            response_item_ids = MAC_VERIFY_ITEM_IDS
                                        elif recipe[1] == 'sign':
                                            response_item_ids = SIGN_ITEM_IDS
                                        elif recipe[1] == 'signature_verify':
                                            response_item_ids = SIGNATURE_VERIFY_ITEM_IDS
                                        elif recipe[1] == 'create_key_pair':
                                            response_item_ids = CREATE_KEY_PAIR_ITEM_IDS
                                        elif recipe[1] == 'archive':
                                            response_item_ids = (
                                                b'archive-create', b'archive-target',
                                                b'archive-revoke', b'archive-destroy')
                                        elif recipe[1] == 'certify':
                                            response_item_ids = CERTIFY_ITEM_IDS
                                        elif recipe[1] == 'pkcs11':
                                            response_item_ids = PKCS11_ITEM_IDS
                                        elif recipe[1] == 'obtain_lease':
                                            response_item_ids = OBTAIN_LEASE_ITEM_IDS
                                        elif recipe[1] == 'create_split_key':
                                            response_item_ids = (CREATE_SPLIT_KEY_ITEM_IDS[exchange_index],)
                                        elif recipe[1] == 'join_split_key':
                                            response_item_ids = (
                                                (JOIN_SPLIT_KEY_ITEM_IDS[0],)
                                                if exchange_index == 0 else JOIN_SPLIT_KEY_ITEM_IDS[1:])
                                        elif recipe[1] == 'register_secret_data':
                                            response_item_ids = SECRET_DATA_ITEM_IDS
                                        elif recipe[1] == 'import_secret_data':
                                            response_item_ids = IMPORT_ITEM_IDS
                                            returned_unique_identifier = IMPORT_IDENTIFIER
                                        elif placeholder_chain_recipe:
                                            response_item_ids = (
                                                (b'placeholder-revoke-cleanup', b'placeholder-destroy-cleanup')
                                                if cleanup_exchange else
                                                (b'placeholder-create', b'placeholder-activate', b'placeholder-get'))
                                        elif batch_order_placeholder_recipe:
                                            response_item_ids = (
                                                (b'placeholder-revoke-cleanup', b'placeholder-destroy-cleanup')
                                                if cleanup_exchange else BATCH_ORDER_PLACEHOLDER_ITEM_IDS)
                                        elif recipe[1] == 'export':
                                            response_item_ids = (
                                                b'interop-create', b'interop-export',
                                                b'interop-revoke', b'interop-destroy')
                                        elif attribute_mutation_recipe:
                                            response_item_ids = (
                                                (b'interop-create',)
                                                if exchange_index == 0 else
                                                (b'interop-delete-attribute',)
                                                if delete_attribute_recipe and
                                                exchange_index == mutation_exchange_index and
                                                not cleanup_exchange else
                                                (b'interop-revoke', b'interop-destroy')
                                                if cleanup_exchange else
                                                (b'interop-attributes',))
                                        elif recipe[1] == 'get_usage_allocation_uid_variants':
                                            response_item_ids = (
                                                b'usage-locate', b'usage-explicit',
                                                b'usage-placeholder')
                                        if recipe[1] in ('batch_count_two_read_items',
                                                         'batch_order_false_two_read_items',
                                                         'batch_continue_after_failure'):
                                            response_item_ids = (b'item-a', b'item-b')
                                            if recipe[1] == 'batch_order_false_two_read_items':
                                                response_operations = tuple(reversed(recipe[0]))
                                                response_item_ids = (b'item-b', b'item-a')
                                        elif recipe[1] == 'batch_stop_on_first_failure':
                                            response_operations = (10,)
                                            response_item_ids = (b'item-a',)
                                        if recipe[1] == 'async_query_outstanding_filter' and exchange_index < 9:
                                            state_digest = peer_state.digest()
                                            if exchange_index == 3:
                                                peer_state.objects['fixture-key-2'] = 'pre_active'
                                                peer_state.attributes['fixture-key-2'] = set()
                                                peer_state.attribute_values['fixture-key-2'] = {}
                                            elif exchange_index == 5:
                                                peer_state.objects['fixture-key-3'] = 'pre_active'
                                                peer_state.attributes['fixture-key-3'] = set()
                                                peer_state.attribute_values['fixture-key-3'] = {}
                                            elif exchange_index in (7, 8):
                                                identifier = 'fixture-key-2' if exchange_index == 7 else 'fixture-key-3'
                                                if identifier not in peer_state.objects:
                                                    raise ValueError('fixture_async_query_cleanup_object_missing')
                                                peer_state.objects.pop(identifier)
                                                peer_state.attributes.pop(identifier, None)
                                                peer_state.attribute_values.pop(identifier, None)
                                            state_evidence = {
                                                'effects': [],
                                                'statuses': [1] if exchange_index == 6 else
                                                [2] if exchange_index < 3 else [0],
                                                'reasons': [256] if exchange_index == 6 else [0],
                                                'peer_observed_state_before_sha256': state_digest,
                                                'peer_observed_state_after_sha256': peer_state.digest(),
                                            }
                                        elif recipe[1] in ('async_poll_final_failure',
                                                         'async_poll_repeated_pending_to_success') and exchange_index < 3:
                                            state_digest = peer_state.digest()
                                            state_evidence = {
                                                'effects': [],
                                                'statuses': [2],
                                                'reasons': [0],
                                                'peer_observed_state_before_sha256': state_digest,
                                                'peer_observed_state_after_sha256': state_digest,
                                            }
                                        elif recipe[1] == 'async_unknown_correlation_rejected':
                                            state_digest = peer_state.digest()
                                            state_evidence = {
                                                'effects': [],
                                                'statuses': [1],
                                                'reasons': [43],
                                                'peer_observed_state_before_sha256': state_digest,
                                                'peer_observed_state_after_sha256': state_digest,
                                            }
                                        else:
                                            if (recipe[1] == 'async_process_then_poll_state' and
                                                    exchange_index == 1):
                                                peer_state.objects['fixture-key-1'] = 'pre_active'
                                                peer_state.attributes['fixture-key-1'] = set()
                                                peer_state.attribute_values['fixture-key-1'] = {}
                                            if (recipe[1] == 'async_poll_repeated_pending_to_success' and
                                                    exchange_index == 3):
                                                peer_state.objects['fixture-key-1'] = 'pre_active'
                                                peer_state.attributes['fixture-key-1'] = set()
                                                peer_state.attribute_values['fixture-key-1'] = {}
                                            state_evidence = peer_state.execute(
                                                request_tree,
                                                continued_batch=recipe[1] == 'batch_continue_after_failure',
                                                stop_after_failure=recipe[1] == 'batch_stop_on_first_failure',
                                                rejected_operation=peer_rejection.get('operation')
                                                if reject_this_exchange else None,
                                                rejection_status=peer_rejection.get('result_status', 1)
                                                if reject_this_exchange else 1,
                                                rejection_reason=peer_rejection.get('result_reason', 5)
                                                if reject_this_exchange else 5)
                                        response_statuses = tuple(state_evidence['statuses'])
                                        response_reasons = tuple(reason or None for reason in state_evidence['reasons'])
                                        if response_operations != recipe[0] and not cleanup_exchange:
                                            response_statuses = tuple(reversed(response_statuses))
                                            response_reasons = tuple(reversed(response_reasons))
                                        if (recipe[1] == 'async_poll_cancel_no_recursion'):
                                            response_wire = (
                                                terminal_poll_response_without_correlation(int(version[-1]))
                                                if exchange_index == 0 else
                                                terminal_cancel_response_with_correlation(int(version[-1])))
                                        elif recipe[1] == 'query_async_empty' and exchange_index == 1:
                                            response_wire = response((57,), int(version[-1]))
                                        elif recipe[1] == 'async_query_outstanding_filter' and exchange_index < 7:
                                            response_wire = (
                                                pending_create_response(
                                                    ASYNC_QUERY_CORRELATION_CREATE_COMPLETED, int(version[-1]))
                                                if exchange_index == 0 else
                                                pending_create_response(
                                                    ASYNC_QUERY_CORRELATION_CREATE_PENDING, int(version[-1]))
                                                if exchange_index == 1 else
                                                pending_create_response(
                                                    ASYNC_QUERY_CORRELATION_GET_PENDING, int(version[-1])).replace(
                                                        tlv(0x42005C, 5, (1).to_bytes(4, 'big')),
                                                        tlv(0x42005C, 5, (10).to_bytes(4, 'big')), 1)
                                                if exchange_index == 2 else
                                                completed_poll_create_response_for_identifier(
                                                    int(version[-1]), 'fixture-key-2')
                                                if exchange_index == 3 else
                                                async_query_outstanding_response(int(version[-1]))
                                                if exchange_index == 4 else
                                                completed_poll_create_response_for_identifier(
                                                    int(version[-1]), 'fixture-key-3')
                                                if exchange_index == 5 else
                                                failed_poll_response(
                                                    int(version[-1]), 'controlled Get failure'))
                                        elif recipe[1] == 'async_query_outstanding_filter':
                                            identifier = 'fixture-key-2' if exchange_index == 7 else 'fixture-key-3'
                                            response_wire = response(
                                                response_operations, int(version[-1]), selected_tag, object_type,
                                                attribute_fixture['attribute_response_wire'] if attribute_fixture else None,
                                                response_item_ids, response_statuses, response_reasons).replace(
                                                    tlv(UNIQUE_IDENTIFIER_TAG, 7, b'fixture-key-1'),
                                                    tlv(UNIQUE_IDENTIFIER_TAG, 7, identifier.encode('ascii')), 1)
                                        elif recipe[1] == 'async_poll_final_failure':
                                            response_wire = (
                                                pending_create_response(CANCEL_CORRELATION_VALUE,
                                                                        int(version[-1]))
                                                if exchange_index == 0 else
                                                pending_poll_response(CANCEL_CORRELATION_VALUE,
                                                                      int(version[-1]))
                                                if exchange_index == 1 else
                                                failed_poll_response(int(version[-1])))
                                        elif recipe[1] == 'async_unknown_correlation_rejected':
                                            response_wire = invalid_async_correlation_poll_response(
                                                int(version[-1]))
                                        elif recipe[1] == 'async_poll_repeated_pending_to_success':
                                            if exchange_index == 4:
                                                response_item_ids = (None,)
                                            response_wire = (
                                                pending_create_response(CANCEL_CORRELATION_VALUE,
                                                                        int(version[-1]))
                                                if exchange_index == 0 else
                                                pending_poll_response(CANCEL_CORRELATION_VALUE,
                                                                      int(version[-1]))
                                                if exchange_index < 3 else
                                                completed_poll_create_response(int(version[-1]))
                                                if exchange_index == 3 else
                                                response((20,), int(version[-1]),
                                                         created_object_type=2,
                                                         unique_batch_item_ids=response_item_ids))
                                        elif recipe[1] == 'async_process_then_poll_state':
                                            if exchange_index == 3:
                                                response_item_ids = (None,)
                                            response_wire = (
                                                pending_create_response(CANCEL_CORRELATION_VALUE,
                                                                        int(version[-1]))
                                                if exchange_index == 0 else
                                                response((58,), int(version[-1]))
                                                if exchange_index == 1 else
                                                completed_poll_create_response(int(version[-1]))
                                                if exchange_index == 2 else
                                                response((20,), int(version[-1]),
                                                         created_object_type=2,
                                                         unique_batch_item_ids=response_item_ids))
                                        elif (recipe[1] == 'async_create_poll_lifecycle' and
                                                exchange_index < 3):
                                            response_wire = (
                                                pending_create_response(CANCEL_CORRELATION_VALUE,
                                                                        int(version[-1]))
                                                if exchange_index == 0 else
                                                query_async_requests_response(CANCEL_CORRELATION_VALUE,
                                                                              int(version[-1]))
                                                if exchange_index == 1 else
                                                completed_poll_create_response(int(version[-1])))
                                        elif recipe[1] in ('async_create_pending', 'cancel_async',
                                                         'cancel_poll_race') and exchange_index == 0:
                                            response_wire = pending_create_response(
                                                CANCEL_CORRELATION_VALUE, int(version[-1]))
                                        elif recipe[1] == 'cancel_poll_race' and exchange_index == 1:
                                            response_wire = pending_poll_response(
                                                CANCEL_CORRELATION_VALUE, int(version[-1]))
                                        elif recipe[1] in ('cancel_async', 'cancel_poll_race'):
                                            response_wire = cancel_response(
                                                CANCEL_CORRELATION_VALUE,
                                                cancel_result,
                                                int(version[-1]))
                                        else:
                                            response_wire = response(
                                                response_operations, int(version[-1]), selected_tag, object_type,
                                                attribute_fixture['attribute_response_wire'] if attribute_fixture else None,
                                                response_item_ids, response_statuses, response_reasons,
                                                rng_length=(numbers(request_tree, 0x4200C4) or [16])[0]
                                                if recipe[1] == 'rng_retrieve' else
                                                len(byte_string_values(request_tree, 0x4200C2)[0])
                                                if recipe[1] == 'rng_seed' else 16,
                                                returned_unique_identifier=returned_unique_identifier,
                                                returned_unique_identifiers=returned_unique_identifiers)
                                        export_response_negative = (
                                            case_id.startswith('test-norm-payload-client-export-response-') and
                                            variant == 'missing_rejected' and
                                            exchange_index == target_operation_exchange_index(recipe))
                                        if export_response_negative:
                                            response_wire = omit_export_response_field(
                                                response_wire, case['requires']['field_name'])
                                        response_tree = parse(response_wire)
                                        if logout_recipe:
                                            verify_login_logout_response(response_tree, exchange_index)
                                        elif recipe[1] == 'async_poll_cancel_no_recursion':
                                            if exchange_index == 0:
                                                verify_terminal_poll_response_without_correlation(response_tree)
                                            else:
                                                verify_terminal_cancel_response_with_correlation(response_tree)
                                        elif recipe[1] == 'async_poll_final_failure':
                                            if exchange_index == 0:
                                                verify_pending_create_response(response_tree)
                                            elif exchange_index == 1:
                                                verify_pending_poll_response(
                                                    response_tree, CANCEL_CORRELATION_VALUE)
                                            else:
                                                verify_failed_poll_response(response_tree)
                                        elif recipe[1] == 'async_unknown_correlation_rejected':
                                            verify_invalid_async_correlation_poll_response(response_tree)
                                        elif recipe[1] == 'async_poll_repeated_pending_to_success':
                                            if exchange_index == 0:
                                                verify_pending_create_response(response_tree)
                                            elif exchange_index < 3:
                                                verify_pending_poll_response(
                                                    response_tree, CANCEL_CORRELATION_VALUE)
                                            elif exchange_index == 3:
                                                verify_completed_poll_create_response(response_tree)
                                            else:
                                                verify_lifecycle((20,), response_tree)
                                        elif recipe[1] == 'async_process_then_poll_state':
                                            if exchange_index == 0:
                                                verify_pending_create_response(response_tree)
                                            elif exchange_index == 1:
                                                verify_process_success_response(response_tree)
                                            elif exchange_index == 2:
                                                verify_completed_poll_create_response(response_tree)
                                            else:
                                                verify_lifecycle((20,), response_tree)
                                        elif recipe[1] == 'async_create_poll_lifecycle' and exchange_index == 1:
                                            verify_query_async_requests_response(response_tree)
                                        elif recipe[1] == 'async_create_poll_lifecycle' and exchange_index == 2:
                                            verify_completed_poll_create_response(response_tree)
                                        elif recipe[1] == 'async_query_outstanding_filter':
                                            if exchange_index == 0:
                                                verify_pending_async_response(
                                                    response_tree, 1,
                                                    ASYNC_QUERY_CORRELATION_CREATE_COMPLETED)
                                            elif exchange_index == 1:
                                                verify_pending_async_response(
                                                    response_tree, 1,
                                                    ASYNC_QUERY_CORRELATION_CREATE_PENDING)
                                            elif exchange_index == 2:
                                                verify_pending_async_response(
                                                    response_tree, 10,
                                                    ASYNC_QUERY_CORRELATION_GET_PENDING)
                                            elif exchange_index == 3:
                                                verify_completed_async_query_create(
                                                    response_tree, 'fixture-key-2')
                                            elif exchange_index == 4:
                                                verify_async_query_outstanding_response(response_tree)
                                            elif exchange_index == 5:
                                                verify_completed_async_query_create(
                                                    response_tree, 'fixture-key-3')
                                            elif exchange_index == 6:
                                                verify_failed_poll_response(
                                                    response_tree, 'controlled Get failure')
                                            else:
                                                verify_lifecycle((20,), response_tree)
                                        elif recipe[1] == 'query_async_empty' and exchange_index == 1:
                                            verify_query_async_empty_response(response_tree)
                                        elif recipe[1] == 'batch_single_item_default_continuation':
                                            response_items = [item for item in response_tree[2]
                                                              if item[0] == BATCH_ITEM_TAG and item[1] == 1]
                                            payloads = [item for item in response_items[0][2]
                                                        if item[0] == RESPONSE_PAYLOAD_TAG and item[1] == 1] \
                                                if len(response_items) == 1 else []
                                            if (len(response_items) != 1 or
                                                    numbers(response_tree, 0x42000D) != [1] or
                                                    numbers(response_items[0], 0x42005C) != [10] or
                                                    numbers(response_items[0], 0x42007F) != [0] or
                                                    byte_string_values(response_items[0], 0x420093) !=
                                                    [b'interop-get'] or len(payloads) != 1 or not payloads[0][2]):
                                                raise ValueError(
                                                    'fixture_single_item_default_continuation_response_mismatch')
                                        elif recipe[1] == 'interop':
                                            verify_interop_response(response_tree)
                                        elif recipe[1] in ('encrypt', 'decrypt'):
                                            verify_crypto_response(response_tree, recipe[1] == 'decrypt')
                                        elif recipe[1] == 'mac':
                                            verify_mac_response(response_tree)
                                        elif recipe[1] == 'mac_verify':
                                            verify_mac_verify_response(response_tree)
                                        elif recipe[1] == 'async_create_sync_default' and exchange_index == 0:
                                            verify_sync_create_response(response_tree)
                                        elif recipe[1] in ('async_create_optional_complete',
                                                           'async_create_prohibited_sync') and exchange_index == 0:
                                            verify_sync_create_response(response_tree)
                                        elif recipe[1] in ('async_create_pending', 'cancel_async',
                                                           'cancel_poll_race') and exchange_index == 0:
                                            verify_pending_create_response(response_tree)
                                        elif recipe[1] == 'cancel_poll_race' and exchange_index == 1:
                                            verify_pending_poll_response(response_tree, CANCEL_CORRELATION_VALUE)
                                        elif recipe[1] in ('cancel_async', 'cancel_poll_race'):
                                            verify_cancel_response(response_tree, CANCEL_CORRELATION_VALUE,
                                                                   cancel_result)
                                        elif recipe[1] == 'batch_stop_on_first_failure':
                                            response_items = [item for item in response_tree[2]
                                                              if item[0] == BATCH_ITEM_TAG and item[1] == 1]
                                            if ([numbers(item, 0x42005C) for item in response_items] != [[10]] or
                                                    [byte_string_values(item, 0x420093) for item in response_items] !=
                                                    [[b'item-a']] or
                                                    [numbers(item, 0x42007F) for item in response_items] != [[1]] or
                                                    [numbers(item, 0x42007E) for item in response_items] != [[55]]):
                                                raise ValueError('fixture_stop_partial_response_mismatch')
                                        elif placeholder_chain_recipe:
                                            verify_placeholder_chain_response(response_tree, response_operations)
                                        elif batch_order_placeholder_recipe:
                                            if cleanup_exchange:
                                                verify_lifecycle(PLACEHOLDER_CLEANUP_OPERATIONS, response_tree)
                                            else:
                                                verify_batch_order_placeholder_response(response_tree)
                                        elif recipe[1] == 'export':
                                            if export_response_negative:
                                                verify_export_response_missing_field(
                                                    response_tree, case['requires']['field_name'])
                                            else:
                                                verify_export_response(response_tree)
                                            verify_lifecycle(recipe[0], response_tree)
                                        elif recipe[1] == 'archive':
                                            verify_archive_response(response_tree)
                                            verify_lifecycle(recipe[0], response_tree)
                                        elif recipe[1] in ('sign', 'signature_verify'):
                                            verify_signature_response(
                                                response_tree, recipe[1] == 'signature_verify')
                                        elif recipe[1] == 'create_key_pair':
                                            verify_create_key_pair_response(response_tree)
                                        elif recipe[1] == 'create_split_key' and exchange_index == 0:
                                            verify_create_split_key_response(response_tree)
                                        elif recipe[1] == 'create_split_key':
                                            verify_split_key_destroy_response(response_tree, exchange_index)
                                        elif recipe[1] == 'join_split_key' and exchange_index == 0:
                                            verify_create_split_key_response(
                                                response_tree, JOIN_SPLIT_KEY_ITEM_IDS[0])
                                        elif recipe[1] == 'join_split_key':
                                            verify_join_split_key_response(response_tree)
                                        elif recipe[1] == 'rekey' and exchange_index == 0:
                                            verify_rekey_create_response(response_tree)
                                        elif recipe[1] == 'rekey':
                                            verify_rekey_response(response_tree)
                                        elif recipe[1] == 'derive_key':
                                            if exchange_index == 0:
                                                verify_derive_key_create_response(response_tree)
                                            else:
                                                verify_derive_key_response(response_tree)
                                        elif recipe[1] == 'register_secret_data':
                                            verify_secret_data_response(response_tree)
                                        elif recipe[1] == 'import_secret_data':
                                            verify_import_response(response_tree)
                                        elif recipe[1] == 'certify':
                                            verify_certify_response(response_tree)
                                        elif recipe[1] == 'pkcs11':
                                            verify_pkcs11_response(response_tree)
                                        elif recipe[1] == 'obtain_lease':
                                            verify_obtain_lease_response(response_tree)
                                        elif recipe[1] == 'get_usage_allocation_uid_variants':
                                            verify_usage_allocation_identifier_variants(response_tree)
                                        elif reject_this_exchange:
                                            response_items = [item for item in response_tree[2]
                                                              if item[0] == BATCH_ITEM_TAG and item[1] == 1]
                                            if ([numbers(item, 0x42005C) for item in response_items] !=
                                                    [[peer_rejection['operation']]] or
                                                    [numbers(item, 0x42007F) for item in response_items] !=
                                                    [[peer_rejection['result_status']]] or
                                                    [numbers(item, 0x42007E) for item in response_items] !=
                                                    [[peer_rejection['result_reason']]]):
                                                raise ValueError('fixture_rejection_response_mismatch')
                                        elif attribute_mutation_recipe:
                                            response_items = [item for item in response_tree[2]
                                                              if item[0] == BATCH_ITEM_TAG and item[1] == 1]
                                            if ([numbers(item, 0x42005C) for item in response_items] !=
                                                    [[operation] for operation in response_operations] or
                                                    [numbers(item, 0x42007F) for item in response_items] !=
                                                    [[0] for _ in response_operations]):
                                                raise ValueError('fixture_attribute_mutation_response_mismatch')
                                        elif recipe[1] in ('set_constraints', 'set_defaults'):
                                            response_items = [item for item in response_tree[2]
                                                              if item[0] == BATCH_ITEM_TAG and item[1] == 1]
                                            if ([numbers(item, 0x42005C) for item in response_items] !=
                                                    [[recipe[0][0]]] or
                                                    [numbers(item, 0x42007F) for item in response_items] != [[0]]):
                                                raise ValueError('fixture_global_operation_response_mismatch')
                                        elif not logout_recipe and recipe[1] not in (
                                                'query_async_empty',
                                                'async_create_poll_lifecycle',
                                                'async_poll_cancel_no_recursion',
                                                'async_poll_final_failure',
                                                'async_poll_repeated_pending_to_success',
                                                'async_process_then_poll_state'):
                                            verify_lifecycle(recipe[0], response_tree)
                                        encoded_response = codec.encode(response_wire, encoding)
                                        if transport == 'https':
                                            content_type = {'TTLV': 'application/octet-stream', 'JSON': 'application/json', 'XML': 'text/xml'}[encoding]
                                            encoded_response = (f'HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {len(encoded_response)}\r\nConnection: close\r\n\r\n'.encode() + encoded_response)
                                        stream.sendall(encoded_response)
                                        observation.append((request_tree, response_tree, state_evidence))
                                        if reject_this_exchange:
                                            seed_rejected = True
                                        elif seed_rejected and cleanup_exchange:
                                            break
                                    stream.close()
                                except TimeoutError:
                                    errors.append(f'fixture_peer_timeout_exchange_{exchange_index}')
                                except ssl.SSLError:
                                    errors.append('fixture_tls_handshake_failed')
                                except ConnectionError:
                                    errors.append('fixture_connection_failed')
                                except ValueError as error:
                                    errors.append(f'fixture_request_rejected:{error}')
                                except Exception as error:
                                    code = str(error)
                                    errors.append(code if re.fullmatch('fixture_[a-z_]+', code)
                                                  else 'controlled_peer_contract_failed')
                                finally:
                                    if stream is not None:
                                        stream.close()
                            thread = None
                            if not boundary:
                                thread = threading.Thread(target=serve, daemon=True)
                                thread.start()
                            config = {'server_id': 'controlled-c-peer', 'server_vendor': '',
                                'server_product': 'KMIPKit controlled C route fixture', 'server_version': 'fixture-2',
                                'server_configuration_fingerprint': hashlib.sha256(b'controlled-c-route-fixture-v2').hexdigest(),
                                'protocol_version': version, 'encoding': encoding, 'transport': transport,
                                'api_level': 'typed-protocol', 'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': port},
                                                                          'https': {'url': f'https://127.0.0.1:{port}/kmip'}},
                                'tls': {'ca_cert_path': str(ca), 'client_cert_path': str(client_cert),
                                        'client_key_path': str(client_key), 'server_name_override': 'localhost'},
                                'environment': {'kind': 'dedicated_test', 'dedicated_test_environment_confirmed': True},
                                'execution_policy': {'probe_unsupported': True, 'allow_mutations': True}}
                            connection_path, catalog_path, result_path, trace_path = (directory / name for name in ('connection.json', 'catalog.json', 'result.json', 'calls.tsv'))
                            trace_path.unlink(missing_ok=True)
                            connection_path.write_text(json.dumps(config))
                            register_object_path = (
                                str(certificate_request_der) if case_id == 'test-obj-certificate-request' else
                                str(certificate_der) if case_id in ('test-op-register', 'test-obj-certificate') or
                                (attribute_fixture and attribute_fixture.get('register_certificate')) else None)
                            peer_case = case_for_controlled_exchange(
                                case, attribute_fixture, register_object_path)
                            catalog_path.write_text(json.dumps({**catalog, 'test_cases': [peer_case]}))
                            run_environment = {
                                **os.environ,
                                'KMIPKIT_C_ROUTE_OBSERVATIONS': str(trace_path),
                            }
                            if recipe[1] in ('get_usage_allocation_uid_variant',
                                             'get_usage_allocation_uid_variants'):
                                run_environment['KMIPKIT_C_USAGE_ALLOCATION_IDENTIFIER_FORM'] = (
                                    'explicit' if variant == 'usage-allocation-explicit' else 'omitted')
                            elif case_id.startswith('test-norm-payload-client-export-request-'):
                                run_environment['KMIPKIT_C_EXPORT_REQUEST_FIELD'] = case['requires']['field_name']
                                run_environment['KMIPKIT_C_EXPORT_REQUEST_VARIANT'] = variant
                            else:
                                run_environment['KMIPKIT_C_ATTRIBUTE_REQUEST_ROUTE'] = variant
                            if case_id in {
                                    'test-norm-payload-client-get-attribute-list-request-20-001',
                                    'test-norm-payload-client-get-attribute-list-request-21-001'}:
                                run_environment['KMIPKIT_C_ATTRIBUTE_REQUEST_IDENTIFIER'] = 'fixture-key-1'
                            if recipe[1] == 'modify_attribute':
                                run_environment['KMIPKIT_C_MODIFY_CURRENT_ATTRIBUTE'] = (
                                    'present' if case_id.endswith('-002') else 'omitted')
                            run = subprocess.run([str(binary.resolve()), 'run', '--connection', str(connection_path),
                                '--catalog', str(catalog_path), '--allow-mutations', '--output', str(result_path)],
                                cwd=ROOT, capture_output=True,
                                timeout=30 if attribute_mutation_recipe else 10,
                                env=run_environment)
                            if thread is not None:
                                thread.join(20 if attribute_mutation_recipe else 6)
                            if recipe[1] == 'create_split_key' and len(observation) == 3:
                                observation[0][2]['effects'] = [
                                    effect for _, _, evidence in observation
                                    for effect in evidence['effects']]
                                observation[0][2]['statuses'] = [
                                    status for _, _, evidence in observation
                                    for status in evidence['statuses']]
                                observation[0][2]['reasons'] = [
                                    reason for _, _, evidence in observation
                                    for reason in evidence['reasons']]
                                observation[0][2]['peer_observed_state_after_sha256'] = \
                                    observation[-1][2]['peer_observed_state_after_sha256']
                            if recipe[1] == 'join_split_key' and len(observation) == 2:
                                observation[1][2]['effects'] = [
                                    effect for _, _, evidence in observation
                                    for effect in evidence['effects']]
                                observation[1][2]['statuses'] = [
                                    status for _, _, evidence in observation
                                    for status in evidence['statuses']]
                                observation[1][2]['reasons'] = [
                                    reason for _, _, evidence in observation
                                    for reason in evidence['reasons']]
                                observation[1][2]['peer_observed_state_before_sha256'] = \
                                    observation[0][2]['peer_observed_state_before_sha256']
                            if recipe[1] == 'rekey' and len(observation) == 2:
                                observation[1][2]['effects'] = [
                                    effect for _, _, evidence in observation
                                    for effect in evidence['effects']]
                                observation[1][2]['statuses'] = [
                                    status for _, _, evidence in observation
                                    for status in evidence['statuses']]
                                observation[1][2]['reasons'] = [
                                    reason for _, _, evidence in observation
                                    for reason in evidence['reasons']]
                                observation[1][2]['peer_observed_state_before_sha256'] = \
                                    observation[0][2]['peer_observed_state_before_sha256']
                            result = json.loads(result_path.read_text()) if result_path.exists() else {}
                            verify_consumer_provenance(result, artifact_sha256, catalog_sha256)
                            result_case = (result.get('cases') or [{}])[0]
                            expected_rejection_status = ('server_unsupported'
                                if peer_rejection and peer_rejection.get('result_reason') in (5, 8) else
                                'policy_restricted'
                                if peer_rejection and peer_rejection.get('result_reason') in (12, 57) else None)
                            expected_rejection_observed = reject_case and result_case.get('status') == expected_rejection_status
                            response_rejection_observed = (
                                case_id.startswith('test-norm-payload-client-export-response-') and
                                variant == 'missing_rejected' and result_case.get('status') == 'failed' and
                                'category=2 cause=2 delivery=3' in result_case.get('diagnostic', '') and
                                result_case.get('library_correctness') == 'not_checked' and
                                result_case.get('server_compatibility') == 'unknown')
                            case_passed = bool(result_case and (
                                result_case.get('status') == 'passed' or expected_rejection_observed or
                                response_rejection_observed))
                            local_placeholder_rejection_observed = (
                                local_placeholder_rejection and result_case.get('status') == 'passed' and
                                result_case.get('server_compatibility') == 'unknown' and
                                result_case.get('library_correctness') == 'passed')
                            if local_placeholder_rejection_observed:
                                case_passed = True
                            trace = [line.split('\t') for line in trace_path.read_text().splitlines()] if trace_path.exists() else []
                            called_symbols = {symbol for symbol, status in trace if status == '0'}
                            invoked_symbols = {symbol for symbol, _ in trace}
                            if boundary:
                                if select.select([server], [], [], 0)[0]:
                                    errors.append('fixture_unexpected_transport_connection')
                                if not result.get('cases') or result['cases'][0].get('server_compatibility') != 'unknown':
                                    errors.append('fixture_not_sent_outcome_missing')
                                # The unsupported typed request has no payload fields.
                                # Its recipe supplies the operation code; the configured
                                # public client rejects it before transport, so no response exists.
                                operation_code = recipe[0][0]
                                version_node = (0x420069, 1, [(0x42006A, 2, bytes.fromhex('00000002')),
                                    (0x42006B, 2, int(version[-1]).to_bytes(4, 'big'))])
                                request_header = [version_node]
                                if local_placeholder_rejection:
                                    request_header.append((0x42000D, 2, (1).to_bytes(4, 'big')))
                                batch_item_fields = [
                                    (0x42005C, 5, operation_code.to_bytes(4, 'big')),
                                    (0x420079, 1, []),
                                ]
                                if local_placeholder_rejection:
                                    batch_item_fields.insert(1, (0x420093, 8, b'orphan-activate'))
                                attempted = (0x420078, 1, [(0x420077, 1, request_header),
                                    (0x42000F, 1, batch_item_fields)])
                                observation.append((attempted, None, {}))
                            code = errors[0] if errors else 'consumer_check_failed' if not case_passed else 'public_api_trace_missing' if not called_symbols else None
                            mutation_index = mutation_exchange_index if attribute_mutation_recipe else \
                                target_operation_exchange_index(recipe)
                            attribute_check = {}
                            if observation and attribute_fixture:
                                request_item = next(item for item in observation[0][0][2]
                                                    if item[0] == 0x42000F and
                                                    numbers(item, 0x42005C) == [11])
                                attribute_check = {
                                    'request_attribute_reference_tags': [
                                        f'{tag:06X}' for tag in numbers(request_item, 0x42013B)],
                                    'request_attribute_reference_names': text_values(request_item, 0x42000A),
                                    'request_vendor_identifications': text_values(request_item, 0x42009D),
                                    'response_attribute_tags': [
                                        f'{item[0]:06X}' for item in next(
                                            descendants(observation[0][1], 0x420125))[2]],
                                }
                                if attribute_fixture.get('attribute_reference_kind') == 'all':
                                    register_item = next(item for item in observation[0][0][2]
                                                         if item[0] == 0x42000F and
                                                         numbers(item, 0x42005C) == [3])
                                    register_payload = next(item for item in register_item[2]
                                                            if item[0] == 0x420079 and item[1] == 1)
                                    register_attributes = next(item for item in register_payload[2]
                                                               if item[0] == 0x420125 and item[1] == 1)
                                    attribute_check['request_certificate_attribute_tags'] = [
                                        f'{item[0]:06X}' for item in register_attributes[2]]
                            batch_continuation_option_check = {}
                            if observation and case_id in BATCH_CONTINUATION_CONSTRUCTOR_CASES:
                                batch_continuation_option_check = {
                                    'peer_batch_error_continuation_option_values': numbers(
                                        observation[0][0], 0x42000E),
                                }
                            checks.append({'case_id': case_id, 'constructor_variant': variant, 'protocol_version': version, 'encoding': encoding,
                                           'transport': transport, 'status': 'passed' if observation and not code and (run.returncode == 0 or response_rejection_observed) else 'failed',
                                           'consumer_case_status': (result.get('cases') or [{}])[0].get('status', 'missing'),
                                           'consumer_server_compatibility': (result.get('cases') or [{}])[0].get('server_compatibility', 'missing'),
                                           'consumer_library_correctness': (result.get('cases') or [{}])[0].get('library_correctness', 'missing'),
                                           'consumer_result_status_code': (result.get('cases') or [{}])[0].get('result_status_code'),
                                           'consumer_result_reason_code': (result.get('cases') or [{}])[0].get('result_reason_code'),
                                           'consumer_diagnostic': (result.get('cases') or [{}])[0].get('diagnostic', ''),
                                           'public_api_trace_symbols': sorted(invoked_symbols),
                                           'public_api_success_symbols': sorted(called_symbols),
                                           **({'local_placeholder_activate_rejected':
                                                   local_placeholder_rejection_observed and
                                                   numbers(observation[0][0], 0x42005C) == [18],
                                               'local_placeholder_batch_count_is_one':
                                                   numbers(observation[0][0], 0x42000D) == [1],
                                               'local_placeholder_unique_identifier_is_omitted':
                                                   not list(descendants(observation[0][0], UNIQUE_IDENTIFIER_TAG)),
                                               'local_placeholder_options_are_omitted':
                                                   not list(descendants(observation[0][0], 0x42000E)) and
                                                   not list(descendants(observation[0][0], 0x420011)),
                                               'local_placeholder_transport_was_not_invoked':
                                                   boundary and not errors and observation[0][1] is None}
                                              if local_placeholder_rejection and observation else {}),
                                           **({'peer_async_response_statuses': [
                                                   numbers(item, 0x42007F)[0]
                                                   for item in observation[0][1][2]
                                                   if item[0] == BATCH_ITEM_TAG],
                                               'peer_async_correlation_match':
                                                   byte_string_values(observation[0][1], 0x420006) ==
                                                   [CANCEL_CORRELATION_VALUE],
                                               'peer_async_empty_response_payload':
                                                   response_has_single_empty_payload(observation[0][1])}
                                              if recipe[1] == 'async_create_pending' else {}),
                                           **({'peer_async_response_statuses': [
                                                   numbers(item, 0x42007F)[0]
                                                   for item in observation[0][1][2]
                                                   if item[0] == BATCH_ITEM_TAG],
                                               'peer_async_indicator_omitted':
                                                   not numbers(next(descendants(observation[0][0], 0x420077),
                                                                    observation[0][0]), 0x420007),
                                               'peer_async_correlation_absent':
                                                   not byte_string_values(observation[0][1], 0x420006)}
                                              if recipe[1] == 'async_create_sync_default' else {}),
                                           **(sync_create_peer_check(
                                               observation, 2, 'optional')
                                               if recipe[1] == 'async_create_optional_complete' else {}),
                                           **(sync_create_peer_check(
                                               observation, 3, 'prohibited')
                                               if recipe[1] == 'async_create_prohibited_sync' else {}),
                                           **({'peer_async_interleaving_operations': [
                                                   numbers(request[0], 0x42005C)[0]
                                                   for request in observation],
                                               'peer_async_response_statuses': [
                                                   numbers(item, 0x42007F)[0]
                                                   for _, response_tree, _ in observation
                                                   for item in response_tree[2]
                                                   if item[0] == BATCH_ITEM_TAG],
                                               'peer_async_terminal_state_count': sum(
                                                   status != 2 for _, response_tree, _ in observation
                                                   for item in response_tree[2]
                                                   if item[0] == BATCH_ITEM_TAG
                                                   for status in numbers(item, 0x42007F))}
                                              if recipe[1] == 'cancel_poll_race' else {}),
                                           **(async_lifecycle_peer_check(observation)
                                              if recipe[1] == 'async_create_poll_lifecycle' and
                                              len(observation) == 4 else {}),
                                           **(nonrecursive_poll_cancel_peer_check(observation)
                                              if recipe[1] == 'async_poll_cancel_no_recursion' and
                                              len(observation) == 2 else {}),
                                           **(async_failure_poll_peer_check(observation)
                                              if recipe[1] == 'async_poll_final_failure' and
                                              len(observation) == 3 else {}),
                                           **(async_repeated_pending_poll_peer_check(observation)
                                              if recipe[1] == 'async_poll_repeated_pending_to_success' and
                                              len(observation) == 5 else {}),
                                           **(async_process_then_poll_peer_check(observation)
                                              if recipe[1] == 'async_process_then_poll_state' and
                                              len(observation) == 4 else {}),
                                           **(async_query_outstanding_peer_check(observation)
                                              if recipe[1] == 'async_query_outstanding_filter' and
                                              len(observation) == 9 else {}),
                                           **(query_async_empty_peer_check(observation)
                                              if recipe[1] == 'query_async_empty' and
                                              len(observation) == 3 else {}),
                                           **(batch_single_item_default_continuation_peer_check(observation)
                                              if recipe[1] == 'batch_single_item_default_continuation' and
                                              len(observation) == 1 else {}),
                                           **(async_unknown_correlation_peer_check(observation)
                                              if recipe[1] == 'async_unknown_correlation_rejected' and
                                              len(observation) == 1 else {}),
                                           **({'peer_final_state_matches_setup':
                                                   observation[0][2]['peer_observed_state_before_sha256'] ==
                                                   observation[0][2]['peer_observed_state_after_sha256']}
                                              if observation and recipe[1] == 'batch_register_crypto' else {}),
                                           **({'peer_seed_operations': [peer_rejection['operation']],
                                               'peer_seed_effects': observation[1][2]['effects'],
                                               'peer_cleanup_operations': [
                                                   numbers(item, 0x42005C)[0]
                                                   for item in observation[-1][0][2]
                                                   if item[0] == 0x42000F],
                                               'peer_final_state_matches_setup':
                                                   observation[0][2]['peer_observed_state_before_sha256'] ==
                                                   observation[-1][2]['peer_observed_state_after_sha256']}
                                              if expected_rejection_observed and len(observation) == 3 else {}),
                                           **({'peer_response_operation_values': [
                                               numbers(item, 0x42005C)[0]
                                               for item in observation[0][1][2]
                                               if item[0] == 0x42000F]}
                                               if observation and case_id in {
                                                   'test-batch-count-matches-items',
                                                   'test-batch-count-mismatch-rejected',
                                                   'test-batch-unique-item-id-response-correlation',
                                                   'test-batch-order-false-correlates-response-items',
                                                   'test-batch-order-required-placeholder-resolution',
                                                   'test-batch-order-omitted-defaults-true',
                                                   'test-batch-continue-after-failure',
                                                   'test-batch-stop-on-first-failure',
                                                   'test-norm-payload-client-get-usage-allocation-request-20-001',
                                                   'test-norm-payload-client-get-usage-allocation-request-21-001'} else {}),
                                           **({'peer_response_result_statuses': [
                                               numbers(item, 0x42007F)[0]
                                               for item in observation[0][1][2]
                                               if item[0] == 0x42000F]}
                                               if observation and case_id in {
                                                   'test-batch-count-matches-items',
                                                   'test-batch-count-mismatch-rejected',
                                                   'test-batch-unique-item-id-response-correlation',
                                                   'test-batch-order-false-correlates-response-items',
                                                   'test-batch-order-required-placeholder-resolution',
                                                   'test-batch-order-omitted-defaults-true',
                                                   'test-batch-continue-after-failure',
                                                   'test-batch-stop-on-first-failure',
                                                   'test-norm-payload-client-get-usage-allocation-request-20-001',
                                                   'test-norm-payload-client-get-usage-allocation-request-21-001'} else {}),
                                           **({'peer_response_unique_batch_item_ids': [
                                               value.decode('utf-8') for item in observation[0][1][2]
                                               if item[0] == BATCH_ITEM_TAG
                                               for value in byte_string_values(item, 0x420093)]}
                                              if observation and case_id ==
                                              'test-batch-stop-on-first-failure' else {}),
                                           **batch_continuation_option_check,
                                           **({'peer_observed_effects': observation[0][2]['effects'],
                                               'peer_observed_state_before_sha256': observation[0][2][
                                                   'peer_observed_state_before_sha256'],
                                               'peer_observed_state_after_sha256': observation[0][2][
                                                   'peer_observed_state_after_sha256']}
                                              if observation and case_id ==
                                              'test-batch-stop-on-first-failure' else {}),
                                           **({'peer_request_unique_identifier_present_by_batch_item': [
                                               bool(text_values(item, UNIQUE_IDENTIFIER_TAG))
                                               for item in observation[0][0][2]
                                               if item[0] == BATCH_ITEM_TAG],
                                               'peer_response_unique_identifier_present_by_batch_item': [
                                               bool(text_values(item, UNIQUE_IDENTIFIER_TAG))
                                               for item in observation[0][1][2]
                                               if item[0] == BATCH_ITEM_TAG]}
                                               if observation and case_id in {
                                                   'test-norm-payload-client-get-usage-allocation-request-20-001',
                                                   'test-norm-payload-client-get-usage-allocation-request-21-001',
                                                   'test-norm-payload-client-get-attribute-list-request-20-001',
                                                   'test-norm-payload-client-get-attribute-list-request-21-001'} else {}),
                                           **({'peer_new_attribute_present': request_add_attribute_has_new_attribute(
                                                   observation[mutation_exchange_index][0])}
                                              if observation and recipe[1] == 'add_attribute' and case_id in {
                                                  'test-norm-payload-client-add-attribute-request-20-002',
                                                  'test-norm-payload-client-add-attribute-request-21-002'} else {}),
                                           **({'peer_current_attribute_present':
                                               request_modify_attribute_fields(
                                                   next(item for item in observation[mutation_exchange_index][0][2]
                                                        if item[0] == BATCH_ITEM_TAG and
                                                        numbers(item, 0x42005C) == [14]))[0] is not None,
                                               'peer_new_attribute_present':
                                               request_modify_attribute_fields(
                                                   next(item for item in observation[mutation_exchange_index][0][2]
                                                        if item[0] == BATCH_ITEM_TAG and
                                                        numbers(item, 0x42005C) == [14]))[1] is not None}
                                              if len(observation) > mutation_exchange_index and
                                              recipe[1] == 'modify_attribute' and
                                              case_id.startswith(
                                                  'test-norm-payload-client-modify-attribute-request-') else {}),
                                           **(peer_cleanup_evidence(observation, include_state_hash=True)
                                              if (placeholder_chain_recipe or batch_order_placeholder_recipe) and
                                              len(observation) == 2 else {}),
                                           **({'peer_request_operations': [
                                                   numbers(item, 0x42005C)[0]
                                                   for item in observation[0][0][2]
                                                   if item[0] == BATCH_ITEM_TAG],
                                               'peer_request_batch_order_option': numbers(
                                                   observation[0][0], 0x420010)}
                                              if batch_order_placeholder_recipe and
                                              len(observation) == 2 else {}),
                                           **({'peer_request_operations': [
                                                   numbers(item, 0x42005C)[0]
                                                   for item in observation[mutation_index][0][2]
                                                   if item[0] == 0x42000F],
                                                'peer_observed_effects':
                                                    observation[mutation_index][2]['effects'],
                                                **({'peer_seed_operations': [
                                                        numbers(item, 0x42005C)[0]
                                                        for item in observation[1][0][2]
                                                        if item[0] == 0x42000F],
                                                    'peer_seed_effects': observation[1][2]['effects'],
                                                    'peer_attribute_seeded':
                                                        observation[1][2]['peer_observed_state_before_sha256'] !=
                                                        observation[1][2]['peer_observed_state_after_sha256']}
                                                   if seeded_attribute_recipe else {}),
                                                'peer_setup_operations': [
                                                    numbers(item, 0x42005C)[0]
                                                    for item in observation[0][0][2]
                                                    if item[0] == 0x42000F],
                                                **peer_cleanup_evidence(observation),
                                                'peer_attribute_state_changed':
                                                    observation[mutation_index][2][
                                                        'peer_observed_state_before_sha256'] !=
                                                    observation[mutation_index][2][
                                                        'peer_observed_state_after_sha256'],
                                                }
                                               if attribute_mutation_recipe and len(observation) ==
                                               (4 if seeded_attribute_recipe else 3) else {}),
                                           **attribute_check,
                                           **({'failure_code': code or 'peer_observation_missing'} if code or not observation else {})})
                            if not observation or code or run.returncode != 0:
                                continue
                            for cell in cells_by_case[case_id]:
                                if _key(cell) in keys:
                                    continue
                                if cell['protocol_version'] != version or cell['public_api_symbol'] not in called_symbols:
                                    continue
                                selected_observation = observation_for_cell(
                                    observation, cell, recipe, mutation_index)
                                if selected_observation is None:
                                    continue
                                request_tree, response_tree, state_evidence = selected_observation
                                response_batch_item_count = len([
                                    item for item in response_tree[2]
                                    if item[0] == BATCH_ITEM_TAG and item[1] == 1
                                ]) if response_tree is not None else 0
                                is_network = cell['transport'] is not None
                                if is_network:
                                    if (cell['encoding'], cell['transport']) != (encoding, transport):
                                        continue
                                    if cell['execution_api_symbol'] not in invoked_symbols:
                                        continue
                                else:
                                    # Accessor routes remain local TTLV obligations.
                                    # Emit them once from an actual decoded TLS response.
                                    batch_continuation_constructor = (
                                        case_id in BATCH_CONTINUATION_CONSTRUCTOR_CASES and
                                        cell['action'] == 'construct' and
                                        cell['public_api_symbol'] ==
                                        'kmipkit_client_batch_add_error_continuation_value')
                                    async_request_constructor = (
                                        recipe[1] in (
                                            'cancel_poll_race',
                                            'async_poll_cancel_no_recursion',
                                                'async_poll_final_failure',
                                                'async_poll_repeated_pending_to_success',
                                            'async_process_then_poll_state',
                                            'async_unknown_correlation_rejected') and
                                        cell['action'] == 'construct' and
                                        cell['public_api_symbol'] in {
                                            'kmipkit_poll_request_create',
                                            'kmipkit_cancel_request_create'})
                                    async_process_constructor = (
                                        recipe[1] == 'async_process_then_poll_state' and
                                        cell['action'] == 'construct' and
                                        cell['public_api_symbol'] ==
                                        'kmipkit_process_request_create')
                                    async_lifecycle_constructor = (
                                        recipe[1] in ('async_create_poll_lifecycle',
                                                      'async_query_outstanding_filter') and
                                        cell['action'] == 'construct' and
                                        cell['public_api_symbol'] in {
                                            'kmipkit_poll_request_create',
                                            'kmipkit_query_async_requests_request_create',
                                            'kmipkit_query_async_requests_request_with_correlation_values',
                                            'kmipkit_query_async_requests_request_with_operations'})
                                    if ((encoding, transport) != ('TTLV', 'raw_tls') or
                                            (cell['action'] not in ('decode', 'encode') and
                                             not batch_continuation_constructor and
                                             not async_request_constructor and
                                             not async_process_constructor and
                                             not async_lifecycle_constructor)):
                                        continue
                                    if (cell['action'] == 'decode' and
                                            not cell['public_api_symbol'].startswith('kmipkit_client_batch_') and
                                            cell['public_api_symbol'] !=
                                            'kmipkit_client_async_operation_outcome_cancel_response_v1' and
                                            cell['public_api_symbol'] !=
                                            'kmipkit_client_async_operation_outcome_poll_response_v1' and
                                            cell['public_api_symbol'] !=
                                            'kmipkit_client_async_operation_outcome_process_response_v1' and
                                            cell['public_api_symbol'] !=
                                            'kmipkit_client_async_operation_outcome_query_async_requests_response_v1'):
                                        continue
                                    if cell['action'] == 'encode' and cell['public_api_symbol'] not in {
                                            'kmipkit_cancel_request_to_ttlv_payload',
                                            'kmipkit_poll_request_to_ttlv_payload',
                                            'kmipkit_process_request_to_ttlv_payload',
                                            'kmipkit_query_async_requests_request_to_ttlv_payload',
                                            'kmipkit_add_attribute_request_to_ttlv_payload',
                                            'kmipkit_adjust_attribute_request_to_ttlv_payload',
                                            'kmipkit_set_attribute_request_to_ttlv_payload',
                                            'kmipkit_delete_attribute_request_to_ttlv_payload',
                                            'kmipkit_get_attributes_request_to_ttlv_payload',
                                            'kmipkit_get_attribute_list_request_to_ttlv_payload'}:
                                        continue
                                target = targets_by_id[cell['target_id']]
                                response_accessor_route = (
                                    cell['public_api_symbol'].startswith(
                                        'kmipkit_client_batch_item_response_') or
                                    cell['public_api_symbol'] ==
                                    'kmipkit_client_async_operation_outcome_cancel_response_v1' or
                                    cell['public_api_symbol'] ==
                                    'kmipkit_client_async_operation_outcome_poll_response_v1' or
                                    cell['public_api_symbol'] ==
                                    'kmipkit_client_async_operation_outcome_process_response_v1')
                                if response_accessor_route:
                                    source_sections = {reference.get('section')
                                                      for reference in target.get('source_refs', [])}
                                    if (case_id == 'test-batch-stop-on-first-failure' and
                                            '8.3' in source_sections):
                                        continue
                                    response_item_index = cell.get('workflow_item_index')
                                    if (response_item_index is not None and
                                            response_item_index >= response_batch_item_count):
                                        continue
                                if target['kind'] == 'message_field':
                                    field_tag = codec.tags.get(normalize(target['name']))
                                    if field_tag is None:
                                        continue
                                    field_tree = response_tree if response_accessor_route else request_tree
                                    field_present = bool(list(descendants(field_tree, field_tag)))
                                    if case_id == 'test-batch-single-item-options-omitted':
                                        expected_presence = {
                                            'Batch Count': True,
                                            'Batch Error Continuation Option': False,
                                            'Batch Order Option': False,
                                            'Batch Item': True,
                                        }.get(target['name'])
                                        if expected_presence is None or field_present != expected_presence:
                                            continue
                                    elif not field_present:
                                        continue
                                elif target['kind'] == 'operation':
                                    operation_value = codec.values.get(
                                        (codec.tag('Operation'), normalize(target['name'])))
                                    operation_tree = response_tree if response_accessor_route else request_tree
                                    if operation_value is None or operation_value not in numbers(
                                            operation_tree, 0x42005C):
                                        continue
                                elif target['kind'] == 'enumeration_value':
                                    option_tag = codec.tag('Batch Error Continuation Option')
                                    expected_value = int(target['wire_value'], 16)
                                    if (case_id not in BATCH_CONTINUATION_CONSTRUCTOR_CASES or
                                            cell['action'] != 'construct' or
                                            numbers(request_tree, option_tag) != [expected_value]):
                                        continue
                                elif target['kind'] == 'operation_payload_field':
                                    if cell['action'] == 'decode':
                                        if (target.get('role') != 'response'
                                                or not any(response_payload_field_present(target, item,
                                                    targets_by_id, codec)
                                                           for item in descendants(response_tree, BATCH_ITEM_TAG))):
                                            continue
                                    elif (target.get('role') != 'request' or
                                          not any(request_payload_field_present(target, item,
                                              targets_by_id, codec)
                                                  for item in descendants(request_tree, BATCH_ITEM_TAG)) or
                                          (cell['action'] not in ('construct', 'encode') and not is_network)):
                                        continue
                                else:
                                    continue
                                event = cell_fields(cell)
                                event.update(status='passed', invocation_count=1, invocation_id=f'c-peer-{len(events)}')
                                digest_tree = (response_tree if response_tree is not None else request_tree) \
                                    if cell['action'] == 'decode' else request_tree
                                if cell['action'] == 'encode' and target['kind'] == 'operation':
                                    target_operation = codec.values.get(
                                        (codec.tag('Operation'), normalize(target['name'])))
                                    target_item = next(item for item in request_tree[2]
                                                       if item[0] == 0x42000F and
                                                       direct_numbers(item, 0x42005C) == [target_operation])
                                    payload = next((child for child in target_item[2]
                                                    if child[0] == 0x420079 and child[1] == 1), None)
                                    digest_tree = payload or target_item
                                if boundary and cell['action'] == 'send':
                                    event.update(request_delivery_state='not_sent',
                                                 transport_invocation_count=0)
                                    if local_placeholder_rejection:
                                        event.update(
                                            batch_operation_sequence=['Activate'],
                                            batch_item_count=1,
                                            local_rejection_code='batch_placeholder_forward_reference',
                                            client_error_category='validation',
                                            client_error_cause='invalid_input')
                                elif cell['action'] == 'send':
                                    event.update(request_delivery_state='response_received', transport_invocation_count=1)
                                if not (boundary and cell['action'] == 'send'):
                                    event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = tree_digest(digest_tree)
                                if (cell['action'] == 'send' and not boundary and
                                        (len(recipe[0]) > 1 or
                                         recipe[1] in ('batch_single_item_default_continuation',
                                                       'batch_placeholder_without_producer')) and
                                        not attribute_mutation_recipe):
                                    operations = [next(t['name'] for t in catalog['normative_targets'] if t['kind'] == 'enumeration_value' and t.get('wire_value') == f'{op:08X}' and any(p.endswith('ENUMERATION-OPERATION') for p in t.get('parent_element_ids', []))) for op in recipe[0]]
                                    response_item_count = len([item for item in response_tree[2]
                                                               if item[0] == BATCH_ITEM_TAG and item[1] == 1])
                                    event.update(batch_operation_sequence=operations, batch_item_count=len(operations),
                                        peer_observed_operation_sequence=operations,
                                        peer_observed_response_count=response_item_count,
                                        peer_observed_result_status_codes=numbers(response_tree, 0x42007F),
                                        peer_observed_effects=state_evidence['effects'],
                                        peer_observed_state_before_sha256=state_evidence[
                                            'peer_observed_state_before_sha256'],
                                        peer_observed_state_after_sha256=state_evidence[
                                            'peer_observed_state_after_sha256'])
                                events.append(event)
                                keys.add(_key(cell))
    return events, keys, checks
