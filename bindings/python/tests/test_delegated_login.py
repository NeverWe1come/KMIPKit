from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions
from kmipkit._client_ffi import ffi
from kmipkit.errors import InvalidInputError


def test_delegated_login_builder_maps_table_199_fields(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    rights = SimpleNamespace(_pointer=lambda: 17)
    usage_limits = SimpleNamespace(_pointer=lambda: 29)
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    monkeypatch.setattr(extensions, "_native_structure", lambda value, **_kwargs: value._pointer())

    item = extensions.client_batch_item_delegated_login(
        rights,
        lease_time_interval=0xFFFF_FFFF,
        request_count=-(1 << 31),
        usage_limits=usage_limits,
    )

    assert item == 23
    assert calls == [(
        "kmipkit_client_batch_item_delegated_login",
        (17, 1, 0xFFFF_FFFF, 1, -(1 << 31), 29),
    )]


def test_delegated_login_builder_maps_signed_datetime_and_allows_omissions(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    rights = SimpleNamespace(_pointer=lambda: 17)
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    monkeypatch.setattr(extensions, "_native_structure", lambda value, **_kwargs: value._pointer())

    extensions.client_batch_item_delegated_login(rights, lease_time_date_time=-(1 << 63))
    extensions.client_batch_item_delegated_login(rights)

    assert calls == [
        ("kmipkit_client_batch_item_delegated_login", (17, 2, -(1 << 63), 0, 0, ffi.NULL)),
        ("kmipkit_client_batch_item_delegated_login", (17, 0, 0, 0, 0, ffi.NULL)),
    ]


@pytest.mark.parametrize(
    "values",
    [
        {"lease_time_interval": -1},
        {"lease_time_interval": 1 << 32},
        {"lease_time_date_time": -(1 << 63) - 1},
        {"lease_time_date_time": 1 << 63},
        {"request_count": -(1 << 31) - 1},
        {"request_count": 1 << 31},
        {"lease_time_interval": 1, "lease_time_date_time": 2},
    ],
)
def test_delegated_login_builder_rejects_invalid_table_199_values(monkeypatch, values) -> None:
    monkeypatch.setattr(extensions, "_make_handle", lambda *_args: 23)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    monkeypatch.setattr(extensions, "_native_structure", lambda value, **_kwargs: value._pointer())

    with pytest.raises(InvalidInputError):
        extensions.client_batch_item_delegated_login(
            SimpleNamespace(_pointer=lambda: 17), **values
        )


def test_delegated_login_requires_generic_rights_structure() -> None:
    with pytest.raises(InvalidInputError):
        extensions.client_batch_item_delegated_login(None)


def test_delegated_login_response_preserves_unknown_ticket_type_and_redacts_value(monkeypatch) -> None:
    secret = b"ticket-value-must-not-appear-in-repr"
    native_secret = ffi.new("uint8_t[]", secret)
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 24)

    def response(_item, present, ticket_present, ticket_type, ticket_value) -> int:
        present[0] = 1
        ticket_present[0] = 1
        ticket_type[0] = 0xFEED_BEEF
        ticket_value.data = native_secret
        ticket_value.length = len(secret)
        return 0

    def response_ttlv(_item, present, output) -> int:
        present[0] = 1
        output[0] = pointer
        return 0

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    ticket_ttlv = OwnedStructure()
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_delegated_login=response,
            kmipkit_client_batch_item_response_delegated_login_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda _pointer: None,
        ),
    )
    monkeypatch.setattr(
        client.ttlv, "_structure_from_native_address", lambda _address: ticket_ttlv
    )

    response_value = client._delegated_login_response(
        SimpleNamespace(_pointer=lambda: object())
    )

    assert isinstance(response_value, client.DelegatedLoginResponse)
    assert response_value.ticket_type_raw == 0xFEED_BEEF
    assert response_value.ticket_value == secret
    assert response_value.ticket_ttlv is ticket_ttlv
    assert secret.decode() not in repr(response_value)
    response_value.close()
    assert ticket_ttlv.closed


def test_absent_delegated_login_ticket_has_zeroed_native_fields(monkeypatch) -> None:
    calls: list[str] = []

    def response(_item, present, ticket_present, ticket_type, ticket_value) -> int:
        present[0] = 1
        ticket_present[0] = 0
        ticket_type[0] = 0
        ticket_value.data = ffi.NULL
        ticket_value.length = 0
        return 0

    def response_ttlv(_item, present, output) -> int:
        calls.append("ttlv")
        present[0] = 0
        output[0] = ffi.NULL
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_delegated_login=response,
            kmipkit_client_batch_item_response_delegated_login_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda _pointer: None,
        ),
    )

    response_value = client._delegated_login_response(
        SimpleNamespace(_pointer=lambda: object())
    )

    assert response_value is not None
    assert response_value.ticket_type_raw is None
    assert response_value.ticket_value is None
    assert response_value.ticket_ttlv is None
    assert calls == ["ttlv"]
