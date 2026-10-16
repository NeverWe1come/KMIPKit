from __future__ import annotations

from types import SimpleNamespace

from kmipkit import client, extensions
from kmipkit._client_ffi import ffi


def test_certify_builder_preserves_open_request_type_and_sensitive_bytes(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []

    def make_handle(handle_type: str, symbol: str, arguments: tuple[object, ...]) -> int:
        calls.append((symbol, arguments))
        return 23

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    secret = b"certificate-request-marker"

    handle = extensions.client_batch_item_certify(
        extensions.UniqueIdentifier.text_string("public-key-7"),
        certificate_request_type=0xFEDC_BA98,
        certificate_request_value=secret,
    )

    assert handle == 23
    assert calls[0][0] == "kmipkit_client_batch_item_certify"
    assert calls[0][1][1:3] == (1, 0xFEDC_BA98)
    assert bytes(ffi.buffer(calls[0][1][3].data, len(secret))) == secret


def test_certify_response_maps_generated_identifier_and_keeps_it_redacted(monkeypatch) -> None:
    def response_certify(_item, present, identifier) -> int:
        present[0] = 1
        identifier.kind = 3
        identifier.integer_value = -17
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_certify=response_certify),
    )
    item = SimpleNamespace(_pointer=lambda: object())

    response = client._certify_response(item)

    assert isinstance(response, client.CertifyResponse)
    assert response.unique_identifier == extensions.UniqueIdentifier.integer(-17)
    assert "-17" not in repr(response)
