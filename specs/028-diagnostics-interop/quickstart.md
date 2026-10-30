# Validation Quickstart: KMIPKIT-0042

Run from the repository root using Rust 1.94. These checks do not claim server availability, official OASIS test-case passes, certificate validity, profile support, or interoperability.

## Focused tests

    cargo test -p kmipkit-protocol interop_operation_tests
    cargo test -p kmipkit-protocol log_operation_tests
    cargo test -p kmipkit-protocol validate_operation_tests
    cargo test -p kmipkit-client interop_execution_tests
    cargo test -p kmipkit-client log_execution_tests
    cargo test -p kmipkit-client validate_execution_tests

Expected evidence: correct required/optional fields and empty responses; open enumeration and unknown-value preservation; Log message redaction; Validate repeated/mixed chain inputs, date omission, each stable requirement ID, and required result indicator; fake transport confirms one exchange, result/Pending/delivery passthrough, and no retry.

## Review gates

    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo llvm-cov --workspace --all-features
    python tools/normative_catalog/validate.py
    python tools/normative_catalog/report.py --check
    git diff --check

Apply the exact gates in docs/development/testing.md: 95% line coverage for the protocol crate and changed production code, 90% Rust workspace coverage, and 100% normative traceability. Apply the 85% per-scope adapter/transport thresholds when those scopes change.

## 1.0 interoperability release gate

Before 1.0, exercise Interop, Log, and Validate against at least two independent KMIP server implementations and record product/version, transport, profile, operation, result, and workaround in docs/development/kmip-interoperability.md and the test report, following docs/development/testing.md. A server that does not support an operation is recorded as SKIP/N/A with its reason; that result is not passing interoperability evidence and leaves the release gate open.

## Implemented Rust examples

See `docs/user-guide/en/diagnostics-and-interop.md` and its Spanish companion.
The examples are exercised by
`cargo test -p kmipkit-protocol --test diagnostics_guide_examples`.
All operation models and typed Rust calls are available. Cross-language
manifest/adapters/parity remain gated by KMIPKIT-0043 (T044).

Run the focused commands above after every operation refactor. Current
source-derived suites contain 8 Interop protocol tests, 7 Log protocol tests,
16 Validate protocol tests, 9 Interop client tests, 9 Log client tests and
11 Validate client tests. Five actor-attributed Validate client requirements
have named tests; the §6.1.57-006 test verifies only that an omitted Validity
Date remains absent in request serialization. It does not verify assumed-date
evaluation.
Actual full-gate results and commit IDs are in `implementation-evidence.md`.
No direct official fixture or two-server passing evidence is available for
these operations, so those release claims remain open.
