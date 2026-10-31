"""High-level Python access to response Message Extensions."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, ttlv
from kmipkit._client_ffi import ffi
from _test_support import batch_result_test_symbols


def test_batch_item_result_returns_caller_owned_generic_extension_copy(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 41)
    structure_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 42)
    expected_structure = object()
    calls: list[int] = []

    class NativeItem:
        def _pointer(self) -> object:
            return item_pointer

    def extension_count(_item: object, output: object) -> int:
        output[0] = 1  # type: ignore[index]
        return 0

    def extension_at(
        _item: object, index: int, present: object, output: object
    ) -> int:
        calls.append(index)
        present[0] = 1  # type: ignore[index]
        output[0] = structure_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_extension_count=extension_count,
            kmipkit_client_batch_item_response_extension_ttlv_at=extension_at,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: expected_structure if address == 42 else None,
    )
    result = client.BatchItemResult(
        outcome=client.BatchOutcome.COMPLETED,
        result_status=0,
        result_reason=None,
        unique_batch_item_id=None,
        pending_correlation_value=None,
    )
    object.__setattr__(result, "_native_response_extensions", NativeItem())

    assert result.extension_count == 1
    assert result.extension_ttlv_at(0) is expected_structure
    assert calls == [0]

    with pytest.raises(errors.InvalidInputError):
        result.extension_ttlv_at(1)
    assert calls == [0]


def test_batch_item_result_without_native_extensions_is_empty() -> None:
    result = client.BatchItemResult(
        outcome=client.BatchOutcome.COMPLETED,
        result_status=0,
        result_reason=None,
        unique_batch_item_id=None,
        pending_correlation_value=None,
    )

    assert result.extension_count == 0
    with pytest.raises(errors.InvalidInputError):
        result.extension_ttlv_at(0)


def test_batch_result_without_extensions_uses_the_native_empty_response_contract(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer = ffi.cast("kmipkit_client_batch_response_t *", 51)
    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 52)

    def metadata(_item: object, output: object) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")  # type: ignore[attr-defined]
        output.abi_version = 2  # type: ignore[attr-defined]
        output.operation = 0x35  # type: ignore[attr-defined]
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        output.result_status = 0  # type: ignore[attr-defined]
        output.has_result_reason = 0  # type: ignore[attr-defined]
        output.result_reason = 0  # type: ignore[attr-defined]
        output.has_result_message = 0  # type: ignore[attr-defined]
        output.reserved = 0  # type: ignore[attr-defined]
        output.result_message_data = ffi.NULL  # type: ignore[attr-defined]
        output.result_message_length = 0  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(**batch_result_test_symbols(ffi, item_pointer, metadata)),
    )

    result = client._batch_result(response_pointer)
    try:
        assert result.items[0].extension_count == 0
        with pytest.raises(errors.InvalidInputError):
            result.items[0].extension_ttlv_at(0)
    finally:
        result.close()
