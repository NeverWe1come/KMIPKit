"""Offline mTLS interoperability fixtures: OASIS KMIP 2.0/2.1 §§6-7.

Only request shape/status metadata is retained. Bodies and sentinel secrets are
never written to evidence artifacts. These peers are controlled fixtures, not
vendor compatibility evidence.
"""
import contextlib
import json
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

ROOT = Path(__file__).resolve().parents[3]
VALIDATOR = ROOT / "interop/rust-kmip-validator/target/debug/rust-kmip-validator"
SERVER = Path(os.environ.get("KMIPKIT_TEST_SERVER", ROOT / "interop/.local/rust-test-server/harness/target/debug/kmipkit-interop-local-server"))
CATALOG = json.loads((ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text())
SCHEMA = json.loads((ROOT / "interop/shared/schemas/validation-results.schema.json").read_text())


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
        elif kind in (2, 5, 6, 9):
            value = int.from_bytes(value, "big")
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
                    self.operations.append(find(tree, 0x42005c))
                    reply = self.responder(tree)
                    if reply is not None:
                        channel.sendall(reply)
        except Exception as error:
            if not self.stop.is_set():
                self.errors.append(str(error))

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
        args = [str(VALIDATOR), "run", "--connection", str(work / "connection.json"), "--catalog", str(work / "catalog.json"), "--capabilities", str(work / "profile.json"), "--output", str(work / "results.json"), *extra]
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
        for reason, status, compatibility in [(5, "server_unsupported", "unsupported"), (12, "policy_restricted", "policy_restricted"), (0x100, "failed", "unknown")]:
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

    def test_version_boundary_is_rejected_before_sending(self):
        with ControlledPeer(self.pki, lambda tree: response(0x18)) as peer:
            _, report = self.command(self.connection(peer.port, version="2.0"), self.rows("test-v20-reject-get-constraints", "test-v20-reject-set-constraints"))
            self.assertEqual(report["summary"]["passed"], 2)
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

    def test_incoming_operations_use_public_listener_and_dispatcher(self):
        cases = [c for c in CATALOG["test_cases"] if c["requires"].get("workflow_type") == "server_to_client"]
        work = self.directory / "incoming-ready"
        payloads = [[], [ttlv(0x420094, 7, "owned-fixture")], [ttlv(0x420094, 7, "owned-fixture"), ttlv(0x4200b1, 5, 1), ttlv(0x42005b, 1, [ttlv(0x420059, 5, 1), ttlv(0x42005a, 8, b"fixture")])], [ttlv(0x420074, 5, 1)], [ttlv(0x420151, 5, 1)]]
        operations = [0x1e, 0x1d, 0x1c, 0x18, 0x32]
        errors = []
        def send():
            try:
                deadline = time.monotonic() + 10
                while not work.exists() and time.monotonic() < deadline:
                    time.sleep(.01)
                host, port = work.read_text().split(":")
                context = ssl.create_default_context(cafile=str(self.pki / "ca.crt"))
                context.minimum_version = ssl.TLSVersion.TLSv1_3
                context.load_cert_chain(self.pki / "client.crt", self.pki / "client.key")
                for operation, payload in zip(operations, payloads):
                    with socket.create_connection((host, int(port)), timeout=5) as raw:
                        with context.wrap_socket(raw, server_hostname="localhost") as channel:
                            channel.sendall(ttlv(0x420078, 1, [ttlv(0x420077, 1, [ttlv(0x420069, 1, [ttlv(0x42006a, 2, 2), ttlv(0x42006b, 2, 1)]), ttlv(0x42000d, 2, 1)]), ttlv(0x42000f, 1, [ttlv(0x42005c, 5, operation), ttlv(0x420079, 1, payload)])]))
                            reply = parse(receive(channel))
                            if find(reply, 0x42005c) != operation or find(reply, 0x42007f) != 0:
                                raise AssertionError("incoming response contract mismatch")
            except Exception as error:
                errors.append(str(error))
        thread = threading.Thread(target=send)
        thread.start()
        _, report = self.command(self.connection(), cases, "--listen", "127.0.0.1:0", "--listener-cert", str(self.pki / "server.crt"), "--listener-key", str(self.pki / "server.key"), "--listener-ready-file", str(work))
        thread.join(timeout=10)
        self.assertFalse(errors, errors)
        self.assertEqual(report["summary"]["passed"], 5, report["cases"])


if __name__ == "__main__":
    unittest.main()
