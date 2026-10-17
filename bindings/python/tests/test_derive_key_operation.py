from __future__ import annotations

import pytest

from kmipkit import extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit.errors import InvalidInputError


def test_derive_key_builder_preserves_ordered_identifier_wire_types(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...], list[tuple[int, bytes, int, int]]]] = []

    def make_handle(_kind, symbol, arguments):
        identifiers_arg = arguments[1]
        identifiers = []
        for index in range(arguments[2]):
            identifier = identifiers_arg[index]
            text = (
                bytes(ffi.buffer(identifier.text_data, identifier.text_length))
                if identifier.kind == 1
                else b""
            )
            identifiers.append(
                (identifier.kind, text, identifier.enumeration_value, identifier.integer_value)
            )
        calls.append((symbol, arguments, identifiers))
        return 23

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    identifiers = (
        extensions.UniqueIdentifier.text_string("source-a"),
        extensions.UniqueIdentifier.enumeration(0xFEED_BEEF),
        extensions.UniqueIdentifier.text_string("source-a"),
    )
    with ttlv.ttlv_structure_create() as parameters, ttlv.ttlv_structure_create() as attributes:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is parameters else 29,
        )
        item = extensions.client_batch_item_derive_key(
            2, identifiers, 0xFEED_BEEF, parameters, attributes
        )

    assert item == 23
    assert len(calls) == 1
    symbol, arguments, copied_identifiers = calls[0]
    assert symbol == "kmipkit_client_batch_item_derive_key"
    assert (arguments[0], arguments[2], arguments[3], arguments[4], arguments[5]) == (
        2,
        3,
        0xFEED_BEEF,
        17,
        29,
    )
    assert copied_identifiers == [
        (1, b"source-a", 0, 0),
        (2, b"", 0xFEED_BEEF, 0),
        (1, b"source-a", 0, 0),
    ]


def test_derive_key_builder_rejects_missing_source_identifiers(monkeypatch) -> None:
    monkeypatch.setattr(extensions, "_make_handle", lambda *_args: 23)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as parameters, ttlv.ttlv_structure_create() as attributes:
        with pytest.raises(InvalidInputError):
            extensions.client_batch_item_derive_key(
                2, (), 1, parameters, attributes
            )
