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
