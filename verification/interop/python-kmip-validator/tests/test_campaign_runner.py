import json
import unittest

from kmipkit.protocol_values import ResultReason
from kmipkit_validator.campaign_runner import OPERATION_CODES, draft_events, query_evidence
from kmipkit_validator.controlled_peer import ControlledTlsPeer, PKI


class CampaignRunnerTests(unittest.TestCase):
    def test_installed_draft_constructors_emit_only_the_matching_obligations(self):
        rows = [json.dumps({
            "language": "python", "kind": "public_route", "protocol_version": "3.0",
            "action": "construct", "execution_status": "required", "obligation_id": f"{i:064x}",
            "public_api_symbol": "kmipkit.extensions." + name,
        }) for i, name in enumerate(OPERATION_CODES)]
        events = list(draft_events(rows))
        self.assertEqual(len(events), 7)
        self.assertTrue(all(event["status"] == "passed" for event in events))

    def test_typed_query_records_only_one_exchange_and_retains_result_codes(self):
        row = json.dumps({
            "language": "python", "protocol_version": "2.1", "encoding": "TTLV", "transport": "raw_tls",
            "case_id": "test-op-query", "target_id": "KMIPKIT-ELEM-OP-C2S-QUERY",
            "public_api_symbol": "kmipkit.extensions.client_batch_item_query",
            "execution_api_symbol": "kmipkit.client.Client.execute", "execution_status": "required",
            "action": "send", "obligation_id": "a" * 64,
        })
        for result_status, reason, outcome, compatibility in (
            (0, None, "passed", "supported"),
            (1, ResultReason.OPERATION_NOT_SUPPORTED, "failed", "unsupported"),
            (1, ResultReason.PERMISSION_DENIED, "blocked", "policy_restricted"),
        ):
            with self.subTest(reason=reason):
                peer = ControlledTlsPeer(status=result_status, reason=reason)
                try:
                    events = query_evidence([row], {
                        "server_id": "loopback", "protocol_version": "2.1", "transport": "raw_tls",
                        "encoding": "TTLV", "api_level": "typed-protocol",
                        "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": peer.port}},
                        "tls": {
                            "ca_cert_path": str(PKI / "server-cert.pem"),
                            "client_cert_path": str(PKI / "client-cert.pem"),
                            "client_key_path": str(PKI / "client-key.pem"),
                            "server_name_override": "localhost", "connect_timeout_seconds": 5,
                            "read_timeout_seconds": 5,
                        },
                    })
                    self.assertEqual(len(peer.requests), 1)
                    self.assertEqual(events[0]["status"], outcome)
                    self.assertEqual(events[0]["server_compatibility"], compatibility)
                    self.assertEqual(events[0]["result_status_code"], result_status)
                    self.assertEqual(events[0].get("result_reason_code"), reason)
                finally:
                    peer.close()
