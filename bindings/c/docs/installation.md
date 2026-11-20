# C installation and build

Run commands from the repository root. Source builds require the Rust toolchain
specified by [Cargo.toml](../../../Cargo.toml), a C11 compiler, CMake 3.20 or
newer, and the native build prerequisites for the Rust crypto dependencies.
See the [Rust workspace guide](../../../docs/development/rust-workspace.md).

```sh
cargo build --locked -p kmipkit-ffi
cmake -S bindings/c -B build/c-consumer
cmake --build build/c-consumer --config Release
```

Cargo produces the native library under `target/debug/`. The CMake project
builds repository consumers and the example against that library. The
`--config Release` argument selects the CMake configuration for generators
such as Visual Studio; the Rust library in this command is still a debug build.

To select a separately built Rust release library:

```sh
cargo build --locked --release -p kmipkit-ffi
cmake -S bindings/c -B build/c-release -DKMIPKIT_TARGET_DIR="$PWD/target/release"
cmake --build build/c-release --config Release
```

The last example uses POSIX shell syntax; use an absolute directory path in
PowerShell. Keep different library selections in separate CMake build directories.

The public header is [kmipkit.h](../include/kmipkit.h). For an application,
add `bindings/c/include` to its include path and link the matching
`kmipkit_ffi` library. [Native linking](native-library.md) describes runtime
resolution and ownership. [package.toml](../package.toml) records distribution
metadata; it does not create an installable archive. The current CMake project
is a consumer build, not an installation or package publication workflow.
