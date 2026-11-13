# KMIPKIT-0115 Implementation Evidence

## Red

- Commit: recorded in the Red development commit for T002/T003.
- Java: `JAVA_TOOL_OPTIONS='-Xmx6g -XX:MaxMetaspaceSize=1g' bash gradlew --no-daemon -Dorg.gradle.jvmargs='-Xmx6g -XX:MaxMetaspaceSize=1g' -Dorg.gradle.workers.max=1 test --tests org.kmipkit.fluent.FluentLifecycleApiShapeTest --tests org.kmipkit.FluentLifecycleNativeIntegrationTest` compiled the test sources and failed the expected missing-method assertions for `FluentRequest.activate`, `KmipClient.activateById`, and the TLS consumer's direct-method lookup.
- Python: `/home/ramp1953/.cache/kmipkit-t082-python-venv-20261016/bin/pytest -q tests/test_fluent_lifecycle.py` failed the expected absence assertions for the lifecycle request builders and client helpers; the consumer tests stopped at those absent entry points before any network exchange.
- The initial Java test attempt used the default Gradle/Javac heap and failed during test compilation with `OutOfMemoryError`. Re-running with a 6 GiB JVM heap compiled all Java tests and reached the expected Red assertions; this was an environment limit, not a test or production failure.

## Green

- Java lifecycle shape and controlled mTLS tests passed after implementing the five builders and direct helpers. The full Java suite passed: 712 tests, 1 skipped.
- Python focused lifecycle tests passed (36 tests), and the full Python suite passed (1,362 tests).
- Java's first full-suite run had two bounded-heap subprocess assertions fail because the parent `JAVA_TOOL_OPTIONS` value was inherited and printed to the subprocess output. With the same Gradle heap configured through `org.gradle.jvmargs` and `JAVA_TOOL_OPTIONS` unset, the two affected tests passed, followed by the full Java suite passing.
- `python3 -B tools/api_manifest/generate.py --write` regenerated all 12 outputs. `generate.py --check` passed.
- `audit_binding_exports.py --check` passed with C, Java, and Python all matching the manifest exactly: no missing, stale, unobserved, or kind-mismatched declarations.
- `git diff --check` passed.

## Refactor

- Red commit: `3d5aee2c`; Green commit: `8309c9df`; Refactor is recorded in its separate commit.
- Centralize one-item batch execution for Get/Destroy/Activate/Archive/Recover, Locate, and Revoke in Java and Python; request construction, timeout options, asynchronous indicator handling, and transfer/release boundaries remain unchanged.
- Post-refactor full suites passed: Java 712 tests, 1 skipped; Python 1,362 tests.
