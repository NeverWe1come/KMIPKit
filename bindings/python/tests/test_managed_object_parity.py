"""Four-language typed managed-object and key-value behavior for KMIPKIT-0053."""

import pytest

from kmipkit import errors, ttlv
from kmipkit.managed_objects import (
    KeyBlock,
    KeyMaterial,
    KeyValue,
    KeyWrappingData,
    ManagedObject,
)


def _structure_item(raw_tag: int) -> ttlv.TtlvItem:
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.raw_tag_from_raw(raw_tag) as raw_tag_handle:
            tag = ttlv.raw_tag_try_checked(raw_tag_handle)
            try:
                value = ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits)
                return ttlv.ttlv_item_create(tag, value, limits)
            finally:
                tag.close()


def _item(raw_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    with ttlv.raw_tag_from_raw(raw_tag) as raw_tag_handle:
        tag = ttlv.raw_tag_try_checked(raw_tag_handle)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()


def _structure_with_children(
    raw_tag: int, limits: ttlv.CodecLimits, children: list[ttlv.TtlvItem]
) -> ttlv.TtlvItem:
    structure = ttlv.ttlv_structure_create()
    for child in children:
        structure = ttlv.ttlv_structure_with_item(structure, child, limits)
    value = ttlv.ttlv_value_structure(structure, limits)
    return _item(raw_tag, value, limits)


def _item_type(item: ttlv.TtlvItem) -> ttlv.ItemType:
    with ttlv.ttlv_item_view_from_item(item) as view:
        return ttlv.ttlv_item_view_type(view)


def _item_bytes(item: ttlv.TtlvItem) -> bytes:
    with ttlv.ttlv_item_view_from_item(item) as item_view:
        with ttlv.ttlv_item_view_value(item_view) as value_view:
            return bytes(
                ttlv.ttlv_value_view_byte_at(value_view, index)
                for index in range(ttlv.ttlv_value_view_byte_length(value_view))
            )


def _child_tags(item: ttlv.TtlvItem) -> list[int]:
    with ttlv.ttlv_item_view_from_item(item) as item_view:
        with ttlv.ttlv_item_view_value(item_view) as value_view:
            with ttlv.ttlv_value_view_structure(value_view) as structure:
                tags = []
                for index in range(ttlv.ttlv_structure_view_item_count(structure)):
                    with ttlv.ttlv_structure_view_item_at(structure, index) as child:
                        with ttlv.ttlv_item_view_tag(child) as tag:
                            tags.append(ttlv.tag_value(tag))
                return tags


def _child_byte_strings(item: ttlv.TtlvItem) -> list[str]:
    with ttlv.ttlv_item_view_from_item(item) as item_view:
        with ttlv.ttlv_item_view_value(item_view) as value_view:
            with ttlv.ttlv_value_view_structure(value_view) as structure:
                values = []
                for index in range(ttlv.ttlv_structure_view_item_count(structure)):
                    with ttlv.ttlv_structure_view_item_at(structure, index) as child:
                        with ttlv.ttlv_item_view_value(child) as child_value:
                            values.append(_item_bytes_from_value(child_value).decode())
                return values


def _item_bytes_from_value(value_view: ttlv.TtlvValueView) -> bytes:
    return bytes(
        ttlv.ttlv_value_view_byte_at(value_view, index)
        for index in range(ttlv.ttlv_value_view_byte_length(value_view))
    )


def test_managed_object_unknown_type_roundtrips_without_losing_generic_ttlv():
    source = _structure_item(0x0054_0001)
    with source, ManagedObject.from_item(0x8000_0001, source) as managed:
        assert managed.object_type == 0x8000_0001
        with ttlv.ttlv_item_view_from_item(managed) as view:
            with ttlv.ttlv_item_view_tag(view) as tag:
                assert ttlv.tag_value(tag) == 0x0054_0001
        assert not source.closed


def test_managed_object_rejects_mismatched_known_type_without_echoing_material():
    source = _structure_item(0x0054_0001)
    try:
        with pytest.raises(errors.InvalidInputError) as failure:
            ManagedObject.from_item(2, source)
        assert "sensitive" not in str(failure.value).lower()
    finally:
        source.close()


def test_key_wrappers_validate_and_retain_owned_generic_items():
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.raw_tag_from_raw(0x0042_0043) as raw_tag:
            tag = ttlv.raw_tag_try_checked(raw_tag)
            try:
                value = ttlv.ttlv_value_byte_string(b"secret-canary", limits)
                key_material = ttlv.ttlv_item_create(tag, value, limits)
            finally:
                tag.close()
    with key_material, KeyMaterial.from_item(1, key_material) as material:
        assert material.format_raw == 1
        with ttlv.ttlv_item_view_from_item(material) as view:
            with ttlv.ttlv_item_view_tag(view) as tag:
                assert ttlv.tag_value(tag) == 0x0042_0043


def test_nested_accessors_return_owned_nodes_with_values_and_vendor_order():
    with ttlv.codec_limits_defaults() as limits:
        key_material = _item(
            0x0042_0043, ttlv.ttlv_value_byte_string(b"key-data", limits), limits
        )
        vendor_one = _item(
            0x0054_0031, ttlv.ttlv_value_byte_string(b"vendor-one", limits), limits
        )
        vendor_two = _item(
            0x0054_0031, ttlv.ttlv_value_byte_string(b"vendor-two", limits), limits
        )
        attributes = _structure_with_children(0x0042_0125, limits, [vendor_one, vendor_two])
        key_value_item = _structure_with_children(
            0x0042_0045, limits, [key_material, attributes]
        )
        format_type = _item(0x0042_0042, ttlv.ttlv_value_enumeration(1), limits)
        source_items = [key_material, vendor_one, vendor_two, attributes, key_value_item, format_type]
        try:
            with KeyBlock.from_fields([format_type, key_value_item]) as block:
                nested_key_value = block.key_value_item()
                assert isinstance(nested_key_value, ttlv.TtlvItem)
                with nested_key_value:
                    assert _item_type(nested_key_value) is ttlv.ItemType.Structure
                    assert _child_tags(nested_key_value) == [0x0042_0043, 0x0042_0125]
                    with KeyValue.from_item(1, nested_key_value) as typed_key_value:
                        nested_material = typed_key_value.key_material_item()
                        nested_attributes = typed_key_value.attributes_item()
                        assert nested_material is not None and nested_attributes is not None
                        with nested_material, nested_attributes:
                            assert _item_bytes(nested_material) == b"key-data"
                            assert _child_tags(nested_attributes) == [0x0054_0031, 0x0054_0031]
                            assert _child_byte_strings(nested_attributes) == [
                                "vendor-one", "vendor-two"
                            ]
                        assert not nested_key_value.closed
            assert not key_value_item.closed
            assert not format_type.closed
        finally:
            for item in source_items:
                item.close()

        method = _item(0x0042_009E, ttlv.ttlv_value_enumeration(0x8000_0020), limits)
        encryption_id = _item(
            0x0042_0094, ttlv.ttlv_value_text_string(b"encryption-id", limits), limits
        )
        mac_id = _item(0x0042_0094, ttlv.ttlv_value_text_string(b"mac-id", limits), limits)
        encryption_vendor_one = _item(
            0x0054_0031, ttlv.ttlv_value_byte_string(b"vendor-one", limits), limits
        )
        encryption_vendor_two = _item(
            0x0054_0031, ttlv.ttlv_value_byte_string(b"vendor-two", limits), limits
        )
        mac_vendor_one = _item(
            0x0054_0031, ttlv.ttlv_value_byte_string(b"vendor-one", limits), limits
        )
        mac_vendor_two = _item(
            0x0054_0031, ttlv.ttlv_value_byte_string(b"vendor-two", limits), limits
        )
        encryption_info = _structure_with_children(
            0x0042_0036,
            limits,
            [encryption_id, encryption_vendor_one, encryption_vendor_two],
        )
        mac_info = _structure_with_children(
            0x0042_004E, limits, [mac_id, mac_vendor_one, mac_vendor_two]
        )
        wrap_sources = [
            method,
            encryption_id,
            mac_id,
            encryption_vendor_one,
            encryption_vendor_two,
            mac_vendor_one,
            mac_vendor_two,
            encryption_info,
            mac_info,
        ]
        try:
            with KeyWrappingData.from_fields(
                [method, encryption_info, mac_info]
            ) as wrapping:
                nested_encryption = wrapping.encryption_key_information_item()
                nested_mac = wrapping.mac_signature_key_information_item()
                assert nested_encryption is not None and nested_mac is not None
                with nested_encryption, nested_mac:
                    expected_tags = [0x0042_0094, 0x0054_0031, 0x0054_0031]
                    assert _child_tags(nested_encryption) == expected_tags
                    assert _child_byte_strings(nested_encryption) == [
                        "encryption-id", "vendor-one", "vendor-two"
                    ]
                    assert _child_tags(nested_mac) == expected_tags
                    assert _child_byte_strings(nested_mac) == [
                        "mac-id", "vendor-one", "vendor-two"
                    ]
                    assert wrapping.field_item(0x0054_0002) is None
                assert not wrapping.closed

            wrapped_value = _item(
                0x0042_0045, ttlv.ttlv_value_byte_string(b"wrapped", limits), limits
            )
            wrapped_format_type = _item(
                0x0042_0042, ttlv.ttlv_value_enumeration(1), limits
            )
            wrapping_item = _structure_with_children(
                0x0042_0046, limits, [method, encryption_info]
            )
            wrapped_sources = [wrapped_value, wrapped_format_type, wrapping_item]
            with KeyBlock.from_fields(
                [wrapped_format_type, wrapped_value, wrapping_item]
            ) as wrapped_block:
                nested_wrapped_value = wrapped_block.key_value_item()
                nested_wrapping = wrapped_block.key_wrapping_data_item()
                assert nested_wrapped_value is not None and nested_wrapping is not None
                with nested_wrapped_value, nested_wrapping:
                    assert _item_type(nested_wrapped_value) is ttlv.ItemType.ByteString
                    assert _item_bytes(nested_wrapped_value) == b"wrapped"
                    assert _child_tags(nested_wrapping) == [0x0042_009E, 0x0042_0036]
                    with KeyWrappingData.from_item(nested_wrapping) as typed_wrapping:
                        nested_encryption = typed_wrapping.encryption_key_information_item()
                        assert nested_encryption is not None
                        with nested_encryption:
                            assert _child_tags(nested_encryption) == [
                                0x0042_0094, 0x0054_0031, 0x0054_0031
                            ]
                            assert _child_byte_strings(nested_encryption) == [
                                "encryption-id", "vendor-one", "vendor-two"
                            ]
            for item in wrapped_sources:
                item.close()
        finally:
            for item in wrap_sources:
                item.close()


def test_dispatches_all_nine_known_object_types_and_reads_key_structures():
    expected = [
        (1, 0x0042_0013),
        (2, 0x0042_008F),
        (3, 0x0042_006D),
        (4, 0x0042_0064),
        (5, 0x0042_0089),
        (7, 0x0042_0085),
        (8, 0x0042_005B),
        (9, 0x0042_00BD),
        (10, 0x0042_0018),
    ]
    with ttlv.codec_limits_defaults() as limits:
        def enum(raw_tag: int, raw_value: int) -> ttlv.TtlvItem:
            return _item(raw_tag, ttlv.ttlv_value_enumeration(raw_value), limits)

        def integer(raw_tag: int, value: int) -> ttlv.TtlvItem:
            return _item(raw_tag, ttlv.ttlv_value_integer(value), limits)

        def byte_string(raw_tag: int, value: bytes = b"\x01") -> ttlv.TtlvItem:
            return _item(raw_tag, ttlv.ttlv_value_byte_string(value, limits), limits)

        def key_block() -> ttlv.TtlvItem:
            return _structure_with_children(
                0x0042_0040, limits, [enum(0x0042_0042, 1)]
            )

        objects = [
            _structure_with_children(0x0042_0013, limits,
                                     [enum(0x0042_001D, 1), byte_string(0x0042_001E)]),
            _structure_with_children(0x0042_008F, limits, [key_block()]),
            _structure_with_children(0x0042_006D, limits, [key_block()]),
            _structure_with_children(0x0042_0064, limits, [key_block()]),
            _structure_with_children(0x0042_0089, limits,
                                     [integer(0x0042_008B, 2), integer(0x0042_0044, 1),
                                      integer(0x0042_008C, 2), enum(0x0042_008A, 1), key_block()]),
            _structure_with_children(0x0042_0085, limits,
                                     [enum(0x0042_0086, 1), key_block()]),
            _structure_with_children(0x0042_005B, limits,
                                     [enum(0x0042_0059, 1), byte_string(0x0042_005A)]),
            _structure_with_children(0x0042_00BD, limits,
                                     [integer(0x0042_00BE, 7), key_block()]),
            _structure_with_children(0x0042_0018, limits,
                                     [enum(0x0042_0019, 1), byte_string(0x0042_0140)]),
        ]
        for (object_type, expected_tag), source in zip(expected, objects, strict=True):
            with source, ManagedObject.from_item(object_type, source) as managed:
                assert managed.object_type == object_type
                with ttlv.ttlv_item_view_from_item(managed) as view:
                    with ttlv.ttlv_item_view_tag(view) as tag:
                        assert ttlv.tag_value(tag) == expected_tag

        key_value = _structure_with_children(
            0x0042_0045, limits, [byte_string(0x0042_0043, b"secret-canary")]
        )
        with key_value, KeyValue.from_item(1, key_value) as typed_key_value:
            with ttlv.ttlv_item_view_from_item(typed_key_value) as view:
                with ttlv.ttlv_item_view_tag(view) as tag:
                    assert ttlv.tag_value(tag) == 0x0042_0045

        wrapping = _structure_with_children(
            0x0042_0046,
            limits,
            [enum(0x0042_009E, 1),
             _structure_with_children(
                 0x0042_0036, limits,
                 [_item(0x0042_0094,
                        ttlv.ttlv_value_text_string(b"wrapping-key", limits), limits)],
             )],
        )
        with wrapping, KeyWrappingData.from_item(wrapping) as typed_wrapping:
            with ttlv.ttlv_item_view_from_item(typed_wrapping) as view:
                with ttlv.ttlv_item_view_tag(view) as tag:
                    assert ttlv.tag_value(tag) == 0x0042_0046

        block = key_block()
        with block, KeyBlock.from_item(block) as typed_block:
            with ttlv.ttlv_item_view_from_item(typed_block) as view:
                with ttlv.ttlv_item_view_tag(view) as tag:
                    assert ttlv.tag_value(tag) == 0x0042_0040


def test_certificate_from_fields_orders_known_fields_and_reads_named_values():
    with ttlv.codec_limits_defaults() as limits:
        certificate_value = _item(
            0x0042_001E,
            ttlv.ttlv_value_byte_string(b"certificate-canary", limits),
            limits,
        )
        vendor_first = _item(
            0x0054_0010, ttlv.ttlv_value_byte_string(b"vendor-one", limits), limits
        )
        certificate_type = _item(
            0x0042_001D, ttlv.ttlv_value_enumeration(0x8000_0042), limits
        )
        vendor_repeat = _item(
            0x0054_0010, ttlv.ttlv_value_byte_string(b"vendor-two", limits), limits
        )
    try:
        with ManagedObject.from_fields(
            1, [certificate_value, vendor_first, certificate_type, vendor_repeat]
        ) as managed:
            assert managed.certificate_type == 0x8000_0042
            assert managed.field_read(0x0042_001D, 3) == 0x8000_0042
            assert managed.certificate_value == b"certificate-canary"
            with ttlv.ttlv_item_view_from_item(managed) as view:
                with ttlv.ttlv_item_view_value(view) as value:
                    with ttlv.ttlv_value_view_structure(value) as structure:
                        tags = []
                        for index in range(ttlv.ttlv_structure_view_item_count(structure)):
                            with ttlv.ttlv_structure_view_item_at(structure, index) as field:
                                with ttlv.ttlv_item_view_tag(field) as tag:
                                    tags.append(ttlv.tag_value(tag))
            assert tags == [0x0042_001D, 0x0054_0010, 0x0042_001E, 0x0054_0010]
            assert not managed.closed
        assert all(not field.closed for field in [certificate_value, certificate_type])
    finally:
        certificate_value.close()
        vendor_first.close()
        certificate_type.close()
        vendor_repeat.close()


def test_transparent_rsa_public_fields_are_typed_and_reordered():
    with ttlv.codec_limits_defaults() as limits:
        exponent = _item(
            0x0042_006C,
            ttlv.ttlv_value_big_integer(b"\x01\x00\x01", limits),
            limits,
        )
        modulus = _item(
            0x0042_0052,
            ttlv.ttlv_value_big_integer(b"\x80\x01", limits),
            limits,
        )
    try:
        with KeyMaterial.from_fields(11, [exponent, modulus]) as material:
            assert material.format_raw == 11
            with ttlv.ttlv_item_view_from_item(material) as item_view:
                with ttlv.ttlv_item_view_value(item_view) as value_view:
                    with ttlv.ttlv_value_view_structure(value_view) as structure:
                        tags = []
                        for index in range(ttlv.ttlv_structure_view_item_count(structure)):
                            with ttlv.ttlv_structure_view_item_at(structure, index) as field:
                                with ttlv.ttlv_item_view_tag(field) as tag:
                                    tags.append(ttlv.tag_value(tag))
            assert tags == [0x0042_0052, 0x0042_006C]
    finally:
        exponent.close()
        modulus.close()


def test_named_object_constructors_read_all_object_specific_fields():
    with ttlv.codec_limits_defaults() as limits:
        def enum(raw_tag: int, raw: int) -> ttlv.TtlvItem:
            return _item(raw_tag, ttlv.ttlv_value_enumeration(raw), limits)

        def integer(raw_tag: int, raw: int) -> ttlv.TtlvItem:
            return _item(raw_tag, ttlv.ttlv_value_integer(raw), limits)

        def byte_string(raw_tag: int, raw: bytes) -> ttlv.TtlvItem:
            return _item(raw_tag, ttlv.ttlv_value_byte_string(raw, limits), limits)

        def key_block() -> ttlv.TtlvItem:
            return _structure_with_children(0x0042_0040, limits, [enum(0x0042_0042, 1)])

        typed = []
        source_items = []

        def construct(constructor, fields, object_type: int) -> None:
            source_items.extend(fields)
            typed.append((constructor(fields), object_type))

        construct(ManagedObject.certificate, [
            enum(0x0042_001D, 0x8000_0042), byte_string(0x0042_001E, b"cert")
        ], 1)
        construct(ManagedObject.symmetric_key, [key_block()], 2)
        construct(ManagedObject.public_key, [key_block()], 3)
        construct(ManagedObject.private_key, [key_block()], 4)
        construct(ManagedObject.split_key, [
            integer(0x0042_008B, 2), integer(0x0042_0044, 1),
            integer(0x0042_008C, 2), enum(0x0042_008A, 1), key_block()
        ], 5)
        construct(ManagedObject.secret_data, [
            enum(0x0042_0086, 3), key_block()
        ], 7)
        construct(ManagedObject.opaque_object, [
            enum(0x0042_0059, 9), byte_string(0x0042_005A, b"opaque")
        ], 8)
        construct(ManagedObject.pgp_key, [
            integer(0x0042_00BE, 7), key_block()
        ], 9)
        construct(ManagedObject.certificate_request, [
            enum(0x0042_0019, 4), byte_string(0x0042_0140, b"request")
        ], 10)
        try:
            assert [managed.object_type for managed, _ in typed] == [
                object_type for _, object_type in typed
            ]
            assert typed[0][0].certificate_type == 0x8000_0042
            assert typed[0][0].certificate_value == b"cert"
            assert typed[4][0].key_part_identifier == 1
            assert typed[4][0].split_key_parts == 2
            assert typed[4][0].split_key_threshold == 2
            assert typed[4][0].split_key_method == 1
            assert typed[5][0].secret_data_type == 3
            assert typed[6][0].opaque_data_type == 9
            assert typed[6][0].opaque_data_value == b"opaque"
            assert typed[7][0].pgp_key_version == 7
            assert typed[8][0].certificate_request_type == 4
            assert typed[8][0].certificate_request_value == b"request"
        finally:
            for managed, _ in typed:
                managed.close()
            for item in source_items:
                item.close()


def test_key_block_value_material_and_wrapping_fields_construct_by_format():
    with ttlv.codec_limits_defaults() as limits:
        material_item = _item(
            0x0042_0043,
            ttlv.ttlv_value_byte_string(b"secret-canary", limits),
            limits,
        )
        try:
            with KeyMaterial.from_fields(1, [material_item]) as material:
                assert material.format_raw == 1
                assert "secret-canary" not in repr(material)
            with KeyValue.from_fields(1, [material_item]) as key_value:
                with ttlv.ttlv_item_view_from_item(key_value) as view:
                    with ttlv.ttlv_item_view_tag(view) as tag:
                        assert ttlv.tag_value(tag) == 0x0042_0045
            plaintext = _structure_with_children(
                0x0042_0045,
                limits,
                [_item(0x0042_0043,
                       ttlv.ttlv_value_byte_string(b"secret-canary", limits), limits)],
            )
            format_type = _item(0x0042_0042, ttlv.ttlv_value_enumeration(1), limits)
            try:
                with KeyBlock.from_fields([plaintext, format_type]) as block:
                    with ttlv.ttlv_item_view_from_item(block) as view:
                        with ttlv.ttlv_item_view_tag(view) as tag:
                            assert ttlv.tag_value(tag) == 0x0042_0040
            finally:
                plaintext.close()
                format_type.close()

            method = _item(0x0042_009E, ttlv.ttlv_value_enumeration(1), limits)
            identifier = _item(
                0x0042_0094,
                ttlv.ttlv_value_text_string(b"wrapping-key", limits),
                limits,
            )
            key_information = _structure_with_children(0x0042_0036, limits, [identifier])
            try:
                with KeyWrappingData.from_fields([key_information, method]) as wrapping:
                    assert wrapping.wrapping_method == 1
            finally:
                method.close()
                key_information.close()
        finally:
            material_item.close()


def test_managed_key_readers_cover_transparent_fields_and_wrapping_metadata():
    with ttlv.codec_limits_defaults() as limits:
        exponent = _item(
            0x0042_006C,
            ttlv.ttlv_value_big_integer(b"\x01\x00\x01", limits),
            limits,
        )
        modulus = _item(
            0x0042_0052,
            ttlv.ttlv_value_big_integer(b"\x80\x01", limits),
            limits,
        )
        format_type = _item(0x0042_0042, ttlv.ttlv_value_enumeration(11), limits)
        algorithm = _item(0x0042_0028, ttlv.ttlv_value_enumeration(0x8000_0010), limits)
        length = _item(0x0042_002A, ttlv.ttlv_value_integer(2048), limits)
        method = _item(0x0042_009E, ttlv.ttlv_value_enumeration(0x8000_0020), limits)
        unique_id = _item(
            0x0042_0094, ttlv.ttlv_value_text_string(b"key-id", limits), limits
        )
        mac_signature = _item(
            0x0042_004D, ttlv.ttlv_value_byte_string(b"signature", limits), limits
        )
        iv = _item(0x0042_003D, ttlv.ttlv_value_byte_string(b"nonce", limits), limits)
        encoding = _item(0x0042_00A3, ttlv.ttlv_value_enumeration(2), limits)
        prime_field_size = _item(
            0x0042_0062, ttlv.ttlv_value_big_integer(b"\x17", limits), limits
        )
        parts = _item(0x0042_008B, ttlv.ttlv_value_integer(3), limits)
        identifier = _item(0x0042_0044, ttlv.ttlv_value_integer(1), limits)
        threshold = _item(0x0042_008C, ttlv.ttlv_value_integer(2), limits)
        split_method = _item(0x0042_008A, ttlv.ttlv_value_enumeration(3), limits)
    limits = ttlv.codec_limits_defaults()
    try:
        with KeyMaterial.from_fields(11, [modulus, exponent]) as material:
            assert material.field_type(0x0042_0052) is ttlv.ItemType.BigInteger
            assert material.modulus == b"\x80\x01"
            assert material.public_exponent == b"\x01\x00\x01"
            assert material.field_bytes(0x0042_006C) == b"\x01\x00\x01"
            attributes = _structure_with_children(0x0042_0125, limits, [])
            try:
                with KeyValue.from_fields(11, [material, attributes]) as key_value:
                    assert key_value.key_material_type is ttlv.ItemType.Structure
                    assert key_value.has_attributes
                    with KeyBlock.from_fields(
                        [format_type, key_value, algorithm, length]
                    ) as key_block:
                        assert key_block.key_format_type == 11
                        assert key_block.key_value_type is ttlv.ItemType.Structure
                        assert key_block.cryptographic_algorithm == 0x8000_0010
                        assert key_block.cryptographic_length == 2048
                        assert not key_block.has_key_wrapping_data
                        with ManagedObject.symmetric_key([key_block]) as symmetric:
                            assert symmetric.has_key_block
            finally:
                attributes.close()

        key_info = _structure_with_children(0x0042_0036, limits, [unique_id])
        try:
            with KeyWrappingData.from_fields(
                [method, key_info, mac_signature, iv, encoding]
            ) as wrapping:
                assert wrapping.wrapping_method == 0x8000_0020
                assert wrapping.has_encryption_key_information
                assert not wrapping.has_mac_signature_key_information
                assert wrapping.mac_signature == b"signature"
                assert wrapping.iv_counter_nonce == b"nonce"
                assert wrapping.encoding_option == 2
        finally:
            key_info.close()

        split_key_block = _structure_with_children(
            0x0042_0040, limits,
            [_item(0x0042_0042, ttlv.ttlv_value_enumeration(1), limits)],
        )
        try:
            with ManagedObject.split_key(
                [parts, identifier, threshold, split_method,
                 prime_field_size, split_key_block]
            ) as split:
                assert split.has_key_block
                assert split.prime_field_size == b"\x17"
        finally:
            split_key_block.close()

        # Ensure all direct field handles stay caller-owned throughout construction.
        assert all(not field.closed for field in (
            exponent, modulus, format_type, algorithm, length, method,
            mac_signature, iv, encoding, prime_field_size,
            parts, identifier, threshold, split_method,
        ))
    finally:
        for field in (
            exponent, modulus, format_type, algorithm, length, method,
            unique_id, mac_signature, iv, encoding, prime_field_size,
            parts, identifier, threshold, split_method,
        ):
            field.close()
        limits.close()


def test_typed_readers_cover_every_transparent_key_format():
    with ttlv.codec_limits_defaults() as limits:
        cases = [
            (7, 0x0042_003F, "bytes", b"sym", "key_byte_string"),
            (8, 0x0042_009F, "big", b"x", "x"),
            (9, 0x0042_00A0, "big", b"y", "y"),
            (10, 0x0042_0063, "big", b"d", "private_exponent"),
            (11, 0x0042_006C, "big", b"e", "public_exponent"),
            (12, 0x0042_009F, "big", b"x", "x"),
            (13, 0x0042_00A0, "big", b"y", "y"),
            (0x14, 0x0042_0075, "enum", 0x8000_0014, "recommended_curve"),
            (0x15, 0x0042_0072, "bytes", b"point", "q_string"),
        ]
        fields_by_format = {
            7: [(0x0042_003F, "bytes", b"sym")],
            8: [
                (0x0042_005E, "big", b"p"),
                (0x0042_0071, "big", b"q"),
                (0x0042_0037, "big", b"g"),
                (0x0042_009F, "big", b"x"),
            ],
            9: [
                (0x0042_005E, "big", b"p"),
                (0x0042_0071, "big", b"q"),
                (0x0042_0037, "big", b"g"),
                (0x0042_00A0, "big", b"y"),
            ],
            10: [
                (0x0042_0052, "big", b"n"),
                (0x0042_0063, "big", b"d"),
            ],
            11: [
                (0x0042_0052, "big", b"n"),
                (0x0042_006C, "big", b"e"),
            ],
            12: [
                (0x0042_005E, "big", b"p"),
                (0x0042_0037, "big", b"g"),
                (0x0042_009F, "big", b"x"),
            ],
            13: [
                (0x0042_005E, "big", b"p"),
                (0x0042_0037, "big", b"g"),
                (0x0042_00A0, "big", b"y"),
            ],
            0x14: [
                (0x0042_0075, "enum", 0x8000_0014),
                (0x0042_002E, "big", b"d"),
            ],
            0x15: [
                (0x0042_0075, "enum", 0x8000_0015),
                (0x0042_0072, "bytes", b"point"),
            ],
        }
        for format_raw, field_tag, value_kind, expected, accessor in cases:
            fields = []
            for raw_tag, kind, value in fields_by_format[format_raw]:
                if kind == "enum":
                    ttlv_value = ttlv.ttlv_value_enumeration(value)
                elif kind == "big":
                    ttlv_value = ttlv.ttlv_value_big_integer(value, limits)
                else:
                    ttlv_value = ttlv.ttlv_value_byte_string(value, limits)
                fields.append(_item(raw_tag, ttlv_value, limits))
            try:
                with KeyMaterial.from_fields(format_raw, fields) as material:
                    if value_kind == "enum":
                        assert material.field_type(field_tag) is ttlv.ItemType.Enumeration
                        assert material.field_enumeration(field_tag) == expected
                    else:
                        expected_type = (
                            ttlv.ItemType.ByteString if value_kind == "bytes"
                            else ttlv.ItemType.BigInteger
                        )
                        assert material.field_type(field_tag) is expected_type
                        assert material.field_bytes(field_tag) == expected
                    assert getattr(material, accessor) == expected
            finally:
                for field in fields:
                    field.close()


def test_oasis_hexadecimal_transparent_ec_formats_and_reserved_values():
    with pytest.raises(errors.InvalidInputError):
        KeyMaterial.from_fields(0x14, [])
    with pytest.raises(errors.InvalidInputError):
        KeyMaterial.from_fields(0x15, [])

    with KeyMaterial.from_fields(14, []) as reserved_private:
        assert reserved_private.format_raw() == 14
    with KeyMaterial.from_fields(15, []) as reserved_public:
        assert reserved_public.format_raw() == 15

