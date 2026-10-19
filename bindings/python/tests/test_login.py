from __future__ import annotations

from types import SimpleNamespace

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi


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


def test_login_response_preserves_unknown_ticket_and_redacts_secret(monkeypatch) -> None:
    secret = b"login-ticket-secret"
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
            kmipkit_client_batch_item_response_login=response,
            kmipkit_client_batch_item_response_login_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda _pointer: None,
        ),
    )
    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", lambda _address: ticket_ttlv)

    response_value = client._login_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(response_value, client.LoginResponse)
    assert response_value.ticket_type_raw == 0xFEED_BEEF
    assert response_value.ticket_value == secret
    assert response_value.ticket_ttlv is ticket_ttlv
    assert secret.decode() not in repr(response_value)
    response_value.close()
    assert ticket_ttlv.closed
