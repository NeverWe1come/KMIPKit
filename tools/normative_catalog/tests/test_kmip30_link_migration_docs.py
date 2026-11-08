"""Keep KMIP 2.1/3.0 Link migration guidance aligned with its source mapping."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class Kmip30LinkMigrationDocsTests(unittest.TestCase):
    def test_link_migration_note_points_to_the_exact_approved_mapping(self) -> None:
        migration_note = (
            ROOT / "docs/development/kmip-3.0-api-migration.md"
        ).read_text(encoding="utf-8")
        link_spec = (
            ROOT / "specs/071-kmip-3-0-draft-support/spec.md"
        ).read_text(encoding="utf-8")

        self.assertIn("§11.28, Table 465", migration_note)
        self.assertIn("`0x101`–`0x10E`", migration_note)
        self.assertIn("../../specs/071-kmip-3-0-draft-support/spec.md", migration_note)
        self.assertIn("14 semantic equivalents", migration_note)
        self.assertIn("`0x101` Certificate Link", link_spec)
        self.assertIn("`0x10E` Wrapping Key Link", link_spec)
        self.assertNotRegex(migration_note, r"Link Type values?[^\n]{0,80}`0x0[1-9A-E]`")

    def test_implemented_link_task_is_not_left_marked_open(self) -> None:
        tasks = (ROOT / "specs/071-kmip-3-0-draft-support/tasks.md").read_text(
            encoding="utf-8"
        )
        task = re.search(r"^- \[([ x])\] \*\*TASK-071-008\*\*:(.*(?:\n  .*)*)", tasks, re.MULTILINE)

        self.assertIsNotNone(task, "TASK-071-008 must remain in the KMIPKIT-0071 task list")
        assert task is not None
        self.assertEqual("x", task.group(1), "the implemented and verified LinkNormalizer task is still open")
        self.assertIn("KMIPKIT-0048", task.group(2))

    def test_0048_traceability_links_the_migration_evidence(self) -> None:
        traceability = (ROOT / "specs/048-typed-attributes/traceability.md").read_text(
            encoding="utf-8"
        )
        requirement = next(
            line for line in traceability.splitlines() if "KMIPKIT-0048-FR-011" in line
        )

        self.assertIn("kmip-3.0-api-migration.md", requirement)
        self.assertIn("link_normalizer_tests.rs", requirement)


if __name__ == "__main__":
    unittest.main()
