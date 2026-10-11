# Quickstart: KMIP 2.1 Policy and Usage Controls

This checklist is for the future implementation PR. It makes no claim that the feature is implemented or that any command has passed.

## Before implementation

1. Confirm the active branch is `feature/KMIPKIT-0039-policy-controls` based on the approved release branch.
2. Verify pinned source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256` and review `spec.md`, `plan.md`, `traceability.md`, and the two API contracts.
3. Check whether the canonical public API feature KMIPKIT-0043 has merged. Rust protocol/client work may proceed without it; C, Java, Python, manifest, and generated parity tasks must wait for its approved contract.
4. Confirm typed Encrypt, MAC, and Sign request variants exist on the active release baseline before implementing guard execution. Reuse them; do not reproduce their wire models.

## Required implementation order

1. Implement Get Constraints and its focused model/client tests.
2. Implement Get Usage Allocation and its tests; then add the local caller guard and its delivery-state tests.
3. Implement Set Constraints, Set Defaults, and client-to-server Set Endpoint Role in separate Red/Green/Refactor sequences.
4. Update requirement traceability and normative catalog implementation/verification references. Run the pinned catalog validator/report generator; never edit generated reports by hand.
5. After 0043 merges, add public manifest entries, adapters, generated outputs, and C/Java/Python parity tests in a coordinated slice.
6. Add English/Spanish examples and update the interop matrix. Record real server/version evidence only after the tests actually run.

## Focused checks

Run from the repository root after the matching tests exist:

```text
cargo test -p kmipkit-protocol --lib get_constraints
cargo test -p kmipkit-client --lib get_constraints
cargo test -p kmipkit-protocol --lib get_usage_allocation
cargo test -p kmipkit-client --lib usage_allocation_guard
cargo test -p kmipkit-client --lib set_constraints
cargo test -p kmipkit-client --lib set_defaults
cargo test -p kmipkit-client --lib set_endpoint_role
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --write
python tools/normative_catalog/report.py --check
python tools/api_manifest/generate.py --check
git diff --check
```

Before requesting review, run the complete repository checks required by `AGENTS.md`:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Run API-manifest unit tests and all binding-specific build/parity commands listed in `docs/development/api-manifest.md` after 0043 is available. The generated manifest check is not a substitute for testing handwritten adapters.

## §6.1.23 focused acceptance checklist

- The request encodes optional Unique Identifier and required Usage Limits Count in Table 232 order and type.
- The response requires and returns the concrete Unique Identifier in Table 233.
- A context requires a complete successful allocation plus the exact request count and caller-known unit.
- Guarded Encrypt/MAC/Sign requires a matching concrete object identifier and positive caller-supplied cost.
- Cost equal to known available units is accepted; cost above it, zero, negative, mismatched ID, and ID Placeholder fail before transport.
- A proven `NotSent` releases the reservation; `PossiblySent`, `ResponseStarted`, complete results, and Pending retain it locally.
- No test or documentation equates local reservation with server consumption or claims coverage of unguarded/raw/external activity.
- Decrypt, MAC Verify, Signature Verify, and Hash are not charged by the usage guard.

Report command, platform, and result for each Red, Green, and Refactor evidence commit. Do not claim a direct official OASIS Test Cases pass for these operations; none is cataloged in the approved source inventory.
