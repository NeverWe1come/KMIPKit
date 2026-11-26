"""Opt-in observations of Cosmian's HTTPS JSON TTLV extensions."""

import http.client
import json
import os
from pathlib import Path
import ssl
import unittest
from urllib.parse import urlsplit

from verification.interop.shared.tools.cosmian_minimum import ROOT, SUPPORT_TABLE
from verification.interop.shared.tools.table_vendor_extensions import GROUP_MEMBER_VALUES, ROWS


def item(tag, value, kind=None):
    # Cosmian 5.28 selects the decoder when it encounters the value field.
    return {"tag": tag, **({"type": kind} if kind else {}), "value": value}


@unittest.skipUnless(os.environ.get("KMIPKIT_COSMIAN_EXTENSION_TEST_CONFIG"),
                     "requires a disposable Cosmian KMS 5.28 instance")
class CosmianAttributeExtensionsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        config = json.loads(Path(os.environ["KMIPKIT_COSMIAN_EXTENSION_TEST_CONFIG"]).read_text())
        url = urlsplit(config["endpoints"]["https"]["url"])
        if url.scheme != "https" or url.hostname not in {"localhost", "127.0.0.1", "::1"}:
            raise ValueError("extension tests require a disposable HTTPS listener on loopback")
        if url.path != "/kmip/2_1":
            raise ValueError("expected Cosmian's KMIP 2.1 JSON endpoint")
        cls.url = url
        cls.report_path = os.environ.get("KMIPKIT_COSMIAN_EXTENSION_TEST_REPORT")
        cls.observations = {}
        cls.server = None
        if cls.report_path:
            cls.server = {"id": config["server_id"], "product": config["server_product"],
                          "version": config["server_version"],
                          "configuration_fingerprint": config["server_configuration_fingerprint"]}
        tls = config["tls"]
        cls.context = ssl.create_default_context(cafile=tls["ca_cert_path"])
        cls.context.load_cert_chain(tls["client_cert_path"], tls["client_key_path"])

    @classmethod
    def tearDownClass(cls):
        if not cls.report_path:
            return
        source = json.loads((ROOT / SUPPORT_TABLE).read_text(encoding="utf-8"))
        report = {
            "schema_version": "1.0", "scope": "live", "transport": "https", "encoding": "JSON",
            "protocol_version": "2.1", "source_sha256": source["source_sha256"],
            "server": cls.server, "observations": cls.observations,
        }
        with Path(cls.report_path).open("x", encoding="utf-8") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")

    def record(self, tag, **checks):
        observation = self.observations.setdefault(
            tag, {"representation": ROWS[tag][1], "http_checks": {}},
        )
        observation["http_checks"].update(checks)
        return observation

    def exchange(self, operation, *fields):
        connection = http.client.HTTPSConnection(
            self.url.hostname, self.url.port, context=self.context, timeout=5,
        )
        try:
            connection.request("POST", self.url.path, json.dumps(item(operation, list(fields))),
                               {"Content-Type": "application/json"})
            response = connection.getresponse()
            return response.status, response.read()
        finally:
            connection.close()

    def success(self, operation, *fields):
        status, payload = self.exchange(operation, *fields)
        self.assertEqual(200, status, operation)
        response = json.loads(payload)
        self.assertEqual(operation + "Response", response["tag"])
        return response["value"]

    def create_request(self, *attributes):
        return (
            item("ObjectType", "SymmetricKey", "Enumeration"),
            item("Attributes", [
                item("CryptographicAlgorithm", "AES", "Enumeration"),
                item("CryptographicLength", 128, "Integer"),
                item("CryptographicUsageMask", 12, "Integer"),
                *attributes,
            ]),
        )

    def create(self):
        fields = self.success("Create", *self.create_request())
        identifiers = [field for field in fields if field["tag"] == "UniqueIdentifier"]
        self.assertEqual(1, len(identifiers))
        self.assertTrue(identifiers[0]["value"])
        return identifiers[0]

    def test_type_rejection(self):
        cases = (
            (item("AttributeIndex", "fixture", "TextString"), b"Expected Integer value"),
            (item("Critical", 1, "Integer"), b"Expected Boolean value"),
            (item("ObjectGroupMember", "Invalid", "Enumeration"), b"expected valid ObjectGroupMember"),
        )
        for field, diagnostic in cases:
            with self.subTest(attribute=field["tag"]):
                status, payload = self.exchange("Create", *self.create_request(field))
                self.assertEqual(422, status)
                self.assertIn(diagnostic, payload)
                self.record(field["tag"], reject_wrong_type=status)

    def test_legacy_named_extensions_acknowledged_but_unreadable(self):
        for field in (item("AttributeIndex", 7, "Integer"), item("Critical", True, "Boolean")):
            with self.subTest(attribute=field["tag"]):
                identifier = self.create()
                self.assertIn(identifier, self.success("GetAttributes", identifier))
                self.assertIn(identifier, self.success("SetAttribute", identifier, item("NewAttribute", [field])))
                status, payload = self.exchange("GetAttributes", identifier)
                self.assertEqual(422, status)
                diagnostic = f"Unknown tag: {field['tag']}".encode()
                self.assertIn(diagnostic, payload)
                self.record(field["tag"], set=200, read_back=status)
                # The pinned image cannot load the record for Destroy either.
                # The disposable provider's filesystem is discarded at teardown.
                status, payload = self.exchange("Destroy", identifier)
                self.assertEqual(422, status)
                self.assertIn(diagnostic, payload)
                self.record(field["tag"], destroy=status)

    def test_group_member_acknowledged_but_omitted_from_get_attributes(self):
        identifier = self.create()
        try:
            for value in GROUP_MEMBER_VALUES:
                with self.subTest(member=value):
                    field = item("ObjectGroupMember", value, "Enumeration")
                    changed = self.success("SetAttribute", identifier, item("NewAttribute", [field]))
                    self.assertIn(identifier, changed)
                    reply = self.success("GetAttributes", identifier)
                    self.assertIn(identifier, reply)
                    attributes = [field["value"] for field in reply if field["tag"] == "Attributes"]
                    self.assertEqual(1, len(attributes))
                    self.assertFalse(any(field["tag"] == "ObjectGroupMember" for field in attributes[0]))
                    observed = self.record("ObjectGroupMember", set=200, read_back=200)
                    observed.setdefault("omitted_values", []).append(value)
        finally:
            self.assertIn(identifier, self.success("Destroy", identifier))
            self.record("ObjectGroupMember", destroy=200)
