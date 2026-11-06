"""Four-language typed direct-attribute behavior for KMIPKIT-0053."""

import pytest

from kmipkit import errors, ttlv
from kmipkit.attribute_operations import AttributeSet
from kmipkit.typed_attributes import StandardAttribute, TypedAttribute


def _item(attribute: StandardAttribute, value: ttlv.Value) -> ttlv.TtlvItem:
    with ttlv.raw_tag_from_raw(attribute.tag) as raw_tag:
        tag = ttlv.raw_tag_try_checked(raw_tag)
        limits = ttlv.codec_limits_defaults()
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            limits.close()


def _raw_item(raw_tag: int, value: ttlv.Value) -> ttlv.TtlvItem:
    with ttlv.raw_tag_from_raw(raw_tag) as raw_tag_handle:
        tag = ttlv.raw_tag_try_checked(raw_tag_handle)
        limits = ttlv.codec_limits_defaults()
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            limits.close()


def _append(structure: ttlv.Structure, raw_tag: int, value: ttlv.Value,
            limits: ttlv.CodecLimits) -> ttlv.Structure:
    with ttlv.raw_tag_from_raw(raw_tag) as raw_tag_handle:
        tag = ttlv.raw_tag_try_checked(raw_tag_handle)
        try:
            item = ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
    return ttlv.ttlv_structure_with_item(structure, item, limits)


def _value_for(attribute: StandardAttribute, limits: ttlv.CodecLimits) -> ttlv.Value:
    kind = attribute.value_kind
    if kind == "BigInteger":
        return ttlv.ttlv_value_big_integer(b"\x01", limits)
    if kind == "Boolean":
        return ttlv.ttlv_value_boolean(True)
    if kind == "ByteString":
        return ttlv.ttlv_value_byte_string(b"\x01", limits)
    if kind == "DateTime":
        return ttlv.ttlv_value_date_time(1_700_000_000)
    if kind == "DateTimeExtended":
        return ttlv.ttlv_value_date_time_extended(1_700_000_000)
    if kind == "Enumeration":
        return ttlv.ttlv_value_enumeration(0xFFFF_FFFF)
    if kind == "Integer":
        return ttlv.ttlv_value_integer(7)
    if kind == "Interval":
        return ttlv.ttlv_value_interval(7)
    if kind in ("TextString", "UniqueIdentifier"):
        return ttlv.ttlv_value_text_string(b"attribute-value", limits)
    if kind == "Structure":
        structure = ttlv.ttlv_structure_create()
        if attribute is StandardAttribute.VENDOR_ATTRIBUTE:
            structure = _append(
                structure, 0x0042_009D,
                ttlv.ttlv_value_text_string(b"KMIPKit.TestVendor_1", limits), limits,
            )
            structure = _append(
                structure, 0x0042_000A,
                ttlv.ttlv_value_text_string(b"Opaque.Attribute", limits), limits,
            )
            structure = _append(
                structure, 0x0042_000B,
                ttlv.ttlv_value_byte_string(b"\x01", limits), limits,
            )
        return ttlv.ttlv_value_structure(structure, limits)
    raise AssertionError(f"unhandled catalogued attribute kind: {kind}")


def _expected_item_type(kind: str) -> ttlv.ItemType:
    return {
        "BigInteger": ttlv.ItemType.BigInteger,
        "Boolean": ttlv.ItemType.Boolean,
        "ByteString": ttlv.ItemType.ByteString,
        "DateTime": ttlv.ItemType.DateTime,
        "DateTimeExtended": ttlv.ItemType.DateTimeExtended,
        "Enumeration": ttlv.ItemType.Enumeration,
        "Integer": ttlv.ItemType.Integer,
        "Interval": ttlv.ItemType.Interval,
        "Structure": ttlv.ItemType.Structure,
        "TextString": ttlv.ItemType.TextString,
        "UniqueIdentifier": ttlv.ItemType.TextString,
    }[kind]


def test_catalog_exposes_all_sixty_two_direct_attributes_with_unique_tags():
    assert len(StandardAttribute) == 62
    assert len({attribute.tag for attribute in StandardAttribute}) == 62
    assert StandardAttribute.ACTIVATION_DATE.tag == 0x0042_0001
    assert StandardAttribute.VENDOR_ATTRIBUTE.tag == 0x0042_0008


def test_typed_attribute_roundtrips_all_catalogued_source_ttlv_types():
    with ttlv.codec_limits_defaults() as limits:
        for attribute in StandardAttribute:
            source = _item(attribute, _value_for(attribute, limits))
            with source, TypedAttribute.from_item(attribute, source) as typed:
                with ttlv.ttlv_item_view_from_item(typed) as item_view:
                    with ttlv.ttlv_item_view_value(item_view) as value_view:
                        assert ttlv.ttlv_value_view_type(value_view) == _expected_item_type(
                            attribute.value_kind
                        ), attribute.name


def test_typed_attribute_constructs_and_reads_actual_catalogued_values():
    with ttlv.codec_limits_defaults() as limits:
        for attribute in StandardAttribute:
            value = _value_for(attribute, limits)
            with value, TypedAttribute.create(attribute, value) as typed:
                with typed.value as copied, ttlv.ttlv_value_view(copied) as actual:
                    assert ttlv.ttlv_value_view_type(actual) == _expected_item_type(
                        attribute.value_kind
                    ), attribute.name
                    if attribute.value_kind == "Enumeration":
                        assert ttlv.ttlv_value_view_enumeration(actual) == 0xFFFF_FFFF
                    elif attribute.value_kind == "BigInteger":
                        assert bytes(ttlv.ttlv_value_view_byte_at(actual, index)
                                     for index in range(ttlv.ttlv_value_view_byte_length(actual))) == b"\x01"
                    elif attribute.value_kind == "Integer":
                        assert ttlv.ttlv_value_view_integer(actual) == 7
                    elif attribute.value_kind == "Boolean":
                        assert ttlv.ttlv_value_view_boolean(actual) is True
                    elif attribute.value_kind in ("ByteString", "TextString"):
                        expected = b"\x01" if attribute.value_kind == "ByteString" else b"attribute-value"
                        assert bytes(ttlv.ttlv_value_view_byte_at(actual, index)
                                     for index in range(ttlv.ttlv_value_view_byte_length(actual))) == expected
                    elif attribute.value_kind == "DateTime":
                        assert ttlv.ttlv_value_view_date_time(actual) == 1_700_000_000
                    elif attribute.value_kind == "DateTimeExtended":
                        assert ttlv.ttlv_value_view_date_time_extended(actual) == 1_700_000_000
                    elif attribute.value_kind == "Interval":
                        assert ttlv.ttlv_value_view_interval(actual) == 7
                    elif attribute.value_kind == "Structure":
                        with ttlv.ttlv_value_view_structure(actual) as structure:
                            expected = 3 if attribute is StandardAttribute.VENDOR_ATTRIBUTE else 0
                            assert ttlv.ttlv_structure_view_item_count(structure) == expected
                            if expected:
                                for index, (expected_tag, expected_value) in enumerate((
                                    (0x0042_009D, b"KMIPKit.TestVendor_1"),
                                    (0x0042_000A, b"Opaque.Attribute"),
                                    (0x0042_000B, b"\x01"),
                                )):
                                    with ttlv.ttlv_structure_view_item_at(structure, index) as child:
                                        with ttlv.ttlv_item_view_tag(child) as tag:
                                            assert ttlv.tag_value(tag) == expected_tag
                                        with ttlv.ttlv_item_view_value(child) as child_value:
                                            assert bytes(
                                                ttlv.ttlv_value_view_byte_at(child_value, offset)
                                                for offset in range(
                                                    ttlv.ttlv_value_view_byte_length(child_value)
                                                )
                                            ) == expected_value


@pytest.mark.parametrize("variant", ["open-enumeration", "integer"])
def test_unique_identifier_preserves_open_non_text_variants(variant: str):
    value = (
        ttlv.ttlv_value_enumeration(0xFFFF_FFFF)
        if variant == "open-enumeration"
        else ttlv.ttlv_value_integer(-23)
    )
    with value, TypedAttribute.create(StandardAttribute.UNIQUE_IDENTIFIER, value) as typed:
        with typed.value as copied, ttlv.ttlv_value_view(copied) as actual:
            if ttlv.ttlv_value_view_type(actual) == ttlv.ItemType.Enumeration:
                assert ttlv.ttlv_value_view_enumeration(actual) == 0xFFFF_FFFF
            else:
                assert ttlv.ttlv_value_view_integer(actual) == -23


def test_attribute_set_typed_insert_read_and_iteration_preserve_repeats():
    limits = ttlv.codec_limits_defaults()
    first_value = ttlv.ttlv_value_enumeration(0xFFFF_FFFF)
    second_value = ttlv.ttlv_value_enumeration(0xFFFF_FFFE)
    with (
        limits,
        first_value,
        second_value,
        TypedAttribute.create(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, first_value) as first,
        TypedAttribute.create(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, second_value) as second,
        AttributeSet.try_new(ttlv.ttlv_structure_create()) as attributes,
    ):
        attributes.try_push_typed(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, first)
        attributes.try_push_typed(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, second)
        assert attributes.typed_count(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM) == 2
        with attributes.typed_at(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM, 1) as actual:
            with actual.value as copied, ttlv.ttlv_value_view(copied) as value:
                assert ttlv.ttlv_value_view_enumeration(value) == 0xFFFF_FFFE
        typed = attributes.typed(StandardAttribute.CRYPTOGRAPHIC_ALGORITHM)
        try:
            assert len(typed) == 2
        finally:
            for attribute in typed:
                attribute.close()


def test_typed_attribute_copies_and_reads_the_item_without_consuming_source():
    source = _item(
        StandardAttribute.ACTIVATION_DATE,
        ttlv.ttlv_value_date_time(1_700_000_000),
    )
    with source, TypedAttribute.from_item(StandardAttribute.ACTIVATION_DATE, source) as typed:
        copied = typed.as_item()
        with copied, ttlv.ttlv_item_view_from_item(copied) as view:
            with ttlv.ttlv_item_view_tag(view) as tag:
                assert ttlv.tag_value(tag) == StandardAttribute.ACTIVATION_DATE.tag
            with ttlv.ttlv_item_view_value(view) as value:
                assert ttlv.ttlv_value_view_date_time(value) == 1_700_000_000
        assert not source.closed


def test_typed_attribute_rejects_wrong_tag_and_value_type_without_exposing_values():
    wrong_tag = _item(
        StandardAttribute.ACTIVATION_DATE,
        ttlv.ttlv_value_date_time(17),
    )
    wrong_type = _item(
        StandardAttribute.NAME,
        ttlv.ttlv_value_date_time(17),
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            TypedAttribute.from_item(StandardAttribute.NAME, wrong_tag)
        with pytest.raises(errors.InvalidInputError):
            TypedAttribute.from_item(StandardAttribute.NAME, wrong_type)
    finally:
        wrong_tag.close()
        wrong_type.close()


def test_repeatable_standard_attribute_remains_lossless_in_generic_attribute_set():
    first = _item(
        StandardAttribute.APPLICATION_SPECIFIC_INFORMATION,
        ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), ttlv.codec_limits_defaults()),
    )
    second = _item(
        StandardAttribute.APPLICATION_SPECIFIC_INFORMATION,
        ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), ttlv.codec_limits_defaults()),
    )
    items = ttlv.ttlv_structure_create()
    items = ttlv.ttlv_structure_with_item(items, first, ttlv.codec_limits_defaults())
    items = ttlv.ttlv_structure_with_item(items, second, ttlv.codec_limits_defaults())
    try:
        with items, AttributeSet.try_new(items) as attributes:
            assert attributes.len() == 2
    finally:
        first.close()
        second.close()


def test_unknown_generic_item_survives_attribute_set_typed_operations():
    value = ttlv.ttlv_value_text_string(b"opaque-value", ttlv.codec_limits_defaults())
    with value:
        item = _raw_item(0x0054_0001, value)
    structure = ttlv.ttlv_structure_create()
    structure = ttlv.ttlv_structure_with_item(structure, item, ttlv.codec_limits_defaults())
    try:
        with structure, AttributeSet.try_new(structure) as attributes:
            known_value = ttlv.ttlv_value_text_string(b"known-name", ttlv.codec_limits_defaults())
            with known_value, TypedAttribute.create(
                StandardAttribute.UNIQUE_IDENTIFIER, known_value
            ) as typed:
                attributes.try_push_typed(StandardAttribute.UNIQUE_IDENTIFIER, typed)
                assert attributes.typed_count(StandardAttribute.UNIQUE_IDENTIFIER) == 1
                with attributes.typed_at(StandardAttribute.UNIQUE_IDENTIFIER, 0) as copied_typed:
                    with copied_typed.value as copied_value, ttlv.ttlv_value_view(copied_value) as view:
                        assert bytes(
                            ttlv.ttlv_value_view_byte_at(view, index)
                            for index in range(ttlv.ttlv_value_view_byte_length(view))
                        ) == b"known-name"
            assert attributes.len() == 2
            with attributes.as_items() as copied, ttlv.ttlv_structure_view(copied) as view:
                assert ttlv.ttlv_structure_view_item_count(view) == 2
                with ttlv.ttlv_structure_view_item_at(view, 0) as child:
                    with ttlv.ttlv_item_view_tag(child) as tag:
                        assert ttlv.tag_value(tag) == 0x0054_0001
                    with ttlv.ttlv_item_view_value(child) as actual:
                        assert bytes(ttlv.ttlv_value_view_byte_at(actual, index)
                                     for index in range(ttlv.ttlv_value_view_byte_length(actual))) == b"opaque-value"
                with ttlv.ttlv_structure_view_item_at(view, 1) as child:
                    with ttlv.ttlv_item_view_tag(child) as tag:
                        assert ttlv.tag_value(tag) == StandardAttribute.UNIQUE_IDENTIFIER.tag
    finally:
        item.close()


def test_typed_attribute_repr_and_validation_error_redact_sensitive_value():
    marker = b"typed-attribute-private-marker"
    value = ttlv.ttlv_value_text_string(marker, ttlv.codec_limits_defaults())
    with value:
        with pytest.raises(errors.InvalidInputError) as caught:
            TypedAttribute.create(StandardAttribute.ACTIVATION_DATE, value)
        assert marker.decode() not in str(caught.value)

    value = ttlv.ttlv_value_text_string(marker, ttlv.codec_limits_defaults())
    with value, TypedAttribute.create(StandardAttribute.UNIQUE_IDENTIFIER, value) as typed:
        assert marker.decode() not in repr(typed)
        with typed.value as copied:
            assert marker.decode() not in repr(copied)

