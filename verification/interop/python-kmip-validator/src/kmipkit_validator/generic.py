"""KMIP 2.x exchanges built from generic tags using standard Python TLS."""

from contextlib import ExitStack
import hashlib
from functools import partial
import socket
import ssl
import struct


HASH_INPUT = b"abc"
RNG_SEED_INPUT = b"A" * 32
CREATE, CREATE_KEY_PAIR, REGISTER, RE_KEY, DERIVE_KEY, CERTIFY, RE_CERTIFY = (
    1, 2, 3, 4, 5, 6, 7,
)
GET, GET_ATTRIBUTES, DESTROY, ARCHIVE, RECOVER = 0x0A, 0x0B, 0x14, 0x15, 0x16
CHECK = 0x09
OBTAIN_LEASE, GET_USAGE_ALLOCATION, VALIDATE, CANCEL, POLL = 0x10, 0x11, 0x17, 0x19, 0x1A
RE_KEY_KEY_PAIR, ENCRYPT, DECRYPT, SIGN, SIGNATURE_VERIFY, MAC, MAC_VERIFY = (
    0x1D, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24,
)
CREATE_SPLIT_KEY, JOIN_SPLIT_KEY, IMPORT, EXPORT = 0x28, 0x29, 0x2A, 0x2B
GET_ATTRIBUTE_LIST, ACTIVATE, REVOKE = 0x0C, 0x12, 0x13
ADD_ATTRIBUTE, MODIFY_ATTRIBUTE, DELETE_ATTRIBUTE, SET_ATTRIBUTE = 0x0D, 0x0E, 0x0F, 0x31
DESCRIPTION = 0x4200FC
DESCRIPTION_MUTATIONS = {ADD_ATTRIBUTE, MODIFY_ATTRIBUTE, DELETE_ATTRIBUTE, SET_ATTRIBUTE}
OWNED_KEY_OPERATIONS = {
    CREATE, CHECK, GET, GET_ATTRIBUTES, GET_ATTRIBUTE_LIST, OBTAIN_LEASE,
    ACTIVATE, REVOKE, DESTROY, ARCHIVE, RECOVER,
} | DESCRIPTION_MUTATIONS
EXTENDED_OPERATIONS = {
    CREATE_KEY_PAIR, REGISTER, RE_KEY, DERIVE_KEY, CERTIFY, RE_CERTIFY,
    GET_USAGE_ALLOCATION, VALIDATE, RE_KEY_KEY_PAIR,
    ENCRYPT, DECRYPT, SIGN, SIGNATURE_VERIFY, MAC, MAC_VERIFY,
    CREATE_SPLIT_KEY, JOIN_SPLIT_KEY, IMPORT, EXPORT,
}


TEXT_ATTRIBUTE_FIXTURE = b"kmipkit-attribute-fixture"
CHANGED_DESCRIPTION = b"kmipkit-changed-description"
KEY_ATTRIBUTES = {
    "test-attr-contact-information": (0x420022, "TextString", TEXT_ATTRIBUTE_FIXTURE),
    "test-attr-description": (0x4200FC, "TextString", TEXT_ATTRIBUTE_FIXTURE),
    "test-attr-comment": (0x4200FD, "TextString", TEXT_ATTRIBUTE_FIXTURE),
    "test-attr-cryptographic-algorithm": (0x420028, "Enumeration", 3),
    "test-op-get-attributes": (0x42002A, "Integer", 256),
    "test-attr-cryptographic-length": (0x42002A, "Integer", 256),
    "test-attr-cryptographic-usage-mask": (0x42002C, "Integer", 4),
    "test-attr-object-type": (0x420057, "Enumeration", 2),
    "test-attr-state": (0x42008D, "Enumeration", 1),
    "test-attr-sensitive": (0x420120, "Boolean", True),
    "test-attr-always-sensitive": (0x420121, "Boolean", True),
    "test-attr-extractable": (0x420122, "Boolean", False),
    "test-attr-never-extractable": (0x420123, "Boolean", True),
}


EXTRA_ATTRIBUTE_TAGS = {
    "Activation Date": 0x420001,
    "Alternative Name": 0x4200BF,
    "Always Sensitive": 0x420121,
    "Application Specific Information": 0x420004,
    "Archive Date": 0x420005,
    "Certificate Length": 0x4200AD,
    "Certificate Type": 0x42001D,
    "Comment": 0x4200FD,
    "Compromise Date": 0x420020,
    "Compromise Occurrence Date": 0x420021,
    "Contact Information": 0x420022,
    "Cryptographic Algorithm": 0x420028,
    "Cryptographic Domain Parameters": 0x420029,
    "Cryptographic Length": 0x42002A,
    "Cryptographic Parameters": 0x42002B,
    "Cryptographic Usage Mask": 0x42002C,
    "Deactivation Date": 0x42002F,
    "Description": 0x4200FC,
    "Destroy Date": 0x420033,
    "Digest": 0x420034,
    "Digital Signature Algorithm": 0x4200AE,
    "Extractable": 0x420122,
    "Fresh": 0x4200A8,
    "Initial Date": 0x420039,
    "Key Format Type": 0x420042,
    "Key Value Location": 0x4200B8,
    "Key Value Present": 0x4200BB,
    "Last Change Date": 0x420048,
    "Lease Time": 0x420049,
    "Link": 0x42004A,
    "Name": 0x420053,
    "Never Extractable": 0x420123,
    "NIST Key Type": 0x42013A,
    "Object Group": 0x420056,
    "Object Type": 0x420057,
    "Opaque Data Type": 0x420059,
    "Original Creation Date": 0x4200BC,
    "PKCS#12 Friendly Name": 0x4200FB,
    "Process Start Date": 0x420067,
    "Protect Stop Date": 0x420068,
    "Protection Level": 0x420145,
    "Protection Period": 0x420146,
    "Protection Storage Mask": 0x42015E,
    "Quantum Safe": 0x420147,
    "Random Number Generator": 0x4200DE,
    "Revocation Reason": 0x420081,
    "Rotate Automatic": 0x42016B,
    "Rotate Date": 0x42016D,
    "Rotate Generation": 0x42016E,
    "Rotate Interval": 0x42016A,
    "Rotate Latest": 0x420172,
    "Rotate Name": 0x42016F,
    "Rotate Offset": 0x42016C,
    "Sensitive": 0x420120,
    "Short Unique Identifier": 0x420136,
    "State": 0x42008D,
    "Unique Identifier": 0x420094,
    "Usage Limits": 0x420095,
    "X.509 Certificate Identifier": 0x4200B5,
    "X.509 Certificate Issuer": 0x4200B6,
    "X.509 Certificate Subject": 0x4200B7,
}


def _operation(case, version):
    if version not in {"2.0", "2.1"}:
        return None
    if case["case_id"] == "test-op-ping":
        return 0x3B if version == "2.1" else None
    if case["case_id"] in KEY_ATTRIBUTES:
        return GET_ATTRIBUTES
    if case["case_id"].startswith("test-attr-"):
        attr_name = case.get("requires", {}).get("attribute")
        if attr_name in EXTRA_ATTRIBUTE_TAGS or attr_name in {"Certificate Attributes", "Vendor Attribute"}:
            return GET_ATTRIBUTES
    if case["operation"] == "Get" and (
        case.get("requires", {}).get("key_block_shape") in {"wrapped", "transparent"}
        or case.get("requires", {}).get("key_material_format") in {"10", "11", "20", "21"}
    ):
        return GET
    if case["operation"] == "Query" and "query_function" in case.get("requires", {}):
        return 0x18
    if case["operation"] == "Locate" and "rotate_attribute_ttlv" in case.get("requires", {}):
        return 0x08
    if case.get("requires", {}).get("block_cipher_mode") == "GCM":
        return {"Encrypt": ENCRYPT, "Decrypt": DECRYPT}.get(case["operation"])
    return {"test-op-create": CREATE, "test-op-create-key-pair": CREATE_KEY_PAIR,
            "test-op-register": REGISTER, "test-op-re-key": RE_KEY,
            "test-op-derive-key": DERIVE_KEY, "test-op-certify": CERTIFY,
            "test-op-re-certify": RE_CERTIFY, "test-op-check": CHECK, "test-op-get": GET,
            "test-op-get-attribute-list": GET_ATTRIBUTE_LIST,
            "test-op-activate": ACTIVATE, "test-op-revoke": REVOKE, "test-op-destroy": DESTROY,
            "test-op-obtain-lease": OBTAIN_LEASE, "test-op-get-usage-allocation": GET_USAGE_ALLOCATION,
            "test-op-archive": ARCHIVE, "test-op-recover": RECOVER, "test-op-validate": VALIDATE,
            "test-op-cancel": CANCEL, "test-op-poll": POLL,
            "test-op-re-key-key-pair": RE_KEY_KEY_PAIR,
            "test-op-locate": 0x08, "test-op-query": 0x18, "test-query-attestation-nonce": 0x18,
            "test-op-discover-versions": 0x1E,
            "test-op-rng-retrieve": 0x25, "test-crypto-rng-retrieve-rng-32": 0x25,
            "test-op-rng-seed": 0x26, "test-crypto-rng-seed-rng-32": 0x26,
            "test-op-hash": 0x27, "test-crypto-hash-sha_256-0": 0x27,
            "test-op-encrypt": ENCRYPT, "test-crypto-encrypt-aes-256-cbc": ENCRYPT,
            "test-op-decrypt": DECRYPT, "test-crypto-decrypt-aes-256-cbc": DECRYPT,
            "test-op-sign": SIGN, "test-crypto-sign-rsa-2048-pkcs1_v1_5": SIGN,
            "test-crypto-sign-ecdsa-256-p_256": SIGN,
            "test-op-signature-verify": SIGNATURE_VERIFY,
            "test-crypto-signature-verify-rsa-2048-pkcs1_v1_5": SIGNATURE_VERIFY,
            "test-crypto-signature-verify-ecdsa-256-p_256": SIGNATURE_VERIFY,
            "test-op-mac": MAC, "test-crypto-mac-hmac_sha256-256": MAC,
            "test-op-mac-verify": MAC_VERIFY, "test-crypto-mac-verify-hmac_sha256-256": MAC_VERIFY,
            "test-op-create-split-key": CREATE_SPLIT_KEY,
            "test-op-join-split-key": JOIN_SPLIT_KEY,
            "test-op-import": IMPORT, "test-op-export": EXPORT,
            "test-op-add-attribute": ADD_ATTRIBUTE, "test-op-modify-attribute": MODIFY_ATTRIBUTE,
            "test-op-delete-attribute": DELETE_ATTRIBUTE, "test-op-set-attribute": SET_ATTRIBUTE,
            "test-multipart-hash-sha256": 0x27}.get(case["case_id"])


def supports(case, connection):
    operation = _operation(case, connection["protocol_version"])
    mutation = case["execution"]["mutation"]
    return (
        operation is not None
        and (mutation == "mutating" if operation in (OWNED_KEY_OPERATIONS | EXTENDED_OPERATIONS | {POLL, CANCEL})
             else mutation == "read_only" or operation in (0x25, 0x26)
             or case["case_id"] == "test-multipart-hash-sha256")
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
        elif kind == ttlv.ItemType.LongInteger:
            body = struct.pack(">q", ttlv.ttlv_value_view_long_integer(value))
        elif kind == ttlv.ItemType.Boolean:
            body = struct.pack(">Q", ttlv.ttlv_value_view_boolean(value))
        elif kind == ttlv.ItemType.DateTime:
            body = struct.pack(">q", ttlv.ttlv_value_view_date_time(value))
        elif kind == ttlv.ItemType.Interval:
            body = struct.pack(">I", ttlv.ttlv_value_view_interval(value))
        elif kind == ttlv.ItemType.DateTimeExtended:
            body = struct.pack(">q", ttlv.ttlv_value_view_date_time_extended(value))
        elif kind in (ttlv.ItemType.BigInteger, ttlv.ItemType.ByteString, ttlv.ItemType.TextString):
            body = ttlv.ttlv_value_view_bytes(value)
            if kind == ttlv.ItemType.BigInteger and (not body or len(body) % 8):
                raise ValueError("invalid Big Integer size")
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
    if operation == 0x08 and "rotate_attribute_ttlv" in case.get("requires", {}):
        return _locate_rotate_attribute(driver, case)
    if operation == GET and "key_material_format" in case["requires"]:
        return _run_asymmetric_key(driver, int(case["requires"]["key_material_format"]))
    if operation == GET and "key_block_shape" in case["requires"]:
        return _run_key_block(driver, case["requires"]["key_block_shape"] == "wrapped")
    if operation in OWNED_KEY_OPERATIONS:
        return _run_owned_key(driver, operation, case)
    if case["case_id"] == "test-query-attestation-nonce":
        return _checked_exchange(
            driver, 0x18, expected=lambda _: True, expect_nonce=True,
            custom_payload=[driver._item(0x420074, "enumeration", 7)],
        )
    if operation == 0x18 and "query_function" in case.get("requires", {}):
        return _query_field(driver, case["requires"])
    if operation in {POLL, CANCEL}:
        return driver._run_async_case(case, driver._fixture_name(case), exchange=partial(_async_target, driver))
    if operation in EXTENDED_OPERATIONS:
        return _run_extended(driver, operation, case["requires"])
    if case["case_id"] == "test-multipart-hash-sha256":
        return _multipart_hash(driver)
    return _execute(driver, operation)


def _has_query_structure(payload, tag):
    from kmipkit import ttlv

    for member in _fields(payload):
        with ttlv.ttlv_item_view_tag(member) as member_tag:
            if ttlv.tag_value(member_tag) != tag or ttlv.ttlv_item_view_type(member) != ttlv.ItemType.Structure:
                continue
            with ttlv.ttlv_item_view_value(member) as value, ttlv.ttlv_value_view_structure(value) as structure:
                if ttlv.ttlv_structure_view_item_count(structure) > 0:
                    return True
    return False


def _query_field(driver, requires):
    tag = int(requires["query_result_tag"])
    return _checked_exchange(driver, 0x18, expected=lambda payload: _has_query_structure(payload, tag), custom_payload=[
        driver._item(0x420074, "enumeration", int(requires["query_function"])),
    ])


def wrapping_specification_fields(driver, identifier):
    item, node = driver._item, driver._structure_item
    return [
        item(0x42009E, "enumeration", 1),
        node(0x420036, [item(0x420094, "text", identifier),
                        node(0x42002B, [item(0x420011, "enumeration", 12)])]),
        item(0x4200A3, "enumeration", 1),
    ]


def key_block_matches(object_wrapper, wrapping_id, format_code=7):
    from kmipkit import ttlv

    with ExitStack() as stack:
        def structure(parent, wanted):
            for member in _fields(parent):
                with ttlv.ttlv_item_view_tag(member) as tag:
                    if ttlv.tag_value(tag) != wanted or ttlv.ttlv_item_view_type(member) != ttlv.ItemType.Structure:
                        continue
                value = stack.enter_context(ttlv.ttlv_item_view_value(member))
                return stack.enter_context(ttlv.ttlv_value_view_structure(value))
            raise ValueError("Get did not return the expected Key Block structure")

        def byte_length(parent, wanted, item_type=ttlv.ItemType.ByteString):
            for member in _fields(parent):
                with ttlv.ttlv_item_view_tag(member) as tag:
                    if ttlv.tag_value(tag) != wanted or ttlv.ttlv_item_view_type(member) != item_type:
                        continue
                with ttlv.ttlv_item_view_value(member) as value:
                    return ttlv.ttlv_value_view_byte_length(value)
            return 0

        try:
            asymmetric = format_code != 7
            ec = format_code in {20, 21}
            object_tag = 0x42008F if not asymmetric else 0x420064 if format_code in {10, 20} else 0x42006D
            algorithms = (6, 14, 26) if ec else (4,) if asymmetric else (3,)
            bits = 2048 if asymmetric and not ec else 256
            block = structure(structure(object_wrapper, object_tag), 0x420040)
            if not (any(_has_field(block, 0x420028, "Enumeration", algorithm) for algorithm in algorithms)
                    and _has_field(block, 0x42002A, "Integer", bits)):
                return False
            if wrapping_id is None:
                if not _has_field(block, 0x420042, "Enumeration", format_code) or _has_field(block, 0x420046, "Structure"):
                    return False
                material = structure(structure(block, 0x420045), 0x420043)
                if format_code == 7:
                    return byte_length(material, 0x42003F) == 32
                if ec:
                    if not _has_field(material, 0x420075, "Enumeration", 7):
                        return False
                    return (0 < byte_length(material, 0x42002E, ttlv.ItemType.BigInteger) <= 40
                            if format_code == 20 else byte_length(material, 0x420072) == 65)
                exponent = 0x420063 if format_code == 10 else 0x42006C
                return (byte_length(material, 0x420052, ttlv.ItemType.BigInteger) in {256, 264}
                        and byte_length(material, exponent, ttlv.ItemType.BigInteger) > 0)
            wrapping = structure(block, 0x420046)
            return (byte_length(block, 0x420045) == 40
                    and _has_field(wrapping, 0x42009E, "Enumeration", 1)
                    and _has_field(wrapping, 0x4200A3, "Enumeration", 1)
                    and _field_text(structure(wrapping, 0x420036), 0x420094) == wrapping_id)
        except ValueError:
            return False


def key_block_attributes(driver, wrapping):
    item = driver._item
    return [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 256),
            item(0x42002C, "integer", 48 if wrapping else 12),
            item(0x420122, "boolean", True)]


def _run_key_block(driver, wrapped):
    item, node = driver._item, driver._structure_item
    owned = []
    wrapping_id = None
    active = False

    def create(wrapping):
        data = [item(0x420057, "enumeration", 2), node(0x420125, key_block_attributes(driver, wrapping))]
        result = _checked_exchange(driver, CREATE, custom_payload=data)
        identifier = result.pop("created_identifier", None)
        if identifier:
            owned.append(identifier)
        return identifier if result["status"] == "passed" else None

    def same_id(payload, identifier):
        return _text_identifier(payload) == identifier

    target = _failure("The prerequisite Create request failed.", "not_run")
    try:
        identifier = create(False)
        if identifier is None:
            return target
        if wrapped:
            wrapping_id = create(True)
            if wrapping_id is None:
                return target
            activation = _checked_exchange(driver, ACTIVATE, wrapping_id, lambda p: same_id(p, wrapping_id))
            if activation["status"] != "passed":
                return _failure("The prerequisite Activate request failed.", "not_run")
            active = True
        fields = [item(0x420094, "text", identifier)]
        if wrapped:
            fields.append(node(0x420047, wrapping_specification_fields(driver, wrapping_id)))
        else:
            fields.append(item(0x420042, "enumeration", 7))
        target = _checked_exchange(
            driver, GET, expected=lambda p: same_id(p, identifier) and _has_field(p, 0x420057, "Enumeration", 2)
            and key_block_matches(p, wrapping_id), custom_payload=fields,
        )
    finally:
        if active:
            _checked_exchange(driver, REVOKE, wrapping_id, lambda p: same_id(p, wrapping_id))
        for key in reversed(owned):
            cleanup = _checked_exchange(driver, DESTROY, key, lambda p, key=key: same_id(p, key))
            if cleanup["status"] != "passed":
                target.update(_failure("Cleanup Destroy failed for a Key Block fixture."))
    return target


def pair_key_attributes(driver, format_code):
    item, node = driver._item, driver._structure_item
    ec = format_code in {20, 21}
    common = [item(0x420028, "enumeration", 26 if ec else 4), item(0x42002A, "integer", 256 if ec else 2048)]
    if ec:
        common.append(node(0x420029, [item(0x420075, "enumeration", 7)]))
    return (common, [item(0x42002C, "integer", 1), item(0x420122, "boolean", True)],
            [item(0x42002C, "integer", 2), item(0x420122, "boolean", True)])


def _run_asymmetric_key(driver, format_code):
    item, node = driver._item, driver._structure_item
    common, private, public = pair_key_attributes(driver, format_code)
    owned = []

    def collect(payload):
        owned.extend((_field_text(payload, 0x420066), _field_text(payload, 0x42006F)))
        return all(owned) and owned[0] != owned[1]

    created = _checked_exchange(
        driver, CREATE_KEY_PAIR, expected=collect,
        custom_payload=[node(0x420126, common), node(0x420127, private), node(0x420128, public)],
    )
    outcome = _failure("The prerequisite Create Key Pair request failed.", "not_run")
    try:
        if created["status"] == "passed":
            public_key = format_code in {11, 21}
            identifier = owned[int(public_key)]
            object_type = 3 if public_key else 4
            outcome = _checked_exchange(
                driver, GET, custom_payload=[item(0x420094, "text", identifier),
                                             item(0x420042, "enumeration", format_code)],
                expected=lambda p: _text_identifier(p) == identifier and _has_field(p, 0x420057, "Enumeration", object_type)
                and key_block_matches(p, None, format_code),
            )
    finally:
        for key in dict.fromkeys(key for key in owned if key is not None):
            cleanup = _checked_exchange(driver, DESTROY, key, lambda p, key=key: _text_identifier(p) == key)
            if cleanup["status"] != "passed":
                outcome.update(_failure("Cleanup Destroy failed for a Key Block fixture."))
    return outcome


def _async_target(driver, operation, correlation):
    from kmipkit.protocol_values import CancellationResult

    code = POLL if operation == "Poll" else CANCEL

    def verify(payload):
        if code == POLL:
            return _text_identifier(payload) is not None and _has_field(payload, 0x420057, "Enumeration", 2)
        from kmipkit import ttlv

        return (ttlv.ttlv_structure_view_item_count(payload) == 2
                and _field_bytes(payload, 0x420006) == correlation
                and any(_has_field(payload, 0x420012, "Enumeration", value)
                        for value in (CancellationResult.CANCELED, CancellationResult.UNABLE_TO_CANCEL,
                                      CancellationResult.COMPLETED, CancellationResult.FAILED,
                                      CancellationResult.UNAVAILABLE)))

    result = _checked_exchange(driver, code, expected=verify, custom_payload=[
        driver._item(0x420006, "bytes", correlation),
    ])
    result["_pending"] = result.get("result_status_code") == 2 and result["status"] == "passed"
    return result


def _locate_rotate_attribute(driver, case):
    from kmipkit.protocol import VersionGate

    with driver._rotate_locate_attributes(case) as attributes:
        if case["requires"].get("expected_status") == "NotSent":
            tag = int(case["requires"]["rotate_attribute_ttlv"][:6], 16)
            minor = int(driver._connection["protocol_version"][-1])
            if VersionGate.validate_tag(2, minor, tag):
                return _failure("Unexpected Rotate tag admission at the version boundary.")
            return {"status": "passed", "library_correctness": "passed", "server_compatibility": "unknown",
                    "parity": "not_compared", "diagnostic": "Rotate tag rejected before transport."}
        return _execute(driver, 0x08, locate_attributes=attributes)


def _multipart_hash(driver):
    from kmipkit import ttlv

    correlation = None
    digest = hashlib.sha256(b"KMIP Hash!").digest()
    for stage, chunk in enumerate((b"KMIP ", b"Hash", b"!")):
        def check(payload):
            nonlocal correlation
            if stage == 1:
                return ttlv.ttlv_structure_view_item_count(payload) == 0
            wanted = 0x4200D6 if stage == 0 else 0x4200C2
            if ttlv.ttlv_structure_view_item_count(payload) != 1 or not _has_field(payload, wanted, "ByteString"):
                return False
            with ttlv.ttlv_structure_view_item_at(payload, 0) as field, ttlv.ttlv_item_view_value(field) as value:
                data = ttlv.ttlv_value_view_bytes(value)
                if stage == 0:
                    correlation = data
                    return bool(data)
                return data == digest

        outcome = _execute(driver, 0x27, expected=check, hash_frame=(stage, chunk, correlation))
        if outcome["status"] != "passed":
            break
    return outcome


def _checked_exchange(driver, operation, identifier=None, expected=None, attribute_reference=None, custom_payload=None,
                      *, expect_nonce=False):
    from kmipkit.errors import KmipKitError

    try:
        return _execute(driver, operation, identifier, expected, attribute_reference, custom_payload=custom_payload,
                        expect_nonce=expect_nonce)
    except (OSError, ValueError, RuntimeError, KmipKitError):
        return _failure("The generic TLS exchange or response validation failed.")


def _failure(diagnostic, status="failed"):
    return {"status": status, "library_correctness": "failed" if status == "failed" else "not_checked",
            "server_compatibility": "unknown", "parity": "not_compared", "diagnostic": diagnostic}


def _run_owned_key(driver, operation, case):
    case_id = case["case_id"]
    attr_name = case.get("requires", {}).get("attribute")
    reference = KEY_ATTRIBUTES[case_id][0] if (operation == GET_ATTRIBUTES and case_id in KEY_ATTRIBUTES) else None
    if operation in (MODIFY_ATTRIBUTE, SET_ATTRIBUTE):
        reference = DESCRIPTION
    elif operation == DELETE_ATTRIBUTE:
        reference = 0x4200FD
    created = _checked_exchange(driver, CREATE, attribute_reference=reference)
    identifier = created.pop("created_identifier", None)
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
    if created["status"] == "passed" and operation == OBTAIN_LEASE:
        target = _checked_exchange(
            driver, OBTAIN_LEASE, identifier, lambda payload: same_id(payload)
            and _has_field(payload, 0x420049, "Interval")
            and _has_field(payload, 0x420048, "DateTime"),
        )
    if created["status"] == "passed" and operation == GET_ATTRIBUTES:
        if case_id in KEY_ATTRIBUTES:
            tag, kind, expected = KEY_ATTRIBUTES[case_id]
            target = _checked_exchange(
                driver, GET_ATTRIBUTES, identifier,
                lambda payload: same_id(payload) and _has_key_attribute(payload, tag, kind, expected),
                attribute_reference=tag,
            )
        else:
            item, node = driver._item, driver._structure_item
            if attr_name == "Vendor Attribute":
                refs = [node(0x42013B, [
                    item(0x42009D, "text", "KMIPKit.Interop"),
                    item(0x42000A, "text", "probe"),
                ])]
            elif attr_name == "Certificate Attributes":
                refs = [item(0x42013B, "enumeration", 0x4200B5), item(0x42013B, "enumeration", 0x4200B6)]
            else:
                refs = [item(0x42013B, "enumeration", EXTRA_ATTRIBUTE_TAGS[attr_name])]
            target = _checked_exchange(
                driver, GET_ATTRIBUTES, identifier, same_id,
                custom_payload=[item(0x420094, "text", identifier), *refs],
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
    if created["status"] == "passed" and operation in DESCRIPTION_MUTATIONS:
        target = _mutate_description(driver, operation, identifier)
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


def _mutate_description(driver, operation, identifier):
    attr_tag = 0x4200FD if operation == DELETE_ATTRIBUTE else DESCRIPTION

    def same_id(payload):
        return _text_identifier(payload) == identifier

    def read(expected):
        return _checked_exchange(
            driver, GET_ATTRIBUTES, identifier,
            lambda payload: same_id(payload) and _description_matches(payload, expected, attr_tag),
            attribute_reference=attr_tag,
        )

    initial = None if operation == ADD_ATTRIBUTE else TEXT_ATTRIBUTE_FIXTURE
    if read(initial)["status"] != "passed":
        return _failure("Could not read the initial Description.")
    target = _checked_exchange(driver, operation, identifier, same_id)
    expected = None if operation == DELETE_ATTRIBUTE else CHANGED_DESCRIPTION
    if target["status"] == "passed" and read(expected)["status"] != "passed":
        return _failure("Could not verify the Description after mutation.")
    return target


def _description_matches(payload, expected, attr_tag=DESCRIPTION):
    from kmipkit import ttlv

    count = ttlv.ttlv_structure_view_item_count(payload)
    if expected is None:
        if count < 1:
            return False
        if not _has_field(payload, 0x420125, "Structure"):
            return True
        for field in _fields(payload):
            with ttlv.ttlv_item_view_tag(field) as tag:
                if ttlv.tag_value(tag) != 0x420125:
                    continue
                with ttlv.ttlv_item_view_value(field) as value, ttlv.ttlv_value_view_structure(value) as attrs:
                    for attr in _fields(attrs):
                        with ttlv.ttlv_item_view_tag(attr) as item_tag:
                            if ttlv.tag_value(item_tag) == attr_tag:
                                return False
        return True
    return count >= 2 and _has_key_attribute(payload, attr_tag, "TextString", expected)


def _execute(driver, operation, identifier=None, expected=None, attribute_reference=None, hash_frame=None,
             custom_payload=None, locate_attributes=None, *, expect_nonce=False):
    from kmipkit import messages, ttlv

    connection = driver._connection
    minor = int(connection["protocol_version"][-1])
    item, node = driver._item, driver._structure_item

    def protocol():
        return node(0x420069, [item(0x42006A, "integer", 2), item(0x42006B, "integer", minor)])

    if custom_payload is not None:
        payload_fields = custom_payload
    elif operation == CREATE:
        attrs = [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 256),
                 item(0x42002C, "integer", 4)]
        if attribute_reference in range(0x420120, 0x420124):
            attrs.extend([item(0x420120, "boolean", True), item(0x420122, "boolean", False)])
        if attribute_reference in (0x420022, 0x4200FC, 0x4200FD):
            attrs.append(item(attribute_reference, "text", TEXT_ATTRIBUTE_FIXTURE.decode("ascii")))
        payload_fields = [item(0x420057, "enumeration", 2), node(0x420125, attrs)]
    elif operation in OWNED_KEY_OPERATIONS:
        payload_fields = [item(0x420094, "text", identifier)]
        if operation == GET_ATTRIBUTES:
            payload_fields.append(item(0x42013B, "enumeration", attribute_reference))
        if operation == MODIFY_ATTRIBUTE:
            payload_fields.append(node(0x42013C, [item(DESCRIPTION, "text", TEXT_ATTRIBUTE_FIXTURE.decode("ascii"))]))
        if operation == DELETE_ATTRIBUTE:
            payload_fields.append(item(0x42013B, "enumeration", 0x4200FD))
        if operation in (ADD_ATTRIBUTE, MODIFY_ATTRIBUTE, SET_ATTRIBUTE):
            payload_fields.append(node(0x42013D, [item(DESCRIPTION, "text", CHANGED_DESCRIPTION.decode("ascii"))]))
        if operation == CHECK:
            payload_fields.append(item(0x42002C, "integer", 4))
        if operation == REVOKE:
            payload_fields.append(node(0x420081, [item(0x420082, "enumeration", 6)]))
    elif operation == 0x08:
        payload_fields = [item(0x42004F, "integer", 1), item(0x4200D4, "integer", 0)]
        if locate_attributes is not None:
            payload_fields.append(item(0x420125, "structure", locate_attributes))
        else:
            payload_fields.append(node(0x420125, []))
    elif operation == 0x18:
        payload_fields = [item(0x420074, "enumeration", 1)]
    elif operation == 0x1E:
        payload_fields = [protocol()]
    elif operation == 0x25:
        payload_fields = [item(0x4200C4, "integer", 32)]
    elif operation == 0x26:
        payload_fields = [item(0x4200C2, "bytes", RNG_SEED_INPUT)]
    elif operation == 0x27:
        stage, chunk, correlation = hash_frame if hash_frame is not None else (None, HASH_INPUT, None)
        payload_fields = [node(0x42002B, [item(0x420038, "enumeration", 6)]),
                          item(0x4200C2, "bytes", chunk)]
        if correlation is not None:
            payload_fields.append(item(0x4200D6, "bytes", correlation))
        if stage in (0, 2):
            payload_fields.append(item(0x4200D7 if stage == 0 else 0x4200D8, "boolean", True))
    else:
        payload_fields = []

    header_fields = [protocol()]
    if expect_nonce:
        header_fields.append(item(0x4200D3, "boolean", True))
    header_fields.append(item(0x42000D, "integer", 1))
    with node(0x420078, [
        node(0x420077, header_fields),
        node(0x42000F, [
            item(0x42005C, "enumeration", operation),
            node(0x420079, payload_fields),
        ]),
    ]) as request, ttlv.ttlv_item_view_from_item(request) as request_view, ttlv.codec_limits_defaults() as limits:
        wire = _exchange(connection, _encode(request_view), ttlv.codec_limits_max_message_bytes(limits))
    payload_matches = True
    created_identifier = None
    pending_correlation = None
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
                if batch.operation != operation and not (
                    batch.operation is None and batch.result_status == 1
                ):
                    raise ValueError("generic response operation mismatch")
                status, reason = batch.result_status, batch.result_reason
                if status == 2 and operation in {POLL, CANCEL}:
                    with_payload = batch.response_payload
                    if with_payload is not None:
                        with_payload.close()
                    token = batch.asynchronous_correlation_value
                    if operation != POLL or with_payload is not None or reason is not None or not token:
                        return _failure("Invalid asynchronous target response.")
                    pending_correlation = bytearray(token)
                if status == 0:
                    payload = batch.response_payload
                    payload_matches = payload is not None
                    if payload is not None:
                        with payload:
                            if operation == CREATE:
                                created_identifier = _text_identifier(payload)
                                payload_matches = created_identifier is not None and _has_field(
                                    payload, 0x420057, "Enumeration", 2,
                                )
                            else:
                                payload_matches = (expected(payload) if expected is not None
                                                   else _matches(payload, operation, protocol))
                    if expect_nonce:
                        nonce = header.nonce
                        if nonce is None:
                            payload_matches = False
                        else:
                            with nonce:
                                payload_matches = payload_matches and _nonce_fields_present(nonce)
    details = {"result_status_code": status, "result_reason_code": reason}
    if created_identifier is not None:
        details["created_identifier"] = created_identifier
    if pending_correlation is not None:
        details["_correlation"] = pending_correlation
    if not payload_matches:
        diagnostic = ("Generic Query did not return a usable Nonce." if expect_nonce
                      else "Generic success payload did not satisfy the case.")
        return {**_failure(diagnostic), **details}
    outcome, compatibility = {
        (0, None): ("passed", "supported"),
        (1, 5): ("server_unsupported", "unsupported"),
        (1, 8): ("server_unsupported", "unsupported"),
        (1, 12): ("policy_restricted", "policy_restricted"),
    }.get((status, reason), ("passed", "supported") if pending_correlation else ("failed", "unknown"))
    return {"status": outcome, "server_compatibility": compatibility,
            "library_correctness": "passed", "parity": "not_compared",
            "diagnostic": "generic TTLV request and response validated", **details}


def nonce_in_header(header):
    from kmipkit import ttlv

    for field in _fields(header):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != 0x4200C8 or ttlv.ttlv_item_view_type(field) != ttlv.ItemType.Structure:
                continue
        with ttlv.ttlv_item_view_value(field) as value, ttlv.ttlv_value_view_structure(value) as nonce:
            return _nonce_fields_present(nonce)
    return False


def _nonce_fields_present(nonce):
    from kmipkit import ttlv

    tags = []
    for field in _fields(nonce):
        with ttlv.ttlv_item_view_tag(field) as tag, ttlv.ttlv_item_view_value(field) as value:
            if (ttlv.ttlv_item_view_type(field) != ttlv.ItemType.ByteString
                    or ttlv.ttlv_value_view_byte_length(value) == 0):
                return False
            tags.append(ttlv.tag_value(tag))
    return tags == [0x4200C9, 0x4200CA]


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


def _has_field(payload, wanted, kind, expected=None):
    from kmipkit import ttlv

    found = False
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != wanted:
                continue
            if found or ttlv.ttlv_item_view_type(field) != getattr(ttlv.ItemType, kind):
                return False
            if expected is not None:
                with ttlv.ttlv_item_view_value(field) as value:
                    if kind == "Integer":
                        actual = ttlv.ttlv_value_view_integer(value)
                    else:
                        actual = ttlv.ttlv_value_view_enumeration(value)
                    if actual != expected:
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


def _has_key_attribute(payload, tag_code, kind, expected):
    from kmipkit import ttlv

    if not _has_field(payload, 0x420125, "Structure"):
        return False
    read_value = {
        "Boolean": ttlv.ttlv_value_view_boolean,
        "Integer": ttlv.ttlv_value_view_integer,
        "TextString": ttlv.ttlv_value_view_bytes,
    }.get(kind)
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) != 0x420125:
                continue
            with ttlv.ttlv_item_view_value(field) as value, ttlv.ttlv_value_view_structure(value) as attrs:
                if kind == "Enumeration":
                    return _has_field(attrs, tag_code, kind, expected)
                found = False
                for attr in _fields(attrs):
                    with ttlv.ttlv_item_view_tag(attr) as attr_tag:
                        if ttlv.tag_value(attr_tag) != tag_code:
                            continue
                        if found or ttlv.ttlv_item_view_type(attr) != getattr(ttlv.ItemType, kind):
                            return False
                        with ttlv.ttlv_item_view_value(attr) as attr_value:
                            if read_value(attr_value) != expected:
                                return False
                        found = True
                return found
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
    if operation == 0x26:
        with ttlv.ttlv_structure_view_item_at(payload, 0) as field:
            with ttlv.ttlv_item_view_tag(field) as tag, ttlv.ttlv_item_view_value(field) as value:
                return (ttlv.tag_value(tag) == 0x4200C4
                        and ttlv.ttlv_item_view_type(field) == ttlv.ItemType.Integer
                        and 0 <= ttlv.ttlv_value_view_integer(value) <= len(RNG_SEED_INPUT))
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


def _field_text(payload, wanted):
    from kmipkit import ttlv

    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) == wanted and ttlv.ttlv_item_view_type(field) == ttlv.ItemType.TextString:
                with ttlv.ttlv_item_view_value(field) as value:
                    text = ttlv.ttlv_value_view_bytes(value).decode("utf-8")
                    if text:
                        return text
    return None


def _all_identifiers(payload):
    from kmipkit import ttlv

    ids = []
    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) == 0x420094 and ttlv.ttlv_item_view_type(field) == ttlv.ItemType.TextString:
                with ttlv.ttlv_item_view_value(field) as value:
                    text = ttlv.ttlv_value_view_bytes(value).decode("utf-8")
                    if text:
                        ids.append(text)
    return ids


def _field_bytes(payload, wanted):
    from kmipkit import ttlv

    for field in _fields(payload):
        with ttlv.ttlv_item_view_tag(field) as tag:
            if ttlv.tag_value(tag) == wanted and ttlv.ttlv_item_view_type(field) == ttlv.ItemType.ByteString:
                with ttlv.ttlv_item_view_value(field) as value:
                    data = ttlv.ttlv_value_view_bytes(value)
                    if data:
                        return data
    return None


def _destroy_all(driver, ids):
    item = driver._item
    for obj_id in ids:
        if obj_id:
            _checked_exchange(driver, REVOKE, custom_payload=[
                item(0x420094, "text", obj_id),
                driver._structure_item(0x420081, [item(0x420082, "enumeration", 6)]),
            ])
            _checked_exchange(driver, DESTROY, custom_payload=[item(0x420094, "text", obj_id)])


def _run_extended(driver, operation, requires):
    item, node = driver._item, driver._structure_item
    cleanup = []

    def create_aes(usage_mask, extractable=False):
        attrs = [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 256),
                 item(0x42002C, "integer", usage_mask)]
        if extractable:
            attrs.append(item(0x420122, "boolean", True))
        res = _checked_exchange(driver, CREATE,
                                custom_payload=[item(0x420057, "enumeration", 2), node(0x420125, attrs)])
        created_id = res.pop("created_identifier", None)
        if created_id:
            cleanup.append(created_id)
        return created_id, res

    def create_rsa_pair():
        priv_id, pub_id = None, None

        def check(payload):
            nonlocal priv_id, pub_id
            priv_id = _field_text(payload, 0x420066)
            pub_id = _field_text(payload, 0x42006F)
            return priv_id is not None and pub_id is not None

        res = _checked_exchange(driver, CREATE_KEY_PAIR, expected=check, custom_payload=[
            node(0x420126, [item(0x420028, "enumeration", 4), item(0x42002A, "integer", 2048)]),
            node(0x420127, [item(0x42002C, "integer", 1)]),
            node(0x420128, [item(0x42002C, "integer", 2)]),
        ])
        cleanup.extend(i for i in (priv_id, pub_id) if i)
        return priv_id, pub_id, res

    try:
        if operation in (REGISTER, IMPORT):
            fields = ([item(0x420094, "text", "kmipkit-generic-import")] if operation == IMPORT else []) + [
                item(0x420057, "enumeration", 2),
                *([item(0x420124, "boolean", False)] if operation == IMPORT else []),
                node(0x420125, [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 128),
                                item(0x42002C, "integer", 4)]),
                node(0x42008F, [node(0x420040, [
                    item(0x420042, "enumeration", 1),
                    node(0x420045, [item(0x420043, "bytes", b"\x11" * 16)]),
                ])]),
            ]
            def check_reg(payload):
                obj_id = _text_identifier(payload)
                if obj_id:
                    cleanup.append(obj_id)
                return obj_id is not None
            return _checked_exchange(driver, operation, expected=check_reg, custom_payload=fields)
        if operation in (CREATE_KEY_PAIR, RE_KEY_KEY_PAIR):
            priv_id, _, res = create_rsa_pair()
            if operation == CREATE_KEY_PAIR or res["status"] != "passed":
                return res
            def check_rekey_kp(payload):
                np, nu = _field_text(payload, 0x420066), _field_text(payload, 0x42006F)
                cleanup.extend(i for i in (np, nu) if i)
                return np is not None and nu is not None
            return _checked_exchange(
                driver, RE_KEY_KEY_PAIR, expected=check_rekey_kp,
                custom_payload=[item(0x420066, "text", priv_id), item(0x420058, "interval", 3600)],
            )
        if operation in (RE_KEY, EXPORT, GET_USAGE_ALLOCATION, DERIVE_KEY, CREATE_SPLIT_KEY, JOIN_SPLIT_KEY):
            key_id, res = create_aes(512 if operation == DERIVE_KEY else 12, extractable=(operation == EXPORT))
            if res["status"] != "passed":
                return res
            if operation == RE_KEY:
                def check_rk(payload):
                    nid = _text_identifier(payload)
                    if nid:
                        cleanup.append(nid)
                    return nid is not None and nid != key_id
                return _checked_exchange(driver, RE_KEY, expected=check_rk,
                                         custom_payload=[item(0x420094, "text", key_id), item(0x420058, "interval", 3600)])
            if operation == EXPORT:
                return _checked_exchange(
                    driver, EXPORT,
                    expected=lambda p: _text_identifier(p) == key_id and _has_field(p, 0x42008F, "Structure"),
                    custom_payload=[item(0x420094, "text", key_id), item(0x420042, "enumeration", 1)],
                )
            if operation == GET_USAGE_ALLOCATION:
                return _checked_exchange(
                    driver, GET_USAGE_ALLOCATION,
                    expected=lambda p: _text_identifier(p) == key_id,
                    custom_payload=[item(0x420094, "text", key_id), item(0x420096, "long_integer", 32)],
                )
            if operation == DERIVE_KEY:
                _checked_exchange(driver, ACTIVATE, expected=lambda p, kid=key_id: _text_identifier(p) == kid,
                                  custom_payload=[item(0x420094, "text", key_id)])
                def check_dk(payload):
                    did = _text_identifier(payload)
                    if did:
                        cleanup.append(did)
                    return did is not None
                return _checked_exchange(driver, DERIVE_KEY, expected=check_dk, custom_payload=[
                    item(0x420057, "enumeration", 2), item(0x420094, "text", key_id),
                    item(0x420031, "enumeration", 2), node(0x420032, [item(0x420038, "enumeration", 6)]),
                    node(0x420125, [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 128),
                                    item(0x42002C, "integer", 4)]),
                ])
            parts = []
            def check_split(payload):
                nonlocal parts
                parts = _all_identifiers(payload)
                cleanup.extend(parts)
                return len(parts) == 2
            split_res = _checked_exchange(driver, CREATE_SPLIT_KEY, expected=check_split, custom_payload=[
                item(0x420057, "enumeration", 2), item(0x420094, "text", key_id),
                item(0x42008B, "integer", 2), item(0x42008C, "integer", 2), item(0x42008A, "enumeration", 1),
                node(0x420125, [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 256),
                                item(0x42002C, "integer", 12)]),
            ])
            if operation == CREATE_SPLIT_KEY or split_res["status"] != "passed":
                return split_res
            def check_join(payload):
                jid = _text_identifier(payload)
                if jid:
                    cleanup.append(jid)
                return jid is not None
            return _checked_exchange(driver, JOIN_SPLIT_KEY, expected=check_join, custom_payload=[
                item(0x420057, "enumeration", 2),
                item(0x420094, "text", parts[0]), item(0x420094, "text", parts[1]),
                node(0x420125, [item(0x420028, "enumeration", 3), item(0x42002A, "integer", 256),
                                item(0x42002C, "integer", 12)]),
            ])
        if operation in (CERTIFY, RE_CERTIFY, VALIDATE):
            _, pub_id, res = create_rsa_pair()
            if res["status"] != "passed":
                return _failure("The prerequisite Create Key Pair request failed.", "not_run")
            cert_id = None
            def check_cert(payload):
                nonlocal cert_id
                cert_id = _text_identifier(payload)
                if cert_id:
                    cleanup.append(cert_id)
                return cert_id is not None
            cert_res = _checked_exchange(driver, CERTIFY, expected=check_cert, custom_payload=[
                item(0x420094, "text", pub_id),
                node(0x420125, [item(0x42001D, "enumeration", 1)]),
            ])
            if operation == CERTIFY:
                return cert_res
            if cert_res["status"] != "passed":
                return _failure("The server rejected the prerequisite Certify request.", "not_run")
            if operation == RE_CERTIFY:
                def check_recert(payload):
                    nid = _text_identifier(payload)
                    if nid:
                        cleanup.append(nid)
                    return nid is not None
                return _checked_exchange(driver, RE_CERTIFY, expected=check_recert,
                                         custom_payload=[item(0x420094, "text", cert_id)])
            return _checked_exchange(
                driver, VALIDATE,
                expected=lambda p: _has_field(p, 0x42009B, "Enumeration", 1),
                custom_payload=[item(0x420094, "text", cert_id)],
            )
        if operation in (ENCRYPT, DECRYPT):
            key_id, res = create_aes(12)
            gcm = requires.get("block_cipher_mode") == "GCM"
            if res["status"] != "passed":
                return _failure("The prerequisite Create request failed.", "not_run") if gcm else res
            act = _checked_exchange(driver, ACTIVATE, expected=lambda p, kid=key_id: _text_identifier(p) == kid,
                                    custom_payload=[item(0x420094, "text", key_id)])
            if act["status"] != "passed":
                return _failure("The prerequisite Activate request failed.", "not_run") if gcm else act
            iv = bytes(12) if gcm else b"\x01" * 16
            plaintext = b"A" * 16 if gcm else b"0123456789abcdef"
            aad = b"kmipkit authenticated data"
            cipher, tag = None, None

            def fields(data):
                parameters = [item(0x420011, "enumeration", 9 if gcm else 1),
                              item(0x42005F, "enumeration", 1 if gcm else 3)]
                if gcm:
                    parameters += [item(0x4200CD, "integer", 96), item(0x4200CE, "integer", 16)]
                result = [item(0x420094, "text", key_id), node(0x42002B, parameters),
                          item(0x4200C2, "bytes", data), item(0x42003D, "bytes", iv)]
                if gcm:
                    result.append(item(0x4200FE, "bytes", aad))
                return result

            def check_enc(payload):
                nonlocal cipher, tag
                cipher = _field_bytes(payload, 0x4200C2)
                if gcm:
                    tag = _field_bytes(payload, 0x4200FF)
                    return (_text_identifier(payload) == key_id and cipher is not None
                            and len(cipher) == len(plaintext) and tag is not None and len(tag) == 16)
                return _text_identifier(payload) == key_id and cipher is not None

            enc_res = _checked_exchange(driver, ENCRYPT, expected=check_enc, custom_payload=fields(plaintext))
            if operation == ENCRYPT:
                return enc_res
            if enc_res["status"] != "passed":
                return _failure("The prerequisite Encrypt request failed.", "not_run") if gcm else enc_res
            decrypt_fields = fields(cipher)
            invalid_tag = gcm and requires.get("expected_status") == "OperationFailed"
            if gcm:
                if invalid_tag:
                    tag = bytes([tag[0] ^ 1]) + tag[1:]
                decrypt_fields.append(item(0x4200FF, "bytes", tag))
            return _checked_exchange(
                driver, DECRYPT,
                expected=lambda p: not invalid_tag and _text_identifier(p) == key_id
                and _field_bytes(p, 0x4200C2) == plaintext,
                custom_payload=decrypt_fields,
            )
        if operation in (MAC, MAC_VERIFY):
            mk_res = _checked_exchange(driver, CREATE, custom_payload=[
                item(0x420057, "enumeration", 2),
                node(0x420125, [item(0x420028, "enumeration", 9), item(0x42002A, "integer", 256),
                                item(0x42002C, "integer", 384)]),
            ])
            mac_id = mk_res.pop("created_identifier", None)
            if mac_id:
                cleanup.append(mac_id)
            if mk_res["status"] != "passed":
                return mk_res
            act = _checked_exchange(driver, ACTIVATE, expected=lambda p, kid=mac_id: _text_identifier(p) == kid,
                                    custom_payload=[item(0x420094, "text", mac_id)])
            if act["status"] != "passed":
                return act
            tag = None
            def check_mac(payload):
                nonlocal tag
                tag = _field_bytes(payload, 0x4200C6)
                return _text_identifier(payload) == mac_id and tag is not None
            mac_res = _checked_exchange(driver, MAC, expected=check_mac, custom_payload=[
                item(0x420094, "text", mac_id), node(0x42002B, [item(0x420038, "enumeration", 6)]),
                item(0x4200C2, "bytes", HASH_INPUT),
            ])
            if operation == MAC or mac_res["status"] != "passed":
                return mac_res
            return _checked_exchange(
                driver, MAC_VERIFY,
                expected=lambda p: _has_field(p, 0x42009B, "Enumeration", 1),
                custom_payload=[item(0x420094, "text", mac_id),
                                node(0x42002B, [item(0x420038, "enumeration", 6)]),
                                item(0x4200C2, "bytes", HASH_INPUT), item(0x4200C6, "bytes", tag)],
            )
        if operation in (SIGN, SIGNATURE_VERIFY):
            priv_id, pub_id, res = create_rsa_pair()
            if res["status"] != "passed":
                return res
            for kid in (priv_id, pub_id):
                act = _checked_exchange(driver, ACTIVATE, expected=lambda p, k=kid: _text_identifier(p) == k,
                                        custom_payload=[item(0x420094, "text", kid)])
                if act["status"] != "passed":
                    return act
            sig = None
            def check_sign(payload):
                nonlocal sig
                sig = _field_bytes(payload, 0x4200C3)
                return _text_identifier(payload) == priv_id and sig is not None
            sign_res = _checked_exchange(driver, SIGN, expected=check_sign, custom_payload=[
                item(0x420094, "text", priv_id),
                node(0x42002B, [
                    item(0x420028, "enumeration", 4),
                    item(0x420038, "enumeration", 6),
                    item(0x42005F, "enumeration", 8),
                    item(0x4200AE, "enumeration", 5),
                ]),
                item(0x4200C2, "bytes", HASH_INPUT),
            ])
            if operation == SIGN or sign_res["status"] != "passed":
                return sign_res
            return _checked_exchange(
                driver, SIGNATURE_VERIFY,
                expected=lambda p: _has_field(p, 0x42009B, "Enumeration", 1),
                custom_payload=[item(0x420094, "text", pub_id),
                                node(0x42002B, [
                                    item(0x420028, "enumeration", 4),
                                    item(0x420038, "enumeration", 6),
                                    item(0x42005F, "enumeration", 8),
                                    item(0x4200AE, "enumeration", 5),
                                ]),
                                item(0x4200C2, "bytes", HASH_INPUT), item(0x4200C3, "bytes", sig)],
            )
        return _failure("Unsupported extended generic operation.")
    finally:
        _destroy_all(driver, cleanup)
