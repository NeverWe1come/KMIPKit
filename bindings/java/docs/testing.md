# Java testing

Run commands from the repository root with the prerequisites in
[installation](installation.md).

```sh
sh bindings/java/gradlew -p bindings/java test
sh bindings/java/gradlew -p bindings/java nativeJarSmoke
```

On Windows replace `sh bindings/java/gradlew` with
`& bindings/java/gradlew.bat` in PowerShell.

`test` runs JUnit against the JNI library built for the host. `nativeJarSmoke`
starts a separate Java process to check loading from the packaged classifier
JAR. The native build script also runs focused C++ checks.

The Maven path is:

```sh
mvn --batch-mode --file bindings/java/pom.xml test
```

It builds JNI, runs JUnit, and executes the
[vendor extension example](../examples/README.md). See
[the Maven configuration](../pom.xml) for coverage lifecycle bindings.

Tests use [shared fixtures](../../shared/fixtures/), including the disposable
[mTLS identities](../../shared/fixtures/login-mtls/README.md). Controlled local
TLS tests require permission to bind a loopback port. External-server checks
live in [verification](../../../verification/README.md) and require their own
server configuration.
