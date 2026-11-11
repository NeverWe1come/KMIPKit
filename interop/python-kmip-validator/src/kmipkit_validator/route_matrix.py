"""Execute KMIPKIT-0092 cells through the installed public Python package.

A recipe is never an API capability declaration. Missing recipes are emitted in
an independent pending-coverage ledger and deliberately fail the shared gate.
Only successful, observed calls can produce a positive execution event.
"""
from __future__ import annotations

from contextlib import ExitStack
import hashlib
import importlib
import inspect
import json
from pathlib import Path
import struct
import sys
import uuid
from typing import Any

from .contract import ROOT, SHARED
from .driver import PythonKmipDriver, _TTLV_TYPE_KINDS, _catalog_model_data

# The shared generator is the sole authority for applicable cells.
sys.path.insert(0, str(SHARED / 'tools'))
from coverage_matrix import iter_cells  # noqa: E402


class RecipePending(Exception):
    """The validator cannot yet demonstrate the required case semantics."""


def digest(value: Any) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


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
            function = public_callable(symbol)
            while callable(function):
                code = getattr(function, '__code__', None)
                if code is not None:
                    self.codes.setdefault(code, set()).add(symbol)
                function = getattr(function, '__wrapped__', None)
        self.previous = None

    def _profile(self, frame, event, arg):
        if event == 'call':
            self.observed.update(self.codes.get(frame.f_code, ()))

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
    kind = requires.get('target_kind')
    target_id = target['target_id']
    by_id, element_types, elements = _catalog_model_data()
    tag = 0x4200FF
    if kind in ('enumeration_value', 'referenced_enum_value', 'bitmask_value'):
        raw_values = PythonKmipDriver._wire_representatives(requires['wire_value'])
        parent_ids = target.get('parent_element_ids', [])
        parent = next((elements.get(value, {}) for value in parent_ids
                       if elements.get(value, {}).get('kind') in ('enumeration', 'bitmask')), {})
        parent_name = parent.get('name', '')
        tag_entry = next((entry for entry in by_id.values() if entry.get('kind') == 'tag'
                          and entry.get('name') == parent_name), None)
        if tag_entry:
            tag = int(tag_entry['wire_value'], 16)
        elif kind == 'referenced_enum_value':
            tag = 0x42015A if 'FUNCTION' in target_id else 0x42015D
        else:
            raise RecipePending
        type_name = 'Integer' if kind == 'bitmask_value' else 'Enumeration'
        return [(tag, type_name, value if type_name == 'Enumeration' or value < 0x80000000
                 else value - 0x100000000) for value in raw_values]
    if kind in ('tag', 'tag_range'):
        # Raw/checked tag allocation and rejection is exercised separately.
        raise RecipePending
    if kind == 'data_type':
        type_names = {target['name']}
    else:
        type_names = PythonKmipDriver._catalog_value_types(case, target, element_types)
        # A type representative alone cannot prove a normative payload branch,
        # managed object shape, attribute persistence or protocol workflow.
        raise RecipePending
    if symbol.startswith('kmipkit.ttlv.ttlv_value_'):
        suffix = symbol.rsplit('ttlv_value_', 1)[1]
        type_names = {name for name in type_names if _TTLV_TYPE_KINDS[name] == suffix}
    samples = {
        'Structure': [None], 'Integer': [-0x80000000, -1, 0, 0x7FFFFFFF],
        'Long Integer': [-(1 << 63), -1, 0, (1 << 63) - 1],
        'Big Integer': [bytes.fromhex('0000000000000001'), bytes.fromhex('ffffffffffffffff')],
        'Enumeration': [0, 1, 0x7FFFFFFF, 0x80000000], 'Boolean': [False, True],
        'Text String': ['', 'KMIP-λ'], 'Byte String': [b'', bytes(range(9))],
        'Date Time': [-1, 0, 1], 'Interval': [0, 1, 0xFFFFFFFF], 'Date Time Extended': [-1, 0, 1],
    }
    return [(tag, name, value) for name in sorted(type_names) for value in samples[name]]


def _generic_cell(cell: dict, case: dict, target: dict) -> dict:
    from kmipkit import ttlv

    symbol = cell['public_api_symbol']
    if symbol not in _GENERIC_SYMBOLS:
        raise RecipePending
    samples = _generic_samples(case, target, symbol)
    if not samples:
        raise RecipePending
    observed = []
    for tag, name, value in samples:
        with ExitStack() as owned:
            if name == 'Structure':
                child = owned.enter_context(PythonKmipDriver._item(0x420001, 'integer', 7))
                children = owned.enter_context(ttlv.ttlv_structure_create())
                with ttlv.codec_limits_defaults() as limits:
                    children = owned.enter_context(ttlv.ttlv_structure_with_item(children, child, limits))
                value = children
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
        *(f'ttlv_value_{name}' for name in _TTLV_TYPE_KINDS.values()),
    )
}


def execute_cell(cell: dict, case: dict, target: dict) -> dict:
    if cell['transport'] is None:
        return _generic_cell(cell, case, target)
    raise RecipePending


def run_matrix(catalog: dict, manifest: dict, artifact: dict,
               *, catalog_sha256: str | None = None, route_manifest_sha256: str | None = None) -> tuple[dict, list[dict]]:
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
    event_fields = ('case_id', 'route_id', 'target_id', 'workflow_step_index', 'workflow_operation',
                    'workflow_item_index', 'public_api_symbol', 'execution_route_id', 'execution_api_symbol',
                    'action', 'protocol_version', 'encoding', 'transport')
    for cell in iter_cells(catalog, manifest, languages=('python',)):
        identity = {field: cell[field] for field in event_fields if field in cell}
        try:
            with CallTrace([cell['public_api_symbol']]) as trace:
                observed = execute_cell(cell, cases[cell['case_id']], targets[cell['target_id']])
            if cell['public_api_symbol'] not in trace.observed:
                raise RecipePending
            report['events'].append({**identity, 'invocation_id': str(uuid.uuid4()),
                                     'invocation_count': 1, 'status': 'passed', **observed})
        except RecipePending:
            gaps.append({**identity, 'failure_code': 'recipe_pending'})
        except Exception:
            # No exception names, sources, native text or raw fixture values leave here.
            gaps.append({**identity, 'failure_code': 'recipe_execution_failed'})
    return report, gaps
