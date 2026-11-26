import json
from pathlib import Path
import tempfile
import unittest

from verification.interop.shared.tools.cosmian_minimum import build_selection
from verification.interop.shared.tools.table_managed_objects import ROOT, attach_evidence, expected_checks, vectors


class ManagedObjectTableFixtureTests(unittest.TestCase):
    def test_typed_managed_object_report_covers_each_applicable_object(self):
        for version in ("2.0", "2.1"):
            _, inventory = build_selection(ROOT, version)
            fixtures = [case for case, _ in vectors(ROOT, (version,))]
            self.assertEqual(56, len(fixtures))
            objects = [row for row in inventory["rows"] if row["kind"] == "managed_object"]
            self.assertEqual({row["name"] for row in objects if row["declared_support"] == "supported"},
                             {case["name"] for case in fixtures})

    def test_import_binds_constructor_and_negative_read_to_typed_cells(self):
        _, inventory = build_selection(ROOT, "2.0")
        artifacts = {"rust": {"library_commit": "a" * 40, "artifact_sha256": "b" * 64}}
        fixtures = [{**case, "checks": {"rust": expected_checks(case, "rust")}}
                    for case, _ in vectors(ROOT, ("2.0",))]
        report = {"scope": "local", "api_level": "typed-protocol", "artifacts": artifacts, "fixtures": fixtures}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "objects.json"
            path.write_text(json.dumps(report), encoding="utf-8")
            attach_evidence(ROOT, inventory, [path], ("rust",), "a" * 40)
            certificate = next(row for row in inventory["rows"]
                               if (row["kind"], row["name"]) == ("managed_object", "Certificate"))
            cell, = [item for item in certificate["obligations"] if "local_managed_object_results" in item]
            self.assertEqual(("rust", "typed-protocol"), (cell["language"], cell["api_level"]))
            complete = next(case for case in cell["local_managed_object_results"] if case["scenario"] == "complete")
            self.assertEqual("kmipkit::protocol::Certificate::try_new", complete["public_api"]["construct"])
            negative = next(case for case in cell["local_managed_object_results"] if case["scenario"] == "wrong-type")
            self.assertEqual({"construct": "not-applicable", "read": "rejected"}, negative["checks"])
            fixtures[0]["object_type"] = 9
            path.write_text(json.dumps(report), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "mismatched or duplicate"):
                attach_evidence(ROOT, inventory, [path], ("rust",), None)
