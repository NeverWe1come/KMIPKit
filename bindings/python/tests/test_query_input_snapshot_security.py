from __future__ import annotations

import pytest

from kmipkit import errors, extensions, ttlv


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
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_query_with_limits([1], groups, limits)
        assert native_calls == []
    finally:
        limits.close()
