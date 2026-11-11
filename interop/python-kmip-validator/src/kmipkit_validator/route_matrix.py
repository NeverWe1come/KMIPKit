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
            function = inspect.unwrap(public_callable(symbol))
            code = getattr(function, '__code__', None)
            if code is not None:
                self.codes.setdefault(code, set()).add(symbol)
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
        type_names = {name for name in type_names if {'bytes': 'byte_string', 'text': 'text_string'}.get(_TTLV_TYPE_KINDS[name], _TTLV_TYPE_KINDS[name]) == suffix}
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
        *(f'ttlv_value_{name}' for name in ('structure', 'integer', 'long_integer', 'big_integer',
                                          'enumeration', 'boolean', 'text_string', 'byte_string',
                                          'date_time', 'interval', 'date_time_extended')),
    )
}


def execute_cell(cell: dict, case: dict, target: dict) -> dict:
    if cell['transport'] is None:
        return _generic_cell(cell, case, target)
    return _network_cell(cell, case, target)


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
            symbols = [cell['public_api_symbol']]
            if cell.get('execution_api_symbol'):
                symbols.append(cell['execution_api_symbol'])
            with CallTrace(symbols) as trace:
                observed = execute_cell(cell, cases[cell['case_id']], targets[cell['target_id']])
            if not set(symbols) <= trace.observed:
                raise RecipePending
            report['events'].append({**identity, 'invocation_id': str(uuid.uuid4()),
                                     'invocation_count': 1, 'status': 'passed', **observed})
        except RecipePending:
            gaps.append({**identity, 'failure_code': 'recipe_pending'})
        except Exception:
            # No exception names, sources, native text or raw fixture values leave here.
            gaps.append({**identity, 'failure_code': 'recipe_execution_failed'})
    return report, gaps


def _ping_tree(wire: bytes, encoding: str) -> list:
    """Normalize the complete deterministic Ping envelope, retaining child order."""
    from kmipkit import protocol_values
    from .controlled_peer import parse_children
    import xml.etree.ElementTree as ET

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


def _network_cell(cell: dict, case: dict, target: dict) -> dict:
    from .controlled_peer import ControlledTlsPeer, PKI
    from kmipkit import extensions, protocol_values

    # Other positive operations need stateful, operation-specific fixtures. An
    # OperationNotSupported response cannot establish their positive semantics.
    if (case['case_id'] != 'test-op-ping' or cell['protocol_version'] != '2.1'
            or cell['public_api_symbol'] != 'kmipkit.extensions.client_batch_item_ping'):
        raise RecipePending
    peer = ControlledTlsPeer(encoding=cell['encoding'], transport=cell['transport'], protocol_version='2.1')
    connection = {
        'server_id': 'controlled-fixture', 'protocol_version': '2.1',
        'transport': cell['transport'], 'encoding': cell['encoding'], 'api_level': 'typed-protocol',
        'endpoints': {'raw_tls': {'host': '127.0.0.1', 'port': peer.port},
                      'https': {'url': f'https://127.0.0.1:{peer.port}/kmip'}},
        'tls': {'ca_cert_path': str(PKI / 'server-cert.pem'),
                'client_cert_path': str(PKI / 'client-cert.pem'),
                'client_key_path': str(PKI / 'client-key.pem'), 'server_name_override': 'localhost',
                'connect_timeout_seconds': 5, 'read_timeout_seconds': 5},
    }
    driver = None
    try:
        driver = PythonKmipDriver(connection)
        with extensions.client_batch_item_ping() as item:
            result = driver._exchange(item)
        if result['status'] != 'passed' or result['result_status_code'] != protocol_values.ResultStatus.SUCCESS:
            raise ValueError('fixture response status assertion failed')
    finally:
        if driver is not None:
            driver.close()
        peer.close()
    if len(peer.requests) != 1 or len(peer.responses) != 1:
        raise ValueError('fixture exchange count assertion failed')
    request = _ping_tree(peer.requests[0], cell['encoding'])
    response = _ping_tree(peer.responses[0], cell['encoding'])
    ping = protocol_values.Operation.PING
    expected_request = [0x420078, 1, [
        [0x420077, 1, [[0x420069, 1, [[0x42006A, 2, 2], [0x42006B, 2, 1]]],
                       [0x4200D3, 6, True], [0x42000D, 2, 1]]],
        [0x42000F, 1, [[0x42005C, 5, ping], [0x420079, 1, []]]]]]
    expected_response = [0x42007B, 1, [
        [0x42007A, 1, [[0x420069, 1, [[0x42006A, 2, 2], [0x42006B, 2, 1]]],
                       [0x420092, 9, '<normalized-time-stamp>'], [0x42000D, 2, 1]]],
        [0x42000F, 1, [[0x42005C, 5, ping], [0x42007F, 5, 0], [0x42007C, 1, []]]]]]
    if request != expected_request or response != expected_response:
        raise ValueError('fixture Ping request or response shape assertion failed')
    return {'request_tree_sha256': digest(request), 'response_tree_sha256': digest(response),
            'wire_bytes_deterministic': True,
            'wire_bytes_sha256': hashlib.sha256(peer.requests[0]).hexdigest()}


def verify_installed_artifact(artifact: dict) -> None:
    """Compare every installed package file with the verified exact wheel."""
    from importlib.metadata import distribution
    import zipfile

    package = distribution('kmipkit')
    module = importlib.util.find_spec('kmipkit')
    if (package.version != '1.0.0' or module is None or module.origin is None
            or not Path(module.origin).resolve().is_relative_to(Path(sys.prefix).resolve())):
        raise ValueError('pinned wheel is not installed in the isolated environment')
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
                              catalog_sha256=catalog_sha, route_manifest_sha256=manifest_sha)
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
               'verified_cells': len(report['events']), 'pending_cells': len(gaps), 'complete': passed}
    (output / 'closure.json').write_text(json.dumps(closure, indent=2, sort_keys=True) + '\n')
    print(json.dumps(closure, sort_keys=True))
    return 0 if passed else 1
