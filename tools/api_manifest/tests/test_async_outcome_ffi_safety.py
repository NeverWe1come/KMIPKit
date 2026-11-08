"""Source-level checks for the Rust safety contract of async outcome exports."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
CLIENT_ABI_SOURCE = REPOSITORY_ROOT / "crates/kmipkit-ffi/src/extension_registry/client_abi.rs"
ASYNC_RESPONSE_EXPORTS = (
    "kmipkit_client_async_operation_outcome_cancel_response_v1",
    "kmipkit_client_async_operation_outcome_poll_response_v1",
    "kmipkit_client_async_operation_outcome_process_response_v1",
    "kmipkit_client_async_operation_outcome_query_async_requests_response_v1",
)


class AsyncOutcomeFfiSafetyTests(unittest.TestCase):
    def test_pointer_dereferencing_async_response_exports_are_unsafe(self) -> None:
        source = CLIENT_ABI_SOURCE.read_text(encoding="utf-8")

        for name in ASYNC_RESPONSE_EXPORTS:
            with self.subTest(function=name):
                declaration = re.search(
                    rf"pub (?:unsafe )?extern \"C\" fn {name}\(", source
                )
                self.assertIsNotNone(declaration)
                assert declaration is not None
                self.assertIn("pub unsafe extern \"C\" fn", declaration.group(0))
                self.assertIn("# Safety", source[max(0, declaration.start() - 400) : declaration.start()])


if __name__ == "__main__":
    unittest.main()
