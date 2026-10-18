from __future__ import annotations

from typing import Any

from kmipkit import extensions


def test_interop_builder_preserves_open_function_and_sensitive_identifier(
    monkeypatch: Any,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 37,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    identifier = b"interop-private-sentinel"
    assert extensions.client_batch_item_interop(0xF1234567, identifier) == 37
    assert calls == [
        (
            "kmipkit_client_batch_item_interop",
            (0xF1234567, identifier),
        )
    ]
