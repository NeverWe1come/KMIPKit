"""Contract tests for the local Cosmian integration deployment."""

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
COMPOSE_FILE = REPOSITORY_ROOT / "tests" / "integration" / "cosmian" / "compose.yaml"
EXPECTED_IMAGES = {
    "alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412",
    "ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e",
}


class CosmianComposeTests(unittest.TestCase):
    def test_every_service_image_is_pinned_by_digest(self) -> None:
        compose = COMPOSE_FILE.read_text(encoding="utf-8")
        image_references = re.findall(r"^\s+image:\s+(\S+)\s*$", compose, re.MULTILINE)

        self.assertTrue(image_references, "the Cosmian compose file declares service images")
        self.assertEqual(set(image_references), EXPECTED_IMAGES)
        for image_reference in image_references:
            with self.subTest(image=image_reference):
                self.assertRegex(image_reference, r"^[^@\s]+@sha256:[0-9a-f]{64}$")


if __name__ == "__main__":
    unittest.main()
