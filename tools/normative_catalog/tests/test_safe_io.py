"""Tests for bounded, repository-confined catalog file access."""

from __future__ import annotations

import tempfile
import unittest
import os
import subprocess
import sys
from pathlib import Path

from tools.normative_catalog.safe_io import (
    PathSecurityError,
    _windows_directory_guard,
    atomic_write_bytes,
    safe_read_bytes,
)


class SafeIoTests(unittest.TestCase):
    def test_windows_directory_guard_prevents_parent_rename(self) -> None:
        if os.name != "nt":
            self.skipTest("Windows directory handles are only available on Windows")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            parent = root / "nested"
            parent.mkdir()
            guard = _windows_directory_guard(root, ("nested",))
            guard.__enter__()
            try:
                with self.assertRaises(OSError):
                    parent.rename(root / "moved")
            finally:
                guard.__exit__(None, None, None)

    def test_reads_small_files_and_rejects_files_over_limit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "small.json").write_bytes(b"{}")
            (root / "large.json").write_bytes(b"123456")

            self.assertEqual(safe_read_bytes(root, "small.json", max_bytes=2), b"{}")
            with self.assertRaisesRegex(PathSecurityError, "size limit"):
                safe_read_bytes(root, "large.json", max_bytes=5)

    @unittest.skipUnless(os.name == "posix" and hasattr(os, "mkfifo"), "POSIX FIFO support is required")
    def test_rejects_fifo_without_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            os.mkfifo(root / "input.fifo")
            script = (
                "import sys\n"
                "from pathlib import Path\n"
                "from tools.normative_catalog.safe_io import PathSecurityError, safe_read_bytes\n"
                "try:\n"
                "    safe_read_bytes(Path(sys.argv[1]), 'input.fifo', max_bytes=8)\n"
                "except PathSecurityError:\n"
                "    raise SystemExit(0)\n"
                "raise SystemExit(1)\n"
            )
            repo_root = Path(__file__).resolve().parents[3]
            try:
                result = subprocess.run(
                    [sys.executable, "-c", script, str(root)],
                    cwd=repo_root,
                    capture_output=True,
                    check=False,
                    timeout=2,
                )
            except subprocess.TimeoutExpired:
                self.fail("safe_read_bytes blocked while opening a FIFO")
            self.assertEqual(result.returncode, 0, result.stderr.decode("utf-8", errors="replace"))

    def test_rejects_symlinked_parent_and_final_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            external = Path(outside)
            (external / "secret.json").write_text("secret", encoding="utf-8")
            try:
                (root / "redirect").symlink_to(external, target_is_directory=True)
                (root / "linked.json").symlink_to(external / "secret.json")
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            with self.assertRaises(PathSecurityError):
                safe_read_bytes(root, "redirect/secret.json", max_bytes=100)
            with self.assertRaises(PathSecurityError):
                safe_read_bytes(root, "linked.json", max_bytes=100)

    def test_rejects_parent_traversal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "nested").mkdir()
            (root / "nested" / "file.json").write_bytes(b"{}")
            for unsafe_path in ("../outside.json", "nested/../nested/file.json", "nested//file.json", "nested/./file.json"):
                with self.subTest(path=unsafe_path), self.assertRaises(PathSecurityError):
                    safe_read_bytes(root, unsafe_path, max_bytes=10)

    def test_atomic_writer_refuses_symlink_output_and_preserves_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            target = Path(outside) / "sentinel.txt"
            target.write_text("unchanged", encoding="utf-8")
            output = root / "report.md"
            try:
                output.symlink_to(target)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            with self.assertRaises(PathSecurityError):
                atomic_write_bytes(root, "report.md", b"overwrite")
            self.assertEqual(target.read_text(encoding="utf-8"), "unchanged")

    def test_atomic_writer_refuses_symlinked_parent(self) -> None:
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            external = Path(outside)
            try:
                (root / "redirect").symlink_to(external, target_is_directory=True)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            with self.assertRaises(PathSecurityError):
                atomic_write_bytes(root, "redirect/report.md", b"overwrite")
            self.assertFalse((external / "report.md").exists())


if __name__ == "__main__":
    unittest.main()
