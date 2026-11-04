"""Pin a disjoint, exhaustive inventory for the T102 macro-lots."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
PARTITION = ROOT / "specs" / "043-public-api-bindings" / "reports" / "t102-macro-lot-partition.json"


class T102MacroLotPartitionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        cls.partition = json.loads(PARTITION.read_text(encoding="utf-8"))

    def test_macro_lots_are_disjoint_exhaustive_and_identity_stable(self) -> None:
        self.assertEqual("KMIPKIT-0043", self.partition["candidateSpecId"])
        self.assertEqual(648, self.partition["baselineBlockedCandidateCount"])
        self.assertEqual(
            {"traitPaths": 58, "errorNameRows": 321, "overlap": 10},
            self.partition["classificationCounts"],
        )
        lot1 = self.partition["macroLots"]["1"]["declarations"]
        lot2 = self.partition["macroLots"]["2"]["declarations"]
        lot1_ids = {item["declarationId"] for item in lot1}
        lot2_ids = {item["declarationId"] for item in lot2}
        self.assertEqual(len(lot1), len(lot1_ids))
        self.assertEqual(len(lot2), len(lot2_ids))
        self.assertTrue(lot1_ids.isdisjoint(lot2_ids))
        self.assertEqual(369, len(lot1_ids))
        self.assertEqual(279, len(lot2_ids))

        public = {
            (row["declarationId"], row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        partition_ids = lot1_ids | lot2_ids
        for item in (*lot1, *lot2):
            identity = (
                item["declarationId"],
                item["packageId"],
                item["exportName"],
            )
            pair = (item["packageId"], item["exportName"])
            self.assertTrue(
                identity in public or pair in exclusions,
                f"candidate identity disappeared from manifest: {identity}",
            )

        blocked = {
            row["declarationId"]
            for row in self.manifest["publicDeclarations"]
            if row.get("declarationState") == "blocked"
            and row.get("candidateSpecId") == "KMIPKIT-0043"
        }
        self.assertTrue(blocked.issubset(partition_ids))
        planned = {
            item["declarationId"]: item for item in (*lot1, *lot2)
        }
        for row in self.manifest["publicDeclarations"]:
            if row.get("declarationState") != "blocked" or row.get("candidateSpecId") != "KMIPKIT-0043":
                continue
            item = planned[row["declarationId"]]
            self.assertEqual(item["packageId"], row["packageId"])
            self.assertEqual(item["exportName"], row["exportName"])

        for item in lot1:
            name = item["exportName"]
            self.assertTrue(" as " in name or "Error" in name, name)
        for item in lot2:
            name = item["exportName"]
            self.assertFalse(" as " in name or "Error" in name, name)


if __name__ == "__main__":
    unittest.main()
