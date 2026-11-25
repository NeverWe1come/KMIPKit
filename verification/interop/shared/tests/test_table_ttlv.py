from collections import Counter, defaultdict
import json
from pathlib import Path
import tempfile
import unittest

from verification.interop.shared.tools.cosmian_minimum import ALIASES, build_selection, normal
from verification.interop.shared.tools.matrix.oasis_xml import xml_rows
from verification.interop.shared.tools import table_ttlv


class TableTtlvTests(unittest.TestCase):
    def test_exercises_every_applicable_standard_row_outside_the_operations_table(self):
        for version in table_ttlv.ALLOWED_VERSIONS:
            with self.subTest(version=version):
                _, inventory = build_selection(table_ttlv.ROOT, version)
                vectors = list(table_ttlv.fixture_vectors(table_ttlv.ROOT, (version,)))
                observed = Counter(tuple(target) for fixture in vectors if fixture["expected"] == "passed"
                                   for target in fixture["rows"])
                rejected = {tuple(target) for fixture in vectors if fixture["expected"] == "rejected"
                            for target in fixture["rows"]}
                self.assertEqual(rejected, {("managed_object", "Template"),
                                            ("base_object", "Template-Attribute Structures")})
                for kind in ("managed_object", "transparent_key", "base_object", "attribute"):
                    applicable = {row["name"] for row in inventory["rows"] if row["kind"] == kind
                                  and row["declared_support"] not in {"not_applicable", "vendor_extension"}}
                    self.assertEqual(applicable, {name for category, name in observed if category == kind})
                    if kind in {"managed_object", "transparent_key"}:
                        for name in applicable:
                            self.assertGreaterEqual(observed[kind, name], 2)
                identifiers = {fixture["fixture"] for fixture in vectors}
                self.assertEqual(len(identifiers), len(vectors))

    def test_oasis_and_table_fixtures_cover_all_operations_and_set_modify_separately(self):
        root = table_ttlv.ROOT
        for version in table_ttlv.ALLOWED_VERSIONS:
            with self.subTest(version=version):
                _, inventory = build_selection(root, version)
                normative = {normal(item["operation"]) for item in xml_rows(root, version, defaultdict(list))}
                fixtures = list(table_ttlv.fixture_vectors(root, (version,)))
                for row in inventory["rows"]:
                    if row["kind"] != "operation":
                        continue
                    names = ALIASES.get(("operation", row["name"]), (row["name"],))
                    for operation in names:
                        fixture = normal(operation) + "-request"
                        self.assertTrue(
                            normal(operation) in normative or any(
                                item["fixture"] == fixture and ["operation", row["name"]] in item["rows"]
                                for item in fixtures
                            ),
                            (version, row["name"], operation),
                        )

    def test_fixture_and_boundary_evidence_is_local_and_bound_to_its_version(self):
        root = table_ttlv.ROOT
        _, inventory_20 = build_selection(root, "2.0")
        _, inventory_21 = build_selection(root, "2.1")
        fixture = next(table_ttlv.fixture_vectors(root, ("2.1",)))
        artifact = {
            "library_commit": "a" * 40,
            "artifact_sha256": "b" * 64,
            "construct": table_ttlv.CONSTRUCTORS["rust"],
            "decode": table_ttlv.DECODERS["rust"],
            "version_gate": table_ttlv.GATE_SYMBOLS["rust"],
        }
        observed = {key: fixture[key] for key in ("protocol_version", "fixture", "fixture_sha256", "rows", "expected")}
        observed["checks"] = {"rust": {"construct": "passed", "decode": "passed"}}
        payload = {
            "scope": "local", "api_level": "generic-ttlv", "artifacts": {"rust": artifact},
            "rotate_tag_boundaries": {name: {"rust": {"2.0": False, "2.1": True}}
                                      for name in table_ttlv.rotate_tag_boundaries(root)},
            "legacy_named_values": {name: {"rust": {"2.0": False, "2.1": False}}
                                    for name in table_ttlv.LEGACY_VALUES},
            "fixtures": [observed],
        }
        _, tampered_inventory = build_selection(root, "2.1")
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "table-ttlv.json"
            report.write_text(json.dumps(payload), encoding="utf-8")
            for inventory in (inventory_20, inventory_21):
                table_ttlv.attach_fixture_evidence(root, inventory, [report], ("rust",), "a" * 40)
                rows = {(row["kind"], row["name"]): row for row in inventory["rows"]}
                boundary, = [result for item in rows["attribute", "Rotate Date"]["obligations"]
                             for result in item.get("version_gate_results", [])]
                self.assertEqual(boundary["status"], "passed")
                self.assertEqual(boundary["accepted"], inventory["protocol_version"] == "2.1")
                reserved, = rows["transparent_key", "ECDH Private Key"]["named_value_results"]
                self.assertEqual((reserved["language"], reserved["code"], reserved["status"]), ("rust", 0x10, "passed"))
                target = tuple(fixture["rows"][0])
                matches = [item for item in rows[target]["obligations"] if "local_fixture_results" in item]
                if inventory["protocol_version"] == "2.1":
                    obligation, = matches
                    self.assertEqual((obligation["language"], obligation["api_level"]), ("rust", "generic-ttlv"))
                    result, = obligation["local_fixture_results"]
                    self.assertEqual(result["scope"], "local")
                else:
                    self.assertFalse(matches)
            observed["fixture_sha256"] = "0" * 64
            report.write_text(json.dumps(payload), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "invalid or duplicate"):
                table_ttlv.attach_fixture_evidence(root, tampered_inventory, [report], ("rust",), "a" * 40)


if __name__ == "__main__":
    unittest.main()
