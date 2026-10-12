# Java and Python packages

## Version and runtimes

The Java and Python package metadata follows the `releaseLine` in the [public
API manifest](../../../specification/api/public-api.json), currently `1.0.0`.
The Java coordinates are `org.kmipkit:kmipkit-java`; the Python distribution
name is `kmipkit`. Gradle, Maven, and Python project metadata use the same
version. This is unreleased project metadata; it does not mean that a 1.0.0
artifact has been published.

The minimum language runtimes are Java 17 and CPython 3.12. The manifest's
current Java and Python package target matrix is:

| Target ID | Operating system and architecture |
|---|---|
| `windows-x86_64-msvc` | Windows, x86-64 (MSVC) |
| `linux-glibc-x86_64` | Linux with glibc, x86-64 |
| `linux-glibc-aarch64` | Linux with glibc, AArch64 |
| `macos-x86_64` | macOS, x86-64 |
| `macos-aarch64` | macOS, Apple silicon |

These are package targets recorded in the manifest, not a claim that every
published artifact or platform test is complete.

## Native library loading

Both package families bundle their native library and do not perform a
runtime executable-code download.

Java selects the bundled native JAR resource for the current operating system
and architecture, verifies its SHA-256 sidecar, extracts it atomically into a
private temporary directory, and checks the loaded library through
`kmipkit_version_get`. It accepts ABI version 1. The explicit
`-Dkmipkit.nativeLibrary=<path>` administrator property replaces the bundled
library selection. It is trusted executable code; an invalid override fails
with no fallback to the packaged library. The loader does not search
`java.library.path` or download a library. See [Java native loading](../../../bindings/java/docs/native-library-loading.md)
for the filesystem and verification details.

Python loads the single regular `.dll`, `.dylib`, or `.so` adjacent to the
installed package's generated CFFI declarations. Symlinks and missing or
ambiguous adjacent libraries are rejected. `KMIPKIT_NATIVE_LIBRARY` is the
only administrator path override; when set, it replaces bundled selection
and a missing or incompatible override fails with no fallback. Before
exposing CFFI, the package calls `kmipkit_version_get` and requires ABI
version 1. Python does not search other directories or download a library.

## Current manifest capability scope

The status labels below are copied from the current manifest. `declared` means
the manifest contains a mapping; it is not evidence that full cross-language
parity or release verification has passed. `blocked` means the capability
must not be advertised as available. The manifest is the source of truth for
changes to this table.

| Capability ID | Manifest state | Current scope |
|---|---|---|
| `generic_ttlv_model` | declared | A four-language mapping is recorded; complete parity verification is still pending. |
| `ttlv_wire_codec` | declared | A four-language mapping is recorded; complete parity verification is still pending. |
| `ffi_runtime_metadata` | declared | Version and capability-query metadata is mapped in Rust, C, Java, and Python. |
| `rust_transport_exchange` | declared | Rust-only transport exchange under accepted ADR-0014; it is not a foreign-language client operation. |
| `public_binding_surface` | blocked | The complete public declaration inventory and mappings are not reconciled. |
| `vendor_extension_registry` | declared | Exact shared mappings are recorded; parity verification remains pending and unreconciled exports stay blocked. |

All **56 / 56** KMIP `client_1_0` operation rows currently have manifest state
`blocked`. Do not infer operation availability from types or implementation
code elsewhere in the repository. KMIPKit does not claim full operation
parity, a supported profile, or formal certification from this snapshot.

For the authoritative operation owners and per-capability mappings, see the
[manifest](../../../specification/api/public-api.json) and the [public API
design](../../architecture/public-api.md). The overall project remains
unreleased.
