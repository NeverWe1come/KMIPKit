# Task 22 Green report — KMIPKIT-0013

## Scope

T022 implements the approved FR-004/FR-005 secret handling and TLS
configuration contract. The T021 Red tests were committed in
`c00556281a1399dbf7281306591a39a9eb13db6b`, with the encrypted-key sentinel
correction in `da52647f728032cd2c81fc78057e550e0900f73c`, the Windows symlink
fallback correction in `0eebc75e7ea84a72b9da6fe523a6016e57b10bb3`, and its
evidence in `f70bec03d674b07358ea663c6477fa80ad7d8763`.

## Implementation

- Added private `SecretBuffer`, which zeroizes initialized bytes already under
  its ownership before its `Vec` allocation is released. A private
  `SecretPrivateKeyDer` guard zeroizes PEM parser output when parsing fails or
  the input is ambiguous. The test-only observer reports initialized length
  and zeroized status only; it is crate-internal and compiled only under
  `cfg(test)`.
- Added explicit PEM and DER file constructors for certificate and private-key
  inputs. Each constructor calls `std::fs::read` once, keeps no source path,
  and maps read errors to `InvalidCredential` without retaining OS error text.
  The existing in-memory constructors and explicit encoding behavior remain.
- Added `# Errors` rustdoc to all four constructors. Existing configuration and
  transport errors remain fixed-category and payload-free. No logger or new
  dependency was introduced.
- Wired the private module into the crate. The source-including test targets
  now include it where needed. Test-only dead-code allowances are limited to
  those path-included modules and observer seams; the workspace lint policy is
  unchanged. Existing test documentation and formatting were adjusted to keep
  strict Clippy clean.

## Verification

- `cargo test -p kmipkit-transport --test secret_redaction_current --offline`:
  9 passed, 0 failed.
- `cargo test -p kmipkit-transport --test secret_redaction --offline`:
  20 passed, 0 failed.
- `cargo test -p kmipkit-transport --all-targets --all-features --offline`:
  177 passed, 0 failed.
- `cargo clippy -p kmipkit-transport --all-targets --all-features --offline
  -- -D warnings`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.

## Commit and limitations

Green implementation commit: `2ae106da79a38d9e1f2d71d998774c4c0ade36f2`.
The follow-up evidence commit records the exact results in
`specs/013-production-transport/tasks.md`.

Zeroization covers initialized bytes in KMIPKit-owned input and guarded parser
buffers. Caller copies, the key material transferred to rustls/AWS-LC, and
copies held by the OS or other dependencies remain outside this guarantee. The
Windows T021 fixture used its symlink-unavailable fallback on this host, so
Windows symlink-following itself still needs verification where file symlink
creation succeeds. Workspace-wide tests, coverage, and cross-platform CI were
not part of this task's package-scoped verification.

## P2 partial-read correction — Red and Green complete

Review identified that the original `std::fs::read` constructors only create
`SecretBuffer` after a successful read. If a read fails after yielding bytes,
the partially initialized `Vec` is released before it enters the zeroizing
owner, so the original Green evidence did not cover this error path. The
correction adds `SecretBuffer::read_from<R: Read>` and a shared private reader
that constructs the zeroizing owner before calling `read_to_end`. Its `?`
error path drops that owner, whose destructor zeros the initialized range.
Both private-key PEM and DER file constructors open the selected path once and
pass that `File` through the production helper; file and read errors still map
to the fixed `InvalidCredential` category. The test-only observer wrapper uses
the same private reader, so the regression exercises the owner used by the
production constructors.

- Red test commit: `991796eb30259ace813e47ea2f07369562869911`.
- The `#[cfg(test)]` regression uses an injected reader that yields a private
  key sentinel and then returns an I/O error. It asserts the observer saw the
  complete initialized length and that those bytes were zeroized before
  release. The sentinel is never included in assertion or error text.
- Expected Red: `cargo test -p kmipkit-transport --lib secret::tests::partially_read_private_key_is_zeroized_when_reader_fails --offline`
  exited 1 at compile time with E0599 for the intentionally absent observer
  wrapper. No production code changed in the Red commit.
- Green correction commit: `a10214ebfc484623805f596ddfa4cf0080934bd7`.
- `cargo test -p kmipkit-transport --lib secret::tests::partially_read_private_key_is_zeroized_when_reader_fails --offline`:
  1 passed, 0 failed.
- `cargo test -p kmipkit-transport --test secret_redaction_current --offline`:
  9 passed, 0 failed.
- `cargo test -p kmipkit-transport --test secret_redaction --offline`:
  21 passed, 0 failed.
- `cargo test -p kmipkit-transport --all-targets --all-features --offline`:
  180 passed, 0 failed.
- Strict package Clippy, `cargo fmt --all --check`, and `git diff --check`:
  passed.

## P2 replaced-allocation cleanup correction — Red, pending review

Final review found that `read_to_end(&mut Vec)` can grow the key buffer after
bytes are initialized. `SecretBuffer::drop` clears only the current allocation,
so any allocation released during implicit `Vec` growth is outside that
cleanup. The new regression extends only the `cfg(test)` observer with a
replacement count and cumulative zero-status; it never exposes buffer bytes.
Its reader yields the first byte separately, then the remaining 64 KiB input,
and the test asserts that at least one replaced allocation was observed and
that every replacement was zeroized before release.

- Red test commit: `8a79a63903c67cfb0de0d458f737cd1eff60e249`.
- Expected Red:
  `cargo test -p kmipkit-transport --lib secret::tests::key_buffer_growth_zeroizes_each_replaced_allocation_before_release --offline`
  exits 1 at the runtime assertion that the read owner must report a replaced
  allocation. The current `read_to_end` path has no per-replacement observer
  or zeroization step. No production code changed in this Red commit.
- `rustfmt --edition 2024 --check crates/kmipkit-transport/src/secret.rs` and
  `git diff --check` passed.
- Green correction and full verification remain pending review of this Red
  test. T022 is incomplete until production reads use explicit zeroizing
  growth and this regression passes.
