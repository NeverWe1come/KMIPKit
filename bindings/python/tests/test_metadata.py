from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import errors, metadata


@pytest.mark.parametrize("invalid_code", [-1, 0x1_0000_0000, "4"])
def test_open_enum_rejects_codes_outside_unsigned_32_bit_integers(
    invalid_code: object,
) -> None:
    with pytest.raises(ValueError):
        metadata._ClientErrorCategory(invalid_code)


def test_version_reports_package_and_abi_components() -> None:
    assert metadata.get_version() == metadata.Version(1, 0, 0, 1)


def test_capability_query_only_reports_manifest_ready_c_capabilities() -> None:
    assert metadata.capability_is_available("generic_ttlv_model")
    assert not metadata.capability_is_available("unknown_capability")


def test_capability_query_limits_standard_utf8_encoded_bytes() -> None:
    assert not metadata.capability_is_available("🔐" * 32)
    with pytest.raises(errors.ResourceLimitError):
        metadata.capability_is_available("🔐" * 33)


def test_capability_query_normalizes_unpaired_surrogate_as_invalid_input(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(
        metadata,
        "lib",
        SimpleNamespace(
            kmipkit_capability_is_available=lambda *_args: pytest.fail(
                "invalid capability text must not reach the native library"
            )
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        metadata.capability_is_available("\ud800")


@pytest.mark.parametrize("capability_id", [None, b"generic_ttlv_model", 17])
def test_capability_query_rejects_non_string_identifiers(
    monkeypatch: pytest.MonkeyPatch, capability_id: object
) -> None:
    monkeypatch.setattr(
        metadata,
        "lib",
        SimpleNamespace(
            kmipkit_capability_is_available=lambda *_args: pytest.fail(
                "non-string capability IDs must not reach the native library"
            )
        ),
    )

    with pytest.raises(TypeError, match="capability_id must be str"):
        metadata.capability_is_available(capability_id)  # type: ignore[arg-type]


def test_internal_status_raises_redacted_generic_error() -> None:
    with pytest.raises(errors.KmipKitError) as raised:
        errors.raise_for_status(7)

    assert type(raised.value) is errors.KmipKitError
    assert str(raised.value) == "KMIPKit operation failed (kmipkit_error)"


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


def test_error_snapshot_preserves_unknown_open_enum_codes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    unknown_category = 0xA17C
    unknown_cause = 0xB28D
    unknown_delivery = 0xC39E

    def populate_snapshot(
        status: list[int],
        category: list[int],
        cause: list[int],
        delivery: list[int],
    ) -> int:
        status[0] = -1
        category[0] = unknown_category
        cause[0] = unknown_cause
        delivery[0] = unknown_delivery
        return 0

    monkeypatch.setattr(
        metadata, "lib", SimpleNamespace(kmipkit_last_error_get=populate_snapshot)
    )

    snapshot = metadata._last_error_snapshot()

    assert snapshot.status == -1
    assert snapshot.category.value == unknown_category
    assert snapshot.category.name == f"UNKNOWN_{unknown_category}"
    assert snapshot.cause.value == unknown_cause
    assert snapshot.cause.name == f"UNKNOWN_{unknown_cause}"
    assert snapshot.delivery.value == unknown_delivery
    assert snapshot.delivery.name == f"UNKNOWN_{unknown_delivery}"


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
