from __future__ import annotations

from collections.abc import Callable
from types import SimpleNamespace

import pytest

from kmipkit import client, extensions
from kmipkit._client_ffi import ffi
from kmipkit.errors import InvalidInputError


def _install_check_response(
    monkeypatch: pytest.MonkeyPatch,
    *,
    present: int = 1,
    identifier_present: int = 0,
    identifier_kind: int = 0,
    count_present: int = 0,
    count: int = 0,
    mask_present: int = 0,
    usage_mask: int = 0,
    lease_present: int = 0,
    lease: int = 0,
    ttlv_present: int = 1,
    ttlv_pointer: object = ffi.cast("kmipkit_ttlv_structure_t *", 24),
    convert_ttlv: Callable[[int], object] | None = None,
) -> list[object]:
    released: list[object] = []

    def response_check(_item, out_present, out_identifier_present, identifier,
                       out_count_present, out_count, out_mask_present, out_mask,
                       out_lease_present, out_lease) -> int:
        out_present[0] = present
        out_identifier_present[0] = identifier_present
        identifier.kind = identifier_kind
        out_count_present[0] = count_present
        out_count[0] = count
        out_mask_present[0] = mask_present
        out_mask[0] = usage_mask
        out_lease_present[0] = lease_present
        out_lease[0] = lease
        return 0

    def response_check_ttlv(_item, out_present, output) -> int:
        out_present[0] = ttlv_present
        output[0] = ttlv_pointer
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_check=response_check,
            kmipkit_client_batch_item_response_check_ttlv=response_check_ttlv,
            kmipkit_ttlv_structure_release=lambda pointer: released.append(pointer),
        ),
    )
    if convert_ttlv is not None:
        monkeypatch.setattr(client.ttlv, "_structure_from_native_address", convert_ttlv)
    else:
        monkeypatch.setattr(
            client.ttlv,
            "_structure_from_native_address",
            lambda _address: SimpleNamespace(close=lambda: None),
        )
    return released


def test_check_builder_preserves_every_optional_field(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_check(
        extensions.UniqueIdentifier.integer(-17),
        usage_limits_count=-(1 << 63),
        cryptographic_usage_mask=-(1 << 31),
        lease_time=0,
    )

    assert item == 23
    assert len(calls) == 1
    assert calls[0][0] == "kmipkit_client_batch_item_check"
    assert calls[0][1][0] != ffi.NULL
    assert calls[0][1][1:] == (1, -(1 << 63), 1, -(1 << 31), 1, 0)


def test_check_builder_omits_optional_fields_with_zeroed_native_values(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_check() == 23

    assert calls == [("kmipkit_client_batch_item_check", (ffi.NULL, 0, 0, 0, 0, 0, 0))]


def test_check_builder_accepts_lease_time_u32_boundaries_and_rejects_out_of_range(
    monkeypatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    for lease_time in (0, (1 << 32) - 1):
        assert extensions.client_batch_item_check(lease_time=lease_time) == 23
    assert [call[1][-1] for call in calls] == [0, (1 << 32) - 1]

    for lease_time in (-1, 1 << 32):
        with pytest.raises(InvalidInputError):
            extensions.client_batch_item_check(lease_time=lease_time)


def test_failed_check_maps_policy_fields_and_owned_generic_ttlv(monkeypatch) -> None:
    response_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    response_ttlv = OwnedStructure()

    def response_check(_item, present, identifier_present, identifier,
                       count_present, count, mask_present, mask, lease_present, lease) -> int:
        present[0] = 1
        identifier_present[0] = 0
        identifier.kind = 0
        count_present[0] = 1
        count[0] = -9
        mask_present[0] = 1
        mask[0] = -1
        lease_present[0] = 1
        lease[0] = 0
        return 0

    def response_check_ttlv(_item, present, output) -> int:
        present[0] = 1
        output[0] = response_pointer
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_check=response_check,
            kmipkit_client_batch_item_response_check_ttlv=response_check_ttlv,
            kmipkit_ttlv_structure_release=lambda _pointer: None,
        ),
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda address: response_ttlv if address == 24 else None,
    )
    item = SimpleNamespace(_pointer=lambda: object())

    response = client._check_response(item)

    assert isinstance(response, client.CheckResponse)
    assert response.unique_identifier is None
    assert response.usage_limits_count == -9
    assert response.cryptographic_usage_mask == -1
    assert response.lease_time == 0
    assert response.response_ttlv is response_ttlv
    response.close()
    assert response_ttlv.closed


@pytest.mark.parametrize(
    "native_values",
    [
        {"present": 2},
        {"identifier_present": 2},
        {"count_present": 2},
        {"mask_present": 2},
        {"lease_present": 2},
        {"count": 1},
        {"usage_mask": 1},
        {"lease": 1},
    ],
)
def test_check_response_rejects_noncanonical_presence_flags(
    monkeypatch, native_values: dict[str, int]
) -> None:
    _install_check_response(monkeypatch, **native_values)
    item = SimpleNamespace(_pointer=lambda: object())

    with pytest.raises(InvalidInputError):
        client._check_response(item)


def test_absent_check_response_has_no_typed_fields_or_generic_ttlv(monkeypatch) -> None:
    _install_check_response(
        monkeypatch, present=0, ttlv_present=0, ttlv_pointer=ffi.NULL
    )
    item = SimpleNamespace(_pointer=lambda: object())

    assert client._check_response(item) is None


def test_present_check_response_allows_omitted_policy_fields(monkeypatch) -> None:
    _install_check_response(monkeypatch)
    item = SimpleNamespace(_pointer=lambda: object())

    response = client._check_response(item)

    assert isinstance(response, client.CheckResponse)
    assert response.unique_identifier is None
    assert response.usage_limits_count is None
    assert response.cryptographic_usage_mask is None
    assert response.lease_time is None


def test_absent_check_response_rejects_policy_fields(monkeypatch) -> None:
    _install_check_response(monkeypatch, present=0, count_present=1, count=3)
    item = SimpleNamespace(_pointer=lambda: object())

    with pytest.raises(InvalidInputError):
        client._check_response(item)


def test_absent_check_response_releases_inconsistent_generic_ttlv(monkeypatch) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)
    released = _install_check_response(
        monkeypatch, present=0, ttlv_present=1, ttlv_pointer=pointer
    )
    item = SimpleNamespace(_pointer=lambda: object())

    with pytest.raises(InvalidInputError):
        client._check_response(item)

    assert released == [pointer]


def test_present_check_response_releases_inconsistent_generic_ttlv(monkeypatch) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)
    released = _install_check_response(
        monkeypatch, present=1, ttlv_present=0, ttlv_pointer=pointer
    )
    item = SimpleNamespace(_pointer=lambda: object())

    with pytest.raises(InvalidInputError):
        client._check_response(item)

    assert released == [pointer]


def test_check_response_releases_ttlv_when_conversion_fails(monkeypatch) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)

    def fail_conversion(_address: int) -> object:
        raise RuntimeError("conversion failed")

    released = _install_check_response(
        monkeypatch,
        present=1,
        ttlv_present=1,
        ttlv_pointer=pointer,
        convert_ttlv=fail_conversion,
    )
    item = SimpleNamespace(_pointer=lambda: object())

    with pytest.raises(RuntimeError, match="conversion failed"):
        client._check_response(item)

    assert released == [pointer]
