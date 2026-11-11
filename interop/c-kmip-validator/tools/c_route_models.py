"""Typed C model recipes backed by the installed public ABI."""
from __future__ import annotations

import ctypes as C
from pathlib import Path

from route_ttlv import TreeAbi, digest, normalized


CERTIFICATE_STRUCTURE = 'KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE'
CERTIFICATE_TYPE_TAG = 0x42001D
CERTIFICATE_VALUE_TAG = 0x42001E
CERTIFICATE_TAG = 0x420013
CERTIFICATE_OBJECT_TYPE = 1
SYMMETRIC_KEY_STRUCTURE = 'KMIPKIT-ELEM-OBJECT-STRUCTURE-SYMMETRIC-KEY'
SYMMETRIC_KEY_TAG = 0x42008F
KEY_BLOCK_TAG = 0x420040
KEY_FORMAT_TYPE_TAG = 0x420042
SYMMETRIC_KEY_OBJECT_TYPE = 2
KEY_BLOCK_STRUCTURE = 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-BLOCK'
KEY_VALUE_STRUCTURE = 'KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE'


class _CryptographicParametersFieldsV1(C.Structure):
    _fields_ = [
        ('struct_size', C.c_uint32), ('abi_version', C.c_uint32),
        ('block_cipher_mode_present', C.c_uint8), ('block_cipher_mode', C.c_uint32),
        ('padding_method_present', C.c_uint8), ('padding_method', C.c_uint32),
        ('hashing_algorithm_present', C.c_uint8), ('hashing_algorithm', C.c_uint32),
        ('key_role_type_present', C.c_uint8), ('key_role_type', C.c_uint32),
        ('digital_signature_algorithm_present', C.c_uint8), ('digital_signature_algorithm', C.c_uint32),
        ('cryptographic_algorithm_present', C.c_uint8), ('cryptographic_algorithm', C.c_uint32),
        ('random_iv_present', C.c_uint8), ('random_iv', C.c_uint8),
        ('iv_length_present', C.c_uint8), ('iv_length', C.c_int32),
        ('tag_length_present', C.c_uint8), ('tag_length', C.c_int32),
        ('fixed_field_length_present', C.c_uint8), ('fixed_field_length', C.c_int32),
        ('invocation_field_length_present', C.c_uint8), ('invocation_field_length', C.c_int32),
        ('counter_length_present', C.c_uint8), ('counter_length', C.c_int32),
        ('initial_counter_value_present', C.c_uint8), ('initial_counter_value', C.c_int32),
        ('salt_length_present', C.c_uint8), ('salt_length', C.c_int32),
        ('mask_generator_present', C.c_uint8), ('mask_generator', C.c_uint32),
        ('mask_generator_hashing_algorithm_present', C.c_uint8),
        ('mask_generator_hashing_algorithm', C.c_uint32),
        ('p_source_present', C.c_uint8), ('p_source', C.POINTER(C.c_uint8)),
        ('p_source_length', C.c_uint64), ('trailer_field_present', C.c_uint8),
        ('trailer_field', C.c_int32),
    ]


class ModelCases:
    """Execute catalog-backed typed object constructors and item conversions."""

    def __init__(self, library: Path, catalog: dict):
        self.abi = TreeAbi(library)
        self.catalog = catalog

    def _read_and_check(self, item: C.c_void_p, expected: list, failure_code: str) -> list:
        actual = normalized(self.abi.read_item(item))
        if actual != expected:
            raise ValueError(failure_code)
        return actual

    def _from_fields(self, symbol: str, trees: list[tuple], limits: C.c_void_p,
                     owned: list[C.c_void_p], prefix: tuple = ()) -> C.c_void_p:
        fields = [self.abi.construct(tree, limits) for tree in trees]
        owned.extend(fields)
        field_array = (C.c_void_p * len(fields))(*fields)
        output = C.c_void_p()
        if prefix:
            signature = [C.c_uint32, C.POINTER(C.c_void_p), C.c_uint64, C.POINTER(C.c_void_p)]
        else:
            signature = [C.POINTER(C.c_void_p), C.c_uint64, C.POINTER(C.c_void_p)]
        self.abi.bind(symbol, signature)
        self.abi.call(symbol, *prefix, field_array, len(fields), C.byref(output))
        owned.append(output)
        return output

    def _from_item(self, symbol: str, item: C.c_void_p, owned: list[C.c_void_p],
                   prefix: tuple = ()) -> C.c_void_p:
        output = C.c_void_p()
        if prefix:
            signature = [C.c_uint32, C.c_void_p, C.POINTER(C.c_void_p)]
        else:
            signature = [C.c_void_p, C.POINTER(C.c_void_p)]
        self.abi.bind(symbol, signature)
        self.abi.call(symbol, *prefix, item, C.byref(output))
        owned.append(output)
        return output

    def _exercise_key_block(self, tree: tuple, limits: C.c_void_p,
                             owned: list[C.c_void_p]) -> None:
        fields = self._from_fields('kmipkit_key_block_from_fields', tree[2], limits, owned)
        converted = self._from_item('kmipkit_key_block_from_item', fields, owned)
        self._read_and_check(converted, normalized(tree), 'key_block_item_tree_mismatch')

    def _execute_symmetric_key(self, protocol_version: str) -> dict:
        key_block_tree = self._key_block_tree(protocol_version)
        limits = C.c_void_p()
        owned: list[C.c_void_p] = []
        try:
            self.abi.call('kmipkit_codec_limits_defaults', C.byref(limits))
            self._exercise_key_block(key_block_tree, limits, owned)
            expected_key_block = normalized(key_block_tree)
            expected = [SYMMETRIC_KEY_TAG, 1, [expected_key_block]]

            symmetric = self._from_fields('kmipkit_symmetric_key_from_fields', [key_block_tree], limits, owned)
            normalized_tree = self._read_and_check(symmetric, expected, 'symmetric_key_fields_tree_mismatch')
            managed = self._from_fields('kmipkit_managed_object_from_fields', [key_block_tree],
                limits, owned, (SYMMETRIC_KEY_OBJECT_TYPE,))
            self._read_and_check(managed, expected, 'managed_symmetric_key_fields_tree_mismatch')
            managed_item = self._from_item('kmipkit_managed_object_from_item', symmetric, owned,
                                           (SYMMETRIC_KEY_OBJECT_TYPE,))
            self._read_and_check(managed_item, expected, 'managed_symmetric_key_item_tree_mismatch')
            return {'sample_count': 1, 'normalized_tree': normalized_tree,
                    'tree_sha256': digest(normalized_tree), 'invoked_symbols': set(self.abi.called)}
        finally:
            for handle in reversed(owned):
                self.abi.release('ttlv_item', handle)
            self.abi.release('codec_limits', limits)

    def _execute_key_block_tree(self, key_block_tree: tuple) -> dict:
        limits = C.c_void_p()
        owned: list[C.c_void_p] = []
        try:
            self.abi.call('kmipkit_codec_limits_defaults', C.byref(limits))
            self._exercise_key_block(key_block_tree, limits, owned)
            normalized_tree = normalized(key_block_tree)
            return {'sample_count': 1, 'normalized_tree': normalized_tree,
                    'tree_sha256': digest(normalized_tree), 'invoked_symbols': set(self.abi.called)}
        finally:
            for handle in reversed(owned):
                self.abi.release('ttlv_item', handle)
            self.abi.release('codec_limits', limits)

    def _execute_key_block(self, protocol_version: str) -> dict:
        return self._execute_key_block_tree(self._key_block_tree(protocol_version))

    def _key_wrapping_data_tree(self, protocol_version: str) -> tuple:
        unique_identifier = (self._tag('Unique Identifier'), 7, b'wrapped-key')
        parameters = (self._tag('Cryptographic Parameters'), 1, [
            self._enum_tree('Block Cipher Mode', 'Block Cipher Mode', 'CBC', protocol_version),
        ])
        encryption_info = (self._tag('Encryption Key Information'), 1, [
            unique_identifier, parameters,
        ])
        mac_signature_info = (self._tag('MAC/Signature Key Information'), 1, [
            unique_identifier, parameters,
        ])
        return (self._tag('Key Wrapping Data'), 1, [
            self._enum_tree('Wrapping Method', 'Wrapping Method', 'Encrypt', protocol_version),
            encryption_info,
            mac_signature_info,
            (self._tag('MAC/Signature'), 8, b'mac'),
            (self._tag('IV/Counter/Nonce'), 8, b'iv'),
            self._enum_tree('Encoding Option', 'Encoding Option', 'TTLV Encoding', protocol_version),
        ])

    def _key_material_section(self, target: dict) -> int | None:
        identities = [target.get('target_id', ''), *target.get('parent_element_ids', [])]
        for section in range(4, 13):
            if any(f'-3-{section}-KEY-MATERIAL' in identity for identity in identities):
                return section
        return None

    def _transparent_key_material(self, section: int, protocol_version: str) -> tuple[int, tuple]:
        formats = {
            4: ('Transparent Symmetric Key', [('Key', 8, b'\xa1')]),
            5: ('Transparent DSA Private Key', [('P', 4, b'\x01'), ('Q', 4, b'\x01'),
                                                ('G', 4, b'\x01'), ('X', 4, b'\x01')]),
            6: ('Transparent DSA Public Key', [('P', 4, b'\x01'), ('Q', 4, b'\x01'),
                                               ('G', 4, b'\x01'), ('Y', 4, b'\x01')]),
            7: ('Transparent RSA Private Key', [
                ('Modulus', 4, b'\x01'), ('Public Exponent', 4, b'\x01\x00\x01'),
                ('Private Exponent', 4, b'\x01'), ('P', 4, b'\x01'), ('Q', 4, b'\x01'),
                ('Prime Exponent P', 4, b'\x01'), ('Prime Exponent Q', 4, b'\x01'),
                ('CRT Coefficient', 4, b'\x01'),
            ]),
            8: ('Transparent RSA Public Key', [('Modulus', 4, b'\x01'),
                                                ('Public Exponent', 4, b'\x01\x00\x01')]),
            9: ('Transparent DH Private Key', [
                ('P', 4, b'\x01'), ('Q', 4, b'\x01'), ('G', 4, b'\x01'),
                ('J', 4, b'\x01'), ('X', 4, b'\x01'),
            ]),
            10: ('Transparent DH Public Key', [
                ('P', 4, b'\x01'), ('Q', 4, b'\x01'), ('G', 4, b'\x01'),
                ('J', 4, b'\x01'), ('Y', 4, b'\x01'),
            ]),
            11: ('Transparent EC Private Key', [
                ('Recommended Curve', 5, self._enum_value('Recommended Curve', 'P-256', protocol_version)
                 .to_bytes(4, 'big')), ('D', 4, b'\x01'),
            ]),
            12: ('Transparent EC Public Key', [
                ('Recommended Curve', 5, self._enum_value('Recommended Curve', 'P-256', protocol_version)
                 .to_bytes(4, 'big')), ('Q String', 8, b'\x04\x01\x02'),
            ]),
        }
        name, members = formats[section]
        key_format = self._enum_value('Key Format Type', name, protocol_version)
        key_material = (self._tag('Key Material'), 1,
                        [(self._tag(member), item_type, value) for member, item_type, value in members])
        return key_format, key_material

    def _execute_key_block_target(self, target: dict, protocol_version: str) -> dict | None:
        kind = target.get('kind')
        name = target.get('name', '').rstrip(',')
        wire_value = target.get('wire_value', '')
        if kind == 'enumeration_value' and 'X' in wire_value.upper():
            boundaries = (int(wire_value.replace('X', '0').replace('x', '0'), 16),
                          int(wire_value.replace('X', 'F').replace('x', 'f'), 16))
            samples = [self._execute_key_block_target(
                {**target, 'wire_value': f'{value:08X}'}, protocol_version) for value in boundaries]
            return self._aggregate_model_samples(samples)
        key_format = self._enum_value('Key Format Type', 'Raw', protocol_version)
        fields = []
        expected_value = None
        target_id = target.get('target_id', '')
        parents = set(target.get('parent_element_ids', []))
        key_material_section = self._key_material_section(target)
        if key_material_section is not None:
            key_format, key_material = self._transparent_key_material(key_material_section, protocol_version)
            fields.append((self._tag('Key Value'), 1, [key_material]))
        elif kind in {'tag', 'structure_member'} and name == 'Key Material':
            fields.append(self._key_value_tree())
        if kind == 'object_structure':
            if target.get('target_id') == KEY_BLOCK_STRUCTURE:
                pass
            elif target.get('target_id') == KEY_VALUE_STRUCTURE:
                fields.append(self._key_value_tree())
            elif key_material_section is not None:
                pass
            elif target.get('name') in {'Key Wrapping Data', 'Encryption Key Information',
                                        'MAC/Signature Key Information'}:
                fields.extend([
                    (self._tag('Key Value'), 8, b'\xa1'),
                    self._key_wrapping_data_tree(protocol_version),
                ])
            else:
                return None
        elif kind == 'enumeration_value':
            parents = set(target.get('parent_element_ids', []))
            wire_value = target.get('wire_value', '')
            if 'KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM' in parents:
                if 'X' in wire_value.upper():
                    return None
                expected_value = int(wire_value, 16)
                fields.append((self._tag('Cryptographic Algorithm'), 5,
                               expected_value.to_bytes(4, 'big')))
                fields.append(self._integer_tree('Cryptographic Length', 128))
            elif 'KMIPKIT-ELEM-ENUMERATION-KEY-FORMAT-TYPE' in parents:
                if 'X' in wire_value.upper():
                    return None
                key_format = int(wire_value, 16)
                expected_value = key_format
            elif 'KMIPKIT-ELEM-ENUMERATION-KEY-COMPRESSION-TYPE' in parents:
                if 'X' in wire_value.upper():
                    return None
                key_format = self._enum_value('Key Format Type', 'Transparent EC Public Key', protocol_version)
                expected_value = int(wire_value, 16)
                fields.append((self._tag('Key Compression Type'), 5,
                               expected_value.to_bytes(4, 'big')))
            else:
                return None
        elif kind in {'tag', 'structure_member', 'attribute'}:
            if name == 'Key Block':
                pass
            elif key_material_section is not None:
                pass
            elif name == 'Key Material':
                pass
            elif name == 'Cryptographic Algorithm':
                expected_value = self._enum_value(name, 'AES', protocol_version)
                fields.append(self._enum_tree(name, name, 'AES', protocol_version))
                fields.append(self._integer_tree('Cryptographic Length', 128))
            elif name == 'Cryptographic Length':
                expected_value = 128
                fields.append(self._enum_tree('Cryptographic Algorithm', 'Cryptographic Algorithm',
                                              'AES', protocol_version))
                fields.append(self._integer_tree(name, expected_value))
            elif name == 'Key Format Type':
                expected_value = key_format
            elif name == 'Key Value':
                fields.append(self._key_value_tree())
            elif name in {'Key Wrapping Data', 'Encryption Key Information',
                          'MAC/Signature Key Information', 'Unique Identifier',
                          'Cryptographic Parameters', 'Wrapping Method', 'MAC/Signature',
                          'IV/Counter/Nonce', 'Encoding Option'}:
                fields.extend([
                    (self._tag('Key Value'), 8, b'\xa1'),
                    self._key_wrapping_data_tree(protocol_version),
                ])
            elif name == 'Key Compression Type':
                key_format = self._enum_value('Key Format Type', 'Transparent EC Public Key', protocol_version)
                expected_value = self._enum_value(name, 'EC Public Key Type Uncompressed', protocol_version)
                fields.append(self._enum_tree(name, name, 'EC Public Key Type Uncompressed', protocol_version))
            else:
                return None
        else:
            return None

        key_block_tree = self._key_block_tree(protocol_version, key_format=key_format,
                                              extra_fields=fields)
        result = self._execute_key_block_tree(key_block_tree)
        if kind == 'object_structure':
            target_tag = self._tag(name)
            if not self._tree_contains_tag(result['normalized_tree'], target_tag):
                return None
        elif kind == 'enumeration_value':
            field_name = 'Key Format Type' if 'KEY-FORMAT-TYPE' in ' '.join(target.get('parent_element_ids', [])) else (
                'Key Compression Type' if 'KEY-COMPRESSION-TYPE' in ' '.join(target.get('parent_element_ids', []))
                else 'Cryptographic Algorithm')
            expected_tag = self._tag(field_name)
            expected_hex = f'{expected_value:08x}'
            if not self._tree_contains_item(result['normalized_tree'], expected_tag, 5, expected_hex):
                return None
        elif kind in {'tag', 'structure_member', 'attribute'}:
            target_tag = self._tag(name)
            if not self._tree_contains_tag(result['normalized_tree'], target_tag):
                return None
            if expected_value is not None:
                item_type = 2 if name == 'Cryptographic Length' else 5
                expected_hex = (int(expected_value).to_bytes(4, 'big', signed=item_type == 2).hex())
                if not self._tree_contains_item(result['normalized_tree'], target_tag, item_type, expected_hex):
                    return None
        return result

    def _tree_contains_tag(self, tree: list, tag: int) -> bool:
        return tree[0] == tag or (tree[1] == 1 and any(self._tree_contains_tag(child, tag) for child in tree[2]))

    def _tree_contains_item(self, tree: list, tag: int, kind: int, value: str) -> bool:
        if tree[0] == tag and tree[1] == kind and tree[2] == value:
            return True
        return tree[1] == 1 and any(self._tree_contains_item(child, tag, kind, value) for child in tree[2])

    def _execute_key_value(self, protocol_version: str) -> dict:
        key_format = self._enum_value('Key Format Type', 'Raw', protocol_version)
        key_value_tree = self._key_value_tree()
        limits = C.c_void_p()
        owned: list[C.c_void_p] = []
        try:
            self.abi.call('kmipkit_codec_limits_defaults', C.byref(limits))
            item = self._from_fields('kmipkit_key_value_from_fields', key_value_tree[2],
                                     limits, owned, (key_format,))
            converted = self._from_item('kmipkit_key_value_from_item', item, owned, (key_format,))
            normalized_tree = self._read_and_check(converted, normalized(key_value_tree),
                                                   'key_value_item_tree_mismatch')
            return {'sample_count': 1, 'normalized_tree': normalized_tree,
                    'tree_sha256': digest(normalized_tree), 'invoked_symbols': set(self.abi.called)}
        finally:
            for handle in reversed(owned):
                self.abi.release('ttlv_item', handle)
            self.abi.release('codec_limits', limits)

    def _crypto_enum_field(self, target: dict) -> tuple[str, str] | None:
        parents = set(target.get('parent_element_ids', []))
        fields = {
            'KMIPKIT-ELEM-ENUMERATION-BLOCK-CIPHER-MODE': ('block_cipher_mode', 'Block Cipher Mode'),
            'KMIPKIT-ELEM-ENUMERATION-PADDING-METHOD': ('padding_method', 'Padding Method'),
            'KMIPKIT-ELEM-ENUMERATION-HASHING-ALGORITHM': ('hashing_algorithm', 'Hashing Algorithm'),
            'KMIPKIT-ELEM-ENUMERATION-KEY-ROLE-TYPE': ('key_role_type', 'Key Role Type'),
            'KMIPKIT-ELEM-ENUMERATION-DIGITAL-SIGNATURE-ALGORITHM':
                ('digital_signature_algorithm', 'Digital Signature Algorithm'),
            'KMIPKIT-ELEM-ENUMERATION-CRYPTOGRAPHIC-ALGORITHM':
                ('cryptographic_algorithm', 'Cryptographic Algorithm'),
            'KMIPKIT-ELEM-ENUMERATION-MASK-GENERATOR': ('mask_generator', 'Mask Generator'),
        }
        matches = [value for parent, value in fields.items() if parent in parents]
        return matches[0] if len(matches) == 1 else None

    def _crypto_target_field(self, target: dict, protocol_version: str) -> tuple[str, str, int, object] | None:
        if target.get('kind') == 'enumeration_value':
            selected = self._crypto_enum_field(target)
            wire_value = target.get('wire_value', '')
            if selected is None:
                return None
            field_name, tag_name = selected
            if 'X' in wire_value.upper():
                return (field_name, tag_name, 5,
                        (int(wire_value.replace('X', '0').replace('x', '0'), 16),
                         int(wire_value.replace('X', 'F').replace('x', 'f'), 16)))
            return field_name, tag_name, 5, int(wire_value, 16)

        if target.get('kind') == 'attribute':
            name = target.get('name')
            if name == 'Cryptographic Parameters':
                return ('block_cipher_mode', 'Block Cipher Mode', 5,
                        self._enum_value('Block Cipher Mode', 'CBC', protocol_version))
            if name == 'Cryptographic Algorithm':
                return ('cryptographic_algorithm', name, 5,
                        self._enum_value(name, 'AES', protocol_version))
            if name == 'Digital Signature Algorithm':
                return ('digital_signature_algorithm', name, 5,
                        self._enum_value(name, 'ECDSA with SHA256', protocol_version))
            return None

        if target.get('kind') not in {'tag', 'structure_member'}:
            return None
        name = target.get('name', '').rstrip(',')
        if name == 'Cryptographic Parameters':
            if target.get('kind') == 'tag':
                return None
            return ('block_cipher_mode', 'Block Cipher Mode', 5,
                    self._enum_value('Block Cipher Mode', 'CBC', protocol_version))
        enum_fields = {
            'Block Cipher Mode': ('block_cipher_mode', 'CBC'),
            'Padding Method': ('padding_method', 'None'),
            'Hashing Algorithm': ('hashing_algorithm', 'SHA-256'),
            'Key Role Type': ('key_role_type', 'DEK'),
            'Digital Signature Algorithm': ('digital_signature_algorithm', 'ECDSA with SHA256'),
            'Cryptographic Algorithm': ('cryptographic_algorithm', 'AES'),
            'Mask Generator': ('mask_generator', 'MFG1'),
            'Mask Generator Hashing Algorithm': ('mask_generator_hashing_algorithm', 'SHA-256'),
        }
        if name in enum_fields:
            field_name, value_name = enum_fields[name]
            family = 'Hashing Algorithm' if name == 'Mask Generator Hashing Algorithm' else name
            return field_name, name, 5, self._enum_value(family, value_name, protocol_version)
        if name == 'Random IV':
            return 'random_iv', name, 6, 0
        integer_fields = {
            'IV Length': 12, 'Tag Length': 16, 'Fixed Field Length': 2,
            'Invocation Field Length': 4, 'Counter Length': 32,
            'Initial Counter Value': 1, 'Salt Length': 8, 'Trailer Field': 1,
        }
        if name in integer_fields:
            return name.lower().replace(' ', '_'), name, 2, integer_fields[name]
        if name == 'P Source':
            return 'p_source', name, 8, b'\x01\x02\x03'
        return None

    def _execute_cryptographic_parameters(self, target: dict, protocol_version: str) -> dict | None:
        selected = self._crypto_target_field(target, protocol_version)
        if selected is None:
            return None
        field_name, tag_name, item_type, value = selected
        if isinstance(value, tuple):
            return self._execute_cryptographic_enum_boundaries(target, value, protocol_version)
        fields = _CryptographicParametersFieldsV1()
        fields.struct_size = C.sizeof(fields)
        fields.abi_version = 1
        setattr(fields, field_name + '_present', 1)
        source_buffer = None
        if field_name == 'p_source':
            source_buffer = C.create_string_buffer(value)
            fields.p_source = C.cast(source_buffer, C.POINTER(C.c_uint8))
            fields.p_source_length = len(value)
            value_bytes = value
        else:
            setattr(fields, field_name, value)
            if item_type == 5:
                value_bytes = int(value).to_bytes(4, 'big')
            elif item_type == 6:
                value_bytes = int(value).to_bytes(8, 'big')
            elif item_type == 2:
                value_bytes = int(value).to_bytes(4, 'big', signed=True)
            else:
                value_bytes = value
        expected_by_field = {field_name: (self._tag(tag_name), item_type, value_bytes)}
        if field_name == 'block_cipher_mode':
            iv_required = {
                self._enum_value('Block Cipher Mode', 'CTR', protocol_version),
                self._enum_value('Block Cipher Mode', 'CCM', protocol_version),
                self._enum_value('Block Cipher Mode', 'GCM', protocol_version),
            }
            if value in iv_required:
                fields.iv_length_present = 1
                fields.iv_length = 12
                expected_by_field['iv_length'] = (
                    self._tag('IV Length'), 2, (12).to_bytes(4, 'big', signed=True))
            if value == self._enum_value('Block Cipher Mode', 'GCM', protocol_version):
                fields.tag_length_present = 1
                fields.tag_length = 16
                expected_by_field['tag_length'] = (
                    self._tag('Tag Length'), 2, (16).to_bytes(4, 'big', signed=True))
        order = ('block_cipher_mode', 'padding_method', 'hashing_algorithm', 'key_role_type',
                 'digital_signature_algorithm', 'cryptographic_algorithm', 'random_iv', 'iv_length',
                 'tag_length', 'fixed_field_length', 'invocation_field_length', 'counter_length',
                 'initial_counter_value', 'salt_length', 'mask_generator',
                 'mask_generator_hashing_algorithm', 'p_source', 'trailer_field')
        expected = [expected_by_field[name] for name in order if name in expected_by_field]
        parameters, structure = C.c_void_p(), C.c_void_p()
        output = _CryptographicParametersFieldsV1()
        output.struct_size = C.sizeof(output)
        output.abi_version = 1
        children = []
        try:
            self.abi.bind('kmipkit_cryptographic_parameters_from_fields_v1',
                          [C.POINTER(_CryptographicParametersFieldsV1), C.POINTER(C.c_void_p)])
            self.abi.call('kmipkit_cryptographic_parameters_from_fields_v1',
                          C.byref(fields), C.byref(parameters))
            self.abi.bind('kmipkit_cryptographic_parameters_read_fields_v1',
                          [C.c_void_p, C.POINTER(_CryptographicParametersFieldsV1)])
            self.abi.call('kmipkit_cryptographic_parameters_read_fields_v1',
                          parameters, C.byref(output))
            for name in expected_by_field:
                if getattr(output, name + '_present') != 1:
                    raise ValueError('cryptographic_parameters_field_presence_mismatch')
                if name == 'p_source':
                    if output.p_source_length != fields.p_source_length or not output.p_source:
                        raise ValueError('cryptographic_parameters_p_source_mismatch')
                    if C.string_at(output.p_source, output.p_source_length) != value:
                        raise ValueError('cryptographic_parameters_p_source_mismatch')
                elif getattr(output, name) != getattr(fields, name):
                    raise ValueError('cryptographic_parameters_field_mismatch')
            self.abi.bind('kmipkit_cryptographic_parameters_into_ttlv',
                          [C.c_void_p, C.POINTER(C.c_void_p)])
            self.abi.call('kmipkit_cryptographic_parameters_into_ttlv',
                          parameters, C.byref(structure))
            parameters = C.c_void_p()
            children.extend(self.abi.read_structure(structure))
            if children != expected:
                raise ValueError('cryptographic_parameters_ttlv_mismatch')
            normalized_tree = [
                [tag, kind, data.hex() if kind != 1 else data]
                for tag, kind, data in children
            ]
            return {'sample_count': 1, 'normalized_tree': normalized_tree,
                    'tree_sha256': digest(normalized_tree), 'invoked_symbols': set(self.abi.called)}
        finally:
            self.abi.release('ttlv_structure', structure)
            self.abi.release('cryptographic_parameters', parameters)

    def _execute_cryptographic_enum_boundaries(self, target: dict, values: tuple[int, int],
                                               protocol_version: str) -> dict | None:
        samples = [self._execute_cryptographic_parameters(
            {**target, 'wire_value': f'{value:08X}'}, protocol_version) for value in values]
        return self._aggregate_model_samples(samples)

    def _aggregate_model_samples(self, samples: list[dict | None]) -> dict | None:
        if any(sample is None for sample in samples):
            return None
        normalized_samples = [sample['normalized_tree'] for sample in samples]
        symbols = set().union(*(sample['invoked_symbols'] for sample in samples))
        return {'sample_count': sum(sample['sample_count'] for sample in samples),
                'normalized_tree': normalized_samples,
                'tree_sha256': digest(normalized_samples), 'invoked_symbols': symbols}

    def _tag(self, name: str) -> int:
        matches = [item for item in self.catalog['normative_targets']
                   if item['kind'] == 'tag' and item['name'] == name]
        if len(matches) != 1:
            raise ValueError('model_fixture_tag_missing')
        return int(matches[0]['wire_value'], 16)

    def _enum_value(self, family: str, name: str, protocol_version: str) -> int:
        normalized_family = ''.join(character for character in family.lower() if character.isalnum())
        matches = [item for item in self.catalog['normative_targets']
                   if item['kind'] == 'enumeration_value' and item['name'] == name
                   and protocol_version in item['protocol_versions']
                   and any(''.join(character for character in parent.removeprefix('KMIPKIT-ELEM-ENUMERATION-').lower()
                                   if character.isalnum()) == normalized_family
                           for parent in item.get('parent_element_ids', []))]
        if len(matches) != 1:
            raise ValueError('model_fixture_enum_value_missing')
        return int(matches[0]['wire_value'], 16)

    def _enum_extension_start(self, family: str, protocol_version: str) -> int:
        normalized_family = ''.join(character for character in family.lower() if character.isalnum())
        matches = [item for item in self.catalog['normative_targets']
                   if item['kind'] == 'enumeration_value' and item.get('allocation') == 'extension'
                   and protocol_version in item['protocol_versions']
                   and any(''.join(character for character in parent.removeprefix('KMIPKIT-ELEM-ENUMERATION-').lower()
                                   if character.isalnum()) == normalized_family
                           for parent in item.get('parent_element_ids', []))]
        if len(matches) != 1 or not all(character in '0123456789abcdefABCDEFxX' for character in matches[0]['wire_value']):
            raise ValueError('model_fixture_enum_extension_missing')
        return int(matches[0]['wire_value'].replace('X', '0').replace('x', '0'), 16)

    def _object_type(self, name: str, protocol_version: str) -> int:
        return self._enum_value('Object Type', name, protocol_version)

    def _enum_tree(self, tag_name: str, family: str, value_name: str,
                   protocol_version: str) -> tuple:
        value = self._enum_value(family, value_name, protocol_version)
        return self._tag(tag_name), 5, value.to_bytes(4, 'big')

    def _integer_tree(self, tag_name: str, value: int) -> tuple:
        return self._tag(tag_name), 2, value.to_bytes(4, 'big', signed=True)

    def _enum_extension_tree(self, tag_name: str, family: str, protocol_version: str) -> tuple:
        value = self._enum_extension_start(family, protocol_version)
        return self._tag(tag_name), 5, value.to_bytes(4, 'big')

    def _key_block_tree(self, protocol_version: str, *, include_key_value: bool = False,
                        key_format: int | None = None, extra_fields: list[tuple] | None = None) -> tuple:
        key_format = (self._enum_value('Key Format Type', 'Raw', protocol_version)
                      if key_format is None else key_format)
        fields = [(KEY_FORMAT_TYPE_TAG, 5, key_format.to_bytes(4, 'big'))]
        if extra_fields:
            fields.extend(extra_fields)
        if include_key_value:
            fields.append(self._key_value_tree())
        return (KEY_BLOCK_TAG, 1, fields)

    def _key_value_tree(self) -> tuple:
        return (self._tag('Key Value'), 1, [
            (self._tag('Key Material'), 8, b'\xa1')])

    def _execute_named_object(self, root: dict, protocol_version: str) -> dict:
        name = root['name']
        key_block_tree = self._key_block_tree(protocol_version, include_key_value=name == 'Secret Data')
        recipes = {
            'Certificate Request': ('kmipkit_certificate_request_from_fields', [
                self._enum_tree('Certificate Request Type', 'Certificate Request Type',
                                'PKCS#10', protocol_version),
                (self._tag('Certificate Request Value'), 8, b'request'),
            ]),
            'PGP Key': ('kmipkit_pgp_key_from_fields', [
                (self._tag('PGP Key Version'), 2, (7).to_bytes(4, 'big')),
                key_block_tree,
            ]),
            'Private Key': ('kmipkit_private_key_from_fields', [key_block_tree]),
            'Public Key': ('kmipkit_public_key_from_fields', [key_block_tree]),
            'Opaque Object': ('kmipkit_opaque_object_from_fields', [
                self._enum_extension_tree('Opaque Data Type', 'Opaque Data Type', protocol_version),
                (self._tag('Opaque Data Value'), 8, b'opaque'),
            ]),
            'Secret Data': ('kmipkit_secret_data_from_fields', [
                self._enum_tree('Secret Data Type', 'Secret Data Type', 'Password', protocol_version),
                key_block_tree,
            ]),
            'Split Key': ('kmipkit_split_key_from_fields', [
                self._integer_tree('Split Key Parts', 2),
                self._integer_tree('Key Part Identifier', 2),
                self._integer_tree('Split Key Threshold', 1),
                self._enum_tree('Split Key Method', 'Split Key Method', 'XOR', protocol_version),
                key_block_tree,
            ]),
        }
        recipe = recipes.get(name)
        if recipe is None:
            raise ValueError('model_fixture_not_implemented')
        constructor, fields = recipe
        root_tag = self._tag(name)
        object_type = self._object_type(name, protocol_version)
        expected = [root_tag, 1, [normalized(tree) for tree in fields]]
        limits = C.c_void_p()
        owned: list[C.c_void_p] = []
        try:
            self.abi.call('kmipkit_codec_limits_defaults', C.byref(limits))
            for field in fields:
                if field[0] != KEY_BLOCK_TAG:
                    continue
                self._exercise_key_block(field, limits, owned)
            item = self._from_fields(constructor, fields, limits, owned)
            normalized_tree = self._read_and_check(item, expected, 'named_object_fields_tree_mismatch')
            managed = self._from_fields('kmipkit_managed_object_from_fields', fields, limits, owned,
                                        (object_type,))
            self._read_and_check(managed, expected, 'managed_object_fields_tree_mismatch')
            managed_item = self._from_item('kmipkit_managed_object_from_item', item, owned, (object_type,))
            self._read_and_check(managed_item, expected, 'managed_object_item_tree_mismatch')
            return {'sample_count': 1, 'normalized_tree': normalized_tree,
                    'tree_sha256': digest(normalized_tree), 'invoked_symbols': set(self.abi.called)}
        finally:
            for handle in reversed(owned):
                self.abi.release('ttlv_item', handle)
            self.abi.release('codec_limits', limits)

    def _supports_target(self, target: dict, result: dict) -> bool:
        if target['kind'] == 'object_structure':
            return result['normalized_tree'][0] == self._tag(target['name'])
        if target['kind'] != 'structure_member':
            return False
        expected_tag = self._tag(target['name'])

        def contains_tag(tree: list) -> bool:
            return tree[0] == expected_tag or (tree[1] == 1 and any(contains_tag(child) for child in tree[2]))

        return contains_tag(result['normalized_tree'])

    def execute(self, target: dict, protocol_version: str,
                public_api_symbol: str | None = None) -> dict | None:
        if public_api_symbol in {'kmipkit_key_block_from_fields', 'kmipkit_key_block_from_item'}:
            self.abi.called.clear()
            return self._execute_key_block_target(target, protocol_version)
        if (public_api_symbol in {None, 'kmipkit_cryptographic_parameters_from_fields_v1',
                                  'kmipkit_cryptographic_parameters_read_fields_v1',
                                  'kmipkit_cryptographic_parameters_into_ttlv'}
                and target.get('kind') in {'enumeration_value', 'tag', 'structure_member', 'attribute'}):
            selected = self._crypto_target_field(target, protocol_version)
            if selected is not None:
                self.abi.called.clear()
                return self._execute_cryptographic_parameters(target, protocol_version)
        parents = set(target.get('parent_element_ids', []))
        if target.get('target_id') == KEY_VALUE_STRUCTURE or KEY_VALUE_STRUCTURE in parents:
            self.abi.called.clear()
            result = self._execute_key_value(protocol_version)
            return result if self._supports_target(target, result) else None
        if target.get('target_id') == KEY_BLOCK_STRUCTURE or KEY_BLOCK_STRUCTURE in parents:
            self.abi.called.clear()
            result = self._execute_key_block(protocol_version)
            return result if self._supports_target(target, result) else None
        if target.get('target_id') == SYMMETRIC_KEY_STRUCTURE or SYMMETRIC_KEY_STRUCTURE in parents:
            self.abi.called.clear()
            result = self._execute_symmetric_key(protocol_version)
            return result if self._supports_target(target, result) else None
        root = next((item for item in self.catalog['normative_targets']
                     if item['kind'] == 'object_structure'
                     and (item['target_id'] == target.get('target_id')
                          or item['target_id'] in parents)), None)
        if root is not None and root['name'] in {
                'Certificate Request', 'PGP Key', 'Private Key', 'Public Key', 'Opaque Object',
                'Secret Data', 'Split Key'}:
            self.abi.called.clear()
            result = self._execute_named_object(root, protocol_version)
            return result if self._supports_target(target, result) else None
        if target.get('target_id') != CERTIFICATE_STRUCTURE and CERTIFICATE_STRUCTURE not in parents:
            return None
        certificate_type = next((item for item in self.catalog['normative_targets']
                                 if item['kind'] == 'enumeration_value'
                                 and item.get('parent_element_ids') == ['KMIPKIT-ELEM-ENUMERATION-CERTIFICATE-TYPE']
                                 and item['name'] == 'X.509'
                                 and protocol_version in item['protocol_versions']), None)
        if certificate_type is None:
            return None

        self.abi.called.clear()

        self.abi.called.clear()
        limits, output, generic_output = C.c_void_p(), C.c_void_p(), C.c_void_p()
        field_items = []
        try:
            self.abi.call('kmipkit_codec_limits_defaults', C.byref(limits))
            type_value = int(certificate_type['wire_value'], 16)
            type_item = self.abi.construct(
                (CERTIFICATE_TYPE_TAG, 5, type_value.to_bytes(4, 'big')), limits)
            value_item = self.abi.construct((CERTIFICATE_VALUE_TAG, 8, bytes.fromhex('010203')), limits)
            field_items.extend((type_item, value_item))
            fields = (C.c_void_p * len(field_items))(*field_items)
            self.abi.bind(
                'kmipkit_certificate_from_fields',
                [C.POINTER(C.c_void_p), C.c_uint64, C.POINTER(C.c_void_p)])
            self.abi.call('kmipkit_certificate_from_fields', fields, len(field_items), C.byref(output))
            expected = [CERTIFICATE_TAG, 1, [
                [CERTIFICATE_TYPE_TAG, 5, type_value.to_bytes(4, 'big').hex()],
                [CERTIFICATE_VALUE_TAG, 8, '010203']]]
            normalized_tree = self._read_and_check(output, expected, 'certificate_fields_tree_mismatch')

            self.abi.bind(
                'kmipkit_managed_object_from_fields',
                [C.c_uint32, C.POINTER(C.c_void_p), C.c_uint64, C.POINTER(C.c_void_p)])
            self.abi.call('kmipkit_managed_object_from_fields', CERTIFICATE_OBJECT_TYPE,
                          fields, len(field_items), C.byref(generic_output))
            self._read_and_check(generic_output, expected, 'managed_certificate_fields_tree_mismatch')

            self.abi.bind(
                'kmipkit_managed_object_from_item',
                [C.c_uint32, C.c_void_p, C.POINTER(C.c_void_p)])
            item_output = C.c_void_p()
            try:
                self.abi.call('kmipkit_managed_object_from_item', CERTIFICATE_OBJECT_TYPE,
                              output, C.byref(item_output))
                self._read_and_check(item_output, expected, 'managed_certificate_item_tree_mismatch')
            finally:
                self.abi.release('ttlv_item', item_output)

            return {'sample_count': 1, 'normalized_tree': normalized_tree,
                    'tree_sha256': digest(normalized_tree), 'invoked_symbols': set(self.abi.called)}
        finally:
            for handle in (generic_output, output, *reversed(field_items)):
                self.abi.release('ttlv_item', handle)
            self.abi.release('codec_limits', limits)
