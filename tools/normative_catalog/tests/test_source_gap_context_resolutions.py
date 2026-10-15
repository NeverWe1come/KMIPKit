"""Source excerpts without standalone REQs keep typed, reviewable dispositions."""

from __future__ import annotations

import json
import re
import unittest
from html.parser import HTMLParser
from pathlib import Path

from tools.normative_catalog.validate import _profile_oracle_reference_issue


ROOT = Path(__file__).resolve().parents[3]
ORACLE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
HASH_MAC_SIGNATURE_TRACEABILITY_PATH = ROOT / "specs/021-hash-mac-signature-operations/traceability.md"
HASH_MAC_SIGNATURE_SPEC_PATH = ROOT / "specs/021-hash-mac-signature-operations/spec.md"
ENCRYPT_DECRYPT_SPEC_PATH = ROOT / "specs/019-cryptographic-operations/spec.md"
ENCRYPT_DECRYPT_PAYLOAD_CONTRACT_PATH = ROOT / "specs/019-cryptographic-operations/contracts/operation-payloads.md"
PROFILE_RESEARCH_PATH = ROOT / "specs/010-profile-conformance/research.md"

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

# KMIPKIT-0019 approves both client 1.0 operations, but the profile oracle does
# not yet have source_table_contracts for these request tables. Their generic
# client scope is known; operation-specific parameter mapping remains blocked.
UNMAPPED_GENERIC_CONTEXT_OPERATION_TABLES = {
    "KMIPKIT-ELEM-OP-C2S-DECRYPT": ("Decrypt", 196),
    "KMIPKIT-ELEM-OP-C2S-ENCRYPT": ("Encrypt", 214),
}

EXPECTED_GENERIC_CONTEXT_OPERATIONS = {
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-006": {
        "KMIPKIT-ELEM-OP-C2S-DECRYPT": None,
        "KMIPKIT-ELEM-OP-C2S-ENCRYPT": None,
        "KMIPKIT-ELEM-OP-C2S-HASH": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CRYPTOGRAPHIC-PARAMETERS-REQUIRED"
        },
        "KMIPKIT-ELEM-OP-C2S-MAC": set(),
        "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGN": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": set(),
    },
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-007": {
        "KMIPKIT-ELEM-OP-C2S-DECRYPT": None,
        "KMIPKIT-ELEM-OP-C2S-ENCRYPT": None,
        "KMIPKIT-ELEM-OP-C2S-HASH": {
            "KMIPKIT-SOURCE-GAP-SPEC-6.1.24-CRYPTOGRAPHIC-PARAMETERS-REQUIRED"
        },
        "KMIPKIT-ELEM-OP-C2S-MAC": set(),
        "KMIPKIT-ELEM-OP-C2S-MAC-VERIFY": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGN": set(),
        "KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY": set(),
    },
    "KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-008": {
        "KMIPKIT-ELEM-OP-C2S-DECRYPT": None,
        "KMIPKIT-ELEM-OP-C2S-ENCRYPT": None,
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
            explicit_blocker = gap["scope_disposition"] == "unresolved"

            with self.subTest(gap_id=gap_id):
                if number in (12, 14, 16):
                    self.assertEqual(linked_requirement_ids, ["KMIPKIT-REQ-SPEC-7.8-001"])
                self.assertTrue(
                    requirement_evidence or contract_evidence or explicit_blocker,
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

        self.assertEqual(set(catalog_operations), approved_operation_ids)
        self.assertEqual(approved_operation_ids_019, set(UNMAPPED_GENERIC_CONTEXT_OPERATION_TABLES))
        self.assertEqual(expected_operation_ids_from_contexts, expected_operation_ids)
        for operation_id, (operation_name, table_number) in UNMAPPED_GENERIC_CONTEXT_OPERATION_TABLES.items():
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
        self.assertEqual(expected_mapped_contract_ids, set(EXPECTED_GENERIC_PARAMETER_CONTRACTS))
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
                    if expected_ids is None:
                        operation_name, table_number = UNMAPPED_GENERIC_CONTEXT_OPERATION_TABLES[operation_id]
                        request_contracts = {
                            contract_id
                            for contract_id, contract in contracts.items()
                            if contract.get("operation_element_id") == operation_id
                            and contract.get("table_number") == table_number
                        }
                        self.assertEqual(
                            request_contracts,
                            set(),
                            f"{gap_id} must keep {operation_name} Table {table_number} unmapped until its source contract is reviewed",
                        )
                        note = resolution.get("review_note", "")
                        self.assertEqual(
                            gap["scope_disposition"],
                            "client_1_0",
                            f"{gap_id} must preserve known client scope while leaving {operation_name} Table {table_number} mapping open",
                        )
                        note_requirements = {
                            "KMIPKIT-0019",
                            operation_name,
                            f"Table {table_number}",
                            "source_table_contracts",
                        }
                        self.assertTrue(
                            all(term in note for term in note_requirements),
                            f"{gap_id} needs an explicit blocker for {operation_name} Table {table_number}",
                        )
                        self.assertTrue(request_contracts <= linked_contract_ids)
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

    def test_paragraph_data_rule_remains_unresolved_and_lease_policy_maps_to_attribute(self) -> None:
        oracle = json.loads(ORACLE_PATH.read_text(encoding="utf-8"))
        catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        gaps = {row["gap_id"]: row for row in oracle["source_only_requirement_gaps"]}

        data_rule = gaps["KMIPKIT-SOURCE-GAP-SPEC-6.1-PARAGRAPH-009"]
        self.assertEqual(data_rule["source_resolution"]["kind"], "unresolved_source_interpretation")
        self.assertEqual(data_rule["scope_disposition"], "unresolved")
        self.assertIsNone(data_rule["candidate_requirement_id"])
        self.assertIsNone(data_rule["proposed_requirement_id"])

        lease_policy = gaps["KMIPKIT-SOURCE-GAP-SPEC-4.30-CLIENT-LEASE-READ-ONLY"]
        resolution = lease_policy["source_resolution"]
        self.assertEqual(resolution["kind"], "attribute_policy_mapping")
        self.assertEqual(resolution["element_id"], "KMIPKIT-ELEM-ATTRIBUTE-LEASE-TIME")
        self.assertIsNone(lease_policy["candidate_requirement_id"])
        self.assertIsNone(lease_policy["proposed_requirement_id"])
        lease_time = next(row for row in catalog["elements"] if row["element_id"] == resolution["element_id"])
        self.assertEqual(lease_time["source_modifiable_by_client"], "No")
        self.assertTrue(any("read-only for clients" in row["source_text"] for row in lease_time["source_operation_restrictions"]))

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
        self.assertIn("lowercase `required`", summary)
        self.assertIn("not a formal `REQUIRED` keyword", summary)
        self.assertIn("independent source-gap oracle records `source_keywords: []`", summary)
        self.assertNotIn("its normalized `REQUIRED` hit", summary)

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
            self.assertEqual(gap["scope_disposition"], "unresolved")
            self.assertEqual(resolution["related_source_table_contract_ids"], [])
            self.assertIn("response-field contract and executable test", resolution["review_note"])

        self.assertIn("Five `KMIPKIT-0021` request-table contracts cover Correlation Value fields", summary)
        self.assertIn("do not establish response-to-follow-up correlation round-trip behavior", summary)
        self.assertIn("KMIPKIT-0019 Encrypt/Decrypt request Correlation Value rows still lack `source_table_contracts`", summary)
        self.assertIn("response Data consumption remains unmapped", summary)
        self.assertNotIn("operation-specific request and response evidence is assigned", summary)

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
