"""Contracts for the minimum Rustls release patched for current advisories."""

from __future__ import annotations

import csv
import re
import tomllib
import unittest
from pathlib import Path
from typing import Any


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
MANIFEST = REPOSITORY_ROOT / "Cargo.toml"
LOCKFILE = REPOSITORY_ROOT / "Cargo.lock"
TRACEABILITY = REPOSITORY_ROOT / "specification" / "compliance" / "requirements" / "KMIPKIT-0045.csv"
PATCHED_FLOOR = (0, 23, 46)


def workspace_rustls_dependency() -> dict[str, Any]:
    manifest = tomllib.loads(MANIFEST.read_text(encoding="utf-8"))
    return manifest["workspace"]["dependencies"]["rustls"]


def version_components(version: str) -> tuple[int, ...]:
    return tuple(int(part) for part in version.removeprefix("=").split("."))


class RustlsAdvisoryPinTests(unittest.TestCase):
    def test_workspace_pin_is_exact_stable_patched_release(self) -> None:
        rustls = workspace_rustls_dependency()
        version = rustls["version"]

        self.assertRegex(version, r"^=0\.23\.\d+$", "Rustls must remain an exact stable 0.23.x pin.")
        self.assertGreaterEqual(
            version_components(version),
            PATCHED_FLOOR,
            "Rustls must meet the >=0.23.46 patched floor for RUSTSEC-2026-0346 through -0349.",
        )

    def test_lockfile_matches_the_exact_workspace_pin(self) -> None:
        lockfile = tomllib.loads(LOCKFILE.read_text(encoding="utf-8"))
        expected_version = workspace_rustls_dependency()["version"].removeprefix("=")
        resolved = [package["version"] for package in lockfile["package"] if package["name"] == "rustls"]

        self.assertEqual(resolved, [expected_version], "Cargo.lock must resolve only the exact workspace Rustls pin.")

    def test_workspace_features_preserve_the_approved_backend(self) -> None:
        rustls = workspace_rustls_dependency()

        self.assertIs(rustls["default-features"], False)
        self.assertEqual(set(rustls["features"]), {"aws_lc_rs", "std"})

    def test_traceability_covers_every_functional_requirement(self) -> None:
        with TRACEABILITY.open(encoding="utf-8", newline="") as source:
            rows = list(csv.DictReader(source))

        self.assertEqual({row["requirement_id"] for row in rows}, {f"FR-{number:03d}" for number in range(1, 6)})
        self.assertTrue(all(row["configuration_path"] and row["implementation_path"] and row["test_reference"] for row in rows))
        self.assertTrue(all(re.fullmatch(r"FR-\d{3}", row["requirement_id"]) for row in rows))


    def test_fr004_traceability_references_tls_policy_tests(self) -> None:
        with TRACEABILITY.open(encoding="utf-8", newline="") as source:
            rows = list(csv.DictReader(source))
        fr004 = next(row for row in rows if row["requirement_id"] == "FR-004")
        references = set(fr004["test_reference"].split(";"))
        tls_policy_tests = {
            "crates/kmipkit-transport/tests/unit/tls_policy_tests.rs::caller_ca_and_matching_mtls_identity_build_a_tls13_only_config",
            "crates/kmipkit-transport/tests/unit/tls_policy_tests.rs::untrusted_chain_expired_leaf_and_hostname_mismatch_fail_closed",
        }

        self.assertTrue(tls_policy_tests.issubset(references), "FR-004 must link TLS 1.3, mTLS, and peer-validation tests.")


if __name__ == "__main__":
    unittest.main()
