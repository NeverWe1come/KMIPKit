from __future__ import annotations

from collections.abc import Callable
from types import SimpleNamespace

import pytest
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def _sentinel_structure(
    tag_value: int, marker: bytes, limits: ttlv.CodecLimits
) -> ttlv.Structure:
    with (
        ttlv.raw_tag_from_raw(tag_value) as raw_tag,
        ttlv.raw_tag_try_checked(raw_tag) as tag,
        ttlv.ttlv_value_byte_string(marker, limits) as value,
        ttlv.ttlv_item_create(tag, value, limits) as item,
    ):
        structure = ttlv.ttlv_structure_create()
        try:
            return ttlv.ttlv_structure_with_item(structure, item, limits)
        finally:
            structure.close()


def _typed_pair_response(
    present_value: int,
    private_value: bytes | None = b"replacement-private",
    public_value: bytes | None = b"replacement-public",
):
    buffers: list[object] = []

    def typed(_item: object, present: object, private: object, public: object) -> int:
        present[0] = present_value
        for record, value in ((private, private_value), (public, public_value)):
            if value is None:
                continue
            buffer = ffi.new("uint8_t[]", value)
            buffers.append(buffer)
            record.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
            record.text_data = buffer
            record.text_length = len(value)
        return 0

    return typed


def _response_item() -> SimpleNamespace:
    return SimpleNamespace(_pointer=lambda: object())


def _transferred_structure() -> tuple[object, int]:
    structure = ttlv.ttlv_structure_create()
    handle, _owner = structure._take()
    address = int(ttlv.ffi.cast("uintptr_t", handle))
    assert structure.closed
    return ffi.cast("kmipkit_ttlv_structure_t *", address), address


def _assert_native_structure_released(address: int) -> None:
    probe = ttlv._structure_from_native_address(address)
    try:
        with pytest.raises(errors.InvalidInputError):
            ttlv.ttlv_structure_view(probe)
    finally:
        probe.close()


def _tracked_native_release() -> tuple[Callable[[object], None], list[object]]:
    release = client.lib.kmipkit_ttlv_structure_release
    released_arguments: list[object] = []

    def release_and_record(handle: object) -> None:
        released_arguments.append(handle)
        release(handle)

    return release_and_record, released_arguments


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
        ttlv.codec_limits_defaults() as limits,
        _sentinel_structure(
            0x541101, b"common-attributes", limits
        ) as common_attributes,
        _sentinel_structure(
            0x541102, b"private-attributes", limits
        ) as private_attributes,
        _sentinel_structure(
            0x541103, b"public-attributes", limits
        ) as public_attributes,
        _sentinel_structure(0x541104, b"common-masks", limits) as common_masks,
        _sentinel_structure(0x541105, b"private-masks", limits) as private_masks,
        _sentinel_structure(0x541106, b"public-masks", limits) as public_masks,
    ):
        item = extensions.client_batch_item_rekey_key_pair(
            common_attributes=common_attributes,
            private_key_attributes=private_attributes,
            public_key_attributes=public_attributes,
            common_protection_storage_masks=common_masks,
            private_protection_storage_masks=private_masks,
            public_protection_storage_masks=public_masks,
        )
        expected_arguments = tuple(
            ffi.cast("uintptr_t", structure._pointer())
            for structure in (
                common_attributes,
                private_attributes,
                public_attributes,
                common_masks,
                private_masks,
                public_masks,
            )
        )
    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_rekey_key_pair"
    assert all(value != ffi.NULL for value in calls[0][1][3:])
    assert tuple(ffi.cast("uintptr_t", value) for value in calls[0][1][3:]) == tuple(
        expected_arguments
    )


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


def test_pair_response_returns_none_for_an_absent_native_response(monkeypatch) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey_key_pair=_typed_pair_response(
                0, None, None
            )
        ),
    )

    assert client._rekey_key_pair_response(_response_item()) is None


def test_pair_response_rejects_invalid_presence_and_missing_identifier(
    monkeypatch,
) -> None:
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey_key_pair=_typed_pair_response(
                2, None, None
            )
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._rekey_key_pair_response(_response_item())

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey_key_pair=_typed_pair_response(
                1, b"replacement-private", None
            )
        ),
    )
    with pytest.raises(errors.InvalidInputError):
        client._rekey_key_pair_response(_response_item())


def test_pair_response_releases_native_ttlv_on_presence_mismatch(monkeypatch) -> None:
    native_ttlv, address = _transferred_structure()
    release_and_record, released_arguments = _tracked_native_release()

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey_key_pair=_typed_pair_response(1),
            kmipkit_client_batch_item_response_rekey_key_pair_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=release_and_record,
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._rekey_key_pair_response(_response_item())

    assert released_arguments == [native_ttlv]
    _assert_native_structure_released(address)


def test_pair_response_releases_native_ttlv_if_wrapper_creation_raises(
    monkeypatch,
) -> None:
    native_ttlv, address = _transferred_structure()
    release_and_record, released_arguments = _tracked_native_release()
    structure_factory = ttlv._structure_from_native_address

    def get_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rekey_key_pair=_typed_pair_response(1),
            kmipkit_client_batch_item_response_rekey_key_pair_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=release_and_record,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("wrapper creation failed")),
    )

    with pytest.raises(RuntimeError, match="wrapper creation failed"):
        client._rekey_key_pair_response(_response_item())

    assert released_arguments == [native_ttlv]
    monkeypatch.setattr(ttlv, "_structure_from_native_address", structure_factory)
    _assert_native_structure_released(address)
