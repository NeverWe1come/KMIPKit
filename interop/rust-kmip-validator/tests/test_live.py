"""Offline mTLS interoperability fixtures: OASIS KMIP 2.0/2.1 §§6-7.

Only request shape/status metadata is retained. Bodies and sentinel secrets are
never written to evidence artifacts. These peers are controlled fixtures, not
vendor compatibility evidence.
"""
import hashlib
import contextlib
import json
import os
from pathlib import Path
import select
import socket
import ssl
import struct
import subprocess
import sys
import tempfile
import threading
import time
import traceback
import unittest
import xml.etree.ElementTree as ET

import jsonschema

import controlled_wire

from request_parity import RequestNormalizer, parity_signature, result_code

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "interop/shared/tools"))
from coverage_matrix import iter_cells
from validate_route_evidence import validate_report

VALIDATOR = Path(os.environ.get("KMIPKIT_VALIDATOR_BIN", ROOT / "interop/rust-kmip-validator/target/debug/rust-kmip-validator"))
SERVER = Path(os.environ.get("KMIPKIT_TEST_SERVER", ROOT / "interop/.local/rust-test-server/harness/target/debug/kmipkit-interop-local-server"))
CATALOG = json.loads((ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
SCHEMA = json.loads((ROOT / "interop/shared/schemas/validation-results.schema.json").read_text())
OPERATION_NAMES_BY_CODE = {
    int(target["wire_value"], 16): target["name"]
    for target in CATALOG["normative_targets"]
    if target["kind"] == "enumeration_value"
    and any("ENUMERATION-OPERATION" in parent for parent in target["parent_element_ids"])
    and all(character in "0123456789abcdefABCDEF" for character in target["wire_value"])
}


def attach_peer_batch_observation(event, batch):
    if "batch_item_count" not in event:
        return
    event["peer_observed_operation_sequence"] = [
        OPERATION_NAMES_BY_CODE[operation] for operation in batch["operations"]
    ]
    event["peer_observed_response_count"] = batch["response_count"]
    if batch["statuses"]:
        event["peer_observed_result_status_codes"] = batch["statuses"]
    if batch["reasons"]:
        event["peer_observed_result_reason_codes"] = batch["reasons"]


def ttlv(tag, kind, value):
    if kind == 1:
        value = b"".join(value)
    elif kind in (2, 5):
        value = struct.pack(">I", value)
    elif kind == 9:
        value = struct.pack(">Q", value)
    elif kind == 6:
        value = struct.pack(">Q", int(value))
    elif kind == 7:
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
        elif kind == 7:
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


def response(operation, minor=1, status=1, reason=5, payload=None, message=None):
    fields = [ttlv(0x42005c, 5, operation), ttlv(0x42007f, 5, status)]
    if reason is not None:
        fields.append(ttlv(0x42007e, 5, reason))
    if message:
        fields.append(ttlv(0x42007d, 7, message))
    if payload is not None:
        fields.append(ttlv(0x42007c, 1, payload))
    return ttlv(0x42007b, 1, [ttlv(0x42007a, 1, [ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, minor)]), ttlv(0x420092, 9, 1794268800), ttlv(0x42000d, 2, 1)]), ttlv(0x42000f, 1, fields)])


def successful_batch(tree):
    request_items = [value for tag, kind, value in tree[0][2] if tag == 0x42000f]
    results = []
    for request_item in request_items:
        operation = find(request_item, 0x42005c)
        fields = [ttlv(0x42005c, 5, operation)]
        batch_id = find(request_item, 0x420093)
        if batch_id is not None:
            fields.append(ttlv(0x420093, 8, batch_id))
        fields.append(ttlv(0x42007f, 5, 0))
        fields.append(ttlv(0x42007c, 1, []))
        results.append(ttlv(0x42000f, 1, fields))
    major = find(tree, 0x42006a)
    minor = find(tree, 0x42006b)
    header = ttlv(0x42007a, 1, [
        ttlv(0x420069, 1, [ttlv(0x42006a, 2, major), ttlv(0x42006b, 2, minor)]),
        ttlv(0x420092, 9, 1794268800),
        ttlv(0x42000d, 2, len(results)),
    ])
    return ttlv(0x42007b, 1, [header, *results])


def operation_failed_batch(tree):
    request_items = [value for tag, kind, value in tree[0][2] if tag == 0x42000f]
    continuation = find(tree, 0x42000e)
    missing_indexes = [
        index for index, request_item in enumerate(request_items)
        if find(request_item, 0x420094) == "kmipkit-route-missing-object"
    ]
    failure_index = missing_indexes[0] if missing_indexes else 0
    statuses = [1] * len(request_items)
    if continuation == 1:
        statuses = [0] * len(request_items)
        statuses[failure_index] = 1
    elif continuation == 2:
        statuses = [0] * len(request_items)
        statuses[failure_index] = 1
    elif continuation == 3:
        statuses = [3] * len(request_items)
        statuses[failure_index] = 1
    results = []
    response_items = request_items[:failure_index + 1] if continuation == 2 else request_items
    for index, request_item in enumerate(response_items):
        operation = find(request_item, 0x42005c)
        fields = [ttlv(0x42005c, 5, operation)]
        batch_id = find(request_item, 0x420093)
        if batch_id is not None:
            fields.append(ttlv(0x420093, 8, batch_id))
        fields.append(ttlv(0x42007f, 5, statuses[index]))
        if statuses[index] == 1:
            fields.append(ttlv(0x42007e, 5, 5))
        else:
            payload = [ttlv(0x420094, 7, "controlled-batch-object")]
            if operation == 0x0c:
                payload.append(ttlv(0x42013b, 5, 0x420001))
            fields.append(ttlv(0x42007c, 1, payload))
        results.append(ttlv(0x42000f, 1, fields))
    major = find(tree, 0x42006a)
    minor = find(tree, 0x42006b)
    header = ttlv(0x42007a, 1, [
        ttlv(0x420069, 1, [ttlv(0x42006a, 2, major), ttlv(0x42006b, 2, minor)]),
        ttlv(0x420092, 9, 1794268800),
        ttlv(0x42000d, 2, len(results)),
    ])
    return ttlv(0x42007b, 1, [header, *results])


def successful_lifecycle_batch(tree, operation_codes):
    request_items = [value for tag, kind, value in tree[0][2] if tag == 0x42000F]
    results = []
    for request_item in request_items:
        operation = find(request_item, 0x42005C)
        fields = [ttlv(0x42005C, 5, operation)]
        batch_id = find(request_item, 0x420093)
        if batch_id is not None:
            fields.append(ttlv(0x420093, 8, batch_id))
        fields.append(ttlv(0x42007F, 5, 0))
        unique_identifier = [ttlv(0x420094, 7, "controlled-batch-object")]
        if operation == operation_codes["Create"]:
            payload = [ttlv(0x420057, 5, 2), *unique_identifier]
        elif operation == operation_codes["Get Attribute List"]:
            payload = [*unique_identifier, ttlv(0x42013B, 5, 0x420001)]
        else:
            payload = unique_identifier
        fields.append(ttlv(0x42007C, 1, payload))
        results.append(ttlv(0x42000F, 1, fields))
    major = find(tree, 0x42006A)
    minor = find(tree, 0x42006B)
    header = ttlv(0x42007A, 1, [
        ttlv(0x420069, 1, [ttlv(0x42006A, 2, major), ttlv(0x42006B, 2, minor)]),
        ttlv(0x420092, 9, 1794268800),
        ttlv(0x42000D, 2, len(results)),
    ])
    return ttlv(0x42007B, 1, [header, *results])


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


class ControlledPeer:
    def __init__(self, pki, responder, encoding="TTLV", transport="raw_tls"):
        self.encoding, self.transport = encoding, transport
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
        self.route_request_tree_hashes = []
        self.route_batches = []
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
                    data = controlled_wire.receive(channel, self.encoding, self.transport, receive)
                    if data is None:
                        continue
                    tree = parse(data) if self.encoding == "TTLV" else controlled_wire.decode(data, self.encoding)
                    self.request_trees.append(self.normalizer.normalize(tree))
                    route_tree = RequestNormalizer().normalize(tree)
                    self.route_request_tree_hashes.append(hashlib.sha256(
                        json.dumps(route_tree, separators=(",", ":")).encode()
                    ).hexdigest())
                    request_items = [batch for tag, kind, batch in tree[0][2] if tag == 0x42000f]
                    batch_record = {
                        "operations": [find(item, 0x42005c) for item in request_items],
                        "response_count": 0,
                        "statuses": [],
                        "reasons": [],
                    }
                    self.route_batches.append(batch_record)
                    self.operations.extend(batch_record["operations"])
                    reply = self.responder(tree)
                    if reply is not None:
                        parsed_reply = parse(reply)
                        response_items = [item for tag, kind, item in parsed_reply[0][2]
                                          if tag == 0x42000F]
                        batch_record["response_count"] = len(response_items)
                        batch_record["statuses"] = [find(item, 0x42007F)
                                                     for item in response_items]
                        batch_record["reasons"] = [find(item, 0x42007E)
                                                    for item in response_items
                                                    if find(item, 0x42007E) is not None]
                        self.normalizer.observe_response(parsed_reply)
                        self.response_trees.append(self.normalizer.normalize(parsed_reply))
                        wire_reply = reply if self.encoding == "TTLV" else controlled_wire.encoded(parsed_reply, self.encoding)
                        controlled_wire.send(channel, wire_reply, self.encoding, self.transport)
        except Exception as error:
            if not self.stop.is_set():
                self.errors.append(type(error).__name__ + ":" + ",".join(f"{Path(frame.filename).name}:{frame.lineno}" for frame in traceback.extract_tb(error.__traceback__)))

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
        fixture_temp = ROOT / "interop/rust-kmip-validator/target/fixture-tmp"
        fixture_temp.mkdir(parents=True, exist_ok=True)
        cls.temp = tempfile.TemporaryDirectory(dir=fixture_temp)
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
        applicable = [c for c in cases if "2.1" in c["execution"]["protocol_versions"]]
        with ControlledPeer(self.pki, successful_batch) as peer:
            process, report = self.command(self.connection(peer.port, mutations=True), cases)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(report["summary"]["total"], len(cases))
        self.assertEqual(report["summary"]["passed"], len(applicable))
        self.assertEqual([find(tree, 0x42000d) for tree in peer.request_trees], [2, 1])
        single_header = peer.request_trees[-1]
        self.assertIsNone(find(single_header, 0x42000E))
        self.assertIsNone(find(single_header, 0x420010))
        self.assertTrue(all(c["server_compatibility"] == "unknown" for c in report["cases"]))

    def test_response_accessor_matrix_runs_through_controlled_mtls(self):
        route_manifest = json.loads((ROOT / "interop/shared/coverage/route-capabilities.json").read_text())
        configuration_fingerprints = []
        response_symbols = {
            "kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value",
            "kmipkit_client::ClientBatchItemResponse::outcome",
            "kmipkit_client::ClientBatchItemResponse::unique_batch_item_id",
            "kmipkit_client::ClientBatchItemResponse::extensions",
            "kmipkit_client::ClientBatchItemResponse::discover_versions_supported_versions",
            "kmipkit_client::ClientBatchResponse::get",
            "kmipkit_client::ClientBatchResponse::len",
            "kmipkit_client::ClientMessageExtension::with_ttlv",
            "kmipkit_client::ClientResponseView::ping",
        }
        for version in ("2.0", "2.1"):
            with self.subTest(version=version):
                minor = int(version[-1])
                cells = [cell for cell in iter_cells(CATALOG, route_manifest, languages=("rust",))
                         if cell["mode"] == "local" and cell["protocol_version"] == version
                         and cell["public_api_symbol"] in response_symbols]
                matrix = self.directory / f"response-accessor-matrix-{version}.jsonl"
                output = self.directory / f"response-accessor-evidence-{version}.json"
                matrix.write_text("".join(json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n" for cell in cells))
                message_extension = ttlv(0x420051, 1, [
                    ttlv(0x42009d, 7, "FixtureVendor"),
                    ttlv(0x420026, 6, 0),
                    ttlv(0x42009c, 1, [
                        ttlv(0x540173, 8, b"fixture-v1"),
                        ttlv(0x540174, 5, 0xffffffff),
                    ]),
                ])

                def answer(tree):
                    request_item = next(value for tag, kind, value in tree[0][2] if tag == 0x42000f)
                    operation = find(request_item, 0x42005c)
                    fields = [ttlv(0x42005c, 5, operation)]
                    batch_id = find(request_item, 0x420093)
                    if batch_id is not None:
                        fields.append(ttlv(0x420093, 8, batch_id))
                    if operation == 0x1f:
                        fields.extend([
                            ttlv(0x42007f, 5, 2),
                            ttlv(0x420006, 8, b"controlled-correlation"),
                            ttlv(0x42007c, 1, []),
                        ])
                    else:
                        fields.append(ttlv(0x42007f, 5, 0))
                        if operation == 0x1e:
                            payload = [ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, 1)])]
                            fields.append(ttlv(0x42007c, 1, payload))
                        else:
                            fields.append(ttlv(0x42007c, 1, []))
                            if operation == 0x18:
                                fields.append(message_extension)
                    header = [
                        ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, minor)]),
                        ttlv(0x420092, 9, 1794268800),
                        ttlv(0x42000d, 2, 1),
                    ]
                    return ttlv(0x42007b, 1, [ttlv(0x42007a, 1, header), ttlv(0x42000f, 1, fields)])

                with ControlledPeer(self.pki, answer) as peer:
                    configuration = self.directory / f"response-accessor-connection-{version}.json"
                    configuration.write_text(json.dumps(self.connection(peer.port, version=version), sort_keys=True))
                    process = subprocess.run(
                        [str(VALIDATOR), "matrix-network", "--matrix", str(matrix), "--connection", str(configuration), "--output", str(output)],
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, text=True,
                    )
                self.assertEqual(process.returncode, 0, process.stderr)
                report = json.loads(output.read_text())
                jsonschema.Draft202012Validator(json.loads((ROOT / "interop/shared/schemas/route-execution-evidence.schema.json").read_text())).validate(report)
                configuration_fingerprints.append(report["server"]["configuration_fingerprint"])
                self.assertEqual(len(report["events"]), len(cells))
                self.assertTrue(all(event["status"] == "passed" and event["invocation_count"] == 1 for event in report["events"]))
                self.assertEqual(
                    [(event["encoding"], event["transport"]) for event in report["events"]],
                    [(cell["encoding"], cell["transport"]) for cell in cells],
                )
                expected_operations = [0x18, 0x1e, 0x1f] + ([0x3b] if version == "2.1" else [])
                self.assertCountEqual(peer.operations, expected_operations)
                evidence = os.environ.get("KMIPKIT_EVIDENCE_DIR")
                if evidence:
                    evidence_path = Path(evidence)
                    evidence_path.mkdir(parents=True, exist_ok=True)
                    (evidence_path / f"rust-response-routes-{version}.json").write_text(json.dumps(report, indent=2) + "\n")
        self.assertEqual(1, len(set(configuration_fingerprints)),
                         "protocol versions must share the same controlled-peer configuration fingerprint")

    def test_typed_query_route_runs_through_controlled_mtls(self):
        """KMIP 2.1 §6.1.40: construct, send, and decode a typed Query route."""
        route_manifest = json.loads((ROOT / "interop/shared/coverage/route-capabilities.json").read_text())
        cells = [
            cell for cell in iter_cells(CATALOG, route_manifest, languages=("rust",))
            if cell["mode"] == "network"
            and cell["case_id"] == "test-op-query"
            and cell["protocol_version"] == "2.1"
            and cell["encoding"] == "TTLV"
            and cell["transport"] == "raw_tls"
        ]
        self.assertEqual({"construct", "send", "decode"}, {cell["action"] for cell in cells})
        with ControlledPeer(
            self.pki,
            lambda tree: response(0x18, status=0, reason=None, payload=[]),
        ) as peer:
            matrix = self.directory / "typed-query-network-matrix.jsonl"
            output = self.directory / "typed-query-network-evidence.json"
            configuration = self.directory / "typed-query-network-connection.json"
            matrix.write_text("".join(
                json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n"
                for cell in cells
            ))
            configuration.write_text(json.dumps(self.connection(peer.port), sort_keys=True))
            process = subprocess.run(
                [str(VALIDATOR), "matrix-network", "--matrix", str(matrix),
                 "--connection", str(configuration), "--output", str(output)],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, text=True,
            )
        self.assertEqual(process.returncode, 0, process.stderr)
        report = json.loads(output.read_text())
        request_hashes = [
            hashlib.sha256(json.dumps(tree, separators=(",", ":")).encode()).hexdigest()
            for tree in peer.request_trees
        ]
        self.assertEqual(len(report["events"]), len(request_hashes))
        for event, request_hash in zip(report["events"], request_hashes, strict=True):
            if event["action"] in {"construct", "send"}:
                event["request_tree_sha256"] = request_hash
        output.write_text(json.dumps(report, indent=2) + "\n")
        evidence_schema = json.loads(
            (ROOT / "interop/shared/schemas/route-execution-evidence.schema.json").read_text()
        )
        jsonschema.Draft202012Validator(evidence_schema).validate(report)
        self.assertEqual(6, len(report["events"]))
        self.assertEqual({"construct", "send", "decode"}, {event["action"] for event in report["events"]})
        self.assertEqual(
            {
                "rust.typed-protocol.client_batch_item.new",
                "rust.typed-protocol.query_request.new",
            },
            {event["route_id"] for event in report["events"]},
        )
        for route_id in {event["route_id"] for event in report["events"]}:
            self.assertEqual(
                {"construct", "send", "decode"},
                {event["action"] for event in report["events"] if event["route_id"] == route_id},
            )
        self.assertTrue(all(event["status"] == "passed" for event in report["events"]))
        self.assertEqual([0x18] * 6, peer.operations)

    def test_network_request_route_matrix_calls_the_named_constructor_and_executor(self):
        """KMIP 2.1 §6.1.37 Query constructor and typed-batch execution route."""
        route_manifest = json.loads((ROOT / "interop/shared/coverage/route-capabilities.json").read_text())
        cells = [cell for cell in iter_cells(CATALOG, route_manifest, languages=("rust",))
                 if cell["mode"] == "network"
                 and cell["public_api_symbol"] == "kmipkit_protocol::QueryRequest::new"
                 and cell["case_id"] == "test-op-query"
                 and cell["protocol_version"] == "2.1"
                 and cell["encoding"] == "TTLV"
                 and cell["transport"] == "raw_tls"]
        self.assertEqual(["construct", "send", "decode"], [cell["action"] for cell in cells])
        matrix = self.directory / "query-network-route-matrix.jsonl"
        output = self.directory / "query-network-route-evidence.json"
        configuration = self.directory / "query-network-route-connection.json"
        matrix.write_text("".join(json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n"
                                   for cell in cells))
        with ControlledPeer(self.pki, successful_batch) as peer:
            configuration.write_text(json.dumps(self.connection(peer.port), sort_keys=True))
            process = subprocess.run(
                [str(VALIDATOR), "matrix-network", "--matrix", str(matrix),
                 "--connection", str(configuration), "--output", str(output)],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, text=True,
            )
        self.assertEqual(0, process.returncode, process.stderr)
        report = json.loads(output.read_text())
        self.assertEqual(len(report["events"]), len(peer.route_request_tree_hashes))
        for event, request_hash in zip(report["events"], peer.route_request_tree_hashes, strict=True):
            if event["action"] in {"construct", "send"}:
                event["request_tree_sha256"] = request_hash
        output.write_text(json.dumps(report, indent=2) + "\n")
        jsonschema.Draft202012Validator(json.loads(
            (ROOT / "interop/shared/schemas/route-execution-evidence.schema.json").read_text()
        )).validate(report)
        self.assertEqual([0x18, 0x18, 0x18], peer.operations)
        self.assertEqual(3, len(report["events"]))
        self.assertEqual(["construct", "send", "decode"], [event["action"] for event in report["events"]])
        self.assertTrue(all(event["status"] == "passed" for event in report["events"]))
        self.assertTrue(all(event["request_tree_sha256"] for event in report["events"][:2]))
        self.assertRegex(report["events"][2]["response_tree_sha256"], r"^[0-9a-f]{64}$")

    def test_generic_batch_item_route_uses_each_workflow_operation(self):
        """KMIP 2.0 §§6.1, 8.2: ClientBatchItem::new wraps the operation for each step."""
        route_manifest = json.loads((ROOT / "interop/shared/coverage/route-capabilities.json").read_text())
        operation_targets = {
            "KMIPKIT-ELEM-OP-C2S-CREATE",
            "KMIPKIT-ELEM-OP-C2S-ACTIVATE",
            "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST",
            "KMIPKIT-ELEM-OP-C2S-REVOKE",
            "KMIPKIT-ELEM-OP-C2S-DESTROY",
        }
        rows = [cell for cell in iter_cells(CATALOG, route_manifest, languages=("rust",))
                if cell["mode"] == "network"
                and cell["public_api_symbol"] == "kmipkit_client::ClientBatchItem::new"
                and cell["case_id"] == "test-batch-create-activate-destroy"
                and cell["target_id"] in operation_targets
                and cell["protocol_version"] == "2.0"
                and cell["encoding"] == "TTLV"
                and cell["transport"] == "raw_tls"
                and cell["action"] == "send"]
        expected_names = ["Create", "Activate", "Get Attribute List", "Revoke", "Destroy"]
        self.assertEqual(expected_names, [row["workflow_operation"] for row in rows])
        operation_codes = {
            element["name"]: int(element["wire_value"], 16)
            for element in json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text())["elements"]
            if element["kind"] == "enumeration_value"
            and element.get("parent_element_ids") == ["KMIPKIT-ELEM-ENUMERATION-OPERATION"]
            and element.get("allocation") == "assigned"
        }
        matrix = self.directory / "generic-batch-item-workflow.jsonl"
        output = self.directory / "generic-batch-item-workflow-evidence.json"
        configuration = self.directory / "generic-batch-item-workflow-connection.json"
        matrix.write_text("".join(
            json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n"
            for cell in rows
        ))
        with ControlledPeer(self.pki, operation_failed_batch) as peer:
            configuration.write_text(json.dumps(
                self.connection(peer.port, version="2.0", mutations=True), sort_keys=True
            ))
            process = subprocess.run(
                [str(VALIDATOR), "matrix-network", "--matrix", str(matrix),
                 "--connection", str(configuration), "--output", str(output)],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, text=True,
            )
        self.assertEqual(0, process.returncode, process.stderr)
        report = json.loads(output.read_text())
        expected_batch = [operation_codes[name] for name in expected_names]
        self.assertEqual(expected_batch * len(rows), peer.operations)
        self.assertEqual(5, len(report["events"]))
        self.assertTrue(all(event["status"] == "passed" for event in report["events"]))
        for event, request_hash, batch in zip(
            report["events"], peer.route_request_tree_hashes, peer.route_batches, strict=True
        ):
            event["request_tree_sha256"] = request_hash
            attach_peer_batch_observation(event, batch)
        jsonschema.Draft202012Validator(json.loads(
            (ROOT / "interop/shared/schemas/route-execution-evidence.schema.json").read_text()
        )).validate(report)

    def test_batch_continue_and_stop_options_match_ordered_peer_outcomes(self):
        route_manifest = json.loads((ROOT / "interop/shared/coverage/route-capabilities.json").read_text())
        schema = json.loads((ROOT / "interop/shared/schemas/route-execution-evidence.schema.json").read_text())
        catalog_bytes = (ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_bytes()
        manifest_bytes = (ROOT / "interop/shared/coverage/route-capabilities.json").read_bytes()
        for case_id, expected_codes in (
            ("test-batch-stop-on-first-failure", [1]),
            ("test-batch-continue-after-failure", [1, 0]),
        ):
            with self.subTest(case_id=case_id):
                rows = [cell for cell in iter_cells(CATALOG, route_manifest, languages=("rust",))
                        if cell["mode"] == "network"
                        and cell["public_api_symbol"] == "kmipkit_client::ClientBatchItem::new"
                        and cell["case_id"] == case_id
                        and cell["protocol_version"] == "2.0"
                        and cell["encoding"] == "TTLV"
                        and cell["transport"] == "raw_tls"
                        and cell["action"] == "send"]
                matrix = self.directory / f"{case_id}.jsonl"
                output = self.directory / f"{case_id}-evidence.json"
                configuration = self.directory / f"{case_id}-connection.json"
                matrix.write_text("".join(
                    json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n"
                    for cell in rows
                ))
                with ControlledPeer(self.pki, operation_failed_batch) as peer:
                    configuration.write_text(json.dumps(
                        self.connection(peer.port, version="2.0", mutations=True), sort_keys=True
                    ))
                    process = subprocess.run(
                        [str(VALIDATOR), "matrix-network", "--matrix", str(matrix),
                         "--connection", str(configuration), "--output", str(output)],
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30, text=True,
                    )
                self.assertEqual(
                    0,
                    process.returncode,
                    process.stderr or process.stdout or (output.read_text() if output.exists() else "no evidence"),
                )
                report = json.loads(output.read_text())
                self.assertEqual(len(rows), len(report["events"]))
                for event, request_hash, batch in zip(
                    report["events"], peer.route_request_tree_hashes, peer.route_batches, strict=True
                ):
                    event["request_tree_sha256"] = request_hash
                    attach_peer_batch_observation(event, batch)
                self.assertEqual(expected_codes, report["events"][0]["peer_observed_result_status_codes"])
                jsonschema.Draft202012Validator(schema).validate(report)
                validate_report(
                    report,
                    CATALOG,
                    route_manifest,
                    catalog_sha256=hashlib.sha256(catalog_bytes).hexdigest(),
                    route_manifest_sha256=hashlib.sha256(manifest_bytes).hexdigest(),
                    expected_artifact_sha256=report["artifact_sha256"],
                    scope_cells=rows,
                )

    def test_complete_network_route_matrix_runs_against_controlled_mtls_peers(self):
        """KMIP 2.0/2.1 §§6-9: every declared request route and transport axis."""
        route_manifest = json.loads((ROOT / "interop/shared/coverage/route-capabilities.json").read_text())
        all_cells = [cell for cell in iter_cells(CATALOG, route_manifest, languages=("rust",))
                     if cell["mode"] == "network"]
        combinations = sorted({
            (cell["protocol_version"], cell["encoding"], cell["transport"])
            for cell in all_cells
        })
        selected = os.environ.get("KMIPKIT_ROUTE_MATRIX_FILTER")
        if selected:
            combinations = [combination for combination in combinations
                            if ":".join(combination) == selected]
            self.assertTrue(combinations, "route-matrix filter must select a declared combination")
        evidence_root = Path(os.environ.get(
            "KMIPKIT_EVIDENCE_DIR", ROOT / "interop/rust-kmip-validator/results"
        )) / "routes"
        evidence_root.mkdir(parents=True, exist_ok=True)
        for version, encoding, transport in combinations:
            with self.subTest(version=version, encoding=encoding, transport=transport):
                cells = [cell for cell in all_cells
                         if (cell["protocol_version"], cell["encoding"], cell["transport"])
                         == (version, encoding, transport)]
                matrix = self.directory / f"network-{version}-{encoding}-{transport}.jsonl"
                output = evidence_root / f"rust-network-routes-{version}-{encoding.lower()}-{transport}.json"
                configuration = self.directory / f"network-{version}-{encoding}-{transport}-connection.json"
                matrix.write_text("".join(
                    json.dumps(cell, sort_keys=True, separators=(",", ":")) + "\n"
                    for cell in cells
                ))
                with ControlledPeer(
                    self.pki,
                    operation_failed_batch,
                    encoding=encoding,
                    transport=transport,
                ) as peer:
                    configuration.write_text(json.dumps(
                        self.connection(peer.port, version=version, encoding=encoding,
                                        transport=transport, mutations=True),
                        sort_keys=True,
                    ))
                    process = subprocess.run(
                        [str(VALIDATOR), "matrix-network", "--matrix", str(matrix),
                         "--connection", str(configuration), "--output", str(output)],
                        stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                        timeout=600, text=True,
                    )
                self.assertEqual(0, process.returncode, process.stderr)
                report = json.loads(output.read_text())
                self.assertEqual(len(cells), len(report["events"]))
                self.assertEqual(len(report["events"]), len({
                    (event["case_id"], event["route_id"], event["target_id"],
                     event.get("workflow_step_index"), event.get("workflow_item_index"),
                     event["action"])
                    for event in report["events"]
                }))
                sent_events = [event for event in report["events"]
                               if event.get("transport_invocation_count") == 1]
                self.assertEqual(len(sent_events), len(peer.route_request_tree_hashes))
                for event, request_hash, batch in zip(
                    sent_events, peer.route_request_tree_hashes, peer.route_batches, strict=True
                ):
                    if event["status"] == "passed" and event["action"] in {"construct", "send"}:
                        event["request_tree_sha256"] = request_hash
                    attach_peer_batch_observation(event, batch)
                jsonschema.Draft202012Validator(json.loads(
                    (ROOT / "interop/shared/schemas/route-execution-evidence.schema.json").read_text()
                )).validate(report)
                output.write_text(json.dumps(report, indent=2) + "\n")

    def test_query_probes_profile_omissions_and_separates_rejections(self):
        for reason, status, compatibility in [(5, "server_unsupported", "unsupported"), (8, "server_unsupported", "unsupported"), (12, "policy_restricted", "policy_restricted"), (3, "failed", "unknown"), (0x100, "failed", "unknown")]:
            with self.subTest(reason=reason), ControlledPeer(self.pki, lambda tree: response(0x18, reason=reason, message="secret-response-sentinel")) as peer:
                _, report = self.command(self.connection(peer.port), self.rows("test-op-query"))
                case = report["cases"][0]
                self.assertEqual(case["status"], status)
                self.assertEqual(case["server_compatibility"], compatibility)
                self.assertEqual(case["library_correctness"], "passed")
                self.assertEqual(peer.operations, [0x18])
                self.assertNotIn("secret-response-sentinel", json.dumps(report))

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

    def test_batch_create_activate_get_attribute_list_revoke_destroy_matches_catalog_order(self):
        """KMIP 2.1 §§6.1.1, 6.1.8, 6.1.15, 6.1.21, and 6.1.44 batch flow."""
        case = self.rows("test-batch-create-activate-destroy")
        scenario = case[0]["requires"]["protocol_scenario"]
        expected_names = scenario["parameters"]["operations_in_request_order"]
        operation_codes = {
            target["name"]: int(target["wire_value"], 16)
            for target in CATALOG["normative_targets"]
            if target["kind"] == "enumeration_value"
            and any("ENUMERATION-OPERATION" in parent for parent in target["parent_element_ids"])
            and all(character in "0123456789abcdefABCDEF" for character in target["wire_value"])
        }
        expected_codes = [operation_codes[name] for name in expected_names]

        with ControlledPeer(
            self.pki,
            lambda tree: successful_lifecycle_batch(tree, operation_codes),
        ) as peer:
            process, report = self.command(
                self.connection(peer.port, mutations=True),
                case,
            )

        self.assertEqual(peer.operations, expected_codes)
        self.assertEqual(process.returncode, 0, process.stderr or report)

    def test_version_boundary_is_rejected_before_sending(self):
        cases = [case for case in CATALOG["test_cases"] if case["case_id"].startswith("test-v20-reject-")]
        with ControlledPeer(self.pki, lambda tree: response(0x18)) as peer:
            _, report = self.command(self.connection(peer.port, version="2.0"), cases)
            self.assertEqual(report["summary"]["passed"], len(cases))
            self.assertEqual(peer.operations, [])
            self.assertTrue(all(c["server_compatibility"] == "unknown" for c in report["cases"]))

    def test_native_server_query_parity_across_apis_encodings_and_versions(self):
        normalized = []
        for version in ("2.0", "2.1"):
            for encoding in ("TTLV", "JSON", "XML"):
                for api in ("high-level", "typed-protocol"):
                    with self.subTest(version=version, encoding=encoding, api=api):
                        _, report = self.command(self.connection(version=version, transport="https", encoding=encoding, api=api), self.rows("test-op-query"))
                        case = report["cases"][0]
                        self.assertEqual(case["status"], "passed", case)
                        normalized.append((case["status"], case["result_status"], case.get("result_reason")))
        self.assertEqual(len(set(normalized)), 1)




    def test_controlled_query_peer_checks_every_encoding_transport_pair(self):
        baseline = None
        for version in ("2.0", "2.1"):
            for encoding in ("TTLV", "JSON", "XML"):
                for transport in ("raw_tls", "https"):
                    with self.subTest(version=version, encoding=encoding, transport=transport):
                        with ControlledPeer(self.pki, lambda tree: response(0x18, minor=int(version[-1]), status=0, reason=None, payload=[])) as peer:
                            peer.encoding, peer.transport = encoding, transport
                            process, report = self.command(self.connection(peer.port, version=version, encoding=encoding, transport=transport), self.rows("test-op-query"))
                        self.assertEqual(process.returncode, 0, process.stderr)
                        self.assertEqual(peer.operations, [0x18])
                        normalized = (report["cases"][0]["status"], report["cases"][0]["result_status"])
                        if baseline is None:
                            baseline = normalized
                        self.assertEqual(normalized, baseline)

    def test_hash_does_not_accept_wrong_successful_digest(self):
        expected = hashlib.sha256(b"A" * 16).digest()
        for value, outcome in ((expected, "passed"), (b"bad-controlled-digest", "failed")):
            with self.subTest(outcome=outcome), ControlledPeer(self.pki, lambda tree: response(0x27, status=0, reason=None, payload=[ttlv(0x4200c2, 8, value)])) as peer:
                _, report = self.command(self.connection(peer.port, mutations=True), self.rows("test-op-hash"))
            self.assertEqual(report["cases"][0]["status"], outcome)

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

    def test_catalog_exchanges_run_every_wire_axis_through_both_api_levels(self):
        for version in ("2.0", "2.1"):
            baseline = None
            for encoding in ("TTLV", "JSON", "XML"):
                for transport in ("raw_tls", "https"):
                    for api in ("typed-protocol", "high-level"):
                        self.fixture_axes = encoding, transport
                        with self.subTest(version=version, api=api, encoding=encoding, transport=transport):
                            report, requests, responses = self.catalog_scenario(version, api)
                            self.assertEqual(report['execution']['encoding'], encoding)
                            self.assertEqual(report['execution']['transport'], transport)
                            signature = parity_signature(report, requests, responses)
                            if baseline is None:
                                baseline = signature
                            self.assertEqual(signature, baseline, 'normalized catalog request/response parity differs across axes')
        del self.fixture_axes

    def catalog_scenario(self, version, api="typed-protocol"):
        encoding, transport = getattr(self, "fixture_axes", ("TTLV", "raw_tls"))
        elements = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text())["elements"]
        operations = {int(e["wire_value"], 16): e["name"] for e in elements if e["kind"] == "enumeration_value" and e["parent_element_ids"] == ["KMIPKIT-ELEM-ENUMERATION-OPERATION"] and e.get("allocation") == "assigned"}
        cipher = (ROOT / "interop/rust-kmip-validator/tests/fixtures/aes256-cbc.bin").read_bytes()
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
                elif name == "Query":
                    payload = []
                elif name == "Logout":
                    payload = []
                elif name == "Cancel":
                    payload = [ttlv(0x420006, 8, find(batch, 0x420006)), ttlv(0x420012, 5, 1)]
                elif name in ("Encrypt", "Decrypt", "MAC", "Sign", "Hash"):
                    if name == "Encrypt" and uid not in active and find(tree, 0x42000d) == 1:
                        status, reason, payload = 1, 7, None
                    elif find(batch, 0x4200d6) is not None:
                        status, reason, payload = 1, 5, None
                    elif find(batch, 0x4200d7) == 1:
                        payload = ([] if name == "Hash" else payload) + [ttlv(0x4200d6, 8, b"controlled-multipart")]
                    else:
                        value_tag = {"MAC": 0x4200c6, "Sign": 0x4200c3}.get(name, 0x4200c2)
                        value = cipher if name == "Encrypt" else hashlib.sha256(b"A" * 16).digest() if name == "Hash" else b"A" * 16
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
        cases = CATALOG["test_cases"]
        environment_non_runs = {c["case_id"] for c in cases if c["requires"].get("workflow_type") == "server_to_client" or version not in c["execution"]["protocol_versions"] or version == "2.0" and c["operation"] in {"Ping", "Process", "Query Asynchronous Requests", "Set Defaults"}}
        with ControlledPeer(self.pki, answer) as peer:
            peer.encoding, peer.transport = encoding, transport
            _, report = self.command(self.connection(peer.port, version=version, api=api, encoding=encoding, transport=transport, mutations=True), cases)
            gaps = [(c["case_id"], c["diagnostic"]) for c in report["cases"] if c["status"] == "not_run" and c["case_id"] not in environment_non_runs and "library_api_unavailable:" not in c["diagnostic"] and "library_preflight_rejection:" not in c["diagnostic"]]
            self.assertEqual(gaps, [])
            failures = [(c["case_id"], c["diagnostic"]) for c in report["cases"] if c["status"] == "failed"]
            self.assertEqual(failures, [])
            by_id = {c["case_id"]: c for c in report["cases"]}
            if version == "2.0":
                for name in ("ping", "process", "query-asynchronous-requests", "set-defaults"):
                    self.assertIn("not_applicable:", by_id["test-op-" + name]["diagnostic"])
                for name in ("login", "logout", "delegated-login"):
                    result = by_id["test-op-" + name]
                    self.assertEqual(result["status"], "passed")
                    self.assertEqual(result["server_compatibility"], "supported")
                    self.assertEqual(result["library_correctness"], "passed")
            outgoing = [
                c for c in cases
                if c["execution"]["mode"] == "server_exchange"
                and c["requires"].get("workflow_type") != "server_to_client"
                and version in c["execution"]["protocol_versions"]
            ]
            self.assertGreaterEqual(
                len(peer.operations), len(outgoing),
                "controlled peer did not receive each applicable outbound catalog case",
            )
            self.assertEqual(report["summary"]["total"], len(CATALOG["test_cases"]))
            evidence = os.environ.get("KMIPKIT_EVIDENCE_DIR")
            if evidence:
                output = Path(evidence)
                output.mkdir(parents=True, exist_ok=True)
                (output / f"rust-controlled-{version}-{api}-{encoding}-{transport}.json").write_text(json.dumps(report, indent=2) + "\n")
            self.assertEqual(len(peer.request_trees), peer.connections)
            self.assertEqual(len(peer.response_trees), len(peer.request_trees))
            return report, tuple(peer.request_trees), tuple(peer.response_trees)

    def test_incoming_operations_use_public_listener_and_dispatcher(self):
        for version in ("2.0", "2.1"):
            with self.subTest(version=version):
                self.incoming_scenario(version)

    def incoming_scenario(self, version):
        cases = [c for c in CATALOG["test_cases"] if c["requires"].get("workflow_type") == "server_to_client"]
        work = self.directory / f"incoming-ready-{version}"
        payloads = [[], [ttlv(0x420094, 7, "owned-fixture")], [ttlv(0x420094, 7, "owned-fixture"), ttlv(0x420070, 5, 1), ttlv(0x42005b, 1, [ttlv(0x420059, 5, 1), ttlv(0x42005a, 8, b"fixture")])], [ttlv(0x420074, 5, 1)], [ttlv(0x420151, 5, 1)]]
        operations = [0x1e, 0x1b, 0x1c, 0x18, 0x32]
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
                            channel.sendall(ttlv(0x420078, 1, [ttlv(0x420077, 1, [ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, int(version[-1]))]), ttlv(0x42000d, 2, 1)]), ttlv(0x42000f, 1, [ttlv(0x42005c, 5, operation), ttlv(0x420079, 1, payload)])]))
                            reply = parse(receive(channel))
                            if find(reply, 0x42005c) != operation or find(reply, 0x42007f) != 0 or find(reply, 0x42006b) != int(version[-1]):
                                raise AssertionError(f"incoming response contract mismatch: expected operation={operation}, observed operation={find(reply, 0x42005c)}, status={find(reply, 0x42007f)}")
            except Exception as error:
                errors.append(str(error))
        thread = threading.Thread(target=send)
        thread.start()
        _, report = self.command(self.connection(version=version), cases, "--listen", "127.0.0.1:0", "--listener-cert", str(self.pki / "server.crt"), "--listener-key", str(self.pki / "server.key"), "--listener-ready-file", str(work))
        thread.join(timeout=10)
        self.assertFalse(errors, errors)
        self.assertEqual(report["summary"]["passed"], 5, report["cases"])


if __name__ == "__main__":
    unittest.main()
