"""Executable acceptance check for the four-language Ping consumers."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import time
import unittest

from local_ping_responder import (
    _children,
    _item,
    _requested_identifier,
    _request_operation,
    _response,
    _structure,
    _validate_operation,
)


CONSUMER_ROOT = Path(__file__).resolve().parent
EXPECTED_LINES = (
    "rust: Ping result and raw TTLV verified",
    "c: Ping result and raw TTLV verified",
    "python: Ping result and raw TTLV verified",
    "rust: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS",
    "c: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS",
    "java: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS",
    "python: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS",
)


class ExecutePingConsumerTests(unittest.TestCase):
    def test_fixture_echoes_destroyed_key_identifier(self) -> None:
        request_payload = _structure(
            0x420079,
            _item(0x420094, 7, b"fluent-private-key"),
        )
        fields = [(0x420079, 1, request_payload[8:])]
        identifier = _requested_identifier(20, fields)
        response = _response(20, unique_identifier=identifier)
        response_item = next(
            value for tag, kind, value in _children(response[8:]) if tag == 0x42000F and kind == 1
        )
        response_fields = _children(response_item)
        response_payload = next(value for tag, kind, value in response_fields if tag == 0x42007C and kind == 1)
        echoed = [value for tag, kind, value in _children(response_payload) if tag == 0x420094 and kind == 7]
        self.assertEqual(echoed, [b"fluent-private-key"])

    def test_fixture_returns_distinct_identifiers_for_create_key_pair(self) -> None:
        root = _children(_response(0x02)[8:])
        response_items = [value for tag, kind, value in root if tag == 0x42000F and kind == 1]
        self.assertEqual(len(response_items), 1)
        response_fields = _children(response_items[0])
        response_payloads = [
            value for tag, kind, value in response_fields if tag == 0x42007C and kind == 1
        ]
        self.assertEqual(len(response_payloads), 1)
        identifiers = {
            tag: value
            for tag, kind, value in _children(response_payloads[0])
            if kind == 7 and tag in (0x420066, 0x42006F)
        }
        self.assertEqual(
            identifiers,
            {0x420066: b"fluent-private-key", 0x42006F: b"fluent-public-key"},
        )

    def test_fixture_rejects_key_pair_with_unexpected_algorithm(self) -> None:
        common_attributes = _structure(
            0x420126,
            _item(0x420028, 5, struct.pack(">I", 4)),
            _item(0x42002A, 2, struct.pack(">i", 256)),
        )
        payload = _structure(0x420079, common_attributes)
        with self.assertRaisesRegex(RuntimeError, "Create Key Pair did not select ECDSA"):
            _validate_operation(2, [(0x420079, 1, payload[8:])], [])

    def test_fixture_parses_ping_from_a_request_message(self) -> None:
        operation = _item(0x42005C, 5, struct.pack(">I", 0x3B))
        ping_item = _structure(
            0x42000F,
            operation,
            _structure(0x420079),
        )
        request_header = _structure(0x420077, _item(0x42000D, 2, struct.pack(">i", 1)))
        request_message = _structure(0x420078, request_header, ping_item)
        self.assertEqual(_request_operation(request_message)[0], 0x3B)

    def test_fixture_records_its_process_id_for_runner_cleanup(self) -> None:
        repository_root = CONSUMER_ROOT.parents[2]
        fixture_root = repository_root / "bindings/python/tests/fixtures/login-mtls"
        with tempfile.TemporaryDirectory(prefix="kmipkit-0053-fixture-") as temporary:
            ready_file = Path(temporary) / "ready.txt"
            pid_file = Path(temporary) / "responder.pid"
            process = subprocess.Popen(
                [
                    sys.executable,
                    str(CONSUMER_ROOT / "local_ping_responder.py"),
                    "--ready-file",
                    str(ready_file),
                    "--pid-file",
                    str(pid_file),
                    "--fixture-dir",
                    str(fixture_root),
                    "--connections",
                    "1",
                ],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
            )
            try:
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    if process.poll() is not None:
                        error = process.stderr.read() if process.stderr is not None else ""
                        self.fail(f"fixture exited before readiness: {error}")
                    if ready_file.exists() and pid_file.exists():
                        break
                    time.sleep(0.01)
                self.assertTrue(ready_file.exists(), "fixture did not publish its ready port")
                responder_pid = int(pid_file.read_text(encoding="ascii"))
                self.assertGreater(responder_pid, 0)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
                if process.stderr is not None:
                    process.stderr.close()

    def test_all_external_consumers_execute_fluent_journeys_and_inspect_raw_ttlv(self) -> None:
        if os.name == "nt":
            shell = shutil.which("pwsh") or shutil.which("powershell")
            self.assertIsNotNone(shell, "PowerShell is required for the Windows consumer runner")
            command = [
                shell or "pwsh",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                str(CONSUMER_ROOT / "verify.ps1"),
            ]
        else:
            command = ["sh", str(CONSUMER_ROOT / "verify.sh")]

        completed = subprocess.run(
            command,
            cwd=CONSUMER_ROOT.parents[2],
            capture_output=True,
            text=True,
            timeout=600,
            check=False,
        )
        output = completed.stdout + completed.stderr
        self.assertEqual(completed.returncode, 0, output)
        for expected in EXPECTED_LINES:
            with self.subTest(language=expected.split(":", 1)[0]):
                self.assertIn(expected, output)


if __name__ == "__main__":
    unittest.main()
