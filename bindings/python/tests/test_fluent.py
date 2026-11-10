from __future__ import annotations

import pytest

from kmipkit import client, errors, extensions, fluent


def test_fluent_symmetric_builder_requires_explicit_key_properties() -> None:
    builder = fluent.SymmetricKeyBuilder()
    try:
        builder.aes_256()
        with pytest.raises(errors.KmipKitError):
            builder.build()
    finally:
        builder.close()


def test_client_request_namespace_and_named_identifier_validation() -> None:
    assert hasattr(client.Client, "request")
    assert hasattr(client.Client, "create_symmetric_key")
    assert hasattr(client.Client, "get_by_id")
    assert hasattr(client.Client, "destroy_by_id")
    with pytest.raises(errors.KmipKitError):
        fluent._identifier("bad")


def test_builder_builds_existing_typed_batch_item() -> None:
    with fluent.SymmetricKeyBuilder() as builder:
        item = builder.aes_256().usage_mask(12).name("test-key").build()
    batch = client.Batch.create()
    try:
        batch.add_item(item)
        assert batch.item_count == 1
    finally:
        batch.close()


def test_builder_context_manager_releases_native_handle() -> None:
    builder = fluent.SymmetricKeyBuilder()
    with builder:
        assert builder._handle != fluent.ffi.NULL
    assert builder._handle == fluent.ffi.NULL
    with pytest.raises(RuntimeError, match="closed"):
        builder.name("too-late")


def test_legacy_extensions_remain_available() -> None:
    assert extensions.UniqueIdentifier.text_string("key-1").text == "key-1"
