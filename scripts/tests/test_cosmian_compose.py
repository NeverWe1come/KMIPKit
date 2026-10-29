"""Contract tests for the local Cosmian integration deployment."""

import os
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
COMPOSE_FILE = REPOSITORY_ROOT / "tests" / "integration" / "cosmian" / "compose.yaml"
PKI_GENERATOR = REPOSITORY_ROOT / "tests" / "integration" / "cosmian" / "generate-pki.sh"
EXPECTED_IMAGES = {
    "alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412",
    "ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e",
}
PKI_MARKER_NAME = ".kmipkit-cosmian-pki"
PKI_MARKER_CONTENTS = "KMIPKit Cosmian integration PKI v1\n"
REQUIRED_PKI_FILES = (
    "ca/ca.crt",
    "server/server.crt",
    "server/server.key",
    "client/client.crt",
    "client/client.key",
    "validation/intermediate.crt",
    "validation/leaf.crt",
    "validation/private-key-password-no-chain.p12",
    "validation/private-key-password-chain.p12",
    "validation/private-key-empty-password-no-chain.p12",
    "validation/private-key-empty-password-chain.p12",
    "validation/rsa-2048-csr-pkcs10.der",
    "validation/rsa-2048-csr-pem.pem",
    "validation/ec-P256-csr-pkcs10.der",
    "validation/ed25519-csr-pkcs10.der",
    "validation/cryptographic-fixture-capabilities.txt",
    "validation/fixture-set-v3",
)


class CosmianComposeTests(unittest.TestCase):
    def run_pki_generator(self, output_dir: Path, apk_exit: int | None = None) -> subprocess.CompletedProcess[str]:
        environment = os.environ.copy()
        environment["KMIPKIT_PKI_OUTPUT_DIR"] = str(output_dir)
        if apk_exit is not None:
            apk_directory = output_dir.parent / "bin"
            apk_directory.mkdir()
            apk = apk_directory / "apk"
            apk.write_text(f"#!/bin/sh\nexit {apk_exit}\n", encoding="utf-8")
            apk.chmod(0o755)
            environment["PATH"] = f"{apk_directory}{os.pathsep}{environment['PATH']}"
        shell = shutil.which("sh")
        if shell is None:
            self.skipTest("requires a POSIX shell")
        return subprocess.run(
            [shell, str(PKI_GENERATOR)],
            cwd=REPOSITORY_ROOT,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
        )

    def create_complete_pki(self, output_dir: Path, marked: bool) -> None:
        for relative_path in REQUIRED_PKI_FILES:
            fixture = output_dir / relative_path
            fixture.parent.mkdir(parents=True, exist_ok=True)
            fixture.write_text("fixture\n", encoding="utf-8")
        if marked:
            (output_dir / PKI_MARKER_NAME).write_text(PKI_MARKER_CONTENTS, encoding="utf-8")

    def test_every_service_image_is_pinned_by_digest(self) -> None:
        compose = COMPOSE_FILE.read_text(encoding="utf-8")
        image_references = re.findall(r"^\s+image:\s+(\S+)\s*$", compose, re.MULTILINE)

        self.assertTrue(image_references, "the Cosmian compose file declares service images")
        self.assertEqual(set(image_references), EXPECTED_IMAGES)
        for image_reference in image_references:
            with self.subTest(image=image_reference):
                self.assertRegex(image_reference, r"^[^@\s]+@sha256:[0-9a-f]{64}$")

    def test_pki_generator_checks_output_ownership_before_cleanup(self) -> None:
        generator = PKI_GENERATOR.read_text(encoding="utf-8")

        self.assertIn(
            'output_dir="${KMIPKIT_PKI_OUTPUT_DIR:-/output}"',
            generator,
            "the mounted output path must be explicit and testable",
        )
        ownership_guard = generator.find("Refusing to clean an unowned PKI output directory")
        cleanup = generator.find('rm -f "$output_dir/ca/"*')

        self.assertGreaterEqual(ownership_guard, 0, "the generator needs an ownership guard")
        self.assertGreaterEqual(cleanup, 0, "the generator must clean only its known PKI paths")
        self.assertLess(ownership_guard, cleanup, "unowned output must be rejected before cleanup")

    @unittest.skipUnless(os.name == "posix" and shutil.which("sh"), "requires a POSIX shell")
    def test_pki_generator_preserves_nonempty_unowned_output(self) -> None:
        with tempfile.TemporaryDirectory(prefix="kmipkit-cosmian-pki-") as temporary:
            output_dir = Path(temporary) / "certs"
            existing_file = output_dir / "ca" / "keep.txt"
            existing_file.parent.mkdir(parents=True)
            existing_file.write_text("preserve me\n", encoding="utf-8")
            result = self.run_pki_generator(output_dir)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Refusing to clean an unowned PKI output directory", result.stderr)
            self.assertEqual(existing_file.read_text(encoding="utf-8"), "preserve me\n")

    @unittest.skipUnless(os.name == "posix", "requires a POSIX shell")
    def test_pki_generator_rejects_complete_unmarked_output(self) -> None:
        with tempfile.TemporaryDirectory(prefix="kmipkit-cosmian-pki-") as temporary:
            output_dir = Path(temporary) / "certs"
            output_dir.mkdir()
            self.create_complete_pki(output_dir, marked=False)

            result = self.run_pki_generator(output_dir)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Refusing to clean an unowned PKI output directory", result.stderr)

    @unittest.skipUnless(os.name == "posix", "requires a POSIX shell")
    def test_pki_generator_accepts_marked_complete_output(self) -> None:
        with tempfile.TemporaryDirectory(prefix="kmipkit-cosmian-pki-") as temporary:
            output_dir = Path(temporary) / "certs"
            output_dir.mkdir()
            self.create_complete_pki(output_dir, marked=True)

            result = self.run_pki_generator(output_dir)

            self.assertEqual(result.returncode, 0, result.stderr)

    @unittest.skipUnless(os.name == "posix", "requires a POSIX shell")
    def test_pki_generator_refuses_symlinked_subdirectories(self) -> None:
        with tempfile.TemporaryDirectory(prefix="kmipkit-cosmian-pki-") as temporary:
            root = Path(temporary)
            output_dir = root / "certs"
            output_dir.mkdir()
            (output_dir / PKI_MARKER_NAME).write_text(PKI_MARKER_CONTENTS, encoding="utf-8")
            external_directory = root / "external"
            external_directory.mkdir()
            sentinel = external_directory / "keep.txt"
            sentinel.write_text("preserve me\n", encoding="utf-8")
            try:
                (output_dir / "ca").symlink_to(external_directory, target_is_directory=True)
            except OSError as error:
                self.skipTest(f"directory symlinks are unavailable: {error}")
            for directory in ("server", "client", "validation"):
                (output_dir / directory).mkdir()

            result = self.run_pki_generator(output_dir, apk_exit=0)

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Refusing to clean an unowned PKI output directory", result.stderr)
            self.assertEqual(sentinel.read_text(encoding="utf-8"), "preserve me\n")

    @unittest.skipUnless(os.name == "posix", "requires a POSIX shell")
    def test_pki_generator_marks_empty_output_before_tool_install(self) -> None:
        with tempfile.TemporaryDirectory(prefix="kmipkit-cosmian-pki-") as temporary:
            output_dir = Path(temporary) / "certs"
            output_dir.mkdir()

            result = self.run_pki_generator(output_dir, apk_exit=23)

            self.assertEqual(result.returncode, 23)
            self.assertEqual(
                (output_dir / PKI_MARKER_NAME).read_text(encoding="utf-8"),
                PKI_MARKER_CONTENTS,
            )

    def test_pki_output_guard_uses_portable_shell_globs(self) -> None:
        generator = PKI_GENERATOR.read_text(encoding="utf-8")

        self.assertNotIn("-maxdepth", generator)
        self.assertIn('for output_entry in "$output_dir"/*', generator)


if __name__ == "__main__":
    unittest.main()
