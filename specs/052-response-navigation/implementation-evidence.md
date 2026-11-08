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

## Changed-code coverage follow-up

- **Red:** the original hosted #149 run `42451776747` failed its selected coverage gate: changed production code was `532/606` lines (`87.79%`) against the required `95%`. The same run's script-contract failure was the stale Log requirement-scope assertion; #0050's parent fix has since been propagated to the stack.
- **Green:** added Python cases for malformed asynchronous response metadata, callback validation, absent correlation, zeroized borrowed views, missing native response owners, and native TTLV release on errors. Added Java coverage for malformed typed-response values, null callbacks, invalid header handles, and absent raw TTLV, plus a Rust empty-batch header-view case. Local Python source suite and coverage: `1119 passed, 1 skipped`, `96.46%`; Java full suite: `650 tests, 0 failures, 0 errors, 3 skipped`; focused Rust empty-response test and `cargo +1.94 fmt --all --check` passed.
- **Refactor:** extracted package-private Java validation helpers so malformed native outputs can be tested without invoking invalid native handles. Removed a redundant `ValueError` catch around `memoryview.release()`; Python 3.12 permits repeated calls, and the callback-release test verifies cleanup and secret wiping. The hosted changed-code coverage gate must still confirm the final threshold.

## Hosted coverage gate follow-up (2026-11-08)

- **Red:** the latest #149 CI run `42467658064` measured changed production coverage at `557/606` (`91.91%`), below the required `95%`. The selected platform and adapter reports were produced successfully; this is a test-coverage gap, not a skipped or unavailable workflow.
- **Green:** the C ABI consumer now exercises output clearing and overlap rejection for all four typed asynchronous response readers. The Java wrong-kind response-bridge test now includes the raw payload and response-header TTLV accessors. Python tests cover a successful Cancel result without an echoed correlation value and the typed Process response correlation forwarding path. Focused verification passed: C consumer test `generated_public_async_client_abi_consumer_compiles_and_runs` (1 passed), Java `NativeResponseHandleValidationTest` (BUILD SUCCESSFUL), and Python `test_async_operation_mapping.py` (37 passed).
- **Additional local verification:** `cargo llvm-cov --no-clean -p kmipkit-ffi --all-targets --all-features --summary-only` passed 399 FFI unit tests and 50 compiled C-consumer tests. It reported 90.63% total FFI line coverage and 88.87% for `client_abi.rs`. The full Python suite passed (1,120 passed, 1 skipped), and `cargo +1.94 fmt --all --check` passed. These local results do not replace the selected hosted three-platform coverage gate.
- **Refactor:** kept the C cases in the existing asynchronous error consumer and the Java cases in the shared response-bridge table; production behavior and coverage thresholds are unchanged. The updated hosted coverage gate must confirm the resulting changed-code line coverage before T022 can close.

## Final hosted verification and T022 closure (2026-11-08)

Hosted CI run [42487112326](https://github.com/NeverWe1come/KMIPKit/actions/runs/42487112326) completed successfully for PR #149 head `a0a8a10300139b116cc4737157f38c3d045a6394`. The selected coverage gate reported changed production at 570/600 (95.00%), TTLV at 740/746 (99.20%), protocol at 24,496/25,458 (96.22%), transport at 4,740/5,036 (94.12%), FFI at 13,360/14,538 (91.90%), Java at 5,666/5,844 (96.95%), Python at 7,338/7,603 (96.51%), JNI at 9,046/10,552 (85.73%), and workspace at 56,382/59,259 (95.15%). All met the thresholds active for this run: 95% for changed production, TTLV, and protocol; 85% for transport, FFI, and language adapters; and 90% workspace-wide.

The same run passed the full hosted matrix: Rust 1.94 and stable on Linux, Windows, and macOS; C consumers on Linux, Windows, and macOS; Java/JNI and Python integration on the supported hosted platforms; coverage producers; C FFI and JNI sanitizers; extension-schema fuzz smoke; dependency policy; CodeQL, zizmor, and SonarCloud; documentation traceability; and normative inventory/immutable-source checks. The CI summary and selected-scope coverage gate both succeeded. This closes T022; older entries above describe earlier failed or incomplete runs and are superseded by this final hosted result.
