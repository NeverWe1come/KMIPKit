from __future__ import annotations

import pytest

from kmipkit import errors, extensions, ttlv


def test_query_with_limits_accepts_exact_ttlv_payload_limit_for_empty_groups(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    limits = ttlv.codec_limits_create(32, 4, 3)
    native_calls: list[str] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, _arguments: native_calls.append(symbol) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    try:
        assert extensions.client_batch_item_query_with_limits([1], [], limits) == 23
        assert native_calls == ["kmipkit_client_batch_item_query_with_limits"]
    finally:
        limits.close()


def test_query_text_string_length_matches_the_ttlv_item_length_field() -> None:
    assert extensions._validate_query_ttlv_text_length(0xFFFF_FFFF) == 0xFFFF_FFFF
    with pytest.raises(errors.ResourceLimitError):
        extensions._validate_query_ttlv_text_length(0x1_0000_0000)


def test_query_parent_structure_length_matches_the_ttlv_item_length_field() -> None:
    assert extensions._validate_query_ttlv_structure_length(0xFFFF_FFFF) == 0xFFFF_FFFF
    with pytest.raises(errors.ResourceLimitError):
        extensions._validate_query_ttlv_structure_length(0x1_0000_0000)


def test_query_with_limits_rechecks_bounded_snapshot_before_native_staging(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class MisreportingGroups(list[str]):
        def __len__(self) -> int:
            return 0

        def __iter__(self):
            return iter(())

    groups = MisreportingGroups(["oversized-group-value"])
    limits = ttlv.codec_limits_create(40, 4, 10)
    native_calls: list[str] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, _arguments: native_calls.append(symbol) or 23,
    )
    monkeypatch.setattr(
        extensions._client_ffi,
        "new",
        lambda *_args: pytest.fail("oversized Query data must be rejected before CFFI staging"),
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_query_with_limits([1], groups, limits)
        assert native_calls == []
    finally:
        limits.close()
