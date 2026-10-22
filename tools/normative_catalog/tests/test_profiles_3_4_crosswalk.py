"""Source-scope regression tests for Profiles v2.1 sections 3 through 4.1.2."""

from __future__ import annotations

import json
import unittest
from html.parser import HTMLParser
from pathlib import Path

from tools.normative_catalog.audit_sources import _decode_document


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification/catalog/kmip-2.1.json"
FIXTURE_PATH = ROOT / "tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json"
SOURCE_PATH = "specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html"


class _TextParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        self.parts.append(data)

    def normalized_text(self) -> str:
        return " ".join("".join(self.parts).split())


def pinned_excerpt(line_range: list[int]) -> str:
    lines = _decode_document((ROOT / SOURCE_PATH).read_bytes()).splitlines()
    parser = _TextParser()
    parser.feed("\n".join(lines[line_range[0] - 1 : line_range[1]]))
    parser.close()
    return parser.normalized_text()


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


CLIENT_IDS = {
    "KMIPKIT-REQ-PROF-3.1-001",
    "KMIPKIT-REQ-PROF-3.1.1-002",
    "KMIPKIT-REQ-PROF-3.1.1-005",
    "KMIPKIT-REQ-PROF-3.1.2-002",
    "KMIPKIT-REQ-PROF-3.1.2-005",
    "KMIPKIT-REQ-PROF-3.2.1-001",
    "KMIPKIT-REQ-PROF-3.2.2-001",
    "KMIPKIT-REQ-PROF-3.2.3-001",
    "KMIPKIT-REQ-PROF-3.2.4-002",
    "KMIPKIT-REQ-PROF-3.2-001",
    "KMIPKIT-REQ-PROF-4.1.2-001",
    "KMIPKIT-REQ-PROF-4.1.2-007",
}
UNRESOLVED_IDS = {
    "KMIPKIT-REQ-PROF-3-002",
    "KMIPKIT-REQ-PROF-3-003",
    "KMIPKIT-REQ-PROF-3-004",
    "KMIPKIT-REQ-PROF-3-006",
    "KMIPKIT-REQ-PROF-4.1.2-005",
}
OUT_OF_SCOPE_IDS = {
    "KMIPKIT-REQ-PROF-3.1.1-004",
    "KMIPKIT-REQ-PROF-4.1-001",
    "KMIPKIT-REQ-PROF-4.1-003",
    "KMIPKIT-REQ-PROF-4.1.2-008",
    "KMIPKIT-REQ-PROF-4.1.2-009",
}
ALL_TARGET_IDS = CLIENT_IDS | UNRESOLVED_IDS | OUT_OF_SCOPE_IDS
ADDITIONAL_BOUNDARY_REVIEW_IDS = {"KMIPKIT-CLAUSE-PROF-3.1.2-004"}


class ProfileSectionsThreeAndFourCrosswalkTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = load_json(CATALOG_PATH)
        self.fixture = load_json(FIXTURE_PATH)
        self.requirements = {
            row["requirement_id"]: row for row in self.catalog["requirements"]
        }
        self.clauses = {row["clause_id"]: row for row in self.catalog["source_clauses"]}
        profile_reviews = {
            row["source_clause_id"]: row
            for row in self.fixture["source_requirement_reviews"]
            if row["source_id"] == "KMIPKIT-SRC-profiles"
        }
        profile_crosswalk = {
            row["requirement_id"]: row
            for row in self.fixture["requirement_crosswalk"]
            if row["source_id"] == "KMIPKIT-SRC-profiles"
        }
        self.reviews = {
            clause_id: row
            for clause_id, row in profile_reviews.items()
            if row["source_section"] == "3"
            or row["source_section"].startswith("3.")
            or row["source_section"] == "4"
            or (
                row["source_section"].startswith("4.")
                and int(row["source_section"].split(".")[1]) <= 1
            )
        }
        self.crosswalk = {
            requirement_id: row
            for requirement_id, row in profile_crosswalk.items()
            if row["source_section"] == "3"
            or row["source_section"].startswith("3.")
            or row["source_section"] == "4"
            or (
                row["source_section"].startswith("4.")
                and int(row["source_section"].split(".")[1]) <= 1
            )
        }

    def assert_review_matches_pinned_source(
        self, review: dict[str, object], clause_id: str, line_range: list[int]
    ) -> None:
        self.assertEqual(review["html_line_range"], line_range)
        self.assertEqual(review["source_excerpt"], pinned_excerpt(line_range))
        self.assertEqual(review["source_locator"], self.clauses[clause_id]["locator"])

    def test_target_ids_have_audited_source_rows_and_only_client_mappings(self) -> None:
        self.assertEqual(len(ALL_TARGET_IDS), 22)
        self.assertTrue(ALL_TARGET_IDS.issubset(self.requirements))
        target_clause_ids = {
            self.requirements[requirement_id]["source_clause_ids"][0]
            for requirement_id in ALL_TARGET_IDS
        }
        self.assertTrue(target_clause_ids.issubset(self.reviews))
        self.assertEqual(set(self.crosswalk), CLIENT_IDS)
        self.assertEqual(
            set(self.reviews), target_clause_ids | ADDITIONAL_BOUNDARY_REVIEW_IDS
        )

    def test_unresolved_and_out_of_scope_rows_are_not_client_obligations(self) -> None:
        self.assertEqual(len(UNRESOLVED_IDS), 5)
        self.assertEqual(len(OUT_OF_SCOPE_IDS), 5)
        self.assertFalse((UNRESOLVED_IDS | OUT_OF_SCOPE_IDS) & self.crosswalk.keys())
        reviewed_clause_ids = {
            self.requirements[requirement_id]["source_clause_ids"][0]
            for requirement_id in UNRESOLVED_IDS | OUT_OF_SCOPE_IDS
        }
        self.assertTrue(reviewed_clause_ids.issubset(self.reviews))
        for requirement_id in UNRESOLVED_IDS:
            requirement = self.requirements[requirement_id]
            review = self.reviews[requirement["source_clause_ids"][0]]
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertEqual(review["source_obligations"][0]["actor"], "unresolved")
            self.assertEqual(review["source_obligations"][0]["scope_disposition"], "unresolved")
        for requirement_id in OUT_OF_SCOPE_IDS:
            requirement = self.requirements[requirement_id]
            review = self.reviews[requirement["source_clause_ids"][0]]
            self.assertEqual(review["expected_client_requirement_ids"], [])
            self.assertEqual(review["source_obligations"][0]["scope_disposition"], "out_of_scope")

    def test_client_crosswalk_rows_preserve_profile_membership_and_source_actors(self) -> None:
        https_ids = {
            "KMIPKIT-REQ-PROF-3.2-001",
            "KMIPKIT-REQ-PROF-3.2.1-001",
            "KMIPKIT-REQ-PROF-3.2.2-001",
            "KMIPKIT-REQ-PROF-3.2.3-001",
            "KMIPKIT-REQ-PROF-3.2.4-002",
        }
        baseline_ids = {
            "KMIPKIT-REQ-PROF-3.1-001",
            "KMIPKIT-REQ-PROF-3.1.1-002",
            "KMIPKIT-REQ-PROF-3.1.1-005",
            "KMIPKIT-REQ-PROF-3.1.2-002",
            "KMIPKIT-REQ-PROF-3.1.2-005",
        }
        for requirement_id in CLIENT_IDS:
            requirement = self.requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            self.assertIn(clause_id, self.reviews)
            review = self.reviews[clause_id]
            row = self.crosswalk[requirement_id]
            expected_applies_to = (
                "KMIPKIT-PROFILE-HTTPS-CLIENT"
                if requirement_id in https_ids
                else "KMIPKIT-PROFILE-BASELINE-CLIENT"
                if requirement_id in baseline_ids
                else "all_client_profiles"
            )
            self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
            expected_actor = (
                "unresolved"
                if requirement_id == "KMIPKIT-REQ-PROF-4.1.2-001"
                else
                "client and server"
                if requirement_id
                in {
                    "KMIPKIT-REQ-PROF-3.1.1-005",
                    "KMIPKIT-REQ-PROF-3.1.2-005",
                    "KMIPKIT-REQ-PROF-3.2.1-001",
                    "KMIPKIT-REQ-PROF-3.2.2-001",
                    "KMIPKIT-REQ-PROF-3.2.3-001",
                }
                else "client"
            )
            self.assertEqual(review["source_obligations"][0]["actor"], expected_actor)
            self.assertEqual(row["source_actor"], expected_actor)
            self.assertEqual(row["applies_to"], expected_applies_to)

    def test_single_item_batch_id_requirement_lists_all_client_profiles(self) -> None:
        """The reviewed §4.1.2 MAY applies across client profiles, not only Baseline."""
        requirement = self.requirements["KMIPKIT-REQ-PROF-4.1.2-007"]
        all_client_profile_ids = sorted(
            profile["profile_id"]
            for profile in self.catalog["profiles"]
            if profile["role"] == "client"
        )

        self.assertEqual(requirement["profile_ids"], all_client_profile_ids)
        self.assertEqual(self.crosswalk[requirement["requirement_id"]]["applies_to"], "all_client_profiles")
        for profile_id in all_client_profile_ids:
            profile = next(row for row in self.catalog["profiles"] if row["profile_id"] == profile_id)
            self.assertIn(requirement["requirement_id"], profile["requirement_ids"], profile_id)

    def test_https_profile_closure_includes_all_explicit_client_requirements(self) -> None:
        """Sections 3.2 and 3.2.1–3.2.4 assign conditions to the HTTPS profile."""
        https_profile_id = "KMIPKIT-PROFILE-HTTPS-CLIENT"
        https_profile = next(
            row for row in self.catalog["profiles"] if row["profile_id"] == https_profile_id
        )
        expected_ids = {
            "KMIPKIT-REQ-PROF-3.2-001",
            "KMIPKIT-REQ-PROF-3.2.1-001",
            "KMIPKIT-REQ-PROF-3.2.2-001",
            "KMIPKIT-REQ-PROF-3.2.3-001",
            "KMIPKIT-REQ-PROF-3.2.4-002",
        }

        for requirement_id in expected_ids:
            requirement = self.requirements[requirement_id]
            self.assertIn(https_profile_id, requirement["profile_ids"])
            if requirement_id == "KMIPKIT-REQ-PROF-3.2-001":
                self.assertNotIn(
                    "KMIPKIT-PROFILE-BASELINE-CLIENT", requirement["profile_ids"]
                )
            else:
                self.assertEqual(requirement["profile_ids"], [https_profile_id])
            self.assertIn(requirement_id, https_profile["requirement_ids"])

    def test_https_authentication_suite_clause_is_client_applicable_without_direction(self) -> None:
        """Sections 3.2 and 6.4 establish client scope, while §3.2 leaves direction unstated."""
        requirement_id = "KMIPKIT-REQ-PROF-3.2-001"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.2-001"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]

        self.assertEqual(requirement["direction"], "unclear")
        self.assertTrue(
            {
                "KMIPKIT-PROFILE-HTTPS-CLIENT",
                "KMIPKIT-PROFILE-HTTPS-SERVER",
                "KMIPKIT-PROFILE-COMPLETE-SERVER",
            }.issubset(requirement["profile_ids"])
        )
        self.assertNotIn("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement["profile_ids"])
        self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
        self.assertEqual(review["source_obligations"][0]["actor"], "client")
        self.assertEqual(review["source_obligations"][0]["direction"], "unresolved")
        self.assertEqual(review["html_line_range"], [4227, 4229])
        self.assertIn(
            "SHALL support the HTTPS Authentication Suite conditions (3.2)",
            pinned_excerpt([10532, 10534]),
        )
        self.assertIn(
            "SHALL support the HTTPS Authentication Suite conditions (3.2)",
            pinned_excerpt([10550, 10552]),
        )

    def test_basic_authentication_suite_requirement_is_in_baseline_client_closure(self) -> None:
        """Sections 3.1 and 6.1 make the conditional TLS requirement part of Baseline Client."""
        requirement_id = "KMIPKIT-REQ-PROF-3.1-001"
        baseline_profile_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        requirement = self.requirements[requirement_id]
        profile = next(
            row for row in self.catalog["profiles"] if row["profile_id"] == baseline_profile_id
        )

        self.assertIn(baseline_profile_id, requirement["profile_ids"])
        self.assertIn(requirement_id, profile["requirement_ids"])
        self.assertEqual(self.crosswalk[requirement_id]["applies_to"], baseline_profile_id)
        self.assertNotIn(requirement_id, UNRESOLVED_IDS)

    def test_baseline_tls13_cipher_suite_prohibition_has_negative_wire_evidence(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        requirement_id = "KMIPKIT-REQ-PROF-3.1.2-005"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1.2-005"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]
        allowed_suites = pinned_excerpt([4068, 4074])
        prohibited_suites = pinned_excerpt([4187, 4188])

        self.assertIn("TLS13-CHACHA20-POLY1305-SHA256", allowed_suites)
        self.assertIn("TLS13-AES-256-GCM-SHA384", allowed_suites)
        self.assertIn("SHALL NOT support any cipher suite not listed above", prohibited_suites)
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "SHALL NOT")
        self.assertEqual(
            requirement["implementation_refs"],
            [
                "crates/kmipkit-protocol/src/profile.rs::ProfileTransport::satisfies_tls13_cipher_suite_policy",
                "crates/kmipkit-protocol/src/profile.rs::ProfileValidationResult::validate_transport",
                "crates/kmipkit-protocol/src/profile.rs::TYPED_RULE_MAPPINGS",
                "crates/kmipkit-transport/src/tls.rs::aws_lc_crypto_provider",
            ],
        )
        profile_test_id = (
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
            "profile_maps_prohibited_tls13_cipher_suites"
        )
        wire_test_id = (
            "crates/kmipkit-transport/tests/raw_tls.rs::"
            "raw_tls_rejects_tls12_server_hello_before_request_dispatch"
        )
        self.assertEqual(
            requirement["verification_refs"],
            [profile_test_id, wire_test_id],
        )
        self.assertTrue(requirement["negative_verification_required"])
        self.assertEqual(requirement["status"], "scoped_verified")

        mappings = [
            row
            for row in load_typed_rule_mappings(ROOT)
            if row["requirement_id"] == requirement_id
        ]
        self.assertEqual(
            {(row["verification_category"], row["test_kind"], row["test_id"]) for row in mappings},
            {
                ("ProhibitedBehavior", "NegativeBehavior", profile_test_id),
                ("ProhibitedBehavior", "NegativeBehavior", wire_test_id),
            },
        )
        self.assertTrue(all(row["transport_only"] for row in mappings))

    def test_baseline_tls13_cipher_suite_support_has_positive_wire_evidence(self) -> None:
        from tools.normative_catalog.report import load_typed_rule_mappings

        requirement_id = "KMIPKIT-REQ-PROF-3.1.2-002"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1.2-002"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]
        client_suites = pinned_excerpt([4068, 4074])

        self.assertIn("Conformant KMIP clients SHALL support at least one", client_suites)
        self.assertIn("TLS13-CHACHA20-POLY1305-SHA256", client_suites)
        self.assertIn("TLS13-AES-256-GCM-SHA384", client_suites)
        self.assertEqual(
            self.clauses[clause_id]["locator"],
            {"ordinal": 2, "block_kind": "paragraph"},
        )
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "SHALL")
        self.assertEqual(requirement["source_clause_ids"], [clause_id])
        self.assertEqual(requirement["source_keyword"], "SHALL")
        self.assertEqual(requirement["normative_strength"], "mandatory")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["scope_state"], "profile_conditional")
        self.assertEqual(requirement["profile_ids"], ["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertFalse(requirement["negative_verification_required"])
        self.assertEqual(
            requirement["implementation_refs"],
            [
                "crates/kmipkit-protocol/src/profile.rs::ProfileTransport::satisfies_tls13_cipher_suite_policy",
                "crates/kmipkit-protocol/src/profile.rs::ProfileValidationResult::validate_transport",
                "crates/kmipkit-protocol/src/profile.rs::TYPED_RULE_MAPPINGS",
                "crates/kmipkit-transport/src/tls.rs::aws_lc_crypto_provider",
            ],
        )
        profile_test_id = (
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
            "profile_maps_required_tls13_cipher_suite_support"
        )
        wire_test_id = (
            "crates/kmipkit-transport/tests/raw_tls.rs::"
            "raw_tls_rejects_tls12_server_hello_before_request_dispatch"
        )
        self.assertEqual(requirement["verification_refs"], [profile_test_id, wire_test_id])
        self.assertEqual(requirement["status"], "scoped_verified")

        mappings = [
            row
            for row in load_typed_rule_mappings(ROOT)
            if row["requirement_id"] == requirement_id
        ]
        self.assertEqual(
            {(row["verification_category"], row["test_kind"], row["test_id"]) for row in mappings},
            {
                ("RequiredBehavior", "PositiveBehavior", profile_test_id),
                ("RequiredBehavior", "PositiveBehavior", wire_test_id),
            },
        )
        self.assertTrue(all(row["transport_only"] for row in mappings))

    def test_baseline_tls_legacy_protocol_prohibition_has_negative_wire_evidence(self) -> None:
        """Profiles §3.1.1 prohibits legacy TLS and SSL protocols for clients."""
        from tools.normative_catalog.report import load_typed_rule_mappings

        requirement_id = "KMIPKIT-REQ-PROF-3.1.1-005"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1.1-005"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]
        source_excerpt = pinned_excerpt([4037, 4052])

        self.assertIn("Conformant KMIP clients or servers SHALL NOT support", source_excerpt)
        self.assertIn("TLS v1.1 [RFC4346]", source_excerpt)
        self.assertIn("TLS v1.0 [RFC2246]", source_excerpt)
        self.assertIn("Any version of the SSL protocol", source_excerpt)
        self.assertEqual(
            self.clauses[clause_id]["locator"],
            {"ordinal": 5, "block_kind": "paragraph"},
        )
        self.assertIn("paragraph 5", review["review_note"])
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "SHALL NOT")
        self.assertEqual(requirement["source_clause_ids"], [clause_id])
        self.assertEqual(requirement["source_keyword"], "SHALL NOT")
        self.assertEqual(requirement["normative_strength"], "prohibited")
        self.assertEqual(requirement["role"], "client")
        self.assertEqual(requirement["direction"], "client_to_server")
        self.assertEqual(requirement["scope_state"], "profile_conditional")
        self.assertEqual(requirement["profile_ids"], ["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        self.assertEqual(requirement["test_case_ids"], [])
        self.assertTrue(requirement["negative_verification_required"])
        self.assertEqual(
            requirement["implementation_refs"],
            [
                "crates/kmipkit-protocol/src/profile.rs::ProfileTransport::satisfies_transport_security",
                "crates/kmipkit-protocol/src/profile.rs::ProfileValidationResult::validate_transport",
                "crates/kmipkit-protocol/src/profile.rs::TYPED_RULE_MAPPINGS",
                "crates/kmipkit-transport/src/tls.rs::tls13_builder",
            ],
        )
        profile_test_id = (
            "crates/kmipkit-protocol/tests/support/profile_unit.rs::"
            "profile_maps_prohibited_legacy_tls_and_ssl_protocols_to_tls13_transport"
        )
        test_id = (
            "crates/kmipkit-transport/tests/raw_tls.rs::"
            "raw_tls_rejects_tls12_server_hello_before_request_dispatch"
        )
        self.assertEqual(requirement["verification_refs"], [profile_test_id, test_id])
        self.assertEqual(requirement["status"], "scoped_verified")

        mapping = next(
            row for row in load_typed_rule_mappings(ROOT) if row["requirement_id"] == requirement_id
        )
        self.assertEqual(mapping["source_clause_id"], clause_id)
        self.assertEqual(mapping["verification_category"], "ProhibitedBehavior")
        self.assertEqual(mapping["test_kind"], "NegativeBehavior")
        self.assertEqual(mapping["test_id"], test_id)
        self.assertTrue(mapping["transport_only"])

    def test_tls12_client_may_is_preserved_but_outside_the_10_profile_scope(self) -> None:
        """Profiles §3.1.1's TLS 1.2 permission cannot expand the TLS 1.3-only ADR."""
        requirement_id = "KMIPKIT-REQ-PROF-3.1.1-004"
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1.1-004"
        requirement = self.requirements[requirement_id]
        review = self.reviews[clause_id]
        baseline = next(
            row
            for row in self.catalog["profiles"]
            if row["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        source_excerpt = pinned_excerpt([4030, 4037])
        transport_decision = (ROOT / "docs/adr/0005-transport-and-tls.md").read_text(
            encoding="utf-8"
        )

        self.assertIn("Conformant KMIP clients MAY support:", source_excerpt)
        self.assertIn("TLS v1.2 [RFC5246]", source_excerpt)
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "MAY")
        self.assertEqual(review["source_obligations"][0]["scope_disposition"], "out_of_scope")
        self.assertEqual(requirement["scope_state"], "out_of_scope")
        self.assertIn("ADR-0005", requirement["review_note"])
        self.assertIn("TLS 1.3 only", transport_decision)
        self.assertIn(clause_id, baseline["source_clause_ids"])
        self.assertEqual(self.clauses[clause_id]["scope_state"], "out_of_scope")
        self.assertIn(requirement_id, self.clauses[clause_id]["requirement_ids"])
        self.assertIn("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement["profile_ids"])
        self.assertNotIn(requirement_id, baseline["requirement_ids"])
        self.assertNotIn(requirement_id, self.crosswalk)

    def test_profile_test_variations_are_a_claimed_profile_evidence_gate(self) -> None:
        """§4.1.2 is conditional conformance evidence, not unconditional runtime behavior."""
        expected_scope = {
            "KMIPKIT-REQ-PROF-4.1-001": ("unclear", "unclear", "out_of_scope"),
            "KMIPKIT-REQ-PROF-4.1-003": ("unclear", "unclear", "out_of_scope"),
            "KMIPKIT-REQ-PROF-4.1.2-001": ("client", "not_applicable", "profile_conditional"),
            "KMIPKIT-REQ-PROF-4.1.2-005": ("unclear", "unclear", "unclear"),
            "KMIPKIT-REQ-PROF-4.1.2-008": (
                "not_applicable",
                "not_applicable",
                "out_of_scope",
            ),
            "KMIPKIT-REQ-PROF-4.1.2-009": (
                "not_applicable",
                "not_applicable",
                "out_of_scope",
            ),
        }
        baseline_id = "KMIPKIT-PROFILE-BASELINE-CLIENT"
        baseline = next(
            profile for profile in self.catalog["profiles"] if profile["profile_id"] == baseline_id
        )

        for requirement_id, (role, direction, scope) in expected_scope.items():
            requirement = self.requirements[requirement_id]
            clause_id = requirement["source_clause_ids"][0]
            clause = self.clauses[clause_id]
            review = self.reviews[clause_id]

            self.assertEqual((requirement["role"], requirement["direction"], requirement["scope_state"]), (role, direction, scope))
            expected_clause_scope = (
                ("unclear", "unclear", scope)
                if requirement_id == "KMIPKIT-REQ-PROF-4.1.2-001"
                else (role, direction, scope)
            )
            self.assertEqual(
                (clause["role"], clause["direction"], clause["scope_state"]),
                expected_clause_scope,
            )
            if requirement_id == "KMIPKIT-REQ-PROF-4.1.2-001":
                self.assertEqual(review["expected_client_requirement_ids"], [requirement_id])
                self.assertIn(requirement_id, self.crosswalk)
                self.assertIn(requirement_id, baseline["requirement_ids"])
                self.assertEqual(requirement["status"], "unassigned")
                self.assertEqual(requirement["implementation_refs"], [])
                self.assertEqual(requirement["verification_refs"], [])
            else:
                self.assertEqual(review["expected_client_requirement_ids"], [])
                self.assertNotIn(requirement_id, self.crosswalk)
                self.assertNotIn(requirement_id, baseline["requirement_ids"])
            if clause["section"] == "4.1.2":
                self.assertIn(clause_id, baseline["source_clause_ids"])
            else:
                self.assertNotIn(clause_id, baseline["source_clause_ids"])

        auth_requirement = self.requirements["KMIPKIT-REQ-PROF-4.1.2-005"]
        self.assertNotIn("client request", auth_requirement["summary"].lower())
        interop_text = pinned_excerpt([4494, 4502]).lower()
        self.assertIn("not expected that production kmip clients will support the interop operation", interop_text)
        clause_id = "KMIPKIT-CLAUSE-PROF-4.1.1-001"
        clause = self.clauses[clause_id]
        excerpt = pinned_excerpt([4330, 4338])

        self.assertIn("An implementation conformant to a Profile MAY vary the following values", excerpt)
        self.assertIn("Unique Batch Item ID", excerpt)
        self.assertEqual(clause["disposition"], "informative_context")
        self.assertEqual((clause["role"], clause["direction"], clause["scope_state"]), ("both", "unclear", "out_of_scope"))
        self.assertEqual(clause["requirement_ids"], [])
        self.assertIn("client", clause["exclusion_rationale"].lower())
        self.assertNotIn(clause_id, baseline["source_clause_ids"])

    def test_profile_4_1_2_007_is_may_without_a_false_must_keyword(self) -> None:
        requirement_id = "KMIPKIT-REQ-PROF-4.1.2-007"
        requirement = self.requirements[requirement_id]
        clause = self.clauses["KMIPKIT-CLAUSE-PROF-4.1.2-007"]
        self.assertIn("KMIPKIT-CLAUSE-PROF-4.1.2-007", self.reviews)
        review = self.reviews["KMIPKIT-CLAUSE-PROF-4.1.2-007"]
        self.assertEqual(requirement["source_keyword"], "MAY")
        self.assertEqual(requirement["status"], "scoped_verified")
        self.assertIn("MAY", clause["source_keywords"])
        self.assertEqual(review["source_obligations"][0]["source_keyword"], "MAY")
        self.assertEqual(review["html_line_range"], [4488, 4493])
        self.assertEqual(review["source_excerpt"], pinned_excerpt([4488, 4493]))

    def test_all_target_review_locators_and_excerpts_match_pinned_html(self) -> None:
        expected_lines = {
            "KMIPKIT-CLAUSE-PROF-3-002": [3967, 3970],
            "KMIPKIT-CLAUSE-PROF-3-003": [3972, 3975],
            "KMIPKIT-CLAUSE-PROF-3-004": [3977, 3980],
            "KMIPKIT-CLAUSE-PROF-3-006": [3987, 3990],
            "KMIPKIT-CLAUSE-PROF-3.1-001": [3999, 4001],
            "KMIPKIT-CLAUSE-PROF-3.1.1-002": [4016, 4016],
            "KMIPKIT-CLAUSE-PROF-3.1.1-004": [4030, 4030],
            "KMIPKIT-CLAUSE-PROF-3.1.1-005": [4037, 4037],
            "KMIPKIT-CLAUSE-PROF-3.1.2-002": [4068, 4069],
            "KMIPKIT-CLAUSE-PROF-3.1.2-004": [4089, 4090],
            "KMIPKIT-CLAUSE-PROF-3.1.2-005": [4187, 4188],
            "KMIPKIT-CLAUSE-PROF-3.2-001": [4227, 4229],
            "KMIPKIT-CLAUSE-PROF-3.2.1-001": [4236, 4237],
            "KMIPKIT-CLAUSE-PROF-3.2.2-001": [4244, 4245],
            "KMIPKIT-CLAUSE-PROF-3.2.3-001": [4252, 4253],
            "KMIPKIT-CLAUSE-PROF-3.2.4-002": [4264, 4266],
            "KMIPKIT-CLAUSE-PROF-4.1-001": [4312, 4314],
            "KMIPKIT-CLAUSE-PROF-4.1-003": [4320, 4321],
            "KMIPKIT-CLAUSE-PROF-4.1.2-001": [4464, 4465],
            "KMIPKIT-CLAUSE-PROF-4.1.2-005": [4482, 4483],
            "KMIPKIT-CLAUSE-PROF-4.1.2-007": [4488, 4493],
            "KMIPKIT-CLAUSE-PROF-4.1.2-008": [4494, 4495],
            "KMIPKIT-CLAUSE-PROF-4.1.2-009": [4496, 4497],
        }
        self.assertEqual(set(self.reviews), set(expected_lines))
        for requirement_id in ALL_TARGET_IDS:
            clause_id = self.requirements[requirement_id]["source_clause_ids"][0]
            self.assertIn(clause_id, expected_lines)
        for clause_id, line_range in expected_lines.items():
            self.assert_review_matches_pinned_source(
                self.reviews[clause_id], clause_id, line_range
            )

    def test_tls_1_2_cipher_suite_may_has_an_explicit_boundary_review(self) -> None:
        """Profiles §3.1.2's TLS 1.2 MAY is excluded by the 1.0 TLS boundary."""
        clause_id = "KMIPKIT-CLAUSE-PROF-3.1.2-004"
        source_excerpt = pinned_excerpt([4089, 4090])
        self.assertEqual(
            source_excerpt,
            "Conformant KMIP clients or servers MAY support the following cipher suites for TLSv1.2 if TLSv1.2 is supported:",
        )

        clause = self.clauses[clause_id]
        self.assertEqual(clause["source_keywords"], ["MAY"])
        self.assertEqual(clause["disposition"], "non_applicable")
        self.assertEqual(clause["scope_state"], "out_of_scope")
        self.assertEqual(clause["requirement_ids"], [])

        gap = next(
            row
            for row in self.fixture["source_only_requirement_gaps"]
            if row["gap_id"] == "KMIPKIT-SOURCE-GAP-PROF-3.1.2-4089"
        )
        self.assertEqual(gap["source_clause_id"], clause_id)
        self.assertEqual(gap["source_locator"], clause["locator"])
        self.assertEqual(gap["html_line_range"], [4089, 4185])
        self.assertEqual(gap["scope_disposition"], "mixed_server_and_out_of_scope")
        self.assertIsNone(gap["candidate_requirement_id"])
        self.assertIsNone(gap["proposed_requirement_id"])

        reviews = [
            row
            for row in self.fixture["source_requirement_reviews"]
            if row["source_clause_id"] == clause_id
        ]
        self.assertEqual(len(reviews), 1)
        review = reviews[0]
        self.assert_review_matches_pinned_source(review, clause_id, [4089, 4090])
        self.assertEqual(review["expected_client_requirement_ids"], [])
        self.assertEqual(review["applies_to"], "KMIPKIT-PROFILE-BASELINE-CLIENT")
        self.assertEqual(
            review["source_obligations"],
            [
                {
                    "source_order": 1,
                    "source_text": source_excerpt,
                    "source_keyword": "MAY",
                    "actor": "client and server",
                    "direction": "TLS 1.2 cipher-suite support",
                    "condition": "Only if TLS 1.2 is supported; the client branch is outside KMIPKit 1.0's TLS 1.3-only boundary and the server branch is outside this client library.",
                    "scope_disposition": "mixed_server_and_out_of_scope",
                    "candidate_requirement_id": None,
                }
            ],
        )

    def test_explicit_server_profile_requirements_remain_outside_client_crosswalk(self) -> None:
        expected = {
            "KMIPKIT-SOURCE-GAP-PROF-3.1.1-4009": ([4009, 4014], "formal_SHALL", "TLS v1.3", "Server; `server_only`."),
            "KMIPKIT-SOURCE-GAP-PROF-3.1.1-4023": ([4023, 4028], "formal_SHOULD", "TLS v1.2", "Server; `server_only`."),
            "KMIPKIT-SOURCE-GAP-PROF-3.1.2-4060": ([4060, 4065], "formal_SHALL", "TLS v1.3", "Server; `server_only`."),
            "KMIPKIT-SOURCE-GAP-PROF-3.1.3-4196": ([4196, 4198], "formal_SHALL", "automated client provisioning", "Server; `server_only`. The client presents its certificate, but this clause assigns enforcement to server."),
            "KMIPKIT-SOURCE-GAP-PROF-3.1.3-4200": ([4200, 4202], "formal_SHALL", "Authentication object is absent", "Server; condition Authentication absent; `server_only`."),
            "KMIPKIT-SOURCE-GAP-PROF-3.1.3-4204": ([4204, 4206], "formal_SHALL", "Authentication object is present", "Server; condition Authentication present; `server_only`."),
            "KMIPKIT-SOURCE-GAP-PROF-3.1.4-4217": ([4217, 4218], "formal_SHALL", "TCP port 5696", "Server; `server_only`."),
            "KMIPKIT-SOURCE-GAP-PROF-3.2.4-4260": ([4260, 4262], "formal_SHOULD", "both HTTPS and non-HTTPS", "Server; dual-support condition; `server_only`."),
        }
        gaps = {
            row["gap_id"]: row for row in self.fixture["source_only_requirement_gaps"]
        }

        for gap_id, (line_range, source_force, condition_fragment, actor_review) in expected.items():
            with self.subTest(gap_id=gap_id):
                gap = gaps[gap_id]
                source_excerpt = pinned_excerpt(line_range)
                self.assertEqual(gap["html_line_range"], line_range)
                self.assertEqual(gap["source_excerpt"], source_excerpt)
                self.assertTrue(
                    "Conformant KMIP servers" in source_excerpt
                    or "KMIP servers conformant to this profile" in source_excerpt
                )
                self.assertEqual(gap.get("actor"), "server")
                self.assertEqual(gap["source_force"], source_force)
                self.assertIn(condition_fragment, gap["condition"])
                self.assertEqual(gap["actor_scope_review"], actor_review)
                self.assertEqual(gap["scope_disposition"], "server_only")
                self.assertEqual(gap["status"], "server_actor_disposition_recorded")
                self.assertIsNone(gap["candidate_requirement_id"])
                self.assertIsNone(gap.get("reused_requirement_id"))
                self.assertIsNone(gap["proposed_requirement_id"])
                self.assertNotIn(gap_id, self.crosswalk)


if __name__ == "__main__":
    unittest.main()
