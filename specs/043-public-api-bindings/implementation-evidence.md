# Implementation Evidence

## T015 — Manifest-backed handle lifecycle and C status contract

This task closes the existing approved C ABI lifecycle implementation against the v2 manifest. Opaque handles use a private `Arc<Handle>` envelope with a kind discriminator; the T022 follow-up below adds a process-local registry of keyed, non-reused tokens so foreign handle values are checked before payload access. Views retain their owning handle, so releasing the original owner does not invalidate a child view. Every manifest opaque-handle type declares library ownership and its matching release symbol. Input byte spans carry explicit `uint64_t` byte lengths, apply bounds before dereferencing, and validate UTF-8 before model construction. C status values remain the manifest-declared stable codes, and safe category/cause/delivery snapshots are stored per thread. Java and Python resource wrappers reject operations on closed handles and serialize closure against native use.

The C release ABI accepts an opaque pointer by value, so it cannot clear the caller's variable. A caller should set its local variable to `NULL` after release. Before the T022 follow-up, stale, dangling, and foreign handle values were treated as caller-precondition violations; the follow-up replaces that limitation with registry lookup and keyed, non-reused tokens while preserving the C ABI. Java/Python wrappers retain local closed state and reject operations after close.

| Verification | Command or evidence | Result |
|---|---|---|
| Rust FFI lifecycle, metadata, ownership, kind, and null guards | `cargo test -p kmipkit-ffi --lib` | PASS: 33 tests. The Windows AWS-LC configure probe and missing NASM emitted build warnings; the test target completed successfully. |
| C handle/error/span consumer coverage | `bindings/c/tests/extension_registry.c::test_redacted_stable_errors_and_invalid_handles`, `test_null_closed_and_wrong_kind_handles_are_rejected`, `test_typed_length_safety`, and `test_handle_release_and_null_output_on_error` | The C consumer covers null and wrong-kind validation, invalid UTF-8/explicit byte lengths, redacted status snapshots, resource limits, release ordering, and the release-then-clear-to-NULL rejection case added for T017. The full C consumer compiled with MSVC and passed locally on Windows; Linux CI for the updated commit remains required before T017 can be checked complete. |
| Java closed-handle and transfer lifecycle | `mvn -f bindings/java/pom.xml '-Dtest=NativeHandleTransferTest,ExtensionRegistryTest' test -q` | PASS: 20 tests, 0 failures (4 handle transfer tests and 16 extension registry tests). |
| Python closed-handle and ownership lifecycle | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_handle_ownership.py bindings/python/tests/test_extension_registry.py -k 'closed or ownership or invalid_inputs' -q` | PASS: 6 tests, 20 deselected. |
| Manifest-generated declarations | `python -B tools/api_manifest/generate.py --check` | PASS: all 12 fixed outputs verified. |
| FFI lint | `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings` | PASS. |

No new behavior or executable contract was introduced in this closeout, so no new Red/Green/Refactor implementation commits were needed. The Linux C consumer remains covered by the PR's Linux CI matrix.

## T017 — C consumer characterization of shared ABI contracts

The C consumer now explicitly demonstrates the agreed safe representation of a closed C handle in `test_null_closed_and_wrong_kind_handles_are_rejected`: create and use an identity, release it, set the caller-owned pointer to `NULL`, then assert that the accessor returns `KMIPKIT_ERROR_INVALID_INPUT` and leaves its output pointer `NULL`. The freed pointer is never passed to an accessor. Existing consumer cases cover successful ABI use, validation errors, resource-limit rejection, view/configuration release ordering, and the legacy limits ABI. No production code changed; the NULL guard was already implemented, so this is a characterization/coverage addition rather than a fabricated Red/Green behavior change.

The implementation's TDD cycle is recorded in the original C ABI commits: `c73a89bb` (Red: failing vendor-registry binding parity test) → `d4528d9a` (expanded failing consumer coverage) → `f08dcf4d` (Green: C ABI implementation) → `c6afef98` (Refactor: shared validated handle-release path). The new release-then-clear-to-`NULL` assertion characterizes the existing NULL guard and changes no production behavior; it extends the consumer evidence for that completed contract.

| Verification | Command or evidence | Result |
|---|---|---|
| Rust FFI implementation and guard tests | `cargo test -p kmipkit-ffi --lib` | PASS: 33 tests. AWS-LC emitted its existing Windows compiler/NASM probe warnings; Cargo completed successfully. |
| Compile the C ABI consumer with MSVC | `cmd.exe /c 'call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul && cl.exe /nologo /std:c11 /W4 /WX /Ibindings/c/include /Itests/fixtures/api/generated bindings/c/tests/extension_registry.c target/debug/kmipkit_ffi.dll.lib /Fe:target/c_consumer.exe'` | PASS: compiled with warnings as errors. |
| Execute compiled C ABI consumer | `$env:PATH = "$PWD\target\debug;$env:PATH"; & target\c_consumer.exe` | PASS: all 16 consumer cases, including `null, closed, and wrong-kind handles are rejected`, succeeded. |
| Updated Linux C-consumer CI | Run `38339977117`, head `08271ae4`, `C consumer / ubuntu-latest` and `C FFI sanitizer / Linux` | PASS: both Linux consumer and AddressSanitizer checks passed. The full run passed all 32 required checks; two configured checks were skipped. The separate Windows/MSVC consumer also passed locally and Windows/macOS C consumer checks passed in this run. |

## T012 — Shared Rust/C parity fixture corpus

The source fixture corpus at `tests/fixtures/api/shared-capability-parity.json` defines six planned parity cases for API-level discovery, per-configuration registry isolation, schema-error redaction, ordered mixed batch outcomes, generic TTLV open-value/order preservation, and extension-registry recognition/preservation. It records stable requirement IDs, planned expected output, Java-observed output, and the shared extension fixture source where applicable. The original v1 `consumerTests.rust` and `consumerTests.c` entries were intended as future test names; those referenced files and tests did not exist. Fixture v2 removes those claims and explicitly records Rust/C as planned consumers, Java as the only executable consumer language, and Java blocked capabilities separately from available mappings. T012 closes the shared contract-fixture creation only; it does not claim Rust/C runtime consumers or parity.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `835b4e8c` | `python -B -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_parity_fixtures_cover_rust_and_c_contracts -v` | Expected failure: the shared API parity corpus did not exist. |
| Green | `6774abcc` | Same focused command | PASS: fixture corpus and planned capability, requirement, output, error, cleanup, and extension-source references were present and valid. Rust/C test-name strings in v1 were plans, not executable consumers. |
| Refactor | `3c548559` | `python -B -m unittest tools.api_manifest.tests.test_parity -v`; `git diff --check` | PASS: all 8 parity tests passed; no whitespace errors. Reused one helper for capability-case lookup without changing fixture behavior. |

Fixture v2 keeps planned outputs distinct from observed per-language results. The validator accepts executable test references only for Java and checks that each referenced method is a Java 17 JUnit test. The Rust/C consumer implementation and cross-language fixture execution remain outstanding under their own tasks.

## T020 — Java 17 shared capability consumers and lifecycle

The Java fixture mappings now cover only contracts present in the current facade: immutable per-configuration extension registries, schema-error diagnostic redaction, generic TTLV open Enumeration and byte preservation with child order, and extension recognition/payload preservation. The shared fixture points each available Java mapping to an executable JUnit method, and the Java test reflects over those references to reject missing or non-test targets. The configuration test closes both `AutoCloseable` configurations and registry views; the TTLV test closes its borrowed Structure view and verifies that the independent Structure owner remains usable afterward.

API-level discovery and ordered batch outcomes remain blocked in the Java mapping. Separate Java tests read the case IDs/capability identifiers from the fixture and assert `RuntimeMetadata.capabilityIsAvailable` returns false; the fixture records no fabricated API-level list, batch statuses, or transport result as Java-observed output. Rust/C consumers remain planned and are not asserted by the Java task.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `bf42c780` | `bindings/java/gradlew.bat test --tests org.kmipkit.SharedCapabilityParityFixtureTest` | Expected failure: the Java 17 test found no Java language/consumer mapping in shared fixture v1. |
| Green | `6b1b3a52` | `bindings/java/gradlew.bat test --rerun-tasks`; `python -B -m unittest tools.api_manifest.tests.test_parity -v`; `git diff --check` | PASS: Java suite ran 88 tests with 0 failures; parity validator ran 8 tests with 0 failures; native DLL and JNI consumer built. AWS-LC emitted Windows compiler/NASM probe warnings, but all Gradle tasks succeeded. |
| Refactor | `d312627d` | `bindings/java/gradlew.bat test --tests org.kmipkit.SharedCapabilityParityFixtureTest`; `python -B -m unittest tools.api_manifest.tests.test_parity -v`; `git diff --check` | PASS: all five shared fixture Java tests, all eight parity validator tests, and whitespace validation passed after centralizing Java mapping parsing. |

This closes T020's Java test-writing and executable-mapping scope. It does not close Java facade expansion (T021–T022), Rust/C fixture execution, Python mapping, full four-language parity (T029/T071), or the blocked API-level and ordered-batch capabilities.

## T022 — Java wrapper ownership and pre-JNI input validation

The Java view chain keeps each native owner strongly reachable: `TtlvStructureView` retains its `TtlvStructure`, `TtlvItemView` retains its parent structure view, and `TtlvValueView` retains its item view. `NativeHandle` serializes scoped JNI use with deterministic, idempotent close. A regression deterministically checks that each private view owner field references its expected parent, then reads the borrowed integer value.

The transfer-site audit found four preflight gaps. `TtlvItem.create` and `TtlvPath.withChildTag` detached the consumed value/path before validating a borrowed tag; `ExtensionDefinition.withInformation` detached the definition before validating its borrowed information handle; and `ExtensionInformation.with_type` detached its handle before dereferencing a nullable type. These paths now reject null arguments with `InvalidInputException`, validate all borrowed owners while holding the shared lifecycle lock, and transfer only after validation succeeds. Regression tests confirm the original owner remains usable after null or closed inputs are rejected. Other transfer sites either validate first, use atomic `transferAll`, or have no second owned input to invalidate.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `c719aaa6` | `mvn -f bindings/java/pom.xml '-Dtest=TtlvItemOwnershipTest,WrapperTransferValidationTest' test -q` | Expected failure: 7 tests produced 4 assertion failures and 3 errors. Null inputs reached `NullPointerException`, and closed borrowed inputs left the first owner unusable after the rejection. |
| Green | `81aaaeaf` | `mvn -f bindings/java/pom.xml '-Dtest=TtlvItemOwnershipTest,TtlvViewOwnerRetentionTest,WrapperTransferValidationTest,NativeHandleTransferTest,PublicWrapperLifecycleCoverageTest' test -q` | PASS: all 16 focused ownership, lifecycle, preflight, and owner-retention tests; 0 failures/errors. |
| Refactor | `6f796605` | Same focused command; `mvn -f bindings/java/pom.xml verify -q`; `git diff --check` | PASS: focused suite 16/16; full Java suite 153 tests, 0 failures/errors, 3 documented skips; JaCoCo line coverage 95.43% (1,461 covered / 70 missed); Maven verification and whitespace checks passed. The refactor normalized invalid-input exception imports without behavior changes. |

Verification ran on Windows x86-64 with the locally built JNI library. Cargo emitted existing AWS-LC compiler-probe and missing-NASM warnings; JNI built and Maven verification succeeded. Cross-platform CI for the current unpushed head remains a PR-matrix check. The earlier FFI opaque-token follow-up's Linux CI remains separate from this Java ownership acceptance.

Independent review rejected the original single-`System.gc()` owner-retention assertion as nondeterministic. The follow-up replaces it with exact assertions that each private view owner field points to the expected parent, then reads the borrowed value. This verifies the declared strong-reference chain without relying on garbage-collector scheduling.
The corrected `TtlvViewOwnerRetentionTest` passed locally (1 test, 0 failures/errors), and `git diff --check` passed.

## T023 — Python shared parity, lifecycle, typing, and package tests

The Python 3.12 tests validate every currently available Python consumer link in the shared capability fixture, preserve the intentionally blocked T077 API-level case, and link the generic TTLV case to an executable consumer. That consumer reads the shared structure, preserves unknown Enumeration `3775005444`, ByteString `80ff01`, and child order, and verifies the view retains its owner until deterministic one-time cleanup. Parameterized wrapper tests exercise normal and exceptional context-manager exits and idempotent close for `Client`, `Batch`, and `BatchResult`. The public API annotation inventory checks parameters, returns, and property accessors. The package test builds an offline wheel with pinned Maturin 1.9.4 and checks `py.typed` plus the adjacent native library in the archive; it does not install or load the wheel.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `03d2376e` | `python -m pytest bindings/python/tests/test_shared_parity_fixtures.py -q` | Expected failure: 14 passed, 1 failed because the Python generic-TTLV fixture consumer was still blocked. No runtime defect was found in the other cases. |
| Red correction | `7ae7ea76` | Same focused command | Added property getter/setter/deleter annotation inspection after independent review identified that omission; expected fixture-status failure remained the only failure. |
| Green | `76841807` | Same focused command; `git diff --check` | PASS: 16 tests. The fixture maps the TTLV case to the new Python consumer; T077 remains blocked. |
| Refactor | `c4b69988` | Same focused command; `git diff --check` | PASS: 16 tests. Extracted fixture structure construction, made Maturin selection prefer the active project environment, and clarified that the test inspects a built wheel. |

Independent review approved the corrected Red, Green, and Refactor evidence. Root reran the Python 3.12 focused suite after Green and Refactor; both runs passed 16/16. The complete Python test suite then passed 209 tests. This task adds tests and fixture evidence only; it changes no runtime or generated code. End-to-end shared API parity is owned by T029/T071.

## T024 — Python 3.12 shared facade mappings

T024 is complete for the approved Python facade scope: the Python package maps
the shared client construction, transport configuration, typed ordered batch
execution and results, generic structurally valid TTLV, stable error metadata,
and explicit wrapper ownership over the C ABI. Public annotations and the
`py.typed` marker are checked by the Python contract suite. The exact mapping
and ownership implementation evidence is recorded in the T076 Python mapping,
native-input, batch-identity, and ordered-outcome sections below; the
fixture-driven Python contract evidence is recorded under T023 above.

| Acceptance evidence | Command or record | Result |
|---|---|---|
| Python behavior, lifecycle, typing, package contents, and fixture consumer | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests -q` | PASS: 209 tests. |
| Shared API declaration accounting | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-14.json --check` | PASS: 556 Python paths observed, 425 exact manifest declarations and 131 reviewed exclusions; no missing, stale, kind-mismatched, or issue paths. The generated current report also reconciles C 272 observed / 242 matches and Java 491 observed / 449 matches. |
| Checked-in generated API artifacts | `python -B tools/api_manifest/generate.py --check` | PASS: all 12 outputs verified. |
| Mapping implementation TDD and review | `T021/T024 unique batch item identity mapping`; T076 Python transport, ordered-outcome, and client-mapping evidence below | Red/Green/Refactor and independent review are recorded with the implementation work; this T024 closeout adds no runtime changes. |

The current binding-export snapshot also exposed a stale exact-exclusion test
and a missing architecture-document citation on the `clientExecuteWithOptions`
JNI bridge exclusion. The follow-up corrects the audit contract and records a
fresh current report; it changes no runtime behavior.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `81ad7141` | `python -B -m unittest tools.api_manifest.tests.test_binding_exports.BindingExportAuditTests.test_internal_bridge_and_dataclass_artifacts_use_exact_exclusions -v` | Expected failure: the current exact JNI exclusion lacked its required architecture-boundary citation. |
| Green | `bcc9a476` | `python -B tools/api_manifest/generate.py --write`; focused exclusion test; `python -B -m unittest tools.api_manifest.tests.test_binding_exports -v`; generator `--check`; pinned export audit with the 2026-10-14 report and `--check` | PASS: focused exclusion test 1/1, audit suite 43/43, all 12 generated outputs current, and C/Java/Python scans exact with zero discrepancies. |
| Refactor | Documentation closeout | `python -B -m unittest tools.api_manifest.tests.test_c_api_docs -v`; `git diff --check` | PASS: 3 documentation/report contracts; the current 491/449 Java scan is distinguished from the retained historical 481/442 report. |

This closes the Python facade mapping task only. The complete four-language
shared-fixture comparison remains T029/T071; offline platform loading remains
T028/T072; and API-level discovery remains T077. T024 does not claim complete
operation-level parity or KMIP profile conformance.

## T002, T006, T007, and T068 — Release inventory and owner resolution

KMIP 2.1 defines 57 client-initiated operation types. The 1.0 catalog contains 56 `client_1_0` entries; Set Endpoint Role (§6.1.54) remains cataloged as `client_1_1` per the disposition recorded in specification 025. The release manifest and generator validate the 56-operation 1.0 inventory without changing the normative catalog.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `2efbb946` | `python -B -m unittest tools.api_manifest.tests.test_catalog_parity -v` | Expected failure before implementation: the generator expected 57 release operations while the approved catalog contained 56 `client_1_0` entries. |
| Green | `8c29168e` | Same focused command | PASS: exact in-scope inventory, numeric ordering, catalog identity, and exclusion of the `client_1_1` operation are validated. |
| Red | `c395bb16` | `python -B -m unittest tools.api_manifest.tests.test_operation_readiness.OperationReadinessSchemaTests.test_approved_release_operations_resolve_to_their_current_owner_specs -v` | Expected failure before owner reconciliation: 26 newly approved operation rows still had candidate/unassigned readiness. |
| Red | `62445866` | `python -B -m unittest tools.api_manifest.tests.test_operation_readiness.OperationReadinessSchemaTests.test_feature_owner_id_resolves_through_spec_feature_branch -v` | Expected failure: owner IDs such as KMIPKIT-0036 do not imply a `specs/036-*` directory; resolution must use the exact declared feature branch. |
| Green | `e8443f92` | `python -B -m unittest tools.api_manifest.tests.test_operation_readiness -v` | PASS: current approved owner records resolve by feature-branch metadata; remaining draft owners stay blocked. |
| Refactor | `8b46a35b` | `python -B -m unittest discover -s tools/api_manifest/tests -v`; `python tools/api_manifest/generate.py --check`; `git diff --check` | PASS: 93 tests passed, 3 symlink tests skipped because Windows denied symlink creation (WinError 1314), all 11 generated outputs current, and no whitespace errors. Cached header-only owner resolution removes the full-suite slowdown. |

T068 records the accepted release-scope decision only; it does not implement Set Endpoint Role. The complete Rust export report and manifest reconciliation are now committed. T011 closed after independent read-only QA and the final cross-platform PR matrix on refreshed head `086d7f1e` passed; see the T011 reconciliation evidence below.

## T016 — Runtime metadata mapping correction

The runtime metadata capability now has a public Rust `RuntimeVersion` value with the synchronized product version and ABI revision, and the C ABI reads its stable revision from the same Rust constant. Python's C thread-local error snapshot helpers are underscore-prefixed implementation details; Java's corresponding helpers remain package-private. The public API manifest maps the actual public version/capability surfaces and records the two C-only sticky-error exports as reviewed exclusions from shared facade parity. The C exports remain present and are still covered by FR-007.

QA reviewed the exclusion rationale and the cross-language mappings on 2026-10-12. The review confirmed that a returned Rust `ClientError` is caller-owned and is not equivalent to C's persistent thread-local snapshot/clear behavior. The exclusion therefore applies only to the shared product-facade inventory, not to the required C ABI.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `0328b93d` | `uv run --project bindings/python --with pytest -- python -m pytest bindings/python/tests/test_metadata.py -q` | Expected failure: the native error snapshot helper was still directly importable as a public Python module name. |
| Green | `40d08439` | Same focused command | PASS: 5 tests; internal snapshot helpers are private while safe, non-destructive C snapshot behavior remains tested. |
| Red | `98e930dc` | `cargo test -p kmipkit --test runtime_metadata public_version_matches_the_synchronized_package_version` | Expected compile failure: Rust had no runtime version value containing both product and ABI metadata. |
| Green | `40d08439` | `cargo test -p kmipkit --test runtime_metadata` | PASS: 2 tests; package version, ABI revision, and manifest-ready capability queries are covered. |
| Red | `88c60687`, `d48bca68`, `8c3f97d9` | Focused API manifest inventory tests | Expected failures: the Rust runtime version and ABI declarations were missing from the 1,760-entry Rust inventory and had no four-language mappings. |
| Green | `40d08439` | `uv run --with jsonschema -- python -B -m unittest discover -s tools/api_manifest/tests -v` | PASS: 99 tests, 3 platform-permission symlink skips; all runtime metadata declarations resolve and inventory count is 1,765. |
| Refactor | `dcaa1a32` | `uv run --with jsonschema -- python -B -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_generated_rust_metadata_lists_only_unblocked_c_mapped_capabilities -v` | PASS: generator tests assert exactly the approved eleven output destinations; the obsolete test proposing a twelfth generated file was removed. |

Additional focused verification: `cargo test -p kmipkit-ffi metadata_tests` passed 4 tests; `mvn -f bindings/java/pom.xml -Dtest=RuntimeMetadataTest test` passed 5 tests and ran the offline example; `python -B tools/api_manifest/generate.py --check` verified all 11 generated outputs; `cargo fmt --all --check` and `git diff --check` passed. This evidence closes only the runtime metadata mapping slice; it does not close the full API inventory, binding facade, package matrix, blocked owner specifications, or release gate.

#### UTF-8 input normalization follow-up

The shared capability query now maps unpaired Unicode surrogates consistently to the public invalid-input error. Rust validates the supplied UTF-8 bytes; the C ABI rejects malformed UTF-8 without modifying the output value and records validation/invalid-input with delivery `NONE`; Java rejects malformed UTF-16 input; Python normalizes `UnicodeEncodeError` to `InvalidInputError` without exposing the invalid string. These are local manifest queries and do not send a KMIP request.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `735c36a0` | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_metadata.py -k unpaired_surrogate -q` | Expected failure: Python leaked `UnicodeEncodeError` instead of the manifest-declared `InvalidInputError`. The added C and Java adapter regression tests passed against their existing mappings. |
| Green | `8e011291` | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_metadata.py -q`; `mvn -f bindings/java/pom.xml -Dtest=RuntimeMetadataTest test -q`; `cargo test -p kmipkit-ffi metadata_tests`; `python -B tools/api_manifest/generate.py --check` | PASS: Python 7 tests, Java runtime metadata suite, C metadata tests, and all 11 generated outputs. |
| Refactor | `aa10ba0b`, `cc146e79` | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_metadata.py -q`; `mvn -f bindings/java/pom.xml -Dtest=RuntimeMetadataTest test -q`; `cargo test -p kmipkit-ffi metadata_tests`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | PASS: Python 7 tests, Java runtime metadata suite, C metadata suite (6 tests), formatter, Clippy, all 11 generated outputs, and whitespace check. Clarified invalid-input docs in Python and Java and simplified the FFI UTF-8 error branch without changing behavior. QA review found only the missing Java `@throws` line; it was added and the Java runtime metadata suite passed afterward. |

### Reviewed C-only last-error exclusions

`kmipkit_last_error_get` is declared as the C view of the shared error taxonomy, not excluded as C-only state. Its status output preserves the existing status values: `8` protocol, `9` transport, and `10` complete server result. The accompanying category, cause, and delivery outputs remain separate values. Rust exposes these semantics through `ClientError` and its `category`, `cause_category`, and `delivery_state` accessors; Java exposes `ClientOperationException.statusCode()`, `categoryCode()`, `causeCode()`, and `deliveryStateCode()`; Python exposes the corresponding `ClientExecutionError` fields. The status constants map to the Rust outcome variants and Java exception subclasses, while Python retains the raw status and safe category values.

Only `kmipkit_last_error_clear` remains an exact C-only exclusion. It clears the calling thread's sticky C diagnostic slot and is an ABI mechanism rather than a shared error capability; Rust errors are caller-owned and Java/Python exceptions need no explicit clear operation. Both symbols remain generated and covered by C API tests. These mappings satisfy KMIPKIT-0043-FR-013 together with KMIPKIT-0007-FR-013 without changing runtime codes or delivery semantics.

## T027 — Python native loading and ABI verification

The Python package loads exactly one regular shared library adjacent to its generated CFFI declarations, or the explicit `KMIPKIT_NATIVE_LIBRARY` administrator override. It rejects a missing/ambiguous bundled artifact and a missing override without fallback; it performs no directory search or runtime download. Before exposing the FFI, it queries `kmipkit_version_get` and requires ABI revision 1. Load and ABI errors redact native paths and implementation details.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `874c697a` | `python -m pytest -q bindings/python/tests/test_native_loading.py` | Expected failure: the package-local loader helpers did not yet exist. |
| Green | `6d4f8b3c` | Same focused command | Added the injectable loader with explicit override, no-fallback, ABI mismatch, and redacted-error behavior. |
| Refactor | `ed920459` | Same focused command; reviewed `docs/architecture/ffi-and-bindings.md` | Clarified the administrator override contract and redacted failure expectations without changing loader behavior. |
| Red | `67354b1b`, `b931cdb3` | `python -m pytest -q bindings/python/tests/test_native_loading.py` | Added failing specifications for the canonical C ABI handshake and deterministic package-local native-library discovery before wiring them into the package import. |
| Green | `ac5df387` | `python -m pytest -q bindings/python/tests/test_native_loading.py` | Integrated CFFI loading with the generated package declarations, ABI query, and exact adjacent-library selection. |
| Refactor | `701d3002` | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_native_loading.py -q`; `git diff --check` | PASS: 11 loader tests. Expanded the architecture contract to document recognized library suffixes, regular-file/symlink handling, ambiguity rejection, ABI handshake, and absence of fallback/search/download. |

Final verification on Windows x86-64: `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests -q` passed 46 tests. `maturin build` produced `kmipkit-1.0.0-py3-none-win_amd64.whl`; installing it into an isolated target directory with `uv pip install --target ... --no-deps` and importing from that installed tree returned product version 1.0.0, ABI 1, and `generic_ttlv_model` available. QA independently reviewed the loader and wheel smoke, confirmed the no-fallback/redaction behavior, and approved closing T027. This does not close T028's Java/Python cross-platform CI matrix.

## T013 — Approved shared Rust declaration inventory (complete)

T013 reconciles the canonical public declarations observed across all five published Rust crates with exact manifest paths. It maps 134 declarations whose shared capabilities have Approved/Accepted owners, existing implementations, and linked owner requirements; only the five bounded transport declarations use the accepted ADR-0014 Rust-only exception. The remaining 9,363 observed Rust declarations stay explicitly blocked pending an approved owner or exact parity evidence. This inventory completion does not promote blocked APIs or claim release readiness.

| Published Rust crate | Observed | Mapped | Blocked |
|---|---:|---:|---:|
| `kmipkit-ttlv` | 331 | 36 | 295 |
| `kmipkit-protocol` | 5,845 | 61 | 5,784 |
| `kmipkit-transport` | 365 | 5 | 360 |
| `kmipkit-client` | 1,031 | 23 | 1,008 |
| `kmipkit` | 1,925 | 9 | 1,916 |
| **Total** | **9,497** | **134** | **9,363** |

The 134 mapped declarations comprise 30 generic TTLV model, 6 TTLV codec, 83 vendor-extension registry, 1 Discover Versions request-item builder, 9 FFI runtime-metadata, and 5 Rust-only transport declarations. The `StructureView` type and its view-wrapper lifecycle belong to the generic model's redaction, serialization, ordering, and lifetime requirements (KMIPKIT-0004-FR-006/009/010/012); registry functions that recognize extensions and return a view remain owned by KMIPKIT-0012. `ClientBatchItem` retains KMIPKIT-0012 ownership for extension attachment and inspection while also citing KMIPKIT-0007-FR-001 for the base typed-request contract. Its Discover Versions builders in Rust, C, Java, and Python are separately mapped to KMIPKIT-0007-FR-001/002. The Discover Versions operation itself remains blocked until typed execution and response parity are complete.

The 36 observed foreign-language declarations without exact callable Rust equivalents remain blocked without signatures or mappings: 22 C release/`Drop` rows, 12 TTLV helper rows, and the Java `RuntimeMetadata` and Python `metadata` namespace rows. Those two metadata namespaces now identify KMIPKIT-0043 as their candidate owner; they remain blocked because the Rust crate root is not an exported declaration matching either namespace. Candidate drafts 0009/0010 remain blocked. The generated T013 audit report preserves all 36 blocked cross-language rows and their exact reasons.

The initial approved TTLV ownership slice remains historical evidence within this task. It covers the four-language declaration rows mapped to `kmipkit_ttlv::` (76 generic-model rows owned by KMIPKIT-0004 and 25 `CodecLimits` rows owned by KMIPKIT-0005); it is a subset of the completed five-crate inventory above.

| Initial slice phase | Commit | Command | Result |
|---|---|---|---|
| Red | `0344edaf` | `python -B -m unittest tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_ttlv_declarations_use_their_approved_shared_spec_owners -v` | Expected failure before owner reconciliation: the manifest had no `generic_ttlv_model` capability record, so the test failed with `KeyError`. |
| Green | `c0cc2a31` | `python -B -m unittest discover -s tools/api_manifest/tests -v`; `python tools/api_manifest/generate.py --check`; `git diff --check` | PASS: 94 tests passed, 3 symlink tests skipped because Windows denied symlink creation (WinError 1314); all 11 generated outputs current; no whitespace errors. |
| Refactor | `5d36f87c` | `uv run --with jsonschema -- python -B -m unittest discover -s tools/api_manifest/tests -v`; `python tools/api_manifest/generate.py --check`; `git diff --check` | PASS: 94 tests passed, 3 symlink tests skipped because Windows denied symlink creation (WinError 1314); all 11 generated outputs current; no whitespace errors. Strengthened the inventory assertion to require exactly 101 declarations split 76/25. |

| Full inventory phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `2e3ade0e`, `0bf96eff` | Failing tests for complete shared Rust inventory gates and exact approved vendor-registry mappings | Expected failures before auditor and manifest reconciliation. |
| Green | `f823ccb8` | `.venv/Scripts/python.exe -B -m unittest discover -s tools/api_manifest/tests -p "test_*.py" -v` | PASS: 224 tests; 3 Windows symlink tests skipped (WinError 1314). Reconciled 9,497 exports and generated the shared Rust inventory. |
| Refactor | `c957b94a` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_shared_rust_inventory -v`; `.venv/Scripts/python.exe tools/api_manifest/generate.py --check`; `.venv/Scripts/python.exe tools/api_manifest/audit_shared_rust_inventory.py --check`; `git diff --check` | PASS: initial T013 inventory and documentation checks; all 11 generated outputs and the report were current. |
| Red | `6aaa1d49` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_runtime_metadata_exports_are_inventoried_in_every_language -v` | Expected failure: the blocked Java/Python namespace rows named KMIPKIT-0012 instead of their KMIPKIT-0043 candidate owner. |
| Green | `2f6a0e13` | Same focused test; `.venv/Scripts/python.exe tools/api_manifest/generate.py --check` | PASS: namespace candidate ownership corrected without changing blocked state or adding mappings; 11 generated outputs remained current. |
| Refactor | `3b68ed94` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_runtime_metadata_namespace_exports_remain_blocked_under_0043 -v`; `git diff --check` | PASS: isolated the namespace ownership regression and confirmed both records remain blocked and mapping-free. |
| Red | `18715166` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_shared_rust_inventory.SharedRustInventoryTests.test_runtime_metadata_namespaces_keep_their_candidate_in_blocked_report -v` | Expected failure: the auditor's registry-only candidate filter omitted both namespaces from the blocked inventory report. |
| Green | `d98fa57a` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_shared_rust_inventory -v`; `.venv/Scripts/python.exe tools/api_manifest/audit_shared_rust_inventory.py` | PASS: all 8 inventory tests; regenerated report retains exactly 36 blocked rows, including the two namespaces under KMIPKIT-0043. |
| Refactor | `49450cc4` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_shared_rust_inventory -v`; `.venv/Scripts/python.exe tools/api_manifest/generate.py --check`; `.venv/Scripts/python.exe tools/api_manifest/audit_shared_rust_inventory.py --check`; `git diff --check` | PASS: 8 inventory tests; 11 generated outputs and inventory report current; no whitespace errors. QA passed at this commit. |

The original full API-manifest suite passed 224 tests with 3 Windows symlink-permission skips; the namespace follow-up passed all 33 `test_manifest_v2` tests and all 8 shared-inventory tests. The generator, shared-inventory auditor, and diff checks pass at QA-approved HEAD `49450cc4`. The Rust export report contains 1,883 conditional declarations, including 13 `core::any::Any` lifetime surfaces whose exact predicates are preserved; those rows intentionally remain blocked without signatures or language mappings and are not auditor issues. T011 is closed on refreshed report head `086d7f1e`; it does not leave T013's approved shared-inventory work incomplete.

### T011 — Rust public-export audit (current snapshot)

The pinned Rust 1.94.0 / rustdoc JSON 57 audit report at [rust-export-report-2026-10-12.json](reports/rust-export-report-2026-10-12.json) was regenerated from the current continuation branch on Windows x64 with all Cargo features. The current `audit_rust_exports.py --check` passes: all 9,497 observed paths match, with zero missing, not-observed, kind-mismatch, issue, or conditional-state/condition-mismatch records. This includes 7,614 verified declarations and 1,883 conditional declarations. The 13 `core::any::Any` lifetime surfaces preserve their exact predicates and remain blocked without signatures or language mappings; they are not unresolved audit issues. Independent QA confirmed the report matches the manifest and is canonically serialized. T011 closed after final cross-platform CI passed on refreshed report head `086d7f1e`.

> Historical pre-Green baseline. The following counts describe the audit before the lifetime-condition model was added and are superseded by the Green evidence later in this file.

| Published package | Verified | Conditional | Observed total | Manifest rows | Not observed | Issues |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `kmipkit-ttlv` | 264 | 65 | 329 | 331 | 2 | 2 |
| `kmipkit-protocol` | 4,585 | 1,250 | 5,835 | 5,845 | 10 | 10 |
| `kmipkit-transport` | 275 | 90 | 365 | 365 | 0 | 0 |
| `kmipkit-client` | 895 | 135 | 1,030 | 1,031 | 1 | 1 |
| `kmipkit` | 1,595 | 330 | 1,925 | 1,925 | 0 | 0 |
| **Total** | **7,614** | **1,870** | **9,484** | **9,497** | **13** | **13** |

Historical pre-Green analysis (superseded by the passing current audit below): that earlier run had 13 unresolved `core::any::Any` predicates because lifetime bounds were not yet represented in the manifest condition model. Its 9,484-path counts and issue state are retained only as historical evidence. The later T011 Green work preserves those exact lifetime predicates; the current report matches all 9,497 observed paths with zero issues or mismatches.

The following verification results are also historical pre-Green results, superseded by the current passing audit and full-suite evidence in the T011 Green section below.

| Verification | Command | Result |
|---|---|---|
| Focused auditor tests | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_rust_export_trait_predicates -v` | PASS: 16 conversion, cycle, and conditional reconciliation tests. |
| Full API-manifest tests | `.venv/Scripts/python.exe -B -m unittest discover -s tools/api_manifest/tests -p 'test_*.py' -v` | PASS: 211 tests; 3 Windows symlink tests skipped because symlink creation was denied (WinError 1314). |
| Rust export inventory | `.venv/Scripts/python.exe -B tools/api_manifest/audit_rust_exports.py --repo-root . --report specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json --check` | Report written; exit 1 with 9,484 observed paths (7,614 verified and 1,870 conditional), 9,484 matched, 0 missing, 13 not observed, 0 kind mismatches, and 13 unresolved `Any` lifetime-predicate issues. Exit 1 is the expected open gate. |
| Blocked Rust reconciliation | `.venv/Scripts/python.exe -B tools/api_manifest/reconcile_rust_exports.py --report specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json --write` | Added 49 exact `blocked` records and recorded exact predicates for 1,870 existing blocked conditional rows; no signatures or language mappings were added. |
| Reconciled export inventory | `.venv/Scripts/python.exe -B tools/api_manifest/audit_rust_exports.py --repo-root . --report specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json --check` | Report written; exit 1 with 0 missing, 0 conditional state/condition mismatches, 0 kind mismatches, and 13 `Any` lifetime-predicate issues that remain explicit. |
| KMIPKit FFI MSRV build | `cargo +1.94.0 build --locked -p kmipkit-ffi` | PASS on Windows x86-64; built `target/debug/kmipkit_ffi.dll` with the vendored `aws-lc-sys@0.45.0`. Optional NASM/stdalign probes emitted warnings; the build completed successfully. |
| MSRV C ABI allowlist | `.venv/Scripts/python.exe -B tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --c-binary target/debug/kmipkit_ffi.dll --check` | PASS: complete C audit, 180 exact manifest rows, 2 approved C-only exclusions, no missing/stale/kind/issues. |
| C consumer locally | `cmake -G "Visual Studio 17 2022" -A x64 -S bindings/c -B build/c-consumer` | Not run: CMake is not installed on this host. The GitHub C matrix remains the C-consumer verification path. |
| Generated manifest outputs | `python -B tools/api_manifest/generate.py --check` | PASS: all 11 generated outputs current. |
| Whitespace validation | `git diff --check` | PASS. |

| Phase | Commit | Evidence |
|---|---|---|
| Red | `910f799c`, `ef257e79` | Reconciliation tests failed on missing exact blocked rows and absent nearest-owner inheritance, with assertion failures and no harness errors. |
| Green | `6c19b62a` | Added 7,681 exact blocked rows; the audit matched all 9,448 observed paths and preserved all 39 unresolved-predicate issues. |
| Refactor | `d792bc81` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_rust_export_reconciliation -v`; `.venv/Scripts/python.exe -B tools/api_manifest/generate.py --check`; `git diff --check` — PASS: 7 reconciliation tests passed, all 11 generated outputs were current, and the shared key extraction preserved the exact package/path pair. |

Historical snapshot note: at that earlier point T013 covered only the approved TTLV owner slice. The current five-crate T013 reconciliation and remaining T011 limitations are recorded above; the historical counts in this subsection do not supersede the current T013 report.

### C/Java/Python public-export audit (local snapshot passes)

The deterministic non-Rust audit is checked in at [binding-export-report-2026-10-12.json](reports/binding-export-report-2026-10-12.json). On Windows x86-64, `.venv/Scripts/python.exe -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check` completed with exit code 0. All three scanners are complete; every observed declaration reconciles to its manifest row or exact approved exclusion, with no missing, stale, kind-mismatch, or issue records. The native ABI regression test detected nine unprefixed jitterentropy exports before the vendor patch and passed after it. CI now runs the C header/binary/manifest allowlist audit on Windows, Linux, and macOS against the C job's already-built library; cross-platform results remain pending remote CI.

| Package | Scanner complete | Observed | Manifest | Exact matches | Missing | Stale | Kind mismatches | Issues |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| C | yes | 182 | 180 | 180 | 0 | 0 | 0 | 0 |
| Java | yes | 329 | 329 | 329 | 0 | 0 | 0 | 0 |
| Python | yes | 274 | 274 | 274 | 0 | 0 | 0 | 0 |

C's observed count includes `kmipkit_last_error_clear` and `kmipkit_last_error_get`, represented by exact approved exclusions because they are C-specific diagnostics rather than shared four-language capabilities. The C ABI patch removes the nine transitive `aws_lc_0_45_0_jent_*` symbols from the published dynamic library while retaining jitterentropy. Java and Python now reconcile their compiled/source public export surfaces exactly with manifest v2. The checked-in report preserves exact paths, kinds, locations, and counts for all three languages.

The binding-auditor changes have distinct failing-test and implementation commits, including the false-include-guard parser case and the real shared-library export leak. At the checkpoint represented by the original ABI Green commit `a7ecd323`, the separate behavior-preserving vendor Refactor had not yet been recorded; that statement is historical and does not close T011.

| Phase | Commit(s) | Evidence |
|---|---|---|
| Red | `c4658505`, `ea830f13`, `e30697c9` | Added failing cases for binary C ABI exports, protected Java declarations, Python export resolution, and ordinal-only PE exports. |
| Green | `22550fc2` | Implemented C header/binary reconciliation, Java `javap -protected` inventory, and Python source export inventory. |
| Red → Green | `b092775c` → `e62a88fe` | C enum and Python public alias traversal tests, then inventory support. |
| Red → Green | `b117cda0` → `56b1641f` | Python same-path module alias test, then duplicate-path suppression. |
| Red → Green | `8fc86d3b` → `f92a13d7` | C tagged struct/union declaration tests, then aggregate inventory and fail-closed parsing. |
| Red → Green | `f53fdb96` → `0be674d1` | C multi-field declaration test for `int left, right;`, then inventory of both fields. |
| Red → Green | `53f34dc5` → `cf816836` | Python augmented `__all__` test, then literal `+=` export resolution and unresolved-import reporting. |
| Refactor | `9d0c18ef` | Extracted Python class-member inventory without changing behavior. At that earlier checkpoint the full export audit still exited 1; later Green commits resolved the binding discrepancies. |
| Red | `1395106`, `2ae084ce`, `ba3cafbe` | Added failing ABI leak, incomplete C include-guard, and C-only existing-binary audit cases. The ABI regression enumerated the nine AWS-LC jitterentropy exports before patching. |
| Green | `22b13a7a`, `a7ecd323` | Made include-guard recognition fail closed; vendored and patched `aws-lc-sys@0.45.0`, constrained the dependency-policy exception, removed leaked symbols from the published ABI, added the cross-platform C audit step, and reconciled C/Java/Python manifests. `python -B -m unittest discover -s tools/api_manifest/tests -v` passed 162 tests (3 skipped); the full binding audit `--check` exited 0; generator check verified 11 outputs. |
| Refactor | Pending | A behavior-preserving cleanup commit is still required for the vendor ABI change. |

T011 remains unchecked pending read-only QA and remote CI. The latest Rust snapshot records every compiler-observed public path exactly; conditional rows remain explicitly blocked, and no missing or stale path, kind mismatch, duplicate declaration, unsupported ABI export, or incomplete traversal is suppressed.

## T030 — Java/Python package metadata and user scope documentation

Java Gradle/Maven metadata and Python project metadata use the manifest's synchronized `releaseLine` value (`1.0.0`). The Java target is Java 17; Python package metadata and the 1.0 documentation target CPython 3.12 only. Bilingual package guides record the five Java/Python OS/architecture targets from the manifest, explain Java and Python native loading, explicit overrides, ABI 1 checks, redacted failures, and the absence of runtime executable-code downloads. They mark all six current manifest capability states and all 56 `client_1_0` operations (`blocked`) without claiming full operation parity or a published package release. Existing English and Spanish vendor-extension guides now distinguish implementation examples from the still-blocked release-parity status.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `79764c1a` | `python -B -m unittest tools.api_manifest.tests.test_t030_package_docs -v` | Expected failure: Java/Python version metadata was already aligned, but the bilingual runtime/loading/capability guides were absent. |
| Green | `cae998db` | `python -B -m unittest tools.api_manifest.tests.test_t030_package_docs -v` | PASS: 4 focused contract tests; added package metadata descriptions, synchronized versions, and bilingual package guides. |
| Refactor | `aba0b9d6` | `python -B -m unittest tools.api_manifest.tests.test_t030_package_docs -v`; `python -B tools/api_manifest/generate.py --check`; `mvn -o -f bindings/java/pom.xml validate -q`; `git diff --check` | PASS: 5 focused contract tests; all 11 generated outputs current; Maven metadata validation and whitespace check pass. Clarified that Python 3.12 is the only 1.0 package minor line and that vendor-extension examples do not establish verified release parity. |

The platform-matrix CI and offline load/install verification remain T028/T072 work; T030 documents the currently declared target matrix without claiming that every target has completed those checks.

## T011 Green — Rust conditional lifetime predicates

The pre-Green Rust counts above are historical. The condition schema now uses a strict object with `subjectBindings` and ordered `predicates`; predicates preserve exact `outlives` bounds and trait bounds, including the `maybe` modifier used by `?Sized`. The reconciler accepts only this closed shape and keeps conditional declarations blocked without signatures, language mappings, or approval data. The fixture is regenerated from authentic Rust 1.94.0 rustdoc JSON and proves both the open `View<'a>` condition and the satisfied `View<'static>` control. The auditor fails closed if rustdoc omits the exact `T: 'static` bound for the canonical `core::any::Any` implementation.

The current snapshot has 7,614 verified declarations and 1,883 conditional declarations (including all 13 Any lifetime surfaces). All 9,497 compiler-observed paths match the manifest; missing, not-observed, kind mismatch, issue, and conditional state/condition mismatch counts are zero. All 1,883 conditional rows remain blocked without signatures or mappings. T011 is not complete until read-only QA and remote CI finish.

| Verification | Command | Result |
|---|---|---|
| Red+Green focal tests | `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_rust_export_trait_predicates tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_blocked_condition_schema_round_trips_lifetime_and_trait_bounds -v` | PASS: 22 tests, including exact predicates, open/static lifetime controls, fail-closed mutation, strict schema round-trip, and blocked reconciliation. |
| Rust export inventory | `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B tools/api_manifest/audit_rust_exports.py --repo-root . --report specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json --check` | PASS: 9,497 observed and matched, including 7,614 verified and 1,883 conditional; all missing/not-observed/mismatch/issue counts are zero. |
| Conditional reconciliation | `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B tools/api_manifest/reconcile_rust_exports.py --report specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json --write` | PASS: updated 1,883 existing condition records; added 0 records; no signatures or mappings were added. |
| Determinism | Reproject Rust 1.94.0/rustdoc JSON 57 fixture; rerun the Rust inventory to a temporary report and compare SHA-256 | PASS: fixture projection `58CAE1A346AADE81AC13BBBCB9DC02090A98BD12187D9C80EBD391045014DCC2`; checked-in and regenerated report both `A4D34F6244C3C690182C38CB42D5DBBE8E92E99D6A3223E971845A0978C1B854`. |
| Generated outputs | `python -B tools/api_manifest/generate.py --check` | PASS: all 11 generated outputs current. |
| Full API-manifest suite | `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest discover -s tools/api_manifest/tests -v` | PASS: 217 tests; 3 Windows symlink tests skipped because creation was denied (WinError 1314). |
| Whitespace validation | `git diff --check` | PASS at the latest local check; rerun before commit. |

## T011 — Initial CI regression repair (remote verification pending)

The initial PR run `38288466279` on `2e3157fe` failed 11 root jobs and two derivative gates. The root failures reduced to two defects: `extension_registry.c` referenced two category/cause identifiers that are not declared by the public C ABI, and the temporary offline guide-example project did not carry the workspace's local `aws-lc-sys` patch. The coverage gate and run-summary failures were downstream of the failed root jobs.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `aa990084` | `.venv/Scripts/python.exe -B -m unittest discover -s scripts/tests -p test_user_guide_examples.py -v` | Expected failure: 5 existing tests passed and the new patch-inheritance regression failed because the temporary manifest omitted `[patch.crates-io] aws-lc-sys`. The initial CI run also showed the C consumer compile failure at the undefined identifiers. |
| Green | `971988eb` | Same script unit suite; `.venv/Scripts/python.exe -B scripts/test_user_guide_examples.py`; `cargo build --locked -p kmipkit-ffi`; MSVC C consumer compile and execution | PASS: 6 script tests; all 20 marked guide examples compiled with Cargo offline; the C consumer compiled against the public header and FFI library and all 14 consumer cases passed. The [public header](../../bindings/c/include/kmipkit.h#L324) documents the category, cause, and delivery values that the consumer verifies. |
| Refactor | `62c05a1a` | Script unit suite; offline guide-example compilation; `git diff --check` | PASS: 6 script tests and 20 offline examples; the regression parses TOML semantically rather than depending on formatting. No runtime behavior changed in this phase. |

Remote CI has not yet rerun on these commits. T011 remains open until the complete PR matrix and a fresh read-only QA review pass.

The Windows C consumer check used the installed Visual Studio 2022 x64 developer environment to compile `bindings/c/tests/extension_registry.c` against `bindings/c/include` and `target/debug/kmipkit_ffi.dll.lib`, then executed `target/debug/extension_registry_consumer.exe`. CMake is unavailable on this host, so the CI-configured CMake matrix remains the cross-platform build authority.

## T011/T028 — Coverage gate and offline native-loader CI follow-up

The PR run `38298147704` on `e3462e91` passed its core, language, sanitizer, dependency-policy, normative-inventory, documentation, and most coverage jobs. The selected-scope coverage gate failed because Cobertura included generated Python CFFI wrappers under `kmipkit/_native_ffi/`, while `bindings/python/pyproject.toml` mistakenly excluded the handwritten `kmipkit/_ffi/` adapter instead. The coverage gate correctly rejected the generated source path; its validation was not weakened. The corrected configuration excludes only `_native_ffi/__init__.py` and `_native_ffi/ffi.py`, leaving handwritten `_ffi` code in measured coverage.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `20c94285` | `scripts/tests/test_multilanguage_coverage.py` | Expected failure: the regression test found that configured omissions did not match the generated `_native_ffi` source paths in Cobertura. |
| Green | `167c774e` | `scripts/tests/test_multilanguage_coverage.py`; full `scripts/tests` suite | PASS: 16 focused tests; 258 script tests passed and 26 were skipped. The configured omissions now target only generated wrappers. |
| T028 test coverage | `9fbfaddc` | `mvn --batch-mode --file bindings/java/pom.xml -Dtest=NativeLibraryLoaderTest test`; Python `test_native_loading.py` via `uv run` | PASS: 10 Java tests and 12 Python tests. Added assertions for redacted unsupported-target errors, missing/ambiguous artifact paths, and simulated wrong-architecture load errors with no fallback. |
| Read-only QA | `9fbfaddc` | Static re-review of the Java/Python loader failure tests | PASS: all three previously identified test gaps are addressed. QA did not execute tests or modify files. |

The three commits above have not yet been verified by remote CI at this evidence checkpoint. T028 remains unchecked until its Java and Python platform matrices pass on Ubuntu, Windows, and macOS. T011 also remains unchecked until the regenerated export audits, full coverage gate, and final PR CI pass on the pushed head and read-only QA signs off. The prior run is failure evidence only; a fresh successful run is required.

## T011 — Disable unused setup-uv cache in C ABI inventory jobs

Run `38299757432` on `7b26939f` showed that the C consumer, ABI export audit, C compilation, and both C consumer tests passed on Ubuntu, Windows, and macOS. Each matrix job then failed only in `Post Set up Python 3.12 for the ABI inventory`: setup-uv's post-job cache save rejected a cache directory that did not exist because the job installs no uv dependencies. The job only needs setup-uv to provide Python 3.12 for the ABI audit, so caching is disabled for this setup step alone.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `7feb4cb5` | `python -B -m unittest scripts.tests.test_workflow.WorkflowContractTests.test_c_abi_inventory_does_not_cache_unused_uv_dependencies -v` | Expected failure: the `language-c` setup step did not declare caching behavior. |
| Green | `c8d0efbc` | Focused workflow contract test; `python -B -m unittest scripts.tests.test_workflow -v`; `git diff --check` | PASS: focused regression passes; all 41 workflow contract tests pass; whitespace check is clean. |
| Read-only QA | `c8d0efbc` | Local review of the workflow diff, test, and adjacent commit history | PASS: only the C ABI inventory setup disables caching; Red and Green are separate DCO-signed commits. |

The setup-uv correction is pushed. Run `38300751595` is checking commit `b79f2f81` and is still in progress at this evidence checkpoint; it does not include the Python coverage correction recorded below. T011 and T028 remain open pending a complete successful run on the final fixed head and passing language/platform matrices.

## T011 — Exercise generated Python API metadata in coverage

PR run `38299757432` also found that `_generated/public_api.py` was selected as eligible handwritten/check-in Python source by the coverage gate but did not appear in the Cobertura report. Keep the generated module measured: its public declaration metadata is now imported and checked by the Python manifest tests, and the coverage contract guards against silently dropping that exercise.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `f7c787c0` | `python -B -m unittest test_multilanguage_coverage.MultiLanguageCoverageTests.test_python_coverage_tests_execute_the_generated_public_api_manifest -v` from `scripts/tests` | Expected failure: the Python manifest suite did not import `public_api`. |
| Green | `6677ea95` | Focused Python manifest tests with pinned coverage requirements; full Python coverage suite; `python -m unittest test_multilanguage_coverage -v` | PASS: 2 focused tests; full Python suite 48/48 and 97.22% coverage; the XML contains `bindings/python/src/kmipkit/_generated/public_api.py`; all 17 multi-language coverage contract tests pass. |
| Read-only QA | `001d3dc3` | Review of the Red/Green change, coverage contract, guide, and evidence | PASS: no findings. The QA reviewer noted the source-text import guard is intentionally narrow; the aggregate coverage gate remains the final check that the module is actually measured. |

The testing guide now documents that only the two generated `_native_ffi` wrappers are excluded and that the checked-in public API manifest stays inside the 85% coverage gate. The full suite produced local Windows evidence; the Linux CI run on the pushed head remains authoritative. T011 remains open pending final CI and read-only QA.

## T028 — Offline Java native package smoke in the platform matrix

The Java matrix already tests loader errors and builds the JNI library on Ubuntu, Windows, and macOS, but did not launch the packaged classifier JAR. The CI job now runs `nativeJarSmoke` after Java tests on every runner. Gradle defaults the package target and library path to the current host and the JNI binary built by the preceding Maven step; its forked `JavaExec` starts a fresh JVM with the classifier JAR, and the smoke class loads the bundled JNI library without a server or runtime download. The Gradle invocation uses offline dependency resolution.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `8f2cb282` | `python -B -m unittest test_workflow.WorkflowContractTests.test_java_package_loader_smoke_runs_offline_on_every_platform -v` from `scripts/tests` | Expected failure: the Java workflow had no packaged native loader smoke step. |
| Green | `4e4fd6e5` | Same workflow contract; `python -B -m unittest test_workflow -v`; `bindings/java/gradlew.bat --offline nativeJarSmoke` | PASS locally: all 42 workflow contracts pass; the Windows classifier JAR launches a fresh JVM and prints `bundled JNI loaded`. QA then identified that the Unix wrapper is tracked non-executable. |
| Red | `9e80cb62` | Same focused workflow contract | Expected failure: the contract now requires `sh ./gradlew` because Git tracks `bindings/java/gradlew` as mode `100644`. |
| Green | `854a5843` | Full workflow contract suite; offline Java package smoke on Windows; `git diff --check` | PASS locally: 42 workflow contracts and Windows packaged smoke pass. Linux/macOS invocation is through `sh`; final matrix verification is pending. |
| Read-only QA | `854a5843` | Review of both CI fixes and the platform-specific command | PASS: Unix invokes the non-executable wrapper via `sh`; Windows keeps the `.bat` entry point; the workflow contract matches. |

T028 remains unchecked until the updated Linux, Windows, and macOS CI jobs finish successfully.

### T028 follow-up — Unix resource filename mismatch

Run `38301807549` on `46f7a1da` reached the packaged Java smoke on Ubuntu and exposed a loader/package contract defect: the loader requested `kmipkit_jni.so`, while the Gradle classifier contains `libkmipkit_jni.so`. The matching macOS filenames had the same missing `lib` prefix. This was independent of the wrapper invocation fix; the Windows filename was already correct.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `39782570` | `NativeLibraryLoaderTest.resolvesUnixLibraryNameFromBundledResources` | Expected failure before the fix: the Unix loader selected a name that does not exist in the packaged resources. |
| Green | `14b8dc10` | `mvn --batch-mode --file bindings/java/pom.xml verify`; Linux Gradle `nativeJarSmoke` under WSL Ubuntu 26.04 | PASS locally: 48 Java tests, JaCoCo coverage, and the vendor example passed; the Linux classifier JAR loaded its bundled JNI library and printed `bundled JNI loaded`. |
| Refactor | `18abc7dd` | Same Maven verification; expanded filename test for x86_64/aarch64 Linux and x86_64/aarch64 macOS | PASS; Windows naming remains unchanged. |
| Read-only QA | `18abc7dd` | Review of loader mapping, platform resource names, and temporary diagnostic cleanup | PASS; no findings and no diagnostic prints remain. |

The Linux smoke failure from `38301807549` is historical evidence on the pre-fix head. A fresh remote CI run on the updated commits is required; leave T028 and T011 unchecked until all required checks on the final pushed head pass.

## T028 — Java and Python offline packaged-consumer probes

The CI keeps the approved `ubuntu-latest`, `windows-2022`, and `macos-latest` matrix. Java's `nativeJarSmoke` now launches isolated JVM consumers against the classifier JAR and verifies packaged load success, a JAR with native resources removed, unsupported `os.arch` selection, and an ABI-2 native fixture. The Java guard blocks socket connect/listen/accept checks; each subprocess reports zero network attempts. The ABI fixture is compiled locally with `rustc`, returns ABI 2 through both KMIPKit's C version symbol and the JNI ABI query, and its digest replaces the packaged checksum so the ABI validation path is reached.

Python CI now builds a wheel with Cargo offline, primes a fresh Python 3.12 consumer environment with CFFI, installs the wheel using `uv pip install --offline --no-deps`, then runs the installed package under `python -I`. The consumer process blocks socket connect, connect-ex, send-to, DNS resolution, and server socket operations, and records any attempted connection. It exercises successful packaged loading, a missing native library, and the ABI-2 fixture; its diagnostics must remain actionable and redact both installed-library and fixture paths. Build/dependency priming is offline as well, while the no-network assertion specifically covers the installed consumer runtime.

Architecture evidence is deliberately narrow. The Java packaged probe verifies that an unsupported architecture is rejected during target selection; it does not pretend that this is a real foreign-machine shared library load. The Python packaged smoke does not load a foreign-architecture binary. `test_wrong_architecture_binary_failure_is_redacted` retains explicit loader-level coverage by injecting an architecture-specific loader error and asserting actionable, path-redacted output. A genuine foreign-architecture shared library was not available for the local Windows run.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `ad5f0685` | `$env:PYTHONPATH='bindings/python/src'; & '.\bindings\python\.venv\Scripts\python.exe' -B -m pytest -p no:cacheprovider -q --tb=short bindings/python/tests/test_native_loading.py -k 'wrong_architecture or abi_mismatch'` | Expected failure: 2 assertions rejected generic, non-actionable loader/ABI messages. |
| Red | `66809f0c` | `python -B -m unittest scripts.tests.test_workflow.WorkflowContractTests.test_java_package_loader_smoke_runs_offline_on_every_platform scripts.tests.test_workflow.WorkflowContractTests.test_python_installed_wheel_smoke_runs_offline_in_isolated_environment -v` | Expected failure: 6 missing workflow/package-smoke contract assertions. |
| Green | `00703bc1` | Focused Python native-loader suite; both workflow contracts; Java `nativeJarSmoke`; Java `compileTestJava`; Python installed-wheel smoke | PASS locally on Windows. Python loader tests: 14 passed. Both workflow contracts passed. Java packaged success, missing-resource, unsupported-architecture, and wrong-ABI probes passed offline. `compileTestJava` passed. The Python CPython 3.12.14 wheel loaded successfully and passed missing-library and ABI mismatch probes offline. |
| Refactor/evidence | this commit | Re-run the focused loader suite, both packaged-workflow contracts, Java packaged smoke, Python installed-wheel smoke, and `git diff --check` after the harness cleanup | PASS: 14 Python loader tests; 2 workflow contracts; Java packaged smoke and Python installed-wheel smoke passed; no whitespace errors. The updated Java missing-resource variant now asserts that it removed exactly one native library, and the Python consumer guard also blocks UDP sends and socket server operations. |

Focused local commands and results from the repository root:

```text
$env:PYTHONPATH='bindings/python/src'; & '.\bindings\python\.venv\Scripts\python.exe' -B -m pytest -p no:cacheprovider -q --tb=short bindings/python/tests/test_native_loading.py
python -B -m unittest scripts.tests.test_workflow.WorkflowContractTests.test_java_package_loader_smoke_runs_offline_on_every_platform scripts.tests.test_workflow.WorkflowContractTests.test_python_installed_wheel_smoke_runs_offline_in_isolated_environment -v
git diff --check
```

Results: `14 passed`; both named workflow contract tests passed; and `git diff --check` returned exit 0. From `bindings/java`, `./gradlew.bat --offline compileTestJava` passed, followed by the `nativeJarSmoke` command above. Re-running the consumer helper against the already installed isolated wheel with the expanded network guard printed `installed Python wheel load, missing-library, and ABI mismatch probes passed offline` (exit 0).

Local Windows Java commands, run from `bindings/java`:

```text
./gradlew.bat --offline "-PnativeLibrary=target/native/kmipkit_jni.dll" nativeJarSmoke
./gradlew.bat --offline compileTestJava
```

The native library argument pointed at the existing `target/native/kmipkit_jni.dll`; Gradle only read that artifact and wrote smoke outputs under `bindings/java/build/`. The packaged smoke printed `packaged JNI load and negative probes passed offline`; Gradle reported six tasks successful. Java ran locally on 21.0.12, with compilation targeting release 17. The Java 17 CI matrix remains authoritative.

Local Python wheel sequence, run from `bindings/python` with a fresh temporary smoke root and `CARGO_TARGET_DIR` outside the worktree:

```text
maturin build --offline --release --out <smoke-root>/wheel
rustc --edition=2024 --crate-name kmipkit_wrong_abi --crate-type cdylib ../../tools/native_package_smoke/wrong_abi.rs -o <smoke-root>/wrong-abi.dll
uv venv --offline --python 3.12 <smoke-root>/consumer-env
uv pip install --offline --python <smoke-root>/consumer-env/Scripts/python.exe cffi==1.17.1
uv pip install --offline --no-deps --python <smoke-root>/consumer-env/Scripts/python.exe <smoke-root>/wheel/kmipkit-1.0.0-py3-none-win_amd64.whl
python -B ../../tools/native_package_smoke/python_smoke.py --consumer-python <smoke-root>/consumer-env/Scripts/python.exe --wrong-abi-library <smoke-root>/wrong-abi.dll --smoke-root <smoke-root>
```

Result: Maturin built the wheel offline; CFFI 1.17.1 and the wheel were installed offline; output was `installed Python wheel load, missing-library, and ABI mismatch probes passed offline` (exit 0). The workflow contracts assert that the wheel smoke and Java packaged probes are wired to the same three CI OS labels. Remote Ubuntu/macOS results and a fresh final-head three-OS CI run are pending; T028 remains unchecked until that matrix passes.

### T028 QA correction — retain the Java smoke matrix in the contract

| Phase | Verification | Result |
|---|---|---|
| Red | Temporarily remove `macos-latest` from the Java job matrix and run `WorkflowContractTests.test_java_package_loader_smoke_runs_offline_on_every_platform` | Expected failure: the contract reports the required three-platform matrix missing. The workflow file was restored byte-for-byte immediately afterward. |
| Green | Run the focused Java workflow contract, then `python -m unittest scripts.tests.test_workflow -v` | PASS: focused contract passes against the restored workflow; all 44 workflow contracts pass. |

This guards the Java smoke's Ubuntu, Windows, and macOS coverage against accidental matrix removal. It does not replace final-head remote CI.

## T011 — CI follow-up for capability, generated C docs, and schema payload safety

The subsequent PR run `38295045556` exposed three independent regressions. First, the manifest correctly declared the C-mapped `vendor_extension_registry` capability while the runtime query and FFI inventory test omitted it. Second, the checked-in generated C header had last-error documentation that was not represented in the manifest or generator. Third, QA demonstrated that a C documentation string containing a backslash-newline splice could escape the generated comment even though `*/` was rejected. Each behavior has a separate Red and Green commit; T011 stays open until remote CI on the new head passes.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `19b5eb54` | Runtime metadata and FFI capability inventory tests | Expected failure: the nonblocked C-mapped `vendor_extension_registry` capability was missing from runtime availability. |
| Green | `04ab5dfe` | Same focused tests | PASS: capability discovery and the FFI inventory now agree with the manifest; QA review found no issue. |
| Red | `a0f81de9` | Generated C-header documentation contract test | Expected failure: generation did not preserve the existing last-error snapshot documentation. |
| Green | `3bbdcb19` | Focused generator test and `python -B tools/api_manifest/generate.py --write` | PASS: manifest-backed C documentation regenerates the header; 11 generated outputs are current. |
| Red | `1d1c6c42` | C-comment splice payload regression test | Expected failure: schema validation accepted `*\\\r/#define KMIPKIT_INJECTED 1/*`. |
| Green | `4ddeb0f3` | Focused rendering/security tests and Draft 2020-12 schema validation | PASS: C documentation rejects comment terminators, CR/LF, and backslashes; QA re-review confirmed the custom validator and JSON Schema both reject the payload and found no remaining early-comment-close path. |

Fresh local verification on this head:

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all --check` | PASS. |
| Rust lint | `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | PASS. |
| Rust workspace tests | `cargo test --locked --workspace --all-features` | PASS, exit 0; all executed tests and doctests passed. Three Cosmian KMS integration tests were ignored because the local KMS is not installed. |
| API-manifest tests | `.venv/Scripts/python.exe -B -m unittest discover -s tools/api_manifest/tests -p 'test_*.py'` | PASS: 228 tests, 3 skipped because Windows denied symlink creation. |
| Generated outputs | `python -B tools/api_manifest/generate.py --check` | PASS: all 11 outputs current. |
| Coverage-gate preflight | `scripts/coverage_gate.py preflight --workspace .` | PASS: exit 0, eligible; source scan found 120 executable or ambiguous function bodies across 186 files. The PR's remote coverage jobs remain authoritative. |
| Read-only QA | Review of the capability, generated-documentation, and C-comment payload fixes | PASS; no remaining injection path or contract inconsistency identified. |
| Remote PR matrix | PR #100 CI run `38334999913` on pre-refresh head `e12a35fe` | PASS: 34 checks succeeded; the nightly informational and active-release-schedule checks were skipped by configuration. A refreshed Rust export report now requires a new run before T011 closes. |

## T013 — Post-audit Discover Versions metadata reconciliation

An independent QA review found that the full v2 manifest's Discover Versions request-item builder had stale owner requirements, a private Rust path, and a JNI symbol that did not match the native bridge. It also found that the Rust runtime query omitted the newly declared request-item capability. The correction maps the builder to KMIPKIT-0007-FR-001/002 and KMIPKIT-0043-FR-004/021, records the public Rust path and actual `NativeExtensionRegistry` JNI symbol, and keeps the operation itself blocked. The compatibility projection continues to reproduce the approved 0012 v1 registry fixture exactly.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `148410db`, `7dd2ce67` | Rust runtime metadata test and `cargo test --locked -p kmipkit-ffi --lib capability_query_only_reports_c_mapped_nonblocked_capabilities -- --nocapture` | Expected failures: Rust omitted `discover_versions_request_item` from runtime availability and the FFI inventory found that a ready generated capability was not available through the Rust facade. |
| Red | `bf5ca24e` | `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_discover_versions_function_maps_the_public_item_builder -v` | Expected failure: the manifest pointed through private `kmipkit_client::execute` rather than the public path and recorded a non-existent JNI bridge. |
| Red | `7c16b1ac` | Full API-manifest suite before projection fix | Expected failures exposed the v1 registry projection drift and a check that incorrectly treated 0007/0043 requirements as part of the 0012-only requirement slice. |
| Green | `c1e601ba` | Focused capability/provenance tests; v1 projection, parity, and extension-fixture tests; API generator and extension-fixture checks | PASS. Runtime, manifest, generated outputs, bilingual docs, and the exact v1 compatibility projection are reconciled; Discover Versions remains blocked. |
| Refactor | `b23dd5b1` | Registry-slice and Discover Versions provenance tests; API generator `--check`; extension-fixture generator `--check`; `git diff --check` | PASS. Extracted the v1 function projection into a dedicated helper without changing generated bytes. |
| Read-only QA | `b23dd5b1` and evidence update | Independent review of runtime parity, ownership and requirement IDs, Rust/JNI paths, operation readiness, v1 projection, generated fixture fingerprints, inventory counts, and recorded verification | PASS; no actionable findings. |

Fresh local verification after the Green change:

| Check | Command | Result |
|---|---|---|
| API-manifest suite | `.venv/Scripts/python.exe -B -m unittest discover -s tools/api_manifest/tests -p 'test_*.py'` | PASS: 243 tests run; 240 passed and 3 skipped because Windows denied symlink creation. |
| Changed Rust packages | `cargo test --locked -p kmipkit -p kmipkit-ffi` | PASS: 38 tests; no failures. |
| Workspace lint | `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | PASS. |
| Formatting | `cargo fmt --all --check` | PASS. |
| Full workspace tests | `cargo test --workspace --all-features` | Command exited 1: three `kmipkit-test-support::transport_fixtures` tests could not bind loopback sockets (Windows error 10013, access denied). Remote platform CI remains required. |
| API generator | `.venv/Scripts/python.exe -B tools/api_manifest/generate.py --check` | PASS: 12 generated outputs current. |
| Extension fixture generator | `.venv/Scripts/python.exe -B tools/extension_fixtures/generate.py --check` | PASS. |
| Shared Rust inventory | `.venv/Scripts/python.exe -B tools/api_manifest/audit_shared_rust_inventory.py --report specs/043-public-api-bindings/reports/shared-rust-inventory-2026-10-12.json --check` | PASS; report current. |
| Binding exports | `.venv/Scripts/python.exe -B tools/api_manifest/audit_binding_exports.py --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check` | PASS; exit 0. |
| Remote PR matrix | PR #100 CI run `38334999913` on pre-refresh head `e12a35fe` | PASS: all 34 required checks succeeded, including three-platform core tests and coverage; the two nightly/inactive schedule checks were skipped by configuration. A refreshed Rust export report now requires a new run before T011 closes. |

## T011 — Rust export report reconciliation

Independent QA found that 25 matched rows in the checked-in Rust export report retained `declarationState: blocked` after the current manifest had promoted those declarations to `declared`. Regenerated the report with the pinned `tools/api_manifest/audit_rust_exports.py` auditor; no report content was edited by hand. The refresh also updated source line locations following the T013 API metadata changes.

| Check | Command | Result |
|---|---|---|
| Rust export report regeneration and audit | `.venv/Scripts/python.exe -B tools/api_manifest/audit_rust_exports.py --report specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json --check` | PASS: 9,497 observed and matched; zero missing, not-observed, kind/state mismatches, or issues. |
| Report-to-manifest comparison | Recompute `audit_rust_exports.compare(report, manifest)` and compare matched rows by `_comparison_key` | PASS: 9,497 matched, zero report/manifest row differences, zero issues. |
| Read-only QA after report refresh | Independent comparison of all matched rows, inventory totals, T011 claims, and generated artifact | PASS: all 9,497 rows match current manifest state; 25 stale states are gone; counts and audit totals are coherent. |
| Independent canonical reconstruction | Recompute auditor comparison and serialize with its canonical JSON settings | PASS: report bytes reproduce exactly from the observed exports and current manifest; 9,497/9,497 rows match; zero issues. |
| Remote PR matrix after report refresh | [PR #100 CI run 38336245642](https://github.com/NeverWe1come/KMIPKit/actions/runs/38336245642), head `086d7f1e` | PASS: 34 checks succeeded, zero failed, and the two configured nightly/inactive schedule checks were skipped. The required cross-platform core, coverage, binding, script-contract, and selected-scope coverage checks passed. |

## T014 — Fixed-width C declarations and ABI preservation

T014's implementation was already present in PR #100 but its task checkbox and evidence had not been closed. The generated public C header is derived from ready manifest declarations; blocked declarations remain absent from the public header. Existing symbols and the legacy opaque registry-limits alias remain intact, and the versioned limits representation uses fixed-width fields. This closes only the C declarations and ABI-preservation slice; C lifecycle handling and compiled consumer coverage remain tracked by T015–T019.

| Phase | Commit(s) | Evidence |
|---|---|---|
| Red | `26feafad`, `9806a8a2` | Added failing tests requiring blocked C declarations to be hidden from the public header. `a45de7a7` added failing coverage for ready TTLV constants and registry limit fields; `d2e09c3f` required the unused legacy limits alias to remain opaque; `8b0fd9bc` required versioned limits at C call boundaries. |
| Green | `58849bd6`, `5b7195c4`, `33bd2938` | Filtered generated C declarations by manifest readiness, published ready fixed-width TTLV constants, and added the versioned registry-limits representation while retaining the historical opaque alias and symbols. |
| Refactor | `cf5a0626` | Centralized selection of ready C exports without changing the generated contract. |
| Verification | Current PR #100 head before this evidence-only update | `.venv/Scripts/python.exe -B tools/api_manifest/generate.py --check` — PASS, 12/12 generated outputs current; `.venv/Scripts/python.exe -B -m unittest tools.api_manifest.tests.test_manifest_v2 -v` — PASS, 40 tests; `cargo test --locked -p kmipkit-ffi` — PASS, 33 tests; `.venv/Scripts/python.exe -B tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --check` — PASS, 181 binary exports observed, 177 manifest declarations matched, zero missing/stale/kind-mismatch/issues, and four exact approved exclusions. Manual signature reconciliation found all 104 C function contracts backed by matching `extern "C"` FFI signatures; the manifest/header use only fixed-width scalar integer types and explicit pointer lengths. |
| Compatibility note | Current C contract | The C-header/binary audit detects current inventory drift, and CI compiles/runs the C consumer. It does not compare the binary against a historical ABI snapshot; compatibility is instead enforced by additive versioned structures/functions, the retained opaque legacy alias, and C consumer tests. |

## T018 — Shared FFI panic boundary

All 129 exported C functions in `kmipkit-ffi` now enter one `catch_unwind` boundary. Panics return the fixed, payload-free `KMIPKIT_ERROR_INTERNAL` status 7; `void` exports record the same status in the existing thread-local error snapshot. The status maps to Rust `ClientCauseCategory::Other`, Java's generic `KmipException`, and Python's generic `KmipKitError`. No panic payload is formatted, retained, or copied into the KMIPKit error snapshot. The embedding process still owns its global panic hook, which can observe a payload before unwinding reaches this boundary; the library does not replace that process-wide hook. The full Rust `ClientCauseCategory` enum remains blocked in the public API manifest because C, Java, and Python do not expose semantically equivalent invalid-input, protocol, and transport causes; only the `Other` variant is claimed as four-language parity.

| Phase | Commit | Evidence |
|---|---|---|
| Red | `213e9d07` | Added failure-first tests for both value-returning and `void` panic fallbacks, the shared last-error snapshot, all exported functions entering the boundary, and language/status manifest mapping. The new tests failed before the implementation was added. |
| Green | `35d30151` | Implemented and applied the boundary to 129/129 exports; added status 7 and regenerated manifest-backed Rust/C/Java/Python projections and parity fixtures; mapped Java/Python/Rust generic error variants. |
| Refactor | `2ccda429` | Documented the boundary's generic fallback and process-owned panic-hook limitation at the shared helper. No behavior changed. |
| Local verification | `2ccda429` | `cargo test --locked -p kmipkit-ffi` — PASS, 38 tests; `cargo test --locked --workspace --all-features` — PASS, exit 0; `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS; `cargo fmt --all --check` — PASS; Python binding pytest — PASS, 63 tests; API-manifest suite — PASS, 41 tests; Java Gradle tests — PASS; API generator — PASS, 12 generated outputs current; extension-fixture generator — PASS; `git diff --check` — PASS. The MSVC C consumer was compiled with `/W4 /WX` and all 16 harness groups passed during independent QA. |
| Regression Red | `e71e55b9`, `4a14b366` | Added tests distinguishing the Rust enum type from its `Other` variant, then requiring the incomplete full enum to remain blocked rather than claiming false four-language parity. The semantic parity test failed before the manifest correction. |
| Regression Green | `45cc38f5`, `1f62389a` | Corrected the Rust type/variant paths, then blocked the full enum with its missing cross-language semantics documented; retained the `Other` status-7 mapping and regenerated the API artifacts and inventory report. |
| Regression Refactor | `d88f09b0` | Strengthened the blocked-enum test to pin each missing cause family (invalid input, protocol, and transport) in the recorded reason. |
| Regression verification | `1f62389a` and `d88f09b0` | Full API-manifest suite — PASS, 248 tests, three platform-dependent skips on `1f62389a`; focused blocked-enum test — PASS on `d88f09b0`; shared Rust inventory suite — PASS, 12 tests; generator `--check` — PASS, 12 outputs; shared inventory auditor `--check` — PASS, 9,497 observed / 135 mapped / 36 blocked; binding export auditor — PASS, 182 C exports / 178 manifest matches; Java Gradle tests — PASS; Python bindings — PASS, 63 tests; `cargo fmt --all --check` and `git diff --check` — PASS. |
| QA | `1f62389a`; final-head review on `22fec63d` | Independent QA found no blockers after verifying that only `Other` is mapped across Rust/C/Java/Python, the complete enum is blocked, and generated outputs and inventory counts are consistent. The final-head review confirmed the task evidence and implementation; the PR matrix below supplies current remote verification. |
| Remote CI | Run `38344844982`, head `dcfefb14716f4e920ff71a5503037e8600e43af8` | PASS: all 34 required checks succeeded; two configured schedule/nightly checks were skipped; zero failures. This run includes the 129-export panic boundary, error mapping, and regenerated artifacts. |
| Final-head CI | Run `38346290184`, head `22fec63da9ff0d11abd5f4c8f1c77fa9c0903f31` | PASS: all 34 required checks succeeded; the two configured schedule/nightly checks were skipped; zero failures. This is the head with the T018/T019 completion records. |

## T021 — Java 17 facade contracts and export audit classpath

Java's existing shared facade now marks its public packages with JSpecify `@NullMarked`, annotates nullable API values and native-library override paths, and exposes `AutoCloseable` on all public wrappers that own native handles. Closing a wrapper releases its handle and repeated close calls are safe. Invalid byte-string inputs produce the stable payload-free `InvalidInputException`. The JSpecify dependency is part of the Java API compile classpath.

The public API inventory now contains 21 blocked declaration rows for the newly exposed Java `close()` methods. These rows satisfy FR-021 inventory coverage without advertising unsupported four-language parity. The pinned export auditor resolves Java's configured compile classpath through the Gradle wrapper with `--offline`, then supplies it to `javac`; this fixes the bare-`javac` regression without runtime dependency downloads. Its regenerated report matches 178/178 C declarations, 351/351 Java declarations, and 274/274 Python declarations.

T021 remains unchecked and incomplete because the shared client/batch foundation is missing. The C ABI and Java facade expose client configuration and batch-item construction for Discover Versions, but no general Client execution entry point, ordered heterogeneous batch submission/outcomes, or complete typed message/result models. The missing C foundation is now explicitly tracked by T076; T021 is the Java adapter over that ABI, and individual operation mappings remain separate under T031–T069. This work did not invent Rust/C APIs and did not perform T022's child-view owner-retention refactor.

| Phase | Commit | Evidence |
|---|---|---|
| Red | `f26c1fdf`, `484c5cff`, `759f1cc0` | Added failure-first Java contract tests for native-resource `AutoCloseable` coverage and JSpecify package nullness, deterministic `TtlvValue.close()`, and stable redacted invalid-input errors. Each focused test failed before its corresponding implementation. |
| Green | `73b3a283` | Implemented `AutoCloseable` wrappers, JSpecify nullness, idempotent handle release coverage, and payload-free `InvalidInputException` handling. `bindings/java/gradlew.bat test --tests org.kmipkit.JavaFacadeContractTest` and full `bindings/java/gradlew.bat test` — PASS. |
| Auditor Red | `1e74055` | Added a Java fixture that first confirms bare `javac --release 17` fails to resolve JSpecify, then requires the auditor's configured classpath resolver. The test failed because the resolver was absent. |
| Auditor Green | `f78afc17` | Added an offline Gradle task exposing `sourceSets.main.compileClasspath`; the auditor now invokes javac with that classpath. The focused regression and all 33 `test_binding_exports` tests — PASS. Pinned `audit_binding_exports.py --check` — PASS: C 178/178, Java 351/351, Python 274/274. The checked-in export report was regenerated by that pinned auditor. |
| Maven dependency Red | `262ddf61` | Added a Gradle/Maven metadata contract test. It failed because Maven declared no compile dependency on JSpecify even though the Java API exposes JSpecify annotations. |
| Maven dependency Green | `3d31213c` | Added JSpecify 1.0.0 at Maven compile scope, matching Gradle's public `api` dependency. `mvn --batch-mode --file bindings/java/pom.xml test-compile` — PASS; `mvn --batch-mode --file bindings/java/pom.xml test` — PASS, 92 tests with three platform skips. |
| Inventory test Red/Green | `35168403` / `3d31213c` | The existing Java inventory test failed with expected 330 versus observed 351. Added coverage requiring all 21 new close declarations to remain blocked until parity is reviewed; corrected the inventory total to 351. |
| Refactor | This evidence commit | Removed a duplicate Java test import, documented the Java public dependency and resource contract, recorded the shared Client/batch blocker without conflating it with operation methods, and reconciled report counts. |
| Remaining blocker | T076 | The shared Client execution and ordered batch outcome foundation remains unavailable in Java/C; T076 owns its C ABI and T021 maps it to Java. Individual operation methods are excluded here and tracked by T031–T069. |
| Final local verification | This evidence commit | Full API-manifest suite via `uv run --offline --no-project --with jsonschema==4.26.0 -- python -B -m unittest discover -s tools/api_manifest/tests -v` — PASS, 251 tests, three skips, 278.065 s; `python -B tools/api_manifest/generate.py --check` — PASS, 12 outputs; `python -B tools/api_manifest/audit_binding_exports.py --check` — PASS, C 178/178, Java 351/351, Python 274/274; `mvn --batch-mode --file bindings/java/pom.xml test-compile` — PASS; Maven test — PASS, 92 tests/3 skips; from `bindings/java`: `gradlew.bat --offline test` — PASS; `git diff --check` — PASS. |

## T019 — C API documentation and generated symbol inventory

The C API guide now links to the v2 manifest and generated header, explains how blocked records differ from declared and verified mappings, and records the source-level and compiled export audits. Its summary counts are checked against the generated binding-export report so documentation cannot silently drift.

| Phase | Commit(s) | Evidence |
|---|---|---|
| Red | `b690d759`, `788e1073` | Added failing checks requiring a manifest-backed C API reference and exact documentation/report count agreement. |
| Green | `7acedca7`, `223e8245` | Added the manifest and generated-header references, C export inventory/audit instructions, and corrected the guide counts to match the report. |
| Refactor | `b86898f9`, `aeff4da1` | Clarified source-level versus compiled verification and documented the exact four intentionally excluded C entries. |
| Compatibility projection correction | Red `e167a69c`; Green `844f063a`; Refactor `5a2e767a` | Pinned the registry-v1 error-category slice, filtered v2-only `internal` from that compatibility projection, and regenerated the fixture. Both focused projection tests pass. |
| Verification | `aeff4da1` and regression head `1f62389a` | Documentation tests — PASS, 2 tests; C binding-export audit — PASS, 182 observed / 178 matched / zero issues; generated API check — PASS, 12 outputs; full API-manifest suite — PASS, 248 tests, three platform-dependent skips. |
| QA | `aeff4da1`; final-head review on `22fec63d` | Independent QA found no blockers: guide counts matched the checked-in report, all four C exclusions were accurate, links resolved, and generator/auditor checks passed. Latest manifest regression and the final-head C documentation/inventory evidence were reviewed. |
| Remote CI | Run `38344844982`, head `dcfefb14716f4e920ff71a5503037e8600e43af8` | PASS: all 34 required checks succeeded; two configured schedule/nightly checks were skipped; zero failures. The C consumer, export audit, generated-file check, and documentation contract checks passed. |
| Final-head CI | Run `38346290184`, head `22fec63da9ff0d11abd5f4c8f1c77fa9c0903f31` | PASS: all 34 required checks succeeded; the two configured schedule/nightly checks were skipped; zero failures. The C consumer, export audit, generated-file, documentation, and cross-platform checks passed. |

## T076 — Shared C client and ordered batch ABI

The C ABI now constructs a production client from validated KMIPKIT-0013 transport configuration and a separate extension configuration, builds ordered typed batches, executes one batch with canonical codec limits, and exposes ordered outcomes including exact pending correlation bytes and sanitized error/delivery metadata. The ABI uses fixed-width forward-compatible discriminators and preserves the existing ABI. The compiled C consumer exercises client creation, ordered metadata, and possibly-sent delivery state without retries or follow-up operations. Configured HTTPS construction is tested; this consumer does not perform an HTTPS wire exchange. T076's 31 newly observed C declarations remain `blocked` in the manifest until Java and Python adapters establish four-language parity; this task does not advertise a C-only public API.

The FFI applies a 16 MiB local material ceiling before copying correlation bytes, zeroizes private-key file bytes from the first read through success and error paths, and bounds certificate reads by the remaining aggregate budget even if a file grows after metadata preflight. The ceiling is a local resource guard rather than an OASIS requirement. File-based revocation lists are rejected; configured CRLs are accepted as memory input. The focused C consumer and Rust tests cover these boundaries, pending metadata, transport validation, and request-frame allocation limits.

| Phase | Commit(s) | Evidence |
|---|---|---|
| Red | `def712ae`, `9e3ea75f`, `0bedfeec`, `8a5617af`, `fcc5dae1` | Added failure-first coverage for the compiled C client/batch ABI, exact pending correlation metadata, pre-copy correlation bounds, key zeroization on success/error/over-bound reads, bounded file reads, and fixture safety. |
| Green | `b49c05bc`, `872cbf50` | Implemented validated client configuration and execution, ordered batch outcomes, correlation metadata, bounded FFI inputs, and zeroizing credential reads. |
| Refactor | `6ac10e73`, `e2712633` | Isolated the client ABI and hardened local TLS test-peer framing, temporary credential handling, and platform-specific temporary directory setup. |
| Independent QA | frozen code `e2712633b99414a018ffeef4a18b024b2aacb1a8` | Static QA found no remaining blockers in the reviewed T076 diff. It confirmed 31 new declarations all remain blocked and FR-019 includes T076. |
| Local verification | frozen code `e2712633b99414a018ffeef4a18b024b2aacb1a8` | `cargo fmt --all --check` — PASS; `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings` — PASS; `cargo test -p kmipkit-ffi --all-features` — PASS, 52 unit tests and 2 C/consumer integration tests; `python -B tools/api_manifest/generate.py --check` — PASS, 12 outputs; `git diff --check` — PASS. Existing AWS-LC/MSVC probe, missing NASM, and linker LNK4217 diagnostics were non-fatal. |
| Remote CI | Pending | The current implementation commits are not yet on the PR head. T076 remains unchecked until the branch is pushed and all applicable CI checks complete successfully. |

## T076 continuation — Python integer boundaries and Java transport coverage

QA reproduced that an `int` subclass could override range comparisons so an out-of-range port or timeout passed Python validation and later raised `OverflowError` during ABI conversion. Validation now extracts the built-in integer value with `int.__index__`, checks that canonical value, and stores canonical built-in integers in `Endpoint` and `TimeoutLimit`. The endpoint class factories now delegate to the canonical dataclass validation path.

| Phase | Commit | Evidence |
|---|---|---|
| Red | `41aa5a63` | Added the malicious `int` subclass regression for over-range endpoint ports and timeout seconds, plus canonicalization assertions. The focused test failed because the over-range timeout was accepted. |
| Green | `4c97b735` | Validated the base integer value and stored canonical values in the frozen public records. The focused regression passed. |
| Refactor | `20c7ef51` | Removed duplicate validation in the Endpoint factories; `__post_init__` remains the single validation and canonicalization path. Python's full client suite remained green. |
| QA | `4c97b735` | Independent read-only review confirmed the range bypass is closed, no equivalent residual was found, and T076 FR-007/FR-013 traceability applies. |
| Local verification | `20c7ef51` | Python 3.12 binding suite with coverage — PASS, 126 tests, 98.16% package coverage; `git diff --check` — PASS. |

The prior remote run `38364373798` at head `2cc478f4` failed Java JaCoCo on Linux at 84% against the required 85% threshold; Java tests themselves had zero failures. JNI coverage also stopped in the same Java Maven verification step. Added portable public-API boundary tests for Java transport timeout, endpoint, mutual-TLS, material-count, and path validation in `677aef81`. Full local Maven `clean verify` then passed 101 tests with zero failures/errors and three platform skips; JaCoCo reported 1,147/1,278 lines (89.75%). This update is local evidence only; the follow-up remote matrix is pending, so T076 and T074 remain unchecked.

### T076 continuation — pre-copy limits and facade revalidation

The Java transport builder now checks individual and aggregate memory/file-path input limits before cloning arrays or UTF-8 encoding paths. Replacement setters subtract the replaced group's prior size before calculating the projected aggregate. The Python `Client.create` boundary now rebuilds transport records through validated base dataclasses immediately before serialization and the FFI call, preventing subclasses from overriding `__post_init__` to bypass endpoint, trust, input, or timeout validation. The Java limit is a local resource guard; it does not create a KMIP requirement.

| Language / phase | Commit(s) | Evidence |
|---|---|---|
| Python Red | `04ff015e` | An adversarial `TransportConfig`/`Endpoint` subclass reached the fake FFI with endpoint length 0 and port 0. The regression failed because FFI was called instead of rejecting the configuration. |
| Python Green | `11aff8ed` | Rebuilt the full transport configuration from base dataclass slots at `Client.create` before record construction and FFI. The adversarial configuration is rejected before the FFI spy is called. |
| Python Refactor | `857e8d4b` | Clarified the snapshot and base-slot invariant without changing behavior. |
| Python QA and verification | `857e8d4b` | Independent QA found no residual subclass bypass. Python 3.12 suite — PASS, 128 tests; package coverage 97.34%, `client.py` 95%; focused facade review — PASS, 65 tests. |
| Java Red | `8e50df3d` | New boundary tests failed as expected for an oversized individual input, aggregate certificate/key/trust/revocation input, and aggregate file-path input before the production fix. |
| Java Green | `355bc7df` | Preflighted memory sizes and UTF-8 path lengths before copying/encoding, including aggregate replacement accounting. The focused boundary suite passed. |
| Java Refactor | `f1ff525d` | Named the shared local material/path bounds; behavior unchanged. |
| Java coverage tests | `f0210552` | Added valid file identity/trust, replacement, blank/oversized path, and multibyte/supplementary UTF-8 boundary cases. |
| Java local verification | `ccb8c838` | `mvn --batch-mode --file bindings/java/pom.xml clean verify` — PASS, 110 tests, zero failures/errors, three skips; JaCoCo gates passed. `TransportConfiguration.java`: 255/263 lines (96.96%); Java bundle: 1,226/1,349 lines (90.88%). |
| Java text-limit Red | `0e2398bd` | Added a child-JVM regression under `-Xmx48m` with a 24,000,000-character TLS server name. The prior implementation reproducibly exited with `encoding-allocation-failed` twice while the test expected a resource-limit result. |
| Java text-limit Green | `820b4d66` | Changed `validateText()` to use the bounded UTF-8 length counter before any encoded copy; preserved the null handling and inclusive 4,096-byte limit. |
| Java text-limit Refactor | `ccb8c838` | Clarified heap headroom accounting and added coverage for UTF-8 replacement of an isolated surrogate. The focused transport boundary class passed 12 tests. |
| Final QA | `ccb8c838` | Independent review found no blockers across Java pre-copy limits, Python FFI-boundary snapshots, UTF-8 sizing, and the bounded-heap probe. All Red/Green/Refactor commits carry the authorized DCO sign-off. |
| Remote CI | Pending | These commits are not yet on the PR head. T076 and T074 remain open until the complete required matrix passes on the final pushed head. |

### CI coverage inventory correction

The full matrix for head `3c87f385` passed the language, platform, Java/Python coverage, and documentation jobs, but the selected coverage gate failed because JaCoCo correctly omitted five `package-info.java` files containing only `@NullMarked` package metadata. The same five files were incorrectly classified as executable adapter sources. They are now documented as metadata-only and excluded by one exact-path inventory shared by source eligibility and JaCoCo parsing; executable Java sources remain required.

| Phase / verification | Commit(s) | Evidence |
|---|---|---|
| Red | `a729810f` | Two regression tests failed before the fix: all five metadata-only files were treated as coverage sources, and the JaCoCo parser included them. |
| Green | `247de035` | Added the five exact exclusions and documented why package metadata has no executable JaCoCo lines. The source scanner and JaCoCo parser use the same inventory. |
| Refactor | `8d54533c` | Clarified the exclusion rationale without changing behavior. |
| Local verification and QA | `8d54533c` | Coverage-gate tests — PASS, 55 tests and three Windows symlink-permission skips; multilanguage coverage tests — PASS, 23 tests; `coverage_gate.py preflight --workspace .` — PASS. Independent QA found no blockers. |
| Remote CI | Pending | The correction is local and awaits a fresh run on the pushed PR head. Keep T074 open until the final required matrix, including the aggregate coverage gate, passes. |

## T076 shared client ABI public-header continuation (2026-10-13)

- **Scope**: Promote the already implemented shared client, transport configuration, ordered batch, and common response ABI into reviewed four-language manifest declarations and the generated public C header. Add an external-style C consumer that compiles only against `bindings/c/include/kmipkit.h`; operation-specific builders and typed payload access remain separately gated by T031–T069.
- **Red**: Added a generator regression requiring the generated public header to include the mapped client handle, transport constructor, and client execution entrypoint. Added a compiled C consumer for the public client and ordered-batch entrypoints. The generator regression fails on the release manifest because those observed T076 C symbols remain blocked and absent from the public header.
- **Command**: `$env:PYTHONPATH = 'tools/api_manifest/tests'; uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest test_generate.ManifestGeneratorCliTests.test_public_header_includes_shared_client_abi_after_binding_mapping -v`
- **Result**: Expected Red: `FAILED (failures=1)` with `the public C header omits the mapped client handle`. The public C consumer still needs compilation in the repository CI matrix because CMake is unavailable on this host; the Java build separately compiled the JNI bridge with MSVC.
- **Commit**: Red test commit `4ca33256`.

## T021 Java shared batch-option parity (2026-10-13)

- **Scope**: Expose the request timestamp, Batch Order Option, Batch Error Continuation Option, and exact client correlation bytes already accepted by the shared C ABI and Python facade.
- **Red**: Added a Java facade contract test that invokes all four shared batch metadata options through the public `ClientBatch` API.
- **Command**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`
- **Result**: Expected Red during test compilation: `ClientBatch` has no `requestTimestamp(long)` method. The remaining chained methods are likewise not available until the facade is implemented.
- **Commit**: Red test commit `cea357e3`.
- **Green**: Added Java facade methods and JNI bridge calls for the four batch options. JNI validates uint32 and correlation byte length before conversion, forwards timestamp/ordering and exact copied correlation bytes to the existing C ABI, and maps native failures through the established exception path.
- **Command**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`
- **Result**: PASS: 9 tests, including a native JNI build and live native batch exercise. The first run caught missing private prototypes in the generated-header boundary; adding the exact ABI declarations to the existing T076 private declaration block resolved it. Contract tests cover unsigned range, null and malformed UTF-8 correlation, and the 16 MiB input limit.
- **Green commit**: `56d1b603`.
- **Refactor**: Consolidated uint32 boundary validation shared by Asynchronous Indicator and Error Continuation values without changing their public exception categories.
- **Command**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`
- **Result**: PASS: 7 tests; native JNI rebuilt successfully and the batch facade contract passed.
- **Refactor commit**: `df81c124`.

## T021/T024 unique batch item identity mapping (2026-10-13)

- **Scope**: Expose the approved Rust `ClientBatchItem::with_unique_batch_item_id` builder in Java and Python over the existing consuming C ABI, preserving caller bytes and explicit ownership transfer.
- **Red**: Added a Java contract test requiring `ClientBatchItem.withUniqueBatchItemId(byte[])` and a Python API contract test requiring `extensions.with_unique_batch_item_id`.
- **Commands**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`; `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_client.py::test_typed_batch_item_exposes_unique_batch_item_id_builder -q`.
- **Results**: Expected Red: Java compilation reports the missing `withUniqueBatchItemId(byte[])` method; Python fails because `with_unique_batch_item_id` is absent. The Python package was built and installed using the repository's CI steps before this test.
- **Commit**: Red test commit `c4d4dd50`.
- **Green**: Java and Python expose consuming item builders over the existing ABI. Both preserve opaque identifier bytes, enforce the 16 MiB local input bound before making their owned copies, and transfer the previous wrapper only when constructing the new typed item.
- **Commands**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`; `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_client.py::test_typed_batch_item_unique_id_builder_preserves_bytes_and_transfers_ownership -q`.
- **Results**: PASS: Java contract suite 8 tests, including a native JNI build; Python ownership/byte-preservation fake-ABI test 1 passed. The complete Python client contract suite also passes: 89 tests.
- **Green commit**: `39eb07ef`.

### Java correlation UTF-8 and resource-limit QA follow-up

- **Red**: Expanded the Java contract to exercise multibyte UTF-8, malformed UTF-8 rejection, and the 16 MiB input ceiling. The previous facade mapped oversized input to `InvalidInputException`, unlike Python's resource-limit category.
- **Command**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`
- **Result**: Expected Red on the previous facade at the oversized correlation assertion; malformed UTF-8 is rejected by the shared C ABI before any request is sent.
- **Commit**: Red test commit `63d723e6`.
- **Green**: Oversized correlation input now raises `ResourceLimitException` before copying or entering JNI. Java documentation states the UTF-8 validity and bound, and clarifies that KMIP continuation semantics are checked at batch execution.
- **Command**: `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeContractTest test`
- **Result**: PASS: 9 tests; C++ JNI bridge rebuilt successfully and all facade boundary tests passed.
- **Green commit**: `39eb07ef`.

### Adapter regression matrix on continuation head `39eb07ef`

| Binding | Command | Result |
|---|---|---|
| Java 17/JNI | `mvn --batch-mode --file bindings/java/pom.xml verify` | PASS: 129 tests, 0 failures/errors, 3 documented platform skips; JaCoCo coverage checks passed; native JNI bridge compiled with MSVC. |
| Python 3.12/CFFI | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests -q` | PASS: 152 tests. |

These local binding checks do not close T076: the C public header remains generated from blocked manifest declarations, the standalone compiled C consumer has not yet run, and remote CI has not run on this continuation head.

## T076 generator layout correctness follow-up (2026-10-13)

QA found correctness gaps in the new versioned C-value layout generator: nested `const` pointer levels were mapped incorrectly to Rust, nested C records were emitted in manifest order without readiness checks, an incomplete value could appear in a ready by-value function signature, and Rust keyword field names could generate invalid source.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `f850cac5` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_rust_ffi_preserves_const_only_at_the_innermost_c_pointer_level tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_rust_ffi_escapes_keyword_names_in_fixed_width_fields tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_c_header_orders_nested_value_definitions_after_forward_declarations tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_c_header_rejects_ready_values_that_reference_blocked_types tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_c_header_rejects_ready_function_using_partial_value_by_value -v` | Expected Red: all five focused regressions failed against the previous generator. |
| Green | `21bffe4b` | Same focused command plus the versioned-value and Rust-field renderer tests | PASS: all seven focused tests. C layouts now have forward declarations, complete by-value dependencies are topologically emitted, blocked/incomplete references are rejected, Rust field names use raw identifiers where needed, and `const` applies to the innermost C pointer level. |
| Refactor | `510fbc8a` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_duplicate_fixed_width_value_field_names tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_require_versioned_c_value_header_fields tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_c_header_orders_nested_value_definitions_after_forward_declarations tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_c_header_rejects_ready_values_that_reference_blocked_types tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_c_header_rejects_ready_function_using_partial_value_by_value tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_rust_ffi_preserves_const_only_at_the_innermost_c_pointer_level -v` | PASS: six focused tests; C field parsing is shared by validation and layout ordering. |

The complete local API-manifest suite ran 261 tests: 257 passed, three Windows symlink-permission tests were skipped, and one pre-existing T076 regression failed because the current blocked manifest still omits `kmipkit_client_t` and related shared client entries from the public C header. That failure is the remaining T076 mapping/promotion work, not a generator-layout regression. `cargo test -p kmipkit-ffi metadata_tests` passed 11 tests. The generator wrote and then verified all 12 checked-in outputs; only the public and internal C headers changed from the corrected struct emission. Remote CI has not run for these commits.

### QA follow-up: exclusions and C identifiers

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `b0b5d022` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_versioned_fixed_width_values_render_declared_c_fields tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_c_keywords_as_fixed_width_field_names -v` | Expected Red: excluded ABI prefix fields left a versioned C value opaque, and `struct` was accepted as a field name. |
| Green | `70bb2d46` | Same two focused tests plus the nested value readiness and by-value signature regressions | PASS: five focused tests. Approved C field exclusions now count toward C layout completeness; C reserved words are rejected. |

These changes affect generator validation and do not alter current T076 declarations, which remain blocked pending exact Rust/C/Java/Python signatures and mapping evidence. A compiled public C consumer and remote CI remain required before T076 can close.

### QA follow-up: C++ header compatibility

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `5a2d1541` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_cpp_keywords_as_fixed_width_field_names -v` | Expected Red: `class` was accepted as a C-layout field name although the public header has a C++ `extern "C"` inclusion guard. |
| Green | `274971e4` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_c_keywords_as_fixed_width_field_names tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_cpp_keywords_as_fixed_width_field_names -v`; MSVC `cl /nologo /TP /Zs /I bindings/c/include <temporary source including kmipkit.h>` | PASS: both C and C++ reserved-field regressions; the generated public header also passed a C++ syntax-only include check. |

Independent QA reviewed the generated layout and confirmed that approved ABI prefix exclusions now complete versioned value layouts. The public T076 client entries themselves remain blocked until their API signatures and language mappings are represented in the manifest; consumer compilation and remote CI are still required for closure.

### C++20 reserved-name precision follow-up

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `422d34dd` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_allow_nonstandard_cpp_extension_identifiers -v` | Expected Red: the generator incorrectly rejected the transactional-memory TS identifiers `atomic_cancel`, `atomic_commit`, and `atomic_noexcept`, plus `synchronized`. |
| Green | `23f6ae32` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_c_keywords_as_fixed_width_field_names tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_cpp_keywords_as_fixed_width_field_names tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_allow_nonstandard_cpp_extension_identifiers -v` | PASS: reserved C/C++20 identifiers are rejected while those four non-keyword identifiers remain accepted. |
| Refactor | `95cf4653` | `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_reject_cpp_keywords_as_fixed_width_field_names tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_references_allow_nonstandard_cpp_extension_identifiers -v`; `cl /nologo /TP /std:c++20 /Zs /I bindings/c/include <temporary source including kmipkit.h>` | PASS: two focused tests and a C++20 syntax-only include check; the keyword set documents its language-version scope. |

On the layout-generator check head before the T076 manifest promotion, `python -m unittest tools.api_manifest.tests.test_generate -v` ran 50 tests: 47 passed, two Windows symlink-permission tests were skipped, and the only failure was the expected public-header T076 gate (`kmipkit_client_t` is still blocked). `python tools/api_manifest/generate.py --check` passed for all 12 fixed outputs. The new C++ keyword-precision changes then passed their focused Red/Green/Refactor checks; rerun the full suite after T076 mappings are promoted.

### T076 public shared-client mapping completeness gate (2026-10-13)

- **Red**: Extended the public-header regression to require every observed T076 C ABI declaration to be ready only after its Rust, Java, and Python mappings are represented. The current manifest fails because all 31 observed T076 declarations are still blocked.
- **Command**: `uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_public_header_includes_shared_client_abi_after_binding_mapping -v`
- **Result**: Expected Red: `FAILED (failures=1)`; the assertion reports all 31 T076 declarations as blocked. The test now guards against exposing only a subset of the shared C client API.
- **Commit**: Red test commit `afaa78d1`.

### T076 gate refinement — exact C ABI exclusions (2026-10-13)

Independent QA reviewed the T076 inventory and found that requiring all 31 C exports to be declared as four-language callable functions/types would misclassify C ABI carriers and manual handle release operations. The generator supports exact reviewed C exclusions while retaining their public C declarations in the generated header. The test now requires each observed T076 export to be either fully mapped or covered by an exact approved exclusion; it still requires the shared client ABI to appear in the public header.

- **Red**: The focused test failed because the manifest resolves none of the 31 T076 observed C exports as mapped or exactly excluded.
- **Command**: `uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_public_header_includes_shared_client_abi_after_binding_mapping -v`
- **Result**: Expected Red: `FAILED (failures=1)`; all 31 exports are reported unresolved. Existing client-handle assertions remain as additional public-header gates.
- **QA**: Read-only review by `/root/qa_010_audit`; no files or tests were changed by QA.
- **Red refinement commit**: `25ed5389`.
- **Fresh Red rerun**: `uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_public_header_includes_shared_client_abi_after_binding_mapping -v`.
- **Fresh Red result**: Expected `FAILED (failures=1)`; the exact-exclusion gate reports all 31 observed T076 declarations unresolved. The base test still verifies the mapped client handle, transport constructor, and execution entry point are present in the generated public header.

### T076 exact C pointer-shape support

- **Red**: Added a generator regression that models the observed consumed `handle **` slot and borrowed fixed-width value output pointer without changing their C ABI shapes.
- **Command**: `uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_consumed_handle_slot_and_output_value_pointer_keep_exact_abi -v`
- **Result**: Expected `FAIL`; the generator rejects the manifest because schema v2 cannot currently represent these exact pointer kinds.

- **Red refinement**: The real T076 output pointer is `kmipkit_byte_span_v1_t *`; the fixture now records that exact C-only carrier layout and verifies the generated public header layout. Added negative mutations for a mismatched `T **`/`handleType` pair and a mismatched `T *`/`valueType` pair.
- **Command**: `uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_consumed_handle_slot_and_output_value_pointer_reject_type_mismatch`
- **Result**: With the two exact type-equality guards temporarily removed from the working generator, both invalid signatures were accepted (`exit=0`), so the regression failed as expected. Restored the guards immediately after the Red run. The byte-span exact-layout test passes with the guarded Green implementation.

- **Red**: Added a client test for the exact zero-argument empty-batch constructor represented by the C, Java, and Python facades.
- **Command**: `cargo test -p kmipkit-client --test operation_boundary empty_batch_constructor_creates_an_empty_batch_without_network_activity`
- **Result**: Expected compile failure: `ClientBatch::empty` is not yet available; existing Rust constructors require one item or an iterator.

- **Red**: Wired the existing C client consumer into the FFI feature build and Rust test harness so it includes only generated `kmipkit.h`.
- **Command**: `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_header_client_consumer_compiles_and_runs`
- **Result**: Expected C compile failure because the current generated public header omits the T076 client/transport types and declarations (`kmipkit_client_t`, `kmipkit_client_create`, `kmipkit_client_batch_create`, and related symbols).

- **Red refinement**: Added a generator test requiring an exact excluded C-only function signature to remain in the public header while its symbol is excluded from four-language callable declarations.
- **Command**: `uv run --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_approved_c_only_function_exclusion_keeps_exact_header_prototype`
- **Result**: Expected `FAIL` because the manifest schema does not yet represent `cFunction` on an approved exclusion; generation rejected the reviewed fixture before rendering.

- **Red: generic response access on typed execution failure**: Extended the compiled C client scenario to call the additive `kmipkit_client_execute_with_error_ttlv_v1` API. The fixture requires a NULL error-TTLV output after success and local validation failure, and requires a protocol status plus the complete ResponseMessage tree after a received response fails typed shape validation. The output is inspected through the existing generic TTLV structure view API.
- **Command**: `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_header_client_consumer_compiles_and_runs -- --nocapture`
- **Result**: Expected build failure because the current generated public header has no T076 shared client declarations and cannot compile the C consumer, including the new fallback call. The same invocation currently stops in `build.rs` with unknown T076 types/functions before reaching the linker; retain this as the public-header Red gate, then rerun after the manifest mapping is added.

### T076 QA follow-up: keep batch metadata carrier test-local

- **Red**: Added a generator regression that requires the batch metadata carrier to remain outside `approvedExclusions` and the public C header, while keeping all 27 T076 declarations blocked.
- **Command**: `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate.ManifestGeneratorCliTests.test_t076_batch_metadata_carrier_is_not_approved_or_public -v`
- **Result**: Expected failure: the assertion found `kmipkit_client_batch_item_metadata_v1_t` in `approvedExclusions` before checking the public header.
- **Red commit**: `e06fe127`.
- **Green**: Removed the C-only carrier from `approvedExclusions` and regenerated the 12 manifest outputs. Both generated C headers now omit the carrier; the exact six-`uint32_t` layout exists only in the compiled C test translation unit. The 27 T076 declarations remain blocked. The post-dispatch transport failure now calls `kmipkit_client_execute_with_error_ttlv_v1` with a sentinel output handle and asserts it is cleared to NULL while preserving Transport/PossiblySent metadata.
- **Commands and results**:
  - `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate -v` — PASS: 56 tests, 2 Windows symlink-permission skips.
  - `python tools/api_manifest/generate.py --check` — PASS: all 12 outputs current.
  - `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process -- --nocapture` — PASS: compiled C consumer executed success, typed-response fallback, local validation, and post-dispatch transport-failure paths.
  - `cargo llvm-cov -p kmipkit-ffi --all-features --summary-only` — PASS: 56 unit tests and 3 integration tests. `client_abi.rs` reports 89.43% line coverage for the entire historical file; the T076 changed executable Rust line delta remains 56/57 (98.25%) against base `4c1632eb`. The C consumer is executed by this suite; LLVM's Rust summary does not report a separate C translation-unit percentage.
  - `cargo clippy -p kmipkit-ffi --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` — PASS.
- **Green commit**: `2c50f7e1`.
- **Refactor**: Added C compile-time size and field-offset assertions for the test-local carrier, making its ABI shape explicit without publishing the type.
- **Refactor command and result**: `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process -- --nocapture` — PASS: the C translation unit compiled all layout assertions and the full loopback TLS scenario passed.

### T076 Windows local FFI and C-consumer verification (2026-10-13)

| Command | Result |
|---|---|
| `cargo test -p kmipkit-ffi --all-features` | PASS: 56 unit tests and 3 integration tests; the integration suite compiled and ran `blocked_client_abi_local_prototype_consumer_compiles_and_runs` and `existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process`. |

This is local execution evidence for the T076 ABI behavior and its test-local C consumer. The build emitted AWS-LC compiler-feature/NASM warnings and linker import warnings but completed successfully. The generated public header still omits T076 symbols while the four-language manifest declarations remain blocked, and remote CI has not run on this branch; keep T076 unchecked.


## T076 release rebase verification (2026-10-13)

- **Red**: Re-ran the complete FFI workspace feature suite after rebasing the T076 branch onto the current release, which includes KMIPKIT-0044. The export-boundary inventory found 155 C exports while the pinned assertion still expected 154.
- **Command**: `cargo test -p kmipkit-ffi --all-features`
- **Result**: 56 passed, one failed at `every_exported_c_function_starts_inside_the_shared_panic_boundary` (`left: 155`, `right: 154`). This is an expected maintenance failure caused by the merged release API surface.

- **Green**: Updated the export-boundary count to the rebased release inventory and reran the full FFI suite.
- **Commands and results**: `cargo test -p kmipkit-ffi --all-features` — PASS: 57 unit tests, 3 compiled C integration consumers, and doc tests passed. The focused export-boundary assertion also passed.
- **Limit**: Windows emitted existing AWS-LC compiler-feature/NASM and linker import warnings; they did not prevent the test build or execution. This local evidence does not close T076 or establish remote CI status.

### Rebased generator verification

- **Commands and results**: `uv run --no-project --with-requirements tools/api_manifest/requirements-test.txt python -m unittest tools.api_manifest.tests.test_generate -v` — PASS: 56 tests, two Windows symlink-permission skips. `python tools/api_manifest/generate.py --check` — PASS: all 12 generated outputs match the merged manifest.
- **Scope**: T076 remains open. The 27 observed T076 declarations remain blocked and the generated public C header still omits the shared client handle pending the required binding mappings.

### T076 shared client manifest mapping — partial promotion (2026-10-13)

This increment maps the exact shared Client, TransportConfig, ClientBatch,
response collection, item identifier, and response-value surfaces owned by
KMIPKIT-0005, -0006, -0007, -0013, and -0043. It does not complete T076: 19 of
the 27 observed declarations are ready with four-language signatures, seven
ABI-only C helpers retain their exact prototypes/layouts through reviewed
C-only exclusions, and `kmipkit_client_execute_with_error_ttlv_v1` remains
blocked because the corresponding four-language surface is not approved here.
Per-operation APIs and draft-owned -0009/-0010 surfaces remain outside this
promotion.

This section supersedes earlier historical notes in this file stating that
all 27 declarations remain blocked or that the generated public header omits
the shared client ABI.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `031bc626` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v` | Expected Red: the comprehensive regression rejects declarations without exact cross-language mappings or reviewed C-only exclusions; no generated-header T076 promotion is accepted. |
| Red (schema subset) | `e51652de` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_generate.ManifestSchemaTests.test_schema_accepts_optional_java_byte_array_return -v` | Expected Red: the reviewed Java `Optional<byte[]>` signature is rejected by the previous generic-argument grammar. |
| Green | `fcb7d7d3` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping tools.api_manifest.tests.test_generate.ManifestSchemaTests.test_schema_accepts_optional_java_byte_array_return -v`; `python tools/api_manifest/generate.py --write`; `python tools/api_manifest/generate.py --check`; `cargo test -p kmipkit-client --test operation_boundary` | PASS: six manifest/mapping tests, all 12 generated outputs, and three Rust client boundary tests. The Java type grammar change is limited to reference-array generic arguments. |
| Refactor | `59b453a5` | `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_compiles_and_runs -- --nocapture`; `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process -- --nocapture`; `cd bindings/java && .\\gradlew.bat test --tests org.kmipkit.ClientFacadeContractTest`; `uv run --offline --directory bindings/python --with-requirements requirements-coverage.txt --with-editable . python -m pytest tests/test_client.py tests/test_generated_api_manifest.py -q`; `git diff --check` | PASS: public generated-header consumer 1/1; runtime C consumer 1/1; Java JNI/native build and facade contract tests; 91 Python client/manifest tests; no whitespace errors. Refactor removes duplicate local carrier definitions so C and JNI consumers use the generated internal-header layouts. |

The mapped records are checked against actual Rust declarations, C prototypes,
Java methods, and Python methods. The C result-metadata helper and its six
`uint32_t` fields remain available in the generated C ABI; Java copied getters,
Python `BatchItemResult` fields, and Rust `ClientBatchItemResponse::outcome()`
with typed KMIP result status/reason provide the equivalent value semantics.
Transport configuration remains a four-language capability through Rust and
Java builders, the Python immutable configuration value used by `Client.create`,
and the C adapter. The exact `kmipkit_transport_config_create_v1` converter is
therefore a C-only representation helper, not a missing transport capability.

`ClientError::response_ttlv()` is described by approved KMIPKIT-0021 Rust
requirements, but the additive C fallback
`kmipkit_client_execute_with_error_ttlv_v1` has no approved Java/Python mapping
contract in the owners in scope. It remains the single blocked declaration;
the exact new cross-language facade would need approval before promotion.
Remote CI has not run, and T076 plus dependent T021/T024 remain unchecked.

### T076 C transport field reconciliation — partial promotion

The C transport field inventory is now classified without hiding whole
records: 24 semantic fields have exact four-language mapping targets; the
three adjacent slice lengths, the generic input byte span, and the optional
value presence bitset are exact representation exclusions. The ten ABI
framing/reserved fields remain separately excluded, and all five C carrier
types retain their exact generated layouts. `transport_config.flags` records
only whether `tls_server_name` and `target_uri` are present; both values are
mapped independently. The three `*_count` fields are only derived slice
lengths. `transport_input.data` is an exact nested-record field exclusion,
while `byte_span.data` and `byte_span.length` are separately excluded pointer
and length framing fields. The generated layouts retain the parent input's
`source_kind`, `encoding`, and nested byte-span fields. The input bytes alone
are not a capability: they are interpreted with their parent configuration
context and selectors.

The C `endpoint_kind` discriminant maps to the existing public endpoint
families: Rust `Endpoint::raw_tls`/`Endpoint::https`, Java builder
`rawTls`/`https`, and Python `Endpoint.raw_tls`/`https`. `endpoint_port` is
applicable only to RAW_TLS. For HTTPS, the `endpoint` field is the absolute
URI and carries its authority, including an explicit port; C requires
`endpoint_port == 0`. `target_uri` remains the separate origin-form path and
optional query. The FFI parser, Java `https` builder, and Python HTTPS
endpoint all preserve this separation. The C ABI regression accepts
`https://server.kmipkit.test:8443` with `endpoint_port == 0` and
`target_uri == "/kms/kmip"`, and rejects putting `8443` in the separate port
field.

C timeout `kind` maps to Rust `TimeoutPolicy`, Java `Timeout.kind`, and Python
`TimeoutLimit.kind`. Kind zero selects the matching member of
`TimeoutPolicy::default()`; Java `Timeout.defaultValue()` and Python
`TimeoutLimit.default()` expose that same default selector. Kinds BOUNDED and
UNBOUNDED remain distinct, with seconds/nanoseconds used only for BOUNDED.
The FFI regression checks DEFAULT for connect, write, read, and total against
the accepted Rust policy, in addition to the existing bounded/unbounded
cases.

Credential mapping covers the complete approved mode matrix, rather than
assuming that a PEM-memory constructor stands for all inputs:

| C parent field | Rust public API | Java public API | Python public API | Supported C input modes |
|---|---|---|---|---|
| `client_certificates` | `CertificateInput` PEM/DER and file constructors | `clientIdentity(List<TransportInput>, TransportInput)`; PEM/DER convenience methods remain | `TransportConfig.client_certificates` with `TransportInput.memory/file` and `CredentialEncoding` | Each chain entry independently selects PEM/DER and memory/file |
| `client_private_key` | `PrivateKeyInput` PEM/DER and file constructors | `clientIdentity(List<TransportInput>, TransportInput)`; PEM/DER convenience methods remain | `TransportConfig.client_private_key` with `TransportInput.memory/file` and `CredentialEncoding` | The key independently selects PEM/DER and memory/file; Java consumes and zeroizes its owned key-input copy |
| `trust_certificates` | `TrustSource::certificate_authorities` over `CertificateInput` PEM/DER and file constructors | `trustCertificates(List<TransportInput>)`; PEM/DER convenience methods remain | `TrustSource.certificate_authorities` over `TransportInput.memory/file` | Each trust entry independently selects PEM/DER and memory/file |
| `revocation_lists` | `RevocationListInput::from_pem/from_der` | `revocationLists(List<TransportInput>)`; PEM/DER convenience methods remain | `TransportConfig.revocation_lists` with in-memory `TransportInput` | Each CRL independently selects PEM/DER and remains memory-only |

Java `TransportInput.memory` takes a defensive copy that callers can deterministically zeroize with `close`; `clientIdentity` closes the input used for the private key after taking its protected builder copy. Its `toString` remains redacted. Documentation states that Java runtime copies outside KMIPKit's control cannot be zeroized. The JNI integration test constructs one configuration with a PEM-memory certificate, DER-file private key, PEM-file and DER-memory trust certificates, and PEM/DER in-memory CRLs, then creates a native-backed client from it.

Java mixed-input TDD evidence (all commits are DCO-signed):

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `9de004b2` | `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride test` | Expected compile failure: `TransportConfiguration.TransportInput` was not defined for the independent PEM/DER and memory/file selections. |
| Green | `670f743d` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=TransportConfigurationBoundaryTest,ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride' test` | PASS: 15 Java/JNI focused tests, including input validation, private-key input consumption, mixed certificate/key/trust source modes, and HTTPS TLS-name override. |
| Additional CRL evidence | `98c49d34` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride' test` | PASS: the mixed configuration also crossed JNI with one valid PEM CRL and one valid DER CRL. |
| Refactor | Pending | Manifest regeneration, traceability updates, API mapping and export-audit tests | Will be recorded after the final refactor checks. |

For each `kmipkit_transport_input_v1_t`, `encoding` selects PEM or DER and
`source_kind` selects material bytes or a UTF-8 path carried in `data`; the
parent configuration field determines whether those bytes/path represent a
certificate chain, private key, trust certificate, or CRL. The FFI's
`certificate_input_bytes_with_budget`, `private_key_input`, and
`revocation_input` implement those contexts. File-backed CRLs are not
approved by KMIPKIT-0013-FR-003 and are rejected before reading in Rust FFI
and Python. The exact `kmipkit_transport_input_v1_t.data` exclusion therefore
does not hide any mode or material kind: source/encoding selectors and each
parent capability are independently mapped, and the nested C layouts remain
generated. `client_private_key` maps to Rust `PrivateKeyInput` and trust
certificates map to the Rust `TrustSource::certificate_authorities` public
constructor.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `72152b97` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_c_transport_and_result_fields_are_semantically_mapped_or_exactly_excluded tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_transport_input_mappings_cover_specified_encodings_and_sources tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_endpoint_kind_mapping_names_the_constructor_selection tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_c_field_targets_are_exact_ready_public_exports -v` | Expected failures: representation fields were still declared, four target types were blocked, and endpoint constructor evidence was absent. |
| Green | This change | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `python -B tools/api_manifest/generate.py --write`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | PASS: all 15 T076 mapping regressions; all 12 generated outputs are current. The generated export audit is refreshed separately and still reports Java/Python inventory gaps, so T011/T076/T021/T024 remain open. |

### T011 Python generated-export inventory — open reconciliation (2026-10-14)

The Python scanner now inventories statically identifiable dataclass-generated
methods and metadata, records constructor signatures derived from source field
annotations/defaults, and reports dynamic dataclass options as incomplete. It
also observes public constructors inherited from package-private bases. It
does not execute decorators or inventory the whole inherited `object` model.
`__post_init__` is treated as the dataclass initialization hook, and the
private handle base's `__del__`/`__repr__` are lifecycle/diagnostic plumbing;
explicit `close` and context-manager methods remain inventoried. The current
`Client.__init__` observation comes from the raw-handle base constructor and is
unresolved ownership/security evidence, not approval to construct `Client`
from a caller-owned handle. The refreshed export report still requires exact
reconciliation; this work does not close T011, T076, T021, or T024.

The existing Python representation regression verifies that generated reprs
do not expose endpoint passwords, certificate/private-key bytes, file paths,
trust inputs, or pending correlation bytes. In particular,
`test_secret_bearing_client_records_redact_repr` and
`test_endpoint_direct_constructor_enforces_text_shape_and_repr_redaction` in
`bindings/python/tests/test_client.py` remain required evidence for the
generated `__repr__` inventory.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `7562bb7b`, `19a74c10`, `c06053a0`, `f1b597b5`, `5b6e4d77`, `3234862f` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_binding_exports -v` (at each focused Red checkpoint) | Expected failures successively exposed missing generated dataclass methods, metadata/signatures, dynamic-option handling, hash declarations, and inherited package constructors. |
| Green | `64c7499d` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_binding_exports -v` | PASS: 40 tests; dataclass-generated exports, signature capture, fail-closed options, explicit hash declarations, and inherited constructors are inventoried. |
| Refactor | This commit | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_binding_exports -v`; `uv run --offline --directory bindings/python --with-requirements requirements-coverage.txt --with-editable . python -m pytest tests/test_client.py::test_secret_bearing_client_records_redact_repr tests/test_client.py::test_endpoint_direct_constructor_enforces_text_shape_and_repr_redaction -q`; `git diff --check` | PASS: 41 scanner tests; 2 existing Python repr-redaction tests; whitespace check. Added characterization for lifecycle hooks and clarified current partial audit/task state. |

### T076 batch outcome and owner lifecycle mappings — partial promotion (2026-10-14)

The common completed/Pending outcome discriminator is mapped from the Rust
typed `ClientBatchItemResponse::outcome`, the exact C metadata field, Java's
forward-compatible `BatchOutcomeKind`, and Python's `BatchOutcome`/numeric
fallback. Result status and optional result reason remain separately mapped.
Three owner release paths (`Client`, `ClientBatch`, and
`ClientBatchResponse`) now have exact C symbols and four-language
`native-handle-release` mappings to the Java and Python `close` methods and
Rust owning types. The C child item-response release and transient transport
configuration release remain explicit exact C ABI exclusions.

After the generated-header promotion, an independent Ubuntu 26.04 run passed
the complete C ABI coverage integration target (3/3):
`CARGO_TARGET_DIR=/tmp/kmipkit-t076-linux-target cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage -- --nocapture`. This includes the compiled public C consumer. It is local platform evidence and does not close the full parity or required-CI gates.

The exact Java predicates `BatchOutcomeKind.isCompleted` and
`BatchOutcomeKind.isPending` are excluded as derived conveniences over the
mapped raw discriminator and constants. `ClientBatchItemResult.close` is an
exact Java-only child-view lifecycle adapter: Rust owns response items in the
response value and Python exposes copied item metadata. If early release of a
child view is later considered a product capability, an approved API contract
must define matching lifetime semantics across languages; this would change
when the child storage can be reclaimed. `TransportConfiguration.close` is
an exact Java-only adapter for the transient native configuration handle;
Rust and Python expose configuration values and the C release helper stays
explicit in the inventory. The cost if this distinction is wrong is different
credential-release timing in Java, which requires review as a security
lifecycle contract before promotion.

The refreshed Windows export snapshot records C 264 observed paths with 237
exact declarations and 27 exact exclusions, Java 469 observed paths with 16
still unclassified, and Python 540 observed paths with 80 still unclassified.
The audit is therefore partial: `matchesManifest` is true for C and false for
Java and Python. T011, T076, T021, and T024 remain open; the generic
`kmipkit_client_execute_with_error_ttlv_v1` helper remains blocked.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `b61fc89b` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_java_client_outcomes_and_owned_handle_lifecycles_are_accounted -v` | Expected failure: 19 unaccounted outcome, exact-exclusion, and owner-release paths. |
| Green | `898286fb` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `python -B tools/api_manifest/generate.py --write`; `python -B tools/api_manifest/generate.py --check`; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_generate -v`; `git diff --check` | PASS: T076 mapping suite 18/18; all 12 generated outputs current; generator suite 57 passed with 2 Windows symlink-permission skips; no whitespace errors. |
| Refactor | This commit | `python -B tools/api_manifest/generate.py --check`; `git diff --check` | PASS: all 12 generated outputs remain current; checkpoint prose and exact lifecycle ruling/costs are recorded without changing runtime behavior. |

### T076 transport facade export reconciliation — checkpoint at `d576d04f` (2026-10-14)

At this checkpoint, the transport facade inventory accounted for all 264 observed C exports
(237 exact manifest declarations plus 27 exact approved exclusions) and all
469 observed Java exports (434 declarations plus 35 exact exclusions). Python
had 540 observed exports: 415 declarations and 77 exact exclusions accounted
for 492, while 48 were missing. The report at that checkpoint therefore had
`matchesManifest=true` for C and Java and `false` for Python. The missing Python
paths include value/dataclass constructors and members, enum members, error
constructors/category exports, context methods, and inherited handle
constructors. Value constructors and native-handle wrapper constructors are
separate reconciliation groups; wrapper constructors must not allow adoption
or duplicate ownership of a live native handle. T011, T076, T021, and T024
remain open.

The four-language transport mappings distinguish endpoint selection and input
representation: HTTPS versus RAW_TLS endpoint constructors; PEM/DER and
approved file inputs for certificates, private keys, and trust; in-memory
PEM/DER for CRLs; `source_kind` versus `encoding`; and timeout `DEFAULT` versus
bounded/unbounded limits. The endpoint port mapping is conditional on RAW_TLS.
CRL file input is not added because KMIPKIT-0013 does not approve that mode.
No runtime behavior or ABI changed in this tranche. The generic
`kmipkit_client_execute_with_error_ttlv_v1` helper remains blocked.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `506b6a15`, `020b99cb`, `4f48758b`, `a0a48084`, `2906c89d` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping` (focused transport assertions at each Red checkpoint) | Expected failures exposed 47 unclassified transport declarations, then the missing `TimeoutLimit` type and six incorrect or incomplete semantic mapping targets. |
| Green | `d576d04f` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping`; `wsl --exec bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/0043-t076-client-mapping/KMIPKit && python3 -B -m unittest tools.api_manifest.tests.test_generate'`; `python -B tools/api_manifest/generate.py --check`; `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json`; `git diff --check` | PASS: focused T076 suite 20/20; generator suite 57/57; all 12 generated outputs current; C/Java exact and Python 48 missing. |
| Refactor | `99f3adbf` | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | The audit command wrote that checkpoint's snapshot and exited 1 because Python had 48 missing paths; C and Java had `matchesManifest=true`. All 12 API generated outputs were current; the task checkpoints and audit instructions recorded this partial state without marking tasks complete. |

### Python copied result-record constructor classification (2026-10-14)

`kmipkit.client.BatchItemResult.__init__` is excluded by its exact export path
as Python dataclass convenience for instantiating an inert DTO from copied
response metadata. It does not execute a KMIP operation or change a response.
The shared `BatchItemResult` type and all five result fields remain declared
and mapped; the shared capability remains execution and read-only response
traversal. This decision does not cover any other Python constructor. If direct
construction of response DTOs becomes a product requirement, define a
four-language construction contract before exposing that capability.

The refreshed audit still has C and Java fully accounted. Python now has 415
declarations and 78 exact exclusions for 493 of 540 paths, leaving 47 missing;
T011, T076, T021, and T024 remain open.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `6bf532e9` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_python_batch_item_result_constructor_is_an_exact_record_exclusion -v` | Expected failure: `BatchItemResult.__init__` had no exact classification; the test also requires its type and five semantic fields to remain declared. |
| Green | `42e0a036` | Same focused test; `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping`; `python -B tools/api_manifest/generate.py --write`; `python -B tools/api_manifest/generate.py --check`; `wsl --exec bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/0043-t076-client-mapping/KMIPKit && python3 -B -m unittest tools.api_manifest.tests.test_generate'`; `git diff --check` | PASS: focused test; T076 mapping suite 21/21; generator suite 57/57; all 12 generated outputs current. |
| Refactor | This commit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | The audit command writes the current snapshot and exits 1 because Python still has 47 missing paths; C and Java remain `matchesManifest=true`. The exact exclusion rationale and partial task status are recorded without changing runtime behavior. |

### T024 Python native-wrapper handle ownership — corrected, inventory open (2026-10-14)

The ownership guard now covers all 29 native-handle wrappers sharing the
private `NativeHandle` base: `Client`, `Batch`, `BatchResult`; nine TTLV
handles/views; and seventeen extension-registry/request handles. Before this
change, the inherited constructor accepted an arbitrary CFFI pointer and each
wrapper later released it, permitting invalid adoption and duplicate release
of a live handle. Construction now requires an identity-checked private token
with no default, checked before subclass initialization; only the private
`_wrap_owned_handle` factory supplies it. Every production callsite that wraps
a returned C handle uses that factory. Its keyword forwarding preserves
`_owner` references for views and child handles. Consuming ABI paths continue
to use the existing `_take`/`_restore` protocol so preflight failures restore
the original owner and accepted transfers invalidate the source wrapper.

This guard is applied uniformly to all native wrappers because fixing only
Client/Batch/BatchResult would leave the same raw-pointer and double-release
path on TTLV and extension handles. The compatibility cost, if this ruling is
too broad, is source compatibility for consumers that depended on undocumented
pointer-taking constructors; review wrapper-constructor compatibility before
1.0. No C ABI or KMIP runtime behavior changed. The scanner still observes
these `__init__` paths, so they remain subject to exact manifest accounting.
The Python inventory remains 540 observed, 415 declarations, 78 exact
exclusions, and 47 missing classifications; C and Java remain exact. T011,
T076, T021, and T024 remain open.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `2218378d` | `uv run --offline --directory bindings/python --with-requirements requirements-coverage.txt --with-editable . python -m pytest tests/test_client.py::test_public_native_handle_wrappers_reject_direct_cffi_handle_adoption tests/test_client.py::test_live_batch_handle_cannot_be_adopted_by_a_second_wrapper -q` | Expected failure: both tests reported `DID NOT RAISE InvalidInputError`, proving public wrapper construction accepted arbitrary and duplicate Batch handles. |
| Green | `bb081faa` | `uv run --offline --directory bindings/python --with-requirements requirements-coverage.txt --with-editable . python -m pytest tests/test_client.py tests/test_handle_ownership.py -q`; `uv run --offline --directory bindings/python --with-requirements requirements-coverage.txt --with-editable . python -m pytest -q`; `git diff --check` | PASS: focused lifecycle/client tests 100/100; Python package suite 166/166; direct adoption, invalid token/null, duplicate Batch, view-owner retention, and add_item/execute transfer/restore tests pass. |
| Refactor | This commit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | The audit writes the refreshed partial snapshot and exits 1 because Python retains 47 missing classifications; C/Java match exactly and all scanner issues/kind mismatches remain zero. The task checkpoint and audit guide describe the fixed ownership guard without marking parity complete. |

### T024 Python native-wrapper initializer inventory — exact exclusions (2026-10-14)

The 29 Python `NativeHandle` subclass `.__init__` paths are now classified as
exact exclusions. Runtime Red/Green evidence above proves that a wrapper
constructor requires the private identity token, rejects direct raw-handle
adoption before subclass initialization, and can only be reached through the
private `_wrap_owned_handle` factory used by production callsites. This
exclusion covers only each exact initializer path; all wrapper type records
remain inventoried, and public constructors/factories remain declared and
mapped or blocked according to their existing readiness. The two TTLV view
types and their blocked construction operations remain blocked.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `132a3e4f` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_python_native_handle_initializers_are_exact_internal_factory_exclusions -v` | Expected failure: all 29 observed native-wrapper `.__init__` paths lacked exact exclusions; the test also protects wrapper type rows and representative public factories from being hidden or promoted. |
| Green | `cf256253` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `python -B tools/api_manifest/generate.py --write`; `python -B tools/api_manifest/generate.py --check`; `wsl --exec bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/0043-t076-client-mapping/KMIPKit && python3 -B -m unittest tools.api_manifest.tests.test_generate'` | PASS: T076 mapping suite 22/22; generator suite 57/57; generation and check verified all 12 outputs. The manifest records exactly 29 new initializer exclusions; no public factory or blocked TTLV view state changed. |
| Refactor | This commit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | The pinned audit refreshed the report and exited 1 as expected: C 264 observed/237 declarations/27 exclusions and Java 469/434/35 both match exactly; Python 540 observed/415 declarations/107 exclusions leaves 18 missing. Scanners are complete with no stale paths, kind mismatches, or issues. The remaining 18 are six inherited context-manager methods, `TrustSource`, nine exception constructors, `ClientExecutionError.category`, `ExtensionRegistryLimits.__init__`, and `Version.__init__`. T011, T076, T021, and T024 remain open. |

### Python final shared-export classification (2026-10-14)

The last 18 observed Python package exports are now classified without changing
runtime behavior: three declarations map `TrustSource`, the operational
`ExtensionRegistryLimits` constructor, and the stable
`ClientExecutionError.category`; exact exclusions cover six context-manager
syntax methods, eight local exception constructors, and the local
`Version.__init__` snapshot constructor. Each exclusion has its own exact
`exportName` and approved rationale. This completes the structural scanner
inventory only. It does not settle semantic equivalence questions in the C
transport field mappings, including timeout `DEFAULT`, all allowed PEM/DER/file
input variants, the reused `transport_input.data` carrier, slice counts, and
the RAW_TLS-only `endpoint_port` condition. Those remain review gates; no task
checkbox was changed.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `cf358e53` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_remaining_python_exports_are_mapped_or_exactly_excluded -v` | Expected failure listed the exact 18 unclassified exports across context-manager methods, `TrustSource`, exception constructors, `ClientExecutionError.category`, and the two dataclass constructors. |
| Green | `8aba7d66` | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `wsl --exec bash -lc 'cd /mnt/c/Users/ramp1953/.codex/worktrees/0043-t076-client-mapping/KMIPKit && python3 -B -m unittest tools.api_manifest.tests.test_generate'`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | PASS: T076 mapping suite 23/23; generator suite 57/57; all 12 generated outputs current. |
| Refactor | This commit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check`; `python -B tools/api_manifest/generate.py --check`; `git diff --check` | PASS: report refreshed; C 264 observed (237 declarations, 27 exclusions), Java 469 (434, 35), and Python 540 (418, 122) all have `matchesManifest=true`, complete scans, and zero missing/stale/kind-mismatched/issue paths. Semantic field review and required parity/CI remain open, so T011, T076, T021, and T024 remain open. |

### T076 semantic transport field review — partial promotion (2026-10-14)

This follow-up closes the semantic review gates left open at the preceding
structural-inventory checkpoint. The C `endpoint_kind` selection is supported
by the RAW_TLS and HTTPS constructors; the port is conditional on RAW_TLS,
while HTTPS carries its authority and optional port in the absolute endpoint
URI and keeps the KMIP target as a separate origin-form path/query. Timeout
DEFAULT selects the approved Rust `TimeoutPolicy::default()` behavior, with
bounded and unbounded policies remaining distinct. The credential mode matrix
and exact carrier exclusions are recorded above. No new exports or transport
behavior were introduced. The audit snapshot did not change because every
observed export was already accounted for; semantic correctness required this
additional review and regression coverage beyond the structural scanner.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `f5c38f0d` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_transport_field_mappings_cover_policy_modes_and_exact_c_carriers -v`; `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage -- --nocapture` | Expected mapping failure: the Rust trust target did not identify `TrustSource::certificate_authorities`; C ABI consumer checks passed 3/3 before the mapping correction. |
| Red | `2d9901ba` | `cargo test -p kmipkit-ffi --lib capability_query_only_reports_c_mapped_nonblocked_capabilities` | Expected failure after updating the assertion: the Rust facade omitted ready capability `client_batch_model` (0 passed, 1 failed). |
| Green | `f57988ca` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `cargo test -p kmipkit-ffi --lib`; `python -B tools/api_manifest/generate.py --check`; `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage -- --nocapture` | PASS: T076 mappings 24/24; FFI unit tests 57/57; all 12 generated outputs current; C ABI coverage consumer 3/3. The generated C header and parity fixtures preserve exact ABI layouts. |
| Refactor | This commit | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_generate -v`; `python -B tools/api_manifest/audit_binding_exports.py --check --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json`; `python -B tools/api_manifest/generate.py --check`; `cargo fmt --all --check`; `git diff --check` | PASS: generator suite 57 passed, 2 Windows symlink-permission skips; audit exit 0 with C 264 observed/237 matched, Java 469/434, Python 540/418, each `matchesManifest=true` with zero missing, stale, kind-mismatched, or issue paths; all 12 generated outputs verified. T011, T076, T021, and T024 remain open pending remaining parity and CI review; `kmipkit_client_execute_with_error_ttlv_v1` remains blocked. |

### T076 exchange-local options and transport mapping reconciliation (2026-10-14)

The shared `RequestOptions` value adds per-exchange connect, write, read, and
total timeout overrides through the versioned
`kmipkit_client_execute_with_options_v1` ABI. The original
`kmipkit_client_execute` symbol and signature remain unchanged and inherit the
configured client policy. Timeout kind 0 inherits that policy, kind 1 is
bounded (including zero as an immediate deadline), and kind 2 is unbounded.
The generic `kmipkit_client_execute_with_error_ttlv_v1` helper remains blocked.

Transport field mappings now point only at existing public Rust constructors
and builder types. `encoding` and `source_kind` are exact C ABI discriminant
exclusions; the shared parent configuration fields represent complete input
semantics across identity, trust, and revocation contexts. The exact
`kmipkit_transport_config_v1_t` carrier remains excluded from cross-language
capability matching while its fixed-width layout is still generated in the C
header. Java native-adapter tests cover identity and trust PEM/DER from memory
and selected files, including trust-from-file. Python adapter tests cover the
same identity/trust source and encoding combinations. CRLs cover in-memory
PEM/DER only under KMIPKIT-0013-FR-003/004. Existing C consumer assertions
verify result-reason absent and present values; the metadata capability test
confirms `production_transport` is in the C-mapped ready set and available in
the Rust facade.

The export snapshot now inventories C 272 observed paths (242 declarations,
30 exact exclusions), Java 481 (442, 39), and Python 556 (425, 131). Every
scanner is complete with `matchesManifest=true` and zero missing, stale,
kind-mismatched, or issue paths. The Java additions are classified by exact
path: `NativeExtensionRegistry.clientExecuteWithOptions` is raw-handle JNI
bridge plumbing, while `RequestOptions.equals`, `hashCode`, and `toString` are
compiler-generated record methods. This closes the export-inventory gaps
introduced by RequestOptions; it does not close T076 or the related task gates.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `3742f0e2`, `00cb6200`, `fb676794`, `085f7447`, `5c2074c6`, `563ce7e7` | Focused assertions in `tools.api_manifest.tests.test_t076_client_mapping` for exact C targets, aggregate source modes, excluded carrier, canonical mappings, and RequestOptions artifacts | Expected failures identified incorrect/non-exported mode targets, the ready mapping's excluded C carrier, unsorted mappings, and unclassified generated RequestOptions dataclass members. |
| Green | `9356bbf1` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B tools/api_manifest/generate.py --write`; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B tools/api_manifest/generate.py --check`; `cargo test -p kmipkit-ffi --lib request_options_v1 -- --nocapture`; `cargo test -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage -- --nocapture`; `mvn -f bindings/java/pom.xml "-Dtest=ClientFacadeContractTest,ClientFacadeTlsIntegrationTest" test`; `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_client_execute_with_options_preserves_timeout_kinds_and_consumes_batch bindings/python/tests/test_client.py::test_transport_record_forwards_identity_and_trust_source_modes` | PASS: T076 mappings 32/32; all 12 generated outputs current; FFI RequestOptions 2/2; C ABI coverage consumer 3/3; Java facade suite 14/14 with 0 failures/errors/skips; Python adapter tests 5/5. The C tests assert result-reason absent and present; the production transport capability is present in the C-mapped ready set and available in the Rust facade. The stable execute symbol remains present and the blocked generic helper remains absent from the generated public header. |
| Red | `b59d9217` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_java_request_options_bridge_and_record_artifacts_are_exactly_accounted -v` | Expected failure listed the four exact Java exports missing from the manifest: the new JNI bridge method and three generated RequestOptions record methods. |
| Green | `b604b984` | Same focused test; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping -v`; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B tools/api_manifest/generate.py --write`; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B tools/api_manifest/generate.py --check`; `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_generate -v`; pinned binding-export audit with `--check` | PASS: focused Red regression 1/1; T076 mapping tests 33/33; generator tests 57 passed with 2 Windows symlink-permission skips; all 12 outputs current; audit exit 0 with C 272/242, Java 481/442, Python 556/425, and every package `matchesManifest=true`. |
| Refactor | This commit | `git diff --check`; inspect the refreshed audit report and evidence against generated outputs | Record the exact partial gate state and clarify that structural export matching does not complete T076, T021, T024, or T011. |

The recorded internal QA review and automated security diff scans are complete for their reviewed ranges; they do not replace the qualified human security review required before release. Required CI on the final PR head and complete four-language semantic parity remain outstanding. T011, T076, T021, and T024 remain open; no generic error-TTLV API was promoted.

### T076 Python native-constructor input evidence and FFI error output (2026-10-14)

The Python `Client.create` adapter test now captures the record at the
`kmipkit_transport_config_create_v1` native-call boundary and asserts that
client certificate chains, private keys, and trust certificates preserve each
supported PEM/DER and memory/file discriminator with the corresponding bytes
or encoded path. CRL inputs are asserted as in-memory PEM/DER. This strengthens
adapter evidence without changing Python production behavior.

The versioned C execute function now initializes a valid response output slot
to null before validating its options pointer. A null options pointer returns
`KMIPKIT_ERROR_INVALID_INPUT` and leaves the output slot null, matching the
output contract used by the other C entry points.

The T076 checkpoint now uses the pinned report's current inventory totals: C
272 observed (242 declarations, 30 exclusions), Java 481 (442, 39), and Python
556 (425, 131). Each scanner is complete and `matchesManifest=true`; this does
not close T076 or the related task gates.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `31c5080c` | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_client_create_forwards_identity_and_trust_modes_to_native_constructor`; `cargo test -p kmipkit-ffi --lib execute_with_options_invalid_options_nulls_response_output -- --nocapture` | Expected failures: Python produced 4 failing parameter cases because the native-constructor stub recorded only `abi_version`; FFI produced 1 failure because the stale response pointer remained unchanged for null options. |
| Green | `2afbdaee` | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_client_create_forwards_identity_and_trust_modes_to_native_constructor bindings/python/tests/test_client.py::test_transport_record_forwards_identity_and_trust_source_modes`; `cargo test -p kmipkit-ffi --lib execute_with_options_invalid_options_nulls_response_output -- --nocapture` | PASS: Python focused adapter tests 8/8; FFI output-slot regression 1/1. Cargo emitted nonfatal AWS-LC `stdalign_check.c` and missing-NASM feature-probe warnings, then compiled and passed. |
| Refactor | This commit | `git diff --check` | PASS: checkpoint totals and phase evidence are updated; no task checkbox changed. |

### T076 C export-boundary count and workspace verification (2026-10-14)

The full Rust workspace run exposed that the shared panic-boundary metadata
test still expected 155 C exports after the approved options-aware execute
entry point raised the observed total to 156. The test expectation now tracks
the current ABI surface. This is test accounting only; it changes no runtime
behavior or public ABI. The test suite then passed across all workspace crates
and doctests on Windows. AWS-LC emitted nonfatal compiler-feature and missing
NASM probe warnings.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `04cd36a4` | `cargo test --workspace --all-features` | Expected failure: `every_exported_c_function_starts_inside_the_shared_panic_boundary` observed 156 exports while the assertion still expected 155. |
| Green | `c1fd34be` | `cargo test --workspace --all-features`; `cargo fmt --all --check`; `git diff --check` | PASS: full workspace tests and doctests, formatting, and whitespace checks; no failing tests. |
| Refactor | This commit | Review the updated T076 checkpoint and evidence table | Record the export-count correction and distinguish automated review from the required human release review; task status remains open. |

The T076 completion gates remain open: final PR CI, complete four-language
semantic parity, and independent human semantic/security review. The public
binding-export inventory is structurally complete at the latest pinned report,
but it does not prove semantic parity or release readiness.

### T076 public release mappings and selected FFI coverage correction (2026-10-14)

The public API manifest now declares the existing C release functions for
`ClientConfiguration` and `ClientExtensionRegistry` with exact Rust, C, Java,
and Python mappings. The deterministic generator refreshed the public C
header, Java/Python API metadata, and parity fixtures. The C consumer now uses
the generated public header to exercise client configuration ownership,
request-option validation, typed execution, and handle release through a local
TLS 1.3 mTLS peer. Java and Python regression tests cover their equivalent
release and request-option facade paths.

Investigation of the selected FFI coverage inputs found that the Linux
collector ran only `tests/c_api_coverage.rs`, leaving the FFI crate's internal
unit tests out of the measured profile. Those unit tests exercise defensive
ABI branches that cannot be manufactured through a public C handle. The
collector now runs all FFI targets and exports both the library unit-test
executable and the compiled C consumer executable; the coverage thresholds
remain unchanged. The isolated Linux run reported line coverage of 95.01% for
`extension_registry.rs` and 90.57% for `extension_registry/client_abi.rs`.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `9cfca080` | `python -m unittest scripts.tests.test_workflow.WorkflowContractTests.test_ffi_coverage_collection_runs_unit_and_public_c_consumer_targets -v` | Expected failure: the collector only selected the C integration target and did not include the library unit-test binary. |
| Green | `35acc859` | `python -m unittest scripts.tests.test_workflow -v`; isolated Linux `bash scripts/collect_ffi_coverage.sh target/t076-ffi-report-2/combined` with `CARGO_TARGET_DIR=target/t076-ffi-target-2` | PASS: workflow contract tests 43/43; FFI unit tests 60/60 and C integration tests 4/4; coverage collection and report normalization succeeded at 95.01% and 90.57% line coverage for the two scoped Rust files. |
| Red | `1367fdf3` | `python -m unittest tools.api_manifest.tests.test_t076_client_mapping -v` | Expected failure before manifest promotion: both release declarations were still blocked. |
| Green | `da6cfb09` | T076 manifest tests; `python -B tools/api_manifest/generate.py --check`; focused Java C/JNI tests; focused Python adapter tests; `cargo test --locked -p kmipkit-ffi --test c_api_coverage --all-features` | PASS: manifest tests 34/34; 12 generated outputs current; Java tests 18/18; Python tests 135/135; Windows C ABI integration tests 4/4. |
| Refactor | This commit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check`; `git diff --check` | PASS: C 272 observed / 242 exact matches, Java 481 / 442, Python 556 / 425; no missing, stale, kind-mismatched, or issue paths. |

The shared batch fixture and Rust, C, Java, and Python regressions now cover an
ordered three-item result (`Success`, `Operation Pending`, `Operation Failed`),
matching Unique Batch Item IDs, and the exact `00a17f` Pending correlation value.
The Rust fake transport, C mTLS consumer, and Java mTLS facade test each assert
one exchange; Python asserts the same copied adapter result mapping. No layer
polls or follows up automatically. Final PR-head CI and independent human
semantic/security review remain outstanding. This checkpoint does not complete
T011, T071, T076, or release readiness.

### T076 ordered mixed-outcome parity follow-up (2026-10-14)

The shared `batch.ordered-completed-pending-failed.v1` fixture now records
three stable request IDs, the exact ordered statuses, one Pending correlation
value, and one transport exchange. Each consumer has a named regression: Rust
uses the scripted transport; C drives the generated public ABI over local TLS
1.3 mTLS; Java drives the facade over local TLS 1.3 mTLS; Python validates the
same ordered FFI-to-dataclass mapping. OASIS KMIP v2.1 §9.21 says the field is
optional and that a response SHALL echo it when present in the request; the
tests use it to assert request/response correlation, not as a requirement that
every multi-item request carry an ID. The server peer returns one response
message, and the clients issue no follow-up exchange.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `e18db839` | Focused Java mTLS and C ABI mixed-outcome regressions before request IDs were assigned | Expected failure before network exchange: multi-item client batch validation requires a Unique Batch Item ID for every item. |
| Green | `faf14ad3` | `mvn -q -f bindings/java/pom.xml "-Dtest=ClientFacadeTlsIntegrationTest,SharedCapabilityParityFixtureTest" test`; `cargo test --locked -p kmipkit-client --lib ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange -- --nocapture`; `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls -- --nocapture`; `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_batch_result_preserves_ordered_mixed_outcomes_ids_and_pending_correlation`; `cargo fmt --all --check`; JSON validation; `git diff --check` | PASS: Java integration and fixture contracts; Rust fake transport; C mTLS public consumer; Python ordered mapping. Each reports the three statuses, all IDs, exact correlation bytes, and one exchange where network execution is exercised. |
| Refactor | This commit | Rerun the focused Java integration and shared-fixture tests after factoring the stable ID helper; `cargo fmt --all --check`; JSON validation; `git diff --check` | PASS: tests retain fixture semantics with one helper for consistent request/response IDs. Rust/C/Python focused tests passed in the Green phase; final PR-head CI remains required. |

T076 remains open. The mixed-outcome parity gap is covered locally. The latest
published CI run (`38522806517`) passed the platform, adapter, sanitizer,
documentation, inventory, and other selected checks, but the aggregate changed
code coverage gate failed at 499/527 (94.69%), below the 95% requirement.
Independent human semantic/security review and a passing CI run on the final
PR head also remain gates.

### T076 shared parity contract and changed-code coverage follow-up (2026-10-14)

The shared API parity contract now requires an explicit available or blocked
mapping for every language on every case. Available non-Java mappings identify
the repository-relative consumer source and exact test function; Java mappings
identify the JUnit class and method. The validator confirms that each available
consumer target exists. Per-case blockers are recorded for the API-level
taxonomy assigned to T077 and for C/Python consumers that do not yet exercise
the exact open-enumeration and child-order case; Rust's separate tests do not
consume this exact case either. The four languages each have
at least one available consumer overall; that aggregate does not imply every
case is complete in every language. The contract runs in the cross-platform
Python workflow.

The failed aggregate coverage run showed two uncovered Python wrapper fallback
lines in `bindings/python/src/kmipkit/extensions.py`: the registry lookup when
the Python definition cache lacks a native result, and the information lookup
when native information exists but cached Python metadata is absent. Focused
tests exercise both paths while asserting that the native identity and TTLV
value remain intact. Running the full Python coverage producer locally reports
98.56% total coverage and records one hit at each changed fallback line. The
coverage thresholds remain unchanged.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `2a2acd00` (CI run `38522806517`) | GitHub `Coverage gate / selected scopes` | Expected failure: changed production coverage was 499/527 (94.69%), below the 95% gate. |
| Red | `c9a8689d` | `python -B -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_fixture_maps_executable_consumer_contracts -v` | Expected failure before fixture completion: five cases had no Rust, C, or Python mapping; the union-only check had hidden those per-case gaps. |
| Green | `f7e7caa7` | Focused fallback tests; full Python coverage suite; `python -B -m unittest tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_t076_client_mapping -q`; mapped Rust client/protocol/extension tests; `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage`; focused Java parity, extension, and client TLS tests | PASS: Python 183/183 at 98.56%; contract tests 42/42; mapped Rust tests passed; C consumer 4/4; Java selected suites passed. Final PR-head aggregate gate remains pending. |
| Refactor | This commit | `python -B -m unittest scripts.tests.test_workflow -q`; `python -B tools/api_manifest/generate.py --check`; `cargo fmt --all --check`; `git diff --check` | Local checks pass; final PR-head CI and aggregate changed-code gate are pending. |

This follow-up does not change production behavior or lower coverage gates.
At the time of this checkpoint, T076 remained open pending final-head CI and
coverage plus independent semantic mapping review. The qualified human security
review remains a 1.0 release gate; it is not represented as completed here.

### T076 consumer-mapping review corrections (2026-10-14)

An independent mapping audit found that the C configuration consumer used a
different fixture shape from the shared case, the C batch mapping pointed at
the wrong function, and the Python batch adapter test did not assert one
execution call. The C registry test now gives each configuration its own
single definition and verifies counts `[1, 1]` with cross-identity lookups
absent. The C batch mapping now points to `execute_public_tls_request`, which
the compiled C consumer harness invokes over TLS 1.3 mTLS. The Python batch
consumer now calls `Client.execute`, verifies all three ordered outcomes, IDs,
and Pending correlation bytes, and asserts one FFI execute call.
That Python facade test uses a stub FFI library, so it proves one native execute
invocation and the returned mapping; it does not directly count network
exchanges. The shared mapping now cites the Rust client test for the native
one-exchange behavior. The C mapping separately cites its compiled TLS 1.3
mTLS harness, which counts exactly one server exchange while invoking the C
consumer.

For schema redaction, the C ABI exposes only numeric status, category, cause,
and delivery outputs; it has no diagnostic text channel. Its consumer checks
repeated schema errors remain the same stable category. The parity validator
checks this numeric-only signature and records `diagnosticSurface: none`, so
the C mapping makes no unsupported claim about redacting a diagnostic string.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `df6caa95` | `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process -- --nocapture` | Expected failure: the changed shared fixture expected one alpha definition, while the C consumer configured two. |
| Green | `0a1a0d2d` | Same focused C consumer test | PASS: separate one-definition registries match counts and identity isolation for alpha and beta. |
| Red | `0706d1e1` | `python -B -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_fixture_maps_executable_consumer_contracts -v` | Expected failure: the newly mapped Python batch consumer function did not yet exist. |
| Green | `c5a3c307` | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_batch_execute_preserves_ordered_mixed_outcomes_and_uses_one_exchange` | PASS: the Python facade copies ordered outcomes, IDs, and exact Pending correlation after exactly one FFI execute call. |
| Red | `853e71da` | Shared parity mapping contract test | Expected failure: the C schema-redaction mapping was blocked despite the numeric-only ABI and existing invalid-schema category test. |
| Green | `2b10df1e` | Shared parity mapping contract test; full C consumer suite | PASS: the contract verifies no diagnostic string output in the C last-error signature; the C consumer passed 4/4. |
| Refactor | This commit | `python -B -m unittest tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_t076_client_mapping -q`; `python -B -m unittest scripts.tests.test_workflow -q`; `python -B tools/api_manifest/generate.py --check`; `cargo fmt --all --check`; `git diff --check` | PASS: contract tests 42/42; workflow tests 43/43; 12 generated API outputs current; formatting and whitespace checks clean. Updated PR-head CI and aggregate changed-code coverage remain pending. |

The shared fixture continues to mark API-level discovery as blocked for T077
and the exact generic TTLV open-enumeration/child-order case as blocked for the
Rust, C, and Python consumers. These are downstream parity items owned by T077
and T029/T071 respectively; they are not T076 exit gates and are not claimed as
four-language parity evidence.

### T076 composite batch exchange evidence (2026-10-14)

The batch fixture now distinguishes each adapter's observable assertions from
supporting evidence for the shared native transport behavior. C verifies the
ordered mixed statuses, IDs, and Pending correlation in its public ABI
consumer; its Rust TLS harness verifies exactly one mTLS server exchange.
Python's test name and mapping state its actual assertion precisely: the
facade invokes the native execute function once and maps ordered outcomes,
IDs, and Pending correlation. A Rust client test is linked as supporting
evidence that the shared execute operation performs one transport exchange.
The Python facade test itself does not claim to observe a real network
exchange.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `1539d541` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_fixture_maps_executable_consumer_contracts -v` | Expected failure: C and Python batch mappings did not yet declare adapter assertions or supporting evidence for `transportExchangeCount`. |
| Green | `f22e992e` | Shared parity and T076 mapping tests; focused Python batch facade test; `cargo test --locked -p kmipkit-ffi --features coverage-c-consumer --test c_api_coverage generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls -- --exact` | PASS: 42 manifest tests; Python adapter 1/1; compiled C mTLS consumer 1/1 and observed exactly one server exchange. The parity validator verifies the cited Rust source contains the corresponding one-exchange assertion. |
| Refactor | This commit | `git diff --check`; review fixture labels and implementation evidence against test behavior | Corrected the Python test name and recorded adapter-level versus transport-level evidence without overstating what the fake FFI observes. |

This closes the batch evidence mapping correction only. T076 remains open for
its final-head CI/coverage gates and confirmation of all T076-specific semantic
mappings. The blocked API taxonomy and generic TTLV fixture cases are assigned
to downstream tasks, not T076.

### T076 export audit and gate-scope reconciliation (2026-10-14)

At head `f2d9a661a5288c22712157ad423fe3d323f4028e`, the pinned binding export
check completed successfully:

| Package | Observed paths | Exact manifest matches | Missing | Unobserved | Kind mismatches | Issues |
|---|---:|---:|---:|---:|---:|---:|
| C | 272 | 242/242 | 0 | 0 | 0 | 0 |
| Java | 481 | 442/442 | 0 | 0 | 0 | 0 |
| Python | 556 | 425/425 | 0 | 0 | 0 | 0 |

Command: `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check`.
All observed paths outside the manifest are covered by the audit's exact exclusions.

The T076 exit gates are the shared client declarations and their approved
four-language mappings, compiled public C consumer and mixed-batch evidence,
complete binding-export audit, semantic review, and required final-head CI.
`api-levels.shared.v1` is owned by T077. The exact generic TTLV
open-enumeration/child-order parity case is tracked by T029/T071. Those cases
remain blocked until their owner work is complete, but they are not T076 exit
gates.

At the time this record was written, GitHub run `38527919451` had 31 successful
checks and 2 skipped checks; `Coverage / macos` was still running. Independent
semantic mapping review was in progress. No result is claimed for either pending
gate.

### T076 HTTPS TLS verification-name parity (2026-10-14)

Java and Python now expose the accepted KMIPKIT-0013 TLS verification-name
override for HTTPS endpoints with IP literals. The override remains a distinct
field from the HTTPS URI, so it cannot change the URI authority or HTTP Host.
Rust remains authoritative for rejecting a name override on a non-IP endpoint.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `ef388850` | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_https_tls_server_name_is_forwarded_separately_from_authority`; focused Java integration test | Expected failures: Python factory rejected the `tls_server_name` keyword; Java validation rejected the HTTPS override as raw-TLS-only. |
| Green | `032a10c1` | Same focused Python test; `mvn -q -f bindings/java/pom.xml -Dtest=ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsMemoryAndFileIdentityAndTrustInPemAndDer test` | PASS: Python C record preserves the IP URI, independent TLS name, and target URI; Java builds a validated HTTPS client configuration with an IP endpoint and DNS SAN name. |
| Refactor | This commit | Re-run both focused tests; `git diff --check` | Clarified the Java/Python API documentation and the integration test name; authority and verification-name fields remain separate. |

### T076 Java mixed credential inputs and builder key cleanup (2026-10-14)

Java now supports independent memory/file sources and PEM/DER encoding for
client certificates, private keys, and trust certificates, plus independent
PEM/DER memory inputs for revocation lists. The private-key input is consumed
after its bytes are copied into builder-owned storage. `Builder.close()` now
zeroizes that owned copy when a builder is abandoned and makes the builder
terminal; `build()` and successful key replacement retain their existing
cleanup behavior. Caller-owned arrays are not modified. This is Java-specific
lifecycle hygiene; runtime copies outside KMIPKit remain outside its guaranteed
zeroization boundary.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `9de004b2` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride' test` | Expected compile failure: `TransportConfiguration.TransportInput` was not yet available to the integration contract. |
| Green | `670f743d` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=TransportConfigurationBoundaryTest,ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride' test` | PASS: 15 tests; mixed selector inputs cross JNI and existing endpoint/credential checks remain intact. |
| Refactor | `98c49d34` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=ClientFacadeTlsIntegrationTest#nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride' test` | PASS: valid PEM and DER CRLs are forwarded together through the Java/JNI path. |
| Red | `16a37241`, `e9018107` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=TransportConfigurationBoundaryTest#closingUnbuiltBuilderZeroizesItsOwnedPrivateKeyCopy+closingUnbuiltBuilderZeroizesKeyCopiedFromLegacyPemSetter' test` | Expected compile failure: `Builder.close()` did not exist for the mixed `TransportInput` or legacy PEM path. |
| Green | `6b87453c` | `mvn --batch-mode --file bindings/java/pom.xml '-Dtest=TransportConfigurationBoundaryTest#closingUnbuiltBuilderZeroizesItsOwnedPrivateKeyCopy+closingUnbuiltBuilderZeroizesKeyCopiedFromLegacyPemSetter' test` | PASS: both owned key copies are zeroized; repeated close is safe; the original caller array is unchanged; closed builders reject build or mutation. |
| Red | `f2678ec0` | `uv run --offline --no-project --with-requirements tools/api_manifest/requirements-test.txt python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_transport_input_facade_exports_are_exactly_reconciled -v` | Expected assertion failure: the new Java `Builder.close` export had no approved exact exclusion. |
| Green | `7430a42c` | Focused `test_transport_input_facade_exports_are_exactly_reconciled`; `python -B tools/api_manifest/generate.py --write` | PASS: manifest classifies `Builder.close` under accepted `KMIPKIT-0013-FR-005` cleanup evidence and regenerates the checked-in API outputs. |
| Refactor | `cc07dc10` | Focused Java transport tests; all T076 manifest tests; `python -B tools/api_manifest/generate.py --check`; `mvn --batch-mode --file bindings/java/pom.xml clean verify` | PASS: 17 focused Java tests, 34 manifest tests, 12 generated API outputs current, and full Maven verification with 144 tests (0 failures, 3 skipped); JaCoCo coverage checks passed. Build and close share one private-key cleanup helper. |

The final local binding-export audit observed 272 C, 491 Java, and 556 Python
paths. It matched all 242 C, 449 Java, and 425 Python declarations exactly, with
zero missing, unobserved, kind-mismatched, or otherwise problematic entries.
An independent read-only review of the Java key-cleanup diff found no concrete
surviving bypass or regression. The review was static and did not inspect JNI
temporary buffers; this change does not alter that boundary.
The T076 PR remains a draft pending complete CI on the updated PR head and the
remaining T076 exit gates.

### T076 Python TLS-name override validation correction (2026-10-14)

Python endpoint validation now matches the Rust transport contract for an
explicit TLS verification-name override: the endpoint host must be an IP
literal. DNS endpoints remain valid without an override, and HTTPS keeps the
verification name separate from its URI authority. The transport-record
fixture now checks that absent optional values produce no ABI flags.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `89781d20` | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests/test_client.py::test_endpoint_rejects_tls_name_override_for_dns_host` | Expected failure in both RAW_TLS and HTTPS cases: Python accepted the DNS override while Rust rejects it. |
| Green | `b602dd11` | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests`; `uv run --offline --project bindings/python --frozen --with-requirements bindings/python/requirements-coverage.txt pytest -q bindings/python/tests --cov=kmipkit --cov-config=bindings/python/pyproject.toml --cov-report=term-missing`; `uv run --offline --project bindings/python --frozen python bindings/python/examples/vendor_extension_registry.py` | PASS: 186 Python tests; 98.52% package coverage against the 85% gate; vendor extension example executes. |
| Refactor | This commit | `uv run --offline --project bindings/python --frozen pytest -q bindings/python/tests`; `uv run --offline --project bindings/python --frozen --with-requirements bindings/python/requirements-coverage.txt pytest -q bindings/python/tests --cov=kmipkit --cov-config=bindings/python/pyproject.toml --cov-report=term-missing`; `git diff --check` | PASS: 186 Python tests; 98.46% package coverage against the 85% gate. A shared IP-literal helper keeps RAW_TLS and HTTPS checks consistent; URL parse failures remain mapped to `InvalidInputError`. |

GitHub run `38532790231` completed against the earlier head `6f4f1ffa`. The
Python platform jobs and Python coverage failed on the same stale parameterized
case, which still expected an HTTPS DNS verification-name override to be
rejected after HTTPS override support had been added. Red commit `89781d20`
removed that obsolete expectation and added explicit RAW_TLS and HTTPS DNS
rejection coverage. The PR stays in draft until CI completes on the corrected
head and all T076 exit gates are met.

### T076 changed-code coverage threshold correction (2026-10-14)

CI run `38533936756` passed its platform and component coverage jobs but the
aggregate gate measured 596/628 changed production lines (94.90%). Inspection
of the merged line reports traced two uncovered changed lines to Python's
`urlsplit` error translation for an HTTPS URI with a malformed bracketed
authority. A negative test now verifies that the facade maps this malformed
authority to `InvalidInputError`.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | CI run `38533936756` | `gh run view 38533936756 --repo NeverWe1come/KMIPKit --job 115755618530 --log` | Failure: changed production coverage was 596/628 (94.90%), below 95%. Downloaded line reports showed the malformed-authority error path was not exercised. |
| Green | `1cbaf9c6` | Focused malformed-authority test; full Python coverage suite; coverage gate aggregate replay against the downloaded run reports | PASS: focused test 1/1; 187 Python tests; 98.58% Python coverage; aggregate changed production coverage 598/628 (95.22%). The aggregate replay also passed every selected component and workspace threshold. |
| Refactor | This commit | Review TDD evidence and changed-line report; `git diff --check` | Documented the CI failure and report-backed correction. |

The aggregate replay used a clean detached worktree at `d010054b` because local
editable-package test artifacts add generated CFFI wrappers that are not part
of source control. It used the run's downloaded LLVM/JaCoCo reports and a
fresh Python report collected after the new test. Full CI run `38537129710`
passed on head `53a028e7`. The PR remains a draft pending the remaining T076
exit gates and review.

### T076 Python-owned private-key zeroization and dataclass compatibility (2026-10-14)

Python in-memory transport inputs now keep KMIPKit-owned bytes in mutable
storage that is zeroized before release. `Client.create` wipes its per-call
private-key snapshot and CFFI key spans immediately after the native
transport-config constructor has synchronously copied them, including status
and exception paths. `_transport_record` also wipes partial spans if record
construction fails. The caller's original buffer and reusable
`TransportConfig` remain unchanged. Public `TransportInput.source` retains its
dataclass field behavior, `dataclasses.replace()`, equality, hashing, `bytes |`
`Path` access, and repr redaction.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `205af3d0` | Focused Python zeroization regression cases in `bindings/python/tests/test_client.py` | Expected failures against the prior implementation: private mutable ownership and `_transport_source` cleanup support were absent. |
| Red | `8589a195` | `bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests/test_client.py -q -k transport_input_preserves_dataclass_replace_and_hash_behavior` | Expected failure: `dataclasses.replace()` passed the changed `_source` field to a constructor that accepted only `source`. |
| Green | `81f882b8` | `& bindings/python/.venv/Scripts/python.exe -m pytest bindings/python/tests -q` | PASS after implementation; final suite now has 193 passing tests. Private owners and CFFI spans are wiped on success, native status failure, exceptions, and partial record-construction failure; caller buffers and reusable configs remain intact. |
| Refactor | `8a8b6047` | `python -B tools/api_manifest/generate.py --check`; T076 manifest/parity unit tests; `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check`; Python coverage suite | PASS: 12 generated outputs current; 42 manifest/parity tests; C 242/242, Java 449/449, Python 425/425 exact declaration matches; 98% overall Python coverage and 96% for `client.py`. Added FR-005 traceability and user-guide lifecycle limits. |
| Follow-up | `8026d12b` | `cargo clippy --workspace --all-targets --all-features -- -D warnings`; GitHub Actions run `38543732660` | PASS: local Clippy and all required current-head CI checks, including Linux/Windows/macOS, coverage thresholds, language bindings, C/JNI consumers, sanitizers, and inventory checks. |

An independent read-only Python lifecycle review confirmed the key-copy cleanup
paths and identified the dataclass compatibility regression before the Green
fix; regression tests now preserve both replacement and hash behavior. The
binding exporter audit is structural evidence and does not by itself prove
semantic parity. T076 remains open pending the separate whole-task QA review
and closure of any concrete four-language shared-mapping or parity gaps.

### T076 independent QA, security review, and final-head CI (2026-10-14)

The independent read-only T076 QA review found no remaining implementation
gate in the task. It verified exact shared-client mappings across Rust, C,
Java, and Python; the typed public C mTLS consumer; ordered mixed Success,
Pending, and Failure outcomes; exact Pending correlation; no automatic
follow-up; complete binding-export reconciliation; and preservation of the
blocked generic error-TTLV helper. The audit totals are C 242/242, Java
449/449, and Python 425/425 declarations, with no missing, stale,
kind-mismatched, or unobserved paths. The reviewer distinguished T021/T022,
T024/T025, T029/T071, T077/T078, and KMIPKIT-0010 as separate remaining work.

The independent security review identified one low-severity misuse risk in
the documented-internal Java JNI bridge: public
`NativeExtensionRegistry.release(int, long)` forwards known handle kinds to
Rust, where a non-null arbitrary value may be dereferenced before its kind is
checked. `NativeHandle.owned(long, int)` is also public. This is not reachable
through the typed T076 facade, but direct use of the documented-internal
bridge can crash or invoke undefined behavior. The path predates T076 for
existing handle kinds; T076 adds new handle kinds to the same bridge. At review
time it was recorded under T022 for Java ownership/bridge hardening and was
required to close before 1.0 release; the final T022 acceptance below records
its local closure. No unauthenticated network path was identified.

### T022 security follow-up (interim checkpoint; superseded by final acceptance below)

- Red commit `01254256` adds an isolated-process FFI regression for forged,
  wrong-kind, and stale handle values. Before the registry change, the forged
  `0x1000` token terminated the child process while the parent test reported a
  failure, keeping the unsafe dereference out of the test runner process.
- Red commit `6bd59a62` adds a low-token enumeration case. The sequential
  candidate failed as expected when value `1` reached the live handle.
- Red commit `9e2ae3a2` adds a deterministic cycle-walk bound test; it failed
  to compile before the bounded helper and limit were implemented.
- Green commit `b04d7d70` stores each `Arc<Handle>` in a global
  mutex-protected table. An AWS-LC-generated HMAC-SHA-256 key drives an
  eight-round Feistel permutation of a monotonic sequence; cycle walking keeps
  tokens nonzero and within the positive pointer domain. Each walk performs at
  most 128 permutations and returns `ERROR_RESOURCE_LIMIT` on exhaustion; the
  sequence remains consumed. The induced mapping is bijective, so released
  tokens are never reused without retaining an unbounded tombstone set. Lookups
  validate liveness and kind without dereferencing token values. Both release
  and consuming operations remove entries through the same checked path. C
  signatures and release symbols remain unchanged. Tokens must not be persisted
  across process restarts or transferred between library images; numeric
  collisions across independent images remain theoretically possible.
- Focused Windows checks pass for the cycle-walk limit, keyed-token
  uniqueness/range, exhaustion, guessed low values, forged access/release,
  wrong-kind access/release, valid access after rejected wrong-kind release,
  repeated release, and stale access. `cargo clippy -p kmipkit-ffi
  --all-targets --all-features -- -D warnings` and
  `cargo fmt --all --check` pass; `cargo test -p kmipkit-ffi --all-features`
  passes 64 unit and 4 C consumer tests. `mvn --batch-mode --file
  bindings/java/pom.xml verify` passes 145 tests (3 documented skips) and the
  configured JaCoCo gate. The JNI integration test also calls release with a
  forged positive `long` and completes safely. At this interim checkpoint, the
  independent security review found no surviving concrete bypass in the
  candidate and its bounded-walk concern was addressed. The later final T022
  acceptance record below supersedes this checkpoint's pending-review status.

Historical interim CI run `38545593697` completed successfully on PR head
`bd4390023bd24dbf0e245190fdf1f1e6b5013f12`, including the Rust stable/MSRV
matrix on Linux, Windows, and macOS; selected coverage gates; Java/JNI and C
consumers; Python bindings and coverage; sanitizers; dependency policy; and
normative inventory checks. The informational nightly and inactive-release
schedule checks were skipped by workflow conditions. This run is an interim
checkpoint superseded by the final T022 acceptance record below. It predates
the ownership commits and does not validate them. T076 is complete. The Java
bridge finding assigned to T022 has an implementation in local commits
`01254256`, `6bd59a62`, `9e2ae3a2`, and `b04d7d70`; local Windows Rust/C and
Java/JNI checks pass. At this interim checkpoint, cross-platform CI was still
outstanding; the subsequent T022 code-head run is recorded below. PR #104
remains a draft while the other KMIPKIT-0043 tasks continue.

### T022 final acceptance (2026-10-14)

T022's Java child-owner retention and deterministic close behavior, together
with the forged/stale JNI handle hardening, are complete locally. The
implementation history records behavior commits `c719aaa6` (Red), `81aaaeaf`
(Green), and `6f796605` (Refactor), followed by evidence commit `8d740e3c`.
Commit `1eb8c0ec` corrects the deterministic owner-retention test, and
`ec1c92f1` corrects stale evidence statements in this documentation.

Focused Java ownership/lifecycle checks pass 16 tests. After deterministic
test correction `1eb8c0ec`, Windows
`mvn --batch-mode --file bindings/java/pom.xml verify -q` passed at HEAD
`867d46dee98851c5a46e480ba126ff8e0f709ca3`: 153 tests, zero failures, zero
errors, and 3 documented skips. JaCoCo reports 95.43% coverage (1,461
covered, 70 missed). Independent review approved the deterministic
owner-retention test and the documentation correction in `ec1c92f1`.

GitHub Actions run `38559053899` passed on the validated T022 code head
`6f1f06c729c074ae2eef304a6a1f541930670e0d`: 36 jobs completed, 0 failed, and
2 were conditionally skipped. This validates the T022 code head; it does not
cover any subsequent T023 changes. Earlier successful run `38556076834` covered only ancestor
`8b3314c676c06c276b2bd1d0121193f631390da0`; it does not validate the ownership
commits. PR #104 remains a draft pending the remaining PR matrix and feature
exit gates.

### T021 Java shared facade acceptance (2026-10-14)

The T021 review used the current worktree rather than the earlier checkpoint
summary. The Java facade constructs a production client from separate
validated client and transport configurations, accepts only typed batches for
execution, and exposes ordered common result metadata, raw forward-compatible
outcome/status values, unique item IDs, and exact Pending correlation bytes.
It adds no raw request or arbitrary Transport injection and makes no automatic
retry or follow-up call. Existing generic TTLV wrappers expose structurally
valid in-memory values through the C ABI. Operation-specific APIs remain
gated by T031-T069.

All five Java source packages (`org.kmipkit`, `extensions`, `ttlv`,
`generated`, and `internal`) carry JSpecify `@NullMarked`. Shared native
wrappers use `AutoCloseable` and the common `NativeHandle` owner; the typed
mTLS consumer exercises client construction, batch execution, response
ownership, and close-on-scope-exit. The execution path maps protocol and
transport failures to stable exception subtypes with status/category/cause/
delivery codes and sanitized fixed diagnostics. The protocol and transport
integration regressions verify that their messages omit fixture paths.

The test-only coverage addition is commit `13bc84b8` (`test(java): cover
shared client lifecycle and nullness`). It explicitly checks the T076 client,
batch, result, and transport wrapper classes are `AutoCloseable`, and checks
all five package annotations. No production behavior or generated output
changed, so this closeout does not claim a new Red/Green/Refactor behavior
cycle. Independent review found the ownership inventory name broader than its
scope; the follow-up narrows it to the T021 shared-facade resource contract.
Operation-specific request ownership remains with its owning feature task.

| Verification | Command | Result |
|---|---|---|
| Focused Java lifecycle/nullness contract | `mvn --batch-mode --file bindings/java/pom.xml -Dtest=JavaFacadeContractTest test` | PASS: 4 tests; JNI bridge compiled. |
| Java adapter verification | `mvn --batch-mode --file bindings/java/pom.xml verify` | PASS: 145 tests, 0 failures/errors, 3 documented platform skips; JaCoCo gate passed. |
| Generated API outputs | `python -B tools/api_manifest/generate.py --check` | PASS: all 12 generated outputs current. |
| Binding export audit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check` | PASS: C 242/242, Java 449/449, Python 425/425 exact declaration matches; observed path exclusions reconcile, with zero issues, missing paths, stale paths, or kind mismatches. |

The Maven native build emitted non-fatal AWS-LC compiler-probe and missing
NASM warnings in this Windows environment; JNI compilation, tests, example,
and coverage verification all completed successfully. The current Java source
surface required no production repair. T022 child-owner retention is accepted
above; T029/T071 complete cross-language fixture gate remains separate work.

## T025 — Python wrapper ownership and fixed-width input boundaries

Python client methods now validate every borrowed handle before transferring a
batch or item wrapper. The shared CFFI call path validates and canonicalizes
fixed-width `int*_t` and `uint*_t` arguments before CFFI conversion and before
consuming ownership. Unknown native TTLV item-type codes map to the stable
Python `InvalidInputError`, matching the Java `InvalidInputException` boundary.
These changes close the T025 Python portion of KMIPKIT-0043-FR-009 and
KMIPKIT-0043-FR-013; they do not expand the binding API.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `e9a121cc` | `$env:PYTHONPATH = 'bindings/python/src'; & '.\bindings\python\.venv\Scripts\python.exe' -m pytest -q --tb=short bindings/python/tests/test_client.py -k 'fixed_width_integer_inputs or client_execute_rejects_closed_borrowed or batch_add_item_rejects_closed or native_unknown_ttlv_item_type'` | Expected failure: 8 failed, 1 passed, 120 deselected. Failures demonstrated bool acceptance, batch/item consumption when borrowed handles were closed, and raw `ValueError` for unknown item types. |
| Green | `79229cb5` | Same focused command | PASS: 9 passed, 120 deselected after validating integer arguments, preflighting borrowed handles, and mapping unsupported item types. |
| Refactor | this evidence/refactor commit | `$env:PYTHONPATH = 'bindings/python/src'; & '.\bindings\python\.venv\Scripts\python.exe' -m pytest -q --tb=short bindings/python/tests` | PASS: 218 tests. The integer-bound calculation was extracted into a private helper without changing behavior. `git diff --check` passed. |

The initial full-suite check exposed ownership unit tests that replace CFFI
with a fake object without `typeof`; the normalizer treats that test seam as a
non-introspectable callable and preserves the existing ownership behavior.
Ruff is not installed in the project virtual environment, and this binding
project does not configure it.

## T028 CI corrections after run 38574059436

Run `38574059436` on head `f5c24ec8b94660fa0d110629dc3c36e955f1c35a` exposed
Python packaged-consumer failures on Ubuntu and macOS. The first diagnosis
incorrectly attributed the global package path to Python isolated mode. The
initial test/fix (`6581ddcc`/`4469d3d5`) encoded that mistaken diagnosis by
removing `-I` from the `sysconfig` query; it did not fix the underlying path
selection. Run `38576144875` reproduced the Ubuntu failure. Its log showed
`main()` had first called `Path.resolve()` on the venv's `bin/python` symlink,
turning it into `/usr/bin/python3.12`; `sysconfig` then correctly returned the
system interpreter's global package directory. The correction keeps the
lexical absolute path with `Path.absolute()`, which preserves the virtual
environment prefix, and restores `-I` for the configuration query and package
probes.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Initial Red | `6581ddcc` | `python -m unittest scripts.tests.test_native_package_smoke -v` before the first attempted fix | Expected failure for the simulated `-I` behavior; this test did not reproduce symlink resolution and was superseded. |
| Initial attempted Green | `4469d3d5` | `python -m unittest scripts.tests.test_native_package_smoke -v` | PASS locally, but CI run `38576144875` showed this did not fix the actual symlink-resolution failure. |
| Symlink Red | `74eb2677` | `python -m unittest scripts.tests.test_native_package_smoke -v` before preserving the path | Expected failure: `main()` passed a simulated system interpreter path instead of the consumer venv executable. |
| Symlink Green | `2308f3bc` | `python -m unittest scripts.tests.test_native_package_smoke -v` | PASS: `main()` preserves the venv executable path while keeping isolated-mode checks enabled. |

The same run's dependency-policy job refreshed RustSec and rejected pinned
`rustls 0.23.45` for [RUSTSEC-2026-0346](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0346.md)
and [RUSTSEC-2026-0349](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0349.md).
The workspace pin and lockfile now use `rustls 0.23.46`, the upstream patched
release ([release notes](https://github.com/rustls/rustls/releases/tag/v%2F0.23.46));
the dependency correction is commit `08be9e09`, and no advisory exception was
added. `pwsh -NoProfile -File
scripts/Test-DependencyPolicy.ps1` passes with the refreshed advisory database
and confirms both lockfiles remain unchanged by the policy check itself.

| Verification | Command | Result |
|---|---|---|
| Python smoke regression | `python -m unittest scripts.tests.test_native_package_smoke -v` | PASS: 1 test. |
| Script contract suite | `python -m unittest discover -s scripts/tests -p 'test_*.py' -q` | PASS: 271 tests, 26 skipped. |
| Rust transport | `cargo +1.94 test --locked -p kmipkit-transport --all-features` | PASS: 136 unit tests, 6 delivery-state tests, 1 dependency-policy test, 88 HTTPS tests, and 5 doctests. |
| Dependency policy | `pwsh -NoProfile -File scripts/Test-DependencyPolicy.ps1` | PASS: current RustSec database, no new exception, lockfile hashes unchanged. |

Run `38576144875` passed dependency policy but failed the Ubuntu and macOS
Python packaged-consumer smokes for the symlink-resolution defect described
above. The corrected head `eb05ed01c16106519f21717c26e51add87e787a9` passed
full CI run `38577034594`: 34 jobs succeeded and 2 schedule-only jobs were
skipped. Java/JNI and Python package smokes passed on Ubuntu, Windows, and
macOS; the dependency-policy and selected-scope coverage gates passed. This
closes the cross-platform CI acceptance gap for T028/T072. No genuine
foreign-architecture shared library was available, so the negative
architecture checks remain simulated through unsupported-target selection and
the injected loader fixture; no physical cross-architecture load is claimed.


## T077 — Local API-level taxonomy parity

Rust, C, Java, and Python now expose the ordered local API taxonomy
`high-level`, `typed-protocol`, and `generic-ttlv`. The C ABI returns borrowed
spans into static strings; Java and Python adapt those values to `List<String>`
and `tuple[str, ...]`. Each facade test reads `api-levels.shared.v1` and calls
its real local facade entry point. The query performs no network exchange and
is documented as distinct from the KMIP Discover Versions operation; the
fixture explicitly leaves operation-mapping completeness unasserted. The API
manifest records KMIPKIT-0043-FR-005/010 and the generator owns all checked-in
bindings and parity output.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `a05cc14ca74336053e40a4f76c8244ee521b9b0b` | Focused Rust, C, Java, and Python API-level fixture tests | Expected failures: missing Rust `api_levels`, unresolved C API-level symbols, missing Java `RuntimeMetadata.apiLevels()`, and missing Python `metadata.get_api_levels()`. |
| Green | `fd4f4c23` | `cargo test -p kmipkit api_levels_match_the_shared_fixture_without_network_calls` | PASS: 1 focused Rust test. |
| Green | `fd4f4c23` | `cargo test -p kmipkit-ffi --features coverage-c-consumer generated_public_client_abi_consumer_compiles_and_runs` | PASS: the C ABI consumer compared all three actual C spans with expected values read from the shared fixture; fixture assertions require zero network calls, distinguish Discover Versions, and leave operation completeness unasserted. |
| Green | `fd4f4c23` | `mvn --batch-mode --file bindings/java/pom.xml -Dtest=SharedCapabilityParityFixtureTest#apiLevelQueryMatchesSharedFixtureLocally test` | PASS: 1 JNI parity test; Maven's existing extension example also completed and reported no network calls. |
| Green | `fd4f4c23` | `PYTHONPATH=src .venv/Scripts/python.exe -m pytest tests/test_shared_parity_fixtures.py -k api_level_query_matches_shared_fixture_locally -q` | PASS: 1 CFFI parity test, 16 deselected; the test observed only the local API-level ABI symbol. |
| Refactor | this evidence/refactor commit | `cargo fmt --all --check`; `git diff --check`; `python -B tools/api_manifest/generate.py --check` | PASS: formatting and whitespace clean; all 12 generated outputs current. |
| Refactor | this evidence/refactor commit | `cargo clippy -p kmipkit -p kmipkit-ffi --all-targets --all-features -- -D warnings` | PASS: affected Rust packages are warning-free. |
| Refactor | this evidence/refactor commit | `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check` | PASS: C 243/243, Java 450/450, Python 426/426 exact declaration matches; zero missing paths, kind mismatches, or issues. |

The Windows Rust/JNI builds emitted existing AWS-LC compiler-probe and
missing-NASM warnings, and the C test link emitted LNK4217 warnings; all
focused consumers completed successfully. The taxonomy query does not perform
or imply a KMIP operation and makes no claim that every in-scope operation is
mapped or complete.

#### Cross-platform CI correction after run `38586769029`

The first full run after T077 identified three branch defects. The shared C
capability test still expected the pre-T077 list; the standalone CMake consumer
invoked `main()` without the three expected API-level arguments; and the new
Rust fixture test lived inline in `crates/kmipkit/src/lib.rs`, which the
coverage-source preflight correctly rejects. The resulting Rust matrix,
script-contract, coverage-source, and selected-coverage failures traced to
those defects. Python package setup on Ubuntu separately failed because the
runner could not connect to `static.crates.io`; this is an external download
failure and still requires a clean CI rerun.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red correction | `bf6ab7aa` | `python -m unittest discover -s scripts/tests -p test_workflow.py` | Expected failure: the new contract detected that CTest did not pass `high-level`, `typed-protocol`, and `generic-ttlv` to the C consumer. The failed T077 CI run also reproduced this at runtime on the C matrix. |
| Green correction | `546fa00c` | `cargo test -p kmipkit api_levels_match_the_shared_fixture_without_network_calls`; `cargo test -p kmipkit-ffi --lib` | PASS: the fixture test passes from the integration-test path; all 64 FFI library tests pass, including the updated capability inventory. |
| Green correction | `546fa00c` | `python -m unittest discover -s scripts/tests -p test_workflow.py`; `python -m unittest discover -s scripts/tests -p test_coverage_gate.py` | PASS: workflow contracts 45/45; coverage-gate contracts 55 passed, 3 skipped. Repository source preflight now accepts `crates/kmipkit/src/lib.rs`. |
| Green correction | `546fa00c` | `cmake -G Ninja -S bindings/c -B build/c-consumer`; `cmake --build build/c-consumer --config Release`; `ctest --test-dir build/c-consumer -C Release --output-on-failure` | Not run locally because CMake is unavailable in this Windows environment. The corrected CTest invocation will be verified by the new full PR matrix. |
| Refactor | this evidence/refactor commit | `python -B tools/api_manifest/generate.py --check`; `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check`; `cargo fmt --all --check`; `git diff --check` | PASS: all 12 generated outputs current; C 243/243, Java 450/450, Python 426/426 declarations exact; formatting and whitespace clean. |

Run `38586769029` also showed the first security report's coverage artifact
was not resolved by the Codex Security workbench at finalization. Its findings
list was empty, but persisted coverage remained partial, so that scan is not
counted as a completed security gate. A corrected diff scan,
`94b34370-bbb4-42db-a5dc-76f72a5b3f44`, completed against the corrected T077
head with complete coverage of all 11 security surfaces and no reportable
findings. The full CI matrix remains open for this draft PR.

#### Rust parity mapping correction after run `38588319434`

The corrected CI run found one more stale fixture reference on Ubuntu and
macOS. After moving the Rust parity test out of production sources, the shared
fixture still pointed to `crates/kmipkit/src/lib.rs`; the parity contract
therefore searched production code for a test declaration. The test now
requires the fixture to map this case to the Rust integration-test file, and
the fixture points to `crates/kmipkit/tests/api_levels.rs`.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `abe393f5` | `python -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_fixture_maps_executable_consumer_contracts` | Expected failure: fixture mapped the Rust test to `src/lib.rs` instead of the integration test path. |
| Green | `1b1f7da1` | `python -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_fixture_maps_executable_consumer_contracts`; `python -m unittest discover -s scripts/tests -p test_workflow.py` | PASS: parity mapping contract 1/1; workflow contracts 45/45. |

The corrected fixture and guard require a fresh full CI run before the PR can
leave draft status.

## T032 — Create, Create Key Pair, and Create Split Key parity

T032 maps the accepted owner contract in `specs/014-server-generated-creation/spec.md`
to C, Java, and Python while reusing the canonical Rust request and response
models. The API manifest and generated outputs declare all three operations,
response identifiers, and the additive result metadata surface. No formal OASIS
conformance claim is made.

The shared success fixtures cover Create's Object Type and signed Integer
Unique Identifier, independent ordered Common/Private/Public groups and the
distinct private/public identifiers for Create Key Pair, and Create Split
Key's signed parameters, open method value, required Attributes, optional
input identifier/masks, ordered repeated identifiers, Pending correlation, and
no implicit follow-up. The C mTLS consumer executes all three in the same
ordered batch. Java and Python consume the same shared fixtures.

OASIS KMIP v2.1 §8.6, Table 399, makes Result Message optional only when Result
Status is neither Pending nor Success; §9.17 permits a descriptive error
message. Accordingly, successful and Pending creation fixtures omit the
field, and `creation.result-message-on-failure.v1` retains the exact UTF-8
message on an Operation Failed result. C v1 metadata layout and signatures
remain unchanged. The additive metadata v2 API explicitly exposes presence
and exact bytes; its C pointer is borrowed while the response-item handle is
alive and is bounded by the decoded message limit. Java and Python copy the
value before native-owner release. The message is excluded from formatting,
logs, default errors, and diagnostics.

| Phase | Commit(s) | Red/Green evidence |
|---|---|---|
| Red | `0e1b76c2`, `18929733`, `a7cd6bc3`, `4b8bb59e`, `141139b5`, `e9d8d9b1` | Added failure-first Rust, C, Java, Python, and manifest tests: `cargo test -p kmipkit-client --test creation_parity_tests`; `python -B -m unittest tools.api_manifest.tests.test_creation_parity_manifest -v`; `uv run pytest tests/test_client.py tests/test_shared_parity_fixtures.py -q` from `bindings/python`; and `gradlew.bat test --tests org.kmipkit.SharedCapabilityParityFixtureTest` from `bindings/java`. The new operation/mapping/metadata assertions failed against the missing surface. After adding the capability assertion, `cargo test -p kmipkit --test runtime_metadata capability_query_matches_manifest_ready_capabilities` failed with `Ok(false)` versus `Ok(true)` for `server_generated_creation`. |
| Green | `099d9902` | Implemented all three cross-language builders and response accessors, C metadata v2, exact operation values and runtime capability mapping; regenerated API output from the manifest. `cargo test -p kmipkit-client -p kmipkit-protocol -p kmipkit-ffi -p kmipkit --all-features`, Java full Gradle tests, and Python full pytest suite passed. |
| Refactor | this evidence commit | Recorded normative traceability and binding lifetime/redaction behavior; updated bilingual user guides and the dependency inventory. Final formatting, generated-file check, binding export audit, and diff validation are recorded below. |

| Verification | Command | Result |
|---|---|---|
| Focused Rust parity and failure metadata | `cargo test -p kmipkit-client --test creation_parity_tests`; `cargo test -p kmipkit-client --lib create_failure_preserves_result_and_does_not_retry` | PASS: 5 parity tests and 1 failure-result/no-retry test. |
| C ABI execution | `cargo test -p kmipkit-ffi --test c_api_coverage --features coverage-c-consumer generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls` | PASS: C consumer constructed and executed the typed mixed batch over TLS. |
| Java parity and metadata | `bindings/java/gradlew.bat test` | PASS: full Java test task. |
| Python parity and client | `uv run pytest -q` from `bindings/python` | PASS: 232 tests. |
| Rust package matrix | `cargo test -p kmipkit-client -p kmipkit-protocol -p kmipkit-ffi -p kmipkit --all-features` | PASS, including doctests and C ABI integration targets. |
| Rust lint and format | `cargo clippy -p kmipkit-client -p kmipkit-protocol -p kmipkit-ffi -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check` | PASS. AWS-LC emitted its existing Windows compiler/NASM probe warnings. |
| Manifest generation and export audit | `python -B tools/api_manifest/generate.py --check`; `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check` | PASS: 12 generated outputs verified; export audit exited 0 with zero issues. |

T032 closes the operation mapping, parity, documentation, and evidence scope.
T034 remains open because its Red/Green/Refactor evidence spans T031 through
T033, and T033 is not part of this change. The branch still requires the
separate coverage-gate repair and final remote CI on the reviewed head.

## Coverage gate follow-up

Run `38830440607` at PR head `5bfcff21` failed the changed-production-code
coverage gate: 1,392 of 1,481 executable lines were covered (93.99 percent;
1,407 hits are required for 95 percent). The preserved CI artifacts identify
76 executable lines without a hit and 13 conservative summary-only misses in
the JNI report. This is the Red result for the coverage work; it does not
indicate a failing behavior assertion.

The report points to reachable boundary and malformed-metadata paths in
`TransportConfiguration`, `TtlvPath`, the Python handle adapter, and the
Python Discover Versions response adapter. These paths already implement
their documented validation behavior, so the follow-up adds characterization
tests rather than production changes or fault-injection seams. The final
aggregate must be rerun against the reviewed head before this gate can be
reported green.

The characterization tests now cover those locally reachable branches. The
Java full Gradle test task passes, and the Python full suite passes 239 tests
with 95.09 percent package coverage. The complete Rust workspace coverage
suite also passes on Windows; its raw and normalized reports are retained
outside the repository under the task's temporary coverage directory. Source
eligibility preflight passes. These local results demonstrate the added tests
execute successfully, but they do not replace the changed-code aggregate:
fresh Linux/macOS Rust reports, C-consumer FFI, cross-platform JaCoCo, Python,
and JNI reports must be combined by the CI aggregate against the pushed head.

| Coverage phase | Commit | Command / result |
|---|---|---|
| Red | `152b7dda` | Reproduced remote run `38830440607`: 1,392/1,481 changed executable lines covered (93.99%; 1,407 required). The preserved report contained 76 uncovered executable lines and 13 conservative JNI summary-only misses. |
| Green tests | `5c7b27d8` | `bindings/java/gradlew.bat test`: PASS. Python full suite: PASS, 239 tests, 95.09% package coverage. `cargo llvm-cov --workspace --all-features --locked --json --output-path <isolated-temp>/coverage-raw-windows.json`: PASS; normalized report and `scripts/coverage_gate.py preflight --workspace .`: PASS. |
| Aggregate gate | pending | A fresh full CI aggregate on the pushed head is required to establish the 95 percent changed-code result. |

#### T032 manifest correction discovered by run `38840021567`

The Ubuntu Script contracts job on head `2eca0100` exposed two manifest
contracts that the initial local checks did not include. The shared-parity
fixture allowlist omitted the already-present
`creation.result-message-on-failure.v1` case. The T076 Python-export inventory
also found new T032 result fields and `UniqueIdentifier` factory methods
unclassified, along with Python dataclass-generated members requiring exact
exclusion records. This is a contract/traceability regression, not an
environmental failure.

| Phase | Commit | Command / result |
|---|---|---|
| Red | this test/evidence commit | `python -B -m unittest tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_t076_client_mapping -v`: reproduced 2 failures among 43 tests. The parity allowlist was updated for the existing failure fixture, and a focused assertion now requires the T032 Python semantic exports to have four-language mappings and exact dataclass exclusions; it fails until the manifest is corrected. |
| Green | pending | Correct the manifest and regenerate its outputs; rerun the focused contract command and full script-contract matrix. |

The same CI run also failed both macOS Core jobs while compiling the C ABI
consumer: the test calls `kmipkit_ttlv_structure_release`, but that symbol was
still blocked in the manifest and omitted from the generated public header.
The handle already names this release function, and the Rust, Java, and Python
wrappers already provide their corresponding native-handle lifecycle. The
follow-up therefore maps the existing lifecycle through the manifest's
`native-handle-release` semantics; it does not add a new TTLV operation or
alter a C layout.

| Phase | Commit | Command / result |
|---|---|---|
| Red | this test/evidence commit | Three focused manifest tests: 6 expected failures. They identify the missing Python and Java T032 exports, the absent three-language TTLV release classification, and the missing generated C prototype. CI run `38840021567` independently reproduced the C compiler error on both macOS toolchains. |
| Green | pending | Add exact four-language manifest mappings and regenerate with the pinned tool; rerun the focused tests and macOS C consumer compilation. |
| Refactor | pending | Update final traceability and record the verified cross-language/script-contract checks. |

The full binding export audit then found 13 additional Java declarations not
classified by the focused tests: seven raw JNI dispatch methods for the new
creation factories and results, four explicitly redacted `toString` methods,
and the two compiler-provided enum helpers for `UniqueIdentifier.Kind`. The
typed capabilities are already mapped through the facade; each bridge or
diagnostic/compiler member needs an exact exclusion rather than a package-wide
exception.

| Phase | Commit | Command / result |
|---|---|---|
| Red | this test/evidence commit | `python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_creation_java_bridge_and_redacted_formatters_are_exactly_excluded -v`: expected failure listing all 13 absent exact exclusions. `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --check`: C and Python inventories match; Java reports 13 missing classifications. |
| Green | pending | Add exact bridge/formatter/enum exclusion records and rerun the Java export audit and manifest contract suite. |

An additional focused contract test now fixes the intended mappings for
`BatchItemResult.operation`, `result_message`, `unique_identifier`, and
`public_key_unique_identifier` across all four languages. Its Red command,
`python -B -m unittest tools.api_manifest.tests.test_t076_client_mapping.T076ClientMappingTests.test_creation_result_fields_have_exact_four_language_mappings -v`,
fails four times because those Python field declarations are not yet in the
manifest; this is the Red evidence for their exact mapping rather than an
implementation failure.

## T031 — Discover Versions response mapping

The four-language mapping now exposes the successful Discover Versions
`ProtocolVersion` list from the existing batch result. The API preserves order
and repeated entries, distinguishes a successful empty list from absent data
on Pending or failed items, and does not issue an implicit follow-up request.
The operation row and shared fixture cite OASIS KMIP 2.1 §6.1.16, Tables
211–213, and the approved owner requirements
`KMIPKIT-REQ-SPEC-6.1.16-001-001`, `-001-002`, and `-004`.

| Phase | Commit | Verification | Result |
|---|---|---|---|
| Red | `a606da8c`, `95288e15` | Rust client response tests and cross-language API mapping tests | Expected failures for the missing typed response mapping and parity fixture. |
| Red | `e50303ad`, `1d686a22` | Python count-boundary test; `mvn --batch-mode --file bindings/java/pom.xml -Dtest=ClientFacadeTlsIntegrationTest#discoverVersionsResponseCanExceedTheDefaultElementLimitWhenConfigured test` | Expected failure: adapters rejected a valid list of 100,001 versions even when the caller raised the TTLV element limit. |
| Green | `acee5404`, `20b20f91` | `cargo test -p kmipkit-client --lib discover_versions`; compiled C ABI consumer; Java TLS integration and shared parity tests; Python client, parity, binding-export, and bilingual-doc tests | PASS: typed response and all adapter mappings preserve empty/absent distinctions; Java accepted 100,001 versions over a TLS 1.3 mTLS peer with a raised codec limit; Python accepted the same count at the CFFI boundary. |
| Refactor | this evidence commit | `cargo fmt --all --check`; `git diff --check`; `python tools/api_manifest/generate.py --check`; `python tools/api_manifest/audit_binding_exports.py --check` | PASS: formatting clean, generated output current, and binding exports reconciled without gaps. |

The Java JNI adapter bounds its flattened `long[]` by the JVM `jsize`
representation, and Python checks the CFFI allocation representation; neither
adds a smaller undocumented 100,000-version ceiling than the configured TTLV
decoder. The full `tools/api_manifest/tests` run still reports eight failures
that reproduce at the pre-T031 PR head `a6696836`; these concern existing
inventory snapshots, C declaration classification, and a T076 Rust mapping.
T031-specific parity, documentation, and exclusion regressions pass at the
current head. The PR's required CI checks must still run against the pushed
final head.
