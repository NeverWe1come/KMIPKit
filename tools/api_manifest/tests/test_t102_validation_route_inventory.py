"""Keep 0051 Rust validation blocked while preserving historical T102 slices."""

from collections import Counter
import json
from pathlib import Path
import unittest

from tools.api_manifest.tests.pre_send_validation_routes import (
    REQUEST_TYPES,
    TRAIT_PATH,
    VALIDATION_TRAIT_ROUTES,
    is_pre_send_validation_trait,
)
from tools.api_manifest.tests.t102_manifest_totals import T102_BLOCKED_CANDIDATES

ROOT = Path(__file__).resolve().parents[3]
REPORT = ROOT / "specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json"
PREDECESSOR_REPORT = ROOT / "specs/050-parameters-auth-client/reports/rust-export-report-post-sync-2026-11-07.json"


class PreSendValidationRouteInventoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads((ROOT / "specification/api/public-api.json").read_text(encoding="utf-8"))
        cls.report = json.loads(REPORT.read_text(encoding="utf-8"))

    def test_exact_252_trait_routes_stay_blocked_under_existing_owners(self) -> None:
        self.assertEqual(56, len(REQUEST_TYPES))
        self.assertEqual(252, len(VALIDATION_TRAIT_ROUTES))
        rows = [row for row in self.manifest["publicDeclarations"] if is_pre_send_validation_trait(row)]
        self.assertEqual(VALIDATION_TRAIT_ROUTES, {(row["packageId"], row["exportName"]) for row in rows})
        observed = {
            (row["packageId"], row["exportName"])
            for row in self.report["declarations"] + self.report["conditionalDeclarations"]
            if f" as {TRAIT_PATH}>::" in row["exportName"]
        }
        self.assertEqual(VALIDATION_TRAIT_ROUTES, observed)
        async_requests = {"CancelRequest", "PollRequest", "ProcessRequest", "QueryAsyncRequestsRequest"}
        for row in rows:
            request = row["exportName"].split(" as ", 1)[0].rsplit("::", 1)[1]
            owner = "KMIPKIT-0009" if request in async_requests else (
                "KMIPKIT-0044" if request == "RegisterRequest" else "KMIPKIT-0043"
            )
            with self.subTest(package=row["packageId"], export=row["exportName"]):
                self.assertEqual(owner, row["candidateSpecId"])
                self.assertEqual("blocked", row["declarationState"])
                self.assertEqual("public_binding_surface", row["capabilityId"])
                self.assertEqual("four-language", row["parityScope"])
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("parityEvidenceIds", row)
                self.assertNotIn("approvalSource", row)
        excluded = {(row["packageId"], row["exportName"]) for row in self.manifest["approvedExclusions"]}
        self.assertFalse(VALIDATION_TRAIT_ROUTES & excluded)

    def test_all_930_stack_additions_remain_blocked_without_parity_promotions(self) -> None:
        predecessor = json.loads(PREDECESSOR_REPORT.read_text(encoding="utf-8"))
        def routes(report):
            return {(row["packageId"], row["exportName"]) for row in report["declarations"] + report["conditionalDeclarations"]}
        additions = routes(self.report) - routes(predecessor)
        self.assertEqual(930, len(additions))
        self.assertFalse(routes(predecessor) - routes(self.report))
        rows = [row for row in self.manifest["publicDeclarations"] if (row["packageId"], row["exportName"]) in additions]
        self.assertEqual(930, len(rows))
        self.assertEqual({"KMIPKIT-0043": 902, "KMIPKIT-0009": 24, "KMIPKIT-0044": 4}, Counter(row["candidateSpecId"] for row in rows))
        for row in rows:
            self.assertEqual("blocked", row["declarationState"])
            self.assertEqual("public_binding_surface", row["capabilityId"])
            self.assertNotIn("languageMappings", row)
            self.assertNotIn("parityEvidenceIds", row)
        self.assertEqual(902, T102_BLOCKED_CANDIDATES)

    def test_trait_selector_does_not_hide_other_members_types_or_packages(self) -> None:
        package, export = sorted(VALIDATION_TRAIT_ROUTES)[0]
        self.assertTrue(is_pre_send_validation_trait({"packageId": package, "exportName": export}))
        for wrong_package, wrong_export in (
            (package, export.rsplit("::", 1)[0] + "::future_member"),
            (package, export.replace(TRAIT_PATH, "core::fmt::Debug")),
            (package, export.replace("Request as ", "FutureRequest as ")),
            ("rust-unowned", export),
        ):
            self.assertFalse(is_pre_send_validation_trait({"packageId": wrong_package, "exportName": wrong_export}))
