from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi


def test_get_constraints_builder_has_empty_request_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 29,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_get_constraints() == 29
    assert calls == [("kmipkit_client_batch_item_get_constraints", ())]


def test_get_constraints_response_owns_complete_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_copy = ttlv.ttlv_structure_create()
    native_handle, owner = native_copy._take()
    assert owner is None

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = client.ffi.cast(
            "kmipkit_ttlv_structure_t *", int(ttlv_ffi.cast("uintptr_t", native_handle))
        )  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_get_constraints_ttlv=response_ttlv),
    )
    result = client._get_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.GetConstraintsResponse)
    assert isinstance(result.response_ttlv, ttlv.Structure)
    assert "constraints-secret-sentinel" not in repr(result)
    result.close()

