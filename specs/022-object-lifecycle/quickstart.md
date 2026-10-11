# Validation Quickstart: KMIPKIT-0036

This guide records the implementation validation path for Certify, Check, Obtain Lease, Re-certify, Re-key, Re-key Key Pair, Re-Provision, Register, and Revoke. It does not claim formal OASIS conformance, profile support, interoperability, or certification by itself.

## Prerequisites

- Start from the active `release/1.0.0` branch in the dedicated implementation worktree.
- Confirm the shared typed messages, batches, client execution, objects/attributes, common results, and secret storage remain available.
- Verify the pinned OASIS checksums and keep upstream files unchanged.
- For language parity, wait for KMIPKIT-0043's canonical manifest/mappings. Rust protocol and client work can proceed independently.

## Focused validation

Run each protocol test target and its fake-transport client target. Cover operation code, one exchange, batch/result association, failure delivery state, Pending passthrough, and no retry. Every test identifies the exact §6.1.x clause/table or requirement ID in [traceability.md](traceability.md).

Use pinned Test Case fixtures only when available. Re-key maps to KMIPKIT-TEST-CN01-2-76 through -87; the approved spec lists no direct Test Case IDs for the other eight operations. Label table-derived vectors accordingly.

## Repository gates

From the repository root, run focused cargo targets, then the repository checks:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
```

Run pinned generation-drift, security/dependency, and supported-platform CI checks per active repository automation. Do not claim a check passed without actual output.

## Release evidence

Record separate Red, Green, and Refactor commit IDs per operation in the draft PR. Include focused/full-suite commands, coverage and traceability results, generator status, binding parity state, unavailable fixture notes, risks, and actual test outputs. Update this guide with results only during implementation.

### Re-key Key Pair (US6)

- RED protocol: `cargo test -p kmipkit-protocol rekey_key_pair_operation_tests --no-fail-fast` failed as expected because `RekeyKeyPairRequest`, `RekeyKeyPairResponse`, and `RekeyKeyPairError` were not yet exported.
- RED client: `cargo test -p kmipkit-client rekey_key_pair_execution_tests --no-fail-fast` failed as expected because the request type, `ClientRequest::RekeyKeyPair`, `ClientOperation::RekeyKeyPair`, and `Client::rekey_key_pair` were not yet implemented.
- GREEN focused: `cargo test -p kmipkit-protocol rekey_key_pair_operation_tests --no-fail-fast` and `cargo test -p kmipkit-client rekey_key_pair_execution_tests --no-fail-fast` both passed, six tests each.
- GREEN package regression: `cargo test -p kmipkit-protocol -p kmipkit-client` passed.
- Guide examples: `python scripts/test_user_guide_examples.py` compiled all 14 marked Rust examples.
- REFACTOR verification: both focused operation tests passed (seven protocol tests and six client tests); `cargo test -p kmipkit-protocol -p kmipkit-client`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` passed.

### Re-Provision (US7)

- RED commit: `07e69112` (`test(022): add Re-Provision RED coverage`). `cargo test -p kmipkit-protocol reprovision_operation_tests --no-fail-fast` and the matching client command both failed because the Re-Provision protocol module did not yet exist.
- GREEN commit: `cd6456a1` (`feat(022): implement Re-Provision operation`). Focused protocol tests passed (7 tests) and fake-transport client tests passed (6 tests), including all three request forms, mutually exclusive object forms, optional/malformed result identifier, server failure, Pending correlation preservation, partial-write delivery state, and no retry.
- REFACTOR verifies the response-view projection through one constructor, retains the typed model and client dispatch, and updates the English lifecycle guide and public API architecture documentation.
- No direct OASIS Test Case is listed for Re-Provision; tests are derived from §6.1.48 Tables 313–315. `KMIPKIT-DISC-040` is applied; upstream source files remain unchanged.
