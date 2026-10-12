from __future__ import annotations

import pytest

from kmipkit import errors, metadata


def test_version_reports_package_and_abi_components() -> None:
    assert metadata.get_version() == metadata.Version(1, 0, 0, 1)


def test_capability_query_only_reports_manifest_ready_c_capabilities() -> None:
    assert metadata.capability_is_available("generic_ttlv_model")
    assert not metadata.capability_is_available("unknown_capability")


def test_capability_query_limits_standard_utf8_encoded_bytes() -> None:
    assert not metadata.capability_is_available("🔐" * 32)
    with pytest.raises(errors.ResourceLimitError):
        metadata.capability_is_available("🔐" * 33)


def test_error_snapshot_is_safe_non_destructive_and_explicitly_clearable() -> None:
    metadata._clear_last_error()
    with pytest.raises(errors.ResourceLimitError):
        metadata.capability_is_available("x" * 129)

    expected = metadata._FfiErrorSnapshot(
        status=6,
        category=metadata._ClientErrorCategory.VALIDATION,
        cause=metadata._ClientCauseCategory.OTHER,
        delivery=metadata._RequestDeliveryState.NONE,
    )
    assert metadata._last_error_snapshot() == expected
    assert metadata._last_error_snapshot() == expected

    metadata._clear_last_error()
    assert metadata._last_error_snapshot() == metadata._FfiErrorSnapshot(
        status=0,
        category=metadata._ClientErrorCategory.NONE,
        cause=metadata._ClientCauseCategory.NONE,
        delivery=metadata._RequestDeliveryState.NONE,
    )


def test_error_snapshot_helpers_are_not_part_of_python_facade_exports() -> None:
    assert "last_error_snapshot" not in metadata.__all__
    assert "clear_last_error" not in metadata.__all__
    assert "FfiErrorSnapshot" not in metadata.__all__


def test_internal_error_snapshot_helpers_use_private_names() -> None:
    assert not hasattr(metadata, "last_error_snapshot")
    assert not hasattr(metadata, "clear_last_error")
    assert not hasattr(metadata, "FfiErrorSnapshot")
    assert not hasattr(metadata, "ClientCauseCategory")
    assert not hasattr(metadata, "RequestDeliveryState")
