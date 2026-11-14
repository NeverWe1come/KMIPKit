"""The C direct-operation API must cover the approved KMIP 3.0 set."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
HEADER = ROOT / "bindings/c/include/kmipkit.h"
SYMBOLS = {
    "kmipkit_client_create_credential_v1",
    "kmipkit_client_create_group_v1",
    "kmipkit_client_create_user_v1",
    "kmipkit_client_deactivate_v1",
    "kmipkit_client_decapsulate_v1",
    "kmipkit_client_encapsulate_v1",
    "kmipkit_client_obliterate_v1",
}


class Kmip30CFluentApiTests(unittest.TestCase):
    def test_header_declares_direct_semantic_operations(self) -> None:
        header = HEADER.read_text(encoding="utf-8")
        missing = sorted(symbol for symbol in SYMBOLS if symbol not in header)
        self.assertEqual(missing, [])


if __name__ == "__main__":
    unittest.main()
