# C server-listener binding evidence

## Scope and approval

This delivery implements the C-only KMIPKIT-0066 listener surface on
`feature/KMIPKIT-0106-server-listener`. It retains the typed Rust listener
owner, keeps incoming `kmipkit_server_request_item_t` handles distinct from
client batch items, and provides no generic TTLV injection route. Query and
Discover Versions remain available on their existing client-to-server routes.

The owner specification is
[KMIPKIT-0066](spec.md), approved historically by merged
[PR #134](https://github.com/NeverWe1come/KMIPKit/pull/134) on
`release/1.0.0` (verified merged at `2026-11-07T15:08:18Z`). The public API
manifest declares the new capability `server_to_client_listener_c_abi` with
`parityScope: c-only` and owner `KMIPKIT-0066`.

## Red, Green, Refactor evidence

### Red

The compiled C-consumer tests were added in these distinct failing commits:

- `f0234d587` — listener API symbols were absent at link time.
- `76c5bc2de` — typed Put listener path was absent.
- `9dcecef72` — a failure response incorrectly accepted `Success` status.
- `6710d73b2` — the credential TextString secret path was absent.
- `4f0bcf5ee` — unknown managed-object tags could not be preserved.

### Green

- `7e2c4842b` — implemented the C ABI, generated API declarations, and
  compiled C consumer for listener construction, mTLS receive, typed access to
  all five S2C operations, typed responses, and Put object ownership.

### Refactor

- This refactor commit narrows the new module's lint exception to
  `too_many_arguments` (the C ABI signatures are fixed) and records callback
  and verification boundaries in this evidence file.
- `aaa8f4903` adds a failing regression test for the C-only record fields
  omitted by the export auditor.
- `2bac0464f` teaches the auditor to inventory declared C-only layouts.
- `d4fb8a4db` documents the writable-pointer safety precondition and factors
  the shared listener response-builder path to reduce duplicated validation.
- `e00ac81f1` uses the established output-slot helper for opaque output handles;
  the focused listener C-consumer suite and Clippy passed again afterward.

## Verification

- `cargo fmt --all --check` — passed.
- `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings` —
  passed.
- `cargo test -p kmipkit-ffi --all-features` — passed: 479 unit tests,
  56 C API coverage tests, and 1 operation-parity test.
- `cargo test -p kmipkit-ffi --all-features s2c_c_consumer_ -- --nocapture` —
  passed: 3 compiled C-consumer tests.
- `python3 -B tools/api_manifest/generate.py --check` — passed: 12 generated
  outputs verified.
- `python3 -m unittest tools.api_manifest.tests.test_binding_exports` — passed:
  60 tests, including the new C-only layout regression.
- `cargo build --locked -p kmipkit-ffi --features coverage-c-consumer` and
  `python3 -B tools/api_manifest/audit_binding_exports.py --repo-root .
  --c-only --c-binary target/debug/libkmipkit_ffi.so --check` — passed; all
  2,080 declarations were matched, with no missing fields or kind mismatches.
- Focused generator unit test
  `test_c_only_handle_and_versioned_record_can_be_declared_in_public_header` —
  passed.
- `git diff --check` — passed.
- The full generator unittest module remains uncompleted; the focused generator
  test and all 60 binding-export audit tests passed. The C FFI AddressSanitizer
  job passed on the preceding draft PR head; the updated PR head must rerun CI.

## Ownership and callback boundary

Incoming Put payloads remain in KMIPKit-owned zeroizing TTLV storage. C receives
typed object and nested-structure handles; secret ByteString/BigInteger fields
are copied into `kmipkit_secret_bytes_t`, and secret TextString fields into
`kmipkit_secret_text_t`. Both are accessible only through the existing
synchronous callback accessors and have redacted debug output. The C consumer
checks that these callbacks reject a NULL function pointer and that Put secret
bytes and credential text are readable during a valid callback.

The existing C secret callbacks return `void`; therefore the ABI cannot
represent a callback-reported application failure. NULL-callback rejection and
input-validation errors are testable and covered. This delivery does not change
the public callback signatures; a returnable callback error needs a separately
approved ABI contract.
