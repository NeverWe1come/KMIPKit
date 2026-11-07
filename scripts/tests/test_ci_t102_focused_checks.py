"""Keep the T102 focused regressions in the hosted Python contract job."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CI_WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
FOCUSED_COMMAND = (
    "& $python -B -m unittest "
    "tools.api_manifest.tests.test_t102_certify_request_builder_mapping "
    "tools.api_manifest.tests.test_t102_client_extension_registry_namespace "
    "tools.api_manifest.tests.test_t102_recertify_rekey_key_pair_builder_mapping "
    "tools.api_manifest.tests.test_log_parity_manifest "
    "tools.api_manifest.tests.test_t102_response_parser_mapping "
    "tools.api_manifest.tests.test_t102_profile_transport_parity "
    "tools.api_manifest.tests.test_t102_profile_model_reexport_mapping -v"
)


class T102FocusedCiContractTests(unittest.TestCase):
    def test_current_t102_manifest_regressions_run_in_python_contract_job(self) -> None:
        lines = CI_WORKFLOW.read_text(encoding="utf-8").splitlines()
        command_line = f"          {FOCUSED_COMMAND}"
        self.assertIn(command_line, lines)
        command_index = lines.index(command_line)
        self.assertEqual(
            "          if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }",
            lines[command_index + 1],
        )


if __name__ == "__main__":
    unittest.main()
