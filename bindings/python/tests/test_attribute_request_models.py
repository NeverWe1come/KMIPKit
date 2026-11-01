from __future__ import annotations

import pytest

from kmipkit import attribute_operations, errors, extensions, ttlv


def _attribute_item() -> tuple[ttlv.CodecLimits, ttlv.TtlvItem]:
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(0x0042_000A)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_text_string(b"Counter", limits)
    item = ttlv.ttlv_item_create(tag, value, limits)
    raw_tag.close()
    tag.close()
    value.close()
    return limits, item


def _assert_standalone_request(request, expected_count: int) -> None:
    with request.to_ttlv_payload() as payload:
        with ttlv.ttlv_structure_view(payload) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == expected_count
    with request.to_batch_item() as item:
        assert item is not None


def test_add_attribute_request_is_independent_inspectable_and_batchable():
    limits, item = _attribute_item()
    identifier = extensions.UniqueIdentifier.text_string("object-7")
    with attribute_operations.AddAttributeRequest.create(identifier, item) as request:
        item.close()
        assert request.unique_identifier == identifier
        with request.new_attribute as copied:
            assert copied is not None
        _assert_standalone_request(request, 2)
    limits.close()


def test_adjust_attribute_request_preserves_raw_type_and_optional_value():
    adjustment_value = ttlv.ttlv_value_integer(5)
    reference = extensions.AttributeReference.name("KMIPKit", "Counter")
    with attribute_operations.AdjustAttributeRequest.create(
        extensions.UniqueIdentifier.text_string("object-7"),
        reference,
        1,
        adjustment_value,
    ) as request:
        adjustment_value.close()
        assert request.attribute_reference == reference
        assert request.adjustment_type == 1
        with request.adjustment_value as copied_value:
            assert copied_value is not None
        _assert_standalone_request(request, 4)


def test_delete_modify_and_set_models_remain_separate_from_batch_items():
    limits, current = _attribute_item()
    _, new = _attribute_item()
    reference = extensions.AttributeReference.tag(0x0042_000A)
    models = (
        attribute_operations.DeleteAttributeRequest.create(
            extensions.UniqueIdentifier.text_string("object-7"), current, reference
        ),
        attribute_operations.ModifyAttributeRequest.create(
            extensions.UniqueIdentifier.text_string("object-7"), current, new
        ),
        attribute_operations.SetAttributeRequest.create(
            extensions.UniqueIdentifier.text_string("object-7"), new
        ),
    )
    current.close()
    new.close()
    for request, expected_count in zip(models, (3, 3, 2), strict=True):
        with request:
            if isinstance(request, attribute_operations.DeleteAttributeRequest):
                assert request.attribute_reference == reference
            else:
                assert not hasattr(request, "adjustment_type")
            _assert_standalone_request(request, expected_count)
    limits.close()


def test_get_attributes_request_preserves_order_and_supports_all_attributes():
    identifier = extensions.UniqueIdentifier.text_string("object-7")
    references = (
        extensions.AttributeReference.tag(0x0042_000A),
        extensions.AttributeReference.name("KMIPKit", "Counter"),
    )
    with attribute_operations.GetAttributesRequest.create(identifier, references) as request:
        assert request.unique_identifier == identifier
        assert request.attribute_references == references
        _assert_standalone_request(request, 3)

    with attribute_operations.GetAttributesRequest.create(None, ()) as request:
        assert request.unique_identifier is None
        assert request.attribute_references == ()
        _assert_standalone_request(request, 0)


def test_get_attributes_request_rejects_duplicate_references():
    reference = extensions.AttributeReference.tag(0x0042_000A)
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.GetAttributesRequest.create(None, (reference, reference))


def test_get_attribute_list_request_has_only_its_optional_identifier():
    identifier = extensions.UniqueIdentifier.text_string("object-7")
    with attribute_operations.GetAttributeListRequest.create(identifier) as request:
        assert request.unique_identifier == identifier
        _assert_standalone_request(request, 1)

    with attribute_operations.GetAttributeListRequest.create(None) as request:
        assert request.unique_identifier is None
        _assert_standalone_request(request, 0)
