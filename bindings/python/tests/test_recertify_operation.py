"""Four-language binding expectations for OASIS KMIP 2.1 §6.1.45."""

from __future__ import annotations

from types import SimpleNamespace

from kmipkit import client, extensions
from kmipkit._client_ffi import ffi


def test_recertify_builder_maps_table_300_values_and_zeroizes_owned_request_bytes(
    monkeypatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    staged_values: list[bytes] = []
    secret = bytearray(b"certificate-request-private-marker")
    original_owned_bytes = extensions._owned_bytes
    staged_buffers: list[object] = []

    def track_owned_bytes(value: object, maximum: int = 4_096) -> tuple[bytes, object]:
        raw, staging = original_owned_bytes(value, maximum)  # type: ignore[arg-type]
        staged_buffers.append(staging)
        return raw, staging

    def make_handle(_kind: str, symbol: str, arguments: tuple[object, ...]) -> int:
        calls.append((symbol, arguments))
        span = arguments[4]
        staged_values.append(bytes(ffi.buffer(span.data, span.length)))  # type: ignore[attr-defined]
        return 41

    monkeypatch.setattr(extensions, "_owned_bytes", track_owned_bytes)
    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_recertify(
        extensions.UniqueIdentifier.text_string("certificate-7"),
        extensions.UniqueIdentifier.text_string("request-7"),
        certificate_request_type=0xFEDC_BA98,
        certificate_request_value=secret,
        offset=0xFFFF_FFFF,
    )

    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_recertify"
    assert calls[0][1][2:4] == (1, 0xFEDC_BA98)
    assert calls[0][1][5:7] == (1, 0xFFFF_FFFF)
    assert calls[0][1][7:] == (ffi.NULL, ffi.NULL)
    assert staged_values == [bytes(secret)]
    assert len(staged_buffers) == 1
    assert bytes(ffi.buffer(staged_buffers[0], len(secret))) == bytes(len(secret))  # type: ignore[arg-type]
    assert secret == bytearray(b"certificate-request-private-marker")


def test_recertify_response_maps_replacement_identifier_and_closes_owned_ttlv(
    monkeypatch,
) -> None:
    response_pointer = ffi.cast("kmipkit_ttlv_structure_t *", 42)
    identifier_storage = ffi.new("uint8_t[]", b"replacement-certificate")
    closed: list[bool] = []

    class OwnedStructure:
        def close(self) -> None:
            closed.append(True)

    response_ttlv = OwnedStructure()

    def response(_item: object, present: object, identifier: object) -> int:
        present[0] = 1  # type: ignore[index]
        identifier.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
        identifier.text_data = identifier_storage  # type: ignore[attr-defined]
        identifier.text_length = len(b"replacement-certificate")  # type: ignore[attr-defined]
        return 0

    def response_ttlv_getter(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_recertify=response,
            kmipkit_client_batch_item_response_recertify_ttlv=response_ttlv_getter,
            kmipkit_ttlv_structure_release=lambda _pointer: None,
        ),
    )
    monkeypatch.setattr(
        client.ttlv,
        "_structure_from_native_address",
        lambda address: response_ttlv if address == 42 else None,
    )

    result = client._recertify_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client.RecertifyResponse)
    assert result.unique_identifier == extensions.UniqueIdentifier.text_string(
        "replacement-certificate"
    )
    assert result.response_ttlv is response_ttlv
    assert "replacement-certificate" not in repr(result)
    result.close()
    assert closed == [True]
