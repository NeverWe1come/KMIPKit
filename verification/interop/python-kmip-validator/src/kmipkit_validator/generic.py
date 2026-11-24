"""KMIP 2.x exchanges built from generic tags using standard Python TLS."""

import hashlib
import socket
import ssl
import struct


HASH_INPUT = b"abc"


def _operation(case, version):
    if version not in {"2.0", "2.1"}:
        return None
    if case["case_id"] == "test-op-ping":
        return 0x3B if version == "2.1" else None
    return {"test-op-locate": 0x08, "test-op-query": 0x18, "test-op-discover-versions": 0x1E,
            "test-crypto-rng-retrieve-rng-32": 0x25, "test-op-hash": 0x27, "test-crypto-hash-sha_256-0": 0x27}.get(case["case_id"])


def supports(case, connection):
    return (
        _operation(case, connection["protocol_version"]) is not None
        and (case["execution"]["mutation"] == "read_only" or case["case_id"] == "test-crypto-rng-retrieve-rng-32")
        and connection["encoding"] == "TTLV"
        and connection["transport"] == "raw_tls"
        and not connection.get("authentication")
    )


def _fields(view):
    from kmipkit import ttlv

    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        with ttlv.ttlv_structure_view_item_at(view, index) as child:
            yield child


def _encode(view):
    from kmipkit import ttlv

    with ttlv.ttlv_item_view_tag(view) as tag, ttlv.ttlv_item_view_value(view) as value:
        code = ttlv.tag_value(tag)
        kind = ttlv.ttlv_item_view_type(view)
        if kind == ttlv.ItemType.Structure:
            with ttlv.ttlv_value_view_structure(value) as fields:
                body = b"".join(_encode(child) for child in _fields(fields))
        elif kind == ttlv.ItemType.Integer:
            body = struct.pack(">i", ttlv.ttlv_value_view_integer(value))
        elif kind == ttlv.ItemType.Enumeration:
            body = struct.pack(">I", ttlv.ttlv_value_view_enumeration(value))
        elif kind == ttlv.ItemType.ByteString:
            body = ttlv.ttlv_value_view_bytes(value)
        else:
            raise ValueError("unsupported generic probe field type")
    return code.to_bytes(3, "big") + bytes([kind]) + struct.pack(">I", len(body)) + body + bytes(-len(body) % 8)


def _read(channel, width):
    result = bytearray()
    while len(result) < width:
        chunk = channel.recv(width - len(result))
        if not chunk:
            raise ValueError("truncated response frame")
        result.extend(chunk)
    return bytes(result)


def _exchange(connection, wire, response_limit):
    endpoint = connection["endpoints"]["raw_tls"]
    tls = connection["tls"]
    context = ssl.create_default_context(cafile=tls["ca_cert_path"])
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.load_cert_chain(tls["client_cert_path"], tls["client_key_path"])
    with socket.create_connection((endpoint["host"], endpoint["port"]), tls.get("connect_timeout_seconds", 10)) as raw:
        with context.wrap_socket(raw, server_hostname=tls.get("server_name_override") or endpoint["host"]) as channel:
            channel.settimeout(tls.get("read_timeout_seconds", 30))
            channel.sendall(wire)
            header = _read(channel, 8)
            size = int.from_bytes(header[4:8], "big")
            size += -size % 8
            if size + 8 > response_limit:
                raise ValueError("response exceeds the codec limit")
            return header + _read(channel, size)


def run(driver, case):
    from kmipkit import messages, ttlv

    connection = driver._connection
    minor = int(connection["protocol_version"][-1])
    operation = _operation(case, connection["protocol_version"])
    if operation is None:
        raise ValueError("no generic exchange recipe")
    item, node = driver._item, driver._structure_item

    def protocol():
        return node(0x420069, [item(0x42006A, "integer", 2), item(0x42006B, "integer", minor)])

    if operation == 0x08:
        payload_fields = [item(0x42004F, "integer", 1), item(0x4200D4, "integer", 0)]
    elif operation == 0x18:
        payload_fields = [item(0x420074, "enumeration", 1)]
    elif operation == 0x1E:
        payload_fields = [protocol()]
    elif operation == 0x25:
        payload_fields = [item(0x4200C4, "integer", 32)]
    elif operation == 0x27:
        payload_fields = [node(0x42002B, [item(0x420038, "enumeration", 6)]),
                          item(0x4200C2, "bytes", HASH_INPUT)]
    else:
        payload_fields = []

    with node(0x420078, [
        node(0x420077, [protocol(), item(0x42000D, "integer", 1)]),
        node(0x42000F, [
            item(0x42005C, "enumeration", operation),
            node(0x420079, payload_fields),
        ]),
    ]) as request, ttlv.ttlv_item_view_from_item(request) as request_view, ttlv.codec_limits_defaults() as limits:
        wire = _exchange(connection, _encode(request_view), ttlv.codec_limits_max_message_bytes(limits))
    with ttlv.ttlv_decode(wire) as root, ttlv.ttlv_item_view_from_item(root) as view:
        with ttlv.ttlv_item_view_tag(view) as tag, ttlv.ttlv_item_view_value(view) as value:
            if ttlv.tag_value(tag) != 0x42007B:
                raise ValueError("unexpected generic response root")
            with ttlv.ttlv_value_view_structure(value) as tree:
                fields = [ttlv.ttlv_item_view_copy(child) for child in _fields(tree)]
        with messages.ResponseMessage.from_ttlv(driver._structure(fields)) as response, response.header() as header:
            version = header.protocol_version
            if (version.major, version.minor) != (2, minor) or response.batch_item_count != 1:
                raise ValueError("generic response version or batch count mismatch")
            with response.batch_item_at(0) as batch:
                if batch.operation != operation:
                    raise ValueError("generic response operation mismatch")
                status, reason = batch.result_status, batch.result_reason
                if status == 0:
                    payload = batch.response_payload
                    if payload is None:
                        raise ValueError("missing generic success payload")
                    with payload:
                        if not _matches(payload, operation, protocol):
                            raise ValueError("generic response did not confirm the requested capability")
    outcome, compatibility = {
        (0, None): ("passed", "supported"),
        (1, 5): ("server_unsupported", "unsupported"),
        (1, 8): ("server_unsupported", "unsupported"),
        (1, 12): ("policy_restricted", "policy_restricted"),
    }.get((status, reason), ("failed", "unknown"))
    return {"status": outcome, "server_compatibility": compatibility,
            "library_correctness": "passed", "parity": "not_compared",
            "result_status_code": status, "result_reason_code": reason,
            "diagnostic": "generic TTLV request and response validated"}


def _matches_locate(payload):
    from kmipkit import ttlv

    total = None
    identifiers = 0
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag, ttlv.ttlv_item_view_value(field) as value:
            kind = ttlv.ttlv_item_view_type(field)
            if ttlv.tag_value(tag) == 0x4200D5 and kind == ttlv.ItemType.Integer and total is None:
                total = ttlv.ttlv_value_view_integer(value)
            elif ttlv.tag_value(tag) == 0x420094 and kind in (
                    ttlv.ItemType.TextString, ttlv.ItemType.Integer, ttlv.ItemType.Enumeration):
                identifiers += 1
            else:
                return False
    return identifiers <= 1 and (total is None or total >= identifiers)


def _matches(payload, operation, protocol):
    from kmipkit import ttlv

    if operation == 0x08:
        return _matches_locate(payload)
    if operation == 0x3B:
        return ttlv.ttlv_structure_view_item_count(payload) == 0
    if operation == 0x18:
        for field in _fields(payload):
            with ttlv.ttlv_item_view_tag(field) as tag:
                if ttlv.tag_value(tag) == 0x42005C:
                    with ttlv.ttlv_item_view_value(field) as value:
                        if ttlv.ttlv_value_view_enumeration(value):
                            return True
        return False
    if ttlv.ttlv_structure_view_item_count(payload) != 1:
        return False
    if operation in (0x25, 0x27):
        with ttlv.ttlv_structure_view_item_at(payload, 0) as field:
            with ttlv.ttlv_item_view_tag(field) as tag, ttlv.ttlv_item_view_value(field) as value:
                return (ttlv.tag_value(tag) == 0x4200C2
                        and ttlv.ttlv_item_view_type(field) == ttlv.ItemType.ByteString
                        and (ttlv.ttlv_value_view_byte_length(value) == 32 if operation == 0x25
                             else ttlv.ttlv_value_view_bytes(value) == hashlib.sha256(HASH_INPUT).digest()))
    with protocol() as expected, ttlv.ttlv_item_view_from_item(expected) as expected_view:
        with ttlv.ttlv_structure_view_item_at(payload, 0) as actual:
            return _encode(actual) == _encode(expected_view)
