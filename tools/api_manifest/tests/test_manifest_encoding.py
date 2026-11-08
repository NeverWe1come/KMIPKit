"""Text encoding contracts for the reviewed public API manifest."""

from __future__ import annotations

import unittest
from pathlib import Path


class ManifestEncodingTests(unittest.TestCase):
    def test_normative_section_citations_are_not_utf8_mojibake(self) -> None:
        repository_root = Path(__file__).resolve().parents[3]
        manifest = (repository_root / "specification/api/public-api.json").read_text(
            encoding="utf-8"
        )

        has_mojibake = any(
            token in manifest
            for token in ("\u00c2\u00a7", "\u00c3\u201a\u00c2\u00a7")
        )
        self.assertFalse(has_mojibake)


if __name__ == "__main__":
    unittest.main()
