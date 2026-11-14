"""Recursive generic TTLV execution through the installed public C ABI.

OASIS KMIP 2.0/2.1 Specification §10.1. All scalar bytes and child ordering
are read back through public views; the harness never substitutes an expected
tree for the actual inspected result. The independent serializer is a fixture
codec and is never labeled a public C wire encoder.
"""
from __future__ import annotations

import ctypes as C
import hashlib
import json
from pathlib import Path

from route_matrix import DecodeError, InstalledAbi

NAMES = {2: 'integer', 3: 'long_integer', 5: 'enumeration', 6: 'boolean',
         9: 'date_time', 10: 'interval', 11: 'date_time_extended'}
CTYPES = {2: C.c_int32, 3: C.c_int64, 5: C.c_uint32, 6: C.c_uint8,
          9: C.c_int64, 10: C.c_uint32, 11: C.c_int64}
SIZES = {2: 4, 3: 8, 5: 4, 6: 8, 9: 8, 10: 4, 11: 8}
BINARY_NAMES = {4: 'big_integer', 7: 'text_string', 8: 'byte_string'}


def encode(tree) -> bytes:
    tag, kind, value = tree
    body = b''.join(encode(child) for child in value) if kind == 1 else value
    return tag.to_bytes(3, 'big') + bytes([kind]) + len(body).to_bytes(4, 'big') + body + bytes(-len(body) % 8)


def parse(wire: bytes):
    if len(wire) < 8:
        raise ValueError('fixture_truncated_header')
    tag, kind, length = int.from_bytes(wire[:3], 'big'), wire[3], int.from_bytes(wire[4:8], 'big')
    if len(wire) != 8 + (length + 7) // 8 * 8:
        raise ValueError('fixture_invalid_length')
    value = wire[8:8 + length]
    if kind == 1:
        children, offset = [], 0
        while offset < length:
            if offset + 8 > length:
                raise ValueError('fixture_truncated_child')
            child_length = 8 + (int.from_bytes(value[offset + 4:offset + 8], 'big') + 7) // 8 * 8
            children.append(parse(value[offset:offset + child_length]))
            offset += child_length
        value = children
    return tag, kind, value


def normalized(tree):
    tag, kind, value = tree
    return [tag, kind, [normalized(child) for child in value] if kind == 1 else value.hex()]


def digest(value) -> str:
    return hashlib.sha256(json.dumps(value, separators=(',', ':'), sort_keys=True).encode()).hexdigest()


class TreeAbi(InstalledAbi):
    def __init__(self, library: Path):
        super().__init__(library)
        self.bind('kmipkit_ttlv_structure_create', [C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_structure_with_item',
                  [C.c_void_p, C.c_void_p, C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_request_message_from_ttlv',
                  [C.c_void_p, C.POINTER(C.c_void_p), C.POINTER(C.c_void_p)])
        self.bind('kmipkit_request_message_ttlv', [C.c_void_p, C.POINTER(C.c_void_p)])
        for kind, name in NAMES.items():
            self.bind(f'kmipkit_ttlv_value_{name}', [CTYPES[kind], C.POINTER(C.c_void_p)])
            self.bind(f'kmipkit_ttlv_value_view_{name}', [C.c_void_p, C.POINTER(CTYPES[kind])])
        for name in BINARY_NAMES.values():
            self.bind(f'kmipkit_ttlv_value_{name}', [C.c_void_p, C.c_void_p, C.c_uint64, C.POINTER(C.c_void_p)])
        release_view = self.library.kmipkit_ttlv_structure_view_release
        release_view.argtypes, release_view.restype = [C.c_void_p], None
        self.bind('kmipkit_ttlv_value_structure', [C.c_void_p, C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_structure_view', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_view_structure', [C.c_void_p, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_structure_view_item_count', [C.c_void_p, C.POINTER(C.c_uint64)])
        self.bind('kmipkit_ttlv_structure_view_item_at', [C.c_void_p, C.c_uint64, C.POINTER(C.c_void_p)])
        self.bind('kmipkit_ttlv_value_view_byte_length', [C.c_void_p, C.POINTER(C.c_uint64)])
        self.bind('kmipkit_ttlv_value_view_byte_at', [C.c_void_p, C.c_uint64, C.POINTER(C.c_uint8)])

    def release(self, name: str, handle: C.c_void_p) -> None:
        if handle.value:
            getattr(self.library, f'kmipkit_{name}_release')(handle)

    def construct(self, tree, limits):
        tag_code, kind, data = tree
        raw_tag, tag, value, structure, item = (C.c_void_p() for _ in range(5))
        try:
            self.call('kmipkit_ttlv_raw_tag_create', tag_code, C.byref(raw_tag))
            self.call('kmipkit_ttlv_raw_tag_try_checked', raw_tag, C.byref(tag))
            if kind == 1:
                self.call('kmipkit_ttlv_structure_create', C.byref(structure))
                for child in data:
                    child_item = self.construct(child, limits)
                    original = structure
                    structure = C.c_void_p()
                    self.call('kmipkit_ttlv_structure_with_item', original, child_item, limits, C.byref(structure))
                original = structure
                structure = C.c_void_p()
                self.call('kmipkit_ttlv_value_structure', original, limits, C.byref(value))
            elif kind in NAMES:
                number = int.from_bytes(data, 'big', signed=kind in (2, 3, 9, 11))
                self.call(f'kmipkit_ttlv_value_{NAMES[kind]}', number, C.byref(value))
            elif kind in BINARY_NAMES:
                buffer = C.create_string_buffer(data)
                self.call(f'kmipkit_ttlv_value_{BINARY_NAMES[kind]}', limits, buffer, len(data), C.byref(value))
            else:
                raise ValueError('fixture_unsupported_ttlv_type')
            original = value
            value = C.c_void_p()
            self.call('kmipkit_ttlv_item_create', tag, original, limits, C.byref(item))
            result = item
            item = C.c_void_p()
            return result
        finally:
            for name, handle in (('raw_tag', raw_tag), ('tag', tag), ('ttlv_value', value),
                                 ('ttlv_structure', structure), ('ttlv_item', item)):
                self.release(name, handle)

    def read_view(self, view):
        tag, value, structure = C.c_void_p(), C.c_void_p(), C.c_void_p()
        try:
            self.call('kmipkit_ttlv_item_view_tag', view, C.byref(tag))
            self.call('kmipkit_ttlv_item_view_value', view, C.byref(value))
            raw_tag, kind = C.c_uint32(), C.c_uint8()
            self.call('kmipkit_ttlv_tag_value', tag, C.byref(raw_tag))
            self.call('kmipkit_ttlv_item_view_type', view, C.byref(kind))
            if kind.value == 1:
                self.call('kmipkit_ttlv_value_view_structure', value, C.byref(structure))
                count = C.c_uint64()
                self.call('kmipkit_ttlv_structure_view_item_count', structure, C.byref(count))
                data = []
                for index in range(count.value):
                    child = C.c_void_p()
                    try:
                        self.call('kmipkit_ttlv_structure_view_item_at', structure, index, C.byref(child))
                        data.append(self.read_view(child))
                    finally:
                        self.release('ttlv_item_view', child)
            elif kind.value in NAMES:
                number = CTYPES[kind.value]()
                self.call(f'kmipkit_ttlv_value_view_{NAMES[kind.value]}', value, C.byref(number))
                data = number.value.to_bytes(
                    SIZES[kind.value], 'big', signed=kind.value in (2, 3, 9, 11))
            else:
                length = C.c_uint64()
                self.call('kmipkit_ttlv_value_view_byte_length', value, C.byref(length))
                data = bytearray()
                for index in range(length.value):
                    byte = C.c_uint8()
                    self.call('kmipkit_ttlv_value_view_byte_at', value, index, C.byref(byte))
                    data.append(byte.value)
                data = bytes(data)
            return raw_tag.value, kind.value, data
        finally:
            for name, handle in (('tag', tag), ('ttlv_value_view', value), ('ttlv_structure_view', structure)):
                self.release(name, handle)

    def read_item(self, item):
        view = C.c_void_p()
        try:
            self.call('kmipkit_ttlv_item_view_from_item', item, C.byref(view))
            return self.read_view(view)
        finally:
            self.release('ttlv_item_view', view)

    def read_structure(self, structure):
        view = C.c_void_p()
        try:
            self.call('kmipkit_ttlv_structure_view', structure, C.byref(view))
            count = C.c_uint64()
            self.call('kmipkit_ttlv_structure_view_item_count', view, C.byref(count))
            items = []
            for index in range(count.value):
                item_view = C.c_void_p()
                try:
                    self.call('kmipkit_ttlv_structure_view_item_at', view, index, C.byref(item_view))
                    items.append(self.read_view(item_view))
                finally:
                    self.release('ttlv_item_view', item_view)
            return items
        finally:
            self.release('ttlv_structure_view', view)

    def construct_structure(self, children, limits):
        structure = C.c_void_p()
        try:
            self.call('kmipkit_ttlv_structure_create', C.byref(structure))
            for child in children:
                child_item = self.construct(child, limits)
                original = structure
                structure = C.c_void_p()
                self.call('kmipkit_ttlv_structure_with_item', original, child_item,
                          limits, C.byref(structure))
            result = structure
            structure = C.c_void_p()
            return result
        finally:
            self.release('ttlv_structure', structure)

    def roundtrip_request_message(self, tree, limits):
        if tree[0] != 0x420078 or tree[1] != 1:
            return
        structure = self.construct_structure(tree[2], limits)
        message, validation_error, returned_structure = (C.c_void_p() for _ in range(3))
        try:
            self.called.add('kmipkit_request_message_from_ttlv')
            status = self.library.kmipkit_request_message_from_ttlv(
                structure, C.byref(message), C.byref(validation_error))
            structure = C.c_void_p()
            if status != 0 or not message.value:
                raise ValueError('typed_request_message_rejected')
            self.call('kmipkit_request_message_ttlv', message, C.byref(returned_structure))
            if self.read_structure(returned_structure) != tree[2]:
                raise ValueError('typed_request_message_tree_mismatch')
        finally:
            self.release('ttlv_structure', structure)
            self.release('ttlv_structure', returned_structure)
            self.release('message_validation_error', validation_error)
            self.release('request_message', message)

    def roundtrip(self, wire):
        self.called.clear()
        limits, constructed, decoded = C.c_void_p(), C.c_void_p(), C.c_void_p()
        try:
            expected = parse(wire)
            self.call('kmipkit_codec_limits_defaults', C.byref(limits))
            constructed = self.construct(expected, limits)
            constructed_tree = self.read_item(constructed)
            error = DecodeError(C.sizeof(DecodeError), 1, 0, 0)
            buffer = C.create_string_buffer(wire)
            self.call('kmipkit_ttlv_decode', buffer, len(wire), C.byref(error), C.byref(decoded))
            decoded_tree = self.read_item(decoded)
            self.roundtrip_request_message(expected, limits)
            if constructed_tree != expected or decoded_tree != expected:
                raise ValueError('installed_abi_tree_mismatch')
            return {'constructed_wire': encode(constructed_tree), 'decoded_wire': encode(decoded_tree),
                    'normalized_tree': normalized(decoded_tree), 'invoked_symbols': set(self.called)}
        finally:
            self.release('ttlv_item', constructed)
            self.release('ttlv_item', decoded)
            self.release('codec_limits', limits)

    def reject_wire(self, wire):
        self.called.clear()
        decoded = C.c_void_p()
        try:
            error = DecodeError(C.sizeof(DecodeError), 1, 0, 0)
            buffer = C.create_string_buffer(wire)
            self.called.add('kmipkit_ttlv_decode')
            status = self.library.kmipkit_ttlv_decode(buffer, len(wire), C.byref(error), C.byref(decoded))
            if status == 0 or decoded.value or error.kind == 0:
                raise ValueError('malformed_ttlv_not_rejected')
            return error.kind
        finally:
            self.release('ttlv_item', decoded)

    def reject_invalid_boolean(self):
        self.called.add('kmipkit_ttlv_value_boolean')
        value = C.c_void_p()
        try:
            return self.library.kmipkit_ttlv_value_boolean(2, C.byref(value)) != 0 and not value.value
        finally:
            self.release('ttlv_value', value)

    def boundary_case(self, vector):
        results, symbols = [], set()
        for sample in vector['samples']:
            wire = bytes.fromhex(sample['hex'])
            if sample['valid']:
                result = self.roundtrip(wire)
                symbols.update(result['invoked_symbols'])
                results.append({'branch': sample['branch'], 'tree': result['normalized_tree']})
            else:
                kind = self.reject_wire(wire)
                symbols.update(self.called)
                results.append({'branch': sample['branch'], 'decode_error_kind': kind})
        if vector['type_code'] == '06':
            if not self.reject_invalid_boolean():
                raise ValueError('invalid_boolean_constructor_not_rejected')
            symbols.update(self.called)
        return {'sample_count': len(results), 'failed_samples': 0, 'invoked_symbols': symbols,
                'tree_sha256': digest(results), 'branches': [result['branch'] for result in results]}
