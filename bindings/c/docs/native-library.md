# C Native Library & Memory Ownership

The [Rust FFI crate](../../../crates/kmipkit-ffi/Cargo.toml) builds `cdylib`,
`staticlib`, and Rust library artifacts exporting the C11 ABI declared in
[kmipkit.h](../include/kmipkit.h).

## Memory Ownership & Error Snapshots

1. **Always use matching destructors:** Never pass pointers returned by `kmipkit_*`
   to `stdlib.h` `free()`. Always call the matching `kmipkit_<type>_free()` or
   release destructor, which zeroizes secret buffers.
2. **Versioned structs:** Initialize `struct_size = sizeof(s)` and `abi_version = 1`
   before calling `kmipkit_last_error_get_v1()` or extension registry limit APIs.
3. **Thread-local error snapshots:** Call `kmipkit_last_error_get_v1(&snap)` after
   any non-`KMIPKIT_OK` status to inspect `category`, `cause`, and `delivery_state`,
   then `kmipkit_last_error_clear()`.

## Linking & Runtime Resolution

- **Shared library names:** `libkmipkit_ffi.so` (Linux), `libkmipkit_ffi.dylib` (macOS),
  and `kmipkit_ffi.dll` (Windows).
- **CMake discovery:** The repository's CMake consumer resolves the library from
  `KMIPKIT_TARGET_DIR` (default `target/debug` relative to the repository) and
  configures a build RPATH for its executables. On Windows, place the matching DLL
  directory on the process `PATH`.
- **ABI check:** Consumers can call `kmipkit_version_get` at runtime to verify the
  loaded library version and ABI version (`1`).
- Observe borrowed-view lifetimes and clear application-owned copies of sensitive data.
  See the [shared binding architecture](../../../docs/architecture/ffi-and-bindings.md)
  and [C example guide](../examples/README.md).
