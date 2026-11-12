import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).parents[1] / "tools" / "compare_route_ledger.py"
SPEC = importlib.util.spec_from_file_location("compare_route_ledger", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
COMPARE_ROUTE_LEDGER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COMPARE_ROUTE_LEDGER)


class RouteLedgerComparisonTests(unittest.TestCase):
    def test_accepts_equal_coverage_when_build_artifact_hashes_differ(self):
        reference = {
            "artifact_sha256": "a" * 64,
            "catalog_sha256": "c" * 64,
            "executed_cells": 8398,
            "pending_cells": 63105,
        }
        generated = {
            "artifact_sha256": "b" * 64,
            "catalog_sha256": "c" * 64,
            "executed_cells": 8398,
            "pending_cells": 63105,
        }

        self.assertTrue(COMPARE_ROUTE_LEDGER.compare_ledgers(reference, generated))

    def test_rejects_coverage_drift(self):
        reference = {
            "artifact_sha256": "a" * 64,
            "catalog_sha256": "c" * 64,
            "executed_cells": 8398,
            "pending_cells": 63105,
        }
        generated = {
            "artifact_sha256": "b" * 64,
            "catalog_sha256": "c" * 64,
            "executed_cells": 8397,
            "pending_cells": 63106,
        }

        with self.assertRaises(ValueError):
            COMPARE_ROUTE_LEDGER.compare_ledgers(reference, generated)


if __name__ == "__main__":
    unittest.main()
