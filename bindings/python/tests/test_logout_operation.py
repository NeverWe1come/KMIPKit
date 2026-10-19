from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions, ttlv
from kmipkit.errors import InvalidInputError


def test_logout_builder_maps_required_ticket_structure(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_native_structure", lambda value, **_kwargs: value._pointer())
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as ticket:
        item = extensions.client_batch_item_logout(ticket)

    assert item == 23
    assert len(calls) == 1
    assert calls[0][0] == "kmipkit_client_batch_item_logout"
    assert len(calls[0][1]) == 1


def test_logout_builder_requires_ticket_structure() -> None:
    with pytest.raises(InvalidInputError):
        extensions.client_batch_item_logout(None)


def test_logout_response_preserves_optional_presence(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_logout=lambda _item, present: present.__setitem__(0, 1)
            or 0
        ),
    )

    assert client._logout_response(item) is True

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_logout=lambda _item, present: present.__setitem__(0, 0)
            or 0
        ),
    )
    assert client._logout_response(item) is None


def test_logout_response_rejects_invalid_presence(monkeypatch) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_logout=lambda _item, present: present.__setitem__(0, 2)
            or 0
        ),
    )

    with pytest.raises(InvalidInputError):
        client._logout_response(SimpleNamespace(_pointer=lambda: object()))
