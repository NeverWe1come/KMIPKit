#!/usr/bin/env python3
"""Execute installed C ABI routes and retain honest, sanitized KMIPKIT-0092 evidence.

The coordinator uses ctypes only for local opaque-handle ABI calls. Network
requests are executed by the compiled C consumer. Fixture bytes stay in memory;
only normalized tree digests and numeric outcomes enter route reports.
"""
from __future__ import annotations

import argparse
import ctypes as C
import hashlib
import json
from pathlib import Path
import re
import sys
import uuid

from artifact_support import INSTALL, PIN, ROOT, sha256, verify_installed_artifact
from route_evidence import cell_fields, pending_cell

sys.path.insert(0, str(ROOT / 'interop/shared/tools'))
from coverage_matrix import iter_cells
from validate_route_evidence import _key, validate_collection


def pending_reason(cell: dict) -> str:
    """An existing public symbol without a recipe is a validator gap."""
    return 'validator_recipe_pending'


def require_complete(events: list[dict], pending: list[dict]) -> None:
    if pending:
        raise ValueError(f'{len(pending)} pending validator recipe cells')
    keys = [_key(event) for event in events]
    if len(keys) != len(set(keys)):
        raise ValueError('duplicate route execution cells')
    if any(event['status'] != 'passed' for event in events):
        raise ValueError('failed route execution cells')


def _name(value: str) -> str:
    return re.sub('[^a-z0-9]', '', value.lower())


def scalar_fixture(target: dict, tags: dict[str, int]) -> tuple[int, int, bytes] | None:
    """Use each exact normative enum/flag/tag; never substitute an operation probe."""
    normalized_tags = {_name(name): value for name, value in tags.items()}
    kind = target['kind']
    if kind == 'tag':
        return int(target['wire_value'], 16), 2, (7).to_bytes(4, 'big')
    if kind not in {'enumeration_value', 'bitmask_value', 'referenced_enum_value'}:
        return None
    parents = target.get('parent_element_ids', [])
    parent = target.get('parent_name', '')
    if parents:
        parent = re.sub(r'^KMIPKIT-ELEM-(ENUMERATION|BITMASK)-', '', parents[0])
    tag = normalized_tags.get(_name(parent))
    if tag is None or not re.fullmatch(r'[0-9a-fA-F]{1,8}', target.get('wire_value', '')):
        return None
    return tag, 2 if kind == 'bitmask_value' else 5, int(target['wire_value'], 16).to_bytes(4, 'big')


class DecodeError(C.Structure):
    _fields_ = [('struct_size', C.c_uint32), ('abi_version', C.c_uint32),
                ('kind', C.c_uint32), ('byte_offset', C.c_uint64)]


class InstalledAbi:
    """Typed signatures and exact ownership from the installed public C header."""
    def __init__(self, library: Path):
        self.library = C.CDLL(str(library))
        self.called: set[str] = set()
        self.bind('kmipkit_codec_limits_defaults', [C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_raw_tag_create', [C.c_uint32, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_raw_tag_try_checked', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_raw_tag_value', [C.c_void_p, C.POINTER(C.c_uint32)])
        self.bind('kmipkit_ttlv_item_create', [C.c_void_p, C.c_void_p, C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_decode', [C.c_void_p, C.c_uint64, C.POINTER(DecodeError), C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_item_view_from_item', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_item_view_tag', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_tag_value', [C.c_void_p, C.POINTER(C.c_uint32)])
        self.bind('kmipkit_ttlv_item_view_type', [C.c_void_p, C.POINTER(C.c_uint8)])
        self.bind('kmipkit_ttlv_item_view_value', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_integer', [C.c_int32, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_enumeration', [C.c_uint32, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_view_integer', [C.c_void_p, C.POINTER(C.c_int32)])
        self.bind('kmipkit_ttlv_value_view_enumeration', [C.c_void_p, C.POINTER(C.c_uint32)])
        self.bind('kmipkit_ttlv_structure_create', [C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_structure_with_item', [C.c_void_p, C.c_void_p, C.c_void_p, C.POINTER(C.c_void_p)])
        for name in ('codec_limits', 'raw_tag', 'tag', 'ttlv_value', 'ttlv_item',
                     'ttlv_item_view', 'ttlv_value_view', 'ttlv_structure'):
            function = getattr(self.library, f'kmipkit_{name}_release')
            function.argtypes, function.restype = [C.c_void_p], None

    def bind(self, name: str, signature: list) -> None:
        function = getattr(self.library, name)
        function.argtypes, function.restype = signature, C.c_int32

    def call(self, symbol: str, *arguments) -> None:
        self.called.add(symbol)
        status = getattr(self.library, symbol)(*arguments)
        if status != 0:
            raise ValueError('installed_abi_call_failed')

    def inspect(self, item: C.c_void_p, fixture: tuple[int, int, bytes]) -> None:
        expected_tag, expected_type, expected_value = fixture
        view, tag, value = C.c_void_p(), C.c_void_p(), C.c_void_p()
        try:
            self.call('kmipkit_ttlv_item_view_from_item', item, C.byref(view))
            self.call('kmipkit_ttlv_item_view_tag', view, C.byref(tag))
            actual_tag, actual_type = C.c_uint32(), C.c_uint8()
            self.call('kmipkit_ttlv_tag_value', tag, C.byref(actual_tag))
            self.call('kmipkit_ttlv_item_view_type', view, C.byref(actual_type))
            self.call('kmipkit_ttlv_item_view_value', view, C.byref(value))
            number = C.c_int32() if expected_type == 2 else C.c_uint32()
            self.call('kmipkit_ttlv_value_view_integer' if expected_type == 2
                      else 'kmipkit_ttlv_value_view_enumeration', value, C.byref(number))
            actual = int(number.value).to_bytes(4, 'big', signed=expected_type == 2)
            if (actual_tag.value, actual_type.value, actual) != fixture:
                raise ValueError('installed_abi_value_mismatch')
        finally:
            for name, handle in (('ttlv_value_view', value), ('tag', tag), ('ttlv_item_view', view)):
                if handle.value:
                    getattr(self.library, f'kmipkit_{name}_release')(handle)


    def execute_reserved_tag(self, symbol: str, tag_code: int) -> bytes:
        self.called.clear()
        wire = tag_code.to_bytes(3, 'big') + bytes.fromhex('02000000040000000700000000')
        raw_tag, checked_tag, item = C.c_void_p(), C.c_void_p(), C.c_void_p()
        try:
            if symbol == 'kmipkit_ttlv_decode':
                error = DecodeError(C.sizeof(DecodeError), 1, 0, 0)
                data = C.create_string_buffer(wire)
                self.called.add(symbol)
                status = self.library.kmipkit_ttlv_decode(data, len(wire), C.byref(error), C.byref(item))
                if status == 0 or item.value or error.kind != 11:
                    raise ValueError('reserved_tag_not_rejected')
            else:
                self.call('kmipkit_ttlv_raw_tag_create', tag_code, C.byref(raw_tag))
                if symbol == 'kmipkit_ttlv_raw_tag_create':
                    actual = C.c_uint32()
                    self.call('kmipkit_ttlv_raw_tag_value', raw_tag, C.byref(actual))
                    if actual.value != tag_code:
                        raise ValueError('reserved_raw_tag_not_preserved')
                elif symbol == 'kmipkit_ttlv_raw_tag_try_checked':
                    self.called.add(symbol)
                    status = self.library.kmipkit_ttlv_raw_tag_try_checked(raw_tag, C.byref(checked_tag))
                    if status == 0 or checked_tag.value:
                        raise ValueError('reserved_tag_not_rejected')
                else:
                    raise ValueError('reserved_tag_checked_constructor_not_representable')
            return wire
        finally:
            for name, handle in (('raw_tag', raw_tag), ('tag', checked_tag), ('ttlv_item', item)):
                if handle.value:
                    getattr(self.library, f'kmipkit_{name}_release')(handle)

    def execute(self, symbol: str, fixture: tuple[int, int, bytes]) -> bytes:
        """Construct and decode independently; compare both public views exactly.

        C exposes no TTLV encoder. The harness encodes the independently supplied
        vector and records no public encode event. Handle transfer follows the
        installed header, including transfers before a late validation failure.
        """
        self.called.clear()
        tag_code, item_type, value_bytes = fixture
        wire = tag_code.to_bytes(3, 'big') + bytes([item_type]) + (4).to_bytes(4, 'big') + value_bytes + bytes(4)
        handles = {name: C.c_void_p() for name in ('codec_limits', 'raw_tag', 'tag', 'ttlv_value', 'ttlv_item', 'ttlv_structure')}
        decoded = C.c_void_p()
        try:
            self.call('kmipkit_codec_limits_defaults', C.byref(handles['codec_limits']))
            self.call('kmipkit_ttlv_raw_tag_create', tag_code, C.byref(handles['raw_tag']))
            self.call('kmipkit_ttlv_raw_tag_try_checked', handles['raw_tag'], C.byref(handles['tag']))
            number = int.from_bytes(value_bytes, 'big', signed=item_type == 2)
            self.call('kmipkit_ttlv_value_integer' if item_type == 2 else 'kmipkit_ttlv_value_enumeration',
                      number, C.byref(handles['ttlv_value']))
            consumed_value = handles['ttlv_value']
            handles['ttlv_value'] = C.c_void_p()
            self.call('kmipkit_ttlv_item_create', handles['tag'], consumed_value,
                      handles['codec_limits'], C.byref(handles['ttlv_item']))
            self.inspect(handles['ttlv_item'], fixture)
            error = DecodeError(C.sizeof(DecodeError), 1, 0, 0)
            data = C.create_string_buffer(wire)
            self.call('kmipkit_ttlv_decode', data, len(wire), C.byref(error), C.byref(decoded))
            self.inspect(decoded, fixture)
            if symbol in {'kmipkit_ttlv_structure_create', 'kmipkit_ttlv_structure_with_item'}:
                self.call('kmipkit_ttlv_structure_create', C.byref(handles['ttlv_structure']))
                original_structure, original_item = handles['ttlv_structure'], handles['ttlv_item']
                handles['ttlv_structure'], handles['ttlv_item'] = C.c_void_p(), C.c_void_p()
                self.call('kmipkit_ttlv_structure_with_item', original_structure, original_item,
                          handles['codec_limits'], C.byref(handles['ttlv_structure']))
            if symbol not in self.called:
                raise ValueError('named_route_not_invoked')
            return wire
        finally:
            if decoded.value:
                self.library.kmipkit_ttlv_item_release(decoded)
            for name, handle in reversed(list(handles.items())):
                if handle.value:
                    getattr(self.library, f'kmipkit_{name}_release')(handle)


LOCAL_SYMBOLS = {
    'kmipkit_ttlv_decode', 'kmipkit_ttlv_item_create', 'kmipkit_ttlv_raw_tag_create',
    'kmipkit_ttlv_raw_tag_try_checked', 'kmipkit_ttlv_value_integer',
    'kmipkit_ttlv_value_enumeration', 'kmipkit_ttlv_structure_create',
    'kmipkit_ttlv_structure_with_item', 'kmipkit_ttlv_item_view_from_item',
}


def event_from_cell(cell: dict, wire: bytes, index: int) -> dict:
    event = cell_fields(cell)
    tree = {'tag': int.from_bytes(wire[:3], 'big'), 'type': wire[3], 'value': wire[8:12].hex()}
    tree_hash = hashlib.sha256(json.dumps(tree, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    event.update(invocation_id=f'c-abi-{index}', status='passed', invocation_count=1,
                 wire_bytes_deterministic=True, wire_bytes_sha256=hashlib.sha256(wire).hexdigest())
    event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = tree_hash
    return event


def run(output: Path, binary: Path | None = None) -> int:
    provenance = verify_installed_artifact()
    library = INSTALL / 'lib/libkmipkit_ffi.so'
    abi = InstalledAbi(library)
    catalog_path = ROOT / 'interop/shared/catalog/kmipkit-1.0-test-catalog.json'
    manifest_path = ROOT / 'interop/shared/coverage/route-capabilities.json'
    catalog, manifest = json.loads(catalog_path.read_text()), json.loads(manifest_path.read_text())
    from probe_route_artifact import probe
    availability = probe(manifest, 'c', INSTALL, provenance)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.with_suffix('.artifact-probe.json').write_text(json.dumps(availability, indent=2) + '\n')
    targets = {target['target_id']: target for target in catalog['normative_targets']}
    cases = {case['case_id']: case for case in catalog['test_cases']}
    tags = {target['name']: int(target['wire_value'], 16) for target in targets.values() if target['kind'] == 'tag'}
    events, pending = [], []
    fixtures = {key: scalar_fixture(target, tags) for key, target in targets.items()}
    from route_ttlv import TreeAbi
    tree_abi = TreeAbi(library)
    boundaries = json.loads((ROOT / 'interop/shared/vectors/ttlv-boundaries.json').read_text())
    boundary_vectors = {int(vector['type_code'], 16): vector for vector in boundaries['vectors']}
    branch_evidence = []
    for index, cell in enumerate(iter_cells(catalog, manifest, languages=('c',))):
        target = targets[cell['target_id']]
        fixture = fixtures[cell['target_id']]
        case = cases[cell['case_id']]
        # Multi-branch semantic cases require dedicated recipes. A scalar check
        # can satisfy only the individual inventory value/tag target case.
        individual_value = case['category'] == 'normative_inventory' and not case.get('requires', {}).get('protocol_scenario')
        if target['kind'] == 'data_type' and cell['transport'] is None:
            result = tree_abi.boundary_case(boundary_vectors[int(target['wire_value'], 16)])
            if cell['public_api_symbol'] in result['invoked_symbols']:
                event = cell_fields(cell)
                event.update(status='passed', invocation_count=1, invocation_id=f'c-boundary-{index}')
                event['response_tree_sha256' if cell['action'] == 'decode' else 'request_tree_sha256'] = result['tree_sha256']
                events.append(event)
                branch_evidence.append({**cell_fields(cell), 'sample_count': result['sample_count'],
                                        'branches': result['branches'], 'tree_sha256': result['tree_sha256']})
                continue
        if (cell['transport'] is None and fixture is not None and individual_value
                and (target['kind'] != 'tag' or target.get('allocation') != 'reserved' or
                     (cell['public_api_symbol'] in
                      {'kmipkit_ttlv_raw_tag_create', 'kmipkit_ttlv_raw_tag_try_checked', 'kmipkit_ttlv_decode'}))
                and cell['public_api_symbol'] in LOCAL_SYMBOLS):
            try:
                if target['kind'] == 'tag' and target.get('allocation') == 'reserved':
                    wire = abi.execute_reserved_tag(cell['public_api_symbol'], fixture[0])
                    event = event_from_cell(cell, wire, index)
                    if cell['action'] == 'decode':
                        event['response_tree_sha256'] = hashlib.sha256(json.dumps({'tag': fixture[0], 'decode_error_kind': 11}, sort_keys=True).encode()).hexdigest()
                    events.append(event)
                else:
                    wire = abi.execute(cell['public_api_symbol'], fixture)
                    events.append(event_from_cell(cell, wire, index))
            except ValueError:
                if cell['public_api_symbol'] not in abi.called:
                    pending.append(pending_cell(cell, 'fixture_preparation_failed'))
                    continue
                event = cell_fields(cell)
                event.update(status='failed', failure_code='installed_abi_call_failed',
                             invocation_id=f'c-abi-{index}', invocation_count=1)
                events.append(event)
        else:
            pending.append(pending_cell(cell, pending_reason(cell)))
    output.parent.mkdir(parents=True, exist_ok=True)
    output.with_suffix('.branches.json').write_text(json.dumps(branch_evidence, indent=2) + '\n')
    if binary is not None:
        from route_peer import execute_network_cells
        network_events, invoked_keys, peer_checks = execute_network_cells(binary, catalog, manifest)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.with_suffix(".peer-checks.json").write_text(json.dumps(peer_checks, indent=2) + "\n")
        events.extend(network_events)
        pending = [cell for cell in pending if _key(cell) not in invoked_keys]
    report = {'schema_version': '1.0', 'run_id': f'c-route-{uuid.uuid4()}', 'language': 'c',
              'server': {'id': 'controlled-c-peer', 'product': 'KMIPKit controlled C route fixture',
                         'version': 'fixture-2', 'configuration_fingerprint': hashlib.sha256(b'controlled-c-route-fixture-v2').hexdigest()},
              'library_commit': PIN, 'artifact_sha256': sha256(library),
              'catalog_sha256': sha256(catalog_path), 'route_manifest_sha256': sha256(manifest_path),
              'protocol_versions': ['2.0', '2.1'],
              'network_combinations': [{'encoding': encoding, 'transport': transport}
                                       for encoding in ('TTLV', 'JSON', 'XML') for transport in ('raw_tls', 'https')],
              'events': events}
    output.parent.mkdir(parents=True, exist_ok=True)
    from jsonschema import Draft202012Validator
    schema = json.loads((ROOT / 'interop/shared/schemas/route-execution-evidence.schema.json').read_text())
    Draft202012Validator(schema).validate(report)
    output.write_text(json.dumps(report, indent=2) + '\n')
    gaps_path = output.with_suffix('.gaps.json')
    gaps_path.write_text(json.dumps({'expected_cells': len(events) + len(pending),
        'invoked_cells': len(events), 'failed_cells': sum(e['status'] != 'passed' for e in events),
        'pending_cells': len(pending), 'pending': pending}, indent=2) + '\n')
    print(json.dumps({'evidence': str(output), 'invoked_cells': len(events), 'pending_cells': len(pending)}))
    try:
        require_complete(events, pending)
        validate_collection([report], catalog, manifest, catalog_sha256=report['catalog_sha256'],
                            route_manifest_sha256=report['route_manifest_sha256'],
                            expected_artifact_sha256=report['artifact_sha256'])
    except ValueError as error:
        print(f'C route matrix incomplete: {error}', file=sys.stderr)
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path)
    args = parser.parse_args()
    return run(args.output, args.binary)


if __name__ == '__main__':
    raise SystemExit(main())
