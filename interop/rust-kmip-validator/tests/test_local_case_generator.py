"""Regression checks for generated Rust local-model case dispatch."""

import json
from pathlib import Path
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[3]
VALIDATOR = ROOT / "interop/rust-kmip-validator"
CATALOG = ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json"
GENERATED = VALIDATOR / "src/local_cases_generated.rs"
GENERATOR = VALIDATOR / "tools/generate_local_cases.py"


class LocalCaseGeneratorTests(unittest.TestCase):
    def test_every_local_catalog_case_has_generated_dispatch(self):
        completed = subprocess.run(
            ["python3", str(GENERATOR)],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)

        catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
        expected = {
            case["case_id"]
            for case in catalog["test_cases"]
            if case["execution"]["mode"] == "local_model"
        }
        source = GENERATED.read_text(encoding="utf-8")
        actual = set(re.findall(r'^\s*"([^"]+)"\s*=>', source, re.MULTILINE))

        self.assertEqual(expected, actual)
        self.assertNotIn("super::generic_model_check()", source)


if __name__ == "__main__":
    unittest.main()
