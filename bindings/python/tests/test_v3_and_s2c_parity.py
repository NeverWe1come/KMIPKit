"""Comprehensive parity test for KMIP 3.0, Server-to-Client (S2C) v6, and protocol utilities."""

from __future__ import annotations

import queue
import struct
import threading
from pathlib import Path

import pytest

from kmipkit import (
    client,
    errors,
    extensions,
    protocol,
    server_to_client,
    ttlv,
)
from kmipkit.protocol import LeaseContext, LinkNormalizer, NistKeyLifecycle, VersionGate, VersionSpan
from kmipkit.protocol_values import EphemeralMode
from kmipkit.server_to_client import (
    ClientListener,
    ClientListenerConnection,
    IncomingServerBatch,
    ItemView,
    ListenerOptions,
    ListenerTransportConfiguration,
    ProtocolVersion,
    PutItemDetails,
    S2cCapabilities,
    S2cEndpointRoleResponse,
    S2cOperationResult,
    S2cUtilities,
    ServerRequestDispatcher,
    ServerRequestHandler,
)

_FIXTURES = Path(__file__).resolve().parents[3] / "bindings" / "shared" / "fixtures" / "login-mtls"


def _ttlv_test_item(tag: int, item_type: int, value: bytes) -> bytes:
    tag_bytes = bytes([(tag >> 16) & 0xFF, (tag >> 8) & 0xFF, tag & 0xFF])
    type_byte = bytes([item_type])
    length = len(value)
    len_bytes = struct.pack(">I", length)
    pad_len = (8 - (length % 8)) % 8
    return tag_bytes + type_byte + len_bytes + value + (b"\x00" * pad_len)


def _int_val(val: int) -> bytes:
    return struct.pack(">i", val)


def _build_full_s2c_batch_bytes() -> bytes:
    # Item 1: Put
    put_payload = (
        _ttlv_test_item(0x00420070, 0x05, _int_val(2))  # PutFunction = Replace (2)
        + _ttlv_test_item(0x00420094, 0x07, b"sym-key-uid")  # UniqueIdentifier
        + _ttlv_test_item(0x00420076, 0x07, b"replaced-sym-uid")  # ReplacedUniqueIdentifier
        + _ttlv_test_item(0x0042008F, 0x01, _ttlv_test_item(0x0042008E, 0x08, b"\x01\x02\x03\x04"))  # SymKey
        + _ttlv_test_item(0x00420125, 0x01, _ttlv_test_item(0x00420008, 0x07, b"CustomAttr"))  # AttrStruct
    )
    item1 = (
        _ttlv_test_item(0x0042005C, 0x05, _int_val(0x0000001C))  # Put
        + _ttlv_test_item(0x00420093, 0x08, b"item-put")
        + _ttlv_test_item(0x00420079, 0x01, put_payload)
    )
    batch_item1 = _ttlv_test_item(0x0042000F, 0x01, item1)

    # Item 2: Query
    query_payload = (
        _ttlv_test_item(0x00420074, 0x05, _int_val(1))  # Operations (1)
        + _ttlv_test_item(0x00420074, 0x05, _int_val(3))  # ServerInformation (3)
    )
    item2 = (
        _ttlv_test_item(0x0042005C, 0x05, _int_val(0x00000018))  # Query
        + _ttlv_test_item(0x00420093, 0x08, b"item-query")
        + _ttlv_test_item(0x00420079, 0x01, query_payload)
    )
    batch_item2 = _ttlv_test_item(0x0042000F, 0x01, item2)

    # Item 3: DiscoverVersions
    disc_pv = _ttlv_test_item(0x0042006A, 0x02, _int_val(2)) + _ttlv_test_item(0x0042006B, 0x02, _int_val(1))
    disc_payload = _ttlv_test_item(0x00420069, 0x01, disc_pv)
    item3 = (
        _ttlv_test_item(0x0042005C, 0x05, _int_val(0x0000001E))  # DiscoverVersions
        + _ttlv_test_item(0x00420093, 0x08, b"item-disc")
        + _ttlv_test_item(0x00420079, 0x01, disc_payload)
    )
    batch_item3 = _ttlv_test_item(0x0042000F, 0x01, item3)

    # Item 4: Notify
    notify_payload = (
        _ttlv_test_item(0x00420094, 0x07, b"notify-uid")
        + _ttlv_test_item(0x00420125, 0x01, _ttlv_test_item(0x00420008, 0x07, b"CustomAttr"))
        + _ttlv_test_item(0x42013B, 0x01, _ttlv_test_item(0x42000A, 0x07, b"first"))
        + _ttlv_test_item(0x42013B, 0x01, _ttlv_test_item(0x42000A, 0x07, b"second"))
    )
    item4 = (
        _ttlv_test_item(0x0042005C, 0x05, _int_val(0x0000001B))  # Notify
        + _ttlv_test_item(0x00420093, 0x08, b"item-notify")
        + _ttlv_test_item(0x00420079, 0x01, notify_payload)
    )
    batch_item4 = _ttlv_test_item(0x0042000F, 0x01, item4)

    # Item 5: SetEndpointRole
    role_payload = (
        _ttlv_test_item(0x00420151, 0x05, _int_val(2))  # EndpointRole = SERVER (2)
        + _ttlv_test_item(0x42009D, 0x07, "server 🔑".encode("utf-8"))
    )
    item5 = (
        _ttlv_test_item(0x0042005C, 0x05, _int_val(0x00000032))  # SetEndpointRole
        + _ttlv_test_item(0x00420093, 0x08, b"item-role")
        + _ttlv_test_item(0x00420079, 0x01, role_payload)
    )
    batch_item5 = _ttlv_test_item(0x0042000F, 0x01, item5)

    # Header
    hdr_pv = _ttlv_test_item(0x0042006A, 0x02, _int_val(2)) + _ttlv_test_item(0x0042006B, 0x02, _int_val(1))
    header = _ttlv_test_item(0x00420069, 0x01, hdr_pv) + _ttlv_test_item(0x0042000D, 0x02, _int_val(5))
    request_header = _ttlv_test_item(0x00420077, 0x01, header)

    root = request_header + batch_item1 + batch_item2 + batch_item3 + batch_item4 + batch_item5
    return _ttlv_test_item(0x00420078, 0x01, root)


def test_ttlv_v3_primitive_types() -> None:
    with ttlv.codec_limits_defaults() as limits:
        with (
            ttlv.raw_tag_from_raw(0x420094) as r1,
            ttlv.raw_tag_try_checked(r1) as tag1,
            ttlv.ttlv_value_identifier(b"id-12345", limits) as val1,
            ttlv.ttlv_item_create(tag1, val1, limits) as item1,
            ttlv.ttlv_item_view_from_item(item1) as view1,
        ):
            assert view1.item_type == ttlv.ItemType.Identifier

        with (
            ttlv.raw_tag_from_raw(0x420094) as r2,
            ttlv.raw_tag_try_checked(r2) as tag2,
            ttlv.ttlv_value_reference(b"ref-67890", limits) as val2,
            ttlv.ttlv_item_create(tag2, val2, limits) as item2,
            ttlv.ttlv_item_view_from_item(item2) as view2,
        ):
            assert view2.item_type == ttlv.ItemType.Reference

        with (
            ttlv.raw_tag_from_raw(0x420094) as r3,
            ttlv.raw_tag_try_checked(r3) as tag3,
            ttlv.ttlv_value_name_reference(b"nameref-abc", limits) as val3,
            ttlv.ttlv_item_create(tag3, val3, limits) as item3,
            ttlv.ttlv_item_view_from_item(item3) as view3,
        ):
            assert view3.item_type != ttlv.ItemType.Reference
            assert view3.item_type == ttlv.ItemType.NameReference


def test_s2c_capabilities_validation() -> None:
    caps = S2cCapabilities(
        operations=[0x01, 0x02, 0x03],
        object_types=[0x01, 0x02],
        extension_tags=[0x540001],
        vendor_identification="KMIPKitTest",
    )
    assert caps.operations == [1, 2, 3]
    assert caps.object_types == [1, 2]
    assert caps.extension_tags == [0x540001]
    assert caps.vendor_identification == "KMIPKitTest"

    # Strict bounds: > 1024 items must be rejected
    too_many = list(range(1025))
    with pytest.raises(errors.KmipKitError):
        S2cCapabilities(operations=too_many)
    with pytest.raises(errors.KmipKitError):
        S2cCapabilities(object_types=too_many)
    with pytest.raises(errors.KmipKitError):
        S2cCapabilities(extension_tags=too_many)

    # uint32 range check: negative or > 0xFFFFFFFF rejected
    with pytest.raises(errors.KmipKitError):
        S2cCapabilities(operations=[-1])
    with pytest.raises(errors.KmipKitError):
        S2cCapabilities(operations=[0x1_0000_0000])


def test_listener_options_and_timeout_validation() -> None:
    # Negative timeout rejection
    builder = ListenerTransportConfiguration.builder()
    with pytest.raises(errors.KmipKitError):
        builder.read_timeout_ms(-1)
    with pytest.raises(errors.KmipKitError):
        builder.write_timeout_ms(-100)
    with pytest.raises(errors.KmipKitError):
        builder.max_request_bytes(0)
    with pytest.raises(errors.KmipKitError):
        builder.max_request_bytes(-5)

    # ListenerOptions target_path validation: must start with "/"
    with pytest.raises(errors.KmipKitError):
        ListenerOptions(target_path="no-leading-slash")
    opts = ListenerOptions(target_path="/kmip")
    assert opts.target_path == "/kmip"


def test_server_request_dispatcher_full_dispatch_and_error_preservation() -> None:
    batch_bytes = _build_full_s2c_batch_bytes()
    with ttlv.codec_limits_defaults() as limits:
        with IncomingServerBatch.from_bytes(batch_bytes, limits) as batch:
            assert batch.item_count() == 5
            dispatcher = ServerRequestDispatcher.create()

            # 1. Successful handler
            class SuccessHandler(ServerRequestHandler):
                def __init__(self) -> None:
                    self.disc_called = False
                    self.notify_called = False
                    self.put_called = False
                    self.query_called = False
                    self.role_called = False

                def handle_discover_versions(
                    self, item: ItemView, client_versions: list[tuple[int, int]]
                ) -> tuple[list[tuple[int, int]], S2cOperationResult]:
                    self.disc_called = True
                    return [(2, 1)], S2cOperationResult.success()

                def handle_notify(
                    self,
                    item: ItemView,
                    unique_identifier: bytes | None,
                    attributes_ttlv: bytes | None,
                    deleted_references_ttlv: bytes | None = None,
                ) -> S2cOperationResult:
                    self.notify_called = True
                    assert unique_identifier == b"notify-uid"
                    return S2cOperationResult.success()

                def handle_put(
                    self,
                    item: ItemView,
                    put_details: PutItemDetails,
                ) -> S2cOperationResult:
                    self.put_called = True
                    assert put_details.put_function == 2
                    assert put_details.unique_identifier == b"sym-key-uid"
                    assert put_details.replaced_unique_identifier == b"replaced-sym-uid"
                    return S2cOperationResult.success()

                def handle_query(
                    self, item: ItemView, query_function_flags: int
                ) -> tuple[S2cCapabilities, S2cOperationResult]:
                    self.query_called = True
                    caps = S2cCapabilities(
                        operations=[0x1C, 0x18],
                        object_types=[1, 2],
                        vendor_identification="KMIPKitPythonQuery",
                    )
                    return caps, S2cOperationResult.success()

                def handle_set_endpoint_role(
                    self, item: ItemView, target_role: int, vendor_identification: str | None = None
                ) -> tuple[S2cEndpointRoleResponse | None, S2cOperationResult]:
                    self.role_called = True
                    assert target_role == 2
                    assert vendor_identification == "server 🔑"
                    resp = S2cEndpointRoleResponse(1, "ack-vendor")
                    return resp, S2cOperationResult.success()

            h = SuccessHandler()
            with dispatcher.dispatch_v6(batch, h) as resp:
                assert resp.batch_item_count == 5
                for i in range(5):
                    with resp.batch_item_at(i) as bi:
                        assert bi.result_status == 0
            assert h.disc_called and h.notify_called and h.put_called and h.query_called and h.role_called

            # 2. Error handler preserving status, reason, message
            class CustomErrorHandler(ServerRequestHandler):
                def handle_discover_versions(
                    self, item: ItemView, client_versions: tuple[ProtocolVersion, ...]
                ) -> tuple[list[tuple[int, int]], S2cOperationResult]:
                    return [], S2cOperationResult.failure(0x01, 0x02, "Custom DiscoverVersions Error")

                def handle_put(
                    self, item: ItemView, put_details: PutItemDetails
                ) -> S2cOperationResult:
                    return S2cOperationResult.failure(0x01, 0x03, "Custom Put Error")

                def handle_query(
                    self, item: ItemView, flags: int
                ) -> tuple[S2cCapabilities, S2cOperationResult]:
                    return S2cCapabilities(), S2cOperationResult.failure(0x01, 0x04, "Custom Query Error")

            eh = CustomErrorHandler()
            with dispatcher.dispatch_v6(batch, eh) as err_resp:
                assert err_resp.batch_item_count == 5
                with err_resp.batch_item_at(0) as bi:  # Put
                    assert bi.result_status == 0x01
                    assert bi.result_reason == 0x03
                    assert bi.result_message == "Custom Put Error"
                with err_resp.batch_item_at(1) as bi:  # Query
                    assert bi.result_status == 0x01
                    assert bi.result_reason == 0x04
                    assert bi.result_message == "Custom Query Error"
                with err_resp.batch_item_at(2) as bi:  # DiscoverVersions
                    assert bi.result_status == 0x01
                    assert bi.result_reason == 0x02
                    assert bi.result_message == "Custom DiscoverVersions Error"
                # Notify and SetEndpointRole default methods should return status 1, reason 5 (OperationNotSupported), NOT silent success (0)
                with err_resp.batch_item_at(3) as bi:  # Notify
                    assert bi.result_status == 0x01
                    assert bi.result_reason == 0x05
                with err_resp.batch_item_at(4) as bi:  # SetEndpointRole
                    assert bi.result_status == 0x01
                    assert bi.result_reason == 0x05


def test_client_listener_end_to_end_network_exchange() -> None:
    import socket
    import ssl

    server_cert = (_FIXTURES / "server-cert.pem").read_bytes()
    server_key = bytearray((_FIXTURES / "server-key.pem").read_bytes())
    client_cert = (_FIXTURES / "client-cert.pem").read_bytes()

    cfg = (
        ListenerTransportConfiguration.builder()
        .add_server_certificate(server_cert)
        .server_private_key(server_key)
        .add_trust_certificate(client_cert)
        .read_timeout_ms(5000)
        .write_timeout_ms(5000)
        .build()
    )

    listener = ClientListener.bind_raw_tls("127.0.0.1", 0, cfg, ListenerOptions())
    port = listener.get_port()
    assert port > 0

    accepted_queue: queue.Queue[ClientListenerConnection] = queue.Queue()
    errors_queue: queue.Queue[Exception] = queue.Queue()

    def server_thread() -> None:
        try:
            conn = listener.accept(timeout_ms=5000)
            accepted_queue.put(conn)
        except Exception as ex:
            errors_queue.put(ex)

    t = threading.Thread(target=server_thread, daemon=True)
    t.start()

    client_result: queue.Queue[bytes] = queue.Queue()

    def client_worker() -> None:
        try:
            ctx = ssl.create_default_context(
                ssl.Purpose.SERVER_AUTH, cafile=str(_FIXTURES / "server-cert.pem")
            )
            ctx.load_cert_chain(
                certfile=str(_FIXTURES / "client-cert.pem"),
                keyfile=str(_FIXTURES / "client-key.pem"),
            )
            with socket.create_connection(("127.0.0.1", port)) as sock:
                with ctx.wrap_socket(sock, server_hostname="localhost") as ssock:
                    batch_bytes = _build_full_s2c_batch_bytes()
                    ssock.sendall(batch_bytes)
                    resp_data = bytearray()
                    while True:
                        chunk = ssock.recv(4096)
                        if not chunk:
                            break
                        resp_data.extend(chunk)
                        if len(resp_data) >= 8:
                            expected_len = 8 + int.from_bytes(resp_data[4:8], "big")
                            if len(resp_data) >= expected_len:
                                break
                    client_result.put(bytes(resp_data))
        except Exception as ex:
            errors_queue.put(ex)

    ct = threading.Thread(target=client_worker, daemon=True)
    ct.start()

    try:
        # Server side
        conn = accepted_queue.get(timeout=6)
        inbound_batch = conn.recv_batch(timeout_ms=5000)
        assert inbound_batch.item_count() == 5

        # Dispatch ping
        dispatcher = ServerRequestDispatcher.create()

        class S2cHandler(ServerRequestHandler):
            def handle_discover_versions(self, item, versions):
                return [(2, 1)], S2cOperationResult.success()

            def handle_put(self, item, put_details):
                return S2cOperationResult.success()

            def handle_query(self, item, flags):
                return S2cCapabilities(operations=[0x1C]), S2cOperationResult.success()

        with dispatcher.dispatch_v6(inbound_batch, S2cHandler()) as response:
            conn.send_response(response, timeout_ms=5000)

        # Role reversed session test
        with conn.into_role_reversed() as session:
            assert session is not None
            assert not session.closed

        ct.join(timeout=6)
        t.join(timeout=6)
        assert errors_queue.empty()
        client_resp = client_result.get(timeout=6)
        assert len(client_resp) > 0
    finally:
        listener.close()


def test_protocol_utilities() -> None:
    # VersionGate
    assert VersionGate.validate_operation(2, 1, 0x01)
    assert VersionGate.validate_tag(2, 1, 0x420001)

    span = VersionGate.operation_span(0x01)
    assert span is not None
    assert span.introduced_major >= 1

    # NistKeyLifecycle
    assert NistKeyLifecycle.can_originate(2)
    assert NistKeyLifecycle.can_process(2)

    # LeaseContext
    uid = extensions.UniqueIdentifier.text_string("lease-target-id")
    with LeaseContext.create(uid, 1700000000, 3600) as ctx:
        assert ctx.expires_at() == 1700003600
        assert not ctx.is_expired(1700001000)
        assert ctx.is_expired(1700004000)
        ctx.check_valid(1700001000)
        with pytest.raises(errors.KmipKitError):
            ctx.check_valid(1700004000)


def test_s2c_utilities() -> None:
    # negotiate_discover_versions
    selected = S2cUtilities.negotiate_discover_versions((2, 1), [(1, 4), (2, 1)])
    assert selected == ProtocolVersion(2, 1)

    # put_effective_function
    eff = S2cUtilities.put_effective_function(1, False)
    assert eff == 1
    eff2 = S2cUtilities.put_effective_function(2, False)
    assert eff2 == 1
    eff3 = S2cUtilities.put_effective_function(2, True)
    assert eff3 == 2

    # build_query_response_ttlv
    caps = S2cCapabilities(operations=[0x1C], object_types=[1])
    with S2cUtilities.build_query_response_ttlv(1, capabilities=caps) as ttlv_data:
        assert ttlv_data is not None
