"""Python facade mappings for OASIS KMIP 2.1 §6.1.46, Tables 303–307."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def test_rekey_builder_maps_optional_identifier_offset_attributes_and_masks(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []

    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: (calls.append((symbol, arguments)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_rekey(
        extensions.UniqueIdentifier.text_string("existing-key"), offset=0xFFFF_FFFF
    )

    assert item == 41
    assert len(calls) == 1
    assert calls[0][0] == "kmipkit_client_batch_item_rekey"
    assert calls[0][1][0].kind == int(extensions.UniqueIdentifierKind.TEXT_STRING)
    assert calls[0][1][1:] == (1, 0xFFFF_FFFF, ffi.NULL, ffi.NULL)


def test_rekey_response_maps_identifier_owned_ttlv_and_redacts(monkeypatch) -> None:
    response_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 42)
    identifier_storage = ffi.new("uint8_t[]", b"replacement-key")
    closed: list[bool] = []

    class OwnedStructure:
        def close(self) -> None:
            closed.append(True)

    response_ttlv = OwnedStructure()

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"replacement-key")  # type: ignore[attr-defined]
        return 0

    def response_ttlv_getter(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey=response,
            kmipkit_client_batch_item_response_rekey_ttlv=response_ttlv_getter,
            kmipkit_ttlv_structure_release=lambda _pointer: None,
        ),
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda address: response_ttlv if address == 42 else None,
    )

    result = client._rekey_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.RekeyResponse)
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("replacement-key")
    assert result.response_ttlv is response_ttlv
    assert "replacement-key" not in repr(result)
    result.close()
    assert closed == [True]


def test_rekey_absent_response_skips_ttlv_and_rejects_unexpected_identifier(monkeypatch) -> None:
    calls: list[str] = []

    def response(_item: object, present: object, output: object) -> int:
        calls.append("typed")
        present[0] = 0  # type: ignore[index]
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, _present: object, _output: object) -> int:
        calls.append("ttlv")
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey=response,
            kmipkit_client_batch_item_response_rekey_ttlv=response_ttlv,
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._rekey_response(SimpleNamespace(_pointer=lambda: object()))
    assert calls == ["typed"]
