# KMIPKIT-0040 implementation evidence

Status: Rust increment and T019 four-language API mapping verified on the
KMIPKIT-0043 feature branch; **specification incomplete** while T022 remains
open. Scope: OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275 and
nested §§7.26–7.30, Tables 378–382. All fixtures are source-derived.

## Branch and source baseline

The dedicated worktree uses `feature/KMIPKIT-0040-pkcs11`. The existing planning
branch was merged from current `origin/release/1.0.0` at
`a0846037eb9930b24d9b9e3d8c63b2e66b438d1a` without conflicts, force push, or
history rewrite. Pinned OASIS sources were checked against that exact base.
Baseline catalog: 4 sources, 1,411 clauses, 4,044 records; validator passed.
All required operation, enum, nested structure/member and tag records exist.

## Authorization and acceptance status

The KMIPKIT-0040 specification remains **Approved for implementation** as
recorded in `spec.md` on 2026-10-11. On 2026-10-20, the maintainer directly
authorized autonomous implementation of the approved plan, including this
contract. This is implementation authorization only; it does not change the
specification's approval status or constitute formal acceptance. The parity
fixture records `formalAcceptance: false` separately from the verified
implementation mapping state. No scope expansion, Profile claim, or
certification claim follows from that authorization.

## Distinct DCO evidence commits

All commits carry `Signed-off-by: NeverWe1come <vgarciaaguilar97@gmail.com>`.

| Phase | Commit | Command / observed evidence |
|---|---|---|
| Planned verification | `a4e8f325` | Exact named table/source verification targets committed before behavior |
| Protocol Red | `769c3fb9` | `cargo test -p kmipkit-protocol pkcs11_operation_tests`: expected E0432 for missing Pkcs11 request/response/error types; no production implementation |
| Protocol fixture correction | `8de8a7bd` | Pending fixture supplies required shared asynchronous correlation; absence of Success Response Payload belongs to shared message validation |
| Protocol Green | `a4112d89` | Same focused command: 9 passed |
| Protocol Refactor | `fe5ad8d5` | Opaque callback accessor reuse; 9 tests remain passing; rustfmt |
| Client Red | `9100cf8e` | `cargo test -p kmipkit-client --lib pkcs11_execution_tests`: expected E0599 for absent variants, typed accessor and explicit client methods |
| Client Green | `024158e0` | Same focused command: 7 passed |
| Protocol lint/boundary cleanup | `b9683fb7` | Malformed optional/return-code, duplicate/depth, sanitized source checks; protocol 12 passed; workspace Clippy passed |
| Client Refactor | `dae83cce` | Shared single-request executor and operation-specific response dispatcher; client 7 passed; format and workspace Clippy passed |

Red test registration hooks were included because this repository discovers
unit suites through external `#[path]` test modules. They add no production
behavior. Numeric separators, borrowed helper arguments and shared match arms
were cleaned up after the passing implementations; no lint gate was relaxed.

## Verified local commands

Runs are on Windows x86_64. Default stable Rust is 1.99.0; the explicit MSRV
check uses Rust 1.94.0. Detailed uncommitted tool logs are under `target/`.

| Command | Result |
|---|---|
| `cargo test -p kmipkit-protocol -p kmipkit-client pkcs11` | PASS: protocol 15; client 9; no failures |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo +1.94.0 check --workspace --all-targets --all-features --locked` | PASS |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked` | PASS |
| `python tools/normative_catalog/validate.py` | PASS: sources=4 clauses=1411 records=4044 |
| `python tools/normative_catalog/report.py --write` then `--check` | PASS; report regenerated only through pinned tool |
| `python tools/api_manifest/generate.py --check` | PASS: 6 current generated outputs; no 0043 mapping added |
| Catalog `generate_ttlv_tags.py`, `generate_result_values.py`, `generate_attribute_types.py`, `generate_attribute_policy.py`, each `--check` | PASS |
| `python tools/normative_catalog/check_immutable_sources.py --base-sha a0846037eb9930b24d9b9e3d8c63b2e66b438d1a` | PASS |
| `python -m unittest discover -s scripts/tests -p 'test_requirement_traceability.py' -v` | PASS: 2 tests |
| `python -m unittest discover -s scripts/tests -p 'test_diagnostic_redaction.py' -v` | PASS: 1 test |
| `python -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v` | PASS after QA fix: 15 tests, 1 fixture skip |
| `git diff --check` | PASS |
| `python -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` | PASS: 213 tests, 7 platform/fixture skips |
| `cargo test --workspace --all-features` | PASS after serial rerun: all tests and doctests passed; first concurrent build had been stopped before tests |
| `cargo llvm-cov --workspace --all-features --locked --json --output-path target/pkcs11-coverage-raw.json` | PASS; full instrumented tests completed and JSON exported |
| `python scripts/coverage_gate.py normalize --workspace . --input target/pkcs11-coverage-raw.json --output target/pkcs11-coverage.json` | PASS: LLVM schema 3.1.0 accepted |
| `pwsh -File scripts/Test-DependencyPolicy.ps1` | PASS: exact cargo-deny 0.20.2; root/fuzz checks, fresh online advisory evidence, negative fixtures and unchanged lockfile hashes |

Full Ubuntu/macOS/Windows plus required adapter coverage reports are needed for
the repository aggregate coverage gate. A Windows-only report is local
measurement and cannot establish the complete multi-platform gate.

## Traceability, security and limitations

Every Table 273/274 row links to stable catalog structures/tags and named
positive/relevant negative verification in `traceability.md`. All Table 275
reasons are exercised, including the three PKCS#11-specific reason values;
unknown Result Reason values remain lossless. No independent §6.1.37 client
normative clause exists, so no synthetic requirement is assigned.

Function and Return Code preserve required nested values and unknown members;
vendor Function Integer is preserved under the approved specification.
Interface and parameter structures remain opaque. Failure/Pending do not
fabricate Success fields. Entire generic response batch items are retained.
Pending asynchronous correlation is caller-enabled and forwarded without
retry/Poll. Invalid requests retain NotSent; partial writes PossiblySent;
malformed received responses ResponseStarted. No transport boundary is added.

Opaque Structure/Item values and SecretBytes use existing owned-memory
zeroization. Debug only shows redacted metadata; Function, Input, Output,
Correlation and generic response sentinel tests verify that values do not
escape through Debug, Display or sanitized errors. Caller-created external
copies are outside KMIPKit-owned memory. Binding and runtime-copy limits are
documented alongside the four-language mappings.

No official PKCS#11 fixture is linked in the pinned catalog. No official
vector pass, profile support, certification, FIPS status or interoperability
claim is made. No live PKCS#11 server exchange was executed. The two-independent
server gate in T022 remains open and the existing matrix stays Pending.


## Independent QA correction evidence

- Red `d2f3362d`: two protocol cases failed because wrong/duplicate known
  nested Interface/Input/Output scalars were accepted; two client cases failed
  because an invalid request reached the transport (PossiblySent) and malformed
  nested Output Parameters were accepted as Success.
- Green `daf0ad6b`: known nested 0..1 Interface TextString and Input/Output
  ByteString members are validated. Protocol 15 and client 9 tests passed.
- Refactor `8549fcc7`: one shared required/optional cardinality validator;
  protocol 15 and client 9 remain passing. Workspace Clippy and explicit MSRV
  1.94.0 all-target/all-feature checks were rerun successfully after the fix.

Source interpretation `KMIPKIT-026-SOURCE-001` is recorded in `research.md` and
linked to existing vendor Integer request/response tests. It uses the explicit
§6.1.37 Tables 273/274 vendor context and preserves §7.26 Table 378's outer
Structure and standard Enumeration shape. The same-primary-document mismatch
and observable alternatives remain documented; no official erratum, profile
or complete conformance claim is made.


## Final serial local and real CI coverage evidence

The serial local test, coverage and dependency-policy commands all returned
exit 0. Native `pkcs11.rs` line coverage was 218/226 (96.46%). Windows-only LLVM
file-summary totals are not the project aggregate: native FFI was 22.73% and
native overall totals were 89.70%, because the Linux C-consumer collector and
other platform/adapter reports are not part of this local Rust-only run. No
missing platform or C-consumer artifact is fabricated from that report.

Actual CI run [38181159004](https://github.com/NeverWe1come/KMIPKit/actions/runs/38181159004)
completed **SUCCESS** for implementation head
`95644a0f1978259c9c30004c64eca8435e00d8d7`. All required core/MSRV, platform,
C-consumer, Java/Python/JNI, sanitizer, generated-output, normative inventory,
dependency-policy and final-summary jobs passed. The real selected-scope
coverage gate consumed its required CI artifacts and reported:

| Scope | Covered / measured lines | Coverage / gate |
|---|---:|---|
| Changed Rust | 244 / 252 | 96.83% / 95% |
| TTLV | 735 / 741 | 99.19% / 95% |
| Protocol | 11,997 / 12,377 | 96.93% / 95% |
| Transport | 3,713 / 3,902 | 95.16% / 85% |
| Rust FFI | 2,035 / 2,137 | 95.23% / 85% |
| Java | 618 / 670 | 92.24% / 85% |
| Python | 773 / 783 | 98.72% / 85% |
| JNI | 1,193 / 1,312 | 90.93% / 85%; 21 summary-only uncovered lines included |
| Workspace | 25,127 / 26,010 | 96.61% / 90% |

Local dependency policy verified fresh online RustSec root and fuzz evidence
at commit `7eebec69c352c7191b1f13eb95dd510eeca5d1de`, database commit timestamp
`2026-10-09T10:12:02+02:00`; the refresh succeeded for both workspace scans.
Existing exact exceptions were KMIPKIT-0011-EX-003 and KMIPKIT-0011-EX-004.
Both Cargo.lock hashes and resolved-package invariance checks passed.

Independent QA confirmed both review observations resolved after the nested
member correction and source-context trace. T022's remaining condition is real
PKCS#11 execution against two independent supported KMIP 2.1 implementations.
The PR remains a draft, and the whole specification is not declared complete.

## Four-language API mapping evidence — 2026-10-20

Implementation authorization and formal acceptance remain distinct: the
contract is **Approved for implementation**, the direct human instruction to
start implementation is dated 2026-10-20, and `formalAcceptance` is `false`.
The operation mapping is marked verified in the API manifest only to describe
implementation and parity evidence; it is not a statement of formal spec
acceptance, complete KMIP conformance, Profile support, or interoperability.

The TDD implementation commits are Red `2416f30a`, `2d7d6442`, `9bfca7d6`,
`f8cba650`, and `60ba97d4`; Green `b75a6a4d`. Review correction Red
`8bf29ae9` reproduced the missing Python star-import export and strengthened
the C/Python response-copy lifetime checks; Green `e2c0e948` exports
`Pkcs11Response` in `client.__all__`. Final Refactor/evidence is recorded in
the feature branch's latest commit. Rust, compiled C over a TLS 1.3
mutual-authentication loopback, Java/JNI over the same fixture, and
Python/CFFI all assert the shared successful response. The checks include the
full Batch Item Response wrapper followed by all five payload tags in order,
unknown Return Code and vendor extension preservation, optional absence,
redaction, response lifetime after result closure, and no retry or automatic
follow-up. The C consumer is compiled and executed; evidence is not limited to
linking or symbol inspection.

The zeroization follow-up uses distinct DCO commits: Red `20cc1eea` reproduces
the Python response retaining the correlation bytes after close and the error
path retaining them after a later field conversion fails; Green `d7599210`
stores the KMIPKit-owned copy in `_SecretBytes`, exposes only caller-owned
`bytes`, and wipes the owned copy on close or conversion failure. JNI cleanup
commit `ffb17be1` uses the existing tested response-array scope so failed
tuple construction wipes the temporary correlation array, while successful
tuple construction transfers it to Java's existing cleanup. Refactor/evidence
commits `de663650` and `36091d3d` wipe Python bytes before releasing TTLV
copies and tighten the PKCS#11 JNI export audit. A candidate JNI residual
memory concern was independently reviewed and suppressed by the exact-diff
security scan because recovery requires privileged same-process memory access
or OS-level diagnostics outside the canonical threat boundary; no reportable
finding was recorded.

The follow-up JNI copy-failure path uses Red `ca840e9f` and Green `2c107239`.
If `SetByteArrayRegion` leaves a pending exception after partially copying a
response byte string, `span_to_java_bytes` now wipes and deletes its local Java
array before returning while preserving the original exception. The native
test exercises cleanup of an allocated response array when the copy reports a
pending exception.

The adapter parity follow-up now exercises three distinct Correlation Value
states over real TLS 1.3 mutual-authentication loopback exchanges: non-empty,
absent, and present with zero bytes. Red commits are Python/CFFI `559027aa`,
Java/JNI `4a023359`, and compiled C `c744254b`; Green commits are respectively
`2b0a7993`, `c53eabc3`, and `49d26f9c`. Each bridge checks that optional
presence is preserved independently of byte length. The C consumer also
checks that absence removes the TTLV field and shifts the remaining ordered
payload items, while present-empty keeps the Byte String field and reports
present with length zero. QA accepted the behavior and raised a P3 fixture
maintenance concern; Refactor `df8cc99d` now derives all three C case values
from `typedResponse.correlationValue` and
`correlationValuePresenceCases`. QA confirmed that finding is resolved. No
production behavior change was needed.

| Command | Result |
|---|---|
| `cargo test -p kmipkit-client pkcs11` | PASS; focused client PKCS#11 suite and public API mapping test |
| `cargo test -p kmipkit-protocol pkcs11` | PASS; 15 tests |
| `cargo test --workspace --all-features` | PASS; full Windows workspace test and doctest run |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage compiled_c_pkcs11_consumer_matches_shared_response_fixture_over_mtls -- --exact` | PASS; compiled C consumer executed its mTLS success exchange |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage` | PASS after `b319c7fb`; all 10 compiled C ABI integration tests, including non-empty, absent, and present-empty correlation responses over mTLS |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage compiled_c_pkcs11_consumer_matches_shared_response_fixture_over_mtls -- --exact` | PASS after fixture-driven Refactor `df8cc99d`; all three fixture cases still pass through the compiled C consumer over mTLS |
| `cd bindings/java; .\gradlew.bat test --rerun-tasks --tests 'org.kmipkit.ClientFacadeTlsIntegrationTest.pkcs11ResponseMatchesSharedFixtureThroughNativeJniOverMutualTls' --tests 'org.kmipkit.extensions.Pkcs11MappingTest'` | PASS; Java/JNI mTLS fixture tests |
| `cd bindings/java; .\gradlew.bat nativeJarSmoke --offline` | PASS; packaged JNI loading and negative probes completed offline |
| `python -B tools/api_manifest/audit_jni_public_exports.py --binary bindings/java/target/native/kmipkit_jni.dll` | PASS; 2 required public JNI symbols present |
| `$env:KMIPKIT_NATIVE_LIBRARY=(Resolve-Path target/debug/kmipkit_ffi.dll).Path; & target/python-t076-venv/Scripts/python.exe -m pytest bindings/python/tests/test_pkcs11_operation.py -q` | PASS; 7 tests using the Windows native library, including import-star and copied response after batch result closure |
| Python 3.12 `pytest bindings/python/tests/test_pkcs11_operation.py -q` with `KMIPKIT_NATIVE_LIBRARY=target/debug/kmipkit_ffi.dll` | PASS after `2b0a7993`; 12 tests, including all three real CFFI mTLS correlation states |
| `cd bindings/java; .\gradlew.bat test --tests 'org.kmipkit.ClientFacadeTlsIntegrationTest.pkcs11ResponseMatchesSharedFixtureThroughNativeJniOverMutualTls' --rerun-tasks` | PASS after `c53eabc3`; three real JNI mTLS cases for non-empty, absent, and present-empty correlation |
| `target/python-t076-venv/Scripts/python.exe -m unittest tools.api_manifest.tests.test_t076_client_mapping tools.api_manifest.tests.test_pkcs11_parity_manifest tools.api_manifest.tests.test_operation_readiness -v` | PASS; 68 tests |
| `python -B -m unittest scripts.tests.test_workflow scripts.tests.test_native_package_smoke -v` | PASS; 53 tests and offline installed-wheel probes |
| `python -B tools/api_manifest/generate.py --write` then `--check` | PASS; 12 generated outputs regenerated and verified with the pinned generator |
| `cargo test -p kmipkit-protocol pkcs11 --all-features` | PASS at `36091d3d`; 16 PKCS#11 protocol tests, including absent versus present-empty correlation |
| `bindings/java/native/build.ps1` | PASS at `ffb17be1`; JNI bridge compiled and native response-secret-array cleanup tests passed |
| `mvn -q -Dkmipkit.nativeLibrary=<built kmipkit_jni.dll> -Dtest=Pkcs11MappingTest,ClientFacadeTlsIntegrationTest test` | PASS at `ffb17be1`; Java/JNI mapping and real mutual-TLS PKCS#11 exchange suites |
| Python 3.12 `pytest tests/test_pkcs11_operation.py -q` with `KMIPKIT_NATIVE_LIBRARY=target/debug/kmipkit_ffi.dll` | PASS at `de663650`; 10 tests, including owned-correlation wipe and conversion-failure cleanup |
| Python 3.12 `python -m unittest tools.api_manifest.tests.test_pkcs11_parity_manifest` | PASS at `36091d3d`; 6 manifest/export tests; candidate and uninventoried PKCS#11 JNI exports are rejected |
| `python -B tools/api_manifest/generate.py --check` | PASS at `36091d3d`; all 12 generated outputs current |
| `cargo fmt --all --check` and `git diff --check` | PASS at `36091d3d` |
| `bindings/java/native/build.ps1` | PASS at `2c107239`; JNI DLL compiled and native tests cover zeroization after response byte-array copy failure |
| `mvn -q -Dkmipkit.nativeLibrary=<built kmipkit_jni.dll> -Dtest=Pkcs11MappingTest,ClientFacadeTlsIntegrationTest test` | PASS after `2c107239`; Java/JNI mapping and mutual-TLS response suites |
| `cargo test -p kmipkit-ffi --features coverage-c-consumer --lib every_exported_c_function_starts_inside_the_shared_panic_boundary` | PASS; export inventory recognizes all 247 C exports and validates their panic boundary |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS; only non-fatal AWS-LC toolchain probe warnings on Windows |
| `git diff --check` | PASS |

| `cargo llvm-cov -p kmipkit-protocol -p kmipkit-client -p kmipkit-ffi --all-features --all-targets --json --output-path target/t057-rust-adapters-current-head-optional-correlation.json` | PASS on Green `49d26f9c`; selected Rust crate set 26,799/28,099 lines (95.37%), including compiled C consumer tests; this does not measure the changed-production aggregate or cross-language gate |
| `cargo llvm-cov --workspace --all-features --json --output-path target/t057-current-coverage-raw.json` | BLOCKED locally before export: unrelated `kmipkit-test-support::dns::loopback_tcp_fixture_answers_a_framed_query_and_closes_on_short_frame` could not bind `127.0.0.1:53817` after 32 attempts (`PermissionDenied`, Windows error 10013) |

The focused T057 success-path checks, source/configuration tests, traceability,
manifest, generator, formatting and Clippy pass locally. Exact-head hosted CI
run `39456251264` on `b0f786243b7acfec6b031caebe02a3685793c57f` passed all 34
required jobs, including the Ubuntu/Windows/macOS adapter matrix and selected-
scope coverage. The changed-production aggregate is 10,508/11,059 lines
(95.02%), including 22 summary-only lines counted as uncovered. This completes
the T057 mapping, parity, platform, and coverage gates. The earlier Windows
workspace coverage bind failure remains a local-environment limitation, not a
failure of the hosted gate. The candidate-only build gates were removed from
the default source/configuration and the default Rust/C/Java/Python path now
exercises the public API. The successful loopback exchange verifies adapter
behavior and is not interoperability evidence. `formalAcceptance` remains
`false`; T022 remains open for two independent KMIP 2.1 server implementations.
