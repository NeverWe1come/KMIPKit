"""Strict per-declaration route and type-system exclusion rules for KMIPKIT-0053."""

from __future__ import annotations

import copy
import unittest

from tools.api_manifest import reconcile_0053_api_parity


class CandidateDeclarationResolutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.approval = {
            "kind": "feature-specification",
            "path": "specs/053-api-parity-facade/approval-record.md",
            "status": "Approved",
            "evidence": "Direct maintainer approval of the Phase 5 implementation plan.",
        }
        self.evidence = [
            "bindings/c/tests/typed_attribute_parity_consumer.c",
            "bindings/java/src/test/java/org/kmipkit/typed/TypedAttributeParityTest.java",
            "bindings/python/tests/test_typed_attribute_parity.py",
            "crates/kmipkit-protocol/tests/unit/typed_attribute_tests.rs",
        ]
        self.candidates = [
            self._candidate("mapped-type", "type", "kmipkit::protocol::Attribute"),
            self._candidate("trait", "trait", "kmipkit::protocol::AttributeDefinition"),
            self._candidate(
                "associated-type",
                "associated-item",
                "kmipkit::protocol::AttributeDefinition::Value",
            ),
            self._candidate(
                "variant", "variant", "kmipkit::protocol::UniqueIdentifierValue::TextString"
            ),
            self._candidate(
                "variant-field",
                "field",
                "kmipkit::protocol::UniqueIdentifierValue::TextString::0",
            ),
            self._candidate("module", "module", "kmipkit::protocol::typed"),
        ]
        self.manifest = {
            "publicDeclarations": copy.deepcopy(self.candidates),
            "approvedExclusions": [],
            "capabilities": [
                {
                    "capabilityId": "typed_object_attributes",
                    "capabilityState": "blocked",
                    "ownerSpecId": "KMIPKIT-0048",
                    "parityScope": "four-language",
                }
            ],
        }

    @staticmethod
    def _candidate(identifier: str, kind: str, export_name: str) -> dict[str, object]:
        return {
            "declarationId": identifier,
            "packageId": "rust-kmipkit",
            "kind": kind,
            "exportName": export_name,
            "capabilityId": "typed_object_attributes",
            "requirementIds": ["KMIPKIT-0048-FR-001"],
            "declarationState": "blocked",
            "parityScope": "four-language",
            "candidateSpecId": "KMIPKIT-0053",
            "blockedReason": "four-language evidence is pending",
        }

    def _mapping(self, candidate: dict[str, object]) -> dict[str, object]:
        identifier = str(candidate["declarationId"])
        export_name = str(candidate["exportName"])
        return {
            "declarationId": identifier,
            "capabilityId": "typed_object_attributes",
            "exportName": export_name,
            "resolution": "mapped",
            "languageMappings": {
                "rust": {"packageId": "rust-kmipkit", "name": export_name},
                "c": {"packageId": "c-kmipkit", "name": f"kmipkit_{identifier}"},
                "java": {"packageId": "java-kmipkit", "name": f"org.kmipkit.{identifier}"},
                "python": {"packageId": "python-kmipkit", "name": f"kmipkit.{identifier}"},
            },
            "evidencePaths": list(self.evidence),
            "approvalSource": copy.deepcopy(self.approval),
        }

    def _exclusion(
        self,
        candidate: dict[str, object],
        artifact_kind: str,
        counterpart_ids: list[str],
    ) -> dict[str, object]:
        return {
            "declarationId": candidate["declarationId"],
            "capabilityId": candidate["capabilityId"],
            "exportName": candidate["exportName"],
            "resolution": "excluded",
            "exclusion": {
                "artifactKind": artifact_kind,
                "counterpartDeclarationIds": counterpart_ids,
            },
            "evidencePaths": list(self.evidence),
            "approvalSource": copy.deepcopy(self.approval),
        }

    def _reconcile(self, resolutions: list[dict[str, object]]) -> dict[str, object]:
        return reconcile_0053_api_parity.reconcile_candidate_resolutions(
            self.manifest, resolutions
        )

    def test_maps_exact_candidate_and_replays_idempotently(self) -> None:
        mapping = self._mapping(self.candidates[0])

        first = self._reconcile([mapping])
        second = reconcile_0053_api_parity.reconcile_candidate_resolutions(first, [mapping])

        declaration = next(
            row for row in second["publicDeclarations"] if row["declarationId"] == "mapped-type"
        )
        self.assertEqual("verified", declaration["declarationState"])
        self.assertEqual("kmipkit::protocol::Attribute", declaration["languageMappings"]["rust"]["name"])
        self.assertEqual(first, second)

    def test_requires_stable_candidate_capability_and_exact_export_identity(self) -> None:
        for field, wrong_value in (
            ("declarationId", "*"),
            ("capabilityId", "*"),
            ("exportName", "kmipkit::protocol::*"),
        ):
            with self.subTest(field=field):
                mapping = self._mapping(self.candidates[0])
                mapping[field] = wrong_value
                with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
                    self._reconcile([mapping])

    def test_rust_mapping_must_match_the_exact_public_export(self) -> None:
        mapping = self._mapping(self.candidates[0])
        mapping["languageMappings"]["rust"]["name"] = "kmipkit::protocol::Other"  # type: ignore[index]

        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            self._reconcile([mapping])

    def test_requires_complete_mapping_evidence_and_approved_source(self) -> None:
        for field in ("c", "java", "python"):
            with self.subTest(language=field):
                mapping = self._mapping(self.candidates[0])
                del mapping["languageMappings"][field]  # type: ignore[index]
                with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
                    self._reconcile([mapping])
        for field in ("evidencePaths", "approvalSource"):
            with self.subTest(field=field):
                mapping = self._mapping(self.candidates[0])
                del mapping[field]
                with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
                    self._reconcile([mapping])

    def test_excludes_only_narrow_artifacts_with_exact_mapped_counterparts(self) -> None:
        trait = self.candidates[1]
        attribute = self.candidates[0]
        mapping = self._mapping(attribute)
        exclusion = self._exclusion(trait, "rust-trait", ["mapped-type"])

        result = self._reconcile([mapping, exclusion])

        self.assertNotIn("trait", [row["declarationId"] for row in result["publicDeclarations"]])
        self.assertEqual(1, len(result["approvedExclusions"]))
        recorded = result["approvedExclusions"][0]
        self.assertEqual("trait", recorded["declarationId"])
        self.assertEqual(trait["exportName"], recorded["exportName"])
        self.assertEqual(["mapped-type"], recorded["counterpartDeclarationIds"])
        replay = reconcile_0053_api_parity.reconcile_candidate_resolutions(result, [mapping, exclusion])
        self.assertEqual(result, replay)

    def test_rejects_exclusions_for_value_routes_or_unknown_artifact_kinds(self) -> None:
        for candidate, artifact in (
            (self.candidates[0], "rust-trait"),
            (self.candidates[1], "capability-wide"),
            (self.candidates[3], "rust-trait"),
        ):
            with self.subTest(candidate=candidate["declarationId"], artifact=artifact):
                exclusion = self._exclusion(candidate, artifact, ["mapped-type"])
                with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
                    self._reconcile([self._mapping(self.candidates[0]), exclusion])

    def test_rejects_unmapped_or_cross_capability_exclusion_counterparts(self) -> None:
        trait = self.candidates[1]
        for counterpart in ("trait", "missing", "other-capability"):
            with self.subTest(counterpart=counterpart):
                manifest = copy.deepcopy(self.manifest)
                if counterpart == "other-capability":
                    manifest["publicDeclarations"].append({
                        **self._candidate("other-capability", "type", "Foreign"),
                        "capabilityId": "other-capability",
                    })
                exclusion = self._exclusion(trait, "rust-trait", [counterpart])
                with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
                    reconcile_0053_api_parity.reconcile_candidate_resolutions(
                        manifest, [exclusion]
                    )

    def test_rejects_duplicate_declaration_resolution(self) -> None:
        mapping = self._mapping(self.candidates[0])
        with self.assertRaises(reconcile_0053_api_parity.ReconciliationError):
            self._reconcile([mapping, mapping])


if __name__ == "__main__":
    unittest.main()
