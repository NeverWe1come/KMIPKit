# Task 29 — Framed Single-Part Client Red Contracts

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

## Scope and evidence boundary

Added two fake-transport client contracts in `crates/kmipkit-client/tests/unit/encrypt_decrypt_multipart_execution_tests.rs` for both Encrypt and Decrypt. The accepted form has Init Indicator=true, Final Indicator=true, no Correlation Value, and Data. It asserts exactly one exchange and inspects the transmitted TTLV to verify both indicators and the exact Data bytes. The Data-omission form asserts the exact sanitized `Validation(InvalidInput, NotSent)` display and zero exchanges while KMIPKIT-DISC-045 remains open. The tests add no client state and do not alter production code.

T027/T028/T029 expected-client tests are compile-Red because `ClientRequest::{Encrypt, Decrypt}` and `ClientResponseView::{encrypt, decrypt}` are deliberately absent until later tasks T031/T038. As a result, client runtime assertions have not run; these tests specify behavior for the future typed dispatch and do not establish it as currently passing. The protocol request-builder matrix remains a separate pre-encoding layer: its valid case passes, while its invalid case fails because validation is not implemented yet.

## Focused protocol matrix

Valid-row command (WSL Ubuntu-26.04):

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests::both_operations_accept_valid_single_initial_middle_and_final_shapes --offline'
```

Result: exit 0; 1 passed.

Invalid-row expected-Red command:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests::both_operations_reject_invalid_data_multipart_shapes_before_encoding --offline'
```

Result: expected exit 1; the single test fails because `to_ttlv_payload` currently accepts 12 invalid matrix cases (six each for Encrypt and Decrypt), including the unframed and true/true Data-omission rows. No unrelated failure occurred.

## Client compile-Red targets

All commands ran via WSL Ubuntu-26.04 with `--offline`:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_id_placeholder_execution_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib client_accepts_framed_single_part_with_data_for_encrypt_and_decrypt --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib client_rejects_framed_single_part_without_data_before_exchange --offline'
```

Each exits 1 during test-crate compilation, before runtime. The compiler reports exactly 14 expected E0599 missing API references: two typed response accessors (`encrypt`, `decrypt`) and 12 `ClientRequest::{Encrypt, Decrypt}` references across the T027, T028, and T029 contracts. No test assertion, fixture, or syntax diagnostics occur. The Red status is the planned client integration boundary, not evidence that the runtime rejection/acceptance criteria already pass.

Formatting and whitespace checks:

```powershell
cargo fmt --all --check
git diff --check
```

Both pass. No production code, dependencies, generated files, or OASIS upstream files changed.
