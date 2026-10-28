from __future__ import annotations

from kmipkit import extensions, ttlv


def test_modify_attribute_builder_preserves_optional_current_and_required_new(
    monkeypatch,
):
    calls: list[tuple[str, tuple[object, ...], tuple[int, ...]]] = []
    identifier = object()
    current_handle = object()
    new_handle = object()
    current_attribute = object.__new__(ttlv.TtlvItem)
    new_attribute = object.__new__(ttlv.TtlvItem)

    monkeypatch.setattr(
        extensions,
        "_client_native_unique_identifier",
        lambda value, _staging: identifier if value is not None else None,
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_item",
        lambda value: current_handle if value is current_attribute else new_handle,
        raising=False,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments, *, consumed_indices=():
            calls.append((symbol, arguments, consumed_indices)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    omitted = extensions.client_batch_item_modify_attribute(
        None, None, new_attribute
    )
    selected = extensions.client_batch_item_modify_attribute(
        extensions.UniqueIdentifier.text_string("object-7"),
        current_attribute,
        new_attribute,
    )

    assert omitted == selected == 73
    assert calls == [
        (
            "kmipkit_client_batch_item_modify_attribute",
            (None, None, new_handle),
            (),
        ),
        (
            "kmipkit_client_batch_item_modify_attribute",
            (identifier, current_handle, new_handle),
            (),
        ),
    ]
