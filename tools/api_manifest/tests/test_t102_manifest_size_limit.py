"""The bounded manifest input limit accommodates reviewed declaration batches."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.api_manifest import generate

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
EXPECTED_MANIFEST_LIMIT = 56 * 1024 * 1024


class T102ManifestSizeLimitTests(unittest.TestCase):
    def test_updated_manifest_fits_bounded_input_limit(self) -> None:
        self.assertEqual(generate.MAX_MANIFEST_BYTES, EXPECTED_MANIFEST_LIMIT)
        self.assertLessEqual(MANIFEST_PATH.stat().st_size, generate.MAX_MANIFEST_BYTES)

    def test_manifest_loader_rejects_a_file_one_byte_over_limit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            oversized_manifest = Path(temporary_directory) / "oversized.json"
            with oversized_manifest.open("wb") as output:
                output.truncate(generate.MAX_MANIFEST_BYTES + 1)

            with self.assertRaisesRegex(generate.ManifestError, "supported file size"):
                generate._load_json(
                    ROOT,
                    oversized_manifest,
                    generate.MAX_MANIFEST_BYTES,
                )


if __name__ == "__main__":
    unittest.main()
