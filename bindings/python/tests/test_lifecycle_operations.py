"""Four-language facade checks for KMIP 2.1 lifecycle operations."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, extensions
from kmipkit._client_ffi import ffi


@pytest.mark.parametrize(
    "factory",
    [
        extensions.client_batch_item_activate,
        extensions.client_batch_item_archive,
        extensions.client_batch_item_destroy,
        extensions.client_batch_item_recover,
    ],
)
def test_lifecycle_factory_creates_and_releases_native_item(factory) -> None:
    identifier = extensions.UniqueIdentifier.text_string("lifecycle-object")
    with factory(identifier) as item:
        assert item is not None


@pytest.mark.parametrize(
    ("operation", "response_type"),
    [
        ("activate", client.ActivateResponse),
        ("archive", client.ArchiveResponse),
        ("destroy", client.DestroyResponse),
        ("recover", client.RecoverResponse),
    ],
)
def test_lifecycle_success_response_copies_uid_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch, operation: str, response_type: type,
) -> None:
    identifier = b"lifecycle-response-secret"
    identifier_buffer = ffi.new("uint8_t[]", identifier)

    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 1
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        output.text_data = identifier_buffer
        output.text_length = len(identifier)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(**{f"kmipkit_client_batch_item_response_{operation}": typed}),
    )
    item = SimpleNamespace(_pointer=lambda: object())
    response = client._lifecycle_response(item, operation, response_type)

    assert response is not None
    assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
        "lifecycle-response-secret"
    )
    assert "lifecycle-response-secret" not in repr(response)


@pytest.mark.parametrize("operation", ["activate", "archive", "destroy", "recover"])
def test_lifecycle_non_success_response_has_no_typed_uid(
    monkeypatch: pytest.MonkeyPatch, operation: str,
) -> None:
    def typed(_item: object, present: object, output: object) -> int:
        present[0] = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(**{f"kmipkit_client_batch_item_response_{operation}": typed}),
    )
    item = SimpleNamespace(_pointer=lambda: object())

    assert client._lifecycle_response(item, operation, client.ActivateResponse) is None
