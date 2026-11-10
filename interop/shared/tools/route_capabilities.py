#!/usr/bin/env python3
"""Validate the authored public construction-route inventory (KMIPKIT-0092).

Construction and execution identities are separate. Public convenience send
methods taking an already-built request do not create independent routes.
Source inspection establishes candidates; installed-package probes establish
artifact availability. Neither establishes an executable validator recipe.
"""
from __future__ import annotations

import argparse
from functools import lru_cache
import json
from pathlib import Path
import re
import subprocess

import jsonschema

ROOT = Path(__file__).resolve().parents[3]
COVERAGE = ROOT / 'interop/shared/coverage'
LANGUAGES = ('rust', 'c', 'java', 'python')
ASYNC = {'cancel', 'poll', 'process', 'queryasyncrequests'}


def normalized(value):
    return re.sub('[^a-z0-9]', '', value.lower()).replace('asynchronous', 'async')


@lru_cache(maxsize=None)
def source(root, commit, path):
    """Read immutable release input, never an uncommitted source approximation."""
    return subprocess.check_output(['git', '-C', str(root), 'show', f'{commit}:{path}'], text=True)


def function_symbols(api):
    """Resolve both observed declarations and authored ABI function mappings."""
    result = {language: set() for language in LANGUAGES}
    packages = {p['id']: p['language'] for p in api['publicPackages']}
    for declaration in api['publicDeclarations']:
        result[packages[declaration['packageId']]].add(declaration['exportName'])
    for function in api['functions']:
        for language in LANGUAGES:
            mapping = function.get(language, {})
            if language == 'c':
                symbol = mapping.get('symbol')
            elif language == 'rust':
                symbol = '::'.join([mapping.get('module', ''), mapping.get('name', '')]).strip(':')
            elif language == 'java':
                symbol = '.'.join(mapping.get(k, '') for k in ('package', 'class', 'method')).strip('.')
            else:
                symbol = mapping.get('function')
                if symbol and not symbol.startswith(mapping.get('module', '') + '.'):
                    symbol = mapping.get('module', '') + '.' + symbol
            if symbol:
                result[language].add(symbol)
    return result


def construction_inventory(root, commit):
    """Enumerate operation constructors from immutable public source.

    The public API metadata lacks individual Query/Ping Rust declarations and
    overload signatures. This source association fills that metadata gap without
    editing generated declarations. Add new constructor forms here when the
    library surface changes; a missing candidate is a completeness failure.
    """
    api = json.loads(source(root, commit, 'specification/api/public-api.json'))
    operations = {normalized(o['name']): o for o in api['operations'] if o['scope'] == 'client_1_0'}
    paths = subprocess.check_output(['git', '-C', str(root), 'ls-tree', '-r', '--name-only', commit,
                                     'crates/kmipkit-protocol/src'], text=True).splitlines()
    inventory = []
    for path in paths:
        if not path.endswith('.rs'):
            continue
        text = source(root, commit, path)
        for match in re.finditer(r'impl (\w+Request)\s*\{', text):
            request_type = match[1]
            operation = operations.get(normalized(request_type.removesuffix('Request')))
            if not operation:
                continue
            # Balance the impl's braces so a response constructor cannot enter the inventory.
            depth, end = 1, match.end()
            while depth and end < len(text):
                depth += (text[end] == '{') - (text[end] == '}')
                end += 1
            body = text[match.end():end - 1]
            for method in re.findall(r'pub\s+(?:const\s+)?fn\s+(new|try_new|from_managed_object)\s*\(', body):
                inventory.append(('rust', f'kmipkit_protocol::{request_type}::{method}', operation['operationId'], path, 'network'))
    # Batch constructors accept raw operation inputs. Methods which only take an
    # existing Register/Revoke model are adapters; their model constructor below
    # is the independent construction route.
    c_path = 'bindings/c/include/kmipkit.h'
    c_text = source(root, commit, c_path)
    py_path = 'bindings/python/src/kmipkit/extensions.py'
    py_text = source(root, commit, py_path)
    java_path = 'bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItem.java'
    java_text = source(root, commit, java_path)
    for key, op in operations.items():
        if key in {'register', 'revoke'}:
            request_type = key.title() + 'Request'
            inventory.extend([
                ('c', f'kmipkit_{key}_request_create', op['operationId'], c_path, 'network'),
                ('java', f'org.kmipkit.extensions.{request_type}.create', op['operationId'], f'bindings/java/src/main/java/org/kmipkit/extensions/{request_type}.java', 'network'),
                ('python', f'kmipkit.extensions.{request_type}.create', op['operationId'], py_path, 'network')])
            continue
        if key in ASYNC:
            request_type = {'queryasyncrequests': 'QueryAsyncRequestsRequest'}.get(key, key.title() + 'Request')
            snake = 'query_async_requests' if key == 'queryasyncrequests' else key
            candidates = [
                ('c', f'kmipkit_{snake}_request_create', c_path, 'local'),
                ('java', f'org.kmipkit.extensions.{request_type}.create', f'bindings/java/src/main/java/org/kmipkit/extensions/{request_type}.java', 'local'),
                ('python', f'kmipkit.async_operations.{request_type}.create', 'bindings/python/src/kmipkit/async_operations.py', 'local'),
                ('c', f'kmipkit_client_execute_{snake}', c_path, 'network'),
                ('java', f'org.kmipkit.extensions.Client.execute{request_type.removesuffix("Request")}', 'bindings/java/src/main/java/org/kmipkit/extensions/Client.java', 'network'),
                ('python', f'kmipkit.client.Client.execute_{snake}', 'bindings/python/src/kmipkit/client.py', 'network')]
            inventory.extend((lang, symbol, op['operationId'], path, mode) for lang, symbol, path, mode in candidates)
            continue
        # Mapping normalized source names avoids inventing naming conventions
        # for rekey/re-certify, PKCS#11, and importObject.
        for suffix in re.findall(r'int32_t kmipkit_client_batch_item_(\w+)\s*\(', c_text):
            if normalized(suffix) == key:
                inventory.append(('c', f'kmipkit_client_batch_item_{suffix}', op['operationId'], c_path, 'network'))
        for suffix in re.findall(r'^def client_batch_item_(\w+)\(', py_text, re.M):
            if normalized(suffix) == key:
                inventory.append(('python', f'kmipkit.extensions.client_batch_item_{suffix}', op['operationId'], py_path, 'network'))
        for suffix in sorted(set(re.findall(r'public static ClientBatchItem (\w+)\(', java_text))):
            if normalized(suffix.removesuffix('Object')) == key:
                inventory.append(('java', f'org.kmipkit.extensions.ClientBatchItem.{suffix}', op['operationId'], java_path, 'network'))
    inventory.extend([
        ('rust', 'kmipkit_ttlv::Item::new', '', 'crates/kmipkit-ttlv/src/item.rs', 'local'),
        ('rust', 'kmipkit_ttlv::codec::decode', '', 'crates/kmipkit-ttlv/src/codec/decoder.rs', 'local'),
        ('c', 'kmipkit_ttlv_item_create', '', c_path, 'local'),
        ('c', 'kmipkit_ttlv_decode', '', c_path, 'local'),
        ('java', 'org.kmipkit.ttlv.TtlvItem.create', '', 'bindings/java/src/main/java/org/kmipkit/ttlv/TtlvItem.java', 'local'),
        ('java', 'org.kmipkit.ttlv.TtlvCodec.decode', '', 'bindings/java/src/main/java/org/kmipkit/ttlv/TtlvCodec.java', 'local'),
        ('python', 'kmipkit.ttlv.ttlv_item_create', '', 'bindings/python/src/kmipkit/ttlv.py', 'local'),
        ('python', 'kmipkit.ttlv.ttlv_decode', '', 'bindings/python/src/kmipkit/ttlv.py', 'local')])
    return sorted(set(inventory))


def validate(data, root=ROOT):
    schema = json.loads((root / 'interop/shared/coverage/route-capabilities.schema.json').read_text())
    try:
        jsonschema.Draft202012Validator(schema).validate(data)
    except jsonschema.ValidationError as error:
        raise ValueError(f'{list(error.absolute_path)}: {error.message}') from None
    ids, constructors = set(), set()
    commit = data['library_commit']
    api = json.loads(source(root, commit, 'specification/api/public-api.json'))
    symbols = function_symbols(api)
    inventory = construction_inventory(root, commit)
    expected = {(lang, symbol) for lang, symbol, *_ in inventory}
    for route in data['routes']:
        if route['route_id'] in ids:
            raise ValueError('duplicate route_id: ' + route['route_id'])
        ids.add(route['route_id'])
        key = route['language'], route['public_api_symbol']
        if key in constructors:
            raise ValueError('same constructor counted twice: ' + str(key))
        constructors.add(key)
        if key not in expected and route['public_api_symbol'] not in symbols[route['language']]:
            raise ValueError('missing public API symbol: ' + str(key))
        for combination in route['combinations']:
            if route['mode'] == 'local' and combination['transport'] is not None:
                raise ValueError('local construction cannot use a network transport')
            if route['mode'] == 'network' and combination['transport'] is None:
                raise ValueError('network route requires transport')
        if route['artifact_commit'] != commit:
            raise ValueError('route artifact commit differs from manifest')
        for path in route['source_paths']:
            source(root, commit, path)
    missing = expected - constructors
    if missing:
        raise ValueError('missing public construction route: ' + repr(sorted(missing)))
    for limitation in data['unavailable']:
        if (limitation['language'], limitation.get('public_api_symbol')) in constructors:
            raise ValueError('public constructor cannot be unavailable')
    return data


def validate_artifact_identity(data, provenance):
    if provenance.get('library_commit') != data['library_commit']:
        raise ValueError('artifact commit does not match route capabilities')


def validate_installed_symbols(data, language, symbols):
    required = {r['public_api_symbol'] for r in data['routes'] if r['language'] == language}
    missing = required - set(symbols)
    if missing:
        raise ValueError('installed artifact is missing public symbols: ' + repr(sorted(missing)))


def build_manifest(root=ROOT, commit='52a2be180340e5da3166401a52f652dfd368ad37'):
    """Generate constructor identities from the exact package's public sources."""
    categories = ['enums_and_bitmasks', 'attributes', 'object_lifecycle',
        'cryptography_and_rng', 'managed_objects', 'protocol_and_session',
        'version_boundaries', 'server_initiated_operations', 'multipart_crypto',
        'negative_and_boundary', 'batch_workflows', 'asynchronous_workflows']
    data = {'schema_version': '1.0', 'library_commit': commit,
            'public_api_manifest': 'specification/api/public-api.json',
            'routes': [], 'execution_routes': [], 'unavailable': []}
    api = json.loads(source(root, commit, data['public_api_manifest']))
    operations = {o['operationId']: o for o in api['operations']}
    execution = {
        'rust': ('kmipkit_client::Client::execute', 'crates/kmipkit-client/src/execute.rs'),
        'c': ('kmipkit_client_execute', 'bindings/c/include/kmipkit.h'),
        'java': ('org.kmipkit.extensions.Client.execute', 'bindings/java/src/main/java/org/kmipkit/extensions/Client.java'),
        'python': ('kmipkit.client.Client.execute', 'bindings/python/src/kmipkit/client.py')}
    for language, (symbol, path) in execution.items():
        data['execution_routes'].append({'execution_route_id': language + '.execute.typed-batch',
            'language': language, 'public_api_symbol': symbol, 'source_paths': [path]})
    for language, symbol, operation_id, path, mode in construction_inventory(root, commit):
        generic = not operation_id
        family = 'generic-ttlv' if generic else 'typed-protocol'
        parts = symbol.replace('::', '.').split('.')
        suffix = '.'.join(parts[-2:]) if language in {'rust', 'java', 'python'} else parts[-1]
        suffix = re.sub(r'([a-z0-9])([A-Z])', r'\1_\2', suffix).lower()
        route_id = f'{language}.{family}.{suffix}'
        execution_id = None if mode == 'local' else language + '.execute.typed-batch'
        is_async_send = operation_id and normalized(operations[operation_id]['name']) in ASYNC and mode == 'network' and language != 'rust'
        if is_async_send:
            execution_id = language + '.execute.' + normalized(operations[operation_id]['name'])
            data['execution_routes'].append({'execution_route_id': execution_id,
                'language': language, 'public_api_symbol': symbol, 'source_paths': [path]})
        combinations = ([{'encoding': 'TTLV', 'transport': None}] if mode == 'local' else
            [{'encoding': encoding, 'transport': transport}
             for encoding in ('TTLV', 'JSON', 'XML') for transport in ('raw_tls', 'https')])
        data['routes'].append({'route_id': route_id, 'language': language, 'api_level': family,
            'mode': mode, 'public_api_symbol': symbol,
            'operation_ids': [operation_id] if operation_id else [],
            'case_categories': categories, 'protocol_versions': ['2.0', '2.1'],
            'combinations': combinations, 'actions': ['decode'] if generic and 'decode' in symbol else ['construct'],
            'source_paths': [path], 'artifact_commit': commit,
            'execution_route_id': execution_id, 'artifact_verification': 'required'})
    for language, (_, path) in execution.items():
        for capability, reason in [
            ('high-level-constructor', 'Public client convenience send methods accept an existing typed request or batch; no independent high-level request builder is exported.'),
            ('generic-encode', 'The public generic TTLV codec exports decode methods only. The client encodes internally for execution; that encoder is not a public generic API.'),
            ('generic-send', 'The public client sends typed batches or dedicated asynchronous inputs; arbitrary generic TTLV request submission is not exported.')]:
            generic_path = next(r['source_paths'][0] for r in data['routes'] if r['language'] == language and r['api_level'] == 'generic-ttlv' and r['actions'] == ['decode'])
            data['unavailable'].append({'language': language, 'capability': capability,
                'reason': reason, 'source_paths': [path if capability != 'generic-encode' else generic_path]})
    data['routes'].sort(key=lambda route: route['route_id'])
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='regenerate metadata from the exact release source')
    parser.add_argument('--manifest', type=Path, default=COVERAGE / 'route-capabilities.json')
    args = parser.parse_args()
    if args.write:
        args.manifest.write_text(json.dumps(build_manifest(), indent=2) + '\n')
    data = validate(json.loads(args.manifest.read_text()))
    print(json.dumps({'library_commit': data['library_commit'], 'routes': len(data['routes'])}, sort_keys=True))


if __name__ == '__main__':
    main()
