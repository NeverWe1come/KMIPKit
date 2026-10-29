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


class CosmianComposeTests(unittest.TestCase):
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
        shell = shutil.which("sh")
        self.assertIsNotNone(shell)

        with tempfile.TemporaryDirectory(prefix="kmipkit-cosmian-pki-") as temporary:
            output_dir = Path(temporary) / "certs"
            existing_file = output_dir / "ca" / "keep.txt"
            existing_file.parent.mkdir(parents=True)
            existing_file.write_text("preserve me\n", encoding="utf-8")
            environment = os.environ.copy()
            environment["KMIPKIT_PKI_OUTPUT_DIR"] = str(output_dir)

            result = subprocess.run(
                [shell, str(PKI_GENERATOR)],
                cwd=REPOSITORY_ROOT,
                env=environment,
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Refusing to clean an unowned PKI output directory", result.stderr)
            self.assertEqual(existing_file.read_text(encoding="utf-8"), "preserve me\n")


if __name__ == "__main__":
    unittest.main()
