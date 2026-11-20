"""Tests for the 8 additional task-oriented KMIP client fluent builders."""

from __future__ import annotations

import pytest

from kmipkit import errors, extensions, fluent, ttlv
from kmipkit.protocol_values import ObjectType, Operation


def test_activate_builder() -> None:
    builder = fluent.ActivateBuilder(unique_identifier="uid-activate-1")
    builder.asynchronous_indicator(1)
    builder.validate()
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.ACTIVATE

    # Test via Request
    req = fluent.Request(None)  # type: ignore[arg-type]
    b2 = req.activate("uid-activate-2")
    with b2.build() as item2:
        assert item2.operation == Operation.ACTIVATE


def test_archive_builder() -> None:
    builder = fluent.ArchiveBuilder(unique_identifier="uid-archive-1")
    builder.validate()
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.ARCHIVE

    # Test via Request
    req = fluent.Request(None)  # type: ignore[arg-type]
    b2 = req.archive("uid-archive-2")
    with b2.build() as item2:
        assert item2.operation == Operation.ARCHIVE


def test_recover_builder() -> None:
    builder = fluent.RecoverBuilder(unique_identifier="uid-recover-1")
    builder.validate()
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.RECOVER

    # Test via Request
    req = fluent.Request(None)  # type: ignore[arg-type]
    b2 = req.recover("uid-recover-2")
    with b2.build() as item2:
        assert item2.operation == Operation.RECOVER


def test_create_split_key_builder() -> None:
    builder = (
        fluent.CreateSplitKeyBuilder()
        .object_type(ObjectType.SYMMETRIC_KEY)
        .method(1)  # Polynomial sharing
        .parts(5)
        .threshold(3)
        .unique_identifier("base-key-uid")
        .prime_field_size(b"\x00" * 32)
    )
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.CREATE_SPLIT_KEY

    # Zeroization test on close
    secret_bytes = bytearray(b"\xaa" * 32)
    builder2 = fluent.CreateSplitKeyBuilder().prime_field_size(secret_bytes)
    assert builder2._prime_field_size == b"\xaa" * 32
    builder2.close()
    assert builder2._prime_field_size is None


def test_register_builder() -> None:
    with ttlv.codec_limits_defaults() as limits:
        with (
            ttlv.raw_tag_from_raw(0x420059) as r1,
            ttlv.raw_tag_try_checked(r1) as tag_type,
            ttlv.ttlv_value_enumeration(0x80000001) as val_type,
            ttlv.ttlv_item_create(tag_type, val_type, limits) as item_type,
            ttlv.raw_tag_from_raw(0x42005A) as r2,
            ttlv.raw_tag_try_checked(r2) as tag_data,
            ttlv.ttlv_value_byte_string(b"register-object-fixture", limits) as val_data,
            ttlv.ttlv_item_create(tag_data, val_data, limits) as item_data,
        ):
            fields = ttlv.ttlv_structure_create()
            fields = ttlv.ttlv_structure_with_item(fields, item_type, limits)
            fields = ttlv.ttlv_structure_with_item(fields, item_data, limits)

        with (
            ttlv.raw_tag_from_raw(0x42005B) as r3,
            ttlv.raw_tag_try_checked(r3) as tag_obj,
            ttlv.ttlv_value_structure(fields, limits) as val_obj,
        ):
            obj_item = ttlv.ttlv_item_create(tag_obj, val_obj, limits)

    builder = (
        fluent.RegisterBuilder()
        .object_item(8, obj_item)
        .opaque_object_context(extensions.OpaqueObjectContext.GENERIC)
    )
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.REGISTER

    obj_item.close()


def test_rekey_builder() -> None:
    builder = (
        fluent.RekeyBuilder()
        .unique_identifier("uid-to-rekey")
        .offset(3600)
    )
    builder.validate()
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.RE_KEY


def test_rekey_key_pair_builder() -> None:
    builder = (
        fluent.RekeyKeyPairBuilder()
        .private_key_unique_identifier("priv-uid-to-rekey")
        .offset(7200)
    )
    builder.validate()
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.RE_KEY_KEY_PAIR


def test_recertify_builder() -> None:
    csr_data = b"dummy-csr-data"
    builder = (
        fluent.RecertifyBuilder()
        .unique_identifier("cert-to-recertify")
        .certificate_request_type(1)
        .certificate_request_value(csr_data)
        .offset(1800)
    )
    builder.validate()
    with builder.build() as item:
        assert isinstance(item, extensions.ClientBatchItem)
        assert item.operation == Operation.RE_CERTIFY

    # Verify wiping of certificate_request_value on close
    assert builder._certificate_request_value is not None
    builder.close()
    assert builder._certificate_request_value is None


def test_fluent_builder_rejection_of_invalid_inputs() -> None:
    b = fluent.ActivateBuilder()
    with pytest.raises(errors.KmipKitError):
        b.unique_identifier(12345)  # type: ignore[arg-type]

    sk = fluent.CreateSplitKeyBuilder()
    with pytest.raises(errors.KmipKitError):
        sk.build()  # Missing required fields

    reg = fluent.RegisterBuilder()
    with pytest.raises(errors.KmipKitError):
        reg.build()  # Missing object
