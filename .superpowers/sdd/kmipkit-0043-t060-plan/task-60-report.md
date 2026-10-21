# T060 implementation report

Date: 2026-10-21

Scope: KMIPKIT-0043 T060, Re-key Key Pair mapping

## Status

The local implementation and verification work is complete. T060 remains
unchecked in `specs/043-public-api-bindings/tasks.md` because controller review,
exact-head cross-platform CI, and the selected-scope coverage gates are still
required. No OASIS conformance, interoperability, profile-support, or
certification claim is made.

## Commits

All three development commits include the DCO `Signed-off-by` trailer:

| Stage | Commit | Subject |
|---|---|---|
| Red | `4e30bb787ef100a90392c6309de1c8aa25e38195` | `test: define Re-key Key Pair manifest mapping` |
| Green | `387a52233cd31bc541af8a93e30bb90a48196116` | `feat: map Re-key Key Pair across bindings` |
| Refactor | This report commit | `docs: record Re-key Key Pair parity evidence` |

## Red evidence

Command:

```powershell
python -B -m unittest tools.api_manifest.tests.test_rekey_key_pair_parity_manifest
```

The two initial manifest tests failed as intended against the blocked,
undeclared operation:

```text
FE
FAIL: test_operation_is_gated_on_approved_four_language_mapping
AssertionError: 'verified' != 'blocked'
- verified
+ blocked

ERROR: test_pair_response_retains_both_identifiers_as_public_api
KeyError: 'declarations'

Ran 2 tests
FAILED (failures=1, errors=1)
```

This established the missing four-language operation declaration and pair
response API mapping before implementation.

## Green implementation

The existing canonical Rust `RekeyKeyPairRequest`, response, and client path
remain authoritative. The change adds the C builder and success-only response
accessors, Java 17/JNI builder and typed response, and Python 3.12/CFFI builder
and response model. All eight optional Table 310 inputs are exposed. Success
returns both required replacement identifiers and complete ordered response
TTLV, including unknown fields. Failure and Pending retain common result,
delivery, and exact correlation metadata without success-only data or raw
error-body access. Missing request identifiers pass through without local
substitution. There is no local key generation, retry, poll, or follow-up.

The checked-in API manifest records the operation and public declarations; all
generated declarations and fixtures were regenerated with the pinned tool.
The manifest records the input Structure copy semantics, borrowed lifetime of
C response identifier spans, caller-owned response TTLV copies, mapping
requirement IDs, and Java/Python parity tests. The bilingual guides, traceability
matrix, dependency matrix, task list, and evidence file were updated.

The full Python suite initially exposed an accidental adjacent Re-key builder
symbol insertion and an outdated positional-field assertion. Both were fixed
before the Green commit. A full Rust test run then exposed a hard-coded C export
count; it was advanced from 253 to 256 for the three new exports. The corrected
full suites pass below.

## Verification

All commands were run from the repository root unless a working directory is
shown.

| Command | Result |
|---|---|
| `cargo test --locked -p kmipkit-client rekey_key_pair` | PASS: 6 canonical client tests. |
| `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --lib ffi_rekey_key_pair_` | PASS: 2 response ABI tests, including Failure output clearing and both identifiers. |
| `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_compiles_and_runs -- --exact` | PASS: compiled public C consumer. |
| `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ClientFacadeTlsIntegrationTest#rekeyKeyPairResponseUsesNativeJniBridgeAndPreservesBothIdentifiers,RekeyKeyPairMappingTest' test` | PASS: 6 focused Java tests (5 mapping, 1 native TLS/JNI integration). |
| `mvn --batch-mode test` (from `bindings/java`) | PASS: 384 tests, 0 failures/errors, 3 skipped. |
| `python -B -m unittest tools.api_manifest.tests.test_rekey_key_pair_parity_manifest` | PASS: 4 manifest contract tests. |
| `python -B -m unittest tools.api_manifest.tests.test_t030_package_docs` | PASS: 7 bilingual package-guide contract tests. |
| `python -B tools/api_manifest/generate.py --write` then `python -B tools/api_manifest/generate.py --check` | PASS: 12 generated outputs written and verified current. |
| `python -B tools/api_manifest/audit_binding_exports.py --check` | PASS: exact public inventory matches C 367/367, Java 679/679, Python 651/651; no missing or mismatched exports. |
| Python full suite (command below) | PASS: 660 tests. |
| `cargo test --workspace --all-features` | PASS: all workspace unit, integration, and documentation tests; 3 local Cosmian integration tests were ignored because they require a local server. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS. |
| `cargo fmt --all --check` | PASS. |
| Scoped Ruff checks on new tests, syntax checks on changed Python source, Ruff formatting checks on the two new test modules, and `python -B -m compileall -q src\kmipkit` | PASS. |
| `git diff --check` and `git diff --cached --check` | PASS; no whitespace errors. |

Python full-suite command, run from `bindings/python` with the existing
candidate Python 3.12 environment:

```powershell
$nativeCopy = Join-Path $env:TEMP ("kmipkit-ffi-t060-" + [guid]::NewGuid().ToString("N") + ".dll")
Copy-Item -LiteralPath ..\..\target\debug\kmipkit_ffi.dll -Destination $nativeCopy
$env:KMIPKIT_NATIVE_LIBRARY = $nativeCopy
$testExitCode = 0
try {
    .\.venv-candidate-win\Scripts\python.exe -B -m pytest tests -q -p no:cacheprovider
    $testExitCode = $LASTEXITCODE
} finally {
    Remove-Item -LiteralPath $nativeCopy
}
exit $testExitCode
```

The temporary DLL copy is necessary on Windows because the full-suite wheel
test rebuilds the Cargo target DLL, which cannot be replaced while the same
process has it loaded. The unique temporary copy was removed after the run.
That run reported `660 passed in 19.79s`.

The scoped Python lint commands were:

```powershell
uv run --no-project --offline --python 3.12 --with ruff ruff check tests/test_rekey_key_pair_native_integration.py tests/test_rekey_key_pair_operation.py
uv run --no-project --offline --python 3.12 --with ruff ruff check --select E4,E7,E9 src/kmipkit/client.py src/kmipkit/extensions.py
uv run --no-project --offline --python 3.12 --with ruff ruff format --check tests/test_rekey_key_pair_native_integration.py tests/test_rekey_key_pair_operation.py
```

## Changed files

- C ABI and consumer: `bindings/c/include/kmipkit.h`, `bindings/c/tests/client_abi_consumer.c`.
- Java/JNI: `bindings/java/native/kmipkit_jni.cpp`, `bindings/java/src/main/java/org/kmipkit/NativeExtensionRegistry.java`, `bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItem.java`, `bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItemResult.java`, `bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java`, and `bindings/java/src/test/java/org/kmipkit/extensions/RekeyKeyPairMappingTest.java`.
- Python/CFFI: `bindings/python/src/kmipkit/_client_ffi.py`, `bindings/python/src/kmipkit/client.py`, `bindings/python/src/kmipkit/extensions.py`, `bindings/python/tests/test_query_ping_operation.py`, `bindings/python/tests/test_rekey_key_pair_operation.py`, and `bindings/python/tests/test_rekey_key_pair_native_integration.py`.
- Rust FFI and tests: `crates/kmipkit-ffi/src/extension_registry.rs`, `crates/kmipkit-ffi/src/extension_registry/client_abi.rs`, `crates/kmipkit-ffi/tests/support/client_abi_tests.rs`, and `crates/kmipkit-ffi/tests/support/metadata_tests.rs`.
- Manifest and generated artifacts: `specification/api/public-api.json`, `crates/kmipkit-ffi/src/public_api_generated.rs`, `bindings/java/src/main/java/org/kmipkit/generated/PublicApi.java`, `bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java`, `bindings/python/src/kmipkit/_generated/public_api.py`, `tests/fixtures/api/generated/kmipkit_internal.h`, and `tests/fixtures/api/generated/public_api_parity.json`.
- Manifest contract: `tools/api_manifest/tests/test_rekey_key_pair_parity_manifest.py` and `tools/api_manifest/tests/test_t030_package_docs.py`.
- Documentation and traceability: `docs/user-guide/en/language-packages.md`, `docs/user-guide/es/paquetes-de-idiomas.md`, `specs/043-public-api-bindings/traceability.md`, `specs/043-public-api-bindings/contracts/operation-dependencies.md`, `specs/043-public-api-bindings/tasks.md`, and `specs/043-public-api-bindings/implementation-evidence.md`.

## Self-review and concerns

- Rust remains the canonical operation implementation; no KMIP request or
  protocol behavior was changed.
- C copies all optional input Structures before returning. C text output spans
  remain borrowed from the response item; the generic TTLV getter returns an
  independently owned structure. Java/Python wrappers close their owned TTLV.
- Success-only data is gated on successful result status. Failure and Pending
  preserve shared result metadata without pair identifiers or response TTLV.
- Generated files were produced by the pinned generator, not hand-edited. The
  binding export audit is exact for all three adapter inventories.
- The broad, unconfigured Ruff default check is not a clean project-wide gate;
  the repository has no Ruff configuration and legacy findings remain outside
  this task. The focused T060 lint and format checks above pass.
- Builds emitted existing AWS-LC warnings because NASM is unavailable and its
  optional `stdalign_check.c` probe failed; builds completed. The compiled C
  target also emitted linker LNK4217 warnings without test failures.
- Exact-head cross-platform CI, selected-scope coverage, and controller review
  remain pending. T060 is deliberately still unchecked.
- No direct official OASIS test-case ID for §6.1.47 was identified; table-based
  parity tests are not represented as official conformance vectors.

## Independent QA parity follow-up

This follow-up addresses two test-evidence gaps identified by independent QA.
It changes tests and their helper only; it does not change normative requirements,
generated output, the API manifest, or production behavior. The T060 task remains
unchecked pending controller review, security review, exact-head CI, and coverage.

### Red

- Commit: `2616a47e89f09e609598957296311e9b6438498b`
  (`test: expose empty Re-key Key Pair mapping fixtures`), DCO signed.
- Java focused command:
  `mvn -f bindings/java/pom.xml -Dtest=RekeyKeyPairMappingTest test`
  failed as expected: 5 tests, 1 failure. The new test
  `requestBuilderCopiesAllSixOptionalStructures` reported
  `expected: <1> but was: <0>` at line 50, showing the six optional structures
  were not yet asserted at their individual JNI argument positions.
- Python focused command from `bindings/python` using the candidate Python 3.12
  environment and a temporary copy of `target/debug/kmipkit_ffi.dll`:
  `python.exe -B -m pytest tests/test_rekey_key_pair_operation.py -q -p no:cacheprovider`
  failed as expected: `1 failed, 2 passed`; the new
  `test_pair_request_passes_each_attribute_and_mask_group` reported
  `assert 0 == 1` for the not-yet-implemented slot assertion.

### Green

- Commit: `821618efb2f9b5c63aee687260650cd03af43766`
  (`test: verify Re-key Key Pair TTLV parity`), DCO signed.
- Java mapping and loopback TLS/JNI verification:
  `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=RekeyKeyPairMappingTest,ClientFacadeTlsIntegrationTest#rekeyKeyPairResponseUsesNativeJniBridgeAndPreservesBothIdentifiers' test`
  passed: 6 tests, 0 failures, 0 errors, 0 skipped. The loopback test inspects
  encoded request TTLV, asserting the six optional field tags and order, the
  86,400-second offset, each distinct nested marker, and absence of a locally
  substituted Private Key Unique Identifier.
- Python focused operation and native loopback tests:
  `python.exe -B -m pytest tests/test_rekey_key_pair_operation.py tests/test_rekey_key_pair_native_integration.py -q -p no:cacheprovider`
  passed: `4 passed`. The encoded request asserts the same field order,
  offset, and six distinct nested marker values.
- C ABI tests:
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --lib ffi_rekey_key_pair_`
  passed: 2 tests. The compiled public C consumer command
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_compiles_and_runs -- --exact`
  passed: 1 test. The consumer checks response field order, unknown tag
  `0x540001` and value `DE AD 00 FF`, and accesses the independent TTLV copy
  after releasing its source result.
- Python native loopback verifies response and payload field order, unknown tag
  `0x541234` and value `01 00 FF`, and that the copied TTLV remains usable after
  closing/freeing the source `ClientBatchResult`.
- At this stage of the parity follow-up, no C outgoing-request TTLV assertion
  had been added. The final QA residual follow-up below closes that gap using
  the existing C consumer and Rust mTLS peer, without changing the public ABI.

### Refactor and final checks

- The test-only Python TTLV parser now returns a named `_TtlvSpan` structure
  instead of a positional tuple, so the parity assertions identify tags, types,
  value spans, and item ends by name.
- `cargo fmt --all --check`: PASS.
- Focused Ruff check of the two changed Python test modules: PASS.
- Focused Ruff format check of the two changed Python test modules: PASS.
- The focused Java, Python, C ABI, and compiled C consumer tests listed above
  passed on the final test logic. Python reported `4 passed in 1.21s` after the
  parser refactor. `git diff --check`: PASS.
- Python tests were invoked with the existing candidate Python 3.12 environment
  and a temporary native DLL copy set via `KMIPKIT_NATIVE_LIBRARY`; the copy was
  removed afterward. `uv run` could not clean a stale `.venv\\lib64` symlink,
  so direct use of the already provisioned candidate interpreter avoided that
  environment cleanup issue. Builds emitted non-fatal existing AWS-LC/NASM and
  C linker warnings.

### Follow-up files and review

- Tests updated: `bindings/c/tests/client_abi_consumer.c`,
  `bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java`,
  `bindings/java/src/test/java/org/kmipkit/extensions/RekeyKeyPairMappingTest.java`,
  `bindings/python/tests/test_rekey_key_pair_operation.py`, and
  `bindings/python/tests/test_rekey_key_pair_native_integration.py`.
- Refactor evidence helper: `bindings/python/tests/test_rekey_key_pair_native_integration.py`.
- This report records the Red and Green follow-up commits; its final update is
  included in the separate DCO-signed Refactor commit.
- Self-review found no production-path, ABI, wire-contract, manifest, generated
  output, or normative-source change in this follow-up. The C request-capture
  gap noted above is closed by the final QA residual follow-up below. Controller
  review, security review, exact-head CI, and coverage checks remain pending.

## Final QA residual C request-capture follow-up

This follow-up closes the remaining request-side C parity gap with a test-only
mode in the compiled public C consumer and an isolated Rust mTLS loopback peer.
The C API constructs the request; the peer captures and decodes the transmitted
TTLV bytes. No production API, ABI, manifest, generated output, or normative
source was changed. T060 remains unchecked pending controller review, security
review, exact-head CI, and coverage.

### Red

- Commit: `9f432e839aece1ff2ce358044964fb079fc595fd`
  (`test: require C Re-key Key Pair request mapping`), DCO signed.
- Command:
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls -- --exact`
- Result: expected failure. The mTLS peer panicked with
  `C ABI consumer must transmit the Re-key Key Pair fields over mTLS`; the
  compiled consumer exited with status 9 (category 3, cause 5, delivery 2),
  one server exchange was observed, and the test assertion reported
  `left: 109, right: 0`. This confirmed the compiled C request had not yet been
  checked for the Re-key Key Pair operation.

### Green

- Commit: `ad60bde9e7680946029332ede71cce2c3b123680`
  (`test: verify C Re-key Key Pair request TTLV`), DCO signed.
- The C consumer test constructs all six optional structures with unique,
  non-empty marker tags `0x541101` through `0x541106` and values
  `common-attributes`, `private-attributes`, `public-attributes`,
  `common-masks`, `private-masks`, and `public-masks`. It calls the C
  Re-key Key Pair builder with private identifier `existing-private-key`,
  offset `86400`, and batch item ID `pair-c`.
- The Rust mTLS peer reads the actual request frame and asserts the unique
  Re-key operation, request payload order, identifier tag `0x420066`, offset
  tag `0x420058`, and optional field tags in KMIP Table 310 order:
  `0x420126`, `0x420127`, `0x420128`, `0x420163`, `0x420164`, `0x420165`.
  It checks each field's nested marker tag and exact byte-string value. The
  response exercises the public C result accessor and checks both replacement
  identifiers.
- Exact dedicated command, rerun after the Refactor cleanup:
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_c_consumer_maps_rekey_key_pair_request_over_mtls -- --exact`
  passed: `1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out`.
- Full focused compiled C consumer harness, rerun after the Refactor cleanup:
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage`
  passed: `11 passed; 0 failed; 0 ignored`.

### Refactor and final checks

- The test-only mode now takes only the TLS credential paths, port, and mode;
  it skips parsing unrelated batch fixture arguments. This keeps the dedicated
  C mTLS path isolated while preserving the original fixture-backed mode.
- Refactor commit: this report update, argv simplification, and check evidence
  are included in the separate DCO-signed Refactor commit.
- `cargo clippy --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage -- -D warnings`:
  PASS, exit 0.
- `cargo fmt --all --check`: PASS, exit 0.
- `git diff --check`: PASS, exit 0 before staging the Refactor changes.
- The focused C runs emitted existing AWS-LC/NASM and `stdalign_check.c`
  build warnings, plus non-fatal MSVC linker LNK4217 warnings. The test targets
  completed successfully.
- Self-review: all changes are confined to C/Rust test harnesses and this
  report. The six request fields are checked by wire tag, order, nested marker,
  and value; the server peer decodes bytes received from the C API over mTLS.
  Unrelated untracked local coverage artifacts were left untouched. No
  production source, public API/ABI, manifest, generated file, or OASIS source
  was edited.
- Remaining limits: security review, controller review, exact-head CI, and
  selected-scope coverage remain pending. T060 is not marked complete.

## CI coverage follow-up (run 39518177296)

### Red

- The exact synthetic merge `6fce6cbf592ed82cb0b5dcecae111a802f54fc5c`
  failed the selected changed-production coverage gate: 11,218 of 11,833
  executable lines (94.8%), below the required 95% / 11,242 lines.
- Artifact attribution confirmed the Python hits map to
  `bindings/python/src/kmipkit/client.py`; this is not a source-path or
  instrumentation mismatch. `%TEMP%\kmipkit-pr104-39518177296-allcov\coverage-python\coverage.xml`
  marks the defensive branches in `_rekey_key_pair_response` at lines
  2459–2461, 2463, 2467, 2477–2479, and 2482–2484 as zero-hit. The baseline
  artifact had only the existing success-path operation test for this helper.
- The same artifact identified uncovered Java cleanup lines in the Re-key Key
  Pair response parser. The existing code already implements cleanup; missing
  semantic tests, rather than a production defect, caused these coverage gaps.
- This coverage failure is the Red evidence for the follow-up. No new failing
  behavior test was manufactured because the target defensive behaviors are
  already implemented and correct.

### Green

- Commit: `d8d0dfc5bd6b0e637b411a8c7ec5c82ce5247028`
  (`test: cover Re-key Key Pair response cleanup branches`), DCO signed.
- Java command: `mvn -q -f bindings/java/pom.xml -Dtest=RekeyKeyPairMappingTest test`.
  Surefire passed: 6 tests, 0 failures, 0 errors, 0 skipped. The added malformed
  success case supplies a real native TTLV handle, a valid private identifier,
  and malformed public identifier data; it asserts the public parser rejects
  the response and verifies the native structure handle has been released.
- Python command:
  `python.exe -m pytest -q bindings/python/tests/test_rekey_key_pair_operation.py`
  with `PYTHONPATH=bindings/python/src` and `KMIPKIT_NATIVE_LIBRARY` set to the
  built native DLL. Passed: 7 tests. The new absent, invalid-presence, missing
  identifier, TTLV presence mismatch, and wrapper-construction error cases
  verify behavior and release actual native handles.
- Focused Python coverage command:
  `python.exe -m coverage run --data-file=%TEMP%\kmipkit-t060-python-focused-20261021.coverage --source=kmipkit.client -m pytest -q bindings/python/tests/test_rekey_key_pair_operation.py`,
  followed by `coverage json`. Passed: 7 tests; no executable lines from
  `_rekey_key_pair_response` at `client.py:2458-2484` were missing in the
  focused report.
- Compiled C consumer command:
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_compiles_and_runs -- --exact`.
  Passed: 1 test, 0 failures, 10 filtered out. This retains the C API boundary
  and wire-capture evidence recorded above.

### Refactor and final checks

- Refactor commit: this report update and a small Python test-only extraction
  of the repeated native-release tracker into `_tracked_native_release`; the
  real native release and registry invalidation assertions remain in place.
- Python lint/format commands:
  `uv tool run --from ruff ruff check bindings/python/tests/test_rekey_key_pair_operation.py`
  and `uv tool run --from ruff ruff format --check bindings/python/tests/test_rekey_key_pair_operation.py`.
  Both passed. Ruff initially reported an import-order issue and formatting
  changes; those were applied, then both final checks passed.
- After the test helper refactor, Python focused pytest passed again: 7 tests.
  A fresh isolated coverage file also showed no missing executable lines in
  `client.py:2458-2484`.
- `cargo fmt --all --check`: PASS. `git diff --check`: PASS.
- The Java Maven run has no configured source formatter task; focused Java
  compilation and tests passed. C formatting is covered by the repository's
  whitespace check for this C test-only change.
- Existing non-fatal build diagnostics included AWS-LC `stdalign_check.c` and
  NASM probe warnings and MSVC linker LNK4217 warnings.
- Self-review: follow-up changes are limited to Python and Java tests plus this
  evidence report. No production implementation, public API/ABI, manifest,
  generated output, or normative source changed. Untracked local build and
  coverage artifacts were not staged or cleaned.
- Remaining limits: this focused run does not recompute the selected changed-
  production coverage aggregate. Controller review, security review, exact-
  head CI and the final coverage gate remain pending; T060 is not marked
  complete.

## Final QA C test cleanup

- Updated `rekey_key_pair_constructor_is_callable_from_c` to release an
  unexpected non-null constructor output when it differs from the live
  sentinel handle. The sentinel remains restored to the shared cleanup path,
  preventing both a leak for a distinct output and a double-free for an alias.
- Command:
  `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_compiles_and_runs -- --exact`
  passed: 1 test, 0 failures, 10 filtered out. The C build emitted the existing
  non-fatal AWS-LC/NASM probe and MSVC LNK4217 warnings.
- `git diff --check`: PASS. This is test-harness cleanup only; no production
  code or ABI changed. No push was performed.
