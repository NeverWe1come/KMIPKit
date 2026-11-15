"""Tests for deterministic KMIPKIT-0046 Rust alias reconciliation."""

from __future__ import annotations

import copy
import unittest

from tools.api_manifest.reconcile_0046_aliases import ReconciliationError, reconcile


class Reconcile0046AliasesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.identity = ["kmipkit_protocol", {"path": "src/lib.rs"}, "struct", ["Value"]]
        self.manifest = {
            "publicDeclarations": [
                {
                    "declarationId": "canonical-value",
                    "packageId": "rust-kmipkit-protocol",
                    "kind": "type",
                    "exportName": "kmipkit_protocol::Value",
                    "capabilityId": "generic_ttlv_model",
                    "requirementIds": ["KMIPKIT-0004-FR-001"],
                    "declarationState": "verified",
                    "parityScope": "four-language",
                    "approvalSource": {"kind": "feature-specification", "path": "spec.md", "status": "Approved"},
                    "languageMappings": {
                        "rust": {"packageId": "rust-kmipkit-protocol", "name": "kmipkit_protocol::Value"},
                        "c": {"packageId": "c-kmipkit", "name": "kmipkit_value_t"},
                        "java": {"packageId": "java-kmipkit", "name": "Value"},
                        "python": {"packageId": "python-kmipkit", "name": "Value"},
                    },
                }
            ],
            "approvedExclusions": [
                {
                    "packageId": "rust-kmipkit-ttlv",
                    "exportName": "kmipkit_ttlv::Value::internal_helper",
                    "reason": "Exact implementation detail.",
                    "approvalSource": {"kind": "feature-specification", "path": "spec.md", "status": "Approved"},
                    "evidence": "Canonical exclusion.",
                }
            ],
            "capabilities": [
                {"capabilityId": "generic_ttlv_model", "mappings": {"rust": ["kmipkit_protocol::Value"]}}
            ],
            "operations": [],
        }
        self.report = {
            "complete": True,
            "issues": [],
            "matched": [
                {
                    "packageId": "rust-kmipkit-protocol",
                    "exportName": "kmipkit_protocol::Value",
                    "identity": self.identity,
                }
            ],
            "missingFromManifest": [
                {
                    "packageId": "rust-kmipkit",
                    "exportName": "kmipkit::protocol::Value",
                    "kind": "type",
                    "identity": self.identity,
                    "aliases": ["kmipkit::protocol::Value", "kmipkit_protocol::Value"],
                },
                {
                    "packageId": "rust-kmipkit",
                    "exportName": "kmipkit::ttlv::Value::internal_helper",
                    "kind": "function",
                    "identity": ["method", "internal_helper"],
                    "aliases": [
                        "kmipkit::ttlv::Value::internal_helper",
                        "kmipkit_ttlv::Value::internal_helper",
                    ],
                },
                {
                    "packageId": "rust-kmipkit",
                    "exportName": "kmipkit::protocol",
                    "kind": "module",
                    "identity": ["module", "protocol"],
                    "aliases": ["kmipkit::protocol"],
                },
            ],
        }

    def test_alias_declaration_reuses_capability_mapping_and_adds_exact_path(self) -> None:
        result = reconcile(self.manifest, self.report)
        alias = next(row for row in result["publicDeclarations"] if row["exportName"] == "kmipkit::protocol::Value")

        self.assertEqual(alias["capabilityId"], "generic_ttlv_model")
        self.assertEqual(alias["requirementIds"], ["KMIPKIT-0004-FR-001"])
        self.assertEqual(alias["languageMappings"]["rust"], {
            "packageId": "rust-kmipkit",
            "name": "kmipkit::protocol::Value",
        })
        self.assertEqual(alias["languageMappings"]["c"], self.manifest["publicDeclarations"][0]["languageMappings"]["c"])
        self.assertIn("kmipkit::protocol::Value", result["capabilities"][0]["mappings"]["rust"])

    def test_alias_exclusion_is_exact_and_module_grouping_is_adr_scoped(self) -> None:
        result = reconcile(self.manifest, self.report)
        names = {row["exportName"] for row in result["approvedExclusions"]}

        self.assertIn(("rust-kmipkit", "kmipkit::ttlv::Value::internal_helper"), {
            (row["packageId"], row["exportName"]) for row in result["approvedExclusions"]
        })
        module = next(row for row in result["approvedExclusions"] if row["exportName"] == "kmipkit::protocol")
        self.assertEqual(module["approvalSource"]["path"], "docs/adr/0017-single-dependency-rust-facade.md")
        self.assertEqual(len(names), 3)

    def test_reconciliation_is_deterministic_and_does_not_mutate_inputs(self) -> None:
        original = copy.deepcopy(self.manifest)
        first = reconcile(self.manifest, self.report)
        second = reconcile(self.manifest, self.report)

        self.assertEqual(first, second)
        self.assertEqual(self.manifest, original)

    def test_reconciliation_does_not_depend_on_rustdoc_match_order(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        alternative = copy.deepcopy(manifest["publicDeclarations"][0])
        alternative["declarationId"] = "alternative-value"
        alternative["exportName"] = "kmipkit_protocol::ZValue"
        alternative["capabilityId"] = "alternative_ttlv_model"
        alternative["languageMappings"]["rust"]["name"] = "kmipkit_protocol::ZValue"
        manifest["publicDeclarations"].append(alternative)
        manifest["capabilities"].append({
            "capabilityId": "alternative_ttlv_model",
            "mappings": {"rust": ["kmipkit_protocol::ZValue"]},
        })
        report = copy.deepcopy(self.report)
        report["matched"].append({
            "packageId": "rust-kmipkit-protocol",
            "exportName": "kmipkit_protocol::ZValue",
            "identity": self.identity,
        })
        reversed_report = copy.deepcopy(report)
        reversed_report["matched"].reverse()

        self.assertEqual(
            reconcile(manifest, report),
            reconcile(manifest, reversed_report),
        )

    def test_unmapped_export_fails_closed(self) -> None:
        report = copy.deepcopy(self.report)
        report["missingFromManifest"].append({
            "packageId": "rust-kmipkit",
            "exportName": "kmipkit::protocol::Unowned",
            "kind": "type",
            "identity": ["unknown"],
            "aliases": ["kmipkit::protocol::Unowned"],
        })

        with self.assertRaisesRegex(ReconciliationError, "no exact approved source"):
            reconcile(self.manifest, report)

    def test_incomplete_rustdoc_report_fails_closed(self) -> None:
        report = copy.deepcopy(self.report)
        report["complete"] = False

        with self.assertRaisesRegex(ReconciliationError, "incomplete"):
            reconcile(self.manifest, report)


if __name__ == "__main__":
    unittest.main()
