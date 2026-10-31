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
    / "rust-export-report-2026-10-31.json"
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
        owner_paths = {
            row["exportName"]: tuple(json.loads(inventory_row["identity"])[3])
            for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC and row.get("kind") == "type"
            for inventory_row in inventory["declarations"]
            if inventory_row.get("exportName") == row["exportName"]
            and isinstance(inventory_row.get("identity"), str)
        }
        self.assertEqual(17, len(owner_paths), "the credential owner type inventory changed")

        inventory_rows = inventory["declarations"] + inventory["conditionalDeclarations"]
        trait_exports: set[str] = set()
        trait_path_rows = [
            row
            for row in declarations + manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == OWNER_SPEC
            and isinstance(row.get("exportName"), str)
            and row["exportName"].startswith("<kmipkit_protocol::")
            and " as " in row["exportName"]
            and ">::" in row["exportName"]
        ]
        trait_exports = {row["exportName"] for row in trait_path_rows}
        self.assertEqual(222, len(trait_exports), "the exact T092 trait declarations changed")

        for export_name in sorted(trait_exports):
            owner_expression, trait_member = export_name[1:].split(" as ", 1)
            owner_export = owner_expression.split("<", 1)[0]
            self.assertIn(owner_export, owner_paths)
            owner_path = owner_paths[owner_export]
            matching_rows = [
                row
                for row in inventory_rows
                if row.get("packageId") == "rust-kmipkit-protocol"
                and row.get("exportName") == export_name
            ]
            self.assertEqual(1, len(matching_rows), f"Rust inventory identity for {export_name}")
            observed = matching_rows[0]
            self.assertEqual("associated-item", observed.get("kind"))

            identity = (
                json.loads(observed["identity"])
                if isinstance(observed.get("identity"), str)
                else None
            )
            direct_trait = (
                isinstance(identity, list)
                and len(identity) >= 9
                and identity[0] == "rust-trait-implementation"
                and identity[1] == "rust-kmipkit-protocol"
                and identity[2] == list(owner_path)
                and isinstance(identity[3], list)
                and bool(identity[3])
            )
            conditional_traits = [
                predicate.get("traitPath")
                for predicate in observed.get("condition", {}).get("predicates", [])
                if isinstance(predicate, dict)
                and predicate.get("kind") == "traitBound"
                and isinstance(predicate.get("traitPath"), list)
                and predicate.get("traitPath")
            ]
            conditional_trait = (
                observed in inventory["conditionalDeclarations"]
                and observed.get("ownerPath") == list(owner_path)
                and bool(conditional_traits)
                and _is_nonblank_string(observed.get("implementationId"))
            )
            self.assertTrue(
                direct_trait or conditional_trait,
                f"Rust inventory does not identify {export_name} as a trait member",
            )

            exported_trait, exported_member = trait_member.rsplit(">::", 1)
            self.assertTrue(exported_member)
            if direct_trait:
                self.assertEqual("::".join(identity[3]), exported_trait.split("<", 1)[0])
                self.assertEqual(identity[7], exported_member)
            trait_exports.add(export_name)
        self.assertEqual(222, len(trait_exports), "the reviewed T092 trait slice changed")

        owner_exclusions = [
            row
            for row in manifest.get("approvedExclusions", [])
            if row.get("packageId") == "rust-kmipkit-protocol"
            and row.get("candidateSpecId") == OWNER_SPEC
        ]
        exclusion_counts: dict[str, int] = {}
        for row in owner_exclusions:
            name = row.get("exportName")
            if isinstance(name, str):
                exclusion_counts[name] = exclusion_counts.get(name, 0) + 1
        duplicates = sorted(name for name, count in exclusion_counts.items() if count != 1)
        self.assertEqual([], duplicates, "duplicate exact KMIPKIT-0008 exclusions")
        actual_exclusions = {
            row["exportName"]: row
            for row in owner_exclusions
            if isinstance(row.get("exportName"), str)
        }
        unaccounted = sorted(trait_exports - actual_exclusions.keys())
        extra_exclusions = sorted(actual_exclusions.keys() - trait_exports)
        self.assertEqual(
            0,
            len(unaccounted) + len(extra_exclusions),
            "unaccounted exact KMIPKIT-0008 Rust trait exports "
            f"({len(unaccounted)}); extra exclusions ({len(extra_exclusions)}): "
            f"{unaccounted[:5]} / {extra_exclusions[:5]}",
        )

        for export_name in sorted(trait_exports):
            exclusion = actual_exclusions[export_name]
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

        meaningful = [
            row for row in declarations
            if row.get("candidateSpecId") == OWNER_SPEC
            and row.get("exportName") not in trait_exports
        ]
        self.assertEqual(95, len(meaningful))
        self.assertTrue(all(row.get("declarationState") == "blocked" for row in meaningful))

    def test_every_nonblocked_owner_declaration_is_mapped_or_reviewedly_excluded(
        self,
    ) -> None:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        declarations = [
            row
            for row in manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertTrue(declarations, f"no declarations found for {OWNER_SPEC}")

        blocked = [
            declaration
            for declaration in declarations
            if declaration.get("declarationState") == "blocked"
        ]
        self.assertEqual(
            95,
            len(blocked),
            "the expected 95 functional KMIPKIT-0008 declarations must remain blocked",
        )

        exclusions = {
            (row.get("packageId"), row.get("exportName")): row
            for row in manifest.get("approvedExclusions", [])
        }
        unresolved: list[str] = []

        for declaration in declarations:
            if declaration.get("declarationState") == "blocked":
                continue

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

            if not (complete_mapping or reviewed_exclusion):
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
