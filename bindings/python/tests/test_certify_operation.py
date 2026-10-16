from __future__ import annotations

from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions
from kmipkit._client_ffi import ffi


def test_certify_builder_preserves_open_request_type_and_sensitive_bytes(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    copied_request_values: list[bytes] = []
    secret = b"certificate-request-marker"

    def make_handle(handle_type: str, symbol: str, arguments: tuple[object, ...]) -> int:
        calls.append((symbol, arguments))
        span = arguments[3]
        copied_request_values.append(bytes(ffi.buffer(span.data, len(secret))))
        return 23

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)
    handle = extensions.client_batch_item_certify(
        extensions.UniqueIdentifier.text_string("public-key-7"),
        certificate_request_type=0xFEDC_BA98,
        certificate_request_value=secret,
    )

    assert handle == 23
    assert calls[0][0] == "kmipkit_client_batch_item_certify"
    assert calls[0][1][1:3] == (1, 0xFEDC_BA98)
    assert copied_request_values == [secret]


@pytest.mark.parametrize(("status", "fails"), ((0, False), (4, True)))
def test_certify_native_staging_bytes_are_zeroized_after_abi_call(
    monkeypatch: pytest.MonkeyPatch, status: int, fails: bool
) -> None:
    caller_input = bytearray(b"certificate-request-secret")
    staged_buffers: list[object] = []
    native_calls: list[bytes] = []
    original_owned_bytes = extensions._owned_bytes

    def track_staging_buffer(
        value: object, maximum: int = 4_096
    ) -> tuple[bytes, object]:
        raw, staging = original_owned_bytes(value, maximum)  # type: ignore[arg-type]
        staged_buffers.append(staging)
        return raw, staging

    handle_storage = extensions.ffi.new("uint8_t[]", 1)
    handle_pointer = extensions.ffi.cast(
        "kmipkit_client_batch_item_t *", handle_storage
    )

    def certify(
        _identifier: object,
        _has_request_type: int,
        _request_type: int,
        request_value: object,
        _attributes: object,
        _protection_masks: object,
        output: object,
    ) -> int:
        native_calls.append(
            bytes(
                extensions.ffi.buffer(
                    request_value.data, request_value.length  # type: ignore[attr-defined]
                )
            )
        )
        if status == 0:
            output[0] = handle_pointer  # type: ignore[index]
        return status

    monkeypatch.setattr(extensions, "_owned_bytes", track_staging_buffer)
    monkeypatch.setattr(
        extensions,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_certify=certify),
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    if fails:
        with pytest.raises(errors.InvalidInputError):
            extensions.client_batch_item_certify(
                certificate_request_value=caller_input
            )
    else:
        assert extensions.client_batch_item_certify(
            certificate_request_value=caller_input
        ) == handle_pointer

    assert native_calls == [b"certificate-request-secret"]
    assert len(staged_buffers) == 1
    zeroed = bytes(len(caller_input))
    assert bytes(
        extensions.ffi.buffer(staged_buffers[0], len(caller_input))  # type: ignore[arg-type]
    ) == zeroed
    assert caller_input == bytearray(b"certificate-request-secret")


def test_certify_builder_allows_identifier_only_request(monkeypatch) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: calls.append((symbol, arguments)) or 23,
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    handle = extensions.client_batch_item_certify(extensions.UniqueIdentifier.integer(-17))

    assert handle == 23
    assert calls[0][0] == "kmipkit_client_batch_item_certify"
    assert calls[0][1][1:3] == (0, 0)
    assert calls[0][1][3] == ffi.NULL


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


def test_certify_response_preserves_absence_without_stale_identifier(monkeypatch) -> None:
    def response_certify(_item, present, identifier) -> int:
        present[0] = 0
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_client_batch_item_response_certify=response_certify),
    )
    item = SimpleNamespace(_pointer=lambda: object())

    assert client._certify_response(item) is None


def test_batch_result_exposes_successful_certify_response(monkeypatch) -> None:
    response_pointer = ffi.cast("kmipkit_client_batch_response_t *", 1)
    item_pointer = ffi.cast("kmipkit_client_batch_item_response_t *", 2)
    released: list[str] = []

    def metadata(_item, output) -> int:
        output.struct_size = ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")
        output.abi_version = 2
        output.reserved = 0
        output.operation = 0x06
        output.outcome_kind = int(client.BatchOutcome.COMPLETED)
        output.result_status = 0
        output.has_result_reason = 0
        output.result_reason = 0
        output.has_result_message = 0
        output.result_message_data = ffi.NULL
        output.result_message_length = 0
        return 0

    def certify_response(_item, present, identifier) -> int:
        present[0] = 1
        identifier.kind = int(extensions.UniqueIdentifierKind.INTEGER)
        identifier.integer_value = -17
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, output: output.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda _item, present, span: (
            present.__setitem__(0, 0) or setattr(span, "data", ffi.NULL) or setattr(span, "length", 0) or 0
        ),
        kmipkit_client_batch_item_response_pending_correlation_value=lambda _item, present, span: (
            present.__setitem__(0, 0) or setattr(span, "data", ffi.NULL) or setattr(span, "length", 0) or 0
        ),
        kmipkit_client_batch_item_response_discover_versions_supported_versions=lambda _item, present, _values, _capacity, count: (
            present.__setitem__(0, 0) or count.__setitem__(0, 0) or 0
        ),
        kmipkit_client_batch_item_response_certify=certify_response,
        **{
            "kmipkit_client_batch_item_response_create": lambda _item, present, object_type, _identifier: (
                present.__setitem__(0, 0) or object_type.__setitem__(0, 0) or 0
            ),
            "kmipkit_client_batch_item_response_create_key_pair": lambda _item, present, _private, _public: (
                present.__setitem__(0, 0) or 0
            ),
            "kmipkit_client_batch_item_response_create_split_key_count": lambda _item, present, count: (
                present.__setitem__(0, 0) or count.__setitem__(0, 0) or 0
            ),
        },
    )
    monkeypatch.setattr(client, "lib", fake)

    result = client._batch_result(response_pointer)

    response = result.items[0].certify_response
    assert isinstance(response, client.CertifyResponse)
    assert response.unique_identifier == extensions.UniqueIdentifier.integer(-17)
    assert "-17" not in repr(result.items[0])
    result.close()
    assert released == ["item", "response"]
