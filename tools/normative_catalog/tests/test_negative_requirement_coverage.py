"""Every in-scope prohibited requirement keeps real, existing negative verification (KMIPKIT-0125-FR-006)."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
IN_SCOPE = {"client_1_0", "mixed", "profile_conditional"}


def prohibited_in_scope() -> list[dict]:
    catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
    return [
        requirement
        for requirement in catalog["requirements"]
        if requirement["scope_state"] in IN_SCOPE
        and (
            requirement["normative_strength"] == "prohibited"
            or requirement["negative_verification_required"] is True
        )
    ]


def reference_problem(reference: str) -> str | None:
    path, separator, symbol = reference.partition("::")
    file = ROOT / path
    if not file.is_file():
        return f"file {path} does not exist"
    if separator and symbol.rsplit("::", 1)[-1] not in file.read_text(encoding="utf-8", errors="replace"):
        return f"symbol {symbol} is not present in {path}"
    return None


class NegativeRequirementCoverageTests(unittest.TestCase):
    def test_catalog_contains_in_scope_prohibited_requirements(self) -> None:
        self.assertGreater(len(prohibited_in_scope()), 0)

    def test_every_in_scope_prohibited_requirement_has_verification_references(self) -> None:
        missing = [
            requirement["requirement_id"]
            for requirement in prohibited_in_scope()
            if not requirement["verification_refs"] and not requirement["decision_id"]
        ]
        self.assertEqual([], missing, "in-scope prohibited requirements without verification or decision")

    def test_every_verification_reference_points_to_an_existing_file_and_symbol(self) -> None:
        problems = {
            requirement["requirement_id"]: [
                problem
                for reference in requirement["verification_refs"]
                if (problem := reference_problem(reference))
            ]
            for requirement in prohibited_in_scope()
        }
        self.assertEqual({}, {key: value for key, value in problems.items() if value})

    def test_verified_status_never_claims_formal_conformance(self) -> None:
        for requirement in prohibited_in_scope():
            with self.subTest(requirement=requirement["requirement_id"]):
                self.assertNotIn("conformance is claimed", requirement["review_note"].lower())


if __name__ == "__main__":
    unittest.main()
