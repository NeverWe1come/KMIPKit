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