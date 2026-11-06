from __future__ import annotations

from types import SimpleNamespace
from typing import Any

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._ffi import ffi, lib as ttlv_lib


def test_log_builder_preserves_and_redacts_message(monkeypatch: Any) -> None:
    calls: list[tuple[str, bytes, int]] = []

    def make_handle(_kind: str, symbol: str, arguments: tuple[object, ...]) -> int:
        message, length = arguments
        assert isinstance(length, int)
        calls.append((symbol, bytes(ffi.buffer(message, length)), length))
        return 37

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    message = "log-message-private-sentinel"
    assert extensions.client_batch_item_log(message) == 37
    assert calls == [
        ("kmipkit_client_batch_item_log", b"log-message-private-sentinel", len(message))
    ]


def test_log_response_owns_complete_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sentinel = b"log-response-private-sentinel"
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
            kmipkit_client_batch_item_response_log_ttlv=response_ttlv,
        ),
    )
    response = client._log_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(response, client.LogResponse)
    with ttlv.ttlv_structure_view(response.response_ttlv) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 1
    assert sentinel.decode() not in repr(response)
    response.close()
    assert response.response_ttlv.closed


def test_log_response_releases_owned_handle_when_presence_is_absent(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure = ttlv.ttlv_structure_create()
    native_handle, owner = structure._take()
    assert owner is None
    address = int(ffi.cast("uintptr_t", native_handle))
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", address)
    released: list[int] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    def release_structure(value: object) -> None:
        released_address = int(client_ffi.cast("uintptr_t", value))
        released.append(released_address)
        ttlv_lib.kmipkit_ttlv_structure_release(
            ffi.cast("kmipkit_ttlv_structure_t *", released_address)
        )

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_log_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=release_structure,
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._log_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [address]


def test_log_response_releases_output_when_abi_status_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 4

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_log_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._log_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [handle]


def test_log_response_returns_none_when_native_response_is_absent(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = client_ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_log_ttlv=response_ttlv),
    )

    assert client._log_response(SimpleNamespace(_pointer=lambda: object())) is None


def test_log_response_releases_native_handle_when_ttlv_wrapper_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    def fail_wrapper(_address: int) -> object:
        raise RuntimeError("test Log response wrapper failure")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_log_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", fail_wrapper)

    with pytest.raises(RuntimeError, match="test Log response wrapper failure"):
        client._log_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [handle]
