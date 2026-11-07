"""Keep the KeyBlock constructor aligned across public language bindings."""

import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
AUDIT = ROOT / "specs/053-api-parity-facade/candidate-resolutions/python-export-audit.json"


class KeyBlockRouteResolutionTests(unittest.TestCase):
    def test_keyblock_from_fields_maps_to_keyblock_constructors_in_each_language(self) -> None:
        document = json.loads(AUDIT.read_text(encoding="utf-8"))
        route = next(
            row for row in document["mappings"]
            if row["name"] == "kmipkit.managed_objects.KeyBlock.from_fields"
        )

        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit",
                    "name": "kmipkit::protocol::KeyBlock::try_from_item",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_key_block_from_fields",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.ttlv.KeyBlock.fromFields",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.managed_objects.KeyBlock.from_fields",
                },
            },
            route["languageMappings"],
        )


if __name__ == "__main__":
    unittest.main()
