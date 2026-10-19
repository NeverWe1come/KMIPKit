from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi


def test_obtain_lease_builder_preserves_optional_identifier(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    handle = extensions.client_batch_item_obtain_lease(
        extensions.UniqueIdentifier.text_string("leased-object")
    )

    assert handle == 23
    assert calls[0][0] == "kmipkit_client_batch_item_obtain_lease"
    assert calls[0][1][0] != ffi.NULL


def test_obtain_lease_builder_omits_identifier_for_batch_placeholder(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions.client_batch_item_obtain_lease(None) == 23
    assert calls[0][1] == (ffi.NULL,)


def test_obtain_lease_response_preserves_zero_and_raw_datetime_and_redacts_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_copy = ttlv.ttlv_structure_create()
    native_handle, owner = native_copy._take()
    assert owner is None
    storage_refs: list[object] = []

    def response(_item: object, present: object, identifier: object,
                 lease_time: object, last_change_date: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        encoded = b"leased-object"
        storage = ffi.new("uint8_t[]", encoded)
        storage_refs.append(storage)
        identifier.text_data = storage  # type: ignore[attr-defined]
        identifier.text_length = len(encoded)  # type: ignore[attr-defined]
        lease_time[0] = 0  # type: ignore[index]
        last_change_date[0] = -7  # type: ignore[index]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = ffi.cast(  # type: ignore[index]
            "kmipkit_ttlv_structure_t *",
            int(ttlv_ffi.cast("uintptr_t", native_handle)),
        )
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_obtain_lease=response,
            kmipkit_client_batch_item_response_obtain_lease_ttlv=response_ttlv,
        ),
    )

    result = client._obtain_lease_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.ObtainLeaseResponse)
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("leased-object")
    assert result.lease_time == 0
    assert result.last_change_date == -7
    assert "leased-object" not in repr(result)
    result.close()


def test_obtain_lease_absent_result_has_no_success_ttlv(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[str] = []

    def response(_item: object, present: object, identifier: object,
                 lease_time: object, last_change_date: object) -> int:
        calls.append("typed")
        present[0] = 0  # type: ignore[index]
        return 0

    def should_not_read_ttlv(*_arguments: object) -> int:
        calls.append("ttlv")
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_obtain_lease=response,
            kmipkit_client_batch_item_response_obtain_lease_ttlv=should_not_read_ttlv,
        ),
    )

    assert client._obtain_lease_response(SimpleNamespace(_pointer=lambda: object())) is None
    assert calls == ["typed"]


@pytest.mark.parametrize(("present", "identifier_kind", "lease_time"), ((0, 0, 1), (2, 1, 0), (1, 0, 0)))
def test_obtain_lease_rejects_inconsistent_success_fields(
    monkeypatch: pytest.MonkeyPatch, present: int, identifier_kind: int, lease_time: int
) -> None:
    def response(_item: object, out_present: object, identifier: object,
                 out_lease_time: object, out_last_change_date: object) -> int:
        out_present[0] = present  # type: ignore[index]
        identifier.kind = identifier_kind  # type: ignore[attr-defined]
        out_lease_time[0] = lease_time  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_obtain_lease=response),
    )

    with pytest.raises(errors.InvalidInputError):
        client._obtain_lease_response(SimpleNamespace(_pointer=lambda: object()))
