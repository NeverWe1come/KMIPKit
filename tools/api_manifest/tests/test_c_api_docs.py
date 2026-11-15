from pathlib import Path
import json
import unittest
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
DOC = ROOT / "docs" / "architecture" / "ffi-and-bindings.md"
AUDIT_GUIDE = ROOT / "tools" / "api_manifest" / "binding-export-audit.md"
AUDIT_REPORT_PATH = Path(
    "specs/043-public-api-bindings/reports/binding-export-report-2026-11-15-versioned-catalogs.json"
)
AUDIT_REPORT = ROOT / AUDIT_REPORT_PATH


def _load_audit_report():
    return json.loads(AUDIT_REPORT.read_text(encoding="utf-8"))


def _audited_declarations_by_package(
    manifest: dict[str, Any],
) -> dict[str, list[dict[str, Any]]]:
    stable_c_error_codes = {
        category["cCode"]
        for category in manifest["errorCategories"]
        if category.get("stableCHeaderCode") is True
    }
    declarations: dict[str, list[dict[str, Any]]] = {}
    for declaration in manifest["publicDeclarations"]:
        if (
            declaration["packageId"] == "c-kmipkit"
            and declaration["declarationState"] == "blocked"
            and declaration["kind"] != "c-symbol"
            and declaration["exportName"] not in stable_c_error_codes
        ):
            continue
        declarations.setdefault(declaration["packageId"], []).append(declaration)
    return declarations


class CApiDocumentationTests(unittest.TestCase):
    def test_c_api_docs_point_to_the_generated_manifest_and_symbol_inventory(self) -> None:
        documentation = DOC.read_text(encoding="utf-8")

        self.assertIn("../../specification/api/public-api.json", documentation)
        self.assertIn("../../bindings/c/include/kmipkit.h", documentation)
        self.assertIn(
            f"../../{AUDIT_REPORT_PATH.as_posix()}",
            documentation,
        )
        self.assertIn("audit_binding_exports.py", documentation)
        self.assertIn("blocked", documentation)

    def test_audit_guide_snapshot_matches_the_generated_report(self) -> None:
        guide = AUDIT_GUIDE.read_text(encoding="utf-8")
        report = _load_audit_report()
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
        report = _load_audit_report()
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json").read_text(
                encoding="utf-8"
            )
        )
        declarations_by_package = _audited_declarations_by_package(manifest)

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
                    len(declarations_by_package[package["packageId"]]),
                    package["counts"]["manifest"],
                    "checked-in audit report must match the current manifest",
                )

    def test_get_attribute_list_c_exports_are_present_in_checked_in_report(self) -> None:
        report = _load_audit_report()
        manifest = json.loads(
            (ROOT / "specification" / "api" / "public-api.json").read_text(
                encoding="utf-8"
            )
        )
        c_package = next(
            package for package in report["packages"]
            if package["packageId"] == "c-kmipkit"
        )
        c_declarations = {
            row["exportName"]
            for row in _audited_declarations_by_package(manifest)["c-kmipkit"]
        }
        expected = {
            "kmipkit_client_batch_item_get_attribute_list",
            "kmipkit_client_batch_item_response_get_attribute_list_payload_ttlv",
        }

        self.assertTrue(
            expected <= c_declarations,
            f"Get Attribute List exports missing from manifest: {sorted(expected - c_declarations)}",
        )
        self.assertTrue(c_package["matchesManifest"])
        self.assertEqual(len(c_declarations), c_package["counts"]["manifest"])


if __name__ == "__main__":
    unittest.main()
