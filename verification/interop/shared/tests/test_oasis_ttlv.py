from collections import Counter
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from verification.interop.shared.tools import campaign, oasis_ttlv


class OasisTtlvTests(unittest.TestCase):
    def test_prepares_every_pinned_request_and_response(self):
        messages = Counter()
        cases = set()
        for row in oasis_ttlv.vectors(oasis_ttlv.ROOT, oasis_ttlv.VERSION_CODES):
            with self.subTest(version=row["protocol_version"], fixture=row["fixture"], step=row["step"]):
                self.assertEqual(row["status"], "ready", row.get("reason"))
                wire = bytes.fromhex(row["ttlv"])
                root_tag = 0x420078 if row["message"] == "RequestMessage" else 0x42007B
                self.assertEqual(row["items"][0][:2], [root_tag, 1])
                self.assertEqual(int.from_bytes(wire[:3], "big"), root_tag)
                self.assertEqual(int.from_bytes(wire[4:8], "big") + 8, len(wire))
            messages[row["protocol_version"]] += 1
            cases.add(row["fixture"])
        self.assertEqual(messages, {"2.0": 1156, "2.1": 1156, "3.0": 1234})
        self.assertEqual(len(cases), 288)

    def test_wrong_scalar_fails_even_when_structure_matches(self):
        items = [[0x420078, 1, 2], [0x42002A, 2, 0, 256], [0x4200DD, 6, 0, True]]
        for corrupted in (
            [[0x420078, 1, 2], [0x42002A, 2, 0, 128], [0x4200DD, 6, 0, True]],
            [[0x420078, 1, 2], [0x42002A, 2, 0, 256], [0x4200DD, 6, 0, 1]],
            [[0x420078, 1, 2], [0x42002A, 2, 0], [0x4200DD, 6, 0]],
        ):
            for construct in (False, True):
                rows = [{"status": "ready", "ttlv": "00", "items": items}]
                artifact = {"library_commit": "a" * 40, "artifact_sha256": "b" * 64}
                with self.subTest(corrupted=corrupted, construct=construct), patch.object(
                    oasis_ttlv.subprocess, "run"
                ) as run:
                    run.return_value.returncode = 0
                    run.return_value.stdout = json.dumps(artifact) + "\n" + json.dumps(corrupted)
                    oasis_ttlv.inspect_vectors(rows, {"rust": Path(__file__)}, construct=construct)
                self.assertEqual({"rust": "failed"}, rows[0]["checks"])

    def test_codec_pass_retains_pending_scenario_exchange(self):
        row = {
            "kind": "scenario_item", "origin": "xml", "case_id": "QS-M-1-30",
            "protocol_version": "3.0", "step_index": 0, "item_index": 0,
            "operation": "Query", "fixture": {"path": "fixture.xml", "sha256": "f" * 64},
        }
        digest, matrix = campaign.plan([row])
        report = campaign.report(matrix, digest, [], ("rust",))
        artifact = {"public_api_symbol": oasis_ttlv.DECODERS["rust"], "library_commit": "a" * 40}
        result = {
            "protocol_version": "3.0", "fixture": "fixture.xml", "fixture_sha256": "f" * 64,
            "step": 0, "message": "ResponseMessage", "checks": {"rust": "passed"},
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "oasis.jsonl"
            header = {"artifacts": {"rust": artifact}}
            path.write_text(json.dumps(header) + "\n" + json.dumps(result) + "\n", encoding="utf-8")
            oasis_ttlv.attach_decoder_evidence(path, report, "a" * 40)
            check = report["rows"][0]["checks"]["rust"]
            self.assertEqual(check, {"status": "pending", "fixture_codec": {"response": "passed"}})
            with self.assertRaisesRegex(ValueError, "library commit"):
                oasis_ttlv.attach_decoder_evidence(path, report, "b" * 40)

    def test_construction_evidence_cannot_be_imported_as_decode_or_server_evidence(self):
        row = {
            "kind": "scenario_item", "origin": "xml", "case_id": "QS-M-1-21",
            "protocol_version": "2.1", "step_index": 0, "item_index": 0,
            "operation": "Query", "fixture": {"path": "fixture.xml", "sha256": "f" * 64},
        }
        digest, matrix = campaign.plan([row])
        report = campaign.report(matrix, digest, [], ("rust",))
        artifact = {"public_api_symbol": oasis_ttlv.CONSTRUCTORS["rust"], "library_commit": "a" * 40}
        result = {
            "protocol_version": "2.1", "fixture": "fixture.xml", "fixture_sha256": "f" * 64,
            "step": 0, "message": "RequestMessage", "checks": {"rust": "passed"},
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "oasis.jsonl"
            header = {"mode": "construct", "artifacts": {"rust": artifact}}
            path.write_text(json.dumps(header) + "\n" + json.dumps(result) + "\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "evidence mode"):
                oasis_ttlv.attach_decoder_evidence(path, report, "a" * 40)
            oasis_ttlv.attach_decoder_evidence(path, report, "a" * 40, construct=True)
            self.assertEqual(report["rows"][0]["checks"]["rust"],
                             {"status": "pending", "fixture_construction": {"request": "passed"}})
