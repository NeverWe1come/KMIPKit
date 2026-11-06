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
    limits = ttlv.codec_limits_defaults()
    tag = ttlv.raw_tag_try_checked(ttlv.raw_tag_from_raw(raw_tag))
    value = ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits)
    return ttlv.ttlv_item_create(tag, value, limits)


def test_managed_object_unknown_type_roundtrips_without_losing_generic_ttlv():
    source = _structure_item(0x0054_0001)
    with source, ManagedObject.from_item(0x8000_0001, source) as managed:
        assert managed.object_type == 0x8000_0001
        with managed.raw() as copied, ttlv.ttlv_item_view_from_item(copied) as view:
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
    key_material = ttlv.ttlv_item_create(
        ttlv.raw_tag_try_checked(ttlv.raw_tag_from_raw(0x0042_0043)),
        ttlv.ttlv_value_byte_string(b"secret-canary", ttlv.codec_limits_defaults()),
        ttlv.codec_limits_defaults(),
    )
    try:
        with key_material, KeyMaterial.from_item(1, key_material) as material:
            assert material.format_raw == 1
            with material.raw() as copied:
                assert not copied.closed
    finally:
        key_material.close()

