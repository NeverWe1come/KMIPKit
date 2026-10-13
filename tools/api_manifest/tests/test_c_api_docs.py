from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[3]
DOC = ROOT / "docs" / "architecture" / "ffi-and-bindings.md"


class CApiDocumentationTests(unittest.TestCase):
    def test_c_api_docs_point_to_the_generated_manifest_and_symbol_inventory(self) -> None:
        documentation = DOC.read_text(encoding="utf-8")

        self.assertIn("../../specification/api/public-api.json", documentation)
        self.assertIn("../../bindings/c/include/kmipkit.h", documentation)
        self.assertIn(
            "../../specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json",
            documentation,
        )
        self.assertIn("audit_binding_exports.py", documentation)
        self.assertIn("blocked", documentation)


if __name__ == "__main__":
    unittest.main()
