# KMIPKIT-0115 Implementation Evidence

## Red

- Commit: recorded in the Red development commit for T002/T003.
- Java: `JAVA_TOOL_OPTIONS='-Xmx6g -XX:MaxMetaspaceSize=1g' bash gradlew --no-daemon -Dorg.gradle.jvmargs='-Xmx6g -XX:MaxMetaspaceSize=1g' -Dorg.gradle.workers.max=1 test --tests org.kmipkit.fluent.FluentLifecycleApiShapeTest --tests org.kmipkit.FluentLifecycleNativeIntegrationTest` compiled the test sources and failed the expected missing-method assertions for `FluentRequest.activate`, `KmipClient.activateById`, and the TLS consumer's direct-method lookup.
- Python: `/home/ramp1953/.cache/kmipkit-t082-python-venv-20261016/bin/pytest -q tests/test_fluent_lifecycle.py` failed the expected absence assertions for the lifecycle request builders and client helpers; the consumer tests stopped at those absent entry points before any network exchange.
- The initial Java test attempt used the default Gradle/Javac heap and failed during test compilation with `OutOfMemoryError`. Re-running with a 6 GiB JVM heap compiled all Java tests and reached the expected Red assertions; this was an environment limit, not a test or production failure.
