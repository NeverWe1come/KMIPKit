"""Map response conversion failures only through the shared client outcome."""

from __future__ import annotations

import copy
import json
import unittest
from collections import Counter
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
PARTITION_PATH = ROOT / "specs" / "043-public-api-bindings" / "reports" / "t102-macro-lot-partition.json"
MAPPING_SEMANTICS = "normalized-client-protocol-error"

FAMILY_COUNTS = {
    "CertifyError": 7,
    "CheckError": 7,
    "DelegatedLoginError": 12,
    "DeriveKeyError": 8,
    "ExportError": 8,
    "GetConstraintsError": 20,
    "GetError": 7,
    "GetUsageAllocationError": 16,
    "ImportError": 8,
    "InteropError": 8,
    "JoinSplitKeyError": 8,
    "LocateError": 7,
    "LogError": 8,
    "LoginError": 12,
    "LogoutError": 14,
    "ObtainLeaseError": 7,
    "Pkcs11Error": 14,
    "ReProvisionError": 7,
    "RecertifyError": 7,
    "RegisterError": 7,
    "RekeyError": 7,
    "RekeyKeyPairError": 7,
    "RevokeError": 7,
    "SetConstraintsError": 14,
    "SetDefaultsError": 14,
}

FAMILY_OWNER = {
    **{family: ("specs/017-managed-object-retrieval/spec.md", "KMIPKIT-0017", {"KMIPKIT-0017-FR-001", "KMIPKIT-0017-FR-011", "KMIPKIT-0017-FR-012"}) for family in ("GetError", "LocateError")},
    **{family: ("specs/022-object-lifecycle/spec.md", "KMIPKIT-0036", {"KMIPKIT-0036-FR-001", "KMIPKIT-0036-FR-004"}) for family in ("CertifyError", "CheckError", "ObtainLeaseError", "ReProvisionError", "RecertifyError", "RegisterError", "RekeyError", "RekeyKeyPairError", "RevokeError")},
    **{family: ("specs/023-key-material-transfer/spec.md", "KMIPKIT-0037", {"KMIPKIT-0037-FR-001", "KMIPKIT-0037-FR-004"}) for family in ("DeriveKeyError", "ExportError", "ImportError", "JoinSplitKeyError")},
    **{family: ("specs/024-credential-sessions/spec.md", "KMIPKIT-0038", {"KMIPKIT-0038-FR-001", "KMIPKIT-0038-FR-004"}) for family in ("DelegatedLoginError", "LoginError", "LogoutError")},
    **{family: ("specs/025-policy-controls/spec.md", "KMIPKIT-0039", {"KMIPKIT-0039-FR-001", "KMIPKIT-0039-FR-004"}) for family in ("GetConstraintsError", "GetUsageAllocationError", "SetConstraintsError", "SetDefaultsError")},
    "Pkcs11Error": ("specs/026-pkcs11/spec.md", "KMIPKIT-0040", {"KMIPKIT-0040-FR-001", "KMIPKIT-0040-FR-004"}),
    **{family: ("specs/028-diagnostics-interop/spec.md", "KMIPKIT-0042", {"KMIPKIT-0042-FR-001", "KMIPKIT-0042-FR-004"}) for family in ("InteropError", "LogError")},
}

EXPECTED_MAPPING = {
    "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::ClientError::Protocol"},
    "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ERROR_PROTOCOL"},
    "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ProtocolException"},
    "python": {"packageId": "python-kmipkit", "name": "kmipkit.errors.ClientExecutionError"},
}


SHARED_PARITY_EVIDENCE = {
    "crates/kmipkit-client/tests/unit/revoke_execution_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeContractTest.java",
    "bindings/python/tests/test_client.py",
    "tools/api_manifest/tests/test_t102_response_conversion_error_mapping.py",
}


def _error_family(export_name: str) -> str | None:
    return next((part for part in export_name.split("::") if part in FAMILY_COUNTS), None)


class T102ResponseConversionErrorMappingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.partition = json.loads(PARTITION_PATH.read_text(encoding="utf-8"))
        cls.client_execution = next(
            capability
            for capability in cls.manifest["capabilities"]
            if capability["capabilityId"] == "client_execution"
        )
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.by_id = {
            row["declarationId"]: row for row in cls.manifest["publicDeclarations"]
        }

    def _candidate_rows(self) -> list[tuple[str, dict[str, object]]]:
        rows = []
        for item in self.partition["macroLots"]["1"]["declarations"]:
            row = self.by_id.get(item["declarationId"])
            if row is None:
                continue
            family = _error_family(row["exportName"])
            if family is not None:
                rows.append((family, row))
        return rows

    def test_conversion_error_rows_map_to_the_normalized_client_protocol_outcome(self) -> None:
        rows = self._candidate_rows()
        self.assertEqual(sum(FAMILY_COUNTS.values()), 241)
        self.assertEqual(FAMILY_COUNTS, dict(Counter(family for family, _ in rows)))
        still_blocked = [row["exportName"] for _, row in rows if row["declarationState"] == "blocked"]
        self.assertEqual(
            0,
            len(still_blocked),
            f"{len(still_blocked)} response-conversion error rows remain blocked; first rows: {still_blocked[:5]}",
        )

        for family, row in rows:
            with self.subTest(export=row["exportName"]):
                _owner_path, _owner_prefix, owner_requirements = FAMILY_OWNER[family]
                response_name = f"kmipkit_protocol::{family.removesuffix('Error')}Response"
                response = self.declarations[("rust-kmipkit-protocol", response_name)]
                requirements = set(row["requirementIds"])
                self.assertIn(row["declarationState"], {"declared", "verified"})
                self.assertNotIn("candidateSpecId", row)
                self.assertNotIn("blockedReason", row)
                self.assertEqual("client_execution", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertEqual(MAPPING_SEMANTICS, row["mappingSemantics"])
                self.assertEqual(EXPECTED_MAPPING, row["languageMappings"])
                self.assertEqual(self.client_execution["approvalEvidence"], row["approvalSource"])
                self.assertEqual("specs/007-client-execution/spec.md", row["approvalSource"]["path"])
                self.assertTrue(owner_requirements <= requirements)
                self.assertNotIn("KMIPKIT-0007-FR-001", requirements)
                self.assertTrue({"KMIPKIT-0007-FR-006", "KMIPKIT-0007-FR-008"} <= requirements)
                self.assertTrue({"KMIPKIT-0043-FR-005", "KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021", "KMIPKIT-0043-FR-022"} <= requirements)
                self.assertTrue(
                    {item for item in response["requirementIds"] if item.startswith("KMIPKIT-REQ-SPEC-")} <= requirements
                )
                self.assertTrue(SHARED_PARITY_EVIDENCE <= set(row["parityEvidenceIds"]))
                self.assertTrue(
                    {item for item in response["parityEvidenceIds"] if item.startswith("crates/") or item.startswith("tools/")} <= set(row["parityEvidenceIds"])
                )

    def test_direct_response_parser_methods_remain_outside_this_mapping(self) -> None:
        for family in FAMILY_COUNTS:
            with self.subTest(family=family):
                response_name = f"kmipkit_protocol::{family.removesuffix('Error')}Response::try_from_response_item"
                parser = self.declarations[("rust-kmipkit-protocol", response_name)]
                self.assertEqual("blocked", parser["declarationState"])
                self.assertEqual("KMIPKIT-0043", parser["candidateSpecId"])

    def test_only_approved_error_families_may_use_normalized_mapping_semantics(self) -> None:
        family_row = next(
            item for item in self.manifest["publicDeclarations"]
            if item.get("mappingSemantics") == MAPPING_SEMANTICS
        )
        generate._validate_normalized_client_protocol_error_source(family_row)

        unrelated = copy.deepcopy(
            self.declarations[("rust-kmipkit-protocol", "kmipkit_protocol::GetResponse")]
        )
        unrelated["declarationState"] = "declared"
        unrelated["capabilityId"] = "client_execution"
        unrelated["parityScope"] = "four-language"
        unrelated["mappingSemantics"] = MAPPING_SEMANTICS
        unrelated.pop("candidateSpecId", None)
        unrelated.pop("blockedReason", None)
        with self.assertRaisesRegex(generate.ManifestError, "approved response error family"):
            generate._validate_normalized_client_protocol_error_source(unrelated)

        work_in_progress = copy.deepcopy(family_row)
        work_in_progress["declarationState"] = "authorized_wip"
        with self.assertRaisesRegex(generate.ManifestError, "ready declaration"):
            generate._validate_normalized_client_protocol_error_source(work_in_progress)

    def test_normalized_mapping_semantics_pins_all_four_execution_targets(self) -> None:
        manifest = copy.deepcopy(self.manifest)
        row = next(
            item for item in manifest["publicDeclarations"]
            if item.get("mappingSemantics") == MAPPING_SEMANTICS
        )
        row["mappingSemantics"] = MAPPING_SEMANTICS

        package_languages = {
            package["id"]: package["language"] for package in manifest["publicPackages"]
        }
        c_release_functions = {
            ("c-kmipkit", type_record["c"]["releaseFunction"])
            for type_record in manifest["types"]
            if isinstance(type_record, dict)
            and isinstance(type_record.get("c"), dict)
            and isinstance(type_record["c"].get("releaseFunction"), str)
        }
        generate._validate_ready_mapping_targets(
            manifest["publicDeclarations"], package_languages, c_release_functions
        )

        for language, mapping in EXPECTED_MAPPING.items():
            with self.subTest(language=language):
                mapped_manifest = copy.deepcopy(manifest["publicDeclarations"])
                mapped_row = next(
                    item for item in mapped_manifest
                    if item["declarationId"] == row["declarationId"]
                )
                mapped_row["languageMappings"][language]["name"] = f"wrong.{language}.target"
                with self.assertRaisesRegex(
                    generate.ManifestError,
                    "normalized client protocol error mapping",
                ):
                    generate._validate_ready_mapping_targets(
                        mapped_manifest, package_languages, c_release_functions
                    )


if __name__ == "__main__":
    unittest.main()
