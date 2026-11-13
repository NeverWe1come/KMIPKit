"""Unit and contract tests for KMIPKit release packaging dry-run tools."""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
WORKFLOWS = ROOT / ".github" / "workflows"
WORKFLOW_FILE = WORKFLOWS / "release-dry-run.yml"

try:
    from tools.release_packaging import dry_run
    from tools.release_packaging import proxy_dry_run
except ImportError:
    dry_run = None
    proxy_dry_run = None


class ReleasePackagingContractTests(unittest.TestCase):
    def test_dry_run_module_imported(self) -> None:
        self.assertIsNotNone(dry_run, "tools.release_packaging.dry_run must be importable")

    def test_verify_metadata_passes_on_current_repo(self) -> None:
        self.assertIsNotNone(dry_run)
        report = dry_run.verify_metadata(ROOT)
        self.assertTrue(report["valid"], f"Metadata verification failed: {report.get('errors')}")
        self.assertEqual("1.0.0", report["version"])
        self.assertEqual(8, len(report["packages"]))

    def test_workflow_security_and_action_pinning(self) -> None:
        self.assertTrue(WORKFLOW_FILE.is_file(), f"{WORKFLOW_FILE} must exist")
        content = WORKFLOW_FILE.read_text(encoding="utf-8")
        
        # Check permissions
        self.assertIn("permissions:", content)
        self.assertIn("contents: read", content)
        
        # Check action pin pattern: uses: owner/repo@sha40 # owner/repo vX.Y.Z
        action_lines = [line.strip() for line in content.splitlines() if line.strip().startswith("uses:")]
        self.assertTrue(action_lines, "Workflow must contain pinned action steps")
        for line in action_lines:
            self.assertRegex(
                line,
                r"^uses:\s+[\w\.\-]+/[\w\.\-]+(?:/[\w\.\-]+)*@[0-9a-f]{40}\s+#\s+.+",
                f"Action invocation must be pinned to 40-char SHA with comment: {line}"
            )

    def test_c_distribution_manifest_template(self) -> None:
        self.assertIsNotNone(dry_run)
        cmake_config = dry_run.generate_cmake_config("1.0.0")
        self.assertIn("set(KMIPKit_VERSION \"1.0.0\")", cmake_config)
        self.assertIn("add_library(KMIPKit::kmipkit_ffi", cmake_config)

        pkg_config = dry_run.generate_pkg_config("1.0.0", "/usr/local")
        self.assertIn("Version: 1.0.0", pkg_config)
        self.assertIn("-lkmipkit_ffi", pkg_config)

    def test_proxy_manifest_is_stable_and_hashes_all_packaged_files(self) -> None:
        self.assertIsNotNone(proxy_dry_run)
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "input"
            source.mkdir()
            image_names = ["universal", *proxy_dry_run.PROVIDERS]
            for name in image_names:
                (source / f"kmipkit-proxy-{name}.tar").write_bytes(name.encode())
            (source / "kmipkit-proxy-1.0.0.tgz").write_bytes(b"helm chart")

            report = proxy_dry_run.assemble_proxy_manifest(
                source, root / "output", "1.0.0"
            )

            self.assertEqual(12, report["artifacts_count"])
            self.assertEqual(sorted(entry["path"] for entry in report["artifacts"]), [
                "kmipkit-proxy-1.0.0.tgz",
                *sorted(f"kmipkit-proxy-{name}.tar" for name in image_names),
            ])
            self.assertTrue((root / "output" / "SHA256SUMS").is_file())
            self.assertTrue((root / "output" / "proxy-artifacts-manifest.json").is_file())

    def test_proxy_manifest_rejects_an_incomplete_package_set(self) -> None:
        self.assertIsNotNone(proxy_dry_run)
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "input"
            source.mkdir()
            (source / "kmipkit-proxy-universal.tar").write_bytes(b"only one image")
            with self.assertRaisesRegex(ValueError, "artifact set mismatch"):
                proxy_dry_run.assemble_proxy_manifest(source, root / "output", "1.0.0")

    def test_proxy_release_workflow_builds_all_variants_without_publishing(self) -> None:
        workflow = WORKFLOWS / "proxy-release-dry-run.yml"
        self.assertTrue(workflow.is_file(), f"{workflow} must exist")
        content = workflow.read_text(encoding="utf-8")
        for provider in (
            "alibaba", "aws", "azure", "gcp", "ibm", "infisical",
            "oci", "otc", "scaleway", "vault",
        ):
            self.assertIn(f"variant: {provider}", content)
        self.assertIn("variant: universal", content)
        self.assertIn("helm package deploy/helm/kmipkit-proxy", content)
        self.assertIn("proxy_dry_run.py", content)
        self.assertNotIn("docker push", content)
        self.assertNotIn("helm push", content)


if __name__ == "__main__":
    unittest.main()
