from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi


def test_get_constraints_builder_has_empty_request_payload(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 29,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_get_constraints() == 29
    assert calls == [("kmipkit_client_batch_item_get_constraints", ())]


def test_get_constraints_response_owns_complete_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sentinel = b"constraints-secret-sentinel"
    limits = ttlv.codec_limits_defaults()

    def item(tag_value: int, value: ttlv.Value) -> ttlv.TtlvItem:
        raw_tag = ttlv.raw_tag_from_raw(tag_value)
        tag = ttlv.raw_tag_try_checked(raw_tag)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            raw_tag.close()

    def response_structure() -> ttlv.Structure:
        secret_item = item(
            0x0054_1234,
            ttlv.ttlv_value_text_string(sentinel, limits),
        )
        constraints = ttlv.ttlv_structure_with_item(
            ttlv.ttlv_structure_create(), secret_item, limits
        )
        constraints_item = item(
            0x0042_0168,
            ttlv.ttlv_value_structure(constraints, limits),
        )
        return ttlv.ttlv_structure_with_item(
            ttlv.ttlv_structure_create(), constraints_item, limits
        )

    def has_sentinel(structure: ttlv.Structure) -> bool:
        with ttlv.ttlv_structure_view(structure) as root:
            with ttlv.ttlv_structure_view_item_at(root, 0) as constraints_item:
                with ttlv.ttlv_item_view_value(constraints_item) as constraints_value:
                    with ttlv.ttlv_value_view_structure(constraints_value) as constraints:
                        with ttlv.ttlv_structure_view_item_at(
                            constraints, 0
                        ) as secret_item:
                            with ttlv.ttlv_item_view_value(secret_item) as secret_value:
                                length = ttlv.ttlv_value_view_byte_length(secret_value)
                                return bytes(
                                    ttlv.ttlv_value_view_byte_at(secret_value, index)
                                    for index in range(length)
                                ) == sentinel

    try:
        native_copy = response_structure()
    finally:
        limits.close()
    assert has_sentinel(native_copy)
    native_handle, owner = native_copy._take()
    assert owner is None

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = client.ffi.cast(
            "kmipkit_ttlv_structure_t *", int(ttlv_ffi.cast("uintptr_t", native_handle))
        )  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_get_constraints_ttlv=response_ttlv),
    )
    result = client._get_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.GetConstraintsResponse)
    assert isinstance(result.response_ttlv, ttlv.Structure)
    assert has_sentinel(result.response_ttlv)
    assert "constraints-secret-sentinel" not in repr(result)
    result.close()
    assert result.response_ttlv.closed


def test_get_constraints_response_releases_unexpected_native_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = client.ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_constraints_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._get_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [handle]


def test_get_constraints_response_releases_handle_when_ttlv_wrapping_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = client.ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_constraints_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("invalid test handle")),
    )

    with pytest.raises(RuntimeError, match="invalid test handle"):
        client._get_constraints_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [handle]
