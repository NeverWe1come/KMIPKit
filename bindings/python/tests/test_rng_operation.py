"""Python binding checks derived from KMIP 2.1 §§6.1.49–6.1.50."""

from __future__ import annotations

from types import SimpleNamespace

import pytest
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi


def _response_item() -> SimpleNamespace:
    return SimpleNamespace(_pointer=lambda: object())


def _owned_structure(address: int):
    native = ffi.cast("kmipkit_ttlv_structure_t *", address)

    class OwnedStructure:
        def __init__(self) -> None:
            self.closed = False

        def close(self) -> None:
            self.closed = True

    owned = OwnedStructure()
    return native, owned


def test_rng_retrieve_builder_preserves_signed_length(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_rng_retrieve(-17)

    assert item == 41
    assert calls == [("kmipkit_client_batch_item_rng_retrieve", (-17,))]


def test_rng_seed_builder_requires_data_and_wipes_native_staging(monkeypatch) -> None:
    caller_data = bytearray(b"KMIP_RNG_SEED_SECRET_SENTINEL")
    staged: list[object] = []
    observed: list[bytes] = []
    original_owned_bytes = extensions._owned_bytes

    def track_owned_bytes(value: object, maximum: int = 4_096):
        raw, buffer = original_owned_bytes(value, maximum)  # type: ignore[arg-type]
        staged.append(buffer)
        return raw, buffer

    monkeypatch.setattr(extensions, "_owned_bytes", track_owned_bytes)
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (
            observed.append(bytes(ffi.buffer(args[0].data, args[0].length))),
            42,
        )[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_rng_seed(caller_data)

    assert item == 42
    assert observed == [bytes(caller_data)]
    assert len(staged) == 1
    assert bytes(ffi.buffer(staged[0], len(caller_data))) == bytes(len(caller_data))
    assert caller_data == bytearray(b"KMIP_RNG_SEED_SECRET_SENTINEL")


def test_rng_seed_builder_preserves_present_empty_data(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, args: (calls.append((symbol, args)), 43)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_rng_seed(b"")

    assert item == 43
    assert calls[0][0] == "kmipkit_client_batch_item_rng_seed"
    assert calls[0][1][0].data == ffi.NULL
    assert calls[0][1][0].length == 0


def test_rng_seed_builder_rejects_omitted_required_data_before_native_call(monkeypatch) -> None:
    calls: list[str] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, _args: (calls.append(symbol), 44)[1],
    )

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_rng_seed(None)

    assert calls == []


def test_rng_retrieve_response_redacts_and_wipes_owned_data(monkeypatch) -> None:
    secret = b"KMIP_RNG_RETRIEVE_SECRET_SENTINEL"
    native_ttlv, owned_ttlv = _owned_structure(52)
    backing = ffi.new("uint8_t[]", secret)

    def typed(_item, present, data_present, data):
        present[0] = 1
        data_present[0] = 1
        data.data = backing
        data.length = len(secret)
        return 0

    def get_ttlv(_item, present, output):
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rng_retrieve=typed,
            kmipkit_client_batch_item_response_rng_retrieve_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned_ttlv if address == 52 else None,
    )

    response = client._rng_retrieve_response(_response_item())

    assert response is not None
    assert response.data == secret
    assert secret.decode() not in repr(response)
    assert response.response_ttlv is owned_ttlv
    owned_data = response._data
    response.close()
    assert owned_data is not None
    assert owned_data.buffer == bytearray(len(secret))
    assert owned_ttlv.closed


def test_rng_seed_response_preserves_zero_length_and_ttlv(monkeypatch) -> None:
    native_ttlv, owned_ttlv = _owned_structure(53)

    def typed(_item, present, length_present, data_length):
        present[0] = 1
        length_present[0] = 1
        data_length[0] = 0
        return 0

    def get_ttlv(_item, present, output):
        present[0] = 1
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rng_seed=typed,
            kmipkit_client_batch_item_response_rng_seed_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda address: owned_ttlv if address == 53 else None,
    )

    response = client._rng_seed_response(_response_item())

    assert response is not None
    assert response.data_length == 0
    assert response.response_ttlv is owned_ttlv
    response.close()
    assert owned_ttlv.closed


@pytest.mark.parametrize(
    ("present_value", "data_present_value", "data_value", "length"),
    [
        (2, 0, ffi.NULL, 0),
        (0, 0, ffi.new("uint8_t[]", b"x"), 1),
        (1, 1, ffi.NULL, 1),
        (1, 0, ffi.NULL, 0),
    ],
)
def test_rng_retrieve_rejects_malformed_native_output(
    monkeypatch, present_value, data_present_value, data_value, length
) -> None:
    def typed(_item, present, data_present, data):
        present[0] = present_value
        data_present[0] = data_present_value
        data.data = data_value
        data.length = length
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_rng_retrieve=typed),
    )

    with pytest.raises(errors.InvalidInputError):
        client._rng_retrieve_response(_response_item())


@pytest.mark.parametrize(
    ("present_value", "length_present_value", "data_length_value"),
    [(2, 0, 0), (0, 0, 1), (1, 2, 0)],
)
def test_rng_seed_rejects_malformed_native_output(
    monkeypatch, present_value, length_present_value, data_length_value
) -> None:
    def typed(_item, present, length_present, data_length):
        present[0] = present_value
        length_present[0] = length_present_value
        data_length[0] = data_length_value
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_rng_seed=typed),
    )

    with pytest.raises(errors.InvalidInputError):
        client._rng_seed_response(_response_item())
