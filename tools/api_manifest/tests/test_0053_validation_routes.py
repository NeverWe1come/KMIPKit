"""Tests for exact KMIPKIT-0053 request-validation route reconciliation."""

from __future__ import annotations

import copy
import json
import re
import unittest
from pathlib import Path

from tools.api_manifest.prepare_0053_validation_routes import (
    ALIAS_CANONICAL_NAMES,
    prepare_validation_routes,
)


ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
REPORT_PATH = (
    ROOT
    / "specs"
    / "053-api-parity-facade"
    / "reports"
    / "rust-export-report-2026-11-07.json"
)
ROUTES_PATH = (
    ROOT / "specs" / "053-api-parity-facade" / "additional-api-routes.json"
)
CANDIDATES_PATH = (
    ROOT
    / "specs"
    / "053-api-parity-facade"
    / "candidate-resolutions"
    / "validation-export-candidates.json"
)


class ValidationRoutePreparationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT_PATH.read_text(encoding="utf-8"))
        cls.routes = json.loads(ROUTES_PATH.read_text(encoding="utf-8"))
        cls.candidates = json.loads(CANDIDATES_PATH.read_text(encoding="utf-8"))
        cls.prepared = prepare_validation_routes(
            cls.manifest, cls.report, cls.routes, cls.candidates
        )

    def test_exact_blocked_inventory_is_mapped_or_excluded(self) -> None:
        blocked = [
            row for row in self.candidates["candidates"]
        ]
        mappings = [
            row
            for row in self.prepared["publicDeclarations"]
            if row.get("candidateSpecId") == "KMIPKIT-0053"
            and row.get("packageId") in {"rust-kmipkit", "rust-kmipkit-protocol", "rust-kmipkit-client"}
            and row.get("capabilityId") == "request_validation"
            and row.get("exportName")
            in {name for name in ALIAS_CANONICAL_NAMES}
        ]
        exclusions = [
            row
            for row in self.prepared["approvedExclusions"]
            if row.get("candidateSpecId") == "KMIPKIT-0053"
            and row.get("capabilityId") == "request_validation"
            and (row.get("packageId"), row.get("exportName"))
            in {(row["packageId"], row["exportName"]) for row in blocked}
        ]
        removals = {
            row["declarationId"]
            for row in self.prepared["removePublicDeclarations"]
        }
        self.assertEqual(934, len(blocked))
        self.assertEqual(12, len(mappings))
        self.assertEqual(922, len(exclusions))
        blocked_ids = {row["declarationId"] for row in blocked}
        self.assertEqual(blocked_ids, removals & blocked_ids)
        self.assertFalse(
            any(
                row.get("declarationState") == "blocked"
                and row.get("candidateSpecId")
                in {"KMIPKIT-0009", "KMIPKIT-0043", "KMIPKIT-0044"}
                for row in self.manifest["publicDeclarations"]
            )
        )
        self.assertEqual(
            {row["exportName"] for row in blocked if row["exportName"] in ALIAS_CANONICAL_NAMES},
            {row["exportName"] for row in mappings},
        )

    def test_each_exclusion_keeps_exact_rustdoc_identity_and_approved_evidence(self) -> None:
        blocked = {
            (row["packageId"], row["exportName"]): row
            for row in self.candidates["candidates"]
        }
        audit_rows = {
            (row["packageId"], row["exportName"]): row
            for section in ("declarations", "conditionalDeclarations")
            for row in self.report[section]
        }
        for exclusion in self.prepared["approvedExclusions"]:
            key = (exclusion.get("packageId"), exclusion.get("exportName"))
            if key not in blocked:
                continue
            with self.subTest(exportName=key[1]):
                self.assertEqual("request_validation", exclusion["capabilityId"])
                self.assertTrue(exclusion["counterpartDeclarationIds"])
                if "condition" in audit_rows[key]:
                    self.assertEqual(
                        "rust-conditional-trait-implementation-member",
                        exclusion["rustArtifactKind"],
                    )
                self.assertEqual("KMIPKIT-0053", exclusion["candidateSpecId"])
                self.assertEqual(
                    "specs/053-api-parity-facade/approval-record.md",
                    exclusion["approvalSource"]["path"],
                )
                audit = audit_rows[key]
                if "condition" in audit:
                    self.assertEqual(
                        {
                            "condition": audit["condition"],
                            "implementationId": audit["implementationId"],
                        },
                        json.loads(exclusion["identity"]),
                    )
                    self.assertEqual(
                        audit["condition"], json.loads(exclusion["condition"])
                    )
                else:
                    self.assertEqual(audit["identity"], exclusion["identity"])
                self.assertTrue(exclusion.get("reason"))
                self.assertTrue(exclusion.get("evidencePaths"))
                self.assertFalse(any(char in exclusion["exportName"] for char in "*?"))

    def test_direct_lower_level_aliases_map_to_the_canonical_facade_routes(self) -> None:
        canonical_routes = {
            row["languageMappings"]["rust"]["name"]: row
            for row in [
                *self.manifest["publicDeclarations"],
                *self.routes["publicDeclarations"],
            ]
            if row.get("capabilityId") == "request_validation"
            and row.get("declarationState") == "verified"
            and isinstance(row.get("languageMappings", {}).get("rust"), dict)
        }
        for declaration in self.prepared["publicDeclarations"]:
            canonical = ALIAS_CANONICAL_NAMES.get(declaration.get("exportName"))
            if canonical is None or declaration.get("candidateSpecId") != "KMIPKIT-0053":
                continue
            source = canonical_routes[canonical]
            with self.subTest(exportName=declaration["exportName"]):
                self.assertEqual("verified", declaration["declarationState"])
                self.assertEqual("request_validation", declaration["capabilityId"])
                self.assertEqual(
                    declaration["exportName"],
                    declaration["languageMappings"]["rust"]["name"],
                )
                self.assertEqual("rust-kmipkit-client" if declaration["exportName"].startswith("kmipkit_client::") else "rust-kmipkit-protocol", declaration["languageMappings"]["rust"]["packageId"])
                self.assertEqual({"rust", "c", "java", "python"}, set(declaration["languageMappings"]))
                for language in ("c", "java", "python"):
                    self.assertEqual(
                        source["languageMappings"][language],
                        declaration["languageMappings"][language],
                    )
                self.assertTrue(
                    {"KMIPKIT-0053-FR-006", "KMIPKIT-0053-FR-011", "KMIPKIT-0053-FR-012"}
                    <= set(declaration["requirementIds"])
                )

    def test_validation_schema_strings_cover_every_rust_enum_variant(self) -> None:
        schema_source = (
            ROOT / "crates" / "kmipkit-protocol" / "src" / "validation" / "schema.rs"
        ).read_text(encoding="utf-8")
        generated_source = (
            ROOT
            / "crates"
            / "kmipkit-protocol"
            / "src"
            / "validation"
            / "schemas_generated.rs"
        ).read_text(encoding="utf-8")
        ffi_source = (
            ROOT
            / "crates"
            / "kmipkit-ffi"
            / "src"
            / "extension_registry"
            / "request_validation.rs"
        ).read_text(encoding="utf-8")

        def variants(source: str, enum_name: str) -> set[str]:
            match = re.search(
                rf"pub\s+enum\s+{enum_name}\s*\{{(.*?)^\}}",
                source,
                re.MULTILINE | re.DOTALL,
            )
            self.assertIsNotNone(match, enum_name)
            return set(re.findall(r"^\s{4}([A-Z][A-Za-z0-9_]*)\s*,?\s*$", match.group(1), re.MULTILINE))

        request_variants = variants(generated_source, "RequestPayloadSchema")
        nested_variants = variants(schema_source, "NestedStructureSchema")
        request_arms = set(
            re.findall(
                r'^\s*"([A-Z][A-Za-z0-9_]*)"\s*=>\s*RequestPayloadSchema::\1,',
                ffi_source,
                re.MULTILINE,
            )
        )
        nested_arms = set(
            re.findall(
                r'^\s*"([A-Z][A-Za-z0-9_]*)"\s*=>\s*NestedStructureSchema::\1,',
                ffi_source,
                re.MULTILINE,
            )
        )
        self.assertEqual(request_variants, request_arms)
        self.assertEqual(nested_variants, nested_arms)
        self.assertIn("SetEndpointRole", request_variants)
        self.assertIn('if value == "Unsupported"', ffi_source)

    def test_preparation_is_deterministic(self) -> None:
        self.assertEqual(
            self.prepared,
            prepare_validation_routes(
                self.manifest, self.report, self.prepared, self.candidates
            ),
        )

    def test_candidate_snapshot_cannot_redirect_reconciliation_to_another_export(self) -> None:
        tampered = copy.deepcopy(self.candidates)
        tampered["sourceCommit"] = "z" * 40
        with self.assertRaises(ValueError):
            prepare_validation_routes(
                self.manifest, self.report, self.routes, tampered
            )

        tampered = copy.deepcopy(self.candidates)
        candidate = next(
            row
            for row in tampered["candidates"]
            if row["exportName"] != "kmipkit_client::ClientRequest::validation_report"
        )
        candidate["exportName"] = "kmipkit::protocol::ManagedValueReader"
        candidate["kind"] = "type"
        with self.assertRaises(ValueError):
            prepare_validation_routes(
                self.manifest, self.report, self.routes, tampered
            )


if __name__ == "__main__":
    unittest.main()
