"""KMIP 2.x exchanges built from generic tags using standard Python TLS."""

import hashlib
import socket
import ssl
import struct


HASH_INPUT = b"abc"
CREATE, GET, GET_ATTRIBUTES, DESTROY, ARCHIVE, RECOVER = 1, 0x0A, 0x0B, 0x14, 0x15, 0x16
CHECK = 0x09
GET_ATTRIBUTE_LIST, ACTIVATE, REVOKE = 0x0C, 0x12, 0x13
OWNED_KEY_OPERATIONS = {CREATE, CHECK, GET, GET_ATTRIBUTES, GET_ATTRIBUTE_LIST, ACTIVATE, REVOKE, DESTROY, ARCHIVE, RECOVER}


def _operation(case, version):
    if version not in {"2.0", "2.1"}:
        return None
    if case["case_id"] == "test-op-ping":
        return 0x3B if version == "2.1" else None
    return {"test-op-create": CREATE, "test-op-check": CHECK, "test-op-get": GET, "test-op-get-attribute-list": GET_ATTRIBUTE_LIST,
            "test-op-activate": ACTIVATE, "test-op-revoke": REVOKE, "test-op-destroy": DESTROY, "test-op-get-attributes": GET_ATTRIBUTES,
            "test-op-archive": ARCHIVE, "test-op-recover": RECOVER, "test-op-locate": 0x08, "test-op-query": 0x18, "test-op-discover-versions": 0x1E,
            "test-crypto-rng-retrieve-rng-32": 0x25, "test-op-hash": 0x27, "test-crypto-hash-sha_256-0": 0x27}.get(case["case_id"])


def supports(case, connection):
    operation = _operation(case, connection["protocol_version"])
    mutation = case["execution"]["mutation"]
    return (
        operation is not None
        and (mutation == "mutating" if operation in OWNED_KEY_OPERATIONS
             else mutation == "read_only" or operation == 0x25)
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
        elif kind in (ttlv.ItemType.ByteString, ttlv.ItemType.TextString):
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
    operation = _operation(case, driver._connection["protocol_version"])
    if operation is None:
        raise ValueError("no generic exchange recipe")
    if operation in OWNED_KEY_OPERATIONS:
        return _run_owned_key(driver, operation)
    return _execute(driver, operation)


def _checked_exchange(driver, operation, identifier=None, expected=None):
    from kmipkit.errors import KmipKitError

    try:
        return _execute(driver, operation, identifier, expected)
    except (OSError, ValueError, RuntimeError, KmipKitError):
        return _failure("The generic TLS exchange or response validation failed.")


def _failure(diagnostic, status="failed"):
    return {"status": status, "library_correctness": "failed" if status == "failed" else "not_checked",
            "server_compatibility": "unknown", "parity": "not_compared", "diagnostic": diagnostic}


def _run_owned_key(driver, operation):
    identifier = None

    def check_create(payload):
        nonlocal identifier
        identifier = _text_identifier(payload)
        return identifier is not None and _has_field(payload, 0x420057, "Enumeration", 2)

    created = _checked_exchange(driver, CREATE, expected=check_create)
    if identifier is None:
        if operation == CREATE or created["status"] == "failed":
            return created
        return _failure("Server rejected isolated Create setup.", "not_run")

    def same_id(payload):
        return _text_identifier(payload) == identifier

    if created["status"] == "passed" and operation == DESTROY:
        return _checked_exchange(driver, DESTROY, identifier, same_id)

    target = created
    if created["status"] == "passed" and operation == GET:
        target = _checked_exchange(
            driver, GET, identifier, lambda payload: same_id(payload)
            and _has_field(payload, 0x420057, "Enumeration", 2)
            and _has_field(payload, 0x42008F, "Structure"),
        )
    if created["status"] == "passed" and operation == GET_ATTRIBUTES:
        target = _checked_exchange(
            driver, GET_ATTRIBUTES, identifier,
            lambda payload: same_id(payload) and _has_length_attribute(payload),
        )
    if created["status"] == "passed" and operation == GET_ATTRIBUTE_LIST:
        target = _checked_exchange(
            driver, GET_ATTRIBUTE_LIST, identifier,
            lambda payload: same_id(payload) and _has_length_reference(payload),
        )
    if created["status"] == "passed" and operation in (ACTIVATE, REVOKE, CHECK):
        activate = _checked_exchange(driver, ACTIVATE, identifier, same_id)
        target = activate
        if activate["status"] == "passed" and operation == CHECK:
            target = _checked_exchange(driver, CHECK, identifier,
                                       lambda payload: same_id(payload) and _matches_check_usage(payload))
        if activate.get("result_status_code") in (None, 0):
            revoke = _checked_exchange(driver, REVOKE, identifier, same_id)
            if activate["status"] == "passed":
                if operation == REVOKE:
                    target = revoke
                elif revoke["status"] != "passed":
                    target = _failure("Cleanup Revoke failed for the activated key.")
        elif operation != ACTIVATE:
            target = _failure("Server rejected isolated Activate setup.", "not_run")
    if created["status"] == "passed" and operation in (ARCHIVE, RECOVER):
        archive = _checked_exchange(driver, ARCHIVE, identifier, same_id)
        target = archive
        if archive.get("result_status_code") == 0:
            recover = _checked_exchange(driver, RECOVER, identifier, same_id)
            if archive["status"] == "passed":
                if operation == RECOVER:
                    target = recover
                elif recover["status"] != "passed":
                    target = _failure("Cleanup Recover failed.")
        elif operation == RECOVER and archive["status"] != "failed":
            target = _failure("Server rejected Archive setup.", "not_run")
    cleanup = _checked_exchange(driver, DESTROY, identifier, same_id)
    return target if cleanup["status"] == "passed" else _failure("Cleanup Destroy failed for the isolated key.")


def _execute(driver, operation, identifier=None, expected=None):
    from kmipkit import messages, ttlv

    connection = driver._connection
    minor = int(connection["protocol_version"][-1])
    item, node = driver._item, driver._structure_item

    def protocol():
        return node(0x420069, [item(0x42006A, "integer", 2), item(0x42006B, "integer", minor)])

    if operation == CREATE:
        payload_fields = [
            item(0x420057, "enumeration", 2),
            node(0x420125, [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 256),
                            item(0x42002C, "integer", 4)]),
        ]
    elif operation in OWNED_KEY_OPERATIONS:
        payload_fields = [item(0x420094, "text", identifier)]
        if operation == GET_ATTRIBUTES:
            payload_fields.append(item(0x42013B, "enumeration", 0x42002A))
        if operation == CHECK:
            payload_fields.append(item(0x42002C, "integer", 4))
        if operation == REVOKE:
            payload_fields.append(node(0x420081, [item(0x420082, "enumeration", 6)]))
    elif operation == 0x08:
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
    payload_matches = True
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
                    payload_matches = payload is not None
                    if payload is not None:
                        with payload:
                            payload_matches = (expected(payload) if expected is not None
                                               else _matches(payload, operation, protocol))
    if not payload_matches:
        return {**_failure("Generic success payload did not satisfy the case."),
                "result_status_code": status, "result_reason_code": reason}
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


def _text_identifier(payload):
    from kmipkit import ttlv

    result = None
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != 0x420094:
                continue
            if result is not None or ttlv.ttlv_item_view_type(field) != ttlv.ItemType.TextString:
                return None
            with ttlv.ttlv_item_view_value(field) as value:
                result = ttlv.ttlv_value_view_bytes(value).decode("utf-8")
                if not result:
                    return None
    return result


def _has_field(payload, wanted, kind, enumeration=None):
    from kmipkit import ttlv

    found = False
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != wanted:
                continue
            if found or ttlv.ttlv_item_view_type(field) != getattr(ttlv.ItemType, kind):
                return False
            if enumeration is not None:
                with ttlv.ttlv_item_view_value(field) as value:
                    if ttlv.ttlv_value_view_enumeration(value) != enumeration:
                        return False
            found = True
    return found


def _matches_check_usage(payload):
    from kmipkit import ttlv

    found = False
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != 0x42002C:
                continue
            if found or ttlv.ttlv_item_view_type(field) != ttlv.ItemType.Integer:
                return False
            with ttlv.ttlv_item_view_value(field) as value:
                if not (ttlv.ttlv_value_view_integer(value) & 4):
                    return False
            found = True
    return True


def _has_length_reference(payload):
    from kmipkit import ttlv

    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) == 0x42013B and ttlv.ttlv_item_view_type(field) == ttlv.ItemType.Enumeration:
                with ttlv.ttlv_item_view_value(field) as value:
                    if ttlv.ttlv_value_view_enumeration(value) == 0x42002A:
                        return True
    return False


def _has_length_attribute(payload):
    from kmipkit import ttlv

    if not _has_field(payload, 0x420125, "Structure"):
        return False
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != 0x420125:
                continue
            with ttlv.ttlv_item_view_value(field) as value, ttlv.ttlv_value_view_structure(value) as attrs:
                if ttlv.ttlv_structure_view_item_count(attrs) != 1 or not _has_field(attrs, 0x42002A, "Integer"):
                    return False
                with ttlv.ttlv_structure_view_item_at(attrs, 0) as length, ttlv.ttlv_item_view_value(length) as size:
                    return ttlv.ttlv_value_view_integer(size) == 256
    return False


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
