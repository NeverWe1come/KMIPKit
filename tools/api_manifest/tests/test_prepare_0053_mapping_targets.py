"""Deterministic inventory preparation for exact KMIPKIT-0053 mapping targets."""

from __future__ import annotations

import copy
import unittest

from tools.api_manifest import prepare_0053_mapping_targets


class Prepare0053MappingTargetsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = {
            "publicDeclarations": [
                {
                    "declarationId": "rust-typed-value",
                    "packageId": "rust-kmipkit",
                    "kind": "type",
                    "exportName": "kmipkit::protocol::TypedValue",
                    "capabilityId": "typed_object_attributes",
                    "requirementIds": ["KMIPKIT-0053-FR-001"],
                    "declarationState": "blocked",
                    "parityScope": "four-language",
                }
            ]
        }
        self.routes = {
            "candidateResolutions": [
                {
                    "declarationId": "rust-typed-value",
                    "capabilityId": "typed_object_attributes",
                    "packageId": "rust-kmipkit",
                    "exportName": "kmipkit::protocol::TypedValue",
                    "resolution": "mapped",
                    "languageMappings": {
                        "rust": {
                            "packageId": "rust-kmipkit",
                            "name": "kmipkit::protocol::TypedValue",
                        },
                        "c": {"packageId": "c-kmipkit", "name": "kmipkit_typed_value"},
                        "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.TypedValue"},
                        "python": {"packageId": "python-kmipkit", "name": "kmipkit.typed.TypedValue"},
                    },
                    "evidencePaths": ["bindings/c/tests/typed_value_consumer.c"],
                    "approvalSource": {
                        "kind": "feature-specification",
                        "path": "specs/053-api-parity-facade/approval-record.md",
                        "status": "Approved",
                        "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
                    },
                }
            ],
            "publicDeclarations": [],
        }
        self.observed = {
            "java-kmipkit": {
                "org.kmipkit.TypedValue": {"kind": "type", "signatures": []}
            },
            "python-kmipkit": {
                "kmipkit.typed.TypedValue": {"kind": "type", "signatures": []}
            },
        }

    def test_prepares_exact_missing_targets_deterministically_and_idempotently(self) -> None:
        first = prepare_0053_mapping_targets.prepare_counterpart_declarations(
            self.manifest, self.routes, self.observed
        )
        repeated = prepare_0053_mapping_targets.prepare_counterpart_declarations(
            self.manifest,
            {**self.routes, "publicDeclarations": copy.deepcopy(first)},
            self.observed,
        )

        self.assertEqual(first, repeated)
        self.assertEqual(
            {
                (row["packageId"], row["exportName"])
                for row in first
            },
            {
                ("java-kmipkit", "org.kmipkit.TypedValue"),
                ("python-kmipkit", "kmipkit.typed.TypedValue"),
            },
        )
        self.assertTrue(all(row["declarationState"] == "verified" for row in first))

    def test_rejects_a_mapping_target_absent_from_the_source_inventory(self) -> None:
        observed = copy.deepcopy(self.observed)
        del observed["java-kmipkit"]["org.kmipkit.TypedValue"]

        with self.assertRaisesRegex(ValueError, "not an exact observed export"):
            prepare_0053_mapping_targets.prepare_counterpart_declarations(
                self.manifest, self.routes, observed
            )


if __name__ == "__main__":
    unittest.main()
