"""Typed direct attributes and AttributeSet paths through the installed C ABI.

Fixtures retain source tags and declared encodings from the shared normative
catalog. Cryptographic fixture choices are explicit and affect this harness
only. No semantic policy branch is inferred from a successful model constructor.
"""
from __future__ import annotations

import ctypes as C
import re

from route_ttlv import TreeAbi, digest, encode, normalized
from route_values import norm


class AttributeCases(TreeAbi):
    def __init__(self, library, catalog):
        super().__init__(library)
        self.catalog = catalog
        self.attributes = {norm(t['name']): t for t in catalog['normative_targets'] if t['kind'] == 'attribute'}
        self.tags = {norm(t['name']): int(t['wire_value'], 16) for t in catalog['normative_targets'] if t['kind'] == 'tag'}
        for symbol, types in {
            'kmipkit_typed_attribute_from_item': [C.c_uint32, C.c_void_p, C.c_void_p],
            'kmipkit_typed_attribute_from_value': [C.c_uint32, C.c_void_p, C.c_void_p],
            'kmipkit_typed_attribute_value': [C.c_uint32, C.c_void_p, C.c_void_p],
            'kmipkit_attribute_set_create': [C.c_void_p, C.c_void_p],
            'kmipkit_attribute_set_try_push': [C.c_void_p, C.c_void_p],
            'kmipkit_attribute_set_try_push_typed': [C.c_void_p, C.c_uint32, C.c_void_p],
            'kmipkit_attribute_set_len': [C.c_void_p, C.c_void_p],
            'kmipkit_attribute_set_typed_at': [C.c_void_p, C.c_uint32, C.c_uint64, C.c_void_p],
        }.items():
            self.bind(symbol, types)
        self.library.kmipkit_attribute_set_release.argtypes = [C.c_void_p]
        self.library.kmipkit_attribute_set_release.restype = None

    def enum_values(self, name, version):
        return sorted({int(t['wire_value'], 16) for t in self.catalog['normative_targets']
            if t['kind'] == 'enumeration_value' and version in t['protocol_versions']
            and re.fullmatch('[0-9a-fA-F]{8}', t.get('wire_value', ''))
            and any(norm(parent.removeprefix('KMIPKIT-ELEM-ENUMERATION-')) == norm(name)
                    for parent in t.get('parent_element_ids', []))})

    def scalar(self, name, kind, value):
        sizes = {2: 4, 3: 8, 5: 4, 6: 8, 9: 8, 10: 4, 11: 8}
        if kind in sizes:
            value = int(value).to_bytes(sizes[kind], 'big', signed=int(value) < 0)
        elif isinstance(value, str):
            value = value.encode()
        return self.tags[norm(name)], kind, value

    def enumeration(self, name, version, value=None):
        choices = self.enum_values(name, version)
        if value is None:
            if not choices:
                raise ValueError('fixture_enum_has_no_assigned_value')
            value = choices[0]
        return self.scalar(name, 5, value)

    def structure(self, name, version):
        text = lambda field, value: self.scalar(field, 7, value)
        binary = lambda field, value: self.scalar(field, 8, value)
        enum = lambda field, value=None: self.enumeration(field, version, value)
        if name in ('Name', 'Alternative Name', 'Rotate Name'):
            return [text(name + ' Value', 'fixture-name'), enum(name + ' Type', 1)]
        if name == 'Application Specific Information':
            return [text('Application Namespace', 'fixture.namespace'), text('Application Data', 'fixture-data')]
        if name == 'Cryptographic Domain Parameters':
            return [enum('Recommended Curve', 7)]
        if name == 'Cryptographic Parameters':
            return [enum('Block Cipher Mode', 1), enum('Padding Method', 1), enum('Cryptographic Algorithm', 3)]
        if name == 'Digest':
            return [enum('Hashing Algorithm', 6), binary('Digest Value', bytes(range(32))), enum('Key Format Type', 1)]
        if name == 'Key Value Location':
            return [text('Key Value Location Value', 'https://fixture.invalid/key'), enum('Key Value Location Type')]
        if name == 'Link':
            return [enum('Link Type'), text('Linked Object Identifier', 'fixture-linked-object')]
        if name == 'Revocation Reason':
            return [enum('Revocation Reason Code', 1), text('Revocation Message', 'fixture-revocation')]
        if name == 'Usage Limits':
            return [self.scalar('Usage Limits Total', 3, 1000), self.scalar('Usage Limits Count', 3, 0), enum('Usage Limits Unit')]
        if name == 'Vendor Attribute':
            return [text('Vendor Identification', 'fixture_vendor'), text('Attribute Name', 'fixture_attribute'), text('Attribute Value', 'fixture-value')]
        if name.startswith('X.509 Certificate'):
            distinguished_name = bytes.fromhex('30123110300e06035504030c0746697874757265')
            if name.endswith('Identifier'):
                return [binary('Issuer Distinguished Name', distinguished_name), binary('Certificate Serial Number', b'\x01')]
            field = 'Issuer Distinguished Name' if name.endswith('Issuer') else 'Subject Distinguished Name'
            return [binary(field, distinguished_name)]
        if name == 'Random Number Generator':
            return [enum('RNG Algorithm')]
        raise ValueError('structured_attribute_fixture_pending')

    def fixtures(self, target, version):
        name = target['name']
        if target['kind'] in ('enumeration_value', 'bitmask_value'):
            parent = target.get('parent_element_ids', [''])[0]
            name = re.sub('^KMIPKIT-ELEM-(ENUMERATION|BITMASK)-', '', parent)
        if target['kind'] == 'attribute' and norm(name) == norm('Vendor Attribute'):
            return [(self.tags[norm('Attribute')], 1, self.structure('Vendor Attribute', version))]
        attribute = self.attributes.get(norm(name))
        if attribute is None or norm(attribute['name']) not in self.tags:
            return None
        name = attribute['name']
        tag = self.tags[norm(name)]
        if target['kind'] in ('enumeration_value', 'bitmask_value'):
            raw = target.get('wire_value', '')
            if not re.fullmatch('[0-9A-Fa-f]{8}', raw):
                return None
            return [(tag, 2 if target['kind'] == 'bitmask_value' else 5, int(raw, 16).to_bytes(4, 'big'))]
        encoding = attribute.get('source_encoding') or attribute.get('attribute_policy', {}).get('value_type')
        if encoding is None:
            return None
        if norm(encoding) in ('structure', 'rngparameters'):
            return [(tag, 1, self.structure(name, version))]
        if norm(encoding) == 'boolean':
            return [self.scalar(name, 6, value) for value in (0, 1)]
        if norm(encoding) == 'enumeration':
            values = self.enum_values(name, version)
            if not values:
                values = [0x80000000]  # Opaque Data Type §11.35 has extension allocation only.
            return [self.scalar(name, 5, value) for value in [*values, 0xFFFFFFFF]]
        if norm(encoding) == 'integer':
            values = [128] if name == 'Cryptographic Length' else [2048] if name == 'Certificate Length' else [0, 1, 2147483647]
            return [self.scalar(name, 2, value) for value in values]
        if norm(encoding) == 'datetime':
            return [self.scalar(name, 9, value) for value in (-1, 0, 1, 1700000000)]
        if norm(encoding) == 'interval':
            return [self.scalar(name, 10, value) for value in (0, 1, 0xFFFFFFFF)]
        if norm(encoding) == 'bytestring':
            return [self.scalar(name, 8, bytes(range(length))) for length in (0, 1, 7, 8, 9)]
        if norm(encoding) == 'textstring':
            return [self.scalar(name, 7, value) for value in ('', 'x', 'fixture', 'fixture8', 'fixture-9', 'prueba-ñ')]
        if norm(encoding) == 'textstringenumerationorinteger':
            return [self.scalar(name, 7, 'fixture-identifier'), self.scalar(name, 5, 1), self.scalar(name, 2, 42)]
        return None

    def make_structure(self, children, limits):
        structure = C.c_void_p()
        try:
            self.call('kmipkit_ttlv_structure_create', C.byref(structure))
            for child in children:
                item = self.construct(child, limits)
                original, structure = structure, C.c_void_p()
                self.call('kmipkit_ttlv_structure_with_item', original, item, limits, C.byref(structure))
            result, structure = structure, C.c_void_p()
            return result
        finally:
            self.release('ttlv_structure', structure)

    def check_set(self, attributes, tree):
        count, item = C.c_uint64(), C.c_void_p()
        try:
            self.call('kmipkit_attribute_set_len', attributes, C.byref(count))
            if count.value != 1:
                raise ValueError('attribute_set_count_mismatch')
            self.call('kmipkit_attribute_set_typed_at', attributes, tree[0], 0, C.byref(item))
            if self.read_item(item) != tree:
                raise ValueError('attribute_set_value_mismatch')
        finally:
            self.release('ttlv_item', item)

    def positive(self, tree):
        self.called.clear()
        limits, item, value, typed_item, source, attributes = (C.c_void_p() for _ in range(6))
        try:
            self.call('kmipkit_codec_limits_defaults', C.byref(limits))
            item = self.construct(tree, limits)
            self.call('kmipkit_typed_attribute_from_item', tree[0], item, C.byref(typed_item))
            if self.read_item(typed_item) != tree:
                raise ValueError('typed_attribute_item_copy_mismatch')
            self.release('ttlv_item', typed_item)
            typed_item = C.c_void_p()
            self.call('kmipkit_typed_attribute_value', tree[0], item, C.byref(value))
            self.call('kmipkit_typed_attribute_from_value', tree[0], value, C.byref(typed_item))
            if self.read_item(typed_item) != tree:
                raise ValueError('typed_attribute_value_copy_mismatch')
            for mode in ('create', 'push', 'typed_push'):
                source = self.make_structure([tree] if mode == 'create' else [], limits)
                self.call('kmipkit_attribute_set_create', source, C.byref(attributes))
                if mode == 'push':
                    self.call('kmipkit_attribute_set_try_push', attributes, item)
                elif mode == 'typed_push':
                    self.call('kmipkit_attribute_set_try_push_typed', attributes, tree[0], item)
                self.check_set(attributes, tree)
                self.release('attribute_set', attributes)
                self.release('ttlv_structure', source)
                attributes, source = C.c_void_p(), C.c_void_p()
            # Compare independent generic decode and all constructor outputs.
            symbols = set(self.called)
            generic = self.roundtrip(encode(tree))
            symbols.update(generic['invoked_symbols'])
            return generic['normalized_tree'], symbols
        finally:
            for name, handle in (('attribute_set', attributes), ('ttlv_structure', source), ('ttlv_item', typed_item),
                                 ('ttlv_value', value), ('ttlv_item', item), ('codec_limits', limits)):
                self.release(name, handle)

    def negative(self, tree):
        self.called.clear()
        limits, item, wrong, value, source, attributes, output = (C.c_void_p() for _ in range(7))
        try:
            self.call('kmipkit_codec_limits_defaults', C.byref(limits))
            item = self.construct(tree, limits)
            if tree[1] in (2, 7, 5):
                wrong_tree = (tree[0], 6, (1).to_bytes(8, 'big'))
                self.call('kmipkit_ttlv_value_boolean', 1, C.byref(value))
            else:
                wrong_tree = (tree[0], 2, (7).to_bytes(4, 'big'))
                self.call('kmipkit_ttlv_value_integer', 7, C.byref(value))
            wrong = self.construct(wrong_tree, limits)
            status = self.library.kmipkit_typed_attribute_from_item(tree[0], wrong, C.byref(output))
            if status == 0 or output.value:
                raise ValueError('wrong_typed_attribute_item_accepted')
            status = self.library.kmipkit_typed_attribute_from_value(tree[0], value, C.byref(output))
            if status == 0 or output.value:
                raise ValueError('wrong_typed_attribute_value_accepted')
            source = self.make_structure([wrong_tree], limits)
            status = self.library.kmipkit_attribute_set_create(source, C.byref(output))
            if status == 0 or output.value:
                raise ValueError('wrong_attribute_set_value_accepted')
            self.release('ttlv_structure', source)
            source = self.make_structure([tree], limits)
            self.call('kmipkit_attribute_set_create', source, C.byref(attributes))
            for symbol, args in (
                ('kmipkit_attribute_set_try_push', (attributes, wrong)),
                ('kmipkit_attribute_set_try_push_typed', (attributes, tree[0], wrong)),
            ):
                status = getattr(self.library, symbol)(*args)
                if status == 0:
                    raise ValueError('wrong_attribute_push_accepted')
                self.check_set(attributes, tree)
            return 5
        finally:
            for name, handle in (('ttlv_item', output), ('attribute_set', attributes), ('ttlv_structure', source),
                                 ('ttlv_value', value), ('ttlv_item', wrong), ('ttlv_item', item), ('codec_limits', limits)):
                self.release(name, handle)

    def execute(self, target, version):
        trees = self.fixtures(target, version)
        if not trees:
            return None
        normalized_trees, symbols = [], set()
        for tree in trees:
            actual, observed = self.positive(tree)
            normalized_trees.append(actual)
            symbols.update(observed)
        negatives = self.negative(trees[0])
        return {'tree_sha256': digest(normalized_trees), 'invoked_symbols': symbols,
                'sample_count': len(trees), 'negative_checks': negatives}
