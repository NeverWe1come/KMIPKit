"""T097 contract for KMIPKIT-0016 attribute-operation declarations."""

from __future__ import annotations

import hashlib
import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
OWNER_SPEC = "KMIPKIT-0016"
BASE_CANDIDATE_COUNT = 708
BASE_CANDIDATE_SHA256 = "fe521c532063ca63bf89a70930e1880ab0b5cddcaecf95a04e6e10bb71acbcce"
BASE_TRAIT_COUNT = 524
BASE_TRAIT_SHA256 = "86e73fe96baf9ee1249010f7cfa8d927e18d22cbf48fb6a86df4a87b8cfe2554"


def _identity_hash(rows: list[dict[str, object]]) -> str:
    identities = sorted(f"{row['packageId']}|{row['exportName']}" for row in rows)
    return hashlib.sha256("\n".join(identities).encode("utf-8")).hexdigest()


class T097AttributeParityManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))

    def test_base_candidate_inventory_is_pinned(self) -> None:
        candidates = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertEqual(BASE_CANDIDATE_COUNT, len(candidates))
        self.assertEqual(BASE_CANDIDATE_SHA256, _identity_hash(candidates))
        traits = [
            row
            for row in candidates
            if row["exportName"].startswith("<") and " as " in row["exportName"]
        ]
        self.assertEqual(BASE_TRAIT_COUNT, len(traits))
        self.assertEqual(BASE_TRAIT_SHA256, _identity_hash(traits))

    def test_no_attribute_owner_candidates_remain_blocked(self) -> None:
        unresolved = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("candidateSpecId") == OWNER_SPEC
        ]
        self.assertEqual(
            0,
            len(unresolved),
            "all exact KMIPKIT-0016 exports must be mapped or specifically excluded; "
            f"unresolved={[(row['packageId'], row['exportName']) for row in unresolved[:8]]}",
        )


if __name__ == "__main__":
    unittest.main()
