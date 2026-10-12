from __future__ import annotations

import pytest

from kmipkit import errors, metadata


def test_version_reports_package_and_abi_components() -> None:
    assert metadata.get_version() == metadata.Version(1, 0, 0, 1)


def test_capability_query_only_reports_manifest_ready_c_capabilities() -> None:
    assert metadata.capability_is_available("generic_ttlv_model")
    assert not metadata.capability_is_available("unknown_capability")


def test_error_snapshot_is_safe_non_destructive_and_explicitly_clearable() -> None:
    metadata.clear_last_error()
    with pytest.raises(errors.ResourceLimitError):
        metadata.capability_is_available("x" * 129)

    expected = metadata.FfiErrorSnapshot(
        status=6,
        category=metadata.ClientErrorCategory.VALIDATION,
        cause=metadata.ClientCauseCategory.OTHER,
        delivery=metadata.RequestDeliveryState.NONE,
    )
    assert metadata.last_error_snapshot() == expected
    assert metadata.last_error_snapshot() == expected

    metadata.clear_last_error()
    assert metadata.last_error_snapshot() == metadata.FfiErrorSnapshot(
        status=0,
        category=metadata.ClientErrorCategory.NONE,
        cause=metadata.ClientCauseCategory.NONE,
        delivery=metadata.RequestDeliveryState.NONE,
    )
