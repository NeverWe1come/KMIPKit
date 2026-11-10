#!/usr/bin/env python3
"""Exercise the installed C client against a local KMIP TTLV/TLS peer."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import socket
import ssl
import struct
import subprocess
import sys
import tempfile
import threading
import time


def tlv(tag: int, item_type: int, value: bytes) -> bytes:
    header = tag.to_bytes(3, "big") + bytes([item_type]) + len(value).to_bytes(4, "big")
    return header + value + bytes((-len(value)) % 8)


def integer(tag: int, value: int, item_type: int = 2) -> bytes:
    return tlv(tag, item_type, struct.pack(">i", value))


def enumeration(tag: int, value: int) -> bytes:
    return tlv(tag, 5, struct.pack(">I", value))


def response(operations: tuple[int, ...], protocol_minor: int) -> bytes:
    version = tlv(0x420069, 1,
                  integer(0x42006A, 2) + integer(0x42006B, protocol_minor))
    header = tlv(
        0x42007A,
        1,
        version + tlv(0x420092, 9, struct.pack(">q", int(time.time())))
        + integer(0x42000D, len(operations)),
    )
    batch_items = []
    for operation in operations:
        payload_value = b""
        if operation == 30:
            payload_value = tlv(
            0x420069, 1,
            integer(0x42006A, 2) + integer(0x42006B, protocol_minor),
            )
        elif operation == 24:
            query_operations = (24, 30, 44) if protocol_minor == 0 else (24, 30, 44, 59)
            payload_value = b"".join(integer(0x42005C, value, 5) for value in query_operations)
            payload_value += tlv(0x42015F, 1, b"")
        elif operation == 8:
            payload_value = (
                integer(0x4200D5, 3)
                + tlv(0x420094, 7, b"fixture-locate-1")
                + enumeration(0x420094, 0xF1234567)
                + integer(0x420094, -12345)
                if protocol_minor == 1
                else b""
            )
        elif operation == 45:
            payload_value = tlv(
                0x420149,
                1,
                enumeration(0x42014A, 1) + tlv(0x42014B, 8, b"controlled-login-ticket"),
            )
        elif operation == 1:
            payload_value = enumeration(0x420057, 2) + tlv(0x420094, 7, b"fixture-key-1")
        elif operation == 11:
            attributes = (
                tlv(0x420008, 1, tlv(0x42000A, 7, b"Cryptographic Algorithm")
                    + enumeration(0x42000B, 3))
                + tlv(0x420008, 1, tlv(0x42000A, 7, b"Cryptographic Length")
                      + integer(0x42000B, 128))
            )
            payload_value = tlv(0x420094, 7, b"fixture-key-1") + tlv(0x420125, 1, attributes)
        elif operation == 20:
            payload_value = tlv(0x420094, 7, b"fixture-key-1")
        batch_items.append(tlv(
            0x42000F,
            1,
            integer(0x42005C, operation, 5)
            + integer(0x42007F, 0, 5)
            + tlv(0x42007C, 1, payload_value),
        ))
    return tlv(0x42007B, 1, header + b"".join(batch_items))


def read_exact(stream: ssl.SSLSocket, length: int) -> bytes:
    chunks = bytearray()
    while len(chunks) < length:
        part = stream.recv(length - len(chunks))
        if not part:
            raise EOFError("peer closed before the complete TTLV message arrived")
        chunks.extend(part)
    return bytes(chunks)


def read_message(stream: ssl.SSLSocket) -> bytes | None:
    try:
        header = read_exact(stream, 8)
    except EOFError:
        return None
    body_length = int.from_bytes(header[4:8], "big")
    body = read_exact(stream, ((body_length + 7) // 8) * 8)
    return header + body


def certificate_files(openssl: str, directory: Path) -> tuple[Path, Path, Path, Path, Path]:
    ca_key = directory / "ca-key.pem"
    ca_cert = directory / "ca-cert.pem"
    subprocess.run(
        [openssl, "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-sha256", "-days", "2",
         "-subj", "/CN=KMIPKit interop test CA", "-keyout", str(ca_key), "-out", str(ca_cert)],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    server_ext = directory / "server-ext.cnf"
    client_ext = directory / "client-ext.cnf"
    server_ext.write_text(
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n"
        "extendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\n",
        encoding="ascii",
    )
    client_ext.write_text(
        "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\n"
        "extendedKeyUsage=clientAuth\n",
        encoding="ascii",
    )
    for name, ext in (("server", server_ext), ("client", client_ext)):
        key = directory / f"{name}-key.pem"
        csr = directory / f"{name}.csr"
        cert = directory / f"{name}-cert.pem"
        subprocess.run(
            [openssl, "req", "-new", "-newkey", "rsa:2048", "-nodes", "-sha256",
             "-subj", f"/CN=KMIPKit {name}", "-keyout", str(key), "-out", str(csr)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        subprocess.run(
            [openssl, "x509", "-req", "-in", str(csr), "-CA", str(ca_cert), "-CAkey", str(ca_key),
             "-CAcreateserial", "-out", str(cert), "-days", "2", "-sha256", "-extfile", str(ext)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    return (ca_cert, directory / "server-cert.pem", directory / "server-key.pem",
            directory / "client-cert.pem", directory / "client-key.pem")


def main() -> int:
    binary, openssl, repository = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
    protocol_version = sys.argv[4] if len(sys.argv) > 4 else "2.1"
    if protocol_version not in ("2.0", "2.1"):
        raise AssertionError(f"unsupported fixture protocol version {protocol_version!r}")
    protocol_minor = int(protocol_version.split(".")[1])
    expected_operations = (1, 8, 11, 20, 24, 44) if protocol_version == "2.0" else (1, 8, 11, 20, 24, 30, 44, 45, 59)
    expected_cases = (
        ("test-op-locate", "test-op-query", "test-op-log")
        if protocol_version == "2.0"
        else ("test-op-discover-versions", "test-op-ping", "test-op-query", "test-op-log",
              "test-op-locate", "test-op-login", "test-op-get-attributes")
    )
    if protocol_version == "2.0":
        expected_cases += ("test-op-get-attributes",)
    shared = repository / "interop/shared"
    catalog = shared / "catalog/kmipkit-1.0-test-catalog.json"
    with tempfile.TemporaryDirectory(prefix="kmipkit-c-live-") as raw_directory:
        directory = Path(raw_directory)
        ca_cert, server_cert, server_key, client_cert, client_key = certificate_files(openssl, directory)
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(4)
        server.settimeout(10)
        port = server.getsockname()[1]
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.minimum_version = ssl.TLSVersion.TLSv1_3
        context.load_cert_chain(str(server_cert), str(server_key))
        context.load_verify_locations(str(ca_cert))
        context.verify_mode = ssl.CERT_REQUIRED
        server_errors: list[BaseException] = []
        handled = 0
        handled_operations: list[int] = []
        handled_batches: list[tuple[int, ...]] = []

        def serve() -> None:
            nonlocal handled
            try:
                while handled < len(expected_operations):
                    accepted, _ = server.accept()
                    with context.wrap_socket(accepted, server_side=True) as stream:
                        stream.settimeout(10)
                        while handled < len(expected_operations):
                            request = read_message(stream)
                            if request is None:
                                break
                            operation_marker = bytes.fromhex("42005c0500000004")
                            offsets = []
                            offset = request.find(operation_marker)
                            while offset >= 0:
                                offsets.append(offset)
                                offset = request.find(operation_marker, offset + len(operation_marker) + 4)
                            if not offsets:
                                raise AssertionError("request has no KMIP Operation field")
                            operations = tuple(int.from_bytes(
                                request[offset + len(operation_marker):offset + len(operation_marker) + 4], "big"
                            ) for offset in offsets)
                            if any(operation not in expected_operations for operation in operations):
                                raise AssertionError(f"unexpected operation batch {operations!r} for {protocol_version}")
                            if 8 in operations:
                                if bytes.fromhex("4201250100000000") not in request:
                                    raise AssertionError("Locate request omitted its required Attributes structure")
                                if integer(0x42004F, 3) not in request:
                                    raise AssertionError("Locate request omitted its bounded Maximum Items value")
                            if integer(0x42006A, 2) not in request or \
                                    integer(0x42006B, protocol_minor) not in request:
                                raise AssertionError(
                                    "request header did not preserve the configured KMIP version"
                                )
                            if tlv(0x420099, 7, b"interop-user") not in request or \
                                    tlv(0x4200A1, 7, b"interop-password") not in request:
                                raise AssertionError("request is missing configured KMIP credentials")
                            stream.sendall(response(operations, protocol_minor))
                            handled += len(operations)
                            handled_operations.extend(operations)
                            handled_batches.append(operations)
            except BaseException as error:  # surfaced in the main test thread below
                server_errors.append(error)
            finally:
                server.close()

        thread = threading.Thread(target=serve, daemon=True)
        thread.start()
        connection = {
            "server_id": "controlled-c-peer",
            "server_vendor": "",
            "server_product": "KMIPKit controlled TLS test peer",
            "server_version": "fixture-1",
            "server_configuration_fingerprint": hashlib.sha256(b"controlled-c-peer-v1").hexdigest(),
            "protocol_version": protocol_version,
            "transport": "raw_tls",
            "encoding": "TTLV",
            "api_level": "typed-protocol",
            "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": port}},
            "tls": {
                "ca_cert_path": str(ca_cert),
                "client_cert_path": str(client_cert),
                "client_key_path": str(client_key),
                "server_name_override": "localhost",
            },
            "authentication": {
                "username_password": {
                    "username_env": "KMIPKIT_INTEROP_TEST_USERNAME",
                    "password_env": "KMIPKIT_INTEROP_TEST_PASSWORD",
                }
            },
            "environment": {"kind": "dedicated_test", "dedicated_test_environment_confirmed": True},
            "execution_policy": {"probe_unsupported": True, "allow_mutations": True},
        }
        connection_path = directory / "connection.json"
        connection_path.write_text(json.dumps(connection), encoding="utf-8")
        result_path = directory / "validation-results.json"
        markdown_path = directory / "validation-results.md"
        html_path = directory / "validation-results.html"
        run = subprocess.run(
            [str(binary), "run", "--connection", str(connection_path), "--catalog", str(catalog),
             "--allow-mutations",
             "--output", str(result_path), "--output-markdown", str(markdown_path),
             "--output-html", str(html_path)],
            cwd=repository,
            capture_output=True,
            text=True,
            env={
                **os.environ,
                "KMIPKIT_INTEROP_TEST_USERNAME": "interop-user",
                "KMIPKIT_INTEROP_TEST_PASSWORD": "interop-password",
            },
            timeout=30,
            check=False,
        )
        thread.join(timeout=10)
        if server_errors:
            raise AssertionError("controlled TLS peer failed") from server_errors[0]
        if (handled != len(expected_operations) or sorted(handled_operations) != sorted(expected_operations)
                or (1, 11, 20) not in handled_batches):
            raise AssertionError(
                f"controlled peer observed batches {handled_batches!r}, "
                f"expected {expected_operations!r}"
            )
        result = json.loads(result_path.read_text(encoding="utf-8"))
        try:
            import jsonschema
        except ImportError:
            jsonschema = None
        if jsonschema is not None:
            schema = json.loads((shared / "schemas/validation-results.schema.json").read_text(encoding="utf-8"))
            jsonschema.validate(result, schema)
        if result["schema_version"] != "2.0" or result["protocol_version"] != protocol_version:
            raise AssertionError("C result metadata does not match the shared evidence schema")
        if result["server"]["id"] != "controlled-c-peer":
            raise AssertionError("C result identifies the wrong test server")
        statuses = {case["case_id"]: case["status"] for case in result["cases"]}
        if any(statuses.get(case_id) != "passed" for case_id in expected_cases):
            details = {case["case_id"]: case.get("diagnostic", "") for case in result["cases"]
                       if case["case_id"] in expected_cases and case["status"] != "passed"}
            raise AssertionError(f"controlled operation response checks failed: {details!r}")
        if protocol_version == "2.0":
            discovery = next(case for case in result["cases"]
                             if case["case_id"] == "test-op-discover-versions")
            if discovery["status"] != "not_run" or "cannot construct a KMIP 2.0 offer" not in discovery.get("diagnostic", ""):
                raise AssertionError(
                    "KMIP 2.0 Discover Versions must report the documented C API offer limitation"
                )
            login = next(case for case in result["cases"] if case["case_id"] == "test-op-login")
            if login["status"] != "not_run" or "Login builder only supports KMIP 2.1" not in login.get("diagnostic", ""):
                raise AssertionError("KMIP 2.0 Login must report the documented public C API limitation")
        if run.returncode != 0:
            raise AssertionError(
                f"C validator exited {run.returncode}; stderr={run.stderr[-400:]!r}; "
                f"summary={run.stdout[:1800]!r}"
            )
        correctness = {case["case_id"]: case["library_correctness"] for case in result["cases"]}
        if any(correctness.get(case_id) != "passed" for case_id in expected_cases):
            raise AssertionError("successful exchanges must report the client library as validated")
        if result["summary"]["policy_restricted"] != 0 or result["summary"]["not_run"] == 0:
            raise AssertionError("opted-in controlled run must preserve uncovered-case classification")
        expected_summary = {
            "total": 212,
            "passed": 74,
            "failed": 0,
            "unsupported": 0,
            "policy_restricted": 0,
            "not_run": 138,
        }
        if protocol_version == "2.1" and result["summary"] != expected_summary:
            raise AssertionError(
                f"controlled run must execute only the mapped recipes and classify all other cases: "
                f"{result['summary']!r}"
            )
        expected_kmip20_summary = {
            "total": 203,
            "passed": 69,
            "failed": 0,
            "unsupported": 0,
            "policy_restricted": 0,
            "not_run": 134,
        }
        if protocol_version == "2.0" and result["summary"] != expected_kmip20_summary:
            raise AssertionError(
                f"controlled KMIP 2.0 run did not preserve complete case classification: "
                f"{result['summary']!r}"
            )
        if not markdown_path.is_file() or not html_path.is_file():
            raise AssertionError("requested human-readable reports were not written")
        secret_values = (b"interop-user", b"interop-password", b"controlled-login-ticket")
        for report_path in (result_path, markdown_path, html_path):
            report_bytes = report_path.read_bytes()
            if any(secret in report_bytes for secret in secret_values):
                raise AssertionError("a report contains configured authentication material")
        if any(secret.decode("ascii") in run.stdout or secret.decode("ascii") in run.stderr
               for secret in secret_values):
            raise AssertionError("console output contains configured authentication material")
        missing_auth_environment = os.environ.copy()
        missing_auth_environment.pop("KMIPKIT_INTEROP_TEST_USERNAME", None)
        missing_auth_environment.pop("KMIPKIT_INTEROP_TEST_PASSWORD", None)
        missing_auth = subprocess.run(
            [str(binary), "run", "--connection", str(connection_path), "--catalog", str(catalog),
             "--output", str(directory / "missing-auth-results.json")],
            cwd=repository,
            capture_output=True,
            text=True,
            env=missing_auth_environment,
            timeout=10,
            check=False,
        )
        if missing_auth.returncode != 2 or "environment variables are unavailable" not in missing_auth.stderr:
            raise AssertionError("missing authentication variables must stop before any network exchange")
        if any(secret.decode("ascii") in missing_auth.stdout or secret.decode("ascii") in missing_auth.stderr
               for secret in secret_values):
            raise AssertionError("credential diagnostics exposed authentication material")
        preflight_without_auth = subprocess.run(
            [str(binary), "preflight", "--connection", str(connection_path)],
            cwd=repository,
            capture_output=True,
            text=True,
            env=missing_auth_environment,
            timeout=10,
            check=False,
        )
        if preflight_without_auth.returncode != 2 or \
                "environment variables are unavailable" not in preflight_without_auth.stderr:
            raise AssertionError("preflight must detect missing authentication without opening a socket")
        for case in result["cases"]:
            if case["status"] in ("not_run", "policy_restricted") and not case.get("diagnostic"):
                raise AssertionError("unexecuted cases must include a sanitized reason")
        if run.stderr:
            print(run.stderr, file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
