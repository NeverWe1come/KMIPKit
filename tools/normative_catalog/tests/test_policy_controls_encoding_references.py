"""Verify P025 encoding citations against the pinned OASIS KMIP 2.1 source."""

from __future__ import annotations

import html
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FEATURE = ROOT / "specs/025-policy-controls"
ENCODING_REFERENCE = re.compile(r"§7\.40(?:, |/| )(?:Table|Tabla) 392")


class PolicyControlsEncodingReferencesTests(unittest.TestCase):
    def test_pinned_table_392_defines_usage_limits_count_as_long_integer(self):
        source = ROOT / "specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html"
        # The immutable upstream HTML declares windows-1252.
        text = html.unescape(re.sub(r"<[^>]*>", " ", source.read_text(encoding="cp1252")))
        text = re.sub(r"\s+", " ", text)
        self.assertIn(
            "Usage Limits Count Long Integer Yes Usage Limits Unit Enumeration Yes "
            "Table 392: Usage limits Structure",
            text,
        )

    def test_encoding_contracts_cite_section_7_40_table_392(self):
        paths = [
            FEATURE / "spec.md",
            FEATURE / "plan.md",
            FEATURE / "research.md",
            FEATURE / "data-model.md",
            FEATURE / "quickstart.md",
            FEATURE / "traceability.md",
            FEATURE / "contracts/operations.md",
            FEATURE / "contracts/usage-allocation-guard.md",
            ROOT / "crates/kmipkit-protocol/src/get_usage_allocation.rs",
            ROOT / "docs/user-guide/en/policy-controls.md",
            ROOT / "docs/user-guide/es/controles-de-uso.md",
        ]
        for path in paths:
            with self.subTest(path=str(path.relative_to(ROOT))):
                self.assertTrue(
                    ENCODING_REFERENCE.search(path.read_text(encoding="utf-8")),
                    "missing exact Usage Limits encoding section/table",
                )

    def test_allocation_acceptance_and_requirement_traceability_agree_on_wire_type(self):
        tasks = (FEATURE / "tasks.md").read_text(encoding="utf-8")
        traceability = (FEATURE / "traceability.md").read_text(encoding="utf-8")
        self.assertTrue("Tables 232–234, exact Long Integer count" in tasks)
        requirement_row = next(
            (
                " ".join(line.split())
                for line in traceability.splitlines()
                if "`KMIPKIT-REQ-SPEC-6.1.23-002-001`" in line
            ),
            "",
        )
        self.assertIn("Long Integer", requirement_row)
        self.assertIn("typed request", requirement_row.casefold())


if __name__ == "__main__":
    unittest.main()
