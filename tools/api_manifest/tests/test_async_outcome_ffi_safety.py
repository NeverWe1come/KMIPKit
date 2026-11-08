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
        export_pattern = re.compile(
            r"(?P<docs>(?:^///.*\n)+)^#\[unsafe\(no_mangle\)\]\n"
            r"^pub (?P<unsafe>unsafe )?extern \"C\" fn (?P<name>\w+)\(",
            re.MULTILINE,
        )
        declarations = {
            declaration.group("name"): declaration
            for declaration in export_pattern.finditer(source)
        }

        for name in ASYNC_RESPONSE_EXPORTS:
            with self.subTest(function=name):
                declaration = declarations.get(name)
                self.assertIsNotNone(declaration)
                assert declaration is not None
                self.assertEqual(declaration.group("unsafe"), "unsafe ")
                self.assertIn("# Safety", declaration.group("docs"))

    def test_async_response_output_helper_keeps_its_unsafe_precondition(self) -> None:
        source = CLIENT_ABI_SOURCE.read_text(encoding="utf-8")
        helper = re.search(
            r"(?P<docs>(?:^///.*\n)+)^unsafe fn async_response_v1_output\(",
            source,
            re.MULTILINE,
        )

        self.assertIsNotNone(helper)
        assert helper is not None
        self.assertIn("# Safety", helper.group("docs"))


if __name__ == "__main__":
    unittest.main()
