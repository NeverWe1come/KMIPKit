# Validation Quickstart: KMIPKIT-0021

This guide is for the implementation PR. The current design/specification PR does not add operation code, so operation-specific commands remain pending until implementation is merged.

## Prerequisites

- KMIPKIT-0019's shared `OperationData`, Cryptographic Parameters, and multipart field types are merged to the active release branch.
- Rust 1.94 / Edition 2024 and the repository's pinned Cargo toolchain are available.
- Work is on `feature/KMIPKIT-0021-hash-mac-signature-operations`, created from the current `release/1.0.0` branch.
- Do not download OASIS material. Use the pinned local files under `specification/oasis/kmip-2.1/`.

## Focused protocol checks

```powershell
cargo test -p kmipkit-protocol hash_operation_tests
cargo test -p kmipkit-protocol mac_operation_tests
cargo test -p kmipkit-protocol mac_verify_operation_tests
cargo test -p kmipkit-protocol sign_operation_tests
cargo test -p kmipkit-protocol signature_verify_operation_tests
```

Expected: each focused test group validates exact table fields, positive and negative payload shapes, and lossless round trips for unknown values. No test runs a local cryptographic algorithm.

## Fake-client transport checks

```powershell
cargo test -p kmipkit-client hash_execution_tests
cargo test -p kmipkit-client mac_execution_tests
cargo test -p kmipkit-client mac_verify_execution_tests
cargo test -p kmipkit-client sign_execution_tests
cargo test -p kmipkit-client signature_verify_execution_tests
```

Expected: each call sends one request with the corresponding operation code; success, failure, Pending, response-shape, and delivery-state behavior use the shared execution path; the fake transport observes no retry or automatic multipart follow-up.

## Workspace and catalog gates

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
git diff --check
```

Expected: formatting, Clippy, all workspace tests, complete catalog validation, generated report consistency, and whitespace checks succeed. Run repository coverage commands and the GitHub Actions Linux/Windows/macOS matrix before requesting review; meet the existing crate/global coverage thresholds.

## Response discrepancy cases

The focused Signature/MAC Verify client tests must include:

1. Single-part response with Validity Indicator: accepted.
2. Non-final multipart response without Validity Indicator: accepted.
3. Non-final multipart response with Validity Indicator: typed response-shape error; generic TTLV retained.
4. Final multipart response with Validity Indicator: accepted while preserving `KMIPKIT-DISC-048` as open.
5. Final multipart response without Validity Indicator: accepted while preserving `KMIPKIT-DISC-048` as open.

Expected: no final multipart case is used to claim server conformance while the discrepancy remains open.

## Official evidence boundary

The linked Hash, MAC, Sign, and Signature Verify fixtures are unavailable in the pinned repository, and MAC Verify has no source-linked official Test Case. Source-derived tests must be labeled accordingly. Do not report an official fixture pass or profile claim from these focused tests.
