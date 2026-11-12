"""Execute KMIPKIT-0092 cells through the installed public Python package.

A recipe is never an API capability declaration. Missing recipes are emitted in
an independent pending-coverage ledger and deliberately fail the shared gate.
Only successful, observed calls can produce a positive execution event.
"""
from __future__ import annotations

from concurrent.futures import Future, ProcessPoolExecutor, ThreadPoolExecutor
from contextlib import ExitStack
import hashlib
import importlib
import inspect
import json
import multiprocessing
from pathlib import Path
import struct
import sys
import uuid
from typing import Any

from .contract import SHARED
from .driver import PythonKmipDriver, _TTLV_TYPE_KINDS, _catalog_model_data

# The shared generator is the sole authority for applicable cells.
sys.path.insert(0, str(SHARED / 'tools'))
from coverage_matrix import iter_cells  # noqa: E402


class RecipePending(Exception):
    """The validator cannot yet demonstrate the required case semantics."""


def digest(value: Any) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def _normalized_name(value: str) -> str:
    return ''.join(character for character in value.casefold() if character.isalnum())


def public_callable(symbol: str):
    parts = symbol.split('.')
    value = importlib.import_module('.'.join(parts[:2]))
    for part in parts[2:]:
        value = getattr(value, part)
    if isinstance(value, property):
        value = value.fget
    return value


class CallTrace:
    """Observe the installed Python function body, including synchronized wrappers."""

    def __init__(self, symbols: list[str]):
        self.codes = {}
        self.observed: set[str] = set()
        for symbol in symbols:
            function = inspect.unwrap(public_callable(symbol))
            code = getattr(function, '__code__', None)
            if code is not None:
                self.codes.setdefault(code, set()).add(symbol)
        self.previous = None

    def _profile(self, frame, event, arg):
        if event == 'call':
            self.observed.update(self.codes.get(frame.f_code, ()))

    def record_invocation(self, symbol: str) -> None:
        """Record an exact public data-member read, which has no Python body."""
        self.observed.add(symbol)

    def __enter__(self):
        self.previous = sys.getprofile()
        sys.setprofile(self._profile)
        return self

    def __exit__(self, *_):
        sys.setprofile(self.previous)


def item_tree(item) -> list:
    """Read ordered, numeric TTLV through public views; never return it in reports."""
    from kmipkit import ttlv

    def view_tree(view):
        with ttlv.ttlv_item_view_tag(view) as tag:
            raw = ttlv.tag_value(tag)
        kind = int(ttlv.ttlv_item_view_type(view))
        with ttlv.ttlv_item_view_value(view) as value:
            if kind == 1:
                with ttlv.ttlv_value_view_structure(value) as children:
                    result = []
                    for index in range(ttlv.ttlv_structure_view_item_count(children)):
                        with ttlv.ttlv_structure_view_item_at(children, index) as child:
                            result.append(view_tree(child))
            elif kind in (4, 7, 8):
                result = ttlv.ttlv_value_view_bytes(value).hex()
            else:
                getter = {2: 'integer', 3: 'long_integer', 5: 'enumeration', 6: 'boolean',
                          9: 'date_time', 10: 'interval', 11: 'date_time_extended'}[kind]
                result = getattr(ttlv, 'ttlv_value_view_' + getter)(value)
        return [raw, kind, result]

    with ttlv.ttlv_item_view_from_item(item) as view:
        return view_tree(view)


def encode_fixture(tree: list) -> bytes:
    """Independent TTLV fixture encoder; the wheel has no public generic encoder."""
    tag, kind, value = tree
    if kind == 1:
        encoded = b''.join(encode_fixture(child) for child in value)
    elif kind in (4, 7, 8):
        encoded = bytes.fromhex(value)
    elif kind == 6:
        encoded = struct.pack('>Q', int(value))
    else:
        encoded = struct.pack({2: '>i', 3: '>q', 5: '>I', 9: '>q', 10: '>I', 11: '>q'}[kind], value)
    return tag.to_bytes(3, 'big') + bytes([kind]) + len(encoded).to_bytes(4, 'big') + encoded + bytes(-len(encoded) % 8)


def _generic_samples(case: dict, target: dict, symbol: str) -> list[tuple[int, str, Any]]:
    """Resolve the finite numeric/type representatives from this exact catalog case."""
    requires = case['requires']
    kind = target.get('kind', requires.get('target_kind'))
    target_id = target['target_id']
    by_id, element_types, elements = _catalog_model_data()
    tag = 0x4200FF
    special_samples: list[tuple[int, str]] | None = None

    def find_tag(*names: str) -> dict | None:
        wanted = {_normalized_name(name) for name in names if name}
        return next((entry for entry in by_id.values() if entry.get('kind') == 'tag'
                     and _normalized_name(entry.get('name', '')) in wanted), None)

    if kind in ('enumeration_value', 'referenced_enum_value', 'bitmask_value'):
        raw_values = PythonKmipDriver._wire_representatives(requires['wire_value'])
        parent_ids = target.get('parent_element_ids', [])
        parent = next((elements.get(value, {}) for value in parent_ids
                       if elements.get(value, {}).get('kind') in ('enumeration', 'bitmask')), {})
        parent_name = parent.get('name', '')
        tag_entry = next((entry for entry in by_id.values() if entry.get('kind') == 'tag'
                          and _normalized_name(entry.get('name', '')) == _normalized_name(parent_name)), None)
        if tag_entry:
            tag = int(tag_entry['wire_value'], 16)
        elif _normalized_name(parent_name) == 'itemtype':
            # Item Type values are the TTLV header type codes, so there is no
            # KMIP tag carrying them as a protocol Enumeration. Keep the exact
            # numeric value while exercising the generic Enumeration value.
            tag = 0x4200FF
        elif kind == 'referenced_enum_value':
            tag = 0x42015A if 'FUNCTION' in target_id else 0x42015D
        else:
            raise RecipePending
        type_name = 'Integer' if kind == 'bitmask_value' else 'Enumeration'
        return [(tag, type_name, value if type_name == 'Enumeration' or value < 0x80000000
                 else value - 0x100000000) for value in raw_values]
    if kind == 'operation':
        operation_enum = next((entry for entry in elements.values()
                               if entry.get('kind') == 'enumeration_value'
                               and entry.get('name') == target.get('name')
                               and 'KMIPKIT-ELEM-ENUMERATION-OPERATION' in entry.get('parent_element_ids', [])), None)
        if operation_enum is None:
            raise RecipePending
        operation_tag = next((entry for entry in elements.values()
                              if entry.get('kind') == 'tag' and entry.get('name') == 'Operation'), None)
        if operation_tag is None:
            raise RecipePending
        return [(int(operation_tag['wire_value'], 16), 'Enumeration',
                 int(operation_enum['wire_value'], 16))]
    if kind == 'object_type':
        object_type_tag = next((entry for entry in elements.values()
                                if entry.get('kind') == 'tag' and _normalized_name(entry.get('name', '')) == 'objecttype'), None)
        if object_type_tag is None:
            raise RecipePending
        values = PythonKmipDriver._wire_representatives(target['wire_value'])
        return [(int(object_type_tag['wire_value'], 16), 'Enumeration', value) for value in values]
    if kind == 'tag' and requires.get('allocation') in ('assigned', 'extension'):
        return [(raw, 'Integer', 0) for raw in PythonKmipDriver._wire_representatives(requires['wire_value'])]
    if kind == 'tag_range' and requires.get('allocation') == 'extension':
        import re
        bounds = re.split(r'\s*[-–—]\s*', requires['wire_value'])
        return [(int(bound.upper().replace('X', fill), 16), 'Integer', 0)
                for bound, fill in zip(bounds, ('0', 'F'), strict=True)]
    if kind in ('tag', 'tag_range'):
        raise RecipePending
    if kind == 'data_type':
        type_names = {target['name']}
    elif kind == 'attribute':
        parent_tag = next((by_id.get(parent_id, {}) for parent_id in target.get('parent_element_ids', [])
                           if by_id.get(parent_id, {}).get('kind') == 'tag'), {})
        if not parent_tag and _normalized_name(target.get('name', '')) == 'vendorattribute':
            parent_tag = next((entry for entry in by_id.values()
                               if entry.get('kind') == 'tag' and entry.get('name') == 'Attribute'), {})
        if not parent_tag:
            raise RecipePending
        tag = int(parent_tag['wire_value'], 16)
        type_names = PythonKmipDriver._catalog_value_types(case, target, element_types)
        if not type_names and target.get('source_encoding'):
            structure_names = {_normalized_name(entry.get('name', '')) for entry in elements.values()
                               if entry.get('kind') in ('operation_structure', 'object_structure')}
            if _normalized_name(target['source_encoding']) in structure_names:
                type_names = {'Structure'}
    elif kind == 'operation_payload_value_variant' and target.get('kind') == 'operation_payload_value_variant' and requires['value_type'] != 'Structure':
        field_name = requires['field_name']
        tag_entry = find_tag(field_name)
        if tag_entry is None:
            raise RecipePending
        tag = int(tag_entry['wire_value'], 16)
        type_names = {requires['value_type']}
    elif kind == 'operation_payload_value_variant':
        field_name = requires['field_name']
        tag_entry = find_tag(field_name)
        if tag_entry is None:
            raise RecipePending
        tag = int(tag_entry['wire_value'], 16)
        type_names = {requires['value_type']}
    elif kind == 'operation_payload_field':
        field_name = requires.get('field_name', target.get('name', ''))
        tag_entry = find_tag(field_name, target.get('name', ''))
        if tag_entry is None and _normalized_name(field_name).startswith('anyobject'):
            object_name = requires.get('object_type', 'Opaque Object')
            tag_entry = find_tag(object_name)
        if tag_entry is None:
            raise RecipePending
        tag = int(tag_entry['wire_value'], 16)
        type_names = PythonKmipDriver._catalog_value_types(case, target, element_types)
        if _normalized_name(field_name) == 'uniqueidentifier':
            type_names = {'Text String'}
    elif kind in ('structure_member', 'message_field', 'attribute_structure',
                  'object_structure', 'operation_structure', 'credential'):
        target_name = target.get('name', '')
        normalized_target_name = _normalized_name(target_name)
        if normalized_target_name.startswith('credential'):
            tag_entry = find_tag('Credential')
        elif normalized_target_name == 'alternativefailoverendpoints':
            tag_entry = find_tag('Alternate Failover Endpoints')
        elif normalized_target_name == 'vendorspecific':
            tag_entry = find_tag('Vendor Extension')
        elif normalized_target_name == 'objecttypeobjecttypes':
            tag_entry = find_tag('Object Type')
        elif normalized_target_name.startswith('anyattribute'):
            tag_entry = find_tag('Activation Date')
        else:
            tag_entry = find_tag(target_name, requires.get('field_name', ''))
        if tag_entry is not None:
            tag = int(tag_entry['wire_value'], 16)
        elif kind in ('attribute_structure', 'credential'):
            # These table-level structures have no standalone assigned tag;
            # the generic route still exercises their exact Structure value.
            tag = 0x4200FF
        else:
            raise RecipePending
        type_names = PythonKmipDriver._catalog_value_types(case, target, element_types)
        if normalized_target_name == 'objecttypeobjecttypes':
            object_type_tag = find_tag('Object Type')
            object_types_tag = find_tag('Object Types')
            if object_type_tag is None or object_types_tag is None:
                raise RecipePending
            special_samples = [
                (int(object_type_tag['wire_value'], 16), 'Enumeration'),
                (int(object_types_tag['wire_value'], 16), 'Structure'),
            ]
        elif normalized_target_name.startswith('anyattribute'):
            by_type: dict[str, tuple[int, str]] = {}
            for attribute in elements.values():
                if attribute.get('kind') != 'attribute':
                    continue
                parent_tag = next((by_id.get(parent, {}) for parent in attribute.get('parent_element_ids', [])
                                   if by_id.get(parent, {}).get('kind') == 'tag'), None)
                if parent_tag is None:
                    continue
                attribute_types = PythonKmipDriver._catalog_value_types(
                    {'requires': {}}, attribute, element_types
                )
                for attribute_type in sorted(attribute_types):
                    by_type.setdefault(attribute_type, (int(parent_tag['wire_value'], 16), attribute_type))
            special_samples = list(by_type.values())
            if not special_samples:
                raise RecipePending
    else:
        type_names = PythonKmipDriver._catalog_value_types(case, target, element_types)
        # A type representative alone cannot prove a normative payload branch,
        # managed object shape, attribute persistence or protocol workflow.
        raise RecipePending
    if symbol.startswith('kmipkit.ttlv.ttlv_value_'):
        suffix = symbol.rsplit('ttlv_value_', 1)[1]
        type_names = {name for name in type_names if {'bytes': 'byte_string', 'text': 'text_string'}.get(_TTLV_TYPE_KINDS[name], _TTLV_TYPE_KINDS[name]) == suffix}
    samples = {
        'Structure': [None], 'Integer': [-0x80000000, -1, 0, 0x7FFFFFFF],
        'Long Integer': [-(1 << 63), -1, 0, (1 << 63) - 1],
        'Big Integer': [bytes.fromhex('0000000000000001'), bytes.fromhex('ffffffffffffffff')],
        'Enumeration': [0, 1, 0x7FFFFFFF, 0x80000000], 'Boolean': [False, True],
        'Text String': ['', 'KMIP-λ'], 'Byte String': [b'', bytes(range(9))],
        'Date Time': [-1, 0, 1], 'Interval': [0, 1, 0xFFFFFFFF], 'Date Time Extended': [-1, 0, 1],
    }
    if kind == 'operation_payload_field' and ''.join(
            character for character in requires.get('field_name', '').casefold() if character.isalnum()
    ) == 'uniqueidentifier':
        return [(tag, 'Text String', 'KMIPKit-interop')]
    if special_samples is not None:
        return [(sample_tag, sample_type, value)
                for sample_tag, sample_type in special_samples
                if sample_type in type_names for value in samples[sample_type]]
    return [(tag, name, value) for name in sorted(type_names) for value in samples[name]]


def _generic_cell(cell: dict, case: dict, target: dict) -> dict:
    from kmipkit import ttlv

    symbol = cell['public_api_symbol']
    if case['requires'].get('target_kind') in ('tag', 'tag_range') and case['requires'].get('allocation') in ('reserved', 'unused'):
        return _tag_rejection_cell(cell, case)
    if symbol not in _GENERIC_SYMBOLS:
        raise RecipePending
    samples = _generic_samples(case, target, symbol)
    if not samples:
        raise RecipePending
    observed = []
    for tag, name, value in samples:
        with ExitStack() as owned:
            if name == 'Structure':
                structure_children = []
                if symbol == 'kmipkit.ttlv.ttlv_structure_with_item':
                    child = owned.enter_context(PythonKmipDriver._item(0x4200FF, 'integer', 7))
                    children = owned.enter_context(ttlv.ttlv_structure_create())
                    limits = owned.enter_context(ttlv.codec_limits_defaults())
                    value = owned.enter_context(ttlv.ttlv_structure_with_item(children, child, limits))
                else:
                    if target.get('kind') == 'attribute' and _normalized_name(target.get('name', '')) == 'vendorattribute':
                        structure_children = [
                            owned.enter_context(PythonKmipDriver._item(0x42009D, 'text', 'KMIPKit.Interop')),
                            owned.enter_context(PythonKmipDriver._item(0x42000A, 'text', 'probe')),
                            owned.enter_context(PythonKmipDriver._item(0x42000B, 'text', 'interop')),
                        ]
                    elif target.get('kind') == 'attribute' and _normalized_name(target.get('name', '')) == 'randomnumbergenerator':
                        structure_children = [
                            owned.enter_context(PythonKmipDriver._item(0x4200DA, 'enumeration', 1)),
                        ]
                    value = owned.enter_context(PythonKmipDriver._structure(structure_children))
            item = owned.enter_context(PythonKmipDriver._item(tag, _TTLV_TYPE_KINDS[name], value))
            tree = item_tree(item)
            # Verify the public constructor's scalar value, not only its type.
            expected_value = value.encode().hex() if isinstance(value, str) else value.hex() if isinstance(value, bytes) else value
            if name != 'Structure' and tree[2] != expected_value:
                raise ValueError('constructor value assertion failed')
            wire = encode_fixture(tree)
            decoded = owned.enter_context(ttlv.ttlv_decode(wire))
            decoded_tree = item_tree(decoded)
            if decoded_tree != tree:
                raise ValueError('generic roundtrip assertion failed')
            observed.append(tree)
    tree_hash = digest(observed)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


_GENERIC_SYMBOLS = {
    'kmipkit.ttlv.' + name for name in (
        'ttlv_item_view_from_item', 'raw_tag_from_raw', 'raw_tag_try_checked',
        'ttlv_structure_create', 'ttlv_structure_with_item', 'ttlv_item_create', 'ttlv_decode',
        *(f'ttlv_value_{name}' for name in ('structure', 'integer', 'long_integer', 'big_integer',
                                          'enumeration', 'boolean', 'text_string', 'byte_string',
                                          'date_time', 'interval', 'date_time_extended')),
    )
}

_MATRIX_WORKERS = 16
_MATRIX_PROCESS_WORKERS = 8
_MATRIX_MAX_IN_FLIGHT = 64


def execute_cell(cell: dict, case: dict, target: dict, *, trace: CallTrace | None = None) -> dict:
    if cell['transport'] is None:
        if cell['public_api_symbol'] == 'kmipkit.client.BatchItemResult.supported_versions':
            return _positive_fixture_cell(cell, case, target, trace=trace)
        if cell['public_api_symbol'] in _TYPED_ATTRIBUTE_SYMBOLS:
            return _typed_attribute_cell(cell, case, target)
        if cell['public_api_symbol'].startswith('kmipkit.cryptographic_parameters.'):
            return _crypto_parameters_cell(cell, case, target)
        return _generic_cell(cell, case, target)
    if case['case_id'] == 'test-op-discover-versions':
        return _positive_fixture_cell(cell, case, target, trace=trace)
    return _network_cell(cell, case, target, trace=trace)


def run_matrix(catalog: dict, manifest: dict, artifact: dict,
               *, catalog_sha256: str | None = None, route_manifest_sha256: str | None = None,
               process_workers: int = 0) -> tuple[dict, list[dict]]:
    if artifact['library_commit'] != manifest['library_commit']:
        raise ValueError('installed artifact and route manifest commits differ')
    cases = {case['case_id']: case for case in catalog['test_cases']}
    targets = {target['target_id']: target for target in catalog['normative_targets']}
    combinations = sorted({(item['encoding'], item['transport']) for route in manifest['routes']
                           if route['language'] == 'python' and route['mode'] == 'network'
                           for item in route['combinations']})
    if not combinations:
        combinations = [('TTLV', 'raw_tls')]
    report = {
        'schema_version': '1.0', 'run_id': 'python-matrix-' + str(uuid.uuid4()), 'language': 'python',
        'server': {'id': 'controlled-fixture', 'product': 'KMIPKit Python controlled peer', 'version': '1',
                   'configuration_fingerprint': digest({'fixture': 'python-controlled-v1'})},
        'library_commit': artifact['library_commit'], 'artifact_sha256': artifact['artifact_sha256'],
        'catalog_sha256': catalog_sha256 or digest(catalog),
        'route_manifest_sha256': route_manifest_sha256 or digest(manifest),
        'protocol_versions': list(catalog['target_versions']),
        'network_combinations': [{'encoding': encoding, 'transport': transport} for encoding, transport in combinations],
        'events': [],
    }
    gaps = []
    outcomes = []
    in_flight = []
    executor_context = (ProcessPoolExecutor(
        max_workers=process_workers, mp_context=multiprocessing.get_context('spawn'))
        if process_workers else ThreadPoolExecutor(max_workers=_MATRIX_WORKERS,
                                                    thread_name_prefix='python-route-matrix'))
    with executor_context as executor:
        for cell in iter_cells(catalog, manifest, languages=('python',)):
            future = executor.submit(_execute_matrix_cell, cell, cases[cell['case_id']],
                                     targets[cell['target_id']])
            outcomes.append(future)
            in_flight.append(future)
            if len(in_flight) >= _MATRIX_MAX_IN_FLIGHT:
                in_flight.pop(0).result()
    for outcome in outcomes:
        if isinstance(outcome, Future):
            outcome = outcome.result()
        kind, value = outcome
        if kind == 'event':
            report['events'].append(value)
        else:
            gaps.append(value)
    return report, gaps


def _execute_matrix_cell(cell: dict, case: dict, target: dict) -> tuple[str, dict]:
    """Execute one matrix cell and return its sanitized event or pending record."""
    event_fields = ('case_id', 'route_id', 'target_id', 'workflow_step_index', 'workflow_operation',
                    'workflow_item_index', 'public_api_symbol', 'execution_route_id', 'execution_api_symbol',
                    'action', 'protocol_version', 'encoding', 'transport')
    identity = {field: cell[field] for field in event_fields if field in cell}
    trace = None
    symbols = []
    try:
        symbols = [cell['public_api_symbol']]
        if cell.get('execution_api_symbol'):
            symbols.append(cell['execution_api_symbol'])
        with CallTrace(symbols) as trace:
            observed = execute_cell(cell, case, target, trace=trace)
        if not set(symbols) <= trace.observed:
            raise RecipePending
        return 'event', {**identity, 'invocation_id': str(uuid.uuid4()),
                         'invocation_count': 1, 'status': 'passed', **observed}
    except RecipePending:
        return 'gap', {**identity, 'failure_code': 'recipe_pending'}
    except Exception:
        # No exception names, sources, native text or raw fixture values leave here.
        if trace is not None and set(symbols) <= trace.observed:
            return 'event', {**identity, 'invocation_id': str(uuid.uuid4()),
                             'invocation_count': 1, 'status': 'failed',
                             'failure_code': 'route_execution_failed'}
        return 'gap', {**identity, 'failure_code': 'recipe_execution_failed'}


def _ping_tree(wire: bytes, encoding: str) -> list:
    """Normalize the complete deterministic Ping envelope, retaining child order."""
    from kmipkit import protocol_values
    from .controlled_peer import parse_children
    import xml.etree.ElementTree as ET
    from datetime import datetime

    tags = {'RequestMessage': 0x420078, 'RequestHeader': 0x420077,
            'ResponseMessage': 0x42007B, 'ResponseHeader': 0x42007A,
            'ProtocolVersion': 0x420069, 'ProtocolVersionMajor': 0x42006A,
            'ProtocolVersionMinor': 0x42006B, 'AsynchronousIndicator': 0x420007,
            'BatchCount': 0x42000D, 'BatchItem': 0x42000F, 'Operation': 0x42005C,
            'RequestPayload': 0x420079, 'ResponsePayload': 0x42007C,
            'ResultStatus': 0x42007F, 'ResultReason': 0x42007E, 'TimeStamp': 0x420092,
            'AttestationCapableIndicator': 0x4200D3}
    types = {'Structure': 1, 'Integer': 2, 'Enumeration': 5, 'Boolean': 6, 'DateTime': 9}

    def scalar(tag, kind, value):
        if tag == 0x420092:
            return '<normalized-time-stamp>'
        if kind == 5 and isinstance(value, str):
            enum = protocol_values.Operation if tag == 0x42005C else protocol_values.ResultStatus
            matching = [raw for name, raw in vars(enum).items() if isinstance(raw, int)
                        and ''.join((enum.name(raw) or '').split()) == value]
            if len(matching) != 1:
                raise ValueError('fixture enum mismatch')
            return matching[0]
        if kind == 6:
            return bool(value) if isinstance(value, (bool, int)) else str(value).lower() == 'true'
        return int(value)

    def binary_nodes(data):
        nodes = []
        for tag, kind, value in parse_children(data):
            result = binary_nodes(value) if kind == 1 else scalar(
                tag, kind, int.from_bytes(value, 'big', signed=kind in (2, 9)))
            nodes.append([tag, kind, result])
        return nodes

    def json_node(node):
        tag = tags[node['tag']]
        kind = types[node.get('type', 'Structure')]
        value = node['value']
        return [tag, kind, [json_node(child) for child in value] if kind == 1 else scalar(tag, kind, value)]

    def xml_node(node):
        tag = tags[node.tag]
        kind = types[node.attrib.get('type', 'Structure')]
        return [tag, kind, [xml_node(child) for child in node] if kind == 1
                else scalar(tag, kind, node.attrib['value'])]

    if encoding == 'TTLV':
        roots = binary_nodes(wire)
        if len(roots) != 1:
            raise ValueError('fixture root count mismatch')
        return roots[0]
    if encoding == 'JSON':
        return json_node(json.loads(wire))
    return xml_node(ET.fromstring(wire))


def _network_tree(wire: bytes, encoding: str) -> list:
    """Normalize a complete peer message to numeric tags and TTLV type codes."""
    from datetime import datetime
    import xml.etree.ElementTree as ET
    from .controlled_peer import parse_children
    from kmipkit import protocol_values

    by_id, _, elements = _catalog_model_data()
    tag_by_name = {_normalized_name(item['name']): int(item['wire_value'], 16)
                   for item in by_id.values() if item.get('kind') == 'tag'}
    type_codes = {'Structure': 1, 'Integer': 2, 'LongInteger': 3, 'BigInteger': 4,
                  'Enumeration': 5, 'Boolean': 6, 'TextString': 7, 'ByteString': 8,
                  'DateTime': 9, 'Interval': 10, 'DateTimeExtended': 11}

    def enum_value(tag: int, value: Any) -> Any:
        if isinstance(value, int):
            return value
        enum_type = {0x42005C: protocol_values.Operation,
                     0x42007F: protocol_values.ResultStatus,
                     0x42007E: protocol_values.ResultReason}.get(tag)
        if enum_type is None:
            return value
        normalized = _normalized_name(str(value))
        matches = [raw for name, raw in vars(enum_type).items() if isinstance(raw, int)
                   and _normalized_name(enum_type.name(raw) or name) == normalized]
        if len(matches) != 1:
            raise ValueError('controlled peer enum does not match the public enum table')
        return matches[0]

    def bitmask_value(tag: int, value: str) -> int:
        tag_entry = next((item for item in by_id.values()
                          if item.get('kind') == 'tag' and int(item['wire_value'], 16) == tag), None)
        if tag_entry is None:
            raise ValueError('bitmask tag is absent from the source catalog')
        bitmask = next((item for item in elements.values()
                        if item.get('kind') == 'bitmask'
                        and _normalized_name(item.get('name', '')) == _normalized_name(tag_entry['name'])), None)
        if bitmask is None:
            raise ValueError('bitmask definition is absent from the source catalog')
        assigned = [item for item in elements.values()
                    if item.get('kind') == 'bitmask_value'
                    and bitmask['element_id'] in item.get('parent_element_ids', [])]
        raw_tokens = value.split('|') if '|' in value else [value]
        bits = 0
        for raw_token in raw_tokens:
            remaining = _normalized_name(raw_token)
            candidates = sorted((item for item in assigned if item.get('allocation') == 'assigned'),
                                key=lambda item: len(_normalized_name(item.get('name', ''))), reverse=True)
            while remaining:
                match = next((item for item in candidates
                              if remaining.startswith(_normalized_name(item.get('name', '')))), None)
                if match is None:
                    raise ValueError('bitmask value is absent from the source catalog')
                token = _normalized_name(match['name'])
                bits |= int(match['wire_value'], 16)
                remaining = remaining[len(token):]
        return bits

    def binary_node(raw_tag: int, kind: int, raw: bytes) -> list:
        if kind == 1:
            value = [binary_node(*entry) for entry in parse_children(raw)]
        elif kind in (4, 7, 8):
            value = raw.hex()
        elif kind == 6:
            value = bool(int.from_bytes(raw, 'big'))
        elif kind in (2, 3, 5, 9, 10, 11):
            value = int.from_bytes(raw, 'big', signed=kind in (2, 3, 9, 11))
        else:
            raise ValueError('controlled peer emitted an unknown TTLV type')
        return [raw_tag, kind, value]

    def document_node(name: str, kind_name: str, value: Any) -> list:
        raw_tag = tag_by_name.get(_normalized_name(name))
        kind = type_codes.get(kind_name)
        if raw_tag is None or kind is None:
            raise ValueError('controlled peer document has an unknown tag or type')
        if kind == 1:
            children = value if isinstance(value, list) else []
            normalized = [child if isinstance(child, list)
                          else document_node(child['tag'], child.get('type', 'Structure'), child['value'])
                          for child in children]
            return [raw_tag, kind, normalized]
        if kind == 5:
            value = enum_value(raw_tag, value)
        elif kind in (7, 8):
            value = str(value).encode('utf-8').hex()
        elif kind in (2, 3, 9, 10, 11):
            if kind in (9, 11) and isinstance(value, str) and not value.lstrip('-').isdigit():
                value = int(datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp())
            elif kind == 2 and isinstance(value, str) and not value.lstrip('-').isdigit():
                value = bitmask_value(raw_tag, value)
            else:
                value = int(value)
        elif kind == 6:
            value = value if isinstance(value, bool) else str(value).casefold() == 'true'
        return [raw_tag, kind, value]

    if encoding == 'TTLV':
        if len(wire) < 8:
            raise ValueError('controlled peer returned a truncated TTLV item')
        roots = parse_children(wire[8:])
        return [binary_node(*root) for root in roots]
    if encoding == 'JSON':
        root = json.loads(wire)
        return [document_node(root['tag'], root.get('type', 'Structure'), root['value'])]
    root = ET.fromstring(wire)

    def xml_value(node):
        kind = node.attrib.get('type', 'Structure')
        if kind == 'Structure':
            value = [xml_value(child) for child in node]
        else:
            value = node.attrib.get('value', '')
        return document_node(node.tag, kind, value)

    return [xml_value(root)]


def _find_tree_value(roots: list, tag: int) -> Any | None:
    for current_tag, _, value in roots:
        if current_tag == tag:
            return value
        if isinstance(value, list):
            found = _find_tree_value(value, tag)
            if found is not None:
                return found
    return None


def _network_cell(cell: dict, case: dict, target: dict, *, trace: CallTrace | None) -> dict:
    from .controlled_peer import ControlledTlsPeer, PKI
    from kmipkit import extensions, protocol_values, ttlv

    if case.get('requires', {}).get('expected_status') == 'NotSent':
        return _not_sent_cell(cell, case)

    positive_ping = (case['case_id'] == 'test-op-ping'
                     and cell['public_api_symbol'] == 'kmipkit.extensions.client_batch_item_ping')
    operation = cell.get('workflow_operation', case['operation'])
    final_async_failure = (
        case['case_id'] == 'test-async-poll-final-operation-failure' and operation == 'Poll'
    )
    if positive_ping:
        peer_status = protocol_values.ResultStatus.SUCCESS
        peer_reason = None
        expected_driver_status = 'passed'
    elif final_async_failure:
        peer_status = protocol_values.ResultStatus.OPERATION_FAILED
        peer_reason = protocol_values.ResultReason.INVALID_OBJECT_TYPE
        expected_driver_status = 'failed'
    else:
        peer_status = protocol_values.ResultStatus.OPERATION_FAILED
        peer_reason = protocol_values.ResultReason.OPERATION_NOT_SUPPORTED
        expected_driver_status = 'server_unsupported'
    peer = ControlledTlsPeer(status=peer_status, reason=peer_reason,
                             encoding=cell['encoding'], transport=cell['transport'],
                             protocol_version=cell['protocol_version'])
    connection = {
        'server_id': 'controlled-fixture', 'protocol_version': cell['protocol_version'],
        'transport': cell['transport'], 'encoding': cell['encoding'], 'api_level': 'typed-protocol',
        'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': peer.port},
                      'https': {'url': f'https://127.0.0.1:{peer.port}/kmip'}},
        'tls': {'ca_cert_path': str(PKI / 'server-cert.pem'),
                'client_cert_path': str(PKI / 'client-cert.pem'),
                'client_key_path': str(PKI / 'client-key.pem'), 'server_name_override': 'localhost',
                'connect_timeout_seconds': 5, 'read_timeout_seconds': 5},
    }
    driver = None
    item = None
    identifier = None
    try:
        driver = PythonKmipDriver(connection)
        symbol = cell['public_api_symbol']
        if symbol.startswith('kmipkit.client.Client.execute_'):
            correlation = None if operation == 'Query Asynchronous Requests' else b'kmipkit-route-correlation'
            result = driver._async_exchange(operation, correlation)
        else:
            identifier = extensions.UniqueIdentifier.text_string('KMIPKit-python-route')
            name = 'KMIPKit-python-route'
            if symbol.endswith('.to_batch_item') and symbol.startswith('kmipkit.attribute_operations.'):
                item = driver._build_high_level_item(case, identifier, name=name)
            elif operation == 'Logout':
                ticket_type = driver._item(0x42014A, 'enumeration', 1)
                ticket_value = driver._item(0x42014B, 'bytes', b'fixture-ticket')
                ticket = driver._structure([ticket_type, ticket_value])
                try:
                    item = extensions.client_batch_item_logout(ticket)
                finally:
                    ticket.close()
                    ticket_type.close()
                    ticket_value.close()
            else:
                item = driver._build_item(case, identifier, name=name)
            result = driver._exchange(item)
        if (result['status'] != expected_driver_status or result['result_status_code'] != peer_status
                or result.get('result_reason_code') != peer_reason):
            raise ValueError('controlled peer response status did not match the expected route outcome')
    finally:
        if item is not None:
            item.close()
        if driver is not None:
            driver.close()
        peer.close()
    if len(peer.requests) != 1 or len(peer.responses) != 1:
        raise ValueError('fixture exchange count assertion failed')
    if (normalized_operation_name := ''.join(character for character in operation.casefold() if character.isalnum())):
        if ''.join(character for character in str(peer.request_observation.get('operation', '')).casefold()
                   if character.isalnum()) != normalized_operation_name:
            raise ValueError('peer did not receive the exact requested operation')
    request = _network_tree(peer.requests[0], cell['encoding'])
    response = _network_tree(peer.responses[0], cell['encoding'])
    operation_codes = [raw for name, raw in vars(protocol_values.Operation).items()
                       if isinstance(raw, int) and ''.join(
                           character for character in (protocol_values.Operation.name(raw) or name).casefold()
                           if character.isalnum()) == normalized_operation_name]
    if len(operation_codes) != 1:
        raise ValueError('catalog operation is absent from the public enum table')
    expected_operation = operation_codes[0]
    request_operation = _find_tree_value(request, 0x42005C)
    response_operation = _find_tree_value(response, 0x42005C)
    if (request_operation != expected_operation
            or response_operation != request_operation
            or _find_tree_value(response, 0x42007F) != peer_status
            or _find_tree_value(response, 0x42007E) != peer_reason
            or (positive_ping and _find_tree_value(response, 0x42007C) != [])
            or (not positive_ping and _find_tree_value(response, 0x42007C) is not None)):
        raise ValueError('controlled peer request or unsupported response shape mismatch')
    if trace is None:
        raise ValueError('network cell has no public-call trace')
    return {'request_tree_sha256': digest(request), 'response_tree_sha256': digest(response),
            'wire_bytes_sha256': hashlib.sha256(
                peer.responses[0] if cell.get('action') == 'decode' else peer.requests[0]).hexdigest(),
            'request_delivery_state': 'response_received', 'transport_invocation_count': 1,
            'peer_observed_response_count': 1,
            'peer_observed_result_status_codes': [peer_status],
            **({'peer_observed_result_reason_codes': [peer_reason]} if peer_reason is not None else {})}


def _positive_fixture_cell(cell: dict, case: dict, target: dict, *, trace: CallTrace | None) -> dict:
    """Execute Discover Versions against a positive peer and check typed output."""
    from .controlled_peer import ControlledTlsPeer, PKI
    from kmipkit import extensions, protocol_values

    if (case['case_id'] != 'test-op-discover-versions'
            or cell['protocol_version'] not in ('2.0', '2.1')
            or cell['public_api_symbol'] not in (
                'kmipkit.extensions.client_batch_item_discover_versions',
                'kmipkit.client.BatchItemResult.supported_versions',
            )):
        raise RecipePending
    encoding = cell['encoding'] if cell['transport'] is not None else 'TTLV'
    transport = cell['transport'] if cell['transport'] is not None else 'raw_tls'
    major, minor = (int(part) for part in cell['protocol_version'].split('.'))
    versions = ((2, 1),)
    peer = ControlledTlsPeer(encoding=encoding, transport=transport,
                             protocol_version=cell['protocol_version'])
    connection = {
        'server_id': 'controlled-fixture', 'protocol_version': cell['protocol_version'],
        'transport': transport, 'encoding': encoding, 'api_level': 'typed-protocol',
        'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': peer.port},
                      'https': {'url': f'https://127.0.0.1:{peer.port}/kmip'}},
        'tls': {'ca_cert_path': str(PKI / 'server-cert.pem'),
                'client_cert_path': str(PKI / 'client-cert.pem'),
                'client_key_path': str(PKI / 'client-key.pem'), 'server_name_override': 'localhost',
                'connect_timeout_seconds': 5, 'read_timeout_seconds': 5},
    }
    driver = None
    response = batch = limits = None
    try:
        driver = PythonKmipDriver(connection)
        batch = driver._client_module.Batch.create()
        limits = driver._client_module.ttlv.codec_limits_defaults()
        with extensions.client_batch_item_discover_versions() as item:
            batch.add_item(item)
        response = driver._client.execute(batch, limits)
        item_result = response.item_at(0)
        if (int(item_result.result_status) != protocol_values.ResultStatus.SUCCESS
                or item_result.result_reason is not None):
            raise ValueError('Discover Versions fixture did not return Success')
        supported_versions = getattr(item_result, 'supported_versions')
        actual_versions = tuple((int(version.major), int(version.minor))
                                for version in (supported_versions or ()))
        if supported_versions is None or actual_versions != versions:
            raise ValueError('Discover Versions response versions did not preserve order')
        if cell['public_api_symbol'] == 'kmipkit.client.BatchItemResult.supported_versions':
            if trace is None:
                raise ValueError('data-member route has no invocation trace')
            trace.record_invocation(cell['public_api_symbol'])
    finally:
        if response is not None:
            response.close()
        if limits is not None:
            limits.close()
        if batch is not None:
            batch.close()
        if driver is not None:
            driver.close()
        peer.close()
    if len(peer.requests) != 1 or len(peer.responses) != 1:
        raise ValueError('Discover Versions fixture exchange count assertion failed')
    request = _ping_tree(peer.requests[0], encoding)
    actual_response = _ping_tree(peer.responses[0], encoding)
    operation = protocol_values.Operation.DISCOVER_VERSIONS
    expected_request = [0x420078, 1, [
        [0x420077, 1, [[0x420069, 1, [[0x42006A, 2, major], [0x42006B, 2, minor]]],
                       [0x4200D3, 6, True], [0x42000D, 2, 1]]],
        [0x42000F, 1, [[0x42005C, 5, operation], [0x420079, 1, [
            [0x420069, 1, [[0x42006A, 2, 2], [0x42006B, 2, 1]]]]]]]]]
    expected_response = [0x42007B, 1, [
        [0x42007A, 1, [[0x420069, 1, [[0x42006A, 2, major], [0x42006B, 2, minor]]],
                       [0x420092, 9, '<normalized-time-stamp>'], [0x42000D, 2, 1]]],
        [0x42000F, 1, [[0x42005C, 5, operation], [0x42007F, 5, 0],
                       [0x42007C, 1, [[0x420069, 1, [[0x42006A, 2, version_major],
                                                           [0x42006B, 2, version_minor]]]
                                      for version_major, version_minor in versions]]]]]]
    if request != expected_request or actual_response != expected_response:
        raise ValueError('Discover Versions request or response shape assertion failed')
    return {'request_tree_sha256': digest(request), 'response_tree_sha256': digest(actual_response),
            'wire_bytes_deterministic': True,
            'wire_bytes_sha256': hashlib.sha256(
                peer.responses[0] if cell.get('action') == 'decode' else peer.requests[0]).hexdigest()}


def verify_installed_artifact(artifact: dict) -> None:
    """Compare every installed package file with the verified exact wheel."""
    from importlib.metadata import distribution
    import zipfile

    package = distribution('kmipkit')
    module = importlib.util.find_spec('kmipkit')
    if (package.version != '1.0.0' or module is None or module.origin is None
            or not Path(module.origin).resolve().is_relative_to(Path(sys.prefix).resolve())):
        raise ValueError('pinned wheel is not installed in the isolated environment')
    installed_origin = json.loads(package.read_text('direct_url.json') or '{}')
    if installed_origin.get('archive_info', {}).get('hashes', {}).get('sha256') != artifact['artifact_sha256']:
        raise ValueError('installed wheel identity differs from recorded archive')
    checked = 0
    with zipfile.ZipFile(artifact['artifact_file']) as wheel:
        for name in wheel.namelist():
            if not name.startswith('kmipkit/') or name.endswith('/'):
                continue
            relative = Path(name)
            if '..' in relative.parts:
                raise ValueError('wheel contains an invalid package path')
            installed = package.locate_file(name).resolve(strict=True)
            if (not installed.is_relative_to(Path(sys.prefix).resolve())
                    or hashlib.sha256(installed.read_bytes()).digest() != hashlib.sha256(wheel.read(name)).digest()):
                raise ValueError('installed package differs from recorded wheel')
            checked += 1
    if not checked:
        raise ValueError('wheel has no installed package files')


def run_command(output_dir: Path) -> int:
    """Write sanitized evidence before failing the incomplete-matrix gate."""
    from .contract import validate_artifact, _validate
    from .reporter import resolve_report_directory
    from validate_route_evidence import validate_collection

    output = resolve_report_directory(output_dir)
    artifact = validate_artifact()
    verify_installed_artifact(artifact)
    catalog_bytes = (SHARED / 'catalog/kmipkit-1.0-test-catalog.json').read_bytes()
    manifest_bytes = (SHARED / 'coverage/route-capabilities.json').read_bytes()
    catalog = json.loads(catalog_bytes)
    manifest = json.loads(manifest_bytes)
    catalog_sha = hashlib.sha256(catalog_bytes).hexdigest()
    manifest_sha = hashlib.sha256(manifest_bytes).hexdigest()
    report, gaps = run_matrix(catalog, manifest, artifact,
                              catalog_sha256=catalog_sha, route_manifest_sha256=manifest_sha,
                              process_workers=_MATRIX_PROCESS_WORKERS)
    _validate(report, SHARED / 'schemas/route-execution-evidence.schema.json',
              'route evidence violates the shared schema')
    output.mkdir(parents=True, exist_ok=True)
    (output / 'route-execution-evidence.json').write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    pending = {'schema_version': '1.0', 'language': 'python', 'library_commit': artifact['library_commit'],
               'artifact_sha256': artifact['artifact_sha256'], 'catalog_sha256': catalog_sha,
               'route_manifest_sha256': manifest_sha, 'pending_cells': len(gaps), 'cells': gaps}
    (output / 'pending-coverage.json').write_text(json.dumps(pending, indent=2, sort_keys=True) + '\n')
    passed = False
    try:
        summary = validate_collection([report], catalog, manifest, catalog_sha256=catalog_sha,
                                      route_manifest_sha256=manifest_sha,
                                      expected_artifact_sha256=artifact['artifact_sha256'])
        passed = not gaps and summary['all_invocations_passed'] and summary['complete_matrix']
    except ValueError:
        # The shared validator contains only axis identities in its diagnostic,
        # but this CLI deliberately emits fixed text and numeric counts only.
        pass
    closure = {'language': 'python', 'required_cells': len(report['events']) + len(gaps),
               'verified_cells': sum(event['status'] == 'passed' for event in report['events']),
               'failed_cells': sum(event['status'] == 'failed' for event in report['events']),
               'pending_cells': len(gaps), 'complete': passed}
    (output / 'closure.json').write_text(json.dumps(closure, indent=2, sort_keys=True) + '\n')
    print(json.dumps(closure, sort_keys=True))
    return 0 if passed else 1


def _not_sent_cell(cell: dict, case: dict) -> dict:
    """Observe native version rejection with a listening, never-used endpoint."""
    from kmipkit import async_operations, errors, extensions, ttlv
    from .controlled_peer import PKI
    import socket

    operation = case['operation']
    if cell['protocol_version'] != '2.0' or operation not in (
            'Ping', 'Get Constraints', 'Set Constraints', 'Set Defaults',
            'Process', 'Query Asynchronous Requests'):
        raise RecipePending
    with socket.socket() as listener, ExitStack() as owned:
        listener.bind(('127.0.0.1', 0))
        listener.listen()
        port = listener.getsockname()[1]
        connection = {
            'server_id': 'controlled-fixture', 'protocol_version': '2.0',
            'transport': cell['transport'], 'encoding': cell['encoding'], 'api_level': 'typed-protocol',
            'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': port},
                          'https': {'url': f'https://127.0.0.1:{port}/kmip'}},
            'tls': {'ca_cert_path': str(PKI / 'server-cert.pem'),
                    'client_cert_path': str(PKI / 'client-cert.pem'),
                    'client_key_path': str(PKI / 'client-key.pem'), 'server_name_override': 'localhost',
                    'connect_timeout_seconds': 1, 'read_timeout_seconds': 1},
        }
        driver = PythonKmipDriver(connection)
        owned.callback(driver.close)
        payload = owned.enter_context(ttlv.ttlv_structure_create())
        correlation = b'kmipkit-version-boundary'
        if operation == 'Process':
            request = owned.enter_context(async_operations.ProcessRequest.create(correlation))
            payload = owned.enter_context(request.to_ttlv_payload())
        elif operation == 'Query Asynchronous Requests':
            request = owned.enter_context(async_operations.QueryAsyncRequestsRequest.create())
            payload = owned.enter_context(request.to_ttlv_payload())
        # The public typed-batch API has no request view. Normalize the exact
        # explicit payload inputs; do not claim wire bytes for an unsent request.
        payload_item = owned.enter_context(PythonKmipDriver._item(0x420079, 'structure', payload))
        inputs = {'protocol_version': [2, 0], 'operation': operation, 'payload': item_tree(payload_item)}
        if operation == 'Set Constraints':
            inputs['parameters'] = {'constraints': []}
        rejected = False
        try:
            if operation in ('Process', 'Query Asynchronous Requests'):
                limits = owned.enter_context(ttlv.codec_limits_defaults())
                outcome = (driver._client.execute_process(correlation, limits) if operation == 'Process'
                           else driver._client.execute_query_async_requests(limits))
                outcome.close()
            else:
                functions = {
                    'Ping': extensions.client_batch_item_ping,
                    'Get Constraints': extensions.client_batch_item_get_constraints,
                    'Set Constraints': extensions.client_batch_item_set_constraints,
                    'Set Defaults': extensions.client_batch_item_set_defaults,
                }
                constraints = owned.enter_context(ttlv.ttlv_structure_create()) if operation == 'Set Constraints' else None
                item = owned.enter_context(functions[operation](constraints) if operation == 'Set Constraints'
                                           else functions[operation]())
                driver._exchange(item)
        except errors.ClientExecutionError as error:
            rejected = error.delivery is errors.DeliveryState.NOT_SENT
        listener.settimeout(0.001)
        attempts = 0
        try:
            accepted, _ = listener.accept()
        except TimeoutError:
            pass
        else:
            accepted.close()
            attempts = 1
        if not rejected or attempts:
            raise ValueError('version-boundary request was not rejected before transport')
        return {'request_tree_sha256': digest(inputs), 'request_delivery_state': 'not_sent',
                'transport_invocation_count': 0}


_TYPED_ATTRIBUTE_SYMBOLS = {
    'kmipkit.typed_attributes.TypedAttribute.create',
    'kmipkit.typed_attributes.TypedAttribute.from_item',
    'kmipkit.attribute_operations.AttributeSet.try_new',
    'kmipkit.attribute_operations.AttributeSet.try_push',
    'kmipkit.attribute_operations.AttributeSet.try_push_typed',
}


def _typed_attribute_cell(cell: dict, case: dict, target: dict) -> dict:
    """Check exact direct-attribute identity, value copies and collection reads."""
    from kmipkit import ttlv
    from kmipkit.attribute_operations import AttributeSet
    from kmipkit.typed_attributes import StandardAttribute, TypedAttribute

    samples = _generic_samples(case, target, 'kmipkit.ttlv.ttlv_item_create')
    attributes = {attribute.tag: attribute for attribute in StandardAttribute}
    if not samples or any(tag not in attributes for tag, _, _ in samples):
        raise RecipePending
    observed = []
    for tag, type_name, value in samples:
        attribute = attributes[tag]
        if case['requires'].get('target_kind') == 'tag':
            # Tag allocation cases have no prescribed value. Use the installed
            # attribute's exact catalogued scalar type instead of inventing one.
            scalar = {'Integer': ('Integer', 0), 'Enumeration': ('Enumeration', 1),
                      'DateTime': ('Date Time', 0), 'Interval': ('Interval', 0),
                      'Boolean': ('Boolean', True), 'TextString': ('Text String', 'fixture'),
                      'ByteString': ('Byte String', b''), 'UniqueIdentifier': ('Text String', 'fixture')}
            if attribute.value_kind not in scalar:
                raise RecipePending
            type_name, value = scalar[attribute.value_kind]
        with ExitStack() as owned:
            original = owned.enter_context(PythonKmipDriver._item(tag, _TTLV_TYPE_KINDS[type_name], value))
            expected = item_tree(original)
            typed = owned.enter_context(TypedAttribute.from_item(attribute, original))
            copied_value = owned.enter_context(typed.value)
            created = owned.enter_context(TypedAttribute.create(attribute, copied_value))
            if typed.attribute is not attribute or item_tree(typed) != expected or item_tree(created) != expected:
                raise ValueError('typed attribute identity or value copy assertion failed')
            populated = owned.enter_context(PythonKmipDriver._structure([original]))
            constructed = owned.enter_context(AttributeSet.try_new(populated))
            for collection, typed_append in ((constructed, None),
                    (owned.enter_context(AttributeSet.try_new(owned.enter_context(ttlv.ttlv_structure_create()))), False),
                    (owned.enter_context(AttributeSet.try_new(owned.enter_context(ttlv.ttlv_structure_create()))), True)):
                if typed_append is False:
                    collection.try_push(owned.enter_context(typed.as_item()))
                elif typed_append is True:
                    collection.try_push_typed(attribute, created)
                if collection.typed_count(attribute) != 1:
                    raise ValueError('attribute collection occurrence count assertion failed')
                retrieved = owned.enter_context(collection.typed_at(attribute, 0))
                if item_tree(retrieved) != expected:
                    raise ValueError('attribute collection value assertion failed')
            wire = encode_fixture(expected)
            decoded = owned.enter_context(ttlv.ttlv_decode(wire))
            if item_tree(decoded) != expected:
                raise ValueError('typed attribute generic roundtrip assertion failed')
            observed.append(expected)
    tree_hash = digest(observed)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _crypto_parameters_cell(cell: dict, case: dict, target: dict) -> dict:
    """Exercise named open-enum setters, typed getters and lossless converters."""
    from kmipkit import ttlv
    from kmipkit.cryptographic_parameters import CryptographicParameters

    if case['requires'].get('target_kind') != 'enumeration_value':
        raise RecipePending
    names = {
        'Block Cipher Mode': 'block_cipher_mode', 'Padding Method': 'padding_method',
        'Hashing Algorithm': 'hashing_algorithm', 'Key Role Type': 'key_role_type',
        'Digital Signature Algorithm': 'digital_signature_algorithm',
        'Cryptographic Algorithm': 'cryptographic_algorithm', 'Mask Generator': 'mask_generator',
    }
    by_id, _, _ = _catalog_model_data()
    methods = {int(item['wire_value'], 16): names[item['name']] for item in by_id.values()
               if item.get('kind') == 'tag' and item.get('name') in names}
    samples = _generic_samples(case, target, 'kmipkit.ttlv.ttlv_item_create')
    if not samples or any(tag not in methods or kind != 'Enumeration' for tag, kind, _ in samples):
        raise RecipePending
    observed = []
    for tag, _, value in samples:
        field = methods[tag]
        # Hashing Algorithm also supplies Mask Generator Hashing Algorithm;
        # choose the exact public setter required by this route.
        if cell['public_api_symbol'].endswith('.mask_generator_hashing_algorithm'):
            field = 'mask_generator_hashing_algorithm'
            tag = 0x420102
        with ExitStack() as owned:
            builder = owned.enter_context(CryptographicParameters.builder())
            getattr(builder, field)(value)
            context_fields = []
            # KMIP 2.1 Table 59 requires IV Length for CTR/CCM/GCM,
            # and Tag Length for GCM; these are explicit fixture inputs.
            if field == 'block_cipher_mode' and value in (6, 8, 9):
                iv_length = 128 if value == 6 else 96
                builder.iv_length(iv_length)
                context_fields.append([0x4200CD, 2, iv_length])
                if value == 9:
                    builder.tag_length(128)
                    context_fields.append([0x4200CE, 2, 128])
            model = owned.enter_context(builder.build())
            if getattr(model, field)() != value:
                raise ValueError('cryptographic parameter getter did not preserve the enum')
            payload = owned.enter_context(model.into_ttlv())
            copied = owned.enter_context(CryptographicParameters.from_ttlv(payload))
            if getattr(copied, field)() != value:
                raise ValueError('cryptographic parameter decoder did not preserve the enum')
            copied_payload = owned.enter_context(copied.into_ttlv())
            item = owned.enter_context(PythonKmipDriver._item(0x42002B, 'structure', payload))
            copied_item = owned.enter_context(PythonKmipDriver._item(0x42002B, 'structure', copied_payload))
            expected = [0x42002B, 1, [[tag, 5, value], *context_fields]]
            if item_tree(item) != expected or item_tree(copied_item) != expected:
                raise ValueError('cryptographic parameter typed/generic tree assertion failed')
            decoded = owned.enter_context(ttlv.ttlv_decode(encode_fixture(expected)))
            if item_tree(decoded) != expected:
                raise ValueError('cryptographic parameter generic roundtrip assertion failed')
            observed.append(expected[2][0])
    tree_hash = digest(observed)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _tag_rejection_cell(cell: dict, case: dict) -> dict:
    """Check all catalog range endpoints without constructing a nonexistent Item."""
    import re
    from kmipkit import errors, ttlv

    requires = case['requires']
    if requires['target_kind'] == 'tag_range':
        bounds = re.split(r'\s*[-–—]\s*', requires['wire_value'])
        values = [int(bound.upper().replace('X', fill), 16)
                  for bound, fill in zip(bounds, ('0', 'F'), strict=True)]
    else:
        values = PythonKmipDriver._wire_representatives(requires['wire_value'])
    inputs = [[raw, 2, 0] for raw in values]
    symbol = cell['public_api_symbol']
    for tree in inputs:
        raw = tree[0]
        if symbol == 'kmipkit.ttlv.ttlv_decode':
            try:
                unexpected = ttlv.ttlv_decode(encode_fixture(tree))
            except errors.TtlvDecodeError:
                continue
            unexpected.close()
            raise ValueError('decoder accepted a reserved or unused tag allocation')
        if symbol not in ('kmipkit.ttlv.raw_tag_from_raw', 'kmipkit.ttlv.raw_tag_try_checked'):
            raise RecipePending
        with ttlv.raw_tag_from_raw(raw) as value:
            if ttlv.raw_tag_value(value) != raw:
                raise ValueError('raw tag allocation identity was not preserved')
            if symbol == 'kmipkit.ttlv.raw_tag_try_checked':
                try:
                    unexpected = ttlv.raw_tag_try_checked(value)
                except errors.InvalidExtensionSchemaError:
                    continue
                unexpected.close()
                raise ValueError('checked tag accepted a reserved or unused allocation')
    if symbol == 'kmipkit.ttlv.ttlv_decode':
        return {'decode_outcome': 'rejected', 'decode_rejection_code': 'tag_not_allocated',
                'decode_input_sha256': hashlib.sha256(encode_fixture(inputs[0])).hexdigest()}
    return {'request_tree_sha256': digest(inputs)}
