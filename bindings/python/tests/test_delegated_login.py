from __future__ import annotations

from functools import partial
from types import SimpleNamespace

import pytest
from _test_support import install_ticket_native_response
from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit.errors import InvalidInputError


def test_delegated_login_builder_maps_table_199_fields(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as rights, ttlv.ttlv_structure_create() as usage_limits:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is rights else 29,
        )
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
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as rights:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is rights else ffi.NULL,
        )
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


def test_delegated_login_rejects_two_lease_time_forms_with_valid_rights(monkeypatch) -> None:
    monkeypatch.setattr(
        extensions, "_make_handle", lambda *_args: pytest.fail("conflicting lease times reached the ABI")
    )

    with ttlv.ttlv_structure_create() as rights, pytest.raises(InvalidInputError):
        extensions.client_batch_item_delegated_login(
            rights, lease_time_interval=1, lease_time_date_time=2
        )


def test_delegated_login_requires_generic_rights_structure() -> None:
    with pytest.raises(InvalidInputError):
        extensions.client_batch_item_delegated_login(None)


def test_delegated_login_response_preserves_unknown_ticket_type_and_redacts_value(monkeypatch) -> None:
    secret = b"ticket-value-must-not-appear-in-repr"

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    ticket_ttlv = OwnedStructure()
    _install_delegated_login_native_response(
        monkeypatch, ticket_type=0xFEED_BEEF, ticket_value=secret
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


def test_absent_delegated_login_response_maps_to_none(monkeypatch) -> None:
    _install_delegated_login_native_response(
        monkeypatch,
        present=0,
        ticket_present=0,
        ticket_type=0,
        ticket_value=b"",
        ttlv_present=0,
        ttlv_pointer=0,
    )

    assert client._delegated_login_response(SimpleNamespace(_pointer=lambda: object())) is None


_install_delegated_login_native_response = partial(
    install_ticket_native_response, operation="delegated_login"
)


def test_delegated_login_response_rejects_inconsistent_flags_and_spans(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    invalid_cases = [
        {"present": 2, "ticket_present": 0, "ticket_type": 0, "ticket_value": b""},
        {"present": 1, "ticket_present": 0, "ticket_type": 4, "ticket_value": b""},
        {"present": 1, "ticket_present": 1, "ticket_value": b"", "span_length": 1},
        {
            "present": 1,
            "ticket_present": 1,
            "ticket_value": b"x",
            "span_length": client._MAX_INPUT_BYTES + 1,
        },
    ]

    for values in invalid_cases:
        _install_delegated_login_native_response(monkeypatch, **values)
        with pytest.raises(InvalidInputError):
            client._delegated_login_response(item)


def test_delegated_login_response_releases_inconsistent_ttlv_and_wipes_secret(
    monkeypatch,
) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    released: list[int] = []
    secret_objects = []

    class TrackedSecret:
        def __init__(self, value) -> None:
            self.value = bytes(value)
            self.wiped = False
            secret_objects.append(self)

        def wipe(self) -> None:
            self.wiped = True

    monkeypatch.setattr(client, "_SecretBytes", TrackedSecret)
    _install_delegated_login_native_response(
        monkeypatch, present=1, ticket_present=1, ttlv_present=0, released=released
    )

    with pytest.raises(InvalidInputError):
        client._delegated_login_response(item)

    assert released == [24]
    assert len(secret_objects) == 1
    assert secret_objects[0].wiped


def test_delegated_login_response_closes_ticket_when_response_is_absent(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()
    monkeypatch.setattr(ttlv, "_structure_from_native_address", lambda _address: structure)
    _install_delegated_login_native_response(monkeypatch, present=0, ticket_present=1)

    with pytest.raises(InvalidInputError):
        client._delegated_login_response(item)

    assert structure.closed


def test_delegated_login_response_wipes_secret_when_ttlv_mapping_raises(monkeypatch) -> None:
    item = SimpleNamespace(_pointer=lambda: object())
    released: list[int] = []
    secret_objects = []

    class TrackedSecret:
        def __init__(self, value) -> None:
            self.value = bytes(value)
            self.wiped = False
            secret_objects.append(self)

        def wipe(self) -> None:
            self.wiped = True

    monkeypatch.setattr(client, "_SecretBytes", TrackedSecret)
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("injected conversion failure")),
    )
    _install_delegated_login_native_response(monkeypatch, released=released)

    with pytest.raises(RuntimeError, match="injected conversion failure"):
        client._delegated_login_response(item)

    assert released == [24]
    assert len(secret_objects) == 1
    assert secret_objects[0].wiped


def test_batch_result_dispatches_delegated_login_to_typed_response(monkeypatch) -> None:
    response_handle = ffi.cast("kmipkit_client_batch_response_t *", 1)
    item_handle = ffi.cast("kmipkit_client_batch_item_response_t *", 2)
    ticket_handle = ffi.cast("kmipkit_ttlv_structure_t *", 24)
    secret = b"batch-result-ticket-secret"
    native_secret = ffi.new("uint8_t[]", secret)
    released: list[str] = []

    class OwnedHandle:
        def __init__(self, handle) -> None:
            self.handle = handle

        def _pointer(self):
            return self.handle

        def close(self) -> None:
            if self.handle == response_handle:
                released.append("response")
            elif self.handle == item_handle:
                released.append("item")

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    ticket_structure = OwnedStructure()

    def metadata(_item, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")
        output.abi_version = 2
        output.reserved = 0
        output.operation = 0x2F
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)
        output.result_status = 0
        output.has_result_reason = 0
        output.result_reason = 0
        output.has_result_message = 0
        output.result_message_data = ffi.NULL
        output.result_message_length = 0
        return 0

    def absent_span(_item, present, span) -> int:
        present[0] = 0
        span.data = ffi.NULL
        span.length = 0
        return 0

    def delegated_response(_item, present, ticket_present, ticket_type, ticket_value) -> int:
        present[0] = 1
        ticket_present[0] = 1
        ticket_type[0] = 0xFEED_BEEF
        ticket_value.data = native_secret
        ticket_value.length = len(secret)
        return 0

    def delegated_ttlv(_item, present, output) -> int:
        present[0] = 1
        output[0] = ticket_handle
        return 0

    def absent_create(_item, present, *_outputs) -> int:
        present[0] = 0
        return 0

    def absent_split(_item, present, count) -> int:
        present[0] = 0
        count[0] = 0
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _handle: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _handle: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _handle, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _handle, _index, output: output.__setitem__(0, item_handle) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=absent_span,
        kmipkit_client_batch_item_response_pending_correlation_value=absent_span,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=lambda _item, present, _values, _capacity, count: (
            present.__setitem__(0, 0) or count.__setitem__(0, 0) or 0
        ),
        kmipkit_client_batch_item_response_create=absent_create,
        kmipkit_client_batch_item_response_create_key_pair=lambda _item, present, *_outputs: present.__setitem__(0, 0) or 0,
        kmipkit_client_batch_item_response_create_split_key_count=absent_split,
        kmipkit_client_batch_item_response_delegated_login=delegated_response,
        kmipkit_client_batch_item_response_delegated_login_ttlv=delegated_ttlv,
        kmipkit_ttlv_structure_release=lambda _handle: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client, "_wrap_owned_handle", lambda _kind, handle, *_owners: OwnedHandle(handle))
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: ticket_structure if address == 24 else None,
    )

    result = client._batch_result(response_handle)

    mapped = result.items[0].delegated_login_response
    assert isinstance(mapped, client.DelegatedLoginResponse)
    assert mapped.ticket_type_raw == 0xFEED_BEEF
    assert mapped.ticket_value == secret
    assert mapped.ticket_ttlv is ticket_structure
    mapped.close()
    result.close()
    assert ticket_structure.closed
    assert released == ["item", "response"]
