from __future__ import annotations

from types import SimpleNamespace

import pytest
from _test_support import batch_result_test_symbols
from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi
from kmipkit.errors import InvalidInputError, ResourceLimitError


def test_derive_key_builder_preserves_ordered_identifier_wire_types(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...], list[tuple[int, bytes, int, int]]]] = []

    def make_handle(_kind, symbol, arguments):
        identifiers_arg = arguments[1]
        identifiers = []
        for index in range(arguments[2]):
            identifier = identifiers_arg[index]
            text = (
                bytes(ffi.buffer(identifier.text_data, identifier.text_length))
                if identifier.kind == 1
                else b""
            )
            identifiers.append(
                (identifier.kind, text, identifier.enumeration_value, identifier.integer_value)
            )
        calls.append((symbol, arguments, identifiers))
        return 23

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    identifiers = (
        extensions.UniqueIdentifier.text_string("source-a"),
        extensions.UniqueIdentifier.enumeration(0xFEED_BEEF),
        extensions.UniqueIdentifier.text_string("source-a"),
    )
    with ttlv.ttlv_structure_create() as parameters, ttlv.ttlv_structure_create() as attributes:
        monkeypatch.setattr(
            extensions,
            "_native_structure",
            lambda value, **_kwargs: 17 if value is parameters else 29,
        )
        item = extensions.client_batch_item_derive_key(
            2, identifiers, 0xFEED_BEEF, parameters, attributes
        )

    assert item == 23
    assert len(calls) == 1
    symbol, arguments, copied_identifiers = calls[0]
    assert symbol == "kmipkit_client_batch_item_derive_key"
    assert (arguments[0], arguments[2], arguments[3], arguments[4], arguments[5]) == (
        2,
        3,
        0xFEED_BEEF,
        17,
        29,
    )
    assert copied_identifiers == [
        (1, b"source-a", 0, 0),
        (2, b"", 0xFEED_BEEF, 0),
        (1, b"source-a", 0, 0),
    ]


def test_derive_key_builder_rejects_missing_source_identifiers(monkeypatch) -> None:
    monkeypatch.setattr(extensions, "_make_handle", lambda *_args: 23)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    with ttlv.ttlv_structure_create() as parameters, ttlv.ttlv_structure_create() as attributes:
        with pytest.raises(InvalidInputError):
            extensions.client_batch_item_derive_key(
                2, (), 1, parameters, attributes
            )


def test_derive_key_builder_rejects_identifier_bytes_over_operation_limit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(extensions, "_MAX_OPERATION_INPUT_BYTES", 3)
    monkeypatch.setattr(
        extensions, "_make_handle", lambda *_args: pytest.fail("oversized inputs reached the ABI")
    )
    identifiers = (extensions.UniqueIdentifier.text_string("source-a"),)

    with ttlv.ttlv_structure_create() as parameters, ttlv.ttlv_structure_create() as attributes:
        with pytest.raises(ResourceLimitError):
            extensions.client_batch_item_derive_key(
                2, identifiers, 1, parameters, attributes
            )


@pytest.mark.parametrize("invalid_field", ("derivation_parameters", "attributes"))
def test_derive_key_builder_requires_structure_inputs(
    monkeypatch: pytest.MonkeyPatch, invalid_field: str
) -> None:
    monkeypatch.setattr(
        extensions, "_make_handle", lambda *_args: pytest.fail("invalid inputs reached the ABI")
    )
    identifiers = (extensions.UniqueIdentifier.text_string("source-a"),)

    with ttlv.ttlv_structure_create() as parameters, ttlv.ttlv_structure_create() as attributes:
        parameters_input = object() if invalid_field == "derivation_parameters" else parameters
        attributes_input = object() if invalid_field == "attributes" else attributes
        with pytest.raises(InvalidInputError):
            extensions.client_batch_item_derive_key(
                2, identifiers, 1, parameters_input, attributes_input
            )


@pytest.mark.parametrize(("status", "expects_response"), ((0, True), (1, False)))
def test_batch_result_maps_derive_key_response_only_on_success(
    monkeypatch: pytest.MonkeyPatch, status: int, expects_response: bool
) -> None:
    response_pointer = ffi.cast("kmipkit_client_batch_response_t *", 1)
    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 2)
    native_handle = ffi.NULL
    if expects_response:
        native_copy = _derive_key_response_ttlv()
        native_handle, owner = native_copy._take()
        assert owner is None
    calls: list[str] = []
    identifier_storage = ffi.new("uint8_t[]", b"derived-key")

    def metadata(_item: object, output: object) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")  # type: ignore[attr-defined]
        output.abi_version = 2  # type: ignore[attr-defined]
        output.reserved = 0  # type: ignore[attr-defined]
        output.operation = 0x05  # type: ignore[attr-defined]
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        output.result_status = status  # type: ignore[attr-defined]
        output.has_result_reason = 0  # type: ignore[attr-defined]
        output.result_reason = 0  # type: ignore[attr-defined]
        output.has_result_message = 0  # type: ignore[attr-defined]
        output.result_message_data = ffi.NULL  # type: ignore[attr-defined]
        output.result_message_length = 0  # type: ignore[attr-defined]
        return 0

    def derive_key_response(_item: object, present: object, identifier: object) -> int:
        calls.append("derive-key")
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"derived-key")  # type: ignore[attr-defined]
        return 0

    def derive_key_ttlv(_item: object, present: object, output: object) -> int:
        calls.append("derive-key-ttlv")
        present[0] = 1  # type: ignore[index]
        output[0] = ffi.cast(
            "kmipkit_ttlv_structure_t *",
            int(ttlv_ffi.cast("uintptr_t", native_handle)),
        )  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        **batch_result_test_symbols(ffi, item_pointer, metadata),
        kmipkit_client_batch_item_response_derive_key=derive_key_response,
        kmipkit_client_batch_item_response_derive_key_ttlv=derive_key_ttlv,
    )
    monkeypatch.setattr(client, "lib", fake)

    result = client._batch_result(response_pointer)

    derive_key = getattr(result.items[0], "derive_key_response", None)
    assert (derive_key is not None) is expects_response
    if expects_response:
        assert derive_key.unique_identifier == extensions.UniqueIdentifier.text_string("derived-key")
        assert "derived-key" not in repr(result.items[0])
        with ttlv.ttlv_structure_view(derive_key.response_ttlv) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 2
            assert _response_tag_at(view, 0) == 0x0042_0094
            assert _response_tag_at(view, 1) == 0x0054_0001
        derive_key.close()
        assert derive_key.response_ttlv.closed
    assert calls == (["derive-key", "derive-key-ttlv"] if expects_response else [])
    result.close()


def test_derive_key_response_rejects_absent_identifier_for_successful_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item = SimpleNamespace(_pointer=lambda: ffi.cast("kmipkit_client_batch_item_response_t *", 1))

    def derive_key_response(_item: object, present: object, _identifier: object) -> int:
        present[0] = 0  # type: ignore[index]
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(
        kmipkit_client_batch_item_response_derive_key=derive_key_response,
    ))

    with pytest.raises(InvalidInputError):
        client._derive_key_response(item)


@pytest.mark.parametrize(
    ("present_value", "identifier_kind"),
    ((0, 1), (2, 0), (1, 0)),
    ids=("absent-with-identifier", "invalid-presence", "present-without-identifier"),
)
def test_derive_key_response_rejects_inconsistent_presence_fields(
    monkeypatch: pytest.MonkeyPatch,
    present_value: int,
    identifier_kind: int,
) -> None:
    item = SimpleNamespace(_pointer=lambda: ffi.cast("kmipkit_client_batch_item_response_t *", 1))

    def derive_key_response(_item: object, present: object, identifier: object) -> int:
        present[0] = present_value  # type: ignore[index]
        identifier.kind = identifier_kind  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(
        kmipkit_client_batch_item_response_derive_key=derive_key_response,
    ))

    with pytest.raises(InvalidInputError):
        client._derive_key_response(item)


@pytest.mark.parametrize("response_ttlv_present", (0, 2))
def test_derive_key_response_releases_mismatched_native_ttlv(
    monkeypatch: pytest.MonkeyPatch,
    response_ttlv_present: int,
) -> None:
    item = SimpleNamespace(_pointer=lambda: ffi.cast("kmipkit_client_batch_item_response_t *", 1))
    identifier_storage = ffi.new("uint8_t[]", b"derived-key")
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 3)
    released: list[object] = []

    def derive_key_response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"derived-key")  # type: ignore[attr-defined]
        return 0

    def derive_key_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = response_ttlv_present  # type: ignore[index]
        output[0] = native_ttlv  # type: ignore[index]
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(
        kmipkit_client_batch_item_response_derive_key=derive_key_response,
        kmipkit_client_batch_item_response_derive_key_ttlv=derive_key_ttlv,
        kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
    ))

    with pytest.raises(InvalidInputError):
        client._derive_key_response(item)
    assert released == [native_ttlv]


def test_derive_key_response_releases_ttlv_when_wrapping_it_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item = SimpleNamespace(_pointer=lambda: ffi.cast("kmipkit_client_batch_item_response_t *", 1))
    identifier_storage = ffi.new("uint8_t[]", b"derived-key")
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 3)
    released: list[object] = []

    def derive_key_response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"derived-key")  # type: ignore[attr-defined]
        return 0

    def derive_key_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = native_ttlv  # type: ignore[index]
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(
        kmipkit_client_batch_item_response_derive_key=derive_key_response,
        kmipkit_client_batch_item_response_derive_key_ttlv=derive_key_ttlv,
        kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
    ))
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("test wrapper failure")),
    )

    with pytest.raises(RuntimeError, match="test wrapper failure"):
        client._derive_key_response(item)
    assert released == [native_ttlv]


def _derive_key_response_ttlv() -> ttlv.Structure:
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
            item(0x0042_0094, ttlv.ttlv_value_text_string(b"derived-key", limits)),
            item(0x0054_0001, ttlv.ttlv_value_byte_string(b"vendor", limits)),
        ):
            response = ttlv.ttlv_structure_with_item(response, child, limits)
        return response
    finally:
        limits.close()


def _response_tag_at(view: ttlv.TtlvStructureView, index: int) -> int:
    item_view = ttlv.ttlv_structure_view_item_at(view, index)
    tag = ttlv.ttlv_item_view_tag(item_view)
    try:
        return ttlv.tag_value(tag)
    finally:
        tag.close()
        item_view.close()
