import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "tools" / "compare_route_ledger.py"
SPEC = importlib.util.spec_from_file_location("compare_route_ledger", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
COMPARE_ROUTE_LEDGER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COMPARE_ROUTE_LEDGER)


def run_compare_cli(reference: Path, generated: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--reference",
            str(reference),
            "--generated",
            str(generated),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


class RouteLedgerComparisonTests(unittest.TestCase):
    def test_cli_does_not_accept_path_overrides_even_inside_allowed_roots(self):
        ledger = {
            "artifact_sha256": "a" * 64,
            "catalog_sha256": "c" * 64,
            "executed_cells": 8398,
            "pending_cells": 63105,
        }
        build_root = SCRIPT.parents[1] / "build"
        build_root_existed = build_root.exists()
        build_root.mkdir(exist_ok=True)
        try:
            with tempfile.TemporaryDirectory(dir=SCRIPT.parents[1]) as allowed_directory:
                with tempfile.TemporaryDirectory(dir=build_root) as generated_directory:
                    reference = Path(allowed_directory) / "reference.json"
                    generated = Path(generated_directory) / "generated.json"
                    reference.write_text(json.dumps(ledger), encoding="utf-8")
                    generated.write_text(json.dumps(ledger), encoding="utf-8")

                    process = run_compare_cli(reference, generated)
        finally:
            if not build_root_existed:
                build_root.rmdir()

        self.assertNotEqual(process.returncode, 0)
        self.assertIn("unrecognized arguments", process.stderr)

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

    def test_reference_refresh_uses_validated_coverage_and_keeps_artifact_hash_build_specific(self):
        reference = {
            "artifact_sha256": "a" * 64,
            "catalog_sha256": "c" * 64,
            "route_manifest_sha256": "d" * 64,
            "planned_cells": 3,
            "executed_cells": 1,
            "pending_cells": 2,
        }
        generated = {
            "artifact_sha256": "b" * 64,
            "catalog_sha256": "c" * 64,
            "route_manifest_sha256": "e" * 64,
            "planned_cells": 3,
            "executed_cells": 2,
            "pending_cells": 1,
            "all_invocations_passed": True,
        }

        refreshed = COMPARE_ROUTE_LEDGER.refresh_reference(reference, generated)

        self.assertEqual("a" * 64, refreshed["artifact_sha256"])
        self.assertEqual("e" * 64, refreshed["route_manifest_sha256"])
        self.assertEqual(2, refreshed["executed_cells"])
        self.assertEqual(1, refreshed["pending_cells"])

    def test_reference_refresh_rejects_failed_invocations(self):
        reference = {"artifact_sha256": "a" * 64}
        generated = {"artifact_sha256": "b" * 64, "all_invocations_passed": False}

        with self.assertRaisesRegex(ValueError, "failed invocations"):
            COMPARE_ROUTE_LEDGER.refresh_reference(reference, generated)

    def test_reference_writer_rejects_paths_outside_validator_root(self):
        original_path = COMPARE_ROUTE_LEDGER.REFERENCE_LEDGER
        with tempfile.TemporaryDirectory() as outside_directory:
            outside_path = Path(outside_directory) / "route-evidence-ledger.json"
            outside_path.write_text("unchanged", encoding="utf-8")
            COMPARE_ROUTE_LEDGER.REFERENCE_LEDGER = outside_path
            try:
                with self.assertRaisesRegex(ValueError, "outside the validator root"):
                    COMPARE_ROUTE_LEDGER._write_reference_ledger({"artifact_sha256": "a" * 64})
            finally:
                COMPARE_ROUTE_LEDGER.REFERENCE_LEDGER = original_path

            self.assertEqual("unchanged", outside_path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
