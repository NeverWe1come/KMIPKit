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

    def _execute_symmetric_key(self, protocol_version: str) -> dict:
        key_format = next((item for item in self.catalog['normative_targets']
                           if item['kind'] == 'enumeration_value' and item['name'] == 'Raw'
                           and item.get('parent_element_ids') == ['KMIPKIT-ELEM-ENUMERATION-KEY-FORMAT-TYPE']
                           and protocol_version in item['protocol_versions']), None)
        if key_format is None:
            raise ValueError('symmetric_key_format_fixture_missing')
        key_format_value = int(key_format['wire_value'], 16)
        limits = C.c_void_p()
        owned: list[C.c_void_p] = []
        try:
            self.abi.call('kmipkit_codec_limits_defaults', C.byref(limits))
            key_block = self._from_fields(
                'kmipkit_key_block_from_fields',
                [(KEY_FORMAT_TYPE_TAG, 5, key_format_value.to_bytes(4, 'big'))], limits, owned)
            key_block_roundtrip = self._from_item('kmipkit_key_block_from_item', key_block, owned)
            expected = [SYMMETRIC_KEY_TAG, 1, [[KEY_BLOCK_TAG, 1, [
                [KEY_FORMAT_TYPE_TAG, 5, key_format_value.to_bytes(4, 'big').hex()]]]]]
            self._read_and_check(key_block_roundtrip, expected[2][0], 'key_block_item_tree_mismatch')

            symmetric = self._from_fields('kmipkit_symmetric_key_from_fields', [
                (KEY_BLOCK_TAG, 1, [[KEY_FORMAT_TYPE_TAG, 5, key_format_value.to_bytes(4, 'big')]])],
                limits, owned)
            normalized_tree = self._read_and_check(symmetric, expected, 'symmetric_key_fields_tree_mismatch')
            managed = self._from_fields('kmipkit_managed_object_from_fields', [
                (KEY_BLOCK_TAG, 1, [[KEY_FORMAT_TYPE_TAG, 5, key_format_value.to_bytes(4, 'big')]])],
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

    def execute(self, target: dict, protocol_version: str) -> dict | None:
        parents = set(target.get('parent_element_ids', []))
        if target.get('target_id') == SYMMETRIC_KEY_STRUCTURE or SYMMETRIC_KEY_STRUCTURE in parents:
            self.abi.called.clear()
            return self._execute_symmetric_key(protocol_version)
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
