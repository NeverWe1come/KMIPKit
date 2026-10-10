"""Contract tests for the local Cosmian integration deployment."""

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
COMPOSE_FILE = REPOSITORY_ROOT / "tests" / "integration" / "cosmian" / "compose.yaml"


class CosmianComposeTests(unittest.TestCase):
    def test_every_service_image_is_pinned_by_digest(self) -> None:
        compose = COMPOSE_FILE.read_text(encoding="utf-8")
        image_references = re.findall(r"^\s+image:\s+(\S+)\s*$", compose, re.MULTILINE)

        self.assertTrue(image_references, "the Cosmian compose file declares service images")
        for image_reference in image_references:
            with self.subTest(image=image_reference):
                self.assertRegex(image_reference, r"^[^@\s]+@sha256:[0-9a-f]{64}$")


if __name__ == "__main__":
    unittest.main()
