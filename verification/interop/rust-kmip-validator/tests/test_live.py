"""Offline mTLS interoperability fixtures: OASIS KMIP 2.0/2.1 §§6-7.

Only request shape/status metadata is retained. Bodies and sentinel secrets are
never written to evidence artifacts. These peers are controlled fixtures, not
vendor compatibility evidence.
"""
import contextlib
import hashlib
import json
from itertools import product
import os
from pathlib import Path
import select
import socket
import ssl
import struct
import subprocess
import tempfile
import threading
import time
import unittest
import xml.etree.ElementTree as ET

import jsonschema

from request_parity import RequestNormalizer, parity_signature, result_code

ROOT = Path(__file__).resolve().parents[4]
VALIDATOR = Path(os.environ.get("KMIPKIT_VALIDATOR_BIN", ROOT / "verification/interop/rust-kmip-validator/target/debug/rust-kmip-validator"))
SERVER = Path(os.environ.get("KMIPKIT_TEST_SERVER", ROOT / "verification/interop/.local/rust-test-server/harness/target/debug/kmipkit-interop-local-server"))
CATALOG = json.loads((ROOT / "verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
SCHEMA = json.loads((ROOT / "verification/interop/shared/schemas/validation-results.schema.json").read_text())


def ttlv(tag, kind, value):
    if kind == 1:
        value = b"".join(value)
    elif kind in (2, 5):
        value = struct.pack(">I", value)
    elif kind == 9:
        value = struct.pack(">Q", value)
    elif kind == 6:
        value = struct.pack(">Q", int(value))
    elif kind in (7, 12):
        value = value.encode()
    return tag.to_bytes(3, "big") + bytes([kind]) + len(value).to_bytes(4, "big") + value + b"\0" * (-len(value) % 8)


def parse(data):
    result = []
    offset = 0
    while offset < len(data):
        tag = int.from_bytes(data[offset:offset+3], "big")
        kind = data[offset+3]
        size = int.from_bytes(data[offset+4:offset+8], "big")
        value = data[offset+8:offset+8+size]
        if kind == 1:
            value = parse(value)
        elif kind in (2, 3, 5, 6, 9, 10, 11):
            value = int.from_bytes(value, "big", signed=kind in (2, 3, 9, 11))
        elif kind in (7, 12):
            value = value.decode()
        result.append((tag, kind, value))
        offset += 8 + size + (-size % 8)
    return result


def find(tree, tag):
    for t, k, value in tree:
        if t == tag:
            return value
        if k == 1:
            match = find(value, tag)
            if match is not None:
                return match
    return None


def response(operation, minor=1, status=1, reason=5, payload=None, message=None, *, major=2):
    fields = [ttlv(0x42005c, 5, operation), ttlv(0x42007f, 5, status)]
    if reason is not None:
        fields.append(ttlv(0x42007e, 5, reason))
    if message:
        fields.append(ttlv(0x42007d, 7, message))
    if payload is not None:
        fields.append(ttlv(0x42007c, 1, payload))
    header = [ttlv(0x420069, 1, [ttlv(0x42006a, 2, major), ttlv(0x42006b, 2, minor)]),
              ttlv(0x420092, 9, 1794268800)]
    if major < 3:
        header.append(ttlv(0x42000d, 2, 1))
    return ttlv(0x42007b, 1, [ttlv(0x42007a, 1, header), ttlv(0x42000f, 1, fields)])


def receive(channel):
    header = b""
    while len(header) < 8:
        chunk = channel.recv(8-len(header))
        if not chunk:
            return None
        header += chunk
    size = int.from_bytes(header[4:8], "big")
    if size > 16 * 1024 * 1024:
        raise AssertionError("request exceeds shared decoder bound")
    size += -size % 8
    body = b""
    while len(body) < size:
        chunk = channel.recv(size-len(body))
        if not chunk:
            raise AssertionError("truncated request")
        body += chunk
    return header + body



class ParityContract(unittest.TestCase):
    def case(self, result_status="Operation Failed", result_reason="Operation Not Supported"):
        return {"cases": [{"case_id": "test-op-query", "status": "server_unsupported", "library_correctness": "passed", "server_compatibility": "unsupported", "result_status": result_status, "result_reason": result_reason}]}

    def test_parity_signature_detects_different_kmip_result_reason(self):
        self.assertNotEqual(parity_signature(self.case()), parity_signature(self.case(result_reason="Feature Not Supported")))

    def test_parity_signature_detects_different_kmip_result_status(self):
        self.assertNotEqual(parity_signature(self.case()), parity_signature(self.case(result_status="Operation Pending")))

    def test_parity_signature_detects_different_response_payload(self):
        symmetric = ((0x420057, 5, 2),)
        private = ((0x420057, 5, 4),)
        self.assertNotEqual(parity_signature(self.case(), response_trees=(symmetric,)), parity_signature(self.case(), response_trees=(private,)))

    def test_result_codes_normalize_names_hex_and_future_values(self):
        self.assertEqual(result_code("OperationFailed", "RESULT-STATUS"), 1)
        self.assertEqual(result_code("Operation Failed", "RESULT-STATUS"), 1)
        self.assertEqual(result_code("0x00000008", "RESULT-REASON"), 8)
        self.assertEqual(result_code("Feature Not Supported", "RESULT-REASON"), 8)
        self.assertEqual(result_code("0x8000ffff", "RESULT-REASON"), 0x8000ffff)
        self.assertTrue(parity_signature(self.case()) == parity_signature(self.case("0x00000001", "0x00000005")))

    def test_request_and_response_normalizer_redacts_secrets_and_volatile_fields(self):
        snapshots = []
        for identifier, name, timestamp, secret in (("owned-a", "name-a", 1, "private-a"), ("owned-b", "name-b", 2, "private-b")):
            normalizer = RequestNormalizer()
            normalizer.observe_response([(0x420094, 7, identifier)])
            tree = [(0x420092, 9, timestamp), (0x420094, 7, identifier), (0x420055, 7, name),
                    (0x42002a, 2, 256), (0x420025, 1, [(0x420099, 7, secret)]),
                    (0x420043, 1, [(0x42003f, 8, secret.encode())]), (0x4200c2, 8, secret.encode())]
            snapshot = normalizer.normalize(tree)
            encoded = json.dumps(snapshot)
            self.assertTrue(all(value not in encoded for value in (identifier, name, secret)), "sensitive value escaped normalization")
            self.assertTrue(all(node[0] != 0x420092 for node in snapshot), "volatile timestamp retained")
            snapshots.append(snapshot)
        self.assertTrue(snapshots[0] == snapshots[1], "redacted trees should normalize equally")

    def test_request_normalizer_preserves_seeded_resource_relationships(self):
        normalizer = RequestNormalizer()
        normalizer.observe_response([(0x420094, 7, "owned-resource")])
        correct = normalizer.normalize([(0x420094, 7, "owned-resource")])
        wrong = normalizer.normalize([(0x420094, 7, "different-resource")])
        self.assertTrue(correct != wrong, "normalization hid a changed resource reference")
        normalizer.observe_response([(0x420043, 1, [(0x420094, 7, "private-material")])])
        self.assertEqual(len(normalizer.aliases["identifier"]), 2)

    def test_parity_signature_detects_different_deterministic_request(self):
        aes256 = ((0x42002a, 2, 256),)
        aes128 = ((0x42002a, 2, 128),)
        self.assertNotEqual(parity_signature(self.case(), (aes256,)), parity_signature(self.case(), (aes128,)))


class TagFixtureContract(unittest.TestCase):
    def test_constructs_all_kmip2_scalar_types_without_the_decoder(self):
        tag = 0x420094
        values = [
            (2, struct.pack(">i", -(2**31))), (3, struct.pack(">q", 2**63 - 1)),
            (4, b"\xff" * 8), (5, struct.pack(">I", 2**32 - 1)),
            (6, b"\x00" * 7 + b"\x01"), (7, "á".encode()), (8, b"abc"),
            (9, struct.pack(">q", -123)), (10, struct.pack(">I", 2**32 - 1)),
            (11, struct.pack(">q", 1234567890123)),
        ]
        fields = [
            tag.to_bytes(3, "big") + bytes([kind]) + struct.pack(">I", len(value))
            + value + bytes(-len(value) % 8)
            for kind, value in values
        ]
        root = ttlv(0x420078, 1, fields)
        malformed = root[:-1]
        invalid_boolean = ttlv(tag, 8, b"\x00" * 7 + b"\x02")
        invalid_boolean = invalid_boolean[:3] + b"\x06" + invalid_boolean[4:]
        expected_scalars = [-(2**31), 2**63 - 1, "ff" * 8, 2**32 - 1, True,
                            "c3a1", "616263", -123, 2**32 - 1, 1234567890123]
        expected = [[0x420078, 1, len(fields)], *[
            [tag, kind, 0, expected_value]
            for (kind, _), expected_value in zip(values, expected_scalars, strict=True)
        ]]
        runner = VALIDATOR.with_name("kmip30-campaign")
        for command in ("--construct-oasis-ttlv", "--inspect-oasis-ttlv"):
            process = subprocess.run([str(runner), command], check=True, capture_output=True, text=True,
                                     input="".join(frame.hex() + "\n" for frame in (root, malformed, invalid_boolean)))
            rows = process.stdout.splitlines()
            self.assertEqual([expected, None, None], [json.loads(row) for row in rows[1:]])


class ControlledPeer:
    def __init__(self, pki, responder):
        self.listener = socket.socket()
        self.listener.bind(("127.0.0.1", 0))
        self.listener.listen()
        self.listener.settimeout(.2)
        self.port = self.listener.getsockname()[1]
        self.context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        self.context.minimum_version = ssl.TLSVersion.TLSv1_3
        self.context.maximum_version = ssl.TLSVersion.TLSv1_3
        self.context.load_cert_chain(pki / "server.crt", pki / "server.key")
        self.context.load_verify_locations(pki / "ca.crt")
        self.context.verify_mode = ssl.CERT_REQUIRED
        self.responder = responder
        self.operations = []
        self.request_trees = []
        self.response_trees = []
        self.normalizer = RequestNormalizer()
        self.connections = 0
        self.stop = threading.Event()
        self.errors = []
        self.thread = threading.Thread(target=self.run)

    def run(self):
        try:
            while not self.stop.is_set():
                try:
                    raw, _ = self.listener.accept()
                except socket.timeout:
                    continue
                self.connections += 1
                with self.context.wrap_socket(raw, server_side=True) as channel:
                    channel.settimeout(5)
                    data = receive(channel)
                    if data is None:
                        continue
                    tree = parse(data)
                    self.request_trees.append(self.normalizer.normalize(tree))
                    self.operations.extend(find(batch, 0x42005c) for tag, kind, batch in tree[0][2] if tag == 0x42000f)
                    reply = self.responder(tree)
                    if reply is not None:
                        parsed_reply = parse(reply)
                        self.normalizer.observe_response(parsed_reply)
                        self.response_trees.append(self.normalizer.normalize(parsed_reply))
                        channel.sendall(reply)
        except Exception as error:
            if not self.stop.is_set():
                self.errors.append(type(error).__name__)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *args):
        self.stop.set()
        self.thread.join(timeout=6)
        self.listener.close()
        if self.thread.is_alive() or self.errors:
            raise AssertionError(f"controlled fixture failed: {self.errors}")


class LiveContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.directory = Path(cls.temp.name)
        cls.pki = cls.directory / "pki"
        cls.server = subprocess.Popen([str(SERVER), "--listen", "127.0.0.1:0", "--data-dir", str(cls.directory / "store"), "--export-pki", str(cls.pki), "--json"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, stdin=subprocess.PIPE, text=True)
        if not select.select([cls.server.stdout], [], [], 15)[0]:
            raise AssertionError("local test-server did not become ready")
        ready = json.loads(cls.server.stdout.readline())
        cls.port = ready["port"]

    @classmethod
    def tearDownClass(cls):
        cls.server.terminate()
        cls.server.communicate(timeout=10)
        cls.temp.cleanup()

    def connection(self, port=None, version="2.1", transport="raw_tls", encoding="TTLV", api="typed-protocol", mutations=False):
        return {"server_id": "controlled-fixture", "protocol_version": version, "transport": transport, "encoding": encoding, "api_level": api,
                "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": port or self.port}, "https": {"url": f"https://127.0.0.1:{port or self.port}/kmip"}},
                "tls": {"ca_cert_path": str(self.pki / "ca.crt"), "client_cert_path": str(self.pki / "client.crt"), "client_key_path": str(self.pki / "client.key"), "server_name_override": "localhost", "connect_timeout_seconds": 2, "read_timeout_seconds": 2},
                "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
                "execution_policy": {"probe_unsupported": False, "allow_mutations": mutations}}

    def command(self, config, cases, *extra):
        work = Path(tempfile.mkdtemp(dir=self.directory))
        (work / "connection.json").write_text(json.dumps(config, sort_keys=True))
        (work / "catalog.json").write_text(json.dumps({**CATALOG, "test_cases": cases}))
        (work / "profile.json").write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled-fixture", "version": "offline-1"}, "operations": {}}))
        args = [str(VALIDATOR), "run", "--connection", str(work / "connection.json"), "--catalog", str(work / "catalog.json"), "--capabilities", str(work / "profile.json"), "--output", str(work / "results.json"), "--markdown", str(work / "results.md"), *extra]
        process = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, text=True)
        report = json.loads((work / "results.json").read_text()) if (work / "results.json").exists() else None
        if report is not None:
            jsonschema.Draft202012Validator(SCHEMA).validate(report)
        return process, report

    def rows(self, *ids):
        return [case for case in CATALOG["test_cases"] if case["case_id"] in ids]

    def test_local_model_cases_do_not_claim_server_support(self):
        cases = [c for c in CATALOG["test_cases"] if c["execution"]["mode"] == "local_model"]
        process, report = self.command(self.connection(), cases)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["summary"]["passed"], 67)
        self.assertTrue(all(c["server_compatibility"] == "unknown" for c in report["cases"]))

    def test_query_probes_profile_omissions_and_separates_rejections(self):
        for reason, status, compatibility in [(5, "server_unsupported", "unsupported"), (8, "server_unsupported", "unsupported"), (12, "policy_restricted", "policy_restricted"), (3, "failed", "unknown"), (0x100, "failed", "unknown")]:
            with self.subTest(reason=reason), ControlledPeer(self.pki, lambda tree: response(0x18, reason=reason, message="secret-response-sentinel")) as peer:
                _, report = self.command(self.connection(peer.port), self.rows("test-op-query"))
                case = report["cases"][0]
                self.assertEqual(case["status"], status)
                self.assertEqual(case["server_compatibility"], compatibility)
                self.assertEqual(case["library_correctness"], "passed")
                self.assertEqual((case["result_status_code"], case["result_reason_code"]), (1, reason))
                self.assertEqual(peer.operations, [0x18])
                self.assertNotIn("secret-response-sentinel", json.dumps(report))

    def test_shared_query_campaign_uses_one_typed_exchange(self):
        schema_path = ROOT / "verification/interop/shared/schemas/campaign-evidence.schema.json"
        evidence_schema = json.loads(schema_path.read_text())
        payload = [ttlv(0x42005c, 5, 0x18), ttlv(0x42015f, 1, [])]
        outcomes = [(0, None, "passed", "supported"), (1, 5, "failed", "unsupported"),
                    (1, 12, "blocked", "policy_restricted")]
        for code, reason, status, compatibility in outcomes:
            def respond(tree):
                self.assertEqual(find(tree, 0x420074), 1)
                return response(0x18, status=code, reason=reason, payload=payload if code == 0 else None)

            with self.subTest(reason=reason), ControlledPeer(self.pki, respond) as peer:
                work = Path(tempfile.mkdtemp(dir=self.directory))
                config = self.connection(peer.port)
                config.update(server_product="controlled fixture", server_version="1",
                              server_configuration_fingerprint="c" * 64)
                (work / "connection.json").write_text(json.dumps(config))
                row = {"language": "rust", "protocol_version": "2.1", "case_id": "test-op-query",
                       "target_id": "KMIPKIT-ELEM-OP-C2S-QUERY", "encoding": "TTLV", "transport": "raw_tls",
                       "public_api_symbol": "kmipkit_protocol::QueryRequest::new",
                       "execution_api_symbol": "kmipkit_client::Client::execute", "obligation_id": "a" * 64,
                       "action": "send", "execution_status": "required"}
                (work / "plan.jsonl").write_text(json.dumps(row) + "\n")
                output = work / "evidence-rust-live.json"
                process = subprocess.run(
                    [str(VALIDATOR), "query-campaign", "--scope", "live", "--matrix-sha256", "b" * 64,
                     "--plan", str(work / "plan.jsonl"), "--connection", str(work / "connection.json"),
                     "--evidence", str(output)], capture_output=True, text=True, timeout=10,
                )
                self.assertEqual(process.returncode, int(status == "failed"), process.stderr)
                evidence = json.loads(output.read_text())
                jsonschema.Draft202012Validator(evidence_schema).validate(evidence)
                self.assertEqual(peer.operations, [0x18])
                self.assertEqual(evidence["events"][0]["status"], status)
                self.assertEqual(evidence["events"][0]["server_compatibility"], compatibility)
                self.assertEqual(evidence["events"][0]["result_status_code"], code)
                self.assertEqual(evidence["events"][0].get("result_reason_code"), reason)

    def test_connection_drop_after_request_is_never_retried(self):
        with ControlledPeer(self.pki, lambda tree: None) as peer:
            process, report = self.command(self.connection(peer.port), self.rows("test-op-query"))
            self.assertNotEqual(process.returncode, 0)
            self.assertEqual(report["cases"][0]["server_compatibility"], "unknown")
            self.assertEqual(peer.operations, [0x18])
            self.assertEqual(peer.connections, 1)

    def test_mutations_and_sessions_need_explicit_opt_in(self):
        with ControlledPeer(self.pki, lambda tree: response(0x18)) as peer:
            _, report = self.command(self.connection(peer.port), self.rows("test-op-create", "test-op-login", "test-op-log", "test-op-query"))
            self.assertEqual(report["summary"]["policy_restricted"], 3)
            self.assertEqual(peer.operations, [0x18])

    def test_version_boundary_is_rejected_before_sending(self):
        with ControlledPeer(self.pki, lambda tree: response(0x18)) as peer:
            _, report = self.command(self.connection(peer.port, version="2.0"), self.rows("test-v20-reject-get-constraints", "test-v20-reject-set-constraints"))
            self.assertEqual(report["summary"]["passed"], 2)
            self.assertEqual(peer.operations, [])
            self.assertTrue(all(c["server_compatibility"] == "unknown" for c in report["cases"]))

    def test_generic_tags_reach_the_peer_in_both_kmip_2_versions(self):
        probes = [("2.0", 0x18), ("2.0", 0x1e), ("2.1", 0x18), ("2.1", 0x1e), ("2.1", 0x3b), ("2.0", 0x27), ("2.1", 0x27), ("2.0", 0x08), ("2.1", 0x08), ("2.0", 0x25), ("2.1", 0x25), ("2.0", 0x26), ("2.1", 0x26)]
        for (version, operation), result in product(probes, ("passed", "server_unsupported", "failed")):
            with self.subTest(version=version, operation=operation, result=result):
                minor = int(version[-1])
                name = {0x08: "locate", 0x18: "query", 0x1e: "discover-versions", 0x25: "rng-retrieve", 0x26: "rng-seed", 0x27: "hash", 0x3b: "ping"}[operation]
                protocol = ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, minor)])
                query_function = ttlv(0x420074, 5, 1)

                def reply(tree):
                    expected = {
                        0x18: query_function, 0x1e: protocol, 0x3b: b"",
                        0x25: ttlv(0x4200c4, 2, 32),
                        0x26: ttlv(0x4200c2, 8, b"A" * 32),
                        0x08: ttlv(0x42004f, 2, 1) + ttlv(0x4200d4, 2, 0),
                        0x27: ttlv(0x42002b, 1, [ttlv(0x420038, 5, 6)]) + ttlv(0x4200c2, 8, b"abc"),
                    }[operation]
                    self.assertEqual(find(tree, 0x420079), parse(expected))
                    if result == "server_unsupported":
                        return response(operation, minor)
                    if result == "failed" and operation not in (0x08, 0x25, 0x26, 0x27):
                        return response(operation, 1 - minor)
                    payload = {
                        0x18: [ttlv(0x42005c, 5, operation), ttlv(0x42015f, 1, [])],
                        0x1e: [protocol],
                        0x3b: [],
                        0x25: [ttlv(0x4200c2, 8, bytes(31 if result == "failed" else 32))],
                        0x26: [ttlv(0x4200c4, 2, 33 if result == "failed" else 0)],
                        0x08: [ttlv(0x4200d5, 2, 2), ttlv(0x420094, 7, "known-id")]
                              + ([ttlv(0x420094, 7, "other-id")] if result == "failed" else []),
                        0x27: [ttlv(0x4200c2, 8, hashlib.sha256(
                            b"wrong" if result == "failed" else b"abc"
                        ).digest())],
                    }[operation]
                    return response(operation, minor, 0, None, payload)

                with ControlledPeer(self.pki, reply) as peer:
                    _, report = self.command(
                        self.connection(peer.port, version=version, api="generic-ttlv", mutations=operation in (0x25, 0x26)),
                        self.rows({0x25: "test-crypto-rng-retrieve-rng-32",
                                   0x26: "test-crypto-rng-seed-rng-32",
                                   0x27: "test-crypto-hash-sha_256-0"}.get(operation, f"test-op-{name}"))
                    )
                    self.assertEqual(peer.operations, [operation])
                self.assertEqual(report["cases"][0]["status"], result)

    def test_generic_key_lifecycle_uses_created_identifier_and_checks_cleanup(self):
        routes = {"create": [1, 20], "get": [1, 10, 20], "get-attributes": [1, 11, 20], "destroy": [1, 20],
                  "check": [1, 18, 9, 19, 20], "activate": [1, 18, 19, 20], "revoke": [1, 18, 19, 20],
                  "get-attribute-list": [1, 12, 20], "archive": [1, 21, 22, 20], "recover": [1, 21, 22, 20]}
        for version, (target, operations) in product(("2.0", "2.1"), routes.items()):
            for valid in ((True, False) if target in ("get", "get-attributes", "get-attribute-list", "activate", "revoke", "check") else (True,)):
                with self.subTest(version=version, target=target, valid=valid):
                    minor = int(version[-1])
                    owned = "controlled-owned-key"
                    identity = ttlv(0x420094, 7, owned)
                    object_type = ttlv(0x420057, 5, 2)

                    def answer(tree):
                        operation = find(tree, 0x42005c)
                        payload = find(tree, 0x420079)
                        fields = [identity]
                        if operation == 1:
                            self.assertEqual(2, find(payload, 0x420057))
                            self.assertEqual(3, find(payload, 0x420028))
                            self.assertEqual(256, find(payload, 0x42002a))
                            fields.insert(0, object_type)
                        else:
                            self.assertEqual(owned, find(payload, 0x420094))
                        if operation == 10:
                            key = ttlv(0x42008f, 1, [ttlv(0x420040, 1, [
                                ttlv(0x420042, 5, 1),
                                ttlv(0x420045, 1, [ttlv(0x420043, 8, bytes(32))]),
                                ttlv(0x420028, 5, 3), ttlv(0x42002a, 2, 256),
                            ])])
                            fields = [object_type, identity if valid else ttlv(0x420094, 7, "foreign-key"), key]
                        if operation == 11:
                            self.assertEqual(0x42002a, find(payload, 0x42013b))
                            fields.append(ttlv(0x420125, 1, [ttlv(0x42002a, 2, 256 if valid else 128)]))
                        if operation == 12:
                            fields.append(ttlv(0x42013b, 5, 0x420028))
                            if valid:
                                fields.append(ttlv(0x42013b, 5, 0x42002a))
                        if operation == 9:
                            self.assertEqual(4, find(payload, 0x42002c))
                            fields.append(ttlv(0x42002c, 2, 4 if valid else 8))
                        if operation == 19:
                            self.assertEqual([(0x420082, 5, 6)], find(payload, 0x420081))
                        if not valid and ((target == "activate" and operation == 18)
                                          or (target == "revoke" and operation == 19)):
                            fields = [ttlv(0x420094, 7, "foreign-key")]
                        return response(operation, minor, 0, None, fields)

                    with ControlledPeer(self.pki, answer) as peer:
                        _, report = self.command(
                            self.connection(peer.port, version=version, api="generic-ttlv", mutations=True),
                            self.rows(f"test-op-{target}"),
                        )
                        self.assertEqual(operations, peer.operations)
                    self.assertEqual("passed" if valid else "failed", report["cases"][0]["status"])

    def test_discover_versions_accepts_only_the_version_offered_by_each_api(self):
        operation = 0x1e

        def protocol_version(major, minor):
            return ttlv(0x420069, 1, [
                ttlv(0x42006a, 2, major),
                ttlv(0x42006b, 2, minor),
            ])

        for version, api, matches in product(
            ("2.0", "2.1", "3.0"), ("typed-protocol", "fluent"), (True, False)
        ):
            with self.subTest(version=version, api=api, matches=matches):
                major, minor = map(int, version.split("."))
                returned = (major, minor) if matches else ((2, 0) if version == "2.1" else (2, 1))

                def reply(tree):
                    payload = find(tree, 0x420079)
                    self.assertEqual(payload, parse(protocol_version(major, minor)))
                    return response(
                        operation, major=major, minor=minor, status=0, reason=None,
                        payload=[protocol_version(*returned)],
                    )

                cases = self.rows("test-op-discover-versions")
                cases[0] = {
                    **cases[0],
                    "execution": {**cases[0]["execution"], "protocol_versions": [version]},
                }
                with ControlledPeer(self.pki, reply) as peer:
                    _, report = self.command(self.connection(peer.port, version=version, api=api), cases)
                    self.assertEqual(peer.operations, [operation])
                self.assertEqual(report["cases"][0]["status"], "passed" if matches else "failed")

    def test_all_new_kmip30_operations_use_the_versioned_typed_and_fluent_exchanges(self):
        operations = {
            "Create Group": 0x3c, "Obliterate": 0x3d, "Create User": 0x3e,
            "Create Credential": 0x3f, "Deactivate": 0x40,
            "Encapsulate": 0x41, "Decapsulate": 0x42,
        }
        template, = self.rows("test-op-create")
        for (name, target), result_status in product(operations.items(), (0, 1)):
            with self.subTest(operation=name, result_status=result_status):
                case = {
                    **template, "case_id": "test-op-" + name.lower().replace(" ", "-"),
                    "operation": name, "requires": {},
                    "execution": {**template["execution"], "protocol_versions": ["3.0"]},
                    "normative_refs": [{"document": "OASIS KMIP 3.0 Working Draft", "section": "6.1"}],
                }
                observations = []
                for api in ("typed-protocol", "fluent"):
                    def reply(tree):
                        self.assertEqual(3, find(tree, 0x42006a))
                        self.assertIsNone(find(tree, 0x42000d))
                        operation = find(tree, 0x42005c)
                        if operation == target and result_status == 1:
                            return response(operation, minor=0, major=3)
                        identifier = find(tree, 0x420094) or "owned-draft-fixture"
                        payload = [ttlv(0x420094, 12, identifier)]
                        if operation == 1:
                            payload.insert(0, ttlv(0x420057, 5, 2))
                        elif operation == 2:
                            payload = [ttlv(0x420066, 12, "owned-private"),
                                       ttlv(0x42006f, 12, "owned-public")]
                        elif operation == 0x41:
                            payload = [ttlv(0x420094, 12, "owned-encapsulated"),
                                       ttlv(0x4200c2, 8, b"fixture-ciphertext")]
                        elif operation == 0x3D:
                            payload = []
                        elif operation == 0x42:
                            payload = [ttlv(0x420094, 12, "owned-decapsulated")]
                        return response(operation, minor=0, major=3, status=0, reason=None, payload=payload)
                    with ControlledPeer(self.pki, reply) as peer:
                        process, report = self.command(
                            self.connection(peer.port, version="3.0", api=api, mutations=True), [case]
                        )
                    self.assertIsNotNone(report, process.stderr)
                    expected = "passed" if result_status == 0 else "server_unsupported"
                    self.assertEqual(expected, report["cases"][0]["status"], (peer.operations, report))
                    index, = [idx for idx, op in enumerate(peer.operations) if op == target]
                    observations.append((peer.request_trees[index], peer.response_trees[index]))
                self.assertEqual(*observations)

    def test_typed_and_fluent_attribute_requests_match_at_the_peer(self):
        cases = self.rows("test-op-get-attributes", "test-attr-name", "test-op-get-attribute-list")
        self.assertEqual(len(cases), 3)
        for minor in (0, 1):
            for case in cases:
                with self.subTest(case=case["case_id"], minor=minor):
                    observations = []
                    for api in ("typed-protocol", "fluent"):
                        def reply(tree):
                            operation = find(tree, 0x42005c)
                            if operation == 1:
                                payload = [ttlv(0x420057, 5, 2), ttlv(0x420094, 7, "fixture-key")]
                            elif operation == 0x14:
                                payload = [ttlv(0x420094, 7, "fixture-key")]
                            else:
                                return response(operation, minor=minor)
                            return response(operation, minor=minor, status=0, reason=None, payload=payload)

                        with ControlledPeer(self.pki, reply) as peer:
                            _, report = self.command(
                                self.connection(peer.port, version=f"2.{minor}", api=api, mutations=True),
                                [case],
                            )
                        self.assertEqual("server_unsupported", report["cases"][0]["status"], report)
                        operation = 0x0c if case["operation"] == "Get Attribute List" else 0x0b
                        index, = [index for index, observed in enumerate(peer.operations) if observed == operation]
                        observations.append((peer.request_trees[index], peer.response_trees[index]))
                    self.assertEqual(*observations)

    def test_generic_attribute_mutations_verify_the_value_and_cleanup(self):
        operations = {"add": 0x0d, "modify": 0x0e, "delete": 0x0f, "set": 0x31}
        initial = "kmipkit-attribute-fixture"
        changed = "kmipkit-changed-description"
        for minor in (0, 1):
            for name, target in operations.items():
                for persist in (True, False):
                    with self.subTest(minor=minor, operation=name, persist=persist):
                        description = None if name == "add" else initial

                        def reply(tree):
                            nonlocal description
                            operation = find(tree, 0x42005c)
                            payload = [ttlv(0x420094, 7, "fixture-key")]
                            if operation == 1:
                                self.assertEqual(description, find(tree, 0x4200fc))
                                payload.insert(0, ttlv(0x420057, 5, 2))
                            elif operation == 0x0b:
                                self.assertEqual(0x4200fc, find(tree, 0x42013b))
                                if description is not None:
                                    payload.append(ttlv(0x420125, 1, [ttlv(0x4200fc, 7, description)]))
                            elif operation == target:
                                current = find(tree, 0x42013c)
                                new = find(tree, 0x42013d)
                                self.assertEqual(initial if name == "modify" else None, find(current or [], 0x4200fc))
                                if name == "delete":
                                    self.assertEqual(0x4200fc, find(tree, 0x42013b))
                                    self.assertIsNone(new)
                                else:
                                    self.assertEqual(changed, find(new or [], 0x4200fc))
                                if persist:
                                    description = None if name == "delete" else changed
                            else:
                                self.assertEqual(0x14, operation)
                            return response(operation, minor=minor, status=0, reason=None, payload=payload)

                        case_id = f"test-op-{name}-attribute"
                        with ControlledPeer(self.pki, reply) as peer:
                            _, report = self.command(
                                self.connection(peer.port, version=f"2.{minor}", api="generic-ttlv", mutations=True),
                                self.rows(case_id),
                            )
                        self.assertEqual([1, 0x0b, target, 0x0b, 0x14], peer.operations)
                        self.assertEqual("passed" if persist else "failed", report["cases"][0]["status"])

    def test_native_server_query_parity_across_apis_encodings_and_versions(self):
        normalized = []
        for version in ("2.0", "2.1"):
            for encoding in ("TTLV", "JSON", "XML"):
                for api in ("high-level", "typed-protocol", "fluent"):
                    with self.subTest(version=version, encoding=encoding, api=api):
                        _, report = self.command(self.connection(version=version, transport="https", encoding=encoding, api=api), self.rows("test-op-query"))
                        case = report["cases"][0]
                        self.assertEqual(case["status"], "passed", case)
                        if api == "high-level":
                            self.assertEqual(case["actual_api_level"], "high-level")
                        normalized.append((case["status"], case["result_status"], case.get("result_reason")))
        self.assertEqual(len(set(normalized)), 1)




    def test_markdown_preserves_case_outcome_and_redacts_server_text(self):
        before = set(self.directory.rglob("results.md"))
        with ControlledPeer(self.pki, lambda tree: response(0x18, message="secret-markdown-sentinel")) as peer:
            self.command(self.connection(peer.port), self.rows("test-op-query"))
        created = set(self.directory.rglob("results.md")) - before
        self.assertEqual(len(created), 1)
        rendered = created.pop().read_text()
        self.assertIn("test-op-query", rendered)
        self.assertIn("server_unsupported", rendered)
        self.assertNotIn("secret-markdown-sentinel", rendered)

    def test_malformed_response_does_not_blame_the_library(self):
        malformed = ttlv(0x42007b, 1, [ttlv(0x42007d, 7, "secret-malformed-sentinel")])
        with ControlledPeer(self.pki, lambda tree: malformed) as peer:
            process, report = self.command(self.connection(peer.port), self.rows("test-op-query"))
            case = report["cases"][0]
            self.assertEqual(process.returncode, 1)
            self.assertEqual(case["status"], "failed")
            self.assertEqual(case["library_correctness"], "not_checked")
            self.assertEqual(case["server_compatibility"], "unknown")
            self.assertIn("ResponseStarted", case["diagnostic"])
            self.assertNotIn("secret-malformed-sentinel", json.dumps(report))

    def test_preactive_encrypt_accepts_only_the_catalog_rejection_reasons(self):
        for reason, expected in ((7, "passed"), (10, "passed"), (0x43, "failed")):
            with self.subTest(reason=reason):
                def answer(tree):
                    operation = find(tree, 0x42005c)
                    if operation == 0x1f:
                        return response(operation, reason=reason)
                    payload = [ttlv(0x420094, 7, "owned-preactive")]
                    if operation == 1:
                        payload.insert(0, ttlv(0x420057, 5, 2))
                    return response(operation, status=0, reason=None, payload=payload)
                with ControlledPeer(self.pki, answer) as peer:
                    _, report = self.command(self.connection(peer.port, mutations=True), self.rows("test-neg-encrypt-preactive-key"))
                    self.assertEqual(report["cases"][0]["status"], expected)

    def test_request_equivalence_between_high_level_and_typed_apis(self):
        payloads = []
        for api in ("high-level", "typed-protocol"):
            def answer(tree):
                payloads.append(find(tree, 0x420079))
                return response(0x18)
            with ControlledPeer(self.pki, answer) as peer:
                self.command(self.connection(peer.port, api=api), self.rows("test-op-query"))
        expected = [(0x420074, 5, 1), (0x420074, 5, 2), (0x420074, 5, 11)]
        self.assertEqual(payloads, [expected, expected])

    def test_every_outgoing_catalog_row_reaches_its_recipe_or_an_explicit_library_gap(self):
        for version in ("2.0", "2.1"):
            baseline = None
            for api in ("typed-protocol", "high-level"):
                with self.subTest(version=version, api=api):
                    report, requests, responses = self.catalog_scenario(version, api)
                    normalized = parity_signature(report, requests, responses)
                    if baseline is None:
                        baseline = normalized
                    else:
                        self.assertTrue(normalized == baseline, "full-catalog normalized KMIP request/result parity differs")

    def test_fluent_catalog_recipes_match_typed_exchanges(self):
        operations = {
            "Create", "Query", "Discover Versions", "Add Attribute", "Set Attribute",
            "Modify Attribute", "Delete Attribute", "Get Usage Allocation", "RNG Retrieve",
            "Revoke", "Get", "Get Attributes", "Get Attribute List", "Locate", "Ping",
            "Get Constraints", "Activate", "Archive", "Recover", "Destroy",
            "Check", "Obtain Lease", "RNG Seed", "Create Split Key", "Re-key", "Re-key Key Pair", "Register",
            "Hash", "Encrypt", "Decrypt", "MAC", "MAC Verify", "Sign", "Signature Verify",
        }
        for version in ("2.0", "2.1"):
            with self.subTest(version=version):
                cases = [
                    case for case in CATALOG["test_cases"]
                    if case["operation"] in operations
                    and version in case["execution"]["protocol_versions"]
                    and case["requires"].get("workflow_type") not in {
                        "server_to_client", "async_poll", "batch_id_placeholder",
                    }
                    and (case["requires"].get("workflow_type") != "multipart_streaming"
                         or case["operation"] == "Hash")
                    and (case["operation"] != "Create"
                         or case["requires"].get("object_type") in (None, "Symmetric Key"))
                ]
                self.assertGreater(len(cases), 50)
                typed = self.catalog_scenario(version, "typed-protocol", cases)
                fluent = self.catalog_scenario(version, "fluent", cases)
                self.assertEqual(parity_signature(*typed), parity_signature(*fluent))

    def catalog_scenario(self, version, api="typed-protocol", cases=None):
        elements = json.loads((ROOT / "verification/conformance/catalog/kmip-2.1.json").read_text())["elements"]
        operations = {int(e["wire_value"], 16): e["name"] for e in elements if e["kind"] == "enumeration_value" and e["parent_element_ids"] == ["KMIPKIT-ELEM-ENUMERATION-OPERATION"] and e.get("allocation") == "assigned"}
        cipher = (ROOT / "verification/interop/rust-kmip-validator/tests/fixtures/aes256-cbc.bin").read_bytes()
        sequence = 0
        active = set()
        def answer(tree):
            nonlocal sequence
            sequence += 1
            minor = find(tree, 0x42006b)
            results = []
            for tag, kind, batch in tree[0][2]:
                if tag != 0x42000f:
                    continue
                operation = find(batch, 0x42005c)
                name = operations[operation]
                uid = find(batch, 0x420094) or f"owned-{sequence}"
                payload = [ttlv(0x420094, 7, uid)]
                status, reason = 0, None
                if name == "Create" and find(tree, 0x420007) == 1:
                    fields = [ttlv(0x42005c, 5, operation), ttlv(0x42007f, 5, 2), ttlv(0x420006, 8, b"controlled-correlation"), ttlv(0x42007c, 1, [])]
                    results.append(ttlv(0x42000f, 1, fields))
                    continue
                if name == "Create":
                    payload.insert(0, ttlv(0x420057, 5, 2))
                elif name == "Create Key Pair":
                    payload = [ttlv(0x420066, 7, f"owned-private-{sequence}"), ttlv(0x42006f, 7, f"owned-public-{sequence}")]
                elif name == "Create Split Key":
                    payload = [ttlv(0x420094, 7, f"owned-share-{sequence}-1"), ttlv(0x420094, 7, f"owned-share-{sequence}-2")]
                elif name in ("Login", "Delegated Login"):
                    payload = [ttlv(0x420149, 1, [ttlv(0x42014a, 5, 1), ttlv(0x42014b, 8, b"controlled-ticket")])]
                elif name == "Logout":
                    payload = []
                elif name == "Cancel":
                    payload = [ttlv(0x420006, 8, find(batch, 0x420006)), ttlv(0x420012, 5, 1)]
                elif name in ("Encrypt", "Decrypt", "MAC", "Sign", "Hash"):
                    if name == "Encrypt" and uid not in active and find(tree, 0x42000d) == 1:
                        status, reason, payload = 1, 7, None
                    elif find(batch, 0x4200d6) is not None:
                        if name == "Hash":
                            self.assertEqual(find(batch, 0x4200d6), b"controlled-multipart")
                            self.assertIsNone(find(batch, 0x4200d7))
                            if find(batch, 0x4200d8) == 1:
                                self.assertEqual(find(batch, 0x4200c2), b"!")
                                payload = [ttlv(0x4200c2, 8, hashlib.sha256(b"KMIP Hash!").digest())]
                            else:
                                self.assertEqual(find(batch, 0x4200c2), b"Hash")
                                self.assertIsNone(find(batch, 0x4200d8))
                                payload = []
                        else:
                            status, reason, payload = 1, 5, None
                    elif find(batch, 0x4200d7) == 1:
                        if name == "Hash":
                            self.assertEqual(find(batch, 0x4200c2), b"KMIP ")
                            self.assertIsNone(find(batch, 0x4200d8))
                        payload = ([] if name == "Hash" else payload) + [ttlv(0x4200d6, 8, b"controlled-multipart")]
                    else:
                        value_tag = {"MAC": 0x4200c6, "Sign": 0x4200c3}.get(name, 0x4200c2)
                        value = {
                            "Encrypt": cipher,
                            "Hash": hashlib.sha256(find(batch, 0x4200c2) or b"").digest(),
                            "MAC": b"M" * 32,
                        }.get(name, b"A" * 16)
                        payload = ([] if name == "Hash" else payload) + [ttlv(value_tag, 8, value)]
                elif name in ("MAC Verify", "Signature Verify"):
                    payload += [ttlv(0x42009b, 5, 1)]
                elif name not in ("Activate", "Revoke", "Destroy", "Register", "Import", "Archive", "Recover", "Join Split Key", "Certify", "Re-certify", "Re-key"):
                    status, reason, payload = 1, 5, None
                if name == "Activate":
                    active.add(uid)
                if name == "Revoke":
                    active.discard(uid)
                if name == "Destroy" and uid in active:
                    status, reason, payload = 1, 0x43, None
                fields = [ttlv(0x42005c, 5, operation), ttlv(0x42007f, 5, status)]
                if reason is not None:
                    fields.append(ttlv(0x42007e, 5, reason))
                if payload is not None:
                    fields.append(ttlv(0x42007c, 1, payload))
                batch_id = find(batch, 0x420093)
                if batch_id is not None:
                    fields.insert(1, ttlv(0x420093, 8, batch_id))
                results.append(ttlv(0x42000f, 1, fields))
            header = ttlv(0x42007a, 1, [ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, minor)]), ttlv(0x420092, 9, 1794268800), ttlv(0x42000d, 2, len(results))])
            return ttlv(0x42007b, 1, [header, *results])
        full_catalog = cases is None
        cases = CATALOG["test_cases"] if full_catalog else cases
        environment_non_runs = {c["case_id"] for c in cases if c["requires"].get("workflow_type") == "server_to_client" or version not in c["execution"]["protocol_versions"] or version == "2.0" and c["operation"] in {"Ping", "Process", "Query Asynchronous Requests", "Set Defaults"}}
        with ControlledPeer(self.pki, answer) as peer:
            _, report = self.command(self.connection(peer.port, version=version, api=api, mutations=True), cases)
            gaps = [(c["case_id"], c["diagnostic"]) for c in report["cases"] if c["status"] == "not_run" and c["case_id"] not in environment_non_runs and "library_api_unavailable:" not in c["diagnostic"] and "library_preflight_rejection:" not in c["diagnostic"]]
            self.assertEqual(gaps, [])
            failures = [(c["case_id"], c["diagnostic"]) for c in report["cases"] if c["status"] == "failed"]
            self.assertEqual(failures, [])
            by_id = {c["case_id"]: c for c in report["cases"]}
            if full_catalog and version == "2.0":
                for name in ("ping", "process", "query-asynchronous-requests", "set-defaults"):
                    self.assertIn("not_applicable:", by_id["test-op-" + name]["diagnostic"])
                for name in ("login", "logout", "delegated-login"):
                    result = by_id["test-op-" + name]
                    self.assertEqual(result["status"], "passed")
                    self.assertEqual(result["server_compatibility"], "supported")
                    self.assertEqual(result["library_correctness"], "passed")
            self.assertGreater(len(peer.operations), len(cases))
            self.assertEqual(report["summary"]["total"], len(cases))
            evidence = os.environ.get("KMIPKIT_EVIDENCE_DIR")
            if evidence and full_catalog:
                output = Path(evidence)
                output.mkdir(parents=True, exist_ok=True)
                (output / f"rust-controlled-{version}-{api}.json").write_text(json.dumps(report, indent=2) + "\n")
            self.assertEqual(len(peer.request_trees), peer.connections)
            self.assertEqual(len(peer.response_trees), len(peer.request_trees))
            return report, tuple(peer.request_trees), tuple(peer.response_trees)

    def test_incoming_operations_use_public_listener_and_dispatcher(self):
        for version in ("2.0", "2.1", "3.0"):
            with self.subTest(version=version):
                self.incoming_scenario(version)

    def test_incoming_kmip2_uses_generic_ttlv_and_echoes_correlation(self):
        for version in ("2.0", "2.1"):
            with self.subTest(version=version):
                self.incoming_scenario(version, "generic-ttlv")

    def test_incoming_generic_rejects_an_invalid_put_payload(self):
        self.incoming_scenario("2.1", "generic-ttlv", invalid_put=True)

    def incoming_scenario(self, version, api="typed-protocol", invalid_put=False):
        cases = [c for c in CATALOG["test_cases"] if c["requires"].get("workflow_type") == "server_to_client"]
        major, minor = map(int, version.split("."))
        if major == 3:
            cases = [{**case, "execution": {**case["execution"], "protocol_versions": [version]}}
                     for case in cases]
        work = self.directory / f"incoming-ready-{version}-{api}-{invalid_put}"
        payloads = [[], [ttlv(0x420094, 7, "owned-fixture")], [ttlv(0x420094, 12 if major == 3 else 7, "owned-fixture"), ttlv(0x420070, 5, 1), ttlv(0x42005b, 1, [ttlv(0x420059, 5, 1), ttlv(0x42005a, 8, b"fixture")])], [ttlv(0x420074, 5, 1)], [ttlv(0x420151, 5, 1)]]
        operations = [0x1e, 0x1b, 0x1c, 0x18, 0x32]
        if invalid_put:
            cases = [case for case in cases if case["operation"] == "Put"]
            payloads, operations = [[ttlv(0x420094, 7, "missing-put-function")]], [0x1c]
        errors = []
        def send():
            try:
                deadline = time.monotonic() + 10
                while not work.exists() and time.monotonic() < deadline:
                    time.sleep(.01)
                host, port = work.read_text().split(":")
                context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
                context.load_verify_locations(self.pki / "ca.crt")
                context.minimum_version = ssl.TLSVersion.TLSv1_3
                context.load_cert_chain(self.pki / "client.crt", self.pki / "client.key")
                for operation, payload in zip(operations, payloads):
                    with socket.create_connection((host, int(port)), timeout=5) as raw:
                        with context.wrap_socket(raw, server_hostname="localhost") as channel:
                            fields = [ttlv(0x420069, 1, [ttlv(0x42006a, 2, major), ttlv(0x42006b, 2, minor)])]
                            if api == "generic-ttlv":
                                fields.append(ttlv(0x420105, 7, "test-correlation"))
                            if major < 3:
                                fields.append(ttlv(0x42000d, 2, 1))
                            batch = [ttlv(0x42005c, 5, operation)]
                            if api == "generic-ttlv":
                                batch.append(ttlv(0x420093, 8, b"fixture-batch"))
                            batch.append(ttlv(0x420079, 1, payload))
                            channel.sendall(ttlv(0x420078, 1, [ttlv(0x420077, 1, fields),
                                ttlv(0x42000f, 1, batch)]))
                            reply = parse(receive(channel))
                            self.assertEqual((find(reply, 0x42005c), find(reply, 0x42007f),
                                              find(reply, 0x42006a), find(reply, 0x42006b)),
                                             (operation, int(invalid_put), major, minor))
                            if api == "generic-ttlv":
                                self.assertEqual(find(reply, 0x420105), "test-correlation")
                                self.assertEqual(find(reply, 0x420093), b"fixture-batch")
                                self.assertEqual(find(reply, 0x42000d), 1)
                            if invalid_put:
                                self.assertEqual(find(reply, 0x42007e), 7)
                                self.assertIsNone(find(reply, 0x42007c))
                            elif operation == 0x18:
                                response_payload = find(reply, 0x42007c)
                                self.assertEqual([value for tag, _, value in response_payload if tag == 0x42005c],
                                                 [0x1e, 0x1b, 0x1c, 0x18, 0x32])
                            elif operation == 0x32:
                                self.assertEqual(find(reply, 0x420151), 1)
            except Exception as error:
                errors.append(str(error))
        thread = threading.Thread(target=send)
        thread.start()
        _, report = self.command(self.connection(version=version, api=api, mutations=True), cases, "--listen", "127.0.0.1:0", "--listener-cert", str(self.pki / "server.crt"), "--listener-key", str(self.pki / "server.key"), "--listener-ready-file", str(work))
        thread.join(timeout=10)
        self.assertFalse(errors, (errors, report))
        if invalid_put:
            self.assertEqual(report["summary"]["failed"], 1, report["cases"])
            self.assertEqual(report["cases"][0]["result_reason_code"], 7)
        else:
            self.assertEqual(report["summary"]["passed"], 5, report["cases"])


if __name__ == "__main__":
    unittest.main()
