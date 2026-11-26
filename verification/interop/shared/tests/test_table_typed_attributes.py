import json
from pathlib import Path
import tempfile
import unittest

from verification.interop.shared.tools.cosmian_minimum import build_selection
from verification.interop.shared.tools.table_typed_attributes import ROOT, attach_evidence, cases, expected_checks


class TypedAttributeFixtureTests(unittest.TestCase):
    def test_version_and_type_boundaries(self):
        fixtures = [fixture for fixture, _ in cases(ROOT)]
        for version, count in (("2.0", 54), ("2.1", 61)):
            scoped = [fixture for fixture in fixtures if fixture["protocol_version"] == version]
            self.assertEqual(3 * count, len(scoped))
            self.assertEqual(count, len({fixture["row"] for fixture in scoped}))
            rotate = {fixture["row"] for fixture in scoped if fixture["row"].startswith("Rotate ")}
            self.assertEqual(0 if version == "2.0" else 7, len(rotate))
        self.assertEqual({"construct": "not-applicable", "read": "rejected"}, expected_checks("wrong-type", "rust"))
        self.assertEqual({"construct": "rejected", "read": "rejected"}, expected_checks("wrong-type", "java"))
        self.assertEqual({"construct": "passed", "read": "rejected"}, expected_checks("wrong-tag", "c"))

    def test_import_preserves_the_route_and_negative_evidence(self):
        _, inventory = build_selection(ROOT, "2.1")
        artifact = {"library_commit": "a" * 40, "artifact_sha256": "b" * 64}
        fixtures = [{**case, "checks": {"python": expected_checks(case["scenario"], "python")}}
                    for case, _ in cases(ROOT)]
        report = {"scope": "local", "api_level": "typed-protocol", "artifacts": {"python": artifact},
                  "fixtures": fixtures}
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "attributes.json"
            path.write_text(json.dumps(report), encoding="utf-8")
            attach_evidence(ROOT, inventory, [path], ("python",), artifact["library_commit"])
            covered = [item for row in inventory["rows"] for item in row["obligations"]
                       if "local_typed_attribute_results" in item]
            self.assertEqual(61, len(covered))
            for item in covered:
                self.assertEqual(("python", "typed-protocol"), (item["language"], item["api_level"]))
                for result in item["local_typed_attribute_results"]:
                    self.assertEqual(result["expected_checks"], result["checks"])
            fixtures[-1]["fixture_sha256"] = "0" * 64
            path.write_text(json.dumps(report), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "mismatched or duplicate"):
                attach_evidence(ROOT, inventory, [path], ("python",), None)
