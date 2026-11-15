"""Checks for the explicitly opt-in KMIP 3.0 draft API surface."""

from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path

from tools.api_manifest.kmip30_draft_api import (
    DraftApiManifestError,
    validate_kmip30_draft_api,
)


ROOT = Path(__file__).resolve().parents[3]
CATALOG_PATH = ROOT / "specification" / "api" / "kmip-3.0-draft-api.json"
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"


class Kmip30DraftApiCatalogTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))
        self.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))

    def test_seven_draft_routes_match_reviewed_codes_and_public_bindings(self) -> None:
        validate_kmip30_draft_api(self.catalog, self.manifest, ROOT)

    def test_catalog_rejects_an_unreviewed_operation_code(self) -> None:
        catalog = copy.deepcopy(self.catalog)
        catalog["operations"][0]["operationCode"] = 0x43

        with self.assertRaises(DraftApiManifestError):
            validate_kmip30_draft_api(catalog, self.manifest, ROOT)

    def test_catalog_rejects_a_public_binding_drift(self) -> None:
        catalog = copy.deepcopy(self.catalog)
        catalog["operations"][0]["python"] = "kmipkit.extensions.wrong_route"

        with self.assertRaises(DraftApiManifestError):
            validate_kmip30_draft_api(catalog, self.manifest, ROOT)


if __name__ == "__main__":
    unittest.main()
