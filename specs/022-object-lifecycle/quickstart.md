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

### Check (US2) maintainer clarification

- The maintainer approved applying §6.1.7 to failed Check responses: preserve and type each returned `Usage Limits Count`, `Cryptographic Usage Mask`, and `Lease Time`; a failed response still does not require `Unique Identifier`.
- Follow-up protocol and fake-client tests now exercise all three policy fields together on `Operation Failed`. `cargo test -p kmipkit-protocol policy_denial_response_returns_the_policy_value_without_a_success_identifier` and `cargo test -p kmipkit-client check_policy_denial_exposes_the_returned_mask_without_an_identifier` passed; focused Clippy, format, and diff checks passed.

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

### Register (US8)

- RED commit: `ec1c31f1` (`test(022): add Register RED coverage`). The protocol and client Register test targets failed as expected before the model and client operation existed.
- GREEN commit: `ff2f1581` (`feat(022): implement Register operation`). Added Table 291 ordered request encoding, Table 293 local required-attribute checks, typed Table 292 Unique Identifier response, failure/Pending passthrough, and one-exchange client methods.
- REFACTOR commit: `a6385456` (`refactor(022): finalize Register contract tests`). Added the ID Placeholder producer-to-Check batch case and verified a Certificate needs Certificate Length without a cryptographic Usage Mask.
- Verification passed: `cargo test -p kmipkit-protocol -p kmipkit-client` (package regression suite), focused Register protocol/client tests (11 and 5 tests), `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, and `git diff --check`.
- No direct OASIS Test Case is listed for Register; tests are derived from §6.1.43 Tables 291–294. The Red/Green/Refactor commits use the maintainer-approved DCO sign-off; pinned upstream sources remain unchanged.

### Revoke (US9)

- RED commits: `997e030f` (`test(022): add Revoke RED coverage`), `2d045c45` (require §4.47/Table 123 nested Structure), and `b0323416` (require the §11.36/Table 470 Revoke operation code `0x13`). The latter tests failed against the prior `0x11` value.
- GREEN commit: `9884ded4` (`feat(022): implement Revoke operation`) adds the raw-preserving reason code, nested Table 123 request encoding, optional compromise date validation, typed success identifier, shared failure/Pending results, and the one-exchange client method. Follow-up fix `bb4f5387` aligns Revoke with Table 470 and corrects the pre-existing Archive code to `0x15`; both had collided at `0x13`. The request Debug output redacts the identifier and audit message.
- REFACTOR commit: `f7831af5` (`refactor(022): reuse shared Revoke payload helpers`) removes duplicate TTLV-tag, item, error, and identifier helpers while preserving behavior.
- Verification passed: focused Revoke protocol and client tests (10 and 6 tests), focused Archive protocol/client tests (12 and 4 tests), `cargo test -p kmipkit-protocol -p kmipkit-client` (420 client unit tests and all protocol tests passed), `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`.
- No direct OASIS Test Case is listed for Revoke; these table-derived tests cover §6.1.44 Tables 295–297, §4.47/Table 123, and §11.48/Table 481. Pinned upstream sources remain unchanged.
