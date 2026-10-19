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
| `vendor_extension_registry` | declared | Exact shared mappings include Rust drop, C release, Java close, and Python close for native handles; parity verification remains pending. |
| `discover_versions_request_item` | declared | Four-language request-item builders are mapped; complete Discover Versions operation conformance remains pending. |
| `client_batch_model` | declared | The four-language client batch model exposes typed ordered requests and results; full operation coverage remains incomplete. |
| `client_execution` | declared | Discover Versions request and response mappings are present in Rust, C, Java, and Python; complete 1.0 operation conformance remains pending. |
| `crypto_response_ttlv` | declared | Generic response access is limited to approved Unique Identifier shape errors in successful MAC, MAC Verify, Sign, and Signature Verify responses. |
| `production_transport` | declared | Rust and the three adapters expose the approved TLS 1.3 client transport configuration; interoperability evidence remains pending. |
| `api_level_discovery` | declared | Rust, C, Java, and Python expose the local API-level taxonomy without network access. |
| `opaque_register_context` | declared | Explicit Opaque Object context and Register request-shape validation are mapped across four languages; full Register operation parity remains blocked. |
| `server_generated_creation` | declared | Server-generated identifiers, result metadata, and unique-identifier representations are mapped across four languages; complete operation conformance remains pending. |

Of the **56** KMIP `client_1_0` operation rows, **33 / 56** remain `blocked`.
Nine rows are `declared`: Create, Create Key Pair, Create Split Key, Discover
Versions, Hash, MAC, MAC Verify, Sign, and Signature Verify. Their four-language
request and response mappings are recorded; `declared` is not evidence of
complete operation conformance. Do not infer operation availability from types
or implementation code elsewhere in the repository. Fourteen operation mappings
are `verified`: Certify, Check, Delegated Login, Derive Key, Export, Get,
Get Constraints, Get Usage Allocation, Import, Interop, Join Split Key, Locate,
Log, and Login. The Derive Key mapping rejects a successful result without its
required Unique Identifier and preserves the
complete ordered response TTLV in all four languages. The Get Constraints mapping
sends the empty §6.1.22 request and exposes its required ordered Constraints
structure and full successful response TTLV in Rust, C, Java, and Python. C
returns a caller-owned copy; Java and Python callers release their owned response
with `close()`. Pending and failure results retain common result and delivery
metadata without exposing successful-response TTLV or triggering a retry. The
Export mapping preserves its optional identifier, raw Table 217 selectors,
wrapping specification, and owned result handles across Rust, C, Java, and
Python. The Get mapping preserves the optional Unique Identifier and Table 220
selectors and exposes its successful Any Object as an owned response in all
four languages. Locate preserves optional request filters, server order,
repeated Unique Identifiers, and the distinction between omitted and empty
Located Items. The Get Usage Allocation mapping includes its optional identifier
and signed count request, resolved identifier response, and common batch outcomes
in Rust, C, Java, and Python. This mapping evidence does not claim full operation
conformance. Log preserves its request message and exposes the complete response
TTLV through its typed result in all four languages. Login maps the optional
Table 253 fields and exposes the raw Ticket Type, Ticket Value, and ordered
generic Ticket TTLV from a successful response. C's
`kmipkit_client_batch_item_response_login` returns a Ticket Value span borrowed
until the response item is released. Its
`kmipkit_client_batch_item_response_login_ttlv` returns an independent
structure, released with `kmipkit_ttlv_structure_release`. Java uses
`ClientBatchItem.login(...)` and `ClientBatchItemResult.LoginResponse`;
`ticketValue()` returns a caller-owned `byte[]` that must be protected and
cleared, and `close()` clears the KMIPKit-owned value and closes the Ticket TTLV
copy. Python uses `extensions.client_batch_item_login(...)` and
`client.LoginResponse`; `ticket_value` returns caller-owned immutable `bytes`,
while `close()` wipes KMIPKit-owned mutable storage and closes its Ticket TTLV
copy. Python callers and runtimes cannot deterministically erase returned
immutable copies. Java and Python redact the Ticket Value in formatted output.
The C Ticket Value span is borrowed and must not be logged. KMIPKit does not
claim full operation parity, a supported profile, or formal certification
from this snapshot.

For the authoritative operation owners and per-capability mappings, see the
[manifest](../../../specification/api/public-api.json) and the [public API
design](../../architecture/public-api.md). The overall project remains
unreleased.
