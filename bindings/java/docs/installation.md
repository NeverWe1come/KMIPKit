# Java Binding Installation & Build

## Maven Central / Gradle Consumer Coordinates

```kotlin
dependencies {
    implementation("org.kmipkit:kmipkit-java:1.0.0")
}
```

```xml
<dependency>
    <groupId>org.kmipkit</groupId>
    <artifactId>kmipkit-java</artifactId>
    <version>1.0.0</version>
</dependency>
```

## Source Build (Gradle Wrapper & Maven)

Run commands from the repository root using JDK 17+ (JDK 21 recommended), the
repository's Rust toolchain, pinned OpenSSL 3.5.9, and a C++17 compiler (MSVC
on Windows; see [build.ps1](../native/build.ps1)).

Build the Java API and host native classifier JAR with the committed wrapper:

```sh
sh bindings/java/gradlew -p bindings/java nativeJar
```

In PowerShell:

```powershell
& bindings/java/gradlew.bat -p bindings/java nativeJar
```

The task builds `kmipkit-ffi`, compiles JNI, and packages the Java classes with
the JNI library and its SHA-256 sidecar under `bindings/java/build/libs/`.

Maven is also supported for building and testing:

```sh
mvn --batch-mode --file bindings/java/pom.xml test
```
