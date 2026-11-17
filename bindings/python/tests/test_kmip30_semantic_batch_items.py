"""KMIP 3.0 §6.1.10, §6.1.13, §6.1.14, and §6.1.41 semantic routes."""

from __future__ import annotations

import pytest
from typing import get_type_hints

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


def test_kmip30_semantic_binding_annotations_are_resolvable() -> None:
    for function_name in (
        "client_batch_item_create_credential_from_fields",
        "client_batch_item_encapsulate_from_fields",
        "client_batch_item_decapsulate_from_fields",
    ):
        get_type_hints(getattr(extensions, function_name))


def test_kmip30_response_readers_exist_on_client() -> None:
    from kmipkit import client

    for dataclass_name in (
        "CreateGroupResponse",
        "CreateUserResponse",
        "CreateCredentialResponse",
        "DeactivateResponse",
        "EncapsulateResponse",
        "DecapsulateResponse",
    ):
        assert hasattr(client, dataclass_name), f"missing {dataclass_name}"

    for field_name in (
        "create_group_response",
        "create_user_response",
        "create_credential_response",
        "obliterate_response",
        "deactivate_response",
        "encapsulate_response",
        "decapsulate_response",
    ):
        assert field_name in client.BatchItemResult.__dataclass_fields__, f"missing field {field_name}"


def test_kmip30_response_readers_logic(monkeypatch: pytest.MonkeyPatch) -> None:
    from types import SimpleNamespace
    from kmipkit import client

    fake_lib = SimpleNamespace(**{name: getattr(client.lib, name) for name in dir(client.lib) if not name.startswith("_")})
    monkeypatch.setattr(client, "lib", fake_lib)

    # 1. Test obliterate_response
    def fake_obliterate_present(item: object, out_present: object) -> int:
        out_present[0] = 1  # type: ignore[index]
        return 0

    fake_lib.kmipkit_client_batch_item_response_obliterate = fake_obliterate_present
    dummy_item = SimpleNamespace(_pointer=lambda: client.ffi.NULL)
    assert client._obliterate_response(dummy_item) is True

    def fake_obliterate_absent(item: object, out_present: object) -> int:
        out_present[0] = 0  # type: ignore[index]
        return 0

    fake_lib.kmipkit_client_batch_item_response_obliterate = fake_obliterate_absent
    assert client._obliterate_response(dummy_item) is None

    # 2. Test encapsulate_response
    uid_bytes = b"encaps-uid"
    uid_c = client.ffi.new("char[]", uid_bytes)
    data_bytes = b"secret-data"
    data_c = client.ffi.new("char[]", data_bytes)

    def fake_encapsulate_present(
        item: object,
        out_present: object,
        out_uid: object,
        out_data_present: object,
        out_data: object,
    ) -> int:
        out_present[0] = 1  # type: ignore[index]
        out_uid.kind = 1  # TEXT_STRING
        out_uid.text_data = uid_c
        out_uid.text_length = len(uid_bytes)
        out_data_present[0] = 1  # type: ignore[index]
        out_data.data = data_c
        out_data.length = len(data_bytes)
        return 0

    fake_lib.kmipkit_client_batch_item_response_encapsulate = fake_encapsulate_present
    res = client._encapsulate_response(dummy_item)
    assert res is not None
    assert res.unique_identifier.text == "encaps-uid"
    assert res.data == data_bytes

    def fake_encapsulate_absent(
        item: object,
        out_present: object,
        out_uid: object,
        out_data_present: object,
        out_data: object,
    ) -> int:
        out_present[0] = 0  # type: ignore[index]
        return 0

    fake_lib.kmipkit_client_batch_item_response_encapsulate = fake_encapsulate_absent
    assert client._encapsulate_response(dummy_item) is None

    # 3. Test lifecycle_response for create_group, create_user, create_credential, deactivate, decapsulate
    for op_name, resp_cls in (
        ("create_group", client.CreateGroupResponse),
        ("create_user", client.CreateUserResponse),
        ("create_credential", client.CreateCredentialResponse),
        ("deactivate", client.DeactivateResponse),
        ("decapsulate", client.DecapsulateResponse),
    ):
        def make_fake_lifecycle(name: str):
            def fake_lc(item: object, out_present: object, out_uid: object) -> int:
                out_present[0] = 1  # type: ignore[index]
                out_uid.kind = 1  # TEXT_STRING
                out_uid.text_data = uid_c
                out_uid.text_length = len(uid_bytes)
                return 0
            return fake_lc

        setattr(fake_lib, f"kmipkit_client_batch_item_response_{op_name}", make_fake_lifecycle(op_name))
        resp = client._lifecycle_response(dummy_item, op_name, resp_cls)
        assert resp is not None
        assert isinstance(resp, resp_cls)
        assert resp.unique_identifier.text == "encaps-uid"
