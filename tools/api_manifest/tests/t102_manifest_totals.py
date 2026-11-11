"""Shared manifest snapshot guards and historical T102 Rust audit totals."""

from typing import Any


def without_0047_versioned_values(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Keep the pre-versioned-value T102 snapshot focused on its original rows."""
    return [
        row for row in rows
        if "KMIPKIT-0047-FR-009" not in row.get("requirementIds", [])
    ]


# KMIPKIT-0053 adds 144 explicitly-owned blocked candidates for follow-up.
T102_BLOCKED_CANDIDATES = 0
# KMIPKIT-0091 adds fluent DX declarations; KMIPKIT-0095 adds response view
# declarations and Rust trait exclusions pending cross-language adapters.
T102_PUBLIC_DECLARATIONS = 13_880
T102_KMIP30_DRAFT_ROUTE_DECLARATIONS = 36
T102_APPROVED_EXCLUSIONS = 18_964
T102_RUST_AUDIT_MANIFEST = 3_287
T102_RUST_AUDIT_EXCLUDED = 7_176
T102_RUST_AUDIT_OBSERVED = 10_463
