# C ABI and language bindings

## ABI contract

The C ABI is the shared boundary for C, Java, Python, and later adapters.

- Prefix every exported symbol with `kmipkit_`.
- Use fixed width integers and explicit pointer lengths.
- Expose opaque handles, never Rust layouts.
- Version public structures with `struct_size` and `abi_version`.
- Provide version and capability query functions.
- Return a stable status code and use output parameters.
- Provide detailed last-error access per thread.
- Catch every Rust panic at each exported function.
- Keep 1.x ABI additions backward compatible.

## Memory ownership

- Input memory is borrowed only for the duration of the call.
- Results are Rust-owned handles or buffers with matching free functions.
- Callers never use the platform allocator to release Rust memory.
- Text is UTF-8 with an explicit length.
- Borrowed views with parent-dependent lifetimes are avoided.
- Secret outputs use zeroizing opaque handles.

## Generated and manual portions

Manually reviewed code owns lifecycle, configuration, execution, errors,
memory, and panic containment. A reviewed public API manifest generates the
repetitive operation builders, accessors, constants, and declarations.

The generated Rust exports are the input to cbindgen. Generated outputs are
checked in and CI verifies deterministic regeneration and compatibility.

## C distribution

- C11 headers under `include/kmipkit/` with an umbrella `kmipkit.h`.
- C++ compatibility through `extern "C"`.
- Dynamic and static libraries.
- Linux SONAME `libkmipkit.so.1`.
- macOS `libkmipkit.1.dylib`.
- Windows `kmipkit.dll` plus import/static libraries.
- Only `kmipkit_*` symbols have default visibility.
- CMake package and pkg-config metadata.

## Java

- Java 17 minimum.
- Gradle Kotlin DSL and pinned wrapper.
- Hand-written idiomatic facade and resource management.
- Generated repetitive models and JNI declarations.
- Minimal JNI bridge into the stable C ABI.
- Unchecked exception hierarchy rooted at `KmipKitException`.
- JSpecify nullness annotations.
- `AutoCloseable` clients and secret values.
- Platform native JARs selected automatically.
- SLF4J integration for redacted structured events.

The adapter blocks only the calling Java thread. It does not implement KMIP
semantics or maintain a separate wire model.

## Python

- Python 3.12 minimum.
- CFFI ABI mode.
- Maturin build backend in `pyproject.toml`.
- uv for development dependency management.
- pytest, Ruff, and Pyright.
- Complete type annotations and `py.typed`.
- Context managers for clients and secrets.
- Blocking native calls release the GIL while waiting for network I/O.

## Native loading

Official Java and Python packages include the native binary. They never
download executable code at runtime.

Java selects the OS and architecture, verifies the bundled SHA-256, extracts
atomically to a private location, and loads with `System.load`. Python loads
the library adjacent to the installed wheel. Both may accept an explicit
administrator-provided native path and fail fast on ABI mismatch.

## Parity gate

A 1.0 feature is complete only when Rust, C, Java, and Python expose equivalent
high level, typed, and generic TTLV capability with tests and documentation.
Language APIs remain idiomatic; parity concerns semantics rather than spelling.
