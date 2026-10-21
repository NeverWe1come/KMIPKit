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
    with pytest.raises(RuntimeError, match="RNG Retrieve response is closed"):
        _ = response.data


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
    assert repr(response) == "RngSeedResponse(data_length=0, response_ttlv=preserved)"
    response.close()
    assert owned_ttlv.closed


def test_rng_retrieve_response_rejects_missing_owned_data_while_open() -> None:
    _, owned_ttlv = _owned_structure(54)
    response = client.RngRetrieveResponse(None, owned_ttlv)  # type: ignore[arg-type]

    with pytest.raises(errors.InvalidInputError):
        _ = response.data

    response.close()


def test_rng_retrieve_absence_requires_an_empty_typed_output(monkeypatch) -> None:
    def absent(_item, present, data_present, data):
        present[0] = 0
        data_present[0] = 0
        data.data = ffi.NULL
        data.length = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_rng_retrieve=absent),
    )

    assert client._rng_retrieve_response(_response_item()) is None

    def inconsistent_absence(_item, present, data_present, data):
        present[0] = 0
        data_present[0] = 1
        data.data = ffi.NULL
        data.length = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_rng_retrieve=inconsistent_absence),
    )
    with pytest.raises(errors.InvalidInputError):
        client._rng_retrieve_response(_response_item())


def test_rng_retrieve_rejects_missing_secret_after_present_native_flags(monkeypatch) -> None:
    def typed(_item, present, data_present, data):
        present[0] = 1
        data_present[0] = 1
        data.data = ffi.NULL
        data.length = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_rng_retrieve=typed),
    )
    monkeypatch.setattr(client, "_optional_output_secret_span", lambda _p, _v: None)

    with pytest.raises(errors.InvalidInputError):
        client._rng_retrieve_response(_response_item())


def test_successful_response_ttlv_releases_inconsistent_native_handle(monkeypatch) -> None:
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 54)
    released: list[object] = []

    def inconsistent(_item, present, output):
        present[0] = 0
        output[0] = native_ttlv
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_ttlv_structure_release=lambda value: released.append(value)),
    )

    with pytest.raises(errors.InvalidInputError):
        client._successful_response_ttlv(
            _response_item(),
            inconsistent,
        )

    assert released == [native_ttlv]


def test_rng_retrieve_wipes_secret_when_native_ttlv_conversion_fails(monkeypatch) -> None:
    secret = b"KMIP_RNG_RETRIEVE_FAILED_TTLV_SENTINEL"
    native_ttlv = ffi.cast("kmipkit_ttlv_structure_t *", 55)
    backing = ffi.new("uint8_t[]", secret)
    released: list[object] = []
    captured: list[object] = []
    original_secret_span = client._optional_output_secret_span

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

    def capture_secret(present, value):
        result = original_secret_span(present, value)
        captured.append(result)
        return result

    def fail_conversion(_address):
        raise RuntimeError("test TTLV conversion failure")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_rng_retrieve=typed,
            kmipkit_client_batch_item_response_rng_retrieve_ttlv=get_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    monkeypatch.setattr(client, "_optional_output_secret_span", capture_secret)
    monkeypatch.setattr(ttlv, "_structure_from_native_address", fail_conversion)

    with pytest.raises(RuntimeError, match="test TTLV conversion failure"):
        client._rng_retrieve_response(_response_item())

    assert released == [native_ttlv]
    assert len(captured) == 1
    assert captured[0] is not None
    assert captured[0].buffer == bytearray(len(secret))


def test_native_optional_client_span_preserves_absence() -> None:
    span, staging = extensions._native_client_optional_span(None)

    assert span == extensions._client_ffi.NULL
    assert staging is None


def test_rng_seed_rejects_an_absent_native_span_before_builder_call(monkeypatch) -> None:
    calls: list[str] = []
    monkeypatch.setattr(
        extensions,
        "_native_client_optional_span",
        lambda _value, _staging: (extensions._client_ffi.NULL, None),
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, _args: (calls.append(symbol), 45)[1],
    )

    with pytest.raises(errors.InvalidInputError):
        extensions.client_batch_item_rng_seed(b"seed")

    assert calls == []


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
