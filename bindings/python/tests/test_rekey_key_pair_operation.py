from __future__ import annotations
from types import SimpleNamespace

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi


def test_pair_request_preserves_omitted_identifier_and_all_optional_fields(
    monkeypatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    item = extensions.client_batch_item_rekey_key_pair(offset=0xFFFF_FFFF)
    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_rekey_key_pair"
    assert calls[0][1][0] == ffi.NULL
    assert calls[0][1][1:3] == (1, 0xFFFF_FFFF)
    assert calls[0][1][3:] == (ffi.NULL,) * 6


def test_pair_request_passes_each_attribute_and_mask_group(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    with (
        ttlv.ttlv_structure_create() as common_attributes,
        ttlv.ttlv_structure_create() as private_attributes,
        ttlv.ttlv_structure_create() as public_attributes,
        ttlv.ttlv_structure_create() as common_masks,
        ttlv.ttlv_structure_create() as private_masks,
        ttlv.ttlv_structure_create() as public_masks,
    ):
        item = extensions.client_batch_item_rekey_key_pair(
            common_attributes=common_attributes,
            private_key_attributes=private_attributes,
            public_key_attributes=public_attributes,
            common_protection_storage_masks=common_masks,
            private_protection_storage_masks=private_masks,
            public_protection_storage_masks=public_masks,
        )
    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_rekey_key_pair"
    assert all(value != ffi.NULL for value in calls[0][1][3:])


def test_pair_response_maps_both_ids_owned_ttlv_and_redaction(monkeypatch) -> None:
    private_value = b"replacement-private"
    public_value = b"replacement-public"
    private_bytes = ffi.new("uint8_t[]", private_value)
    public_bytes = ffi.new("uint8_t[]", public_value)
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 42)
    closed: list[bool] = []

    class OwnedStructure:
        def close(self) -> None:
            closed.append(True)

    owned = OwnedStructure()

    def typed(_item: object, present: object, private: object, public: object) -> int:
        present[0] = 1
        for record, data in ((private, private_bytes), (public, public_bytes)):
            record.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
            record.text_data = data
            record.text_length = (
                len(private_value) if record is private else len(public_value)
            )
        return 0

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey_key_pair=typed,
            kmipkit_client_batch_item_response_rekey_key_pair_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned if address == 42 else None,
    )
    response = client._rekey_key_pair_response(
        SimpleNamespace(_pointer=lambda: object())
    )
    assert response is not None
    assert (
        response.private_key_unique_identifier
        == extensions.UniqueIdentifier.text_string("replacement-private")
    )
    assert (
        response.public_key_unique_identifier
        == extensions.UniqueIdentifier.text_string("replacement-public")
    )
    assert response.response_ttlv is owned
    assert "replacement-private" not in repr(
        response
    ) and "replacement-public" not in repr(response)
    response.close()
    assert closed == [True]
