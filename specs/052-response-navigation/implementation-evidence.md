# KMIPKIT-0052 CI remediation evidence (2026-11-08)

## Asynchronous response FFI safety

- **Red:** commit `783132326` adds a source contract requiring the Cancel, Poll, Process, and Query Asynchronous Requests response exports to be unsafe Rust functions with documented safety preconditions. It failed on the original #149 source because all four declarations were safe despite writing through raw output pointers.
- **Green:** commit `2131122e4` marks the four Rust exports and their shared output initializer `unsafe`, documents live-handle, aligned writable output, non-overlap, and borrowed-span lifetime requirements, and updates Rust callers to make the unsafe boundary explicit. The C ABI symbol names, calling convention, and headers are unchanged. The Python concurrency test now catches `Exception` rather than `BaseException`.
- **Refactor:** commit `6f85173e3` adds coverage for the private initializer's own unsafe contract.
- **Verification:** `cargo +1.94 fmt --all --check`; `cargo +1.94 clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings`; `cargo +1.94 test -p kmipkit-ffi --all-features async_operation_abi_tests` (10 passed); `python -B -m unittest tools.api_manifest.tests.test_async_outcome_ffi_safety -q` (2 passed); Python 3.12 `test_async_operation_mapping.py` (29 passed); API manifest generation `--check` (12 outputs); and validation-schema generation `--check` all passed on the merged stack worktree. AWS-LC emitted non-fatal local Windows compiler-probe/NASM diagnostics.

These are local checks. The updated hosted CI and SonarCloud quality gate must still complete before the PR is marked ready.
