from __future__ import annotations

from types import SimpleNamespace
from typing import Any

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._ffi import ffi, lib as ttlv_lib


def test_interop_builder_preserves_open_function_and_sensitive_identifier(
    monkeypatch: Any,
) -> None:
    calls: list[tuple[str, int, bytes]] = []

    def make_handle(_kind: str, symbol: str, arguments: tuple[object, ...]) -> int:
        function, identifier, length = arguments
        assert isinstance(function, int)
        assert isinstance(length, int)
        calls.append((symbol, function, bytes(ffi.buffer(identifier, length))))
        return 37

    monkeypatch.setattr(
        extensions,
        "_make_handle",
        make_handle,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    identifier = "interop-private-sentinel"
    assert extensions.client_batch_item_interop(0xF1234567, identifier) == 37
    assert calls == [
        ("kmipkit_client_batch_item_interop", 0xF1234567, b"interop-private-sentinel")
    ]


def test_interop_builder_rejects_out_of_range_function_and_invalid_utf8() -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_interop(1 << 32, b"identifier")

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_interop(1, b"\xff")


def test_interop_response_owns_complete_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sentinel = b"interop-response-private-sentinel"
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(0x0054_1234)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_text_string(sentinel, limits)
    item = ttlv.ttlv_item_create(tag, value, limits)
    structure = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), item, limits)
    limits.close()
    tag.close()
    raw_tag.close()
    value.close()
    item.close()
    native_handle, owner = structure._take()
    assert owner is None

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = client_ffi.cast(
            "kmipkit_ttlv_structure_t *", int(ffi.cast("uintptr_t", native_handle))
        )  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_interop_ttlv=response_ttlv,
        ),
    )
    response = client._interop_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(response, client.InteropResponse)
    with ttlv.ttlv_structure_view(response.response_ttlv) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 1
        with ttlv.ttlv_structure_view_item_at(view, 0) as response_item:
            with ttlv.ttlv_item_view_value(response_item) as response_value:
                length = ttlv.ttlv_value_view_byte_length(response_value)
                assert bytes(
                    ttlv.ttlv_value_view_byte_at(response_value, index)
                    for index in range(length)
                ) == sentinel
    assert sentinel.decode() not in repr(response)
    response.close()
    assert response.response_ttlv.closed


def test_interop_response_rejects_absent_indicator_with_owned_ttlv(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_interop_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._interop_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [handle]


def test_interop_response_returns_none_when_payload_is_absent(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = client_ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_interop_ttlv=response_ttlv,
        ),
    )

    assert client._interop_response(SimpleNamespace(_pointer=lambda: object())) is None


def test_interop_response_releases_output_handle_when_abi_status_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle, address = _take_interop_response_ttlv_handle()
    released = _install_interop_response_abi(
        monkeypatch, status=4, present=1, handle=handle
    )

    try:
        with pytest.raises(errors.InvalidInputError):
            client._interop_response(SimpleNamespace(_pointer=lambda: object()))

        assert released == [address]
    finally:
        if address not in released:
            ttlv_lib.kmipkit_ttlv_structure_release(
                ffi.cast("kmipkit_ttlv_structure_t *", address)
            )


def test_interop_response_releases_output_handle_for_malformed_presence(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle, address = _take_interop_response_ttlv_handle()
    released = _install_interop_response_abi(
        monkeypatch, status=0, present=2, handle=handle
    )

    try:
        with pytest.raises(errors.InvalidInputError):
            client._interop_response(SimpleNamespace(_pointer=lambda: object()))

        assert released == [address]
    finally:
        if address not in released:
            ttlv_lib.kmipkit_ttlv_structure_release(
                ffi.cast("kmipkit_ttlv_structure_t *", address)
            )


def test_interop_response_releases_output_handle_when_native_wrapper_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle, address = _take_interop_response_ttlv_handle()
    released = _install_interop_response_abi(
        monkeypatch, status=0, present=1, handle=handle
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("test wrapper failure")),
    )

    try:
        with pytest.raises(RuntimeError, match="test wrapper failure"):
            client._interop_response(SimpleNamespace(_pointer=lambda: object()))

        assert released == [address]
    finally:
        if address not in released:
            ttlv_lib.kmipkit_ttlv_structure_release(
                ffi.cast("kmipkit_ttlv_structure_t *", address)
            )


def _take_interop_response_ttlv_handle() -> tuple[object, int]:
    structure = ttlv.ttlv_structure_create()
    native_handle, owner = structure._take()
    assert owner is None
    address = int(ffi.cast("uintptr_t", native_handle))
    return client_ffi.cast("kmipkit_ttlv_structure_t *", address), address


def _install_interop_response_abi(
    monkeypatch: pytest.MonkeyPatch,
    *,
    status: int,
    present: int,
    handle: object,
) -> list[int]:
    released: list[int] = []

    def response_ttlv(_item: object, out_present: object, output: object) -> int:
        out_present[0] = present  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return status

    def release_structure(value: object) -> None:
        address = int(client_ffi.cast("uintptr_t", value))
        released.append(address)
        ttlv_lib.kmipkit_ttlv_structure_release(
            ffi.cast("kmipkit_ttlv_structure_t *", address)
        )

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_interop_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=release_structure,
        ),
    )
    return released
