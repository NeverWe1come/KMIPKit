"""Source excerpts without standalone REQs keep typed, reviewable dispositions."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from html.parser import HTMLParser
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"


class _PinnedHtmlText(HTMLParser):
    """Collect the rendered text in a pinned source line span."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        self.parts.append(data)

    def normalized_text(self) -> str:
        return " ".join("".join(self.parts).split())


def pinned_source_excerpt(source_path: str, html_line_range: list[int]) -> str:
    """Extract normalized text from the oracle's one-based pinned HTML span."""
    from tools.normative_catalog.audit_sources import _decode_document

    lines = _decode_document((ROOT / source_path).read_bytes()).splitlines()
    parser = _PinnedHtmlText()
    parser.feed("\n".join(lines[html_line_range[0] - 1 : html_line_range[1]]))
    parser.close()
    return parser.normalized_text()


def _collect_strings(value: object) -> set[str]:
    if isinstance(value, str):
        return {value}
    if isinstance(value, list):
        return set().union(*(_collect_strings(item) for item in value)) if value else set()
    if isinstance(value, dict):
        return set().union(*(_collect_strings(item) for item in value.values())) if value else set()
    return set()


class SourceGapContextResolutionTests(unittest.TestCase):
    def test_procedure_steps_and_generic_parameters_are_context_not_modal_reqs(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        procedure_ids = [f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}" for number in range(11, 21)]
        generic_ids = [f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}" for number in range(6, 9)]

        for gap_id in procedure_ids:
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                resolution = gap["source_resolution"]
                self.assertEqual(gap["source_force"], "imperative_procedure_without_formal_modal")
                self.assertEqual(resolution["kind"], "procedure_context")
                self.assertTrue(resolution["context_id"].startswith("KMIPKIT-PROC-SPEC-6.1-"))
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])

        for gap_id in generic_ids:
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                resolution = gap["source_resolution"]
                self.assertEqual(resolution["kind"], "generic_parameter_context")
                self.assertTrue(resolution["context_id"].startswith("KMIPKIT-CONTEXT-SPEC-6.1-"))
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])

        for number in (12, 14, 16):
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}"]
            self.assertEqual(gap["source_resolution"]["related_requirement_ids"], ["KMIPKIT-REQ-SPEC-7.8-001"])

    def test_procedure_context_requires_evidence_or_an_explicit_blocker(self) -> None:
        """KMIP Specification v2.1 §6.1 procedure steps need traceable evidence."""
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        contracts = {row["gap_id"]: row for row in oracle["source_table_contracts"]}

        for number in range(11, 21):
            gap_id = f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}"
            gap = gaps[gap_id]
            resolution = gap["source_resolution"]
            linked_requirement_ids = resolution.get("related_requirement_ids", [])
            linked_requirements = [
                requirements[identifier]
                for identifier in linked_requirement_ids
                if identifier in requirements
            ]
            requirement_evidence = bool(linked_requirements) and all(
                requirement["implementation_refs"] and requirement["verification_refs"]
                for requirement in linked_requirements
            )
            linked_contract_ids = _collect_strings(resolution) & contracts.keys()
            contract_evidence = bool(linked_contract_ids) and all(
                contracts[identifier]["implementation_refs"] and contracts[identifier]["verification_refs"]
                for identifier in linked_contract_ids
            )
            explicit_blocker = gap["scope_disposition"] == "unresolved"

            with self.subTest(gap_id=gap_id):
                if number in (12, 14, 16):
                    self.assertEqual(linked_requirement_ids, ["KMIPKIT-REQ-SPEC-7.8-001"])
                self.assertTrue(
                    requirement_evidence or contract_evidence or explicit_blocker,
                    f"{gap_id} is treated as closed by procedure_context without linked executable evidence or an explicit blocker",
                )

    def test_generic_parameter_context_links_operation_contract_evidence(self) -> None:
        """KMIP Specification v2.1 §6.1 generic instructions need request contracts."""
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        contracts = {row["gap_id"]: row for row in oracle["source_table_contracts"]}

        for number in range(6, 9):
            gap_id = f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}"
            gap = gaps[gap_id]
            resolution = gap["source_resolution"]
            linked_contract_ids = _collect_strings(resolution) & contracts.keys()
            contract_evidence = [
                contracts[identifier]
                for identifier in linked_contract_ids
                if contracts[identifier]["implementation_refs"] and contracts[identifier]["verification_refs"]
            ]
            with self.subTest(gap_id=gap_id):
                self.assertTrue(
                    contract_evidence or gap["scope_disposition"] == "unresolved",
                    f"{gap_id} is treated as closed without explicit links to operation request contracts and their evidence",
                )

    def test_generic_parameter_spans_retain_the_complete_pinned_instruction(self) -> None:
        """KMIP Specification v2.1 §6.1 paragraph spans must preserve exact source text."""
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        for number in range(6, 9):
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}"]
            extracted = pinned_source_excerpt(gap["source_path"], gap["html_line_range"])
            with self.subTest(gap_id=gap["gap_id"]):
                self.assertEqual(gap["source_excerpt"], extracted)
                self.assertTrue(extracted.endswith("any other required parameters"))

    def test_public_validator_rejects_unclosed_context_in_an_isolated_repository_copy(self) -> None:
        """KMIP Specification v2.1 §6.1 oracle gaps must fail public validation."""
        with tempfile.TemporaryDirectory(prefix="kmipkit-context-validator-") as temporary_directory:
            isolated_root = Path(temporary_directory) / "repository"
            clone = subprocess.run(
                ["git", "clone", "--quiet", "--shared", "--no-checkout", str(ROOT), str(isolated_root)],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(clone.returncode, 0, clone.stderr)
            checkout = subprocess.run(
                ["git", "checkout", "--quiet", "HEAD"],
                cwd=isolated_root,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(checkout.returncode, 0, checkout.stderr)

            oracle_path = isolated_root / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
            copied_oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
            gaps = {row["gap_id"]: row for row in copied_oracle["source_only_requirement_gaps"]}
            procedure = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-011"]
            procedure["source_resolution"] = {
                "kind": "procedure_context",
                "context_id": "KMIPKIT-PROC-SPEC-6.1-011",
            }
            oracle_path.write_text(json.dumps(copied_oracle, indent=2) + "\n", encoding="utf-8")

            validation = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "tools.normative_catalog.validate",
                    "--repo-root",
                    str(isolated_root),
                    "--structural-only",
                ],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(
                validation.returncode,
                0,
                "the public validator accepted an isolated oracle copy whose client procedure has no traceable evidence: "
                f"stdout={validation.stdout!r}, stderr={validation.stderr!r}",
            )

    def test_paragraph_data_rule_remains_unresolved_and_lease_policy_maps_to_attribute(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        data_rule = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"]
        self.assertEqual(data_rule["source_resolution"]["kind"], "unresolved_source_interpretation")
        self.assertEqual(data_rule["scope_disposition"], "unresolved")
        self.assertIsNone(data_rule["candidate_requirement_id"])
        self.assertIsNone(data_rule["proposed_requirement_id"])

        lease_policy = gaps["KMIPKIT-SOURCE-GAP-SPEC-4.30-CLIENT-LEASE-READ-ONLY"]
        resolution = lease_policy["source_resolution"]
        self.assertEqual(resolution["kind"], "attribute_policy_mapping")
        self.assertEqual(resolution["element_id"], "KMIPKIT-ELEM-ATTRIBUTE-LEASE-TIME")
        self.assertIsNone(lease_policy["candidate_requirement_id"])
        self.assertIsNone(lease_policy["proposed_requirement_id"])
        lease_time = next(row for row in catalog["elements"] if row["element_id"] == resolution["element_id"])
        self.assertEqual(lease_time["source_modifiable_by_client"], "No")
        self.assertTrue(any("read-only for clients" in row["source_text"] for row in lease_time["source_operation_restrictions"]))


if __name__ == "__main__":
    unittest.main()
