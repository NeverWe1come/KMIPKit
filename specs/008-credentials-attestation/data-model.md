# Data Model: KMIPKIT-0008 Credentials and Attestation

## Credential

An ordered protocol value containing `Credential Type` (raw 32-bit Enumeration) and a value represented according to that type. Required outer fields and field order follow Specification §9.11, Table 410.

| Type | Member | TTLV type | Required | Catalog element ID |
|---|---|---|---|---|
| All Credential Types (Table 410) | Credential Type | Enumeration | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-TYPE` |
| All Credential Types (Table 410) | Credential Value | Varies by Credential Type | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-VALUE` |
| Username and Password (1), Table 411 | Username | Text String | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-USERNAME` |
| Username and Password (1), Table 411 | Password | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-USERNAME-AND-PASSWORD-PASSWORD` |
| Device (2), Table 412 | Device Serial Number | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-SERIAL-NUMBER` |
| Device (2), Table 412 | Password | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-PASSWORD` |
| Device (2), Table 412 | Device Identifier | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-DEVICE-IDENTIFIER` |
| Device (2), Table 412 | Network Identifier | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-NETWORK-IDENTIFIER` |
| Device (2), Table 412 | Machine Identifier | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MACHINE-IDENTIFIER` |
| Device (2), Table 412 | Media Identifier | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-DEVICE-MEDIA-IDENTIFIER` |
| Attestation (3), Table 413 | Nonce | Structure | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-NONCE` |
| Attestation (3), Table 413 | Attestation Type | Enumeration | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-TYPE` |
| Attestation (3), Table 413 | Attestation Measurement | Byte String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-MEASUREMENT` |
| Attestation (3), Table 413 | Attestation Assertion | Byte String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ATTESTATION-ATTESTATION-ASSERTION` |
| One Time Password (4), Table 414 | Username | Text String | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-USERNAME` |
| One Time Password (4), Table 414 | Password | Text String | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-PASSWORD` |
| One Time Password (4), Table 414 | One Time Password | Text String | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-ONE-TIME-PASSWORD-ONE-TIME-PASSWORD` |
| Hashed Password (5), Table 415 | Username | Text String | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-USERNAME` |
| Hashed Password (5), Table 415 | Timestamp | Date Time Extended | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-TIMESTAMP` |
| Hashed Password (5), Table 415 | Hashing Algorithm | Enumeration | No | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHING-ALGORITHM` |
| Hashed Password (5), Table 415 | Hashed Password | Byte String | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-HASHED-PASSWORD` |
| Ticket (6), Table 416 | Ticket | Structure | Yes | `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-TICKET-TICKET` |

Extensions (`8XXXXXXX`) and unknown Credential Type values use opaque generic TTLV; no member schema is inferred. All rows above retain their catalog stable IDs and are grounded in Specification §9.11, Tables 410–416.

Unknown children within a known structure remain available through the retained generic TTLV representation. Typed construction emits the canonical field order. Raw enum values are retained for Credential Type, Hashing Algorithm, and Attestation Type. The seven stable Credential Type value IDs from §11.11/Table 442 are `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006`, and `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-EXTENSIONS-8XXXXXXX`.

## Authentication

An optional message-header structure (Specification §9.4, Table 403) containing a non-empty, ordered, repeatable list of Credentials. It is absent or contains one or more entries; it is never present-empty. The model does not determine whether a server accepts the entries or whether multiple entries are satisfied.

Selection for an outgoing operation can inherit client defaults or replace them at request/batch scope per `docs/architecture/transport-security.md`. The exact representation, explicit omission behavior, and precedence at the client boundary are gated by OD-005 and KMIPKIT-0007.

## Nonce

Server-produced attestation input (Specification §9.14, Table 419): `Nonce ID` and `Nonce Value`, both byte strings. Preserve exact bytes and ownership. The client does not generate a Nonce. Attestation Credential references the server-sourced Nonce structure per §9.11/Table 413.

## Secret Value

Caller-owned sensitive data wrapped so KMIPKit formatting never exposes contents. When KMIPKit owns a copy, drop/consumption semantics zeroize its initialized bytes before owner deallocation, consistent with `docs/architecture/public-api.md` §Secrets. This does not guarantee erasure of spare or uninitialized capacity, buffers left by pre-transfer reallocations, copies held by callers or dependencies, borrowed-view copies, temporary stack/register copies, or foreign-runtime copies; capacity is covered only when initialized and cleanup is verified. Likely secret fields include Password, One Time Password, Hashed Password, Ticket contents, Nonce Value, attestation evidence, and any opaque Credential value. Username and device identifiers are redacted with the containing Credential to avoid diagnostic leaks.

Java/Python copies are outside Rust's zeroization guarantee; bindings must document their runtime limits before release.

## Attestation capability

The indicator is a Boolean property of the shipped public API: true only if a caller can construct a structurally valid Attestation Credential using caller-provided Nonce and data; false otherwise. Omitted has effective false per §9.3/Table 402. This property does not mean KMIPKit generates, verifies, or can satisfy server attestation evidence requirements.

## Source constraints

- Device uniqueness cannot be established by a client without an external registry. The four named fields in §9.11 prose and the separate `Device Identifier` in Table 412 require OD-002 disposition before an at-least-one validator is fixed.
- Hashed Password monotonic Timestamp state and comparison scope require OD-003 disposition. Timestamp pass-through alone is not compliance evidence.
- This feature is in-memory only and never adds a production credential send path, regardless of OD-004. Any future send path belongs to a separate approved feature that owns the candidate callsite and its owner-through-transport lifecycle test.
