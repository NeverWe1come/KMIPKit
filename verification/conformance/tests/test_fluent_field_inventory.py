"""Validate the public API field inventory against the pinned specification sources."""

import unittest
from pathlib import Path

from verification.conformance.fluent_field_inventory import inventory


class FluentFieldInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.versions = inventory(Path(__file__).resolve().parents[3])["versions"]

    def test_every_normative_operation_is_listed_for_each_version(self):
        for version, count in (("2.0", 56), ("2.1", 62), ("3.0", 69)):
            with self.subTest(version=version):
                operations = self.versions[version]["operations"]
                self.assertEqual(count, len(operations))
                self.assertTrue(all(operation["wireValue"] for operation in operations))

    def test_get_exposes_all_five_request_fields_in_each_version(self):
        for version, spec in self.versions.items():
            with self.subTest(version=version):
                operation = next(row for row in spec["operations"] if row["name"] == "Get")
                self.assertEqual(
                    [
                        "Unique Identifier",
                        "Key Format Type",
                        "Key Wrap Type",
                        "Key Compression Type",
                        "Key Wrapping Specification",
                    ],
                    [field["name"] for field in operation["payloads"]["request"]["fields"]],
                )

    def test_special_payload_forms_preserve_their_source_shape(self):
        operations = {row["name"]: row for row in self.versions["3.0"]["operations"]}
        self.assertEqual({"message"}, operations["Put"]["payloads"].keys())
        self.assertEqual({"request"}, operations["Poll"]["payloads"].keys())
        self.assertIn("original operation", operations["Poll"]["responsePayload"])
        self.assertEqual([], operations["Obliterate"]["payloads"]["response"]["fields"])
