# Validation Quickstart: KMIPKIT-0040

These are implementation checks, not evidence of formal conformance, interoperability, or PKCS#11 Profile support. Run at repository root with Rust 1.94.

## Focused checks

    cargo test -p kmipkit-protocol pkcs11_operation_tests
    cargo test -p kmipkit-client pkcs11_execution_tests

The first suite covers Tables 273/274, optional absence, Function Enumeration/Integer preservation, generic structures, malformed shapes, and redaction. The fake-transport suite proves operation selection, one exchange, result/Pending/correlation/delivery preservation, and no retry.

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

Before 1.0, exercise this operation against at least two independent KMIP server implementations and record product/version, transport, profile, operation, result, and workaround in docs/development/kmip-interoperability.md and the test report, following docs/development/testing.md. A server that does not support the operation is recorded as SKIP/N/A with its reason; that result is not a passing interoperability test and leaves the release gate open.
