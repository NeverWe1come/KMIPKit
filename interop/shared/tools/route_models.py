"""Public typed model construction candidates and normative applicability.

Metadata supplies language aliases; immutable package sources decide whether a
method actually constructs or modifies a model. Readers and send-only facades
are excluded. A required route still needs a validator recipe that calls its
exact public symbol; metadata is never execution evidence.
"""
from __future__ import annotations

import ast
import re
import subprocess


def norm(value):
    return re.sub('[^a-z0-9]', '', value.lower()).replace('asynchronous', 'async')


def target_ids(targets, names, *, all_attributes=False, all_objects=False, operation_id=None):
    """Associate model/field identities, then expand their normative children."""
    targets = [target for target in targets if 'operation_id' not in target
        or (operation_id is not None and target['operation_id'] == operation_id
            and target.get('role', 'request') == 'request')]
    names = {norm(name) for name in names}
    roots = {t['target_id'] for t in targets if norm(t['name']) in names or norm(t['target_id']) in names}
    if all_attributes:
        roots.update(t['target_id'] for t in targets if t['kind'] in {'attribute', 'attribute_structure'})
    if all_objects:
        roots.update(t['target_id'] for t in targets if t['kind'] in {'object_type', 'object_structure'})
    # Parent families are referenced by values even when a source inventory
    # emits individual values rather than a separate family row.
    for target in targets:
        for parent in target.get('parent_element_ids', []):
            family = parent.replace('KMIPKIT-ELEM-ENUMERATION-', '').replace('KMIPKIT-ELEM-BITMASK-', '')
            if norm(family) in names:
                roots.add(target['target_id'])
    changed = True
    while changed:
        previous = len(roots)
        for target in targets:
            if roots.intersection(target.get('parent_element_ids', [])):
                roots.add(target['target_id'])
        changed = len(roots) != previous
    return sorted(roots)


def model_inventory(root, commit, source, api, targets):
    candidates = {}
    operations = {norm(op['name']): op['operationId'] for op in api['operations'] if op['scope'] == 'client_1_0'}
    operations['setendpointrole'] = 'KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE'
    generated = source(root, commit, 'crates/kmipkit-protocol/src/typed_attributes_generated.rs')
    markers = re.findall(r'element_id: "([^"]+)",\s*source_name: "([^"]+)",\s*marker_name: "([^"]+)"', generated)

    def add(language, symbol, path, names, *, attributes=False, objects=False, operation_id=None):
        if attributes:
            names = [name for _, name, _ in markers] + [marker for _, _, marker in markers]
        ids = target_ids(targets, names, all_objects=objects, operation_id=operation_id)
        if operation_id and any(norm(name) in operations and operations[norm(name)] == operation_id for name in names):
            ids = sorted(set(ids) | {target['target_id'] for target in targets
                if target.get('operation_id') == operation_id and target.get('role', 'request') == 'request'})
        if not ids:
            return
        key = language, symbol
        if key not in candidates:
            candidates[key] = {'language': language, 'public_api_symbol': symbol,
                'source_paths': [path], 'normative_target_ids': ids, 'operation_id': operation_id}
        else:
            candidates[key]['normative_target_ids'] = sorted(set(ids + candidates[key]['normative_target_ids']))

    paths = subprocess.check_output(['git', '-C', str(root), 'ls-tree', '-r', '--name-only', commit,
        'crates/kmipkit-protocol/src', 'crates/kmipkit-client/src'], text=True).splitlines()
    rust_sources = {path: source(root, commit, path) for path in paths if path.endswith('.rs')}
    # Retain the actually exported protocol namespace; facade re-exports are
    # aliases and cannot count as another independent construction path.
    for declaration in api['publicDeclarations']:
        symbol = declaration['exportName']
        package = declaration['packageId']
        if package not in {'rust-kmipkit-protocol', 'rust-kmipkit-client'}:
            continue
        match = re.fullmatch(r'kmipkit_(?:protocol|client)::(\w+)::(\w+)', symbol)
        if not match:
            continue
        owner, method = match.groups()
        if package == 'rust-kmipkit-client' and (owner not in {'ClientBatch', 'ClientBatchItem'} or not method.startswith('with_')):
            continue
        if owner.endswith('Response') or owner.endswith('CredentialObject'):
            continue
        if owner.endswith('Request') and owner != 'CertificateRequest' and method in {'new', 'try_new', 'from_managed_object'}:
            continue
        selected = None
        for path, text in rust_sources.items():
            block = re.search(r'impl\s+' + owner + r'\s*\{', text)
            if block:
                start = block.end()
                depth, end = 1, start
                while depth and end < len(text):
                    depth += (text[end] == '{') - (text[end] == '}')
                    end += 1
                signature = re.search(r'pub\s+(?:const\s+)?fn\s+' + method + r'(?:<[^\n]+>)?\s*\([^{}]*?\)\s*->\s*([^{}]+)\{', text[start:end])
                if signature and (re.search(r'\bSelf\b', signature[1]) or (owner.endswith('Builder') and method == 'build')):
                    selected = path
                    break
            if method == 'try_from_item' and re.search(r'(?:object_wrapper|transparent_key)!\(\s*' + owner + r'\s*,', text):
                selected = path
                break
        if selected is None and method == 'try_new':
            selected = next((p for p, text in rust_sources.items() if re.search(r'dh_key!\(\s*' + owner + r'\s*,', text)), None)
        if selected is None:
            continue
        names = [owner.removesuffix('Builder'), owner.removesuffix('Credential')]
        # OASIS KMIP 2.0/2.1 §§3.4-3.12 define distinct Key Material tables
        # even though their table rows share the label 'Key Material'.
        material_sections = {'TransparentSymmetricKey': '3-4', 'TransparentDsaPrivateKey': '3-5',
            'TransparentDsaPublicKey': '3-6', 'TransparentRsaPrivateKey': '3-7',
            'TransparentRsaPublicKey': '3-8', 'TransparentDhPrivateKey': '3-9',
            'TransparentDhPublicKey': '3-10', 'TransparentEcPrivateKey': '3-11', 'TransparentEcPublicKey': '3-12'}
        if owner in material_sections:
            names.append('KMIPKIT-ELEM-OBJECT-STRUCTURE-' + material_sections[owner] + '-KEY-MATERIAL')
        if method.startswith('with_'):
            names = [method.removeprefix('with_')]
            if method == 'with_request_time_stamp':
                names = ['Time Stamp']
            if method == 'with_correlation_values':
                names = ['Asynchronous Correlation Values']
            if method == 'with_cryptographic_properties':
                names = ['Cryptographic Algorithm', 'Cryptographic Length']
        operation_id = operations.get(norm(owner.removesuffix('Request'))) if owner.endswith('Request') else None
        add('rust', symbol, selected, names, objects=owner == 'ManagedObject', operation_id=operation_id)
        # These are mappings of actual public typed constructors, not inferred
        # parity. The installed probes subsequently check every mapped symbol.
        for language in ('c',):
            mapping = declaration.get('languageMappings', {}).get(language, {})
            mapped = mapping.get('name', '')
            if not mapped or 'NativeExtensionRegistry' in mapped:
                continue
            if language == 'c':
                path = 'bindings/c/include/kmipkit.h'
                if not any(part in mapped for part in ('_from_', '_create', '_try_new', '_set_', '_try_push', '_with_')):
                    continue
                if not re.search(r'\b' + re.escape(mapped) + r'\s*\(', source(root, commit, path)):
                    continue
            elif language == 'python':
                path = 'bindings/python/src/kmipkit/' + mapped.split('.')[1] + '.py'
            else:
                class_parts = mapped.split('.')[:-1]
                if 'Builder' in class_parts:
                    class_parts = class_parts[:class_parts.index('Builder')]
                path = 'bindings/java/src/main/java/' + '/'.join(class_parts) + '.java'
            add(language, mapped, path, names, objects=owner == 'ManagedObject', operation_id=operation_id)

    # Open typed enumeration/mask wrappers that really construct a value.
    for path, text in rust_sources.items():
        for owner in re.findall(r'pub struct (\w+)\(u32\)', text):
            if re.search(r'pub const fn from_raw\(raw: u32\) -> Self', text):
                add('rust', f'kmipkit_protocol::{owner}::from_raw', path, [owner])

    # Specialize Attribute<D> per source marker so an untyped Value cannot
    # satisfy the typed attribute route for another normative target.
    generated = source(root, commit, 'crates/kmipkit-protocol/src/typed_attributes_generated.rs')
    markers = re.findall(r'element_id: "([^"]+)",\s*source_name: "([^"]+)",\s*marker_name: "([^"]+)"', generated)
    attribute_ids = []
    for element_id, name, marker in markers:
        attribute_ids.append(element_id)
        for method in ('new', 'from_item'):
            symbol = f'kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::{marker}>::{method}'
            add('rust', symbol, 'crates/kmipkit-protocol/src/typed_attributes.rs', [name, marker])
        add('rust', f'kmipkit_protocol::AttributeSet::try_push_typed::<kmipkit_protocol::attribute::typed::{marker}>', 'crates/kmipkit-protocol/src/attribute.rs', [name, marker])
    for method in ('try_new', 'try_push'):
        add('rust', f'kmipkit_protocol::AttributeSet::{method}',
            'crates/kmipkit-protocol/src/attribute.rs', [], attributes=True)

    for owner in ('CurrentAttribute', 'NewAttribute'):
        add('rust', f'kmipkit_protocol::{owner}::new',
            'crates/kmipkit-protocol/src/attribute.rs', [], attributes=True)
        for _, name, marker in markers:
            add('rust', f'kmipkit_protocol::{owner}::from_typed::<kmipkit_protocol::attribute::typed::{marker}>',
                'crates/kmipkit-protocol/src/attribute.rs', [name, marker])

    # Inspect binding facades directly: the older function metadata sometimes
    # points at an internal JNI trampoline rather than the installed public API.
    py_modules = ('managed_objects', 'typed_attributes', 'cryptographic_parameters', 'attribute_operations', 'credentials', 'messages', 'client', 'async_operations')
    class_names = {}
    for module in py_modules:
        path = f'bindings/python/src/kmipkit/{module}.py'
        tree = ast.parse(source(root, commit, path))
        for cls in [node for node in tree.body if isinstance(node, ast.ClassDef) and not node.name.startswith('_')]:
            class_names[cls.name] = module
            for method in [node for node in cls.body if isinstance(node, ast.FunctionDef) and not node.name.startswith('_')]:
                decorators = {ast.unparse(d) for d in method.decorator_list}
                returns = ast.unparse(method.returns) if method.returns else ''
                construct = bool({'classmethod', 'staticmethod'} & decorators) and method.name not in {'from_native'}
                modify = (cls.name.endswith('Builder') and (returns in {'Self', cls.name} or method.name == 'build')) or (module != 'client' and method.name.startswith('with_') and returns in {'Self', cls.name})
                batch_modifier = cls.name == 'ClientBatch' and method.name.startswith('set_')
                mutate_attributes = cls.name in {'AttributeSet', 'CurrentAttribute', 'NewAttribute'} and method.name in {'try_push', 'try_push_typed', 'from_typed', 'from_item', 'try_new'}
                if not (construct or modify or mutate_attributes or batch_modifier):
                    continue
                names = [cls.name.removesuffix('Builder'), cls.name.removesuffix('Credential'), cls.name.removesuffix('Request')]
                if batch_modifier:
                    field = method.name.removeprefix('set_')
                    names = [{'ordered': 'Batch Order Option', 'request_timestamp': 'Time Stamp'}.get(field, field)]
                if method.name.startswith('with_'):
                    names = [method.name.removeprefix('with_')]
                    if method.name == 'with_correlation_values':
                        names = ['Asynchronous Correlation Values']
                if cls.name == 'CryptographicParametersBuilder':
                    names = ['Cryptographic Parameters'] if method.name == 'build' else [method.name]
                elif cls.name == 'ManagedObject' and method.name not in {'from_item', 'from_fields'}:
                    names = [method.name]
                add('python', f'kmipkit.{module}.{cls.name}.{method.name}', path, names,
                    attributes=cls.name in {'TypedAttribute', 'AttributeSet', 'CurrentAttribute', 'NewAttribute'},
                    objects=cls.name == 'ManagedObject' and method.name in {'from_item', 'from_fields'},
                    operation_id=operations.get(norm(cls.name.removesuffix('Request'))) if cls.name.endswith('Request') else None)

    java_paths = subprocess.check_output(['git', '-C', str(root), 'ls-tree', '-r', '--name-only', commit,
        'bindings/java/src/main/java/org/kmipkit/ttlv',
        'bindings/java/src/main/java/org/kmipkit/protocol/CryptographicParameters.java',
        'bindings/java/src/main/java/org/kmipkit/credentials',
        'bindings/java/src/main/java/org/kmipkit/message',
        'bindings/java/src/main/java/org/kmipkit/extensions/ClientBatch.java',
        'bindings/java/src/main/java/org/kmipkit/extensions/QueryAsyncRequestsRequest.java',
        'bindings/java/src/main/java/org/kmipkit/extensions/PollRequest.java',
        'bindings/java/src/main/java/org/kmipkit/extensions/CancelRequest.java',
        'bindings/java/src/main/java/org/kmipkit/extensions/ProcessRequest.java',
        'bindings/java/src/main/java/org/kmipkit/extensions/AttributeSet.java'], text=True).splitlines()
    for path in java_paths:
        if not path.endswith('.java'):
            continue
        text = source(root, commit, path)
        package = re.search(r'^package ([^;]+);', text, re.M)[1]
        owner = path.rsplit('/', 1)[1].removesuffix('.java')
        if owner not in class_names and owner not in {'CryptographicParameters'} and not target_ids(targets, [owner]):
            continue
        for match in re.finditer(r'public\s+(static\s+)?([\w<>]+)\s+(\w+)\s*\(', text):
            static, returned, method = match.groups()
            if method == 'fromNative':
                continue
            if not ((static and returned not in {'void', 'boolean', 'int', 'long', 'String'}) or (returned in {'Builder', 'ClientBatch', owner} and not static)):
                continue
            names = [owner, owner.removesuffix('Credential'), owner.removesuffix('Request')]
            if method.startswith('with') and not static:
                names = [method.removeprefix('with')]
                if method == 'withCorrelationValues':
                    names = ['Asynchronous Correlation Values']
            if owner == 'ClientBatch' and not static:
                names = [{'ordered': 'Batch Order Option', 'requestTimestamp': 'Time Stamp', 'setAuthentication': 'Authentication'}.get(method, method)]
            symbol_owner = owner
            if returned == 'Builder' and not static:
                names, symbol_owner = [method], owner + '.Builder'
            elif method == 'build' and not static:
                names, symbol_owner = [owner], owner + '.Builder'
            elif static and returned != owner and returned != 'Builder':
                names = [method]
            elif owner == 'ManagedObject' and method not in {'fromItem', 'fromFields'}:
                names = [method]
            add('java', f'{package}.{symbol_owner}.{method}', path, names,
                attributes=owner in {'TypedAttribute', 'AttributeSet'},
                objects=owner == 'ManagedObject' and method in {'fromItem', 'fromFields'},
                operation_id=operations.get(norm(owner.removesuffix('Request'))) if owner.endswith('Request') else None)

    # Public C constructors for the common field/item models and attribute set.
    c_path = 'bindings/c/include/kmipkit.h'
    for symbol in re.findall(r'int32_t (kmipkit_\w+)\s*\(', source(root, commit, c_path)):
        match = re.fullmatch(r'kmipkit_(\w+)_(from_item|from_ttlv|from_fields(?:_v1)?|from_value|create|try_push(?:_typed)?)', symbol)
        if not match:
            continue
        owner, method = match.groups()
        if package == 'rust-kmipkit-client' and (owner not in {'ClientBatch', 'ClientBatchItem'} or not method.startswith('with_')):
            continue
        if owner not in {'managed_object', 'key_block', 'key_value', 'key_material', 'key_wrapping_data',
            'certificate', 'certificate_request', 'symmetric_key', 'public_key', 'private_key', 'secret_data',
            'opaque_object', 'pgp_key', 'split_key', 'typed_attribute', 'attribute_set', 'cryptographic_parameters', 'request_message', 'response_message'}:
            continue
        add('c', symbol, c_path, [owner], attributes=owner in {'typed_attribute', 'attribute_set'}, objects=owner == 'managed_object')
    for symbol in re.findall(r'int32_t (kmipkit_client_batch_set_\w+)\s*\(', source(root, commit, c_path)):
        field = symbol.removeprefix('kmipkit_client_batch_set_')
        add('c', symbol, c_path, [{'order_option': 'Batch Order Option', 'request_timestamp': 'Time Stamp'}.get(field, field)])
    for candidate in generic_model_inventory(api, targets):
        candidates[candidate['language'], candidate['public_api_symbol']] = candidate
    return sorted(candidates.values(), key=lambda row: (row['language'], row['public_api_symbol']))


def value_inventory(api, targets):
    """Map each assigned enum/mask target to its actual public named constant."""
    declarations = [d for d in api['publicDeclarations'] if d['packageId'] == 'c-kmipkit'
        and d.get('capabilityId') == 'named_protocol_values' and 'cValue' in d]
    indexed = {norm(d['exportName'].removeprefix('KMIPKIT_ENUMERATION_').removeprefix('KMIPKIT_BITMASK_')): d for d in declarations}
    exceptions = {
        'KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-3DES-00000002': 'KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_VALUE_3DES',
        'KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000002': 'KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_POLYNOMIAL_SHARING_GF_216',
        'KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000004': 'KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_POLYNOMIAL_SHARING_GF_28'}
    result = []
    for target in targets:
        if target['kind'] == 'referenced_enum_value':
            declaration = indexed.get(norm(target.get('parent_name', '') + target['name']))
            if declaration is None:
                constructors = {'rust': 'kmipkit_ttlv::Value::enumeration', 'c': 'kmipkit_ttlv_value_enumeration',
                    'java': 'org.kmipkit.ttlv.TtlvValue.Enumeration', 'python': 'kmipkit.ttlv.ttlv_value_enumeration'}
                for language, constructor in constructors.items():
                    result.append({'language': language, 'target_id': target['target_id'], 'public_api_symbol': None,
                        'wire_value': int(target['wire_value'], 16), 'availability': 'numeric-only', 'numeric_constructor': constructor})
                continue
        if target['kind'] not in {'enumeration_value', 'bitmask_value', 'referenced_enum_value'} or target.get('allocation') != 'assigned':
            continue
        target_id = target['target_id']
        stem = target_id.replace('KMIPKIT-ELEM-ENUM-VALUE-', '').replace('KMIPKIT-ELEM-BITMASK-VALUE-', '').rsplit('-', 1)[0]
        if target['kind'] != 'referenced_enum_value':
            declaration = indexed.get(norm(stem))
        if target_id in exceptions:
            declaration = next(d for d in declarations if d['exportName'] == exceptions[target_id])
        if declaration is None:
            raise ValueError('public named value metadata is missing target ' + target_id)
        if declaration['cValue'] != int(target['wire_value'], 16):
            raise ValueError('named public constant differs from normative wire value: ' + target_id)
        for language, mapping in declaration['languageMappings'].items():
            symbol = mapping['name']
            if language == 'rust':
                symbol = symbol.replace('kmipkit::protocol::', 'kmipkit_protocol::')
            result.append({'language': language, 'target_id': target_id,
                'public_api_symbol': symbol, 'wire_value': declaration['cValue'], 'availability': 'named-constant'})
    return sorted(result, key=lambda row: (row['language'], row['target_id']))


def generic_model_inventory(api, targets):
    """Retain each public generic value/tag/structure constructor separately."""
    result = []
    for function in api['functions']:
        name = function['id']
        if name.startswith('ttlv_value_'):
            value_kind = name.removeprefix('ttlv_value_')
            if value_kind not in {'integer', 'long_integer', 'big_integer', 'enumeration', 'boolean',
                'text_string', 'byte_string', 'date_time', 'date_time_extended', 'interval', 'structure'}:
                continue
            ids = [t['target_id'] for t in targets if
                (t['kind'] == 'data_type' and norm(t['name']) == norm(value_kind))
                or norm(t.get('source_encoding', '').split(',')[0]) == norm(value_kind)
                or (value_kind == 'enumeration' and t['kind'] in {'enumeration_value', 'referenced_enum_value'})
                or (value_kind == 'integer' and t['kind'] == 'bitmask_value')]
            rust_symbol = 'kmipkit_ttlv::Value::' + value_kind
            rust_path = 'crates/kmipkit-ttlv/src/value.rs'
        elif name in {'ttlv_structure_create', 'ttlv_structure_with_item'}:
            ids = [t['target_id'] for t in targets if t['kind'].endswith('_structure')
                or (t['kind'] == 'data_type' and t['name'] == 'Structure')]
            rust_symbol = 'kmipkit_ttlv::Structure::' + ('new' if name.endswith('create') else 'try_push')
            rust_path = 'crates/kmipkit-ttlv/src/structure.rs'
        elif name in {'ttlv_raw_tag_create', 'ttlv_raw_tag_try_checked'}:
            ids = [t['target_id'] for t in targets if t['kind'] == 'tag']
            rust_symbol = 'kmipkit_ttlv::RawTag::' + ('new' if name.endswith('create') else 'try_checked')
            rust_path = 'crates/kmipkit-ttlv/src/tag.rs'
        else:
            continue
        if not ids:
            continue
        for language in ('rust', 'c', 'java', 'python'):
            mapping = function[language]
            if language == 'rust':
                symbol, path = rust_symbol, rust_path
            elif language == 'c':
                symbol, path = mapping['symbol'], 'bindings/c/include/kmipkit.h'
            elif language == 'java':
                symbol = '.'.join(mapping[key] for key in ('package', 'class', 'method'))
                path = 'bindings/java/src/main/java/' + symbol.rsplit('.', 1)[0].replace('.', '/') + '.java'
            else:
                symbol = mapping['function']
                if not symbol.startswith(mapping['module'] + '.'):
                    symbol = mapping['module'] + '.' + symbol
                path = 'bindings/python/src/kmipkit/ttlv.py'
            result.append({'language': language, 'public_api_symbol': symbol,
                'source_paths': [path], 'normative_target_ids': sorted(ids), 'api_level': 'generic-ttlv'})
    return result
