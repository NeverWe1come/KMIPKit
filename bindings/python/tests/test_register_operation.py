"""Derived Python binding checks for OASIS KMIP 2.1 §6.1.43, Tables 291–294."""

from __future__ import annotations

from types import SimpleNamespace

import pytest
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def _response_item() -> SimpleNamespace:
    return SimpleNamespace(_pointer=lambda: object())


def test_register_success_response_preserves_identifier_and_owned_ttlv(monkeypatch) -> None:
    identifier = b"registered-object"
    identifier_buffer = ffi.new("uint8_t[]", identifier)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 42)

    class OwnedStructure:
        def __init__(self) -> None:
            self.closed = False

        def close(self) -> None:
            self.closed = True

    owned = OwnedStructure()

    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = identifier_buffer
        output.text_length = len(identifier)
        return 0

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_register=typed,
            kmipkit_client_batch_item_response_register_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned if address == 42 else None,
    )

    response = client._register_response(_response_item())
    assert response is not None
    assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
        "registered-object"
    )
    assert response.response_ttlv is owned
    assert "registered-object" not in repr(response)
    response.close()
    assert owned.closed


def test_register_failure_or_pending_has_no_success_response(monkeypatch) -> None:
    def absent(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output.kind = 0
        output.text_data = ffi.NULL
        output.text_length = 0
        output.enumeration_value = 0
        output.integer_value = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_register=absent),
    )
    assert client._register_response(_response_item()) is None


def test_register_success_requires_a_required_identifier(monkeypatch) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 43)
    released: list[object] = []

    def missing_identifier(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = 0
        output.text_data = ffi.NULL
        output.text_length = 0
        output.enumeration_value = 0
        output.integer_value = 0
        return 0

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_register=missing_identifier,
            kmipkit_client_batch_item_response_register_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._register_response(_response_item())
    assert released == []
