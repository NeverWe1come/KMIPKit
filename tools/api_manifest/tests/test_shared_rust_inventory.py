"""T013 traceability and parity gates for mapped shared Rust declarations."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[3]
MODULE_PATH = REPO_ROOT / "tools" / "api_manifest" / "audit_shared_rust_inventory.py"
RUST_REPORT_DIR = REPO_ROOT / "specs" / "043-public-api-bindings" / "reports"
RUST_REPORT_PATH = max(RUST_REPORT_DIR.glob("rust-export-report-*.json"))
EXPECTED_RUST_ONLY_DECLARATIONS = {
    "kmipkit_transport::Transport",
    "kmipkit_transport::Transport::exchange",
    "kmipkit_transport::TransportResponse",
    "kmipkit_transport::TransportResponse::new",
    "kmipkit_transport::TransportResponse::as_bytes",
    "kmipkit_transport::RequestDeliveryState",
    "kmipkit_transport::RequestDeliveryState::NotSent",
    "kmipkit_transport::RequestDeliveryState::PossiblySent",
    "kmipkit_transport::RequestDeliveryState::ResponseStarted",
    "kmipkit_transport::RequestDeliveryState::not_sent",
    "kmipkit_transport::RequestDeliveryState::write_started",
    "kmipkit_transport::RequestDeliveryState::response_bytes_received",
    "kmipkit_transport::TransportCauseCategory",
    "kmipkit_transport::TransportCauseCategory::Io",
    "kmipkit_transport::TransportCauseCategory::Tls",
    "kmipkit_transport::TransportCauseCategory::Http",
    "kmipkit_transport::TransportCauseCategory::Timeout",
    "kmipkit_transport::TransportCauseCategory::Other",
    "kmipkit_transport::TransportError",
    "kmipkit_transport::TransportError::new",
    "kmipkit_transport::TransportError::delivery_state",
    "kmipkit_transport::TransportError::cause_category",
}
EXPECTED_RUST_ONLY_ROOT_ALIASES = {
    "kmipkit::RequestDeliveryState",
    "kmipkit::RequestDeliveryState::NotSent",
    "kmipkit::RequestDeliveryState::PossiblySent",
    "kmipkit::RequestDeliveryState::ResponseStarted",
    "kmipkit::RequestDeliveryState::not_sent",
    "kmipkit::RequestDeliveryState::response_bytes_received",
    "kmipkit::RequestDeliveryState::write_started",
    "kmipkit::TransportCauseCategory",
    "kmipkit::TransportCauseCategory::Http",
    "kmipkit::TransportCauseCategory::Io",
    "kmipkit::TransportCauseCategory::Other",
    "kmipkit::TransportCauseCategory::Timeout",
    "kmipkit::TransportCauseCategory::Tls",
    "kmipkit::TransportError",
    "kmipkit::TransportError::cause_category",
    "kmipkit::TransportError::delivery_state",
    "kmipkit::TransportError::new",
}
SPEC = importlib.util.spec_from_file_location("audit_shared_rust_inventory", MODULE_PATH)
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)


class SharedRustInventoryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(
            (REPO_ROOT / "specification" / "api" / "public-api.json").read_text(encoding="utf-8")
        )
        self.rust_report = json.loads(
            RUST_REPORT_PATH.read_text(encoding="utf-8")
        )

    def test_approved_owner_lookup_uses_feature_requirements_when_directory_number_differs(self) -> None:
        owner_requirements = AUDIT._read_approved_owner("KMIPKIT-0044", REPO_ROOT)

        self.assertIn("KMIPKIT-0044-FR-001", owner_requirements)
        self.assertIn("KMIPKIT-0044-FR-007", owner_requirements)

    def test_shared_inventory_report_names_the_rust_audit_it_reads(self) -> None:
        expected_source = (
            "specs/043-public-api-bindings/reports/rust-export-report-2026-10-30.json"
        )
        generated = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        checked_in = json.loads(
            (REPO_ROOT / AUDIT.REPORT_PATH).read_text(encoding="utf-8")
        )

        self.assertEqual(expected_source, generated["source"])
        self.assertEqual(expected_source, checked_in["source"])

    def test_inventory_accounts_for_exact_approved_rust_exclusions_separately(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)

        self.assertEqual(inventory["approvedExcludedRustDeclarationCount"], 1399)
        self.assertEqual(
            sum(record["excludedDeclarationCount"] for record in inventory["crates"]),
            1399,
        )
        self.assertEqual(
            sum(record["blockedDeclarationCount"] for record in inventory["crates"]),
            inventory["observedRustDeclarationCount"]
            - inventory["mappedRustDeclarationCount"]
            - inventory["approvedExcludedRustDeclarationCount"],
        )

    def test_operation_mappings_are_not_counted_as_shared_capabilities(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        operation_ids = {row["operationId"] for row in self.manifest["operations"]}

        self.assertFalse(
            any(row["capabilityId"] in operation_ids for row in inventory["mappedDeclarations"])
        )

    def test_current_inventory_links_every_mapped_rust_declaration_to_approved_owner_requirements(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)

        self.assertEqual(inventory["observedRustDeclarationCount"], 10457)
        self.assertEqual(inventory["mappedRustDeclarationCount"], 509)
        self.assertEqual(len(inventory["blockedUnreconciledDeclarations"]), 2)
        self.assertEqual(set(inventory["draftOwnerGates"]), {"KMIPKIT-0010"})
        self.assertEqual(
            {record["packageId"] for record in inventory["crates"]},
            {
                "rust-kmipkit-ttlv",
                "rust-kmipkit-protocol",
                "rust-kmipkit-transport",
                "rust-kmipkit-client",
                "rust-kmipkit",
            },
        )
        for declaration in inventory["mappedDeclarations"]:
            self.assertTrue(declaration["requirementIds"])
            if declaration["ownerSpecId"] is not None:
                self.assertTrue(declaration["ownerRequirementIds"])
                self.assertTrue(
                    set(declaration["ownerRequirementIds"]) <= set(declaration["requirementIds"])
                )
            else:
                self.assertEqual(declaration["ownerAdrId"], "ADR-0014")
                self.assertFalse(declaration["ownerRequirementIds"])
                self.assertIn("KMIPKIT-0043-FR-020", declaration["requirementIds"])

    def test_candidate_declarations_remain_blocked_and_approved_async_operations_are_declared(self) -> None:
        for record in self.manifest["publicDeclarations"]:
            if record.get("candidateSpecId") in {"KMIPKIT-0009", "KMIPKIT-0010"}:
                self.assertEqual(record["declarationState"], "blocked")
                self.assertNotIn("approvalSource", record)
                self.assertNotIn("languageMappings", record)
        asynchronous_operations = [
            operation
            for operation in self.manifest["operations"]
            if operation.get("ownerDependency", {}).get("specId") == "KMIPKIT-0009"
        ]
        self.assertEqual(len(asynchronous_operations), 4)
        for operation in self.manifest["operations"]:
            dependency = operation.get("ownerDependency", {})
            if dependency.get("specId") == "KMIPKIT-0009":
                self.assertEqual(dependency.get("state"), "approved")
                self.assertEqual(operation["apiState"], "declared")
                self.assertNotIn("blockedReason", operation)
            elif dependency.get("state") == "candidate" and dependency.get("specId") == "KMIPKIT-0010":
                self.assertEqual(operation["apiState"], "blocked")

    def test_transport_is_the_only_accepted_rust_only_capability(self) -> None:
        rows = [row for row in self.manifest["publicDeclarations"] if row["parityScope"] == "rust-only-adr"]

        self.assertEqual(
            {row["exportName"] for row in rows},
            EXPECTED_RUST_ONLY_DECLARATIONS | EXPECTED_RUST_ONLY_ROOT_ALIASES,
        )
        self.assertEqual({row["capabilityId"] for row in rows}, {"rust_transport_exchange"})
        self.assertEqual(
            {row["packageId"] for row in rows},
            {"rust-kmipkit-transport", "rust-kmipkit"},
        )
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        self.assertEqual(
            len(inventory["rustOnlyDeclarations"]),
            len(EXPECTED_RUST_ONLY_DECLARATIONS | EXPECTED_RUST_ONLY_ROOT_ALIASES),
        )
        root_declarations = {
            row["exportName"]: row
            for row in rows
            if row["packageId"] == "rust-kmipkit"
        }
        for export_name in EXPECTED_RUST_ONLY_ROOT_ALIASES:
            with self.subTest(export=export_name):
                declaration = root_declarations[export_name]
                self.assertEqual(declaration["parityScope"], "rust-only-adr")
                self.assertEqual(
                    declaration["languageMappings"],
                    {
                        "rust": {
                            "packageId": "rust-kmipkit",
                            "name": export_name,
                        }
                    },
                )

    def test_approved_registry_capability_matches_its_existing_four_language_declarations(self) -> None:
        capability = next(
            row for row in self.manifest["capabilities"] if row["capabilityId"] == "vendor_extension_registry"
        )
        declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row["capabilityId"] == "vendor_extension_registry"
            and row["declarationState"] in {"declared", "verified"}
        ]

        self.assertEqual(capability["capabilityState"], "declared")
        self.assertEqual(capability["ownerSpecId"], "KMIPKIT-0012")
        self.assertEqual(capability["approvalEvidence"]["status"], "Approved")
        language_packages = {
            "rust": {
                package["id"]
                for package in self.manifest["publicPackages"]
                if package["language"] == "rust"
            },
            "c": {"c-kmipkit"},
            "java": {"java-kmipkit"},
            "python": {"python-kmipkit"},
        }
        for language, packages in language_packages.items():
            expected = sorted(
                {
                    row["exportName"]
                    for row in declarations
                    if row["packageId"] in packages
                }
            )
            self.assertEqual(capability["mappings"][language], expected)

    def test_response_ttlv_access_uses_its_approved_crypto_operation_owner(self) -> None:
        response_ttlv = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit_client::ClientError::response_ttlv"
        )
        self.assertEqual("crypto_response_ttlv", response_ttlv["capabilityId"])
        self.assertIn("KMIPKIT-0021-FR-015", response_ttlv["requirementIds"])
        self.assertNotIn("KMIPKIT-0007-FR-006", response_ttlv["requirementIds"])
        response_ttlv_field = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit::ClientError::ProtocolResponse::response_ttlv"
        )
        self.assertEqual("crypto_response_ttlv", response_ttlv_field["capabilityId"])
        self.assertIn("KMIPKIT-0021-FR-015", response_ttlv_field["requirementIds"])
        response_ttlv_method = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["exportName"] == "kmipkit::ClientError::response_ttlv"
        )
        self.assertEqual("crypto_response_ttlv", response_ttlv_method["capabilityId"])
        self.assertIn("KMIPKIT-0021-FR-015", response_ttlv_method["requirementIds"])

        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        declarations = {row["exportName"]: row for row in inventory["mappedDeclarations"]}

        structure_view = declarations["kmipkit_ttlv::StructureView"]
        self.assertEqual(structure_view["capabilityId"], "generic_ttlv_model")
        self.assertEqual(structure_view["ownerSpecId"], "KMIPKIT-0004")
        self.assertTrue(
            {
                "KMIPKIT-0004-FR-009",
                "KMIPKIT-0004-FR-010",
                "KMIPKIT-0004-FR-012",
            }
            <= set(structure_view["requirementIds"])
        )

        batch_item = declarations["kmipkit_client::ClientBatchItem"]
        self.assertEqual(batch_item["ownerSpecId"], "KMIPKIT-0012")
        self.assertIn("KMIPKIT-0007-FR-001", batch_item["requirementIds"])

        discover_versions = declarations["kmipkit_client::ClientBatchItem::discover_versions"]
        self.assertEqual(discover_versions["capabilityId"], "discover_versions_request_item")
        self.assertEqual(discover_versions["ownerSpecId"], "KMIPKIT-0007")
        self.assertTrue(
            {
                "KMIPKIT-0007-FR-001",
                "KMIPKIT-0007-FR-002",
            }
            <= set(discover_versions["ownerRequirementIds"])
        )

        operation = next(row for row in self.manifest["operations"] if row["name"] == "Discover Versions")
        self.assertEqual(operation["apiState"], "verified")
        self.assertEqual(operation["featureSpecId"], "KMIPKIT-0007")
        self.assertEqual(
            set(operation["declarations"]),
            {"rust", "c", "java", "python"},
        )

    def test_all_ready_capability_mappings_match_exact_ready_exports(self) -> None:
        language_packages = {
            language: {
                package["id"]
                for package in self.manifest["publicPackages"]
                if package["language"] == language
            }
            for language in ("rust", "c", "java", "python")
        }
        for capability in self.manifest["capabilities"]:
            if "mappings" not in capability:
                continue
            for language, packages in language_packages.items():
                if capability["parityScope"] == "rust-only-adr" and language != "rust":
                    continue
                expected = sorted(
                    {
                        row["languageMappings"][language]["name"]
                        for row in self.manifest["publicDeclarations"]
                        if row["capabilityId"] == capability["capabilityId"]
                        and row["declarationState"] in {"declared", "verified"}
                        and language in row.get("languageMappings", {})
                        and row["languageMappings"][language]["packageId"] in packages
                    }
                )
                self.assertEqual(
                    capability["mappings"].get(language, []),
                    expected,
                    f"{capability['capabilityId']} {language} mappings must name exact ready exports",
                )

    def test_unrepresented_runtime_error_enum_stays_blocked_but_other_variant_maps(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "rust-kmipkit"
        }
        error_type = declarations["kmipkit::ClientCauseCategory"]
        other_variant = declarations["kmipkit::ClientCauseCategory::Other"]

        self.assertEqual(error_type["kind"], "type")
        self.assertEqual(error_type["declarationState"], "blocked")
        self.assertEqual(error_type["candidateSpecId"], "KMIPKIT-0043")
        for missing_category in ("invalid-input", "protocol", "transport"):
            with self.subTest(category=missing_category):
                self.assertIn(missing_category, error_type["blockedReason"])
        self.assertNotIn("languageMappings", error_type)
        self.assertNotIn("approvalSource", error_type)
        self.assertEqual(other_variant["kind"], "variant")
        self.assertEqual(other_variant["declarationState"], "declared")
        self.assertEqual(
            other_variant["languageMappings"]["rust"],
            {"packageId": "rust-kmipkit", "name": "kmipkit::ClientCauseCategory::Other"},
        )

    def test_every_ready_language_mapping_resolves_to_an_exact_ready_export(self) -> None:
        declarations_by_export = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        approved_exclusions_by_export = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        for declaration in self.manifest["publicDeclarations"]:
            if declaration["declarationState"] not in {"declared", "verified"}:
                continue
            for language, mapping in declaration["languageMappings"].items():
                target = declarations_by_export.get((mapping["packageId"], mapping["name"]))
                if target is None:
                    exclusion = approved_exclusions_by_export.get(
                        (mapping["packageId"], mapping["name"])
                    )
                    self.assertIsNotNone(
                        exclusion,
                        f"{declaration['declarationId']} maps {language} to an unlisted export",
                    )
                    self.assertEqual(
                        (mapping["packageId"], mapping["name"]),
                        ("c-kmipkit", "kmipkit_client_batch_item_response_release"),
                    )
                    self.assertIn("lifecycle counterpart", exclusion["reason"])
                    self.assertIn("distinct four-language capability", exclusion["reason"])
                    self.assertIn("Encrypt and Decrypt response lifecycle mappings", exclusion["evidence"])
                    self.assertIn(
                        declaration["declarationId"],
                        {
                            "T040:encrypt:response-close:java",
                            "T040:encrypt:response-close:python",
                            "T040:decrypt:response-close:java",
                            "T040:decrypt:response-close:python",
                        },
                    )
                    continue
                self.assertIsNotNone(
                    target,
                    f"{declaration['declarationId']} maps {language} to an unlisted export",
                )
                self.assertIn(
                    target["declarationState"],
                    {"declared", "verified"},
                    f"{declaration['declarationId']} maps {language} to a blocked export",
                )

    def test_only_unmapped_runtime_namespace_roots_remain_blocked(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        blocked = inventory["blockedUnreconciledDeclarations"]

        self.assertEqual(
            {row["exportName"] for row in blocked},
            {"org.kmipkit.RuntimeMetadata", "kmipkit.metadata"},
        )
        self.assertEqual(
            sum("callable public declaration" in row["blockedReason"] for row in blocked),
            0,
        )
        self.assertEqual(
            sum("No exact equivalent public Rust declaration" in row["blockedReason"] for row in blocked),
            0,
        )
        for row in blocked:
            source = next(
                declaration
                for declaration in self.manifest["publicDeclarations"]
                if declaration["declarationId"] == row["declarationId"]
            )
            self.assertEqual(source["declarationState"], "blocked")
            self.assertNotIn("languageMappings", source)
            self.assertNotIn("approvalSource", source)

    def test_runtime_metadata_namespaces_keep_their_candidate_in_blocked_report(self) -> None:
        inventory = AUDIT.build_inventory(self.manifest, self.rust_report, REPO_ROOT)
        blocked = {
            row["declarationId"]: row
            for row in inventory["blockedUnreconciledDeclarations"]
        }
        expected = {
            "org.kmipkit.RuntimeMetadata:java": "org.kmipkit.RuntimeMetadata",
            "kmipkit.metadata:python": "kmipkit.metadata",
        }

        for declaration_id, export_name in expected.items():
            with self.subTest(declaration=declaration_id):
                row = blocked[declaration_id]
                self.assertEqual(row["exportName"], export_name)
                self.assertEqual(row["candidateSpecId"], "KMIPKIT-0043")
                self.assertIn("crate root is not an exported declaration", row["blockedReason"])

                source = next(
                    declaration
                    for declaration in self.manifest["publicDeclarations"]
                    if declaration["declarationId"] == declaration_id
                )
                self.assertEqual(source["declarationState"], "blocked")
                self.assertNotIn("languageMappings", source)
                self.assertNotIn("approvalSource", source)

    def test_ready_declaration_with_an_unobserved_rust_mapping_is_rejected(self) -> None:
        altered = json.loads(json.dumps(self.manifest))
        declaration = next(
            row
            for row in altered["publicDeclarations"]
            if row.get("packageId") == "java-kmipkit"
            and row.get("declarationState") == "declared"
        )
        declaration["languageMappings"]["rust"]["name"] = "kmipkit::invented_public_item"

        with self.assertRaises(AUDIT.InventoryError):
            AUDIT.build_inventory(altered, self.rust_report, REPO_ROOT)

    def test_unapproved_rust_only_owner_is_rejected(self) -> None:
        altered = json.loads(json.dumps(self.manifest))
        declaration = next(row for row in altered["publicDeclarations"] if row["parityScope"] == "rust-only-adr")
        declaration["capabilityId"] = "public_binding_surface"

        with self.assertRaises(AUDIT.InventoryError):
            AUDIT.build_inventory(altered, self.rust_report, REPO_ROOT)


if __name__ == "__main__":
    unittest.main()
