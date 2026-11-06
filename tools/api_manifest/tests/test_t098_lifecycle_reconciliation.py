"""Pin KMIPKIT-0018 lifecycle declarations to exact mappings or exclusions."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import unittest
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
BASELINE_PATH = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "t098-lifecycle-candidate-baseline-2026-11-05.json"
)
OWNER_SPEC = "specs/018-managed-object-lifecycle/spec.md"
OWNER_APPROVAL = "specs/018-managed-object-lifecycle/approval-record.md"
OPERATIONS = ("Activate", "Archive", "Destroy", "Recover")
ERROR_FAMILIES = tuple(f"{operation}Error" for operation in OPERATIONS)
NORMALIZED_ERROR_MAPPING = {
    "rust": {
        "packageId": "rust-kmipkit-client",
        "name": "kmipkit_client::ClientError::Protocol",
    },
    "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ERROR_PROTOCOL"},
    "java": {
        "packageId": "java-kmipkit",
        "name": "org.kmipkit.ProtocolException",
    },
    "python": {
        "packageId": "python-kmipkit",
        "name": "kmipkit.errors.ClientExecutionError",
    },
}


def _inventory_identity(row: dict[str, str]) -> str:
    return "|".join(
        (row["declarationId"], row["packageId"], row["kind"], row["exportName"])
    )


def _expected_mapped_ids(rows: list[dict[str, str]]) -> set[str]:
    expected: set[str] = set()
    for row in rows:
        name = row["exportName"]
        package_id = row["packageId"]
        if " as " in name:
            continue
        if package_id == "rust-kmipkit-protocol":
            if any(f"kmipkit_protocol::{family}" in name for family in ERROR_FAMILIES):
                expected.add(row["declarationId"])
                continue
            if any(
                name == f"kmipkit_protocol::{operation}{suffix}"
                for operation in OPERATIONS
                for suffix in (
                    "Request::unique_identifier",
                    "Request::to_ttlv_payload",
                    "Response::try_from_response_item",
                )
            ):
                expected.add(row["declarationId"])
                continue
        if package_id in {"rust-kmipkit-client", "rust-kmipkit"} and any(
            f"::{operation}" in name or f"::{operation.lower()}" in name
            for operation in OPERATIONS
        ):
            expected.add(row["declarationId"])
    return expected


def _operation_for_name(name: str) -> str:
    for operation in OPERATIONS:
        if f"::{operation}" in name or f"::{operation.lower()}" in name:
            return operation
    raise AssertionError(f"no lifecycle operation identified in {name}")


def _expected_operation_targets(name: str, operation: str) -> dict[str, str]:
    lowered = operation.lower()
    if "::Client::" in name:
        return {
            "c": "kmipkit_client_execute",
            "java": "org.kmipkit.extensions.Client.execute",
            "python": "kmipkit.client.Client.execute",
        }
    if "::ClientOperation::" in name:
        return {
            "c": "kmipkit_client_batch_item_metadata_v2_t.operation",
            "java": "org.kmipkit.extensions.ClientBatchItemMetadataV2.operation",
            "python": "kmipkit.client.BatchItemMetadataV2.operation",
        }
    if (
        "::ClientResponseView::" in name
        or "::ClientBatchOutcome::" in name
        or "Response::try_from_response_item" in name
    ):
        return {
            "c": f"kmipkit_client_batch_item_response_{lowered}",
            "java": f"org.kmipkit.extensions.ClientBatchItemResult.{lowered}Response",
            "python": f"kmipkit.client.BatchItemResult.{lowered}_response",
        }
    return {
        "c": f"kmipkit_client_batch_item_{lowered}",
        "java": f"org.kmipkit.extensions.ClientBatchItem.{lowered}",
        "python": f"kmipkit.extensions.client_batch_item_{lowered}",
    }


class T098LifecycleReconciliationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.baseline = json.loads(BASELINE_PATH.read_text(encoding="utf-8"))
        cls.baseline_rows = cls.baseline["declarations"]
        cls.declarations_by_id = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }
        cls.declarations_by_path = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }

    def test_frozen_baseline_contains_the_exact_348_owner_identities(self) -> None:
        identities = [_inventory_identity(row) for row in self.baseline_rows]
        digest = hashlib.sha256("\n".join(identities).encode("utf-8")).hexdigest()
        self.assertEqual(348, self.baseline["candidateCount"])
        self.assertEqual(len(identities), len(set(identities)))
        self.assertEqual(self.baseline["inventorySha256"], digest)
        self.assertEqual(
            {"rust-kmipkit-protocol": 292, "rust-kmipkit-client": 28, "rust-kmipkit": 28},
            self.baseline["packageCounts"],
        )
        self.assertEqual(
            {"associated-item": 288, "variant": 36, "field": 20, "type": 4},
            self.baseline["kindCounts"],
        )
        for item in self.baseline_rows:
            current = self.declarations_by_id.get(item["declarationId"])
            if current is not None:
                current_identity = (
                    current["packageId"],
                    current["kind"],
                    current["exportName"],
                )
            else:
                exclusion = next(
                    (
                        row
                        for row in self.manifest["approvedExclusions"]
                        if row.get("packageId") == item["packageId"]
                        and row.get("exportName") == item["exportName"]
                    ),
                    None,
                )
                self.assertIsNotNone(exclusion, item["declarationId"])
                current_identity = (
                    exclusion["packageId"],
                    item["kind"],
                    exclusion["exportName"],
                )
            self.assertEqual(
                (item["packageId"], item["kind"], item["exportName"]),
                current_identity,
            )

    def test_reconciled_manifest_stays_within_the_bounded_parser_input_size(self) -> None:
        self.assertLessEqual(MANIFEST_PATH.stat().st_size, generate.MAX_MANIFEST_BYTES)

    def test_every_baseline_declaration_is_resolved_without_owner_candidates(self) -> None:
        baseline_ids = {row["declarationId"] for row in self.baseline_rows}
        mapped_ids = _expected_mapped_ids(self.baseline_rows)
        trait_ids = {
            row["declarationId"]
            for row in self.baseline_rows
            if row["packageId"] == "rust-kmipkit-protocol"
            and " as " in row["exportName"]
        }
        self.assertEqual(92, len(mapped_ids))
        self.assertEqual(256, len(trait_ids))
        self.assertEqual(baseline_ids, mapped_ids | trait_ids)

        blocked = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0018"
            or row.get("declarationState") == "blocked"
            and row["declarationId"] in baseline_ids
        ]
        self.assertEqual([], blocked, f"unresolved T098 declarations: {blocked[:8]}")

        mapped = [self.declarations_by_id[declaration_id] for declaration_id in mapped_ids]
        self.assertEqual(92, len(mapped))
        self.assertTrue(all(row["declarationState"] in {"declared", "verified"} for row in mapped))
        self.assertEqual(
            24,
            sum(row.get("mappingSemantics") == "normalized-client-protocol-error" for row in mapped),
        )

    def test_lifecycle_error_families_use_the_shared_redacted_protocol_outcome(self) -> None:
        for family in ERROR_FAMILIES:
            family_rows = [
                row
                for row in self.baseline_rows
                if row["packageId"] == "rust-kmipkit-protocol"
                and family in row["exportName"]
                and " as " not in row["exportName"]
            ]
            self.assertEqual(6, len(family_rows), family)
            for item in family_rows:
                row = self.declarations_by_id[item["declarationId"]]
                with self.subTest(export=row["exportName"]):
                    self.assertEqual("verified", row["declarationState"])
                    self.assertEqual("client_execution", row["capabilityId"])
                    self.assertEqual("four-language", row["parityScope"])
                    self.assertEqual("normalized-client-protocol-error", row["mappingSemantics"])
                    self.assertEqual(NORMALIZED_ERROR_MAPPING, row["languageMappings"])
                    requirements = set(row["requirementIds"])
                    self.assertTrue(
                        {
                            "KMIPKIT-0018-FR-001",
                            "KMIPKIT-0018-FR-006",
                            "KMIPKIT-0018-FR-010",
                            "KMIPKIT-0007-FR-006",
                            "KMIPKIT-0007-FR-008",
                            "KMIPKIT-0007-FR-013",
                        }
                        <= requirements
                    )
                    generate._validate_normalized_client_protocol_error_source(row)

    def test_lifecycle_error_families_are_explicitly_allowed_by_the_generator(self) -> None:
        candidate = next(
            row
            for row in self.baseline_rows
            if row["exportName"] == "kmipkit_protocol::ActivateError"
        )
        row = copy.deepcopy(self.declarations_by_id[candidate["declarationId"]])
        row["declarationState"] = "verified"
        row["requirementIds"] = sorted(
            set(row["requirementIds"])
            | {
                "KMIPKIT-0018-FR-001",
                "KMIPKIT-0018-FR-006",
                "KMIPKIT-0018-FR-010",
                "KMIPKIT-0007-FR-006",
                "KMIPKIT-0007-FR-008",
                "KMIPKIT-0007-FR-013",
                "KMIPKIT-0043-FR-005",
                "KMIPKIT-0043-FR-010",
                "KMIPKIT-0043-FR-021",
                "KMIPKIT-0043-FR-022",
            }
        )
        generate._validate_normalized_client_protocol_error_source(row)

    def test_other_mapped_rows_preserve_the_operation_and_language_targets(self) -> None:
        for item in self.baseline_rows:
            if item["declarationId"] not in _expected_mapped_ids(self.baseline_rows):
                continue
            row = self.declarations_by_id[item["declarationId"]]
            if row.get("mappingSemantics") == "normalized-client-protocol-error":
                continue
            operation = _operation_for_name(row["exportName"])
            with self.subTest(export=row["exportName"]):
                self.assertEqual("verified", row["declarationState"])
                self.assertEqual(f"KMIPKIT-ELEM-OP-C2S-{operation.upper()}", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                mappings = row["languageMappings"]
                self.assertEqual(
                    {
                        "rust": {
                            "packageId": row["packageId"],
                            "name": row["exportName"],
                        },
                        **{
                            language: {
                                "packageId": {
                                    "c": "c-kmipkit",
                                    "java": "java-kmipkit",
                                    "python": "python-kmipkit",
                                }[language],
                                "name": target,
                            }
                            for language, target in _expected_operation_targets(
                                row["exportName"], operation
                            ).items()
                        },
                    },
                    mappings,
                )
                self.assertEqual(
                    "Approved",
                    row["approvalSource"]["status"],
                )
                self.assertEqual(OWNER_APPROVAL, row["approvalSource"]["path"])
                self.assertIn("KMIPKIT-0018-FR-001", row["requirementIds"])

    def test_operation_rows_reference_each_reconciled_rust_declaration(self) -> None:
        for operation in OPERATIONS:
            expected = {
                row["declarationId"]
                for row in self.baseline_rows
                if row["declarationId"] in _expected_mapped_ids(self.baseline_rows)
                and _operation_for_name(row["exportName"]) == operation
            }
            manifest_operation = next(
                item
                for item in self.manifest["operations"]
                if item.get("operationId") == f"KMIPKIT-ELEM-OP-C2S-{operation.upper()}"
            )
            with self.subTest(operation=operation):
                self.assertTrue(
                    expected <= set(manifest_operation["declarations"]["rust"]),
                    f"operation row omits mapped Rust declarations: {sorted(expected - set(manifest_operation['declarations']['rust']))[:5]}",
                )

    def test_all_trait_paths_are_exact_owner_backed_rust_only_exclusions(self) -> None:
        expected = {
            (row["packageId"], row["exportName"]): row
            for row in self.baseline_rows
            if row["declarationId"] not in _expected_mapped_ids(self.baseline_rows)
        }
        actual = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        self.assertEqual(256, len(expected))
        for key, baseline_row in expected.items():
            exclusion = actual.get(key)
            self.assertIsNotNone(exclusion, key)
            with self.subTest(export=baseline_row["exportName"]):
                self.assertEqual(OWNER_APPROVAL, exclusion["approvalSource"]["path"])
                self.assertEqual("Approved", exclusion["approvalSource"]["status"])
                self.assertIn("Rust standard-trait", exclusion["reason"])
                self.assertTrue(
                    {
                        "KMIPKIT-0018-FR-001",
                        "KMIPKIT-0018-FR-006",
                        "KMIPKIT-0018-FR-010",
                        "KMIPKIT-0043-FR-010",
                        "KMIPKIT-0043-FR-021",
                    }
                    <= set(re.findall(r"KMIPKIT-[A-Z0-9-]+", exclusion["evidence"]))
                )

    def test_direct_maintainer_approval_is_scoped_and_review_checklists_stay_open(self) -> None:
        spec = (ROOT / OWNER_SPEC).read_text(encoding="utf-8")
        approval = (ROOT / OWNER_APPROVAL).read_text(encoding="utf-8")
        self.assertIn(
            "**Status**: Approved for implementation; independent review remains pending",
            spec,
        )
        self.assertIn("2026-11-05", approval)
        self.assertIn("T098 manifest reconciliation", approval)
        self.assertIn("does not assert formal acceptance", approval)
        for checklist in ("requirements.md", "protocol.md"):
            content = (
                ROOT / "specs" / "018-managed-object-lifecycle" / "checklists" / checklist
            ).read_text(encoding="utf-8")
            self.assertIn("direct maintainer approval", content)
            self.assertIn("independent review remains outstanding", content)
            self.assertNotRegex(content, r"(?m)^- \[x\]")

        for operation in self.manifest["operations"]:
            if operation.get("featureSpecId") != "KMIPKIT-0018":
                continue
            with self.subTest(operation=operation["name"]):
                self.assertEqual("Approved", operation["approvalEvidence"]["status"])
                self.assertEqual(OWNER_SPEC, operation["approvalEvidence"]["path"])
                self.assertIn("Approved for implementation; independent review remains pending", operation["approvalEvidence"]["evidence"])


if __name__ == "__main__":
    unittest.main()
