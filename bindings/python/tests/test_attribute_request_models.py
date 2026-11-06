from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import attribute_operations, errors, extensions, ttlv


def _attribute_item() -> tuple[ttlv.CodecLimits, ttlv.TtlvItem]:
    limits = ttlv.codec_limits_defaults()
    # KMIP Specification v2.1 §4.12 defines Contact Information as Text String.
    raw_tag = ttlv.raw_tag_from_raw(0x0042_0022)
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
    reference = extensions.AttributeReference.tag(0x0042_0022)
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
        extensions.AttributeReference.tag(0x0042_0022),
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
    reference = extensions.AttributeReference.tag(0x0042_0022)
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


def test_attribute_set_facade_validates_append_and_preserves_owned_order():
    limits, first = _attribute_item()
    source = ttlv.ttlv_structure_create()
    source = ttlv.ttlv_structure_with_item(source, first, limits)
    with source, attribute_operations.AttributeSet.try_new(source) as attributes:
        assert attributes.len() == 1
        assert not attributes.is_empty()

        # KMIP Specification v2.1 §4.13 defines Cryptographic Algorithm as Enumeration.
        second_tag = ttlv.raw_tag_from_raw(0x0042_0028)
        checked_second_tag = ttlv.raw_tag_try_checked(second_tag)
        second_value = ttlv.ttlv_value_enumeration(0xFFFF_FFFF)
        second = ttlv.ttlv_item_create(checked_second_tag, second_value, limits)
        second_tag.close()
        checked_second_tag.close()
        second_value.close()
        attributes.try_push(second)
        second.close()
        assert attributes.len() == 2

        invalid_tag = ttlv.raw_tag_from_raw(0x0042_0028)
        checked_invalid_tag = ttlv.raw_tag_try_checked(invalid_tag)
        invalid_value = ttlv.ttlv_value_integer(7)
        invalid = ttlv.ttlv_item_create(checked_invalid_tag, invalid_value, limits)
        invalid_tag.close()
        checked_invalid_tag.close()
        invalid_value.close()
        with pytest.raises(errors.InvalidInputError):
            attributes.try_push(invalid)
        invalid.close()
        assert attributes.len() == 2, "failed validation must leave the set unchanged"

        with attributes.as_items() as copied:
            with ttlv.ttlv_structure_view(copied) as view:
                assert ttlv.ttlv_structure_view_item_count(view) == 2
                tags = []
                for index in range(2):
                    with ttlv.ttlv_structure_view_item_at(view, index) as item_view:
                        with ttlv.ttlv_item_view_tag(item_view) as tag:
                            tags.append(ttlv.tag_value(tag))
                assert tags == [0x0042_0022, 0x0042_0028]

        with attributes.into_items() as transferred:
            assert attributes.closed
            with ttlv.ttlv_structure_view(transferred) as view:
                assert ttlv.ttlv_structure_view_item_count(view) == 2
    limits.close()


def test_attribute_request_factories_reject_wrong_typed_arguments() -> None:
    reference = extensions.AttributeReference.tag(0x0042_0022)
    with pytest.raises(errors.InvalidInputError):
        attribute_operations._validate_identifier(object())  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations._validate_identifier(extensions.UniqueIdentifier.integer(7))

    with pytest.raises(errors.InvalidInputError):
        attribute_operations.AddAttributeRequest.create(None, None)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.AdjustAttributeRequest.create(None, object(), 1)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.DeleteAttributeRequest.create(None, object(), None)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.DeleteAttributeRequest.create(None, None, object())  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.ModifyAttributeRequest.create(None, None, None)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.SetAttributeRequest.create(None, None)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.GetAttributesRequest.create(None, (reference, object()))  # type: ignore[arg-type]


def test_attribute_set_rejects_wrong_handles_and_invalid_native_empty_flag(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        attribute_operations.AttributeSet.try_new(None)  # type: ignore[arg-type]

    source = ttlv.ttlv_structure_create()
    with source, attribute_operations.AttributeSet.try_new(source) as attributes:
        with pytest.raises(errors.InvalidInputError):
            attributes.try_push(None)  # type: ignore[arg-type]

        native_lib = attribute_operations._client_lib
        monkeypatch.setattr(
            attribute_operations,
            "_client_lib",
            SimpleNamespace(
                kmipkit_attribute_set_is_empty=lambda _attributes, output: output.__setitem__(0, 2) or 0
            ),
        )
        with pytest.raises(errors.InvalidInputError):
            attributes.is_empty()
        monkeypatch.setattr(attribute_operations, "_client_lib", native_lib)


def test_empty_attribute_set_closes_normally() -> None:
    source = ttlv.ttlv_structure_create()
    with source:
        attributes = attribute_operations.AttributeSet.try_new(source)
        attributes.close()
        assert attributes.closed


def test_attribute_request_optional_selectors_remain_absent_and_required_copies_are_owned():
    limits, item = _attribute_item()
    reference = extensions.AttributeReference.tag(0x0042_0022)

    with attribute_operations.AdjustAttributeRequest.create(None, reference, 1) as request:
        request._adjustment_type = None
        with pytest.raises(errors.InvalidInputError):
            _ = request.adjustment_type
        request._attribute_reference_value = None
        with pytest.raises(errors.InvalidInputError):
            _ = request.attribute_reference
        assert request.adjustment_value is None

    with attribute_operations.DeleteAttributeRequest.create(None, None, None) as request:
        assert request.current_attribute is None
        assert request.attribute_reference is None

    with attribute_operations.ModifyAttributeRequest.create(None, None, item) as request:
        assert request.current_attribute is None
        with request.new_attribute as copied:
            assert copied is not None

    with attribute_operations.SetAttributeRequest.create(None, item) as request:
        with request.new_attribute as copied:
            assert copied is not None

    item.close()
    limits.close()
