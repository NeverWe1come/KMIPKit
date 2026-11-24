# Java Native Library Loading

`NativeLibraryLoader` automatically extracts the platform-matched `libkmipkit_jni` and `libkmipkit_ffi` binaries bundled in the JAR. To override the library path in restricted containers:

```bash
java -Dkmipkit.native.path=/opt/kmipkit/lib -jar app.jar
```

Always wrap `TransportConfiguration`, `ClientConfiguration`, `Client`, `Authentication`, `SecretBytes`, `SecretText`, and `ClientBatchResponse` in `try-with-resources` so native handles are freed and zeroized immediately.
