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
sys.path.insert(0, str(ROOT / 'interop/c-kmip-validator/tests'))
from controlled_kms_test import certificate_files, response, tlv
sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import _key

TYPES = ('', 'Structure', 'Integer', 'LongInteger', 'BigInteger', 'Enumeration',
         'Boolean', 'TextString', 'ByteString', 'DateTime', 'Interval', 'DateTimeExtended')

def normalize(name: str) -> str:
    return re.sub('[^a-z0-9]', '', name.lower())


class Codec:
    def __init__(self, catalog):
        self.tags = {normalize(t['name']): int(t['wire_value'], 16)
                     for t in catalog['normative_targets'] if t['kind'] == 'tag'}
        self.values = {}
        for target in catalog['normative_targets']:
            if target['kind'] not in ('enumeration_value', 'bitmask_value'):
                continue
            if not re.fullmatch('[0-9A-Fa-f]{8}', target['wire_value']):
                continue
            parent = target['parent_element_ids'][0]
            name = re.sub('^KMIPKIT-ELEM-(ENUMERATION|BITMASK)-', '', parent)
            tag = self.tags.get(normalize(name))
            self.values[(tag, normalize(target['name']))] = int(target['wire_value'], 16)

    def tag(self, value):
        return int(value, 16) if value.startswith('0x') else self.tags[normalize(value)]

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
    return [int.from_bytes(item[2], 'big') for item in descendants(node, tag)]


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
    'test-op-rng-retrieve': ((37,), 'rng_retrieve'),
    'test-op-rng-seed': ((38,), 'rng_seed'),
    'test-v20-reject-get-constraints': ((56,), 'get_constraints'),
    'test-op-create': ((1, 20), 'create'),
    'test-op-get': ((1, 10, 20), 'get'),
    'test-op-get-attributes': ((1, 11, 20), 'get_attributes'),
    'test-op-get-attribute-list': ((1, 12, 20), 'get_attribute_list'),
    'test-op-check': ((1, 9, 19, 20), 'check'),
    'test-op-activate': ((1, 18, 19, 20), 'activate'),
    'test-op-revoke': ((1, 19, 20), 'revoke'),
    'test-op-destroy': ((1, 20, 19, 20), 'destroy'),
    'test-op-get-constraints': ((1, 56, 19, 20), 'get_constraints'),
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


def assert_recipe(node, case_id, version):
    expected, _ = RECIPES[case_id]
    if tuple(numbers(node, 0x42005C)) != expected:
        raise ValueError('fixture_operation_order_mismatch')
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
            'test-attr-cryptographic-algorithm', 'test-attr-cryptographic-length', 'test-op-get-constraints')
        if len(numbers(node, 0x420094)) < len(expected) - 1 - int(implicit_target):
            raise ValueError('fixture_id_placeholder_missing')
        if len(list(descendants(node, 0x420093))) != len(expected):
            raise ValueError('fixture_batch_item_identity_missing')
    if case_id == 'test-attr-cryptographic-algorithm' and numbers(node, 0x42013B) != [0x420028]:
        raise ValueError('fixture_attribute_reference_mismatch')
    if case_id == 'test-attr-cryptographic-length' and numbers(node, 0x42013B) != [0x42002A]:
        raise ValueError('fixture_attribute_reference_mismatch')
    if RECIPES[case_id][1] == 'query' and numbers(node, 0x420074) != [1]:
        raise ValueError('fixture_query_function_mismatch')
    if case_id == 'test-norm-payload-client-query-request-21-002':
        groups = list(descendants(node, 0x420166))
        if (len(groups) != 1 or groups[0][1] != 1
                or [item[2] for item in groups[0][2] if item[0] == 0x420056] != [b'KMIPKit-Test-Group']):
            raise ValueError('fixture_query_object_groups_mismatch')
    if case_id == 'test-op-locate' and numbers(node, 0x42004F) != [3]:
        raise ValueError('fixture_locate_bound_missing')
    if case_id == 'test-op-log' and not list(descendants(node, 0x420141)):
        raise ValueError('fixture_log_message_missing')
    if case_id == 'test-op-hash':
        if numbers(node, 0x420038) != [6] or [item[2] for item in descendants(node, 0x4200C2)] != [b'abc']:
            raise ValueError('fixture_hash_input_mismatch')
    if case_id == 'test-op-rng-retrieve' and numbers(node, 0x4200C4) != [16]:
        raise ValueError('fixture_rng_length_mismatch')
    if case_id == 'test-op-rng-seed' and [item[2] for item in descendants(node, 0x4200C2)] != [bytes(range(16))]:
        raise ValueError('fixture_rng_seed_mismatch')



def verify_lifecycle(operations, response_tree):
    """Check the fixture state machine against every ordered response item."""
    if not operations or operations[0] != 1:
        return []
    state = None
    effects, expected_statuses, expected_reasons = [], [], []
    labels = {9: 'checked', 10: 'object_retrieved', 11: 'attributes_retrieved',
              12: 'attribute_list_retrieved', 56: 'constraints_retrieved'}
    for operation in operations:
        status, reason = 0, 0
        if operation == 1:
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


def read_request(stream, encoding, transport):
    data = bytearray()
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


def execute_network_cells(binary, catalog, manifest, *, protocol_versions=('2.0', '2.1'),
                          network_combinations=(('TTLV', 'raw_tls'), ('TTLV', 'https'),
                                                ('JSON', 'raw_tls'), ('JSON', 'https'),
                                                ('XML', 'raw_tls'), ('XML', 'https'))):
    codec = Codec(catalog)
    targets_by_id = {target['target_id']: target for target in catalog['normative_targets']}
    events, keys, checks = [], set(), []
    all_cells = list(iter_cells(catalog, manifest, languages=('c',)))
    artifact_sha256 = sha256(INSTALL / 'lib/libkmipkit_ffi.so')
    catalog_sha256 = sha256(ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json')
    case_variants = [(case, variant) for case in catalog['test_cases'] for variant in
        (('direct', 'standalone') if case['case_id'] in {'test-op-get-attributes', 'test-op-get-attribute-list',
         'test-attr-cryptographic-algorithm', 'test-attr-cryptographic-length'} else ('direct',))]
    cells_by_case = {}
    for cell in all_cells:
        cells_by_case.setdefault(cell['case_id'], []).append(cell)
    with tempfile.TemporaryDirectory(prefix='kmipkit-c-route-peer-') as raw:
        directory = Path(raw)
        ca, cert, key, client_cert, client_key = certificate_files(shutil.which('openssl') or 'openssl', directory)
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_3
        context.load_cert_chain(cert, key)
        context.load_verify_locations(ca)
        context.verify_mode = ssl.CERT_REQUIRED
        for version in protocol_versions:
            for encoding, transport in network_combinations:
                    for case, variant in case_variants:
                        case_id = case['case_id']
                        if case_id not in RECIPES or version not in case['execution']['protocol_versions']:
                            continue
                        if version == '2.0' and case_id in ('test-op-discover-versions', 'test-op-login', 'test-op-get-constraints', 'test-op-ping'):
                            continue
                        observation, errors = [], []
                        boundary = case_id == 'test-v20-reject-get-constraints'
                        with socket.socket() as server:
                            server.bind(('127.0.0.1', 0))
                            server.listen(1)
                            server.settimeout(5)
                            port = server.getsockname()[1]
                            def serve():
                                try:
                                    accepted, _ = server.accept()
                                    with context.wrap_socket(accepted, server_side=True) as stream:
                                        stream.settimeout(5)
                                        wire = read_request(stream, encoding, transport)
                                        request_tree = codec.decode(wire, encoding)
                                        assert_recipe(request_tree, case_id, version)
                                        selected_tag = {'test-attr-cryptographic-algorithm': 0x420028,
                                                        'test-attr-cryptographic-length': 0x42002A}.get(case_id, 0)
                                        object_type = 4 if case_id == 'test-obj-private-key' else 3 if case_id == 'test-obj-public-key' else 2
                                        response_wire = response(RECIPES[case_id][0], int(version[-1]), selected_tag, object_type)
                                        response_tree = parse(response_wire)
                                        verify_lifecycle(RECIPES[case_id][0], response_tree)
                                        encoded_response = codec.encode(response_wire, encoding)
                                        if transport == 'https':
                                            content_type = {'TTLV': 'application/octet-stream', 'JSON': 'application/json', 'XML': 'text/xml'}[encoding]
                                            encoded_response = (f'HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {len(encoded_response)}\r\nConnection: close\r\n\r\n'.encode() + encoded_response)
                                        stream.sendall(encoded_response)
                                        observation.append((request_tree, response_tree))
                                except TimeoutError:
                                    errors.append('fixture_peer_accept_timeout')
                                except ssl.SSLError:
                                    errors.append('fixture_tls_handshake_failed')
                                except ConnectionError:
                                    errors.append('fixture_connection_failed')
                                except Exception as error:
                                    code = str(error)
                                    errors.append(code if re.fullmatch('fixture_[a-z_]+', code) else 'controlled_peer_contract_failed')
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
                            peer_case = case
                            if (case['case_id'].startswith('test-norm-payload-client-query-request-')
                                    and case.get('execution', {}).get('mode') == 'local_model'):
                                # The public C Query constructor is opaque outside
                                # a client exchange; send the locally specified
                                # request through this controlled peer to inspect it.
                                peer_case = {**case, 'execution': {
                                    **case['execution'], 'mode': 'server_exchange'}}
                            catalog_path.write_text(json.dumps({**catalog, 'test_cases': [peer_case]}))
                            run = subprocess.run([str(binary.resolve()), 'run', '--connection', str(connection_path),
                                '--catalog', str(catalog_path), '--allow-mutations', '--output', str(result_path)],
                                cwd=ROOT, capture_output=True, timeout=10,
                                env={**os.environ, 'KMIPKIT_C_ROUTE_OBSERVATIONS': str(trace_path),
                                     'KMIPKIT_C_ATTRIBUTE_REQUEST_ROUTE': variant})
                            if thread is not None:
                                thread.join(6)
                            result = json.loads(result_path.read_text()) if result_path.exists() else {}
                            verify_consumer_provenance(result, artifact_sha256, catalog_sha256)
                            case_passed = bool(result.get('cases') and result['cases'][0]['status'] == 'passed')
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
                                observation.append((attempted, None))
                            code = errors[0] if errors else 'consumer_check_failed' if not case_passed else 'public_api_trace_missing' if not called_symbols else None
                            checks.append({'case_id': case_id, 'constructor_variant': variant, 'protocol_version': version, 'encoding': encoding,
                                           'transport': transport, 'status': 'passed' if observation and not code and run.returncode == 0 else 'failed',
                                           'consumer_case_status': (result.get('cases') or [{}])[0].get('status', 'missing'),
                                           'consumer_server_compatibility': (result.get('cases') or [{}])[0].get('server_compatibility', 'missing'),
                                           **({'failure_code': code or 'peer_observation_missing'} if code or not observation else {})})
                            if not observation or code or run.returncode != 0:
                                continue
                            request_tree, response_tree = observation[0]
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
                                            'kmipkit_get_attributes_request_to_ttlv_payload',
                                            'kmipkit_get_attribute_list_request_to_ttlv_payload'}:
                                        continue
                                target = targets_by_id[cell['target_id']]
                                if target['kind'] == 'operation':
                                    if target['name'] != operation_name:
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
                                if cell['action'] == 'encode':
                                    target_operation = 11 if case_id in {'test-op-get-attributes', 'test-attr-cryptographic-algorithm', 'test-attr-cryptographic-length'} else 12
                                    target_item = next(item for item in request_tree[2] if item[0] == 0x42000F and numbers(item, 0x42005C) == [target_operation])
                                    digest_tree = next(descendants(target_item, 0x420079))
                                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = tree_digest(digest_tree)
                                if boundary and cell['action'] == 'send':
                                    event.update(request_delivery_state='not_sent', transport_invocation_count=0)
                                elif cell['action'] == 'send':
                                    event.update(request_delivery_state='response_received', transport_invocation_count=1)
                                if cell['action'] == 'send' and len(RECIPES[case_id][0]) > 1:
                                    operations = [next(t['name'] for t in catalog['normative_targets'] if t['kind'] == 'enumeration_value' and t.get('wire_value') == f'{op:08X}' and any(p.endswith('ENUMERATION-OPERATION') for p in t.get('parent_element_ids', []))) for op in RECIPES[case_id][0]]
                                    event.update(batch_operation_sequence=operations, batch_item_count=len(operations),
                                        peer_observed_operation_sequence=operations,
                                        peer_observed_response_count=len(operations),
                                        peer_observed_result_status_codes=numbers(response_tree, 0x42007F),
                                        peer_observed_effects=verify_lifecycle(RECIPES[case_id][0], response_tree))
                                events.append(event)
                                keys.add(_key(cell))
    return events, keys, checks
