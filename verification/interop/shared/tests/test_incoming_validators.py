"""Controlled peers for the independently installed KMIP 2.x direct tag listeners."""

from functools import lru_cache
import hashlib
import importlib.util
import json
import os
import select
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
from verification.interop.shared.tools.table_asymmetric_key_exchanges import asymmetric_key_cases
from verification.interop.shared.tools.table_authenticated_encryption import authenticated_encryption_cases
from verification.interop.shared.tools.table_key_block_exchanges import key_block_cases
from verification.interop.shared.tools.table_query_fields import nonce_case, query_cases
from verification.interop.shared.tools.table_rotate_boundaries import rotate_cases

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

    def test_each_installed_validator_receives_notify_and_put_through_typed_listener(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                for invalid in (False, True):
                    with self.subTest(language=language, minor=minor, invalid=invalid):
                        if not available:
                            self.skipTest(f"Installed {language} validator required")
                        self.exchange(language, minor, invalid, api="typed-protocol")

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

    def test_rotate_attributes_cross_the_version_boundary_through_each_api(self):
        for language, available in installed_validators():
            routes = ["generic-ttlv", "typed-protocol", "fluent"]
            if language == "rust":
                routes.append("high-level")
            for minor in (0, 1):
                for api in routes:
                    with self.subTest(language=language, minor=minor, api=api):
                        if not available:
                            self.skipTest(f"Installed {language} validator required")
                        self.exchange_rotate_attributes(language, minor, api)

    def exchange_rotate_attributes(self, language, minor, api):
        protocol_version = f"2.{minor}"
        catalog = version_catalog(protocol_version)
        base = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-locate")
        cases = [case for _, case in rotate_cases(ROOT, protocol_version, base)]
        self.assertEqual(7, len(cases))
        expected = [parse_items(bytes.fromhex(case["requires"]["rotate_attribute_ttlv"])) for case in cases]
        with tempfile.TemporaryDirectory(prefix="kmipkit-rotate-boundary-") as dirname:
            work = Path(dirname)
            catalog_path = work / "catalog.json"
            catalog_path.write_text(json.dumps(with_cases(catalog, cases)))
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"},
                                           "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.minimum_version = ssl.TLSVersion.TLSv1_3
            tls.verify_mode = ssl.CERT_REQUIRED
            tls.load_verify_locations(cafile=PKI / "client-cert.pem")
            tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(15)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = api
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(catalog_path), "--connection", str(connection)]
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        if minor == 1:
                            for _ in cases:
                                raw, _ = self.accept_request(listener, process, "Locate with Rotate attribute")
                                raw.settimeout(5)
                                with raw, tls.wrap_socket(raw, server_side=True) as peer:
                                    request = only(read_message(peer), 0x420078, 1)
                                    batch = only(request, 0x42000F, 1)
                                    self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", 8))
                                    payload = only(batch, 0x420079, 1)
                                    self.assertEqual(only(payload, 0x42004F, 2), struct.pack(">i", 1))
                                    self.assertEqual(only(payload, 0x4200D4, 2), struct.pack(">i", 0))
                                    expected.remove(only(payload, 0x420125, 1))
                                    batch_ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                                    peer.sendall(structure(
                                        0x42007B,
                                        structure(0x42007A, version(minor), wire_item(0x420092, 9, bytes(8)),
                                                  number(0x42000D, 2, 1)),
                                        structure(0x42000F, number(0x42005C, 5, 8), *batch_ids,
                                                  number(0x42007F, 5, 0), structure(0x42007C, number(0x4200D5, 2, 0))),
                                    ))
                            self.assertFalse(expected)
                        stdout, stderr = process.communicate(timeout=25)
                        self.assertEqual(process.returncode, 0, (stdout + stderr).decode(errors="replace"))
                        self.assertFalse(select.select([listener], [], [], 0)[0], "unexpected additional connection")
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            evidence = json.loads(report.read_text())
            self.assertEqual(api, evidence["execution"]["api_level"])
            self.assertEqual({case["case_id"] for case in cases}, {case["case_id"] for case in evidence["cases"]})
            for case in evidence["cases"]:
                self.assertEqual(("passed", "passed"), (case["status"], case["library_correctness"]))
                self.assertEqual(0 if minor == 1 else None, case.get("result_status_code"))
                if api == "high-level":
                    self.assertEqual(api, case.get("actual_api_level"))
            save_evidence(report, catalog_path, f"{language}-{protocol_version}-{api}-rotate-boundary")

    def test_certificate_workflows_do_not_report_prerequisite_failure_as_target_evidence(self):
        for language, available in installed_validators():
            routes = ("generic-ttlv", "typed-protocol") if language == "java" else ("generic-ttlv",)
            for minor in (0, 1):
                for api in routes:
                    with self.subTest(language=language, minor=minor, api=api):
                        if not available:
                            self.skipTest(f"Installed {language} validator required")
                        self.reject_certificate_prerequisite(language, minor, api)

    def reject_certificate_prerequisite(self, language, minor, api):
        catalog = version_catalog(f"2.{minor}")
        names = {"test-op-certify", "test-op-re-certify", "test-op-validate"}
        cases = [case for case in catalog["test_cases"] if case["case_id"] in names]
        self.assertEqual(len(names), len(cases))
        tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        tls.minimum_version = ssl.TLSVersion.TLSv1_3
        tls.verify_mode = ssl.CERT_REQUIRED
        tls.load_verify_locations(cafile=PKI / "client-cert.pem")
        tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
        with tempfile.TemporaryDirectory(prefix="kmipkit-certificate-prerequisite-") as dirname:
            work = Path(dirname)
            catalog_path = work / "catalog.json"
            catalog_path.write_text(json.dumps(with_cases(catalog, cases)))
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"},
                                           "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(15)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = api
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(catalog_path), "--connection", str(connection)]
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        for _ in cases:
                            raw, _ = self.accept_request(listener, process, "Create Key Pair prerequisite")
                            raw.settimeout(5)
                            with raw, tls.wrap_socket(raw, server_side=True) as peer:
                                request = only(read_message(peer), 0x420078, 1)
                                batch = only(request, 0x42000F, 1)
                                self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", 2))
                                batch_ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                                peer.sendall(structure(
                                    0x42007B,
                                    structure(0x42007A, version(minor), wire_item(0x420092, 9, bytes(8)),
                                              number(0x42000D, 2, 1)),
                                    structure(0x42000F, number(0x42005C, 5, 2), *batch_ids,
                                              number(0x42007F, 5, 1), number(0x42007E, 5, 5)),
                                ))
                        stdout, stderr = process.communicate(timeout=25)
                        self.assertTrue(report.is_file(), (stdout + stderr).decode(errors="replace"))
                        self.assertFalse(select.select([listener], [], [], 0)[0], "target was sent after failed setup")
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            evidence = json.loads(report.read_text())["cases"]
            self.assertEqual(names, {case["case_id"] for case in evidence})
            for case in evidence:
                self.assertEqual(case["status"], "not_run")
                self.assertIsNone(case.get("result_status_code"))
                self.assertIsNone(case.get("result_reason_code"))

    def test_archive_unsupported_response_preserves_cleanup(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                with self.subTest(language=language, minor=minor):
                    if not available:
                        self.skipTest(f"Installed {language} validator required")
                    self.owned_key_exchange(language, minor, "test-op-archive", 21, unsupported=True)

    def test_generic_query_checks_requested_operations_without_a_typed_decoder(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                with self.subTest(language=language, minor=minor):
                    if not available:
                        self.skipTest(f"Installed {language} validator required")
                    self.query_operations(language, minor)

    def test_query_fields_require_the_selected_structure(self):
        for language, available in installed_validators():
            routes = ("generic-ttlv", "typed-protocol", "fluent", "high-level")
            for minor in (0, 1):
                for route in routes:
                    for invalid in (False, True):
                        with self.subTest(language=language, minor=minor, route=route, invalid=invalid):
                            if not available:
                                self.skipTest(f"Installed {language} validator required")
                            self.query_operations(language, minor, fields=True, invalid=invalid, route=route)

    def test_generic_c_query_retains_failure_without_operation(self):
        validators = dict(installed_validators())
        if not validators.get("c"):
            self.skipTest("Installed C validator required")
        for minor in (0, 1):
            with self.subTest(minor=minor):
                self.query_operations("c", minor, fields=True, operation_omitted=True)

    def query_operations(self, language, minor, *, fields=False, invalid=False, route="generic-ttlv",
                         operation_omitted=False):
        with tempfile.TemporaryDirectory(prefix="kmipkit-query-operations-") as dirname:
            work = Path(dirname)
            catalog = version_catalog(f"2.{minor}")
            query = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-query")
            cases = list(query_cases(f"2.{minor}", query)) if fields else [query]
            if operation_omitted:
                cases = [case for case in cases if case["case_id"] == "test-query-rng-parameters"]
            catalog_path = work / "catalog.json"
            catalog_path.write_text(json.dumps(with_cases(catalog, cases)))
            members = {
                3: wire_item(0x42012D, 7, b"controlled-server"),
                5: wire_item(0x4200A5, 7, b"org.kmipkit.fixture"),
                8: number(0x4200DA, 5, 1),
                9: number(0x4200E0, 5, 1),
                10: number(0x4200EC, 5, 1),
                11: wire_item(0x4200EF, 6, struct.pack(">Q", 1)),
            }
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"},
                                           "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.minimum_version = ssl.TLSVersion.TLSv1_3
            tls.verify_mode = ssl.CERT_REQUIRED
            tls.load_verify_locations(cafile=PKI / "client-cert.pem")
            tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(10)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = route
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(catalog_path), "--connection", str(connection)]
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        expected_functions = {
                            int(case["requires"].get("query_function", 1)): case for case in cases
                        }
                        for _ in cases:
                            raw, _ = self.accept_request(listener, process, "Query")
                            raw.settimeout(5)
                            with raw, tls.wrap_socket(raw, server_side=True) as peer:
                                request = only(read_message(peer), 0x420078, 1)
                                batch = only(request, 0x42000F, 1)
                                self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", 0x18))
                                payload = only(batch, 0x420079, 1)
                                function = int.from_bytes(only(payload, 0x420074, 5), "big")
                                self.assertEqual(len(payload), 1)
                                self.assertIn(function, expected_functions)
                                case = expected_functions.pop(function)
                                if fields:
                                    target = int(case["requires"]["query_result_tag"])
                                    field = (wire_item(target, 7, b"wrong-type") if invalid
                                             else structure(target, members[function]))
                                    returned = field + structure(0x42015F)
                                else:
                                    returned = number(0x42005C, 5, 1)
                                batch_ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                                if operation_omitted:
                                    result_fields = [number(0x42007F, 5, 1), number(0x42007E, 5, 4)]
                                else:
                                    result_fields = [number(0x42005C, 5, 0x18), *batch_ids,
                                                     number(0x42007F, 5, 0), structure(0x42007C, returned)]
                                peer.sendall(structure(
                                    0x42007B,
                                    structure(0x42007A, version(minor), wire_item(0x420092, 9, bytes(8)),
                                              number(0x42000D, 2, 1)),
                                    structure(0x42000F, *result_fields),
                                ))
                        _, stderr = process.communicate(timeout=15)
                        self.assertEqual(process.returncode != 0, invalid or operation_omitted,
                                         stderr.decode(errors="replace"))
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            results = json.loads(report.read_text())["cases"]
            self.assertEqual({result["case_id"] for result in results}, {case["case_id"] for case in cases})
            for result in results:
                self.assertEqual(result["status"], "failed" if invalid or operation_omitted else "passed")
                if operation_omitted:
                    self.assertEqual((result["result_status_code"], result["result_reason_code"]), (1, 4))
                elif not invalid:
                    self.assertEqual(result["result_status_code"], 0)
                    if route == "high-level":
                        self.assertEqual(result["actual_api_level"], route)
            if not invalid and not operation_omitted:
                label = "query-fields" if fields else "query-operations"
                save_evidence(report, catalog_path, f"{language}-2.{minor}-{route}-{label}")

    def test_discover_versions_checks_the_explicit_offer(self):
        for language, available in installed_validators():
            if language == "rust":
                continue
            routes = ("typed-protocol", "fluent", "high-level")
            for minor in (0, 1):
                for route in routes:
                    for valid in (True, False):
                        with self.subTest(language=language, minor=minor, route=route, valid=valid):
                            if not available:
                                self.skipTest(f"Installed {language} validator required")
                            self.header_fixture_exchange(language, minor, route, valid)

    def test_attestation_nonce_checks_response_header(self):
        for language, available in installed_validators():
            for minor in (0, 1):
                for route in ("generic-ttlv", "typed-protocol", "fluent", "high-level"):
                    for variant in ("valid", "absent", "wrong-type"):
                        with self.subTest(language=language, minor=minor, route=route, variant=variant):
                            if not available:
                                self.skipTest(f"Installed {language} validator required")
                            self.header_fixture_exchange(language, minor, route, variant == "valid", nonce=variant)

    def header_fixture_exchange(self, language, minor, route, valid, *, nonce=None):
        with tempfile.TemporaryDirectory(prefix="kmipkit-typed-header-") as dirname:
            work = Path(dirname)
            catalog = version_catalog(f"2.{minor}")
            source_id = "test-op-query" if nonce else "test-op-discover-versions"
            case = next(case for case in catalog["test_cases"] if case["case_id"] == source_id)
            if nonce:
                case = nonce_case(f"2.{minor}", case)
            catalog_path = work / "catalog.json"
            catalog_path.write_text(json.dumps(with_cases(catalog, [case])))
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled",
                                                      "version": "1"}, "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.minimum_version = ssl.TLSVersion.TLSv1_3
            tls.verify_mode = ssl.CERT_REQUIRED
            tls.load_verify_locations(cafile=PKI / "client-cert.pem")
            tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(10)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = route
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(catalog_path), "--connection", str(connection)]
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        raw, _ = self.accept_request(listener, process, case["operation"])
                        raw.settimeout(5)
                        with raw, tls.wrap_socket(raw, server_side=True) as peer:
                            request = only(read_message(peer), 0x420078, 1)
                            header = only(request, 0x420077, 1)
                            self.assertEqual(only(header, 0x420069, 1), parse_items(version(minor))[0][2])
                            batch = only(request, 0x42000F, 1)
                            operation = 0x18 if nonce else 0x1E
                            self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", operation))
                            extra_header = []
                            if nonce:
                                self.assertEqual(only(header, 0x4200D3, 6), struct.pack(">Q", 1))
                                self.assertEqual(only(batch, 0x420079, 1), parse_items(number(0x420074, 5, 7)))
                                if nonce != "absent":
                                    extra_header.append(structure(
                                        0x4200C8,
                                        wire_item(0x4200C9, 8, b"nonce-id"),
                                        wire_item(0x4200CA, 8 if valid else 7, b"nonce-value"),
                                    ))
                                payload = number(0x4200C7, 5, 1)
                            else:
                                self.assertEqual(only(batch, 0x420079, 1), parse_items(version(minor)))
                                payload = version(minor if valid else 1 - minor)
                            batch_ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                            peer.sendall(structure(
                                0x42007B,
                                structure(0x42007A, version(minor), wire_item(0x420092, 9, bytes(8)),
                                          *extra_header, number(0x42000D, 2, 1)),
                                structure(0x42000F, number(0x42005C, 5, operation), *batch_ids,
                                          number(0x42007F, 5, 0), structure(0x42007C, payload)),
                            ))
                        _, stderr = process.communicate(timeout=15)
                        self.assertEqual(process.returncode == 0, valid, stderr.decode(errors="replace"))
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            observed, = json.loads(report.read_text())["cases"]
            self.assertEqual(observed["status"], "passed" if valid else "failed")
            if valid or nonce == "absent":
                self.assertEqual(observed["result_status_code"], 0)
                if route == "high-level":
                    self.assertEqual(observed["actual_api_level"], route)
            if valid:
                fixture = "attestation-nonce" if nonce else "discover-versions"
                save_evidence(report, catalog_path, f"{language}-2.{minor}-{route}-{fixture}")

    def test_authenticated_encryption_round_trip_and_invalid_tag(self):
        for language, available in installed_validators():
            routes = ("generic-ttlv", "typed-protocol", "fluent")
            if language == "rust":
                routes += ("high-level",)
            for minor in (0, 1):
                for route in routes:
                    with self.subTest(language=language, minor=minor, route=route):
                        if not available:
                            self.skipTest(f"Installed {language} validator required")
                        self.authenticated_encryption(language, minor, route)

    def authenticated_encryption(self, language, minor, route):
        identifier = wire_item(0x420094, 7, b"gcm-fixture-key")
        plaintext, ciphertext = b"A" * 16, b"C" * 16
        auth_tag = b"T" * 16
        with tempfile.TemporaryDirectory(prefix="kmipkit-gcm-") as dirname:
            work = Path(dirname)
            catalog = version_catalog(f"2.{minor}")
            by_id = {case["case_id"]: case for case in catalog["test_cases"]}
            cases = list(authenticated_encryption_cases(f"2.{minor}", by_id))
            selection = work / "catalog.json"
            selection.write_text(json.dumps(with_cases(catalog, cases)))
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"},
                                           "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.minimum_version = ssl.TLSVersion.TLSv1_3
            tls.verify_mode = ssl.CERT_REQUIRED
            tls.load_verify_locations(cafile=PKI / "client-cert.pem")
            tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(8)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = route
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(selection), "--connection", str(connection)]
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    try:
                        for case in cases:
                            decrypt = case["operation"] == "Decrypt"
                            invalid = case["requires"].get("expected_status") == "OperationFailed"
                            for operation in (1, 0x12, 0x1F, *((0x20,) if decrypt else ()), 0x13, 0x14):
                                raw, _ = self.accept_request(listener, process, f"AES-GCM operation {operation}")
                                raw.settimeout(5)
                                with raw, tls.wrap_socket(raw, server_side=True) as peer:
                                    batch = only(only(read_message(peer), 0x420078, 1), 0x42000F, 1)
                                    self.assertEqual(only(batch, 0x42005C, 5), struct.pack(">I", operation))
                                    request = only(batch, 0x420079, 1)
                                    fields = [identifier]
                                    if operation == 1:
                                        attributes = only(request, 0x420125, 1)
                                        self.assertEqual(only(attributes, 0x420028, 5), struct.pack(">I", 3))
                                        self.assertEqual(only(attributes, 0x42002A, 2), struct.pack(">I", 256))
                                        fields.insert(0, number(0x420057, 5, 2))
                                    else:
                                        self.assertEqual(only(request, 0x420094, 7), b"gcm-fixture-key")
                                    if operation in (0x1F, 0x20):
                                        params = only(request, 0x42002B, 1)
                                        self.assertEqual(only(params, 0x420011, 5), struct.pack(">I", 9))
                                        self.assertEqual(only(params, 0x4200CD, 2), struct.pack(">I", 96))
                                        self.assertEqual(only(params, 0x4200CE, 2), struct.pack(">I", 16))
                                        self.assertEqual(only(request, 0x42003D, 8), bytes(12))
                                        self.assertEqual(only(request, 0x4200FE, 8), b"kmipkit authenticated data")
                                        self.assertEqual(only(request, 0x4200C2, 8), ciphertext if operation == 0x20 else plaintext)
                                        if operation == 0x20:
                                            sent = bytes([auth_tag[0] ^ 1]) + auth_tag[1:] if invalid else auth_tag
                                            self.assertEqual(only(request, 0x4200FF, 8), sent)
                                            fields.append(wire_item(0x4200C2, 8, plaintext))
                                        else:
                                            fields.extend((wire_item(0x4200C2, 8, ciphertext), wire_item(0x4200FF, 8, auth_tag)))
                                    ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                                    result = ([number(0x42007F, 5, 1), number(0x42007E, 5, 10)]
                                              if operation == 0x20 and invalid else
                                              [number(0x42007F, 5, 0), structure(0x42007C, *fields)])
                                    peer.sendall(structure(
                                        0x42007B,
                                        structure(0x42007A, version(minor), wire_item(0x420092, 9, bytes(8)),
                                                  number(0x42000D, 2, 1)),
                                        structure(0x42000F, number(0x42005C, 5, operation), *ids, *result),
                                    ))
                        _, stderr = process.communicate(timeout=12)
                        self.assertEqual(process.returncode, 0, stderr.decode(errors="replace"))
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            results = json.loads(report.read_text())["cases"]
            self.assertEqual(len(results), len(cases))
            for result, case in zip(results, cases):
                self.assertEqual(result["case_id"], case["case_id"])
                self.assertEqual(result["status"], "passed")
                invalid = case["requires"].get("expected_status") == "OperationFailed"
                self.assertEqual(result["result_status_code"], 1 if invalid else 0)
                if invalid:
                    self.assertEqual(result["result_reason_code"], 10)
                if route == "high-level":
                    self.assertEqual(result["actual_api_level"], route)
            save_evidence(report, selection, f"{language}-2.{minor}-{route}-authenticated-encryption")

    def test_key_block_get_checks_wrapping_and_transparent_material(self):
        for language, available in installed_validators():
            routes = ("generic-ttlv", "typed-protocol")
            if language != "c":
                routes += ("fluent",)
            if language == "rust":
                routes += ("high-level",)
            for minor in (0, 1):
                for route in routes:
                    for invalid in (False, True):
                        with self.subTest(language=language, minor=minor, route=route, invalid=invalid):
                            if not available:
                                self.skipTest(f"Installed {language} validator required")
                            self.key_block_get(language, minor, route, invalid)

    def key_block_get(self, language, minor, route, invalid):
        target_id = b"target-key-block-fixture"
        wrapping_id = b"wrapping-key-block-fixture"
        private_id = b"private-key-block-fixture"
        public_id = b"public-key-block-fixture"
        with tempfile.TemporaryDirectory(prefix="kmipkit-key-block-") as dirname:
            work = Path(dirname)
            catalog = version_catalog(f"2.{minor}")
            base = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-get")
            cases = [*key_block_cases(f"2.{minor}", base), *asymmetric_key_cases(f"2.{minor}", base)]
            selection = work / "catalog.json"
            selection.write_text(json.dumps(with_cases(catalog, cases)))
            profile = work / "profile.json"
            profile.write_text(json.dumps({"server": {"id": "controlled-fixture", "product": "controlled", "version": "1"},
                                           "operations": {}}))
            report = work / "results.json"
            command, env = self.command(language, profile, report)
            tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            tls.minimum_version = ssl.TLSVersion.TLSv1_3
            tls.verify_mode = ssl.CERT_REQUIRED
            tls.load_verify_locations(cafile=PKI / "client-cert.pem")
            tls.load_cert_chain(PKI / "server-cert.pem", PKI / "server-key.pem")
            with socket.create_server(("127.0.0.1", 0)) as listener:
                listener.settimeout(7)
                connection = work / "connection.json"
                config = connection_config(minor, listener.getsockname()[1], incoming=False)
                config["api_level"] = route
                connection.write_text(json.dumps(config))
                command += ["--catalog", str(selection), "--connection", str(connection)]
                with subprocess.Popen(command, cwd=work, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
                    def receive(stage, expect, build_payload):
                        raw, _ = self.accept_request(listener, process, stage)
                        raw.settimeout(5)
                        with raw, tls.wrap_socket(raw, server_side=True) as peer:
                            batch = only(only(read_message(peer), 0x420078, 1), 0x42000F, 1)
                            operation = int.from_bytes(only(batch, 0x42005C, 5), "big")
                            self.assertIn(operation, expect)
                            request = only(batch, 0x420079, 1)
                            payload = build_payload(operation, request)
                            ids = [wire_item(tag, kind, value) for tag, kind, value in batch if tag == 0x420093]
                            peer.sendall(structure(
                                0x42007B,
                                structure(0x42007A, version(minor), wire_item(0x420092, 9, bytes(8)),
                                          number(0x42000D, 2, 1)),
                                structure(0x42000F, number(0x42005C, 5, operation), *ids,
                                          number(0x42007F, 5, 0), structure(0x42007C, *payload)),
                            ))

                    def create_payload(request, key, usage_mask):
                        self.assertEqual(only(request, 0x420057, 5), struct.pack(">I", 2))
                        attributes = only(request, 0x420125, 1)
                        self.assertEqual(only(attributes, 0x420028, 5), struct.pack(">I", 3))
                        self.assertEqual(only(attributes, 0x42002A, 2), struct.pack(">I", 256))
                        self.assertEqual(only(attributes, 0x42002C, 2), struct.pack(">I", usage_mask))
                        self.assertEqual(only(attributes, 0x420122, 6), struct.pack(">Q", 1))
                        return number(0x420057, 5, 2), wire_item(0x420094, 7, key)

                    def activate_payload(request):
                        self.assertEqual(only(request, 0x420094, 7), wrapping_id)
                        return (wire_item(0x420094, 7, wrapping_id),)

                    def get_payload(request, wrapped):
                        self.assertEqual(only(request, 0x420094, 7), target_id)
                        encoding = number(0x4200A3, 5, 1)
                        wrapping_method = number(0x42009E, 5, 1)
                        wrapping = []
                        if wrapped:
                            self.assertFalse(any(tag == 0x420042 for tag, _, _ in request))
                            spec = only(request, 0x420047, 1)
                            self.assertEqual(only(spec, 0x42009E, 5), struct.pack(">I", 1))
                            self.assertEqual(only(spec, 0x4200A3, 5), struct.pack(">I", 1))
                            key_info = only(spec, 0x420036, 1)
                            self.assertEqual(only(key_info, 0x420094, 7), wrapping_id)
                            parameters = only(key_info, 0x42002B, 1)
                            self.assertEqual(only(parameters, 0x420011, 5), struct.pack(">I", 12))
                            value = wire_item(0x420045, 8, bytes(40))
                            wrapper = b"foreign-key-block-fixture" if invalid else wrapping_id
                            wrapping = [structure(0x420046, wrapping_method,
                                                  structure(0x420036, wire_item(0x420094, 7, wrapper)), encoding)]
                        else:
                            self.assertEqual(only(request, 0x420042, 5), struct.pack(">I", 7))
                            self.assertFalse(any(tag == 0x420047 for tag, _, _ in request))
                            value = structure(0x420045, structure(0x420043, wire_item(0x42003F, 8, bytes(31 if invalid else 32))))
                        block = structure(0x420040, number(0x420042, 5, 7), value, number(0x420028, 5, 3),
                                          number(0x42002A, 2, 256), *wrapping)
                        return number(0x420057, 5, 2), wire_item(0x420094, 7, target_id), structure(0x42008F, block)

                    def pair_payload(request, format_code):
                        ec = format_code in {20, 21}
                        common = only(request, 0x420126, 1)
                        self.assertEqual(only(common, 0x420028, 5), struct.pack(">I", 26 if ec else 4))
                        self.assertEqual(only(common, 0x42002A, 2), struct.pack(">I", 256 if ec else 2048))
                        if ec:
                            domain = only(common, 0x420029, 1)
                            self.assertEqual(only(domain, 0x420075, 5), struct.pack(">I", 7))
                        for tag, usage in ((0x420127, 1), (0x420128, 2)):
                            attributes = only(request, tag, 1)
                            self.assertEqual(only(attributes, 0x42002C, 2), struct.pack(">I", usage))
                            self.assertEqual(only(attributes, 0x420122, 6), struct.pack(">Q", 1))
                        return wire_item(0x420066, 7, private_id), wire_item(0x42006F, 7, public_id)

                    def asymmetric_payload(request, format_code):
                        ec = format_code in {20, 21}
                        public = format_code in {11, 21}
                        key_id = public_id if public else private_id
                        self.assertEqual(only(request, 0x420094, 7), key_id)
                        self.assertEqual(only(request, 0x420042, 5), struct.pack(">I", format_code))
                        if ec:
                            component = (wire_item(0x420072, 8, b"\x04" + bytes(64)) if public
                                         else wire_item(0x42002E, 4, bytes(31) + b"\x01"))
                            fields = [number(0x420075, 5, 10 if invalid else 7), component]
                        else:
                            exponent = wire_item(0x42006C if public else 0x420063, 4, bytes(7) + b"\x03")
                            fields = [wire_item(0x420052, 4, b"\x01" + bytes(247 if invalid else 255)), exponent]
                            if not public:
                                fields.append(wire_item(0x42006C, 4, bytes(7) + b"\x03"))
                        value = structure(0x420045, structure(0x420043, *fields))
                        block = structure(0x420040, number(0x420042, 5, format_code), value,
                                          number(0x420028, 5, 14 if ec else 4),
                                          number(0x42002A, 2, 256 if ec else 2048))
                        return (number(0x420057, 5, 3 if public else 4),
                                wire_item(0x420094, 7, key_id),
                                structure(0x42006D if public else 0x420064, block))

                    def clean_payload(operation, request, outstanding, revoke_pending):
                        key = only(request, 0x420094, 7)
                        self.assertIn(key, outstanding)
                        if operation == 0x14:
                            self.assertNotIn(key, revoke_pending)
                            outstanding.remove(key)
                        else:
                            revoke_pending.discard(key)
                        return (wire_item(0x420094, 7, key),)

                    try:
                        for case in cases:
                            requires = case["requires"]
                            if "key_material_format" in requires:
                                format_code = int(requires["key_material_format"])
                                receive("Create Key Pair", {2}, lambda _, request: pair_payload(request, format_code))
                                receive("Get", {0x0A}, lambda _, request: asymmetric_payload(request, format_code))
                                outstanding, revoke_pending = {private_id, public_id}, set()
                            else:
                                wrapped = requires["key_block_shape"] == "wrapped"
                                receive("Create data key", {1}, lambda _, request: create_payload(request, target_id, 12))
                                if wrapped:
                                    receive("Create wrapping key", {1}, lambda _, request: create_payload(request, wrapping_id, 48))
                                    receive("Activate wrapping key", {0x12}, lambda _, request: activate_payload(request))
                                receive("Get", {0x0A}, lambda _, request: get_payload(request, wrapped))
                                outstanding = {target_id, wrapping_id} if wrapped else {target_id}
                                revoke_pending = {wrapping_id} if wrapped else set()
                            # Older teardown helpers also revoke the pre-active fixture before destroying it.
                            for _ in range(2 * len(outstanding)):
                                if not outstanding:
                                    break
                                receive("Clean up", {0x13, 0x14},
                                        lambda operation, request: clean_payload(operation, request, outstanding, revoke_pending))
                            self.assertFalse(outstanding)
                        _, stderr = process.communicate(timeout=12)
                        self.assertEqual(process.returncode != 0, invalid, stderr.decode(errors="replace"))
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            process.communicate(timeout=5)
            results = json.loads(report.read_text())["cases"]
            self.assertEqual({case["case_id"] for case in cases}, {result["case_id"] for result in results})
            for result in results:
                self.assertEqual(result["status"], "failed" if invalid else "passed")
                if not invalid and route == "high-level":
                    self.assertEqual(result["actual_api_level"], route)
            if not invalid:
                save_evidence(report, selection, f"{language}-2.{minor}-{route}-key-block")

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

    def exchange(self, language, minor, invalid, *, api="generic-ttlv"):
        protocol_version = f"2.{minor}"
        catalog = version_catalog(protocol_version)
        cases = [case for case in catalog["test_cases"] if direction(case) == "server_to_client"
                 and (not invalid or case["operation"] == "Put")
                 and (api == "generic-ttlv" or case["operation"] in {"Notify", "Put"})]
        self.assertEqual(len(cases), 1 if invalid else 5 if api == "generic-ttlv" else 2)
        with tempfile.TemporaryDirectory(prefix="kmipkit-incoming-") as dirname:
            work = Path(dirname)
            catalog_path = work / "catalog.json"
            catalog_path.write_text(json.dumps(with_cases(catalog, cases)))
            connection = work / "connection.json"
            config = connection_config(minor, 1, incoming=True)
            config["api_level"] = api
            connection.write_text(json.dumps(config))
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
                        self.trigger(tls, peer, case["operation"], minor, invalid, api)
                    _, stderr = process.communicate(timeout=15)
                    self.assertEqual(process.returncode != 0, invalid, stderr.decode(errors="replace"))
                finally:
                    if process.poll() is None:
                        process.terminate()
                        process.communicate(timeout=5)
            results = json.loads(report.read_text())
            self.assertEqual(results["execution"]["api_level"], api)
            self.assertEqual([case["case_id"] for case in results["cases"]], [case["case_id"] for case in cases])
            for evidence in results["cases"]:
                self.assertEqual(evidence["status"], "failed" if invalid else "passed")
                status = None if invalid and api == "typed-protocol" else int(invalid)
                reason = 7 if invalid and api == "generic-ttlv" else None
                self.assertEqual(evidence.get("result_status_code"), status)
                self.assertEqual(evidence.get("result_reason_code"), reason)
            if not invalid:
                save_evidence(report, catalog_path, f"{language}-2.{minor}-{api}-incoming")

    def wait_for_listener(self, process, ready):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and process.poll() is None:
            if ready.exists() and (address := ready.read_text()):
                host, port = address.rsplit(":", 1)
                return host, int(port)
            time.sleep(.01)
        _, stderr = process.communicate(timeout=5)
        self.fail(f"Listener did not become ready: {stderr.decode(errors='replace')}")

    def trigger(self, context, peer, name, minor, invalid, api):
        operation = OPERATIONS[name]
        payload = offered_payload(name, minor, invalid)
        header_fields = [version(minor)]
        batch_fields = [number(0x42005C, 5, operation)]
        if api == "generic-ttlv":
            header_fields.append(wire_item(0x420105, 7, b"test-correlation"))
            batch_fields.append(wire_item(0x420093, 8, b"test-item"))
        header_fields.append(number(0x42000D, 2, 1))
        batch_fields.append(structure(0x420079, *payload))
        packet = structure(0x420078, structure(0x420077, *header_fields), structure(0x42000F, *batch_fields))
        with socket.create_connection(peer, timeout=5) as tcp, context.wrap_socket(tcp, server_hostname="localhost") as channel:
            channel.sendall(packet)
            if invalid and api == "typed-protocol":
                self.assertEqual(channel.recv(1), b"")
                return
            response = only(read_message(channel), 0x42007B, 1)
        header = only(response, 0x42007A, 1)
        batch = only(response, 0x42000F, 1)
        self.assertEqual(only(header, 0x420069, 1), only(parse_items(version(minor)), 0x420069, 1))
        self.assertEqual(len(only(header, 0x420092, 9)), 8)
        self.assertEqual(only(header, 0x42000D, 2), struct.pack(">I", 1))
        if api == "generic-ttlv":
            self.assertEqual(only(header, 0x420105, 7), b"test-correlation")
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
        elif api == "generic-ttlv" or any(tag == 0x42007C for tag, _, _ in batch):
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
