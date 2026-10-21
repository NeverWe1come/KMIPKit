"""Python facade mappings for OASIS KMIP 2.1 §6.1.46, Tables 303–307."""

from __future__ import annotations

from types import SimpleNamespace

import pytest
from kmipkit import client, errors, extensions
from kmipkit._client_ffi import ffi


def test_rekey_builder_maps_optional_identifier_offset_attributes_and_masks(
    monkeypatch,
) -> None:
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
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string(
        "replacement-key"
    )
    assert result.response_ttlv is response_ttlv
    assert "replacement-key" not in repr(result)
    result.close()
    assert closed == [True]


def test_rekey_absent_response_skips_ttlv_and_rejects_unexpected_identifier(
    monkeypatch,
) -> None:
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


def _stub_rekey_response(
    monkeypatch,
    *,
    present: int,
    identifier: bytes | None = None,
    response_ttlv_present: int = 0,
    response_ttlv_handle: int = 42,
) -> tuple[list[str], list[int]]:
    calls: list[str] = []
    released: list[int] = []
    response_pointer = ffi.cast("kmipkit_ttlv_structure_t *", response_ttlv_handle)
    identifier_storage = (
        None if identifier is None else ffi.new("uint8_t[]", identifier)
    )

    def response(_item: object, presence: object, output: object) -> int:
        calls.append("typed")
        presence[0] = present  # type: ignore[index]
        if identifier_storage is not None:
            output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
            output.text_data = identifier_storage  # type: ignore[attr-defined]
            output.text_length = len(identifier or b"")  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, presence: object, output: object) -> int:
        calls.append("ttlv")
        presence[0] = response_ttlv_present  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        return 0

    def release(pointer: object) -> None:
        released.append(int(ffi.cast("uintptr_t", pointer)))

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey=response,
            kmipkit_client_batch_item_response_rekey_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=release,
        ),
    )
    return calls, released


def test_rekey_absent_response_with_empty_identifier_returns_none(monkeypatch) -> None:
    calls, released = _stub_rekey_response(monkeypatch, present=0)

    assert client._rekey_response(SimpleNamespace(_pointer=lambda: object())) is None
    assert calls == ["typed"]
    assert released == []


def test_rekey_response_rejects_unknown_presence_and_missing_identifier(
    monkeypatch,
) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    _stub_rekey_response(monkeypatch, present=2)
    with pytest.raises(errors.InvalidInputError):
        client._rekey_response(item)

    _stub_rekey_response(monkeypatch, present=1)
    with pytest.raises(errors.InvalidInputError):
        client._rekey_response(item)


def test_rekey_response_releases_ttlv_when_presence_disagrees(monkeypatch) -> None:
    calls, released = _stub_rekey_response(
        monkeypatch,
        present=1,
        identifier=b"existing-key",
        response_ttlv_present=0,
    )

    with pytest.raises(errors.InvalidInputError):
        client._rekey_response(SimpleNamespace(_pointer=lambda: object()))

    assert calls == ["typed", "ttlv"]
    assert released == [42]


def test_rekey_response_releases_ttlv_when_owned_wrapper_fails(monkeypatch) -> None:
    _, released = _stub_rekey_response(
        monkeypatch,
        present=1,
        identifier=b"existing-key",
        response_ttlv_present=1,
    )

    def fail_wrapper(_address: int) -> object:
        raise RuntimeError("test wrapper failure")

    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", fail_wrapper)
    with pytest.raises(RuntimeError, match="test wrapper failure"):
        client._rekey_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [42]
