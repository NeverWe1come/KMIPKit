# KMIPKIT-0052 CI remediation evidence (2026-11-08)

## Asynchronous response FFI safety

- **Red:** commit `783132326` adds a source contract requiring the Cancel, Poll, Process, and Query Asynchronous Requests response exports to be unsafe Rust functions with documented safety preconditions. It failed on the original #149 source because all four declarations were safe despite writing through raw output pointers.
- **Green:** commit `2131122e4` marks the four Rust exports and their shared output initializer `unsafe`, documents live-handle, aligned writable output, non-overlap, and borrowed-span lifetime requirements, and updates Rust callers to make the unsafe boundary explicit. The C ABI symbol names, calling convention, and headers are unchanged. The Python concurrency test catches `Exception` rather than `BaseException` and checks worker results from the main test thread.
- **Refactor:** commit `6f85173e3` adds coverage for the private initializer's own unsafe contract.
- **Verification:** `cargo +1.94 fmt --all --check`; `cargo +1.94 clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings`; `cargo +1.94 test -p kmipkit-ffi --all-features async_operation_abi_tests` (10 passed); `python -B -m unittest tools.api_manifest.tests.test_async_outcome_ffi_safety -q` (2 passed); Python 3.12 `test_async_operation_mapping.py` (29 passed); API manifest generation `--check` (12 outputs); and validation-schema generation `--check` all passed on the merged stack worktree. AWS-LC emitted non-fatal local Windows compiler-probe/NASM diagnostics.

These are local checks. The updated hosted CI and SonarCloud quality gate must still complete before the PR is marked ready.

## SonarCloud assertion handling

- **Red:** commit `5bc874b35` records and checks the worker's boolean result in the test thread. It failed because the worker only asserted inside a handler that caught `Exception`, so the assertion failure was swallowed into the captured error list instead of producing an observable result.
- **Green:** commit `89670b835` stores the return value outside the `try` and appends it only in the `else` branch; the test thread then asserts success. All 29 tests in `test_async_operation_mapping.py` passed.
- **Refactor:** the follow-up renames the captured collections to `read_results` and `read_errors` and records this evidence. The focused file suite passed again (29 tests).
- **Sonar finding:** `bindings/python/tests/test_async_operation_mapping.py:596-599` no longer contains an assertion inside a `try` whose `except Exception` also catches `AssertionError`.

## Python response test follow-ups

- **Red:** commit `9ecca3fc2` corrected the response-extension test to attach its fake item through `_native_response_item` and made the delegated-login test use the real `BatchResult` owner wrapper. The three focused checks then had one expected failure: the four asynchronous response constructors still lacked parameter annotations.
- **Green:** commit `183b7773f` annotates the three typed response parameters and the Query Asynchronous Requests parameter as `Any`. This makes their public signatures resolvable without asserting an inaccurate concrete CFFI type. The focused checks passed (3 tests).
- **Refactor:** corrected ownership test doubles now exercise the real batch-result cleanup path, while item handles remain controlled test doubles. The full Python suite passed (1,110 passed, 1 skipped).

## Java mutual-TLS response-navigation CI follow-up

- **Red:** CI run `42448040610` failed the mixed-outcome mutual-TLS test because it searched for the Get response sentinel under `Key Value` (`0x420045`). Commit `76f85255b` corrects the expected tag to `Key Material` (`0x420043`); the focused test then exposes a second failure at `itemAt(0)`, since the traversal skips the enclosing `Any Object` and `Managed Object` structures.
- **Green:** commit `eca604a9d` follows `Response Payload → Any Object → Managed Object → Key Block`, verifies the Key Block's ordered `Key Format Type`, `Key Value`, and vendor-extension children, and reads the nested vendor integer. The focused Maven test passed on Windows: `mvn -ntp -Dtest=ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange test` (1 test, BUILD SUCCESS).
- **Refactor:** the test closes each borrowed native view at its owning scope and checks both the nested response value and vendor TTLV path. The correction changes test navigation only; protocol behavior is unchanged. The updated hosted CI still needs to complete.
