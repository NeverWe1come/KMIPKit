from pathlib import Path
import json
import unittest


ROOT = Path(__file__).resolve().parents[3]
DOC = ROOT / "docs" / "architecture" / "ffi-and-bindings.md"
AUDIT_GUIDE = ROOT / "tools" / "api_manifest" / "binding-export-audit.md"
AUDIT_REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "binding-export-report-2026-10-20.json"
)


class CApiDocumentationTests(unittest.TestCase):
    def test_c_api_docs_point_to_the_generated_manifest_and_symbol_inventory(self) -> None:
        documentation = DOC.read_text(encoding="utf-8")

        self.assertIn("../../specification/api/public-api.json", documentation)
        self.assertIn("../../bindings/c/include/kmipkit.h", documentation)
        self.assertIn(
            "../../specs/043-public-api-bindings/reports/binding-export-report-2026-10-20.json",
            documentation,
        )
        self.assertIn("audit_binding_exports.py", documentation)
        self.assertIn("blocked", documentation)

    def test_audit_guide_snapshot_matches_the_generated_report(self) -> None:
        guide = AUDIT_GUIDE.read_text(encoding="utf-8")
        report = json.loads(AUDIT_REPORT.read_text(encoding="utf-8"))
        labels = {"c-kmipkit": "C", "java-kmipkit": "Java", "python-kmipkit": "Python"}

        guide_rows = {
            fields[0]: fields
            for line in guide.splitlines()
            if line.startswith(("| C |", "| Java |", "| Python |"))
            for fields in [[field.strip() for field in line.strip("|").split("|")]]
        }

        for package in report["packages"]:
            with self.subTest(package=package["packageId"]):
                fields = guide_rows[labels[package["packageId"]]]
                counts = package["counts"]
                self.assertEqual(
                    fields[2:5],
                    [
                        f"{counts['observed']:,}",
                        f"{counts['manifest']:,}",
                        f"{counts['exactMatches']:,}",
                    ],
                )

    def test_checked_in_binding_export_report_reconciles_every_package(self) -> None:
        report = json.loads(AUDIT_REPORT.read_text(encoding="utf-8"))
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json").read_text(
                encoding="utf-8"
            )
        )
        declarations_by_package: dict[str, int] = {}
        for declaration in manifest["publicDeclarations"]:
            if declaration["declarationState"] == "blocked":
                continue
            package_id = declaration["packageId"]
            declarations_by_package[package_id] = (
                declarations_by_package.get(package_id, 0) + 1
            )

        for package in report["packages"]:
            with self.subTest(package=package["packageId"]):
                self.assertTrue(package["complete"], "scanner inventory must be complete")
                self.assertTrue(
                    package["matchesManifest"],
                    f"unreconciled exports for {package['packageId']}: "
                    f"missing={len(package['missingFromManifest'])}, "
                    f"stale={len(package['notObserved'])}, "
                    f"kind={len(package['kindMismatches'])}, "
                    f"issues={len(package['issues'])}",
                )
                self.assertEqual([], package["missingFromManifest"])
                self.assertEqual([], package["notObserved"])
                self.assertEqual([], package["kindMismatches"])
                self.assertEqual([], package["issues"])
                self.assertEqual(
                    declarations_by_package[package["packageId"]],
                    package["counts"]["manifest"],
                    "checked-in audit report must match the current manifest",
                )


if __name__ == "__main__":
    unittest.main()
