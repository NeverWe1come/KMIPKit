"""Contract tests for the offline normative-catalog validator."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.validate import (
    CatalogValidationError,
    _JsonPreflight,
    _check_operation_inventory,
    _check_tag_registry,
    validate_catalog,
)


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


def test_case(*, fixture_path: str | None, fixture_availability: str) -> dict[str, object]:
    return {
        "test_id": "KMIPKIT-TEST-SPEC-001",
        "official_case_id": "KMIP-TC-001",
        "source_id": "KMIPKIT-SRC-testcases",
        "source_section": "1",
        "evidence_category": "conformance",
        "mandatory_status": "unknown",
        "profile_ids": [],
        "requirement_ids": [],
        "element_ids": [],
        "raw_href": "../fixtures/TC-001.xml",
        "fixture_path": fixture_path,
        "fixture_availability": fixture_availability,
        "mapping_confidence": "unmapped",
    }


def profile_record(**overrides: object) -> dict[str, object]:
    record: dict[str, object] = {
        "profile_id": "KMIPKIT-PROFILE-BASELINE",
        "name": "Baseline profile",
        "role": "client",
        "source_refs": [{"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"}],
        "source_clause_ids": [],
        "dependency_profile_ids": [],
        "transport_requirements": ["ttlv_tls"],
        "encoding_requirements": ["ttlv"],
        "applicability": "client_1_0",
        "claim_state": "not_claimed",
        "requirement_ids": [],
        "element_ids": [],
        "test_case_ids": [],
    }
    record.update(overrides)
    return record


def discrepancy_record(**overrides: object) -> dict[str, object]:
    record: dict[str, object] = {
        "discrepancy_id": "KMIPKIT-DISC-001",
        "summary": "Conflicting continuation behavior wording.",
        "source_refs": [
            {"source_id": "KMIPKIT-SRC-spec", "section": "11.5"},
            {"source_id": "KMIPKIT-SRC-profiles", "section": "5.1"},
        ],
        "source_authority": "primary_normative",
        "normative_status": "normative_conflict",
        "alternatives": ["continue", "stop"],
        "affected_requirement_ids": [],
        "affected_element_ids": [],
        "affected_profile_ids": [],
        "affected_policy_ids": [],
        "downstream_impact": "Batch error behavior remains gated.",
        "state": "open",
        "decision_id": None,
        "erratum_source_refs": [],
    }
    record.update(overrides)
    return record


def decision_record(**overrides: object) -> dict[str, object]:
    record: dict[str, object] = {
        "decision_id": "KMIPKIT-DEC-001",
        "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "8.1"}],
        "requirement_ids": [],
        "discrepancy_ids": [],
        "policy_ids": [],
        "interpretation": "Use the reviewed client interpretation.",
        "approver": "Qualified reviewer",
        "approval_evidence": "https://example.invalid/approval",
        "approved_at": "2026-10-04",
        "consequence": "The affected requirement is implementable.",
        "status": "accepted",
    }
    record.update(overrides)
    return record


def deviation_catalog(decision: dict[str, object]) -> dict[str, object]:
    document = minimal_catalog()
    document["source_clauses"] = [
        {
            "clause_id": "KMIPKIT-CLAUSE-SPEC-8.1-001",
            "source_id": "KMIPKIT-SRC-spec",
            "section": "8.1",
            "locator": {"ordinal": 1, "block_kind": "paragraph"},
            "source_keywords": ["SHOULD"],
            "normative_strength": "recommended",
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
            "source_keyword": "SHOULD",
            "normative_strength": "recommended",
            "subject": "client",
            "summary": "A recommended client behavior.",
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
            "decision_id": "KMIPKIT-DEC-001",
            "status": "deviated",
            "review_note": "Reviewed deviation.",
        }
    ]
    document["decisions"] = [decision]
    return document


CLIENT_OPERATIONS = {
    "Activate": "6.1.1", "Add Attribute": "6.1.2", "Adjust Attribute": "6.1.3",
    "Archive": "6.1.4", "Cancel": "6.1.5", "Certify": "6.1.6", "Check": "6.1.7",
    "Create": "6.1.8", "Create Key Pair": "6.1.9", "Create Split Key": "6.1.10",
    "Decrypt": "6.1.11", "Delegated Login": "6.1.12", "Delete Attribute": "6.1.13",
    "Derive Key": "6.1.14", "Destroy": "6.1.15", "Discover Versions": "6.1.16",
    "Encrypt": "6.1.17", "Export": "6.1.18", "Get": "6.1.19",
    "Get Attributes": "6.1.20", "Get Attribute List": "6.1.21", "Get Constraints": "6.1.22",
    "Get Usage Allocation": "6.1.23", "Hash": "6.1.24", "Import": "6.1.25",
    "Interop": "6.1.26", "Join Split Key": "6.1.27", "Locate": "6.1.28", "Log": "6.1.29",
    "Login": "6.1.30", "Logout": "6.1.31", "MAC": "6.1.32", "MAC Verify": "6.1.33",
    "Modify Attribute": "6.1.34", "Obtain Lease": "6.1.35", "Ping": "6.1.36",
    "PKCS#11": "6.1.37", "Poll": "6.1.38", "Process": "6.1.39", "Query": "6.1.40",
    "Query Asynchronous Requests": "6.1.41", "Recover": "6.1.42", "Register": "6.1.43",
    "Revoke": "6.1.44", "Re-certify": "6.1.45", "Re-key": "6.1.46",
    "Re-key Key Pair": "6.1.47", "Re-Provision": "6.1.48", "RNG Retrieve": "6.1.49",
    "RNG Seed": "6.1.50", "Set Attribute": "6.1.51", "Set Constraints": "6.1.52",
    "Set Defaults": "6.1.53", "Set Endpoint Role": "6.1.54", "Sign": "6.1.55",
    "Signature Verify": "6.1.56", "Validate": "6.1.57",
}
SERVER_OPERATIONS = {
    "Discover Versions": "6.2.1", "Notify": "6.2.2", "Put": "6.2.3",
    "Query": "6.2.4", "Set Endpoint Role": "6.2.5",
}
CLIENT_OPERATION_PAYLOAD_TABLES = (
    (164, 165), (167, 168), (170, 171), (173, 174), (176, 177), (179, 180), (183, 184),
    (186, 187), (189, 190), (193, 194), (196, 197), (199, 200), (202, 203), (205, 206),
    (208, 209), (211, 212), (214, 215), (217, 218), (220, 221), (223, 224), (226, 227),
    (229, 230), (232, 233), (235, 236), (238, 239), (241, 242), (244, 245), (247, 248),
    (250, 251), (253, 254), (256, 257), (259, 260), (262, 263), (265, 266), (268, 269),
    (271, 272), (273, 274), (276, None), (278, 279), (282, 283), (285, 286), (288, 289),
    (291, 292), (295, 296), (300, 301), (305, 306), (310, 311), (313, 314), (316, 317),
    (319, 320), (322, 323), (325, 326), (328, 329), (331, 332), (334, 335), (337, 338),
    (340, 341),
)
SERVER_OPERATION_PAYLOAD_TABLES = {
    "Discover Versions": (343, None), "Notify": (None, None), "Put": (None, None),
    "Query": (347, None), "Set Endpoint Role": (349, 350),
}
RESERVED_TAGS = {
    "420009": "(Reserved)", "420014": "(Reserved)", "420015": "(Reserved)",
    "420016": "(Reserved)", "420017": "(Reserved)", "42001A": "(Reserved)",
    "42001B": "(Reserved)", "42001C": "(Reserved)", "42001F": "(Reserved)",
    "42002D": "(Reserved)", "42003B": "(Reserved)", "42005D": "(Reserved)",
    "420065": "(Reserved)", "42006E": "(Reserved)", "420087": "(Reserved)",
    "420090": "(Reserved)", "420091": "(Reserved)", "420137": "Reserved",
    "42013E": "(Reserved)", "42013F": "(Reserved)",
}
EXPECTED_TAG_RANGES = (
    ("unused", "000000 - 420000"),
    ("reserved", "420XXX \u2013 42FFFF"),
    ("unused", "430000 \u2013 53FFFF"),
    ("extension", "540000 \u2013 54FFFF"),
    ("unused", "550000 - FFFFFF"),
)


def operation_element(name: str, section: str, direction: str) -> dict[str, object]:
    slug = re.sub(r"[^A-Z0-9]+", "-", name.upper()).strip("-")
    direction_code = "C2S" if direction == "client_to_server" else "S2C"
    scope = "client_1_0" if direction == "client_to_server" else "client_1_1"
    if direction == "client_to_server":
        operation_names = list(CLIENT_OPERATIONS)
        request_table, response_table = CLIENT_OPERATION_PAYLOAD_TABLES[operation_names.index(name)]
    else:
        request_table, response_table = SERVER_OPERATION_PAYLOAD_TABLES[name]
    payload_tables: list[dict[str, object]] = []
    for role, table_number in (("request", request_table), ("response", response_table)):
        if table_number is not None:
            caption = f"{name} {role.title()} Payload"
            if name == "Query Asynchronous Requests" and role == "response":
                caption = "PKCS#11 Response Payload"
            payload_tables.append({"role": role, "table_number": table_number, "caption": caption})
    record: dict[str, object] = {
        "element_id": f"KMIPKIT-ELEM-OP-{direction_code}-{slug}",
        "kind": "operation",
        "name": name,
        "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": section}],
        "direction": direction,
        "scope_state": scope,
        "payload_tables": payload_tables,
        "asynchronous_response": {
            "Cancel": "cancellation_result_not_async",
            "Poll": "pending_or_original_operation_payload",
        }.get(name),
        "parent_element_ids": [],
        "requirement_ids": [],
        "profile_ids": [],
        "test_case_ids": [],
        "feature_spec": None,
        "implementation_refs": [],
        "verification_refs": [],
    }
    if direction == "server_to_client":
        record["scope_reason"] = "Server-initiated operation support is scheduled for KMIPKit 1.1."
    return record


class CatalogValidationTests(unittest.TestCase):
    def test_json_number_preflight_matches_without_copying_remaining_input(self) -> None:
        class NoSlice(str):
            def __getitem__(self, key: object) -> str:
                if isinstance(key, slice) and key.stop is None:
                    raise AssertionError("preflight copied the unconsumed JSON suffix")
                return super().__getitem__(key)  # type: ignore[arg-type]

        _JsonPreflight(NoSlice('{"schema_version":1}')).validate()

    def test_operation_inventory_matches_all_client_and_server_definitions(self) -> None:
        elements = [
            *(operation_element(name, section, "client_to_server") for name, section in CLIENT_OPERATIONS.items()),
            *(operation_element(name, section, "server_to_client") for name, section in SERVER_OPERATIONS.items()),
        ]
        _check_operation_inventory(elements)

        missing_operation = elements[:-1]
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(missing_operation)

    def test_operation_inventory_rejects_a_wrong_source_section(self) -> None:
        elements = [
            *(operation_element(name, section, "client_to_server") for name, section in CLIENT_OPERATIONS.items()),
            *(operation_element(name, section, "server_to_client") for name, section in SERVER_OPERATIONS.items()),
        ]
        elements[0]["source_refs"] = [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.2"}]
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(elements)

    def test_operation_inventory_requires_exact_payload_tables_and_async_responses(self) -> None:
        elements = [
            *(operation_element(name, section, "client_to_server") for name, section in CLIENT_OPERATIONS.items()),
            *(operation_element(name, section, "server_to_client") for name, section in SERVER_OPERATIONS.items()),
        ]
        _check_operation_inventory(elements)

        query_async = next(row for row in elements if row["name"] == "Query Asynchronous Requests")
        query_async["payload_tables"][-1]["caption"] = "Query Asynchronous Requests Response Payload"
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(elements)

        poll = next(row for row in elements if row["name"] == "Poll")
        poll["payload_tables"].append({"role": "response", "table_number": 277, "caption": "Poll Response Payload"})
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            _check_operation_inventory(elements)

    def test_tag_inventory_reconciles_all_reserved_values_and_ranges(self) -> None:
        elements = [
            {
                "element_id": f"KMIPKIT-ELEM-TAG-{value}",
                "kind": "tag",
                "name": name,
                "wire_value": value,
                "allocation": "reserved",
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
            for value, name in RESERVED_TAGS.items()
        ]
        for index in range(354):
            value = f"A{index:05X}"
            elements.append(
                {
                    "element_id": f"KMIPKIT-ELEM-TAG-{value}",
                    "kind": "tag",
                    "name": f"Test Tag {index}",
                    "wire_value": value,
                    "allocation": "assigned",
                    "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
                }
            )
        ranges = [
            {
                "range_id": f"KMIPKIT-RANGE-{index:03}",
                "value_range": value_range,
                "allocation": allocation,
                "source_order": index,
                "source_refs": [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}],
            }
            for index, (allocation, value_range) in enumerate(EXPECTED_TAG_RANGES, start=1)
        ]
        _check_tag_registry(elements, ranges)

        elements[17]["name"] = "(Reserved)"
        with self.assertRaisesRegex(CatalogValidationError, "reserved tag"):
            _check_tag_registry(elements, ranges)

    def test_accepts_exact_pinned_source_manifest_and_empty_record_collections(self) -> None:
        result = validate(minimal_catalog())
        self.assertEqual(result["source_count"], 4)

    def test_complete_validation_rejects_omitted_operation_and_element_records(self) -> None:
        raw = json.dumps(minimal_catalog()).encode("utf-8")
        with self.assertRaisesRegex(CatalogValidationError, "operation"):
            validate_catalog(raw, ROOT, require_complete=True)

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

    def test_duplicate_key_errors_do_not_echo_untrusted_control_text(self) -> None:
        raw = b'{"schema_version":{"attacker\\u001b\\n":1,"attacker\\u001b\\n":2}}'
        with self.assertRaisesRegex(CatalogValidationError, "duplicate JSON object keys") as raised:
            validate_catalog(raw, ROOT)
        self.assertNotIn("attacker", str(raised.exception))
        self.assertNotIn("\x1b", str(raised.exception))

    def test_preflight_rejects_unknown_top_level_key_before_object_decode(self) -> None:
        raw = b'{"unexpected":{"nested":[1,2,3]}}'
        with patch("tools.normative_catalog.validate.json.loads", wraps=json.loads) as decoder:
            with self.assertRaisesRegex(CatalogValidationError, "unknown top-level"):
                validate_catalog(raw, ROOT)
        self.assertNotIn(raw.decode("utf-8"), [call.args[0] for call in decoder.call_args_list])

    def test_preflight_rejects_duplicate_keys_before_object_decode(self) -> None:
        raw = b'{"schema_version":1,"schema_version":1}'
        with patch("tools.normative_catalog.validate.json.loads", wraps=json.loads) as decoder:
            with self.assertRaisesRegex(CatalogValidationError, "duplicate"):
                validate_catalog(raw, ROOT)
        self.assertNotIn(raw.decode("utf-8"), [call.args[0] for call in decoder.call_args_list])

    def test_preflight_rejects_oversized_string_before_object_decode(self) -> None:
        raw = b'{"schema_version":"' + b'a' * 65_537 + b'"}'
        with patch("tools.normative_catalog.validate.json.loads", wraps=json.loads) as decoder:
            with self.assertRaisesRegex(CatalogValidationError, "string"):
                validate_catalog(raw, ROOT)
        self.assertNotIn(raw.decode("utf-8"), [call.args[0] for call in decoder.call_args_list])
        self.assertFalse(any(len(call.args[0]) > 65_536 for call in decoder.call_args_list))

    def test_rejects_invalid_utf8_and_unpaired_escaped_surrogate(self) -> None:
        for raw in (b"\xff", b'{"schema_version":"\\ud800"}'):
            with self.subTest(raw=raw), self.assertRaises(CatalogValidationError):
                validate_catalog(raw, ROOT)

    def test_accepts_a_well_formed_escaped_surrogate_pair(self) -> None:
        from tools.normative_catalog.validate import _reject_surrogates

        _reject_surrogates("\ud83d\ude00")

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
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
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
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
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
                "feature_spec": None,
                "implementation_refs": [],
                "verification_refs": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_unsafe_fixture_paths_without_opening_them(self) -> None:
        for unsafe_path in (
            "C:/secret.xml",
            "/secret.xml",
            "../outside.xml",
            "specification/oasis/kmip-2.1/../../outside.xml",
            "specification/oasis/kmip-2.1/fixtures\\secret.xml",
            "\\\\server\\share\\fixture.xml",
        ):
            document = minimal_catalog()
            document["test_cases"] = [test_case(fixture_path=unsafe_path, fixture_availability="available")]
            with self.subTest(path=unsafe_path), self.assertRaises(CatalogValidationError):
                validate(document)

    def test_rejects_fixture_claimed_available_when_git_tree_has_no_file(self) -> None:
        document = minimal_catalog()
        document["test_cases"] = [
            test_case(
                fixture_path="specification/oasis/kmip-2.1/fixtures/TC-001.xml",
                fixture_availability="available",
            )
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_command_line_validator_reports_catalog_counts(self) -> None:
        script = ROOT / "tools" / "normative_catalog" / "validate.py"
        result = subprocess.run(
            [sys.executable, str(script), "--repo-root", str(ROOT), "--structural-only"],
            cwd=ROOT,
            capture_output=True,
            check=False,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("sources=4", result.stdout)

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
                "affected_element_kinds": ["enumeration_value"],
                "requirement_ids": [],
            }
        ]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_rejects_profile_with_unknown_claim_state(self) -> None:
        document = minimal_catalog()
        document["profiles"] = [profile_record(claim_state="certified")]
        with self.assertRaisesRegex(CatalogValidationError, "profile"):
            validate(document)

    def test_profile_enum_with_non_string_json_type_fails_as_validation_error(self) -> None:
        document = minimal_catalog()
        document["profiles"] = [profile_record(role=[])]
        with self.assertRaises(CatalogValidationError):
            validate(document)

    def test_open_discrepancy_cannot_select_a_decision(self) -> None:
        document = minimal_catalog()
        document["decisions"] = [decision_record()]
        document["discrepancies"] = [discrepancy_record(decision_id="KMIPKIT-DEC-001")]
        with self.assertRaisesRegex(CatalogValidationError, "open discrepancy"):
            validate(document)

    def test_recommendation_deviation_requires_accepted_decision_evidence(self) -> None:
        document = deviation_catalog(
            decision_record(status="proposed", approver="", approval_evidence="")
        )
        with self.assertRaisesRegex(CatalogValidationError, "accepted decision"):
            validate(document)

    def test_deviation_decision_must_name_the_affected_requirement(self) -> None:
        with self.assertRaisesRegex(CatalogValidationError, "decision.*requirement"):
            validate(deviation_catalog(decision_record()))

    def test_resolved_discrepancy_decision_must_name_the_discrepancy(self) -> None:
        document = minimal_catalog()
        document["decisions"] = [decision_record()]
        document["discrepancies"] = [
            discrepancy_record(state="resolved_by_approved_decision", decision_id="KMIPKIT-DEC-001")
        ]
        with self.assertRaisesRegex(CatalogValidationError, "decision.*discrepancy"):
            validate(document)

    def test_profile_cannot_claim_complete_evidence_without_resolved_links(self) -> None:
        document = minimal_catalog()
        document["profiles"] = [profile_record(claim_state="evidence_complete")]
        with self.assertRaisesRegex(CatalogValidationError, "profile.*evidence"):
            validate(document)

    def test_discrepancy_authority_must_derive_from_cited_source(self) -> None:
        document = minimal_catalog()
        document["discrepancies"] = [
            discrepancy_record(
                source_refs=[{"source_id": "KMIPKIT-SRC-usage-guide", "section": "4.1"}],
                source_authority="primary_normative",
            )
        ]
        with self.assertRaisesRegex(CatalogValidationError, "authority does not match"):
            validate(document)

    def test_erratum_resolution_requires_pinned_erratum_source(self) -> None:
        document = minimal_catalog()
        document["discrepancies"] = [discrepancy_record(state="resolved_by_erratum")]
        with self.assertRaisesRegex(CatalogValidationError, "erratum.*pinned"):
            validate(document)

    def test_policy_provenance_requires_a_verifiable_locator(self) -> None:
        document = minimal_catalog()
        document["policies"] = [
            {
                "policy_id": "KMIPKIT-POLICY-UNKNOWN-VALUES",
                "summary": "Preserve unknown values.",
                "provenance": "AGENTS.md",
                "affected_element_kinds": ["enumeration_value"],
                "requirement_ids": [],
            }
        ]
        with self.assertRaisesRegex(CatalogValidationError, "project policy"):
            validate(document)

    def test_policy_provenance_accepts_an_existing_exact_heading(self) -> None:
        document = minimal_catalog()
        document["policies"] = [
            {
                "policy_id": "KMIPKIT-POLICY-UNKNOWN-VALUES",
                "summary": "Preserve unknown values.",
                "provenance": "AGENTS.md",
                "provenance_ref": {"path": "AGENTS.md", "heading": "9. Public API and compatibility"},
                "affected_element_kinds": ["enumeration_value"],
                "requirement_ids": [],
            }
        ]
        self.assertEqual(validate(document)["source_count"], 4)

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
