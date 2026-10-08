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
  inputs. Certificate PEM/DER constructors call `std::fs::read` once. Private
  key PEM/DER constructors each call `File::open` once and route key bytes
  through `SecretBuffer::read_from`. Neither form retains the source path;
  open/read errors map to `InvalidCredential` without retaining OS/path error
  text. The existing in-memory constructors and explicit encoding behavior
  remain.
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
partial-read correction added `SecretBuffer::read_from<R: Read>` and routed
both private-key file constructors through that owner before attempting the
read. Its first implementation used `read_to_end`; the subsequent growth
correction below replaces that implementation with fixed scratch reads and
controlled zeroizing growth. Both PEM and DER constructors still open the
selected path once and pass that `File` through the same production helper;
file and read errors map to the fixed `InvalidCredential` category. The
observer wrapper exercises this same private reader.

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

## P2 replaced-allocation cleanup correction — Red and Green complete

Final review found that `read_to_end(&mut Vec)` can grow the key buffer after
bytes are initialized. `SecretBuffer::drop` clears only the current allocation,
so an allocation released during implicit `Vec` growth could remain uncleared.
The Green correction removes `read_to_end` from key reads. It reads through a
fixed 4 KiB scratch owner that zeroizes each consumed chunk and clears its full
array on success, error, or unwind. When the current owner lacks capacity, it
computes the new length with checked arithmetic, fallibly reserves a new
`SecretBuffer` while the old allocation remains live, copies the initialized
bytes and new chunk, zeroizes the old initialized range, synchronously passes
that now-zero borrowed range to the test-only observer, and then replaces the
old `Vec`. The temporary replacement is itself a zeroizing owner, including
reserve/error and unwind paths. Reader, overflow, and reservation errors map to
a fixed generic I/O error; the public file constructors continue mapping those
to `InvalidCredential` without a source or path.

Both private-key PEM and DER constructors use `SecretBuffer::read_from`, which
routes through this same controlled reader. The accepted test seam starts with
one byte of capacity; its reader fills that byte and then supplies the other
64 KiB, forcing replacement after initialized key data exists. The cfg(test)
observer callback examines the borrowed old range synchronously and stores
only an event count and cumulative zero-status. Its status begins false, so a
missing observer event cannot pass; the test requires at least one replacement
and verifies every observed old range was zero before release. The observer
never retains, exposes, or formats key bytes.

- Initial Red test commit: `8a79a63903c67cfb0de0d458f737cd1eff60e249`.
- One-byte-capacity Red refinement: `f49f229c876ccf7953993b79936978ba6aa77f22`.
- Borrowed-range observer Red refinement: `ce9c42f80b435f5f702433b4ab22207b3d48f136`.
- Final Red evidence commit: `4d01e8bdbabece4a3fb5907f9bc41d623cb1dd00`.
- Red command before the controlled replacement hook:
  `cargo test -p kmipkit-transport --lib secret::tests::key_buffer_growth_zeroizes_each_replaced_allocation_before_release --offline`
  exited 1 at `the read owner must report at least one replaced allocation`.
  This proves the regression stays Red when the controlled growth callback is
  omitted.
- Green source commit: `7c4ba929a53fec960f93f879f5696a6220423e3a`.
- `cargo test -p kmipkit-transport --lib secret::tests::key_buffer_growth_zeroizes_each_replaced_allocation_before_release --offline`:
  1 passed, 0 failed.
- `cargo test -p kmipkit-transport --lib secret::tests::partially_read_private_key_is_zeroized_when_reader_fails --offline`:
  1 passed, 0 failed.
- `cargo test -p kmipkit-transport --test secret_redaction_current --offline`:
  9 passed, 0 failed.
- `cargo test -p kmipkit-transport --test secret_redaction --offline`:
  22 passed, 0 failed.
- `cargo test -p kmipkit-transport --all-targets --all-features --offline`:
  183 passed, 0 failed.
- `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings`,
  `cargo fmt --all --check`, and `git diff --check`: passed.

The T021 symlink fixture used its documented fallback on this host, so actual
symlink following still needs platform verification where symlink creation
succeeds. External caller copies and key bytes transferred to rustls/AWS-LC or
held by the OS and other dependencies remain outside this zeroization claim.

## Review-driven key-content preservation assertion

Review confirmed the growth test also needed to prove that controlled copying
preserves the complete key input before it checks cleanup. Test correction
commit: `c3f55bb1782884000f5cb0c7865d70c06a5db9cc`. The test now inspects the
successful owner before drop with a boolean equality assertion against the
test sentinel, using fixed diagnostic text; after drop it retains the existing
assertions that the final buffer and every observed replaced allocation were
zeroized. The assertion never formats the sentinel.

As a mutation check, the implementation's old-buffer copy was temporarily
omitted and the focused growth test failed at `the read owner must preserve all
input bytes`. The production line was restored and the temporary mutation was
not committed. With the assertion in place, the focused growth and partial-read
tests each pass 1/1; `secret_redaction_current` passes 9/9,
`secret_redaction` passes 22/22, and all package targets/features pass 183
tests. Strict package Clippy, `cargo fmt --all --check`, and
`git diff --check` pass.
