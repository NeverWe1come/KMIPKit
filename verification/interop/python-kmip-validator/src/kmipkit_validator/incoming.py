"""Receive isolated KMIP operations from an authenticated test peer."""

from contextlib import contextmanager
import ipaddress
from pathlib import Path
import socket
import ssl
import time

from .driver import PythonKmipDriver
from .generic import _encode, _fields, _read

TYPED_OPERATIONS = {"Notify", "Put"}
OPERATIONS = {"Discover Versions": 0x1E, "Notify": 0x1B, "Put": 0x1C, "Query": 0x18, "Set Endpoint Role": 0x32}


def is_incoming(case):
    return case["requires"].get("workflow_type") == "server_to_client"


def supports_connection(connection):
    return (connection["protocol_version"] in {"2.0", "2.1"}
            and connection["transport"] == "raw_tls" and connection["encoding"] == "TTLV"
            and connection["api_level"] in {"typed-protocol", "generic-ttlv"})


def supports(case, connection):
    operations = TYPED_OPERATIONS if connection["api_level"] == "typed-protocol" else OPERATIONS
    return is_incoming(case) and case["operation"] in operations and supports_connection(connection)


def publish_address(publish_ready, host, port):
    if publish_ready is not None:
        publish_ready(f"[{host}]:{port}" if host.version == 6 else f"{host}:{port}")


def listener_address(raw):
    host, separator, port = raw.rpartition(":")
    if not separator or not port.isascii() or not port.isdecimal() or not 0 <= int(port) <= 65535:
        raise ValueError("listener address must be a numeric IP address and port")
    if host.startswith("[") and host.endswith("]"):
        host = host[1:-1]
    return ipaddress.ip_address(host), int(port)


@contextmanager
def bind(connection, address, certificate, key, publish_ready):
    if address is None:
        yield None
        return
    if not supports_connection(connection):
        raise ValueError("incoming listener requires KMIP 2.x TTLV over raw TLS")
    host, port = address
    tls = connection["tls"]
    if connection["api_level"] == "typed-protocol":
        from kmipkit.server_to_client import ClientListener, ListenerTransportConfiguration

        timeout = tls.get("read_timeout_seconds", 30) * 1000
        with ListenerTransportConfiguration(
            server_certificates=(Path(certificate or tls["client_cert_path"]),),
            server_private_key=Path(key or tls["client_key_path"]),
            trust_certificates=(Path(tls["ca_cert_path"]),),
            read_timeout_ms=timeout, write_timeout_ms=timeout,
        ) as transport, ClientListener.bind_raw_tls(str(host), port, transport) as listener:
            publish_address(publish_ready, host, listener.get_port())
            yield listener
        return

    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.verify_mode = ssl.CERT_REQUIRED
    context.load_verify_locations(cafile=tls["ca_cert_path"])
    context.load_cert_chain(certificate or tls["client_cert_path"], key or tls["client_key_path"])
    family = socket.AF_INET6 if host.version == 6 else socket.AF_INET
    with socket.create_server((str(host), port), family=family) as listener:
        timeout = tls.get("read_timeout_seconds", 30)
        listener.settimeout(timeout)
        publish_address(publish_ready, host, listener.getsockname()[1])
        yield listener, context


def exchange_typed(listener, case, connection):
    from kmipkit import ttlv
    from kmipkit.errors import KmipKitError
    from kmipkit.server_to_client import (
        S2cCapabilities, S2cOperationResult, ServerRequestDispatcher, ServerRequestHandler,
    )

    class Handler(ServerRequestHandler):
        def handle_discover_versions(self, item_view, client_versions):
            return (), S2cOperationResult.failure(1, 5, "operation not supported")

        def handle_query(self, item_view, query_function_flags):
            return S2cCapabilities(), S2cOperationResult.failure(1, 5, "operation not supported")

        def handle_notify(self, item_view, identifier, attributes, deleted_references=None):
            return S2cOperationResult.success() if identifier else S2cOperationResult.failure(1, 7, "invalid field")

        def handle_put(self, item_view, put_details):
            valid = (put_details.unique_identifier and put_details.object_item is not None
                     and put_details.put_function in {1, 2}
                     and (put_details.put_function != 2 or put_details.replaced_unique_identifier))
            if valid:
                with ttlv.ttlv_item_view_from_item(put_details.object_item) as view:
                    valid = ttlv.ttlv_item_view_type(view) == ttlv.ItemType.Structure
            return S2cOperationResult.success() if valid else S2cOperationResult.failure(1, 7, "invalid field")

    timeout = connection["tls"].get("read_timeout_seconds", 30) * 1000
    operation = OPERATIONS[case["operation"]]
    minor = int(connection["protocol_version"][-1])
    try:
        with listener.accept(timeout) as peer, peer.recv_batch(timeout) as batch:
            version = batch.protocol_version()
            if (version.major, version.minor) != (2, minor) or batch.item_count() != 1 \
                    or batch.operation_at(0) != operation:
                raise ValueError("unexpected incoming batch")
            with ServerRequestDispatcher.create() as dispatcher, dispatcher.dispatch_v6(batch, Handler()) as response:
                with response.batch_item_at(0) as result:
                    status, reason = result.result_status, result.result_reason
                    if result.operation != operation:
                        raise ValueError("unexpected reply operation")
                peer.send_response(response, timeout)
    except (OSError, ValueError, KmipKitError):
        return "failed", None, None, "typed incoming request validation or delivery failed"
    if status != 0:
        return "failed", status, reason, "typed incoming handler rejected the payload"
    return "passed", 0, None, "typed incoming request validated and response sent"


def exchange(listener_context, case, connection):
    from kmipkit.errors import KmipKitError

    if connection["api_level"] == "typed-protocol":
        return exchange_typed(listener_context, case, connection)

    listener, context = listener_context
    try:
        raw, _ = listener.accept()
    except TimeoutError:
        return "not_run", None, None, "incoming peer did not connect before the listener deadline"
    try:
        with raw:
            raw.settimeout(listener.gettimeout())
            with context.wrap_socket(raw, server_side=True) as channel:
                status = _reply(
                    channel, OPERATIONS[case["operation"]], int(connection["protocol_version"][-1]),
                )
    except (OSError, ValueError, KmipKitError):
        return "failed", None, None, "incoming request validation or response delivery failed"
    if status != 0:
        return "failed", status, 7, "peer sent an invalid payload; Invalid Field sent"
    return "passed", 0, None, "incoming request validated and direct-tag response sent"


def _reply(channel, operation, minor):
    from kmipkit import messages, ttlv
    from kmipkit.errors import KmipKitError
    from kmipkit.server_to_client import IncomingServerBatch

    item = PythonKmipDriver._item
    node = PythonKmipDriver._structure_item
    header_bytes = _read(channel, 8)
    size = int.from_bytes(header_bytes[4:], "big")
    padded = size + (-size % 8)
    with ttlv.codec_limits_defaults() as limits:
        if padded + 8 > ttlv.codec_limits_max_message_bytes(limits):
            raise ValueError("incoming TTLV request exceeds the codec limit")
        wire = header_bytes + _read(channel, padded)
        with ttlv.ttlv_decode(wire) as root, ttlv.ttlv_item_view_from_item(root) as view:
            with ttlv.ttlv_item_view_tag(view) as tag, ttlv.ttlv_item_view_value(view) as value:
                if ttlv.tag_value(tag) != 0x420078:
                    raise ValueError("expected Request Message")
                with ttlv.ttlv_value_view_structure(value) as tree:
                    fields = [ttlv.ttlv_item_view_copy(child) for child in _fields(tree)]
        with messages.RequestMessage.from_ttlv(PythonKmipDriver._structure(fields)) as request, request.header() as header:
            version = header.protocol_version
            if (version.major, version.minor) != (2, minor) or request.batch_item_count != 1:
                raise ValueError("incoming protocol version or batch count mismatch")
            correlation = header.client_correlation_value
            max_response = header.maximum_response_size
            with request.batch_item_at(0) as batch:
                if batch.operation != operation:
                    raise ValueError("incoming operation does not match the selected case")
                batch_id = batch.unique_batch_item_id
        try:
            peer = IncomingServerBatch.decode_ttlv(wire, limits)
        except KmipKitError:
            status, payload = 1, None
        else:
            with peer:
                payload = _payload(peer, operation, minor)
            status = 0

    header = [_version(minor), item(0x420092, "date_time", int(time.time()))]
    if correlation is not None:
        header.append(item(0x420105, "text", correlation))
    header.append(item(0x42000D, "integer", 1))
    batch = [item(0x42005C, "enumeration", operation)]
    if batch_id is not None:
        batch.append(item(0x420093, "bytes", batch_id))
    batch.append(item(0x42007F, "enumeration", status))
    batch.append(item(0x42007E, "enumeration", 7) if payload is None else node(0x42007C, payload))
    with node(0x42007B, [node(0x42007A, header), node(0x42000F, batch)]) as response:
        with ttlv.ttlv_item_view_from_item(response) as view:
            encoded = _encode(view)
    if max_response is not None and len(encoded) > max_response:
        raise ValueError("incoming maximum response size would be exceeded")
    channel.sendall(encoded)
    return status


def _version(minor):
    item = PythonKmipDriver._item
    return PythonKmipDriver._structure_item(
        0x420069, [item(0x42006A, "integer", 2), item(0x42006B, "integer", minor)],
    )


def _payload(peer, operation, minor):
    item = PythonKmipDriver._item
    if operation == 0x1E:
        versions = peer.discover_versions_item(0)
        return ([_version(minor)] if not versions
                or any((version.major, version.minor) == (2, minor) for version in versions) else [])
    if operation == 0x18:
        flags = peer.query_item(0)
        fields = []
        if flags & 1:
            fields.extend(item(0x42005C, "enumeration", code) for code in OPERATIONS.values())
        if flags & 4:
            fields.append(item(0x42009D, "text", "KMIPKit"))
            fields.append(PythonKmipDriver._structure_item(0x420088, []))
        return fields
    if operation == 0x32:
        return [item(0x420151, "enumeration", peer.set_endpoint_role_item(0))]
    return []
