# Task 27 — Multipart Client Red Contract Tests

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

## Scope

Added test-only fake-transport contract tests for Encrypt and Decrypt. Each scenario queues three server responses and makes three separate `Client::execute` calls: an initial request, a middle request, and a final request. The first success response carries an opaque Correlation Value containing NUL and non-UTF-8 bytes. The caller reads it from the typed response and supplies those exact bytes to both later request models. Captured TTLV assertions verify exact correlation bytes on both requests, AAD on only the initial Encrypt and Decrypt requests, and Decrypt Authenticated Encryption Tag on only the initial Decrypt request. Request counts advance once per explicit call; there is no automatic follow-up or hidden stream state.

The tests target the existing public batch pattern with the planned Encrypt/Decrypt variants and typed response accessors:

```text
ClientRequest::{Encrypt, Decrypt}
ClientBatchItem::new
Client::execute
response.get(0).outcome().response().{encrypt,decrypt}().correlation_value()
```

The protocol request/response models already exist, but the client integration is intentionally scheduled for T038. Accordingly, T027's Red target is compile-time until those planned client variants and outcome accessors are added; no production or test-only client API was introduced. The focused target failed with exactly eight E0599 errors: six `ClientRequest::{Encrypt,Decrypt}` uses and the two absent `ClientResponseView::{encrypt,decrypt}` accessors. No unrelated helper, type, or assertion errors were reported.

## Security review correction — Red

Independent security review identified that the test helper copied response Correlation Value bytes into ordinary `Vec<u8>`, whose contents were not zeroized when the test owner dropped. Added explicit `Zeroizing<Vec<u8>>` type expectations for both Encrypt and Decrypt captured owners and converted only at the `SecretBytes::new` boundary in request construction. Before Green, the focused target failed with the expected two E0308 mismatches (`response_correlation` returned `Vec<u8>` instead of `Zeroizing<Vec<u8>>`) plus the eight planned E0599 missing client API errors. No other diagnostics were reported.

Focused correction Red command (WSL Ubuntu-26.04):

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline'
```

Result: expected compile failure, exit code 1, exactly two E0308 secret-owner type mismatches and eight E0599 planned missing client API diagnostics.

## Security review correction — Green and Refactor

`response_correlation` now returns `Zeroizing<Vec<u8>>`, wrapping the copied bytes read from `SecretBytes`. Both Encrypt and Decrypt tests retain that zeroizing owner while comparing and reusing the server value. A plain `Vec<u8>` is created only at the `SecretBytes::new` boundary when constructing each subsequent request. The helper comment records this ownership invariant. No production code, client API, protocol behavior, or dependency changed.

Re-ran the focused correction target after Green:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline'
```

Result: expected compile failure, exit code 1, now only the same eight E0599 missing `ClientRequest::{Encrypt,Decrypt}` variants and `ClientResponseView::{encrypt,decrypt}` accessors planned for T038. Both zeroizing-owner E0308 errors are resolved; there are no unrelated diagnostics. Runtime assertions remain pending T038.

`cargo fmt --all --check` and `git diff --check` both pass after Green and Refactor.

Focused Red command (run through WSL Ubuntu-26.04):

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-client --lib encrypt_decrypt_multipart_execution_tests --offline'
```

Result: expected compile failure, exit code 1, eight missing planned client API symbols as described above. Runtime assertions are therefore not yet executable and remain a Green verification requirement after T038.

Formatting and whitespace checks passed:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo fmt --all --check'
git diff --check
```

No production code, dependencies, generated files, or OASIS upstream sources changed.

## T026 evidence carried forward

T026 is checked in `specs/019-cryptographic-operations/tasks.md`. Commit `ed83e39` added the Red matrix; DCO-signed correction `444470c` removed the indistinguishable unframed/middle negative case and recorded why caller continuation is represented by the server-issued Correlation Value. Its focused command was:

```powershell
wsl.exe -d Ubuntu-26.04 -- sh -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0019-encrypt-decrypt/KMIPKit && cargo test -p kmipkit-protocol --lib multipart_validation_tests --offline'
```

After correction, the valid matrix test passed and the invalid matrix test failed as expected because the current builders accepted the 12 remaining invalid cases. `cargo fmt --all --check` and `git diff --check` passed. The same QA reviewer returned Clear for `444470c`.
