# Testing the Java Bindings

Run commands from the repository root after setting up the prerequisites in
[installation](installation.md):

```sh
sh bindings/java/gradlew -p bindings/java test
sh bindings/java/gradlew -p bindings/java nativeJarSmoke
```

On Windows in PowerShell:

```powershell
& bindings/java/gradlew.bat -p bindings/java test
& bindings/java/gradlew.bat -p bindings/java nativeJarSmoke
```

- `test` compiles the JNI bridge, runs focused native C++ checks, and runs JUnit.
- `nativeJarSmoke` builds the classifier JAR and launches an isolated JVM process
  verifying extraction and loading from the packaged JAR without setting
  `-Dkmipkit.nativeLibrary`.

## Maven Test & Coverage Path

```sh
mvn --batch-mode --file bindings/java/pom.xml test
```

Maven builds JNI, runs JUnit, and executes the
[vendor extension example](../examples/README.md). Tests use shared fixtures under
[bindings/shared/fixtures](../../shared/fixtures/), including disposable
[mTLS identities](../../shared/fixtures/login-mtls/README.md).
