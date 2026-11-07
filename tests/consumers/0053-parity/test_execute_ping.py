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

from local_ping_responder import _is_ping_request, _item, _structure


CONSUMER_ROOT = Path(__file__).resolve().parent
EXPECTED_LINES = (
    "rust: Ping result and raw TTLV verified",
    "c: Ping result and raw TTLV verified",
    "java: Ping result and raw TTLV verified",
    "python: Ping result and raw TTLV verified",
)


class ExecutePingConsumerTests(unittest.TestCase):
    def test_fixture_accepts_exactly_one_ping_operation(self) -> None:
        operation = _item(0x42005C, 5, struct.pack(">I", 0x3B))
        other_operation = _item(0x42005C, 5, struct.pack(">I", 0x18))
        ping_item = _structure(0x42000F, operation)
        query_item = _structure(0x42000F, other_operation)
        self.assertTrue(_is_ping_request(_structure(0x420078, ping_item)))
        self.assertFalse(_is_ping_request(_structure(0x420078, ping_item, query_item)))
        multiple_operations = _structure(0x42000F, operation, other_operation)
        self.assertFalse(_is_ping_request(_structure(0x420078, multiple_operations)))

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

    def test_all_external_consumers_execute_ping_and_inspect_raw_ttlv(self) -> None:
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
