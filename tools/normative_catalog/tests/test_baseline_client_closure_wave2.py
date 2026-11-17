"""Wave 2 (KMIPKIT-0123) tests for Baseline Client catalog closure."""

from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path

from tools.normative_catalog import report as report_module
from tools.normative_catalog import validate

ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification" / "catalog" / "kmip-2.1.json"


class BaselineClientClosureWave2Tests(unittest.TestCase):
    def test_baseline_client_has_no_catalog_or_discrepancy_blockers(self) -> None:
        raw_bytes = CATALOG_PATH.read_bytes()
        catalog = validate.load_validated_catalog(raw_bytes, ROOT, require_complete=True)

        baseline = next(
            p for p in catalog["profiles"] if p["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        self.assertEqual("evidence_complete", baseline["claim_state"])

        discrepancies = {d["discrepancy_id"]: d for d in catalog["discrepancies"]}
        for disc_id in (
            "KMIPKIT-DISC-003",
            "KMIPKIT-DISC-013",
            "KMIPKIT-DISC-014",
            "KMIPKIT-DISC-019",
            "KMIPKIT-DISC-042",
        ):
            self.assertEqual(
                "resolved_by_approved_decision",
                discrepancies[disc_id]["state"],
                f"{disc_id} must be resolved_by_approved_decision",
            )
            self.assertIsNotNone(discrepancies[disc_id]["decision_id"])

        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = report_module.main(["--readiness", "--repo-root", str(ROOT)])
        self.assertEqual(0, rc)
        output = buf.getvalue()
        self.assertNotIn("optional client capability source mapping incomplete", output)
        self.assertNotIn("unresolved source discrepancy", output)


if __name__ == "__main__":
    unittest.main()
