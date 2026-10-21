"""Derived Python binding checks for KMIP 2.1 §6.1.48, Tables 313–315."""

from __future__ import annotations

from types import SimpleNamespace

import pytest
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def _response_item() -> SimpleNamespace:
    return SimpleNamespace(_pointer=lambda: object())


def _typed_response(present: int, identifier_present: int, value: bytes | None = None):
    buffers: list[object] = []

    def typed(
        _item: object,
        out_present: object,
        out_identifier_present: object,
        out_identifier: object,
    ) -> int:
        out_present[0] = present
        out_identifier_present[0] = identifier_present
        if value is not None:
            buffer = ffi.new("uint8_t[]", value)
            buffers.append(buffer)
            out_identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
            out_identifier.text_data = buffer
            out_identifier.text_length = len(value)
        return 0

    return typed


def test_reprovision_builder_forwards_each_optional_object(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    with ttlv.ttlv_structure_create() as certificate_request:
        item = extensions.client_batch_item_reprovision(
            certificate_request=certificate_request
        )
        expected = int(ttlv.ffi.cast("uintptr_t", certificate_request._pointer()))
    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_reprovision"
    assert int(ttlv.ffi.cast("uintptr_t", calls[0][1][0])) == expected
    assert calls[0][1][1] == ffi.NULL


def test_reprovision_builder_rejects_both_objects_before_native_call(monkeypatch) -> None:
    calls: list[str] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, _args: (calls.append(symbol), 41)[1],
    )
    with ttlv.ttlv_structure_create() as certificate_request:
        with ttlv.ttlv_structure_create() as certificate:
            with pytest.raises(errors.InvalidInputError):
                extensions.client_batch_item_reprovision(
                    certificate_request=certificate_request,
                    certificate=certificate,
                )
    assert calls == []


def test_reprovision_builder_allows_no_optional_object(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_reprovision()

    assert item == 41
    assert calls[0][1] == (ffi.NULL, ffi.NULL)


def test_reprovision_builder_forwards_certificate(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    with ttlv.ttlv_structure_create() as certificate:
        item = extensions.client_batch_item_reprovision(certificate=certificate)
        expected = int(ttlv.ffi.cast("uintptr_t", certificate._pointer()))

    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_reprovision"
    assert calls[0][1][0] == ffi.NULL
    assert int(ttlv.ffi.cast("uintptr_t", calls[0][1][1])) == expected


def test_reprovision_response_preserves_optional_identifier_and_ttlv(monkeypatch) -> None:
    identifier = b"reprovisioned-certificate"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 42)

    class OwnedStructure:
        def __init__(self) -> None:
            self.closed = False

        def close(self) -> None:
            self.closed = True

    owned = OwnedStructure()

    def typed(
        _item: object,
        present: object,
        identifier_present: object,
        out_identifier: object,
    ) -> int:
        present[0] = 1
        identifier_present[0] = 1
        out_identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        out_identifier.text_data = identifier_buffer
        out_identifier.text_length = len(identifier)
        return 0

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=typed,
            kmipkit_client_batch_item_response_reprovision_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned if address == 42 else None,
    )
    response = client._reprovision_response(_response_item())
    assert response is not None
    assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
        "reprovisioned-certificate"
    )
    assert response.response_ttlv is owned
    assert "reprovisioned-certificate" not in repr(response)
    response.close()
    assert owned.closed


def test_reprovision_success_without_identifier_is_not_treated_as_absent_response(
    monkeypatch,
) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 43)

    class OwnedStructure:
        def close(self) -> None:
            pass

    owned = OwnedStructure()

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(1, 0),
            kmipkit_client_batch_item_response_reprovision_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned if address == 43 else None,
    )
    response = client._reprovision_response(_response_item())
    assert response is not None
    assert response.unique_identifier is None
    assert response.response_ttlv is owned
    response.close()


def test_reprovision_releases_ttlv_when_native_wrapper_creation_fails(monkeypatch) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 45)
    released: list[object] = []

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    def reject_wrapper(_address: int) -> object:
        raise RuntimeError("invalid native handle")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(1, 0),
            kmipkit_client_batch_item_response_reprovision_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(ttlv, "_structure_from_native_address", reject_wrapper)

    with pytest.raises(RuntimeError, match="invalid native handle"):
        client._reprovision_response(_response_item())

    assert released == [native_ttlv]


def test_reprovision_rejects_present_ttlv_without_native_handle(monkeypatch) -> None:
    def omit_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = ffi.NULL
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(1, 0),
            kmipkit_client_batch_item_response_reprovision_ttlv=omit_ttlv,
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._reprovision_response(_response_item())


def test_reprovision_response_rejects_identifier_presence_mismatches(monkeypatch) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(1, 2),
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._reprovision_response(_response_item())
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(1, 1),
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._reprovision_response(_response_item())


def test_reprovision_absent_response_rejects_present_identifier(monkeypatch) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(0, 1),
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._reprovision_response(_response_item())


def test_reprovision_absent_response_returns_none_without_ttlv(monkeypatch) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(0, 0),
        ),
    )

    assert client._reprovision_response(_response_item()) is None


def test_reprovision_absent_identifier_rejects_unflagged_output(monkeypatch) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_reprovision=_typed_response(
                1, 0, b"unflagged-identifier"
            ),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._reprovision_response(_response_item())
