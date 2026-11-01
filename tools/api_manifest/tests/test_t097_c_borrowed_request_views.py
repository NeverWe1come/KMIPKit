"""T097 contract for request-owned C ABI text spans."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
HEADER = ROOT / "bindings" / "c" / "include" / "kmipkit.h"
BORROWED_REQUEST_VIEWS = (
    "kmipkit_add_attribute_request_unique_identifier",
    "kmipkit_adjust_attribute_request_unique_identifier",
    "kmipkit_adjust_attribute_request_attribute_reference",
    "kmipkit_delete_attribute_request_unique_identifier",
    "kmipkit_delete_attribute_request_attribute_reference",
    "kmipkit_modify_attribute_request_unique_identifier",
    "kmipkit_set_attribute_request_unique_identifier",
)


class BorrowedRequestViewDocumentationTests(unittest.TestCase):
    def test_request_text_getters_document_the_owner_lifetime(self) -> None:
        header = HEADER.read_text(encoding="utf-8")
        for symbol in BORROWED_REQUEST_VIEWS:
            with self.subTest(symbol=symbol):
                declaration = re.search(rf"\bint32_t\s+{symbol}\(", header)
                self.assertIsNotNone(declaration, f"missing C declaration: {symbol}")
                position = declaration.start()
                traceability_comment = header.rfind("/*", 0, position)
                api_comment = header.rfind("/*", 0, traceability_comment)
                documentation = header[api_comment:traceability_comment].casefold()
                self.assertIn("borrowed view", documentation)
                self.assertIn(
                    "remains valid only while request handle remains open",
                    documentation,
                )
                self.assertIn("copy the bytes before releasing", documentation)


if __name__ == "__main__":
    unittest.main()
