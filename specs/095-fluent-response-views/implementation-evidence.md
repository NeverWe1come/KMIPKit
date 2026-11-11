# KMIPKIT-0095 binding implementation evidence

## Red

- `cargo test -p kmipkit-ffi client_response_extension_abi_returns_ordered_generic_ttlv_copy --lib`: exit 101, E0425 for the three missing C response renderers.
- `mvn -q -f bindings/java/pom.xml test-compile`: exit 1, missing `ClientBatchItemResult.ttlvHex/json/xml(CodecLimits)`.
- Python contracts request the same three explicit methods and verify native ownership transfer with no transport available in the fake bridge.

The approved scope is T009 only. Exact remote CI remains T008 and is owned by the parent task.

## Green

- Red commit: `525ffa212` (DCO signed off).
- `CARGO_BUILD_JOBS=2 cargo test -p kmipkit-ffi --all-features --lib`: 473 passed, including compiled C response-view consumers, independent owners and category translation.
- `/tmp/kmipkit-0095-py312/bin/python -m pytest bindings/python/tests/test_response_views.py bindings/python/tests/test_response_raw_navigation.py -q`: 16 passed.
- `/tmp/kmipkit-0095-py312/bin/python -m pytest bindings/python/tests -q`: 1299 passed, 119 subtests passed.
- `sh bindings/java/native/build.sh`: passed native helper tests and JNI compilation.
- `mvn -q -f bindings/java/pom.xml -Dexec.skip=true '-Dtest=ResponseViewsTest,ClientFacadeTlsIntegrationTest#pingQueryAndLogoutRoundTripThroughNativeJniOverDeterministicTls13Peer' test`: 2 passed. The skip reuses the immediately built JNI library.
- `python3 tools/api_manifest/reconcile_0095_response_views.py` and `python3 tools/api_manifest/generate.py --write`: 12 generated outputs, without manual generated edits.

The tests compare complete response bytes and multi-item ordering, retained
vendor fields, size/depth/element limits, owned results after source/client
closure, scoped callback erasure (including exceptions), dead handles,
payload-free failures, and the no-send inspection boundary.

A workspace build and independent audits initially exhausted the 7.6 GiB host
and 2 GiB swap. They were stopped and reissued sequentially with two Cargo jobs
and a disk-backed temporary directory where supported. The no-feature FFI test
attempt also exposed the existing requirement to enable the compiled-consumer
feature; all-feature testing is the applicable command used above.

## Refactor and final verification

- Green commit: `a93e80fb0` (DCO signed off).
- Consolidated Java limit/ownership dispatch, JNI owned-result adoption, and the Rust renderer function type. No public signature or encoding behavior changed.
- Added Python adoption-failure cleanup verification; final response-view tests: 17 passed against the freshly built native DSO through `KMIPKIT_NATIVE_LIBRARY`.
- `CARGO_BUILD_JOBS=2 cargo test --workspace --all-features --locked`: 4231 passed, 0 failed, 3 ignored (existing local-Cosmian cases).
- `CARGO_BUILD_JOBS=2 cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all --check`: passed.
- `mvn -q -f bindings/java/pom.xml -Dexec.skip=true test`: 706 tests, 0 failures/errors, 1 existing skipped case.
- `python3 tools/api_manifest/generate.py --check`: all 12 outputs verified.
- Binding audit `tools.api_manifest.audit_binding_exports.run(root, c_binary=.../target/debug/libkmipkit_ffi.so)`: C 1998, Java 2390, Python 2596 exact manifest matches, all inventories complete and zero missing declarations or kind mismatches. This uses the supported audit runner directly to reuse the native binary across all three language inventories; the CLI only allows binary reuse with `--c-only`.
- `TMPDIR=/home/ramp1953/.cache/kmipkit-0095-tmp CARGO_BUILD_JOBS=2 python3 tools/api_manifest/audit_rust_exports.py --check --report /tmp/0095-rust-audit-final.json`: 9373 declared Rust exports matched, 20843 approved exclusions, zero issues/mismatches.
- `git diff --check`: passed.

The first full-workspace attempt ran concurrently with a default-feature JNI
build, which replaced the shared C DSO and caused an undefined test-only symbol
in a later dynamic C consumer. After native builds completed, rebuilding the
all-feature DSO and rerunning the full workspace passed. Subsequent checks use
two Cargo jobs and disk-backed temporary audit targets to avoid host memory
pressure. No production fix or test relaxation was needed for this scheduling
issue.

### Final coverage and contract gates

- `CARGO_BUILD_JOBS=2 TMPDIR=/home/ramp1953/.cache/kmipkit-0095-tmp cargo llvm-cov --workspace --all-features --json --summary-only --output-path /tmp/0095-rust-coverage.json`: passed; Rust 65641/70908 lines (92.57%); C ABI adapter 6296/7132 (88.28%).
- `CARGO_BUILD_JOBS=2 bash scripts/collect_jni_coverage.sh /home/ramp1953/.cache/kmipkit-0095-tmp/jni-coverage`: passed the pinned LLVM 20.1.8 collector and clean Maven verify. JNI 20468/22250 lines (91.99%); Java 7239/7623 (94.96%); Java coverage checks passed.
- Full Python suite against the fresh DSO with pytest-cov and disk-backed temporary files: 1300 passed, 119 subtests passed; 9789/10172 lines (96.23%).
- Workspace-owned LLVM source regions, JaCoCo, and Python coverage mapped to the T009 diff from `a9a87d9`: changed Rust 37/38 (97.37%), JNI 25/25, Java 7/7, Python 18/18; combined 87/88 (98.86%). These local changed-line figures use the repository parser and executable-line filter; remote exact-head CI remains T008.
- Required CI command `python -B -m unittest tools.api_manifest.tests.test_binding_exports tools.api_manifest.tests.test_0095_response_view_bindings -v`: 60 passed. New JNI exclusions have an exact requirement/owner contract and the new response-view manifest test is registered in Script contracts.
- Focused API snapshots, ownership mappings, current audit documentation and workflow contracts: 119 passed, 264 subtests passed.
- Final generated-output check: 12 outputs verified. The current dated binding audit report is `binding-export-report-2026-11-11-response-views.json`; older reports are preserved as historical snapshots.

The optional blanket API-tool run was interrupted after 215 passes and 3121
subtests to inspect failures. T009's exact JNI exclusion whitelist and current
audit-report references were corrected and all affected/required contracts
passed afterward. A separate pre-existing failure remains in
`CandidateExclusionManifestTests.test_candidate_exclusions_require_an_exact_verified_counterpart`:
`ManifestError not raised`. It was reproduced by invoking the exact test method
with both the base `a9a87d9` manifest and the current manifest; the validator and
test source files are identical at both revisions. The selected legacy route
exclusion lacks `artifactKind`, so that test does not enter the validator's
counterpart-checking path. This module is absent from the current explicit CI
API-module list and is not covered by blanket discovery in that workflow.
No T009 scope or required check was relaxed to hide it.

One initial Python coverage report encountered `/tmp` tmpfs exhaustion after
1299 passing tests. The final full run above uses disk-backed temporary files
and report output, and passed completely. Local analysis projects workspace
coverage regions because the installed newer Rust toolchain also emits
standard-library function filenames outside the repository; raw reports and
all final metric inputs remain available in the task cache.

## FR-005 review follow-up: final hexadecimal output limit

Review confirmed that the inherited inspection writer bounded binary TTLV but
reserved/emitted twice that size as hexadecimal. The limit must apply to the
requested rendered output, before its allocation or writes.

### Follow-up Red

- `CARGO_BUILD_JOBS=2 cargo test -p kmipkit-client --all-features --lib ttlv_hex_limits_bound_the_rendered_output_not_only_binary_ttlv`: exit 101; the new assertion fails because binary TTLV fits the cap while the hexadecimal output exceeds it.
- The regression also checks one byte below the final hex length, exact-boundary acceptance, and that inspection leaves the exchange count at one.
- Python's native response fixture fails with `DID NOT RAISE ResourceLimitError` for a cap equal to the binary message size.
- Equivalent C compiled-consumer and Java mTLS regressions are recorded in this Red change before production changes. Java/Python also check exact final-length acceptance.

No response contents are included in failure diagnostics; callers receive only
the existing payload-free resource-limit category.
