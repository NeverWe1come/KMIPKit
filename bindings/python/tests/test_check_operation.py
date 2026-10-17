from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions
from kmipkit._client_ffi import ffi
from kmipkit.errors import InvalidInputError


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
