"""Consumer parity tests for KMIPKIT-0044 Register context selection."""

from __future__ import annotations

from functools import lru_cache

import pytest

from kmipkit import errors, extensions, ttlv


_OBJECT_TYPE_OPAQUE = 8
_OPAQUE_OBJECT_TAG = 0x42005B
_OPAQUE_DATA_TYPE_TAG = 0x420059
_OPAQUE_DATA_VALUE_TAG = 0x42005A
_CRYPTOGRAPHIC_ALGORITHM_TAG = 0x420028
_CRYPTOGRAPHIC_LENGTH_TAG = 0x42002A
_CRYPTOGRAPHIC_USAGE_MASK_TAG = 0x42002C
_UNKNOWN_ATTRIBUTE_TAG = 0x541234


@lru_cache(maxsize=1)
def _limits() -> ttlv.CodecLimits:
    return ttlv.codec_limits_defaults()


def _item(raw_tag: int, value: ttlv.Value) -> ttlv.TtlvItem:
    raw = ttlv.raw_tag_from_raw(raw_tag)
    tag = ttlv.raw_tag_try_checked(raw)
    return ttlv.ttlv_item_create(tag, value, _limits())


def _opaque_object() -> ttlv.TtlvItem:
    fields = ttlv.ttlv_structure_create()
    fields = ttlv.ttlv_structure_with_item(
        fields, _item(_OPAQUE_DATA_TYPE_TAG, ttlv.ttlv_value_enumeration(0x80000001)), _limits()
    )
    fields = ttlv.ttlv_structure_with_item(
        fields,
        _item(_OPAQUE_DATA_VALUE_TAG, ttlv.ttlv_value_byte_string(b"opaque-data", _limits())),
        _limits(),
    )
    return _item(_OPAQUE_OBJECT_TAG, ttlv.ttlv_value_structure(fields, _limits()))


def _attributes(
    *, usage_mask: bool = True, algorithm: bool = True, length: bool = True
) -> ttlv.Structure:
    attributes = ttlv.ttlv_structure_create()
    if usage_mask:
        attributes = ttlv.ttlv_structure_with_item(
            attributes,
            _item(_CRYPTOGRAPHIC_USAGE_MASK_TAG, ttlv.ttlv_value_integer(3)),
            _limits(),
        )
    if algorithm:
        attributes = ttlv.ttlv_structure_with_item(
            attributes,
            _item(_CRYPTOGRAPHIC_ALGORITHM_TAG, ttlv.ttlv_value_enumeration(3)),
            _limits(),
        )
    if length:
        attributes = ttlv.ttlv_structure_with_item(
            attributes,
            _item(_CRYPTOGRAPHIC_LENGTH_TAG, ttlv.ttlv_value_integer(256)),
            _limits(),
        )
    # Retain an unknown vendor attribute after the standard attributes.
    return ttlv.ttlv_structure_with_item(
        attributes,
        _item(_UNKNOWN_ATTRIBUTE_TAG, ttlv.ttlv_value_enumeration(0xE1020304)),
        _limits(),
    )


def test_register_consumer_accepts_generic_opaque_object_without_crypto_attributes() -> None:
    request = extensions.RegisterRequest.create(
        _OBJECT_TYPE_OPAQUE, ttlv.ttlv_structure_create(), _opaque_object()
    )

    with extensions.client_batch_item_register(request) as item:
        assert isinstance(item, extensions.ClientBatchItem)


def test_register_consumer_accepts_managed_context_with_required_attributes() -> None:
    request = extensions.RegisterRequest.create(
        _OBJECT_TYPE_OPAQUE, _attributes(), _opaque_object()
    ).with_opaque_object_context(
        extensions.OpaqueObjectContext.MANAGED_CRYPTOGRAPHIC_OBJECT
    )

    with extensions.client_batch_item_register(request) as item:
        assert isinstance(item, extensions.ClientBatchItem)


@pytest.mark.parametrize(
    "omitted",
    ["usage_mask", "algorithm", "length"],
)
def test_register_consumer_rejects_managed_context_missing_each_required_attribute(
    omitted: str,
) -> None:
    present = {"usage_mask": True, "algorithm": True, "length": True}
    present[omitted] = False
    request = extensions.RegisterRequest.create(
        _OBJECT_TYPE_OPAQUE, _attributes(**present), _opaque_object()
    ).with_opaque_object_context(
        extensions.OpaqueObjectContext.MANAGED_CRYPTOGRAPHIC_OBJECT
    )

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_register(request)


def test_register_consumer_rejects_opaque_context_for_a_non_opaque_object_type() -> None:
    request = extensions.RegisterRequest.create(
        2, _attributes(), _opaque_object()
    ).with_opaque_object_context(extensions.OpaqueObjectContext.GENERIC)

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_register(request)


def test_register_request_rejects_out_of_range_object_type_before_ffi() -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.RegisterRequest.create(-1, None, None)


def test_register_request_rejects_unknown_context_without_consuming_request() -> None:
    request = extensions.RegisterRequest.create(
        _OBJECT_TYPE_OPAQUE, ttlv.ttlv_structure_create(), _opaque_object()
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            request.with_opaque_object_context("managed")
    finally:
        request.close()
