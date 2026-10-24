"""Source excerpts without standalone REQs keep typed, reviewable dispositions."""

from __future__ import annotations

import json
import re
import unittest
from html.parser import HTMLParser
from pathlib import Path

from tools.normative_catalog.validate import (
    _profile_oracle_reference_issue,
    _linked_clause_scope,
    validate_catalog,
    validate_source_table_contracts,
)
from tools.normative_catalog.tests.test_feature_traceability import FeatureTraceabilityTests


ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
HASH_MAC_SIGNATURE_TRACEABILITY_PATH = ROOT / "specs/021-hash-mac-signature-operations/traceability.md"
HASH_MAC_SIGNATURE_SPEC_PATH = ROOT / "specs/021-hash-mac-signature-operations/spec.md"
ENCRYPT_DECRYPT_SPEC_PATH = ROOT / "specs/019-cryptographic-operations/spec.md"
ENCRYPT_DECRYPT_PAYLOAD_CONTRACT_PATH = ROOT / "specs/019-cryptographic-operations/contracts/operation-payloads.md"
PROFILE_RESEARCH_PATH = ROOT / "specs/010-profile-conformance/research.md"


def _aggregate_source_obligation_scope_for_test(source_obligations: list[dict[str, object]]) -> str | None:
    """Aggregate independently reviewed atom scopes for this focused regression only."""
    scopes = list(dict.fromkeys(str(atom["scope_disposition"]) for atom in source_obligations))
    if not scopes:
        return None
    if "unresolved" in scopes:
        return "unclear"
    if "(split/shared)" in " ".join(scopes) or len(scopes) > 1:
        return "mixed"
    return str(scopes[0])


DECRYPT_CORRELATION_CONTRACT_ID = "KMIPKIT-SOURCE-GAP-SPEC-6.1.11-CORRELATION-VALUE-OPTIONAL"
ENCRYPT_CORRELATION_CONTRACT_ID = "KMIPKIT-SOURCE-GAP-SPEC-6.1.17-CORRELATION-VALUE-OPTIONAL"
SINGLE_PART_PROCEDURE_TEST_REFS = {
    "crates/kmipkit-protocol/tests/unit/encrypt_tests.rs::request_omits_cryptographic_parameters_when_the_caller_omits_them",
    "crates/kmipkit-protocol/tests/unit/decrypt_tests.rs::request_omits_cryptographic_parameters_and_iv_when_the_caller_omits_them",
    "crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs::request_requires_single_part_data_and_omits_it_for_multipart",
    "crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs::request_allows_optional_identifier_and_parameters_to_be_omitted",
    "crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs::request_allows_optional_key_parameters_and_original_data_to_be_omitted",
    "crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs::request_allows_optional_identifier_and_parameters_to_be_omitted",
    "crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs::request_allows_optional_key_parameters_and_input_data_to_be_omitted",
}

# These are the request rows relevant to the five KMIPKIT-0021 operations
# assigned by the approved traceability. The first two §6.1 contexts cover
# multipart requests; the third covers single-part requests, including Sign's
# Digested Data alternative. Operation IDs, request tables, requiredness, and
# feature assignments are checked against the catalog/spec below.
EXPECTED_GENERIC_PARAMETER_CONTRACTS = {
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CRYPTOGRAPHIC-PARAMETERS-REQUIRED": (
        "KMIPKIT-ELEM-OP-C2S-HASH",
        235,
        {"FR-002"},
        "always",
    ),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-DATA-REQUIRED": (
        "KMIPKIT-ELEM-OP-C2S-HASH",
        235,
        {"FR-002"},
        "single_part",
    ),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.32-DATA-REQUIRED": (
        "KMIPKIT-ELEM-OP-C2S-MAC",
        259,
        {"FR-004"},
        "single_part",
    ),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.33-MAC-DATA-REQUIRED": (
        "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY",
        262,
        {"FR-007"},
        "single_part",
    ),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.55-DATA-CONDITIONAL": (
        "KMIPKIT-ELEM-OP-C2S-SIGN",
        334,
        {"FR-004"},
        "single_part",
    ),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.55-DIGESTED-DATA-OPTIONAL": (
        "KMIPKIT-ELEM-OP-C2S-SIGN",
        334,
        {"FR-004"},
        "optional",
    ),
    "KMIPKIT-SOURCE-GAP-SPEC-6.1.56-SIGNATURE-DATA-CONDITIONAL": (
        "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY",
        337,
        {"FR-007"},
        "single_part",
    ),
}

# KMIPKIT-0019 approves both client 1.0 operations. Only their optional
# Correlation Value request rows are mapped; broader parameter mapping remains open.
PARTIALLY_MAPPED_GENERIC_CONTEXT_OPERATION_TABLES = {
    "KMIPKIT-ELEM-OP-C2S-DECRYPT": (
        "Decrypt",
        196,
        DECRYPT_CORRELATION_CONTRACT_ID,
    ),
    "KMIPKIT-ELEM-OP-C2S-ENCRYPT": (
        "Encrypt",
        214,
        ENCRYPT_CORRELATION_CONTRACT_ID,
    ),
}

EXPECTED_GENERIC_CONTEXT_OPERATIONS = {
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-006": {
        "KMIPKIT-ELEM-OP-C2S-DECRYPT": {
            DECRYPT_CORRELATION_CONTRACT_ID
        },
        "KMIPKIT-ELEM-OP-C2S-ENCRYPT": {
            ENCRYPT_CORRELATION_CONTRACT_ID
        },
        "KMIPKIT-ELEM-OP-C2S-HASH": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CRYPTOGRAPHIC-PARAMETERS-REQUIRED"
        },
        "KMIPKIT-ELEM-OP-C2S-MAC": set(),
        "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGN": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": set(),
    },
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-007": {
        "KMIPKIT-ELEM-OP-C2S-DECRYPT": {
            DECRYPT_CORRELATION_CONTRACT_ID
        },
        "KMIPKIT-ELEM-OP-C2S-ENCRYPT": {
            ENCRYPT_CORRELATION_CONTRACT_ID
        },
        "KMIPKIT-ELEM-OP-C2S-HASH": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CRYPTOGRAPHIC-PARAMETERS-REQUIRED"
        },
        "KMIPKIT-ELEM-OP-C2S-MAC": set(),
        "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGN": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": set(),
    },
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-008": {
        "KMIPKIT-ELEM-OP-C2S-DECRYPT": {
            DECRYPT_CORRELATION_CONTRACT_ID
        },
        "KMIPKIT-ELEM-OP-C2S-ENCRYPT": {
            ENCRYPT_CORRELATION_CONTRACT_ID
        },
        "KMIPKIT-ELEM-OP-C2S-HASH": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CRYPTOGRAPHIC-PARAMETERS-REQUIRED",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-DATA-REQUIRED",
        },
        "KMIPKIT-ELEM-OP-C2S-MAC": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.32-DATA-REQUIRED",
        },
        "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.33-MAC-DATA-REQUIRED",
        },
        "KMIPKIT-ELEM-OP-C2S-SIGN": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.55-DATA-CONDITIONAL",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.55-DIGESTED-DATA-OPTIONAL",
        },
        "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.56-SIGNATURE-DATA-CONDITIONAL",
        },
    },
}


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


def _requiredness_matches(contract: dict[str, object], expected: str) -> bool:
    """Match the reviewed table requiredness category for a procedure mode."""
    requiredness = contract.get("requiredness")
    if expected == "always":
        return requiredness == "Yes"
    if expected == "single_part":
        return (
            isinstance(requiredness, str)
            and requiredness.startswith("Yes for single-part")
            and "No for multi-part" in requiredness
        )
    if expected == "optional":
        return requiredness == "No"
    return False


class SourceGapContextResolutionTests(unittest.TestCase):
    def test_result_message_research_states_validator_scope_boundary(self) -> None:
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")

        self.assertIn("focused regression", research)
        self.assertIn(
            "The production catalog validator does not load the independent source-review oracle or aggregate source-atom scopes.",
            research,
        )
        self.assertIn("checks the stored clause scope against linked requirement scopes", research)
        self.assertNotIn(
            "The validator derives §9.17 aggregate scope from both source atoms.",
            research,
        )

    def test_encrypt_decrypt_correlation_request_rows_have_optional_feature_contracts(self) -> None:
        """Tables 196 and 214 map only their optional Correlation Value request rows."""
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        expected = {
            DECRYPT_CORRELATION_CONTRACT_ID: {
                "source_section": "6.1.11",
                "html_line_range": [25165, 25181],
                "table_number": 196,
                "operation_element_id": "KMIPKIT-ELEM-OP-C2S-DECRYPT",
                "source_excerpt": (
                    "Correlation Value No Specifies the existing stream or by-parts "
                    "cryptographic operation (as returned from a previous call to this operation)."
                ),
                "implementation_ref": "crates/kmipkit-protocol/src/decrypt.rs::DecryptRequest::to_ttlv_payload",
                "verification_refs": [
                    "crates/kmipkit-protocol/tests/unit/decrypt_tests.rs::request_emits_present_table_196_members_in_exact_order_and_accepts_decrypt_tag",
                    "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::decrypt_caller_reuses_initial_server_correlation_and_keeps_aad_and_tag_initial",
                ],
            },
            ENCRYPT_CORRELATION_CONTRACT_ID: {
                "source_section": "6.1.17",
                "html_line_range": [26590, 26606],
                "table_number": 214,
                "operation_element_id": "KMIPKIT-ELEM-OP-C2S-ENCRYPT",
                "source_excerpt": (
                    "Correlation Value No Specifies the existing stream or by-parts "
                    "cryptographic operation (as returned from a previous call to this operation)."
                ),
                "implementation_ref": "crates/kmipkit-protocol/src/encrypt.rs::EncryptRequest::to_ttlv_payload",
                "verification_refs": [
                    "crates/kmipkit-protocol/tests/unit/encrypt_tests.rs::request_emits_present_table_214_members_in_exact_order_and_preserves_values",
                    "crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs::encrypt_caller_reuses_initial_server_correlation_for_middle_and_final_parts",
                ],
            },
        }
        contracts = {
            row["gap_id"]: row
            for row in oracle["source_table_contracts"]
            if row.get("gap_id") in expected
        }

        self.assertEqual(set(contracts), set(expected))
        if set(contracts) != set(expected):
            return
        elements = {row["element_id"]: row for row in catalog["elements"]}
        for gap_id, reviewed in expected.items():
            contract = contracts[gap_id]
            with self.subTest(gap_id=gap_id):
                self.assertEqual(contract["contract_kind"], "operation_request_table_field")
                self.assertEqual(contract["source_id"], "KMIPKIT-SRC-spec")
                self.assertEqual(
                    contract["source_path"],
                    "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html",
                )
                self.assertEqual(contract["source_section"], reviewed["source_section"])
                self.assertEqual(contract["source_locator"]["kind"], "table_body_row")
                self.assertEqual(contract["source_locator"]["section"], reviewed["source_section"])
                self.assertEqual(contract["source_locator"]["row_label"], "Correlation Value")
                self.assertEqual(contract["source_locator"]["table"], "request_payload")
                self.assertEqual(contract["html_line_range"], reviewed["html_line_range"])
                self.assertEqual(contract["source_locator"]["html_line_range"], reviewed["html_line_range"])
                self.assertEqual(contract["source_excerpt"], reviewed["source_excerpt"])
                self.assertEqual(
                    pinned_source_excerpt(contract["source_path"], contract["html_line_range"]),
                    reviewed["source_excerpt"],
                )
                self.assertEqual(contract["source_force"], "optional_table_body_value")
                self.assertEqual(contract["requiredness"], "No")
                self.assertEqual(contract["table_number"], reviewed["table_number"])
                self.assertEqual(contract["operation_element_id"], reviewed["operation_element_id"])
                self.assertEqual(contract["functional_requirement_ids"], ["FR-004"])
                self.assertEqual(
                    contract["implementation_refs"],
                    [reviewed["implementation_ref"]],
                )
                self.assertEqual(contract["verification_refs"], reviewed["verification_refs"])
                self.assertEqual(contract["actor"], "client")
                self.assertEqual(contract["direction"], f"client→server {elements[reviewed['operation_element_id']]['name']} request")
                self.assertEqual(contract["scope_disposition"], "client_1_0")
                self.assertEqual(contract["status"], "mapped_to_approved_feature_contract")
                self.assertNotIn("candidate_requirement_id", contract)
                self.assertNotIn("proposed_requirement_id", contract)
                self.assertNotIn("requirement_id", contract)

                operation = elements[reviewed["operation_element_id"]]
                self.assertEqual(operation["feature_spec"], "KMIPKIT-0019")
                self.assertEqual(operation["direction"], "client_to_server")
                self.assertEqual(operation["scope_state"], "client_1_0")
                self.assertIn(
                    reviewed["table_number"],
                    {
                        table["table_number"]
                        for table in operation["payload_tables"]
                        if table.get("role") == "request"
                    },
                )

        self.assertEqual(validate_source_table_contracts(oracle, ROOT), [])

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
            raw_contract_ids = resolution.get("related_source_table_contract_ids", [])
            with self.subTest(gap_id=gap_id, link_field="related_source_table_contract_ids"):
                self.assertIsInstance(raw_contract_ids, list)
            if not isinstance(raw_contract_ids, list):
                raw_contract_ids = []
            self.assertTrue(all(isinstance(identifier, str) for identifier in raw_contract_ids))
            linked_contract_ids = {
                identifier for identifier in raw_contract_ids if isinstance(identifier, str)
            }
            known_linked_contract_ids = linked_contract_ids & contracts.keys()
            self.assertEqual(known_linked_contract_ids, linked_contract_ids)
            self.assertEqual(len(known_linked_contract_ids), len(raw_contract_ids))
            contract_evidence = bool(known_linked_contract_ids) and all(
                contracts[identifier]["implementation_refs"] and contracts[identifier]["verification_refs"]
                for identifier in known_linked_contract_ids
            )
            raw_verification_refs = resolution.get("verification_refs", [])
            with self.subTest(gap_id=gap_id, link_field="verification_refs"):
                self.assertIsInstance(raw_verification_refs, list)
            if not isinstance(raw_verification_refs, list):
                raw_verification_refs = []
            if number == 19:
                self.assertEqual(set(raw_verification_refs), SINGLE_PART_PROCEDURE_TEST_REFS)
            verification_evidence = bool(raw_verification_refs) and all(
                isinstance(test_ref, str)
                and FeatureTraceabilityTests._is_executable_test_ref(test_ref)
                for test_ref in raw_verification_refs
            )
            review_note = resolution.get("review_note", "")
            explicit_blocker = (
                gap["scope_disposition"] == "unresolved"
                or "Keep this imperative source gap unresolved" in review_note
            )

            with self.subTest(gap_id=gap_id):
                if number in (12, 14, 16):
                    self.assertEqual(linked_requirement_ids, ["KMIPKIT-REQ-SPEC-7.8-001"])
                self.assertTrue(
                    requirement_evidence or contract_evidence or verification_evidence or explicit_blocker,
                    f"{gap_id} is treated as closed by procedure_context without linked executable evidence or an explicit blocker",
                )

    def test_generic_parameter_context_links_operation_contract_evidence(self) -> None:
        """KMIP Specification v2.1 §6.1 generic instructions need request contracts."""
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        approved_traceability = HASH_MAC_SIGNATURE_TRACEABILITY_PATH.read_text(encoding="utf-8")
        approved_spec = HASH_MAC_SIGNATURE_SPEC_PATH.read_text(encoding="utf-8")
        encrypt_decrypt_spec = ENCRYPT_DECRYPT_SPEC_PATH.read_text(encoding="utf-8")
        encrypt_decrypt_payload_contract = ENCRYPT_DECRYPT_PAYLOAD_CONTRACT_PATH.read_text(encoding="utf-8")
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        contracts = {row["gap_id"]: row for row in oracle["source_table_contracts"]}
        catalog_operations = {
            row["element_id"]: row
            for row in catalog["elements"]
            if row.get("feature_spec") in {"KMIPKIT-0019", "KMIPKIT-0021"}
            and row.get("element_id", "").startswith("KMIPKIT-ELEM-OP-C2S-")
        }
        approved_operation_ids_021 = set(
            re.findall(r"`(KMIPKIT-ELEM-OP-C2S-[A-Z-]+)`", approved_traceability)
        )
        approved_operation_ids_019 = set(
            re.findall(r"`(KMIPKIT-ELEM-OP-C2S-[A-Z-]+)`", encrypt_decrypt_spec)
        )
        approved_operation_ids = approved_operation_ids_019 | approved_operation_ids_021
        approved_requirement_ids = set(re.findall(r"\bFR-\d{3}\b", approved_spec))
        expected_operation_ids = set(catalog_operations)
        expected_requirement_ids = set().union(
            *(entry[2] for entry in EXPECTED_GENERIC_PARAMETER_CONTRACTS.values())
        )
        expected_operation_ids_from_contexts = set().union(
            *(set(operation_contracts) for operation_contracts in EXPECTED_GENERIC_CONTEXT_OPERATIONS.values())
        )
        expected_mapped_contract_ids = set().union(
            *(
                contract_ids
                for operation_contracts in EXPECTED_GENERIC_CONTEXT_OPERATIONS.values()
                for contract_ids in operation_contracts.values()
                if contract_ids is not None
            )
        )
        correlation_request_contract_ids = {
            contract_id
            for _, _, contract_id in PARTIALLY_MAPPED_GENERIC_CONTEXT_OPERATION_TABLES.values()
        }

        self.assertEqual(set(catalog_operations), approved_operation_ids)
        self.assertEqual(
            approved_operation_ids_019,
            set(PARTIALLY_MAPPED_GENERIC_CONTEXT_OPERATION_TABLES),
        )
        self.assertEqual(expected_operation_ids_from_contexts, expected_operation_ids)
        for operation_id, (operation_name, table_number, _) in PARTIALLY_MAPPED_GENERIC_CONTEXT_OPERATION_TABLES.items():
            with self.subTest(operation_id=operation_id):
                self.assertIn(operation_id, catalog_operations)
                self.assertEqual(catalog_operations[operation_id]["feature_spec"], "KMIPKIT-0019")
                self.assertIn(f"{operation_name} Table {table_number}", encrypt_decrypt_payload_contract)
                self.assertIn(
                    table_number,
                    {
                        table["table_number"]
                        for table in catalog_operations[operation_id]["payload_tables"]
                        if table.get("role") == "request"
                    },
                )
        self.assertEqual(
            expected_mapped_contract_ids,
            set(EXPECTED_GENERIC_PARAMETER_CONTRACTS) | correlation_request_contract_ids,
        )
        self.assertTrue(expected_requirement_ids)
        self.assertTrue(expected_requirement_ids <= approved_requirement_ids)

        for contract_id, (operation_id, table_number, requirement_ids, requiredness) in EXPECTED_GENERIC_PARAMETER_CONTRACTS.items():
            with self.subTest(contract_id=contract_id):
                contract = contracts.get(contract_id)
                self.assertIsNotNone(contract, f"missing approved operation contract {contract_id}")
                if contract is None:
                    continue
                self.assertEqual(contract["contract_kind"], "operation_request_table_field")
                self.assertEqual(contract["operation_element_id"], operation_id)
                self.assertEqual(contract["table_number"], table_number)
                self.assertTrue(_requiredness_matches(contract, requiredness))
                request_tables = {
                    table["table_number"]
                    for table in catalog_operations[operation_id]["payload_tables"]
                    if table.get("role") == "request"
                }
                self.assertIn(table_number, request_tables)
                self.assertTrue(requirement_ids <= set(contract["functional_requirement_ids"]))
                self.assertTrue(set(contract["functional_requirement_ids"]) <= approved_requirement_ids)
                self.assertTrue(contract["implementation_refs"])
                self.assertTrue(contract["verification_refs"])
                self.assertTrue(
                    all(
                        _profile_oracle_reference_issue(ROOT, ref, executable_test=False) is None
                        for ref in contract["implementation_refs"]
                    )
                )
                self.assertTrue(
                    all(
                        _profile_oracle_reference_issue(ROOT, ref, executable_test=True) is None
                        for ref in contract["verification_refs"]
                    )
                )

        for gap_id, operation_contracts in EXPECTED_GENERIC_CONTEXT_OPERATIONS.items():
            gap = gaps[gap_id]
            resolution = gap["source_resolution"]
            raw_contract_ids = resolution.get("related_source_table_contract_ids")
            with self.subTest(gap_id=gap_id, link_field="related_source_table_contract_ids"):
                self.assertIsInstance(
                    raw_contract_ids,
                    list,
                    f"{gap_id} must use the structured contract-ID field, not review_note prose",
                )
            if not isinstance(raw_contract_ids, list):
                raw_contract_ids = []
            self.assertTrue(all(isinstance(identifier, str) for identifier in raw_contract_ids))
            linked_contract_ids = {
                identifier for identifier in raw_contract_ids if isinstance(identifier, str)
            }
            known_linked_contract_ids = linked_contract_ids & contracts.keys()
            self.assertEqual(known_linked_contract_ids, linked_contract_ids)
            expected_contract_ids = set().union(
                *(contract_ids for contract_ids in operation_contracts.values() if contract_ids is not None)
            )
            self.assertEqual(len(linked_contract_ids), len(raw_contract_ids))
            self.assertEqual(set(operation_contracts), expected_operation_ids)
            for operation_id, expected_ids in operation_contracts.items():
                with self.subTest(gap_id=gap_id, operation_id=operation_id):
                    if operation_id in PARTIALLY_MAPPED_GENERIC_CONTEXT_OPERATION_TABLES:
                        operation_name, table_number, expected_optional_contract_id = (
                            PARTIALLY_MAPPED_GENERIC_CONTEXT_OPERATION_TABLES[operation_id]
                        )
                        request_contracts = {
                            contract_id
                            for contract_id, contract in contracts.items()
                            if contract.get("operation_element_id") == operation_id
                            and contract.get("table_number") == table_number
                        }
                        self.assertEqual(
                            request_contracts,
                            {expected_optional_contract_id},
                            f"{gap_id} may map only the optional Correlation Value row from {operation_name} Table {table_number}",
                        )
                        note = resolution.get("review_note", "")
                        self.assertEqual(
                            gap["scope_disposition"],
                            "client_1_0",
                            f"{gap_id} must preserve client scope while broader {operation_name} Table {table_number} parameter mapping stays open",
                        )
                        note_requirements = {
                            "KMIPKIT-0019",
                            operation_name,
                            f"Table {table_number}",
                            "optional Correlation Value",
                            "broader operation-specific parameter mapping remains open",
                        }
                        self.assertTrue(
                            all(term in note for term in note_requirements),
                            f"{gap_id} needs an explicit remaining-scope note for {operation_name} Table {table_number}",
                        )
                        self.assertTrue(expected_ids <= linked_contract_ids)
                        for contract_id in request_contracts:
                            contract = contracts[contract_id]
                            self.assertTrue(contract["implementation_refs"])
                            self.assertTrue(contract["verification_refs"])
                            self.assertTrue(
                                all(
                                    _profile_oracle_reference_issue(ROOT, ref, executable_test=False) is None
                                    for ref in contract["implementation_refs"]
                                )
                            )
                            self.assertTrue(
                                all(
                                    _profile_oracle_reference_issue(ROOT, ref, executable_test=True) is None
                                    for ref in contract["verification_refs"]
                                )
                            )
                        continue
                    actual_operation_ids = {
                        contract_id
                        for contract_id in known_linked_contract_ids
                        if contracts[contract_id]["operation_element_id"] == operation_id
                    }
                    self.assertEqual(
                        actual_operation_ids,
                        expected_ids,
                        f"{gap_id} has an incomplete or misassigned mapping for {operation_id}",
                    )
            with self.subTest(gap_id=gap_id, mapping="complete-set"):
                self.assertEqual(
                    linked_contract_ids,
                    expected_contract_ids,
                    f"{gap_id} must link the complete reviewed request contract set for each operation",
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

    def test_paragraph_006_to_008_capture_client_request_sequence_scope(self) -> None:
        """§6.1 procedure context is client-scoped without inventing formal requirement IDs."""
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        expected = {
            6: {
                "source_range": [22433, 22435],
                "condition": "Multipart is optional and applies only to operations with Correlation Value, Init Indicator, and Final Indicator request parameters; this step is for the first request.",
                "context_ranges": ((22417, 22420), (22425, 22427)),
                "context_text": ("are optionally supported", "On the first request"),
            },
            7: {
                "source_range": [22453, 22455],
                "condition": "Multipart is optional and applies only to operations with Correlation Value, Init Indicator, and Final Indicator request parameters; this step is for subsequent requests.",
                "context_ranges": ((22417, 22420), (22445, 22447)),
                "context_text": ("are optionally supported", "On subsequent requests"),
            },
            8: {
                "source_range": [22490, 22492],
                "condition": "Single-part cryptographic operations; this step applies to each request.",
                "context_ranges": ((22477, 22483),),
                "context_text": ("Single-part cryptographic operations", "On each request"),
            },
        }

        for number, expected_context in expected.items():
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}"]
            with self.subTest(gap_id=gap["gap_id"]):
                self.assertEqual(gap["html_line_range"], expected_context["source_range"])
                context = " ".join(
                    pinned_source_excerpt(gap["source_path"], list(line_range))
                    for line_range in expected_context["context_ranges"]
                )
                self.assertTrue(all(text in context for text in expected_context["context_text"]))
                self.assertEqual(gap["actor"], "client")
                self.assertEqual(gap["direction"], "client cryptographic-operation request parameters")
                self.assertEqual(gap["condition"], expected_context["condition"])
                self.assertEqual(gap["scope_disposition"], "client_1_0")
                self.assertEqual(gap["source_force"], "imperative_procedure_without_formal_modal")
                self.assertEqual(gap["source_keywords"], [])
                self.assertEqual(gap["status"], "reviewed_generic_parameter_context")
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertEqual(gap["source_resolution"]["kind"], "generic_parameter_context")
                self.assertEqual(gap["source_resolution"]["related_requirement_ids"], [])

    def test_paragraph_data_rule_uses_table_precedence_and_lease_policy_maps_to_attribute(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        data_rule = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"]
        resolution = data_rule.get("source_resolution")
        self.assertIsInstance(resolution, dict)
        if not isinstance(resolution, dict):
            return
        self.assertEqual(resolution["kind"], "operation_table_precedence")
        self.assertEqual(resolution["decision_id"], "KMIPKIT-DEC-015")
        self.assertEqual(data_rule["scope_disposition"], "client_1_0")
        self.assertEqual(data_rule["status"], "reviewed_table_precedence")
        self.assertIsNone(data_rule["candidate_requirement_id"])
        self.assertIsNone(data_rule["proposed_requirement_id"])
        self.assertIn("Table 334", resolution["review_note"])
        self.assertIn("Digested Data", resolution["review_note"])
        self.assertIn("framed true/true", resolution["review_note"])

        table_contracts = {
            row["gap_id"]: row for row in oracle["source_table_contracts"]
        }
        for gap_id in (
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-DATA-REQUIRED",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.32-DATA-REQUIRED",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.55-DATA-CONDITIONAL",
        ):
            with self.subTest(gap_id=gap_id):
                disposition = table_contracts[gap_id]["evidence_disposition"]
                self.assertEqual(disposition["status"], "resolved_by_project_interpretation")
                self.assertEqual(disposition["decision_id"], "KMIPKIT-DEC-015")
                self.assertIn("no standalone paragraph-009 requirement id", disposition["claim"].casefold())

        sign_row = table_contracts["KMIPKIT-SOURCE-GAP-SPEC-6.1.55-DATA-CONDITIONAL"]
        self.assertIn("unless Digested Data is supplied", sign_row["requiredness"])
        self.assertTrue(
            any(
                reference.endswith("::request_requires_data_or_digested_data_for_single_part_and_accepts_digest_only")
                for reference in sign_row["verification_refs"]
            )
        )

        lease_policy = gaps["KMIPKIT-SOURCE-GAP-SPEC-4.30-CLIENT-LEASE-READ-ONLY"]
        resolution = lease_policy["source_resolution"]
        self.assertEqual(resolution["kind"], "attribute_policy_mapping")
        self.assertEqual(resolution["element_id"], "KMIPKIT-ELEM-ATTRIBUTE-LEASE-TIME")
        self.assertIsNone(lease_policy["candidate_requirement_id"])
        self.assertIsNone(lease_policy["proposed_requirement_id"])
        lease_time = next(row for row in catalog["elements"] if row["element_id"] == resolution["element_id"])
        self.assertEqual(lease_time["source_modifiable_by_client"], "No")
        self.assertTrue(any("read-only for clients" in row["source_text"] for row in lease_time["source_operation_restrictions"]))

    def test_multipart_optional_preface_is_a_client_capability_context(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-MULTIPART-OPTIONAL-PREFACE"]

        self.assertEqual(gap["html_line_range"], [22417, 22420])
        self.assertIn("optionally supported", gap["source_excerpt"])
        self.assertEqual(gap["source_keywords"], [])
        self.assertEqual(gap["actor"], "client")
        self.assertEqual(gap["direction"], "client_to_server")
        self.assertEqual(gap["scope_disposition"], "client_1_0")
        self.assertEqual(gap["status"], "reviewed_optional_client_capability_context")
        self.assertIsNone(gap["candidate_requirement_id"])
        self.assertIsNone(gap["proposed_requirement_id"])
        resolution = gap.get("source_resolution")
        self.assertIsInstance(resolution, dict)
        if not isinstance(resolution, dict):
            return
        self.assertEqual(resolution["kind"], "optional_client_capability_context")
        self.assertEqual(resolution["related_requirement_ids"], [])
        self.assertIn("Correlation Value", resolution["review_note"])
        self.assertIn("Init Indicator", resolution["review_note"])
        self.assertIn("Final Indicator", resolution["review_note"])
        self.assertIn("lowercase", resolution["review_note"])

    def test_result_message_may_atoms_split_server_output_from_client_use(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        reviews = {row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]}
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}

        gap = gaps["KMIPKIT-SOURCE-GAP-SPEC-9.17-001"]
        resolution = gap.get("source_resolution")
        self.assertIsInstance(resolution, dict)
        if not isinstance(resolution, dict):
            return
        self.assertEqual(resolution["kind"], "mixed_response_and_client_access")
        self.assertEqual(resolution["related_requirement_ids"], ["KMIPKIT-REQ-SPEC-9.17-001-002"])
        self.assertEqual(gap["scope_disposition"], "mixed")

        review = reviews["KMIPKIT-CLAUSE-SPEC-9.17-001"]
        atoms = review["source_obligations"]
        self.assertEqual([atom["source_keyword"] for atom in atoms], ["MAY", "MAY"])
        self.assertEqual([atom["scope_disposition"] for atom in atoms], ["server_only", "client_1_0"])
        self.assertEqual(atoms[0]["direction"], "server_to_client")
        self.assertEqual(atoms[0]["actor"], "unspecified")
        self.assertEqual(atoms[1]["actor"], "client")
        self.assertEqual(atoms[1]["direction"], "server_to_client")
        self.assertEqual(atoms[1]["candidate_requirement_id"], "KMIPKIT-REQ-SPEC-9.17-001-002")
        self.assertEqual(review["expected_client_requirement_ids"], ["KMIPKIT-REQ-SPEC-9.17-001-002"])

        clause = clauses["KMIPKIT-CLAUSE-SPEC-9.17-001"]
        self.assertEqual(clause["requirement_ids"], ["KMIPKIT-REQ-SPEC-9.17-001-002"])
        requirement = requirements.get("KMIPKIT-REQ-SPEC-9.17-001-002")
        self.assertIsInstance(requirement, dict)
        if not isinstance(requirement, dict):
            return
        self.assertEqual(requirement["source_keyword"], "MAY")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "server_to_client")
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertTrue(
            any(reference.startswith("crates/kmipkit-protocol/src/result.rs::") for reference in requirement["implementation_refs"])
        )
        self.assertIn(
            "crates/kmipkit-protocol/tests/unit/common_operation_result_tests.rs::shared_helper_and_encrypt_decrypt_preserve_unknown_reason_and_result_message",
            requirement["verification_refs"],
        )
        self.assertIn("result_contract.rs::result_message_debug_is_redacted", " ".join(requirement["verification_refs"]))
        self.assertIn("does not automatically log", requirement["review_note"].lower())

    def test_result_message_source_atoms_and_catalog_record_mixed_scope(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        review = next(
            row for row in oracle["source_requirement_reviews"]
            if row["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.17-001"
        )
        clause = next(
            row for row in catalog["source_clauses"]
            if row["clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.17-001"
        )
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}

        self.assertEqual(
            _aggregate_source_obligation_scope_for_test(review["source_obligations"]),
            "mixed",
        )
        self.assertEqual(clause["scope_state"], "mixed")
        self.assertEqual(_linked_clause_scope(clause, requirements), "mixed")
        self.assertEqual(
            validate_catalog(CATALOG_PATH.read_bytes(), ROOT)["clause_count"],
            len(catalog["source_clauses"]),
        )

    def test_t002_research_describes_current_paragraph_009_source_force_and_context(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")
        start = research.index("- **Procedure and generic-parameter context:**")
        end = research.find("\n- **", start + 1)
        summary = research[start:end]

        data_rule = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"]
        self.assertEqual(data_rule["source_force"], "structural_requirement_statement_lowercase_required")
        self.assertEqual(data_rule["source_keywords"], [])
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        self.assertEqual(clauses["KMIPKIT-CLAUSE-SPEC-6.1-009"]["source_keywords"], ["REQUIRED"])
        self.assertIn("lowercase `required`", summary)
        self.assertIn("not a formal `REQUIRED` keyword", summary)
        self.assertIn("independent source-gap oracle records `source_keywords: []`", summary)
        self.assertIn("catalog's case-insensitive lexical `REQUIRED` hit", summary)

        for number in range(6, 9):
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}"]
            self.assertEqual(gap["actor"], "client")
            self.assertEqual(gap["direction"], "client cryptographic-operation request parameters")
            self.assertEqual(gap["scope_disposition"], "client_1_0")
        self.assertIn("identify the client actor", summary)
        self.assertIn("request-parameter direction", summary)
        self.assertNotIn("actor/direction/scope unclear", summary)

    def test_research_inventory_table_matches_paragraph_006_to_008_scope(self) -> None:
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")
        rows = {}
        for line in research.splitlines():
            if "`KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-" not in line:
                continue
            columns = [column.strip().strip("`") for column in line.strip().strip("|").split("|")]
            gap_id = columns[0]
            rows[gap_id] = columns

        for number in range(6, 9):
            gap_id = f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-{number:03d}"
            with self.subTest(gap_id=gap_id):
                self.assertIn(gap_id, rows)
                self.assertEqual(rows[gap_id][2], "client_1_0")
                self.assertEqual(rows[gap_id][3], "reviewed_generic_parameter_context")
                self.assertEqual(rows[gap_id][4], "—")

    def test_t002_research_bounds_procedure_evidence_to_verified_request_contracts(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        contracts = {row["gap_id"]: row for row in oracle["source_table_contracts"]}
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        requirements = {row["requirement_id"]: row for row in catalog["requirements"]}
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")
        start = research.index("- **Procedure and generic-parameter context:**")
        end = research.find("\n- **", start + 1)
        summary = research[start:end]

        request_contract_ids = {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CORRELATION-VALUE-OPTIONAL",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.32-CORRELATION-VALUE-OPTIONAL",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.33-CORRELATION-VALUE-OPTIONAL",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.55-CORRELATION-VALUE-OPTIONAL",
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.56-CORRELATION-VALUE-OPTIONAL",
            DECRYPT_CORRELATION_CONTRACT_ID,
            ENCRYPT_CORRELATION_CONTRACT_ID,
        }
        for number in (12, 14, 16):
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}"]
            resolution = gap["source_resolution"]
            self.assertEqual(resolution["related_requirement_ids"], ["KMIPKIT-REQ-SPEC-7.8-001"])
            self.assertEqual(set(resolution["related_source_table_contract_ids"]), request_contract_ids)
            self.assertIn("do not establish response-to-follow-up correlation round-trip behavior", resolution["review_note"])

        for contract_id in request_contract_ids:
            self.assertIn(contract_id, contracts)
        existing_requirement = requirements["KMIPKIT-REQ-SPEC-7.8-001"]
        self.assertFalse(existing_requirement["implementation_refs"])
        self.assertFalse(existing_requirement["verification_refs"])

        for number in (13, 15, 18, 20):
            gap = gaps[f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}"]
            resolution = gap["source_resolution"]
            self.assertEqual(gap["scope_disposition"], "client_1_0")
            self.assertEqual(resolution["related_source_table_contract_ids"], [])
            self.assertIn("response-field contract and executable test", resolution["review_note"])
            self.assertIn("Keep this imperative source gap unresolved", resolution["review_note"])

        self.assertIn(
            "Seven Correlation Value request-table contracts are recorded: five `KMIPKIT-0021` contracts plus only the two optional KMIPKIT-0019 Encrypt/Decrypt request rows",
            summary,
        )
        self.assertIn("do not establish response-to-follow-up correlation round-trip behavior", summary)
        self.assertIn(
            "Encrypt/Decrypt Correlation Value request rows are mapped, while broader operation-specific parameter mapping remains open",
            summary,
        )
        self.assertIn("response Data consumption remains unmapped", summary)
        self.assertNotIn("operation-specific request and response evidence is assigned", summary)

    def test_t002_research_index_matches_procedure_scope_dispositions(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")
        rows = {}
        for line in research.splitlines():
            if not line.startswith("| `KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-"):
                continue
            cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
            rows[cells[0].strip("`")] = cells

        for number in range(11, 21):
            gap_id = f"KMIPKIT-SOURCE-GAP-SPEC-6.1-PROCEDURE-{number:03d}"
            with self.subTest(gap_id=gap_id):
                self.assertIn(gap_id, rows)
                self.assertEqual(rows[gap_id][2].strip("`").strip(), gaps[gap_id]["scope_disposition"])
                self.assertEqual(rows[gap_id][3].strip("`").strip(), "reviewed_procedure_context")

    def test_t002_research_records_already_verified_profile_inclusion_edges(self) -> None:
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")
        lines = research.splitlines()
        start = next(
            index
            for index, line in enumerate(lines)
            if line.startswith("- **Profiles") and "6.13 and" in line and "6.14" in line
        )
        end = next(
            (index for index in range(start + 1, len(lines)) if lines[index].startswith("- **")),
            len(lines),
        )
        summary = "\n".join(lines[start:end])

        self.assertIn("KMIPKIT-INCLUSION-PROF-613-BASIC-SYMMETRIC-CLIENT", summary)
        self.assertIn("KMIPKIT-INCLUSION-PROF-614-BASIC-SYMMETRIC-CLIENT", summary)
        self.assertIn("already encoded in the catalog and independent oracle", summary)
        self.assertNotIn("edges still need to encode", summary)

    def test_t002_reconciles_profile_source_gap_actor_and_scope_dispositions(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}
        profiles_html = (
            ROOT / "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252")
        normalized_profiles_html = re.sub(r"\s+", " ", profiles_html)
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        clauses = {row["clause_id"]: row for row in catalog["source_clauses"]}
        research = PROFILE_RESEARCH_PATH.read_text(encoding="utf-8")

        tls12 = gaps["KMIPKIT-SOURCE-GAP-PROF-3.1.2-4076"]
        self.assertRegex(
            normalized_profiles_html,
            r"Conformant KMIP clients or servers SHALL support the following cipher suites for TLSv1\.2 if TLSv1\.2 is supported",
        )
        self.assertEqual(tls12.get("actor"), "client or server")
        self.assertEqual(tls12["scope_disposition"], "mixed_server_and_out_of_scope")
        self.assertIn("client condition false under product's TLS1.3-only boundary", tls12["actor_scope_review"])
        self.assertIn("server half `server_only`", tls12["actor_scope_review"])
        tls12_clause = clauses["KMIPKIT-CLAUSE-PROF-3.1.2-003"]
        self.assertEqual(tls12_clause["role"], "both")
        self.assertEqual(tls12_clause["scope_state"], "mixed")
        self.assertEqual(tls12_clause["disposition"], "non_applicable")

        for section, gap_id, clause_id, first_line, last_line, discrepancy_id in (
            (
                "6.13",
                "KMIPKIT-SOURCE-GAP-PROF-6.13-10848",
                "KMIPKIT-CLAUSE-PROF-6.13-003",
                10843,
                10853,
                "KMIPKIT-DISC-011",
            ),
            (
                "6.14",
                "KMIPKIT-SOURCE-GAP-PROF-6.14-10867",
                "KMIPKIT-CLAUSE-PROF-6.14-003",
                10862,
                10872,
                "KMIPKIT-DISC-012",
            ),
        ):
            with self.subTest(section=section):
                source_excerpt = re.sub(
                    r"\s+",
                    " ",
                    " ".join(profiles_html.splitlines()[first_line - 1 : last_line]),
                )
                gap = gaps[gap_id]
                self.assertIn("KMIP client implementations conformant to this profile", source_excerpt)
                self.assertIn("Basic Symmetric Key Foundry Client", source_excerpt)
                self.assertEqual(gap.get("actor"), "client")
                self.assertEqual(gap["scope_disposition"], "profile_conditional")
                self.assertIn(discrepancy_id, gap["actor_scope_review"])
                self.assertIn(discrepancy_id, research)
                resolution = gap["source_resolution"]
                self.assertEqual(resolution["kind"], "normative_inclusion")
                self.assertEqual(resolution["source_clause_id"], clause_id)
                self.assertEqual(
                    resolution["inclusion_id"],
                    f"KMIPKIT-INCLUSION-PROF-{section.replace('.', '')}-BASIC-SYMMETRIC-CLIENT",
                )
                self.assertEqual(resolution["included_source_id"], "KMIPKIT-SRC-profiles")
                self.assertEqual(resolution["included_section"], "5.7.1")
                self.assertIn("no standalone requirement ID", resolution["review_note"])
                clause = clauses[clause_id]
                self.assertEqual(clause["role"], "client")
                self.assertEqual(clause["scope_state"], "profile_conditional")
                self.assertEqual(clause["disposition"], "source_discrepancy")


if __name__ == "__main__":
    unittest.main()
