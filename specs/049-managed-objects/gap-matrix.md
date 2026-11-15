# KMIPKIT-0049 Existing-Capability Gap Matrix

**Source of truth**: pinned KMIP Specification v2.1, referenced as
`KMIPKIT-SRC-spec` in `specification/catalog/kmip-2.1.json`.

## Existing foundation

| Area | Existing behavior | Gap for 0049 |
|---|---|---|
| Generic TTLV | `Item`, `Structure`, and `Value` retain order, repeats, unknown allocated tags, and nested data; Debug is metadata-only; owned payloads recursively zeroize on drop. | Callers still assemble every object and nested structure manually. |
| Object type | `ObjectType` preserves raw Enumeration values; 0047 adds named values. | No typed mapping from object type to its §2 object Structure. |
| Key metadata | Raw-preserving `KeyFormatType`, `KeyWrapType`, and `KeyCompressionType` exist. | No composable Key Block, Key Value, Key Material, or wrapping-data models. |
| Parsed typed-model updates | `Item` and `Structure` preserve order, repeats, and unknown descendants. | Earlier `KeyBlock`/`KeyWrappingData` readers returned the retained source tree even after typed setters were called; new-builder `with_raw()` also failed. FR-011 now specifies local mutation and inspection semantics, including preservation of untouched extension children. |
| Object operations | RegisterRequest and ImportRequest accept generic Any Object Items; GetResponse retains a generic Any Object. Register/Import perform existing outer-shape/cardinality checks. | No typed object constructor can feed Register/Import; GetResponse typed object integration is deferred to 0052. |
| Attributes | 0048 provides typed direct attributes, typed `AttributeSet`, and generic fallback. | No KeyValue Attributes composition helper; raw Structure must be assembled manually. |
| Catalog/source | Catalog has records for all 9 object structures, their table members, key formats, and source requirement IDs; OASIS source is pinned and immutable. | Records are not yet bound to typed object models, constructors, or per-schema tests. |

## Object Structures (§§2.1–2.9, Tables 2–10)

| OASIS object | Structure tag | Required/conditional fields from source | Existing mapping | 0049 gap |
|---|---|---|---|---|
| Certificate (§2.1, Table 2) | `0x00420013` | Certificate Type `Enumeration`; Certificate Value `Byte String`; both required. | Generic Item only. | Typed owner/view and canonical constructor. |
| Certificate Request (§2.2, Table 3) | `0x00420018` | Certificate Request Type `Enumeration`; Certificate Request Value `Byte String`; both required. | Generic Item only. | Typed owner/view and canonical constructor. |
| Opaque Object (§2.3, Table 4) | `0x0042005B` | Opaque Data Type `Enumeration`; Opaque Data Value `Byte String`; both required. | Register has separate local OpaqueObjectContext; payload remains generic. | Typed object fields without changing the out-of-band context policy. |
| PGP Key (§2.4, Table 5) | `0x004200BD` | PGP Key Version `Integer`; Key Block; both required. Public-only or public/private block representations are preserved. | Generic Item only. | Typed wrapper; no mandatory PGP decomposition. |
| Private Key (§2.5, Table 6) | `0x00420064` | Key Block required. | Generic Item only. | Typed wrapper. |
| Public Key (§2.6, Table 7) | `0x0042006D` | Key Block required. | Generic Item only. | Typed wrapper. |
| Secret Data (§2.7, Table 8) | `0x00420085` | Secret Data Type `Enumeration`; Key Block; both required; wrapped Key Value is permitted. | Generic Item only. | Typed wrapper and ownership semantics. |
| Split Key (§2.8, Table 9) | `0x00420089` | Key Block, Key Part Identifier, Split Key Method, Split Key Parts, Split Key Threshold required. Prime Field Size required only for Polynomial Sharing Prime Field. Key Part Identifier is 1..=Split Key Parts. | Generic Item only. | Typed wrapper and conditional fields. |
| Symmetric Key (§2.9, Table 10) | `0x0042008F` | Key Block required. | Generic Item only. | Typed wrapper. |

## Nested Key Structures (§§3.1–3.12, Tables 12–26)

| Structure | Source-defined shape | Existing mapping | 0049 gap |
|---|---|---|---|
| Key Block (§3.1, Table 12) | Key Format Type required; Key Value is optional and is Structure when plaintext or Byte String when wrapped; Key Compression Type optional; Cryptographic Algorithm/Length are conditionally optional and must appear together when present; Key Wrapping Data is present only for wrapped keys. | Raw format/wrap/compression enums and generic Structure. | Typed paired fields and safe distinction of plaintext/wrapped values. |
| Key Value (§3.2, Table 13) | Key Material required; optional Attributes Structure. Key Material encoding depends on Key Format Type and may be an extension format. | 0048 AttributeSet and generic Item exist separately. | Typed composition, raw-format fallback, and order-preserving conversion. |
| Key Wrapping Data (§3.3, Table 14) | Wrapping Method required; Encryption Key Information and MAC/Signature Key Information optional; at least one of those information structures is supplied when the wrapping schema requires it; optional MAC/Signature, IV/Counter/Nonce, and Encoding Option. | Generic Structure and operation-level KeyWrapType only. | Typed metadata structures; no cryptographic wrapping operation. |
| Encryption/MAC Signature Key Information (§3.3, Tables 15–17) | Unique Identifier required; optional Cryptographic Parameters. MAC/signature information identifies the applicable key. | Generic Structure. | Typed fields and raw fallback. |
| Transparent Symmetric Key (§3.4, Table 18) | Required Key Byte String. | Generic Structure. | Typed key material. |
| Transparent DSA Private/Public (§§3.5–3.6, Tables 19–20) | DSA P/Q/G/X/Y Big Integer members with source-defined requiredness. | Generic Structure. | Typed material and field-presence rules. |
| Transparent RSA Private/Public (§§3.7–3.8, Tables 21–22) | Modulus and public exponent; private components optional subject to at least one §3.7 permitted representation. | Generic Structure. | Typed material and RSA alternative checks. |
| Transparent DH Private/Public (§§3.9–3.10, Tables 23–24) | P/G/X or P/G/Y required; Q/J optional as defined. | Generic Structure. | Typed material and source-defined fields. |
| Transparent EC Private/Public (§§3.11–3.12, Tables 25–26) | Private D plus Recommended Curve; public Q String plus Recommended Curve. | Generic Structure. | Typed material and source-defined fields. |

## Normative boundaries and preserved behavior

- Field order comes from each defining table and KMIP 2.1 §10.1.2; it is not
  inferred from numeric tag values or builder call order.
- The structure tag is taken from the pinned tag catalog; it is distinct from
  the Object Type Enumeration value. Register and Import carry the typed
  object-specific tagged Item directly, with no additional wrapper.
- The source catalog's `source_requiredness`, `source_encoding`, `parent_element_ids`,
  requirement IDs, and any linked test-case IDs are recorded in
  `traceability.md` before implementation.
- Unknown Object Type, Key Format Type, child Items, and vendor tags stay
  representable through generic TTLV. Typed convenience must not narrow that
  fallback.
- This feature does not move response parsing into object models. 0052 adds
  typed response readers and `.raw()` navigation.
- Existing `Value` ownership already recursively zeroizes payloads. Typed
  models must reuse it and document caller/external copies as out of scope.

## Approved KMIP 3.0 system credential object addendum

| System object | Object Type / Structure tag | Source fields | Gap at the start of this addendum |
|---|---|---|---|
| Password Credential | `0x0D` / `0x004201A1` | §2.1.3.1, Table 5; attributes Table 6 | No typed object reader or constructor; the Password Link alternative is carried in Attributes and cannot be decided from the object tree alone. |
| Device Credential | `0x0E` / `0x004201A6` | §2.1.3.2, Tables 7–8 | No typed model or at-least-one-field validation. |
| One Time Password Credential | `0x0F` / `0x004201A7` | §2.1.3.3, Tables 9–10 | No typed model or required OTP Algorithm validation. |
| Hashed Password Credential | `0x10` / `0x004201AF` | §2.1.3.4, Tables 11–12 | No typed model or required pair of hash-field validation. |

The assigned tags and Object Type values are taken from the pinned KMIP 3.0
§11.35, Table 582, and §11.58, Table 605; numeric ranges do not imply an
assignment. The approved contract is limited to lossless typed structure,
source-defined local checks, redaction, and zeroization. Version legality and
cross-structure Attribute Set checks remain with KMIPKIT-0051.
