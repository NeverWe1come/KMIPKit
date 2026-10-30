"""T042 contract for operation-owner and API-readiness reconciliation."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
T042_OPERATION_NAMES = frozenset(
    {
        "Cancel",
        "Poll",
        "Process",
        "Query Asynchronous Requests",
        "Add Attribute",
        "Adjust Attribute",
        "Delete Attribute",
        "Get Attribute List",
        "Get Attributes",
        "Modify Attribute",
        "Set Attribute",
        "Get",
        "Locate",
        "Activate",
        "Archive",
        "Destroy",
        "Recover",
        "Decrypt",
        "Encrypt",
        "Ping",
        "Query",
    }
)


def _operation_rows(document: str) -> dict[str, str]:
    rows = {}
    for line in document.splitlines():
        if not line.startswith("| §6.1."):
            continue
        columns = [column.strip() for column in line.strip("|").split("|")]
        rows[columns[1]] = columns[3]
    return rows


class T042OwnerReadinessMatrixTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        matrix_path = (
            ROOT
            / "specs"
            / "043-public-api-bindings"
            / "contracts"
            / "operation-dependencies.md"
        )
        cls.matrix_rows = _operation_rows(matrix_path.read_text(encoding="utf-8"))

    def test_t035_through_t041_matrix_states_match_manifest(self) -> None:
        operations = {operation["name"]: operation for operation in self.manifest["operations"]}
        self.assertTrue(T042_OPERATION_NAMES <= set(operations))
        self.assertTrue(T042_OPERATION_NAMES <= set(self.matrix_rows))

        for name in sorted(T042_OPERATION_NAMES):
            operation = operations[name]
            row = self.matrix_rows[name]
            api_state = operation["apiState"]
            with self.subTest(operation=name, api_state=api_state):
                self.assertIn(f"manifest API state: `{api_state}`", row)
                if api_state == "verified":
                    self.assertNotRegex(row, r"\bin progress\b|\bremain incomplete\b")
                elif api_state == "declared":
                    self.assertIn("parity", row.lower())
                    self.assertNotIn("conformant", row.lower())

    def test_t036_profile_validation_owner_is_accepted_and_has_closeout_evidence(self) -> None:
        spec_path = ROOT / "specs" / "010-profile-conformance" / "spec.md"
        status = re.search(
            r"(?m)^\*\*Status\*\*:\s*([^\r\n]+)",
            spec_path.read_text(encoding="utf-8"),
        )
        self.assertIsNotNone(status)
        self.assertTrue(status.group(1).startswith("Accepted for implementation"))

        tasks = (
            ROOT / "specs" / "010-profile-conformance" / "tasks.md"
        ).read_text(encoding="utf-8")
        self.assertIn("- [x] T026 ", tasks)
        self.assertIn("- [x] T057 ", tasks)

        dependencies = (
            ROOT
            / "specs"
            / "043-public-api-bindings"
            / "contracts"
            / "shared-dependencies.md"
        ).read_text(encoding="utf-8")
        profile_row = next(
            line for line in dependencies.splitlines() if line.startswith("| KMIPKIT-0010 |")
        )
        self.assertIn("| Accepted |", profile_row)
        self.assertIn("profile validation may be mapped without a support or conformance claim", profile_row)


    def test_t042_stays_open_until_t036_profile_api_parity_is_complete(self) -> None:
        tasks_path = ROOT / "specs" / "043-public-api-bindings" / "tasks.md"
        task_rows = [
            line
            for line in tasks_path.read_text(encoding="utf-8").splitlines()
            if line.startswith("- [") and " T042 " in line
        ]
        self.assertEqual(len(task_rows), 1)
        self.assertTrue(task_rows[0].startswith("- [ ] T042 "))
        self.assertNotIn("while T036 remains blocked on Draft KMIPKIT-0010", task_rows[0])

if __name__ == "__main__":
    unittest.main()
