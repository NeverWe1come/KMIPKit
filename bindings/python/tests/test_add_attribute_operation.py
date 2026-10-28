from __future__ import annotations

from kmipkit import extensions, ttlv


def test_add_attribute_builder_preserves_identifier_and_transfers_new_attribute(
    monkeypatch,
):
    calls: list[tuple[str, tuple[object, ...], tuple[int, ...]]] = []
    attribute_handle = object()
    omitted_identifier = object()
    present_identifier = object()
    attribute = object.__new__(ttlv.TtlvItem)

    monkeypatch.setattr(
        extensions,
        "_native_unique_identifier",
        lambda value, _staging: (
            omitted_identifier if value is None else present_identifier,
            None,
        ),
    )
    monkeypatch.setattr(
        extensions,
        "_client_native_item",
        lambda _value: attribute_handle,
        raising=False,
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments, *, consumed_indices=():
            calls.append((symbol, arguments, consumed_indices)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    omitted = extensions.client_batch_item_add_attribute(None, attribute)
    present = extensions.client_batch_item_add_attribute(
        extensions.UniqueIdentifier.text_string("object-7"), attribute
    )

    assert omitted == present == 73
    assert calls == [
        (
            "kmipkit_client_batch_item_add_attribute",
            (omitted_identifier, attribute_handle),
            (1,),
        ),
        (
            "kmipkit_client_batch_item_add_attribute",
            (present_identifier, attribute_handle),
            (1,),
        ),
    ]
