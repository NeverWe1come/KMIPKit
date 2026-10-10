# Task 28 — ID Placeholder Client Red Contracts

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

## Scope

Added T028 fake-transport client contract tests for Encrypt and Decrypt ID Placeholder eligibility. The tests are derived from OASIS KMIP v2.1 §6.1 (batch-scoped ID Placeholder producers and consumers) and §9.8 (Batch Order Option). They do not modify shared batch semantics or production code.

The tests require a payload with omitted Unique Identifier to be rejected locally with the exact sanitized `Validation(InvalidInput, NotSent)` error when the request is standalone, follows Ping (which cannot establish the placeholder), or is after Create while Batch Order Option is false. They also assert that Create followed by an Encrypt with omitted Unique Identifier is encoded as two ordered items when Batch Order Option is true, preserving the omitted field on the Encrypt payload. A two-item response test returns distinct server failures for Create and Encrypt in reverse response order, then checks that both exact per-item results remain associated with their request IDs after the producer fails. It sets Batch Error Continuation to Continue so the fixture represents an attempted later item.

The tests intentionally do not predict whether a prior producer succeeds, resolve identifiers locally, or synthesize Unique Identifier values. The existing typed client does not expose Encrypt/Decrypt request variants or typed response accessors until T038, so these assertions remain compile-Red until client integration.

## Red evidence

Focused command (WSL Ubuntu-26.04):

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_id_placeholder_execution_tests --offline'
```

Result: expected compile failure, exit code 1, with exactly ten E0599 diagnostics and no unrelated type/syntax/fixture errors. Eight are the already-recorded T027 references to missing `ClientRequest::{Encrypt,Decrypt}` variants and `ClientResponseView::{encrypt,decrypt}` accessors; two are the new T028 Encrypt/Decrypt request variant references. Unit tests do not run before T038 supplies those typed client APIs.

Formatting and whitespace checks:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo fmt --all --check'
git diff --check
```

Both checks pass. No production code, dependencies, generated files, or OASIS upstream files changed.
