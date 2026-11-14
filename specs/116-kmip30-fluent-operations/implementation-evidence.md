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
