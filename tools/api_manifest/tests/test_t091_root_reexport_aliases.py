"""T091 contract for approved direct reexports from the Rust facade crate."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT_PATH = (
    ROOT / "specs" / "043-public-api-bindings" / "reports"
    / "rust-export-report-2026-10-30.json"
)
CASES_PATH = Path(__file__).parent / "fixtures" / "t091_root_reexport_alias_cases.json"


class T091RootReexportAliasTests(unittest.TestCase):
    def test_approved_root_aliases_reuse_exact_canonical_declarations(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        rust_report = json.loads(RUST_REPORT_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(CASES_PATH.read_text(encoding="utf-8"))
        cases = fixture["cases"]
        self.assertEqual(1, fixture["formatVersion"])
        self.assertEqual(139, len(cases))

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

        self.assertEqual([], problems, f"{len(problems)} root alias mappings are unresolved")

    def test_transport_root_aliases_reuse_the_accepted_rust_only_scope(self) -> None:
        manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        fixture = json.loads(CASES_PATH.read_text(encoding="utf-8"))
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
        package_languages = {
            row["id"]: row["language"]
            for row in manifest["publicPackages"]
        }
        requirements = set(manifest["requirementIds"])

        for case in fixture["cases"]:
            if case["canonicalPackageId"] != "rust-kmipkit-transport":
                continue
            canonical = declarations[(case["canonicalPackageId"], case["canonicalExportName"])]
            alias = copy.deepcopy(canonical)
            alias["declarationId"] = case["rootDeclarationId"]
            alias["packageId"] = "rust-kmipkit"
            alias["exportName"] = case["rootExportName"]
            alias["languageMappings"]["rust"] = {
                "packageId": "rust-kmipkit",
                "name": case["rootExportName"],
            }

            with self.subTest(export=case["rootExportName"]):
                generate._validate_public_declarations(
                    [canonical, alias],
                    capabilities,
                    operations,
                    package_languages,
                    requirements,
                    ROOT,
                )

        case = next(
            row
            for row in fixture["cases"]
            if row["canonicalPackageId"] == "rust-kmipkit-transport"
        )
        canonical = declarations[(case["canonicalPackageId"], case["canonicalExportName"])]
        unrelated_alias = copy.deepcopy(canonical)
        unrelated_alias["declarationId"] = "rust-observed-unrelated-transport-alias"
        unrelated_alias["packageId"] = "rust-kmipkit"
        unrelated_alias["exportName"] = "kmipkit::UnrelatedTransportDeclaration"
        unrelated_alias["languageMappings"]["rust"] = {
            "packageId": "rust-kmipkit",
            "name": unrelated_alias["exportName"],
        }
        with self.assertRaisesRegex(
            generate.ManifestError,
            "Rust-only parity is reserved for accepted ADR-0014 transport declarations",
        ):
            generate._validate_public_declarations(
                [canonical, unrelated_alias],
                capabilities,
                operations,
                package_languages,
                requirements,
                ROOT,
            )


if __name__ == "__main__":
    unittest.main()
