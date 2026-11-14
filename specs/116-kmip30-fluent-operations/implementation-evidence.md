# KMIPKIT-0116 Implementation Evidence

## Red

The first public-surface tests were run before production changes. Each failed
at the expected missing API assertion:

- Cross-language manifest contract:
  `python3.12 -B -m unittest tools.api_manifest.tests.test_0116_kmip30_fluent_manifest -v`
  initially failed because the seven 3.0 operation rows were absent from the
  public API matrix. After reviewing the manifest's strict `client_1_0`
  operation inventory boundary, the test was refined to require a distinct
  verified KMIP 3.0 fluent capability; rerunning it fails because that
  capability and per-operation mappings are not yet present.
- Scoped capability contract:
  `python3 -B -m unittest tools.api_manifest.tests.test_0116_kmip30_fluent_manifest -v`
  failed because `kmip30_fluent_operations` is not yet declared. It also
  asserts that the default `client_1_0` inventory remains exactly 56 entries.
- Versioned matrix contract:
  `python3 -B -m unittest tools.normative_catalog.tests.test_fluent_operation_inventory.FluentOperationInventoryTests.test_kmip30_fluent_routes_are_visible_in_versioned_api_matrix -v`
  failed for all seven KMIP 3.0 rows because their `publicApiScope` is still
  absent from the generated inventory.
- C header contract:
  `python3.12 -B -m unittest tools.api_manifest.tests.test_0116_kmip30_c_public_api -v`
  failed because all seven semantic direct-operation symbols are absent.
- Java consumer contract:
  `JAVA_TOOL_OPTIONS='-Xmx6g -XX:MaxMetaspaceSize=1g' bash gradlew --no-daemon -Dorg.gradle.jvmargs='-Xmx6g -XX:MaxMetaspaceSize=1g' -Dorg.gradle.workers.max=1 test --tests org.kmipkit.fluent.Kmip30FluentApiShapeTest`
  compiled the consumer test and failed its expected public-method assertions.
- Python consumer contract:
  `/home/ramp1953/.cache/kmipkit-t082-python-venv-20261016/bin/pytest -q bindings/python/tests/test_fluent_kmip30.py`
  failed because `Request.create_credential` is not present.
- Rust consumer contract:
  `cargo test -p kmipkit-client kmip30_fluent -- --nocapture` failed at
  compile time because `RequestBuilder` did not yet expose the seven KMIP 3.0
  fluent operation methods. The only diagnostics were those missing methods;
  the focused test source compiled far enough to identify the absent API.
- Credential alternative contract:
  `cargo test -p kmipkit-protocol --lib create_credential_uses_table_277_order_and_redacts_credential_material`
  failed to compile with E0061 because the test used the required two-argument
  `CreateCredentialRequest::new(attributes, object)` while the implementation
  still required a separately supplied Credential Type. This Red test guards
  deriving the type from the closed credential alternative.

No production code was changed before these tests failed.

## Green

- Rust fluent builders are now available for all seven KMIP 3.0 operations.
  `cargo test -p kmipkit-client kmip30_fluent -- --nocapture` passed all
  three focused tests: operation-specific typed request construction,
  no-send rejection when KMIP 3.0 is not explicitly selected, and explicit
  KMIP 3.0 wire version/operation encoding. `cargo fmt --all --check` passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
  passed after documenting the validation result contract.
- Java/JNI/CFFI expose the seven KMIP 3.0 builders with exact version gating,
  one-exchange execution, typed results, redaction, and owned native handles.
  `bash ./gradlew --no-daemon --init-script
  /home/ramp1953/.codex/tmp/kmipkit-0116/java-temp.init.gradle
  -Dorg.gradle.jvmargs='-Xmx2g -XX:MaxMetaspaceSize=512m'
  -Dorg.gradle.workers.max=1 test` passed: 718 tests, zero failures, one
  skipped test. The init script placed test temporary files on the worktree
  volume because `/tmp` was full.
- `TMPDIR=/home/ramp1953/.codex/tmp/kmipkit-0116
  ./.venv/bin/python -m pytest -q` in `bindings/python` passed all 1,370 tests.
  The KMIP 3.0 execution and shared-parity focused suites passed 46 tests.
- The two-argument credential constructor is now used throughout the Rust,
  C, Java, and Python surfaces. The protocol request derives Credential Type
  from its closed typed alternative, avoiding an inconsistent public numeric
  selector.
- `TMPDIR=/home/ramp1953/.codex/tmp/kmipkit-0116 cargo test -p
  kmipkit-client --all-features` passed: 924 unit tests, all integration and
  doc tests, with the existing three live Cosmian tests ignored.
- `TMPDIR=/home/ramp1953/.codex/tmp/kmipkit-0116 cargo test -p kmipkit-ffi
  --all-features` passed: 480 FFI unit tests, 58 compiled C consumer tests,
  and the 56-operation parity test. The C consumer includes all seven
  KMIP 3.0 builders and the inferred credential-type call signature.

## Manifest and inventory repair

The pre-0116 Rust export baseline had 304 `missingFromManifest` paths. The
current feature head had 381 before reconciliation: the delta was exactly 77
(63 compiler-generated standard-trait paths, seven builder `build` methods,
and seven `fluent::RequestBuilder` aliases). The manifest now inventories all
Rust compiler-observed paths: the 179 non-trait misses are explicit records,
the 202 standard-library trait paths have exact approved exclusions, and all
14 new 0116 functional paths carry four-language mappings. The baseline
non-0116 entries remain blocked under KMIPKIT-0093 until their owning API work
is addressed.

- `python3 -B tools/api_manifest/generate.py --write` and `--check` passed.
- `audit_rust_exports.py --check` passed with 30,695 observed paths, 9,632
  matched declarations, 21,063 exclusions, and zero missing, stale,
  kind-mismatched, or incomplete paths.
- `audit_binding_exports.py --check` passed: C 2,063/2,063, Java 2,468/2,468,
  and Python 2,721/2,721 exports matched with no missing, stale, or kind
  mismatches.
- The Rust audit report used by the manifest regression test was regenerated
  with `audit_rust_exports.py`; the focused Rust inventory, T102 profile, and
  KMIP 3.0 manifest tests passed 6/6.
- The T102 snapshot is now 16,884 public declarations and 22,888 approved
  exclusions. Its pinned Rust audit values remain tied to the historical T102
  report and are not replaced by the current whole-workspace audit totals.
- The branch was rebased onto the approved KMIPKIT-0116 operations parent at
  `51a9a5e1c`; the C consumer now uses the object-derived Create Credential
  signature. `python3 -B tools/normative_catalog/generate_fluent_operation_inventory.py`
  and `--check` passed, and
  `python3 -B -m unittest
  tools.normative_catalog.tests.test_fluent_operation_inventory -v` passed all
  18 tests. The generated versioned inventory records seven routes while the
  56-entry KMIP 2.1 client inventory remains unchanged.
- `uv build --wheel bindings/python --out-dir
  /home/ramp1953/.codex/tmp/kmipkit-0116/wheels` passed. The resulting wheel
  contains both `kmipkit/fluent.pyi` and `kmipkit/py.typed`.
- After the rebase, `python3 -B tools/api_manifest/generate.py --check`,
  `python3 -B tools/normative_catalog/generate_fluent_operation_inventory.py
  --check`, and `git diff --check` all passed.
- The full manifest suite previously spent over 11 minutes in the Java
  overload schema test because it revalidated all 22,888 unrelated approved
  exclusions for each one-row declaration assertion. The test now validates
  the same valid signature, incompatible declaration kind, and duplicate
  signature cases directly against `$defs.publicDeclaration`; its focused
  run passed in 0.316 seconds. The complete suite is being rerun after this
  test-only performance refactor.
- The duplicate-record schema test also used the full 16,884-declaration and
  22,888-exclusion manifest. It now uses the small valid v2 fixture with one
  duplicated declaration and exclusion, and asserts `uniqueItems` rejects
  both arrays. The focused test passed in 0.853 seconds.
- Final manifest verification passed after both focused test refactors:
  `python3 -B -m unittest tools.api_manifest.tests.test_manifest_v2 -v`
  completed with 156 tests in 337.105 seconds and exit code 0.

## Supplemental protocol coverage and local native sanitizers

The KMIP 3.0 typed protocol models gained tests for credential-alternative
inference, result parsing and redaction, operation-specific accessors, and
malformed KEM response shapes. The private parser/error tests are in
`crates/kmipkit-protocol/tests/support/kmip30_operations_unit.rs` and included
through a test-only path so the production source coverage preflight can
distinguish test code from executable code.

- `cargo test -p kmipkit-protocol --all-features` passed the complete protocol
  package suite, including 1,460 unit tests and all integration and doc tests.
- `CARGO_TARGET_DIR=/home/ramp1953/.codex/tmp/kmipkit-0116/coverage-target
  TMPDIR=/home/ramp1953/.codex/tmp/kmipkit-0116 cargo llvm-cov --workspace
  --all-features --locked --summary-only` passed with 91.12% line, 92.54%
  branch, and 93.42% function coverage. `kmip30_operations.rs` is at 91.59%
  line coverage and `kem.rs` at 91.26%.
- `CARGO_TARGET_DIR=/home/ramp1953/.codex/tmp/kmipkit-0116/coverage-target
  TMPDIR=/home/ramp1953/.codex/tmp/kmipkit-0116 cargo llvm-cov
  -p kmipkit-protocol --all-features --locked --summary-only` independently
  confirmed 90.08% protocol-package line coverage.
- `python3 scripts/coverage_gate.py preflight --workspace . --github-output
  /home/ramp1953/.codex/tmp/kmipkit-0116/coverage-github-output.txt` returned
  `eligible=true` after relocating tests. This resolves the
  exact failure from the first Windows coverage job, which rejected an inline
  `#[cfg(test)]` module in the production source before collecting coverage.
- The C sanitizer workflow commands were reproduced using the isolated
  worktree-volume target. `cargo +nightly rustc --locked -p kmipkit-ffi
  --lib -- -Zsanitizer=address`, followed by compiling and running
  `bindings/c/tests/extension_registry.c` under AddressSanitizer, passed all
  20 internal FFI harness cases with leak detection enabled.
- `bindings/java/native/tests/zeroizing_bytes_test.cpp` compiled with
  AddressSanitizer and UndefinedBehaviorSanitizer and ran with leak detection
  and halt-on-error enabled; it exited 0 without diagnostics.
- `cargo fmt --all --check` and `git diff --check` passed after the tests were
  added.
- `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D
  warnings` passed after changing the local depth-limit assertion to
  `assert_ne!`; its focused test passed as well. The complete protocol suite
  and Clippy were rerun after moving the private tests to the excluded test
  path.

## Manifest scope ruling

Ruling: Keep the public `client_1_0` operation inventory at its 56 KMIP 2.1
entries and represent the seven KMIP 3.0 fluent routes through a separate
`kmip30_fluent_operations` capability, linked into the generated versioned
operation inventory. The existing manifest validator requires the main
operation list to match its approved KMIP 2.1 dependency matrix exactly, and
the separate capability preserves explicit opt-in and draft status. If this
is wrong, clients that inspect only the main operation list may miss the
versioned 3.0 routes; the generated inventory will link them through the
capability and source edition.

## Development commits

- Red: `cdf4257cf` and `97eea9e07` add failing API-surface and credential
  alternative tests before implementation.
- Green: `51b23bb2e` implements the Java/JNI and Python/CFFI routes alongside
  the matching typed API integration.
- Refactor: `ddc6fbc09` reconciles manifest exports and generated bindings;
  `c79d98018` regenerates the versioned fluent operation inventory after the
  approved parent rebase.
- Refactor: `c4f753197` records the final manifest verification and makes two
  schema tests validate only the records relevant to their assertions.

## Draft pull request

Draft PR [#213](https://github.com/NeverWe1come/KMIPKit/pull/213) is open with
the required `[WIP]` title and is stacked on KMIPKIT-0115. Its last pushed
head when this evidence was recorded was `2849e50ba`; the supplemental protocol
coverage commit is pending. Keep the WIP marker until all required checks on
the current PR head are green.
