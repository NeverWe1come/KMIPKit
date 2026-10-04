"""Contract tests for the offline normative-catalog validator."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tools.normative_catalog.validate import CatalogValidationError, validate_catalog


ROOT = Path(__file__).resolve().parents[3]


def minimal_catalog() -> dict[str, object]:
    """Return the smallest catalog with exact pinned source metadata."""
    source_rows = [
        (
            "profiles",
            "Key Management Interoperability Protocol Profiles Version 2.1",
            "OASIS Standard",
            "14 December 2020",
            "upstream/kmip-profiles-v2.1-os.html",
            "https://docs.oasis-open.org/kmip/kmip-profiles/v2.1/os/kmip-profiles-v2.1-os.html",
            "f11144aa793335f38dbd1864ec4709ca83ac80f588bbb790ba37f6c53be207a9",
            "profile_normative",
        ),
        (
            "spec",
            "Key Management Interoperability Protocol Specification Version 2.1",
            "OASIS Standard",
            "14 December 2020",
            "upstream/kmip-spec-v2.1-os.html",
            "https://docs.oasis-open.org/kmip/kmip-spec/v2.1/os/kmip-spec-v2.1-os.html",
            "8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf",
            "primary_normative",
        ),
        (
            "testcases",
            "Key Management Interoperability Protocol Test Cases Version 2.1",
            "Committee Note 01",
            "16 November 2020",
            "upstream/kmip-testcases-v2.1-cn01.html",
            "https://docs.oasis-open.org/kmip/kmip-testcases/v2.1/cn01/kmip-testcases-v2.1-cn01.html",
            "305e0638af25aa8163a2b4a67a1a617a0207af67b3b3032fa499966b22d42b88",
            "test_evidence",
        ),
        (
            "usage-guide",
            "Key Management Interoperability Protocol Usage Guide Version 2.1",
            "Committee Note 01",
            "16 November 2020",
            "upstream/kmip-ug-v2.1-cn01.html",
            "https://docs.oasis-open.org/kmip/kmip-ug/v2.1/cn01/kmip-ug-v2.1-cn01.html",
            "f6997ac3ac34d588b2b43ace49835dceaa6af2086f9472a79c27bce43f5cbe38",
            "informative",
        ),
    ]
    sources = [
        {
            "source_id": f"KMIPKIT-SRC-{slug}",
            "title": title,
            "version": "2.1",
            "stage": stage,
            "publication_date": date,
            "local_path": f"specification/oasis/kmip-2.1/{path}",
            "canonical_url": url,
            "sha256": digest,
            "authority_class": authority,
        }
        for slug, title, stage, date, path, url, digest, authority in source_rows
    ]
    return {
        "schema_version": 1,
        "sources": sources,
        "source_clauses": [],
        "elements": [],
        "tag_ranges": [],
        "requirements": [],
        "policies": [],
        "profiles": [],
        "test_cases": [],
        "discrepancies": [],
        "decisions": [],
    }


def validate(document: dict[str, object]) -> dict[str, object]:
    return validate_catalog(json.dumps(document).encode("utf-8"), ROOT)


class CatalogValidationTests(unittest.TestCase):
    def test_accepts_exact_pinned_source_manifest_and_empty_record_collections(self) -> None:
        result = validate(minimal_catalog())
        self.assertEqual(result["source_count"], 4)

    def test_rejects_unknown_top_level_fields(self) -> None:
        document = minimal_catalog()
        document["unexpected"] = []
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_source_checksum_mismatch(self) -> None:
        document = minimal_catalog()
        document["sources"][1]["sha256"] = "0" * 64
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_duplicate_json_keys(self) -> None:
        raw = b'{"schema_version":1,"schema_version":1}'
        with self.assertRaises(CatalogValidationError):
            validate_catalog(raw, ROOT)

    def test_rejects_invalid_utf8_and_unpaired_escaped_surrogate(self) -> None:
        for raw in (b"\xff", b'{"x":"\\ud800"}'):
            with self.subTest(raw=raw), self.assertRaises(CatalogValidationError):
                validate_catalog(raw, ROOT)

    def test_rejects_excessive_input_before_object_construction(self) -> None:
        with self.assertRaises(CatalogValidationError):
            validate_catalog(b" " * (16 * 1024 * 1024 + 1), ROOT)

    def test_rejects_unanchored_record_identifiers(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "prefix-KMIPKIT-ELEM-OP-CREATE",
                "kind": "operation",
                "name": "Create",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.1"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_unknown_protocol_element_fields(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-OP-CREATE",
                "kind": "operation",
                "name": "Create",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.1"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "unexpected": "value",
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_unresolved_protocol_element_relationships(self) -> None:
        document = minimal_catalog()
        document["elements"] = [
            {
                "element_id": "KMIPKIT-ELEM-OP-CREATE",
                "kind": "operation",
                "name": "Create",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.1"}],
                "direction": "client_to_server",
                "scope_state": "client_1_0",
                "parent_element_ids": [],
                "requirement_ids": [],
                "profile_ids": ["KMIPKIT-PROFILE-MISSING"],
                "test_case_ids": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_requires_every_top_level_record_collection(self) -> None:
        document = minimal_catalog()
        del document["policies"]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_clause_with_unknown_disposition_or_no_review_rationale(self) -> None:
        document = minimal_catalog()
        document["source_clauses"] = [
            {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-1.1-001",
                "source_id": "KMIPKIT-SRC-spec",
                "section": "1.1",
                "locator": {"ordinal": 1, "block_kind": "paragraph"},
                "source_keywords": ["MAY"],
                "normative_strength": "permission_or_optional",
                "disposition": "maybe",
                "requirement_ids": [],
                "exclusion_rationale": None,
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_tag_range_using_singleton_tag_allocation(self) -> None:
        document = minimal_catalog()
        document["tag_ranges"] = [
            {
                "range_id": "KMIPKIT-RANGE-001",
                "value_range": "0x420000",
                "allocation": "assigned",
                "source_order": 1,
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_project_policy_without_valid_provenance(self) -> None:
        document = minimal_catalog()
        document["policies"] = [
            {
                "policy_id": "KMIPKIT-POLICY-UNKNOWN-VALUES",
                "summary": "Preserve unknown values.",
                "provenance": "a random note",
                "affected_element_classes": ["enumeration_value"],
                "requirement_ids": [],
                "source_refs": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_requirement_with_unresolved_source_references(self) -> None:
        document = minimal_catalog()
        document["requirements"] = [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-404"],
                "source_refs": [{"source_id": "KMIPKIT-SRC-missing", "section": "8.1"}],
                "source_keyword": "MUST",
                "normative_strength": "mandatory",
                "subject": "client",
                "summary": "A client obligation.",
                "role": "client",
                "direction": "client_to_server",
                "condition": None,
                "scope_state": "client_1_0",
                "element_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "negative_verification_required": False,
                "decision_id": None,
                "status": "unassigned",
                "review_note": None,
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_requires_prohibited_requirements_to_have_negative_verification(self) -> None:
        document = minimal_catalog()
        document["source_clauses"] = [
            {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
                "source_id": "KMIPKIT-SRC-spec",
                "section": "8.1",
                "locator": {"ordinal": 1, "block_kind": "paragraph"},
                "source_keywords": ["MUST NOT"],
                "normative_strength": "prohibited",
                "disposition": "requirement",
                "requirement_ids": ["KMIPKIT-REQ-SPEC-8.1-001"],
                "exclusion_rationale": None,
            }
        ]
        document["requirements"] = [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-8.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-8.1-001"],
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
                "source_keyword": "MUST NOT",
                "normative_strength": "prohibited",
                "subject": "client",
                "summary": "A prohibited behavior.",
                "role": "client",
                "direction": "client_to_server",
                "condition": None,
                "scope_state": "client_1_0",
                "element_ids": [],
                "profile_ids": [],
                "test_case_ids": [],
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
                "negative_verification_required": False,
                "decision_id": None,
                "status": "unassigned",
                "review_note": None,
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)


if __name__ == "__main__":
    unittest.main()
