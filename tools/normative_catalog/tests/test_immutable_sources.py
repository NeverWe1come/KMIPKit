"""Tests for the immutable OASIS base-tree gate."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.normative_catalog.check_immutable_sources import (
    ImmutableSourceError,
    check_immutable_sources,
)


class ImmutableSourceGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self._git("init", "--quiet", "--initial-branch=main")
        self._git("config", "user.name", "KMIPKit test")
        self._git("config", "user.email", "tests@example.invalid")
        source = self.root / "specification/oasis/kmip-2.1/upstream/source.html"
        source.parent.mkdir(parents=True)
        source.write_bytes(b"<html>pinned</html>\n")
        oasis_docs = self.root / "specification/oasis/kmip-2.1"
        oasis_docs.mkdir(parents=True, exist_ok=True)
        (oasis_docs / "README.md").write_text("Project inventory.\n", encoding="utf-8")
        (oasis_docs / "SOURCES.md").write_text("Pinned source inventory.\n", encoding="utf-8")
        self._git("add", "specification/oasis")
        self._git("commit", "--quiet", "-m", "base")
        self.base_sha = self._git("rev-parse", "HEAD").decode().strip()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _git(self, *arguments: str, input_bytes: bytes | None = None) -> bytes:
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.root,
            input=input_bytes,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        return result.stdout

    def test_accepts_unchanged_oasis_tree_for_exact_base_commit(self) -> None:
        check_immutable_sources(self.root, self.base_sha)

    def test_accepts_project_inventory_edits_and_new_fixture(self) -> None:
        oasis_docs = self.root / "specification/oasis/kmip-2.1"
        (oasis_docs / "README.md").write_text("Updated project inventory.\n", encoding="utf-8")
        (oasis_docs / "SOURCES.md").write_text("Updated source inventory.\n", encoding="utf-8")
        fixture = oasis_docs / "fixtures/TC-CREATE-SD-1-21.xml"
        fixture.parent.mkdir()
        fixture.write_bytes(b"<TestCase/>\n")

        check_immutable_sources(self.root, self.base_sha)

    def test_change_detection_uses_quiet_diffs_and_streamed_untracked_paths(self) -> None:
        real_run = subprocess.run
        with patch(
            "tools.normative_catalog.check_immutable_sources.subprocess.run",
            wraps=real_run,
        ) as run:
            check_immutable_sources(self.root, self.base_sha)

        commands = [call.args[0] for call in run.call_args_list]
        diff_commands = [command for command in commands if len(command) > 1 and command[1] == "diff"]
        self.assertEqual(len(diff_commands), 2)
        self.assertTrue(all("--quiet" in command for command in diff_commands))
        self.assertFalse(any(command[1:2] == ["ls-files"] for command in commands))

    def test_rejects_missing_or_non_commit_base_sha(self) -> None:
        for invalid in ("", "main", "0" * 40):
            with self.subTest(base_sha=invalid), self.assertRaises(ImmutableSourceError):
                check_immutable_sources(self.root, invalid)

    def test_rejects_modified_source_content(self) -> None:
        path = self.root / "specification/oasis/kmip-2.1/upstream/source.html"
        path.write_bytes(b"<html>changed</html>\n")
        with self.assertRaises(ImmutableSourceError):
            check_immutable_sources(self.root, self.base_sha)

    def test_rejects_added_untracked_source(self) -> None:
        path = self.root / "specification/oasis/new.html"
        path.write_text("new", encoding="utf-8")
        with self.assertRaises(ImmutableSourceError):
            check_immutable_sources(self.root, self.base_sha)

    def test_rejects_deleted_source(self) -> None:
        self._git("rm", "--quiet", "specification/oasis/kmip-2.1/upstream/source.html")
        with self.assertRaises(ImmutableSourceError):
            check_immutable_sources(self.root, self.base_sha)

    def test_rejects_renamed_source(self) -> None:
        self._git(
            "mv",
            "specification/oasis/kmip-2.1/upstream/source.html",
            "specification/oasis/kmip-2.1/upstream/renamed.html",
        )
        with self.assertRaises(ImmutableSourceError):
            check_immutable_sources(self.root, self.base_sha)

    def test_rejects_executable_mode_change(self) -> None:
        self._git("update-index", "--chmod=+x", "specification/oasis/kmip-2.1/upstream/source.html")
        with self.assertRaises(ImmutableSourceError):
            check_immutable_sources(self.root, self.base_sha)

    def test_rejects_regular_file_replaced_by_symlink_mode(self) -> None:
        path = "specification/oasis/kmip-2.1/upstream/source.html"
        blob = self._git("hash-object", "-w", "--stdin", input_bytes=b"target")
        self._git("update-index", "--cacheinfo", "120000", blob.decode().strip(), path)
        with self.assertRaises(ImmutableSourceError):
            check_immutable_sources(self.root, self.base_sha)


if __name__ == "__main__":
    unittest.main()
