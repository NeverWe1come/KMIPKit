# Java Native Library Loading & Security Model

Each platform classifier JAR packages its JNI library and lowercase SHA-256 sidecar
at `META-INF/native/<target>/kmipkit_jni.<ext>` and
`META-INF/native/<target>/kmipkit_jni.<ext>.sha256`:

| Target ID | Operating system | Architecture | Library |
|---|---|---|---|
| `windows-x86_64-msvc` | Windows | x86-64 | `kmipkit_jni.dll` |
| `linux-glibc-x86_64` | Linux with glibc | x86-64 | `libkmipkit_jni.so` |
| `linux-glibc-aarch64` | Linux with glibc | AArch64 | `libkmipkit_jni.so` |
| `macos-x86_64` | macOS | x86-64 | `libkmipkit_jni.dylib` |
| `macos-aarch64` | macOS | AArch64 | `libkmipkit_jni.dylib` |

## Administrator Override

`NativeLibraryLoader` automatically extracts and verifies the platform-matched
binary bundled in the JAR. To override the library path explicitly in controlled
environments:

```sh
java -Dkmipkit.nativeLibrary=/opt/kmipkit/lib/libkmipkit_jni.so -jar app.jar
```

The optional `-Dkmipkit.nativeLibrary=<absolute-path>` property bypasses
bundled-resource selection and extraction and is trusted as executable code.

## Fail-Closed Extraction & Verification Model

- The loader never downloads executable code or falls back to `java.library.path`.
- It stages the resource under a freshly randomized directory in `java.io.tmpdir`
  with strict permissions: POSIX mode `0700` on directories and `0600` on files,
  or Windows ACLs restricted solely to the owner without inherited ACEs.
- Files are created with `CREATE_NEW` and `NOFOLLOW_LINKS`, fsynced, verified
  against the SHA-256 sidecar digest before and after atomic rename, and checked
  for identity before invoking `System.load`.
- After loading, the JNI bridge calls `kmipkit_version_get` and fails closed if
  the ABI version is not `1`.
- Always wrap `TransportConfiguration`, `ClientConfiguration`, `Client`,
  `Authentication`, `SecretBytes`, `SecretText`, and `ClientBatchResponse` in
  `try-with-resources` so native handles are freed and zeroized deterministically.
