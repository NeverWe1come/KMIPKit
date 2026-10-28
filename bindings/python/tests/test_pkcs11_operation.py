"""Public Python API and CFFI parity tests for KMIP PKCS#11 operations."""

from __future__ import annotations

import json
import queue
import socket
import ssl
import struct
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest
from _test_support import mtls_transport_config
from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._client_ffi import lib as client_lib
from kmipkit._ffi import ffi


def test_pkcs11_response_is_exported_by_client_star_import() -> None:
    namespace: dict[str, object] = {}
    exec("from kmipkit.client import *", namespace)
    assert namespace["Pkcs11Response"] is client.Pkcs11Response


ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"
TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
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


def _pkcs11_response_message(
    expected: dict[str, object], correlation_case: str = "nonempty"
) -> bytes:
    typed = expected["typedResponse"]
    function = typed["function"]
    correlation = typed["correlationValue"]
    output_parameters = typed["outputParameters"]
    return_code = typed["returnCode"]
    vendor = expected["vendorExtension"]
    correlation_tag = int(correlation["tag"], 16)
    if correlation_case == "nonempty":
        correlation_bytes = bytes.fromhex(correlation["hex"])
    elif correlation_case == "presentEmpty":
        correlation_bytes = b""
    elif correlation_case == "absent":
        correlation_bytes = None
    else:
        raise AssertionError(f"unsupported correlation presence case: {correlation_case}")
    fields = {
        int(function["tag"], 16): _ttlv_structure(
            int(function["tag"], 16),
            _ttlv_item(
                int(function["childTag"], 16),
                2,
                struct.pack(">i", function["childValue"]),
            ),
        ),
        int(output_parameters["tag"], 16): _ttlv_structure(
            int(output_parameters["tag"], 16),
            _ttlv_item(
                int(output_parameters["childTag"], 16),
                8,
                bytes.fromhex(output_parameters["childHex"]),
            ),
        ),
        int(return_code["tag"], 16): _ttlv_structure(
            int(return_code["tag"], 16),
            _ttlv_item(
                int(return_code["childTag"], 16),
                5,
                struct.pack(">I", int(return_code["childValue"], 16)),
            ),
        ),
        int(vendor["tag"], 16): _ttlv_item(
            int(vendor["tag"], 16), 2, struct.pack(">i", vendor["integer"])
        ),
    }
    if correlation_bytes is not None:
        fields[correlation_tag] = _ttlv_item(correlation_tag, 8, correlation_bytes)
    response_tags = [int(tag, 16) for tag in expected["responseTags"]]
    if correlation_case == "absent":
        response_tags.remove(correlation_tag)
    assert set(response_tags) == set(fields)
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
    payload = _ttlv_structure(0x42007C, *(fields[tag] for tag in response_tags))
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


def test_pkcs11_native_symbols_are_available_in_the_default_runtime() -> None:
    assert hasattr(client_lib, "kmipkit_client_batch_item_pkcs11")
    assert hasattr(client_lib, "kmipkit_client_batch_item_response_pkcs11_ttlv")


def test_pkcs11_cffi_constructor_accepts_optional_structures_and_bytes() -> None:
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


def test_pkcs11_cffi_constructor_accepts_absent_correlation_value() -> None:
    function = ttlv.ttlv_structure_create()
    item = extensions.client_batch_item_pkcs11(function)
    try:
        assert isinstance(item, extensions.ClientBatchItem)
    finally:
        item.close()
        function.close()


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
    correlation_cases = fixture["expectedOutput"]["correlationValuePresenceCases"]
    assert correlation_cases["absent"] == {"present": False}
    assert correlation_cases["presentEmpty"] == {"present": True, "hex": ""}
    assert bytes.fromhex(correlation_cases["presentEmpty"]["hex"]) == b""
    assert "profileSupport" not in fixture["expectedOutput"]


def test_pkcs11_correlation_mapping_distinguishes_absent_from_present_empty(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    cases = fixture["expectedOutput"]["correlationValuePresenceCases"]
    item = SimpleNamespace(_pointer=lambda: client_ffi.NULL)

    for name, expected in cases.items():
        raw = bytes.fromhex(expected.get("hex", ""))
        native_bytes = client_ffi.new("uint8_t[]", raw) if raw else client_ffi.NULL

        def get_correlation(_item, present, output):
            present[0] = int(expected["present"])
            output[0].data = native_bytes
            output[0].length = len(raw)
            return 0

        monkeypatch.setattr(
            client,
            "lib",
            SimpleNamespace(
                kmipkit_client_batch_item_response_pkcs11_correlation_value=get_correlation
            ),
        )
        actual = client._pkcs11_correlation_value(item)
        assert (
            None if actual is None else actual.to_bytes()
        ) == (raw if expected["present"] else None), name


def test_pkcs11_response_zeroizes_owned_correlation_when_closed() -> None:
    correlation = b"pkcs11-correlation-private-sentinel"
    owned = client._SecretBytes(correlation)
    response = client.Pkcs11Response(
        _structure_with_text(0x420159, "response"), correlation_value=owned
    )

    assert response.correlation_value == correlation

    response.close()
    response.close()

    assert owned.buffer == bytearray(len(correlation))
    assert "redacted" in repr(response)
    with pytest.raises(RuntimeError, match="PKCS#11 response is closed"):
        _ = response.correlation_value


def test_pkcs11_owned_structure_releases_native_handles_on_invalid_results(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item = SimpleNamespace(_pointer=lambda: client_ffi.NULL)
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", 1)
    released: list[object] = []

    def get_error(_item: object, _present: object, output: object) -> int:
        output[0] = handle  # type: ignore[index]
        return 4

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            response=get_error,
            kmipkit_ttlv_structure_release=released.append,
        ),
    )
    with pytest.raises(client.errors.InvalidInputError):
        client._pkcs11_owned_structure(item, "response")
    assert released == [handle]

    released.clear()

    def absent_with_handle(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(
        response=absent_with_handle,
        kmipkit_ttlv_structure_release=released.append,
    ))
    with pytest.raises(client.errors.InvalidInputError):
        client._pkcs11_owned_structure(item, "response")
    assert released == [handle]


def test_pkcs11_owned_structure_releases_handle_when_mapping_raises(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item = SimpleNamespace(_pointer=lambda: client_ffi.NULL)
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", 1)
    released: list[object] = []

    def get_structure(_item: object, present: object, output: object) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            response=get_structure,
            kmipkit_ttlv_structure_release=released.append,
        ),
    )
    monkeypatch.setattr(
        ttlv,
        "_structure_from_native_address",
        lambda _address: (_ for _ in ()).throw(RuntimeError("mapping failed")),
    )

    with pytest.raises(RuntimeError, match="mapping failed"):
        client._pkcs11_owned_structure(item, "response")
    assert released == [handle]


@pytest.mark.parametrize(
    ("status", "present_value", "span_address", "span_length"),
    [
        (4, 0, 0, 0),
        (0, 2, 0, 0),
        (0, 0, 1, 1),
        (0, 1, 0, 1),
    ],
)
def test_pkcs11_correlation_accessor_rejects_inconsistent_native_spans(
    monkeypatch: pytest.MonkeyPatch,
    status: int,
    present_value: int,
    span_address: int,
    span_length: int,
) -> None:
    item = SimpleNamespace(_pointer=lambda: client_ffi.NULL)
    span_data = (
        client_ffi.cast("uint8_t *", span_address)
        if span_address
        else client_ffi.NULL
    )

    def get_span(_item: object, present: object, output: object) -> int:
        present[0] = present_value  # type: ignore[index]
        output[0].data = span_data  # type: ignore[index]
        output[0].length = span_length  # type: ignore[index]
        return status

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_pkcs11_correlation_value=get_span
        ),
    )

    with pytest.raises(client.errors.InvalidInputError):
        client._pkcs11_correlation_value(item)


def test_pkcs11_response_mapping_handles_absent_response_and_optional_interface(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item = SimpleNamespace()
    monkeypatch.setattr(client, "_pkcs11_owned_structure", lambda *_args: None)
    assert client._pkcs11_response(item) is None

    response_ttlv = _structure_with_text(0x420159, "response")
    interface_ttlv = _structure_with_text(0x420159, "interface")
    structures = iter((response_ttlv, interface_ttlv, None, None, None))
    monkeypatch.setattr(
        client, "_pkcs11_owned_structure", lambda *_args: next(structures)
    )
    monkeypatch.setattr(client, "_pkcs11_correlation_value", lambda _item: None)

    response = client._pkcs11_response(item)
    assert response is not None
    assert response.response_ttlv is response_ttlv
    assert response.interface_ttlv is interface_ttlv
    response.close()


def test_pkcs11_response_wipes_correlation_if_later_mapping_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    correlation = b"pkcs11-correlation-private-sentinel"
    owned = client._SecretBytes(correlation)
    response_ttlv = _structure_with_text(0x420159, "response")

    def get_structure(_item: object, symbol: str) -> ttlv.Structure | None:
        if symbol.endswith("_pkcs11_ttlv"):
            return response_ttlv
        if "output_parameters" in symbol:
            raise RuntimeError("injected output-parameter mapping failure")
        return None

    monkeypatch.setattr(client, "_pkcs11_owned_structure", get_structure)
    monkeypatch.setattr(client, "_pkcs11_correlation_value", lambda _item: owned)

    with pytest.raises(RuntimeError, match="injected output-parameter mapping failure"):
        client._pkcs11_response(SimpleNamespace())

    assert owned.buffer == bytearray(len(correlation))


@pytest.mark.parametrize("correlation_case", ["nonempty", "absent", "presentEmpty"])
def test_pkcs11_success_path_preserves_order_and_unknown_values(
    correlation_case: str,
) -> None:
    """Check parity with OASIS KMIP v2.1 §6.1.37, Tables 273–275."""
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    expected = fixture["expectedOutput"]
    function = _structure_with_integer(0x42015A, -12345)
    interface = _structure_with_text(0x420159, "test-interface")
    input_parameters = _structure_with_bytes(0x42015B, b"opaque-input")
    correlation = b"pkcs11-correlation"
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
                stream.sendall(_pkcs11_response_message(expected, correlation_case))
        except BaseException as failure:
            peer_failures.put(failure)
        finally:
            listener.close()

    peer = threading.Thread(target=serve_one_pkcs11_response, daemon=True)
    peer.start()
    transport = mtls_transport_config(port, TLS_FIXTURES)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result = None
    response_ttlv = None
    pkcs11_response = None
    try:
        native_request = extensions.client_batch_item_pkcs11(
            function, interface, correlation, input_parameters
        )
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
        pkcs11_response = result.items[0].pkcs11_response
        assert pkcs11_response is not None
        typed = expected["typedResponse"]
        response_ttlv = pkcs11_response.response_ttlv
        result.close()
        result = None
        assert pkcs11_response.interface_ttlv is None
        expected_correlation = (
            bytes.fromhex(typed["correlationValue"]["hex"])
            if correlation_case == "nonempty"
            else None
            if correlation_case == "absent"
            else b""
        )
        assert pkcs11_response.correlation_value == expected_correlation
        function_ttlv = pkcs11_response.function_ttlv
        output_ttlv = pkcs11_response.output_parameters_ttlv
        return_code_ttlv = pkcs11_response.return_code_ttlv
        assert function_ttlv is not None
        assert output_ttlv is not None
        assert return_code_ttlv is not None
        assert _structure_tags(function_ttlv) == [int(typed["function"]["childTag"], 16)]
        assert _structure_tags(output_ttlv) == [
            int(typed["outputParameters"]["childTag"], 16)
        ]
        assert _structure_tags(return_code_ttlv) == [
            int(typed["returnCode"]["childTag"], 16)
        ]
        with ttlv.ttlv_structure_view(function_ttlv) as function_view:
            assert ttlv.ttlv_structure_view_item_count(function_view) == 1
            with ttlv.ttlv_structure_view_item_at(function_view, 0) as function_item:
                assert ttlv.ttlv_item_view_type(function_item) is ttlv.ItemType.Integer
                with ttlv.ttlv_item_view_value(function_item) as function_value:
                    assert ttlv.ttlv_value_view_integer(function_value) == typed[
                        "function"
                    ]["childValue"]
        with ttlv.ttlv_structure_view(output_ttlv) as output_view:
            assert ttlv.ttlv_structure_view_item_count(output_view) == 1
            with ttlv.ttlv_structure_view_item_at(output_view, 0) as output_item:
                assert ttlv.ttlv_item_view_type(output_item) is ttlv.ItemType.ByteString
                with ttlv.ttlv_item_view_value(output_item) as output_value:
                    assert bytes(
                        ttlv.ttlv_value_view_byte_at(output_value, index)
                        for index in range(ttlv.ttlv_value_view_byte_length(output_value))
                    ) == bytes.fromhex(typed["outputParameters"]["childHex"])
        with ttlv.ttlv_structure_view(return_code_ttlv) as return_code_view:
            assert ttlv.ttlv_structure_view_item_count(return_code_view) == 1
            with ttlv.ttlv_structure_view_item_at(return_code_view, 0) as return_item:
                assert ttlv.ttlv_item_view_type(return_item) is ttlv.ItemType.Enumeration
                with ttlv.ttlv_item_view_value(return_item) as return_value:
                    assert ttlv.ttlv_value_view_enumeration(return_value) == int(
                        typed["returnCode"]["childValue"], 16
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
                expected_tags = [int(tag, 16) for tag in expected["responseTags"]]
                if correlation_case == "absent":
                    expected_tags.remove(0x4200D6)
                assert response_tags == expected_tags
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
        if pkcs11_response is not None:
            pkcs11_response.close()
        if result is not None:
            result.close()
        function.close()
        interface.close()
        input_parameters.close()
        peer.join(timeout=10)
        if peer.is_alive():
            raise AssertionError("the loopback PKCS#11 mTLS peer did not finish")
        if not peer_failures.empty():
            raise peer_failures.get()
