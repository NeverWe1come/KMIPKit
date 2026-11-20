# Java installation and build

Run commands from the repository root. Use a JDK compatible with the committed
Gradle wrapper (JDK 21 is suitable for this build), the repository's Rust
toolchain, and a C++17 compiler. Set `JAVA_HOME` to the JDK when automatic
header discovery is unsuitable. On Windows use the MSVC build environment;
see [build.ps1](../native/build.ps1). Java sources compile with `--release 17`.

Build the Java API and host native classifier JAR with the committed wrapper:

```sh
sh bindings/java/gradlew -p bindings/java nativeJar
```

In PowerShell use:

```powershell
& bindings/java/gradlew.bat -p bindings/java nativeJar
```

The task builds the Rust library, runs the build script's focused native
checks, compiles JNI, and packages the Java classes with the JNI library and
its SHA-256 sidecar. Output is under `bindings/java/build/libs/`; the classifier
identifies the host OS and architecture. Package on the target platform.

Maven is also supported for the Java build and tests:

```sh
mvn --batch-mode --file bindings/java/pom.xml test
```

Maven runs the local vendor extension example during its test lifecycle. Gradle's
`nativeJar` task supplies the explicit classifier JAR packaging described above.
See [native loading](native-library.md), [testing](testing.md), and
[examples](../examples/README.md) for usage and verification.
