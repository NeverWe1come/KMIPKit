# C ABI and language bindings

## ABI contract

The C ABI is the shared boundary for C, Java, Python, and later adapters.

- Prefix every exported symbol with `kmipkit_`.
- Use fixed width integers and explicit pointer lengths.
- Expose opaque handles, never Rust layouts.
- Version public structures with `struct_size` and `abi_version`. Callers set
  `struct_size` to the byte size of the concrete versioned structure and
  `abi_version` to its declared ABI revision; these fields are C ABI metadata,
  not KMIPKit facade values.
- Provide version and capability query functions.
- Return a stable status code and use output parameters.
- Provide detailed last-error access per thread.
- Catch every Rust panic at each exported function.
- Keep 1.x ABI additions backward compatible.

The extension-registry limits v1 value is passed to the C defaults, validation,
and registry-creation entry points. Callers initialize `struct_size` to exactly
`sizeof(kmipkit_extension_registry_limits_v1_t)` and `abi_version` to `1`.
These entry points reject a null pointer, a different size, or an unsupported
ABI version before reading or writing the limit fields. The defaults function
preserves the initialized metadata fields while filling the twelve values.

### Runtime metadata and error snapshots

`kmipkit_version_get` returns the synchronized package version as three
`uint32_t` components and the stable C ABI revision (`1` for the initial 1.x
ABI). `kmipkit_capability_is_available` accepts a UTF-8 capability ID and
returns `1` only when the generated manifest declares a non-blocked C mapping;
this does not imply that a remote KMIP server supports the capability. The
identifier is limited to 128 UTF-8 bytes in all four language facades; an
oversized identifier produces the shared validation/resource-limit failure.

`kmipkit_last_error_get` copies the calling thread's most recent nonzero C
status and safe metadata without clearing it. `kmipkit_last_error_clear` resets
that snapshot. The category values are `0` none, `1` validation, `2` protocol,
`3` transport, `4` server result, and `5` other. Cause values currently
exposed by the C surface are `0` none, `1` invalid input, and `10` other. The
delivery value `0` means no KMIP request delivery information is attached;
the current C surface does not execute client requests. Status values remain
the existing `KMIPKIT_ERROR_*` codes and are independent from category/cause
values. `KMIPKIT_ERROR_INTERNAL` (`7`) is the fixed fallback when a Rust panic
is caught at an exported boundary; its snapshot uses category `5` (other),
cause `10` (other), and delivery `0` (none). Bindings map it to their generic
base exception types. The returned status and snapshot do not contain the panic
payload. KMIPKit does not change Rust’s process-wide panic hook; an embedder’s
hook may observe panic details, so KMIPKit panic messages must never include
secrets. Snapshots contain no source strings or request payloads.

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

### Public C declarations and symbol inventory

The v2 [public API manifest](../../specification/api/public-api.json) owns the
C declaration inventory. Its generated [public C header](../../bindings/c/include/kmipkit.h)
contains only C mappings in the manifest's `declared` or `verified` state;
blocked rows remain inventory records and do not create callable declarations.
The [binding export report](../../specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json)
records the observed C, Java, and Python exports and their reconciliation with
the manifest. Regenerate or validate that inventory with the pinned commands
in [the export audit guide](../../tools/api_manifest/binding-export-audit.md);
the C source-level check is `python -B
tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --check`.
CI additionally audits the compiled C library on Windows, Linux, and macOS.

## C distribution

- C11 public umbrella header: [`kmipkit.h`](../../bindings/c/include/kmipkit.h).
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
the single regular `.dll`, `.dylib`, or `.so` file beside the generated CFFI
declarations. Symlinks and ambiguous or missing package-local libraries are
rejected. Python accepts only the explicit `KMIPKIT_NATIVE_LIBRARY`
environment variable as an administrator path override. When set, that path
replaces the bundled selection; a missing or incompatible override fails
without falling back to the bundled library. Before exposing the FFI, Python
calls `kmipkit_version_get` and requires ABI version 1. Neither loader searches
other directories or downloads executable code.
Java also accepts an explicit administrator-provided native path. Both fail
fast on ABI mismatch.

## Parity gate

A 1.0 feature is complete only when Rust, C, Java, and Python expose equivalent
high level, typed, and generic TTLV capability with tests and documentation.
Language APIs remain idiomatic; parity concerns semantics rather than spelling.
