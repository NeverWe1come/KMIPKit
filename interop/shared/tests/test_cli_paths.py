"""CLI path and immutable Git input checks used by the shared tools."""
from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1] / "tools"
sys.path.insert(0, str(TOOLS))

import cli_paths  # noqa: E402
from cli_paths import resolve_within, validate_git_commit, validate_git_path  # noqa: E402


class CliPathTests(unittest.TestCase):
    def test_paths_must_resolve_under_an_explicitly_allowed_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.assertEqual((root / "reports/out.json").resolve(),
                             resolve_within(root / "reports/out.json", (root,)))
            with self.assertRaisesRegex(ValueError, "outside the allowed"):
                resolve_within(root.parent / "outside.json", (root,))

    def test_confined_text_writer_creates_only_under_approved_roots(self):
        writer = getattr(cli_paths, "write_text_within", None)
        self.assertTrue(callable(writer), "CLI output needs a tested confined writer")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "repository"
            root.mkdir()
            output = root / "reports" / "ledger.json"
            resolved = writer(output, (root,), "{}\n", encoding="utf-8")
            self.assertEqual(output.resolve(), resolved)
            self.assertEqual("{}\n", resolved.read_text(encoding="utf-8"))

            outside = Path(directory) / "ledger-outside.json"
            with self.assertRaisesRegex(ValueError, "outside the allowed"):
                writer(outside, (root,), "must not be written", encoding="utf-8")
            self.assertFalse(outside.exists())

    def test_git_commit_requires_full_lowercase_object_id(self):
        self.assertEqual("a" * 40, validate_git_commit("a" * 40))
        self.assertEqual("b" * 64, validate_git_commit("b" * 64))
        for commit in ("main", "-c", "A" * 40, "a" * 39, "a" * 41):
            with self.subTest(commit=commit), self.assertRaises(ValueError):
                validate_git_commit(commit)

    def test_git_paths_are_normalized_repository_relative_names(self):
        self.assertEqual("crates/kmipkit/src/lib.rs", validate_git_path("crates/kmipkit/src/lib.rs"))
        for path in ("../outside", "/absolute/path", "-option", "crates//lib.rs", "crates/./lib.rs"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                validate_git_path(path)


if __name__ == "__main__":
    unittest.main()
