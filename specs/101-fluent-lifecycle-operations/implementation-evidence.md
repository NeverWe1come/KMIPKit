# KMIPKIT-0101 Implementation Evidence

## Branch and dependency

- Branch: `feature/KMIPKIT-0101-fluent-lifecycle-operations`.
- Base: PR #183 head `31bd4ec1b`.
- Scope: Rust fluent builders for Locate, Activate, Archive, Recover, Revoke,
  Re-key, Re-key Key Pair, and Re-certify.

## Red

- Added `fluent_lifecycle_operation_tests.rs` with a typed request-variant
  construction case for each operation.
- `cargo test -p kmipkit-client --lib fluent_lifecycle_operation_tests --no-run`
  failed as expected: `RequestBuilder` had no `activate`, `archive`, `recover`,
  `revoke`, `locate`, `rekey`, `rekey_key_pair`, or `recertify` methods.
- No production implementation has been added yet.
