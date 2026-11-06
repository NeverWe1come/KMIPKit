"""Require every ready 0053 mapping target to be listed as an exact export."""

from __future__ import annotations

import unittest
from pathlib import Path

from tools.api_manifest import generate


ROOT = Path(__file__).resolve().parents[3]


class MappingTargetInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = generate._load_json(
            ROOT, generate.MANIFEST_PATH, generate.MAX_MANIFEST_BYTES
        )

    def test_every_ready_mapping_resolves_to_an_exact_listed_export(self) -> None:
        packages = {
            package["id"]: package["language"]
            for package in self.manifest["publicPackages"]
        }
        c_release_functions = {
            ("c-kmipkit", type_record["c"]["releaseFunction"])
            for type_record in self.manifest.get("types", [])
            if isinstance(type_record, dict)
            and isinstance(type_record.get("c"), dict)
            and isinstance(type_record["c"].get("releaseFunction"), str)
        }

        generate._validate_ready_mapping_targets(
            self.manifest["publicDeclarations"], packages, c_release_functions
        )


if __name__ == "__main__":
    unittest.main()
