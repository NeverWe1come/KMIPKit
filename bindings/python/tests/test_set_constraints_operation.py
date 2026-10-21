from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def test_set_constraints_builder_passes_the_caller_structure_to_native_abi(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure_handle = object()
    constraints = SimpleNamespace(_pointer=lambda: structure_handle)
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 41,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_set_constraints(constraints) == 41
    assert calls == [
        ("kmipkit_client_batch_item_set_constraints", (structure_handle,))
    ]


def test_set_constraints_response_owns_generic_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_set_constraints_ttlv=response_ttlv),
    )
    monkeypatch.setattr(ttlv, "_structure_from_native_address", lambda _address: structure)

    result = client._set_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.SetConstraintsResponse)
    assert result.response_ttlv is structure
    assert "123" not in repr(result)
    result.close()
    assert structure.closed


def test_set_constraints_response_releases_unexpected_native_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = native_handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_set_constraints_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._set_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [native_handle]
