# Task 22 Green report — KMIPKIT-0013

## Scope

T022 implements the approved FR-004/FR-005 secret handling and TLS
configuration contract. The T021 Red tests were committed in
`c00556281a1399dbf7281306591a39a9eb13db6b`, with the encrypted-key sentinel
correction in `da52647f728032cd2c81fc78057e550e0900f73c`, the Windows symlink
fallback correction in `0eebc75e7ea84a72b9da6fe523a6016e57b10bb3`, and its
evidence in `f70bec03d674b07358ea663c6477fa80ad7d8763`.

## Implementation

- Added private `SecretBuffer`, which owns private-key input and zeroizes its
  initialized bytes before its `Vec` allocation is released. A private
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
