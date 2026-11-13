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

No production code was changed before these tests failed.

## Green

- Rust fluent builders are now available for all seven KMIP 3.0 operations.
  `cargo test -p kmipkit-client kmip30_fluent -- --nocapture` passed all
  three focused tests: operation-specific typed request construction,
  no-send rejection when KMIP 3.0 is not explicitly selected, and explicit
  KMIP 3.0 wire version/operation encoding. `cargo fmt --all --check` passed.
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
  passed after documenting the validation result contract.

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
