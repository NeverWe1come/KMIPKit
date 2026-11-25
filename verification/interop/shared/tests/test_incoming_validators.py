"""Controlled peers for the independently installed KMIP 2.x direct tag listeners."""

from functools import lru_cache
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import socket
import ssl
import struct
import subprocess
import sys
import tempfile
import time
import unittest

from verification.interop.shared.tools.operation_catalog import build_catalog, direction, with_cases

ROOT = Path(__file__).resolve().parents[4]
INTEROP = ROOT / "verification/interop"
PKI = ROOT / "bindings/python/tests/fixtures/login-mtls"
JAR_CONSUMER = INTEROP / "java-kmip-validator"
PYTHON_CONSUMER = INTEROP / "python-kmip-validator"
PYTHON_MANIFEST = PYTHON_CONSUMER / "build/artifact-manifest.json"
RUST = Path(os.environ.get(
    "KMIPKIT_VALIDATOR_BIN", INTEROP / "rust-kmip-validator/target/debug/rust-kmip-validator",
)).resolve()
C = INTEROP / "c-kmip-validator/build/kmip_validator_c"
JAVA = JAR_CONSUMER / "build/install/java-kmip-validator/bin/java-kmip-validator"
OPERATIONS = {
    "Discover Versions": 0x1E, "Notify": 0x1B, "Put": 0x1C, "Query": 0x18, "Set Endpoint Role": 0x32,
}


def wire_item(tag, kind, payload):
    return tag.to_bytes(3, "big") + bytes([kind]) + struct.pack(">I", len(payload)) + payload + bytes(-len(payload) % 8)


def structure(tag, *fields):
    return wire_item(tag, 1, b"".join(fields))


def number(tag, kind, value):
    return wire_item(tag, kind, value.to_bytes(4, "big"))


def version(minor):
    return structure(0x420069, number(0x42006A, 2, 2), number(0x42006B, 2, minor))


def offered_payload(name, minor, invalid):
    if name == "Discover Versions":
        return [version(minor), version(1 - minor)]
    if name == "Notify":
        return [wire_item(0x420094, 7, b"fixture-notification")]
    if name == "Put":
        fields = [wire_item(0x420094, 7, b"fixture-object")]
        if not invalid:
            fields += [
                number(0x420070, 5, 1),
                structure(0x42005B, number(0x420059, 5, 1), wire_item(0x42005A, 8, b"fixture")),
            ]
        return fields
    if name == "Query":
        return [number(0x420074, 5, 1), number(0x420074, 5, 3)]
    return [number(0x420151, 5, 1)]


def read_exact(stream, length):
    value = bytearray()
    while len(value) < length:
        chunk = stream.recv(length - len(value))
        if not chunk:
            raise ValueError("Listener closed without returning a complete TTLV frame")
        value.extend(chunk)
    return value


def parse_items(wire):
    items = []
    while wire:
        if len(wire) < 8:
            raise ValueError("Truncated TTLV header")
        size = int.from_bytes(wire[4:8], "big")
        end = 8 + size
        next_item = end + (-size % 8)
        if len(wire) < next_item or any(wire[end:next_item]):
            raise ValueError("Truncated TTLV value or non-zero padding")
        tag, kind = int.from_bytes(wire[:3], "big"), wire[3]
        value = bytes(wire[8:end])
        items.append((tag, kind, parse_items(value) if kind == 1 else value))
        wire = wire[next_item:]
    return items


def only(items, tag, kind):
    matches = [(actual_kind, value) for actual_tag, actual_kind, value in items if actual_tag == tag]
    if len(matches) != 1 or matches[0][0] != kind:
        raise ValueError(f"Expected one TTLV tag 0x{tag:06x} of type {kind}")
    return matches[0][1]


@lru_cache(maxsize=2)
def version_catalog(version):
    return build_catalog(ROOT, version)


def installed_validators():
    validators = (
        ("rust", RUST.is_file()), ("c", C.is_file()), ("java", JAVA.is_file()),
        ("python", PYTHON_MANIFEST.is_file() and importlib.util.find_spec("kmipkit") is not None),
    )
    required = os.environ.get("KMIPKIT_REQUIRED_INTEROP_VALIDATOR")
    if not required:
        return validators
    if not dict(validators).get(required):
        raise AssertionError(f"Required installed validator is unavailable: {required}")
    return ((required, True),)



def connection_config(minor, port, *, incoming):
    trust = "client-cert.pem" if incoming else "server-cert.pem"
    return {
        "server_id": "controlled-fixture", "protocol_version": f"2.{minor}",
        "server_product": "controlled", "server_version": "1", "server_configuration_fingerprint": "a" * 64,
        "transport": "raw_tls", "encoding": "TTLV", "api_level": "generic-ttlv",
        "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": port}},
        "tls": {"ca_cert_path": str(PKI / trust), "client_cert_path": str(PKI / "client-cert.pem"),
                "client_key_path": str(PKI / "client-key.pem"), "read_timeout_seconds": 5,
                "server_name_override": "localhost"},
        "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
        "execution_policy": {"probe_unsupported": True, "allow_mutations": True},
    }


def read_message(stream):
    frame = read_exact(stream, 8)
    size = int.from_bytes(frame[4:8], "big")
    frame += read_exact(stream, size + (-size % 8))
    return parse_items(frame)


def save_evidence(report, catalog, label):
    destination = os.environ.get("KMIPKIT_TABLE_EVIDENCE_DIR")
    if not destination:
        return
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=True)
    report_bytes = report.read_bytes()
    catalog_bytes = catalog.read_bytes()
    digest = hashlib.sha256(catalog_bytes).hexdigest()
    (destination / f"catalog-{digest}.json").write_bytes(catalog_bytes)
    (destination / f"results-{label}.json").write_bytes(report_bytes)


class DirectTagValidatorTests(unittest.TestCase):
    def test_each_installed_validator_receives_direct_tags(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                for invalid in (False, True):
                    with self.subTest(language=language, minor=minor, invalid_put=invalid):
                        if not available:
                            self.skipTest(f"Installed {language} validator required")
                        self.exchange(language, minor, invalid)

    def test_isolated_key_attributes_use_their_own_tag_and_expected_value(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                for tag, kind, value, case_id in (
                    (0x420028, 5, 3, "test-attr-cryptographic-algorithm"),
                    (0x42002A, 2, 256, "test-op-get-attributes"),
                    (0x42002A, 2, 256, "test-attr-cryptographic-length"),
                    (0x42002C, 2, 4, "test-attr-cryptographic-usage-mask"),
                    (0x420057, 5, 2, "test-attr-object-type"),
                    (0x42008D, 5, 1, "test-attr-state"),
                    (0x420022, 7, b"kmipkit-attribute-fixture", "test-attr-contact-information"),
                    (0x4200FC, 7, b"kmipkit-attribute-fixture", "test-attr-description"),
                    (0x4200FD, 7, b"kmipkit-attribute-fixture", "test-attr-comment"),
                    (0x420120, 6, 1, "test-attr-sensitive"),
                    (0x420121, 6, 1, "test-attr-always-sensitive"),
                    (0x420122, 6, 0, "test-attr-extractable"),
                    (0x420123, 6, 1, "test-attr-never-extractable"),
                ):
                    for invalid in (False, True):
                        with self.subTest(language=language, minor=minor, case=case_id, invalid=invalid):
                            if not available:
                                self.skipTest(f"Installed {language} validator required")
                            self.owned_key_exchange(language, minor, case_id, 11,
                                                    attribute=(tag, kind, value), invalid=invalid)

    def test_archive_unsupported_response_preserves_cleanup(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                with self.subTest(language=language, minor=minor):
                    if not available:
                        self.skipTest(f"Installed {language} validator required")
                    self.owned_key_exchange(language, minor, "test-op-archive", 21, unsupported=True)

    def test_obtain_lease_validates_result_and_cleans_up(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                for verdict in ("passed", "failed", "server_unsupported"):
                    with self.subTest(language=language, minor=minor, verdict=verdict):
                        if not available:
                            self.skipTest(f"Installed {language} validator required")
                        self.owned_key_exchange(language, minor, "test-op-obtain-lease", 16,
                                                invalid=verdict == "failed",
                                                unsupported=verdict == "server_unsupported")

    def test_multipart_hash_across_api_levels(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                for route in ("typed-protocol", "fluent", "generic-ttlv"):
                    for invalid in (False, True):
                        with self.subTest(language=language, minor=minor, route=route, invalid=invalid):
                            if not available:
                                self.skipTest(f"Installed {language} validator required")
                            self.multipart_hash(language, minor, route, invalid)

    def multipart_hash(self, language, minor, route, invalid):
        correlation = b"validator-peer-correlation"
        digest = bytearray(hashlib.sha256(b"KMIP Hash!").digest())
        if invalid:
            digest[-1] ^= 1
        with tempfile.TemporaryDirectory(prefix="kmipkit-multipart-hash-") as dirname:
            work = Path(dirname)
            catalog_path = INTEROP / "shared/catalog/kmipkit-1.0-test-catalog.json"
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"}, "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(10)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = route
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(catalog_path), "--connection", str(connection),
                            "--case", "test-multipart-hash-sha256"]
                tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
                tls.minimum_version = ssl.TLSVersion.TLSv1_3
                tls.verify_mode = ssl.CERT_REQUIRED
                tls.load_verify_locations(cafile=PKI / "client-cert.pem")
                tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        for stage, chunk in enumerate((b"KMIP ", b"Hash", b"!")):
                            raw, _ = self.accept_request(listener, process, f"Hash stage {stage + 1}")
                            raw.settimeout(5)
                            with raw, tls.wrap_socket(raw, server_side=True) as channel:
                                request = only(read_message(channel), 0x420078, 1)
                                batch = only(request, 0x42000F, 1)
                                operation = only(batch, 0x42005C, 5)
                                self.assertEqual(operation, struct.pack(">I", 0x27))
                                payload = only(batch, 0x420079, 1)
                                self.assertEqual(only(payload, 0x4200C2, 8), chunk)
                                parameters = only(payload, 0x42002B, 1)
                                self.assertEqual(only(parameters, 0x420038, 5), struct.pack(">I", 6))
                                framing = {tag: value for tag, _, value in payload if tag in (0x4200D6, 0x4200D7, 0x4200D8)}
                                expected = ({0x4200D7: struct.pack(">Q", 1)} if stage == 0
                                            else {0x4200D6: correlation} if stage == 1
                                            else {0x4200D6: correlation, 0x4200D8: struct.pack(">Q", 1)})
                                self.assertEqual(framing, expected)
                                fields = ([wire_item(0x4200D6, 8, correlation)] if stage == 0
                                          else [wire_item(0x4200C2, 8, digest)] if stage == 2 else [])
                                batch_ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                                channel.sendall(structure(
                                    0x42007B,
                                    structure(0x42007A, version(minor),
                                              wire_item(0x420092, 9, struct.pack(">Q", 1)),
                                              number(0x42000D, 2, 1)),
                                    structure(0x42000F, wire_item(0x42005C, 5, operation), *batch_ids,
                                              number(0x42007F, 5, 0), structure(0x42007C, *fields)),
                                ))
                        _, stderr = process.communicate(timeout=10)
                        self.assertEqual(process.returncode != 0, invalid, stderr.decode(errors="replace"))
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            evidence, = json.loads(report.read_text())["cases"]
            self.assertEqual(evidence["status"], "failed" if invalid else "passed")
            if not invalid:
                self.assertEqual(evidence["result_status_code"], 0)
                save_evidence(report, catalog_path, f"{language}-2.{minor}-{route}-multipart-hash")

    def accept_request(self, listener, process, stage):
        try:
            return listener.accept()
        except TimeoutError as error:
            if process.poll() is not None:
                stdout, stderr = process.communicate()
                self.fail(f"{stage} did not reach the peer. Exit {process.returncode}: "
                          f"{(stdout + stderr).decode(errors='replace')}")
            raise AssertionError(f"{stage} did not reach the peer") from error

    def owned_key_exchange(self, language, minor, case_id, target, *, attribute=None, invalid=False, unsupported=False):
        tag, kind, value = attribute or (None, None, None)
        identifier = wire_item(0x420094, 7, b"owned-attribute-fixture")
        with tempfile.TemporaryDirectory(prefix="kmipkit-read-attributes-") as dirname:
            work = Path(dirname)
            catalog_path = INTEROP / "shared/catalog/kmipkit-1.0-test-catalog.json"
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"}, "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(5)
                connection = work / "connection.json"
                connection.write_text(json.dumps(connection_config(minor, listener.getsockname()[1], incoming=False)))
                command += ["--catalog", str(catalog_path), "--connection", str(connection), "--case", case_id]
                tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
                tls.minimum_version = ssl.TLSVersion.TLSv1_3
                tls.verify_mode = ssl.CERT_REQUIRED
                tls.load_verify_locations(cafile=PKI / "client-cert.pem")
                tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        for operation in (1, target, 20):
                            raw, _ = self.accept_request(listener, process, f"operation {operation}")
                            raw.settimeout(5)
                            with raw, tls.wrap_socket(raw, server_side=True) as channel:
                                request = only(read_message(channel), 0x420078, 1)
                                batch = only(request, 0x42000F, 1)
                                self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", operation))
                                payload = only(batch, 0x420079, 1)
                                if operation == 1:
                                    attributes = only(payload, 0x420125, 1)
                                    self.assertEqual(only(attributes, 0x420028, 5), struct.pack(">I", 3))
                                    self.assertEqual(only(attributes, 0x42002A, 2), struct.pack(">I", 256))
                                    self.assertEqual(only(attributes, 0x42002C, 2), struct.pack(">I", 4))
                                    if kind == 6:
                                        self.assertEqual(only(attributes, 0x420120, 6), struct.pack(">Q", 1))
                                        self.assertEqual(only(attributes, 0x420122, 6), struct.pack(">Q", 0))
                                    if kind == 7:
                                        self.assertEqual(only(attributes, tag, kind), value)
                                    fields = [number(0x420057, 5, 2), identifier]
                                else:
                                    self.assertEqual(only(payload, 0x420094, 7), b"owned-attribute-fixture")
                                    fields = [identifier]
                                    if operation == 11:
                                        self.assertEqual(only(payload, 0x42013B, 5), struct.pack(">I", tag))
                                        if kind == 7:
                                            attribute = wire_item(tag, kind, b"wrong value" if invalid else value)
                                        elif kind == 6:
                                            attribute = wire_item(tag, kind, struct.pack(">Q", value ^ int(invalid)))
                                        else:
                                            attribute = number(tag, kind, value + int(invalid))
                                        fields.append(structure(0x420125, attribute))
                                    if operation == 16:
                                        fields.extend((number(0x420049, 2 if invalid else 10, 60),
                                                       wire_item(0x420048, 9, struct.pack(">q", 1))))
                                response = ([number(0x42007F, 5, 1), number(0x42007E, 5, 5)]
                                            if operation == target and unsupported else
                                            [number(0x42007F, 5, 0), structure(0x42007C, *fields)])
                                channel.sendall(structure(
                                    0x42007B,
                                    structure(0x42007A, version(minor),
                                              wire_item(0x420092, 9, struct.pack(">Q", 1)),
                                              number(0x42000D, 2, 1)),
                                    structure(0x42000F, number(0x42005C, 5, operation),
                                              *response),
                                ))
                        _, stderr = process.communicate(timeout=10)
                        if not unsupported:
                            self.assertEqual(process.returncode != 0, invalid, stderr.decode(errors="replace"))
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            evidence, = json.loads(report.read_text())["cases"]
            self.assertEqual(evidence["case_id"], case_id)
            if unsupported:
                self.assertEqual(evidence["status"], "server_unsupported")
                self.assertEqual(evidence["result_status_code"], 1)
                self.assertEqual(evidence["result_reason_code"], 5)
            else:
                self.assertEqual(evidence["status"], "failed" if invalid else "passed")
            if not invalid:
                result = "unsupported" if unsupported else "success"
                save_evidence(report, catalog_path, f"{language}-2.{minor}-{case_id}-{result}")

    def exchange(self, language, minor, invalid):
        protocol_version = f"2.{minor}"
        catalog = version_catalog(protocol_version)
        cases = [case for case in catalog["test_cases"] if direction(case) == "server_to_client"
                 and (not invalid or case["operation"] == "Put")]
        self.assertEqual(len(cases), 1 if invalid else 5)
        with tempfile.TemporaryDirectory(prefix="kmipkit-incoming-") as dirname:
            work = Path(dirname)
            catalog_path = work / "catalog.json"
            catalog_path.write_text(json.dumps(with_cases(catalog, cases)))
            connection = work / "connection.json"
            connection.write_text(json.dumps(connection_config(minor, 1, incoming=True)))
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"}, "operations": {}}))
            ready = work / "ready"
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            command += [
                "--catalog", str(catalog_path), "--connection", str(connection),
                "--listen", "127.0.0.1:0", "--listener-cert", str(PKI / "server-cert.pem"),
                "--listener-key", str(PKI / "server-key.pem"), "--listener-ready-file", str(ready),
            ]
            with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                try:
                    peer = self.wait_for_listener(process, ready)
                    tls = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
                    tls.minimum_version = ssl.TLSVersion.TLSv1_3
                    tls.load_verify_locations(cafile=PKI / "server-cert.pem")
                    tls.load_cert_chain(PKI / "client-cert.pem", PKI / "client-key.pem")
                    for case in cases:
                        self.trigger(tls, peer, case["operation"], minor, invalid)
                    _, stderr = process.communicate(timeout=15)
                    self.assertEqual(process.returncode != 0, invalid, stderr.decode(errors="replace"))
                finally:
                    if process.poll() is None:
                        process.terminate()
                        process.communicate(timeout=5)
            results = json.loads(report.read_text())
            self.assertEqual(results["execution"]["api_level"], "generic-ttlv")
            self.assertEqual([case["case_id"] for case in results["cases"]], [case["case_id"] for case in cases])
            for evidence in results["cases"]:
                self.assertEqual(evidence["status"], "failed" if invalid else "passed")
                self.assertEqual(evidence["result_status_code"], int(invalid))
                self.assertEqual(evidence.get("result_reason_code"), 7 if invalid else None)
            if not invalid:
                save_evidence(report, catalog_path, f"{language}-2.{minor}-incoming")

    def wait_for_listener(self, process, ready):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and process.poll() is None:
            if ready.exists() and (address := ready.read_text()):
                host, port = address.rsplit(":", 1)
                return host, int(port)
            time.sleep(.01)
        _, stderr = process.communicate(timeout=5)
        self.fail(f"Listener did not become ready: {stderr.decode(errors='replace')}")

    def trigger(self, context, peer, name, minor, invalid):
        operation = OPERATIONS[name]
        payload = offered_payload(name, minor, invalid)
        packet = structure(
            0x420078,
            structure(0x420077, version(minor), wire_item(0x420105, 7, b"test-correlation"), number(0x42000D, 2, 1)),
            structure(0x42000F, number(0x42005C, 5, operation), wire_item(0x420093, 8, b"test-item"),
                      structure(0x420079, *payload)),
        )
        with socket.create_connection(peer, timeout=5) as tcp, context.wrap_socket(tcp, server_hostname="localhost") as channel:
            channel.sendall(packet)
            response = only(read_message(channel), 0x42007B, 1)
        header = only(response, 0x42007A, 1)
        batch = only(response, 0x42000F, 1)
        self.assertEqual(only(header, 0x420069, 1), only(parse_items(version(minor)), 0x420069, 1))
        self.assertEqual(len(only(header, 0x420092, 9)), 8)
        self.assertEqual(only(header, 0x420105, 7), b"test-correlation")
        self.assertEqual(only(header, 0x42000D, 2), struct.pack(">I", 1))
        self.assertEqual(only(batch, 0x420093, 8), b"test-item")
        self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", operation))
        self.assertEqual(only(batch, 0x42007F, 5), struct.pack(">I", int(invalid)))
        if invalid:
            self.assertEqual(only(batch, 0x42007E, 5), struct.pack(">I", 7))
            self.assertFalse(any(tag == 0x42007C for tag, _, _ in batch))
        elif name == "Discover Versions":
            self.assertEqual(only(batch, 0x42007C, 1), parse_items(version(minor)))
        elif name == "Query":
            result = only(batch, 0x42007C, 1)
            self.assertEqual([int.from_bytes(value, "big") for tag, kind, value in result
                              if tag == 0x42005C and kind == 5], list(OPERATIONS.values()))
            self.assertEqual(only(result, 0x42009D, 7), b"KMIPKit")
            self.assertEqual(only(result, 0x420088, 1), [])
        elif name == "Set Endpoint Role":
            self.assertEqual(only(only(batch, 0x42007C, 1), 0x420151, 5), struct.pack(">I", 1))
        else:
            self.assertEqual(only(batch, 0x42007C, 1), [])

    def command(self, language, profile, report):
        env = dict(os.environ)
        if language == "rust":
            return [str(RUST), "run", "--capabilities", str(profile), "--output", str(report)], env
        if language == "c":
            return [str(C), "run", "--allow-mutations", "--output", str(report)], env
        if language == "java":
            return [str(JAVA), "run", "--enable-mutations", "--output-dir", str(report.parent),
                    "--artifact-manifest", str(JAR_CONSUMER / "build/artifact-manifest.json")], env
        env["PYTHONPATH"] = str(PYTHON_CONSUMER / "src")
        env.setdefault("KMIPKIT_INTEROP_LIBRARY_COMMIT", json.loads(PYTHON_MANIFEST.read_text())["library_commit"])
        return [sys.executable, "-m", "kmipkit_validator", "run", "--enable-mutations",
                "--output-dir", str(report.parent), "--artifact-manifest", str(PYTHON_MANIFEST)], env
