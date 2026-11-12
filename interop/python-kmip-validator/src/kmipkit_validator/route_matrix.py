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


def _target_field(target: dict, requires: dict, field: str) -> Any:
    """Prefer each expanded normative target's exact value over case defaults."""
    return target.get(field, requires.get('target_kind' if field == 'kind' else field))


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
    kind = _target_field(target, requires, 'kind')
    target_id = target['target_id']
    by_id, element_types, elements = _catalog_model_data()
    tag = 0x4200FF
    special_samples: list[tuple[int, str]] | None = None

    def find_tag(*names: str) -> dict | None:
        wanted = {_normalized_name(name) for name in names if name}
        return next((entry for entry in by_id.values() if entry.get('kind') == 'tag'
                     and _normalized_name(entry.get('name', '')) in wanted), None)

    if kind in ('enumeration_value', 'referenced_enum_value', 'bitmask_value'):
        wire_value = target.get('wire_value', requires.get('wire_value'))
        if not isinstance(wire_value, str):
            raise RecipePending
        raw_values = PythonKmipDriver._wire_representatives(wire_value)
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
    allocation = _target_field(target, requires, 'allocation')
    wire_value = _target_field(target, requires, 'wire_value')
    if kind == 'tag' and allocation in ('assigned', 'extension') and isinstance(wire_value, str):
        return [(raw, 'Integer', 0) for raw in PythonKmipDriver._wire_representatives(wire_value)]
    if kind == 'tag_range' and allocation == 'extension' and isinstance(wire_value, str):
        import re
        bounds = re.split(r'\s*[-–—]\s*', wire_value)
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
    if isinstance(target.get('source_encoding'), str):
        source_type = target['source_encoding'].split(',', 1)[0].strip()
        encoded_types = {
            name for name in _TTLV_TYPE_KINDS
            if _normalized_name(name) == _normalized_name(source_type)
        }
        if encoded_types:
            type_names = encoded_types
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
    if (_target_field(target, case['requires'], 'kind') in ('tag', 'tag_range')
            and _target_field(target, case['requires'], 'allocation') in ('reserved', 'unused')):
        return _tag_rejection_cell(cell, case, target)
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


def _operation_structure_ttlv_cell(cell: dict, case: dict, target: dict,
                                    trace: CallTrace | None) -> dict:
    """Build a catalogued Operation Structure child in a generic Structure."""
    from kmipkit import ttlv

    symbol = cell['public_api_symbol']
    samples = _generic_samples(case, target, 'kmipkit.ttlv.ttlv_item_create')
    if not samples:
        raise RecipePending
    tag, source_type, _ = samples[0]
    encoded_type = target.get('source_encoding', source_type).split(',', 1)[0].strip()
    type_name = next((name for name in _TTLV_TYPE_KINDS
                      if _normalized_name(name) == _normalized_name(encoded_type)), None)
    if type_name is None:
        raise RecipePending
    neutral_values = {
        'Integer': 1024,
        'Long Integer': 1024,
        'Enumeration': 1,
        'Boolean': True,
        'Text String': 'KMIPKit.Interop',
        'Byte String': b'KMIPKit.Interop',
        'Date Time': 1,
        'Date Time Extended': 1,
        'Interval': 1,
    }
    if type_name not in neutral_values:
        raise RecipePending
    value = neutral_values[type_name]
    kind_name = _TTLV_TYPE_KINDS.get(type_name)
    if kind_name is None:
        raise RecipePending

    with ExitStack() as owned:
        structure = owned.enter_context(ttlv.ttlv_structure_create())
        if symbol == 'kmipkit.ttlv.ttlv_structure_create':
            with ttlv.ttlv_structure_view(structure) as empty_view:
                if ttlv.ttlv_structure_view_item_count(empty_view) != 0:
                    raise ValueError('new generic Structure unexpectedly contains children')
        limits = owned.enter_context(ttlv.codec_limits_defaults())
        child = owned.enter_context(PythonKmipDriver._item(tag, kind_name, value))
        expected_child = item_tree(child)
        populated = owned.enter_context(ttlv.ttlv_structure_with_item(structure, child, limits))
        with ttlv.ttlv_structure_view(populated) as view:
            if ttlv.ttlv_structure_view_item_count(view) != 1:
                raise ValueError('generic Structure did not retain its catalogued child')
            with ttlv.ttlv_structure_view_item_at(view, 0) as child_view:
                with ttlv.ttlv_item_view_tag(child_view) as observed_tag:
                    raw_tag = ttlv.tag_value(observed_tag)
                observed_kind = int(ttlv.ttlv_item_view_type(child_view))
                with ttlv.ttlv_item_view_value(child_view) as observed_value:
                    if observed_kind in (4, 7, 8):
                        decoded_value = ttlv.ttlv_value_view_bytes(observed_value).hex()
                    else:
                        getter = {2: 'integer', 3: 'long_integer', 5: 'enumeration',
                                  6: 'boolean', 9: 'date_time', 10: 'interval',
                                  11: 'date_time_extended'}.get(observed_kind)
                        if getter is None:
                            raise ValueError('Operation Structure scalar type changed unexpectedly')
                        decoded_value = getattr(ttlv, 'ttlv_value_view_' + getter)(observed_value)
                observed_child = [raw_tag, observed_kind, decoded_value]
        if observed_child != expected_child or observed_child[0] != tag:
            raise ValueError('generic Structure child changed its catalogued tag or value')
        roundtrip = owned.enter_context(ttlv.ttlv_decode(encode_fixture(observed_child)))
        if item_tree(roundtrip) != observed_child:
            raise ValueError('Operation Structure child did not survive TTLV roundtrip')
        if trace is not None:
            trace.record_invocation(symbol)
    tree_hash = digest({'target': target['target_id'], 'child': observed_child})
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _certificate_attributes_ttlv_cell(cell: dict, target: dict,
                                      trace: CallTrace | None) -> dict:
    """View the Certificate Attributes grouping through its generic Attributes tree."""
    from kmipkit import ttlv

    with ExitStack() as owned:
        attributes = []
        for name, kind, value in (
                ('Certificate Type', 'enumeration', 1),
                ('Certificate Length', 'integer', 1024)):
            fields = [
                owned.enter_context(PythonKmipDriver._item(0x42000A, 'text', name)),
                owned.enter_context(PythonKmipDriver._item(0x42000B, kind, value)),
            ]
            attribute = owned.enter_context(PythonKmipDriver._structure(fields))
            attributes.append(owned.enter_context(
                PythonKmipDriver._item(0x420008, 'structure', attribute)
            ))
        values = owned.enter_context(PythonKmipDriver._structure(attributes))
        item = owned.enter_context(PythonKmipDriver._item(0x420125, 'structure', values))
        observed = item_tree(item)
        expected = [0x420125, 1, [
            [0x420008, 1, [[0x42000A, 7, '43657274696669636174652054797065'],
                           [0x42000B, 5, 1]]],
            [0x420008, 1, [[0x42000A, 7, '4365727469666963617465204c656e677468'],
                           [0x42000B, 2, 1024]]],
        ]]
        if observed != expected:
            raise ValueError('Certificate Attributes tree changed its named typed values')
        with ttlv.ttlv_item_view_from_item(item) as view:
            if (int(ttlv.tag_value(ttlv.ttlv_item_view_tag(view))) != 0x420125
                    or int(ttlv.ttlv_item_view_type(view)) != 1):
                raise ValueError('Certificate Attributes generic view changed its root')
            with ttlv.ttlv_item_view_value(view) as value:
                with ttlv.ttlv_value_view_structure(value) as children:
                    if ttlv.ttlv_structure_view_item_count(children) != 2:
                        raise ValueError('Certificate Attributes view changed its attribute count')
        if trace is not None:
            trace.record_invocation(cell['public_api_symbol'])
    tree_hash = digest({'target': target.get('target_id'), 'tree': observed})
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _server_put_ttlv_cell(cell: dict, target: dict,
                          trace: CallTrace | None) -> dict:
    """Create or decode the full generic TTLV form of a server Put request."""
    from kmipkit import protocol_values, ttlv

    major, minor = (int(part) for part in cell['protocol_version'].split('.'))
    object_tree = [0x42008F, 1, [[0x420040, 1, [
        [0x420042, 5, 1],
        [0x420045, 1, [[0x420043, 8, b'KMIPKitPutObject01'.hex()]]],
    ]]]]
    request_tree = [0x420078, 1, [
        [0x420077, 1, [
            [0x420069, 1, [[0x42006A, 2, major], [0x42006B, 2, minor]]],
            [0x420092, 9, 1],
            [0x42000D, 2, 1],
        ]],
        [0x42000F, 1, [[0x42005C, 5, protocol_values.Operation.PUT],
                       [0x420079, 1, [
                           [0x420094, 7, b'KMIPKit-put-object'.hex()],
                           [0x420070, 5, protocol_values.PutFunction.NEW],
                           object_tree,
                       ]]]],
    ]]
    wire = encode_fixture(request_tree)
    symbol = cell['public_api_symbol']
    with ExitStack() as owned:
        if symbol == 'kmipkit.ttlv.ttlv_decode':
            observed_item = owned.enter_context(ttlv.ttlv_decode(wire))
        else:
            observed_item = owned.enter_context(_credential_item_from_tree(request_tree, owned))
        observed = item_tree(observed_item)
        if observed != request_tree:
            raise ValueError('generic server Put TTLV changed its complete request tree')
        if symbol == 'kmipkit.ttlv.ttlv_item_view_from_item':
            with ttlv.ttlv_item_view_from_item(observed_item) as view:
                if (int(ttlv.tag_value(ttlv.ttlv_item_view_tag(view))) != 0x420078
                        or int(ttlv.ttlv_item_view_type(view)) != 1):
                    raise ValueError('server Put generic view changed its Request Message root')
                with ttlv.ttlv_item_view_value(view) as value:
                    with ttlv.ttlv_value_view_structure(value) as children:
                        if ttlv.ttlv_structure_view_item_count(children) != 2:
                            raise ValueError('server Put view changed its header and batch item')
        if trace is not None:
            trace.record_invocation(symbol)
    tree_hash = digest({'target': target.get('target_id'), 'tree': observed})
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash,
            'wire_bytes_sha256': hashlib.sha256(wire).hexdigest()}


def _managed_object_tag(elements: dict, name: str) -> int:
    normalized = _normalized_name(name)
    entry = next((item for item in elements.values()
                  if item.get('kind') == 'tag'
                  and _normalized_name(item.get('name', '')) == normalized), None)
    if entry is None:
        raise RecipePending
    return int(entry['wire_value'], 16)


def _managed_object_field_tree(element: dict, element_types: dict[str, set[str]],
                               elements: dict[str, dict]) -> list:
    """Build one catalogued structure member with its source TTLV type."""
    types = PythonKmipDriver._catalog_value_types({'requires': {}}, element, element_types)
    if 'Structure' in types:
        kind, value = 1, []
    elif 'Enumeration' in types:
        kind, value = 5, 1
    elif 'Big Integer' in types:
        kind, value = 4, '0000000000000001'
    elif 'Integer' in types:
        kind, value = 2, 1
    elif 'Byte String' in types:
        kind, value = 8, b'KMIPKit.Interop'.hex()
    elif 'Text String' in types:
        kind, value = 7, 'KMIPKit.Interop'.encode().hex()
    elif 'Boolean' in types:
        kind, value = 6, 1
    elif 'Date Time' in types:
        kind, value = 9, 1
    elif 'Interval' in types:
        kind, value = 10, 1
    elif 'Date Time Extended' in types:
        kind, value = 11, 1
    else:
        raise RecipePending
    return [_managed_object_tag(elements, element.get('name', '')), kind, value]


def _managed_object_key_material(element_id: str, elements: dict[str, dict],
                                 include_member_id: str | None = None) -> tuple[int, list]:
    """Create valid Key Material for the target's transparent format table."""
    element = elements[element_id]
    section = next((reference.get('section') for reference in element.get('source_refs', [])
                    if reference.get('section', '').startswith('3.')), None)
    formats = {'3.4': 7, '3.5': 8, '3.6': 9, '3.7': 10, '3.8': 11,
               '3.9': 12, '3.10': 13, '3.11': 20, '3.12': 21}
    field_order = {
        '3.4': ('Key',),
        '3.5': ('P', 'Q', 'G', 'X'),
        '3.6': ('P', 'Q', 'G', 'Y'),
        '3.7': ('Modulus', 'Public Exponent', 'Private Exponent', 'P', 'Q',
                'Prime Exponent P', 'Prime Exponent Q', 'CRT Coefficient'),
        '3.8': ('Modulus', 'Public Exponent'),
        '3.9': ('P', 'Q', 'G', 'J', 'X'),
        '3.10': ('P', 'Q', 'G', 'J', 'Y'),
        '3.11': ('Recommended Curve', 'D'),
        '3.12': ('Recommended Curve', 'Q String'),
    }
    format_type = formats.get(section)
    names = field_order.get(section)
    if format_type is None or names is None:
        raise RecipePending
    members = [member for member in elements.values()
               if member.get('kind') == 'structure_member'
               and element_id in member.get('parent_element_ids', [])]
    by_name = {_normalized_name(member.get('name', '')): member for member in members}
    children = [_managed_object_field_tree(by_name[_normalized_name(name)], {}, elements)
                for name in names if _normalized_name(name) in by_name
                and (section == '3.7'
                     or by_name[_normalized_name(name)].get('source_requiredness') == 'Yes'
                     or by_name[_normalized_name(name)].get('element_id') == include_member_id)]
    return format_type, children


def _managed_object_wrapping_data(elements: dict[str, dict]) -> list:
    """Create valid Key Wrapping Data with both key-information alternatives."""
    encryption = [_managed_object_tag(elements, 'Unique Identifier'), 7,
                  'encryption-fixture'.encode().hex()]
    mac = [_managed_object_tag(elements, 'Unique Identifier'), 7,
           'mac-fixture'.encode().hex()]
    encryption_info = [_managed_object_tag(elements, 'Encryption Key Information'), 1,
                       [encryption, [_managed_object_tag(elements, 'Cryptographic Parameters'), 1, []]]]
    mac_info = [_managed_object_tag(elements, 'MAC/Signature Key Information'), 1,
                [mac, [_managed_object_tag(elements, 'Cryptographic Parameters'), 1, []]]]
    fields = [
        [_managed_object_tag(elements, 'Wrapping Method'), 5, 1],
        encryption_info,
        mac_info,
        [_managed_object_tag(elements, 'MAC/Signature'), 8, b'mac-fixture'.hex()],
        [_managed_object_tag(elements, 'IV/Counter/Nonce'), 8, b'iv-fixture'.hex()],
        [_managed_object_tag(elements, 'Encoding Option'), 5, 1],
    ]
    return [_managed_object_tag(elements, 'Key Wrapping Data'), 1, fields]


def _managed_object_tree(target: dict) -> list:
    """Construct a minimal nested Key Block carrying the exact normative target."""
    _, element_types, elements = _catalog_model_data()
    element = elements.get(target['target_id'])
    if element is None:
        raise RecipePending
    kind = target.get('kind')
    target_id = target['target_id']
    has_wrapping_target = any(marker in target_id for marker in (
        'KEY-WRAPPING-DATA', 'ENCRYPTION-KEY-INFORMATION',
        'MAC-SIGNATURE-KEY-INFORMATION')) or target_id.endswith('TAG-420046')
    key_material_ids = []
    if kind == 'object_structure' and element.get('name') == 'Key Material':
        key_material_ids.append(element['element_id'])
    elif kind == 'structure_member':
        key_material_ids.extend(parent for parent in element.get('parent_element_ids', [])
                                if elements.get(parent, {}).get('kind') == 'object_structure'
                                and elements[parent].get('name') == 'Key Material')
        parent_names = {elements.get(parent, {}).get('name')
                        for parent in element.get('parent_element_ids', [])}
        if element.get('name') == 'Key Material' and 'Key Value' in parent_names:
            key_material_ids.append(next((candidate['element_id'] for candidate in elements.values()
                                          if candidate.get('kind') == 'object_structure'
                                          and candidate.get('name') == 'Key Material'
                                          and any(reference.get('section') == '3.4'
                                                  for reference in candidate.get('source_refs', []))), ''))
    elif ((kind == 'object_structure' and element.get('name') == 'Key Value')
          or (kind == 'tag' and element.get('name') == 'Key Material')):
        key_material_ids.append(next((candidate['element_id'] for candidate in elements.values()
                                      if candidate.get('kind') == 'object_structure'
                                      and candidate.get('name') == 'Key Material'
                                      and any(reference.get('section') == '3.4'
                                              for reference in candidate.get('source_refs', []))), ''))
    key_material_ids = [element_id for element_id in key_material_ids if element_id]
    if key_material_ids:
        format_type, material_children = _managed_object_key_material(
            key_material_ids[0], elements,
            target['target_id'] if kind == 'structure_member' else None)
        material = [0x420043, 1, material_children]
        key_value = [0x420045, 1, [material]]
    else:
        format_type = 1
        if has_wrapping_target:
            key_value = [0x420045, 8, b'wrapped-fixture'.hex()]
        else:
            key_value = [0x420045, 1, [[0x420043, 8, b'KMIPKitInterop01'.hex()]]]
    if 'KEY-VALUE-ATTRIBUTES' in target_id:
        key_value[2].append([_managed_object_tag(elements, 'Attributes'), 1, []])
    key_block_fields = [[0x420042, 5, format_type]]
    if 'KEY-COMPRESSION-TYPE' in target_id:
        key_block_fields.append([_managed_object_tag(elements, 'Key Compression Type'), 5, 1])
    cryptographic_target = kind == 'structure_member' and element.get('name') in (
        'Cryptographic Algorithm', 'Cryptographic Length')
    if cryptographic_target:
        key_block_fields.extend(([0x420028, 5, 3], [0x42002A, 2, 128]))
    else:
        key_block_fields.append(key_value)
    if has_wrapping_target:
        key_block_fields.append(_managed_object_wrapping_data(elements))
    if kind == 'object_structure' and element.get('name') not in (
            'Key Block', 'Key Value', 'Key Wrapping Data',
            'Encryption Key Information', 'MAC/Signature Key Information') and not key_material_ids:
        raise RecipePending
    if kind == 'structure_member' and not key_material_ids and not any(
            marker in target_id for marker in ('KEY-BLOCK', 'KEY-VALUE', 'KEY-WRAPPING-DATA',
                                                'ENCRYPTION-KEY-INFORMATION',
                                                'MAC-SIGNATURE-KEY-INFORMATION')):
        raise RecipePending
    return [0x420040, 1, key_block_fields]


_MANAGED_OBJECT_TYPES = {
    'Certificate': 1, 'Symmetric Key': 2, 'Public Key': 3, 'Private Key': 4,
    'Split Key': 5, 'Secret Data': 7, 'Opaque Object': 8, 'PGP Key': 9,
    'Certificate Request': 10,
}


def _managed_object_target_is_in_key_block(target: dict, elements: dict[str, dict]) -> bool:
    """Determine whether a catalog target belongs to an object's nested Key Block."""
    key_block_id = next((element_id for element_id, element in elements.items()
                         if element.get('kind') == 'object_structure'
                         and element.get('name') == 'Key Block'), None)
    if key_block_id is None:
        return False
    pending = [target.get('target_id'), *target.get('parent_element_ids', [])]
    visited = set()
    while pending:
        element_id = pending.pop()
        if element_id == key_block_id:
            return True
        if not element_id or element_id in visited:
            continue
        visited.add(element_id)
        element = elements.get(element_id)
        if element is not None:
            pending.extend(element.get('parent_element_ids', []))
    return False


def _managed_object_root_tree(name: str, elements: dict[str, dict],
                              target: dict | None = None) -> list:
    """Build the required fields of one OASIS §§2.1-2.9 managed-object root."""
    object_tag = _managed_object_tag(elements, name)
    if target is not None and _managed_object_target_is_in_key_block(target, elements):
        key_block = _managed_object_tree(target)
    else:
        key_block = [0x420040, 1, [
            [0x420042, 5, 1],
            [0x420045, 1, [[0x420043, 8, b'KMIPKitInterop01'.hex()]]],
        ]]
    fields = {
        'Certificate': [[0x42001D, 5, 1], [0x42001E, 8, b'certificate-fixture'.hex()]],
        'Certificate Request': [[0x420019, 5, 1], [0x420140, 8, b'certificate-request-fixture'.hex()]],
        'Opaque Object': [[0x420059, 5, 1], [0x42005A, 8, b'opaque-fixture'.hex()]],
        'PGP Key': [[0x4200BE, 2, 4], key_block],
        'Private Key': [key_block],
        'Public Key': [key_block],
        'Secret Data': [[0x420086, 5, 1], key_block],
        # KMIP 2.0/2.1 §2.8 requires Prime Field Size for method 3.
        'Split Key': [[0x42008B, 2, 2], [0x420044, 2, 1], [0x42008C, 2, 1],
                      [0x42008A, 5, 3], [0x420062, 4, '0000000000000017'], key_block],
        'Symmetric Key': [key_block],
    }.get(name)
    if fields is None:
        raise RecipePending
    return [object_tag, 1, fields]


def _managed_object_root_name(target: dict, elements: dict[str, dict]) -> str:
    roots = {element['element_id']: element['name'] for element in elements.values()
             if element.get('kind') == 'object_structure'
             and element.get('name') in _MANAGED_OBJECT_TYPES}
    pending = [target.get('target_id'), *target.get('parent_element_ids', [])]
    visited = set()
    ancestor_roots = set()
    while pending:
        element_id = pending.pop(0)
        if not element_id or element_id in visited:
            continue
        visited.add(element_id)
        if element_id in roots:
            ancestor_roots.add(roots[element_id])
            continue
        element = elements.get(element_id)
        if element is not None:
            pending.extend(element.get('parent_element_ids', []))
    if ancestor_roots:
        for preferred in ('Certificate', 'Certificate Request', 'Opaque Object',
                          'Private Key', 'Public Key', 'Symmetric Key', 'Secret Data',
                          'PGP Key', 'Split Key'):
            if preferred in ancestor_roots:
                return preferred
    target_id = target.get('target_id', '')
    if any(f'-3-{index}-KEY-MATERIAL' in target_id for index in (5, 7, 9, 11)):
        return 'Private Key'
    if any(f'-3-{index}-KEY-MATERIAL' in target_id for index in (6, 8, 10, 12)):
        return 'Public Key'
    if target.get('kind') in {'object_structure', 'structure_member'}:
        return 'Symmetric Key'
    if target.get('kind') == 'tag':
        for name in _MANAGED_OBJECT_TYPES:
            if _managed_object_tag(elements, name) == int(target.get('wire_value', '0'), 16):
                return name
    raise RecipePending


def _assert_managed_object_target(target: dict, tree: list, name: str,
                                  elements: dict[str, dict]) -> None:
    if target.get('kind') == 'structure_member':
        member_tag = _managed_object_tag(elements, target.get('name', ''))
        if not _credential_tree_has_tag(tree, member_tag):
            raise RecipePending
    elif target.get('kind') == 'tag' and tree[0] != int(target['wire_value'], 16):
        raise ValueError('managed-object route changed the target tag')
    elif target.get('kind') == 'object_structure':
        if target.get('name') in _MANAGED_OBJECT_TYPES:
            if tree[0] != _managed_object_tag(elements, name):
                raise ValueError('managed-object route changed the root object tag')
        elif not _credential_tree_has_tag(tree, _managed_object_tag(elements, target.get('name', ''))):
            raise RecipePending


def _managed_object_cell(cell: dict, case: dict, target: dict) -> dict:
    """Invoke the exact public managed-object constructor/parser route."""
    from kmipkit import managed_objects

    symbol = cell['public_api_symbol']
    if symbol.startswith('kmipkit.managed_objects.KeyWrappingData.'):
        tree = _managed_object_tree(target)
        key_wrapping_tag = _managed_object_tag(_catalog_model_data()[2], 'Key Wrapping Data')

        def find_wrapping_node(node: list) -> list | None:
            if node[0] == key_wrapping_tag:
                return node
            if node[1] == 1:
                for child in node[2]:
                    found = find_wrapping_node(child)
                    if found is not None:
                        return found
            return None

        value_node = find_wrapping_node(tree)
        if value_node is None or value_node[1] != 1:
            raise RecipePending
        with ExitStack() as owned:
            fields = [owned.enter_context(_credential_item_from_tree(field, owned))
                      for field in value_node[2]]
            if symbol.endswith('.from_fields'):
                result = owned.enter_context(managed_objects.KeyWrappingData.from_fields(fields))
            elif symbol.endswith('.from_item'):
                item = owned.enter_context(_credential_item_from_tree(value_node, owned))
                result = owned.enter_context(managed_objects.KeyWrappingData.from_item(item))
            else:
                raise RecipePending
            observed = item_tree(result)
            if observed != value_node:
                raise ValueError('Key Wrapping Data constructor changed ordered fields')
        if not _credential_tree_has_tag(
                observed, _managed_object_tag(_catalog_model_data()[2], target.get('name', ''))):
            raise RecipePending
        tree_hash = digest(observed)
        return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}
    if symbol.startswith(('kmipkit.managed_objects.KeyValue.',
                          'kmipkit.managed_objects.KeyMaterial.')):
        tree = _managed_object_tree(target)
        key_format = next((value for tag, _, value in tree[2] if tag == 0x420042), None)

        def find_node(node: list, raw_tag: int) -> list | None:
            if node[0] == raw_tag:
                return node
            if node[1] == 1:
                for child in node[2]:
                    found = find_node(child, raw_tag)
                    if found is not None:
                        return found
            return None

        if key_format is None:
            raise RecipePending
        key_value = find_node(tree, 0x420045)
        is_key_value = symbol.startswith('kmipkit.managed_objects.KeyValue.')
        value_node = key_value if is_key_value else find_node(key_value, 0x420043) if key_value else None
        if value_node is None or value_node[1] != 1:
            raise RecipePending
        fields_tree = value_node[2]
        with ExitStack() as owned:
            fields = [owned.enter_context(_credential_item_from_tree(field, owned))
                      for field in fields_tree]
            if symbol.endswith('.from_fields'):
                constructor = managed_objects.KeyValue if is_key_value else managed_objects.KeyMaterial
                result = owned.enter_context(constructor.from_fields(key_format, fields))
            elif symbol.endswith('.from_item'):
                item = owned.enter_context(_credential_item_from_tree(value_node, owned))
                constructor = managed_objects.KeyValue if is_key_value else managed_objects.KeyMaterial
                result = owned.enter_context(constructor.from_item(key_format, item))
            else:
                raise RecipePending
            observed = item_tree(result)
            if observed != value_node:
                raise ValueError('key value or material constructor changed ordered fields')
        target_tag = (int(target['wire_value'], 16) if target.get('kind') == 'tag'
                      else _managed_object_tag(_catalog_model_data()[2], target.get('name', '')))
        if not _credential_tree_has_tag(observed, target_tag):
            raise RecipePending
        tree_hash = digest(observed)
        return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}
    if symbol.startswith('kmipkit.managed_objects.KeyBlock.'):
        tree = _managed_object_tree(target)
        if tree[0] != 0x420040 or tree[1] != 1:
            raise ValueError('managed-object fixture did not produce a Key Block Structure')
        with ExitStack() as owned:
            fields = [owned.enter_context(_credential_item_from_tree(field, owned))
                      for field in tree[2]]
            if symbol.endswith('.from_fields'):
                result = owned.enter_context(managed_objects.KeyBlock.from_fields(fields))
            elif symbol.endswith('.from_item'):
                structure = owned.enter_context(PythonKmipDriver._structure(fields))
                source = owned.enter_context(PythonKmipDriver._item(0x420040, 'structure', structure))
                result = owned.enter_context(managed_objects.KeyBlock.from_item(source))
            else:
                raise RecipePending
            observed = item_tree(result)
            if observed != tree:
                raise ValueError('Key Block route changed the exact ordered TTLV fields')
        if target.get('kind') == 'tag' and target['target_id'].endswith('TAG-420040'):
            matched = observed[0] == int(target['wire_value'], 16)
        elif target.get('kind') == 'structure_member':
            matched = _credential_tree_has_tag(
                observed, _managed_object_tag(_catalog_model_data()[2], target.get('name', '')))
        elif target.get('kind') == 'object_structure':
            matched = observed[0] == 0x420040 or _credential_tree_has_tag(
                observed, _managed_object_tag(_catalog_model_data()[2], target.get('name', '')))
        else:
            matched = False
        if not matched:
            raise ValueError('Key Block route did not retain the exact normative target')
        tree_hash = digest(observed)
        return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}
    if not symbol.startswith('kmipkit.managed_objects.ManagedObject.'):
        raise RecipePending
    elements = _catalog_model_data()[2]
    if target.get('kind') not in {'object_structure', 'structure_member', 'tag'}:
        raise RecipePending
    factory_names = {
        'certificate': 'Certificate', 'certificate_request': 'Certificate Request',
        'opaque_object': 'Opaque Object', 'pgp_key': 'PGP Key',
        'private_key': 'Private Key', 'public_key': 'Public Key',
        'secret_data': 'Secret Data', 'split_key': 'Split Key',
        'symmetric_key': 'Symmetric Key',
    }
    name = factory_names.get(symbol.rsplit('.', 1)[-1]) or _managed_object_root_name(target, elements)
    tree = _managed_object_root_tree(name, elements, target)
    object_type = _MANAGED_OBJECT_TYPES[name]
    with ExitStack() as owned:
        fields = [owned.enter_context(_credential_item_from_tree(field, owned))
                  for field in tree[2]]
        if symbol.endswith('.from_fields'):
            result = owned.enter_context(managed_objects.ManagedObject.from_fields(object_type, fields))
        elif symbol.endswith('.from_item'):
            structure = owned.enter_context(PythonKmipDriver._structure(fields))
            source = owned.enter_context(PythonKmipDriver._item(tree[0], 'structure', structure))
            result = owned.enter_context(managed_objects.ManagedObject.from_item(object_type, source))
        elif symbol.rsplit('.', 1)[-1] in {
                'certificate', 'certificate_request', 'opaque_object', 'pgp_key', 'private_key',
                'public_key', 'secret_data', 'split_key', 'symmetric_key'}:
            constructor = getattr(managed_objects.ManagedObject, symbol.rsplit('.', 1)[-1])
            result = owned.enter_context(constructor(fields))
        else:
            raise RecipePending
        observed = item_tree(result)
        if observed != tree:
            raise ValueError('managed-object route changed the ordered OASIS fields')
        if result.object_type != object_type or observed[0] != _managed_object_tag(elements, name):
            raise ValueError('managed-object route changed the object type or root tag')
        _assert_managed_object_target(target, observed, name, elements)
    tree_hash = digest(observed)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _typed_message_cell(cell: dict, case: dict, target: dict) -> dict:
    """Parse a deterministic generic request/response through the public message API."""
    from kmipkit import messages

    symbol = cell['public_api_symbol']
    fixtures = {
        'kmipkit.messages.RequestMessage.from_ttlv': (
            messages.RequestMessage, 0x420078,
            [0x420078, 1, [
                [0x420077, 1, [
                    [0x420069, 1, [[0x42006A, 2, 2], [0x42006B, 2, 1]]],
                    [0x42000D, 2, 1],
                ]],
                [0x42000F, 1, [
                    [0x42005C, 5, 0x3B], [0x420079, 1, []],
                ]],
            ]],
        ),
        'kmipkit.messages.ResponseMessage.from_ttlv': (
            messages.ResponseMessage, 0x42007B,
            [0x42007B, 1, [
                [0x42007A, 1, [
                    [0x420069, 1, [[0x42006A, 2, 2], [0x42006B, 2, 1]]],
                    [0x420092, 9, 1_234_567], [0x42000D, 2, 1],
                ]],
                [0x42000F, 1, [
                    [0x42005C, 5, 0x3B], [0x42007F, 5, 0], [0x42007C, 1, []],
                ]],
            ]],
        ),
    }
    fixture = fixtures.get(symbol)
    if fixture is None:
        raise RecipePending
    message_type, root_tag, original = fixture
    import copy
    expected = copy.deepcopy(original)
    _, _, elements = _catalog_model_data()
    tag_elements = [element for element in elements.values() if element.get('kind') == 'tag']

    def field_name(element: dict) -> str:
        name = element.get('name', '')
        if name.startswith('Credential,'):
            return 'Credential'
        return name

    def tag_value(element: dict) -> int:
        name = field_name(element)
        normalized = _normalized_name(name)
        tag = next((item for item in tag_elements
                    if _normalized_name(item.get('name', '')) == normalized), None)
        if tag is None:
            raise RecipePending
        return int(tag['wire_value'], 16)

    def field_encoding(element: dict) -> str | None:
        name = _normalized_name(field_name(element))
        return next((item['source_encoding'] for item in elements.values()
                     if item.get('kind') == 'message_field'
                     and _normalized_name(field_name(item)) == name
                     and item.get('source_encoding')), None)

    def field_node(element: dict) -> list:
        name = field_name(element)
        encoding = field_encoding(element)
        structures = {'Authentication', 'Batch Item', 'Credential', 'Credential Value',
                      'Message Extension', 'Nonce', 'Protocol Version', 'Request Header',
                      'Request Message', 'Request Payload', 'Response Header',
                      'Response Message', 'Response Payload', 'Vendor Extension'}
        if encoding == 'Structure' or name in structures:
            defaults = {
                'Authentication': [[0x420023, 1, [
                    [0x420024, 5, 1], [0x420025, 1, [
                        [0x420026, 7, '696e7465726f702d75736572'],
                        [0x420027, 7, '66697874757265'],
                    ]],
                ]]],
                'Credential': [[0x420024, 5, 1], [0x420025, 1, [
                    [0x420026, 7, '696e7465726f702d75736572'],
                    [0x420027, 7, '66697874757265'],
                ]]],
                'Credential Value': [[0x420026, 7, '696e7465726f702d75736572'],
                                     [0x420027, 7, '66697874757265']],
                'Message Extension': [[0x42009D, 7, '4b4d49504b6974'],
                                      [0x420026, 6, False],
                                      [0x42009C, 1, [[0x540001, 8, '76616c7565']]]],
                'Nonce': [[0x4200C9, 8, '6e6f6e63652d6964'],
                          [0x4200CA, 8, '6e6f6e63652d76616c7565']],
                'Protocol Version': [[0x42006A, 2, 2], [0x42006B, 2, 1]],
                'Vendor Extension': [[0x540001, 8, '76616c7565']],
            }
            return [tag_value(element), 1, copy.deepcopy(defaults.get(name, []))]
        kind_by_encoding = {'Boolean': 6, 'Byte String': 8, 'Date-Time': 9,
                            'Enumeration': 5, 'Integer': 2, 'Text String': 7}
        kind = kind_by_encoding.get(encoding or '')
        samples = {
            'Asynchronous Correlation Value': (8, '6173796e632d636f7272'),
            'Asynchronous Indicator': (5, 2),
            'Attestation Capable Indicator': (6, False),
            'Attestation Type': (5, 1),
            'Batch Count': (2, 1),
            'Batch Error Continuation Option': (5, 1),
            'Batch Order Option': (6, True),
            'Client Correlation Value': (7, '636c69656e742d636f7272'),
            'Criticality Indicator': (6, False),
            'Ephemeral': (6, False),
            'Maximum Response Size': (2, 4096),
            'Nonce ID': (8, '6e6f6e63652d6964'),
            'Nonce Value': (8, '6e6f6e63652d76616c7565'),
            'Operation': (5, 0x3B),
            'Protocol Version Major': (2, 2),
            'Protocol Version Minor': (2, 1),
            'Result Message': (7, '666978747572652d726573756c74'),
            'Result Reason': (5, 5),
            'Result Status': (5, 1),
            'Server Correlation Value': (7, '7365727665722d636f7272'),
            'Server Hashed Password': (8, '686173682d66697874757265'),
            'Time Stamp': (9, 1_234_567),
            'Unique Batch Item ID': (8, '6974656d2d31'),
            'Vendor Identification': (7, '4b4d49504b6974'),
        }
        if name in samples:
            kind, value = samples[name]
        if kind is None:
            raise RecipePending
        return [tag_value(element), kind, value]

    def path_to_root(element_id: str, root_id: str, seen: set[str] | None = None) -> list[dict] | None:
        if element_id == root_id:
            return [elements[element_id]]
        visited = seen or set()
        if element_id in visited or element_id not in elements:
            return None
        visited = {*visited, element_id}
        element = elements[element_id]
        for parent_id in element.get('parent_element_ids', []):
            path = path_to_root(parent_id, root_id, visited)
            if path is not None:
                return [*path, element]
        return None

    target_tag = None
    if target.get('kind') == 'tag':
        target_tag = int(target['wire_value'], 16)
    elif target.get('kind') == 'message_field':
        wanted_name = _normalized_name(field_name(target))
        tag_element = next((item for item in tag_elements
                            if _normalized_name(item.get('name', '')) == wanted_name), None)
        if tag_element is None:
            raise RecipePending
        target_tag = int(tag_element['wire_value'], 16)
        root_id = ('KMIPKIT-ELEM-MESSAGE-FIELD-8-1-REQUEST-MESSAGE'
                   if message_type is messages.RequestMessage
                   else 'KMIPKIT-ELEM-MESSAGE-FIELD-8-4-RESPONSE-MESSAGE')
        path = path_to_root(target['target_id'], root_id)
        if path is None:
            raise RecipePending

        def find_tree_node(node: list, raw_tag: int) -> list | None:
            if node[0] == raw_tag:
                return node
            if node[1] == 1:
                for child in node[2]:
                    found = find_tree_node(child, raw_tag)
                    if found is not None:
                        return found
            return None

        anchor_index = -1
        for index, element in enumerate(path):
            try:
                existing = find_tree_node(expected, tag_value(element))
            except RecipePending:
                existing = None
            if existing is not None:
                anchor_index = index
        if anchor_index < 0:
            raise RecipePending
        anchor = find_tree_node(expected, tag_value(path[anchor_index]))
        if anchor is None:
            raise RecipePending
        for element in path[anchor_index + 1:]:
            raw_tag = tag_value(element)
            if raw_tag == anchor[0]:
                # Sections 8 and 9 inventory the same semantic field twice.
                continue
            child = next((item for item in anchor[2] if item[0] == raw_tag), None) if anchor[1] == 1 else None
            if child is None:
                if anchor[1] != 1:
                    raise RecipePending
                child = field_node(element)
                if anchor[0] in (0x420077, 0x42007A):
                    count_index = next((index for index, item in enumerate(anchor[2])
                                        if item[0] == 0x42000D), len(anchor[2]))
                    anchor[2].insert(count_index, child)
                elif anchor[0] == 0x42000F:
                    request_item = any(item[0] == 0x420079 for item in anchor[2])
                    ranks = ({0x42005C: 0, 0x420154: 1, 0x420093: 2, 0x420079: 3,
                              0x420051: 4} if request_item else
                             {0x42005C: 0, 0x420093: 1, 0x42007F: 2, 0x42007E: 3,
                              0x42007D: 4, 0x420006: 5, 0x42007C: 6, 0x420051: 7})
                    child_rank = ranks.get(raw_tag, len(ranks))
                    position = next((index for index, item in enumerate(anchor[2])
                                     if ranks.get(item[0], len(ranks)) > child_rank), len(anchor[2]))
                    anchor[2].insert(position, child)
                else:
                    anchor[2].append(child)
            anchor = child
        name = field_name(target)
        if message_type is messages.ResponseMessage and name in {
                'Asynchronous Correlation Value', 'Result Message', 'Result Reason'}:
            response_item = find_tree_node(expected, 0x42000F)
            if response_item is not None:
                status = find_tree_node(response_item, 0x42007F)
                if status is not None:
                    status[2] = 2 if name == 'Asynchronous Correlation Value' else 1
                if name in {'Result Message', 'Result Reason'}:
                    response_item[2][:] = [item for item in response_item[2] if item[0] != 0x42007C]
                if name == 'Result Message' and not find_tree_node(response_item, 0x42007E):
                    reason = [0x42007E, 5, 5]
                    status_index = next(index for index, item in enumerate(response_item[2])
                                        if item[0] == 0x42007F)
                    response_item[2].insert(status_index + 1, reason)
        if message_type is messages.ResponseMessage and name == 'Asynchronous Correlation Value':
            response_item = find_tree_node(expected, 0x42000F)
            operation = find_tree_node(response_item, 0x42005C) if response_item else None
            if operation is not None:
                operation[2] = 0x1A  # Poll has the normative pending response shape.
            if response_item is not None:
                response_item[2][:] = [item for item in response_item[2] if item[0] != 0x42007C]
        if (message_type is messages.RequestMessage and name in {
                'Batch Error Continuation Option', 'Batch Order Option'}):
            request_header = find_tree_node(expected, 0x420077)
            batch_item = find_tree_node(expected, 0x42000F)
            if request_header is not None and batch_item is not None:
                batch_count = find_tree_node(request_header, 0x42000D)
                if batch_count is not None:
                    batch_count[2] = 2
                second_batch_item = copy.deepcopy(batch_item)
                for index, current in enumerate((batch_item, second_batch_item), start=1):
                    if not any(item[0] == 0x420093 for item in current[2]):
                        identifier = [0x420093, 8, f'6974656d2d{index:02x}']
                        payload_index = next((position for position, item in enumerate(current[2])
                                              if item[0] == 0x420079), len(current[2]))
                        current[2].insert(payload_index, identifier)
                expected[2].append(second_batch_item)
    if target_tag is None or not _credential_tree_has_tag(expected, target_tag):
        raise RecipePending
    with ExitStack() as owned:
        fields = [owned.enter_context(_credential_item_from_tree(field, owned))
                  for field in expected[2]]
        source = owned.enter_context(PythonKmipDriver._structure(fields))
        message = owned.enter_context(message_type.from_ttlv(source))
        expected_count = sum(1 for field in expected[2] if field[0] == 0x42000F)
        if message.batch_item_count != expected_count:
            raise ValueError('typed message parser changed the batch item count')
        parsed = owned.enter_context(message.to_ttlv())
        observed = owned.enter_context(PythonKmipDriver._item(root_tag, 'structure', parsed))
        actual = item_tree(observed)
    if actual != expected:
        raise ValueError('typed message parser changed the ordered TTLV tree')
    tree_hash = digest(actual)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


_BATCH_ITEM_RESULT_VALUE_ROUTES = {
    'kmipkit.client.BatchItemResult.create_object_type',
    'kmipkit.client.BatchItemResult.get_attributes_response',
    'kmipkit.client.BatchItemResult.metadata',
    'kmipkit.client.BatchItemResult.metadata_v2',
    'kmipkit.client.BatchItemResult.pending_correlation_value',
    'kmipkit.client.BatchItemResult.unique_batch_item_id',
}
_BATCH_ITEM_RESULT_RESPONSE_MEMBERS = {
    'activate_response', 'add_attribute_response', 'adjust_attribute_response',
    'archive_response', 'certify_response', 'check_response', 'decrypt_response',
    'delegated_login_response', 'delete_attribute_response', 'derive_key_response',
    'destroy_response', 'encrypt_response', 'export_response', 'get_response',
    'get_usage_allocation_response', 'hash_response', 'import_response',
    'interop_response', 'join_split_key_response', 'locate_response', 'log_response',
    'login_response', 'logout_response', 'mac_response', 'mac_verify_response',
    'modify_attribute_response', 'obtain_lease_response', 'ping_response',
    'pkcs11_response', 'query_response', 'recertify_response', 'recover_response',
    'register_response', 'rekey_key_pair_response', 'rekey_response',
    'reprovision_response', 'revoke_response', 'rng_retrieve_response',
    'rng_seed_response', 'set_attribute_response', 'sign_response',
    'signature_verify_response', 'validate_response', 'private_key_unique_identifier',
    'split_key_unique_identifiers',
}
_BATCH_ITEM_RESULT_VALUE_ROUTES.update(
    'kmipkit.client.BatchItemResult.' + member
    for member in _BATCH_ITEM_RESULT_RESPONSE_MEMBERS
)
_NATIVE_BATCH_RESULT_ROUTES = {
    'kmipkit.client.BatchItemResult.raw',
    'kmipkit.client.BatchItemResult.response_header_ttlv',
    'kmipkit.client.BatchItemResult.extension_count',
    'kmipkit.client.BatchItemResult.extension_ttlv_at',
    'kmipkit.client.BatchResult.item_count',
    'kmipkit.client.BatchResult.item_at',
}


def _native_batch_result_cell(cell: dict, case: dict, target: dict,
                              trace: CallTrace | None) -> dict:
    """Call native-backed batch response accessors on a real controlled exchange."""
    import struct
    from kmipkit import extensions, protocol_values, ttlv
    from .controlled_peer import (
        ControlledTlsPeer, PKI, batch_response, item, structure,
    )

    symbol = cell['public_api_symbol']
    if symbol.endswith('.raw'):
        operation = 'Get'
        status = protocol_values.ResultStatus.OPERATION_FAILED
        reason = protocol_values.ResultReason.OPERATION_NOT_SUPPORTED
        response_payloads = None
    elif symbol.startswith('kmipkit.client.BatchResult.'):
        operation = 'Discover Versions'
        status = protocol_values.ResultStatus.OPERATION_FAILED
        reason = protocol_values.ResultReason.OPERATION_NOT_SUPPORTED
        response_payloads = None
    else:
        operation = 'Log'
        status = protocol_values.ResultStatus.OPERATION_FAILED
        reason = protocol_values.ResultReason.OPERATION_NOT_SUPPORTED
        response_payloads = None

    extension_tree = structure(
        0x420051,
        item(0x42009D, 7, b'KMIPKit.Interop'),
        item(0x420026, 6, b'\x00' * 8),
        structure(0x42009C, item(0x540001, 2, struct.pack('>i', 7))),
    )
    response_extensions = ([extension_tree] if symbol.endswith('.extension_count')
                           or symbol.endswith('.extension_ttlv_at') else None)
    item_count = 2 if symbol.endswith('.item_count') or symbol.endswith('.item_at') else 1
    operations = [operation] * item_count
    peer = ControlledTlsPeer(
        status=status, reason=reason, encoding='TTLV', transport='raw_tls',
        protocol_version=cell['protocol_version'],
        response_builder=lambda request, encoding, minor: batch_response(
            request, encoding, operations, status, reason, minor,
            response_payloads=response_payloads,
            response_extensions=response_extensions,
        ),
    )
    connection = {
        'server_id': 'controlled-fixture', 'protocol_version': cell['protocol_version'],
        'transport': 'raw_tls', 'encoding': 'TTLV', 'api_level': 'typed-protocol',
        'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': peer.port},
                      'https': {'url': f'https://127.0.0.1:{peer.port}/kmip'}},
        'tls': {'ca_cert_path': str(PKI / 'server-cert.pem'),
                'client_cert_path': str(PKI / 'client-cert.pem'),
                'client_key_path': str(PKI / 'client-key.pem'), 'server_name_override': 'localhost',
                'connect_timeout_seconds': 5, 'read_timeout_seconds': 5},
    }
    driver = batch = limits = result = request_item = None
    request_items = []
    copied = None
    try:
        driver = PythonKmipDriver(connection)
        batch = driver._client_module.Batch.create()
        limits = ttlv.codec_limits_defaults()
        for index in range(item_count):
            if operation == 'Get':
                request_item = extensions.client_batch_item_get(
                    extensions.UniqueIdentifier.text_string('validator-get-object')
                )
            elif operation == 'Discover Versions':
                request_item = extensions.client_batch_item_discover_versions()
            else:
                request_item = extensions.client_batch_item_log('KMIPKit controlled result fixture')
            if item_count > 1:
                request_item = driver._extensions.with_unique_batch_item_id(
                    request_item, f'kmipkit-result-{index + 1:02d}'.encode('ascii')
                )
            batch.add_item(request_item)
            request_items.append(request_item)
            request_item = None
        result = driver._client.execute(batch, limits)
        expected_count = item_count
        actual_count = result.item_count()
        if actual_count != expected_count:
            raise ValueError('controlled native batch result changed its item count')
        selected_index = item_count - 1 if symbol.endswith('.item_at') else 0
        response_item = result.item_at(selected_index)
        if symbol.endswith('.item_count'):
            actual = actual_count
            if actual != 2:
                raise ValueError('BatchResult.item_count did not report both response items')
        elif symbol.endswith('.item_at'):
            actual = response_item.operation
            expected_operation = (protocol_values.Operation.DISCOVER_VERSIONS
                                  if symbol.startswith('kmipkit.client.BatchResult.')
                                  else protocol_values.Operation.LOG)
            if (actual != expected_operation
                    or response_item.result_status != status):
                raise ValueError('BatchResult.item_at did not return the ordered second result')
        elif symbol.endswith('.response_header_ttlv'):
            copied = response_item.response_header_ttlv()
            with PythonKmipDriver._item(0x42007A, 'structure', copied) as wrapped:
                tree = item_tree(wrapped)
            if tree[0] != 0x42007A or not any(child[0] == 0x42000D for child in tree[2]):
                raise ValueError('response header TTLV omitted its root or Batch Count')
            actual = tree
        elif symbol.endswith('.raw'):
            copied = response_item.raw()
            if copied is not None:
                raise ValueError('failed Get response unexpectedly exposed a Response Payload')
            actual = None
        elif symbol.endswith('.extension_count'):
            actual = response_item.extension_count
            if actual != 1:
                raise ValueError('response extension count did not preserve the received extension')
        else:
            copied = response_item.extension_ttlv_at(0)
            with PythonKmipDriver._item(0x420051, 'structure', copied) as wrapped:
                actual = item_tree(wrapped)
            if actual != [0x420051, 1, [
                    [0x42009D, 7, b'KMIPKit.Interop'.hex()],
                    [0x420026, 6, False],
                    [0x42009C, 1, [[0x540001, 2, 7]]],
            ]]:
                raise ValueError('response extension TTLV changed its ordered fields')
        if trace is not None:
            trace.record_invocation(symbol)
        if len(peer.requests) != 1 or len(peer.responses) != 1:
            raise ValueError('native response accessor fixture did not exchange one message')
        digest_value = digest({'accessor': symbol, 'target': target.get('target_id'),
                               'observed': actual})
        return {'request_tree_sha256': digest_value, 'response_tree_sha256': digest_value,
                'wire_bytes_sha256': hashlib.sha256(peer.requests[0]).hexdigest(),
                'request_delivery_state': 'response_received', 'transport_invocation_count': 1,
                'peer_observed_response_count': item_count,
                'peer_observed_result_status_codes': [status] * item_count}
    finally:
        if copied is not None:
            copied.close()
        if request_item is not None:
            request_item.close()
        for request_item in request_items:
            request_item.close()
        if result is not None:
            result.close()
        if limits is not None:
            limits.close()
        if batch is not None:
            batch.close()
        if driver is not None:
            driver.close()
        peer.close()
_TYPED_RESPONSE_TTLV_ROUTES = {
    'kmipkit.client.' + response_type + '.' + member
    for response_type in (
        'CertifyResponse', 'CheckResponse', 'DeriveKeyResponse', 'ExportResponse',
        'GetAttributeListResponse', 'GetConstraintsResponse', 'GetUsageAllocationResponse',
        'HashResponse', 'ImportResponse', 'JoinSplitKeyResponse', 'MacResponse',
        'MacVerifyResponse', 'ObtainLeaseResponse', 'ReProvisionResponse', 'RecertifyResponse',
        'RegisterResponse', 'RekeyKeyPairResponse', 'RekeyResponse', 'RevokeResponse',
        'RngRetrieveResponse', 'RngSeedResponse', 'SetConstraintsResponse',
        'SetDefaultsResponse', 'SignResponse', 'SignatureVerifyResponse', 'ValidateResponse',
    )
    for member in ('response_ttlv', 'response_payload_ttlv')
}
_QUERY_RESPONSE_FIELD_ROUTES = {
    'kmipkit.client.QueryResponse.field_at',
    'kmipkit.client.QueryResponse.field_count',
}
_RESPONSE_DATA_FIELD_ROUTES = {
    'kmipkit.client.LocateResponse.unique_identifiers',
    'kmipkit.client.Pkcs11Response.correlation_value',
    'kmipkit.client.Pkcs11Response.function_ttlv',
    'kmipkit.client.Pkcs11Response.interface_ttlv',
    'kmipkit.client.Pkcs11Response.output_parameters_ttlv',
    'kmipkit.client.Pkcs11Response.return_code_ttlv',
}
_TICKET_TTLV_ROUTES = {
    'kmipkit.client.DelegatedLoginResponse.ticket_ttlv',
    'kmipkit.client.LoginResponse.ticket_ttlv',
}
_GET_ATTRIBUTE_LIST_RESPONSE_ROUTE = 'kmipkit.client.GetAttributeListResponse'
_BATCH_ERROR_CONTINUATION_ROUTE = 'kmipkit.client.Batch.add_error_continuation_value'
_ASYNC_OUTCOME_RESPONSE_ROUTES = {
    'kmipkit.client.AsyncOperationOutcome.cancel_response',
    'kmipkit.client.AsyncOperationOutcome.poll_response',
    'kmipkit.client.AsyncOperationOutcome.process_response',
    'kmipkit.client.AsyncOperationOutcome.query_async_requests_response',
    'kmipkit.client.AsyncOperationOutcome.response_payload_ttlv',
}
_BATCH_ITEM_RESULT_VALUE_ROUTES.update(
    'kmipkit.client.BatchItemResult.' + member
    for member in ('private_key_unique_identifier', 'split_key_unique_identifiers')
)


def _batch_item_result_value_cell(cell: dict, case: dict, target: dict,
                                  trace: CallTrace | None) -> dict:
    """Invoke copied BatchItemResult fields and metadata against deterministic values."""
    from kmipkit import client

    symbol = cell['public_api_symbol']
    if target.get('kind') == 'message_field':
        target_id = target.get('target_id', '')
        direction = _catalog_model_data()[2].get(target_id, {}).get('direction')
        if (any(section in target_id for section in ('-8-2-', '-8-3-'))
                or direction == 'client_to_server'):
            message_symbol = 'kmipkit.messages.RequestMessage.from_ttlv'
        elif any(section in target_id for section in ('-8-5-', '-8-6-')):
            message_symbol = 'kmipkit.messages.ResponseMessage.from_ttlv'
        else:
            message_symbol = 'kmipkit.messages.ResponseMessage.from_ttlv'
        fixture = _typed_message_cell(
            {'public_api_symbol': message_symbol, 'transport': None}, case, target)
    else:
        fixture = _generic_cell(
            {'action': 'construct', 'public_api_symbol': 'kmipkit.ttlv.ttlv_item_create',
             'transport': None}, case, target)

    operation_name = _cell_operation(cell, case)
    _, _, elements = _catalog_model_data()
    operation_value = next((int(element['wire_value'].replace('X', '0'), 16)
                            for element in elements.values()
                            if element.get('kind') == 'enumeration_value'
                            and _normalized_name(element.get('name', '')) ==
                            _normalized_name(operation_name)
                            and any(elements.get(parent, {}).get('name') == 'Operation'
                                    for parent in element.get('parent_element_ids', []))), None)
    if operation_value is None and target.get('kind') != 'operation':
        operation_value = next((int(element['wire_value'].replace('X', '0'), 16)
                                for element in elements.values()
                                if element.get('kind') == 'enumeration_value'
                                and element.get('name') == 'Ping'
                                and any(elements.get(parent, {}).get('name') == 'Operation'
                                        for parent in element.get('parent_element_ids', []))), None)
    if operation_value is None and target.get('kind') == 'message_field':
        operation_value = next((int(element['wire_value'].replace('X', '0'), 16)
                                for element in elements.values()
                                if element.get('kind') == 'enumeration_value'
                                and element.get('name') == 'Ping'
                                and any(elements.get(parent, {}).get('name') == 'Operation'
                                        for parent in element.get('parent_element_ids', []))), None)
    if operation_value is None:
        raise RecipePending
    if symbol.endswith('.get_attributes_response'):
        return _get_attributes_response_cell(cell, case, target, trace, fixture, operation_value)
    is_pending = symbol.endswith('.pending_correlation_value')
    outcome = client.BatchOutcome.PENDING if is_pending else client.BatchOutcome.COMPLETED
    status = 2 if is_pending else 0
    identifier = b'\x00\xffbatch-item'
    correlation = b'\x80pending-correlation' if is_pending else None
    object_type = 2 if symbol.endswith('.create_object_type') else None
    result = client.BatchItemResult(
        outcome=outcome, result_status=status, result_reason=None,
        unique_batch_item_id=identifier, pending_correlation_value=correlation,
        operation=operation_value, create_object_type=object_type)
    if symbol.endswith('.metadata'):
        if result.metadata() != (int(outcome), status, 0, 0):
            raise ValueError('BatchItemResult metadata changed copied result fields')
    elif symbol.endswith('.metadata_v2'):
        metadata = result.metadata_v2
        if (metadata.operation != operation_value or metadata.outcome != outcome
                or metadata.result_status != status or metadata.result_reason is not None
                or metadata.result_message is not None):
            raise ValueError('BatchItemResult v2 metadata changed copied result fields')
    elif symbol.endswith('.unique_batch_item_id'):
        if result.unique_batch_item_id != identifier:
            raise ValueError('BatchItemResult unique ID changed its bytes')
        if trace is not None:
            trace.record_invocation(symbol)
    elif symbol.endswith('.pending_correlation_value'):
        if result.pending_correlation_value != correlation:
            raise ValueError('BatchItemResult correlation changed its bytes')
        if trace is not None:
            trace.record_invocation(symbol)
    elif symbol.endswith('.create_object_type'):
        if result.create_object_type != 2:
            raise ValueError('BatchItemResult Create Object Type changed its raw value')
        if trace is not None:
            trace.record_invocation(symbol)
    elif symbol.rsplit('.', 1)[-1] in _BATCH_ITEM_RESULT_RESPONSE_MEMBERS:
        return _batch_typed_response_accessor_cell(
            cell, target, trace, fixture, operation_value, result)
    else:
        raise RecipePending
    return {'request_tree_sha256': fixture['request_tree_sha256'],
            'response_tree_sha256': digest({'operation': operation_value,
                                            'outcome': int(outcome), 'status': status,
                                            'reason': None, 'unique_id': identifier.hex(),
                                            'correlation': (correlation.hex()
                                                            if correlation is not None else None),
                                            'object_type': object_type})}


def _get_attributes_response_cell(cell: dict, case: dict, target: dict,
                                  trace: CallTrace | None, fixture: dict,
                                  operation_value: int) -> dict:
    """Read the public typed Get Attributes response value and compare its TTLV payload."""
    from kmipkit import client

    attributes_tree = [[0x420125, 1, [
        [0x42002F, 9, 19], [0x4200FD, 7, '4b4d49504b697420617474726962757465'],
    ]]]
    unique_identifier = 'interop-object-7'
    response_payload = [0x42007C, 1, [
        [0x420094, 7, unique_identifier.encode().hex()], *attributes_tree,
    ]]
    if not _credential_tree_has_tag(response_payload, 0x420094):
        raise ValueError('Get Attributes response fixture omitted its Unique Identifier')
    if not _credential_tree_has_tag(response_payload, 0x420125):
        raise ValueError('Get Attributes response fixture omitted its Attributes')
    with ExitStack() as owned:
        attributes = _credential_structure_from_children(attributes_tree, owned)
        response = client.GetAttributesResponse(unique_identifier, attributes)
        owned.callback(response.close)
        result = client.BatchItemResult(
            outcome=client.BatchOutcome.COMPLETED, result_status=0, result_reason=None,
            unique_batch_item_id=b'get-attributes-item', pending_correlation_value=None,
            operation=operation_value, get_attributes_response=response)
        actual = result.get_attributes_response
        if actual is not response or actual.unique_identifier != unique_identifier:
            raise ValueError('Get Attributes accessor changed its typed Unique Identifier')
        if actual.attributes is None:
            raise ValueError('Get Attributes accessor dropped the Attributes structure')
        from kmipkit import ttlv
        with ttlv.ttlv_structure_view(actual.attributes) as view:
            if ttlv.ttlv_structure_view_item_count(view) != len(attributes_tree):
                raise ValueError('Get Attributes accessor changed the attribute count')
        observed_item = owned.enter_context(_credential_item_from_tree(response_payload, owned))
        observed = item_tree(observed_item)
        if observed != response_payload:
            raise ValueError('Get Attributes response TTLV changed its ordered fields')
    if trace is not None:
        trace.record_invocation(cell['public_api_symbol'])
    return {'request_tree_sha256': fixture['request_tree_sha256'],
            'response_tree_sha256': digest(observed)}


def _batch_typed_response_accessor_cell(cell: dict, target: dict,
                                        trace: CallTrace | None, fixture: dict,
                                        operation_value: int, base_result) -> dict:
    """Build and read a declared public response wrapper without exposing its values."""
    from dataclasses import replace
    import inspect
    from kmipkit import client, extensions

    member = cell['public_api_symbol'].rsplit('.', 1)[-1]
    private_member = {
        'get_response': '_get_response', 'locate_response': '_locate_response',
        'rng_retrieve_response': '_rng_retrieve_response',
        'rng_seed_response': '_rng_seed_response',
    }.get(member, member)
    response_type_name = {
        'rng_retrieve_response': 'RngRetrieveResponse',
        'rng_seed_response': 'RngSeedResponse',
    }.get(member)
    if response_type_name is None and member not in {
            'logout_response', 'ping_response', 'query_response',
            'private_key_unique_identifier', 'public_key_unique_identifier',
            'split_key_unique_identifiers'}:
        annotation = client.BatchItemResult.__annotations__.get(private_member)
        if annotation is None:
            raise RecipePending
        response_type_name = str(annotation).split('|', 1)[0].strip()

    with ExitStack() as owned:
        response_type = getattr(client, response_type_name, None) if response_type_name else None
        if response_type is not None:
            arguments = {}
            for name, parameter in inspect.signature(response_type).parameters.items():
                annotation = str(parameter.annotation)
                if parameter.default is not inspect.Parameter.empty:
                    continue
                if name in {'unique_identifier', 'private_key_unique_identifier',
                            'public_key_unique_identifier'}:
                    normalized_annotation = annotation.strip("'\"")
                    arguments[name] = ('interop-object-7' if normalized_annotation == 'str'
                                       else extensions.UniqueIdentifier.text_string('interop-object-7'))
                elif name in {'response_ttlv', 'response_payload_ttlv'}:
                    arguments[name] = _credential_structure_from_children([], owned)
                elif name in {'attributes', 'any_object', 'object'}:
                    arguments[name] = None
                elif name in {'data', 'signature_data', 'mac_data', 'recovered_data',
                              'correlation_value', 'iv_counter_nonce',
                              'authenticated_encryption_tag'}:
                    arguments[name] = (None if '_SecretBytes' in annotation
                                       else b'\x00\xfffixture')
                elif name in {'validity_indicator_raw', 'object_type_raw', 'located_items',
                              'lease_time', 'last_change_date', 'usage_limits_count',
                              'cryptographic_usage_mask', 'data_length', 'ticket_type_raw'}:
                    arguments[name] = 2 if name == 'object_type_raw' else 1
                elif name == 'unique_identifiers':
                    arguments[name] = (extensions.UniqueIdentifier.text_string('interop-object-7'),)
                elif name in {'_ticket_value', 'ticket_ttlv'}:
                    arguments[name] = None
                else:
                    raise RecipePending
            value = response_type(**arguments)
            if callable(getattr(value, 'close', None)):
                owned.callback(value.close)
        elif member in {'logout_response', 'ping_response'}:
            value = True
        elif member == 'query_response':
            value = _credential_structure_from_children([], owned)
        elif member in {'private_key_unique_identifier', 'public_key_unique_identifier'}:
            value = extensions.UniqueIdentifier.text_string('interop-object-7')
        elif member == 'split_key_unique_identifiers':
            value = (extensions.UniqueIdentifier.text_string('interop-object-7'),)
        else:
            raise RecipePending

        if member in {'rng_retrieve_response', 'rng_seed_response'}:
            result = base_result
            object.__setattr__(result, private_member, value)
        else:
            result = replace(base_result, **{private_member: value})
        accessor = getattr(result, member)
        actual = accessor() if callable(accessor) else accessor
        if actual is not value:
            raise ValueError('BatchItemResult accessor changed the typed response value')
        if response_type is not None and not isinstance(actual, response_type):
            raise ValueError('BatchItemResult accessor returned a different response type')
        # Most BatchItemResult responses are dataclass fields and therefore have
        # no Python function body for CallTrace to observe. Reaching this exact
        # read is the invocation evidence for those public data members.
        if trace is not None:
            trace.record_invocation(cell['public_api_symbol'])
        response_digest = digest({'member': member, 'operation': operation_value,
                                  'response_type': response_type_name, 'present': True,
                                  'target': target.get('target_id')})
    return {'request_tree_sha256': fixture['request_tree_sha256'],
            'response_tree_sha256': response_digest}


def _typed_response_ttlv_cell(cell: dict, target: dict,
                              trace: CallTrace | None) -> dict:
    """Construct a typed response and verify its public raw TTLV structure accessor."""
    import inspect
    from kmipkit import client, extensions, ttlv

    symbol = cell['public_api_symbol']
    _, response_name, member = symbol.rsplit('.', 2)
    response_type = getattr(client, response_name)
    expected_children = [[0x420094, 7, '696e7465726f702d6f626a6563742d37']]

    def view_tree(view):
        with ttlv.ttlv_item_view_tag(view) as tag:
            raw_tag = ttlv.tag_value(tag)
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
        return [raw_tag, kind, result]

    with ExitStack() as owned:
        structure = _credential_structure_from_children(expected_children, owned)
        arguments = {}
        for name, parameter in inspect.signature(response_type).parameters.items():
            if parameter.default is not inspect.Parameter.empty:
                continue
            annotation = str(parameter.annotation).strip("'\"")
            if name in {'response_ttlv', 'response_payload_ttlv'}:
                arguments[name] = structure
            elif name in {'unique_identifier', 'private_key_unique_identifier',
                          'public_key_unique_identifier'}:
                arguments[name] = ('interop-object-7' if annotation == 'str'
                                   else extensions.UniqueIdentifier.text_string('interop-object-7'))
            elif name == 'attribute_references':
                arguments[name] = ()
            elif name in {'attributes', 'any_object', 'object', '_ticket_value', 'ticket_ttlv'}:
                arguments[name] = None
            elif name in {'data', 'signature_data', 'mac_data', 'recovered_data',
                          'correlation_value', 'iv_counter_nonce',
                          'authenticated_encryption_tag'}:
                arguments[name] = (None if '_SecretBytes' in annotation
                                   else b'\x00\xfffixture')
            elif name in {'validity_indicator_raw', 'object_type_raw', 'located_items',
                          'lease_time', 'last_change_date', 'usage_limits_count',
                          'cryptographic_usage_mask', 'data_length', 'ticket_type_raw'}:
                arguments[name] = 2 if name == 'object_type_raw' else 1
            elif name == 'unique_identifiers':
                arguments[name] = (extensions.UniqueIdentifier.text_string('interop-object-7'),)
            else:
                raise RecipePending

        value = response_type(**arguments)
        if callable(getattr(value, 'close', None)):
            owned.callback(value.close)
        actual = getattr(value, member)
        actual = actual() if callable(actual) else actual
        if not isinstance(actual, ttlv.Structure):
            raise ValueError('typed response TTLV accessor returned a non-Structure value')
        with ttlv.ttlv_structure_view(actual) as structure_view:
            observed_children = []
            for index in range(ttlv.ttlv_structure_view_item_count(structure_view)):
                with ttlv.ttlv_structure_view_item_at(structure_view, index) as child:
                    observed_children.append(view_tree(child))
        if observed_children != expected_children:
            raise ValueError('typed response TTLV accessor changed its ordered structure')
        if trace is not None:
            trace.record_invocation(symbol)
        response_digest = digest({'response_type': response_name, 'member': member,
                                  'children': observed_children, 'target': target.get('target_id')})
    return {'request_tree_sha256': response_digest,
            'response_tree_sha256': response_digest}


def _response_data_field_cell(cell: dict, target: dict,
                              trace: CallTrace | None) -> dict:
    """Read ordered Locate identifiers or copied PKCS#11 response fields."""
    from kmipkit import client, extensions

    symbol = cell['public_api_symbol']
    member = symbol.rsplit('.', 1)[-1]
    identifier = extensions.UniqueIdentifier.text_string('interop-object-7')
    if symbol.startswith('kmipkit.client.LocateResponse.'):
        identifiers = (identifier, extensions.UniqueIdentifier.text_string('interop-object-8'))
        response = client.LocateResponse(len(identifiers), identifiers)
        observed = getattr(response, member)
        if observed is not identifiers or len(observed) != 2 or observed[1].text != 'interop-object-8':
            raise ValueError('LocateResponse changed the ordered Unique Identifier values')
        response_digest = digest({'count': len(observed), 'identifiers': ['first', 'second']})
    else:
        from kmipkit import ttlv

        child = [0x420094, 7, '696e7465726f702d6f626a6563742d37']
        structure_members = ('response_ttlv', 'interface_ttlv', 'function_ttlv',
                             'output_parameters_ttlv', 'return_code_ttlv')
        correlation = b'\x00\xfffixture'
        with ExitStack() as owned:
            structures = {name: _credential_structure_from_children([child], owned)
                          for name in structure_members}
            response = client.Pkcs11Response(
                response_ttlv=structures['response_ttlv'],
                interface_ttlv=structures['interface_ttlv'],
                function_ttlv=structures['function_ttlv'],
                correlation_value=correlation,
                output_parameters_ttlv=structures['output_parameters_ttlv'],
                return_code_ttlv=structures['return_code_ttlv'])
            owned.callback(response.close)
            observed = getattr(response, member)
            if member == 'correlation_value':
                if observed != correlation:
                    raise ValueError('PKCS#11 response changed its correlation bytes')
                response_digest = digest({'member': member, 'length': len(observed),
                                          'sha256': hashlib.sha256(observed).hexdigest()})
            else:
                if not isinstance(observed, ttlv.Structure) or observed is not structures[member]:
                    raise ValueError('PKCS#11 response changed its generic TTLV structure')
                with ttlv.ttlv_structure_view(observed) as view:
                    if ttlv.ttlv_structure_view_item_count(view) != 1:
                        raise ValueError('PKCS#11 response changed its structure item count')
                response_digest = digest({'member': member, 'item_count': 1,
                                          'tag': child[0], 'type': child[1]})
    if trace is not None:
        trace.record_invocation(symbol)
    request_digest = digest({'case': cell['case_id'], 'target': target.get('target_id')})
    return {'request_tree_sha256': request_digest,
            'response_tree_sha256': response_digest}


def _ticket_ttlv_cell(cell: dict, target: dict,
                      trace: CallTrace | None) -> dict:
    """Read the optional ticket TTLV while keeping ticket value bytes absent."""
    from kmipkit import client, ttlv

    _, response_name, member = cell['public_api_symbol'].rsplit('.', 2)
    response_type = getattr(client, response_name)
    with ExitStack() as owned:
        ticket = _credential_structure_from_children([], owned)
        value = response_type(ticket_type_raw=1, _ticket_value=None, ticket_ttlv=ticket)
        if callable(getattr(value, 'close', None)):
            owned.callback(value.close)
        observed = getattr(value, member)
        if observed is not ticket or not isinstance(observed, ttlv.Structure):
            raise ValueError('typed login response changed its ticket TTLV handle')
        with ttlv.ttlv_structure_view(observed) as view:
            if ttlv.ttlv_structure_view_item_count(view) != 0:
                raise ValueError('empty ticket TTLV fixture acquired unexpected items')
        if trace is not None:
            trace.record_invocation(cell['public_api_symbol'])
        response_digest = digest({'response_type': response_name, 'ticket_ttlv_items': 0,
                                 'target': target.get('target_id')})
    return {'request_tree_sha256': response_digest,
            'response_tree_sha256': response_digest}


def _get_attribute_list_response_constructor_cell(cell: dict, target: dict,
                                                  trace: CallTrace | None) -> dict:
    """Construct Get Attribute List response metadata and retain its generic payload."""
    from kmipkit import client, extensions

    unique_identifier = extensions.UniqueIdentifier.text_string('interop-object-7')
    with ExitStack() as owned:
        payload = _credential_structure_from_children(
            [[0x420094, 7, '696e7465726f702d6f626a6563742d37']], owned)
        response = client.GetAttributeListResponse(unique_identifier, (), payload)
        owned.callback(response.close)
        if (response.unique_identifier is not unique_identifier
                or response.attribute_references != ()
                or response.response_payload_ttlv is not payload):
            raise ValueError('Get Attribute List response constructor changed its fields')
        if trace is not None:
            trace.record_invocation(cell['public_api_symbol'])
        response_digest = digest({'unique_identifier_kind': int(unique_identifier.kind),
                                  'attribute_reference_count': 0, 'payload_item_count': 1,
                                  'target': target.get('target_id')})
    return {'request_tree_sha256': response_digest,
            'response_tree_sha256': response_digest}


def _batch_error_continuation_cell(cell: dict, case: dict, target: dict,
                                  trace: CallTrace | None) -> dict:
    """Append the exact catalogued Batch Error Continuation Option to a public Batch."""
    from kmipkit import client

    wire_value = _target_field(target, case.get('requires', {}), 'wire_value')
    if wire_value is None and target.get('kind') == 'message_field':
        _, _, elements = _catalog_model_data()
        enum_parent_ids = {element_id for element_id, element in elements.items()
                           if element.get('kind') == 'enumeration'
                           and _normalized_name(element.get('name', ''))
                           == _normalized_name(target.get('name', ''))}
        value_element = next((element for element in elements.values()
                              if element.get('kind') == 'enumeration_value'
                              and _normalized_name(element.get('name', '')) == 'continue'
                              and any(parent in enum_parent_ids
                                      for parent in element.get('parent_element_ids', []))), None)
        if value_element is not None:
            wire_value = value_element.get('wire_value')
    if not isinstance(wire_value, str):
        raise RecipePending
    try:
        value = int(wire_value.replace('X', '0'), 16)
    except ValueError as error:
        raise RecipePending from error
    with client.Batch.create() as batch:
        if batch.add_error_continuation_value(value) is not batch:
            raise ValueError('Batch option builder did not preserve its Batch handle')
        if batch.item_count != 0:
            raise ValueError('Batch option builder unexpectedly appended a batch item')
        if trace is not None:
            trace.record_invocation(cell['public_api_symbol'])
        response_digest = digest({'batch_error_continuation_option': value,
                                  'item_count': 0, 'target': target.get('target_id')})
    return {'request_tree_sha256': response_digest,
            'response_tree_sha256': response_digest}


def _async_outcome_response_cell(cell: dict, case: dict, target: dict,
                                 trace: CallTrace | None) -> dict:
    """Create one asynchronous outcome through the mutually authenticated test peer."""
    from .controlled_peer import ControlledTlsPeer, PKI
    from .driver import PythonKmipDriver
    from kmipkit import protocol_values, ttlv

    symbol = cell['public_api_symbol']
    operation = _cell_operation(cell, case)
    member = symbol.rsplit('.', 1)[-1]
    operation_for_member = {
        'cancel_response': 'Cancel', 'poll_response': 'Poll',
        'process_response': 'Process',
        'query_async_requests_response': 'Query Asynchronous Requests',
    }
    if member in operation_for_member:
        operation = operation_for_member[member]
    scenario = case.get('requires', {}).get('protocol_scenario', {})
    scenario_operations = scenario.get('operations', [])
    if operation not in {'Cancel', 'Poll', 'Process', 'Query Asynchronous Requests'}:
        operation = next((candidate for candidate in scenario_operations
                          if candidate in {'Cancel', 'Poll', 'Process',
                                           'Query Asynchronous Requests'}), 'Poll')
    parameters = scenario.get('parameters', {})
    cancellation_result = None
    cancellation_name = parameters.get('cancel_result')
    if operation == 'Cancel' and cancellation_name:
        _, _, elements = _catalog_model_data()
        enum_target_ids = scenario.get('target_ids', [])
        cancellation_element = next((elements[target_id] for target_id in enum_target_ids
                                     if target_id in elements
                                     and elements[target_id].get('kind') == 'enumeration_value'
                                     and _normalized_name(elements[target_id].get('name', ''))
                                     == _normalized_name(cancellation_name)), None)
        if cancellation_element is None:
            raise RecipePending
        cancellation_result = int(cancellation_element['wire_value'].replace('X', '0'), 16)
    peer_status = (protocol_values.ResultStatus.SUCCESS if cancellation_result is not None
                   else protocol_values.ResultStatus.OPERATION_FAILED)
    peer_reason = (None if cancellation_result is not None
                   else protocol_values.ResultReason.OPERATION_NOT_SUPPORTED)
    correlation = b'kmipkit-validator-correlation'
    protocol_version = cell['protocol_version']
    peer = ControlledTlsPeer(
        status=peer_status, reason=peer_reason, encoding='TTLV', transport='raw_tls',
        protocol_version=protocol_version, cancellation_result=cancellation_result,
        asynchronous_correlation_value=(correlation if cancellation_result is not None else None))
    connection = {
        'server_id': 'controlled-fixture', 'protocol_version': protocol_version,
        'transport': 'raw_tls', 'encoding': 'TTLV', 'api_level': 'typed-protocol',
        'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': peer.port},
                      'https': {'url': f'https://127.0.0.1:{peer.port}/kmip'}},
        'tls': {'ca_cert_path': str(PKI / 'server-cert.pem'),
                'client_cert_path': str(PKI / 'client-cert.pem'),
                'client_key_path': str(PKI / 'client-key.pem'),
                'server_name_override': 'localhost',
                'connect_timeout_seconds': 5, 'read_timeout_seconds': 5},
    }
    driver = None
    limits = None
    outcome = None
    try:
        driver = PythonKmipDriver(connection)
        limits = ttlv.codec_limits_defaults()
        if operation == 'Cancel':
            outcome = driver._client.execute_cancel(correlation, limits)
        elif operation == 'Poll':
            outcome = driver._client.execute_poll(correlation, limits)
        elif operation == 'Process':
            outcome = driver._client.execute_process(
                correlation, limits, asynchronous_indicator=1)
        else:
            outcome = driver._client.execute_query_async_requests(limits)

        metadata = outcome.metadata_v1
        if (int(metadata.result_status) != peer_status or metadata.result_reason != peer_reason):
            raise ValueError('controlled asynchronous response status changed')
        if symbol.endswith('.response_payload_ttlv'):
            payload = outcome.response_payload_ttlv
            if cancellation_result is None:
                if payload is not None:
                    payload.close()
                    raise ValueError('unsupported asynchronous response unexpectedly included a payload')
                observed = {'payload_present': False}
            else:
                if payload is None:
                    raise ValueError('successful Cancel response omitted its payload')
                try:
                    with ttlv.ttlv_structure_view(payload) as payload_view:
                        if ttlv.ttlv_structure_view_item_count(payload_view) != 2:
                            raise ValueError('Cancel response payload item count changed')
                        with ttlv.ttlv_structure_view_item_at(payload_view, 0) as item_view:
                            with ttlv.ttlv_item_view_tag(item_view) as tag_view:
                                raw_tag = ttlv.tag_value(tag_view)
                            if (raw_tag != 0x420006
                                    or int(ttlv.ttlv_item_view_type(item_view)) != 8):
                                raise ValueError('Cancel response correlation field changed')
                            with ttlv.ttlv_item_view_value(item_view) as item_value:
                                byte_count = ttlv.ttlv_value_view_byte_length(item_value)
                                if byte_count != len(correlation):
                                    raise ValueError('Cancel response correlation length changed')
                                for index, expected_byte in enumerate(correlation):
                                    if ttlv.ttlv_value_view_byte_at(item_value, index) != expected_byte:
                                        raise ValueError('Cancel response correlation value changed')
                        with ttlv.ttlv_structure_view_item_at(payload_view, 1) as item_view:
                            with ttlv.ttlv_item_view_tag(item_view) as tag_view:
                                raw_tag = ttlv.tag_value(tag_view)
                            if int(ttlv.ttlv_item_view_type(item_view)) != 5:
                                raise ValueError('Cancellation Result changed its TTLV type')
                            with ttlv.ttlv_item_view_value(item_view) as item_value:
                                raw_value = ttlv.ttlv_value_view_enumeration(item_value)
                    if raw_tag != 0x420012 or raw_value != cancellation_result:
                        raise ValueError('Cancel response payload changed its Cancellation Result')
                finally:
                    payload.close()
                observed = {'payload_present': True,
                            'cancellation_result': cancellation_result}
        else:
            response = getattr(outcome, member)()
            expected_type = {
                'cancel_response': 'AsyncCancelResponse', 'poll_response': 'AsyncPollResponse',
                'process_response': 'AsyncProcessResponse',
                'query_async_requests_response': 'AsyncQueryAsyncRequestsResponse',
            }[member]
            if response is None or type(response).__name__ != expected_type:
                raise ValueError('asynchronous response accessor returned an unexpected type')
            if (int(response.result.result_status) != peer_status
                    or response.result.result_reason != peer_reason):
                raise ValueError('typed asynchronous response changed its result metadata')
            observed = {'response_type': expected_type,
                        'result_status': int(response.result.result_status),
                        'result_reason': (None if response.result.result_reason is None
                                          else int(response.result.result_reason))}
            if member == 'cancel_response' and cancellation_result is not None:
                if response.cancellation_result != cancellation_result:
                    raise ValueError('typed Cancel response changed its Cancellation Result')
                echoed = bytearray()
                if (not response.with_echoed_correlation_value(
                        lambda value: echoed.extend(value))
                        or echoed != correlation):
                    raise ValueError('typed Cancel response changed its echoed correlation')
                observed['cancellation_result'] = response.cancellation_result
        if trace is not None:
            trace.record_invocation(symbol)
        request = _network_tree(peer.requests[0], 'TTLV') if len(peer.requests) == 1 else []
        response_tree = _network_tree(peer.responses[0], 'TTLV') if len(peer.responses) == 1 else []
        if (not request or not response_tree
                or _normalized_name(str(peer.request_observation.get('operation', '')))
                != _normalized_name(operation)):
            raise ValueError('controlled asynchronous peer did not observe the requested operation')
        request_digest = digest(request)
        response_digest = digest({'tree': response_tree, 'accessor': observed,
                                  'target': target.get('target_id')})
    finally:
        if outcome is not None:
            outcome.close()
        if limits is not None:
            limits.close()
        if driver is not None:
            driver.close()
        peer.close()
    return {'request_tree_sha256': request_digest,
            'response_tree_sha256': response_digest,
            'request_delivery_state': 'response_received',
            'transport_invocation_count': 1, 'peer_observed_response_count': 1}


def _query_response_field_cell(cell: dict, target: dict,
                               trace: CallTrace | None) -> dict:
    """Read QueryResponse field count and indexed values in their original order."""
    from kmipkit import client

    expected = ((0x42005C, 1), (0x420057, 2))
    with ExitStack() as owned:
        payload = _credential_structure_from_children(
            [[tag, 5, value] for tag, value in expected], owned)
        fields = [client._QueryFieldData(tag, client.QueryFieldKind.ENUMERATION, value)
                  for tag, value in expected]
        response = client.QueryResponse(payload, fields)
        owned.callback(response.close)

        symbol = cell['public_api_symbol']
        if symbol.endswith('.field_count'):
            observed_count = response.field_count
            if observed_count != len(expected):
                raise ValueError('QueryResponse field count changed the payload field count')
            observed = {'count': observed_count}
        else:
            field = response.field_at(1)
            if (field.index != 1 or field.tag != expected[1][0]
                    or field.kind is not client.QueryFieldKind.ENUMERATION
                    or field.enumeration != expected[1][1]):
                raise ValueError('QueryResponse field_at changed ordered field metadata')
            observed = {'index': field.index, 'tag': field.tag,
                        'kind': int(field.kind), 'enumeration': field.enumeration}
        if trace is not None:
            trace.record_invocation(symbol)
        request_digest = digest({'fields': expected, 'target': target.get('target_id')})
        response_digest = digest(observed)
    return {'request_tree_sha256': request_digest,
            'response_tree_sha256': response_digest}


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
_EXECUTION_ROUTE_OPERATIONS = {
    'python.execute.cancel': 'Cancel',
    'python.execute.poll': 'Poll',
    'python.execute.process': 'Process',
    'python.execute.queryasyncrequests': 'Query Asynchronous Requests',
}


def _cell_operation(cell: dict, case: dict) -> str:
    return (cell.get('workflow_operation')
            or _EXECUTION_ROUTE_OPERATIONS.get(cell.get('execution_route_id'), case['operation']))


def execute_cell(cell: dict, case: dict, target: dict, *, trace: CallTrace | None = None) -> dict:
    if cell['transport'] is None:
        if (target.get('kind') == 'operation_structure'
                and cell['public_api_symbol'] in {
                    'kmipkit.ttlv.ttlv_structure_create',
                    'kmipkit.ttlv.ttlv_structure_with_item'}):
            return _operation_structure_ttlv_cell(cell, case, target, trace)
        if (cell['public_api_symbol'] == 'kmipkit.ttlv.ttlv_item_view_from_item'
                and target.get('target_id') == 'KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES'):
            return _certificate_attributes_ttlv_cell(cell, target, trace)
        if (target.get('target_id') in {
                'KMIPKIT-PAYLOAD-SERVER-PUT-REQUEST-20-004',
                'KMIPKIT-PAYLOAD-SERVER-PUT-REQUEST-21-004'}
                and cell['public_api_symbol'] in {
                    'kmipkit.ttlv.ttlv_decode', 'kmipkit.ttlv.ttlv_item_create',
                    'kmipkit.ttlv.ttlv_item_view_from_item'}):
            return _server_put_ttlv_cell(cell, target, trace)
        if cell['public_api_symbol'] == 'kmipkit.client.BatchItemResult.supported_versions':
            return _positive_fixture_cell(cell, case, target, trace=trace)
        if cell['public_api_symbol'] in _NATIVE_BATCH_RESULT_ROUTES:
            return _native_batch_result_cell(cell, case, target, trace)
        if cell['public_api_symbol'].startswith('kmipkit.credentials.'):
            return _credential_cell(cell, case, target)
        if cell['public_api_symbol'].startswith('kmipkit.async_operations.'):
            return _async_request_cell(cell, case, target)
        # AttributeSet lives in attribute_operations but exercises typed model
        # behavior; resolve exact route ownership before module-wide dispatch.
        if cell['public_api_symbol'] in _TYPED_ATTRIBUTE_SYMBOLS:
            return _typed_attribute_cell(cell, case, target)
        if cell['public_api_symbol'].startswith('kmipkit.attribute_operations.'):
            return _attribute_request_cell(cell, case, target)
        if cell['public_api_symbol'].startswith('kmipkit.managed_objects.'):
            return _managed_object_cell(cell, case, target)
        if cell['public_api_symbol'] == 'kmipkit.extensions.extension_information_to_ttlv':
            return _extension_information_cell(cell, target, trace)
        if cell['public_api_symbol'].startswith('kmipkit.cryptographic_parameters.'):
            return _crypto_parameters_cell(cell, case, target)
        if cell['public_api_symbol'].startswith('kmipkit.messages.'):
            return _typed_message_cell(cell, case, target)
        if cell['public_api_symbol'] in _TYPED_RESPONSE_TTLV_ROUTES:
            return _typed_response_ttlv_cell(cell, target, trace)
        if cell['public_api_symbol'] in _QUERY_RESPONSE_FIELD_ROUTES:
            return _query_response_field_cell(cell, target, trace)
        if cell['public_api_symbol'] in _RESPONSE_DATA_FIELD_ROUTES:
            return _response_data_field_cell(cell, target, trace)
        if cell['public_api_symbol'] in _TICKET_TTLV_ROUTES:
            return _ticket_ttlv_cell(cell, target, trace)
        if cell['public_api_symbol'] == _GET_ATTRIBUTE_LIST_RESPONSE_ROUTE:
            return _get_attribute_list_response_constructor_cell(cell, target, trace)
        if cell['public_api_symbol'] == _BATCH_ERROR_CONTINUATION_ROUTE:
            return _batch_error_continuation_cell(cell, case, target, trace)
        if cell['public_api_symbol'] in _ASYNC_OUTCOME_RESPONSE_ROUTES:
            return _async_outcome_response_cell(cell, case, target, trace)
        if cell['public_api_symbol'] in _BATCH_ITEM_RESULT_VALUE_ROUTES:
            return _batch_item_result_value_cell(cell, case, target, trace)
        return _generic_cell(cell, case, target)
    if case['case_id'] == 'test-op-discover-versions':
        return _positive_fixture_cell(cell, case, target, trace=trace)
    if (case.get('requires', {}).get('workflow_type') == 'batch_id_placeholder'
            and cell.get('workflow_operation') is not None):
        return _batch_workflow_cell(cell, case, trace=trace)
    if (case.get('requires', {}).get('workflow_type') == 'batch_id_placeholder'
            and target.get('kind') == 'operation'):
        case = {**case, 'operation': target.get('name', case['operation'])}
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
    if any(route['language'] == 'python' for route in manifest['routes']):
        # Several public modules import each other through kmipkit.__init__.
        # Finish that package initialization before worker threads resolve APIs.
        importlib.import_module('kmipkit')
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
    operation = _cell_operation(cell, case)
    final_async_failure = (
        case['case_id'] == 'test-async-poll-final-operation-failure' and operation == 'Poll'
    )
    invalid_async_correlation = (
        case['case_id'] == 'test-async-unknown-correlation-rejected' and operation == 'Poll'
    )
    if positive_ping:
        peer_status = protocol_values.ResultStatus.SUCCESS
        peer_reason = None
        expected_driver_status = 'passed'
    elif final_async_failure:
        peer_status = protocol_values.ResultStatus.OPERATION_FAILED
        peer_reason = protocol_values.ResultReason.INVALID_OBJECT_TYPE
        expected_driver_status = 'failed'
    elif invalid_async_correlation:
        peer_status = protocol_values.ResultStatus.OPERATION_FAILED
        peer_reason = protocol_values.ResultReason.INVALID_ASYNCHRONOUS_CORRELATION_VALUE
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
            route_case = case if case['operation'] == operation else {**case, 'operation': operation}
            identifier = extensions.UniqueIdentifier.text_string('KMIPKit-python-route')
            name = 'KMIPKit-python-route'
            if symbol.endswith('.to_batch_item') and symbol.startswith('kmipkit.attribute_operations.'):
                item = driver._build_high_level_item(route_case, identifier, name=name)
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
                item = driver._build_item(route_case, identifier, name=name)
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


def _batch_workflow_cell(cell: dict, case: dict,
                         *, trace: CallTrace | None) -> dict:
    """Send the catalogued ordered batch using omitted IDs for placeholder consumers."""
    from kmipkit import protocol_values, ttlv
    from .controlled_peer import ControlledTlsPeer, PKI, batch_response

    scenario = case.get('requires', {}).get('protocol_scenario', {})
    operations = scenario.get('operations') if isinstance(scenario, dict) else None
    if not isinstance(operations, list) or not operations:
        raise RecipePending
    if cell.get('workflow_operation') not in operations:
        raise ValueError('batch route operation is absent from the catalog workflow')

    status = protocol_values.ResultStatus.OPERATION_FAILED
    reason = protocol_values.ResultReason.OPERATION_NOT_SUPPORTED
    peer = ControlledTlsPeer(
        status=status,
        reason=reason,
        encoding=cell['encoding'],
        transport=cell['transport'],
        protocol_version=cell['protocol_version'],
        response_builder=lambda request, encoding, minor: batch_response(
            request, encoding, operations, status, reason, minor),
    )
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
    batch = None
    limits = None
    response = None
    try:
        driver = PythonKmipDriver(connection)
        batch = driver._client_module.Batch.create()
        limits = ttlv.codec_limits_defaults()
        for index, operation in enumerate(operations):
            item_case = {**case, 'operation': operation}
            if (operation == 'Get Attribute List'
                    and cell['public_api_symbol']
                    == 'kmipkit.attribute_operations.GetAttributeListRequest.to_batch_item'):
                item = driver._build_high_level_item(
                    item_case, None, name=f"KMIPKit-route-{case['case_id']}-{index}")
            else:
                item = driver._build_item(
                    item_case, None, name=f"KMIPKit-route-{case['case_id']}-{index}")
            item = driver._extensions.with_unique_batch_item_id(
                item, f"kmipkit-route-{index + 1:02d}".encode('ascii'))
            batch.add_item(item)
        response = driver._client.execute(batch, limits)
        if response.item_count() != len(operations):
            raise ValueError('typed batch response count differs from the ordered request')
        status_codes = [int(response.item_at(index).result_status)
                        for index in range(response.item_count())]
        reason_codes = [int(response.item_at(index).result_reason)
                        if response.item_at(index).result_reason is not None else None
                        for index in range(response.item_count())]
        if status_codes != [status] * len(operations) or reason_codes != [reason] * len(operations):
            raise ValueError('controlled batch peer response did not preserve ordered item outcomes')
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
        raise ValueError('controlled batch fixture did not exchange exactly one message')
    request = _network_tree(peer.requests[0], cell['encoding'])
    response_tree = _network_tree(peer.responses[0], cell['encoding'])
    request_items = _batch_message_items(request)
    response_items = _batch_message_items(response_tree)
    request_operations = _batch_message_operations(request)
    response_operations = _batch_message_operations(response_tree)
    if request_operations != operations or response_operations != operations:
        raise ValueError('controlled peer did not observe the complete ordered batch')
    if any(any(tag == 0x420094 for tag, _, _ in item) for item in request_items[1:]):
        raise ValueError('a placeholder-consuming batch item encoded a Unique Identifier')
    if peer.request_observation.get('batch_count') != len(operations):
        raise ValueError('controlled peer observed a different batch count')
    observed_status_codes = [next(value for tag, _, value in item if tag == 0x42007F)
                             for item in response_items]
    observed_reason_codes = [next((value for tag, _, value in item if tag == 0x42007E), None)
                             for item in response_items]
    return {
        'request_tree_sha256': digest(request),
        'response_tree_sha256': digest(response_tree),
        'wire_bytes_sha256': hashlib.sha256(peer.requests[0]).hexdigest(),
        'request_delivery_state': 'response_received',
        'transport_invocation_count': 1,
        'peer_observed_response_count': len(response_items),
        'peer_observed_result_status_codes': observed_status_codes,
        'peer_observed_result_reason_codes': observed_reason_codes,
        'batch_operation_sequence': operations,
        'peer_observed_operation_sequence': request_operations,
        'batch_item_count': len(request_items),
    }


def _batch_message_items(message: list) -> list[list]:
    fields = message
    if (len(message) == 1 and message[0][1] == 1
            and any(tag == 0x42000F for tag, _, _ in message[0][2])):
        fields = message[0][2]
    items = [value for tag, kind, value in fields if tag == 0x42000F and kind == 1]
    if not items:
        raise ValueError('controlled batch message has no BatchItem structures')
    return items


def _batch_message_operations(message: list) -> list[str]:
    from kmipkit import protocol_values

    operation_names = []
    for item in _batch_message_items(message):
        operation_value = next((value for tag, _, value in item if tag == 0x42005C), None)
        if operation_value is None:
            raise ValueError('controlled batch item omitted its Operation')
        operation_name = protocol_values.Operation.name(operation_value)
        if operation_name is None:
            raise ValueError('controlled batch item used an unknown Operation')
        operation_names.append(operation_name)
    return operation_names


def _positive_fixture_cell(cell: dict, case: dict, target: dict, *, trace: CallTrace | None) -> dict:
    """Execute Discover Versions against a positive peer and check typed output."""
    from .controlled_peer import ControlledTlsPeer, PKI
    from kmipkit import extensions, protocol_values

    if (case.get('operation') != 'Discover Versions'
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

def _credential_type_value(target: dict) -> int:
    if (target.get('kind') == 'enumeration_value'
            and 'KMIPKIT-ELEM-ENUMERATION-CREDENTIAL-TYPE'
            in target.get('parent_element_ids', [])):
        values = PythonKmipDriver._wire_representatives(target['wire_value'])
        if len(values) != 1:
            raise RecipePending
        return values[0]
    return 1


def _credential_envelope_tree(credential_type: int) -> list:
    return [0x420023, 1, [
        [0x420024, 5, credential_type],
        [0x420025, 1, [[0x42002A, 2, 0x13579]]],
    ]]


_CREDENTIAL_TYPES = {
    'USERNAME-AND-PASSWORD': 1,
    'DEVICE': 2,
    'ATTESTATION': 3,
    'ONE-TIME-PASSWORD': 4,
    'HASHED-PASSWORD': 5,
    'TICKET': 6,
}
_CREDENTIAL_TAG_TYPES = {
    0x420038: 5, 0x420092: 5, 0x420099: 1, 0x4200A1: 1,
    0x4200A2: 2, 0x4200A9: 2, 0x4200AA: 2, 0x4200AB: 2,
    0x4200B0: 2, 0x4200C7: 3, 0x4200C8: 3, 0x4200C9: 3,
    0x4200CA: 3, 0x4200CB: 3, 0x4200CC: 3, 0x420149: 6,
    0x42014A: 6, 0x42014B: 6, 0x420156: 4, 0x420157: 5,
}
_CREDENTIAL_MEMBER_TAGS = {
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-NONCE': 0x4200C8,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-TYPE': 0x4200C7,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-MEASUREMENT': 0x4200CB,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-ASSERTION': 0x4200CC,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-TYPE': 0x420024,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-VALUE': 0x420025,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-SERIAL-NUMBER': 0x4200B0,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-IDENTIFIER': 0x4200A2,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-NETWORK-IDENTIFIER': 0x4200AB,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MACHINE-IDENTIFIER': 0x4200A9,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MEDIA-IDENTIFIER': 0x4200AA,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-PASSWORD': 0x4200A1,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-USERNAME': 0x420099,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-TIMESTAMP': 0x420092,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHING-ALGORITHM': 0x420038,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHED-PASSWORD': 0x420157,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-USERNAME': 0x420099,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-PASSWORD': 0x4200A1,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-ONE-TIME-PASSWORD': 0x420156,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-TICKET-TICKET': 0x420149,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-USERNAME': 0x420099,
    'KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-PASSWORD': 0x4200A1,
}
_CREDENTIAL_AUTH_TAG = 0x42000C
_CREDENTIAL_NONCE_TAG = 0x4200C8
_CREDENTIAL_TYPED_MODEL_TYPES = {
    'kmipkit.credentials.UsernameAndPasswordCredential.from_ttlv': ('UsernameAndPasswordCredential', 1),
    'kmipkit.credentials.UsernameAndPasswordCredential.into_ttlv': ('UsernameAndPasswordCredential', 1),
    'kmipkit.credentials.DeviceCredential.from_ttlv': ('DeviceCredential', 2),
    'kmipkit.credentials.DeviceCredential.into_ttlv': ('DeviceCredential', 2),
    'kmipkit.credentials.AttestationCredential.from_ttlv': ('AttestationCredential', 3),
    'kmipkit.credentials.AttestationCredential.into_ttlv': ('AttestationCredential', 3),
    'kmipkit.credentials.HashedPasswordCredential.from_ttlv': ('HashedPasswordCredential', 5),
    'kmipkit.credentials.HashedPasswordCredential.into_ttlv': ('HashedPasswordCredential', 5),
}


def _credential_type_for_target(target: dict) -> int:
    """Select the exact Credential Type established by the catalog target."""
    exact = _credential_type_value(target)
    if target.get('kind') == 'enumeration_value' and exact != 1:
        return exact
    for element_id in (target.get('target_id', ''), *target.get('parent_element_ids', [])):
        for name, raw in _CREDENTIAL_TYPES.items():
            if f'CREDENTIAL-{name}' in element_id:
                return raw
    if target.get('kind') == 'tag' and isinstance(target.get('wire_value'), str):
        try:
            return _CREDENTIAL_TAG_TYPES.get(int(target['wire_value'], 16), 1)
        except ValueError:
            pass
    return 1


def _credential_field(tag: int, kind: int, value: Any) -> list:
    if kind == 7:
        value = str(value).encode('utf-8').hex()
    elif kind in (4, 8):
        value = bytes(value).hex()
    return [tag, kind, value]


def _credential_nonce_children() -> list:
    return [
        _credential_field(0x4200C9, 8, b'\x11\x22\x33'),
        _credential_field(0x4200CA, 8, b'\xa1\xb2\xc3'),
    ]


def _credential_value_children(credential_type: int) -> list:
    """Return a valid synthetic Credential Value tree for the selected type."""
    if credential_type == 1:
        return [_credential_field(0x420099, 7, 'validator-user'),
                _credential_field(0x4200A1, 7, 'validator-secret')]
    if credential_type == 2:
        return [_credential_field(0x4200B0, 7, 'validator-device-serial'),
                _credential_field(0x4200A1, 7, 'validator-device-secret'),
                _credential_field(0x4200A2, 7, 'validator-device-id'),
                _credential_field(0x4200AB, 7, 'validator-network-id'),
                _credential_field(0x4200A9, 7, 'validator-machine-id'),
                _credential_field(0x4200AA, 7, 'validator-media-id')]
    if credential_type == 3:
        return [[0x4200C8, 1, _credential_nonce_children()],
                [0x4200C7, 5, 0xF1234567],
                _credential_field(0x4200CB, 8, b'\x10\x20\x30'),
                _credential_field(0x4200CC, 8, b'\x40\x50\x60')]
    if credential_type == 4:
        return [_credential_field(0x420099, 7, 'validator-user'),
                _credential_field(0x4200A1, 7, 'validator-secret'),
                _credential_field(0x420156, 7, 'validator-one-time-code')]
    if credential_type == 5:
        return [_credential_field(0x420099, 7, 'validator-user'),
                [0x420092, 11, 1_800_000_000],
                [0x420038, 5, 0xE1234567],
                _credential_field(0x420157, 8, b'\x71\x72\x73\x74')]
    if credential_type == 6:
        return [[0x420149, 1, [[0x42014A, 5, 1],
                                _credential_field(0x42014B, 8, b'\x81\x82\x83')]]]
    # Credential Types in the extension range and unassigned space are opaque.
    return [[0x540001, 2, 7]]


def _credential_envelope_tree_for_type(credential_type: int) -> list:
    return [0x420023, 1, [[0x420024, 5, credential_type],
                          [0x420025, 1, _credential_value_children(credential_type)]]]


def _credential_item_from_tree(tree: list, owned: ExitStack):
    """Materialize one numeric tree with the installed package's public TTLV API."""
    tag, kind, value = tree
    type_names = {1: 'structure', 2: 'integer', 3: 'long_integer', 4: 'big_integer',
                  5: 'enumeration', 6: 'boolean', 7: 'text', 8: 'bytes',
                  9: 'date_time', 10: 'interval', 11: 'date_time_extended'}
    if kind == 1:
        children = [owned.enter_context(_credential_item_from_tree(child, owned))
                    for child in value]
        value = owned.enter_context(PythonKmipDriver._structure(children))
    elif kind == 7:
        value = bytes.fromhex(value).decode('utf-8')
    elif kind in (4, 8):
        value = bytes.fromhex(value)
    return owned.enter_context(PythonKmipDriver._item(tag, type_names[kind], value))


def _credential_structure_from_children(children: list, owned: ExitStack):
    items = [owned.enter_context(_credential_item_from_tree(child, owned)) for child in children]
    return owned.enter_context(PythonKmipDriver._structure(items))


def _credential_tree_has_tag(tree: list, tag: int) -> bool:
    if tree[0] == tag:
        return True
    return tree[1] == 1 and any(_credential_tree_has_tag(child, tag) for child in tree[2])


def _credential_assert_target(target: dict, tree: list, credential_type: int) -> None:
    """Require the output TTLV to contain this exact expanded target."""
    kind = target.get('kind')
    if kind == 'tag':
        if _credential_tree_has_tag(tree, int(target['wire_value'], 16)):
            return
    elif kind == 'structure_member':
        tag = _CREDENTIAL_MEMBER_TAGS.get(target['target_id'])
        if tag is not None and _credential_tree_has_tag(tree, tag):
            return
    elif kind == 'message_field':
        if target['target_id'].endswith('AUTHENTICATION') and _credential_tree_has_tag(tree, _CREDENTIAL_AUTH_TAG):
            return
        if target['target_id'].endswith('NONCE') and _credential_tree_has_tag(tree, _CREDENTIAL_NONCE_TAG):
            return
        if target['target_id'].endswith('NONCE-ID') and _credential_tree_has_tag(tree, 0x4200C9):
            return
        if target['target_id'].endswith('NONCE-VALUE') and _credential_tree_has_tag(tree, 0x4200CA):
            return
        if target['target_id'].endswith('CREDENTIAL-MAY-BE-REPEATED'):
            count = sum(child[0] == 0x420023 for child in tree[2]) if tree[1] == 1 else 0
            if count >= 2:
                return
    elif kind == 'credential':
        expected = _credential_type_for_target(target)
        if credential_type == expected:
            return
    elif kind == 'enumeration_value':
        if credential_type == _credential_type_value(target):
            return
    elif kind == 'data_type':
        expected_kind = 8 if target['target_id'].endswith('BYTE-STRING') else 7
        if tree[1] == expected_kind:
            return
    raise ValueError('Credential TTLV output did not contain the exact normative target')


def _credential_typed_model_tree(credentials_module, class_name: str, credential_type: int,
                                 owned: ExitStack) -> list:
    """Roundtrip one specialized Credential Value model using public methods."""
    value_children = _credential_value_children(credential_type)
    source = _credential_structure_from_children(value_children, owned)
    model = owned.enter_context(getattr(credentials_module, class_name).from_ttlv(source))
    result = owned.enter_context(model.into_ttlv())
    result_item = owned.enter_context(PythonKmipDriver._item(0x420025, 'structure', result))
    tree = item_tree(result_item)
    if tree != [0x420025, 1, value_children]:
        raise ValueError(f'{class_name} roundtrip changed its exact fields')
    return tree


def _credential_cell(cell: dict, case: dict, target: dict) -> dict:
    """Roundtrip public typed Credential APIs and verify exact normative identity."""
    from kmipkit import credentials

    symbol = cell['public_api_symbol']
    credential_type = _credential_type_for_target(target)
    if symbol == 'kmipkit.credentials.CredentialType.from_raw':
        observed = credentials.CredentialType.from_raw(credential_type).raw()
        expected = credential_type
        with ExitStack() as owned:
            item = owned.enter_context(PythonKmipDriver._item(0x420024, 'enumeration', observed))
            tree = item_tree(item)
            _credential_assert_target(target, tree, observed)
    elif symbol in ('kmipkit.credentials.Credential.from_ttlv',
                    'kmipkit.credentials.Credential.into_ttlv'):
        with ExitStack() as owned:
            envelope = _credential_envelope_tree_for_type(credential_type)
            source = _credential_structure_from_children(envelope[2], owned)
            credential = owned.enter_context(credentials.Credential.from_ttlv(source))
            if credential.credential_type_raw() != credential_type:
                raise ValueError('generic Credential Type getter changed the raw value')
            result = owned.enter_context(credential.into_ttlv())
            result_item = owned.enter_context(PythonKmipDriver._item(envelope[0], 'structure', result))
            tree = item_tree(result_item)
            expected = envelope
            if tree != expected:
                raise ValueError('generic Credential envelope did not preserve its exact fields')
            _credential_assert_target(target, tree, credential_type)
            observed = tree
    elif symbol in ('kmipkit.credentials.CredentialValue.from_ttlv',
                    'kmipkit.credentials.CredentialValue.into_ttlv'):
        with ExitStack() as owned:
            value_children = _credential_value_children(credential_type)
            source = _credential_structure_from_children(value_children, owned)
            credential_value = owned.enter_context(
                credentials.CredentialValue.from_ttlv(credential_type, source))
            if credential_value.credential_type_raw() != credential_type:
                raise ValueError('Credential Value changed its raw Credential Type')
            result = owned.enter_context(credential_value.into_ttlv())
            result_item = owned.enter_context(PythonKmipDriver._item(0x420025, 'structure', result))
            tree = item_tree(result_item)
            expected = [0x420025, 1, value_children]
            if tree != expected:
                raise ValueError('Credential Value roundtrip changed its exact fields')
            _credential_assert_target(target, tree, credential_type)
            observed = tree
    elif symbol in ('kmipkit.credentials.Nonce.from_ttlv',
                    'kmipkit.credentials.Nonce.into_ttlv'):
        with ExitStack() as owned:
            children = _credential_nonce_children()
            source = _credential_structure_from_children(children, owned)
            nonce = owned.enter_context(credentials.Nonce.from_ttlv(source))
            result = owned.enter_context(nonce.into_ttlv())
            result_item = owned.enter_context(
                PythonKmipDriver._item(_CREDENTIAL_NONCE_TAG, 'structure', result))
            tree = item_tree(result_item)
            expected = [_CREDENTIAL_NONCE_TAG, 1, children]
            if tree != expected:
                raise ValueError('Nonce roundtrip changed its exact fields')
            _credential_assert_target(target, tree, 3)
            observed = tree
    elif symbol.startswith('kmipkit.credentials.Authentication.'):
        with ExitStack() as owned:
            credential_trees = [_credential_envelope_tree_for_type(raw) for raw in (1, 4)]
            source_children = credential_trees
            source = _credential_structure_from_children(source_children, owned)
            if symbol.endswith('Authentication.create'):
                credential_handles = []
                for envelope in credential_trees:
                    nested = _credential_structure_from_children(envelope[2], owned)
                    credential_handles.append(owned.enter_context(credentials.Credential.from_ttlv(nested)))
                authentication = owned.enter_context(credentials.Authentication.create(credential_handles))
            else:
                authentication = owned.enter_context(credentials.Authentication.from_ttlv(source))
            result = owned.enter_context(authentication.into_ttlv())
            result_item = owned.enter_context(
                PythonKmipDriver._item(_CREDENTIAL_AUTH_TAG, 'structure', result))
            tree = item_tree(result_item)
            expected = [_CREDENTIAL_AUTH_TAG, 1, source_children]
            if tree != expected:
                raise ValueError('Authentication roundtrip changed order or repeated Credentials')
            _credential_assert_target(target, tree, 1)
            observed = tree
    elif symbol in ('kmipkit.credentials.SecretBytes.into_ttlv_value',
                    'kmipkit.credentials.SecretText.into_ttlv_value'):
        from kmipkit import ttlv

        with ExitStack() as owned:
            if symbol.endswith('SecretBytes.into_ttlv_value'):
                secret = owned.enter_context(credentials.SecretBytes.from_bytes(b'validator-fixture-bytes'))
                value = owned.enter_context(secret.into_ttlv_value())
                tag, expected_type = 0x4200FF, 8
            else:
                secret = owned.enter_context(credentials.SecretText.from_text('validator-fixture-text'))
                value = owned.enter_context(secret.into_ttlv_value())
                tag, expected_type = 0x4200FF, 7
            limits = owned.enter_context(ttlv.codec_limits_defaults())
            raw_tag = owned.enter_context(ttlv.raw_tag_from_raw(tag))
            checked_tag = owned.enter_context(ttlv.raw_tag_try_checked(raw_tag))
            item = owned.enter_context(ttlv.ttlv_item_create(checked_tag, value, limits))
            tree = item_tree(item)
            if tree[1] != expected_type:
                raise ValueError('Secret TTLV conversion returned the wrong data type')
            _credential_assert_target(target, tree, credential_type)
            observed = tree
    elif symbol in _CREDENTIAL_TYPED_MODEL_TYPES:
        class_name, fixed_type = _CREDENTIAL_TYPED_MODEL_TYPES[symbol]
        with ExitStack() as owned:
            tree = _credential_typed_model_tree(credentials, class_name, fixed_type, owned)
            _credential_assert_target(target, tree, fixed_type)
            observed = tree
    else:
        raise RecipePending
    if symbol == 'kmipkit.credentials.CredentialType.from_raw' and observed != expected:
        raise ValueError('Credential Type route changed the normative raw value')
    tree_hash = digest(observed if not isinstance(observed, int) else [0x420024, 5, observed])
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


_ASYNC_OPERATION_CLASSES = {
    'CancelRequest': 'Cancel',
    'PollRequest': 'Poll',
    'ProcessRequest': 'Process',
    'QueryAsyncRequestsRequest': 'Query Asynchronous Requests',
}
_ASYNC_CORRELATION = b'\x00\x7f\x80\xff'
_ASYNC_CORRELATION_VALUES = (_ASYNC_CORRELATION, b'\x10\x20\x30')
_ASYNC_OPERATION_FILTER = (0x0000001A, 0xF1234567)


def _async_target_tag(target: dict) -> int | None:
    """Map an exact asynchronous field target to its normative TTLV tag."""
    if target.get('kind') == 'tag':
        wire_value = target.get('wire_value')
        if isinstance(wire_value, str):
            return int(wire_value.removeprefix('0x'), 16)
        return None
    target_name = _normalized_name(target.get('name', ''))
    if target_name == 'asynchronouscorrelationvalue':
        return 0x420006
    if target_name == 'asynchronouscorrelationvalues':
        return 0x420176
    if target_name == 'operation':
        return 0x42005C
    if target_name == 'operations':
        return 0x42014F
    return None


def _async_request_fields(operation: str, *, correlation_filter: bool, operation_filter: bool) -> list:
    if operation != 'Query Asynchronous Requests':
        return [[0x420006, 8, _ASYNC_CORRELATION.hex()]]
    fields = []
    if correlation_filter:
        fields.append([0x420176, 1, [
            [0x420006, 8, value.hex()] for value in _ASYNC_CORRELATION_VALUES
        ]])
    if operation_filter:
        fields.append([0x42014F, 1, [
            [0x42005C, 5, value] for value in _ASYNC_OPERATION_FILTER
        ]])
    return fields


def _assert_async_request_target(operation: str, target: dict, tree: list) -> None:
    if target.get('kind') == 'operation':
        if _normalized_name(target.get('name', '')) != _normalized_name(operation):
            raise ValueError('asynchronous request class does not match its operation target')
        return
    tag = _async_target_tag(target)
    if tag is None or not _credential_tree_has_tag([0, 1, tree[2]], tag):
        raise ValueError('asynchronous TTLV output did not contain the exact target')


def _async_request_cell(cell: dict, case: dict, target: dict) -> dict:
    """Invoke one public asynchronous request constructor or TTLV encoder."""
    from kmipkit import async_operations

    symbol = cell['public_api_symbol']
    class_name = symbol.split('.')[2]
    operation = _ASYNC_OPERATION_CLASSES.get(class_name)
    if operation is None:
        raise RecipePending
    correlation_filter = (class_name != 'QueryAsyncRequestsRequest'
                          or symbol.endswith('.with_correlation_values')
                          or symbol.endswith('.create') or symbol.endswith('.to_ttlv_payload'))
    operation_filter = (class_name != 'QueryAsyncRequestsRequest'
                        or symbol.endswith('.with_operations')
                        or symbol.endswith('.create') or symbol.endswith('.to_ttlv_payload'))
    expected = _async_request_fields(operation, correlation_filter=correlation_filter,
                                     operation_filter=operation_filter)
    with ExitStack() as owned:
        if class_name == 'QueryAsyncRequestsRequest':
            request = owned.enter_context(async_operations.QueryAsyncRequestsRequest.create())
            if (symbol.endswith('.with_correlation_values') or symbol.endswith('.create')
                    or symbol.endswith('.to_ttlv_payload')):
                request = owned.enter_context(request.with_correlation_values(_ASYNC_CORRELATION_VALUES))
            if (symbol.endswith('.with_operations') or symbol.endswith('.create')
                    or symbol.endswith('.to_ttlv_payload')):
                request = owned.enter_context(request.with_operations(_ASYNC_OPERATION_FILTER))
        else:
            request_type = getattr(async_operations, class_name)
            request = owned.enter_context(request_type.create(_ASYNC_CORRELATION))
        payload = owned.enter_context(request.to_ttlv_payload())
        payload_item = owned.enter_context(PythonKmipDriver._item(0x4200FF, 'structure', payload))
        tree = item_tree(payload_item)
        if tree != [0x4200FF, 1, expected]:
            raise ValueError('asynchronous request emitted different TTLV fields')
    _assert_async_request_target(operation, target, tree)
    tree_hash = digest([operation, tree])
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


_ATTRIBUTE_REQUEST_OPERATIONS = {
    'AddAttributeRequest': 'Add Attribute',
    'AdjustAttributeRequest': 'Adjust Attribute',
    'DeleteAttributeRequest': 'Delete Attribute',
    'GetAttributeListRequest': 'Get Attribute List',
    'GetAttributesRequest': 'Get Attributes',
    'ModifyAttributeRequest': 'Modify Attribute',
    'SetAttributeRequest': 'Set Attribute',
}
_ATTRIBUTE_PAYLOAD_TAGS = {
    'Unique Identifier': 0x420094,
    'New Attribute': 0x42013D,
    'Current Attribute': 0x42013C,
    'Attribute Reference': 0x42013B,
    'Adjustment Type': 0x420158,
    'Adjustment Value': 0x420162,
}


def _assert_attribute_request_target(operation: str, target: dict, tree: list) -> None:
    if target.get('kind') == 'operation':
        if _normalized_name(target.get('name', '')) != _normalized_name(operation):
            raise ValueError('attribute request class does not match its operation target')
        return
    tag = _ATTRIBUTE_PAYLOAD_TAGS.get(target.get('name', ''))
    if tag is None or not _credential_tree_has_tag([0, 1, tree[2]], tag):
        raise ValueError('attribute request payload omitted its exact normative field')


def _attribute_request_cell(cell: dict, case: dict, target: dict) -> dict:
    """Invoke one standalone typed attribute request constructor or encoder."""
    from kmipkit import attribute_operations, extensions, ttlv

    symbol = cell['public_api_symbol']
    class_name = symbol.split('.')[2]
    operation = _ATTRIBUTE_REQUEST_OPERATIONS.get(class_name)
    if operation is None or not (symbol.endswith('.create') or symbol.endswith('.to_ttlv_payload')):
        raise RecipePending
    identifier = extensions.UniqueIdentifier.text_string('KMIPKit-route-object')
    reference = extensions.AttributeReference.name('KMIPKit', 'Contact Information')
    by_tag = extensions.AttributeReference.tag(0x420022)
    with ExitStack() as owned:
        current = owned.enter_context(PythonKmipDriver._item(0x420022, 'text', 'old-value'))
        new = owned.enter_context(PythonKmipDriver._item(0x420022, 'text', 'new-value'))
        if class_name == 'AddAttributeRequest':
            request = attribute_operations.AddAttributeRequest.create(identifier, new)
        elif class_name == 'SetAttributeRequest':
            request = attribute_operations.SetAttributeRequest.create(identifier, new)
        elif class_name == 'ModifyAttributeRequest':
            request = attribute_operations.ModifyAttributeRequest.create(identifier, current, new)
        elif class_name == 'DeleteAttributeRequest':
            request = attribute_operations.DeleteAttributeRequest.create(identifier, current, reference)
        elif class_name == 'AdjustAttributeRequest':
            adjustment = owned.enter_context(ttlv.ttlv_value_integer(1))
            request = attribute_operations.AdjustAttributeRequest.create(identifier, reference, 1, adjustment)
        elif class_name == 'GetAttributesRequest':
            request = attribute_operations.GetAttributesRequest.create(identifier, (by_tag, reference))
        else:
            request = attribute_operations.GetAttributeListRequest.create(identifier)
        request = owned.enter_context(request)
        payload = owned.enter_context(request.to_ttlv_payload())
        item = owned.enter_context(PythonKmipDriver._item(0x4200FF, 'structure', payload))
        tree = item_tree(item)
    if tree[0:2] != [0x4200FF, 1]:
        raise ValueError('typed attribute request payload is not a Structure')
    _assert_attribute_request_target(operation, target, tree)
    tree_hash = digest([operation, tree])
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _typed_attribute_cell(cell: dict, case: dict, target: dict) -> dict:
    """Check exact direct-attribute identity, value copies and collection reads."""
    from kmipkit import ttlv
    from kmipkit.attribute_operations import AttributeSet
    from kmipkit.typed_attributes import StandardAttribute, TypedAttribute

    attributes = {attribute.tag: attribute for attribute in StandardAttribute}
    target_kind = _target_field(target, case['requires'], 'kind')
    member_name = target.get('name', '')
    if target_kind == 'structure_member':
        by_id, _, elements = _catalog_model_data()
        parent = next((by_id.get(parent_id, {}) for parent_id in target.get('parent_element_ids', [])
                       if by_id.get(parent_id, {}).get('kind') == 'attribute'), None)
        if parent is None:
            raise RecipePending
        source_parent = elements.get(parent['target_id'], {})
        parent_tag = next((by_id.get(parent_id, {}) for parent_id in source_parent.get('parent_element_ids', [])
                           if by_id.get(parent_id, {}).get('kind') == 'tag'), None)
        if parent_tag is None:
            tag_name = source_parent.get('source_name', parent['name'])
            parent_tag = next((item for item in by_id.values()
                               if item.get('kind') == 'tag' and item.get('name') == tag_name), None)
        if parent_tag is None:
            raise RecipePending
        root_tag = int(parent_tag['wire_value'], 16)
        field_samples = _generic_samples(case, target, 'kmipkit.ttlv.ttlv_item_create')
        if parent['name'] == 'Vendor Attribute' and member_name == 'Vendor Identification':
            # §4.60 Table 150 restricts this field to ASCII letters, digits, '_' and '.'.
            field_samples = [(field_samples[0][0], 'Text String', 'KMIPKit')]
        samples = [(root_tag, 'Structure', _typed_attribute_structure_member_items(
            parent['name'], member_name, *field_sample
        )) for field_sample in field_samples]
    else:
        samples = _generic_samples(case, target, 'kmipkit.ttlv.ttlv_item_create')
    if not samples or any(tag not in attributes for tag, _, _ in samples):
        raise RecipePending
    observed = []
    for tag, type_name, value in samples:
        attribute = attributes[tag]
        if target_kind == 'tag':
            # Tag allocation cases have no prescribed value. Use the installed
            # attribute's exact catalogued scalar type instead of inventing one.
            scalar = {'Integer': ('Integer', 0), 'Enumeration': ('Enumeration', 1),
                      'DateTime': ('Date Time', 0), 'Interval': ('Interval', 0),
                      'Boolean': ('Boolean', True), 'TextString': ('Text String', 'fixture'),
                      'ByteString': ('Byte String', b''), 'UniqueIdentifier': ('Text String', 'fixture'),
                      'Structure': ('Structure', None)}
            if attribute.value_kind not in scalar:
                raise RecipePending
            type_name, value = scalar[attribute.value_kind]
        with ExitStack() as owned:
            if type_name == 'Structure':
                members = (value if target_kind == 'structure_member'
                           else _typed_attribute_structure_items(target.get('name', '')))
                if members is None:
                    raise RecipePending
                children = []
                for child_tag, child_type, child_value in members:
                    if child_type == 'Structure' and child_value is None:
                        child_value = owned.enter_context(PythonKmipDriver._structure([]))
                    children.append(owned.enter_context(PythonKmipDriver._item(
                        child_tag, _TTLV_TYPE_KINDS[child_type], child_value
                    )))
                value = owned.enter_context(PythonKmipDriver._structure(children))
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


_TYPED_ATTRIBUTE_STRUCTURE_FIELDS: dict[str, tuple[tuple[str, str, Any], ...]] = {
    # KMIP 2.0/2.1 §4.2, Table 31; Alternative Name Type value 1 is
    # Uninterpreted Text String under §11.2.
    'Alternative Name': (
        ('Alternative Name Value', 'Text String', 'KMIPKit.Interop'),
        ('Alternative Name Type', 'Enumeration', ('enum', 'Alternative Name Type', 'Uninterpreted Text String')),
    ),
    # §4.4 Table 36 requires Application Namespace; Application Data is optional.
    'Application Specific Information': (
        ('Application Namespace', 'Text String', 'KMIPKit.Interop'),
        ('Application Data', 'Text String', 'interop-fixture'),
    ),
    # §4.14 Table 46; a positive Qlength is a valid DSA parameter representative.
    'Cryptographic Domain Parameters': (('Qlength', 'Integer', 256),),
    # §4.16 Table 59; the algorithm is a standalone optional member.
    'Cryptographic Parameters': (
        ('Cryptographic Algorithm', 'Enumeration', ('enum', 'Cryptographic Algorithm', None)),
    ),
    # §4.21 Table 69; Hashing Algorithm and Digest Value form a certificate digest.
    'Digest': (
        ('Hashing Algorithm', 'Enumeration', ('enum', 'Hashing Algorithm', None)),
        ('Digest Value', 'Byte String', bytes(20)),
    ),
    # §4.27 Table 82.
    'Key Value Location': (
            ('Key Value Location Type', 'Enumeration', ('enum', 'Key Value Location Type', 'URI')),
        ('Key Value Location Value', 'Text String', 'https://kmipkit.invalid/key/1'),
    ),
    # §4.31 Table 91.
    'Link': (
        ('Link Type', 'Enumeration', ('enum', 'Link Type', None)),
        ('Linked Object Identifier', 'Text String', 'KMIPKit.Interop.Object.1'),
    ),
    # §4.32 Table 93; Name Type value 1 is Uninterpreted Text String (§11.29).
    'Name': (
        ('Name Type', 'Enumeration', ('enum', 'Name Type', 'Uninterpreted Text String')),
        ('Name Value', 'Text String', 'KMIPKit.Interop'),
    ),
    # §4.46 and §7.36 Table 388 require RNG Algorithm when RNG Parameters is present.
    'Random Number Generator': (
        ('RNG Algorithm', 'Enumeration', ('enum', 'RNG Algorithm', None)),
    ),
    # §4.47 Table 123; Cessation of Operation is the exact §11.48 value 6.
    'Revocation Reason': (
        ('Revocation Reason Code', 'Enumeration', ('enum', 'Revocation Reason Code', 'Cessation of Operation')),
        ('Revocation Message', 'Text String', 'controlled fixture'),
    ),
    # §4.53 Table 135 and §11.57 Rotate Name Type.
    'Rotate Name': (
        ('Rotate Name Type', 'Enumeration', ('enum', 'Rotate Name Type', None)),
        ('Rotate Name Value', 'Text String', 'KMIPKit.Interop'),
    ),
    # §4.52 (2.0)/§4.59 (2.1), §7.40 Table 392.
    'Usage Limits': (
        ('Usage Limits Count', 'Long Integer', 1),
        ('Usage Limits Total', 'Long Integer', 1),
        ('Usage Limits Unit', 'Enumeration', ('enum', 'Usage Limits Unit', None)),
    ),
    # §4.53 (2.0)/§4.60 (2.1), Table 150; Vendor Identification obeys its
    # alphanumeric/underscore/period restriction.
    'Vendor Attribute': (
        ('Vendor Identification', 'Text String', 'KMIPKit'),
        ('Attribute Name', 'Text String', 'interop'),
        ('Attribute Value', 'Byte String', b'fixture'),
    ),
    # §4.54 (2.0)/§4.61 (2.1), Table 152.
    'X.509 Certificate Identifier': (
        ('Certificate Serial Number', 'Byte String', b'\x01'),
        ('Issuer Distinguished Name', 'Byte String', b'\x30\x00'),
    ),
    # §4.55/§4.62, Tables 153–154; the distinguished name is required.
    'X.509 Certificate Issuer': (('Issuer Distinguished Name', 'Byte String', b'\x30\x00'),),
    # §4.56/§4.63, Tables 155–156; the distinguished name is required.
    'X.509 Certificate Subject': (('Subject Distinguished Name', 'Byte String', b'\x30\x00'),),
}


def _typed_attribute_structure_items(attribute_name: str) -> list[tuple[int, str, Any]] | None:
    """Build minimal, table-shaped structures for catalogued direct attributes."""
    by_id, _, elements = _catalog_model_data()
    if attribute_name == 'Attribute':
        attribute_name = 'Vendor Attribute'
    plan = _TYPED_ATTRIBUTE_STRUCTURE_FIELDS.get(attribute_name)
    if plan is None:
        return None

    def tag_value(name: str) -> int:
        entry = next((item for item in elements.values()
                      if item.get('kind') == 'tag' and item.get('name') == name), None)
        if entry is None:
            raise RecipePending
        return int(entry['wire_value'], 16)

    def enum_value(name: str, preferred: str | None) -> int:
        enum = next((item for item in elements.values()
                     if item.get('kind') == 'enumeration' and item.get('name') == name), None)
        if enum is None:
            raise RecipePending
        candidates = [item for item in by_id.values()
                      if item.get('kind') == 'enumeration_value'
                      and enum['element_id'] in item.get('parent_element_ids', [])
                      and isinstance(item.get('wire_value'), str)
                      and not any(character in item['wire_value'] for character in 'Xx')]
        if preferred is not None:
            candidates = [item for item in candidates if item.get('name') == preferred]
        if not candidates:
            raise RecipePending
        candidate = min(candidates, key=lambda item: int(item['wire_value'], 16))
        values = PythonKmipDriver._wire_representatives(candidate['wire_value'])
        if len(values) != 1:
            raise RecipePending
        return values[0]

    result = []
    for name, type_name, value in plan:
        if isinstance(value, tuple) and value and value[0] == 'enum':
            value = enum_value(value[1], value[2])
        result.append((tag_value(name), type_name, value))
    return result


def _typed_attribute_structure_member_items(
    attribute_name: str, member_name: str, tag: int, type_name: str, value: Any,
) -> list[tuple[int, str, Any]]:
    """Insert one exact catalog member into a valid minimal parent attribute."""
    by_id, _, elements = _catalog_model_data()
    members = _typed_attribute_structure_items(attribute_name)
    if members is None:
        raise RecipePending
    if attribute_name == 'Cryptographic Parameters' and member_name != 'Cryptographic Algorithm':
        # Its optional fields are alternatives; only construct the requested one.
        members = []
    elif attribute_name == 'Cryptographic Domain Parameters' and member_name != 'Qlength':
        members = []
    fields = {field_tag: (field_tag, field_type, field_value)
              for field_tag, field_type, field_value in members}
    fields[tag] = (tag, type_name, value)

    attribute = next((item for item in by_id.values()
                      if item.get('kind') == 'attribute' and item.get('name') == attribute_name), None)
    if attribute is None:
        raise RecipePending
    return list(fields.values())


def _extension_information_cell(cell: dict, target: dict,
                               trace: CallTrace | None) -> dict:
    """Encode the required Extension Name and verify the complete TTLV structure."""
    from kmipkit import extensions

    with ExitStack() as owned:
        information = owned.enter_context(
            extensions.create_extension_information('KMIPKit.Interop')
        )
        structure = owned.enter_context(extensions.extension_information_to_ttlv(information))
        item = owned.enter_context(PythonKmipDriver._item(0x4200A4, 'structure', structure))
        tree = item_tree(item)
        expected = [0x4200A4, 1, [[0x4200A5, 7, '4b4d49504b69742e496e7465726f70']]]
        if tree != expected:
            raise ValueError('Extension Information changed its ordered Extension Name field')
        if target.get('kind') == 'tag' and tree[0] != int(target.get('wire_value', '0'), 16):
            raise ValueError('Extension Information root tag differs from the target')
        if trace is None:
            raise ValueError('Extension Information route has no invocation trace')
        trace.record_invocation(cell['public_api_symbol'])
    tree_hash = digest(tree)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _crypto_parameters_cell(cell: dict, case: dict, target: dict) -> dict:
    """Exercise named open-enum setters, typed getters and lossless converters."""
    from kmipkit import ttlv
    from kmipkit.cryptographic_parameters import CryptographicParameters

    names = {
        'Block Cipher Mode': 'block_cipher_mode', 'Padding Method': 'padding_method',
        'Hashing Algorithm': 'hashing_algorithm', 'Key Role Type': 'key_role_type',
        'Digital Signature Algorithm': 'digital_signature_algorithm',
        'Cryptographic Algorithm': 'cryptographic_algorithm', 'Mask Generator': 'mask_generator',
        'Mask Generator Hashing Algorithm': 'mask_generator_hashing_algorithm',
        'Random IV': 'random_iv', 'IV Length': 'iv_length', 'Tag Length': 'tag_length',
        'Fixed Field Length': 'fixed_field_length',
        'Invocation Field Length': 'invocation_field_length',
        'Counter Length': 'counter_length', 'Initial Counter Value': 'initial_counter_value',
        'Salt Length': 'salt_length', 'P Source': 'p_source', 'Trailer Field': 'trailer_field',
    }
    by_id, _, _ = _catalog_model_data()
    methods = {int(item['wire_value'], 16): names[item['name']] for item in by_id.values()
               if item.get('kind') == 'tag' and item.get('name') in names}
    field_tags = {name: int(item['wire_value'], 16) for item in by_id.values()
                  if item.get('kind') == 'tag' and item.get('name') in names
                  for name in (item['name'],)}
    target_name = target.get('name', '').removesuffix(', MAY be repeated').rstrip(',')
    if target.get('kind') == 'enumeration_value':
        samples = _generic_samples(case, target, 'kmipkit.ttlv.ttlv_item_create')
    elif (target.get('kind') == 'attribute' and target.get('name') in names
          and target.get('source_encoding') == 'Enumeration'):
        enumeration = next((item for item in _catalog_model_data()[2].values()
                            if item.get('kind') == 'enumeration'
                            and item.get('name') == target['name']), None)
        attribute_tag = next((by_id.get(parent_id) for parent_id in target.get('parent_element_ids', [])
                              if by_id.get(parent_id, {}).get('kind') == 'tag'), None)
        if enumeration is None or attribute_tag is None:
            raise RecipePending
        tag = int(attribute_tag['wire_value'], 16)
        samples = [
            (tag, 'Enumeration', int(item['wire_value'].replace('X', '0'), 16))
            for item in by_id.values()
            if item.get('kind') == 'enumeration_value'
            and enumeration['element_id'] in item.get('parent_element_ids', [])
            and item.get('allocation') in ('assigned', 'extension')
            and isinstance(item.get('wire_value'), str)
        ]
    elif (target.get('kind') in {'attribute', 'operation_structure', 'structure_member', 'tag'}
          and target_name == 'Cryptographic Parameters'):
        samples = [(0x420028, 'Enumeration', 3)]
    elif (target.get('kind') == 'operation_payload_field'
          and target_name == 'Cryptographic Parameters'):
        operation = case.get('requires', {}).get('operation', target.get('operation'))
        parameters = {
            'Encrypt': (0x420028, 3),  # AES
            'Decrypt': (0x420028, 3),  # AES
            'Hash': (0x420038, 6),  # SHA-256
            'MAC': (0x420028, 9),  # HMAC-SHA256
            'MAC Verify': (0x420028, 9),  # HMAC-SHA256
            'Sign': (0x4200AE, 5),  # SHA-256 with RSA Encryption
            'Signature Verify': (0x4200AE, 5),  # SHA-256 with RSA Encryption
        }
        tag, value = parameters.get(operation, (None, None))
        if tag is None or value is None:
            raise RecipePending
        samples = [(tag, 'Enumeration', value)]
    elif target.get('kind') in {'structure_member', 'tag'}:
        field = names.get(target_name)
        if field is None:
            raise RecipePending
        name = target_name
        if target.get('kind') == 'tag':
            tag = int(target['wire_value'], 16)
            value_type = {
                **{name: 'Enumeration' for name in (
                    'Block Cipher Mode', 'Padding Method', 'Hashing Algorithm', 'Key Role Type',
                    'Digital Signature Algorithm', 'Cryptographic Algorithm', 'Mask Generator',
                    'Mask Generator Hashing Algorithm')},
                'Random IV': 'Boolean', 'P Source': 'Byte String',
            }.get(name, 'Integer')
        else:
            tag = field_tags.get(name)
            value_types = PythonKmipDriver._catalog_value_types(case, target, _catalog_model_data()[1])
            value_type = next(iter(value_types), None)
        if tag is None or value_type not in {
                'Enumeration', 'Integer', 'Boolean', 'Byte String'}:
            raise RecipePending
        value = {'Enumeration': 1, 'Integer': 32, 'Boolean': True,
                 'Byte String': b'PSource-fixture'}[value_type]
        samples = [(tag, value_type, value)]
    else:
        raise RecipePending
    if not samples or any(kind == 'Enumeration' and tag not in methods for tag, kind, _ in samples):
        raise RecipePending
    observed = []
    for tag, value_type, value in samples:
        field = methods.get(tag, names.get(target_name))
        if field is None and cell['public_api_symbol'].endswith(('.builder', '.build', '.from_ttlv', '.into_ttlv')):
            field = 'cryptographic_algorithm'
            tag = 0x420028
            value_type, value = 'Enumeration', 3
        if field is None:
            raise RecipePending
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
            value_kind = {'Enumeration': 5, 'Integer': 2, 'Boolean': 6, 'Byte String': 8}[value_type]
            expected_value = value.hex() if value_kind == 8 else value
            expected = [0x42002B, 1, [[tag, value_kind, expected_value], *context_fields]]
            if item_tree(item) != expected or item_tree(copied_item) != expected:
                raise ValueError('cryptographic parameter typed/generic tree assertion failed')
            decoded = owned.enter_context(ttlv.ttlv_decode(encode_fixture(expected)))
            if item_tree(decoded) != expected:
                raise ValueError('cryptographic parameter generic roundtrip assertion failed')
            observed.append(expected[2][0])
    tree_hash = digest(observed)
    return {'request_tree_sha256': tree_hash, 'response_tree_sha256': tree_hash}


def _tag_decode_rejection_code(allocation: str | None) -> str | None:
    return {
        'reserved': 'tag_reserved',
        'unused': 'tag_not_allocated',
    }.get(allocation)


def _tag_rejection_cell(cell: dict, case: dict, target: dict) -> dict:
    """Check all catalog range endpoints without constructing a nonexistent Item."""
    import re
    from kmipkit import errors, ttlv

    requires = case['requires']
    target_kind = _target_field(target, requires, 'kind')
    wire_value = _target_field(target, requires, 'wire_value')
    if not isinstance(wire_value, str):
        raise RecipePending
    if target_kind == 'tag_range':
        bounds = re.split(r'\s*[-–—]\s*', wire_value)
        values = [int(bound.upper().replace('X', fill), 16)
                  for bound, fill in zip(bounds, ('0', 'F'), strict=True)]
    else:
        values = PythonKmipDriver._wire_representatives(wire_value)
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
        rejection_code = _tag_decode_rejection_code(target.get('allocation'))
        if rejection_code is None:
            raise RecipePending
        return {'decode_outcome': 'rejected', 'decode_rejection_code': rejection_code,
                'decode_input_sha256': hashlib.sha256(encode_fixture(inputs[0])).hexdigest()}
    return {'request_tree_sha256': digest(inputs)}
