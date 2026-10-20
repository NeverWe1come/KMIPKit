"""Candidate-only CFFI smoke tests; these bindings are not shipped in the wheel."""

from __future__ import annotations

import json
import os
import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

import pytest

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._client_ffi import lib as client_lib
from kmipkit._ffi import ffi
from kmipkit._handles import _wrap_owned_handle


ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"
TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
CANDIDATE_RUNTIME = os.environ.get("KMIPKIT_PKCS11_CANDIDATE_TEST_RUNTIME") == "1"

client_ffi.cdef(
    """
    int32_t kmipkit_client_batch_item_pkcs11(
        kmipkit_ttlv_structure_t *function,
        kmipkit_ttlv_structure_t *interface,
        const kmipkit_byte_span_v1_t *correlation_value,
        kmipkit_ttlv_structure_t *input_parameters,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_response_pkcs11_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    """
)


def _client_structure(value: ttlv.Structure | None) -> object:
    if value is None:
        return client_ffi.NULL
    pointer = extensions._native_structure(value, required=True)
    address = int(ffi.cast("uintptr_t", pointer))
    return client_ffi.cast("kmipkit_ttlv_structure_t *", address)


def _ttlv_item(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def _ttlv_structure(tag: int, *children: bytes) -> bytes:
    return _ttlv_item(tag, 1, b"".join(children))


def _parse_ttlv_items(data: bytes) -> list[tuple[int, int, bytes, list[tuple]]]:
    items: list[tuple[int, int, bytes, list[tuple]]] = []
    offset = 0
    while offset < len(data):
        assert len(data) - offset >= 8, "captured TTLV item has a truncated header"
        tag = int.from_bytes(data[offset : offset + 3], "big")
        item_type = data[offset + 3]
        length = int.from_bytes(data[offset + 4 : offset + 8], "big")
        value_start = offset + 8
        value_end = value_start + length
        padded_end = value_end + (-length) % 8
        assert padded_end <= len(data), "captured TTLV item is truncated"
        value = data[value_start:value_end]
        children = _parse_ttlv_items(value) if item_type == 1 else []
        items.append((tag, item_type, value, children))
        offset = padded_end
    assert offset == len(data)
    return items


def _find_ttlv_node(
    nodes: list[tuple[int, int, bytes, list[tuple]]], tag: int
) -> tuple[int, int, bytes, list[tuple]]:
    matches = [node for node in nodes if node[0] == tag]
    assert len(matches) == 1, f"expected exactly one TTLV tag 0x{tag:06X}"
    return matches[0]


def _structure_with_bytes(tag_value: int, value_bytes: bytes) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(tag_value)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_byte_string(value_bytes, limits)
    item = ttlv.ttlv_item_create(tag, value, limits)
    structure = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), item, limits)
    tag.close()
    raw_tag.close()
    limits.close()
    return structure


def _structure_with_text(tag_value: int, value_text: str) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(tag_value)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_text_string(value_text.encode("utf-8"), limits)
    item = ttlv.ttlv_item_create(tag, value, limits)
    structure = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), item, limits)
    tag.close()
    raw_tag.close()
    limits.close()
    return structure


def _pkcs11_response_message() -> bytes:
    version = _ttlv_structure(
        0x420069,
        _ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        _ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _ttlv_structure(
        0x42007A,
        version,
        _ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        _ttlv_item(0x42000D, 2, struct.pack(">i", 1)),
    )
    payload = _ttlv_structure(
        0x42007C,
        _ttlv_structure(0x42015A, _ttlv_item(0x42015A, 2, struct.pack(">i", -12345))),
        _ttlv_item(0x4200D6, 8, b"pkcs11-response-correlation"),
        _ttlv_structure(0x42015C, _ttlv_item(0x42015C, 8, b"vendor-output")),
        _ttlv_structure(0x42015D, _ttlv_item(0x42015D, 5, struct.pack(">I", 0xFFFF1234))),
        _ttlv_item(0x00541234, 2, struct.pack(">i", 7)),
    )
    result = _ttlv_structure(
        0x42000F,
        _ttlv_item(0x42005C, 5, struct.pack(">I", 0x33)),
        _ttlv_item(0x42007F, 5, struct.pack(">I", 0)),
        payload,
    )
    return _ttlv_structure(0x42007B, header, result)


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


def _structure_tags(value: ttlv.Structure) -> list[int]:
    with ttlv.ttlv_structure_view(value) as view:
        return _structure_view_tags(view)


def _structure_view_tags(view: ttlv.TtlvStructureView) -> list[int]:
    tags: list[int] = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        with ttlv.ttlv_structure_view_item_at(view, index) as item:
            with ttlv.ttlv_item_view_tag(item) as tag:
                tags.append(ttlv.tag_value(tag))
    return tags


def _structure_with_integer(tag_value: int, integer_value: int) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(tag_value)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    value = ttlv.ttlv_value_integer(integer_value)
    item = ttlv.ttlv_item_create(tag, value, limits)
    structure = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), item, limits)
    tag.close()
    raw_tag.close()
    limits.close()
    return structure


def test_pkcs11_facade_and_cffi_symbols_are_public_by_default() -> None:
    assert callable(extensions.client_batch_item_pkcs11)
    assert hasattr(client.BatchItemResult, "pkcs11_response")
    assert hasattr(client, "Pkcs11Response")
    assert hasattr(client_lib, "kmipkit_client_batch_item_pkcs11")
    assert hasattr(client_lib, "kmipkit_client_batch_item_response_pkcs11_ttlv")


def test_pkcs11_candidate_symbol_is_present_only_in_the_candidate_runtime() -> None:
    assert hasattr(client_lib, "kmipkit_client_batch_item_pkcs11") is CANDIDATE_RUNTIME


@pytest.mark.skipif(
    not CANDIDATE_RUNTIME,
    reason="owner-gated PKCS#11 symbols exist only in the isolated candidate runtime",
)
def test_pkcs11_cffi_candidate_constructor_accepts_optional_structures_and_bytes() -> None:
    function = ttlv.ttlv_structure_create()
    interface = ttlv.ttlv_structure_create()
    input_parameters = ttlv.ttlv_structure_create()
    correlation = b"pkcs11-correlation-private-sentinel"
    staged = client_ffi.new("uint8_t[]", correlation)
    correlation_span = client_ffi.new("kmipkit_byte_span_v1_t *")
    correlation_span.data = staged
    correlation_span.length = len(correlation)
    item = client_ffi.new("kmipkit_client_batch_item_t **")
    try:
        status = client_lib.kmipkit_client_batch_item_pkcs11(
            _client_structure(function),
            _client_structure(interface),
            correlation_span,
            _client_structure(input_parameters),
            item,
        )
        assert status == 0
        assert item[0] != client_ffi.NULL
        client_lib.kmipkit_client_batch_item_release(item[0])
        item[0] = client_ffi.NULL
    finally:
        client_ffi.buffer(staged, len(correlation))[:] = bytes(len(correlation))
        if item[0] != client_ffi.NULL:
            client_lib.kmipkit_client_batch_item_release(item[0])
        function.close()
        interface.close()
        input_parameters.close()


@pytest.mark.skipif(
    not CANDIDATE_RUNTIME,
    reason="owner-gated PKCS#11 symbols exist only in the isolated candidate runtime",
)
def test_pkcs11_cffi_response_accessor_clears_outputs_for_invalid_handle() -> None:
    present = client_ffi.new("uint8_t *", 0xFF)
    response = client_ffi.new(
        "kmipkit_ttlv_structure_t **",
        client_ffi.cast("kmipkit_ttlv_structure_t *", 1),
    )
    status = client_lib.kmipkit_client_batch_item_response_pkcs11_ttlv(
        client_ffi.NULL, present, response
    )
    assert status == 4
    assert present[0] == 0
    assert response[0] == client_ffi.NULL


def test_pkcs11_shared_fixture_declares_the_canonical_contract() -> None:
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert fixture["capability"] == "KMIPKIT-ELEM-OP-C2S-PKCS-11"
    assert fixture["normativeSource"] == (
        "OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275"
    )
    assert set(fixture["consumerTests"]) == {"rust", "c", "java", "python"}
    assert fixture["expectedOutput"]["automaticRetry"] is False
    assert fixture["expectedOutput"]["automaticFollowUp"] is False
    assert "profileSupport" not in fixture["expectedOutput"]


@pytest.mark.skipif(
    not CANDIDATE_RUNTIME,
    reason="owner-gated PKCS#11 symbols exist only in the isolated candidate runtime",
)
def test_pkcs11_candidate_success_path_preserves_order_and_unknown_values() -> None:
    """Check parity with OASIS KMIP v2.1 §6.1.37, Tables 273–275."""
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    expected = fixture["expectedOutput"]
    function = _structure_with_integer(0x42015A, -12345)
    interface = _structure_with_text(0x420159, "test-interface")
    input_parameters = _structure_with_bytes(0x42015B, b"opaque-input")
    correlation = b"pkcs11-correlation"
    staged = client_ffi.new("uint8_t[]", correlation)
    correlation_span = client_ffi.new("kmipkit_byte_span_v1_t *")
    correlation_span.data = staged
    correlation_span.length = len(correlation)
    item_output = client_ffi.new("kmipkit_client_batch_item_t **")
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(5)
    port = listener.getsockname()[1]
    peer_failures: queue.Queue[BaseException] = queue.Queue()
    received_requests: list[bytes] = []
    accepted_sockets: list[socket.socket] = []

    def serve_one_pkcs11_response() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(
                TLS_FIXTURES / "server-cert.pem", TLS_FIXTURES / "server-key.pem"
            )
            context.load_verify_locations(cafile=TLS_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            raw, _address = listener.accept()
            raw.settimeout(5)
            accepted_sockets.append(raw)
            with raw, context.wrap_socket(raw, server_side=True) as stream:
                received_requests.append(_read_ttlv_frame(stream))
                stream.sendall(_pkcs11_response_message())
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_one_pkcs11_response, daemon=True)
    peer.start()
    client_certificate = client.TransportInput.file(
        TLS_FIXTURES / "client-cert.pem", client.CredentialEncoding.PEM
    )
    client_key = client.TransportInput.file(
        TLS_FIXTURES / "client-key.pem", client.CredentialEncoding.PEM
    )
    server_certificate = client.TransportInput.file(
        TLS_FIXTURES / "server-cert.pem", client.CredentialEncoding.PEM
    )
    transport = client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("127.0.0.1", port),
        client_certificates=(client_certificate,),
        client_private_key=client_key,
        trust=client.TrustSource.certificate_authorities((server_certificate,)),
        connect_timeout=client.TimeoutLimit.bounded(5),
        write_timeout=client.TimeoutLimit.bounded(5),
        read_timeout=client.TimeoutLimit.bounded(5),
        total_timeout=client.TimeoutLimit.bounded(8),
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    response_item = None
    response_ttlv = None
    try:
        assert client_lib.kmipkit_client_batch_item_pkcs11(
            _client_structure(function),
            _client_structure(interface),
            correlation_span,
            _client_structure(input_parameters),
            item_output,
        ) == 0
        assert item_output[0] != client_ffi.NULL
        native_request = _wrap_owned_handle(
            extensions.ClientBatchItem,
            ffi.cast(
                "kmipkit_client_batch_item_t *",
                int(client_ffi.cast("uintptr_t", item_output[0])),
            ),
        )
        item_output[0] = client_ffi.NULL
        with configuration, client.Client.create(configuration, transport) as kmip_client:
            with ttlv.codec_limits_defaults() as limits, client.Batch.create() as batch:
                batch.add_item(native_request)
                result = kmip_client.execute(batch, limits)

        assert peer_failures.empty(), "the loopback mTLS peer must complete the response"
        assert len(received_requests) == 1
        request = _parse_ttlv_items(received_requests[0])
        message = _find_ttlv_node(request, 0x420078)
        batch_item = _find_ttlv_node(message[3], 0x42000F)
        payload = _find_ttlv_node(batch_item[3], 0x420079)
        assert [item[0] for item in payload[3]] == [
            0x420159,
            0x42015A,
            0x4200D6,
            0x42015B,
        ]
        interface_node = _find_ttlv_node(payload[3], 0x420159)
        assert interface_node[1] == 1
        assert interface_node[3] == [(0x420159, 7, b"test-interface", [])]
        function_node = _find_ttlv_node(payload[3], 0x42015A)
        assert function_node[1] == 1
        assert function_node[3] == [(0x42015A, 2, struct.pack(">i", -12345), [])]
        correlation_node = _find_ttlv_node(payload[3], 0x4200D6)
        assert correlation_node[1:3] == (8, correlation)
        input_node = _find_ttlv_node(payload[3], 0x42015B)
        assert input_node[1] == 1
        assert input_node[3] == [(0x42015B, 8, b"opaque-input", [])]
        assert (
            b"\x42\x00\x5c\x05\x00\x00\x00\x04\x00\x00\x00\x33"
            in received_requests[0]
        )
        assert result is not None and len(result.items) == 1
        assert result.items[0].operation == 0x33
        assert result.items[0].result_status == 0

        batch_response = client_ffi.cast(
            "kmipkit_client_batch_response_t *",
            int(ffi.cast("uintptr_t", result._pointer())),
        )
        item_output_for_response = client_ffi.new(
            "kmipkit_client_batch_item_response_t **"
        )
        assert client_lib.kmipkit_client_batch_response_item_at(
            batch_response, 0, item_output_for_response
        ) == 0
        assert item_output_for_response[0] != client_ffi.NULL
        response_item = _wrap_owned_handle(
            client._ClientBatchItemResponse,
            client_ffi.cast(
                "kmipkit_client_batch_item_response_t *",
                int(client_ffi.cast("uintptr_t", item_output_for_response[0])),
            ),
        )
        present = client_ffi.new("uint8_t *")
        response_ttlv_output = client_ffi.new("kmipkit_ttlv_structure_t **")
        assert client_lib.kmipkit_client_batch_item_response_pkcs11_ttlv(
            client_ffi.cast(
                "kmipkit_client_batch_item_response_t *",
                int(ffi.cast("uintptr_t", response_item._pointer())),
            ),
            present,
            response_ttlv_output,
        ) == 0
        assert present[0] == 1
        assert response_ttlv_output[0] != client_ffi.NULL
        response_ttlv = ttlv._structure_from_native_address(
            int(client_ffi.cast("uintptr_t", response_ttlv_output[0]))
        )

        with ttlv.ttlv_structure_view(response_ttlv) as response_view:
            payload_view = None
            for index in range(ttlv.ttlv_structure_view_item_count(response_view)):
                with ttlv.ttlv_structure_view_item_at(response_view, index) as item_view:
                    with ttlv.ttlv_item_view_tag(item_view) as tag:
                        if ttlv.tag_value(tag) != 0x42007C:
                            continue
                    with ttlv.ttlv_item_view_value(item_view) as value_view:
                        payload_view = ttlv.ttlv_value_view_structure(value_view)
                        break
            assert payload_view is not None
            with payload_view:
                response_tags = _structure_view_tags(payload_view)
                assert response_tags == [
                    int(tag, 16) for tag in expected["responseTags"]
                ]
                if expected["preserveOptionalAbsence"]:
                    assert 0x420159 not in response_tags
                    assert 0x42015B not in response_tags

                return_code_index = response_tags.index(0x42015D)
                with ttlv.ttlv_structure_view_item_at(
                    payload_view, return_code_index
                ) as return_code_item:
                    with ttlv.ttlv_item_view_value(return_code_item) as return_code_value:
                        with ttlv.ttlv_value_view_structure(
                            return_code_value
                        ) as return_code_view:
                            assert _structure_view_tags(return_code_view) == [0x42015D]
                            with ttlv.ttlv_structure_view_item_at(
                                return_code_view, 0
                            ) as raw_return_code_item:
                                with ttlv.ttlv_item_view_value(
                                    raw_return_code_item
                                ) as raw_return_code_value:
                                    assert ttlv.ttlv_value_view_enumeration(
                                        raw_return_code_value
                                    ) == int(expected["unknownReturnCode"], 16)

                vendor_tag = int(expected["vendorExtension"]["tag"], 16)
                vendor_index = response_tags.index(vendor_tag)
                with ttlv.ttlv_structure_view_item_at(
                    payload_view, vendor_index
                ) as vendor_item:
                    with ttlv.ttlv_item_view_value(vendor_item) as vendor_value:
                        assert ttlv.ttlv_value_view_integer(vendor_value) == expected[
                            "vendorExtension"
                        ]["integer"]
    finally:
        listener.close()
        for accepted_socket in accepted_sockets:
            accepted_socket.close()
        if response_ttlv is not None:
            response_ttlv.close()
        if response_item is not None:
            response_item.close()
        if result is not None:
            result.close()
        if item_output[0] != client_ffi.NULL:
            client_lib.kmipkit_client_batch_item_release(item_output[0])
        client_ffi.buffer(staged, len(correlation))[:] = bytes(len(correlation))
        function.close()
        interface.close()
        input_parameters.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback PKCS#11 mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()
