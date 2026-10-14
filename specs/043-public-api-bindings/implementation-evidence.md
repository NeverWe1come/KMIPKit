# Implementation Evidence

## T015 — Manifest-backed handle lifecycle and C status contract

This task closes the existing approved C ABI lifecycle implementation against the v2 manifest; it required no production-code change. Opaque handles are represented by one `Arc<Handle>` envelope with a private kind discriminator. Accessors reject null and wrong-kind live handles before reading payloads; views retain their owning handle, so releasing the original owner does not invalidate a child view. Every manifest opaque-handle type declares library ownership and its matching release symbol. Input byte spans carry explicit `uint64_t` byte lengths, apply bounds before dereferencing, and validate UTF-8 before model construction. C status values remain the manifest-declared stable codes, and safe category/cause/delivery snapshots are stored per thread. Java and Python resource wrappers reject operations on closed handles and serialize closure against native use.

The C release ABI accepts an opaque pointer by value, so it cannot clear the caller's variable. A caller must set its local variable to `NULL` after release; safe C accessors reject that null sentinel. A caller must not reuse a C pointer after release; use-after-release, dangling, and foreign pointers remain caller-precondition violations and are not dereferenced or claimed to be safely detectable. This preserves the accepted ABI and avoids a process-global handle registry or tombstone allocation scheme. Java/Python wrappers retain local closed state and reject operations after close.

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
| `client_certificates` | `CertificateInput` PEM/DER and file constructors | `clientIdentityPem`, `clientIdentityDer`, `clientIdentityFiles` | `TransportConfig.client_certificates` with `TransportInput.memory/file` and `CredentialEncoding` | PEM/DER memory and selected files |
| `client_private_key` | `PrivateKeyInput` PEM/DER and file constructors | `clientIdentityPem`, `clientIdentityDer`, `clientIdentityFiles` | `TransportConfig.client_private_key` with `TransportInput.memory/file` and `CredentialEncoding` | PEM/DER memory and selected files |
| `trust_certificates` | `TrustSource::certificate_authorities` over `CertificateInput` PEM/DER and file constructors | `trustCertificatesPem`, `trustCertificatesDer`, `trustCertificateFiles` | `TrustSource.certificate_authorities` over `TransportInput.memory/file` | PEM/DER memory and selected files |
| `revocation_lists` | `RevocationListInput::from_pem/from_der` | `revocationListsPem`, `revocationListsDer` | `TransportConfig.revocation_lists` with in-memory `TransportInput` | PEM/DER memory only |

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

Independent semantic review and the remaining cross-platform/required CI gates are still outstanding. T011, T076, T021, and T024 remain open; no generic error-TTLV API was promoted.

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
