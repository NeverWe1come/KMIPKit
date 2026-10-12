# Implementation Evidence

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

T068 records the accepted release-scope decision only; it does not implement Set Endpoint Role. T011 remains open because a complete, reproducible export-audit report and reconciliation against every manifest row have not yet been committed; do not treat uncommitted audit observations as completion evidence.

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

## T013 — Approved generic TTLV declaration ownership (partial)

The canonical declaration slice whose Rust paths begin with `kmipkit_ttlv::` contains exactly 101 declarations: 76 generic-model declarations owned by KMIPKIT-0004 and 25 `CodecLimits` declarations owned by KMIPKIT-0005. The manifest maps these to the two approved shared capabilities with their owner requirements and evidence. This closes only this audited slice; T013 remains open until the remaining published Rust crate declarations are audited and mapped or explicitly excluded under an accepted authority.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `0344edaf` | `python -B -m unittest tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_ttlv_declarations_use_their_approved_shared_spec_owners -v` | Expected failure before owner reconciliation: the manifest had no `generic_ttlv_model` capability record, so the test failed with `KeyError`. |
| Green | `c0cc2a31` | `python -B -m unittest discover -s tools/api_manifest/tests -v`; `python tools/api_manifest/generate.py --check`; `git diff --check` | PASS: 94 tests passed, 3 symlink tests skipped because Windows denied symlink creation (WinError 1314); all 11 generated outputs current; no whitespace errors. |
| Refactor | `5d36f87c` | `uv run --with jsonschema -- python -B -m unittest discover -s tools/api_manifest/tests -v`; `python tools/api_manifest/generate.py --check`; `git diff --check` | PASS: 94 tests passed, 3 symlink tests skipped because Windows denied symlink creation (WinError 1314); all 11 generated outputs current; no whitespace errors. Strengthened the inventory assertion to require exactly 101 declarations split 76/25; no API behavior or manifest data changed. |
