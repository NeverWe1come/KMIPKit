"""KMIP 3.0 §6.1.10, §6.1.13, §6.1.14, and §6.1.41 semantic routes."""

from __future__ import annotations

from kmipkit import extensions, ttlv
from kmipkit.attribute_operations import AttributeSet
from kmipkit.protocol_values import (
    V3_0_OPERATION_CREATE_GROUP,
    V3_0_OPERATION_CREATE_USER,
    V3_0_OPERATION_DEACTIVATE,
    V3_0_OPERATION_OBLITERATE,
)


def test_system_object_operations_accept_semantic_inputs() -> None:
    with ttlv.ttlv_structure_create() as source, AttributeSet.try_new(source) as attributes:
        with extensions.client_batch_item_create_group_from_attributes(attributes) as item:
            assert item.operation == V3_0_OPERATION_CREATE_GROUP
        with extensions.client_batch_item_create_user_from_attributes(attributes) as item:
            assert item.operation == V3_0_OPERATION_CREATE_USER

    identifier = extensions.UniqueIdentifier.text_string("system-object")
    with extensions.client_batch_item_obliterate_from_identifier(identifier) as item:
        assert item.operation == V3_0_OPERATION_OBLITERATE
    with extensions.client_batch_item_deactivate_from_fields(identifier) as item:
        assert item.operation == V3_0_OPERATION_DEACTIVATE
