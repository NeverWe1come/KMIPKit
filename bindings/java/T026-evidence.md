# T026 Java native loading evidence

This file records Java-loader evidence. It does not close T028's multi-platform
matrix or claim a Java 17 runtime run. Local commands ran on Windows 11 x86-64
with Oracle JDK 21.0.12; Maven and Gradle compile Java source with release 17.

## Red

| Commit | Command | Result |
|---|---|---|
| `e9e8567a` | `mvn -f bindings/java/pom.xml -Dtest=NativeLibraryLoaderTest test` | Expected compile failure: `NativeLibraryLoader` was not implemented. |
| `4bd26409` | `mvn -f bindings/java/pom.xml -Dtest=NativeLibraryLoaderTest test` | Expected compile failure at `validateNativeAbiVersion`: the test specifying rejection of unsupported ABI metadata was written before the version-check implementation. |

## Green

| Commit | Command | Result |
|---|---|---|
| `9966a452` | `mvn -f bindings/java/pom.xml -Dtest=NativeLibraryLoaderTest test` | PASS: seven focused loader tests; Maven compiled Java with `--release 17`, built the Windows JNI library, and completed the vendor extension example. |
| `9966a452` | `bindings/java/gradlew.bat -p bindings/java test` | PASS: Java tests using the Gradle Kotlin DSL and native build task. The wrapper distribution is pinned to Gradle 9.7.1 with SHA-256 `acd53f1edaf02f1a8ff99879f8a34b302661a057d9b063ae9e35b552f804d20a`; the committed wrapper JAR SHA-256 is `7a9ce74cff467ca1bf60a4fcd9f05185acceda4d0f382434d393e17864262c5d`. |
| `9966a452` | `bindings/java/gradlew.bat -p bindings/java nativeJarSmoke -PnativeTarget=windows-x86_64-msvc -PnativeLibrary=<absolute path to target/native/kmipkit_jni.dll>` | PASS: packaged `kmipkit-java-0.1.0-SNAPSHOT-windows-x86_64-msvc.jar`; a separate Java process printed `bundled JNI loaded` without the administrator override. The JNI resource SHA-256 was `affa5e6e66f36122ac8c417b5c91716ad0ff5d9b9af8fde611aa611b5e023215`. |
| `11aba28a` | `mvn -f bindings/java/pom.xml -Dtest=NativeLibraryLoaderTest test` | PASS: eight focused tests; the rebuilt Windows JNI library linked the `kmipkit_version_get` query and the test passed with generic, cause-free mismatch diagnostics. |

The wrapper distribution checksum was verified against the downloaded official
9.7.1 binary distribution before generating the wrapper. The wrapper then
downloaded and started Gradle 9.7.1 under a fresh `GRADLE_USER_HOME`, and the
Java test task passed.

## Refactor

The signed Refactor commit centralizes explicit-path and bundled-path loading
through one load-and-verify operation. Verification after this refactor ran on
Windows 11 x86-64 with Oracle JDK 21.0.12 and a fresh Gradle user home:

| Command | Result |
|---|---|
| `mvn -f bindings/java/pom.xml -Dtest=NativeLibraryLoaderTest test` | PASS: eight focused loader tests; native JNI rebuilt and the vendor extension example completed. |
| `bindings/java/gradlew.bat -p bindings/java --rerun-tasks test` | PASS: all Gradle Java tests and a Windows JNI rebuild; Java compilation used `--release 17`. |
| `bindings/java/gradlew.bat -p bindings/java --rerun-tasks nativeJarSmoke -PnativeTarget=windows-x86_64-msvc -PnativeLibrary=<absolute path to target/native/kmipkit_jni.dll>` | PASS: the packaged JAR started a fresh Java process, loaded its bundled JNI library, queried ABI 1 through `kmipkit_version_get`, and printed `bundled JNI loaded` without the override property. JAR SHA-256: `c4f45cf312e04709bc30cfd2bcd4f50f2b4c3c6a3dbff13949544c1bbd93b77f`; bundled JNI SHA-256: `1efb51b28b847bb2cca14402797929edaf2af51cb19601174f954f8334e46712`. |

## Scope and limitations

- Unit tests exercise missing resources, corrupt digests, destination
  collisions, owner-only permissions/ACLs, fresh extraction paths, ABI
  mismatch, and redacted failures using per-test temporary directories.
- Local Windows verification confirms the ACL path on JDK 21. The selected
  source and test compile against Java 17, but this host does not have a JDK 17
  runtime. Java 17 runtime verification and the supported-target test matrix
  remain part of T028.
- The current Gradle package smoke run verifies the Windows target only. It is
  not evidence for Linux or macOS package loading.
- The committed `bindings/java/pom.xml` is retained; this change adds the
  Gradle wrapper and Kotlin DSL without deleting the existing Maven build.
