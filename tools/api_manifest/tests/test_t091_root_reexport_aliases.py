"""T091 contract for approved direct reexports from the Rust facade crate."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT_PATH = (
    ROOT / "specs" / "043-public-api-bindings" / "reports"
    / "rust-export-report-2026-10-30.json"
)
CASES_PATH = Path(__file__).parent / "fixtures" / "t091_root_reexport_alias_cases.json"
TRANSPORT_ALIAS_EXCLUSION_REASON = (
    "This exact root reexport aliases a declaration of the accepted ADR-0014 "
    "Rust-only low-level transport error and delivery contract. Its canonical "
    "declaration remains scoped to kmipkit-transport; the four-language client "
    "facade exposes safe delivery and cause metadata through ClientError under "
    "KMIPKIT-0007."
)
TRANSPORT_ALIAS_EXCLUSION_EVIDENCE_PREFIX = (
    "Exact compiler-observed root alias to "
)
TRANSPORT_ALIAS_APPROVAL_SOURCE = {
    "kind": "accepted-adr",
    "path": "docs/adr/0014-public-transport-exchange-contract.md",
    "status": "Accepted",
    "evidence": "**Status**: Accepted under delegated maintainer authorization on 2026-10-06.",
}


class T091RootReexportAliasTests(unittest.TestCase):
    def test_approved_root_aliases_reuse_exact_canonical_declarations(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(CASES_PATH.read_text(encoding="utf-8"))
        cases = fixture["cases"]
        excluded_aliases = fixture["excludedAliases"]
        self.assertEqual(1, fixture["formatVersion"])
        self.assertEqual(122, len(cases))
        self.assertEqual(17, len(excluded_aliases))

        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["publicDeclarations"]
        }
        capabilities = {
            row["capabilityId"]: row for row in manifest["capabilities"]
        }
        operations = {
            row["operationId"]: row for row in manifest["operations"]
        }
        observations = {
            row["exportName"]: row for row in rust_report["declarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in manifest["approvedExclusions"]
        }
        problems: list[str] = []

        for case in cases:
            root_name = case["rootExportName"]
            canonical_name = case["canonicalExportName"]
            facade = declarations[("rust-kmipkit", root_name)]
            canonical = declarations[(case["canonicalPackageId"], canonical_name)]
            root_observation = observations[root_name]
            canonical_observation = observations[canonical_name]

            self.assertEqual(root_observation["identity"], canonical_observation["identity"])
            self.assertIn(root_name, root_observation["aliases"])
            self.assertIn(canonical_name, root_observation["aliases"])
            self.assertIn(canonical["declarationState"], {"declared", "verified"})
            expected = copy.deepcopy(canonical)
            expected["declarationId"] = case["rootDeclarationId"]
            expected["packageId"] = "rust-kmipkit"
            expected["exportName"] = root_name
            expected["languageMappings"]["rust"] = {
                "packageId": "rust-kmipkit",
                "name": root_name,
            }
            if facade != expected:
                problems.append(f"{root_name} does not match {canonical_name}")

            capability = capabilities.get(canonical["capabilityId"])
            if capability is not None:
                if root_name not in capability["mappings"]["rust"]:
                    problems.append(f"{root_name} is absent from its capability inventory")
            else:
                operation = operations.get(canonical["capabilityId"])
                if operation is None or case["rootDeclarationId"] not in operation["declarations"]["rust"]:
                    problems.append(f"{root_name} is absent from its operation inventory")

        for case in excluded_aliases:
            root_name = case["rootExportName"]
            canonical_name = case["canonicalExportName"]
            canonical = declarations.get((case["canonicalPackageId"], canonical_name))
            root_observation = observations[root_name]
            canonical_observation = observations[canonical_name]
            self.assertIsNotNone(canonical)
            self.assertEqual(root_observation["identity"], canonical_observation["identity"])
            self.assertIn(root_name, root_observation["aliases"])
            self.assertIn(canonical_name, root_observation["aliases"])
            self.assertEqual("verified", canonical["declarationState"])

            exclusion = exclusions.get(("rust-kmipkit", root_name))
            if exclusion is None:
                problems.append(f"{root_name} lacks its exact ADR-0014 exclusion")
                continue
            if ("rust-kmipkit", root_name) in declarations:
                problems.append(f"{root_name} is both declared and excluded")
            if exclusion["reason"] != TRANSPORT_ALIAS_EXCLUSION_REASON:
                problems.append(f"{root_name} has an imprecise transport exclusion reason")
            if exclusion["approvalSource"] != TRANSPORT_ALIAS_APPROVAL_SOURCE:
                problems.append(f"{root_name} lacks accepted ADR-0014 exclusion authority")
            evidence = exclusion.get("evidence", "")
            if not evidence.startswith(TRANSPORT_ALIAS_EXCLUSION_EVIDENCE_PREFIX + canonical_name):
                problems.append(f"{root_name} lacks exact canonical-alias evidence")
            for requirement_id in (
                "KMIPKIT-0007-FR-005",
                "KMIPKIT-0007-FR-013",
                "KMIPKIT-0043-FR-010",
                "KMIPKIT-0043-FR-020",
                "KMIPKIT-0043-FR-021",
            ):
                if requirement_id not in evidence:
                    problems.append(f"{root_name} exclusion omits {requirement_id}")

        self.assertEqual([], problems, f"{len(problems)} root alias mappings are unresolved")


if __name__ == "__main__":
    unittest.main()
