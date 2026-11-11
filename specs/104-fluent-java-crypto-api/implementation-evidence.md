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
- Command: `bash gradlew test --no-daemon --console=plain
  -Dorg.gradle.jvmargs='-Xmx3g -XX:MaxMetaspaceSize=768m'` from `bindings/java`.
- Expected Red result: `:compileTestJava` fails because the Java crypto builder
  classes do not yet exist. The first local run used the default heap and ended
  in `OutOfMemoryError` before compiling tests; rerunning with the Gradle heap
  set to 3 GiB reached and confirmed the intended missing-class failure.

## Green

Pending.

## Refactor

Pending.

## Verification

Pending exact-head CI and local test evidence.
