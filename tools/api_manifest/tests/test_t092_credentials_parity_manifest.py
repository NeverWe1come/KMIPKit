"""T092 contract for KMIPKIT-0008 credential declaration parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0008"
LANGUAGES = {"rust", "c", "java", "python"}
RUST_EXPORT_INVENTORY = (
    ROOT
    / "specs"
    / "043-public-api-bindings"
    / "reports"
    / "rust-export-report-2026-10-30.json"
)


def _is_nonblank_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _has_approved_source(record: dict[str, object]) -> bool:
    source = record.get("approvalSource")
    return (
        isinstance(source, dict)
        and source.get("status") == "Approved"
        and _is_nonblank_string(source.get("path"))
        and _is_nonblank_string(source.get("evidence"))
    )


class T092CredentialsParityManifestTests(unittest.TestCase):
    def test_exact_trait_implementation_artifacts_have_owner_backed_exclusions(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        inventory = json.loads(RUST_EXPORT_INVENTORY.read_text(encoding="utf-8"))
        declarations = manifest["publicDeclarations"]
        owner_types = {
            row["exportName"].rsplit("::", 1)[-1]
            for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC and row.get("kind") == "type"
        }
        inventory_rows = inventory["declarations"] + inventory["conditionalDeclarations"]
        trait_exports = {
            row["exportName"]
            for row in inventory_rows
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("kind") == "associated-item"
            and any(
                row["exportName"].startswith(f"<kmipkit_protocol::{name} as ")
                for name in owner_types
            )
        }
        self.assertEqual(222, len(trait_exports), "the reviewed T092 trait slice changed")

        exclusions = {
            row.get("exportName"): row
            for row in manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == OWNER_SPEC
        }
        unaccounted = sorted(trait_exports - exclusions.keys())
        self.assertEqual(
            0,
            len(unaccounted),
            "unaccounted exact KMIPKIT-0008 Rust trait exports "
            f"({len(unaccounted)}): {unaccounted[:5]}",
        )

        for export_name in sorted(trait_exports):
            exclusion = exclusions[export_name]
            self.assertTrue(export_name.startswith("<kmipkit_protocol::"))
            self.assertIn(" as ", export_name)
            self.assertTrue(_is_nonblank_string(exclusion.get("reason")))
            self.assertTrue(_is_nonblank_string(exclusion.get("evidence")))
            self.assertTrue(_has_approved_source(exclusion))
            self.assertEqual(
                "specs/008-credentials-attestation/spec.md",
                exclusion["approvalSource"]["path"],
            )

        retained_trait_declarations = {
            row.get("exportName")
            for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC
            and row.get("exportName") in trait_exports
        }
        self.assertEqual(set(), retained_trait_declarations)

        meaningful = [row for row in declarations if row.get("candidateSpecId") == OWNER_SPEC]
        self.assertEqual(95, len(meaningful))
        self.assertTrue(all(row.get("declarationState") == "blocked" for row in meaningful))

    def test_every_exact_owner_declaration_is_mapped_or_reviewedly_excluded(self) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = [
            row
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertTrue(declarations, f"no declarations found for {OWNER_SPEC}")

        exclusions = {
            (row.get("packageId"), row.get("exportName")): row
            for row in manifest.get("approvedExclusions", [])
        }
        unresolved: list[str] = []

        for declaration in declarations:
            key = (declaration.get("packageId"), declaration.get("exportName"))
            mappings = declaration.get("languageMappings")
            complete_mapping = (
                isinstance(mappings, dict)
                and set(mappings) == LANGUAGES
                and all(
                    isinstance(mappings[language], dict)
                    and _is_nonblank_string(mappings[language].get("packageId"))
                    and _is_nonblank_string(mappings[language].get("name"))
                    for language in LANGUAGES
                )
                and _has_approved_source(declaration)
            )

            exclusion = exclusions.get(key)
            reviewed_exclusion = (
                isinstance(exclusion, dict)
                and exclusion.get("candidateSpecId") == OWNER_SPEC
                and _is_nonblank_string(exclusion.get("reason"))
                and _is_nonblank_string(exclusion.get("evidence"))
                and _has_approved_source(exclusion)
            )

            if declaration.get("declarationState") == "blocked" or not (
                complete_mapping or reviewed_exclusion
            ):
                unresolved.append(
                    f"{declaration.get('packageId')}:{declaration.get('exportName')}"
                )

        self.assertEqual(
            0,
            len(unresolved),
            f"unresolved {OWNER_SPEC} declarations ({len(unresolved)} rows): "
            f"{unresolved[:5]}",
        )


if __name__ == "__main__":
    unittest.main()
