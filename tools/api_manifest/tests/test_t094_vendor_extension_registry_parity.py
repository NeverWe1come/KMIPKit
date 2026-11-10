"""T094 contract for KMIPKIT-0012 public declaration parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
RUST_REPORT = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-11-01.json"
)
OWNER_SPEC = "KMIPKIT-0012"
OWNER_PATH = "specs/012-vendor-extension-registry/spec.md"
OWNER_APPROVAL = (
    "**Status**: Approved for implementation under the maintainer's standing direct "
    "authorization to execute the complete KMIPKit plan without further "
    "approval requests."
)
PROJECT_REQUIREMENTS = {
    "KMIPKIT-0043-FR-004",
    "KMIPKIT-0043-FR-005",
    "KMIPKIT-0043-FR-010",
    "KMIPKIT-0043-FR-017",
    "KMIPKIT-0043-FR-021",
    "KMIPKIT-0043-FR-022",
}
LANGUAGES = {"rust", "c", "java", "python"}

EXPECTED_TRAIT_ROOTS = {
    "kmipkit::ClientBatchItem",
    "kmipkit_client::extension_registry::ClientConfiguration",
    "kmipkit_client::extension_registry::ClientExtensionRegistry",
    "kmipkit_client::extension_registry::ClientRequestMessageExtension",
    "kmipkit_client::extension_registry::ExtensionRecognition",
    "kmipkit_client::extension_registry::RegisteredExtensionValue",
    "kmipkit_protocol::extension::Compatibility",
    "kmipkit_protocol::extension::Discriminator",
    "kmipkit_protocol::extension::ExtensionChildRule",
    "kmipkit_protocol::extension::ExtensionDefinition",
    "kmipkit_protocol::extension::ExtensionDefinitionAccounting",
    "kmipkit_protocol::extension::ExtensionIdentity",
    "kmipkit_protocol::extension::ExtensionInformation",
    "kmipkit_protocol::extension::ExtensionOrderConstraint",
    "kmipkit_protocol::extension::ExtensionRegistryLimits",
    "kmipkit_protocol::extension::ExtensionSchema",
    "kmipkit_protocol::extension::TtlvPath",
    "kmipkit_protocol::extension::ValidatedExtensionValue",
}

EXPECTED_FUNCTIONAL_EXPORTS = {
    ("rust-kmipkit", "kmipkit::ClientBatchItem"),
    ("python-kmipkit", "kmipkit.extensions.with_extension"),
    ("c-kmipkit", "kmipkit_validated_extension_value_value_at"),
    ("java-kmipkit", "org.kmipkit.extensions.ValidatedExtensionValue.valueAt"),
    ("python-kmipkit", "kmipkit.extensions.value_at"),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ValidatedExtensionValue::with_value_at",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::ExtensionRecognition::is_recognized",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::ExtensionRecognition::validated_value",
    ),
}

EXPECTED_NONTRAIT_EXCLUSIONS = {
    ("java-kmipkit", "org.kmipkit.extensions.ExtensionIdentity.equals"),
    ("java-kmipkit", "org.kmipkit.extensions.ExtensionIdentity.hashCode"),
    ("java-kmipkit", "org.kmipkit.extensions.ExtensionIdentity.toString"),
    ("java-kmipkit", "org.kmipkit.extensions.ExtensionRecognition.toString"),
    ("java-kmipkit", "org.kmipkit.extensions.RegisteredExtensionValue.toString"),
    ("java-kmipkit", "org.kmipkit.extensions.TtlvPath.nativeHandleOwner"),
    ("java-kmipkit", "org.kmipkit.extensions.ValidatedExtensionValue.toString"),
    ("java-kmipkit", "org.kmipkit.ttlv.Tag.handle"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvItemType.code"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvItemType.fromCode"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvItemType.valueOf"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvItemType.values"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvItemView.fromNative"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvStructure.fromNative"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvStructure.handle"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvStructureView.fromNative"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvValue.handle"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvValue.nativeHandleOwner"),
    ("java-kmipkit", "org.kmipkit.ttlv.TtlvValueView.fromNative"),
    ("python-kmipkit", "kmipkit.errors"),
    ("python-kmipkit", "kmipkit.extensions"),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::ClientConfiguration::from_shared_registry",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::ClientConfiguration::shared_extension_registry",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::RegistryDefinitionPreflight",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::RegistryDefinitionPreflight::add_definition",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::RegistryDefinitionPreflight::finish",
    ),
    (
        "rust-kmipkit-client",
        "kmipkit_client::extension_registry::RegistryDefinitionPreflight::new",
    ),
    ("rust-kmipkit-protocol", "kmipkit_protocol::extension"),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::Discriminator::has_same_scalar",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::Discriminator::matches_value",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::Discriminator::matches_value_ref",
    ),
    ("rust-kmipkit-protocol", "kmipkit_protocol::extension::Discriminator::path"),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::Discriminator::scalar_fingerprint",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinition::compatibility",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinition::discriminator",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinition::identity_ref",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting.constraint_members",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting.discriminator_bytes",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting.discriminator_path_depth",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting.maximum_text_field_bytes",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting.schema_nodes",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionDefinitionAccounting.text_bytes",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::attribute",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::description",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::enumeration",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::item_type",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::name",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::parent_structure_tag",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::ExtensionInformation::tag",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::SchemaValidationOutcome",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::SchemaValidationOutcome::SchemaInvalid",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::SchemaValidationOutcome::SchemaInvalid::0",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::SchemaValidationOutcome::SchemaValid",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::SchemaValidationOutcome::SchemaValid::0",
    ),
    ("rust-kmipkit-protocol", "kmipkit_protocol::extension::TtlvPath::tags"),
    ("rust-kmipkit-protocol", "kmipkit_protocol::extension::accounting"),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::clone_extension_definition",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::clone_extension_discriminator",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::scalar_value_fingerprint",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::validate_schema_limits",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::validate_schema_limits_with_aggregate",
    ),
    (
        "rust-kmipkit-protocol",
        "kmipkit_protocol::extension::validate_schema_only",
    ),
}


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def trait_exports_from_report() -> set[tuple[str, str]]:
    report = load_json(RUST_REPORT)
    declarations = report["declarations"] + report["conditionalDeclarations"]
    return {
        (row["packageId"], row["exportName"])
        for row in declarations
        if any(
            row.get("exportName", "").startswith(f"<{root} as ")
            for root in EXPECTED_TRAIT_ROOTS
        )
    }


class T094VendorExtensionRegistryParityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = load_json(MANIFEST)

    def test_no_kmipkit_0012_declarations_remain_unresolved(self) -> None:
        unresolved = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertEqual([], unresolved, "KMIPKIT-0012 declarations remain unresolved")

    def test_published_rust_value_path_uses_a_borrowed_ttlv_callback(self) -> None:
        function = next(
            row
            for row in self.manifest["functions"]
            if row["id"] == "validated_extension_value_value_at"
        )
        self.assertEqual(
            {
                "module": "kmipkit_protocol::extension::ValidatedExtensionValue",
                "name": "with_value_at",
                "owner": "protocol",
                "requestBoundary": "not-applicable",
                "parameters": [
                    {"name": "value", "type": "&ValidatedExtensionValue"},
                    {"name": "path", "type": "&TtlvPath"},
                    {
                        "name": "callback",
                        "type": "impl for<'a> FnOnce(ValueView<'a>) -> R",
                    },
                ],
                "returnType": "Result<R, ProtocolError>",
            },
            function["rust"],
        )

    def test_all_functional_registry_exports_have_reviewed_four_language_mappings(self) -> None:
        declarations = self.manifest["publicDeclarations"]
        actual = {
            (row.get("packageId"), row.get("exportName"))
            for row in declarations
            if row.get("capabilityId") == "vendor_extension_registry"
            and row.get("approvalSource", {}).get("path") == OWNER_PATH
            and (row.get("packageId"), row.get("exportName"))
            in EXPECTED_FUNCTIONAL_EXPORTS
        }
        self.assertEqual(EXPECTED_FUNCTIONAL_EXPORTS, actual)

        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row.get("capabilityId") == "vendor_extension_registry"
        )
        for language_mappings in capability["mappings"].values():
            self.assertEqual(sorted(set(language_mappings)), language_mappings)

        for package, export in sorted(EXPECTED_FUNCTIONAL_EXPORTS):
            row = next(
                item
                for item in declarations
                if item.get("packageId") == package and item.get("exportName") == export
            )
            with self.subTest(export=export):
                self.assertEqual("declared", row.get("declarationState"))
                self.assertNotIn("candidateSpecId", row)
                self.assertEqual("four-language", row.get("parityScope"))
                self.assertEqual("vendor_extension_registry", row.get("capabilityId"))
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": OWNER_PATH,
                        "status": "Approved",
                        "evidence": OWNER_APPROVAL,
                    },
                    row.get("approvalSource"),
                )
                self.assertEqual(LANGUAGES, set(row.get("languageMappings", {})))
                observed_language = {
                    "c-kmipkit": "c",
                    "java-kmipkit": "java",
                    "python-kmipkit": "python",
                }.get(package, "rust")
                self.assertEqual(export, row["languageMappings"][observed_language]["name"])
                self.assertEqual(
                    package,
                    row["languageMappings"][observed_language]["packageId"],
                )
                self.assertTrue(
                    PROJECT_REQUIREMENTS <= set(row.get("requirementIds", [])),
                    "the mapping must cite every public-surface parity requirement",
                )
                self.assertTrue(
                    any(value.startswith(f"{OWNER_SPEC}-FR-") for value in row["requirementIds"]),
                    "the mapping must cite an accepted vendor-extension requirement",
                )
                for language in ("c", "java", "python"):
                    target = row["languageMappings"][language]
                    self.assertTrue(target.get("name"))
                    self.assertEqual(
                        [target["name"]],
                        [
                            name
                            for name in capability["mappings"][language]
                            if name == target["name"]
                        ],
                        f"{language} mapping is outside the approved capability inventory",
                    )
                self.assertTrue(row.get("parityEvidenceIds"))
                for evidence_path in row["parityEvidenceIds"]:
                    self.assertTrue((ROOT / evidence_path).is_file(), evidence_path)

    def test_rust_trait_implementation_artifacts_are_exact_owner_exclusions(self) -> None:
        expected = trait_exports_from_report()
        self.assertEqual(254, len(expected), "the reviewed Rust trait inventory changed")
        exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("candidateSpecId") == OWNER_SPEC
            and row.get("exportName", "").startswith("<")
            # Root facade aliases added by 0053 are accounted for by its own
            # generated route audit, not by the original 0012 candidate set.
            and not row.get("exportName", "").startswith(
                ("<kmipkit::protocol::extension::", "<kmipkit::client_config::")
            )
        ]
        actual = {(row.get("packageId"), row.get("exportName")) for row in exclusions}
        self.assertEqual(expected, actual)

        for row in exclusions:
            with self.subTest(export=row.get("exportName")):
                self.assertIn("trait implementation artifact", row.get("reason", ""))
                self.assertTrue(row.get("evidence"))
                self.assertIn(row["exportName"], row["evidence"])
                self.assertTrue(row["evidence"].startswith("Exact export "))
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": OWNER_PATH,
                        "status": "Approved",
                        "evidence": OWNER_APPROVAL,
                    },
                    row.get("approvalSource"),
                )

    def test_each_nonfunctional_export_has_its_exact_owner_exclusion(self) -> None:
        exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row.get("candidateSpecId") == OWNER_SPEC
            and not row.get("exportName", "").startswith("<")
            and row.get("packageId") != "rust-kmipkit"
        ]
        actual = {(row.get("packageId"), row.get("exportName")) for row in exclusions}
        self.assertEqual(EXPECTED_NONTRAIT_EXCLUSIONS, actual)

        for row in exclusions:
            with self.subTest(export=row.get("exportName")):
                self.assertTrue(row.get("reason"), "an exclusion needs an exact rationale")
                self.assertTrue(row.get("evidence"), "an exclusion needs auditable evidence")
                self.assertIn(row["exportName"], row["evidence"])
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": OWNER_PATH,
                        "status": "Approved",
                        "evidence": OWNER_APPROVAL,
                    },
                    row.get("approvalSource"),
                )
