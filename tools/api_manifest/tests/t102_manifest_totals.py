"""Shared manifest snapshot guards and historical T102 Rust audit totals."""

from typing import Any


def without_0047_versioned_values(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Keep the pre-versioned-value T102 snapshot focused on its original rows."""
    return [
        row for row in rows
        if "KMIPKIT-0047-FR-009" not in row.get("requirementIds", [])
    ]


T102_BLOCKED_CANDIDATES = 0
T102_PUBLIC_DECLARATIONS = 12_701
T102_APPROVED_EXCLUSIONS = 17_594
T102_RUST_AUDIT_MANIFEST = 3_287
T102_RUST_AUDIT_EXCLUDED = 7_176
T102_RUST_AUDIT_OBSERVED = 10_463
