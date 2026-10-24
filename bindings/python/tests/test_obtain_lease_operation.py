"""Obtain Lease binding parity for OASIS KMIP 2.1 §6.1.35, Tables 268–270.

Traces KMIPKIT-REQ-SPEC-6.1.35-001-001/-001-002/-002/-005 and owner requirements
KMIPKIT-0036-FR-001 through KMIPKIT-0036-FR-008.
"""

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


def test_batch_item_result_preserves_legacy_positional_field_order() -> None:
    check_response = object()
    legacy_arguments = [client.BatchOutcome.COMPLETED, 0, None, None, None]
    legacy_arguments.extend((None, 0, None, *([None] * 12)))
    legacy_arguments.append(check_response)

    result = client.BatchItemResult(*legacy_arguments)  # type: ignore[arg-type]

    assert result.check_response is check_response
    assert result.obtain_lease_response is None
    field_order = client.BatchItemResult.__match_args__
    assert field_order.index("revoke_response") < field_order.index("set_defaults_response")
    assert field_order.index("obtain_lease_response") < field_order.index("encrypt_response")
    assert field_order.index("encrypt_response") < field_order.index("decrypt_response")


def test_obtain_lease_response_preserves_zero_and_raw_datetime_and_redacts_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_copy = _obtain_lease_response_ttlv()
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
    with ttlv.ttlv_structure_view(result.response_ttlv) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 4
        assert _response_tag_at(view, 0) == 0x0042_0094
        assert _response_tag_at(view, 1) == 0x0042_0049
        assert _response_tag_at(view, 2) == 0x0042_0048
        assert _response_tag_at(view, 3) == 0x0054_1234
    result.close()


def test_obtain_lease_releases_unexpected_ttlv_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    ttlv_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 73)
    identifier_text = ffi.new("uint8_t[]", b"leased-object")
    released: list[object] = []

    def response(_item: object, present: object, identifier: object,
                 lease_time: object, last_change_date: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_text  # type: ignore[attr-defined]
        identifier.text_length = len(b"leased-object")  # type: ignore[attr-defined]
        lease_time[0] = 5  # type: ignore[index]
        last_change_date[0] = 7  # type: ignore[index]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = ttlv_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_obtain_lease=response,
            kmipkit_client_batch_item_response_obtain_lease_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._obtain_lease_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [ttlv_pointer]


def test_obtain_lease_releases_ttlv_handle_when_wrapper_conversion_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    ttlv_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 74)
    identifier_text = ffi.new("uint8_t[]", b"leased-object")
    released: list[object] = []

    def response(_item: object, present: object, identifier: object,
                 lease_time: object, last_change_date: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_text  # type: ignore[attr-defined]
        identifier.text_length = len(b"leased-object")  # type: ignore[attr-defined]
        lease_time[0] = 5  # type: ignore[index]
        last_change_date[0] = 7  # type: ignore[index]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = ttlv_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_obtain_lease=response,
            kmipkit_client_batch_item_response_obtain_lease_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    def fail_conversion(_address: int) -> ttlv.Structure:
        raise RuntimeError("test conversion failure")

    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", fail_conversion)

    with pytest.raises(RuntimeError, match="test conversion failure"):
        client._obtain_lease_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [ttlv_pointer]


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


def _response_tag_at(view: ttlv.TtlvStructureView, index: int) -> int:
    item_view = ttlv.ttlv_structure_view_item_at(view, index)
    tag = ttlv.ttlv_item_view_tag(item_view)
    try:
        return ttlv.tag_value(tag)
    finally:
        tag.close()
        item_view.close()


def _obtain_lease_response_ttlv() -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()

    def item(tag_value: int, value: ttlv.Value) -> ttlv.TtlvItem:
        raw_tag = ttlv.raw_tag_from_raw(tag_value)
        tag = ttlv.raw_tag_try_checked(raw_tag)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            raw_tag.close()

    try:
        response = ttlv.ttlv_structure_create()
        for child in (
            item(0x0042_0094, ttlv.ttlv_value_text_string(b"leased-object", limits)),
            item(0x0042_0049, ttlv.ttlv_value_interval(0)),
            item(0x0042_0048, ttlv.ttlv_value_date_time(-7)),
            item(0x0054_1234, ttlv.ttlv_value_integer(-7)),
        ):
            response = ttlv.ttlv_structure_with_item(response, child, limits)
        return response
    finally:
        limits.close()
