from __future__ import annotations

from types import SimpleNamespace
from typing import Any

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._ffi import ffi


def test_validate_builder_accepts_identifier_only_chain_and_optional_date(
    monkeypatch: Any,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []

    def make_handle(_kind: str, symbol: str, arguments: tuple[object, ...]) -> int:
        calls.append((symbol, arguments))
        return 61

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    identifiers = [extensions.UniqueIdentifier.text_string("chain-id")]
    assert extensions.client_batch_item_validate((), identifiers, validity_date=123) == 61
    assert len(calls) == 1
    assert calls[0][0] == "kmipkit_client_batch_item_validate"
    assert identifiers == [extensions.UniqueIdentifier.text_string("chain-id")]


def test_validate_builder_rejects_missing_lists_and_out_of_range_date() -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_validate(None, (), validity_date=None)  # type: ignore[arg-type]

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_validate((), (), validity_date=1 << 63)


def test_validate_response_preserves_open_indicator_and_redacts_generic_ttlv(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    limits.close()
    native_handle, owner = structure._take()
    assert owner is None
    address = int(ffi.cast("uintptr_t", native_handle))

    def indicator(_item: object, present: object, raw: object) -> int:
        present[0] = 1  # type: ignore[index]
        raw[0] = 0xF1234567  # type: ignore[index]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = client_ffi.cast("kmipkit_ttlv_structure_t *", address)  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_validate=indicator,
            kmipkit_client_batch_item_response_validate_ttlv=response_ttlv,
        ),
    )
    response = client._validate_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(response, client.ValidateResponse)
    assert response.validity_indicator_raw == 0xF1234567
    with ttlv.ttlv_structure_view(response.response_ttlv) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 0
    assert "response_ttlv" not in repr(response)
    response.close()
    assert response.response_ttlv.closed
