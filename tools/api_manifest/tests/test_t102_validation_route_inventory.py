"""Keep the 0051 Rust validation routes reconciled after 0053 parity work."""

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

    def test_exact_252_trait_routes_are_reconciled_as_explicit_exclusions(self) -> None:
        self.assertEqual(56, len(REQUEST_TYPES))
        self.assertEqual(252, len(VALIDATION_TRAIT_ROUTES))
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        rows = [
            exclusions[route]
            for route in VALIDATION_TRAIT_ROUTES
            if route in exclusions
        ]
        self.assertEqual(252, len(rows))
        self.assertEqual(
            VALIDATION_TRAIT_ROUTES,
            {(row["packageId"], row["exportName"]) for row in rows},
        )
        observed = {
            (row["packageId"], row["exportName"])
            for row in self.report["declarations"] + self.report["conditionalDeclarations"]
            if f" as {TRAIT_PATH}>::" in row["exportName"]
        }
        self.assertEqual(VALIDATION_TRAIT_ROUTES, observed)
        for row in rows:
            with self.subTest(package=row["packageId"], export=row["exportName"]):
                self.assertEqual(
                    {
                        "kind": "feature-specification",
                        "path": "specs/053-api-parity-facade/approval-record.md",
                        "status": "Approved",
                        "evidence": "Direct maintainer instruction approving the complete Phase 5 plan and autonomous implementation.",
                    },
                    row["approvalSource"],
                )
                self.assertIn("exact Rust trait projection", row["reason"])
                self.assertTrue(row["evidence"])

    def test_all_930_stack_additions_are_reconciled_without_blocked_rows(self) -> None:
        predecessor = json.loads(PREDECESSOR_REPORT.read_text(encoding="utf-8"))
        def routes(report):
            return {(row["packageId"], row["exportName"]) for row in report["declarations"] + report["conditionalDeclarations"]}
        additions = routes(self.report) - routes(predecessor)
        self.assertFalse(routes(predecessor) - routes(self.report))
        public = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }
        self.assertEqual(930, len(additions))
        self.assertEqual(930, sum(route in public or route in exclusions for route in additions))
        self.assertFalse(set(public) & set(exclusions) & additions)
        public_rows = [public[route] for route in additions if route in public]
        self.assertEqual(24, len(public_rows))
        self.assertTrue(all(row["declarationState"] == "verified" for row in public_rows))
        self.assertTrue(all(route in exclusions for route in additions if route not in public))
        self.assertFalse(any(row.get("declarationState") == "blocked" for row in public_rows))
        self.assertEqual(0, T102_BLOCKED_CANDIDATES)

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
