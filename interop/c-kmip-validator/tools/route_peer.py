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
UNIQUE_IDENTIFIER_TAG = 0x420094
ENUMERATION_TTLV_TYPE = 5
ID_PLACEHOLDER_ENUMERATION = 1
PLACEHOLDER_CHAIN_OPERATIONS = (1, 18, 10)
PLACEHOLDER_CLEANUP_OPERATIONS = (19, 20)

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


def text_values(node, tag):
    return [item[2].decode('utf-8') for item in descendants(node, tag) if item[1] == 7]


def byte_string_values(node, tag):
    return [item[2] for item in descendants(node, tag) if item[1] == 8]


def request_payload_field_present(target, batch_item, targets, codec):
    """Check a request payload field under its exact operation's payload."""
    if (target.get('kind') != 'operation_payload_field'
            or target.get('role') != 'request'
            or not isinstance(target.get('operation_id'), str)
            or batch_item[0] != 0x42000F or batch_item[1] != 1):
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
                if child[0] == codec.tag('Request Payload') and child[1] == 1]
    return len(payloads) == 1 and any(child[0] == field_tag for child in payloads[0][2])


def tree_digest(node):
    # Redact nondeterministic and sensitive fields before hashing. Preserve
    # numeric result codes, tag/type distinctions, and ordered batch structure.
    sensitive = {0x420094, 0x420092, 0x420099, 0x4200A1, 0x420043, 0x42014B}
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
    'test-op-hash': ((39,), 'hash'),
    'test-crypto-hash-sha_256-0': ((39,), 'hash'),
    'test-op-rng-retrieve': ((37,), 'rng_retrieve'),
    'test-crypto-rng-retrieve-rng-32': ((37,), 'rng_retrieve'),
    'test-op-rng-seed': ((38,), 'rng_seed'),
    'test-crypto-rng-seed-rng-32': ((38,), 'rng_seed'),
    'test-v20-reject-get-constraints': ((56,), 'get_constraints'),
    'test-op-create': ((1, 20), 'create'),
    'test-op-register': ((3, 20), 'register'),
    'test-op-add-attribute': ((1, 13, 19, 20), 'add_attribute'),
    'test-op-set-attribute': ((1, 49, 19, 20), 'set_attribute'),
    'test-op-set-constraints': ((55,), 'set_constraints'),
    'test-op-set-defaults': ((54,), 'set_defaults'),
    'test-op-delete-attribute': ((1, 13, 15, 19, 20), 'delete_attribute'),
    'test-obj-certificate': ((3, 20), 'register'),
    'test-obj-certificate-request': ((3, 20), 'register_certificate_request'),
    'test-batch-create-activate-destroy': ((1, 18, 12, 19, 20), 'create_activate_attribute_list_revoke_destroy'),
    'test-batch-placeholder-chain-resolves': (PLACEHOLDER_CHAIN_OPERATIONS, 'batch_placeholder_chain'),
    'test-batch-single-item-options-omitted': ((10,), 'batch_single_item_options_omitted'),
    'test-batch-count-matches-items': ((10, 12), 'batch_count_two_read_items'),
    'test-batch-unique-item-id-response-correlation': ((10, 12), 'batch_count_two_read_items'),
    'test-batch-order-false-correlates-response-items': ((10, 12), 'batch_order_false_two_read_items'),
    'test-batch-continue-after-failure': ((10, 12), 'batch_continue_after_failure'),
    'test-norm-payload-client-create-request-20-001': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-20-002': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-21-001': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-21-002': ((1, 20), 'create'),
    'test-norm-payload-client-create-request-21-003': ((1, 20), 'create'),
    'test-op-get': ((1, 10, 20), 'get'),
    'test-op-get-attributes': ((1, 11, 20), 'get_attributes'),
    'test-op-get-attribute-list': ((1, 12, 20), 'get_attribute_list'),
    'test-op-check': ((1, 9, 19, 20), 'check'),
    'test-op-activate': ((1, 18, 19, 20), 'activate'),
    'test-op-revoke': ((1, 19, 20), 'revoke'),
    'test-op-destroy': ((1, 20, 19, 20), 'destroy'),
    'test-op-get-constraints': ((1, 56, 19, 20), 'get_constraints'),
    'test-op-get-usage-allocation': ((1, 17, 19, 20), 'get_usage_allocation'),
    'test-op-locate': ((8,), 'locate'), 'test-op-query': ((24,), 'query'),
    'test-norm-payload-client-query-request-20-001': ((24,), 'query'),
    'test-norm-payload-client-query-request-21-001': ((24,), 'query'),
    'test-norm-payload-client-query-request-21-002': ((24,), 'query'),
    'test-op-discover-versions': ((30,), 'discover_versions'),
    'test-op-log': ((44,), 'log'), 'test-op-login': ((45,), 'login'),
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
    'test-norm-payload-client-query-request-20-001',
    'test-norm-payload-client-query-request-21-001',
    'test-norm-payload-client-query-request-21-002',
    'test-norm-payload-client-create-request-20-001',
    'test-norm-payload-client-create-request-20-002',
    'test-norm-payload-client-create-request-21-001',
    'test-norm-payload-client-create-request-21-002',
    'test-norm-payload-client-create-request-21-003',
})


def assert_recipe(node, case_id, version, recipe=None, attribute_fixture=None):
    selected_recipe = recipe or RECIPES[case_id]
    expected, _ = selected_recipe
    if tuple(numbers(node, 0x42005C)) != expected:
        raise ValueError('fixture_operation_order_mismatch')
    assert_placeholder_usage(node, case_id)
    header = next(descendants(node, 0x420077), node)
    if numbers(header, 0x42006A) != [2] or numbers(header, 0x42006B) != [int(version[-1])]:
        raise ValueError('fixture_protocol_version_mismatch')
    if numbers(node, 0x42000D) != [len(expected)]:
        raise ValueError('fixture_batch_count_mismatch')
    if expected[0] == 1:
        object_type = 4 if case_id == 'test-obj-private-key' else 3 if case_id == 'test-obj-public-key' else 2
        if numbers(node, 0x420057) != [object_type]:
            raise ValueError('fixture_object_type_mismatch')
        if numbers(node, 0x420028) != [4 if object_type in (3, 4) else 3]:
            raise ValueError('fixture_algorithm_mismatch')
        if numbers(node, 0x42002A) != [2048 if object_type in (3, 4) else 128]:
            raise ValueError('fixture_key_length_mismatch')
        if numbers(node, 0x42000E) != [1]:
            raise ValueError('fixture_cleanup_continuation_missing')
        implicit_target = case_id in ('test-op-get-attributes', 'test-op-get-attribute-list',
            'test-batch-create-activate-destroy', 'test-attr-cryptographic-algorithm',
            'test-attr-cryptographic-length', 'test-op-get-constraints') or attribute_fixture is not None
        minimum_unique_identifiers = 0 if case_id == 'test-batch-placeholder-chain-resolves' else (
            len(expected) - 1 - int(implicit_target))
        if len(numbers(node, 0x420094)) < minimum_unique_identifiers:
            raise ValueError('fixture_id_placeholder_missing')
        if len(list(descendants(node, 0x420093))) != len(expected):
            raise ValueError('fixture_batch_item_identity_missing')
        if selected_recipe[1] == 'batch_placeholder_chain':
            items = [child for child in node[2] if child[0] == 0x42000F and child[1] == 1]
            if ([numbers(item, 0x42005C) for item in items] != [[1], [18], [10]] or
                    [byte_string_values(item, 0x420093) for item in items] !=
                    [[b'placeholder-create'], [b'placeholder-activate'], [b'placeholder-get']]):
                raise ValueError('fixture_placeholder_chain_shape_mismatch')
            if numbers(node, 0x42000E) != [1]:
                raise ValueError('fixture_placeholder_chain_continue_option_missing')
    if expected[0] == 3:
        register_item = next(item for item in node[2]
                             if item[0] == 0x42000F and numbers(item, 0x42005C) == [3])
        request_payload = next((item for item in register_item[2]
                                if item[0] == 0x420079 and item[1] == 1), None)
        attributes = next((item for item in request_payload[2]
                           if item[0] == 0x420125 and item[1] == 1), None) if request_payload else None
        if selected_recipe[1] == 'register_certificate_request':
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
    if selected_recipe[1] == 'batch_single_item_options_omitted':
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
                              'batch_continue_after_failure'):
        items = [child for child in node[2]
                 if child[0] == 0x42000F and child[1] == 1]
        first_id = 'missing-key' if selected_recipe[1] == 'batch_continue_after_failure' else 'fixture-key-1'
        if (numbers(node, 0x42000D) != [2] or
                [numbers(item, 0x42005C) for item in items] != [[10], [12]] or
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
                       'test-batch-placeholder-chain-resolves'):
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


def recipe_for_case(case, attribute_fixture):
    recipe = RECIPES.get(case['case_id'])
    if recipe is not None:
        return recipe
    if attribute_fixture is not None:
        if attribute_fixture.get('register_certificate'):
            return ((3, 11, 20), 'get_attributes')
        return ((1, 11, 20), 'get_attributes')
    return None



class PeerState:
    """Maintain the public object lifecycle represented by the controlled peer."""

    def __init__(self, *, existing_object=False):
        self.objects = {'fixture-key-1': 'pre_active'} if existing_object else {}
        self.attributes = {'fixture-key-1': set()} if existing_object else {}

    def digest(self):
        state_value = {'objects': self.objects}
        if any(self.attributes.values()):
            state_value['attributes'] = {
                key: sorted(value) for key, value in self.attributes.items() if value
            }
        state = json.dumps(state_value, sort_keys=True, separators=(',', ':'))
        return hashlib.sha256(state.encode()).hexdigest()

    def execute(self, request_tree, *, continued_batch=False, rejected_operation=None,
                rejection_status=1, rejection_reason=5):
        before = self.digest()
        effects, statuses, reasons = [], [], []
        placeholder = None
        labels = {9: 'checked', 10: 'object_retrieved', 11: 'attributes_retrieved',
                  12: 'attribute_list_retrieved', 13: 'attribute_added',
                  15: 'attribute_deleted',
                  49: 'attribute_set',
                  56: 'constraints_retrieved'}
        items = [item for item in request_tree[2] if item[0] == BATCH_ITEM_TAG]
        for index, item in enumerate(items):
            operation = numbers(item, 0x42005C)[0]
            identifiers = text_values(item, UNIQUE_IDENTIFIER_TAG)
            identifier = identifiers[0] if identifiers else placeholder
            status, reason = 0, 0
            if operation == rejected_operation:
                status, reason, effect = rejection_status, rejection_reason, 'operation_rejected'
            elif operation in (1, 3):
                identifier = 'fixture-key-1'
                if identifier in self.objects:
                    raise ValueError('fixture_duplicate_create')
                self.objects[identifier] = 'pre_active'
                self.attributes[identifier] = set()
                placeholder = identifier
                effect = 'created'
            elif operation in (9, 10, 11, 12, 13, 15, 18, 19, 20, 49, 56):
                state = self.objects.get(identifier)
                if state is None:
                    status, reason, effect = 1, 55, 'object_not_found'
                elif operation == 18:
                    if state != 'pre_active':
                        raise ValueError('fixture_activate_state_mismatch')
                    self.objects[identifier] = 'active'
                    effect = 'activated'
                elif operation == 19:
                    self.objects[identifier] = 'revoked'
                    effect = 'revoked'
                elif operation == 20:
                    if state == 'active':
                        status, reason, effect = 1, 67, 'illegal_destroy_rejected'
                    else:
                        del self.objects[identifier]
                        self.attributes.pop(identifier, None)
                        effect = 'destroyed'
                elif operation in (13, 49):
                    new_attribute = next((child for child in item[2]
                                          if child[0] == REQUEST_PAYLOAD_TAG), None)
                    new_attribute_value = next((child for child in new_attribute[2]
                                                if child[0] == 0x42013D), None) \
                        if new_attribute is not None and new_attribute[1] == 1 else None
                    attribute_tag = new_attribute_value[2][0][0] \
                        if new_attribute_value is not None and new_attribute_value[1] == 1 and \
                        new_attribute_value[2] else None
                    if attribute_tag is None:
                        raise ValueError('fixture_add_attribute_tag_missing')
                    self.attributes.setdefault(identifier, set()).add(attribute_tag)
                    effect = 'attribute_added' if operation == 13 else 'attribute_set'
                elif operation == 15:
                    references = numbers(item, 0x42013B)
                    if references != [0x4200FD]:
                        raise ValueError('fixture_delete_attribute_reference_mismatch')
                    attributes = self.attributes.setdefault(identifier, set())
                    if 0x4200FD not in attributes:
                        raise ValueError('fixture_delete_attribute_target_missing')
                    else:
                        attributes.remove(0x4200FD)
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
                          peer_rejection=None):
    codec = Codec(catalog)
    targets_by_id = {target['target_id']: target for target in catalog['normative_targets']}
    events, keys, checks = [], set(), []
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
        (('direct', 'standalone') if case['case_id'] in {'test-op-get-attributes', 'test-op-get-attribute-list',
                                                         'test-op-add-attribute', 'test-op-set-attribute',
                                                         'test-op-delete-attribute'}
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
                        placeholder_chain_recipe = recipe[1] == 'batch_placeholder_chain'
                        attribute_mutation_recipe = recipe[1] in (
                            'add_attribute', 'set_attribute', 'delete_attribute')
                        delete_attribute_recipe = recipe[1] == 'delete_attribute'
                        attribute_mutation_operation = {
                            'add_attribute': 13, 'set_attribute': 49, 'delete_attribute': 15,
                        }.get(recipe[1])
                        mutation_exchange_index = 2 if delete_attribute_recipe else 1
                        attribute_cleanup_exchange_index = 3 if delete_attribute_recipe else 2
                        reject_case = peer_rejection is not None and peer_rejection.get('case_id') == case_id
                        if version == '2.0' and case_id in ('test-op-discover-versions', 'test-op-login', 'test-op-get-constraints', 'test-op-ping'):
                            continue
                        observation, errors = [], []
                        peer_state = PeerState(existing_object=recipe[0][0] not in (1, 3))
                        boundary = case_id == 'test-v20-reject-get-constraints'
                        with socket.socket() as server:
                            server.bind(('127.0.0.1', 0))
                            server.listen(1)
                            server.settimeout(5)
                            port = server.getsockname()[1]
                            def serve():
                                stream = None
                                try:
                                    exchange_count = 4 if delete_attribute_recipe else 3 if attribute_mutation_recipe else (
                                        2 if placeholder_chain_recipe else 1)
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
                                            placeholder_chain_recipe and exchange_index == 1) or (
                                            attribute_mutation_recipe and
                                            (exchange_index == attribute_cleanup_exchange_index or
                                             (delete_attribute_recipe and seed_rejected and
                                              exchange_index == attribute_cleanup_exchange_index - 1)))
                                        if cleanup_exchange:
                                            if placeholder_chain_recipe:
                                                assert_placeholder_cleanup_request(request_tree, version)
                                            else:
                                                assert_add_attribute_cleanup_request(request_tree, version)
                                            response_operations = PLACEHOLDER_CLEANUP_OPERATIONS
                                        elif attribute_mutation_recipe and exchange_index == 0:
                                            assert_add_attribute_setup_request(request_tree, version)
                                            response_operations = (1,)
                                        elif delete_attribute_recipe and exchange_index == 1:
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
                                            assert_recipe(request_tree, case_id, version, recipe, attribute_fixture)
                                            response_operations = recipe[0]
                                        else:
                                            assert_recipe(request_tree, case_id, version, recipe, attribute_fixture)
                                            response_operations = recipe[0]
                                        selected_tag = {'test-attr-cryptographic-algorithm': 0x420028,
                                                        'test-attr-cryptographic-length': 0x42002A}.get(
                                                            case_id, int(attribute_fixture.get(
                                                                'attribute_response_tag', '000000'), 16)
                                                            if attribute_fixture else 0)
                                        object_type = 4 if case_id == 'test-obj-private-key' else 3 if case_id == 'test-obj-public-key' else 2
                                        response_item_ids = None
                                        if placeholder_chain_recipe:
                                            response_item_ids = (
                                                (b'placeholder-revoke-cleanup', b'placeholder-destroy-cleanup')
                                                if cleanup_exchange else
                                                (b'placeholder-create', b'placeholder-activate', b'placeholder-get'))
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
                                        if recipe[1] in ('batch_count_two_read_items',
                                                         'batch_order_false_two_read_items',
                                                         'batch_continue_after_failure'):
                                            response_item_ids = (b'item-a', b'item-b')
                                            if recipe[1] == 'batch_order_false_two_read_items':
                                                response_operations = tuple(reversed(recipe[0]))
                                                response_item_ids = (b'item-b', b'item-a')
                                        state_evidence = peer_state.execute(
                                            request_tree,
                                            continued_batch=recipe[1] == 'batch_continue_after_failure',
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
                                        response_wire = response(
                                            response_operations, int(version[-1]), selected_tag, object_type,
                                            attribute_fixture['attribute_response_wire'] if attribute_fixture else None,
                                            response_item_ids, response_statuses, response_reasons,
                                            rng_length=(numbers(request_tree, 0x4200C4) or [16])[0]
                                            if recipe[1] == 'rng_retrieve' else
                                            len(byte_string_values(request_tree, 0x4200C2)[0])
                                            if recipe[1] == 'rng_seed' else 16)
                                        response_tree = parse(response_wire)
                                        if placeholder_chain_recipe:
                                            verify_placeholder_chain_response(response_tree, response_operations)
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
                                        else:
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
                                    errors.append('fixture_peer_accept_timeout')
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
                            run = subprocess.run([str(binary.resolve()), 'run', '--connection', str(connection_path),
                                '--catalog', str(catalog_path), '--allow-mutations', '--output', str(result_path)],
                                cwd=ROOT, capture_output=True,
                                timeout=30 if attribute_mutation_recipe else 10,
                                env={**os.environ, 'KMIPKIT_C_ROUTE_OBSERVATIONS': str(trace_path),
                                     'KMIPKIT_C_ATTRIBUTE_REQUEST_ROUTE': variant})
                            if thread is not None:
                                thread.join(20 if attribute_mutation_recipe else 6)
                            result = json.loads(result_path.read_text()) if result_path.exists() else {}
                            verify_consumer_provenance(result, artifact_sha256, catalog_sha256)
                            result_case = (result.get('cases') or [{}])[0]
                            expected_rejection_status = ('server_unsupported'
                                if peer_rejection and peer_rejection.get('result_reason') in (5, 8) else
                                'policy_restricted'
                                if peer_rejection and peer_rejection.get('result_reason') in (12, 57) else None)
                            expected_rejection_observed = reject_case and result_case.get('status') == expected_rejection_status
                            case_passed = bool(result_case and (
                                result_case.get('status') == 'passed' or expected_rejection_observed))
                            trace = [line.split('\t') for line in trace_path.read_text().splitlines()] if trace_path.exists() else []
                            called_symbols = {symbol for symbol, status in trace if status == '0'}
                            invoked_symbols = {symbol for symbol, _ in trace}
                            if boundary:
                                if select.select([server], [], [], 0)[0]:
                                    errors.append('fixture_unexpected_transport_connection')
                                if not result.get('cases') or result['cases'][0].get('server_compatibility') != 'unknown':
                                    errors.append('fixture_not_sent_outcome_missing')
                                # The unsupported typed request has no fields. Its version
                                # is enforced by the configured public client and observed
                                # through the expected pre-send rejection. No response exists.
                                version_node = (0x420069, 1, [(0x42006A, 2, bytes.fromhex('00000002')),
                                    (0x42006B, 2, bytes(4))])
                                attempted = (0x420078, 1, [(0x420077, 1, [version_node]),
                                    (0x42000F, 1, [(0x42005C, 5, (56).to_bytes(4, 'big')), (0x420079, 1, [])])])
                                observation.append((attempted, None, {}))
                            code = errors[0] if errors else 'consumer_check_failed' if not case_passed else 'public_api_trace_missing' if not called_symbols else None
                            mutation_index = mutation_exchange_index if attribute_mutation_recipe else 0
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
                            checks.append({'case_id': case_id, 'constructor_variant': variant, 'protocol_version': version, 'encoding': encoding,
                                           'transport': transport, 'status': 'passed' if observation and not code and run.returncode == 0 else 'failed',
                                           'consumer_case_status': (result.get('cases') or [{}])[0].get('status', 'missing'),
                                           'consumer_server_compatibility': (result.get('cases') or [{}])[0].get('server_compatibility', 'missing'),
                                           'consumer_library_correctness': (result.get('cases') or [{}])[0].get('library_correctness', 'missing'),
                                           'consumer_result_status_code': (result.get('cases') or [{}])[0].get('result_status_code'),
                                           'consumer_result_reason_code': (result.get('cases') or [{}])[0].get('result_reason_code'),
                                           'consumer_diagnostic': (result.get('cases') or [{}])[0].get('diagnostic', ''),
                                           'public_api_trace_symbols': sorted(invoked_symbols),
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
                                                   'test-batch-unique-item-id-response-correlation',
                                                   'test-batch-order-false-correlates-response-items',
                                                   'test-batch-continue-after-failure'} else {}),
                                           **({'peer_response_result_statuses': [
                                               numbers(item, 0x42007F)[0]
                                               for item in observation[0][1][2]
                                               if item[0] == 0x42000F]}
                                               if observation and case_id in {
                                                   'test-batch-count-matches-items',
                                                   'test-batch-unique-item-id-response-correlation',
                                                   'test-batch-order-false-correlates-response-items',
                                                   'test-batch-continue-after-failure'} else {}),
                                           **(peer_cleanup_evidence(observation, include_state_hash=True)
                                              if placeholder_chain_recipe and len(observation) == 2 else {}),
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
                                                   if delete_attribute_recipe else {}),
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
                                               (4 if delete_attribute_recipe else 3) else {}),
                                           **attribute_check,
                                           **({'failure_code': code or 'peer_observation_missing'} if code or not observation else {})})
                            if not observation or code or run.returncode != 0:
                                continue
                            request_tree, response_tree, state_evidence = observation[mutation_index]
                            operation_name = case['operation']
                            for cell in cells_by_case[case_id]:
                                if _key(cell) in keys:
                                    continue
                                if cell['protocol_version'] != version or cell['public_api_symbol'] not in called_symbols:
                                    continue
                                is_network = cell['transport'] is not None
                                if is_network:
                                    if (cell['encoding'], cell['transport']) != (encoding, transport):
                                        continue
                                    if cell['execution_api_symbol'] not in invoked_symbols:
                                        continue
                                else:
                                    # Accessor routes remain local TTLV obligations.
                                    # Emit them once from an actual decoded TLS response.
                                    if (encoding, transport) != ('TTLV', 'raw_tls') or cell['action'] not in ('decode', 'encode'):
                                        continue
                                    if (cell['action'] == 'decode' and not cell['public_api_symbol'].startswith('kmipkit_client_batch_')):
                                        continue
                                    if cell['action'] == 'encode' and cell['public_api_symbol'] not in {
                                            'kmipkit_add_attribute_request_to_ttlv_payload',
                                            'kmipkit_set_attribute_request_to_ttlv_payload',
                                            'kmipkit_delete_attribute_request_to_ttlv_payload',
                                            'kmipkit_get_attributes_request_to_ttlv_payload',
                                            'kmipkit_get_attribute_list_request_to_ttlv_payload'}:
                                        continue
                                target = targets_by_id[cell['target_id']]
                                if target['kind'] == 'message_field':
                                    field_tag = codec.tags.get(normalize(target['name']))
                                    if field_tag is None:
                                        continue
                                    field_present = bool(list(descendants(request_tree, field_tag)))
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
                                    if operation_value is None or operation_value not in numbers(
                                            request_tree, 0x42005C):
                                        continue
                                elif target['kind'] == 'operation_payload_field':
                                    # Request fields are proved from the actual
                                    # request tree during construction and send.
                                    # A response decode cannot prove a request field.
                                    if (not is_network or cell['action'] == 'decode'
                                            or not any(request_payload_field_present(target, item,
                                                targets_by_id, codec)
                                                       for item in descendants(request_tree, 0x42000F))):
                                        continue
                                else:
                                    continue
                                event = cell_fields(cell)
                                event.update(status='passed', invocation_count=1, invocation_id=f'c-peer-{len(events)}')
                                digest_tree = response_tree if cell['action'] == 'decode' else request_tree
                                if cell['action'] == 'encode' and target['kind'] == 'operation':
                                    target_operation = codec.values.get(
                                        (codec.tag('Operation'), normalize(target['name'])))
                                    target_item = next(item for item in request_tree[2]
                                                       if item[0] == 0x42000F and
                                                       numbers(item, 0x42005C) == [target_operation])
                                    payload = next((child for child in target_item[2]
                                                    if child[0] == 0x420079 and child[1] == 1), None)
                                    digest_tree = payload or target_item
                                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = tree_digest(digest_tree)
                                if boundary and cell['action'] == 'send':
                                    event.update(request_delivery_state='not_sent', transport_invocation_count=0)
                                elif cell['action'] == 'send':
                                    event.update(request_delivery_state='response_received', transport_invocation_count=1)
                                if (cell['action'] == 'send' and len(recipe[0]) > 1 and
                                        not attribute_mutation_recipe):
                                    operations = [next(t['name'] for t in catalog['normative_targets'] if t['kind'] == 'enumeration_value' and t.get('wire_value') == f'{op:08X}' and any(p.endswith('ENUMERATION-OPERATION') for p in t.get('parent_element_ids', []))) for op in recipe[0]]
                                    event.update(batch_operation_sequence=operations, batch_item_count=len(operations),
                                        peer_observed_operation_sequence=operations,
                                        peer_observed_response_count=len(operations),
                                        peer_observed_result_status_codes=numbers(response_tree, 0x42007F),
                                        peer_observed_effects=state_evidence['effects'],
                                        peer_observed_state_before_sha256=state_evidence[
                                            'peer_observed_state_before_sha256'],
                                        peer_observed_state_after_sha256=state_evidence[
                                            'peer_observed_state_after_sha256'])
                                events.append(event)
                                keys.add(_key(cell))
    return events, keys, checks
