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

- Red commit: `3d5aee2c`; Green commit: `8309c9df`; Refactor commit: `aaafa4a7`.
- Centralize one-item batch execution for Get/Destroy/Activate/Archive/Recover, Locate, and Revoke in Java and Python; request construction, timeout options, asynchronous indicator handling, and transfer/release boundaries remain unchanged.
- Post-refactor full suites passed: Java 712 tests, 1 skipped; Python 1,362 tests.

## Hosted T102 snapshot correction

The first hosted Script contracts run (`43499699503`) failed on Ubuntu because
the approved public API manifest contains 16,495 declarations while the shared
T102 checkpoint still expected 16,423. The 72-row increase is accounted for
by the ten C lifecycle declarations from KMIPKIT-0114 and the 62 Java/Python
lifecycle facade declarations from KMIPKIT-0115. After correcting the total,
the focused Certify snapshot test exposed stale declaration and operation
fingerprints. The shared T102 snapshot now records declaration hash
`23cf276c657dc3cf0ca453b97601aa5cd319f182a8eeabe87cb5d70880f3587d` and
operation hash
`574b163b50c24b128219f644956cc21c990879c1a5174486cd5b8b9d609597ff`.

Local verification so far:

| Command | Result |
|---|---|
| `python3.12 -B -m unittest tools.api_manifest.tests.test_t102_certify_request_builder_mapping tools.api_manifest.tests.test_t102_pkcs11_request_trait_exclusions -v` | Passed: 14 tests. |
| `python3.12 -B tools/api_manifest/generate.py --check` | Passed: all 12 generated outputs are current. |
| `git diff --check` | Passed. |

The rebased branch also changed declarations captured by the shared
T102 optional-request and response-view snapshots. The snapshots now account
for the inherited KMIPKIT-0093-FR-008 result links and Locate aliases, the ten
KMIPKIT-0114 C lifecycle declarations, and the 62 KMIPKIT-0115 Java/Python
lifecycle declarations. Their focused tests pass.

## Rebased T102 and export verification

- The complete T102 discovery ran in fresh subprocess groups of four modules
  to bound peak memory. All 108 modules passed: 405 tests, zero failed groups.
- `python3.12 -B tools/api_manifest/generate.py --check` passed; all 12
  generated outputs are current.
- `python3.12 -B tools/api_manifest/audit_binding_exports.py --repo-root . --check`
  passed for all three audited bindings. Exact manifest matches: C 2,022/2,022,
  Java 2,432/2,432, Python 2,643/2,643; zero missing, stale, unobserved, or
  kind-mismatched declarations.
- `git diff --check` passed.

PR #211 remains a draft while a new exact-head CI run validates these snapshot
and documentation updates. The prior hosted jobs were for the old remote SHA
and are not used as evidence for the rebased head.
