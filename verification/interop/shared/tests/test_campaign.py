import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from verification.interop.shared.tools import campaign


class CampaignTests(unittest.TestCase):
    def setUp(self):
        self.row = {
            "protocol_version": "2.1", "case_id": "query", "language": "rust",
            "mode": "network", "action": "send", "public_api_symbol": "query",
            "execution_api_symbol": "execute", "encoding": "TTLV", "transport": "raw_tls",
            "operation": "Query", "execution_status": "required",
        }
        self.digest, self.matrix = campaign.plan([self.row])
        self.event = {
            "obligation_id": next(iter(self.matrix)), "public_api_symbol": "query",
            "execution_api_symbol": "execute", "encoding": "TTLV", "transport": "raw_tls",
            "status": "passed",
        }
        self.run = {
            "schema_version": "1.0", "run_id": "fixture", "scope": "controlled",
            "language": "rust", "matrix_sha256": self.digest, "library_commit": "a" * 40,
            "artifact_sha256": "b" * 64, "events": [self.event],
        }

    def test_digest_changes_with_the_planned_assertion(self):
        modified_digest, _ = campaign.plan([{**self.row, "action": "decode"}])
        self.assertNotEqual(self.digest, modified_digest)
        with self.assertRaisesRegex(ValueError, "duplicate"):
            campaign.plan([self.row, self.row])

    def test_controlled_and_live_evidence_remain_independent(self):
        profile = {"server": {"id": "kms"}, "operations": {"query": {"supported": True}}}
        server = {"id": "kms", "product": "Test", "version": "1", "configuration_fingerprint": "c" * 64}
        live_event = {**self.event, "server_compatibility": "unsupported", "status": "failed"}
        live_run = {**self.run, "run_id": "remote", "scope": "live", "server": server, "events": [live_event]}
        report = campaign.report(self.matrix, self.digest, [live_run], ("rust",), profile)
        self.assertEqual(report["summary"]["rust"], {"pending": 1})
        report = campaign.report(self.matrix, self.digest, [live_run, self.run], ("rust",), profile)
        check = report["rows"][0]["checks"]["rust"]
        self.assertEqual(check["status"], "passed")
        self.assertEqual(check["live"][0]["declaration_check"], "advertised_but_unsupported")
        self.assertEqual(report["public_api"]["rust"], {"complete": 2, "total": 2})
        self.assertEqual(report["server_compatibility"], {"unsupported": 1})

    def test_reject_wrong_scope_and_route(self):
        for run in (
            {**self.run, "scope": "local"},
            {**self.run, "events": [{**self.event, "transport": "https"}]},
            {**self.run, "events": [{**self.event, "public_api_symbol": "other"}]},
            {**self.run, "matrix_sha256": "0" * 64},
        ):
            with self.subTest(run=run), self.assertRaises(ValueError):
                campaign.report(self.matrix, self.digest, [run], ("rust",))

    def test_report_embeds_safe_data_and_retains_machine_evidence(self):
        digest, matrix = campaign.plan([{**self.row, "case_id": "</script>"}])
        result = campaign.report(matrix, digest, [], ("rust",))
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir)
            campaign.write_report(output, result)
            page = (output / "index.html").read_text(encoding="utf-8")
            self.assertIn("\\u003c/script>", page)
            self.assertIn("1", (output / "summary.md").read_text(encoding="utf-8"))
            self.assertEqual(json.loads((output / "checks.jsonl").read_text(encoding="utf-8"))["case_id"], "</script>")

    def test_live_schema_requires_server_identity(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = Path(tmpdir) / "evidence.json"
            path.write_text(json.dumps(self.run), encoding="utf-8")
            self.assertEqual(campaign.read_validated(path, "campaign-evidence.schema.json"), self.run)
            path.write_text(json.dumps({**self.run, "scope": "live"}), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "invalid campaign-evidence"):
                campaign.read_validated(path, "campaign-evidence.schema.json")

    def test_same_semantics_are_required_across_comparable_languages(self):
        second = {**self.row, "language": "c", "public_api_symbol": "kmipkit_query"}
        digest, matrix = campaign.plan([self.row, second])
        rust_id, c_id = matrix
        event = {**self.event, "obligation_id": rust_id, "parity_group": "d" * 64, "semantic_sha256": "e" * 64}
        rust = {**self.run, "matrix_sha256": digest, "events": [event]}
        c_event = {**event, "obligation_id": c_id, "public_api_symbol": "kmipkit_query"}
        c_run = {**rust, "language": "c", "run_id": "c", "events": [c_event]}
        languages = ("rust", "c")
        self.assertEqual(campaign.report(matrix, digest, [rust], languages)["parity"], {"incomplete": 1})
        self.assertEqual(campaign.report(matrix, digest, [rust, c_run], languages)["parity"], {"matched": 1})
        c_event["semantic_sha256"] = "f" * 64
        self.assertEqual(campaign.report(matrix, digest, [rust, c_run], languages)["parity"], {"mismatched": 1})

    def test_launch_uses_fixed_scope_and_absolute_path_arguments(self):
        args = SimpleNamespace(scope="controlled", connection=None, profile=None, output_dir=Path("--results"))
        runner = Path("validator").resolve()
        with patch.object(campaign.subprocess, "run", return_value=SimpleNamespace(returncode=1)) as run:
            self.assertEqual(campaign.launch_validators({"rust": runner}, args, self.digest), ([], ["rust"]))
            directory = args.output_dir.resolve()
            run.assert_called_once_with(
                [str(runner), "--plan", str(directory / "plan.jsonl"), "--matrix-sha256", self.digest,
                 "--scope", "controlled", "--evidence", str(directory / "evidence-rust-controlled.json")],
                check=False,
            )

    def test_launch_rejects_invalid_scope_before_starting_a_process(self):
        args = SimpleNamespace(scope="--invalid")
        with patch.object(campaign.subprocess, "run") as run:
            with self.assertRaisesRegex(ValueError, "invalid campaign scope"):
                campaign.launch_validators({"rust": Path("validator")}, args, self.digest)
            run.assert_not_called()
