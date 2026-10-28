from __future__ import annotations

from functools import partial
from types import SimpleNamespace

import pytest
from _test_support import install_ticket_native_response

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit.errors import InvalidInputError


def test_login_builder_maps_table_253_fields(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as usage_limits:
        monkeypatch.setattr(extensions, "_native_structure", lambda _value, **_kwargs: 29)
        item = extensions.client_batch_item_login(
            lease_time_interval=0xFFFF_FFFF,
            request_count=-(1 << 31),
            usage_limits=usage_limits,
        )

    assert item == 23
    assert calls == [(
        "kmipkit_client_batch_item_login",
        (1, 0xFFFF_FFFF, 1, -(1 << 31), 29),
    )]


def test_login_builder_maps_lease_time_date_time(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_login(lease_time_date_time=-(1 << 63))

    assert item == 23
    assert calls == [
        ("kmipkit_client_batch_item_login", (2, -(1 << 63), 0, 0, ffi.NULL))
    ]


def test_login_builder_rejects_two_lease_time_fields() -> None:
    with pytest.raises(InvalidInputError):
        extensions.client_batch_item_login(
            lease_time_interval=1,
            lease_time_date_time=1,
        )


def test_login_response_preserves_unknown_ticket_and_redacts_secret(monkeypatch) -> None:
    secret = b"login-ticket-secret"

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    ticket_ttlv = OwnedStructure()
    _install_login_native_response(
        monkeypatch, ticket_type=0xFEED_BEEF, ticket_value=secret
    )
    monkeypatch.setattr(
        client.ttlv, "_structure_from_native_address", lambda _address: ticket_ttlv
    )

    response_value = client._login_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(response_value, client.LoginResponse)
    assert response_value.ticket_type_raw == 0xFEED_BEEF
    assert response_value.ticket_value == secret
    assert response_value.ticket_ttlv is ticket_ttlv
    assert secret.decode() not in repr(response_value)
    response_value.close()
    assert ticket_ttlv.closed


def test_login_response_rejects_inconsistent_native_outputs(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    invalid_cases = [
        {"present": 2, "ticket_present": 0},
        {"present": 1, "ticket_present": 2},
        {"present": 1, "ticket_present": 0, "ticket_type": 4},
        {"present": 1, "ticket_present": 1, "ticket_value": b"", "span_length": 1},
        {
            "present": 1,
            "ticket_present": 1,
            "ticket_value": b"x",
            "span_length": client._MAX_INPUT_BYTES + 1,
        },
    ]

    for values in invalid_cases:
        _install_login_native_response(monkeypatch, **values)
        with pytest.raises(InvalidInputError):
            client._login_response(item)


def test_login_response_releases_inconsistent_ttlv_and_wipes_secret(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    released: list[int] = []
    secrets = []

    class TrackedSecret:
        def __init__(self, value) -> None:
            self.value = bytes(value)
            self.wiped = False
            secrets.append(self)

        def wipe(self) -> None:
            self.wiped = True

    monkeypatch.setattr(client, "_SecretBytes", TrackedSecret)
    _install_login_native_response(
        monkeypatch, present=1, ticket_present=1, ttlv_present=0, released=released
    )

    with pytest.raises(InvalidInputError):
        client._login_response(item)

    assert released == [24]
    assert len(secrets) == 1
    assert secrets[0].wiped


def test_login_response_releases_native_ttlv_when_mapping_raises(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    released: list[int] = []
    secrets = []

    class TrackedSecret:
        def __init__(self, value) -> None:
            self.value = bytes(value)
            self.wiped = False
            secrets.append(self)

        def wipe(self) -> None:
            self.wiped = True

    monkeypatch.setattr(client, "_SecretBytes", TrackedSecret)
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("injected conversion failure")),
    )
    _install_login_native_response(monkeypatch, released=released)

    with pytest.raises(RuntimeError, match="injected conversion failure"):
        client._login_response(item)

    assert released == [24]
    assert len(secrets) == 1
    assert secrets[0].wiped


def test_login_response_closes_ticket_if_native_response_is_absent(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()
    monkeypatch.setattr(ttlv, "_structure_from_native_address", lambda _address: structure)
    _install_login_native_response(monkeypatch, present=0, ticket_present=1)

    with pytest.raises(InvalidInputError):
        client._login_response(item)

    assert structure.closed


def test_absent_login_response_maps_to_none(monkeypatch) -> None:
    _install_login_native_response(
        monkeypatch,
        present=0,
        ticket_present=0,
        ticket_type=0,
        ticket_value=b"",
        ttlv_present=0,
        ttlv_pointer=0,
    )

    assert client._login_response(SimpleNamespace(_pointer=lambda: object())) is None


_install_login_native_response = partial(
    install_ticket_native_response, operation="login"
)
