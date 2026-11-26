#!/usr/bin/env python3
"""Check table-specific KMIP 2.x wire fixtures against installed generic TTLV APIs."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from xml.etree.ElementTree import Element, fromstring

ROOT = Path(__file__).resolve().parents[4]
if __package__ in (None, ""):
    sys.path.insert(0, str(ROOT))

from verification.interop.shared.tools.cosmian_minimum import ALLOWED_VERSIONS, normal
from verification.interop.shared.tools.matrix.oasis_xml import OASIS_FIXTURES
from verification.interop.shared.tools.oasis_ttlv import (
    CONSTRUCTORS, DECODERS, WireValues, encode, inspect_vectors, vectors as oasis_vectors,
)

# Public synthetic values. These fixtures exercise the KMIP structures and scalar
# codecs. Collectively they are not cryptographic test keys.
GATE_SYMBOLS = {
    "rust": "kmipkit::VersionGate::validate_tag",
    "c": "kmipkit_version_gate_validate_tag",
    "java": "org.kmipkit.protocol.VersionGate.validateTag",
    "python": "kmipkit.protocol.VersionGate.validate_tag",
}

LEGACY_VALUES = {
    "Template": ("object-type", 6),
    "ECDSA Private Key": ("key-format", 0x0E),
    "ECDSA Public Key": ("key-format", 0x0F),
    "ECDH Private Key": ("key-format", 0x10),
    "ECDH Public Key": ("key-format", 0x11),
    "ECMQV Private Key": ("key-format", 0x12),
    "ECMQV Public Key": ("key-format", 0x13),
}
NAME_SYMBOLS = {
    "rust": "kmipkit::protocol::named_values_generated::values::enumeration::{}::name",
    "c": "KMIPKIT_ENUMERATION_{}_name",
    "java": "org.kmipkit.protocol.KmipValues.{}.name",
    "python": "kmipkit.protocol_values.{}.name",
}


def name_symbol(language, family):
    parts = {"key-format": ("key_format_type", "KEY_FORMAT_TYPE", "KeyFormatType"),
             "object-type": ("object_type", "OBJECT_TYPE", "ObjectType")}[family]
    index = {"rust": 0, "c": 1, "java": 2, "python": 2}[language]
    return NAME_SYMBOLS[language].format(parts[index])


KEY_FORMATS = (
    ("Symmetric Key", "SymmetricKey", "AES", 128, "Encrypt",
     (("Key", "ByteString", "00112233445566778899aabbccddeeff"),)),
    ("DSA Private Key", "PrivateKey", "DSA", 1024, "Sign",
     (("P", "BigInteger", "49"), ("Q", "BigInteger", "13"),
      ("G", "BigInteger", "09"), ("X", "BigInteger", "05"))),
    ("DSA Public Key", "PublicKey", "DSA", 1024, "Verify",
     (("P", "BigInteger", "49"), ("Q", "BigInteger", "13"),
      ("G", "BigInteger", "09"), ("Y", "BigInteger", "27"))),
    ("RSA Private Key", "PrivateKey", "RSA", 2048, "Sign",
     (("Modulus", "BigInteger", "008001"), ("PrivateExponent", "BigInteger", "3457"),
      ("PublicExponent", "BigInteger", "010001"))),
    ("RSA Public Key", "PublicKey", "RSA", 2048, "Verify",
     (("Modulus", "BigInteger", "008001"), ("PublicExponent", "BigInteger", "010001"))),
    ("DH Private Key", "PrivateKey", "DH", 2048, "KeyAgreement",
     (("P", "BigInteger", "47"), ("Q", "BigInteger", "23"),
      ("G", "BigInteger", "02"), ("J", "BigInteger", "03"), ("X", "BigInteger", "03"))),
    ("DH Public Key", "PublicKey", "DH", 2048, "KeyAgreement",
     (("P", "BigInteger", "47"), ("Q", "BigInteger", "23"),
      ("G", "BigInteger", "02"), ("J", "BigInteger", "03"), ("Y", "BigInteger", "08"))),
    ("EC Private Key", "PrivateKey", "EC", 256, "Sign",
     (("RecommendedCurve", "Enumeration", "P-256"), ("D", "BigInteger", "01"))),
    ("EC Public Key", "PublicKey", "EC", 256, "Verify",
     (("RecommendedCurve", "Enumeration", "P-256"),
      ("QString", "ByteString",
       "046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296"
       "4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"))),
)


def field(tag, kind, value):
    return Element(tag, {"type": kind, "value": str(value)})


def structure(tag, *children):
    node = Element(tag)
    node.extend(children)
    return node


def protocol_version(version):
    return structure("ProtocolVersion", field("ProtocolVersionMajor", "Integer", 2),
                     field("ProtocolVersionMinor", "Integer", version[-1]))


def message(version, role, operation, *payload, header_fields=(), result_reason=None):
    header = [protocol_version(version)]
    batch = [field("Operation", "Enumeration", operation)]
    if role == "Response":
        header.append(field("TimeStamp", "DateTime", 1700000000))
        status = "OperationFailed" if result_reason else "Success"
        batch.append(field("ResultStatus", "Enumeration", status))
        if result_reason:
            batch.append(field("ResultReason", "Enumeration", result_reason))
    header.extend(header_fields)
    header.append(field("BatchCount", "Integer", 1))
    if role == "Request" or payload:
        batch.append(structure(role + "Payload", *payload))
    return structure(role + "Message", structure(role + "Header", *header), structure("BatchItem", *batch))


def object_messages(version, fixture, object_tag, managed_object, attributes=None):
    object_type = field("ObjectType", "Enumeration", object_tag)
    request_fields = [object_type]
    if attributes is not None:
        request_fields.append(attributes)
    request_fields.append(managed_object)
    yield fixture + "-register", message(version, "Request", "Register", *request_fields)
    identifier = field("UniqueIdentifier", "TextString", "fixture-" + fixture)
    yield fixture + "-get", message(version, "Response", "Get", object_type, identifier, managed_object)


def fixture_records(version, names, messages, targets, *, expected="passed"):
    for fixture, node in messages:
        wire, items = encode(node, names)
        yield {
            "fixture": fixture, "fixture_sha256": hashlib.sha256(wire).hexdigest(),
            "protocol_version": version, "rows": targets, "expected": expected,
            "status": "ready", "ttlv": wire.hex(), "items": items,
        }


def other_objects(root, version):
    # Reuse pinned OASIS payloads containing the X.509 certificate and the
    # Secret Data and Opaque Object values.
    fixtures = root / OASIS_FIXTURES / f"kmip-{version}"
    checksums = {filename: digest for digest, filename in
                 (line.split("  ", 1) for line in (fixtures / "CHECKSUMS.sha256").read_text().splitlines())}
    suffix = version.replace(".", "")
    for index, name, object_tag in ((10, "Certificate", "Certificate"),
                                    (4, "Secret Data", "SecretData"),
                                    (5, "Opaque Data", "OpaqueObject")):
        filename = f"mandatory/BL-M-{index}-{suffix}.xml"
        data = (fixtures / filename).read_bytes()
        if hashlib.sha256(data).hexdigest() != checksums[filename]:
            raise ValueError(f"fixture checksum mismatch: {filename}")
        content = fromstring(data)
        for request in content.findall("RequestMessage"):
            for payload in request.findall("./BatchItem/RequestPayload"):
                obj = payload.find(object_tag)
                if obj is not None:
                    yield name, object_tag, obj, payload.find("Attributes")
                    break
            else:
                continue
            break
        else:
            raise ValueError(f"missing {object_tag} in {filename}")

    def key_block(material_hex, format_type):
        material = field("KeyMaterial", "ByteString", material_hex)
        return structure("KeyBlock", field("KeyFormatType", "Enumeration", format_type),
                         structure("KeyValue", material))

    split_key = structure("SplitKey", field("SplitKeyParts", "Integer", 3),
                          field("KeyPartIdentifier", "Integer", 2),
                          field("SplitKeyThreshold", "Integer", 3),
                          field("SplitKeyMethod", "Enumeration", "XOR"),
                          key_block("0102030405060708090a0b0c0d0e0f10", "Raw"))
    yield "Split Key", "SplitKey", split_key, structure(
        "Attributes", field("CryptographicUsageMask", "Integer", "Encrypt"),
        field("CryptographicAlgorithm", "Enumeration", "AES"), field("CryptographicLength", "Integer", 128))

    pgp_key = structure("PGPKey", field("PGPKeyVersion", "Integer", 4), key_block("0102030405", "Opaque"))
    yield "PGP Key", "PGPKey", pgp_key, structure(
        "Attributes", field("CryptographicUsageMask", "Integer", "Verify"))


def transparent_object_fixture(name, object_tag, algorithm, length, usage, components):
    algorithm_field = field("CryptographicAlgorithm", "Enumeration", algorithm)
    length_field = field("CryptographicLength", "Integer", length)
    attributes = structure("Attributes", field("CryptographicUsageMask", "Integer", usage),
                           algorithm_field, length_field)
    material = structure("KeyMaterial", *(field(*component) for component in components))
    key_block = structure("KeyBlock", field("KeyFormatType", "Enumeration", f"Transparent {name}"),
                          structure("KeyValue", material), algorithm_field, length_field)
    return structure(object_tag, key_block), attributes


def fixture_vectors(root, versions):
    for version in versions:
        names = WireValues(root, version, [])
        for key_format in KEY_FORMATS:
            name, object_tag, *_ = key_format
            obj, attributes = transparent_object_fixture(*key_format)
            managed_name = {"SymmetricKey": "Symmetric Key", "PublicKey": "Public Key",
                            "PrivateKey": "Private Key"}[object_tag]
            fixture = f"transparent-{'-'.join(name.lower().split())}"
            yield from fixture_records(
                version, names, object_messages(version, fixture, object_tag, obj, attributes),
                [["transparent_key", name], ["managed_object", managed_name],
                 ["base_object", "Key Block"], ["base_object", "Key Value"]],
            )

        for name, object_tag, obj, attributes in other_objects(root, version):
            fixture = "-".join(name.lower().split())
            yield from fixture_records(version, names, object_messages(version, fixture, object_tag, obj, attributes),
                                       [["managed_object", name]])

        yield from attribute_vectors(root, version, names)
        yield from context_vectors(version, names)
        yield from operation_vectors(version, names)
        yield from oasis_base_object_vectors(root, version, names)
        yield from legacy_tag_vectors(version, names)


def attribute_vectors(root, version, names):
    directory = root / "verification/interop/shared/vectors"
    fixtures = fromstring((directory / "kmip2x-table-attributes.xml").read_bytes())
    identifier = field("UniqueIdentifier", "TextString", "fixture-attribute-target")
    for entry in fixtures:
        if entry.get("since") == "2.1" and version == "2.0":
            continue
        row = entry.attrib["row"]
        fixture = "-".join(row.lower().split())
        references = [
            structure("AttributeReference", *attribute[:2]) if attribute.tag == "Attribute"
            else field("AttributeReference", "Enumeration", attribute.tag)
            for attribute in entry
        ]
        request = message(version, "Request", "GetAttributes", identifier, *references)
        response = message(version, "Response", "GetAttributes", identifier, structure("Attributes", *entry))
        targets = [["attribute", row]]
        if row == "Vendor Attribute":
            targets.append(["base_object", "Attribute"])
        yield from fixture_records(version, names, ((fixture + "-request", request), (fixture + "-response", response)),
                                   targets)


def context_vectors(version, names):
    masks = structure("ProtectionStorageMasks",
                      field("ProtectionStorageMask", "Integer", "Hardware OnPremises"),
                      field("ProtectionStorageMask", "Integer", "Software OnPremises"))
    attributes = structure("Attributes", field("CryptographicAlgorithm", "Enumeration", "AES"),
                           field("CryptographicLength", "Integer", 256),
                           field("CryptographicUsageMask", "Integer", "Encrypt"))
    request = message(version, "Request", "Create", field("ObjectType", "Enumeration", "SymmetricKey"),
                      attributes, masks)
    yield from fixture_records(version, names, (("protection-storage-masks-create", request),),
                               [["attribute", "Protection Storage Masks"]])

    locate_group_member = message(version, "Request", "Locate", field("ObjectGroupMember", "Enumeration", 1))
    yield from fixture_records(version, names, (("object-group-member-locate", locate_group_member),),
                               [["attribute", "Object Group Member"]])

    critical_msg = message(version, "Request", "Locate", field("CriticalityIndicator", "Boolean", True))
    yield from fixture_records(version, names, (("criticality-indicator-probe", critical_msg),),
                               [["attribute", "Critical"]])

    query_fields = (
        ("Extension Information", "QueryExtensionList",
         structure("ExtensionInformation", field("ExtensionName", "TextString", "org.kmipkit.fixture"),
                   field("ExtensionTag", "Integer", 0x540001),
                   field("ExtensionType", "Enumeration", "TextString"))),
        ("RNG Parameters", "QueryRNGs",
         structure("RNGParameters", field("RNGAlgorithm", "Enumeration", "Unspecified"))),
        ("Profile Information", "QueryProfiles",
         structure("ProfileInformation", field("ProfileName", "Enumeration", "BaselineServer"))),
        ("Validation Information", "QueryValidations",
         structure("ValidationInformation", field("ValidationAuthorityType", "Enumeration", "NISTCMVP"),
                   field("ValidationVersionMajor", "Integer", 3),
                   field("ValidationType", "Enumeration", "Software"), field("ValidationLevel", "Integer", 1))),
        ("Capability Information", "QueryCapabilities",
         structure("CapabilityInformation", field("StreamingCapability", "Boolean", "true"),
                   field("AsynchronousCapability", "Boolean", "false"))),
    )
    for row, query, result in query_fields:
        label = "-".join(row.lower().split())
        request = message(version, "Request", "Query", field("QueryFunction", "Enumeration", query))
        response = message(version, "Response", "Query", result)
        yield from fixture_records(version, names, ((label + "-query", request), (label + "-result", response)),
                                   [["base_object", row]])

    credential = structure("Authentication", structure(
        "Credential", field("CredentialType", "Enumeration", "UsernameAndPassword"),
        structure("CredentialValue", field("Username", "TextString", "fixture-user"),
                  field("Password", "TextString", "public-fixture-password"))))
    query = message(version, "Request", "Query", field("QueryFunction", "Enumeration", "QueryOperations"),
                    header_fields=(credential,))
    yield from fixture_records(version, names, (("credential-query", query),), [["base_object", "Credential"]])

    nonce = structure("Nonce", field("NonceID", "ByteString", "616263"),
                      field("NonceValue", "ByteString", "223344556677889900aabbccddeeff00"))
    query = message(version, "Request", "Query", field("QueryFunction", "Enumeration", "QueryAttestationTypes"),
                    header_fields=(field("AttestationCapableIndicator", "Boolean", "true"),))
    response = message(version, "Response", "Query", field("AttestationType", "Enumeration", "TPMQuote"),
                       header_fields=(nonce,))
    yield from fixture_records(version, names, (("nonce-query", query), ("nonce-result", response)),
                               [["base_object", "Nonce"]])


def operation_vectors(version, names):
    """Supplement profile tests with the operations and aliases missing from those XML cases."""
    def identifier(value):
        return field("UniqueIdentifier", "TextString", "fixture-" + value)

    def split(value):
        return identifier(f"split-{value}")

    symmetric_type = field("ObjectType", "Enumeration", "SymmetricKey")
    attributes = structure("Attributes", field("CryptographicAlgorithm", "Enumeration", "AES"),
                           field("CryptographicLength", "Integer", 128),
                           field("CryptographicUsageMask", "Integer", "Encrypt"))
    block = structure("KeyBlock", field("KeyFormatType", "Enumeration", "Raw"),
                      structure("KeyValue", field("KeyMaterial", "ByteString", "00112233445566778899aabbccddeeff")),
                      field("CryptographicAlgorithm", "Enumeration", "AES"),
                      field("CryptographicLength", "Integer", 128))
    key = structure("SymmetricKey", block)
    secret_data = structure("SecretData", field("SecretDataType", "Enumeration", "Password"),
                            structure("KeyBlock", field("KeyFormatType", "Enumeration", "Raw"),
                                      structure("KeyValue", field("KeyMaterial", "ByteString", "01020304"))))
    correlation = field("AsynchronousCorrelationValue", "ByteString", "010203040506")
    private_key = field("PrivateKeyUniqueIdentifier", "TextString", "fixture-private")
    public_key = field("PublicKeyUniqueIdentifier", "TextString", "fixture-public")
    current_key = identifier("key")
    certificate = identifier("certificate")
    new_key = identifier("replacement")
    new_certificate = identifier("replacement-certificate")
    offered_versions = (protocol_version("2.1"), protocol_version("2.0"))
    cases = (
        ("Archive", "Archive", (current_key,), (current_key,)),
        ("Cancel", "Cancel", (correlation,),
         (correlation, field("CancellationResult", "Enumeration", "Canceled"))),
        ("Certify", "Certify", (identifier("public"),), (certificate,)),
        ("Create Split Key", "CreateSplitKey",
         (symmetric_type, current_key, field("SplitKeyParts", "Integer", 2),
          field("SplitKeyThreshold", "Integer", 2), field("SplitKeyMethod", "Enumeration", "XOR"), attributes),
         (split(1), split(2))),
        ("DeriveKey", "DeriveKey",
         (symmetric_type, current_key, field("DerivationMethod", "Enumeration", "Hash"),
          structure("DerivationParameters", field("HashingAlgorithm", "Enumeration", "SHA256")), attributes),
         (identifier("derived-key"),)),
        ("Discover Versions", "DiscoverVersions", offered_versions, offered_versions),
        ("Export", "Export", (current_key, field("KeyFormatType", "Enumeration", "Raw")),
         (symmetric_type, current_key, attributes, key)),
        ("Get Usage Allocation", "GetUsageAllocation",
         (current_key, field("UsageLimitsCount", "LongInteger", 32)), (current_key,)),
        ("Import", "Import", (current_key, symmetric_type, field("ReplaceExisting", "Boolean", "false"), attributes, key),
         (current_key,)),
        ("Join Split Key", "JoinSplitKey", (symmetric_type, split(1), split(2), attributes), (new_key,)),
        ("Notify", "Notify", (current_key, structure("Attributes", field("State", "Enumeration", "Deactivated"))), ()),
        ("Obtain Lease", "ObtainLease", (current_key,),
         (current_key, field("LeaseTime", "Interval", 30), field("LastChangeDate", "DateTime", 1700000000))),
        ("Poll", "Poll", (correlation,), None),
        ("Put", "Put", (identifier("secret"), field("PutFunction", "Enumeration", "New"), secret_data), ()),
        ("Re-certify", "ReCertify", (certificate,), (new_certificate,)),
        ("Re-key", "ReKey", (current_key, field("Offset", "Interval", 3600)), (new_key,)),
        ("Re-key Key Pair", "ReKeyKeyPair", (private_key, field("Offset", "Interval", 3600)),
         (field("PrivateKeyUniqueIdentifier", "TextString", "fixture-new-private"), public_key)),
        ("Recover", "Recover", (current_key,), (current_key,)),
        ("Set Attribute (Modify)", "SetAttribute",
         (current_key, structure("NewAttribute", field("Description", "TextString", "changed-description"))),
         (current_key,)),
        ("Validate", "Validate", (certificate, field("ValidityDate", "DateTime", 1700000000)),
         (field("ValidityIndicator", "Enumeration", "Valid"),)),
    )
    unsupported = {"Archive", "Cancel", "GetUsageAllocation", "Notify", "ObtainLease", "Poll", "Put", "ReCertify",
                   "Recover"}
    for row, operation, fields, result in cases:
        fixture = operation.lower()
        messages = [(fixture + "-request", message(version, "Request", operation, *fields))]
        if result is not None:
            messages.append((fixture + "-response", message(version, "Response", operation, *result)))
        if operation in unsupported:
            messages.append((fixture + "-unsupported", message(
                version, "Response", operation, result_reason="OperationNotSupported")))
        yield from fixture_records(version, names, messages, [["operation", row]])


def oasis_base_object_vectors(root, version, names):
    """Select contextual request and response steps from the pinned XML scenarios."""
    selections = {
        ("AX-M-2", 1, "RequestMessage"): ("Key Wrapping Specification",),
        ("AX-M-2", 1, "ResponseMessage"): ("Key Wrapping Data",),
        ("SASED-M-1", 0, "ResponseMessage"): ("Server Information",),
        ("CS-AC-M-7", 0, "RequestMessage"): ("Data",),
        ("CS-AC-M-7", 0, "ResponseMessage"): ("Data",),
        ("CS-RNG-M-1", 0, "RequestMessage"): ("Data Length",),
        ("CS-AC-M-1", 1, "ResponseMessage"): ("Signature Data",),
        ("CS-AC-M-2", 1, "RequestMessage"): ("Signature Data",),
        ("CS-AC-M-4", 1, "ResponseMessage"): ("MAC Data",),
        ("CS-AC-M-5", 1, "RequestMessage"): ("MAC Data",),
        ("CS-BC-M-GCM-3", 1, "RequestMessage"): ("Init Indicator",),
        ("CS-BC-M-GCM-3", 1, "ResponseMessage"): ("Correlation Value",),
        ("CS-BC-M-GCM-3", 2, "RequestMessage"): ("Final Indicator", "Correlation Value"),
        ("CS-BC-M-CHACHA20POLY1305-1", 1, "RequestMessage"): ("Authenticated Encryption Additional Data",),
        ("CS-BC-M-CHACHA20POLY1305-1", 1, "ResponseMessage"): ("Authenticated Encryption Tag",),
        ("CS-BC-M-CHACHA20POLY1305-1", 2, "RequestMessage"):
            ("Authenticated Encryption Additional Data", "Authenticated Encryption Tag"),
    }
    suffix = "-" + version.replace(".", "")
    for row in oasis_vectors(root, (version,)):
        fixture = Path(row["fixture"]).stem
        role = row["message"]
        step = row["step"]
        targets = selections.pop((fixture.removesuffix(suffix), step, role), None)
        if targets is None:
            continue
        if row["status"] != "ready" or not {names.tags[normal(name)] for name in targets} <= {
            item[0] for item in row["items"]
        }:
            raise ValueError(f"missing base object in {fixture} step {step} {role}")
        yield {
            "fixture": f"{fixture}-{step}-{role.lower()}",
            "fixture_sha256": hashlib.sha256(bytes.fromhex(row["ttlv"])).hexdigest(),
            "protocol_version": version, "rows": [["base_object", target] for target in targets],
            "expected": "passed", "status": "ready", "ttlv": row["ttlv"], "items": row["items"],
        }
    if selections:
        raise ValueError(f"missing KMIP {version} base object fixtures: {sorted(selections)}")


def legacy_tag_vectors(version, names):
    # KMIP 2.0 and 2.1 reserve the former managed Template and the four
    # Template-Attribute containers. Use complete messages so strict rejection
    # also exercises nested tags.
    names.tags.update({
        "legacytemplate": 0x420090,
        "legacytemplateattribute": 0x420091,
        "legacycommontemplateattribute": 0x42001F,
        "legacyprivatetemplateattribute": 0x420065,
        "legacypublictemplateattribute": 0x42006E,
    })
    registration = message(version, "Request", "Register",
                           field("ObjectType", "Enumeration", 6), structure("LegacyTemplate"))
    yield from fixture_records(version, names, (("reserved-template", registration),),
                               [["managed_object", "Template"]], expected="rejected")

    symmetric_type = field("ObjectType", "Enumeration", "SymmetricKey")
    containers = (
        ("template-attribute", "LegacyTemplateAttribute", "Create", (symmetric_type,)),
        ("common-template-attribute", "LegacyCommonTemplateAttribute", "CreateKeyPair", ()),
        ("private-template-attribute", "LegacyPrivateTemplateAttribute", "CreateKeyPair", ()),
        ("public-template-attribute", "LegacyPublicTemplateAttribute", "CreateKeyPair", ()),
    )
    for fixture, tag, operation, prefix in containers:
        request = message(version, "Request", operation, *prefix, structure(tag))
        yield from fixture_records(version, names, (("reserved-" + fixture, request),),
                                   [["base_object", "Template-Attribute Structures"]], expected="rejected")


def rotate_tag_boundaries(root):
    path = root / "verification/conformance/catalog/kmip-versioned-tags.json"
    assignments = json.loads(path.read_text(encoding="utf-8"))["v2_1_tags"]
    tags = {item["name"]: int(item["wire_value"], 16) for item in assignments
            if item.get("is_object_attribute") and item["name"].startswith("Rotate ")}
    if len(tags) != 7:
        raise ValueError("expected seven KMIP 2.1 Rotate attributes")
    return tags


def check_boundaries(root, runners, artifacts):
    tags = {name: ("tag", code) for name, code in rotate_tag_boundaries(root).items()}
    cases = {**tags, **LEGACY_VALUES}
    results = {name: {} for name in cases}
    for language, executable in runners.items():
        completed = subprocess.run(
            [str(executable.resolve(strict=True)), "--inspect-table-version"],
            input="".join(f"{kind} {code}\n" for kind, code in cases.values()),
            text=True, capture_output=True, check=False,
        )
        lines = completed.stdout.splitlines()
        if completed.returncode != 0 or len(lines) != 1 + 2 * len(cases):
            raise ValueError(f"{language}: version check did not return both results for every input")
        provenance = json.loads(lines[0])
        if any(provenance[key] != artifacts[language][key] for key in ("library_commit", "artifact_sha256")):
            raise ValueError(f"{language}: version gate artifact differs from the fixture artifact")
        for index, name in enumerate(cases):
            pair = [json.loads(line) for line in lines[1 + 2 * index:3 + 2 * index]]
            if any(type(result) is not bool for result in pair):
                raise ValueError(f"{language}: version gate returned a non-Boolean result")
            results[name][language] = dict(zip(ALLOWED_VERSIONS, pair, strict=True))
    return ({name: results[name] for name in tags}, {name: results[name] for name in LEGACY_VALUES})


def fixture_results(rows, runners):
    construction = inspect_vectors(rows, runners, construct=True)
    decode_rows = [{key: value for key, value in row.items() if key != "checks"} for row in rows]
    decoding = inspect_vectors(decode_rows, runners)
    for row, decoded in zip(rows, decode_rows, strict=True):
        row["checks"] = {language: {"construct": status, "decode": decoded["checks"][language]}
                         for language, status in row["checks"].items()}
    artifacts = {}
    for language in runners:
        constructed = {key: value for key, value in construction[language].items() if key != "public_api_symbol"}
        decoded = {key: value for key, value in decoding[language].items() if key != "public_api_symbol"}
        if constructed != decoded:
            raise ValueError(f"{language}: artifact identity changed between fixture checks")
        artifacts[language] = {
            **constructed, "construct": CONSTRUCTORS[language], "decode": DECODERS[language],
            "version_gate": GATE_SYMBOLS[language],
        }
    return artifacts


def attach_fixture_evidence(root, inventory, reports, languages, expected_commit):
    """Join observations to the exact public fixture bytes for the selected table version."""
    version = inventory["protocol_version"]
    expected = {row["fixture"]: row for row in fixture_vectors(root, (version,))}
    obligations = {(row["kind"], row["name"]): row for row in inventory["rows"]}
    seen = set()
    boundaries_seen = set()
    legacy_seen = set()
    tags = rotate_tag_boundaries(root)
    for path in reports:
        report = json.loads(path.read_text(encoding="utf-8"))
        if (report["scope"], report["api_level"]) != ("local", "generic-ttlv"):
            raise ValueError(f"{path.name}: unexpected fixture scope or API")
        for language, artifact in report["artifacts"].items():
            if language not in languages:
                continue
            if (artifact["construct"], artifact["decode"]) != (CONSTRUCTORS[language], DECODERS[language]):
                raise ValueError(f"{path.name}: unexpected {language} fixture API")
            for key, width in (("library_commit", 40), ("artifact_sha256", 64)):
                value = artifact[key]
                if len(value) != width or any(char not in "0123456789abcdef" for char in value):
                    raise ValueError(f"{path.name}: invalid {language} fixture provenance")
            if expected_commit is not None and artifact["library_commit"] != expected_commit:
                raise ValueError(f"{path.name}: fixture evidence does not match the requested library commit")
            if artifact.get("version_gate") != GATE_SYMBOLS[language]:
                raise ValueError(f"{path.name}: unexpected {language} version gate API")
            if set(report["rotate_tag_boundaries"]) != tags.keys():
                raise ValueError(f"{path.name}: missing or unexpected Rotate attribute boundary")
            for name, outcomes in report["rotate_tag_boundaries"].items():
                observed = outcomes[language][version]
                key = name, language, artifact["artifact_sha256"]
                if type(observed) is not bool or key in boundaries_seen:
                    raise ValueError(f"{path.name}: invalid or duplicate {language} boundary evidence for {name}")
                boundaries_seen.add(key)
                obligation = next(item for item in obligations["attribute", name]["obligations"]
                                  if (item["language"], item["api_level"]) == (language, "generic-ttlv"))
                obligation.setdefault("version_gate_results", []).append({
                    "scope": "local", "tag": tags[name], "api_symbol": GATE_SYMBOLS[language],
                    "accepted": observed, "expected": version == "2.1",
                    "status": "passed" if observed == (version == "2.1") else "failed",
                    "library_commit": artifact["library_commit"], "artifact_sha256": artifact["artifact_sha256"],
                })
            if set(report["legacy_named_values"]) != LEGACY_VALUES.keys():
                raise ValueError(f"{path.name}: missing or unexpected legacy Enumeration value")
            for name, outcomes in report["legacy_named_values"].items():
                assigned = outcomes[language][version]
                key = name, language, artifact["artifact_sha256"]
                if type(assigned) is not bool or key in legacy_seen:
                    raise ValueError(f"{path.name}: invalid or duplicate {language} legacy evidence for {name}")
                legacy_seen.add(key)
                family, code = LEGACY_VALUES[name]
                kind = "managed_object" if family == "object-type" else "transparent_key"
                obligations[kind, name].setdefault("named_value_results", []).append({
                    "language": language, "scope": "local", "code": code, "api_symbol": name_symbol(language, family),
                    "assigned": assigned, "expected": False, "status": "failed" if assigned else "passed",
                    "library_commit": artifact["library_commit"], "artifact_sha256": artifact["artifact_sha256"],
                })
            for result in report["fixtures"]:
                if result["protocol_version"] != version:
                    continue
                fixture = result["fixture"]
                reference = expected.get(fixture)
                checks = result["checks"].get(language, {})
                key = fixture, language, artifact["artifact_sha256"]
                if (reference is None or result["fixture_sha256"] != reference["fixture_sha256"] or key in seen
                        or result["rows"] != reference["rows"] or result["expected"] != reference["expected"]
                        or set(checks) != {"construct", "decode"}
                        or not set(checks.values()) <= {"passed", "failed", "rejected"}):
                    raise ValueError(f"{path.name}: invalid or duplicate {language} fixture evidence for {fixture}")
                seen.add(key)
                for kind, name in reference["rows"]:
                    obligation = next(item for item in obligations[kind, name]["obligations"]
                                      if (item["language"], item["api_level"]) == (language, "generic-ttlv"))
                    expects_rejection = reference["expected"] == "rejected"
                    assertion = ("strict construction and decoding reject reserved tag" if expects_rejection
                                 else "complete message and scalar values match")
                    obligation.setdefault("local_fixture_results", []).append({
                        "fixture": fixture, "fixture_sha256": reference["fixture_sha256"],
                        "scope": "local", "assertion": assertion, "expected": reference["expected"],
                        "checks": checks, **artifact,
                    })



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", nargs=2, metavar=("LANGUAGE", "EXECUTABLE"), action="append", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    runners = {language: Path(executable) for language, executable in args.runner}
    if len(runners) != len(args.runner) or not runners.keys() <= DECODERS.keys():
        parser.error("give one runner per supported language")

    try:
        vectors = list(fixture_vectors(ROOT, ALLOWED_VERSIONS))
        artifacts = fixture_results(vectors, runners)
        boundaries, legacy = check_boundaries(ROOT, runners, artifacts)
        report = {
            "rotate_tag_boundaries": boundaries, "legacy_named_values": legacy,
            "scope": "local", "api_level": "generic-ttlv", "artifacts": artifacts,
            "fixtures": [{key: value for key, value in row.items() if key not in {"ttlv", "items", "status"}}
                         for row in vectors],
        }
        with args.output.open("x", encoding="utf-8") as output:
            json.dump(report, output, indent=2)
            output.write("\n")
    except (OSError, ValueError, KeyError) as error:
        parser.exit(2, f"KMIP table fixture check failed: {error}\n")
    failures = [(row["protocol_version"], row["fixture"], language, checks) for row in vectors
                for language, checks in row["checks"].items() if set(checks.values()) != {row["expected"]}]
    for version, fixture, language, checks in failures:
        print(f"{language} KMIP {version} {fixture}: {checks}", file=sys.stderr)
    print(f"Checked {len(vectors)} table fixtures with {len(runners)} installed validators")
    boundary_failures = [(name, language, results) for name, outcomes in boundaries.items()
                         for language, results in outcomes.items() if results != {"2.0": False, "2.1": True}]
    for name, language, results in boundary_failures:
        print(f"{language} {name}: unexpected version gate results {results}", file=sys.stderr)
    return int(bool(failures or boundary_failures))


if __name__ == "__main__":
    raise SystemExit(main())
