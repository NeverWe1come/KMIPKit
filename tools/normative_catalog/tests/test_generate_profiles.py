"""Tests for catalog-derived, fail-closed client profile metadata."""

from __future__ import annotations

import html
import io
import json
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import call, patch
from xml.etree import ElementTree

from tools.normative_catalog.generate_profiles import (
    CATALOG_PATH,
    MAX_CATALOG_BYTES,
    ProfileGenerationError,
    load_profile_targets,
    compute_profile_closure,
    main as generate_profiles_main,
    parse_profile_targets,
    ProfileTargets,
    render_profiles_source,
    rust_string_literal,
)
from tools.normative_catalog.audit_sources import audit_document
from tools.normative_catalog.validate import load_validated_catalog


ROOT = Path(__file__).resolve().parents[3]


def _mapping_applies_to_profile(
    mapping: dict[str, object],
    profile_id: str,
) -> bool:
    """Use explicit source-reviewed applicability, including normative inheritance."""
    applies_to = mapping.get("applies_to")
    inherited_by_profiles = mapping.get("inherited_by_profiles", [])
    return (
        applies_to == "all_client_profiles"
        or applies_to == profile_id
        or (isinstance(applies_to, list) and profile_id in applies_to)
        or (isinstance(inherited_by_profiles, list) and profile_id in inherited_by_profiles)
    )


def _normalized_html_source_span(lines: list[str], line_range: list[int]) -> str:
    """Normalize one-based pinned HTML lines to source text for crosswalk assertions."""
    source_span = " ".join(lines[line_range[0] - 1 : line_range[1]])
    return " ".join(html.unescape(re.sub(r"<[^>]+>", " ", source_span)).split())


def closure_catalog() -> dict[str, object]:
    return {
        "source_clauses": [
            {"clause_id": "KMIPKIT-CLAUSE-PROF-6.1-001", "source_id": "profiles", "section": "6.1"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-6.1.1-001", "source_id": "profiles", "section": "6.1.1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-1-001", "source_id": "spec", "section": "1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-1.1-001", "source_id": "spec", "section": "1.1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-2-001", "source_id": "spec", "section": "2"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001", "source_id": "profiles", "section": "3.1"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-3.1.1-001", "source_id": "profiles", "section": "3.1.1"},
        ],
        "normative_inclusions": [
            {
                "inclusion_id": "KMIPKIT-INCLUSION-001",
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.1-001",
                "included_source_id": "spec",
                "included_section": None,
            },
            {
                "inclusion_id": "KMIPKIT-INCLUSION-002",
                "source_clause_id": "KMIPKIT-CLAUSE-SPEC-1-001",
                "included_source_id": "profiles",
                "included_section": "3.1",
            },
        ],
        "requirements": [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                "role": "client",
                "scope_state": "client_1_0",
            },
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
                "role": "client",
                "scope_state": "client_1_0",
            },
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-2-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-2-001"],
                "role": "server",
                "scope_state": "server_only",
            },
            {
                "requirement_id": "KMIPKIT-REQ-PROF-3.1.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-3.1.1-001"],
                "role": "client",
                "scope_state": "profile_conditional",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            },
        ],
    }


class ProfileMetadataGeneratorTests(unittest.TestCase):
    def test_current_baseline_condition_summary_matches_reviewed_structural_crosswalks(self) -> None:
        """The current T002 summary must not describe structurally reviewed items as pending."""
        research = (ROOT / "specs" / "010-profile-conformance" / "research.md").read_text(encoding="utf-8")
        summary = research.split("## Normative-reference cross-check for T002", maxsplit=1)[1].split(
            "## Fixture inventory update", maxsplit=1
        )[0]

        self.assertIn("Items 1–8 have reviewed inventory/crosswalk dispositions", summary)
        self.assertIn("item 5 and item 6 statuses are structural-only", summary)
        self.assertNotIn("Item 6 remains pending crosswalk review", summary)

    def test_maximum_response_size_evidence_does_not_claim_general_server_policy(self) -> None:
        """Keep Query test evidence and the error reason distinct from a general server rule."""
        research = (ROOT / "specs" / "010-profile-conformance" / "research.md").read_text(encoding="utf-8")
        implementation_evidence = (
            ROOT / "specs" / "010-profile-conformance" / "implementation-evidence.md"
        ).read_text(encoding="utf-8")

        self.assertNotIn(
            "no numeric default, numeric range, or server behavior when a declared limit is exceeded",
            research,
        )
        self.assertNotIn("server behavior on an exceeded limit", implementation_evidence)
        for document in (research, implementation_evidence):
            with self.subTest(document="research" if document is research else "implementation evidence"):
                self.assertIn("no general server response policy", document)
                self.assertIn("not a general server SHALL", document)

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertEqual(
            _normalized_html_source_span(profiles_lines, [5190, 5197]),
            'name="_Toc27473253"> 5.3.3.1 MSGENC-HTTPS-M-1-21 Perform a Query operation, querying the Operations '
            "and Objects supported by the server, with a restriction on the Maximum Response Size set in the "
            "request header. Since the resulting Query response is too big, an error is returned. Increase the "
            "Maximum Response Size, resubmit the Query request, and get a successful response.",
        )

        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [56703, 56713]),
            "Response Too Large Maximum Response Size has been exceeded",
        )

    def test_response_too_large_table_is_cited_under_section_11_46(self) -> None:
        """Table 478 belongs to §11.46 Result Reason Enumeration, not §13.1.4."""
        documents = (
            ROOT / "specs" / "010-profile-conformance" / "research.md",
            ROOT / "specs" / "010-profile-conformance" / "implementation-evidence.md",
        )
        for path in documents:
            with self.subTest(path=path.name):
                document = path.read_text(encoding="utf-8")
                self.assertIn("§11.46 Table 478", document)
                self.assertNotIn("§13.1.4 Table 478", document)

        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [55918, 55920]),
            "11.46 Result Reason Enumeration",
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [56921, 56921]),
            "Table 478: Result Reason Encoding Descriptions",
        )

    def test_https_client_inherits_baseline_basic_authentication_root(self) -> None:
        """Profiles v2.1 §§6.4 and 6.1 make §3.1 inherited, not direct."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-HTTPS-CLIENT"
        profile = next(item for item in catalog["profiles"] if item["profile_id"] == profile_id)
        expected = next(item for item in oracle["profiles"] if item["profile_id"] == profile_id)
        inherited_clause_id = "KMIPKIT-CLAUSE-PROF-3.1-001"
        reviewed_exclusion = next(
            item
            for item in expected["direct_root_exclusions"]
            if item["source_clause_id"] == inherited_clause_id
        )

        self.assertEqual(reviewed_exclusion["expected_disposition"], "inherited_not_direct")
        self.assertNotIn(inherited_clause_id, profile["direct_source_clause_ids"])
        self.assertIn(inherited_clause_id, profile["source_clause_ids"])

    def test_foundry_intermediate_and_advanced_clients_inherit_basic_conditions(self) -> None:
        """Profiles v2.1 §§6.13–6.14 item 3 both incorporate §5.7.1 literally."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        source_clause_ids = {
            "KMIPKIT-CLAUSE-PROF-5.7.1-001",
            "KMIPKIT-CLAUSE-PROF-5.7.1-002",
            "KMIPKIT-CLAUSE-PROF-5.7.1-003",
        }
        requirement_ids = {
            "KMIPKIT-REQ-PROF-5.7.1-001",
            "KMIPKIT-REQ-PROF-5.7.1-002",
            "KMIPKIT-REQ-PROF-5.7.1-003",
        }
        cases = (
            (
                "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT",
                "KMIPKIT-CLAUSE-PROF-6.13-003",
                "KMIPKIT-INCLUSION-PROF-613-BASIC-SYMMETRIC-CLIENT",
                "Profiles v2.1 §6.13 item 3",
            ),
            (
                "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT",
                "KMIPKIT-CLAUSE-PROF-6.14-003",
                "KMIPKIT-INCLUSION-PROF-614-BASIC-SYMMETRIC-CLIENT",
                "Profiles v2.1 §6.14 item 3",
            ),
        )
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}
        oracle_profiles = {item["profile_id"]: item for item in oracle["profiles"]}
        catalog_requirements = {item["requirement_id"]: item for item in catalog["requirements"]}

        for profile_id, source_clause_id, inclusion_id, source_locator in cases:
            with self.subTest(profile_id=profile_id):
                profile = catalog_profiles[profile_id]
                expected = oracle_profiles[profile_id]
                inclusion = next(
                    (item for item in catalog["normative_inclusions"] if item["inclusion_id"] == inclusion_id),
                    None,
                )
                oracle_edge = next(
                    (
                        item
                        for item in oracle["provenance"]["inclusion_edges"]
                        if item["inclusion_id"] == inclusion_id
                    ),
                    None,
                )

                self.assertIn(source_clause_id, profile["direct_source_clause_ids"])
                self.assertNotIn("KMIPKIT-CLAUSE-PROF-5.7.1-001", profile["direct_source_clause_ids"])
                self.assertEqual(
                    inclusion,
                    {
                        "inclusion_id": inclusion_id,
                        "source_clause_id": source_clause_id,
                        "included_source_id": "KMIPKIT-SRC-profiles",
                        "included_section": "5.7.1",
                    },
                )
                self.assertEqual(
                    oracle_edge,
                    {
                        "inclusion_id": inclusion_id,
                        "source_clause_id": source_clause_id,
                        "included_source_id": "KMIPKIT-SRC-profiles",
                        "included_section": "5.7.1",
                        "source_locator": source_locator,
                    },
                )
                self.assertIn(inclusion_id, expected["normative_inclusion_ids"])
                self.assertTrue(source_clause_ids.issubset(expected["source_clause_ids"]))
                self.assertTrue(requirement_ids.issubset(expected["requirement_ids"]))
                self.assertTrue(requirement_ids.issubset(expected["requirement_ids_audited_subset"]))
                inherited_ids = sorted(
                    {
                        "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT",
                        "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT",
                    }
                )
                requirement_mappings = {
                    item["requirement_id"]: item
                    for item in oracle["requirement_crosswalk"]
                    if item["requirement_id"] in requirement_ids
                }
                source_reviews = {
                    item["expected_client_requirement_ids"][0]: item
                    for item in oracle["source_requirement_reviews"]
                    if item.get("expected_client_requirement_ids")
                    and item["expected_client_requirement_ids"][0] in requirement_ids
                }
                for requirement_id in requirement_ids:
                    self.assertEqual(
                        requirement_mappings[requirement_id]["inherited_by_profiles"],
                        inherited_ids,
                    )
                    self.assertEqual(
                        source_reviews[requirement_id]["inherited_by_profiles"],
                        inherited_ids,
                    )
                self.assertTrue(
                    source_clause_ids.issubset(
                        {
                            item["source_clause_id"]
                            for item in expected["direct_root_exclusions"]
                            if item["expected_disposition"] == "included_not_direct"
                        }
                    )
                )
                for requirement_id in requirement_ids:
                    self.assertIn(profile_id, catalog_requirements[requirement_id]["profile_ids"])
                    self.assertIn(requirement_id, profile["requirement_ids"])

                closure = compute_profile_closure(
                    catalog,
                    profile["direct_source_clause_ids"],
                    profile_id=profile_id,
                )
                self.assertTrue(source_clause_ids.issubset(closure.source_clause_ids))
                self.assertTrue(requirement_ids.issubset(closure.requirement_ids))

    def test_all_client_profiles_inherit_baseline_conditions_from_section_6_1_item_3(self) -> None:
        """Every profile dependency on Baseline also closes over §5.1.1 conditions."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profiles_html = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252")
        normalized_profiles_html = " ".join(profiles_html.split())
        inclusion_id = "KMIPKIT-INCLUSION-PROF-BASELINE-CONDITIONS"
        source_clause_id = "KMIPKIT-CLAUSE-PROF-5.1.1-001"
        source_clause = "KMIPKIT-CLAUSE-PROF-6.1-003"
        edge = next(
            (item for item in catalog["normative_inclusions"] if item["inclusion_id"] == inclusion_id),
            None,
        )
        expected_edge = next(
            (
                item
                for item in oracle["provenance"]["inclusion_edges"]
                if item["inclusion_id"] == inclusion_id
            ),
            None,
        )

        self.assertIn("SHALL support the Baseline Client conditions (5.1.1)", normalized_profiles_html)
        self.assertEqual(
            edge,
            {
                "inclusion_id": inclusion_id,
                "source_clause_id": source_clause,
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.1.1",
            },
        )
        self.assertEqual(
            expected_edge,
            {
                "inclusion_id": inclusion_id,
                "source_clause_id": source_clause,
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.1.1",
                "source_locator": "Profiles v2.1 §6.1 item 3",
            },
        )

        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"] if item["role"] == "client"}
        oracle_profiles = {item["profile_id"]: item for item in oracle["profiles"] if item["role"] == "client"}
        self.assertEqual(set(catalog_profiles), set(oracle_profiles))
        for profile_id in sorted(catalog_profiles):
            with self.subTest(profile_id=profile_id):
                profile = catalog_profiles[profile_id]
                expected = oracle_profiles[profile_id]
                self.assertIn(inclusion_id, expected["normative_inclusion_ids"])
                self.assertIn(source_clause_id, expected["source_clause_ids"])
                self.assertIn(source_clause_id, profile["source_clause_ids"])
                if profile_id == "KMIPKIT-PROFILE-BASELINE-CLIENT":
                    self.assertIn(source_clause_id, profile["direct_source_clause_ids"])
                else:
                    self.assertNotIn(source_clause_id, profile["direct_source_clause_ids"])
                self.assertNotIn("KMIPKIT-REQ-PROF-5.1.1-001", expected["requirement_ids"])

    def test_all_client_profiles_close_over_the_shared_batch_item_id_condition(self) -> None:
        """The Profiles §4.1.2 item that says 'for all profiles' belongs to every client closure."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json").read_text(
                encoding="utf-8"
            )
        )
        profiles_html = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252")
        normalized_profiles_text = " ".join(re.sub(r"<[^>]+>", " ", html.unescape(profiles_html)).split())
        clause_id = "KMIPKIT-CLAUSE-PROF-4.1.2-007"
        requirement_id = "KMIPKIT-REQ-PROF-4.1.2-007"
        section_clause_ids = {
            item["clause_id"]
            for item in catalog["source_clauses"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "4.1.2"
        }

        self.assertIn("For all profiles in each Batch Item of a client request", normalized_profiles_text)
        self.assertIn("Unique Batch Item ID", normalized_profiles_text)
        self.assertEqual(len(section_clause_ids), 10)
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"] if item["role"] == "client"}
        oracle_profiles = {item["profile_id"]: item for item in oracle["profiles"] if item["role"] == "client"}
        self.assertEqual(set(catalog_profiles), set(oracle_profiles))
        self.assertEqual(len(catalog_profiles), 18)

        for profile_id in sorted(catalog_profiles):
            with self.subTest(profile_id=profile_id):
                profile = catalog_profiles[profile_id]
                expected = oracle_profiles[profile_id]
                self.assertEqual(set(expected["direct_source_clause_ids"]) & section_clause_ids, {clause_id})
                self.assertTrue(section_clause_ids.issubset(expected["source_clause_ids"]))
                self.assertIn(requirement_id, expected["requirement_ids"])
                self.assertEqual(set(profile["direct_source_clause_ids"]) & section_clause_ids, {clause_id})
                self.assertTrue(section_clause_ids.issubset(profile["source_clause_ids"]))
                self.assertIn(requirement_id, profile["requirement_ids"])

                closure = compute_profile_closure(
                    catalog,
                    profile["direct_source_clause_ids"],
                    profile_id=profile_id,
                )
                self.assertTrue(section_clause_ids.issubset(closure.source_clause_ids))
                self.assertIn(requirement_id, closure.requirement_ids)

    def test_baseline_profile_condition_inventory_covers_the_complete_pinned_section(self) -> None:
        """The inclusion oracle accounts for every §5.1.1 condition without inventing REQ IDs."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profiles_html = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252")
        inventory = oracle["provenance"].get("profile_condition_inventories", [])
        baseline = next(
            (
                item
                for item in inventory
                if item.get("source_id") == "KMIPKIT-SRC-profiles" and item.get("section") == "5.1.1"
            ),
            None,
        )

        self.assertIsNotNone(baseline)
        assert baseline is not None
        self.assertEqual(
            baseline["inclusion_id"],
            "KMIPKIT-INCLUSION-PROF-BASELINE-CONDITIONS",
        )
        self.assertEqual(baseline["source_path"], "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html")
        self.assertEqual(baseline["html_line_range"], [4529, 4636])
        self.assertEqual(baseline["governing_requirement_id"], "KMIPKIT-REQ-PROF-6.1-003")
        self.assertEqual(len(baseline["conditions"]), 8)

        expected_conditions = (
            (1, [4533, 4534], "Supports the conditions required by the KMIP Client Implementation Conformance clauses [KMIP-SPEC]", (), "mandatory_profile_condition"),
            (2, [4535, 4540], "Supports the following Attribute Data Structures [KMIP-SPEC]", ("Attributes",), "mandatory_profile_condition"),
            (
                3,
                [4541, 4562],
                "Supports the following Object Attributes [KMIP-SPEC]",
                (
                    "Activation Date", "Deactivation Date", "Digest", "Initial Date", "Last Change Date",
                    "Object Type", "State", "Unique Identifier",
                ),
                "mandatory_profile_condition",
            ),
            (
                4,
                [4563, 4574],
                "Supports the following Client-to-Server Operations [KMIP-SPEC]",
                ("Get", "Get Attributes", "Locate", "Query"),
                "mandatory_profile_condition",
            ),
            (
                5,
                [4575, 4613],
                "Supports the following Message Data Structures [KMIP-SPEC]",
                (
                    "Asynchronous Indicator", "Batch Count", "Batch Error Continuation Option", "Batch Item",
                    "Batch Order Option", "Maximum Response Size", "Message Extension", "Operation",
                    "Protocol Version", "Result Reason", "Result Status", "Server Correlation Value",
                    "Time Stamp", "Unique Batch Item ID",
                ),
                "mandatory_profile_condition",
            ),
            (
                6,
                [4614, 4627],
                "Supports the Message Protocols [KMIP-SPEC]",
                ("Authentication", "Transport", "TTLV"),
                "mandatory_profile_condition",
            ),
            (7, [4628, 4631], "Optionally supports any clause within [KMIP-SPEC] that is not listed above", (), "optional_profile_capability"),
            (
                8,
                [4632, 4635],
                "Optionally supports extensions outside the scope of this standard (e.g., vendor extensions, conformance clauses) that do not contradict any KMIP requirements",
                (),
                "optional_profile_capability",
            ),
        )
        observed_conditions = tuple(
            (
                item["item"],
                item["html_line_range"],
                item["source_text"],
                tuple(item["supported_items"]),
                item["condition_kind"],
            )
            for item in baseline["conditions"]
        )
        self.assertEqual(observed_conditions, expected_conditions)

        source_lines = profiles_html.splitlines()
        normalized_inventory_span = _normalized_html_source_span(source_lines, baseline["html_line_range"])
        self.assertIn(
            "An implementation is a conforming Baseline Client if it meets the following conditions:",
            normalized_inventory_span,
        )
        research_text = (ROOT / "specs" / "010-profile-conformance" / "research.md").read_text(encoding="utf-8")
        self.assertIn("condition inventory covers all eight top-level conditions", research_text)
        self.assertNotIn("pinned HTML lines 4532–4636", research_text)
        for item, line_range, source_text, supported_items, condition_kind in expected_conditions:
            with self.subTest(item=item):
                condition = baseline["conditions"][item - 1]
                self.assertEqual(condition["html_line_range"], line_range)
                normalized_span = _normalized_html_source_span(source_lines, line_range)
                self.assertIn(source_text, normalized_span)
                for supported_item in supported_items:
                    self.assertIn(supported_item, normalized_span)
                self.assertEqual(condition["condition_id"], f"KMIPKIT-PROF-COND-5.1.1-{item:03d}")
                self.assertEqual(condition["condition_kind"], condition_kind)
                self.assertIsNone(condition["requirement_id"])
                self.assertEqual(condition["scope_disposition"], "profile_conditional")
                expected_mapping_status = "reviewed" if item in (1, 2, 3, 4, 5, 6, 7, 8) else "pending_t002_crosswalk"
                self.assertEqual(condition["mapping_status"], expected_mapping_status)
        self.assertEqual(
            {item["requirement_id"] for item in baseline["conditions"]},
            {None},
            "§5.1.1's title-case support statements are profile conditions, not standalone RFC modal REQs",
        )

    def test_baseline_conformance_condition_is_covered_by_whole_spec_inclusion(self) -> None:
        """Baseline item 1 adds no condition REQ beyond its existing whole-Spec closure."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profiles_html = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252")
        spec_html = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252")
        inventory = next(item for item in oracle["provenance"]["profile_condition_inventories"] if item["section"] == "5.1.1")
        condition = inventory["conditions"][0]

        self.assertEqual(condition["condition_id"], "KMIPKIT-PROF-COND-5.1.1-001")
        self.assertEqual(condition["html_line_range"], [4533, 4534])
        self.assertEqual(condition["mapping_status"], "reviewed")
        self.assertEqual(
            condition["source_review"],
            {
                "disposition": "covered_by_baseline_whole_spec_inclusion",
                "profiles_source": "Profiles v2.1 §5.1.1 item 1",
                "profiles_line_range": [4533, 4534],
                "specification_source": "KMIP Specification v2.1 §14.1",
                "specification_line_ranges": [[66041, 66042], [66044, 66047]],
                "inclusion_id": "KMIPKIT-INCLUSION-PROF-BASELINE-SPEC",
                "covered_requirement_ids": ["KMIPKIT-REQ-SPEC-14.1-001", "KMIPKIT-REQ-SPEC-14.1-002"],
                "additional_source_clause_ids": [],
                "additional_requirement_ids": [],
            },
        )
        self.assertEqual(condition["supported_items"], [])
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")

        baseline_profile = next(
            item for item in oracle["profiles"] if item["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        self.assertTrue(
            {"KMIPKIT-REQ-SPEC-14.1-001", "KMIPKIT-REQ-SPEC-14.1-002"}.issubset(
                baseline_profile["requirement_ids"]
            )
        )
        whole_spec_edge = next(
            item
            for item in oracle["provenance"]["inclusion_edges"]
            if item["inclusion_id"] == "KMIPKIT-INCLUSION-PROF-BASELINE-SPEC"
        )
        self.assertEqual(whole_spec_edge["source_clause_id"], "KMIPKIT-CLAUSE-PROF-6.1-001")
        self.assertEqual(whole_spec_edge["included_source_id"], "KMIPKIT-SRC-spec")
        self.assertIsNone(whole_spec_edge["included_section"])

        profiles_lines = profiles_html.splitlines()
        self.assertIn(
            "Supports the conditions required by the KMIP Client Implementation Conformance clauses [KMIP-SPEC]",
            _normalized_html_source_span(profiles_lines, [4533, 4534]),
        )
        spec_lines = spec_html.splitlines()
        self.assertIn("A KMIP client implementation SHALL be a conforming KMIP Client.", _normalized_html_source_span(spec_lines, [66041, 66042]))
        self.assertIn(
            "the implementation SHALL conform to all normative statements within the clauses specified for that profile and for any subclauses to each of those clauses",
            _normalized_html_source_span(spec_lines, [66044, 66047]),
        )

    def test_baseline_open_ended_optional_clause_is_reviewed_without_inventing_scope(self) -> None:
        """Baseline item 7 is an open-ended optional capability, not a finite closure edge."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        inventories = oracle["provenance"]["profile_condition_inventories"]
        baseline = next(item for item in inventories if item["section"] == "5.1.1")
        condition = baseline["conditions"][6]

        self.assertEqual(condition["condition_id"], "KMIPKIT-PROF-COND-5.1.1-007")
        self.assertEqual(condition["mapping_status"], "reviewed")
        self.assertEqual(
            condition["source_review"],
            {
                "disposition": "open_ended_optional_capability",
                "profiles_source": "Profiles v2.1 §5.1.1 item 7",
                "profiles_line_range": [4628, 4631],
                "closure_basis": "Profiles §6.1 item 1 already includes the KMIP Specification v2.1 source for the Baseline Client closure.",
                "additional_source_clause_ids": [],
                "additional_requirement_ids": [],
            },
        )
        self.assertEqual(condition["supported_items"], [])
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")

    def test_baseline_out_of_standard_extensions_are_not_selected_for_1_0(self) -> None:
        """Optional vendor extensions stay distinct from the standard Message Extension mechanism."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        inventories = oracle["provenance"]["profile_condition_inventories"]
        baseline = next(item for item in inventories if item["section"] == "5.1.1")
        condition = baseline["conditions"][7]
        message_data_condition = baseline["conditions"][4]
        message_extension = next(
            item for item in message_data_condition["supported_item_mappings"] if item["item"] == "Message Extension"
        )

        self.assertEqual(condition["condition_id"], "KMIPKIT-PROF-COND-5.1.1-008")
        self.assertEqual(condition["html_line_range"], [4632, 4635])
        self.assertEqual(
            condition["source_text"],
            "Optionally supports extensions outside the scope of this standard (e.g., vendor extensions, conformance clauses) that do not contradict any KMIP requirements",
        )
        self.assertEqual(condition["mapping_status"], "reviewed")
        self.assertEqual(
            condition["source_review"],
            {
                "disposition": "optional_out_of_standard_extensions_not_selected_for_1_0",
                "profiles_source": "Profiles v2.1 §5.1.1 item 8",
                "profiles_line_range": [4632, 4635],
                "rationale": "No out-of-standard extension or vendor-specific semantics is selected for the 1.0 capability claim. This does not exclude the standard-defined Message Extension structure and client duties in KMIP-SPEC §9.13, which remain crosswalked under Baseline item 5.",
                "additional_source_clause_ids": [],
                "additional_requirement_ids": [],
            },
        )
        self.assertEqual(condition["condition_kind"], "optional_profile_capability")
        self.assertEqual(condition["supported_items"], [])
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(message_data_condition["mapping_status"], "reviewed")
        self.assertEqual(
            message_extension["requirement_ids"],
            [
                "KMIPKIT-REQ-SPEC-8.3-004",
                "KMIPKIT-REQ-SPEC-9.13-001-004",
                "KMIPKIT-REQ-SPEC-9.13-001-005",
            ],
        )
        self.assertTrue(set(message_extension["requirement_ids"]).isdisjoint(condition["source_review"]["additional_requirement_ids"]))

    def test_baseline_attributes_condition_has_exact_reviewed_capability_crosswalk(self) -> None:
        """Pin Profiles v2.1 §5.1.1 item 2 to Specification v2.1 §5.1 Table 157."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        profiles_path = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        spec_path = "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-002"
        )
        expected_mapping = {
            "item": "Attributes",
            "element_id": "KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-1-ATTRIBUTES",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-5.1-002"],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-5.1-002"],
            "mapping_status": "reviewed",
        }
        self.assertEqual(condition["supported_items"], ["Attributes"])
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition.get("supported_item_mappings"), [expected_mapping])

        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        profiles_text = _normalized_html_source_span(profiles_lines, [4535, 4540])
        self.assertIn("Attribute Data Structures", profiles_text)
        self.assertIn("Attributes", profiles_text)

        source_line_ranges = {
            "KMIPKIT-CLAUSE-SPEC-5.1-002": [21800, 21820],
            "KMIPKIT-CLAUSE-SPEC-3.2-001": [7837, 7845],
        }
        expected_requirement_details = {
            "KMIPKIT-REQ-SPEC-5.1-002": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-5.1-002",
                "keyword": "MAY",
                "direction": "both",
                "locator": {"ordinal": 2, "block_kind": "table_row"},
                "source_direction_phrase": "client encodes and decodes Attributes, including repeated attribute values",
            },
        }
        expected_clause_ids = set(expected_mapping["source_clause_ids"])
        source_candidates = {
            item["clause_id"]: item
            for item in audit_document((ROOT / spec_path).read_bytes(), "KMIPKIT-SRC-spec")
            if item["clause_id"] in expected_clause_ids
        }
        catalog_clauses = {item["clause_id"]: item for item in catalog["source_clauses"]}
        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        source_reviews = {
            item["source_clause_id"]: item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] in expected_clause_ids
        }
        self.assertEqual(set(source_candidates), expected_clause_ids)
        self.assertEqual(set(source_reviews), expected_clause_ids)

        spec_lines = (ROOT / spec_path).read_text(encoding="cp1252").splitlines()
        for requirement_id, details in expected_requirement_details.items():
            clause_id = details["clause_id"]
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                clause = catalog_clauses[clause_id]
                candidate = source_candidates[clause_id]
                review = source_reviews[clause_id]
                source_text = _normalized_html_source_span(spec_lines, source_line_ranges[clause_id])

                self.assertEqual(requirement["source_clause_ids"], [clause_id])
                self.assertEqual(requirement["source_keyword"], details["keyword"])
                self.assertEqual(requirement["normative_strength"], "permission_or_optional")
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], details["direction"])
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertIn(expected_mapping["element_id"], requirement["element_ids"])
                self.assertEqual(requirement["test_case_ids"], [])
                self.assertEqual(clause["section"], "5.1" if "5.1-" in clause_id else "3.2")
                self.assertEqual(clause["requirement_ids"], [requirement_id])
                self.assertEqual(clause["locator"], details["locator"])
                self.assertEqual(clause["source_keywords"], [details["keyword"]])
                self.assertEqual(candidate["locator"], details["locator"])
                self.assertEqual(review["source_path"], spec_path)
                self.assertEqual(review["source_locator"], details["locator"])
                self.assertEqual(review["html_line_range"], source_line_ranges[clause_id])
                self.assertIn(details["keyword"], source_text)
                self.assertIn(details["keyword"], review["source_excerpt"])
                self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
                source_obligation = review["source_obligations"][0]
                self.assertEqual(source_obligation["candidate_requirement_id"], requirement_id)
                self.assertEqual(source_obligation["source_keyword"], "MAY")
                self.assertEqual(source_obligation["scope_disposition"], "client_1_0")
                self.assertEqual(source_obligation["actor"], "client")
                self.assertEqual(
                    source_obligation["direction"], details["source_direction_phrase"]
                )

        repeated_attribute_text = _normalized_html_source_span(spec_lines, [21800, 21820])
        self.assertIn("Any attribute in", repeated_attribute_text)
        self.assertIn("MAY be repeated", repeated_attribute_text)
        self.assertIn("No", repeated_attribute_text)
        key_value_text = _normalized_html_source_span(spec_lines, [7837, 7845])
        self.assertIn("OPTIONAL attribute information", key_value_text)
        self.assertIn("differs from the attributes associated with Managed Objects", key_value_text)
        table_13_attributes_text = _normalized_html_source_span(spec_lines, [7919, 7930])
        self.assertIn("Attributes", table_13_attributes_text)
        self.assertIn("Structure", table_13_attributes_text)

        repeated_item_ids = {
            "KMIPKIT-REQ-SPEC-4-001-001",
            "KMIPKIT-REQ-SPEC-4-001-002",
            "KMIPKIT-REQ-SPEC-4-001-003",
        }
        self.assertTrue(repeated_item_ids.isdisjoint(expected_mapping["requirement_ids"]))
        self.assertNotIn("KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE", requirements["KMIPKIT-REQ-SPEC-5.1-002"]["element_ids"])
        self.assertIn("KMIPKIT-ELEM-OBJECT-STRUCTURE-KEY-VALUE", requirements["KMIPKIT-REQ-SPEC-3.2-001"]["element_ids"])
        self.assertIn("KMIPKIT-ELEM-OP-C2S-REGISTER", requirements["KMIPKIT-REQ-SPEC-3.2-001"]["element_ids"])

        repeated_attributes_element = expected_mapping["element_id"]
        test_case = next(item for item in catalog["test_cases"] if item["official_case_id"] == "BL-M-4-21")
        attributes_element = next(
            item for item in catalog["elements"] if item["element_id"] == repeated_attributes_element
        )
        self.assertNotIn(repeated_attributes_element, test_case["element_ids"])
        self.assertNotIn(test_case["test_id"], attributes_element["test_case_ids"])
        self.assertTrue(
            repeated_item_ids.union(expected_mapping["requirement_ids"]).isdisjoint(test_case["requirement_ids"])
        )
        fixture_html_text = _normalized_html_source_span(profiles_lines, [5018, 5021])
        self.assertIn("BL-M-4-21", fixture_html_text)
        self.assertIn("test-cases/kmip-v2.1/mandatory/BL-M-4-21.xml", fixture_html_text)
        self.assertNotIn("Attributes", fixture_html_text)
        fixture_root = ElementTree.parse(ROOT / "specification/oasis/kmip-2.1/fixtures/BL-M-4-21.xml").getroot()
        locate_attributes = [
            attributes
            for batch_item in fixture_root.iter("BatchItem")
            if batch_item.find("./Operation").get("value") == "Locate"
            for attributes in batch_item.findall("./RequestPayload/Attributes")
        ]
        self.assertEqual(len(locate_attributes), 1)
        repeated_protect_stop_dates = locate_attributes[0].findall("./ProtectStopDate")
        self.assertEqual(len(repeated_protect_stop_dates), 2)
        self.assertEqual(requirements["KMIPKIT-REQ-SPEC-5.1-002"]["test_case_ids"], [])
        self.assertEqual(requirements["KMIPKIT-REQ-SPEC-3.2-001"]["test_case_ids"], [])
        self.assertEqual(condition["mapping_status"], "reviewed")

    def test_baseline_object_attributes_condition_has_exact_element_crosswalk(self) -> None:
        """Baseline item labels map to elements without assigning unresolved §4 requirements."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        profiles_path = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-003"
        )
        expected_items = [
            ("Activation Date", "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "4.1", "Date-Time", "420001"),
            ("Deactivation Date", "KMIPKIT-ELEM-ATTRIBUTE-DEACTIVATION-DATE", "4.18", "Date-Time", "42002F"),
            ("Digest", "KMIPKIT-ELEM-ATTRIBUTE-DIGEST", "4.21", "Structure", "420034"),
            ("Initial Date", "KMIPKIT-ELEM-ATTRIBUTE-INITIAL-DATE", "4.25", "Date-Time", "420039"),
            ("Last Change Date", "KMIPKIT-ELEM-ATTRIBUTE-LAST-CHANGE-DATE", "4.29", "Date-Time", "420048"),
            ("Object Type", "KMIPKIT-ELEM-ATTRIBUTE-OBJECT-TYPE", "4.36", "Enumeration", "420057"),
            ("State", "KMIPKIT-ELEM-ATTRIBUTE-STATE", "4.57", "Enumeration", "42008D"),
            ("Unique Identifier", "KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER", "4.58", "Text String, Enumeration or Integer", "420094"),
        ]
        expected_mappings = [
            {
                "item": item,
                "element_id": element_id,
                "requirement_ids": [],
                "source_clause_ids": [],
                "mapping_status": "reviewed",
            }
            for item, element_id, _, _, _ in expected_items
        ]

        self.assertEqual(
            condition["supported_items"],
            [item for item, _, _, _, _ in expected_items],
        )
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["html_line_range"], [4541, 4562])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")
        self.assertEqual(condition.get("supported_item_mappings"), expected_mappings)

        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        profiles_text = _normalized_html_source_span(profiles_lines, [4541, 4562])
        self.assertIn("Supports the following Object Attributes", profiles_text)
        elements = {item["element_id"]: item for item in catalog["elements"]}

        for item, element_id, section, encoding, wire_value in expected_items:
            with self.subTest(item=item):
                self.assertIn(item, profiles_text)
                element = elements[element_id]
                tag_id = f"KMIPKIT-ELEM-TAG-{wire_value}"
                self.assertEqual(element["name"], item)
                self.assertEqual(element["kind"], "attribute")
                self.assertEqual(
                    element["source_refs"],
                    [
                        {"source_id": "KMIPKIT-SRC-spec", "section": section},
                        {"source_id": "KMIPKIT-SRC-spec", "section": "11.56"},
                    ],
                )
                self.assertEqual(element["direction"], "both")
                self.assertEqual(element["scope_state"], "client_1_0")
                self.assertEqual(element["source_encoding"], encoding)
                self.assertEqual(element["parent_element_ids"], [tag_id])

                tag = elements[tag_id]
                self.assertEqual(tag["kind"], "tag")
                self.assertEqual(tag["name"], item)
                self.assertEqual(tag["wire_value"], wire_value)
                self.assertEqual(tag["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}])
                self.assertEqual(tag["parent_element_ids"], ["KMIPKIT-ELEM-ENUMERATION-TAG"])

    def test_baseline_query_operation_condition_has_exact_reviewed_client_crosswalk(self) -> None:
        """The four item mappings are source-backed; the parent is reviewed only as a crosswalk."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        profiles_path = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        spec_path = "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-004"
        )
        expected_mapping = {
            "item": "Query",
            "element_id": "KMIPKIT-ELEM-OP-C2S-QUERY",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-6.1.40-014", "KMIPKIT-REQ-SPEC-6.1.40-016"],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-6.1.40-014", "KMIPKIT-CLAUSE-SPEC-6.1.40-016"],
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"], ["Get", "Get Attributes", "Locate", "Query"])
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["mapping_status"], "reviewed")
        self.assertEqual(
            [mapping for mapping in condition.get("supported_item_mappings", []) if mapping["item"] == "Query"],
            [expected_mapping],
        )

        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        profiles_text = _normalized_html_source_span(profiles_lines, [4563, 4574])
        self.assertIn("Client-to-Server Operations", profiles_text)
        self.assertIn("Query", profiles_text)

        source_candidates = {
            item["clause_id"]: item
            for item in audit_document((ROOT / spec_path).read_bytes(), "KMIPKIT-SRC-spec")
            if item["clause_id"] in expected_mapping["source_clause_ids"]
        }
        catalog_clauses = {item["clause_id"]: item for item in catalog["source_clauses"]}
        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        query_element = next(
            item for item in catalog["elements"] if item["element_id"] == expected_mapping["element_id"]
        )
        source_reviews = {
            item["source_clause_id"]: item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] in expected_mapping["source_clause_ids"]
        }
        source_line_ranges = {
            "KMIPKIT-CLAUSE-SPEC-6.1.40-014": [31992, 31993],
            "KMIPKIT-CLAUSE-SPEC-6.1.40-016": [32025, 32042],
        }
        expected_keywords = {
            "KMIPKIT-CLAUSE-SPEC-6.1.40-014": "SHALL",
            "KMIPKIT-CLAUSE-SPEC-6.1.40-016": "MAY",
        }
        expected_strengths = {
            "KMIPKIT-REQ-SPEC-6.1.40-014": "mandatory",
            "KMIPKIT-REQ-SPEC-6.1.40-016": "permission_or_optional",
        }
        self.assertEqual(set(source_candidates), set(expected_mapping["source_clause_ids"]))
        self.assertEqual(set(source_reviews), set(expected_mapping["source_clause_ids"]))
        self.assertEqual(query_element["name"], "Query")
        self.assertEqual(query_element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "6.1.40"}])
        self.assertEqual(query_element["direction"], "client_to_server")
        self.assertEqual(query_element["requirement_ids"], expected_mapping["requirement_ids"])

        spec_lines = (ROOT / spec_path).read_text(encoding="cp1252").splitlines()
        for requirement_id, clause_id in zip(
            expected_mapping["requirement_ids"], expected_mapping["source_clause_ids"], strict=True
        ):
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                clause = catalog_clauses[clause_id]
                candidate = source_candidates[clause_id]
                review = source_reviews[clause_id]
                keyword = expected_keywords[clause_id]
                source_text = _normalized_html_source_span(spec_lines, source_line_ranges[clause_id])

                self.assertEqual(requirement["source_clause_ids"], [clause_id])
                self.assertEqual(requirement["source_keyword"], keyword)
                self.assertEqual(requirement["normative_strength"], expected_strengths[requirement_id])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertIn(expected_mapping["element_id"], requirement["element_ids"])
                self.assertEqual(clause["source_id"], "KMIPKIT-SRC-spec")
                self.assertEqual(clause["section"], "6.1.40")
                self.assertEqual(clause["requirement_ids"], [requirement_id])
                self.assertEqual(clause["role"], "client")
                self.assertEqual(clause["direction"], "client_to_server")
                self.assertEqual(clause["locator"], candidate["locator"])
                self.assertEqual(clause["source_keywords"], candidate["source_keywords"])
                self.assertEqual(candidate["source_keywords"], [keyword])
                self.assertEqual(review["source_path"], spec_path)
                self.assertEqual(review["source_locator"], clause["locator"])
                self.assertEqual(review["html_line_range"], source_line_ranges[clause_id])
                self.assertEqual(review["source_excerpt"], source_text)
                self.assertIn("Query Function", source_text)
                self.assertIn(keyword, source_text)
                self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
                self.assertEqual(review["source_obligations"][0]["candidate_requirement_id"], requirement_id)
                self.assertEqual(review["source_obligations"][0]["actor"], "client")

    def test_baseline_get_attributes_locate_conditions_have_exact_reviewed_client_crosswalks(self) -> None:
        """Baseline item labels map only to their reviewed client request clauses."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        profiles_path = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        spec_path = "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-004"
        )
        expected_mappings = [
            {
                "item": "Get",
                "element_id": "KMIPKIT-ELEM-OP-C2S-GET",
                "requirement_ids": ["KMIPKIT-REQ-SPEC-6.1.19-002"],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-6.1.19-002"],
                "mapping_status": "reviewed",
            },
            {
                "item": "Get Attributes",
                "element_id": "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES",
                "requirement_ids": [
                    "KMIPKIT-REQ-SPEC-6.1.20-001-001",
                    "KMIPKIT-REQ-SPEC-6.1.20-004",
                ],
                "source_clause_ids": [
                    "KMIPKIT-CLAUSE-SPEC-6.1.20-001",
                    "KMIPKIT-CLAUSE-SPEC-6.1.20-004",
                ],
                "mapping_status": "reviewed",
            },
            {
                "item": "Locate",
                "element_id": "KMIPKIT-ELEM-OP-C2S-LOCATE",
                "requirement_ids": [
                    "KMIPKIT-REQ-SPEC-6.1.28-001",
                    "KMIPKIT-REQ-SPEC-6.1.28-002",
                    "KMIPKIT-REQ-SPEC-6.1.28-004-001",
                    "KMIPKIT-REQ-SPEC-6.1.28-007",
                    "KMIPKIT-REQ-SPEC-6.1.28-013-001",
                    "KMIPKIT-REQ-SPEC-6.1.28-013-002",
                ],
                "source_clause_ids": [
                    "KMIPKIT-CLAUSE-SPEC-6.1.28-001",
                    "KMIPKIT-CLAUSE-SPEC-6.1.28-002",
                    "KMIPKIT-CLAUSE-SPEC-6.1.28-004",
                    "KMIPKIT-CLAUSE-SPEC-6.1.28-007",
                    "KMIPKIT-CLAUSE-SPEC-6.1.28-013",
                ],
                "mapping_status": "reviewed",
            },
            {
                "item": "Query",
                "element_id": "KMIPKIT-ELEM-OP-C2S-QUERY",
                "requirement_ids": ["KMIPKIT-REQ-SPEC-6.1.40-014", "KMIPKIT-REQ-SPEC-6.1.40-016"],
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-6.1.40-014", "KMIPKIT-CLAUSE-SPEC-6.1.40-016"],
                "mapping_status": "reviewed",
            },
        ]
        operation_mappings = expected_mappings[:3]
        mapped_requirement_ids = {
            requirement_id
            for mapping in expected_mappings
            for requirement_id in mapping["requirement_ids"]
        }
        operation_requirement_ids = {
            requirement_id
            for mapping in operation_mappings
            for requirement_id in mapping["requirement_ids"]
        }
        expected_requirements = {
            "KMIPKIT-REQ-SPEC-6.1.19-002": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.19-002",
                "line_range": [27135, 27136],
                "locator": {"ordinal": 2, "block_kind": "paragraph"},
                "keyword": "SHALL",
                "strength": "mandatory",
                "source_phrase": "key format capabilities SHALL be assumed by the client",
            },
            "KMIPKIT-REQ-SPEC-6.1.20-001-001": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.20-001",
                "line_range": [27440, 27447],
                "locator": {"ordinal": 1, "block_kind": "paragraph"},
                "keyword": "SHALL NOT",
                "strength": "prohibited",
                "source_phrase": "same Attribute Reference SHALL NOT be present more than once",
            },
            "KMIPKIT-REQ-SPEC-6.1.20-004": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.20-004",
                "line_range": [27506, 27515],
                "locator": {"ordinal": 4, "block_kind": "table_row"},
                "keyword": "MAY",
                "strength": "permission_or_optional",
                "source_phrase": "MAY be repeated",
            },
            "KMIPKIT-REQ-SPEC-6.1.28-001": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-001",
                "line_range": [29143, 29148],
                "locator": {"ordinal": 1, "block_kind": "paragraph"},
                "keyword": "MAY",
                "strength": "permission_or_optional",
                "source_phrase": "request MAY contain a Maximum Items field",
            },
            "KMIPKIT-REQ-SPEC-6.1.28-002": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-002",
                "line_range": [29150, 29155],
                "locator": {"ordinal": 2, "block_kind": "paragraph"},
                "keyword": "MAY",
                "strength": "permission_or_optional",
                "source_phrase": "request MAY contain an Offset Items field",
            },
            "KMIPKIT-REQ-SPEC-6.1.28-004-001": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-004",
                "line_range": [29168, 29170],
                "locator": {"ordinal": 4, "block_kind": "paragraph"},
                "keyword": "REQUIRED",
                "strength": "mandatory",
                "source_phrase": "Recover and Get operations are REQUIRED to be",
            },
            "KMIPKIT-REQ-SPEC-6.1.28-007": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-007",
                "line_range": [29206, 29210],
                "locator": {"ordinal": 7, "block_kind": "paragraph"},
                "keyword": "REQUIRED",
                "strength": "permission_or_optional",
                "source_phrase": "all of the structure fields are not REQUIRED to be specified",
            },
            "KMIPKIT-REQ-SPEC-6.1.28-013-001": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-013",
                "line_range": [29341, 29362],
                "locator": {"ordinal": 13, "block_kind": "table_row"},
                "keyword": "REQUIRED",
                "strength": "mandatory",
                "source_phrase": "REQUIRED to match those in a candidate object",
            },
            "KMIPKIT-REQ-SPEC-6.1.28-013-002": {
                "clause_id": "KMIPKIT-CLAUSE-SPEC-6.1.28-013",
                "line_range": [29341, 29362],
                "locator": {"ordinal": 13, "block_kind": "table_row"},
                "keyword": "MAY",
                "strength": "permission_or_optional",
                "source_phrase": "Attributes structure MAY be empty",
            },
        }
        expected_requirement_conditions = {
            "KMIPKIT-REQ-SPEC-6.1.19-002": "When requesting the server to return an object in a particular format.",
            "KMIPKIT-REQ-SPEC-6.1.20-001-001": None,
            "KMIPKIT-REQ-SPEC-6.1.20-004": None,
            "KMIPKIT-REQ-SPEC-6.1.28-001": None,
            "KMIPKIT-REQ-SPEC-6.1.28-002": None,
            "KMIPKIT-REQ-SPEC-6.1.28-004-001": "When the client needs to obtain archived matches.",
            "KMIPKIT-REQ-SPEC-6.1.28-007": None,
            "KMIPKIT-REQ-SPEC-6.1.28-013-001": None,
            "KMIPKIT-REQ-SPEC-6.1.28-013-002": "When requesting an unrestricted match.",
        }
        expected_source_obligation_conditions = {
            "KMIPKIT-REQ-SPEC-6.1.19-002": (
                "When the client requests the server to return an object in a particular format."
            ),
            "KMIPKIT-REQ-SPEC-6.1.20-001-001": None,
            "KMIPKIT-REQ-SPEC-6.1.20-004": None,
            "KMIPKIT-REQ-SPEC-6.1.28-001": "Maximum Items is included in the Locate request.",
            "KMIPKIT-REQ-SPEC-6.1.28-002": "Offset Items may be specified; zero is equivalent to omission.",
            "KMIPKIT-REQ-SPEC-6.1.28-004-001": "The Located objects are archived.",
            "KMIPKIT-REQ-SPEC-6.1.28-007": (
                "A structured attribute is specified in the request; not all fields need be supplied."
            ),
            "KMIPKIT-REQ-SPEC-6.1.28-013-001": (
                "Attributes are supplied; their values are required to match a candidate object."
            ),
            "KMIPKIT-REQ-SPEC-6.1.28-013-002": (
                "Client chooses an empty Attributes structure; the lower-case 'should' is not an additional formal keyword."
            ),
        }

        self.assertEqual(condition["supported_items"], ["Get", "Get Attributes", "Locate", "Query"])
        self.assertEqual(condition.get("supported_item_mappings"), expected_mappings)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        profiles_text = _normalized_html_source_span(profiles_lines, [4563, 4574])
        spec_lines = (ROOT / spec_path).read_text(encoding="cp1252").splitlines()
        elements = {item["element_id"]: item for item in catalog["elements"]}
        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        clauses = {item["clause_id"]: item for item in catalog["source_clauses"]}
        source_reviews = {
            item["source_clause_id"]: item
            for item in oracle["source_requirement_reviews"]
            if item.get("source_clause_id") in {details["clause_id"] for details in expected_requirements.values()}
        }
        expected_client_ids_by_clause = {
            clause_id: [
                requirement_id
                for requirement_id, details in expected_requirements.items()
                if details["clause_id"] == clause_id
            ]
            for clause_id in {details["clause_id"] for details in expected_requirements.values()}
        }
        expected_element_sections = {
            "Get": "6.1.19",
            "Get Attributes": "6.1.20",
            "Locate": "6.1.28",
        }

        for mapping in operation_mappings:
            with self.subTest(item=mapping["item"]):
                self.assertIn(mapping["item"], profiles_text)
                element = elements[mapping["element_id"]]
                self.assertEqual(element["name"], mapping["item"])
                self.assertEqual(
                    element["source_refs"],
                    [{"source_id": "KMIPKIT-SRC-spec", "section": expected_element_sections[mapping["item"]]}],
                )
                self.assertEqual(element["direction"], "client_to_server")
                self.assertEqual(element["scope_state"], "client_1_0")
                self.assertTrue(set(mapping["requirement_ids"]).issubset(element["requirement_ids"]))
                self.assertEqual(
                    mapping["source_clause_ids"],
                    list(
                        dict.fromkeys(
                            expected_requirements[requirement_id]["clause_id"]
                            for requirement_id in mapping["requirement_ids"]
                        )
                    ),
                )

        for requirement_id, details in expected_requirements.items():
            if requirement_id not in operation_requirement_ids:
                continue
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                clause = clauses[details["clause_id"]]
                review = source_reviews[details["clause_id"]]
                line_text = _normalized_html_source_span(spec_lines, details["line_range"])
                obligation = next(
                    item for item in review["source_obligations"]
                    if item.get("candidate_requirement_id") == requirement_id
                )

                self.assertEqual(requirement["source_clause_ids"], [details["clause_id"]])
                self.assertEqual(requirement["source_keyword"], details["keyword"])
                self.assertEqual(requirement["normative_strength"], details["strength"])
                self.assertEqual(requirement["condition"], expected_requirement_conditions[requirement_id])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(clause["source_id"], "KMIPKIT-SRC-spec")
                self.assertEqual(clause["section"], details["clause_id"].removeprefix("KMIPKIT-CLAUSE-SPEC-").rsplit("-", 1)[0])
                self.assertEqual(clause["locator"], details["locator"])
                self.assertIn(details["keyword"], clause["source_keywords"])
                self.assertEqual(review["source_path"], spec_path)
                self.assertEqual(review["source_locator"], details["locator"])
                self.assertEqual(review["expected_client_requirement_ids"], expected_client_ids_by_clause[details["clause_id"]])
                self.assertEqual(review["source_excerpt"], _normalized_html_source_span(spec_lines, review["html_line_range"]))
                self.assertIn(details["source_phrase"], line_text)
                self.assertEqual(obligation["source_keyword"], details["keyword"])
                self.assertEqual(obligation["condition"], expected_source_obligation_conditions[requirement_id])
                self.assertEqual(obligation["actor"], "client")
                self.assertTrue(obligation["direction"].startswith("client"))
                self.assertEqual(obligation["scope_disposition"], "client_1_0")
                self.assertEqual(obligation["candidate_requirement_id"], requirement_id)

        excluded_server_requirements = {
            "KMIPKIT-REQ-SPEC-6.1.20-001-002": "KMIPKIT-CLAUSE-SPEC-6.1.20-001",
            "KMIPKIT-REQ-SPEC-6.1.28-004-002": "KMIPKIT-CLAUSE-SPEC-6.1.28-004",
            "KMIPKIT-REQ-SPEC-6.1.28-008-001": "KMIPKIT-CLAUSE-SPEC-6.1.28-008",
            "KMIPKIT-REQ-SPEC-6.1.28-008-002": "KMIPKIT-CLAUSE-SPEC-6.1.28-008",
            "KMIPKIT-REQ-SPEC-6.1.28-009-001": "KMIPKIT-CLAUSE-SPEC-6.1.28-009",
            "KMIPKIT-REQ-SPEC-6.1.28-009-002": "KMIPKIT-CLAUSE-SPEC-6.1.28-009",
            "KMIPKIT-REQ-SPEC-6.1.28-011": "KMIPKIT-CLAUSE-SPEC-6.1.28-011",
            "KMIPKIT-REQ-SPEC-6.1.28-012": "KMIPKIT-CLAUSE-SPEC-6.1.28-012",
        }
        for requirement_id, clause_id in excluded_server_requirements.items():
            with self.subTest(excluded_requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                review = next(
                    item for item in oracle["source_requirement_reviews"]
                    if item.get("source_clause_id") == clause_id
                )
                self.assertNotIn(requirement_id, mapped_requirement_ids)
                self.assertIn(requirement["scope_state"], {"server_only", "out_of_scope"})
                self.assertNotIn(requirement_id, review["expected_client_requirement_ids"])

        locate_007 = requirements["KMIPKIT-REQ-SPEC-6.1.28-007"]
        self.assertEqual(locate_007["source_keyword"], "REQUIRED")
        self.assertEqual(locate_007["normative_strength"], "permission_or_optional")
        self.assertIn("not REQUIRED to be specified", locate_007["review_note"])
        self.assertIn("matching behavior belongs to the server", locate_007["review_note"])
        locate_007_source = _normalized_html_source_span(spec_lines, [29206, 29210])
        self.assertIn("Linked Object Identifier value is specified without the Link Type value", locate_007_source)

    def test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk(self) -> None:
        """The fourteen reviewed Message Data Structure mappings stay structurally bounded."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        profiles_path = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        spec_path = "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_async_requirement_ids = ["KMIPKIT-REQ-SPEC-8-003-001"]
        expected_async_clause_ids = ["KMIPKIT-CLAUSE-SPEC-8-003", "KMIPKIT-CLAUSE-SPEC-9.2-001"]
        expected_async_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-2-ASYNCHRONOUS-INDICATOR"
        expected_async_mapping = {
            "item": "Asynchronous Indicator",
            "element_id": expected_async_element_id,
            "requirement_ids": expected_async_requirement_ids,
            "source_clause_ids": expected_async_clause_ids,
            "mapping_status": "reviewed",
        }
        expected_requirement_ids = [
            "KMIPKIT-REQ-SPEC-9.5-001-001",
            "KMIPKIT-REQ-SPEC-9.5-001-002",
        ]
        expected_clause_id = "KMIPKIT-CLAUSE-SPEC-9.5-001"
        expected_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT"
        expected_mapping = {
            "item": "Batch Count",
            "element_id": expected_element_id,
            "requirement_ids": expected_requirement_ids,
            "source_clause_ids": [expected_clause_id],
            "mapping_status": "reviewed",
        }
        expected_error_requirement_ids = [
            "KMIPKIT-REQ-SPEC-6.1.7-002-002",
            "KMIPKIT-REQ-SPEC-9.6-001-001",
            "KMIPKIT-REQ-SPEC-9.6-001-002",
            "KMIPKIT-REQ-SPEC-9.6-001-003",
        ]
        expected_error_item_requirement_ids = [
            "KMIPKIT-REQ-SPEC-9.6-001-001",
            "KMIPKIT-REQ-SPEC-9.6-001-002",
            "KMIPKIT-REQ-SPEC-9.6-001-003",
        ]
        expected_error_clause_id = "KMIPKIT-CLAUSE-SPEC-9.6-001"
        expected_error_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-6-BATCH-ERROR-CONTINUATION-OPTION"
        expected_error_mapping = {
            "item": "Batch Error Continuation Option",
            "element_id": expected_error_element_id,
            "requirement_ids": expected_error_item_requirement_ids,
            "source_clause_ids": [expected_error_clause_id],
            "mapping_status": "reviewed",
        }
        expected_batch_item_clause_id = "KMIPKIT-CLAUSE-SPEC-9.7-001"
        expected_batch_item_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-7-BATCH-ITEM"
        expected_batch_item_mapping = {
            "item": "Batch Item",
            "element_id": expected_batch_item_element_id,
            "requirement_ids": [],
            "source_clause_ids": [expected_batch_item_clause_id],
            "mapping_status": "reviewed",
        }
        expected_batch_order_option_clause_id = "KMIPKIT-CLAUSE-SPEC-9.8-001"
        expected_batch_order_option_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION"
        expected_batch_order_option_mapping = {
            "item": "Batch Order Option",
            "element_id": expected_batch_order_option_element_id,
            "requirement_ids": [],
            "source_clause_ids": [expected_batch_order_option_clause_id],
            "mapping_status": "reviewed",
        }
        expected_maximum_response_size_clause_id = "KMIPKIT-CLAUSE-SPEC-9.12-001"
        expected_maximum_response_size_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE"
        expected_maximum_response_size_requirement_ids = [
            "KMIPKIT-REQ-SPEC-9.12-001-001",
            "KMIPKIT-REQ-SPEC-9.12-001-002",
            "KMIPKIT-REQ-SPEC-9.12-001-003",
        ]
        expected_maximum_response_size_mapping = {
            "item": "Maximum Response Size",
            "element_id": expected_maximum_response_size_element_id,
            "requirement_ids": expected_maximum_response_size_requirement_ids,
            "source_clause_ids": [expected_maximum_response_size_clause_id],
            "mapping_status": "reviewed",
        }
        expected_message_extension_mapping = {
            "item": "Message Extension",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION",
            "requirement_ids": [
                "KMIPKIT-REQ-SPEC-8.3-004",
                "KMIPKIT-REQ-SPEC-9.13-001-004",
                "KMIPKIT-REQ-SPEC-9.13-001-005",
            ],
            "source_clause_ids": [
                "KMIPKIT-CLAUSE-SPEC-8.3-004",
                "KMIPKIT-CLAUSE-SPEC-8.6-004",
                "KMIPKIT-CLAUSE-SPEC-9.13-001",
            ],
            "mapping_status": "reviewed",
        }
        expected_operation_mapping = {
            "item": "Operation",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION",
            "requirement_ids": [],
            "source_clause_ids": [],
            "mapping_status": "reviewed",
        }
        expected_protocol_version_mapping = {
            "item": "Protocol Version",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION",
            "requirement_ids": [],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.16-001"],
            "mapping_status": "reviewed",
        }
        expected_result_reason_mapping = {
            "item": "Result Reason",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-18-RESULT-REASON",
            "requirement_ids": [],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.18-001"],
            "mapping_status": "reviewed",
        }
        expected_result_status_mapping = {
            "item": "Result Status",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-19-RESULT-STATUS",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-9.19-002"],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.19-001", "KMIPKIT-CLAUSE-SPEC-9.19-002"],
            "mapping_status": "reviewed",
        }
        expected_server_correlation_value_mapping = {
            "item": "Server Correlation Value",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE",
            "requirement_ids": [],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.10-001"],
            "mapping_status": "reviewed",
        }
        expected_time_stamp_mapping = {
            "item": "Time Stamp",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-9.20-001-001", "KMIPKIT-REQ-SPEC-9.20-001-002"],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.20-001"],
            "mapping_status": "reviewed",
        }
        expected_unique_batch_item_id_mapping = {
            "item": "Unique Batch Item ID",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID",
            "requirement_ids": [
                "KMIPKIT-REQ-SPEC-8.3-003",
                "KMIPKIT-REQ-SPEC-9.21-001-001",
                "KMIPKIT-REQ-PROF-4.1.2-007",
            ],
            "source_clause_ids": [
                "KMIPKIT-CLAUSE-SPEC-8.3-003",
                "KMIPKIT-CLAUSE-SPEC-9.21-001",
                "KMIPKIT-CLAUSE-PROF-4.1.2-007",
            ],
            "mapping_status": "reviewed",
        }
        expected_mappings = [
            expected_async_mapping,
            expected_mapping,
            expected_error_mapping,
            expected_batch_item_mapping,
            expected_batch_order_option_mapping,
            expected_maximum_response_size_mapping,
            expected_message_extension_mapping,
            expected_operation_mapping,
            expected_protocol_version_mapping,
            expected_result_reason_mapping,
            expected_result_status_mapping,
            expected_server_correlation_value_mapping,
            expected_time_stamp_mapping,
            expected_unique_batch_item_id_mapping,
        ]

        self.assertEqual(condition["item"], 5)
        self.assertEqual(condition["html_line_range"], [4575, 4613])
        self.assertEqual(condition["condition_kind"], "mandatory_profile_condition")
        self.assertEqual(condition["source_text"], "Supports the following Message Data Structures [KMIP-SPEC]")
        self.assertEqual(
            condition["supported_items"],
            [
                "Asynchronous Indicator",
                "Batch Count",
                "Batch Error Continuation Option",
                "Batch Item",
                "Batch Order Option",
                "Maximum Response Size",
                "Message Extension",
                "Operation",
                "Protocol Version",
                "Result Reason",
                "Result Status",
                "Server Correlation Value",
                "Time Stamp",
                "Unique Batch Item ID",
            ],
        )
        self.assertEqual(condition.get("supported_item_mappings"), expected_mappings)
        self.assertEqual(
            [mapping["item"] for mapping in condition["supported_item_mappings"]],
            [
                "Asynchronous Indicator",
                "Batch Count",
                "Batch Error Continuation Option",
                "Batch Item",
                "Batch Order Option",
                "Maximum Response Size",
                "Message Extension",
                "Operation",
                "Protocol Version",
                "Result Reason",
                "Result Status",
                "Server Correlation Value",
                "Time Stamp",
                "Unique Batch Item ID",
            ],
        )
        self.assertEqual(len(condition["supported_items"]), 14)
        self.assertEqual(len(condition["supported_item_mappings"]), 14)
        unmapped_items = [
            item
            for item in condition["supported_items"]
            if item not in {mapping["item"] for mapping in condition["supported_item_mappings"]}
        ]
        self.assertEqual(
            unmapped_items,
            [],
        )
        self.assertEqual(
            set(condition["supported_items"]) - {mapping["item"] for mapping in condition["supported_item_mappings"]},
            set(),
        )
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        async_element = next(
            item for item in catalog["elements"] if item["element_id"] == expected_async_element_id
        )
        self.assertEqual(async_element["kind"], "message_field")
        self.assertEqual(async_element["name"], "Asynchronous Indicator")
        self.assertEqual(async_element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.2"}])
        self.assertEqual(async_element["direction"], "client_to_server")
        self.assertEqual(async_element["scope_state"], "client_1_0")
        self.assertEqual(
            async_element["parent_element_ids"], ["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-ASYNCHRONOUS-INDICATOR"]
        )
        self.assertEqual(async_element["source_encoding"], "Enumeration")
        self.assertEqual(async_element["requirement_ids"], ["KMIPKIT-REQ-SPEC-9.2-001"])
        self.assertEqual(async_element["profile_ids"], [])

        async_requirement = next(
            item for item in catalog["requirements"] if item["requirement_id"] == expected_async_requirement_ids[0]
        )
        self.assertEqual(async_requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-8-003"])
        self.assertEqual(async_requirement["source_keyword"], "MAY")
        self.assertEqual(async_requirement["normative_strength"], "permission_or_optional")
        self.assertEqual(async_requirement["role"], "client")
        self.assertEqual(async_requirement["direction"], "client_to_server")
        self.assertEqual(async_requirement["condition"], "Only if the client can accept asynchronous responses.")
        self.assertEqual(async_requirement["scope_state"], "client_1_0")
        self.assertEqual(async_requirement["element_ids"], [])

        spec_lines = (ROOT / spec_path).read_text(encoding="cp1252").splitlines()
        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        self.assertIn("Asynchronous Indicator", _normalized_html_source_span(profiles_lines, [4579, 4580]))
        async_clause_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-8-003"
        )
        async_clause_excerpt = (
            "If the client is capable of accepting asynchronous responses, then it MAY set the Asynchronous "
            "Indicator in the header of a batched request. The batched responses MAY contain a mixture of "
            "synchronous and asynchronous responses only if the Asynchronous Indicator is present in the header."
        )
        self.assertEqual(async_clause_review["source_path"], spec_path)
        self.assertEqual(async_clause_review["source_section"], "8")
        self.assertEqual(async_clause_review["html_line_range"], [42443, 42447])
        self.assertEqual(async_clause_review["source_excerpt"], async_clause_excerpt)
        self.assertEqual(_normalized_html_source_span(spec_lines, [42443, 42447]), async_clause_excerpt)
        self.assertEqual(async_clause_review["expected_client_requirement_ids"], expected_async_requirement_ids)
        self.assertEqual(async_clause_review["applies_to"], "all_client_profiles")
        async_client_obligation, mixed_response_obligation = async_clause_review["source_obligations"]
        self.assertEqual(async_client_obligation["source_keyword"], "MAY")
        self.assertEqual(async_client_obligation["actor"], "client")
        self.assertEqual(async_client_obligation["direction"], "client_to_server")
        self.assertEqual(async_client_obligation["condition"], "The client can accept asynchronous responses.")
        self.assertEqual(async_client_obligation["scope_disposition"], "client_1_0")
        self.assertEqual(async_client_obligation["candidate_requirement_id"], expected_async_requirement_ids[0])
        self.assertEqual(mixed_response_obligation["source_keyword"], "MAY")
        self.assertEqual(mixed_response_obligation["actor"], "response producer (endpoint not named)")
        self.assertEqual(mixed_response_obligation["direction"], "server_to_client")
        self.assertEqual(mixed_response_obligation["scope_disposition"], "unresolved")
        self.assertEqual(mixed_response_obligation["candidate_requirement_id"], "KMIPKIT-REQ-SPEC-8-003-002")
        self.assertNotIn("KMIPKIT-REQ-SPEC-8-003-002", expected_async_mapping["requirement_ids"])

        async_field_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.2-001"
        )
        self.assertEqual(async_field_review["html_line_range"], [43567, 43569])
        async_field_source = _normalized_html_source_span(spec_lines, [43563, 43569])
        self.assertIn("If not present in a request, then Prohibited is assumed.", async_field_source)
        self.assertIn("the server SHALL process the request synchronously", async_field_source)
        self.assertEqual(async_field_review["expected_client_requirement_ids"], [])
        self.assertEqual(async_field_review["source_obligations"][0]["actor"], "server")
        self.assertEqual(async_field_review["source_obligations"][0]["scope_disposition"], "server_only")
        self.assertNotIn("KMIPKIT-REQ-SPEC-9.2-001", expected_async_mapping["requirement_ids"])
        self.assertIn(
            "Asynchronous Indicator 420007",
            _normalized_html_source_span(spec_lines, [59017, 59023]),
        )

        batch_item_element = next(
            item for item in catalog["elements"] if item["element_id"] == expected_batch_item_element_id
        )
        self.assertEqual(batch_item_element["kind"], "message_field")
        self.assertEqual(batch_item_element["name"], "Batch Item")
        self.assertEqual(
            batch_item_element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.7"}]
        )
        self.assertEqual(batch_item_element["direction"], "both")
        self.assertEqual(batch_item_element["scope_state"], "client_1_0")
        self.assertEqual(
            batch_item_element["parent_element_ids"],
            [
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-1-BATCH-ITEM",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-4-BATCH-ITEM",
            ],
        )
        self.assertEqual(batch_item_element["requirement_ids"], [])

        batch_item_clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == expected_batch_item_clause_id
        )
        self.assertEqual(batch_item_clause["source_keywords"], ["REQUIRED"])
        self.assertEqual(batch_item_clause["role"], "unclear")
        self.assertEqual(batch_item_clause["direction"], "unclear")
        self.assertEqual(batch_item_clause["scope_state"], "unclear")
        self.assertEqual(batch_item_clause["requirement_ids"], ["KMIPKIT-REQ-SPEC-9.7-001"])

        batch_item_requirement = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-SPEC-9.7-001"
        )
        self.assertEqual(batch_item_requirement["role"], "unspecified")
        self.assertEqual(batch_item_requirement["direction"], "unclear")
        self.assertEqual(batch_item_requirement["scope_state"], "unclear")
        self.assertEqual(batch_item_requirement["element_ids"], [])

        spec_lines = (ROOT / spec_path).read_text(encoding="cp1252").splitlines()
        batch_item_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == expected_batch_item_clause_id
        )
        self.assertEqual(batch_item_review["html_line_range"], [43819, 43821])
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [43819, 43821]),
            "This field consists of a structure that holds the individual requests or responses in a batch, "
            "and is REQUIRED. The contents of the batch items are described in Section 12.",
        )
        self.assertEqual(batch_item_review["expected_client_requirement_ids"], [])
        self.assertEqual(batch_item_review["source_obligations"][0]["actor"], "unspecified")
        self.assertEqual(batch_item_review["source_obligations"][0]["direction"], "unspecified")
        self.assertEqual(batch_item_review["source_obligations"][0]["scope_disposition"], "unresolved")

        batch_order_option_element = next(
            item for item in catalog["elements"] if item["element_id"] == expected_batch_order_option_element_id
        )
        self.assertEqual(batch_order_option_element["kind"], "message_field")
        self.assertEqual(batch_order_option_element["name"], "Batch Order Option")
        self.assertEqual(
            batch_order_option_element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.8"}]
        )
        self.assertEqual(batch_order_option_element["direction"], "client_to_server")
        self.assertEqual(batch_order_option_element["scope_state"], "client_1_0")
        self.assertEqual(
            batch_order_option_element["parent_element_ids"], ["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION"]
        )
        self.assertEqual(
            batch_order_option_element["requirement_ids"], ["KMIPKIT-REQ-SPEC-9.8-001-001"]
        )

        batch_order_option_clause = next(
            item
            for item in catalog["source_clauses"]
            if item["clause_id"] == expected_batch_order_option_clause_id
        )
        self.assertEqual(batch_order_option_clause["source_keywords"], ["SHALL", "MAY"])
        self.assertEqual(batch_order_option_clause["role"], "both")
        self.assertEqual(batch_order_option_clause["direction"], "both")
        self.assertEqual(batch_order_option_clause["scope_state"], "mixed")
        self.assertEqual(
            batch_order_option_clause["requirement_ids"],
            [
                "KMIPKIT-REQ-SPEC-9.8-001-001",
                "KMIPKIT-REQ-SPEC-9.8-001-002",
                "KMIPKIT-REQ-SPEC-9.8-001-003",
            ],
        )
        batch_order_option_requirements = {
            item["requirement_id"]: item
            for item in catalog["requirements"]
            if item["requirement_id"].startswith("KMIPKIT-REQ-SPEC-9.8-001-")
        }
        retired_client_candidate = batch_order_option_requirements["KMIPKIT-REQ-SPEC-9.8-001-001"]
        self.assertEqual(retired_client_candidate["status"], "retired")
        self.assertEqual(retired_client_candidate["role"], "not_applicable")
        self.assertEqual(retired_client_candidate["direction"], "not_applicable")
        self.assertEqual(retired_client_candidate["scope_state"], "out_of_scope")
        for requirement_id in (
            "KMIPKIT-REQ-SPEC-9.8-001-002",
            "KMIPKIT-REQ-SPEC-9.8-001-003",
        ):
            with self.subTest(server_requirement_id=requirement_id):
                server_requirement = batch_order_option_requirements[requirement_id]
                self.assertEqual(server_requirement["role"], "server")
                self.assertEqual(server_requirement["scope_state"], "server_only")
                self.assertNotIn(requirement_id, expected_batch_order_option_mapping["requirement_ids"])

        batch_order_option_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == expected_batch_order_option_clause_id
        )
        self.assertEqual(batch_order_option_review["html_line_range"], [43862, 43866])
        self.assertEqual(
            _normalized_html_source_span(spec_lines, batch_order_option_review["html_line_range"]),
            batch_order_option_review["source_excerpt"],
        )
        self.assertEqual(
            batch_order_option_review["expected_client_requirement_ids"], []
        )
        self.assertEqual(
            batch_order_option_review["rejected_candidate_mappings"][0]["expected_disposition"],
            "not_a_normative_client_requirement",
        )
        self.assertEqual(
            batch_order_option_review["rejected_candidate_mappings"][0]["requirement_id"],
            "KMIPKIT-REQ-SPEC-9.8-001-001",
        )
        self.assertEqual(
            [obligation["actor"] for obligation in batch_order_option_review["source_obligations"]],
            ["server", "server"],
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [42762, 42778]),
            "Batch Order Option No If omitted, then True is assumed",
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [43862, 43866]),
            "A Boolean value used in requests where the Batch Count is greater than 1. If True, then batched "
            "operations SHALL be executed in the order in which they appear within the request. If False, then "
            "the server MAY choose to execute the batched operations in any order. If not specified, then True "
            "is assumed.",
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [43884, 43901]),
            "Batch Order Option Boolean Table 407: Batch Order Option in Message Request Header",
        )
        check_specific_should = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-SPEC-6.1.7-002-001"
        )
        self.assertEqual(check_specific_should["source_keyword"], "SHOULD")
        self.assertEqual(check_specific_should["condition"], "When a batch contains Check.")
        self.assertEqual(
            check_specific_should["element_ids"],
            [
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-8-BATCH-ORDER-OPTION",
                "KMIPKIT-ELEM-OP-C2S-CHECK",
            ],
        )
        self.assertNotIn(
            "KMIPKIT-REQ-SPEC-6.1.7-002-001", expected_batch_order_option_mapping["requirement_ids"]
        )

        maximum_response_size_element = next(
            item
            for item in catalog["elements"]
            if item["element_id"] == expected_maximum_response_size_element_id
        )
        self.assertEqual(maximum_response_size_element["kind"], "message_field")
        self.assertEqual(maximum_response_size_element["name"], "Maximum Response Size")
        self.assertEqual(
            maximum_response_size_element["source_refs"],
            [{"source_id": "KMIPKIT-SRC-spec", "section": "9.12"}],
        )
        self.assertEqual(maximum_response_size_element["direction"], "client_to_server")
        self.assertEqual(maximum_response_size_element["scope_state"], "client_1_0")
        self.assertEqual(
            maximum_response_size_element["parent_element_ids"],
            ["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE"],
        )
        self.assertEqual(
            maximum_response_size_element["requirement_ids"],
            ["KMIPKIT-REQ-SPEC-9.12-001-001"],
        )
        self.assertEqual(maximum_response_size_element["source_encoding"], "Integer")

        maximum_response_size_clause = next(
            item
            for item in catalog["source_clauses"]
            if item["clause_id"] == expected_maximum_response_size_clause_id
        )
        self.assertEqual(maximum_response_size_clause["source_keywords"], ["OPTIONAL", "SHALL", "SHOULD"])
        self.assertEqual(maximum_response_size_clause["role"], "client")
        self.assertEqual(maximum_response_size_clause["direction"], "client_to_server")
        self.assertEqual(maximum_response_size_clause["scope_state"], "client_1_0")
        self.assertEqual(
            maximum_response_size_clause["requirement_ids"], expected_maximum_response_size_requirement_ids
        )

        maximum_response_size_requirements = {
            item["requirement_id"]: item
            for item in catalog["requirements"]
            if item["requirement_id"].startswith("KMIPKIT-REQ-SPEC-9.12-001-")
        }
        expected_maximum_response_size_ownership = {
            "KMIPKIT-REQ-SPEC-9.12-001-001": [
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-MAXIMUM-RESPONSE-SIZE",
                expected_maximum_response_size_element_id,
            ],
            "KMIPKIT-REQ-SPEC-9.12-001-002": [],
            "KMIPKIT-REQ-SPEC-9.12-001-003": [],
        }
        expected_maximum_response_size_keywords = {
            "KMIPKIT-REQ-SPEC-9.12-001-001": "OPTIONAL",
            "KMIPKIT-REQ-SPEC-9.12-001-002": "SHALL",
            "KMIPKIT-REQ-SPEC-9.12-001-003": "SHOULD",
        }
        for requirement_id, element_ids in expected_maximum_response_size_ownership.items():
            with self.subTest(maximum_response_size_requirement_id=requirement_id):
                requirement = maximum_response_size_requirements[requirement_id]
                self.assertEqual(requirement["source_clause_ids"], [expected_maximum_response_size_clause_id])
                self.assertEqual(requirement["source_keyword"], expected_maximum_response_size_keywords[requirement_id])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(requirement["element_ids"], element_ids)

        maximum_response_size_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == expected_maximum_response_size_clause_id
        )
        maximum_response_size_excerpt = (
            "This is an OPTIONAL field contained in a request message, and is used to indicate the maximum size "
            "of a response, in bytes, that the requester SHALL be able to handle. It SHOULD only be sent in "
            "requests that possibly return large replies."
        )
        self.assertEqual(maximum_response_size_review["source_path"], spec_path)
        self.assertEqual(maximum_response_size_review["source_section"], "9.12")
        self.assertEqual(maximum_response_size_review["source_locator"], {"ordinal": 1, "block_kind": "paragraph"})
        self.assertEqual(maximum_response_size_review["html_line_range"], [44811, 44814])
        self.assertEqual(maximum_response_size_review["source_excerpt"], maximum_response_size_excerpt)
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [44811, 44814]), maximum_response_size_excerpt
        )
        self.assertEqual(
            maximum_response_size_review["expected_client_requirement_ids"],
            expected_maximum_response_size_requirement_ids,
        )
        self.assertEqual(
            [obligation["source_keyword"] for obligation in maximum_response_size_review["source_obligations"]],
            ["OPTIONAL", "SHALL", "SHOULD"],
        )
        self.assertEqual(
            [obligation["actor"] for obligation in maximum_response_size_review["source_obligations"]],
            ["requester", "requester", "requester"],
        )
        self.assertEqual(
            [obligation["direction"] for obligation in maximum_response_size_review["source_obligations"]],
            ["client_to_server", "client_to_server", "client_to_server"],
        )
        self.assertEqual(
            [obligation["scope_disposition"] for obligation in maximum_response_size_review["source_obligations"]],
            ["client_1_0", "client_1_0", "client_1_0"],
        )
        self.assertEqual(
            [obligation["candidate_requirement_id"] for obligation in maximum_response_size_review["source_obligations"]],
            expected_maximum_response_size_requirement_ids,
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [42609, 42625]), "Maximum Response Size No"
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [44835, 44842]), "Maximum Response Size Integer"
        )
        self.assertEqual(
            _normalized_html_source_span(spec_lines, [59962, 59973]), "Maximum Response Size 420050"
        )

        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        self.assertEqual(
            _normalized_html_source_span(profiles_lines, condition["html_line_range"]),
            "Supports the following Message Data Structures [KMIP-SPEC]: Asynchronous Indicator "
            "Batch Count Batch Error Continuation Option Batch Item Batch Order Option Maximum "
            "Response Size Message Extension Operation Protocol Version Result Reason Result Status "
            "Server Correlation Value Time Stamp Unique Batch Item ID",
        )
        expected_excerpt = (
            "This field contains the number of Batch Items in a message and is REQUIRED. If only a single "
            "operation is being requested, then the batch count SHALL be set to 1. The Message Payload, "
            "which follows the Message Header, contains one or more batch items."
        )
        self.assertEqual(_normalized_html_source_span(spec_lines, [43728, 43731]), expected_excerpt)

        element = next(item for item in catalog["elements"] if item["element_id"] == expected_element_id)
        self.assertEqual(element["kind"], "message_field")
        self.assertEqual(element["name"], "Batch Count")
        self.assertEqual(element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.5"}])
        self.assertEqual(element["direction"], "both")
        self.assertEqual(element["scope_state"], "client_1_0")
        self.assertEqual(element["requirement_ids"], expected_requirement_ids)

        clause = next(item for item in catalog["source_clauses"] if item["clause_id"] == expected_clause_id)
        self.assertEqual(clause["source_id"], "KMIPKIT-SRC-spec")
        self.assertEqual(clause["section"], "9.5")
        self.assertEqual(clause["locator"], {"ordinal": 1, "block_kind": "paragraph"})
        self.assertEqual(clause["source_keywords"], ["REQUIRED", "SHALL"])
        self.assertEqual(clause["disposition"], "requirement")
        self.assertEqual(clause["requirement_ids"], expected_requirement_ids)
        self.assertEqual(clause["role"], "client")
        self.assertEqual(clause["direction"], "client_to_server")
        self.assertEqual(clause["scope_state"], "client_1_0")

        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        expected_requirements = {
            "KMIPKIT-REQ-SPEC-9.5-001-001": {
                "source_keyword": "REQUIRED",
                "summary": "Set Batch Count to the number of Batch Items in the message.",
                "condition": None,
            },
            "KMIPKIT-REQ-SPEC-9.5-001-002": {
                "source_keyword": "SHALL",
                "summary": "Set Batch Count to one for a single requested operation.",
                "condition": "For one operation.",
            },
        }
        for requirement_id, expected in expected_requirements.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                self.assertEqual(requirement["source_clause_ids"], [expected_clause_id])
                self.assertEqual(requirement["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.5"}])
                self.assertEqual(requirement["source_keyword"], expected["source_keyword"])
                self.assertEqual(requirement["normative_strength"], "mandatory")
                self.assertEqual(requirement["subject"], "Batch Count")
                self.assertEqual(requirement["summary"], expected["summary"])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["condition"], expected["condition"])
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(requirement["element_ids"], [
                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-COUNT",
                    "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-BATCH-COUNT",
                    expected_element_id,
                ])
                self.assertEqual(requirement["status"], "verified")

        review = next(
            item for item in oracle["source_requirement_reviews"] if item["source_clause_id"] == expected_clause_id
        )
        self.assertEqual(review["source_path"], spec_path)
        self.assertEqual(review["source_section"], "9.5")
        self.assertEqual(review["source_locator"], {"ordinal": 1, "block_kind": "paragraph"})
        self.assertEqual(review["html_line_range"], [43728, 43731])
        self.assertEqual(review["source_excerpt"], expected_excerpt)
        self.assertEqual(review["expected_client_requirement_ids"], expected_requirement_ids)
        self.assertEqual(
            review["source_obligations"],
            [
                {
                    "source_order": 1,
                    "source_text": "This field contains the number of Batch Items in a message and is REQUIRED.",
                    "source_keyword": "REQUIRED",
                    "actor": "client",
                    "direction": "client_to_server",
                    "condition": "A client request is constructed.",
                    "scope_disposition": "client_1_0",
                    "candidate_requirement_id": expected_requirement_ids[0],
                },
                {
                    "source_order": 2,
                    "source_text": "If only a single operation is being requested, then the batch count SHALL be set to 1.",
                    "source_keyword": "SHALL",
                    "actor": "client",
                    "direction": "client_to_server",
                    "condition": "Only one operation is requested.",
                    "scope_disposition": "client_1_0",
                    "candidate_requirement_id": expected_requirement_ids[1],
                },
            ],
        )

        self.assertEqual(_normalized_html_source_span(profiles_lines, [4581, 4582]), "Batch Count")
        self.assertEqual(
            _normalized_html_source_span(profiles_lines, [4583, 4585]),
            "Batch Error Continuation Option",
        )
        self.assertEqual(_normalized_html_source_span(profiles_lines, [4586, 4587]), "Batch Item")
        error_html_span = (
            "This option SHALL only be present if the Batch Count is greater than 1. This option SHALL have "
            "one of three values ( Undo , Stop or Continue). If not specified, then Stop is assumed."
        )
        error_excerpt = (
            "This option SHALL only be present if the Batch Count is greater than 1. This option SHALL have "
            "one of three values (Undo, Stop or Continue). If not specified, then Stop is assumed."
        )
        self.assertEqual(_normalized_html_source_span(spec_lines, [43773, 43775]), error_html_span)

        error_element = next(
            item for item in catalog["elements"] if item["element_id"] == expected_error_element_id
        )
        self.assertEqual(error_element["kind"], "message_field")
        self.assertEqual(error_element["name"], "Batch Error Continuation Option")
        self.assertEqual(error_element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.6"}])
        self.assertEqual(error_element["direction"], "client_to_server")
        self.assertEqual(error_element["scope_state"], "client_1_0")
        self.assertEqual(error_element["requirement_ids"], expected_error_requirement_ids)

        error_clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == expected_error_clause_id
        )
        self.assertEqual(error_clause["source_id"], "KMIPKIT-SRC-spec")
        self.assertEqual(error_clause["section"], "9.6")
        self.assertEqual(error_clause["locator"], {"ordinal": 1, "block_kind": "paragraph"})
        self.assertEqual(error_clause["source_keywords"], ["SHALL"])
        self.assertEqual(error_clause["disposition"], "requirement")
        self.assertEqual(error_clause["requirement_ids"], expected_error_item_requirement_ids)
        self.assertEqual(error_clause["role"], "client")
        self.assertEqual(error_clause["direction"], "client_to_server")
        self.assertEqual(error_clause["scope_state"], "client_1_0")

        expected_error_requirements = {
            "KMIPKIT-REQ-SPEC-9.6-001-001": {
                "summary": "Include Batch Error Continuation Option only when Batch Count is greater than one.",
                "condition": "When including the option.",
            },
            "KMIPKIT-REQ-SPEC-9.6-001-002": {
                "summary": "Use Undo, Stop, or Continue as the option value.",
                "condition": "When the option is present.",
                "source_refs": [
                    {"source_id": "KMIPKIT-SRC-spec", "section": "9.6"},
                    {"source_id": "KMIPKIT-SRC-spec", "section": "11.5"},
                ],
            },
            "KMIPKIT-REQ-SPEC-9.6-001-003": {
                "summary": "If omitted, the option defaults to Stop.",
                "condition": "When omitted.",
            },
        }
        for requirement_id, expected in expected_error_requirements.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                self.assertEqual(requirement["source_clause_ids"], [expected_error_clause_id])
                self.assertEqual(
                    requirement["source_refs"],
                    expected.get(
                        "source_refs",
                        [{"source_id": "KMIPKIT-SRC-spec", "section": "9.6"}],
                    ),
                )
                self.assertEqual(requirement["source_keyword"], "SHALL")
                self.assertEqual(requirement["normative_strength"], "mandatory")
                self.assertEqual(requirement["subject"], "Batch Error Continuation Option")
                self.assertEqual(requirement["summary"], expected["summary"])
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["direction"], "client_to_server")
                self.assertEqual(requirement["condition"], expected["condition"])
                self.assertEqual(requirement["scope_state"], "client_1_0")
                self.assertEqual(
                    requirement["element_ids"],
                    [
                        "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ERROR-CONTINUATION-OPTION",
                        expected_error_element_id,
                    ],
                )
                self.assertEqual(requirement["status"], "verified")

        error_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == expected_error_clause_id
        )
        self.assertEqual(error_review["source_path"], spec_path)
        self.assertEqual(error_review["source_section"], "9.6")
        self.assertEqual(error_review["source_locator"], {"ordinal": 1, "block_kind": "paragraph"})
        self.assertEqual(error_review["html_line_range"], [43773, 43775])
        self.assertEqual(error_review["source_excerpt"], error_excerpt)
        self.assertEqual(error_review["expected_client_requirement_ids"], expected_error_item_requirement_ids)
        self.assertEqual(
            error_review["source_obligations"],
            [
                {
                    "source_order": 1,
                    "source_text": "This option SHALL only be present if the Batch Count is greater than 1.",
                    "source_keyword": "SHALL",
                    "actor": "client",
                    "direction": "client_to_server",
                    "condition": "The client includes Batch Error Continuation Option.",
                    "scope_disposition": "client_1_0",
                    "candidate_requirement_id": expected_error_item_requirement_ids[0],
                },
                {
                    "source_order": 2,
                    "source_text": "This option SHALL have one of three values (Undo, Stop or Continue).",
                    "source_keyword": "SHALL",
                    "actor": "client",
                    "direction": "client_to_server",
                    "condition": "The client includes Batch Error Continuation Option.",
                    "scope_disposition": "client_1_0",
                    "candidate_requirement_id": expected_error_item_requirement_ids[1],
                },
                {
                    "source_order": 3,
                    "source_text": (
                        "This option SHALL have one of three values (Undo, Stop or Continue). If not specified, "
                        "then Stop is assumed."
                    ),
                    "source_keyword": "SHALL",
                    "actor": "client",
                    "direction": "client_to_server",
                    "condition": "The option is omitted; the source states Stop as the default.",
                    "scope_disposition": "client_1_0",
                    "candidate_requirement_id": expected_error_item_requirement_ids[2],
                },
            ],
        )
        self.assertEqual(
            error_review["review_note"],
            "The third source anchor includes the preceding SHALL value constraint and following source-stated "
            "Stop default; the default has no independent uppercase modal.",
        )
        mapped_requirement_ids = {
            requirement_id
            for mapping in condition["supported_item_mappings"]
            for requirement_id in mapping["requirement_ids"]
        }
        self.assertNotIn("KMIPKIT-REQ-SPEC-6.1.7-002-002", mapped_requirement_ids)

    def test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk(self) -> None:
        """Message Extension cites both placements while only reviewed client duties enter the mapping."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        profiles_path = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"
        spec_path = "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        requirement_ids = [
            "KMIPKIT-REQ-SPEC-8.3-004",
            "KMIPKIT-REQ-SPEC-9.13-001-004",
            "KMIPKIT-REQ-SPEC-9.13-001-005",
        ]
        clause_ids = [
            "KMIPKIT-CLAUSE-SPEC-8.3-004",
            "KMIPKIT-CLAUSE-SPEC-8.6-004",
            "KMIPKIT-CLAUSE-SPEC-9.13-001",
        ]
        element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION"
        expected_mapping = {
            "item": "Message Extension",
            "element_id": element_id,
            "requirement_ids": [
                "KMIPKIT-REQ-SPEC-8.3-004",
                "KMIPKIT-REQ-SPEC-9.13-001-001",
                "KMIPKIT-REQ-SPEC-9.13-001-002",
                "KMIPKIT-REQ-SPEC-9.13-001-003",
                "KMIPKIT-REQ-SPEC-9.13-001-004",
                "KMIPKIT-REQ-SPEC-9.13-001-005",
            ],
            "source_clause_ids": clause_ids,
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"][5:7], ["Maximum Response Size", "Message Extension"])
        message_extension_mapping = next(
            mapping for mapping in condition["supported_item_mappings"] if mapping["item"] == "Message Extension"
        )
        self.assertEqual(message_extension_mapping, expected_mapping)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_lines = (ROOT / profiles_path).read_text(encoding="cp1252").splitlines()
        profile_item_text = _normalized_html_source_span(profiles_lines, [4594, 4596])
        self.assertEqual(profile_item_text, "Message Extension")

        spec_lines = (ROOT / spec_path).read_text(encoding="cp1252").splitlines()
        source_ranges = {
            "KMIPKIT-CLAUSE-SPEC-8.3-004": [42956, 42974],
            "KMIPKIT-CLAUSE-SPEC-8.6-004": [43479, 43503],
            "KMIPKIT-CLAUSE-SPEC-9.13-001": [44858, 44872],
        }
        source_candidates = {
            item["clause_id"]: item
            for item in audit_document((ROOT / spec_path).read_bytes(), "KMIPKIT-SRC-spec")
            if item["clause_id"] in clause_ids
        }
        catalog_clauses = {item["clause_id"]: item for item in catalog["source_clauses"]}
        self.assertEqual(set(source_candidates), set(clause_ids))
        self.assertEqual(set(catalog_clauses) & set(clause_ids), set(clause_ids))
        for clause_id in clause_ids:
            with self.subTest(clause_id=clause_id):
                expected_locator = {
                    "ordinal": 1 if clause_id.endswith("9.13-001") else 4,
                    "block_kind": "paragraph" if clause_id.endswith("9.13-001") else "table_row",
                }
                self.assertEqual(source_candidates[clause_id]["locator"], expected_locator)
                self.assertEqual(catalog_clauses[clause_id]["locator"], expected_locator)
                self.assertEqual(
                    catalog_clauses[clause_id]["section"], clause_id.split("-SPEC-")[1].split("-")[0]
                )

        request_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-8.3-004"
        )
        self.assertEqual(request_review["html_line_range"], source_ranges["KMIPKIT-CLAUSE-SPEC-8.3-004"])
        self.assertEqual(request_review["source_locator"], {"ordinal": 4, "block_kind": "table_row"})
        self.assertEqual(request_review["expected_client_requirement_ids"], [requirement_ids[0]])
        request_obligation = request_review["source_obligations"][0]
        self.assertEqual(request_obligation["source_text"], "Message Extension No, MAY be repeated")
        self.assertEqual(request_obligation["source_keyword"], "MAY")
        self.assertEqual(request_obligation["actor"], "client")
        self.assertEqual(request_obligation["direction"], "client_to_server")
        self.assertEqual(request_obligation["condition"], "A Request Message contains a Batch Item.")
        self.assertEqual(request_obligation["scope_disposition"], "client_1_0")
        self.assertEqual(
            _normalized_html_source_span(spec_lines, source_ranges["KMIPKIT-CLAUSE-SPEC-8.3-004"]),
            request_review["source_excerpt"],
        )
        requirements_by_id = {item["requirement_id"]: item for item in catalog["requirements"]}
        request_requirement = requirements_by_id["KMIPKIT-REQ-SPEC-8.3-004"]
        self.assertEqual(
            request_requirement["element_ids"],
            [
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-MESSAGE-EXTENSION",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION",
            ],
        )
        elements_by_id = {item["element_id"]: item for item in catalog["elements"]}
        self.assertEqual(request_requirement["direction"], "client_to_server")
        self.assertEqual(
            {elements_by_id[element_id]["direction"] for element_id in request_requirement["element_ids"]},
            {"client_to_server", "both"},
        )
        request_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-MESSAGE-EXTENSION"
        structure_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION"
        response_element_id = "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION"
        for element_id in (request_element_id, structure_element_id):
            with self.subTest(element_id=element_id):
                self.assertIn(request_requirement["requirement_id"], elements_by_id[element_id]["requirement_ids"])
        response_element = elements_by_id[response_element_id]
        self.assertEqual(response_element["direction"], "server_to_client")
        self.assertEqual(
            response_element["requirement_ids"],
            ["KMIPKIT-REQ-SPEC-9.13-001-001", "KMIPKIT-REQ-SPEC-9.13-001-002"],
        )

        response_clause = catalog_clauses["KMIPKIT-CLAUSE-SPEC-8.6-004"]
        self.assertEqual(response_clause["disposition"], "server_only")
        self.assertEqual(response_clause["requirement_ids"], [])
        response_placement_text = _normalized_html_source_span(
            spec_lines, source_ranges["KMIPKIT-CLAUSE-SPEC-8.6-004"]
        )
        self.assertIn("Message Extension No", response_placement_text)
        self.assertIn("Table 399: Response Batch Item Structure", response_placement_text)

        extension_review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.13-001"
        )
        extension_text = _normalized_html_source_span(
            spec_lines, source_ranges["KMIPKIT-CLAUSE-SPEC-9.13-001"]
        )
        self.assertEqual(extension_review["html_line_range"], source_ranges["KMIPKIT-CLAUSE-SPEC-9.13-001"])
        self.assertEqual(re.sub(r"\s+([.,;:])", r"\1", extension_text), extension_review["source_excerpt"])
        self.assertEqual(
            extension_review["expected_client_requirement_ids"],
            [
                "KMIPKIT-REQ-SPEC-9.13-001-001",
                "KMIPKIT-REQ-SPEC-9.13-001-002",
                "KMIPKIT-REQ-SPEC-9.13-001-003",
                *requirement_ids[1:],
            ],
        )
        obligations = extension_review["source_obligations"]
        sender_shape_ids = {
            "KMIPKIT-REQ-SPEC-9.13-001-001",
            "KMIPKIT-REQ-SPEC-9.13-001-002",
            "KMIPKIT-REQ-SPEC-9.13-001-003",
        }
        sender_shape_obligations = [
            item for item in obligations if item["candidate_requirement_id"] in sender_shape_ids
        ]
        self.assertEqual(
            {item["candidate_requirement_id"] for item in sender_shape_obligations}, sender_shape_ids
        )
        self.assertTrue(
            all(
                item["actor"] == "unspecified"
                and item["direction"] == "unspecified"
                and item["scope_disposition"] == "client_1_0"
                for item in sender_shape_obligations
            )
        )
        self.assertTrue(sender_shape_ids.issubset(expected_mapping["requirement_ids"]))

        receiver_obligations = {
            item["candidate_requirement_id"]: item
            for item in obligations
            if item["candidate_requirement_id"] in requirement_ids[1:]
        }
        self.assertEqual(set(receiver_obligations), set(requirement_ids[1:]))
        expected_receiver_obligations = {
            requirement_ids[1]: {
                "source_order": 5,
                "source_text": "the receiver SHALL reject the entire message.",
                "source_keyword": "SHALL",
                "actor": "receiver (client or server)",
                "direction": "both",
                "condition": "Receiver does not understand a critical extension.",
                "scope_disposition": "client_1_0",
            },
            requirement_ids[2]: {
                "source_order": 6,
                "source_text": "the receiver MAY process the rest of the message as if the extension were not present.",
                "source_keyword": "MAY",
                "actor": "receiver (client or server)",
                "direction": "both",
                "condition": "Receiver does not understand a non-critical extension.",
                "scope_disposition": "client_1_0",
            },
        }
        for receiver_requirement_id, expected in expected_receiver_obligations.items():
            obligation = receiver_obligations[receiver_requirement_id]
            for field, value in expected.items():
                self.assertEqual(obligation[field], value)

        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        expected_requirement_details = {
            requirement_ids[0]: ("MAY", "permission_or_optional", "client", "client_to_server"),
            requirement_ids[1]: ("SHALL", "mandatory", "both", "both"),
            requirement_ids[2]: ("MAY", "permission_or_optional", "both", "both"),
        }
        for requirement_id, (keyword, strength, role, direction) in expected_requirement_details.items():
            with self.subTest(requirement_id=requirement_id):
                requirement = requirements[requirement_id]
                expected_clause_id = clause_ids[0] if requirement_id == requirement_ids[0] else clause_ids[2]
                self.assertEqual(requirement["source_clause_ids"], [expected_clause_id])
                self.assertEqual(requirement["source_keyword"], keyword)
                self.assertEqual(requirement["normative_strength"], strength)
                self.assertEqual(requirement["role"], role)
                self.assertEqual(requirement["direction"], direction)
                self.assertEqual(requirement["scope_state"], "client_1_0")

        elements = {item["element_id"]: item for item in catalog["elements"]}
        extension_element = elements[element_id]
        self.assertEqual(extension_element["kind"], "message_field")
        self.assertEqual(extension_element["name"], "Message Extension")
        self.assertEqual(
            extension_element["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.13"}]
        )
        self.assertEqual(extension_element["source_encoding"], "Structure")
        self.assertEqual(
            extension_element["parent_element_ids"],
            [
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-MESSAGE-EXTENSION",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-MESSAGE-EXTENSION",
            ],
        )
        tag_details = [
            ("Message Extension", "KMIPKIT-ELEM-TAG-420051", "420051", [59976, 59986]),
            ("Vendor Identification", "KMIPKIT-ELEM-TAG-42009D", "42009D", [60964, 60974]),
            ("Criticality Indicator", "KMIPKIT-ELEM-TAG-420026", "420026", [59416, 59427]),
            ("Vendor Extension", "KMIPKIT-ELEM-TAG-42009C", "42009C", [60950, 60961]),
        ]
        for name, tag_id, wire_value, line_range in tag_details:
            with self.subTest(tag=tag_id):
                tag = elements[tag_id]
                self.assertEqual(tag["kind"], "tag")
                self.assertEqual(tag["name"], name)
                self.assertEqual(tag["wire_value"], wire_value)
                self.assertEqual(tag["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}])
                source_text = _normalized_html_source_span(spec_lines, line_range)
                self.assertIn(name, source_text)
                self.assertIn(wire_value, source_text)

        table_418_text = _normalized_html_source_span(spec_lines, [44894, 44944])
        self.assertIn("Table 418: Message Extension Structure in Batch Item", table_418_text)
        child_fields = [
            (
                "Vendor Identification",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-IDENTIFICATION",
                "Text String (with usage limited to alphanumeric, underscore and period – i.e. [A-Za-z0-9_.])",
            ),
            ("Criticality Indicator", "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-CRITICALITY-INDICATOR", "Boolean"),
            ("Vendor Extension", "KMIPKIT-ELEM-MESSAGE-FIELD-9-13-VENDOR-EXTENSION", "Structure"),
        ]
        for name, child_id, encoding in child_fields:
            with self.subTest(child=child_id):
                child = elements[child_id]
                self.assertEqual(child["kind"], "message_field")
                self.assertEqual(child["name"], name)
                self.assertEqual(child["source_encoding"], encoding)
                self.assertEqual(child["parent_element_ids"], [element_id])
                self.assertIn(name, table_418_text)
                self.assertIn(encoding, table_418_text)
        self.assertNotIn("default", extension_text.lower())
        self.assertNotIn("default", table_418_text.lower())

    def test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk(self) -> None:
        """Operation maps to §9.15; §8.3/§8.6 table placements are not normative source clauses."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_mapping = {
            "item": "Operation",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION",
            "requirement_ids": [],
            "source_clause_ids": [],
            "mapping_status": "reviewed",
        }
        mapped_items = [item["item"] for item in condition["supported_item_mappings"]]
        operation_mapping = next(
            item for item in condition["supported_item_mappings"] if item["item"] == "Operation"
        )
        self.assertEqual(operation_mapping, expected_mapping)
        self.assertEqual(condition["supported_items"][6:9], ["Message Extension", "Operation", "Protocol Version"])
        self.assertIn("Operation", mapped_items)
        self.assertEqual(len(condition["supported_items"]), 14)
        self.assertEqual(len(mapped_items), 14)
        self.assertEqual(
            [item for item in condition["supported_items"] if item not in set(mapped_items)],
            [],
        )
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_path = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        spec_path = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        profiles_lines = profiles_path.read_text(encoding="cp1252").splitlines()
        spec_lines = spec_path.read_text(encoding="cp1252").splitlines()
        self.assertEqual(_normalized_html_source_span(profiles_lines, [4597, 4598]), "Operation")

        source_ranges = {
            "KMIPKIT-CLAUSE-SPEC-8.3-001": [42882, 42890],
            "KMIPKIT-CLAUSE-SPEC-8.6-001": [43344, 43353],
        }
        expected_placements = {
            "KMIPKIT-CLAUSE-SPEC-8.3-001": "Operation Yes",
            "KMIPKIT-CLAUSE-SPEC-8.6-001": "Operation Yes, if specified in Request Batch Item",
        }
        source_candidates = {
            item["clause_id"]: item
            for item in audit_document(spec_path.read_bytes(), "KMIPKIT-SRC-spec")
            if item["clause_id"] in source_ranges
        }
        clauses = {item["clause_id"]: item for item in catalog["source_clauses"]}
        elements = {item["element_id"]: item for item in catalog["elements"]}
        self.assertEqual(set(source_candidates), set(source_ranges))
        self.assertFalse(
            any(
                review.get("source_clause_id") in source_ranges
                for review in oracle["source_requirement_reviews"]
            )
        )
        for clause_id, line_range in source_ranges.items():
            with self.subTest(clause=clause_id):
                text = _normalized_html_source_span(spec_lines, line_range)
                self.assertEqual(text, expected_placements[clause_id])
                self.assertEqual(source_candidates[clause_id]["section"], clause_id.split("-SPEC-")[1].split("-")[0])
                self.assertEqual(source_candidates[clause_id]["locator"], {"ordinal": 1, "block_kind": "table_row"})
                clause = clauses[clause_id]
                self.assertEqual(clause["locator"], {"ordinal": 1, "block_kind": "table_row"})
                self.assertEqual(clause["source_keywords"], ["REQUIRED"])
                self.assertEqual(clause["disposition"], "non_applicable")
                self.assertEqual(clause["requirement_ids"], [])
                self.assertEqual(
                    clause["exclusion_rationale"],
                    "Heading, enum/table value, or field-name extractor hit rather than independent obligation.",
                )

        request_field = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-3-OPERATION"]
        response_field = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-6-OPERATION"]
        self.assertEqual(request_field["source_requiredness"], "Yes")
        self.assertEqual(response_field["source_requiredness"], "Yes, if specified in Request Batch Item")
        self.assertEqual(request_field["direction"], "client_to_server")
        self.assertEqual(request_field["scope_state"], "client_1_0")
        self.assertEqual(response_field["direction"], "server_to_client")
        self.assertEqual(response_field["scope_state"], "client_1_0")
        self.assertEqual(request_field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "8.3"}])
        self.assertEqual(response_field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "8.6"}])

        field = elements[expected_mapping["element_id"]]
        self.assertEqual(field["kind"], "message_field")
        self.assertEqual(field["name"], "Operation")
        self.assertEqual(field["direction"], "both")
        self.assertEqual(field["scope_state"], "client_1_0")
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.15"}])
        self.assertEqual(field["source_encoding"], "Enumeration")
        self.assertEqual(
            field["parent_element_ids"],
            ["KMIPKIT-ELEM-MESSAGE-FIELD-8-3-OPERATION", "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-OPERATION"],
        )
        operation_definition = _normalized_html_source_span(spec_lines, [45042, 45080])
        self.assertIn(
            "This field indicates the operation being requested or the operation for which the response is being returned.",
            operation_definition,
        )
        self.assertIn("Operation Enumeration", operation_definition)
        self.assertIn("Table 420: Operation in Batch Item", operation_definition)

        enumeration = elements["KMIPKIT-ELEM-ENUMERATION-OPERATION"]
        self.assertEqual(enumeration["kind"], "enumeration")
        self.assertEqual(enumeration["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "11.36"}])
        enum_span = "\n".join(spec_lines[52884:53697])
        enum_rows = re.findall(r"<tr\b[^>]*>(.*?)</tr>", enum_span, re.IGNORECASE | re.DOTALL)
        source_values = []
        for row in enum_rows:
            cells = re.findall(r"<td\b[^>]*>(.*?)</td>", row, re.IGNORECASE | re.DOTALL)
            if len(cells) == 2:
                source_values.append(
                    tuple(
                        " ".join(html.unescape(re.sub(r"<[^>]+>", " ", cell)).split())
                        for cell in cells
                    )
                )
        source_values = [value for value in source_values if value[0] != "Name"]
        catalog_values = [
            (element["name"], element["wire_value"])
            for element in catalog["elements"]
            if element["kind"] == "enumeration_value"
            and "KMIPKIT-ELEM-ENUMERATION-OPERATION" in element["parent_element_ids"]
        ]
        self.assertEqual(len(source_values), 60)
        self.assertEqual(sorted(catalog_values), sorted(source_values))
        self.assertTrue(
            {
                "Create": "00000001",
                "Locate": "00000008",
                "Get": "0000000A",
                "Get Attributes": "0000000B",
                "Query": "00000018",
                "Extensions": "8XXXXXXX",
            }.items()
            <= dict(source_values).items()
        )

        operation_tag = elements["KMIPKIT-ELEM-TAG-42005C"]
        self.assertEqual(operation_tag["kind"], "tag")
        self.assertEqual(operation_tag["name"], "Operation")
        self.assertEqual(operation_tag["wire_value"], "42005C")
        self.assertEqual(operation_tag["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "11.56"}])
        self.assertIn("Operation 42005C", _normalized_html_source_span(spec_lines, [60122, 60128]))

        same_name_ids = {item["element_id"] for item in catalog["elements"] if item["name"] == "Operation"}
        self.assertEqual(
            same_name_ids,
            {
                "KMIPKIT-ELEM-ENUMERATION-OPERATION",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-OPERATION",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-OPERATION",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-2-OPERATION",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-25-OPERATION",
                "KMIPKIT-ELEM-TAG-42005C",
            },
        )
        feature_spec = (ROOT / "specs" / "010-profile-conformance" / "spec.md").read_text(encoding="utf-8")
        self.assertIn("Profile-required capabilities are minimum requirements, not an allowlist:", feature_spec)

    def test_baseline_protocol_version_condition_preserves_structure_and_unresolved_compatibility(self) -> None:
        """Protocol Version maps structurally while §9.16 compatibility remains unresolved."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_mapping = {
            "item": "Protocol Version",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION",
            "requirement_ids": [],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.16-001"],
            "mapping_status": "reviewed",
        }
        protocol_version_mapping = next(
            item for item in condition["supported_item_mappings"] if item["item"] == "Protocol Version"
        )
        self.assertEqual(protocol_version_mapping, expected_mapping)
        mapped_items = [item["item"] for item in condition["supported_item_mappings"]]
        self.assertEqual(condition["supported_items"][8], "Protocol Version")
        self.assertIn("Protocol Version", mapped_items)
        self.assertEqual(
            [item for item in condition["supported_items"] if item not in set(mapped_items)],
            [],
        )
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_path = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        spec_path = ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        profiles_lines = profiles_path.read_text(encoding="cp1252").splitlines()
        spec_lines = spec_path.read_text(encoding="cp1252").splitlines()
        self.assertIn("Protocol Version", _normalized_html_source_span(profiles_lines, [4600, 4600]))
        self.assertIn("Protocol Version Yes", _normalized_html_source_span(spec_lines, [42592, 42601]))
        self.assertIn("Protocol Version Yes", _normalized_html_source_span(spec_lines, [43128, 43134]))
        protocol_version_definition = _normalized_html_source_span(spec_lines, [45086, 45091])
        self.assertIn(
            "The version number SHALL be specified in two parts, major and minor.", protocol_version_definition
        )
        self.assertIn(
            "SHALL support backward compatibility with versions of the protocol with the same major version.",
            protocol_version_definition,
        )
        self.assertIn(
            "Support for backward compatibility with different major versions is OPTIONAL.",
            protocol_version_definition,
        )

        elements = {item["element_id"]: item for item in catalog["elements"]}
        request_field = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-PROTOCOL-VERSION"]
        response_field = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-5-PROTOCOL-VERSION"]
        field = elements[expected_mapping["element_id"]]
        self.assertEqual(request_field["source_requiredness"], "Yes")
        self.assertEqual(request_field["direction"], "client_to_server")
        self.assertEqual(request_field["scope_state"], "client_1_0")
        self.assertEqual(request_field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "8.2"}])
        self.assertEqual(response_field["source_requiredness"], "Yes")
        self.assertEqual(response_field["direction"], "server_to_client")
        self.assertEqual(response_field["scope_state"], "client_1_0")
        self.assertEqual(response_field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "8.5"}])
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.16"}])
        self.assertEqual(field["source_encoding"], "Structure")
        self.assertEqual(field["direction"], "both")
        self.assertEqual(field["scope_state"], "client_1_0")
        self.assertEqual(
            field["parent_element_ids"],
            ["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-PROTOCOL-VERSION", "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-PROTOCOL-VERSION"],
        )
        for child_id, child_name in (
            ("KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MAJOR", "Protocol Version Major"),
            ("KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MINOR", "Protocol Version Minor"),
        ):
            with self.subTest(protocol_version_child=child_name):
                child = elements[child_id]
                self.assertEqual(child["name"], child_name)
                self.assertEqual(child["source_encoding"], "Integer")
                self.assertEqual(child["parent_element_ids"], [expected_mapping["element_id"]])

        unresolved_clause_id = "KMIPKIT-CLAUSE-SPEC-9.16-001"
        clause = next(item for item in catalog["source_clauses"] if item["clause_id"] == unresolved_clause_id)
        self.assertEqual(clause["disposition"], "source_discrepancy")
        self.assertEqual(clause["requirement_ids"], [])
        self.assertEqual(clause["role"], "unclear")
        self.assertEqual(clause["direction"], "unclear")
        self.assertEqual(clause["scope_state"], "unclear")
        self.assertIn("KMIP 2.1-only 1.x boundary", clause["exclusion_rationale"])
        self.assertFalse(
            any(
                review["source_clause_id"] == unresolved_clause_id
                for review in oracle["source_requirement_reviews"]
            )
        )
        discover_versions_requirement_ids = {
            "KMIPKIT-REQ-SPEC-6.1.16-001-001",
            "KMIPKIT-REQ-SPEC-6.1.16-001-002",
            "KMIPKIT-REQ-SPEC-6.1.16-004",
        }
        catalog_requirement_ids = {item["requirement_id"] for item in catalog["requirements"]}
        self.assertTrue(discover_versions_requirement_ids.issubset(catalog_requirement_ids))
        self.assertTrue(discover_versions_requirement_ids.isdisjoint(expected_mapping["requirement_ids"]))

    def test_baseline_result_reason_condition_preserves_response_structure(self) -> None:
        """Result Reason maps to its response-only field without inventing a client requirement."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_mapping = {
            "item": "Result Reason",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-18-RESULT-REASON",
            "requirement_ids": [],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.18-001"],
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"][9], "Result Reason")
        result_reason_mapping = next(
            item for item in condition["supported_item_mappings"] if item["item"] == "Result Reason"
        )
        self.assertEqual(result_reason_mapping, expected_mapping)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertIn("Result Reason", _normalized_html_source_span(profiles_lines, [4602, 4602]))
        result_reason_definition = _normalized_html_source_span(spec_lines, [45197, 45203])
        self.assertIn("SHALL be present in responses that return a Result Status of Failure", result_reason_definition)
        self.assertIn("SHALL NOT be present in any response that returns a Result Status of Success", result_reason_definition)
        self.assertIn(
            "Result Reason Yes, if Result Status is Failure",
            _normalized_html_source_span(spec_lines, [43399, 43418]),
        )

        elements = {item["element_id"]: item for item in catalog["elements"]}
        response_placement = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-REASON"]
        self.assertEqual(response_placement["direction"], "server_to_client")
        self.assertEqual(response_placement["scope_state"], "client_1_0")
        self.assertEqual(response_placement["source_requiredness"], "Yes, if Result Status is Failure")
        self.assertEqual(response_placement["source_comment"], "REQUIRED if Result Status is Failure, otherwise OPTIONAL")
        self.assertEqual(response_placement["requirement_ids"], [])

        field = elements[expected_mapping["element_id"]]
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.18"}])
        self.assertEqual(field["source_encoding"], "Enumeration")
        self.assertEqual(field["direction"], "server_to_client")
        self.assertEqual(field["parent_element_ids"], ["KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-REASON"])
        self.assertEqual(field["requirement_ids"], [])

        clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.18-001"
        )
        self.assertEqual(clause["disposition"], "server_only")
        self.assertEqual(clause["requirement_ids"], [])
        self.assertEqual(clause["direction"], "server_to_client")
        self.assertEqual(clause["scope_state"], "server_only")

    def test_baseline_result_status_condition_maps_pending_client_duty(self) -> None:
        """Result Status maps its response structure and conditional Pending client duty."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_mapping = {
            "item": "Result Status",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-19-RESULT-STATUS",
            "requirement_ids": ["KMIPKIT-REQ-SPEC-9.19-002"],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.19-001", "KMIPKIT-CLAUSE-SPEC-9.19-002"],
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"][10], "Result Status")
        result_status_mapping = next(
            (item for item in condition["supported_item_mappings"] if item["item"] == "Result Status"),
            None,
        )
        self.assertEqual(result_status_mapping, expected_mapping)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertIn("Result Status", _normalized_html_source_span(profiles_lines, [4604, 4604]))
        self.assertIn("Result Status Yes", _normalized_html_source_span(spec_lines, [43380, 43398]))
        result_status_definition = _normalized_html_source_span(spec_lines, [45244, 45258])
        self.assertIn("The following values MAY be set in this field", result_status_definition)
        self.assertIn("Operation Pending", result_status_definition)
        self.assertIn("actual result via asynchronous polling", result_status_definition)
        self.assertIn(
            "The asynchronous correlation value SHALL be used for the subsequent polling",
            result_status_definition,
        )
        self.assertIn(
            "Result Status Enumeration",
            _normalized_html_source_span(spec_lines, [45272, 45303]),
        )

        elements = {item["element_id"]: item for item in catalog["elements"]}
        response_placement = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-STATUS"]
        self.assertEqual(response_placement["direction"], "server_to_client")
        self.assertEqual(response_placement["scope_state"], "client_1_0")
        self.assertEqual(response_placement["source_requiredness"], "Yes")
        self.assertEqual(response_placement["requirement_ids"], [])

        field = elements[expected_mapping["element_id"]]
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.19"}])
        self.assertEqual(field["source_encoding"], "Enumeration")
        self.assertEqual(field["direction"], "server_to_client")
        self.assertEqual(field["parent_element_ids"], ["KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESULT-STATUS"])
        self.assertEqual(field["requirement_ids"], [])

        clauses = {item["clause_id"]: item for item in catalog["source_clauses"]}
        server_clause = clauses["KMIPKIT-CLAUSE-SPEC-9.19-001"]
        self.assertEqual(server_clause["disposition"], "server_only")
        self.assertEqual(server_clause["requirement_ids"], [])
        self.assertEqual(server_clause["direction"], "server_to_client")
        pending_clause = clauses["KMIPKIT-CLAUSE-SPEC-9.19-002"]
        self.assertEqual(pending_clause["disposition"], "requirement")
        self.assertEqual(pending_clause["requirement_ids"], ["KMIPKIT-REQ-SPEC-9.19-002"])
        self.assertEqual(pending_clause["role"], "client")
        self.assertEqual(pending_clause["direction"], "client_to_server")
        self.assertEqual(pending_clause["scope_state"], "client_1_0")
        self.assertEqual(pending_clause["condition"], "When Result Status is Pending.")

        requirement = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-SPEC-9.19-002"
        )
        self.assertEqual(requirement["source_clause_ids"], ["KMIPKIT-CLAUSE-SPEC-9.19-002"])
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["scope_state"], "client_1_0")
        self.assertEqual(requirement["condition"], "When Result Status is Pending.")

        requirement_mapping = next(
            item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == "KMIPKIT-REQ-SPEC-9.19-002"
        )
        self.assertEqual(requirement_mapping["applies_to"], "all_client_profiles")
        self.assertEqual(requirement_mapping["role"], "client")
        self.assertEqual(requirement_mapping["scope_state"], "client_1_0")
        baseline = next(item for item in oracle["profiles"] if item["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT")
        self.assertIn("KMIPKIT-REQ-SPEC-9.19-002", baseline["requirement_ids"])

        async_spec = (ROOT / "specs" / "009-asynchronous-operations" / "spec.md").read_text(encoding="utf-8")
        self.assertIn("`KMIPKIT-REQ-SPEC-9.19-002`", async_spec)
        self.assertIn("pending_poll_returns_without_repeating_and_zeroizes_request_and_response_copies", async_spec)

    def test_baseline_server_correlation_value_condition_maps_server_response_metadata(self) -> None:
        """The Server Correlation Value is response metadata for client-initiated operations."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_mapping = {
            "item": "Server Correlation Value",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE",
            "requirement_ids": [],
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.10-001"],
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"][11], "Server Correlation Value")
        server_correlation_mapping = next(
            (item for item in condition["supported_item_mappings"] if item["item"] == "Server Correlation Value"),
            None,
        )
        self.assertEqual(server_correlation_mapping, expected_mapping)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertEqual(
            _normalized_html_source_span(profiles_lines, [4605, 4607]),
            "Server Correlation Value",
        )
        server_correlation_definition = _normalized_html_source_span(spec_lines, [43957, 43963])
        self.assertIn("SHOULD be provided by the server", server_correlation_definition)
        self.assertIn("SHOULD be globally unique", server_correlation_definition)
        self.assertIn("SHOULD be logged by the server with each request", server_correlation_definition)
        self.assertIn(
            "For client to server operations the Server Correlation Value is provided in the response",
            server_correlation_definition,
        )
        self.assertIn(
            "For server to client operations, the Server Correlation Value is provided in the request",
            server_correlation_definition,
        )
        self.assertIn("Server Correlation Value Text String", _normalized_html_source_span(spec_lines, [43981, 43991]))
        self.assertIn("420106", _normalized_html_source_span(spec_lines, [62330, 62340]))
        self.assertIn("Server Correlation Value No", _normalized_html_source_span(spec_lines, [42648, 42658]))
        self.assertIn("Server Correlation Value No", _normalized_html_source_span(spec_lines, [43241, 43252]))

        elements = {item["element_id"]: item for item in catalog["elements"]}
        field = elements[expected_mapping["element_id"]]
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.10"}])
        self.assertEqual(field["source_encoding"], "Text String")
        self.assertEqual(field["direction"], "both")
        self.assertEqual(
            field["parent_element_ids"],
            [
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-SERVER-CORRELATION-VALUE",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-CORRELATION-VALUE",
            ],
        )
        self.assertEqual(field["requirement_ids"], [])
        self.assertEqual(elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-SERVER-CORRELATION-VALUE"]["source_requiredness"], "No")
        self.assertEqual(elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-CORRELATION-VALUE"]["source_requiredness"], "No")

        clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.10-001"
        )
        self.assertEqual(clause["disposition"], "server_only")
        self.assertEqual(clause["scope_state"], "server_only")
        self.assertEqual(clause["requirement_ids"], [])

        execution_spec = (ROOT / "specs" / "007-client-execution" / "spec.md").read_text(encoding="utf-8")
        execution_tests = (ROOT / "crates" / "kmipkit-client" / "tests" / "unit" / "execution_tests.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("KMIPKIT-0007-FR-016", execution_spec)
        self.assertIn("outgoing_client_request_omits_server_correlation_value", execution_tests)

    def test_baseline_time_stamp_condition_maps_client_capabilities_not_server_duties(self) -> None:
        """Time Stamp maps its optional request and conditional client capabilities only."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_requirement_ids = ["KMIPKIT-REQ-SPEC-9.20-001-001", "KMIPKIT-REQ-SPEC-9.20-001-002"]
        expected_mapping = {
            "item": "Time Stamp",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP",
            "requirement_ids": expected_requirement_ids,
            "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-9.20-001"],
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"][12], "Time Stamp")
        time_stamp_mapping = next(
            (item for item in condition["supported_item_mappings"] if item["item"] == "Time Stamp"),
            None,
        )
        self.assertEqual(time_stamp_mapping, expected_mapping)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        review = next(
            item
            for item in oracle["source_requirement_reviews"]
            if item["source_clause_id"] == "KMIPKIT-CLAUSE-SPEC-9.20-001"
        )
        self.assertEqual(review["expected_client_requirement_ids"], expected_requirement_ids)
        obligations = review["source_obligations"]
        self.assertEqual(
            [
                (item["actor"], item["direction"], item["scope_disposition"], item["candidate_requirement_id"])
                for item in obligations
            ],
            [
                ("client", "client_to_server", "client_1_0", expected_requirement_ids[0]),
                ("server", "both", "server_only", None),
                ("unspecified", "unspecified", "unresolved", None),
                ("server", "server_to_client", "server_only", None),
                ("client", "client_to_server", "client_1_0", expected_requirement_ids[1]),
            ],
        )

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertIn("Time Stamp", _normalized_html_source_span(profiles_lines, [4609, 4609]))
        time_stamp_definition = _normalized_html_source_span(spec_lines, [45309, 45316])
        self.assertIn("OPTIONAL field contained in a client request", time_stamp_definition)
        self.assertIn("REQUIRED in a server request and response", time_stamp_definition)
        self.assertIn("MAY be used to enforce reasonable time usage at a client", time_stamp_definition)
        self.assertIn("a server MAY choose to reject a request", time_stamp_definition)
        self.assertIn("the time stamp MAY be used by a client that has no real-time clock", time_stamp_definition)
        self.assertIn("Time Stamp No", _normalized_html_source_span(spec_lines, [42780, 42796]))
        self.assertIn("Time Stamp Yes", _normalized_html_source_span(spec_lines, [43143, 43159]))
        self.assertIn("Time Stamp Date-Time", _normalized_html_source_span(spec_lines, [45335, 45345]))
        self.assertIn("420092", _normalized_html_source_span(spec_lines, [60820, 60830]))

        elements = {item["element_id"]: item for item in catalog["elements"]}
        request_placement = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP"]
        response_placement = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-5-TIME-STAMP"]
        self.assertEqual(request_placement["direction"], "client_to_server")
        self.assertEqual(request_placement["source_requiredness"], "No")
        self.assertEqual(response_placement["direction"], "server_to_client")
        self.assertEqual(response_placement["source_requiredness"], "Yes")

        field = elements[expected_mapping["element_id"]]
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.20"}])
        self.assertEqual(field["source_encoding"], "Date-Time")
        self.assertEqual(field["direction"], "both")
        self.assertEqual(
            field["parent_element_ids"],
            ["KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP", "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-TIME-STAMP"],
        )

    def test_baseline_unique_batch_item_id_maps_client_batch_requirements_not_server_echo(self) -> None:
        """Unique Batch Item ID maps conditional client use without a server echo duty."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-005"
        )
        expected_requirement_ids = [
            "KMIPKIT-REQ-SPEC-8.3-003",
            "KMIPKIT-REQ-SPEC-9.21-001-001",
            "KMIPKIT-REQ-PROF-4.1.2-007",
        ]
        expected_mapping = {
            "item": "Unique Batch Item ID",
            "element_id": "KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID",
            "requirement_ids": expected_requirement_ids,
            "source_clause_ids": [
                "KMIPKIT-CLAUSE-SPEC-8.3-003",
                "KMIPKIT-CLAUSE-SPEC-9.21-001",
                "KMIPKIT-CLAUSE-PROF-4.1.2-007",
            ],
            "mapping_status": "reviewed",
        }

        self.assertEqual(condition["supported_items"][13], "Unique Batch Item ID")
        unique_batch_mapping = next(
            (item for item in condition["supported_item_mappings"] if item["item"] == "Unique Batch Item ID"),
            None,
        )
        self.assertEqual(unique_batch_mapping, expected_mapping)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")

        reviews = {item["source_clause_id"]: item for item in oracle["source_requirement_reviews"]}
        batch_count_review = reviews["KMIPKIT-CLAUSE-SPEC-8.3-003"]
        self.assertEqual(batch_count_review["expected_client_requirement_ids"], [expected_requirement_ids[0]])
        self.assertIn("REQUIRED if Batch Count > 1", batch_count_review["source_excerpt"])
        self.assertEqual(batch_count_review["source_obligations"][0]["actor"], "client")
        self.assertEqual(batch_count_review["source_obligations"][0]["direction"], "client_to_server")
        self.assertEqual(batch_count_review["source_obligations"][0]["scope_disposition"], "client_1_0")

        request_optional_review = reviews["KMIPKIT-CLAUSE-SPEC-9.21-001"]
        self.assertEqual(
            request_optional_review["expected_client_requirement_ids"],
            ["KMIPKIT-REQ-SPEC-9.21-001-001"],
        )
        self.assertEqual(
            [item["candidate_requirement_id"] for item in request_optional_review["source_obligations"]],
            ["KMIPKIT-REQ-SPEC-9.21-001-001", "KMIPKIT-REQ-SPEC-9.21-001-002"],
        )
        self.assertEqual(
            request_optional_review["source_obligations"][1]["actor"],
            "response producer (endpoint not named)",
        )
        self.assertEqual(request_optional_review["source_obligations"][1]["direction"], "server_to_client")
        self.assertEqual(request_optional_review["source_obligations"][1]["scope_disposition"], "unclear")
        self.assertNotIn("KMIPKIT-REQ-SPEC-9.21-001-002", unique_batch_mapping["requirement_ids"])

        profile_may_review = reviews["KMIPKIT-CLAUSE-PROF-4.1.2-007"]
        self.assertEqual(profile_may_review["expected_client_requirement_ids"], [expected_requirement_ids[2]])
        self.assertIn("MAY be specified", profile_may_review["source_excerpt"])

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        self.assertIn("Unique Batch Item ID", _normalized_html_source_span(profiles_lines, [4611, 4612]))
        self.assertIn("multiple batch items", _normalized_html_source_span(profiles_lines, [4488, 4493]))
        self.assertIn(
            "Unique Batch Item ID No REQUIRED if Batch Count > 1",
            _normalized_html_source_span(spec_lines, [42921, 42935]),
        )
        self.assertIn(
            "Unique Batch Item ID No REQUIRED if present in Request",
            _normalized_html_source_span(spec_lines, [43361, 43377]),
        )
        request_response_definition = _normalized_html_source_span(spec_lines, [45355, 45360])
        self.assertIn("OPTIONAL field contained in a request", request_response_definition)
        self.assertIn("responses to that request SHALL have the same Unique Batch Item ID", request_response_definition)
        self.assertIn("Unique Batch Item ID Byte String", _normalized_html_source_span(spec_lines, [45378, 45395]))
        self.assertIn("420093", _normalized_html_source_span(spec_lines, [60833, 60844]))

        elements = {item["element_id"]: item for item in catalog["elements"]}
        request_placement = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-3-UNIQUE-BATCH-ITEM-ID"]
        response_placement = elements["KMIPKIT-ELEM-MESSAGE-FIELD-8-6-UNIQUE-BATCH-ITEM-ID"]
        self.assertEqual(request_placement["direction"], "client_to_server")
        self.assertEqual(response_placement["direction"], "server_to_client")
        response_clause = next(
            item for item in catalog["source_clauses"] if item["clause_id"] == "KMIPKIT-CLAUSE-SPEC-8.6-003"
        )
        self.assertEqual(response_clause["disposition"], "server_only")

        field = elements[expected_mapping["element_id"]]
        self.assertEqual(field["source_refs"], [{"source_id": "KMIPKIT-SRC-spec", "section": "9.21"}])
        self.assertEqual(field["source_encoding"], "Byte String")
        self.assertEqual(field["direction"], "both")
        self.assertEqual(
            field["parent_element_ids"],
            [
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-UNIQUE-BATCH-ITEM-ID",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-UNIQUE-BATCH-ITEM-ID",
            ],
        )

    def test_baseline_message_protocols_crosswalk_keeps_unresolved_and_server_duties_out(self) -> None:
        """Baseline Message Protocols excludes unresolved and server-only duties from client support mappings."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        inventory = next(
            item
            for item in oracle["provenance"]["profile_condition_inventories"]
            if item["source_id"] == "KMIPKIT-SRC-profiles" and item["section"] == "5.1.1"
        )
        condition = next(
            item for item in inventory["conditions"] if item["condition_id"] == "KMIPKIT-PROF-COND-5.1.1-006"
        )
        expected_mappings = [
            {
                "item": "Authentication",
                "element_id": None,
                "requirement_ids": [
                    "KMIPKIT-REQ-PROF-3.1-001",
                    "KMIPKIT-REQ-PROF-3.1.1-002",
                    "KMIPKIT-REQ-PROF-3.1.1-005",
                    "KMIPKIT-REQ-PROF-3.1.2-002",
                    "KMIPKIT-REQ-PROF-3.1.2-005",
                ],
                "source_clause_ids": [
                    "KMIPKIT-CLAUSE-PROF-3-002",
                    "KMIPKIT-CLAUSE-PROF-3-003",
                    "KMIPKIT-CLAUSE-PROF-3-004",
                    "KMIPKIT-CLAUSE-PROF-3.1-001",
                    "KMIPKIT-CLAUSE-PROF-3.1.1-001",
                    "KMIPKIT-CLAUSE-PROF-3.1.1-002",
                    "KMIPKIT-CLAUSE-PROF-3.1.1-003",
                    "KMIPKIT-CLAUSE-PROF-3.1.1-004",
                    "KMIPKIT-CLAUSE-PROF-3.1.1-005",
                    "KMIPKIT-CLAUSE-PROF-3.1.2-001",
                    "KMIPKIT-CLAUSE-PROF-3.1.2-002",
                    "KMIPKIT-CLAUSE-PROF-3.1.2-003",
                    "KMIPKIT-CLAUSE-PROF-3.1.2-004",
                    "KMIPKIT-CLAUSE-PROF-3.1.2-005",
                    "KMIPKIT-CLAUSE-PROF-3.1.3-001",
                    "KMIPKIT-CLAUSE-PROF-3.1.3-002",
                    "KMIPKIT-CLAUSE-PROF-3.1.3-003",
                    "KMIPKIT-CLAUSE-PROF-3.1.4-001",
                ],
                "mapping_status": "reviewed",
            },
            {
                "item": "Transport",
                "element_id": None,
                "requirement_ids": [
                    "KMIPKIT-REQ-SPEC-10.4-001-001",
                    "KMIPKIT-REQ-SPEC-10.4-001-002",
                    "KMIPKIT-REQ-SPEC-10.4-001-003",
                    "KMIPKIT-REQ-PROF-3.2-001",
                    "KMIPKIT-REQ-PROF-3.2.1-001",
                    "KMIPKIT-REQ-PROF-3.2.2-001",
                    "KMIPKIT-REQ-PROF-3.2.3-001",
                    "KMIPKIT-REQ-PROF-3.2.4-002",
                    "KMIPKIT-REQ-PROF-5.3.1-001",
                    "KMIPKIT-REQ-PROF-5.3.1-002",
                    "KMIPKIT-REQ-PROF-5.3.1-003",
                    "KMIPKIT-REQ-PROF-5.3.1-004",
                    "KMIPKIT-REQ-PROF-5.3.1-008",
                    "KMIPKIT-REQ-PROF-5.3.1-009",
                ],
                "source_clause_ids": [
                    "KMIPKIT-CLAUSE-SPEC-10.4-001",
                    "KMIPKIT-CLAUSE-PROF-3.2-001",
                    "KMIPKIT-CLAUSE-PROF-3.2.1-001",
                    "KMIPKIT-CLAUSE-PROF-3.2.2-001",
                    "KMIPKIT-CLAUSE-PROF-3.2.3-001",
                    "KMIPKIT-CLAUSE-PROF-3.2.4-002",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-001",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-002",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-003",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-004",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-008",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-009",
                ],
                "mapping_status": "reviewed",
            },
            {
                "item": "TTLV",
                "element_id": None,
                "requirement_ids": [
                    "KMIPKIT-REQ-SPEC-10.1.2-001",
                    "KMIPKIT-REQ-SPEC-10.1.2-002-001",
                    "KMIPKIT-REQ-SPEC-10.1.2-002-002",
                    "KMIPKIT-REQ-SPEC-10.1.5-001-001",
                    "KMIPKIT-REQ-SPEC-10.1.5-001-002",
                    "KMIPKIT-REQ-SPEC-11.56-001",
                    "KMIPKIT-REQ-PROF-5.3.1-005",
                    "KMIPKIT-REQ-PROF-5.3.1-010",
                ],
                "source_clause_ids": [
                    "KMIPKIT-CLAUSE-SPEC-10.1.2-001",
                    "KMIPKIT-CLAUSE-SPEC-10.1.2-002",
                    "KMIPKIT-CLAUSE-SPEC-10.1.5-001",
                    "KMIPKIT-CLAUSE-SPEC-10.2-001",
                    "KMIPKIT-CLAUSE-SPEC-11.56-001",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-005",
                    "KMIPKIT-CLAUSE-PROF-5.3.1-010",
                ],
                "mapping_status": "reviewed",
            },
        ]

        self.assertEqual(condition["supported_items"], ["Authentication", "Transport", "TTLV"])
        self.assertEqual(condition.get("supported_item_mappings"), expected_mappings)
        self.assertIsNone(condition["requirement_id"])
        self.assertEqual(condition["scope_disposition"], "profile_conditional")
        self.assertEqual(condition["mapping_status"], "reviewed")
        server_authentication_clause = next(
            item
            for item in catalog["source_clauses"]
            if item["clause_id"] == "KMIPKIT-CLAUSE-SPEC-10.3-001"
        )
        self.assertEqual(server_authentication_clause["disposition"], "server_only")
        self.assertNotIn("KMIPKIT-CLAUSE-SPEC-10.3-001", expected_mappings[0]["source_clause_ids"])

        reviews = {item["source_clause_id"]: item for item in oracle["source_requirement_reviews"]}
        unresolved_auth_ids = {
            "KMIPKIT-REQ-PROF-3-002",
            "KMIPKIT-REQ-PROF-3-003",
            "KMIPKIT-REQ-PROF-3-004",
        }
        unresolved_auth_clauses = {
            "KMIPKIT-CLAUSE-PROF-3-002": "KMIPKIT-REQ-PROF-3-002",
            "KMIPKIT-CLAUSE-PROF-3-003": "KMIPKIT-REQ-PROF-3-003",
            "KMIPKIT-CLAUSE-PROF-3-004": "KMIPKIT-REQ-PROF-3-004",
        }
        for clause_id, requirement_id in unresolved_auth_clauses.items():
            review = reviews[clause_id]
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertEqual(review["source_obligations"][0]["candidate_requirement_id"], requirement_id)
            self.assertEqual(review["source_obligations"][0]["actor"], "unresolved")
            self.assertEqual(review["source_obligations"][0]["direction"], "unresolved")

        requirement_ids = {req_id for mapping in expected_mappings for req_id in mapping["requirement_ids"]}
        self.assertTrue(unresolved_auth_ids.isdisjoint(requirement_ids))
        https_authenticity_review = reviews["KMIPKIT-CLAUSE-PROF-3.2.3-001"]
        self.assertEqual(
            https_authenticity_review["expected_client_requirement_ids"],
            ["KMIPKIT-REQ-PROF-3.2.3-001"],
        )
        self.assertEqual(https_authenticity_review["source_obligations"][0]["actor"], "client and server")
        self.assertEqual(https_authenticity_review["source_obligations"][0]["direction"], "unspecified")
        self.assertEqual(
            https_authenticity_review["source_obligations"][0]["scope_disposition"],
            "profile_conditional",
        )
        self.assertIn("KMIPKIT-REQ-PROF-3.2.3-001", requirement_ids)
        https_authenticity_requirement = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-PROF-3.2.3-001"
        )
        self.assertEqual(https_authenticity_requirement["scope_state"], "profile_conditional")
        self.assertTrue(
            {
                "KMIPKIT-REQ-SPEC-10.1.2-001",
                "KMIPKIT-REQ-SPEC-10.1.2-002-001",
                "KMIPKIT-REQ-SPEC-10.1.2-002-002",
                "KMIPKIT-REQ-SPEC-10.1.5-001-001",
                "KMIPKIT-REQ-SPEC-10.1.5-001-002",
                "KMIPKIT-REQ-SPEC-11.56-001",
            }.issubset(requirement_ids)
        )
        self.assertNotIn("KMIPKIT-REQ-PROF-5.3.1-011", requirement_ids)

        profiles_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-profiles-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        spec_lines = (
            ROOT / "specification" / "oasis" / "kmip-2.1" / "upstream" / "kmip-spec-v2.1-os.html"
        ).read_text(encoding="cp1252").splitlines()
        message_protocols = _normalized_html_source_span(profiles_lines, [4614, 4627])
        for item_name in ("Authentication", "Transport", "TTLV"):
            self.assertIn(item_name, message_protocols)
        self.assertIn(
            "Conformant KMIP clients SHOULD support",
            _normalized_html_source_span(profiles_lines, [4016, 4022]),
        )
        self.assertIn(
            "at least one of the following cipher suites",
            _normalized_html_source_span(profiles_lines, [4068, 4074]),
        )
        self.assertIn(
            "servers and clients SHALL handle client authenticity",
            _normalized_html_source_span(profiles_lines, [4250, 4253]),
        )
        self.assertIn("application/octet-stream", _normalized_html_source_span(profiles_lines, [5137, 5146]))
        self.assertIn("mandatory TTLV messaging protocol", _normalized_html_source_span(spec_lines, [45793, 45797]))

        for clause_id in (
            "KMIPKIT-CLAUSE-SPEC-10.1.2-001",
            "KMIPKIT-CLAUSE-SPEC-10.1.2-002",
            "KMIPKIT-CLAUSE-SPEC-10.1.5-001",
            "KMIPKIT-CLAUSE-SPEC-11.56-001",
        ):
            self.assertTrue(reviews[clause_id]["expected_client_requirement_ids"])
        informative_ttlv_context = next(
            item
            for item in catalog["source_clauses"]
            if item["clause_id"] == "KMIPKIT-CLAUSE-SPEC-10.2-001"
        )
        self.assertEqual(informative_ttlv_context["disposition"], "informative_context")
        self.assertEqual(informative_ttlv_context["requirement_ids"], [])

    def test_foundry_basic_condition_text_names_its_inheriting_profiles(self) -> None:
        """The catalog and source review describe the same normative inheritance."""
        catalog = json.loads((ROOT / "specification" / "catalog" / "kmip-2.1.json").read_text(encoding="utf-8"))
        oracle = json.loads(
            (ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json")
            .read_text(encoding="utf-8")
        )
        clause_id = "KMIPKIT-CLAUSE-PROF-5.7.1-001"
        requirement_id = "KMIPKIT-REQ-PROF-5.7.1-001"
        expected_condition = (
            "Applies to the Basic Symmetric Key Foundry Client profile and to Intermediate and Advanced "
            "Symmetric Key Foundry Client profiles that incorporate section 5.7.1 by reference; "
            "no support claim is inferred."
        )
        clause = next(item for item in catalog["source_clauses"] if item["clause_id"] == clause_id)
        requirement = next(item for item in catalog["requirements"] if item["requirement_id"] == requirement_id)
        review = next(item for item in oracle["source_requirement_reviews"] if item["source_clause_id"] == clause_id)
        obligation = review["source_obligations"][0]

        self.assertEqual(clause["condition"], expected_condition)
        self.assertEqual(obligation["condition"], expected_condition)
        self.assertEqual(requirement["condition"], expected_condition)
        self.assertEqual(review["applies_to"], "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-BASIC-CLIENT")
        self.assertEqual(
            review["inherited_by_profiles"],
            [
                "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-ADVANCED-CLIENT",
                "KMIPKIT-PROFILE-SYMMETRIC-KEY-FOUNDRY-INTERMEDIATE-CLIENT",
            ],
        )

    def test_tape_library_includes_normatively_referenced_mandatory_test_cases(self) -> None:
        """Profiles v2.1 §6.28 item 6 incorporates the §5.12.6 test cases."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profile_id = "KMIPKIT-PROFILE-TAPE-LIBRARY-CLIENT"
        profile = next(item for item in catalog["profiles"] if item["profile_id"] == profile_id)
        expected = next(item for item in oracle["profiles"] if item["profile_id"] == profile_id)
        test_case_clauses = {
            "KMIPKIT-CLAUSE-PROF-5.12.6.1-001",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-001",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-002",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-003",
            "KMIPKIT-CLAUSE-PROF-5.12.6.2-004",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-001",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-002",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-003",
            "KMIPKIT-CLAUSE-PROF-5.12.6.3-004",
        }
        inclusion_id = "KMIPKIT-INCLUSION-PROF-TAPE-LIBRARY-TEST-CASES"
        inclusion = next(
            (item for item in catalog["normative_inclusions"] if item["inclusion_id"] == inclusion_id),
            None,
        )
        oracle_edge = next(
            (item for item in oracle["provenance"]["inclusion_edges"] if item["inclusion_id"] == inclusion_id),
            None,
        )

        self.assertEqual(
            inclusion,
            {
                "inclusion_id": inclusion_id,
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.28-006",
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.12.6",
            },
        )
        self.assertIn("5.12.6", expected["direct_profiles_sections"])
        self.assertIn(inclusion_id, expected["normative_inclusion_ids"])
        self.assertEqual(
            oracle_edge,
            {
                "inclusion_id": inclusion_id,
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.28-006",
                "included_source_id": "KMIPKIT-SRC-profiles",
                "included_section": "5.12.6",
                "source_locator": "Profiles v2.1 §6.28 item 6",
            },
        )
        self.assertTrue(test_case_clauses.issubset(expected["source_clause_ids"]))
        self.assertFalse(test_case_clauses.intersection(profile["direct_source_clause_ids"]))
        self.assertTrue(test_case_clauses.issubset(profile["source_clause_ids"]))
        self.assertTrue(
            test_case_clauses.issubset(
                {
                    item["source_clause_id"]
                    for item in expected["direct_root_exclusions"]
                    if item["expected_disposition"] == "included_not_direct"
                }
            )
        )
        reviewed_client_requirements = {
            "KMIPKIT-REQ-PROF-5.12.6.1-001",
            "KMIPKIT-REQ-PROF-5.12.6.2-002",
            "KMIPKIT-REQ-PROF-5.12.6.2-003",
            "KMIPKIT-REQ-PROF-5.12.6.2-004",
            "KMIPKIT-REQ-PROF-5.12.6.3-001",
            "KMIPKIT-REQ-PROF-5.12.6.3-002",
            "KMIPKIT-REQ-PROF-5.12.6.3-003",
        }
        self.assertTrue(reviewed_client_requirements.issubset(expected["requirement_ids_audited_subset"]))
        self.assertEqual(
            reviewed_client_requirements,
            {item for item in profile["requirement_ids"] if item in reviewed_client_requirements},
        )
        cleanup_requirement = next(
            item
            for item in catalog["requirements"]
            if item["requirement_id"] == "KMIPKIT-REQ-PROF-5.12.6.3-004"
        )
        self.assertEqual(cleanup_requirement["role"], "unclear")
        self.assertEqual(cleanup_requirement["scope_state"], "unclear")
        self.assertNotIn(cleanup_requirement["requirement_id"], profile["requirement_ids"])

    def test_unresolved_actor_requirement_is_not_a_client_profile_obligation(self) -> None:
        """Specification v2.1 §4.57 does not assign the Active predicates to a client."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        clause_id = "KMIPKIT-CLAUSE-SPEC-4.57-003"
        requirement_id = "KMIPKIT-REQ-SPEC-4.57-003-004"
        requirement = next(item for item in catalog["requirements"] if item["requirement_id"] == requirement_id)
        review = next(item for item in oracle["source_requirement_reviews"] if item["source_clause_id"] == clause_id)
        crosswalk = [item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == requirement_id]
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}

        self.assertEqual(requirement["role"], "unspecified")
        self.assertEqual(requirement["scope_state"], "unclear")
        self.assertEqual(review["expected_client_requirement_ids"], [])
        self.assertEqual(crosswalk, [])
        self.assertEqual(len(oracle["profiles"]), 18)
        for expected in oracle["profiles"]:
            with self.subTest(profile_id=expected["profile_id"]):
                actual = catalog_profiles[expected["profile_id"]]
                self.assertIn(clause_id, expected["source_clause_ids"])
                self.assertIn(clause_id, actual["source_clause_ids"])
                self.assertNotIn(requirement_id, expected["requirement_ids_audited_subset"])
                self.assertNotIn(requirement_id, actual["requirement_ids"])

    def test_oracle_audits_all_mapped_client_1_0_requirements_in_each_profile_closure(self) -> None:
        """Every source-reviewed client_1_0 requirement is represented in the profile audit subset."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        mappings = oracle["requirement_crosswalk"]

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            closure_clause_ids = set(profile["source_clause_ids"])
            expected_requirement_ids = {
                mapping["requirement_id"]
                for mapping in mappings
                if mapping.get("source_clause_id") in closure_clause_ids
                and _mapping_applies_to_profile(mapping, profile_id)
                and mapping.get("role") in {"client", "both"}
                and mapping.get("scope_state") == "client_1_0"
            }
            missing_requirement_ids = expected_requirement_ids - set(profile["requirement_ids_audited_subset"])

            with self.subTest(profile_id=profile_id):
                self.assertFalse(
                    missing_requirement_ids,
                    f"missing {len(missing_requirement_ids)} source-mapped client_1_0 IDs: "
                    f"{sorted(missing_requirement_ids)[:8]}",
                )

    def test_absent_link_may_is_in_each_independently_computed_client_profile_closure(self) -> None:
        """The §4.31 client MAY is inherited by each profile through its source closure."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        requirement_id = "KMIPKIT-REQ-SPEC-4.31-002"
        clause_id = "KMIPKIT-CLAUSE-SPEC-4.31-002"
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"] if item["role"] == "client"}
        oracle_profiles = [item for item in oracle["profiles"] if item["role"] == "client"]

        self.assertEqual(len(catalog_profiles), 18)
        self.assertEqual(len(oracle_profiles), 18)
        for audited_profile in oracle_profiles:
            profile_id = audited_profile["profile_id"]
            catalog_profile = catalog_profiles[profile_id]
            computed = compute_profile_closure(
                catalog,
                catalog_profile["direct_source_clause_ids"],
                profile_id=profile_id,
            )
            with self.subTest(profile_id=profile_id):
                self.assertIn(clause_id, audited_profile["source_clause_ids"])
                self.assertIn(clause_id, computed.source_clause_ids)
                self.assertTrue(
                    requirement_id in computed.requirement_ids,
                    f"the independently computed source closure omits {requirement_id}",
                )
                self.assertTrue(
                    requirement_id in catalog_profile["requirement_ids"],
                    f"the generated catalog profile omits {requirement_id}",
                )
                self.assertTrue(
                    requirement_id in audited_profile["requirement_ids_audited_subset"],
                    f"the independent oracle subset omits {requirement_id}",
                )

    def test_oracle_audits_all_mapped_conditional_requirements_in_each_profile_closure(self) -> None:
        """Every source-reviewed conditional client requirement is scoped to its applicable profile."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        mappings = oracle["requirement_crosswalk"]
        applicable_scopes = {
            "profile_conditional",
            "client_1_0 + server_only (split/shared)",
        }

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            closure_clause_ids = set(profile["source_clause_ids"])
            expected_requirement_ids = {
                mapping["requirement_id"]
                for mapping in mappings
                if mapping.get("source_clause_id") in closure_clause_ids
                and _mapping_applies_to_profile(mapping, profile_id)
                and mapping.get("role") in {"client", "both"}
                and mapping.get("scope_state") in applicable_scopes
            }
            missing_requirement_ids = expected_requirement_ids - set(profile["requirement_ids_audited_subset"])

            with self.subTest(profile_id=profile_id):
                self.assertFalse(
                    missing_requirement_ids,
                    f"missing {len(missing_requirement_ids)} mapped conditional IDs: "
                    f"{sorted(missing_requirement_ids)[:8]}",
                )

    def test_profile_oracle_requirement_arrays_match_independent_source_reviews(self) -> None:
        """Full profile requirement sets come from reviewed source records, not profile arrays."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        source_clauses = {row["clause_id"]: row for row in oracle["source_clause_crosswalk"]}
        requirement_ids_by_source_gap: dict[str, set[str]] = {}
        for requirement in catalog["requirements"]:
            source_gap_id = requirement.get("source_gap_id")
            if isinstance(source_gap_id, str):
                requirement_ids_by_source_gap.setdefault(source_gap_id, set()).add(
                    requirement["requirement_id"]
                )

        baseline_attribute_requirement_ids = {
            "KMIPKIT-REQ-SPEC-4.1-001-001",
            "KMIPKIT-REQ-SPEC-4.1-001-002",
            "KMIPKIT-REQ-SPEC-4.1-001-003",
            "KMIPKIT-REQ-SPEC-4.18-001-001",
            "KMIPKIT-REQ-SPEC-4.18-001-002",
        }
        mappings_by_requirement_id: dict[str, list[dict[str, object]]] = {}
        for mapping in oracle["requirement_crosswalk"]:
            mappings_by_requirement_id.setdefault(mapping["requirement_id"], []).append(mapping)

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            source_clause_ids = set(profile["source_clause_ids"])
            included_sections = {
                (source_clauses[clause_id]["source_id"], source_clauses[clause_id]["section"])
                for clause_id in source_clause_ids
            }
            expected_requirement_ids: set[str] = set()

            for review in oracle["source_requirement_reviews"]:
                applies_to = review.get("applies_to")
                explicitly_client_applicable = any(
                    obligation.get("actor") == "client"
                    for obligation in review.get("source_obligations", [])
                )
                review_requirement_ids = set(review["expected_client_requirement_ids"])
                # Requirement-level scope is more specific than clause-level
                # profile applicability. An explicit empty mapping must not
                # be widened by an inherited source clause.
                explicitly_unassigned_ids = {
                    requirement_id
                    for requirement_id in review_requirement_ids
                    if any(
                        mapping.get("scope_state") == "profile_conditional"
                        and mapping.get("applies_to") == []
                        for mapping in mappings_by_requirement_id.get(requirement_id, [])
                    )
                }
                review_requirement_ids -= explicitly_unassigned_ids
                if (
                    (
                        applies_to == "all_client_profiles"
                        and review.get("scope_state") == "profile_conditional"
                        and explicitly_client_applicable
                    )
                    or (
                        review["source_clause_id"] in source_clause_ids
                        and _mapping_applies_to_profile(
                            {
                                "applies_to": applies_to,
                                "inherited_by_profiles": review.get("inherited_by_profiles", []),
                            },
                            profile_id,
                        )
                    )
                ):
                    expected_requirement_ids.update(review_requirement_ids)

            for mapping in oracle["requirement_crosswalk"]:
                applies_to_profile = _mapping_applies_to_profile(mapping, profile_id)
                global_client_capability = (
                    mapping.get("scope_state") == "profile_conditional"
                    and mapping.get("applies_to") == "all_client_profiles"
                    and mapping.get("role") == "client"
                )
                conditional_profile_scope = mapping.get("scope_state") == "profile_conditional"
                source_clause_applies = (
                    mapping.get("source_clause_id") in source_clause_ids
                    or (mapping.get("source_id"), mapping.get("source_section")) in included_sections
                )
                if (
                    applies_to_profile
                    and mapping.get("role") in {"client", "both"}
                    and mapping.get("scope_state") in {"client_1_0", "profile_conditional"}
                    and (
                        # Conditional capability applicability comes from the
                        # requirement-level mapping. Clause closure cannot
                        # broaden an explicit empty or profile-specific scope.
                        (conditional_profile_scope and (global_client_capability or applies_to_profile))
                        or (not conditional_profile_scope and source_clause_applies)
                    )
                ):
                    expected_requirement_ids.add(mapping["requirement_id"])

            for gap in oracle["source_only_requirement_gaps"]:
                conditional_mapping_applies_to_profile = any(
                    mapping.get("requirement_id") == gap.get("candidate_requirement_id")
                    and mapping.get("role") in {"client", "both"}
                    and mapping.get("scope_state") == "profile_conditional"
                    and _mapping_applies_to_profile(mapping, profile_id)
                    for mapping in oracle["requirement_crosswalk"]
                )
                gap_applies_to_profile = (
                    gap.get("scope_disposition") == "client_1_0"
                    or (
                        gap.get("scope_disposition") == "profile_conditional"
                        and conditional_mapping_applies_to_profile
                    )
                )
                if (
                    gap.get("candidate_requirement_id") is not None
                    and gap_applies_to_profile
                    and (
                        gap.get("source_clause_id") in source_clause_ids
                        or (gap.get("source_id"), gap.get("source_section")) in included_sections
                    )
                ):
                    expected_requirement_ids.add(gap["candidate_requirement_id"])

            source_table_requirement_ids = {
                requirement_id
                for contract in oracle.get("source_table_contracts", [])
                if contract.get("actor") in {"client", "both"}
                and contract.get("scope_disposition") in {"client_1_0", "profile_conditional"}
                and (
                    contract.get("source_clause_id") in source_clause_ids
                    or (contract.get("source_id"), contract.get("source_section")) in included_sections
                )
                for requirement_id in requirement_ids_by_source_gap.get(contract["gap_id"], set())
            }
            expected_requirement_ids.update(source_table_requirement_ids)

            if (
                profile_id == "KMIPKIT-PROFILE-BASELINE-CLIENT"
                or "KMIPKIT-PROFILE-BASELINE-CLIENT" in profile.get("dependency_profile_ids", [])
            ):
                for requirement_id in baseline_attribute_requirement_ids:
                    with self.subTest(profile_id=profile_id, requirement_id=requirement_id):
                        self.assertTrue(
                            any(
                                mapping.get("applies_to") == "all_client_profiles"
                                for mapping in mappings_by_requirement_id.get(requirement_id, [])
                            ),
                            "Baseline Client's explicitly selected Activation/Deactivation Date attributes apply through its profile dependency",
                        )
                        self.assertIn(requirement_id, expected_requirement_ids)

            explicitly_unassigned_conditional_ids = {
                requirement_id
                for requirement_id, mappings in mappings_by_requirement_id.items()
                if requirement_id not in baseline_attribute_requirement_ids
                and any(
                    mapping.get("scope_state") == "profile_conditional"
                    and mapping.get("applies_to") == []
                    for mapping in mappings
                )
            }
            self.assertFalse(
                expected_requirement_ids & explicitly_unassigned_conditional_ids,
                "source-clause membership must not override an empty requirement-level profile assignment",
            )

            expected = sorted(expected_requirement_ids)
            with self.subTest(profile_id=profile_id):
                missing_source_table_requirement_ids = sorted(
                    source_table_requirement_ids - expected_requirement_ids
                )
                self.assertEqual(
                    missing_source_table_requirement_ids,
                    [],
                    "client source-table contracts in the profile closure must contribute their existing stable labels",
                )
                self.assertEqual(
                    profile.get("requirement_ids"),
                    expected,
                    "full requirement closure must be independently enumerated from the source review fixture",
                )
                self.assertEqual(profile["requirement_ids_audited_subset"], expected)

    def test_catalog_scopes_source_reviewed_conditional_requirements_to_profiles(self) -> None:
        """Catalog profile IDs and closures match source-reviewed conditional applicability."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}
        requirements = {item["requirement_id"]: item for item in catalog["requirements"]}
        mappings = oracle["requirement_crosswalk"]

        for expected_profile in oracle["profiles"]:
            profile_id = expected_profile["profile_id"]
            source_clauses = set(expected_profile["source_clause_ids"])
            applicable_mappings = [
                mapping
                for mapping in mappings
                if mapping.get("source_clause_id") in source_clauses
                and _mapping_applies_to_profile(mapping, profile_id)
                and mapping.get("role") in {"client", "both"}
                and mapping.get("scope_state") == "profile_conditional"
            ]
            actual_profile = catalog_profiles[profile_id]

            for mapping in applicable_mappings:
                requirement_id = mapping["requirement_id"]
                requirement = requirements[requirement_id]
                with self.subTest(profile_id=profile_id, requirement_id=requirement_id):
                    self.assertIn(profile_id, requirement["profile_ids"])
                    self.assertIn(requirement_id, actual_profile["requirement_ids"])

        client_profile_ids = sorted(
            profile["profile_id"] for profile in oracle["profiles"]
        )
        explicitly_client_scoped_ids = {
            "KMIPKIT-REQ-PROF-6-001",
            "KMIPKIT-REQ-SPEC-4.1-001-001",
            "KMIPKIT-REQ-SPEC-4.1-001-002",
            "KMIPKIT-REQ-SPEC-4.1-001-003",
            "KMIPKIT-REQ-SPEC-4.18-001-001",
            "KMIPKIT-REQ-SPEC-4.18-001-002",
        }
        for requirement_id in explicitly_client_scoped_ids:
            with self.subTest(requirement_id=requirement_id):
                self.assertEqual(
                    requirements[requirement_id]["profile_ids"],
                    client_profile_ids,
                )
                for profile in catalog["profiles"]:
                    if profile["role"] != "client":
                        self.assertNotIn(requirement_id, profile["requirement_ids"])

    def test_client_profiles_audit_the_inherited_baseline_authentication_requirement(self) -> None:
        """Client profile dependencies inherit Baseline Client's §3.1 requirement mapping."""
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1-001"
        requirement_id = "KMIPKIT-REQ-PROF-3.1-001"
        basic_auth_inclusion = "KMIPKIT-INCLUSION-PROF-BASELINE-BASIC-AUTH"
        baseline_profile = "KMIPKIT-PROFILE-BASELINE-CLIENT"

        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            with self.subTest(profile_id=profile_id):
                self.assertIn(clause_id, profile["source_clause_ids"])
                self.assertIn(basic_auth_inclusion, profile["normative_inclusion_ids"])
                if profile_id != baseline_profile:
                    self.assertIn(baseline_profile, profile["dependency_profile_ids"])
                self.assertTrue(
                    requirement_id in profile["requirement_ids_audited_subset"],
                    f"missing inherited requirement {requirement_id}",
                )

    def test_oracle_audits_the_required_derive_key_table_body_row(self) -> None:
        """Specification v2.1 Table 205's required Attributes row is in every client closure."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        requirement_id = "KMIPKIT-REQ-SPEC-6.1.14-007-001"
        gap_id = "KMIPKIT-SOURCE-GAP-SPEC-6.1.14-ATTRIBUTES-REQUIRED"
        mapping = next(item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == requirement_id)
        clauses = {item["clause_id"]: item for item in oracle["source_clause_crosswalk"]}
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}

        self.assertEqual(mapping["source_gap_id"], gap_id)
        self.assertEqual(mapping["source_section"], "6.1.14")
        self.assertEqual(mapping["applies_to"], "all_client_profiles")
        self.assertEqual(mapping["scope_state"], "client_1_0")
        self.assertIsNone(mapping.get("source_clause_id"))
        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            included_spec_sections = {
                clauses[clause_id]["section"]
                for clause_id in profile["source_clause_ids"]
                if clauses[clause_id]["source_id"] == "KMIPKIT-SRC-spec"
            }
            with self.subTest(profile_id=profile_id):
                self.assertIn("6.1.14", included_spec_sections)
                self.assertTrue(
                    requirement_id in profile["requirement_ids_audited_subset"],
                    f"missing source-reviewed table row {requirement_id}",
                )
                self.assertTrue(
                    requirement_id in catalog_profiles[profile_id]["requirement_ids"],
                    f"catalog omits source-reviewed table row {requirement_id}",
                )

    def test_oracle_crosswalks_the_attestation_indicator_client_requirement(self) -> None:
        """Specification v2.1 §9.3's client SHALL has one exact source mapping."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        requirement_id = "KMIPKIT-REQ-SPEC-9.3-001-001"
        clause_id = "KMIPKIT-CLAUSE-SPEC-9.3-001"
        mapping = next(
            (item for item in oracle["requirement_crosswalk"] if item["requirement_id"] == requirement_id),
            None,
        )
        review = next(item for item in oracle["source_requirement_reviews"] if item["source_clause_id"] == clause_id)
        obligation = next(item for item in review["source_obligations"] if item["candidate_requirement_id"] == requirement_id)
        catalog_profiles = {item["profile_id"]: item for item in catalog["profiles"]}

        self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
        self.assertEqual(obligation["actor"], "client")
        self.assertEqual(obligation["scope_disposition"], "client_1_0")
        self.assertEqual(
            mapping,
            {
                "requirement_id": requirement_id,
                "source_clause_id": clause_id,
                "source_id": "KMIPKIT-SRC-spec",
                "source_section": "9.3",
                "source_locator": {"ordinal": 1, "block_kind": "paragraph"},
                "source_keyword": "SHALL",
                "role": "client",
                "scope_state": "client_1_0",
                "applies_to": "all_client_profiles",
                "review_note": "Specification v2.1 §9.3 lines 43612–43619; this crosswalk row covers the client's Attestation Capable Indicator obligation only. The separate server response obligation remains server-only.",
                "source_obligation_order": 1,
                "source_actor": "client",
            },
        )
        for profile in oracle["profiles"]:
            profile_id = profile["profile_id"]
            with self.subTest(profile_id=profile_id):
                self.assertIn(clause_id, profile["source_clause_ids"])
                self.assertTrue(
                    requirement_id in profile["requirement_ids_audited_subset"],
                    f"missing reviewed client requirement {requirement_id}",
                )
                self.assertTrue(
                    requirement_id in catalog_profiles[profile_id]["requirement_ids"],
                    f"catalog omits reviewed client requirement {requirement_id}",
                )

    def test_source_oracle_covers_all_profiles_and_expected_scope_counts(self) -> None:
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profiles = oracle["profiles"]

        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(profiles), 18)
        self.assertEqual(
            sum(profile["applicability"] in {"client_1_0", "conditional"} for profile in profiles),
            15,
        )
        self.assertEqual(
            sum(profile["applicability"] == "out_of_scope" for profile in profiles),
            3,
        )
        baseline = next(
            profile for profile in profiles
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        mandatory_available = [
            case for case in baseline["test_cases"]
            if case["fixture_status"] == "available"
        ]
        self.assertEqual(len(baseline["test_cases"]), 16)
        self.assertEqual(len(mandatory_available), 16)
        self.assertTrue(all("-M-" in case["official_case_id"] for case in baseline["test_cases"]))

    def test_checked_in_client_profile_closures_match_current_scope(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))

        violations = []
        for profile in catalog["profiles"]:
            if profile["role"] not in {"client", "both"}:
                continue
            closure = compute_profile_closure(
                catalog,
                profile["direct_source_clause_ids"],
                profile_id=profile["profile_id"],
            )
            computed_ids = set(closure.requirement_ids)
            stored_ids = set(profile["requirement_ids"])
            stale_ids = sorted(stored_ids - computed_ids)
            missing_ids = sorted(computed_ids - stored_ids)
            if (
                profile["source_clause_ids"] != list(closure.source_clause_ids)
                or stale_ids
                or missing_ids
            ):
                violations.append(
                    f"{profile['profile_id']}: stale={stale_ids[:3]}, missing={missing_ids[:3]}"
                )

        self.assertEqual(violations, [], "client profile closures differ from current catalog scope")

    def test_client_profile_closure_keeps_split_client_obligations_without_server_obligations(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        baseline = next(
            profile for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )

        closure = compute_profile_closure(
            catalog,
            baseline["direct_source_clause_ids"],
            profile_id=baseline["profile_id"],
        )

        requirements = {
            row["requirement_id"]: row for row in catalog["requirements"]
        }
        source_reviews = {
            row["source_clause_id"]: row for row in oracle["source_requirement_reviews"]
        }
        # The pinned source review assigns client requests and server responses
        # separate requirement IDs, even when they share a source clause.
        for clause_id, client_id, server_id in (
            (
                "KMIPKIT-CLAUSE-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.5-001",
                "KMIPKIT-REQ-SPEC-6.1.5-002",
            ),
            (
                "KMIPKIT-CLAUSE-SPEC-6.1.7-005",
                "KMIPKIT-REQ-SPEC-6.1.7-005",
                "KMIPKIT-REQ-SPEC-6.1.7-006",
            ),
        ):
            with self.subTest(requirement_id=client_id):
                source_review = source_reviews[clause_id]
                client_requirement = requirements[client_id]
                server_requirement = requirements[server_id]
                source_atoms = {
                    atom.get("candidate_requirement_id"): atom
                    for atom in source_review["source_obligations"]
                }

                self.assertEqual(client_requirement["role"], "client")
                self.assertEqual(client_requirement["scope_state"], "client_1_0")
                self.assertIn(client_id, source_review["expected_client_requirement_ids"])
                self.assertIn(client_id, closure.requirement_ids)
                self.assertEqual(source_atoms[client_id]["actor"], "client")
                self.assertEqual(source_atoms[client_id]["scope_disposition"], "client_1_0")

                self.assertEqual(server_requirement["role"], "server")
                self.assertEqual(server_requirement["scope_state"], "server_only")
                self.assertNotIn(server_id, source_review["expected_client_requirement_ids"])
                self.assertNotIn(server_id, closure.requirement_ids)
                self.assertEqual(source_atoms[server_id]["actor"], "server")
                self.assertEqual(source_atoms[server_id]["scope_disposition"], "server_only")

    def test_generated_client_requirement_metadata_covers_profile_closures(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)
        requirement_block = generated.split("pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[", 1)[1]
        requirement_block = requirement_block.split("\n];", 1)[0]
        metadata_ids = set(re.findall(r'requirement_id: "([^"]+)"', requirement_block))
        closed_requirement_ids = {
            requirement_id
            for profile in catalog["profiles"]
            if profile["role"] in {"client", "both"}
            for requirement_id in profile["requirement_ids"]
        }

        missing_metadata = sorted(closed_requirement_ids - metadata_ids)
        self.assertEqual(
            missing_metadata,
            [],
            "every requirement in a client profile closure needs generated runtime metadata",
        )

    def test_profile_closure_generator_matches_the_pinned_source_oracle(self) -> None:
        """Compare closure expansion with static source-derived arrays for all 18 clients."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        oracle_profiles = {profile["profile_id"]: profile for profile in oracle["profiles"]}
        catalog_profiles = {
            profile["profile_id"]: profile
            for profile in catalog["profiles"]
            if profile["role"] == "client"
        }

        self.assertEqual(
            oracle["source_closure_status"],
            "complete",
            "profile closure comparison requires the completed pinned-source oracle",
        )
        self.assertIs(
            oracle["counts"]["requirement_crosswalk_complete"],
            False,
            "source-accounted exclusions remain distinct from accepted direct mappings",
        )
        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(oracle_profiles), 18)
        profiles_without_full_requirements = sorted(
            profile_id
            for profile_id, expected in oracle_profiles.items()
            if "requirement_ids" not in expected
        )
        self.assertEqual(
            profiles_without_full_requirements,
            [],
            "every profile must carry a complete source-derived requirement set",
        )

        self.assertEqual(set(catalog_profiles), set(oracle_profiles))
        for profile_id, expected in oracle_profiles.items():
            with self.subTest(profile_id=profile_id):
                profile = catalog_profiles[profile_id]
                self.assertEqual(
                    profile["direct_source_clause_ids"],
                    expected["direct_source_clause_ids"],
                    f"{profile_id} direct source roots differ from pinned Profiles section headings",
                )
                closure = compute_profile_closure(
                    catalog,
                    profile["direct_source_clause_ids"],
                    profile_id=profile_id,
                )
                self.assertEqual(
                    closure.source_clause_ids,
                    tuple(expected["source_clause_ids"]),
                    f"{profile_id} clause closure differs from the pinned-source oracle",
                )
                self.assertEqual(
                    set(closure.requirement_ids),
                    set(expected["requirement_ids"]),
                    f"{profile_id} generated requirement closure differs from the complete pinned-source oracle",
                )

    def test_closure_expands_whole_sources_sections_and_applicable_requirements(self) -> None:
        catalog = closure_catalog()

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertEqual(
            closure.source_clause_ids,
            (
                "KMIPKIT-CLAUSE-PROF-3.1-001",
                "KMIPKIT-CLAUSE-PROF-3.1.1-001",
                "KMIPKIT-CLAUSE-PROF-6.1-001",
                "KMIPKIT-CLAUSE-PROF-6.1.1-001",
                "KMIPKIT-CLAUSE-SPEC-1-001",
                "KMIPKIT-CLAUSE-SPEC-1.1-001",
                "KMIPKIT-CLAUSE-SPEC-2-001",
            ),
        )
        self.assertEqual(
            closure.requirement_ids,
            (
                "KMIPKIT-REQ-PROF-3.1.1-001",
                "KMIPKIT-REQ-SPEC-1-001",
                "KMIPKIT-REQ-SPEC-1.1-001",
            ),
        )

    def test_table_body_requirements_follow_included_section_boundaries(self) -> None:
        catalog = closure_catalog()
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1.1-TABLE-OPTIONAL",
                "source_clause_ids": [],
                "source_refs": [{"source_id": "spec", "section": "1.1"}],
                "source_force": "optional_table_body_value",
                "role": "client",
                "scope_state": "client_1_0",
            }
        )
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1-TABLE-OPTIONAL",
                "source_clause_ids": [],
                "source_refs": [{"source_id": "spec", "section": "1"}],
                "source_force": "optional_table_body_value",
                "role": "client",
                "scope_state": "client_1_0",
            }
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertIn("KMIPKIT-REQ-SPEC-1.1-TABLE-OPTIONAL", closure.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-1-TABLE-OPTIONAL", closure.requirement_ids)

    def test_closure_does_not_expand_informative_references(self) -> None:
        catalog = closure_catalog()
        catalog["normative_inclusions"] = catalog["normative_inclusions"][:1]
        catalog["informative_references"] = [
            {
                "source_clause_id": "KMIPKIT-CLAUSE-SPEC-1-001",
                "included_source_id": "profiles",
                "included_section": "3.1",
            }
        ]

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertIn("KMIPKIT-CLAUSE-SPEC-1-001", closure.source_clause_ids)
        self.assertNotIn("KMIPKIT-CLAUSE-PROF-3.1-001", closure.source_clause_ids)

    def test_conditional_requirement_applies_only_to_profiles_linked_by_the_catalog(self) -> None:
        catalog = closure_catalog()

        baseline = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )
        unrelated = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-UNRELATED-CLIENT",
        )

        self.assertIn("KMIPKIT-REQ-PROF-3.1.1-001", baseline.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-PROF-3.1.1-001", unrelated.requirement_ids)

    def test_explicit_profile_membership_includes_requirement_outside_profile_clause_closure(self) -> None:
        catalog = closure_catalog()
        catalog["source_clauses"].append(
            {"clause_id": "KMIPKIT-CLAUSE-PROF-4.1.2-007", "source_id": "profiles", "section": "4.1.2"}
        )
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-PROF-4.1.2-007",
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-4.1.2-007"],
                "role": "client",
                "scope_state": "profile_conditional",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            }
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertIn("KMIPKIT-REQ-PROF-4.1.2-007", closure.requirement_ids)

    def test_closure_reports_unresolved_client_scope_without_applying_it(self) -> None:
        catalog = closure_catalog()
        catalog["requirements"].extend(
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-1.1-UNCLEAR",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
                    "role": "client",
                    "scope_state": "unclear",
                },
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-2-CONDITIONAL",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-2-001"],
                    "role": "both",
                    "scope_state": "profile_conditional",
                    "profile_ids": [],
                },
            ]
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertNotIn("KMIPKIT-REQ-SPEC-1.1-UNCLEAR", closure.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-2-CONDITIONAL", closure.requirement_ids)
        blockers = getattr(closure, "unresolved_scope_obligations", ())
        self.assertEqual(
            [
                (item.requirement_id, item.source_clause_id, item.scope_state)
                for item in blockers
            ],
            [
                ("KMIPKIT-REQ-SPEC-1.1-UNCLEAR", "KMIPKIT-CLAUSE-SPEC-1.1-001", "unclear"),
                (
                    "KMIPKIT-REQ-SPEC-2-CONDITIONAL",
                    "KMIPKIT-CLAUSE-SPEC-2-001",
                    "profile_conditional",
                ),
            ],
        )

    def test_closure_fails_closed_for_unresolved_endpoint_attribution(self) -> None:
        catalog = closure_catalog()
        catalog["source_clauses"].append(
            {
                "clause_id": "KMIPKIT-CLAUSE-PROF-4.1.2-001",
                "source_id": "profiles",
                "section": "4.1.2",
            }
        )
        catalog["requirements"].append(
            {
                "requirement_id": "KMIPKIT-REQ-PROF-4.1.2-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-4.1.2-001"],
                "role": "unclear",
                "direction": "unclear",
                "scope_state": "unclear",
            }
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-4.1.2-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertNotIn("KMIPKIT-REQ-PROF-4.1.2-001", closure.requirement_ids)
        self.assertEqual(
            [
                (item.requirement_id, item.source_clause_id, item.scope_state)
                for item in closure.unresolved_scope_obligations
            ],
            [
                (
                    "KMIPKIT-REQ-PROF-4.1.2-001",
                    "KMIPKIT-CLAUSE-PROF-4.1.2-001",
                    "unclear",
                )
            ],
        )

    def test_closure_excludes_server_only_obligations_from_client_scope(self) -> None:
        catalog = closure_catalog()
        catalog["requirements"].extend(
            [
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-1-SERVER-ROLE",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                    "role": "server",
                    "scope_state": "server_only",
                },
                {
                    "requirement_id": "KMIPKIT-REQ-SPEC-1-SERVER-SCOPE",
                    "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                    "role": "client",
                    "scope_state": "server_only",
                },
            ]
        )

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertNotIn("KMIPKIT-REQ-SPEC-1-SERVER-ROLE", closure.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-SPEC-1-SERVER-SCOPE", closure.requirement_ids)
        self.assertEqual(closure.unresolved_scope_obligations, ())

    def test_closure_rejects_recursive_normative_inclusion_cycles(self) -> None:
        catalog = closure_catalog()
        inclusions = catalog["normative_inclusions"]
        assert isinstance(inclusions, list)
        inclusions.append(
            {
                "inclusion_id": "KMIPKIT-INCLUSION-CYCLE",
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001",
                "included_source_id": "profiles",
                "included_section": "6.1",
            }
        )

        with self.assertRaisesRegex(ProfileGenerationError, "cyclic normative inclusion"):
            compute_profile_closure(
                catalog,
                ["KMIPKIT-CLAUSE-PROF-6.1-001"],
                profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
            )

    def test_profile_generator_rejects_invalid_and_over_limit_catalog_inputs(self) -> None:
        for label, raw_catalog in (
            ("invalid JSON", b"{"),
            ("over-limit input", b"x" * (MAX_CATALOG_BYTES + 1)),
        ):
            with self.subTest(label=label):
                reads: list[tuple[Path, str, int]] = []

                def read_catalog(repository: Path, path: str, *, max_bytes: int) -> bytes:
                    reads.append((repository, path, max_bytes))
                    if path != CATALOG_PATH:
                        raise AssertionError(f"generator read {path} after catalog validation should have failed")
                    return raw_catalog

                stderr = io.StringIO()
                with (
                    patch("tools.normative_catalog.generate_profiles.safe_read_bytes", side_effect=read_catalog),
                    patch("sys.stderr", new=stderr),
                ):
                    result = generate_profiles_main(["--check", "--repo-root", str(ROOT)])

                self.assertEqual(result, 1)
                self.assertIn("profile metadata generation failed:", stderr.getvalue())
                self.assertEqual(reads, [(ROOT.resolve(), CATALOG_PATH, MAX_CATALOG_BYTES)])

    def test_rust_string_literal_escapes_syntax_and_control_characters(self) -> None:
        value = 'quote" slash\\ line\n tab\t control\x01 snowman ☃'

        self.assertEqual(
            rust_string_literal(value),
            '"quote\\" slash\\\\ line\\n tab\\t control\\u{1} snowman ☃"',
        )


    @unittest.skipUnless(shutil.which("rustc") and shutil.which("rustfmt"), "Rust toolchain is required")
    def test_generated_source_compiles_and_adversarial_metadata_round_trips(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        baseline = next(
            profile
            for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        value = 'quote" slash\\ line\n carriage\r tab\t nul\x00 control\x01 DEL\x7f C1\x85 snowman ☃ emoji 🦄'
        baseline["name"] = value
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        generated = render_profiles_source(catalog, targets)
        expected_bytes = ", ".join(str(byte) for byte in value.encode("utf-8"))
        generated += "\n".join([
            "#[cfg(test)]",
            "mod generated_metadata_literal_test {",
            "    use super::CLIENT_PROFILES;",
            "",
            "    #[test]",
            "    fn source_metadata_is_data_after_compilation() {",
            "        let baseline = CLIENT_PROFILES",
            "            .iter()",
            "            .find(|profile| profile.profile_id == \"KMIPKIT-PROFILE-BASELINE-CLIENT\")",
            "            .expect(\"the fixture catalog includes Baseline Client\");",
            f"        assert_eq!(baseline.name.as_bytes(), &[{expected_bytes}]);",
            "    }",
            "}",
            "",
        ])
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "profiles_generated.rs"
            binary = Path(directory) / "profiles_generated_tests.exe"
            source.write_text(generated, encoding="utf-8")
            compile_result = subprocess.run(
                ["rustc", "--edition=2024", "--test", str(source), "-o", str(binary)],
                cwd=directory,
                capture_output=True,
                check=False,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(compile_result.returncode, 0, compile_result.stderr)
            run_result = subprocess.run(
                [str(binary)],
                cwd=directory,
                capture_output=True,
                check=False,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(run_result.returncode, 0, run_result.stdout + run_result.stderr)


class ProfileTargetManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = {
            "profiles": [
                {
                    "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                    "role": "client",
                    "applicability": "client_1_0",
                },
                {
                    "profile_id": "KMIPKIT-PROFILE-SERVER-ONLY",
                    "role": "server",
                    "applicability": "server_only",
                },
            ]
        }
        self.manifest = {
            "release_line": "1.0",
            "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            "decision_ref": "KMIPKIT-0010/FR-003",
        }

    def parse(self, manifest: dict[str, object] | bytes | str):
        if isinstance(manifest, dict):
            raw = json.dumps(manifest).encode("utf-8")
        elif isinstance(manifest, str):
            raw = manifest.encode("utf-8")
        else:
            raw = manifest
        return parse_profile_targets(raw, self.catalog, ROOT)

    def test_loads_fixed_baseline_target_and_keeps_selection_separate(self) -> None:
        targets = self.parse(self.manifest)

        self.assertEqual(targets.release_line, "1.0")
        self.assertEqual(targets.profile_ids, ("KMIPKIT-PROFILE-BASELINE-CLIENT",))
        self.assertEqual(targets.decision_ref, "KMIPKIT-0010/FR-003")

    def test_rejects_duplicate_keys_and_invalid_utf8(self) -> None:
        duplicate_keys = (
            b'{"release_line":"1.0","release_line":"1.0",'
            b'"profile_ids":["KMIPKIT-PROFILE-BASELINE-CLIENT"],'
            b'"decision_ref":"KMIPKIT-0010/FR-003"}'
        )
        with self.assertRaises(ProfileGenerationError):
            self.parse(duplicate_keys)
        with self.assertRaises(ProfileGenerationError):
            self.parse(b"\xff")

    def test_rejects_wrong_release_unknown_profile_and_unapproved_role(self) -> None:
        for overrides in (
            {"release_line": "2.0"},
            {"profile_ids": ["KMIPKIT-PROFILE-UNKNOWN"]},
            {"profile_ids": ["KMIPKIT-PROFILE-SERVER-ONLY"]},
        ):
            with self.subTest(overrides=overrides):
                with self.assertRaises(ProfileGenerationError):
                    self.parse(self.manifest | overrides)

    def test_rejects_empty_duplicate_and_unresolved_decision_targets(self) -> None:
        for overrides in (
            {"profile_ids": []},
            {"profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"] * 2},
            {"decision_ref": "KMIPKIT-0010/FR-999"},
            {"extra": "unknown"},
        ):
            with self.subTest(overrides=overrides):
                with self.assertRaises(ProfileGenerationError):
                    self.parse(self.manifest | overrides)

    def test_rejects_missing_fields_wrong_types_empty_ids_and_invalid_json(self) -> None:
        invalid_documents = (
            {},
            {"release_line": "1.0", "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"]},
            self.manifest | {"release_line": 1},
            self.manifest | {"profile_ids": "KMIPKIT-PROFILE-BASELINE-CLIENT"},
            self.manifest | {"profile_ids": [None]},
            self.manifest | {"profile_ids": [""]},
            self.manifest | {"decision_ref": []},
            self.manifest | {"decision_ref": ""},
        )
        for document in invalid_documents:
            with self.subTest(document=document), self.assertRaises(ProfileGenerationError):
                self.parse(document)

        with self.assertRaises(ProfileGenerationError):
            self.parse(b'{"release_line":')
        oversized = json.dumps(self.manifest).encode("utf-8")
        oversized += b" " * (65_537 - len(oversized))
        with self.assertRaisesRegex(ProfileGenerationError, "size limit"):
            self.parse(oversized)

    def test_fixed_loader_rejects_a_symlinked_target_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            target = repository / "specification" / "catalog" / "profile-targets.json"
            target.parent.mkdir(parents=True)
            external = repository / "external-targets.json"
            external.write_text(json.dumps(self.manifest), encoding="utf-8")
            try:
                target.symlink_to(external)
            except OSError as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            with self.assertRaises(ProfileGenerationError):
                load_profile_targets(repository, self.catalog)

    def test_fixed_loader_rejects_a_junctioned_target_manifest_directory(self) -> None:
        if not shutil.which("cmd.exe"):
            self.skipTest("Windows cmd.exe is required to create a directory junction")

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory) / "repository"
            specification = repository / "specification"
            specification.mkdir(parents=True)
            external = Path(directory) / "external-catalog"
            external.mkdir()
            (external / "profile-targets.json").write_text(
                json.dumps(self.manifest), encoding="utf-8"
            )
            junction = specification / "catalog"
            mklink_command = (
                "mklink /J "
                + subprocess.list2cmdline([str(junction), str(external)])
            )
            result = subprocess.run(
                ["cmd.exe", "/d", "/c", mklink_command],
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(
                result.returncode,
                0,
                msg=f"could not create test junction: {result.stdout}{result.stderr}",
            )

            with self.assertRaises(ProfileGenerationError):
                load_profile_targets(repository, self.catalog)

    def test_fixed_loader_uses_a_64_kib_read_bound(self) -> None:
        with patch(
            "tools.normative_catalog.generate_profiles.safe_read_bytes",
            side_effect=lambda _root, path, *, max_bytes: (
                json.dumps(self.manifest).encode("utf-8")
                if path == "specification/catalog/profile-targets.json"
                else b"**FR-003**"
            ),
        ) as safe_read:
            load_profile_targets(ROOT, self.catalog)

        self.assertIn(
            call(ROOT, "specification/catalog/profile-targets.json", max_bytes=65_536),
            safe_read.call_args_list,
        )

    def test_generated_metadata_preserves_closure_evidence_and_separate_target_state(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)

        self.assertEqual(generated, render_profiles_source(catalog, targets))
        self.assertEqual(generated.count("\n    GeneratedClientProfile {"), 18)
        self.assertIn('claim_state: "not_claimed"', generated)
        self.assertIn("selected_target: true", generated)
        self.assertIn('fixture_availability: "unavailable"', generated)
        self.assertNotIn('"KMIPKIT-REQ-SPEC-2.1-001"', generated)
        self.assertNotIn('"KMIPKIT-REQ-PROF-3.1.1-004"', generated)
        self.assertNotIn("ProfileRuleMapping", generated)

    def test_target_and_profile_prose_do_not_change_requirement_evidence_metadata(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        baseline_target = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        alternate_target = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-AES-XTS-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        baseline_output = render_profiles_source(catalog, baseline_target)
        alternate_target_output = render_profiles_source(catalog, alternate_target)
        changed_prose = json.loads(json.dumps(catalog))
        baseline = next(
            profile
            for profile in changed_prose["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        baseline["name"] = "This prose does not define executable profile rules"
        changed_prose_output = render_profiles_source(changed_prose, baseline_target)

        def requirement_metadata(source: str) -> str:
            return source.split("pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[", 1)[1].split(
                "pub const CLIENT_PROFILES:", 1
            )[0]

        self.assertEqual(requirement_metadata(baseline_output), requirement_metadata(alternate_target_output))
        self.assertEqual(requirement_metadata(baseline_output), requirement_metadata(changed_prose_output))
        self.assertIn("selected_target: true", baseline_output)
        self.assertIn("selected_target: true", alternate_target_output)
        self.assertNotIn("ProfileRuleMapping", baseline_output)
        self.assertNotIn("ProfileRuleMapping", alternate_target_output)

    def test_source_gap_locator_rejects_missing_or_malformed_catalog_location(self) -> None:
        from tools.normative_catalog.generate_profiles import _source_gap_locator

        with self.assertRaises(ProfileGenerationError):
            _source_gap_locator({"source_gap_id": "KMIPKIT-SOURCE-GAP-TEST"})

        with self.assertRaises(ProfileGenerationError):
            _source_gap_locator(
                {
                    "source_gap_id": "KMIPKIT-SOURCE-GAP-TEST",
                    "source_locator": {
                        "section": "2.3",
                        "row_label": "Opaque Data Type",
                        "html_line_range": [0, 10],
                    },
                }
            )

    def test_generated_requirement_metadata_preserves_exact_normative_keywords(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)

        self.assertTrue("pub struct GeneratedClientRequirement" in generated)
        self.assertTrue("pub const CLIENT_REQUIREMENTS" in generated)
        expected_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        for keyword in expected_keywords:
            with self.subTest(keyword=keyword):
                self.assertTrue(f'source_keyword: Some("{keyword}")' in generated)

        table_requirements = [
            requirement
            for requirement in catalog["requirements"]
            if requirement.get("source_force") in {"required_table_body_value", "optional_table_body_value"}
            and requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
        ]
        for requirement in table_requirements:
            marker = f'requirement_id: "{requirement["requirement_id"]}"'
            row_start = generated.index(marker)
            row_end = generated.index("    },", row_start)
            row = generated[row_start:row_end]
            with self.subTest(requirement_id=requirement["requirement_id"]):
                self.assertIn("source_keyword: None", row)
                self.assertIn(f'source_force: Some("{requirement["source_force"]}")', row)
                self.assertIn(f'source_gap_id: Some("{requirement["source_gap_id"]}")', row)
                locator = requirement["source_locator"]
                line_start, line_end = locator["html_line_range"]
                table = f', table {locator["table"]}' if locator.get("table") else ""
                expected_locator = (
                    f'Section {locator["section"]}{table}, '
                    f'row {locator["row_label"]}, HTML lines {line_start}-{line_end}'
                )
                self.assertIn("source_locator: Some(", row)
                self.assertIn(expected_locator, row)
        self.assertNotIn("ProfileRuleMapping", generated)

    def test_generated_requirements_classify_keyword_specific_verification_categories(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        catalog = json.loads(json.dumps(catalog))
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        expected_categories = {
            "MUST": "RequiredBehavior",
            "SHALL": "RequiredBehavior",
            "REQUIRED": "RequiredBehavior",
            "MUST NOT": "ProhibitedBehavior",
            "SHALL NOT": "ProhibitedBehavior",
            "SHOULD": "RecommendedBehaviorOrDeviation",
            "RECOMMENDED": "RecommendedBehaviorOrDeviation",
            "SHOULD NOT": "DiscouragedAbsenceOrDeviation",
            "MAY": "OptionalCapability",
            "OPTIONAL": "OptionalCapability",
        }
        observed_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        template = next(
            requirement
            for requirement in catalog["requirements"]
            if requirement["role"] == "client" and requirement["scope_state"] == "client_1_0"
        )
        for index, keyword in enumerate(sorted(set(expected_categories) - observed_keywords), start=1):
            synthetic_requirement = template.copy()
            synthetic_requirement["requirement_id"] = f"KMIPKIT-REQ-SPEC-TEST-KEYWORD-{index:02}"
            synthetic_requirement["source_keyword"] = keyword
            catalog["requirements"].append(synthetic_requirement)

        generated = render_profiles_source(catalog, targets)
        blocks = re.findall(
            r"    GeneratedClientRequirement \{\n(.*?)\n    \},",
            generated,
            flags=re.DOTALL,
        )
        actual_categories: dict[str, str] = {}
        for block in blocks:
            requirement_id = re.search(r'requirement_id: "([^"]+)"', block)
            category = re.search(
                r"verification_category: GeneratedVerificationCategory::([A-Za-z]+)",
                block,
            )
            if requirement_id is not None and category is not None:
                actual_categories[requirement_id.group(1)] = category.group(1)

        scoped_requirements = {
            requirement["requirement_id"]: expected_categories[requirement["source_keyword"]]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        observed_scoped_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
            and isinstance(requirement.get("source_keyword"), str)
        }
        actual_categories = {
            requirement_id: category
            for requirement_id, category in actual_categories.items()
            if requirement_id in scoped_requirements
        }

        self.assertEqual(observed_scoped_keywords, set(expected_categories))
        self.assertEqual(actual_categories, scoped_requirements)
        self.assertEqual(
            set(actual_categories.values()),
            set(expected_categories.values()),
        )


if __name__ == "__main__":
    unittest.main()
