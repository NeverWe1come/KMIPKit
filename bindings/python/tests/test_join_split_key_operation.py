from __future__ import annotations

# KMIP Specification v2.1 §6.1.27, Tables 244–246.

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi, lib as ttlv_lib
from kmipkit import _handles


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

    def make_handle(_kind: str, symbol: str, arguments: tuple[object, ...]) -> object:
        nonlocal next_handle
        next_handle += 1
        calls.append((symbol, arguments))
        if symbol == "kmipkit_split_key_reference_create":
            return extensions.ffi.cast("kmipkit_split_key_reference_t *", next_handle)
        return next_handle

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(
        _handles,
        "lib",
        SimpleNamespace(
            kmipkit_split_key_reference_release=lambda handle: released.append(
                ("kmipkit_split_key_reference_release", handle)
            ),
            kmipkit_ttlv_structure_release=ttlv_lib.kmipkit_ttlv_structure_release,
        ),
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    attributes = ttlv.ttlv_structure_create()
    masks = ttlv.ttlv_structure_create()
    references = (_split_reference("same-split-id", 1), _split_reference("same-split-id", 2))
    try:
        assert [reference.unique_identifier.text for reference in references] == [
            "same-split-id",
            "same-split-id",
        ]
        assert [reference.split_key_threshold for reference in references] == [1, 2]
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
        assert [
            int(extensions.ffi.cast("uintptr_t", native_references[index]))
            for index in range(2)
        ] == [101, 102]
        assert calls[2][1][2:5] == (2, 1, 0xFEDCBA98)
    finally:
        for reference in references:
            reference.close()
        masks.close()
        attributes.close()
    assert [
        (symbol, int(ffi.cast("uintptr_t", handle))) for symbol, handle in released
    ] == [
        ("kmipkit_split_key_reference_release", 101),
        ("kmipkit_split_key_reference_release", 102),
    ]


def test_join_split_key_builder_rejects_missing_and_insufficient_threshold_context_before_native_call(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[str] = []
    def make_handle(_kind: str, symbol: str, _arguments: tuple[object, ...]) -> object:
        calls.append(symbol)
        if symbol == "kmipkit_split_key_reference_create":
            return extensions.ffi.cast("kmipkit_split_key_reference_t *", 22)
        return 22

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    reference = _split_reference("one-split-id", 2)
    calls.clear()

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_join_split_key(1, [reference, None])  # type: ignore[list-item]
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_join_split_key(1, [reference])
    assert calls == []


def test_split_key_reference_rejects_non_identifier_and_redacts_identifier() -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.SplitKeyReference(object(), 1)  # type: ignore[arg-type]

    reference = _split_reference("private-split-key-sentinel", 2)
    try:
        representation = repr(reference)
        assert "private-split-key-sentinel" not in representation
        assert "split_key_threshold=2" in representation
    finally:
        reference.close()


@pytest.mark.parametrize(
    ("keyword", "value"),
    [
        ("attributes", object()),
        ("protection_storage_masks", object()),
    ],
)
def test_join_split_key_rejects_non_structure_context_before_native_call(
    monkeypatch: pytest.MonkeyPatch,
    keyword: str,
    value: object,
) -> None:
    reference = _split_reference("context-validation-id", 1)

    def unexpected_native_call(*_arguments: object, **_keywords: object) -> object:
        raise AssertionError("invalid context must be rejected before the native call")

    monkeypatch.setattr(extensions, "_make_handle", unexpected_native_call)
    try:
        with pytest.raises(errors.InvalidInputError):
            extensions.client_batch_item_join_split_key(
                1,
                [reference],
                **{keyword: value},
            )
    finally:
        reference.close()


def test_join_split_key_rejects_aggregate_repeated_identifier_bytes_at_native_boundary() -> None:
    reference = _split_reference("x" * ((16 * 1024 * 1024) // 2 + 1), 1)
    try:
        with pytest.raises(errors.ResourceLimitError):
            extensions.client_batch_item_join_split_key(1, [reference, reference])
    finally:
        reference.close()


def test_join_split_key_native_cffi_builder_creates_and_releases_the_typed_item() -> None:
    first = _split_reference("native-repeat-id", 1)
    second = _split_reference("native-repeat-id", 2)
    attributes = ttlv.ttlv_structure_create()
    masks = ttlv.ttlv_structure_create()
    item = None
    try:
        item = extensions.client_batch_item_join_split_key(
            7,
            (first, second),
            secret_data_type=3,
            attributes=attributes,
            protection_storage_masks=masks,
        )
        assert not item.closed
    finally:
        if item is not None:
            item.close()
        masks.close()
        attributes.close()
        second.close()
        first.close()


def test_join_split_key_response_owns_ttlv_and_redacts_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure = _join_split_key_response_ttlv()
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
    view = ttlv.ttlv_structure_view(result.response_ttlv)
    try:
        assert ttlv.ttlv_structure_view_item_count(view) == 4
        assert [_response_tag_at(view, index) for index in range(4)] == [
            0x0042_005C,
            0x0042_0093,
            0x0042_007F,
            0x0042_007C,
        ]
        payload_item = ttlv.ttlv_structure_view_item_at(view, 3)
        payload_value = ttlv.ttlv_item_view_value(payload_item)
        payload_view = ttlv.ttlv_value_view_structure(payload_value)
        try:
            assert ttlv.ttlv_structure_view_item_count(payload_view) == 2
            assert [_response_tag_at(payload_view, index) for index in range(2)] == [
                0x0042_0094,
                0x0054_0001,
            ]
        finally:
            payload_view.close()
            payload_value.close()
            payload_item.close()
    finally:
        view.close()
    result.close()
    assert result.response_ttlv.closed


@pytest.mark.parametrize("presence_value", [2, 255])
def test_join_split_key_response_rejects_non_boolean_presence_output(
    monkeypatch: pytest.MonkeyPatch,
    presence_value: int,
) -> None:
    response_calls, ttlv_calls, _released = _install_join_split_key_response_abi(
        monkeypatch, response_present=presence_value
    )

    with pytest.raises(errors.InvalidInputError):
        client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

    assert response_calls == ["response"]
    assert ttlv_calls == []


def test_join_split_key_response_rejects_identifier_when_response_is_absent(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_calls, ttlv_calls, _released = _install_join_split_key_response_abi(
        monkeypatch, response_present=0, identifier_text="unexpected-identifier"
    )

    with pytest.raises(errors.InvalidInputError):
        client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

    assert response_calls == ["response"]
    assert ttlv_calls == []


def test_join_split_key_response_rejects_inconsistent_empty_identifier_when_absent(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_calls, ttlv_calls, _released = _install_join_split_key_response_abi(
        monkeypatch,
        response_present=0,
        malformed_absent_identifier=True,
    )

    with pytest.raises(errors.InvalidInputError):
        client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

    assert response_calls == ["response"]
    assert ttlv_calls == []


def test_join_split_key_response_rejects_success_without_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_calls, ttlv_calls, _released = _install_join_split_key_response_abi(
        monkeypatch, response_present=1
    )

    with pytest.raises(errors.InvalidInputError):
        client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

    assert response_calls == ["response"]
    assert ttlv_calls == []


def test_join_split_key_response_returns_none_for_consistent_absent_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_calls, ttlv_calls, released = _install_join_split_key_response_abi(
        monkeypatch, response_present=0
    )

    result = client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

    assert result is None
    assert response_calls == ["response"]
    assert ttlv_calls == []
    assert released == []


@pytest.mark.parametrize(
    ("response_ttlv_present", "include_handle", "expected_release"),
    [(0, True, True), (1, False, False), (2, True, True)],
)
def test_join_split_key_response_rejects_mismatched_ttlv_presence_and_handle(
    monkeypatch: pytest.MonkeyPatch,
    response_ttlv_present: int,
    include_handle: bool,
    expected_release: bool,
) -> None:
    handle = _take_join_split_key_response_ttlv_handle() if include_handle else ffi.NULL
    address = int(ffi.cast("uintptr_t", handle)) if include_handle else None
    response_calls, ttlv_calls, released = _install_join_split_key_response_abi(
        monkeypatch,
        response_present=1,
        identifier_text="split-key-id",
        response_ttlv_present=response_ttlv_present,
        response_ttlv_handle=handle,
    )

    try:
        with pytest.raises(errors.InvalidInputError):
            client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

        assert response_calls == ["response"]
        assert ttlv_calls == ["response_ttlv"]
        assert released == ([address] if expected_release else [])
    finally:
        if include_handle and address not in released:
            ttlv_lib.kmipkit_ttlv_structure_release(
                ttlv_ffi.cast("kmipkit_ttlv_structure_t *", address)
            )


def test_join_split_key_response_releases_ttlv_when_native_wrapper_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = _take_join_split_key_response_ttlv_handle()
    address = int(ffi.cast("uintptr_t", handle))
    response_calls, ttlv_calls, released = _install_join_split_key_response_abi(
        monkeypatch,
        response_present=1,
        identifier_text="split-key-id",
        response_ttlv_present=1,
        response_ttlv_handle=handle,
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("test wrapper failure")),
    )

    try:
        with pytest.raises(RuntimeError, match="test wrapper failure"):
            client._join_split_key_response(SimpleNamespace(_pointer=lambda: object()))

        assert response_calls == ["response"]
        assert ttlv_calls == ["response_ttlv"]
        assert released == [address]
    finally:
        if address not in released:
            ttlv_lib.kmipkit_ttlv_structure_release(
                ttlv_ffi.cast("kmipkit_ttlv_structure_t *", address)
            )


def test_join_split_key_result_dispatch_uses_operation_0x29_and_success_only(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    expected = object()
    calls: list[object] = []

    def successful_response(item: object) -> object:
        calls.append(item)
        return expected

    monkeypatch.setattr(client, "_join_split_key_response", successful_response)
    item = object()
    assert client._join_split_key_response_for_result(
        item, outcome=client.BatchOutcome.COMPLETED, result_status=0, operation=0x29
    ) is expected
    assert calls == [item]
    assert client._join_split_key_response_for_result(
        item, outcome=client.BatchOutcome.COMPLETED, result_status=0, operation=0x2C
    ) is None
    assert client._join_split_key_response_for_result(
        item, outcome=client.BatchOutcome.COMPLETED, result_status=1, operation=0x29
    ) is None
    assert client._join_split_key_response_for_result(
        item, outcome=client.BatchOutcome.PENDING, result_status=0, operation=0x29
    ) is None


def test_failure_and_pending_join_split_key_results_retain_common_metadata() -> None:
    failure = client.BatchItemResult(
        client.BatchOutcome.COMPLETED,
        1,
        0xF1234567,
        b"batch-item-id",
        None,
        operation=0x29,
        result_message="join split key failed",
    )
    assert failure.metadata_v2.operation == 0x29
    assert failure.metadata_v2.result_status == 1
    assert failure.metadata_v2.result_reason == 0xF1234567
    assert failure.metadata_v2.result_message == "join split key failed"
    assert failure.join_split_key_response is None

    pending = client.BatchItemResult(
        client.BatchOutcome.PENDING,
        0,
        None,
        b"batch-item-id",
        b"pending-correlation",
        operation=0x29,
    )
    assert pending.outcome == client.BatchOutcome.PENDING
    assert pending.result_status == 0
    assert pending.pending_correlation_value == b"pending-correlation"
    assert pending.join_split_key_response is None


def _join_split_key_response_ttlv() -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()

    def item(tag_value: int, value: ttlv.Value) -> ttlv.TtlvItem:
        raw_tag = ttlv.raw_tag_from_raw(tag_value)
        tag = ttlv.raw_tag_try_checked(raw_tag)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            raw_tag.close()

    try:
        payload = ttlv.ttlv_structure_create()
        for child in (
            item(0x0042_0094, ttlv.ttlv_value_text_string(b"private-split-key-id", limits)),
            item(0x0054_0001, ttlv.ttlv_value_byte_string(b"unknown-vendor-value", limits)),
        ):
            payload = ttlv.ttlv_structure_with_item(payload, child, limits)
        payload_item = item(0x0042_007C, ttlv.ttlv_value_structure(payload, limits))
        response = ttlv.ttlv_structure_create()
        for child in (
            item(0x0042_005C, ttlv.ttlv_value_enumeration(0x29)),
            item(0x0042_0093, ttlv.ttlv_value_byte_string(b"batch-item-id", limits)),
            item(0x0042_007F, ttlv.ttlv_value_enumeration(0)),
            payload_item,
        ):
            response = ttlv.ttlv_structure_with_item(response, child, limits)
        return response
    finally:
        limits.close()


def _response_tag_at(view: ttlv.TtlvStructureView, index: int) -> int:
    item_view = ttlv.ttlv_structure_view_item_at(view, index)
    tag = ttlv.ttlv_item_view_tag(item_view)
    try:
        return ttlv.tag_value(tag)
    finally:
        tag.close()
        item_view.close()


def _install_join_split_key_response_abi(
    monkeypatch: pytest.MonkeyPatch,
    *,
    response_present: int,
    identifier_text: str | None = None,
    malformed_absent_identifier: bool = False,
    response_ttlv_present: int = 1,
    response_ttlv_handle: object = ffi.NULL,
) -> tuple[list[str], list[str], list[int]]:
    response_calls: list[str] = []
    ttlv_calls: list[str] = []
    released: list[int] = []
    identifier_storage = ffi.new("uint8_t[]", b"identifier")
    if identifier_text is not None:
        identifier_storage = ffi.new("uint8_t[]", identifier_text.encode("utf-8"))

    def response(_item: object, present: object, identifier: object) -> int:
        response_calls.append("response")
        present[0] = response_present  # type: ignore[index]
        if identifier_text is not None:
            identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
            identifier.text_data = identifier_storage  # type: ignore[attr-defined]
            identifier.text_length = len(identifier_text.encode("utf-8"))  # type: ignore[attr-defined]
        elif malformed_absent_identifier:
            identifier.text_data = identifier_storage  # type: ignore[attr-defined]
            identifier.text_length = 1  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        ttlv_calls.append("response_ttlv")
        present[0] = response_ttlv_present  # type: ignore[index]
        output[0] = response_ttlv_handle  # type: ignore[index]
        return 0

    def release_structure(handle: object) -> None:
        address = int(ffi.cast("uintptr_t", handle))
        released.append(address)
        ttlv_lib.kmipkit_ttlv_structure_release(
            ttlv_ffi.cast("kmipkit_ttlv_structure_t *", address)
        )

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_join_split_key=response,
            kmipkit_client_batch_item_response_join_split_key_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=release_structure,
        ),
    )
    return response_calls, ttlv_calls, released


def _take_join_split_key_response_ttlv_handle() -> object:
    structure = _join_split_key_response_ttlv()
    raw_handle, owner = structure._take()
    assert owner is None
    address = int(ttlv_ffi.cast("uintptr_t", raw_handle))
    return ffi.cast("kmipkit_ttlv_structure_t *", address)
