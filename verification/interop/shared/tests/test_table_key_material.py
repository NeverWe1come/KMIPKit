import json
from pathlib import Path
import tempfile
import unittest

from verification.interop.shared.tools.cosmian_minimum import build_selection
from verification.interop.shared.tools.table_key_material import ROOT, SYMBOLS, attach_key_material_evidence, vectors


class TypedKeyMaterialTableTests(unittest.TestCase):
    def test_fixture_results_attach_only_to_the_verified_typed_route(self):
        _, inventory = build_selection(ROOT, "2.0")
        artifacts = {"rust": {"library_commit": "a" * 40, "artifact_sha256": "b" * 64}}
        fixtures = []
        for case, _ in vectors(ROOT):
            fixtures.append({**case, "checks": {"rust": {"construct": case["expected"], "read": case["expected"]}}})
        self.assertEqual(len(fixtures), 72)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "evidence.json"
            report = {"scope": "local", "api_level": "typed-protocol", "artifacts": artifacts, "fixtures": fixtures}
            path.write_text(json.dumps(report), encoding="utf-8")
            attach_key_material_evidence(ROOT, inventory, [path], ("rust",), "a" * 40)
            observed = [row for row in inventory["rows"]
                        if any("local_key_material_results" in item for item in row["obligations"])]
            self.assertEqual(len(observed), 9)
            for row in observed:
                self.assertEqual(row["kind"], "transparent_key")
                obligation, = [item for item in row["obligations"] if "local_key_material_results" in item]
                self.assertEqual((obligation["language"], obligation["api_level"]), ("rust", "typed-protocol"))
                checks = obligation["local_key_material_results"]
                self.assertEqual({check["scenario"] for check in checks},
                                 {"complete", "missing-field", "wrong-type", "duplicate-field"})
                self.assertEqual(checks[0]["public_api"], dict(zip(("construct", "read"), SYMBOLS["rust"])))
            fixtures[0]["fixture_sha256"] = "0" * 64
            path.write_text(json.dumps(report), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "mismatched or duplicate"):
                attach_key_material_evidence(ROOT, inventory, [path], ("rust",), None)
