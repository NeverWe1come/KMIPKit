# Validation Quickstart: KMIPKIT-0041

Run from repository root using Rust 1.94. These checks do not claim official vectors, interoperability, certification, or an RNG profile.

## Focused checks

    cargo test -p kmipkit-protocol rng_retrieve_operation_tests
    cargo test -p kmipkit-protocol rng_seed_operation_tests
    cargo test -p kmipkit-client rng_retrieve_execution_tests
    cargo test -p kmipkit-client rng_seed_execution_tests

Expect table-derived field/type and round-trip tests, malformed request/response negatives, opaque-byte redaction, exactly one fake-transport exchange, Pending/result/delivery preservation, and no retry. A zero Data Length response for RNG Seed must remain successful.

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

Before 1.0, exercise RNG Retrieve and RNG Seed against at least two independent KMIP server implementations and record product/version, transport, profile, operation, result, and workaround in docs/development/kmip-interoperability.md and the test report, following docs/development/testing.md. A server that does not support an operation is recorded as SKIP/N/A with its reason; that result is not passing interoperability evidence and leaves the release gate open.