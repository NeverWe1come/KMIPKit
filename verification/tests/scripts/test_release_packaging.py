"""Unit tests for KMIPKit release packaging and verification scripts."""

from __future__ import annotations

import json
import shutil
import tarfile
import tempfile
import unittest
from pathlib import Path

from verification.release.generate_release_metadata import (
    generate_cyclonedx_sbom,
    generate_release_notes,
    write_sha256_sums,
)
from verification.release.package_c_sdk import (
    package_c_sdk,
    render_cmake_config,
    render_pkg_config,
)
from verification.release.package_proxy import archive_stem, resolve_variants
from verification.release.verify_version import parse_semver, verify_repo_version

REPO_ROOT = Path(__file__).resolve().parents[3]


class VerifyVersionTests(unittest.TestCase):
    def test_current_repository_has_unified_semver(self) -> None:
        version = verify_repo_version(REPO_ROOT)
        self.assertEqual(parse_semver(version), (1, 0, 0))

    def test_parse_semver_rejects_prerelease_and_prefixes(self) -> None:
        for invalid in ("v1.0.0", "1.0.0-alpha.1", "1.0", "01.0.0", "1.0.0+build"):
            with self.assertRaises(ValueError):
                parse_semver(invalid)


class PackageCSdkTests(unittest.TestCase):
    def test_packages_headers_libs_cmake_and_pkgconfig(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            target_dir = tmp_path / "target" / "release"
            target_dir.mkdir(parents=True)
            (target_dir / "libkmipkit_ffi.so").write_bytes(b"ELF_MOCK_SO")
            (target_dir / "libkmipkit_ffi.a").write_bytes(b"AR_MOCK_A")
            output_dir = tmp_path / "dist"

            archive = package_c_sdk(
                repo_root=REPO_ROOT,
                target_dir=target_dir,
                output_dir=output_dir,
                version="1.0.0",
                target="linux-glibc-x86_64",
                smoke_test=False,
            )
            self.assertTrue(archive.is_file())
            with tarfile.open(archive, "r:gz") as tar:
                names = set(tar.getnames())

            prefix = "kmipkit-c-1.0.0-linux-glibc-x86_64"
            self.assertIn(f"{prefix}/include/kmipkit.h", names)
            self.assertIn(f"{prefix}/lib/libkmipkit_ffi.so", names)
            self.assertIn(f"{prefix}/lib/libkmipkit_ffi.a", names)
            self.assertIn(f"{prefix}/lib/cmake/KMIPKit/KMIPKitConfig.cmake", names)
            self.assertIn(f"{prefix}/lib/cmake/KMIPKit/KMIPKitConfigVersion.cmake", names)
            self.assertIn(f"{prefix}/lib/pkgconfig/kmipkit.pc", names)

    def test_rendered_cmake_and_pc_contain_expected_targets(self) -> None:
        cmake = render_cmake_config("windows-x86_64-msvc")
        self.assertIn("KMIPKit::kmipkit_ffi", cmake)
        self.assertIn("kmipkit_ffi.dll.lib", cmake)
        pc = render_pkg_config("1.0.0", "linux-glibc-aarch64")
        self.assertIn("Version: 1.0.0", pc)
        self.assertIn("-lkmipkit_ffi", pc)


class PackageProxyTests(unittest.TestCase):
    def test_resolve_variants_all_and_subset(self) -> None:
        all_v = resolve_variants("all")
        self.assertEqual(len(all_v), 11)
        self.assertIn("all-clouds", all_v)
        self.assertIn("vault", all_v)
        self.assertEqual(resolve_variants("aws, gcp"), ["aws", "gcp"])
        with self.assertRaises(ValueError):
            resolve_variants("aws,nonexistent")

    def test_archive_stem_naming(self) -> None:
        self.assertEqual(
            archive_stem("all-clouds", "1.0.0", "linux-glibc-x86_64"),
            "kmipkit-proxy-1.0.0-linux-glibc-x86_64",
        )
        self.assertEqual(
            archive_stem("aws", "1.0.0", "linux-glibc-x86_64"),
            "kmipkit-proxy-aws-1.0.0-linux-glibc-x86_64",
        )


class ReleaseMetadataTests(unittest.TestCase):
    def test_generates_deterministic_sums_sbom_and_notes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            artifacts = tmp_path / "artifacts"
            artifacts.mkdir()
            (artifacts / "a.tar.gz").write_bytes(b"payload-a")
            (artifacts / "b.whl").write_bytes(b"payload-b")

            sums_path = artifacts / "SHA256SUMS.txt"
            entries = write_sha256_sums(artifacts, sums_path)
            self.assertEqual(len(entries), 2)
            self.assertIn("a.tar.gz", sums_path.read_text(encoding="utf-8"))

            sbom_path = artifacts / "kmipkit-1.0.0-sbom.cdx.json"
            sbom = generate_cyclonedx_sbom(REPO_ROOT, "1.0.0", sbom_path)
            self.assertEqual(sbom["bomFormat"], "CycloneDX")
            parsed = json.loads(sbom_path.read_text(encoding="utf-8"))
            self.assertGreater(len(parsed["components"]), 10)

            notes_path = artifacts / "RELEASE_NOTES.md"
            generate_release_notes("1.0.0", entries, notes_path)
            notes = notes_path.read_text(encoding="utf-8")
            self.assertIn("KMIPKit v1.0.0", notes)
            self.assertIn("a.tar.gz", notes)


if __name__ == "__main__":
    unittest.main()
