"""T100 contract for KMIPKIT-0020 Query and Ping declarations."""

from __future__ import annotations

import hashlib
import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
BASELINE_INVENTORY = ROOT / "specs" / "043-public-api-bindings" / "reports" / "t100-query-ping-baseline.json"
OWNER_SPEC = "specs/020-query-ping/spec.md"
BASE_CANDIDATE_COUNT = 262
BASE_CANDIDATE_SHA256 = "a525c1d1f8af9eb4d76ef7c243f0ab780bdb2fbbe8a486865de933059cf6a48a"
T100_MAPPING_COUNT = 12
T100_EXCLUSION_COUNT = 250
SHARED_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
LANGUAGES = {"rust", "c", "java", "python"}
APPROVAL_SOURCE = {
    "kind": "feature-specification",
    "path": OWNER_SPEC,
    "status": "Approved",
}
EVIDENCE = {
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/ClientBatchItemPingQueryMappingTest.java",
    "bindings/python/tests/test_query_ping_operation.py",
    "crates/kmipkit-client/tests/unit/ping_execution_tests.rs",
    "crates/kmipkit-client/tests/unit/query_execution_tests.rs",
    "crates/kmipkit-protocol/tests/unit/query_operation_tests.rs",
}


def _owner_frs(*numbers: int) -> set[str]:
    return {f"KMIPKIT-0020-FR-{number:03}" for number in numbers}


TRAIT_OWNER_REQUIREMENTS = {
    "kmipkit_protocol::PingError": _owner_frs(8, 12),
    "kmipkit_protocol::PingRequest": _owner_frs(1, 2),
    "kmipkit_protocol::PingResponse": _owner_frs(2, 8, 12),
    "kmipkit_protocol::QueryError": _owner_frs(8, 12),
    "kmipkit_protocol::QueryFunction": _owner_frs(3, 5),
    "kmipkit_protocol::QueryRequest": _owner_frs(3, 4, 5),
    "kmipkit_protocol::QueryResponse": _owner_frs(6, 7, 11, 12),
    "kmipkit_protocol::QueryResponseField": _owner_frs(6, 11),
}


def _expected_exclusion_owner_requirements(export_name: str) -> set[str]:
    if export_name.startswith("<"):
        owner_type = export_name[1:].split(" as ", maxsplit=1)[0]
        return TRAIT_OWNER_REQUIREMENTS[owner_type]
    if "::ClientBatchOutcome::Ping" in export_name:
        return _owner_frs(2, 8, 12)
    if "::ClientBatchOutcome::Query" in export_name:
        return _owner_frs(6, 7, 8, 11, 12)
    if "::QueryResponseField" in export_name:
        return _owner_frs(6, 11, 12) if export_name.endswith("::with_ttlv") else _owner_frs(6, 11)
    if "::PingResponse" in export_name:
        return _owner_frs(2, 8, 12)
    if "::PingRequest" in export_name:
        return _owner_frs(1, 2)
    if "::PingError" in export_name or "::QueryError" in export_name:
        return _owner_frs(8, 12)
    if "::QueryFunction" in export_name:
        return _owner_frs(3, 5)
    if "::QueryRequest" in export_name:
        return _owner_frs(3, 4, 5)
    if "::QueryResponse" in export_name or "::ClientResponseView::query" in export_name:
        return _owner_frs(6, 7, 11, 12)
    if "::ClientOperation::" in export_name:
        return _owner_frs(1, 8)
    if export_name.endswith("::Client::ping") or export_name.endswith("::Client::query"):
        return _owner_frs(1, 9)
    raise AssertionError(f"T100 exclusion lacks an owner-requirement classification: {export_name}")


def _expected_mappings() -> dict[str, tuple[str, dict[str, str], set[str]]]:
    expected: dict[str, tuple[str, dict[str, str], set[str]]] = {}
    operations = {
        "Ping": {
            "capability": "KMIPKIT-ELEM-OP-C2S-PING",
            "owner_requirements": {"KMIPKIT-0020-FR-001", "KMIPKIT-0020-FR-002", "KMIPKIT-0020-FR-008", "KMIPKIT-0020-FR-009"},
            "batch_item": {
                "c": "kmipkit_client_batch_item_ping",
                "java": "org.kmipkit.extensions.ClientBatchItem.ping",
                "python": "kmipkit.extensions.client_batch_item_ping",
            },
        },
        "Query": {
            "capability": "KMIPKIT-ELEM-OP-C2S-QUERY",
            "owner_requirements": {"KMIPKIT-0020-FR-001", "KMIPKIT-0020-FR-003", "KMIPKIT-0020-FR-004", "KMIPKIT-0020-FR-005", "KMIPKIT-0020-FR-006", "KMIPKIT-0020-FR-007", "KMIPKIT-0020-FR-008", "KMIPKIT-0020-FR-009", "KMIPKIT-0020-FR-012"},
            "batch_item": {
                "c": "kmipkit_client_batch_item_query",
                "java": "org.kmipkit.extensions.ClientBatchItem.query",
                "python": "kmipkit.extensions.client_batch_item_query",
            },
        },
    }
    for package in ("kmipkit_client", "kmipkit"):
        package_id = f"rust-{package.replace('_', '-') }"
        for operation, details in operations.items():
            request_prefix = f"{package}::ClientRequest"
            request_targets = {
                "rust": f"{request_prefix}::{operation}",
                **details["batch_item"],
            }
            request_requirements = details["owner_requirements"]
            for export_name in (
                f"{request_prefix}::{operation}",
                f"{request_prefix}::{operation}::0",
                f"{request_prefix}::{operation.lower()}",
            ):
                expected[export_name] = (
                    details["capability"],
                    {**request_targets, "rust": export_name},
                    request_requirements,
                )
    return expected


def _baseline_candidate_identities() -> set[tuple[str, str]]:
    inventory = json.loads(BASELINE_INVENTORY.read_text(encoding="utf-8"))
    return {
        (row["packageId"], row["exportName"])
        for row in inventory["identities"]
    }


class T100QueryPingDeclarationMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_base_candidate_and_full_owner_identities_are_pinned(self) -> None:
        inventory = json.loads(BASELINE_INVENTORY.read_text(encoding="utf-8"))
        identities = sorted(
            f"{row['packageId']}|{row['exportName']}"
            for row in inventory["identities"]
        )
        rows = inventory["identities"]
        self.assertEqual(BASE_CANDIDATE_COUNT, inventory["count"])
        self.assertEqual(BASE_CANDIDATE_SHA256, inventory["sha256"])
        self.assertEqual(BASE_CANDIDATE_COUNT, len(rows))
        self.assertEqual(BASE_CANDIDATE_SHA256, hashlib.sha256("\n".join(identities).encode()).hexdigest())
        self.assertEqual(
            {"rust-kmipkit": 14, "rust-kmipkit-client": 15, "rust-kmipkit-protocol": 233},
            {
                package: sum(row["packageId"] == package for row in rows)
                for package in ("rust-kmipkit", "rust-kmipkit-client", "rust-kmipkit-protocol")
            },
        )

    def test_no_kmipkit_0020_candidates_remain_blocked(self) -> None:
        unresolved = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0020"
        ]
        self.assertEqual([], unresolved)

    def test_t100_mappings_reuse_existing_ping_query_batch_facades(self) -> None:
        expected = _expected_mappings()
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
        }
        self.assertEqual(T100_MAPPING_COUNT, len(expected))
        for export_name, (capability, targets, owner_requirements) in expected.items():
            with self.subTest(export=export_name):
                row = declarations[export_name]
                self.assertEqual(capability, row["capabilityId"])
                self.assertEqual("verified", row["declarationState"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(APPROVAL_SOURCE, {key: row["approvalSource"][key] for key in APPROVAL_SOURCE})
                self.assertEqual(LANGUAGES, set(row["languageMappings"]))
                self.assertEqual(
                    targets,
                    {language: mapping["name"] for language, mapping in row["languageMappings"].items()},
                )
                self.assertEqual(row["packageId"], row["languageMappings"]["rust"]["packageId"])
                self.assertEqual(
                    {"rust": row["packageId"], "c": "c-kmipkit", "java": "java-kmipkit", "python": "python-kmipkit"},
                    {language: mapping["packageId"] for language, mapping in row["languageMappings"].items()},
                )
                self.assertEqual(
                    owner_requirements,
                    {
                        requirement
                        for requirement in row["requirementIds"]
                        if requirement.startswith("KMIPKIT-0020-FR-")
                    },
                )
                self.assertTrue(SHARED_REQUIREMENTS <= set(row["requirementIds"]))
                self.assertTrue(EVIDENCE <= set(row["parityEvidenceIds"]))
                for path in EVIDENCE:
                    self.assertTrue((ROOT / path).is_file(), path)

    def test_exact_t100_baseline_is_partitioned_into_twelve_mappings_and_250_exclusions(self) -> None:
        baseline = _baseline_candidate_identities()
        mapped = {
            (row["packageId"], row["exportName"])
            for row in self.manifest["publicDeclarations"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and row["exportName"] in _expected_mappings()
        }
        excluded = {
            (row["packageId"], row["exportName"])
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and "T100" in row.get("evidence", "")
            and (row["packageId"], row["exportName"]) in baseline
        }

        self.assertEqual(T100_MAPPING_COUNT, len(mapped))
        self.assertEqual(T100_EXCLUSION_COUNT, len(excluded))
        self.assertEqual(baseline, mapped | excluded)

    def test_every_t100_exclusion_is_exact_and_owner_backed(self) -> None:
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and "T100" in row.get("evidence", "")
            and (row["packageId"], row["exportName"]) in _baseline_candidate_identities()
        ]
        self.assertEqual(T100_EXCLUSION_COUNT, len(excluded))
        identities = [(row["packageId"], row["exportName"]) for row in excluded]
        self.assertEqual(len(identities), len(set(identities)))
        for row in excluded:
            with self.subTest(export=row["exportName"]):
                self.assertEqual(APPROVAL_SOURCE, {key: row["approvalSource"][key] for key in APPROVAL_SOURCE})
                self.assertTrue(row["reason"].strip())
                self.assertTrue(row["evidence"].strip())
                self.assertIn("KMIPKIT-0020-FR-", row["reason"])
                self.assertIn("KMIPKIT-0020-FR-", row["evidence"])
                self.assertTrue(all(requirement in row["evidence"] for requirement in SHARED_REQUIREMENTS))

    def test_every_t100_exclusion_cites_only_applicable_owner_requirements(self) -> None:
        excluded = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and "T100" in row.get("evidence", "")
            and (row["packageId"], row["exportName"]) in _baseline_candidate_identities()
        ]
        self.assertEqual(T100_EXCLUSION_COUNT, len(excluded))
        self.assertEqual(133, sum(row["exportName"].startswith("<") for row in excluded))
        self.assertEqual(4, sum("::ClientBatchOutcome::Ping" in row["exportName"] for row in excluded))
        self.assertEqual(4, sum("::ClientBatchOutcome::Query" in row["exportName"] for row in excluded))
        concrete = [row for row in excluded if not row["exportName"].startswith("<")]
        self.assertEqual(33, sum("::QueryResponseField" in row["exportName"] for row in concrete))
        self.assertEqual(3, sum("::PingResponse" in row["exportName"] for row in concrete))
        for row in excluded:
            with self.subTest(export=row["exportName"]):
                expected = _expected_exclusion_owner_requirements(row["exportName"])
                self.assertEqual(expected, set(re.findall(r"KMIPKIT-0020-FR-\d{3}", row["reason"])))
                self.assertEqual(expected, set(re.findall(r"KMIPKIT-0020-FR-\d{3}", row["evidence"])))

    def test_client_convenience_methods_are_not_mapped_to_batch_item_construction(self) -> None:
        expected = {
            ("rust-kmipkit-client", "kmipkit_client::Client::ping"): "Client::ping",
            ("rust-kmipkit-client", "kmipkit_client::Client::query"): "Client::query",
            ("rust-kmipkit", "kmipkit::Client::ping"): "Client::ping",
            ("rust-kmipkit", "kmipkit::Client::query"): "Client::query",
        }
        excluded = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
            if row.get("approvalSource", {}).get("path") == OWNER_SPEC
            and "T100" in row.get("evidence", "")
            and (row["packageId"], row["exportName"]) in _baseline_candidate_identities()
        }
        self.assertTrue(expected.keys() <= excluded.keys())
        for identity, direct_method in expected.items():
            row = excluded[identity]
            self.assertIn(direct_method, row["reason"])
            self.assertIn("KMIPKIT-0020-FR-009", row["reason"])


if __name__ == "__main__":
    unittest.main()
