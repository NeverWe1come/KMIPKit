# T023 Refactor Report — KMIPKIT-0013

## Source refactor

Refactor commit: `f379ba7efe26e3c51b9aac9dfaa72d1e676a9218`.

In `crates/kmipkit-transport/src/secret.rs`, the fixed-size
`SecretReadScratch` owner is now scoped to one read/append iteration. Its Drop
zeroizes the entire scratch array immediately after each successful append,
and also runs on EOF, read or append errors, and unwinding. This removes the
separate explicit post-append clear and makes the scratch lifetime match the
operation that uses it.

The refactor leaves the read size, sanitized error mapping, checked length,
fallible allocation, controlled replacement allocation, copy-before-wipe
ordering, test observer timing, output bytes, and public API unchanged. The
existing partial-read and growth tests exercise error cleanup, exact-content
preservation, and replaced-allocation zeroization.

## Baseline before the source edit

- `cargo test -p kmipkit-transport --test secret_redaction_current --offline`:
  9 passed, 0 failed.
- `cargo test -p kmipkit-transport --test secret_redaction --offline`:
  22 passed, 0 failed.

## Verification after the source edit

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
- `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings`:
  passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed before the source commit.

The T021 file-symlink fixture used its documented fallback on this host; actual
symlink-following still needs verification on a platform where symlink creation
succeeds. The source refactor does not change that fixture or its behavior.
