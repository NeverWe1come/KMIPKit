"""KMIP 3.0 §6.1.10, §6.1.13, §6.1.14, and §6.1.41 semantic routes."""

from __future__ import annotations

import pytest

from kmipkit import errors, extensions, ttlv
from kmipkit.attribute_operations import AttributeSet
from kmipkit.credentials import CredentialType
from kmipkit.protocol_values import (
    V3_0_OBJECT_TYPE_DEVICE_CREDENTIAL,
    V3_0_OPERATION_CREATE_CREDENTIAL,
    V3_0_OPERATION_CREATE_GROUP,
    V3_0_OPERATION_DECAPSULATE,
    V3_0_OPERATION_ENCAPSULATE,
    V3_0_OPERATION_CREATE_USER,
    V3_0_OPERATION_DEACTIVATE,
    V3_0_OPERATION_OBLITERATE,
)


def test_system_object_operations_accept_semantic_inputs() -> None:
    with ttlv.ttlv_structure_create() as source:
        attributes = AttributeSet.try_new(source)
    with attributes:
        create_group = extensions.client_batch_item_create_group_from_attributes(attributes)
        create_user = extensions.client_batch_item_create_user_from_attributes(attributes)
    with create_group, create_user:
        assert create_group.operation == V3_0_OPERATION_CREATE_GROUP
        assert create_user.operation == V3_0_OPERATION_CREATE_USER

    identifier = extensions.UniqueIdentifier.text_string("system-object")
    with extensions.client_batch_item_obliterate_from_identifier(identifier) as item:
        assert item.operation == V3_0_OPERATION_OBLITERATE
    with extensions.client_batch_item_deactivate_from_fields(identifier) as item:
        assert item.operation == V3_0_OPERATION_DEACTIVATE

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_obliterate_from_identifier(None)  # type: ignore[arg-type]


def test_credential_and_kem_operations_accept_semantic_inputs() -> None:
    limits = ttlv.codec_limits_defaults()
    with limits, ttlv.raw_tag_from_raw(0x004200A2) as identifier_raw_tag:
        identifier_tag = ttlv.raw_tag_try_checked(identifier_raw_tag)
        with identifier_tag, ttlv.ttlv_structure_create() as empty_fields:
            identifier_value = ttlv.ttlv_value_text_string(b"device-id", limits)
            with identifier_value:
                identifier_item = ttlv.ttlv_item_create(
                    identifier_tag, identifier_value, limits
                )
            with identifier_item:
                fields = ttlv.ttlv_structure_with_item(
                    empty_fields, identifier_item, limits
                )
            with fields, ttlv.raw_tag_from_raw(0x004201A6) as raw_tag:
                tag = ttlv.raw_tag_try_checked(raw_tag)
                with tag:
                    value = ttlv.ttlv_value_structure(fields, limits)
                    with value:
                        object_item = ttlv.ttlv_item_create(tag, value, limits)
        with object_item:
            with ttlv.ttlv_structure_create() as source:
                attributes = AttributeSet.try_new(source)
            with attributes:
                with extensions.client_batch_item_create_credential_from_fields(
                    CredentialType.from_raw(CredentialType.DEVICE),
                    V3_0_OBJECT_TYPE_DEVICE_CREDENTIAL,
                    attributes,
                    object_item,
                ) as item:
                    assert item.operation == V3_0_OPERATION_CREATE_CREDENTIAL
                with pytest.raises(errors.InvalidInputError):
                    extensions.client_batch_item_create_credential_from_fields(
                        CredentialType.from_raw(CredentialType.DEVICE),
                        13,
                        attributes,
                        object_item,
                    )
                identifier = extensions.UniqueIdentifier.text_string("kem-key")
                with extensions.client_batch_item_encapsulate_from_fields(
                    identifier, attributes=attributes
                ) as item:
                    assert item.operation == V3_0_OPERATION_ENCAPSULATE
                with extensions.client_batch_item_decapsulate_from_fields(
                    identifier, b"12", attributes=attributes
                ) as item:
                    assert item.operation == V3_0_OPERATION_DECAPSULATE
