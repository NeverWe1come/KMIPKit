"""Text encoding contracts for the reviewed public API manifest."""

from __future__ import annotations

import unittest
from pathlib import Path


class ManifestEncodingTests(unittest.TestCase):
    def test_manifest_text_has_no_common_utf8_mojibake(self) -> None:
        repository_root = Path(__file__).resolve().parents[3]
        manifest = (repository_root / "specification/api/public-api.json").read_text(
            encoding="utf-8"
        )

        common_mojibake = (
            "\u00c2\u00a7",       # UTF-8 section sign decoded as Windows-1252
            "\u00c3\u201a\u00c2\u00a7",  # section sign decoded twice
            "\u00e2\u20ac",       # UTF-8 typographic punctuation decoded as Windows-1252
            "\u00c3\u0192",       # common second-pass UTF-8 corruption
        )
        has_mojibake = any(token in manifest for token in common_mojibake)
        self.assertFalse(has_mojibake)


if __name__ == "__main__":
    unittest.main()
