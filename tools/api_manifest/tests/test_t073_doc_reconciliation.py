"""Documentation contracts for the current T073 reconciliation slice."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"


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
        self.assertIn(
            "[java and python package guide](../user-guide/en/language-packages.md)",
            normalized_typed.lower(),
        )
        self.assertIn(
            "[public api manifest](../../specification/api/public-api.json)",
            normalized_typed.lower(),
        )
        self.assertNotIn("manifest status table below", reference)

    def test_t073_stays_open_until_c_package_identity_is_authoritative(self) -> None:
        tasks_path = ROOT / "specs" / "043-public-api-bindings" / "tasks.md"
        tasks = tasks_path.read_text(encoding="utf-8")
        t073_line = next(line for line in tasks.splitlines() if "T073 [US1]" in line)
        self.assertTrue(t073_line.startswith("- [ ] T073 [US1]"))
        self.assertIn(
            "C package identity lacks authoritative metadata; leave T073 open",
            t073_line,
        )
        self.assertNotRegex(tasks, r"(?m)^- \[x\] T073 \[US1\]")

        evidence = (
            ROOT / "specs" / "043-public-api-bindings" / "implementation-evidence.md"
        ).read_text(encoding="utf-8")
        reconciliation = evidence.split(
            "### T073 readiness wording reconciliation", maxsplit=1
        )[1]
        self.assertIn(
            "C distribution identity still lacks authoritative package metadata",
            re.sub(r"\s+", " ", reconciliation),
        )


if __name__ == "__main__":
    unittest.main()
