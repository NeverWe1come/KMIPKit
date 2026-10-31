"""T092 contract for KMIPKIT-0008 credential declaration parity."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0008"
LANGUAGES = {"rust", "c", "java", "python"}


def _has_approved_source(record: dict[str, object]) -> bool:
    source = record.get("approvalSource")
    return (
        isinstance(source, dict)
        and source.get("status") == "Approved"
        and isinstance(source.get("path"), str)
        and bool(source["path"])
        and isinstance(source.get("evidence"), str)
        and bool(source["evidence"])
    )


class T092CredentialsParityManifestTests(unittest.TestCase):
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
                    and bool(mappings[language].get("packageId"))
                    and bool(mappings[language].get("name"))
                    for language in LANGUAGES
                )
                and _has_approved_source(declaration)
            )

            exclusion = exclusions.get(key)
            reviewed_exclusion = (
                isinstance(exclusion, dict)
                and bool(exclusion.get("reason"))
                and bool(exclusion.get("evidence"))
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
