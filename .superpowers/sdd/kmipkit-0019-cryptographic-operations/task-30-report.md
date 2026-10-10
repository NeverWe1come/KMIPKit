# Task 30 — Encrypt/Decrypt Data and Multipart Validation

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

## Scope

Implemented the shared private `validate_data_multipart_shape` gate for Encrypt and Decrypt protocol request payload conversion. It enforces required Data for unframed single-part and middle requests, optional Data for initial and final multipart requests, correlation absence on initial and framed one-request shapes, correlation presence on final requests, and Data presence for the framed Init=true/Final=true single request while KMIPKIT-DISC-045 remains open. Failures use the existing payload-free `InvalidValue` protocol error. The per-operation request validators still perform their own accepted-field, type, and singleton checks before invoking this shared matrix.

Two existing all-fields request fixtures used the invalid combination Init=true plus Correlation Value. They now use the valid middle shape Init=false/Final=false with Correlation Value, preserving their field-order assertions. T030 covers only protocol pre-encoding Data, indicator, and Correlation Value validation through `to_ttlv_payload`. The one-exchange-per-invocation property remains in FR-008 and is not evidenced by this protocol task; T034/T038 own its fake-client verification. The T027/T029 client tests are compile-Red contracts and do not count as T030 client evidence. No client variants, production dependencies, generated output, or OASIS sources changed.

## Red evidence

The pre-implementation focused valid case passed (1/1), while the invalid case failed as expected with all 12 invalid cases accepted, six each for Encrypt and Decrypt:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests::both_operations_accept_valid_single_initial_middle_and_final_shapes --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests::both_operations_reject_invalid_data_multipart_shapes_before_encoding --offline'
```

The earlier T026 Red commit and review evidence remain in `.superpowers/sdd/kmipkit-0019-cryptographic-operations/task-26-report.md`.

### QA fix round 1 — Red evidence

Added invalid `to_ttlv_payload` cases for explicit `Init=false`, explicit `Final=false`, and both indicators explicitly false, each with Data but without Correlation Value. The existing valid case with both indicators absent and no Correlation Value remains valid. The focused valid case passed (1/1); the focused invalid case failed as expected and listed all six newly accepted invalid shapes (three per operation).

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests::both_operations_accept_valid_single_initial_middle_and_final_shapes --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests::both_operations_reject_invalid_data_multipart_shapes_before_encoding --offline'
```

The invalid-case failure is expected Red evidence: Encrypt and Decrypt each accepted explicit false-indicator middle shapes with Data and no Correlation Value. No production code changed in this Red phase.

### QA fix round 1 — Green evidence

The shared protocol validator now rejects explicit `Init=false` and/or `Final=false` middle shapes without Correlation Value, even when Data is present. It still accepts the all-indicators-absent, no-correlation unframed single-part shape; explicit middle shapes with Correlation Value and Data remain valid; framed `Init=true/Final=true` without Correlation Value remains valid only when Data is present under the open DISC-045 ruling. The focused matrix passes 2/2. Encrypt requests pass 4/4; Decrypt requests 4/4; malformed crypto payload tests 14/14; Table 59 parameter-structure tests 5/5. Protocol Clippy, formatting, and whitespace checks pass.

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib encrypt_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib decrypt_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib malformed_crypto_payload_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib encrypt_parameter_structure_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings'
cargo fmt --all --check
git diff --check
```

The client exchange-count assertion remains pending the fake-transport execution work in T034/T038 under FR-008; T030 adds no client dispatch behavior.

## Green verification

All targeted commands ran via WSL Ubuntu-26.04 with `--offline`:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib encrypt_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib decrypt_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib malformed_crypto_payload_tests --offline'
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib encrypt_parameter_structure_tests --offline'
```

Results: multipart matrix 2 passed; Encrypt request suite 4 passed; Decrypt request suite 4 passed; malformed crypto payload suite 14 passed; Table 59 request validation suite 5 passed. No failures.

Formatting and whitespace checks:

```powershell
cargo fmt --all --check
git diff --check
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings'
```

All pass. The full workspace suite was not run.
