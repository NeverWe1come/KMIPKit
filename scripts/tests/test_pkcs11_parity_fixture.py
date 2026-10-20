"""Checks the shared PKCS#11 response parity contract."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = REPOSITORY_ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"


class Pkcs11ParityFixtureTests(unittest.TestCase):
    def test_fixture_records_ordered_success_response_and_unknown_values(self) -> None:
        fixture = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
        expected = fixture["expectedOutput"]

        self.assertEqual(
            expected["responseTags"],
            [
                "0x0042015A",
                "0x004200D6",
                "0x0042015C",
                "0x0042015D",
                "0x00541234",
            ],
        )
        self.assertEqual(expected["unknownReturnCode"], "0xFFFF1234")
        self.assertEqual(
            expected["vendorExtension"],
            {"tag": "0x00541234", "integer": 7},
        )


if __name__ == "__main__":
    unittest.main()
