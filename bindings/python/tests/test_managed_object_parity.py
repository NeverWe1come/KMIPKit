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

