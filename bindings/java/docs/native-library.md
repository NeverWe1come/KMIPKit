# Java native library

The Java build can produce one classifier JAR per supported native target. Each
JAR contains the JNI library and a lowercase SHA-256 sidecar at
`META-INF/native/<target>/kmipkit_jni.<ext>` and
`META-INF/native/<target>/kmipkit_jni.<ext>.sha256`.

| Target ID | Operating system | Architecture | Library |
|---|---|---|---|
| `windows-x86_64-msvc` | Windows | x86-64 | `kmipkit_jni.dll` |
| `linux-glibc-x86_64` | Linux with glibc | x86-64 | `libkmipkit_jni.so` |
| `linux-glibc-aarch64` | Linux with glibc | AArch64 | `libkmipkit_jni.so` |
| `macos-x86_64` | macOS | x86-64 | `libkmipkit_jni.dylib` |
| `macos-aarch64` | macOS | AArch64 | `libkmipkit_jni.dylib` |

Run commands from the repository root with the prerequisites in
[installation](installation.md). Build a target package on its target OS and
architecture (the native library is built as a task dependency):

```text
sh bindings/java/gradlew -p bindings/java nativeJar
```

The result is `bindings/java/build/libs/kmipkit-java-<version>-<target-id>.jar`.
Use `-PnativeTarget=<target-id>` to name the host target explicitly and
`-PnativeLibrary=<absolute-path-to-library>` to package a specific trusted JNI build.
The task copies the selected binary to the expected resource path and calculates its
SHA-256 from those copied bytes. `nativeJarSmoke` builds the same package and
starts a fresh Java process that loads the JNI resource from the JAR without
setting the native-library override:

```text
sh bindings/java/gradlew -p bindings/java nativeJarSmoke
```

The adapter selects a resource from `os.name` and `os.arch`; it does not
download executable code or fall back to `java.library.path`. It hashes the
resource while copying it to a newly created directory below
`java.io.tmpdir`. POSIX filesystems require exact directory mode `0700` and
file mode `0600`. Windows requires an ACL containing only the directory or
file owner with full permissions and no inherited ACEs. Unsupported or
unverifiable permissions fail closed.

The staging file uses `CREATE_NEW` and `NOFOLLOW_LINKS`. The loader forces the
copied file, checks the packaged digest, atomically moves it to its final name
without a replace option, then verifies its private permissions, final digest,
and identity before calling `System.load` with an absolute path. If NIO does
not expose a file key, the loader verifies identity through a temporary hard
link and `Files.isSameFile`; inability to make that check fails closed.
Directories and filenames are freshly randomized and are never reused.

The optional `-Dkmipkit.nativeLibrary=<absolute-path>` property is an explicit
administrator override. It bypasses bundled-resource selection and extraction
and is trusted as executable code. After either load path, the JNI bridge calls
the canonical `kmipkit_version_get` function and accepts only ABI version 1.
Missing symbols, a failed query, or any other ABI version fail closed. Loading
failures have a generic message and do not expose the path, resource, or
underlying exception.

The SHA-256 sidecar binds the native bytes to the sidecar inside the package; it
does not authenticate a JAR if both entries are replaced together. Package
authenticity remains the responsibility of the package distribution and its
signing/provenance controls. The loader's trust boundary excludes processes
running as the same user and administrators because `System.load` accepts a
pathname that can be changed by those actors.

On Windows use `bindings/java/gradlew.bat` in PowerShell instead of
`sh bindings/java/gradlew`. See [testing](testing.md) for the packaged loader check.
