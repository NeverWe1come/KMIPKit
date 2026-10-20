"""Default Python package source must not contain owner-gated candidate code."""

from __future__ import annotations

import unittest
from pathlib import Path


PACKAGE_ROOT = Path(__file__).resolve().parents[1] / "src" / "kmipkit"
CANDIDATE_TOKENS = (
    "kmipkit_client_batch_item_pkcs11",
    "kmipkit_client_batch_item_response_pkcs11_ttlv",
    "_client_batch_item_pkcs11_candidate",
    "_Pkcs11Response",
    "_Pkcs11BatchItemResult",
    "_pkcs11_response",
)


class CandidatePackageIsolationTests(unittest.TestCase):
    def test_default_python_package_sources_exclude_pkcs11_candidate_support(self) -> None:
        source_files = sorted(PACKAGE_ROOT.rglob("*.py"))
        self.assertTrue(source_files, "the default Python package source tree is missing")
        exposed_tokens = {
            token
            for source_file in source_files
            for token in CANDIDATE_TOKENS
            if token in source_file.read_text(encoding="utf-8")
        }
        self.assertFalse(exposed_tokens, sorted(exposed_tokens))


if __name__ == "__main__":
    unittest.main()
