# KMIPKIT-0116 Implementation Evidence

## Red

The first public-surface tests were run before production changes. Each failed
at the expected missing API assertion:

- Cross-language manifest contract:
  `python3.12 -B -m unittest tools.api_manifest.tests.test_0116_kmip30_fluent_manifest -v`
  failed because the seven 3.0 operation rows are absent from the public API
  matrix.
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
