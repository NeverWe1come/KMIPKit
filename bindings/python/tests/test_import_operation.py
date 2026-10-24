from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi
from kmipkit._ffi import ffi as ttlv_ffi


def test_import_builder_preserves_required_values_and_explicit_optional_choices(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 29,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    attributes = ttlv.ttlv_structure_create()
    any_object = ttlv.ttlv_structure_create()

    item = extensions.client_batch_item_import(
        extensions.UniqueIdentifier.integer(-29), 0xF1234567, attributes, any_object,
        replace_existing=False, key_wrap_type=0xF1234567,
    )

    assert item == 29
    assert calls[0][0] == "kmipkit_client_batch_item_import"
    arguments = calls[0][1]
    assert arguments[1:6] == (0xF1234567, 1, 0, 1, 0xF1234567)
    attributes.close()
    any_object.close()


def test_import_builder_rejects_missing_required_values_and_non_boolean_replacement() -> None:
    structure = ttlv.ttlv_structure_create()
    try:
        with pytest.raises(errors.InvalidInputError):
            extensions.client_batch_item_import(None, 1, structure, structure)  # type: ignore[arg-type]
        with pytest.raises(errors.InvalidInputError):
            extensions.client_batch_item_import(
                extensions.UniqueIdentifier.text_string("id"), 1 << 32, structure, structure
            )
        with pytest.raises(errors.InvalidInputError):
            extensions.client_batch_item_import(
                extensions.UniqueIdentifier.text_string("id"), 1, structure, structure,
                replace_existing=1,  # type: ignore[arg-type]
            )
    finally:
        structure.close()


def test_success_response_owns_ttlv_and_redacts_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    structure = ttlv.ttlv_structure_create()
    raw_handle, owner = structure._take()
    assert owner is None
    pointer = ffi.cast(
        "kmipkit_ttlv_structure_t *", int(ttlv_ffi.cast("uintptr_t", raw_handle))
    )
    identifier_bytes = b"private-import-id"
    storage = ffi.new("uint8_t[]", identifier_bytes)
    retained = [storage]

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = storage  # type: ignore[attr-defined]
        identifier.text_length = len(identifier_bytes)  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_import=response,
            kmipkit_client_batch_item_response_import_ttlv=response_ttlv,
        ),
    )

    result = client._import_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.ImportResponse)
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string("private-import-id")
    assert "private-import-id" not in repr(result)
    assert result.response_ttlv._pointer() == pointer
    result.close()
    assert result.response_ttlv.closed
    assert retained


def test_non_success_has_no_import_payload_and_inconsistent_success_is_rejected(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def absent(_item: object, present: object, _identifier: object) -> int:
        present[0] = 0  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client, "lib", SimpleNamespace(kmipkit_client_batch_item_response_import=absent)
    )
    assert client._import_response(SimpleNamespace(_pointer=lambda: object())) is None

    def missing_identifier(_item: object, present: object, _identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client, "lib", SimpleNamespace(kmipkit_client_batch_item_response_import=missing_identifier)
    )
    with pytest.raises(errors.InvalidInputError):
        client._import_response(SimpleNamespace(_pointer=lambda: object()))


@pytest.mark.parametrize(
    ("present_value", "identifier_kind"),
    ((2, 0), (0, 1), (1, 0)),
    ids=("invalid-presence", "absent-with-identifier", "present-without-identifier"),
)
def test_import_response_rejects_inconsistent_identifier_presence(
    monkeypatch: pytest.MonkeyPatch,
    present_value: int,
    identifier_kind: int,
) -> None:
    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = present_value  # type: ignore[index]
        identifier.kind = identifier_kind  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client, "lib", SimpleNamespace(kmipkit_client_batch_item_response_import=response)
    )

    with pytest.raises(errors.InvalidInputError):
        client._import_response(SimpleNamespace(_pointer=lambda: object()))


@pytest.mark.parametrize("response_ttlv_present", (0, 2))
def test_import_response_releases_inconsistent_native_ttlv(
    monkeypatch: pytest.MonkeyPatch,
    response_ttlv_present: int,
) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 3)
    released: list[object] = []
    identifier_storage = ffi.new("uint8_t[]", b"private-import-id")

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"private-import-id")  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = response_ttlv_present  # type: ignore[index]
        output[0] = native_ttlv  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_import=response,
            kmipkit_client_batch_item_response_import_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._import_response(SimpleNamespace(_pointer=lambda: object()))
    assert released == [native_ttlv]


def test_import_response_releases_ttlv_when_native_wrapper_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 3)
    released: list[object] = []
    identifier_storage = ffi.new("uint8_t[]", b"private-import-id")

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"private-import-id")  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = native_ttlv  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_import=response,
            kmipkit_client_batch_item_response_import_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda handle: released.append(handle),
        ),
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("test wrapper failure")),
    )

    with pytest.raises(RuntimeError, match="test wrapper failure"):
        client._import_response(SimpleNamespace(_pointer=lambda: object()))
    assert released == [native_ttlv]
