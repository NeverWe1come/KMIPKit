"""Pin exact T102 exclusions for Rust-only error diagnostics."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "specification" / "api" / "public-api.json"
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
APPROVAL_SOURCE = {
    "crypto_kind": {
        "kind": "feature-specification",
        "path": "specs/021-hash-mac-signature-operations/spec.md",
        "status": "Approved",
        "evidence": "**Status**: Approved",
    },
    "rng_kind": {
        "kind": "feature-specification",
        "path": "specs/027-rng-operations/spec.md",
        "status": "Approved",
        "evidence": "**Status**: Approved for implementation — 2026-10-11",
    },
    "ticket": {
        "kind": "feature-specification",
        "path": "specs/024-credential-sessions/spec.md",
        "status": "Approved",
        "evidence": "**Status**: Approved for implementation — 2026-10-11",
    },
    "usage": {
        "kind": "feature-specification",
        "path": "specs/025-policy-controls/spec.md",
        "status": "Approved",
        "evidence": "**Status**: Approved for implementation — 2026-10-11",
    },
}
REASONS = {
    "crypto_kind": (
        "This exact Rust kind accessor exposes parser detail that has no foreign API; "
        "the corresponding client flow exposes only the normalized shared error outcome."
    ),
    "rng_kind": (
        "This exact Rust kind accessor exposes parser detail that has no foreign API; "
        "the corresponding client flow exposes only the normalized shared error outcome."
    ),
    "ticket": (
        "This exact error taxonomy is emitted by the typed Rust Ticket parser; C, Java, "
        "and Python use generic TTLV/session APIs and expose no equivalent typed Ticket parser."
    ),
    "usage": (
        "This exact Rust error taxonomy refines local Usage Allocation validation; foreign "
        "context/guard APIs collapse failures to generic invalid-input/status errors."
    ),
}
EVIDENCE = {
    "crypto_kind": (
        "KMIPKIT-0021 owner; KMIPKIT-0043-FR-010/021. OASIS KMIP v2.1 §§6.1.32, "
        "6.1.33, 6.1.55, and 6.1.56 govern the operations, not this Rust parser accessor."
    ),
    "rng_kind": (
        "KMIPKIT-0041-FR-001/003/004/005/006; KMIPKIT-0043-FR-010/021. OASIS KMIP "
        "v2.1 §6.1.50 governs RNG operations, not this Rust parser accessor."
    ),
    "ticket": (
        "KMIPKIT-0038-FR-001/002/005/008; KMIPKIT-0043-FR-010/021. OASIS KMIP v2.1 "
        "§7.39 Table 391 and §11.57 Table 488 define Ticket data, not this Rust parser taxonomy."
    ),
    "usage": (
        "KMIPKIT-0039-FR-005/009; KMIPKIT-0043-FR-010/021. OASIS KMIP v2.1 §6.1.23 "
        "Table 234 gives allocation context; these local Rust errors are not protocol statuses."
    ),
}


def expected_exclusions() -> dict[tuple[str, str], str]:
    expected: dict[tuple[str, str], str] = {
        (
            "rust-kmipkit-protocol",
            "kmipkit_protocol::CryptographicOperationError::kind",
        ): "crypto_kind",
    }
    for package, crate in (
        ("rust-kmipkit", "kmipkit"),
        ("rust-kmipkit-protocol", "kmipkit_protocol"),
    ):
        for error in ("RngRetrieveError", "RngSeedError"):
            expected[(package, f"{crate}::{error}::kind")] = "rng_kind"

        ticket = f"{crate}::TicketValidationError"
        ticket_kind = f"{crate}::TicketValidationErrorKind"
        for name in (
            ticket,
            f"{ticket}::kind",
            ticket_kind,
            *(f"{ticket_kind}::{variant}" for variant in (
                "DuplicateField",
                "FieldOutOfOrder",
                "InvalidTtlvStructure",
                "MissingField",
                "WrongFieldType",
            )),
        ):
            expected[(package, name)] = "ticket"

    for package, crate in (
        ("rust-kmipkit", "kmipkit"),
        ("rust-kmipkit-client", "kmipkit_client"),
    ):
        prefix = f"{crate}::UsageAllocationContextError"
        for name in (
            prefix,
            *(f"{prefix}::{variant}" for variant in (
                "AllocationNotSuccessful",
                "CostExceedsAvailable",
                "InvalidAllocationCount",
                "InvalidUsageCost",
                "MismatchedUniqueIdentifier",
                "MissingResolvedIdentifier",
                "MissingUniqueIdentifier",
                "ReservationOverflow",
                "UnsupportedOperation",
            )),
        ):
            expected[(package, name)] = "usage"
    return expected


class T102MacroLot1ErrorTaxonomyExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
        cls.declarations = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["publicDeclarations"]
        }
        cls.exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in cls.manifest["approvedExclusions"]
        }

    def test_exact_rust_error_diagnostics_are_approved_exclusions(self) -> None:
        expected = expected_exclusions()
        self.assertEqual(len(expected), 41)
        for identity, category in expected.items():
            with self.subTest(identity=identity):
                self.assertTrue(identity not in self.declarations, f"still declared: {identity}")
                row = self.exclusions[identity]
                self.assertEqual(row["candidateSpecId"], "KMIPKIT-0043")
                self.assertEqual(row["approvalSource"], APPROVAL_SOURCE[category])
                self.assertEqual(row["reason"], REASONS[category])
                self.assertEqual(row["evidence"], EVIDENCE[category])
                self.assertTrue(all(item in row["evidence"] for item in T102_REQUIREMENTS))
                self.assertLessEqual(len(row["evidence"].encode("utf-8")), 250)


if __name__ == "__main__":
    unittest.main()
