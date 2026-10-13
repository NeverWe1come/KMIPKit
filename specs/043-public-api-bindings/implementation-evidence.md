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

The source fixture corpus at `tests/fixtures/api/shared-capability-parity.json` defines six language-neutral cases for API-level discovery, per-configuration registry isolation, resource-limit error/redaction behavior, ordered mixed batch outcomes with a Pending correlation value, generic TTLV open-value/order preservation, and extension-registry recognition/preservation. Every case records stable 0043 and owning-feature requirement IDs, a literal input setup, expected observable output, error category when applicable, cleanup expectations, and Rust/C consuming test names. Extension cases resolve to the existing shared extension fixture source.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `835b4e8c` | `python -B -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_parity_fixtures_cover_rust_and_c_contracts -v` | Expected failure: the shared API parity corpus did not exist. |
| Green | `6774abcc` | Same focused command | PASS: fixture corpus and the required capability, requirement, output, error, cleanup, consumer, and extension-source references were present and valid. |
| Refactor | `3c548559` | `python -B -m unittest tools.api_manifest.tests.test_parity -v`; `git diff --check` | PASS: all 8 parity tests passed; no whitespace errors. Reused one helper for capability-case lookup without changing fixture behavior. |

The Rust and C consumer test names are the planned consumers for subsequent binding tasks; this task establishes the shared input and expected-output source. Runtime C/JNI/Python parity and manifest-backed evidence remain gated by their respective implementation tasks.

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

`kmipkit_last_error_get` and `kmipkit_last_error_clear` are C ABI diagnostics required by FR-007. The language facades expose failures as caller-owned Rust errors and Java/Python exceptions; they do not expose the C thread-local slot or its explicit clear operation. These exact symbols are excluded only from the cross-language facade parity inventory and remain in the generated C header and C API tests. This is a reviewed surface distinction, not a claim that FR-007's C requirement is out of scope.

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
