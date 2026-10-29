"""Documentation contracts for the current T073 reconciliation slice."""

from __future__ import annotations

import json
import re
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


def _markdown_section(document: str, heading: str) -> str | None:
    match = re.search(
        rf"(?ms)^### {re.escape(heading)}\n(.*?)(?=^### |\Z)", document
    )
    return None if match is None else match.group(1)


class T073DocumentationReconciliationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_dependency_matrix_reports_declared_hash_mac_and_signature_operations(self) -> None:
        matrix_path = (
            ROOT
            / "specs"
            / "043-public-api-bindings"
            / "contracts"
            / "operation-dependencies.md"
        )
        matrix = matrix_path.read_text(encoding="utf-8")
        expected_operations = {"Hash", "MAC", "MAC Verify", "Sign", "Signature Verify"}
        operations = {
            operation["name"]: operation for operation in self.manifest["operations"]
        }

        for name in expected_operations:
            with self.subTest(operation=name):
                operation = operations[name]
                row = next(
                    line
                    for line in matrix.splitlines()
                    if f"| {name} |" in line and line.startswith("| §6.1.")
                )
                self.assertEqual("declared", operation["apiState"])
                self.assertIn("manifest API state: `declared`", row)
                self.assertIn(
                    "four-language request and response mappings are recorded, and parity verification remains incomplete",
                    row,
                )
                self.assertNotIn("binding remains blocked", row)

    def test_bilingual_guides_separate_reconciled_inventory_from_incomplete_mappings(self) -> None:
        guides = {
            "en": ROOT / "docs" / "user-guide" / "en" / "language-packages.md",
            "es": ROOT / "docs" / "user-guide" / "es" / "paquetes-de-idiomas.md",
        }
        expected = {
            "en": (
                "T011",
                "every observed declaration has a manifest record or an exact approved exclusion",
                "capability mappings remain incomplete",
            ),
            "es": (
                "T011",
                "cada declaración observada tiene un registro en el manifiesto o una exclusión aprobada exacta",
                "los mapeos de la capacidad siguen incompletos",
            ),
        }

        for language, path in guides.items():
            guide_text = path.read_text(encoding="utf-8")
            guide = re.sub(r"\s+", " ", guide_text).lower()
            row = next(
                line.lower() for line in guide_text.splitlines() if "public_binding_surface" in line
            )
            with self.subTest(language=language):
                self.assertIn("blocked", row)
                for phrase in expected[language]:
                    self.assertIn(phrase.lower(), row)
                self.assertNotIn("inventory and mappings are not reconciled", row)

    def test_public_api_reference_scopes_high_level_subset_and_complete_typed_set(self) -> None:
        reference = (ROOT / "docs" / "architecture" / "public-api.md").read_text(
            encoding="utf-8"
        )
        high_level = reference.split("### High level", maxsplit=1)[1].split(
            "### Low-level transport", maxsplit=1
        )[0]
        typed_protocol = reference.split("### Typed protocol", maxsplit=1)[1].split(
            "### Managed-object lifecycle", maxsplit=1
        )[0]
        normalized_high_level = re.sub(r"\s+", " ", high_level)
        normalized_typed = re.sub(r"\s+", " ", typed_protocol)
        operations = {item["name"] for item in self.manifest["operations"]}
        async_followups = {
            "Cancel",
            "Poll",
            "Process",
            "Query Asynchronous Requests",
        }
        expected_typed_set = operations - async_followups
        operations_by_name = {
            item["name"]: item for item in self.manifest["operations"]
        }
        documented_states = {
            name.strip(): state.strip()
            for name, state in re.findall(
                r"(?m)^\|\s*([^|]+?)\s*\|\s*(blocked|declared|verified)\s*\|$",
                reference,
            )
        }

        self.assertEqual(52, len(expected_typed_set))
        self.assertIn(
            "initial Rust operation subset of the complete 52-variant typed request set",
            normalized_high_level,
        )
        self.assertIn("Variants: ", normalized_typed)
        typed_list = normalized_typed.split("Variants: ", maxsplit=1)[1].split(
            ". Cross-language availability", maxsplit=1
        )[0]
        documented_typed_set = {
            name.strip().removeprefix("and ") for name in typed_list.split(",")
        }
        self.assertEqual(expected_typed_set, documented_typed_set)
        self.assertIn(
            "it excludes the four explicit asynchronous follow-up operations",
            normalized_typed.lower(),
        )
        self.assertIn(
            "which remain blocked in the manifest pending kmipkit-0009",
            normalized_typed.lower(),
        )
        for operation in async_followups:
            with self.subTest(excluded_async_operation=operation):
                self.assertIn(operation.lower(), normalized_typed.lower())
                self.assertEqual("blocked", operations_by_name[operation]["apiState"])
                self.assertEqual("blocked", documented_states.get(operation))
        self.assertIn(
            "[java and python package guide](../user-guide/en/language-packages.md)",
            normalized_typed.lower(),
        )
        self.assertIn(
            "[public api manifest](../../specification/api/public-api.json)",
            normalized_typed.lower(),
        )
        self.assertNotIn("manifest status table below", reference)

    def test_t073_full_reconciliation_is_recorded(self) -> None:
        tasks_path = ROOT / "specs" / "043-public-api-bindings" / "tasks.md"
        tasks = tasks_path.read_text(encoding="utf-8")
        t073_line = next(line for line in tasks.splitlines() if "T073 [US1]" in line)
        self.assertTrue(t073_line.startswith("- [x] T073 [US1]"))

        package_metadata = tomllib.loads(
            (ROOT / "bindings" / "c" / "package.toml").read_text(encoding="utf-8")
        )["package"]
        c_package = next(
            item for item in self.manifest["publicPackages"] if item["id"] == "c-kmipkit"
        )
        self.assertEqual(package_metadata["name"], c_package["packageName"])
        self.assertEqual(package_metadata["artifact_family"], c_package["artifactFamily"])
        self.assertEqual(package_metadata["version_source"], c_package["versionSource"])
        self.assertEqual(package_metadata["supported_targets"], c_package["supportedTargets"])

        data_model = (
            ROOT / "specs" / "043-public-api-bindings" / "data-model.md"
        ).read_text(encoding="utf-8")
        self.assertIn("`bindings/c/package.toml`", data_model)
        self.assertIn("does not establish publication", data_model.lower())

        evidence = (
            ROOT / "specs" / "043-public-api-bindings" / "implementation-evidence.md"
        ).read_text(encoding="utf-8")
        reconciliation = _markdown_section(
            evidence, "T073 full reconciliation audit (2026-10-29)"
        )
        self.assertIsNotNone(reconciliation)
        self.assertIn(
            "T073 is complete",
            re.sub(r"\s+", " ", reconciliation),
        )
        self.assertIn("9,520 Rust declarations matched", reconciliation)
        self.assertIn("430/430 C, 789/789 Java, and 768/768 Python", reconciliation)
        self.assertIn("all 12 generated outputs current", reconciliation)
        self.assertIn(
            "does not claim an external registry publication",
            re.sub(r"\s+", " ", reconciliation).lower(),
        )

    def test_named_t073_evidence_section_excludes_later_sections(self) -> None:
        rationale = "C distribution identity still lacks authoritative package metadata."
        document = (
            "### T073 readiness wording reconciliation (2026-10-22)\n"
            "The T073 section is present.\n\n"
            "### T073 unrelated evidence\n"
            f"{rationale}\n"
        )

        section = _markdown_section(
            document, "T073 readiness wording reconciliation (2026-10-22)"
        )

        self.assertIsNotNone(section)
        self.assertNotIn(rationale, section)


if __name__ == "__main__":
    unittest.main()
