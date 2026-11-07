# Java and Python packages

## Rust facade

Rust applications can declare `kmipkit` as their only direct KMIPKit
dependency. The facade exposes the complete typed protocol namespace as
`kmipkit::protocol`, generic TTLV values and bounded decoding as
`kmipkit::ttlv`, and the client and TLS configuration types needed to create
the supported client. Existing root-level Rust imports remain available.

```toml
[dependencies]
kmipkit = "1.0"
```

```rust
use kmipkit::protocol::{EncryptRequest, GetRequest, PollRequest};
use kmipkit::ttlv::codec::{decode_with_limits, CodecLimits};
use kmipkit::ttlv::{Item, Value};
use kmipkit::{Client, ClientBatch, ClientBatchResponse, ClientError};
use kmipkit::client_config::{client_extension_registry, ClientConfiguration};
use kmipkit::transport_config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};

fn request_types(_: Option<(GetRequest, EncryptRequest, PollRequest)>) {}

fn build_client(
    certificate_pem: Vec<u8>,
    private_key_pem: Vec<u8>,
) -> Result<Client, Box<dyn std::error::Error>> {
    let registry = client_extension_registry(
        Vec::new(),
        kmipkit::protocol::extension::defaults(),
    )?;
    let client_configuration = ClientConfiguration::new(registry);
    let transport = TransportConfig::builder(Endpoint::raw_tls("localhost", 5696))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem(certificate_pem),
            PrivateKeyInput::from_pem(private_key_pem),
        ))
        .trust_source(TrustSource::platform())
        .build()?;
    Ok(Client::new(client_configuration, transport)?)
}

fn inspect_ttlv(encoded_ttlv: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let limits = CodecLimits::defaults();
    let _decoded = decode_with_limits(encoded_ttlv, &limits);
    let _value = Item::new(
        kmipkit::ttlv::RawTag::new(0x0042_0077)?.try_checked()?,
        Value::integer(1),
    );
    Ok(())
}

fn execute(
    client: &mut Client,
    batch: ClientBatch,
) -> Result<ClientBatchResponse, ClientError> {
    let limits = CodecLimits::defaults();
    client.execute(batch, &limits)
}
```

`kmipkit::ttlv` provides the generic value model and bounded decoder. It does
not expose a general-purpose outbound encoder; requests are encoded through
the typed client execution API. The facade omits raw transport exchange and
concrete low-level adapters.

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

## Named KMIP protocol values

The catalog-backed values are available from each language package. For
example, Rust uses the typed protocol namespace:

```rust
use kmipkit::protocol::named_values_generated::values::enumeration::cryptographic_algorithm;

let algorithm = cryptographic_algorithm::AES;
let name = cryptographic_algorithm::name(algorithm);
assert_eq!(name, Some("AES"));
```

C exposes fixed-width constants and lookup helpers in `kmipkit.h`:

```c
uint32_t algorithm = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES;
const char *name = KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_name(algorithm);
```

Java and Python expose the same catalog values through their protocol-value
namespace:

```java
long algorithm = KmipValues.CryptographicAlgorithm.AES;
String name = KmipValues.CryptographicAlgorithm.name(algorithm);
```

```python
from kmipkit import protocol_values

algorithm = protocol_values.CryptographicAlgorithm.AES
name = protocol_values.CryptographicAlgorithm.name(algorithm)
```

The generated set contains assigned values only. Unknown raw Enumeration
values and unassigned Bit Mask bits remain available; lookup helpers report
known names without changing the caller's raw value. A named value does not
claim that a server supports it or that an algorithm choice is appropriate.

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
| `public_binding_surface` | verified | T108/T109 and T011 confirm every observed declaration has a manifest record or an exact approved exclusion. The inventory is reconciled; operation mappings outside this verified inventory remain incomplete and blocked APIs stay unavailable. The verified inventory does not claim KMIP conformance or release readiness. |
| `vendor_extension_registry` | declared | Exact shared mappings include Rust drop, C release, Java close, and Python close for native handles; parity verification remains pending. |
| `discover_versions_request_item` | declared | Four-language request-item builders are mapped; complete Discover Versions operation conformance remains pending. |
| `client_batch_model` | declared | The four-language client batch model exposes typed ordered requests and results; full operation coverage remains incomplete. |
| `client_execution` | declared | Discover Versions request and response mappings are present in Rust, C, Java, and Python; complete 1.0 operation conformance remains pending. |
| `wire_encoding_selection` | declared | TTLV, JSON, and XML defaults and per-exchange selection are mapped across Rust, C, Java, and Python. |
| `client_asynchronous_operations` | declared | Cancel, Poll, Process, and Query Asynchronous Requests use explicit one-exchange calls across four languages; exact correlation bytes and the generic Query TTLV response are preserved, with no automatic polling. |
| `crypto_response_ttlv` | declared | The separate FR-015 error-response access remains limited to its four approved accessors; typed success-response TTLV is recorded on the five operation rows. |
| `production_transport` | declared | Rust and the three adapters expose the approved TLS 1.3 client transport configuration; interoperability evidence remains pending. |
| `api_level_discovery` | declared | Rust, C, Java, and Python expose the local API-level taxonomy without network access. |
| `opaque_register_context` | declared | Explicit Opaque Object context and Register request-shape validation are mapped across four languages; this separate capability remains declared. The Register operation row is verified. |
| `server_generated_creation` | declared | Server-generated identifiers, result metadata, and unique-identifier representations are mapped across four languages; complete operation conformance remains pending. |
| `usage_allocation_guard` | declared | A caller-owned local Usage Allocation context guards Encrypt, MAC, and Sign execution across four languages; it does not enforce server policy. See [policy controls](policy-controls.md). |
| `client_profile_validation` | declared | Profile-validation entry points and typed validation errors map across four languages; this mapping does not claim Baseline profile support. |
| `client_profile_readiness` | declared | Profile readiness evaluation reports evidence blockers across the four languages; it does not claim Baseline profile conformance. |
| `credential_type_parity` | declared | Credential Type values, including unknown values, map across four languages; complete parity verification remains pending. |
| `typed_message_model` | declared | Typed request/response message models and extension views map across four languages; complete parity verification remains pending. |
| `nonce_ttlv_parity` | declared | Nonce TTLV conversion and lifecycle operations map across four languages; complete parity verification remains pending. |
| `credential_ttlv_parity` | declared | Typed credential and authentication TTLV models map across four languages; complete parity verification remains pending. |
| `ffi_shared_owner_status` | declared | C exposes the `KMIPKIT_ERROR_SHARED_OWNER` status; this capability is C-only. |
| `credential_validation_error_parity` | declared | Typed credential-validation error kinds map across four languages; complete parity verification remains pending. |
| `credential_validation_error_abi_layout` | declared | C maps the versioned credential-validation error ABI layout fields; this capability is C-only. |
| `named_protocol_values` | declared | Assigned KMIP 2.1 Enumeration and Bit Mask values have catalog-backed names in Rust, C, Java, and Python. This maps names and lookups; it does not claim server support or cryptographic suitability. |
| `hashed_password_timestamp_sequence` | declared | A locally owned timestamp sequence for Hashed Password credentials is mapped across the four languages; server validation remains authoritative. |
| `credential_secret_parity` | declared | Secret byte and text wrappers map across four languages; complete parity verification remains pending. |

Of the **56** KMIP `client_1_0` operation rows, **0 / 56** remain `blocked`.
**12** rows are `declared`: Cancel, Create, Create Key Pair, Create Split Key,
Hash, MAC, MAC Verify, Poll, Process, Query Asynchronous Requests, Sign, and
Signature Verify. The asynchronous APIs follow the approved KMIPKIT-0009 scope;
Process Table 278's catalog requirement gap and reviewer-owned checklists remain
open. Query returns a generic TTLV response under KMIPKIT-DEC-015. `declared`
records a mapping and does not establish complete operation conformance. Do not
infer operation availability from types or implementation code elsewhere in
the repository.
**44** operation mappings are `verified`:
Activate, Add Attribute, Adjust Attribute, Archive, Certify, Check, Decrypt,
Delegated Login, Delete Attribute, Derive Key, Destroy, Discover Versions,
Encrypt, Export, Get, Get Attribute List, Get Attributes, Get Constraints,
Get Usage Allocation, Import, Interop, Join Split Key, Locate, Log, Login,
Logout, Modify Attribute, Obtain Lease, PKCS#11, Ping, Query, Recover,
Register, Revoke, Re-certify, Re-key, Re-key Key Pair, Re-Provision,
RNG Retrieve, RNG Seed, Set Attribute, Set Constraints, Set Defaults, and
Validate. These are API mappings, not full operation-conformance or server-interoperability claims. Logout sends the caller-supplied
Ticket and does not add credential data to later requests or log out automatically;
see [credential sessions](credential-sessions.md). The Derive Key mapping rejects a successful result without its
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
four languages. Ping maps the empty request and successful response payloads and
does not imply general server health. Query preserves caller-supplied function
and Object Group order and unknown response items; it tolerates both response
shapes described in §6.1.40 while `KMIPKIT-DISC-047` remains unresolved. These
are API mappings, not server-conformance claims. Locate preserves optional request filters, server order,
repeated Unique Identifiers, and the distinction between omitted and empty
Located Items. The Get Usage Allocation mapping includes its optional identifier
and signed count request, resolved identifier response, and common batch outcomes
in Rust, C, Java, and Python. This mapping evidence does not claim full operation
conformance. Re-certify maps the Table 300 request fields: optional Unique
Identifier and Certificate Request Unique Identifier, Certificate Request
Type (required when Certificate Request Value is present), Certificate Request
Value, Offset, Attributes, and Protection Storage Masks. Successful results
expose the replacement identifier and complete ordered response TTLV across
Rust, C, Java, and Python. Re-key maps Table 305's optional Unique Identifier,
Offset, Attributes, and Protection Storage Masks; the caller should send it no
more than once for a given key. Successful Table 306 results expose the
replacement Unique Identifier and complete ordered response TTLV, including
unknown fields. C returns a caller-owned TTLV copy; Java and Python return an
owned response that callers close. Failure and Pending retain common result and
delivery metadata without success fields, raw error bodies, or automatic
retries. This is an API mapping, not a conformance claim. Re-key Key Pair
maps §6.1.47 Tables 308–312: its request may carry the Private Key Unique
Identifier, Offset, three distinct Attributes groups, and three distinct
Protection Storage Masks groups. Java uses
`ClientBatchItem.rekeyKeyPair(...)`; Python uses
`extensions.client_batch_item_rekey_key_pair(...)`. C uses
`kmipkit_client_batch_item_rekey_key_pair`; it copies each supplied structure
before returning. An omitted request identifier remains a server-side ID
Placeholder and is not synthesized locally. On Success, the typed response
exposes both replacement identifiers and the complete ordered response TTLV,
including unknown fields. Java and Python callers close the owned response;
C's typed text spans are borrowed until the response item is released, while
`kmipkit_client_batch_item_response_rekey_key_pair_ttlv` returns an independent
caller-owned copy. Failure and Pending retain shared status, reason,
correlation, and delivery metadata without successful pair fields or automatic
follow-up. This mapping does not claim conformance, interoperability, profile
support, or certification. Re-Provision maps §6.1.48 Tables 313–315. Java uses
`ClientBatchItem.reprovision(...)`; Python uses
`extensions.client_batch_item_reprovision(...)`; and C uses
`kmipkit_client_batch_item_reprovision`. The optional Certificate Request and
Certificate are mutually exclusive, and omitting both leaves credential
creation to the server. A successful result exposes the optional Unique
Identifier and complete ordered response TTLV; Java and Python callers close
the owned response, while C's identifier span is borrowed until the response
item is released and its TTLV accessor returns a caller-owned copy. Failure and
Pending expose only common result metadata, without success fields, raw bodies,
or automatic follow-up. This is API mapping evidence, not a conformance claim.
Log preserves its request message and exposes the complete response
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
from this snapshot. Obtain Lease maps its successful response fields and
retains the complete ordered response TTLV in all four languages. C returns an
owned TTLV structure; Java exposes `ObtainLeaseResponse.responseTtlv()` and
Python exposes `ObtainLeaseResponse.response_ttlv`. Callers close the owned
result when done. KMIPKit does not track lease expiration or start a timer.

For Certify, Obtain Lease, Re-certify, Re-key, Re-key Key Pair, Re-Provision,
Register, and Revoke, generic response TTLV is available only on Success.
Rust's `with_ttlv` returns `None` for Failure and Pending; C, Java, and Python
likewise omit the typed response copy. The common result status, reason, and
message remain available on the batch result, along with Pending correlation
metadata. Check retains its specified policy-denial fields.

Successful Hash, MAC, MAC Verify, Sign, and Signature Verify results also expose
their complete ordered generic response TTLV. Rust lends a `StructureView`; C
returns a caller-owned `kmipkit_ttlv_structure_t` that must be released with
`kmipkit_ttlv_structure_release`; Java `responseTtlv()` and Python
`response_ttlv` return the owned `TtlvStructure` or `ttlv.Structure`, which the
caller closes when finished. These copies preserve unknown fields and their
order. A non-success result or a different operation has no success-response
TTLV accessor. This does not broaden the separate FR-015 error-response access.

Encrypt and Decrypt are also mapped in all four languages. Java callers create
items with `ClientBatchItem.encrypt(...)` and `ClientBatchItem.decrypt(...)`;
Python callers use `extensions.client_batch_item_encrypt(...)` and
`extensions.client_batch_item_decrypt(...)`. On a successful result, Java
exposes `encryptResponse()` or `decryptResponse()`, and Python exposes the
corresponding `encrypt_response` or `decrypt_response` field. Encrypt preserves
the required Unique Identifier and each present Data, IV/Counter/Nonce,
Correlation Value, and Authenticated Encryption Tag; Decrypt preserves its
required identifier and present Data and Correlation Value. These operations
send cryptographic work to the KMIP server; KMIPKit does not encrypt or decrypt
locally. Java response accessors return caller-owned `byte[]` copies that the
caller should protect and clear. Python returns immutable `bytes`; callers and
the Python runtime cannot deterministically erase those copies. KMIPKit redacts
these response values from Java `toString()` and Python `repr()` output. Close
each Java `EncryptResponse` or `DecryptResponse`, and call `close()` on each
Python response, after use to wipe the mutable bytes retained by KMIPKit; closing
the batch item does not close the separately returned typed response.

For the authoritative operation owners and per-capability mappings, see the
[manifest](../../../specification/api/public-api.json) and the [public API
design](../../architecture/public-api.md). The overall project remains
unreleased.
