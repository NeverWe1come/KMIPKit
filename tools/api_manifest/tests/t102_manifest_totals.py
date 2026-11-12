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
# KMIPKIT-0096 adds four expert-position declarations; KMIPKIT-0097 adds
# 46 exact Rust key-pair declarations; KMIPKIT-0099 adds 50 exact Rust
# Create Split Key declarations and 18 standard-trait exclusions; KMIPKIT-0100
# adds 40 exact Rust Register declarations and 18 standard-trait exclusions;
# KMIPKIT-0101 adds 188 exact Rust lifecycle declarations.
# KMIPKIT-0093 reconciliation adds 1,614 explicitly blocked declarations and
# 3,467 reviewed Rust trait exclusions.
# KMIPKIT-0102 adds the Rust fluent cryptographic operation builders.
# KMIPKIT-0103 adds 27 C, KMIPKIT-0104 adds 121 Java, and KMIPKIT-0105 adds
# 148 Python fluent crypto declarations. KMIPKIT-0096 also adds one C expert
# attribute insertion and one blocked Java insertion declaration.
# KMIPKIT-0095 adds 9 C/Java/Python response rendering declarations and 3 JNI exclusions.
# KMIPKIT-0096 adds the Python expert attribute insertion declaration.
# Update this full-manifest snapshot only when an approved feature intentionally
# adds or removes declarations.
T102_PUBLIC_DECLARATIONS = 16_359
T102_KMIP30_DRAFT_ROUTE_DECLARATIONS = 36
T102_APPROVED_EXCLUSIONS = 22_614
T102_RUST_AUDIT_MANIFEST = 3_287
T102_RUST_AUDIT_EXCLUDED = 7_176
T102_RUST_AUDIT_OBSERVED = 10_463
