# C native library

The [Rust FFI crate](../../../crates/kmipkit-ffi/Cargo.toml) builds `cdylib`,
`staticlib`, and Rust library artifacts. Applications use the C ABI declared
in [kmipkit.h](../include/kmipkit.h).

Typical shared library names are `libkmipkit_ffi.so` on Linux,
`libkmipkit_ffi.dylib` on macOS, and `kmipkit_ffi.dll` on Windows. Link against
the artifact for the application's operating system, architecture, and toolchain.
Static linking can also require platform system libraries; use the platform
link requirements reported by the Rust toolchain.

The repository's CMake consumer resolves the library only from
`KMIPKIT_TARGET_DIR`, which defaults to `target/debug` relative to the repository.
It configures a build RPATH for its executables. On Windows, place the matching
DLL on the process `PATH` before running consumers. An application must arrange
its own deployment and runtime library search path.

For missing-library or missing-symbol errors, verify the selected directory,
architecture, and header/library versions. The `kmipkit_version_get` ABI query
allows consumers to inspect the loaded library's version and ABI. See the
[shared binding architecture](../../../docs/architecture/ffi-and-bindings.md).

Release every owned opaque handle with its matching release function. Observe
borrowed-view lifetimes and clear application-owned copies of sensitive data.
The [C example guide](../examples/README.md) identifies its additional private
repository header dependency.
