from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi


def _split_reference(identifier: str, threshold: int):
    reference_type = getattr(extensions, "SplitKeyReference", None)
    assert reference_type is not None, "the Python facade must expose paired split-key references"
    return reference_type(extensions.UniqueIdentifier.text_string(identifier), threshold)


def test_join_split_key_builder_preserves_repeated_reference_order_and_per_entry_thresholds(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    released: list[tuple[str, object]] = []
    next_handle = 100

    def make_handle(_kind: str, symbol: str, arguments: tuple[object, ...]) -> int:
        nonlocal next_handle
        next_handle += 1
        calls.append((symbol, arguments))
        return next_handle

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_invoke", lambda symbol, handle: released.append((symbol, handle)))
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    attributes = ttlv.ttlv_structure_create()
    masks = ttlv.ttlv_structure_create()
    references = (_split_reference("same-split-id", 1), _split_reference("same-split-id", 2))
    try:
        item = extensions.client_batch_item_join_split_key(
            0xF1234567,
            references,
            secret_data_type=0xFEDCBA98,
            attributes=attributes,
            protection_storage_masks=masks,
        )
        assert item == 103
        assert [call[0] for call in calls] == [
            "kmipkit_split_key_reference_create",
            "kmipkit_split_key_reference_create",
            "kmipkit_client_batch_item_join_split_key",
        ]
        assert [call[1][1] for call in calls[:2]] == [1, 2]
        native_references = calls[2][1][1]
        assert [int(native_references[index]) for index in range(2)] == [101, 102]  # type: ignore[index]
        assert calls[2][1][2:5] == (2, 1, 0xFEDCBA98)
    finally:
        masks.close()
        attributes.close()
    assert released == [
        ("kmipkit_split_key_reference_release", 101),
        ("kmipkit_split_key_reference_release", 102),
    ]


def test_join_split_key_builder_rejects_missing_and_insufficient_threshold_context_before_native_call(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[str] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, _arguments: calls.append(symbol) or 22,
    )
    reference = _split_reference("one-split-id", 2)

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_join_split_key(1, [reference, None])  # type: ignore[list-item]
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_join_split_key(1, [reference])
    assert calls == []


def test_join_split_key_response_owns_ttlv_and_redacts_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure = ttlv.ttlv_structure_create()
    raw_handle, owner = structure._take()
    assert owner is None
    pointer = ffi.cast(
        "kmipkit_ttlv_structure_t *", int(ttlv_ffi.cast("uintptr_t", raw_handle))
    )
    identifier_bytes = b"private-split-key-id"
    storage = ffi.new("uint8_t[]", identifier_bytes)

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = storage  # type: ignore[attr-defined]
        identifier.text_length = len(identifier_bytes)  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_join_split_key=response,
            kmipkit_client_batch_item_response_join_split_key_ttlv=response_ttlv,
        ),
    )

    result = client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.JoinSplitKeyResponse)
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("private-split-key-id")
    assert "private-split-key-id" not in repr(result)
    assert result.response_ttlv._pointer() == pointer
    result.close()
    assert result.response_ttlv.closed


def test_non_success_join_split_key_has_no_typed_response() -> None:
    response_type = getattr(client, "JoinSplitKeyResponse", None)
    assert response_type is not None, "the Python facade must expose typed Join Split Key results"
    assert callable(getattr(client, "_join_split_key_response", None))
