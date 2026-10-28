"""Four-language binding expectations for OASIS KMIP 2.1 §6.1.45."""

from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest
from _test_support import mtls_transport_config
from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_UNKNOWN_RESPONSE_TAG = 0x541234


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    output = bytearray()
    while len(output) < length:
        chunk = stream.recv(length - len(output))
        if not chunk:
            raise AssertionError("client closed before sending a complete TTLV frame")
        output.extend(chunk)
    return bytes(output)


def _read_ttlv_frame(stream: ssl.SSLSocket) -> bytes:
    header = _read_exact(stream, 8)
    return header + _read_exact(stream, int.from_bytes(header[4:8], "big"))


def _recertify_response_message() -> bytes:
    protocol_version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    response_header = _ttlv_structure(
        0x42007A,
        protocol_version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    response_payload = _ttlv_structure(
        0x42007C,
        _ttlv_item(0x420094, 7, b"replacement-certificate-7"),
        _ttlv_item(_UNKNOWN_RESPONSE_TAG, 2, struct.pack(">i", -7)),
    )
    response_item = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x07)),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        response_payload,
    )
    return _ttlv_structure(0x42007B, response_header, response_item)


def _structure_view_tags(view: ttlv.TtlvStructureView) -> list[int]:
    tags: list[int] = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        with ttlv.ttlv_structure_view_item_at(view, index) as item:
            with ttlv.ttlv_item_view_tag(item) as tag:
                tags.append(ttlv.tag_value(tag))
    return tags


def test_recertify_builder_maps_table_300_values_and_zeroizes_owned_request_bytes(
    monkeypatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []
    staged_values: list[bytes] = []
    secret = bytearray(b"certificate-request-private-marker")
    original_owned_bytes = extensions._owned_bytes
    request_buffers: list[object] = []

    def track_owned_bytes(value: object, maximum: int = 4_096) -> tuple[bytes, object]:
        raw, staging = original_owned_bytes(value, maximum)  # type: ignore[arg-type]
        if raw == bytes(secret):
            request_buffers.append(staging)
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
    assert len(request_buffers) == 1
    assert bytes(extensions.ffi.buffer(request_buffers[0], len(secret))) == bytes(
        len(secret)
    )
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


def _stub_recertify_response(
    monkeypatch: pytest.MonkeyPatch,
    *,
    present: int,
    identifier_kind: int = 0,
    response_ttlv_present: int = 1,
    response_ttlv_handle: object = ffi.cast("kmipkit_ttlv_structure_t *", 42),
) -> list[object]:
    identifier_text = ffi.new("uint8_t[]", b"replacement")
    pointer = response_ttlv_handle
    released: list[object] = []

    def response(_item: object, out_present: object, identifier: object) -> int:
        out_present[0] = present  # type: ignore[index]
        identifier.kind = identifier_kind  # type: ignore[attr-defined]
        if identifier_kind == int(extensions.UniqueIdentifierKind.TEXT_STRING):
            identifier.text_data = identifier_text  # type: ignore[attr-defined]
            identifier.text_length = len(b"replacement")  # type: ignore[attr-defined]
        return 0

    def response_ttlv(_item: object, out_present: object, output: object) -> int:
        out_present[0] = response_ttlv_present  # type: ignore[index]
        output[0] = pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_recertify=response,
            kmipkit_client_batch_item_response_recertify_ttlv=response_ttlv,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )
    return released


def test_recertify_absent_response_has_zeroed_identifier_and_skips_ttlv(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[str] = []

    def unexpected_ttlv(_item: object, _out_present: object, _output: object) -> int:
        calls.append("ttlv")
        return 0

    def response(_item: object, out_present: object, _identifier: object) -> int:
        calls.append("typed")
        out_present[0] = 0  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_recertify=response,
            kmipkit_client_batch_item_response_recertify_ttlv=unexpected_ttlv,
            kmipkit_ttlv_structure_release=lambda _value: None,
        ),
    )

    assert client._recertify_response(SimpleNamespace(_pointer=lambda: object())) is None
    assert calls == ["typed"]


def test_recertify_absent_response_rejects_unexpected_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    _stub_recertify_response(
        monkeypatch,
        present=0,
        identifier_kind=int(extensions.UniqueIdentifierKind.TEXT_STRING),
        response_ttlv_present=0,
        response_ttlv_handle=ffi.NULL,
    )

    with pytest.raises(errors.InvalidInputError):
        client._recertify_response(SimpleNamespace(_pointer=lambda: object()))


def test_recertify_response_rejects_invalid_presence_and_missing_identifier(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    _stub_recertify_response(monkeypatch, present=2)
    with pytest.raises(errors.InvalidInputError):
        client._recertify_response(SimpleNamespace(_pointer=lambda: object()))

    _stub_recertify_response(monkeypatch, present=1)
    with pytest.raises(errors.InvalidInputError):
        client._recertify_response(SimpleNamespace(_pointer=lambda: object()))


def test_recertify_response_releases_unexpected_ttlv_and_rejects_absence(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 73)
    released = _stub_recertify_response(
        monkeypatch,
        present=1,
        identifier_kind=int(extensions.UniqueIdentifierKind.TEXT_STRING),
        response_ttlv_present=0,
        response_ttlv_handle=pointer,
    )

    with pytest.raises(errors.InvalidInputError):
        client._recertify_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [pointer]


def test_recertify_response_releases_ttlv_when_wrapper_conversion_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pointer = ffi.cast("kmipkit_ttlv_structure_t *", 74)
    released = _stub_recertify_response(
        monkeypatch,
        present=1,
        identifier_kind=int(extensions.UniqueIdentifierKind.TEXT_STRING),
        response_ttlv_present=1,
        response_ttlv_handle=pointer,
    )

    def fail_conversion(_address: int) -> ttlv.Structure:
        raise RuntimeError("test wrapper conversion failure")

    monkeypatch.setattr(client.ttlv, "_structure_from_native_address", fail_conversion)
    with pytest.raises(RuntimeError, match="test wrapper conversion failure"):
        client._recertify_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [pointer]


def test_recertify_builder_maps_omitted_optional_type_and_offset(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, tuple[object, ...]]] = []

    monkeypatch.setattr(
        extensions,
        "_make_handle",
        lambda _kind, symbol, arguments: (calls.append((symbol, arguments)), 41)[1],
    )
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    item = extensions.client_batch_item_recertify()

    assert item == 41
    assert calls[0][0] == "kmipkit_client_batch_item_recertify"
    assert calls[0][1][2:4] == (0, 0)
    assert calls[0][1][5:7] == (0, 0)


def test_recertify_builder_passes_typed_identifier_to_native_client_abi() -> None:
    item = extensions.client_batch_item_recertify(
        extensions.UniqueIdentifier.text_string("certificate-7"),
        certificate_request_type=5,
        certificate_request_value=b"certificate-request",
    )
    item.close()


def test_recertify_round_trips_through_native_client_over_mtls_and_retains_ttlv(
) -> None:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(10)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[BaseException] = queue.Queue()
    received_requests: list[bytes] = []
    accepted_sockets: list[socket.socket] = []

    def serve_recertify_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                _TLS_FIXTURES / "server-cert.pem", _TLS_FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=_TLS_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            raw.settimeout(5)
            accepted_sockets.append(raw)
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(_recertify_response_message())
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_recertify_response, daemon=True)
    peer.start()
    transport = mtls_transport_config(port, _TLS_FIXTURES)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    response = None
    response_ttlv = None
    try:
        with configuration, client.Client.create(configuration, transport) as kmip_client:
            with ttlv.codec_limits_defaults() as limits, client.Batch.create() as batch:
                batch.add_item(
                    extensions.client_batch_item_recertify(
                        extensions.UniqueIdentifier.text_string("certificate-7"),
                        certificate_request_type=5,
                        certificate_request_value=b"test-certificate-request",
                    )
                )
                result = kmip_client.execute(batch, limits)

        assert peer_failures.empty(), "the loopback mTLS peer must complete its response"
        assert len(received_requests) == 1
        assert _ttlv_item(0x42005C, 5, struct.pack(">I", 0x07)) in received_requests[0]
        assert _ttlv_item(0x420140, 8, b"test-certificate-request") in received_requests[0]
        assert result is not None and len(result.items) == 1
        item = result.items[0]
        assert item.operation == 0x07
        assert item.outcome is client.BatchOutcome.COMPLETED
        assert item.result_status == 0
        response = item.recertify_response
        assert response is not None
        assert response.unique_identifier == extensions.UniqueIdentifier.text_string(
            "replacement-certificate-7"
        )
        response_ttlv = response.response_ttlv

        # Both the request batch and native result/item handles are closed before
        # reading the independently owned generic response copy.
        result.close()
        result = None
        assert not response_ttlv.closed
        with ttlv.ttlv_structure_view(response_ttlv) as response_view:
            assert _structure_view_tags(response_view) == [0x42005C, 0x42007F, 0x42007C]
            with ttlv.ttlv_structure_view_item_at(response_view, 2) as payload_item:
                with ttlv.ttlv_item_view_value(payload_item) as payload_value:
                    with ttlv.ttlv_value_view_structure(payload_value) as payload_view:
                        assert _structure_view_tags(payload_view) == [
                            0x420094,
                            _UNKNOWN_RESPONSE_TAG,
                        ]
                        with ttlv.ttlv_structure_view_item_at(payload_view, 1) as unknown:
                            with ttlv.ttlv_item_view_value(unknown) as value:
                                assert ttlv.ttlv_value_view_integer(value) == -7
    finally:
        listener.close()
        for accepted_socket in accepted_sockets:
            accepted_socket.close()
        if response is not None:
            response.close()
        if result is not None:
            result.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback Re-certify peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()
