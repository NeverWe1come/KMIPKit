# KMIPKIT-0104 Implementation Evidence

## Baseline

- Branch: `feature/KMIPKIT-0104-java-fluent-crypto-api`.
- Parent: `feature/KMIPKIT-0103-fluent-c-crypto-api` at the current published
  head when this slice was created.
- Scope: Java facade for the seven KMIPKIT-0102 cryptographic request builders,
  using the opaque C builder ABI from KMIPKIT-0103.
- PR: pending implementation and publication; title will retain `[WIP]` until
  all required checks on the exact head pass.

## Red

- Added `FluentCryptoApiShapeTest` to require all seven typed entry points,
  common explicit-parameter and execution methods, and operation-specific
  setters.
- Added a failing API-audit test for public builders that inherit semantic
  setters through package-private Java interfaces. Before the scanner change,
  the test failed because `EncryptBuilder.data` was absent from the inventory.
- Command: `bash gradlew test --no-daemon --console=plain
  -Dorg.gradle.jvmargs='-Xmx3g -XX:MaxMetaspaceSize=768m'` from `bindings/java`.
- Expected Red result: `:compileTestJava` fails because the Java crypto builder
  classes do not yet exist. The first local run used the default heap and ended
  in `OutOfMemoryError` before compiling tests; rerunning with the Gradle heap
  set to 3 GiB reached and confirmed the intended missing-class failure.
- The API-audit contract test also failed before implementation because
  package-private interface setters were omitted from the Java export
  inventory.

## Green

- Added JNI wrappers over the Rust-owned C builder for all seven crypto
  operations and Java builders with operation-specific setters.
- Added a mutual-TLS TLS 1.3 fixture test that builds and executes each
  operation once, checks the encoded Operation and Data, verifies a typed
  Success result, and confirms `RequestOptions` and the explicit asynchronous
  indicator reach the exchange separately.
- Added English and Spanish usage guidance for the Java facade and ownership
  boundaries; added all Java declarations and requirement IDs to the source
  public API manifest and regenerated the 12 manifest outputs.
- Targeted command from `bindings/java`:
  `bash gradlew test --no-daemon --console=plain -Dorg.gradle.jvmargs='-Xmx3g -XX:MaxMetaspaceSize=768m' --tests 'org.kmipkit.CryptoOperationsJavaFacadeTest.fluentBuildersExecuteOneTypedOperationEachAgainstMutualTlsFixture' --tests org.kmipkit.fluent.FluentCryptoApiShapeTest`
- Result: `BUILD SUCCESSFUL` after the request fixture verified the KMIP 2.1
  Asynchronous Indicator Enumeration tag and the request payload's Data field.
- Manifest command: `python3 tools/api_manifest/generate.py --check`.
- Result: `public API manifest outputs verified: 12 files`.
- Added package-private interface traversal to the Java declaration audit and
  kept the JNI bridge and its helper methods package-private so they do not
  become user-facing exports.
- Audit test command: `python3 -m unittest
  tools.api_manifest.tests.test_binding_exports`.
- Result: all 58 tests passed.
- Exact binding export command: `python3
  tools/api_manifest/audit_binding_exports.py --report
  /tmp/kmipkit-0104-java-exports-v3.json --check`.
- Result: C, Java, and Python inventories match all declared exports; zero
  missing, stale, or mismatched declarations.
- Full Java command from `bindings/java`:
  `bash gradlew test --no-daemon --console=plain
  -Dorg.gradle.jvmargs='-Xmx3g -XX:MaxMetaspaceSize=768m'`.
- Result: `BUILD SUCCESSFUL`.
- Rust/C ABI command: `cargo test -p kmipkit-ffi --all-features --locked`.
- Result: 471 unit tests, 55 C ABI coverage tests, and the operation-parity test
  passed.
- Rust client command: `cargo test -p kmipkit-client --all-features --locked`.
- Result: 879 unit tests, integration tests, UI compile tests, and doctests
  passed; external Cosmian tests remained ignored because that local service is
  not configured.
- Formatting command: `cargo fmt --all --check`.
- Result: passed.

## Refactor

Pending the separate cleanup commit and exact-head CI.

## Verification

Pending exact-head CI and local test evidence.
