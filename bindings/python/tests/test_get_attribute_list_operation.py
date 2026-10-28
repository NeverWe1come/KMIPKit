from __future__ import annotations

from types import SimpleNamespace

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi


def test_get_attribute_list_builder_preserves_optional_unique_identifier(monkeypatch):
    calls: list[tuple[str, tuple[object, ...]]] = []
    identifier_handle = object()
    monkeypatch.setattr(
        extensions,
        "_native_unique_identifier",
        lambda value, _staging: (ffi.NULL if value is None else identifier_handle, None),
    )
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 73,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    result = extensions.client_batch_item_get_attribute_list(None)

    assert result == 73
    assert calls == [
        ("kmipkit_client_batch_item_get_attribute_list", (ffi.NULL,)),
    ]


def test_get_attribute_list_response_owns_generic_payload_and_redacts_repr(monkeypatch):
    native_handle = ffi.cast("kmipkit_ttlv_structure_t *", 123)

    class OwnedStructure:
        closed = False

        def close(self) -> None:
            self.closed = True

    structure = OwnedStructure()

    def response_ttlv(_item, present, output):
        present[0] = 1
        output[0] = native_handle
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv=response_ttlv
        ),
    )
    monkeypatch.setattr(
        ttlv, "_structure_from_native_address", lambda _address: structure
    )

    result = client._get_attribute_list_response(
        SimpleNamespace(_pointer=lambda: object())
    )

    assert isinstance(result, client.GetAttributeListResponse)
    assert result.response_payload_ttlv is structure
    assert "123" not in repr(result)
    result.close()
    assert structure.closed
