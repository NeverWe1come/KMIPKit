# Feature Specification: Typed Managed Objects and Key Material

**Feature Branch**: `feature/KMIPKIT-0049-managed-objects-release`
**Created**: 2026-11-05
**Status**: Approved by maintainer plan authorization
**Input**: Approved KMIPKit Phase 5 plan, item 0049: typed construction and reading for all nine KMIP 2.1 managed-object types, key structures, wrapping structures, explicit secret ownership and cleanup, and the four KMIP 3.0 system credential objects authorized on 2026-11-08.

## Purpose

Let Rust client authors construct and inspect KMIP 2.1 managed objects, KMIP
3.0 system credential objects, and key material through named, lossless types.
Preserve generic TTLV access for future and vendor extensions, and make the
lifetime of KMIPKit-owned secret material explicit.

## User Scenarios & Testing

### User Story 1 - Construct and inspect managed objects (Priority: P1)

As a Rust client author, I can create and inspect the nine object structures
defined by KMIP 2.1 without manually assembling their outer TTLV fields.

**Independent Test**: For each object type, construct the typed value, encode
it to the canonical source-defined field order, decode it, and inspect the
same typed values and raw tree.

**Acceptance Scenarios**:

1. Given any of Certificate, Certificate Request, Opaque Object, PGP Key,
   Private Key, Public Key, Secret Data, Split Key, or Symmetric Key, when a
   caller constructs it, then it becomes the single object-specific Structure
   Item supplied directly to Register or Import, using the structure tag and
   fields from KMIP 2.1 §§2.1–2.9, Tables 2–10. No extra typed wrapper is
   emitted on the wire.
2. Given a generic Any Object with an unknown Object Type or extension fields,
   when a caller uses the generic path or the typed unknown fallback, then its
   values, descendant order, repetitions, and tags remain available unchanged.
3. Given a PGP Key, when a caller provides either supported public-only or
   public-and-private Key Block representation, then KMIPKit preserves the
   caller's key data and does not parse, decompose, or cryptographically
   transform it.

### User Story 2 - Build key material and wrapping structures (Priority: P1)

As a Rust client author, I can construct Key Block, Key Value, key material,
and wrapping metadata with types that distinguish plaintext structured
material from wrapped bytes.

**Independent Test**: Build representative raw, transparent, and wrapped key
blocks; verify required and conditional fields, exact ordering, lossless
round-trip, and payload-free errors.

**Acceptance Scenarios**:

1. Given a plaintext Key Block, when its Key Value is encoded, then the Key
   Value is a Structure containing required Key Material and optional
   Attributes as defined by §§3.1–3.2, Tables 12–13.
2. Given a wrapped Key Block, when it is encoded, then Key Value is represented
   as a Byte String and the supplied Key Wrapping Data is preserved; KMIPKit
   does not perform encryption, decryption, signing, or MAC operations. The
   typed wrapped value owns its bytes through `SecretBytes` immediately, so
   dropping it before transfer to a Key Block still zeroizes KMIPKit-owned
   storage.
3. Given a transparent key format, when its material is constructed or read,
   then the typed fields match the definitions in §§3.4–3.12, Tables 18–26.
   Raw, Opaque, PKCS1, PKCS8, ECPrivateKey, and extension formats remain
   representable using the source-defined Byte String or Structure form.
4. Given optional or conditionally required material fields, when the caller
   builds the structure, then KMIPKit preserves valid caller choices and
   reports structural errors without embedding key bytes in diagnostics.
5. Given a parsed Key Block or Key Wrapping Data value, when a caller changes
   a typed field and converts it back to `Item`, then the emitted tree reflects
   that change while retaining untouched unknown fields, their values, and
   their relative order. `with_raw()` on a builder exposes its current
   (possibly incomplete) structure; final conversion continues to validate it.

### User Story 3 - Transfer and release managed material safely (Priority: P1)

As an application developer, I can pass typed objects to existing object
operations and release KMIPKit-owned key material without it appearing in
formatting or errors.

**Independent Test**: Pass a typed object through Register or Import using the
fake transport, inspect the emitted TTLV tree, and verify formatting
redaction and owned-payload cleanup behavior.

**Acceptance Scenarios**:

1. Given an existing RegisterRequest or ImportRequest, when the caller uses a
   typed managed object, then it emits the same operation fields and remains
   compatible with the existing generic Item constructors.
2. Given an owned typed key value, when its KMIPKit-owned owner is dropped,
   then its secret payload storage is zeroized using the existing TTLV
   ownership contract.
3. Given a debug or error representation of a secret-bearing object, when it
   is formatted, then it contains structural metadata only and no secret
   bytes or raw KMIP body.

### User Story 4 - Construct and inspect KMIP 3.0 credential system objects
(Priority: P1)

As a Rust client author, I can construct and read the four credential system
objects from named fields while keeping passwords, identifiers, OTP seeds, and
hash material redacted and zeroizing.

**Independent Test**: Build or parse each of the four object structures,
exercise its required and conditional fields, and verify exact Object Type and
Structure tag mapping, field order, unknown-child preservation, and redacted
formatting.

**Acceptance Scenarios**:

1. Given a Password Credential, when it is built or read, then KMIPKit enforces
   Table 5 field types and order and rejects a structure that includes both
   Password and Salted Password or a Salted Password without its algorithm.
   Password Link remains an Attribute and is checked with the Attribute Set.
2. Given a Device Credential, when it is built or read, then at least one
   Table 7 identifier is present; uniqueness remains the server's decision.
3. Given an OTP Credential, when it is built or read, then OTP Algorithm is
   required and all optional fields use Table 9 types and order.
4. Given a Hashed Password Credential, when it is built or read, then both
   Table 11 hash fields are required and the optional algorithm and unknown
   values remain lossless; an absent algorithm reports the source-defined
   SHA-256 default without inserting a field. KMIPKit does not compute either
   hash.
5. Given any of the four structures, when it contains an unknown extension or
   vendor child, then conversion retains its tag, value, and relative order.
6. Given a credential object with secret text or bytes, when it is formatted or
   dropped, then formatting omits the secret and KMIPKit-owned payload storage
   is zeroized.

### Edge Cases

- The `Object Type` and `Key Format Type` are open numeric values. Unknown and
  extension values remain usable through a generic fallback.
- `Key Value` in `Key Block` can be absent, a plaintext Structure, or wrapped
  Byte String according to the general source schema. A Secret Data object
  specifically requires its Key Block to contain a Key Value under §2.7.
- `Key Material` encoding is selected explicitly by the caller's Key Format
  Type. KMIPKit does not infer algorithm, size, usage, or protection policy.
- A Split Key's Prime Field Size is required only for Polynomial Sharing
  Prime Field; Key Part Identifier must be from 1 through Split Key Parts.
  KMIP 2.1 §2.8 does not state numeric bounds for Split Key Threshold, so the
  model preserves that integer without imposing an additional local range.
- Builders follow source table order even where it is not intuitive: Table 9
  places Key Block after Split Key Parts, Key Part Identifier, Split Key
  Threshold, Split Key Method, and optional Prime Field Size; Table 12 places
  Key Compression Type before Key Value.
- Transparent RSA private material permits the alternative field sets
  specified by §3.7; the model preserves whichever supported form the caller
  supplies.
- Key Wrapping Data preserves the supplied wrapping method, key identifiers,
  cryptographic parameters, MAC/signature, IV/counter/nonce, and encoding
  option. It does not infer server-side key capabilities.
- Unknown tags, vendor extensions, repeated children, and original order are
  preserved in the generic representation.
- KMIPKit zeroizes only payload copies it owns; source buffers, copies made by
  callers, external runtimes, and TLS dependencies remain outside that
  guarantee.

## Requirements

### Functional Requirements

- **KMIPKIT-0049-FR-001**: The Rust protocol API MUST provide typed construction
  and borrowed inspection for all nine managed-object structures in KMIP 2.1
  §§2.1–2.9, Tables 2–10, with the exact object-specific Item tag, TTLV types,
  requiredness, and schema order. The typed Object Type and structure tag MUST
  remain consistently associated; the structure Item is supplied directly as
  the Register/Import object and no extra typed wrapper is encoded.
- **KMIPKIT-0049-FR-002**: The Rust protocol API MUST provide typed Key Block,
  Key Value, and Key Wrapping Data representations, including the Encryption
  Key Information and MAC/Signature Key Information child Structures in
  §3.3, Tables 12–17.
- **KMIPKIT-0049-FR-003**: Key Material MUST cover the Byte String and Structure
  forms defined by Key Format Type and all transparent material Structures in
  §§3.4–3.12, Tables 18–26. The assigned/reserved mapping MUST match §11.25,
  Table 461 exactly, including reserved values `0x0E`–`0x13` and assigned
  formats `0x14`–`0x17`. Unknown and extension key formats MUST remain
  representable without narrowing the generic TTLV path.
- **KMIPKIT-0049-FR-004**: Builders MUST emit each known Structure's fields in
  KMIP 2.1 §10.1.2 order regardless of setter call order. They MUST retain
  source-defined optionality and conditional requirements, including wrapped
  Key Value representation, Split Key requirements, and Transparent RSA
  private-key alternatives.
- **KMIPKIT-0049-FR-005**: Typed models MUST convert to and from the existing
  generic `Item`/`Structure` representation without losing unknown tags,
  extension fields, repetitions, or descendant order. Existing public generic
  APIs and operation constructors MUST remain source-compatible.
- **KMIPKIT-0049-FR-006**: RegisterRequest and ImportRequest MUST accept typed
  managed objects through additive APIs while preserving their existing
  generic constructors, operation behavior, and delivery-state handling.
  GetResponse remains generically accessible; response-level typed readers
  are assigned to KMIPKIT-0052.
- **KMIPKIT-0049-FR-007**: Secret-bearing owned payloads MUST be redacted from
  `Debug` and error formatting and zeroized when their KMIPKit-owned storage
  is released. Borrowed views MUST not outlive their source owner. Documentation
  MUST describe copies outside KMIPKit's ownership boundary.
- **KMIPKIT-0049-FR-008**: The client MUST NOT perform cryptographic algorithms,
  wrapping, unwrapping, signing, MAC computation, format conversion, or
  implicit cryptographic-policy selection as part of these models.
- **KMIPKIT-0049-FR-009**: Every applicable normative requirement MUST have a
  stable traceability record linking its exact pinned OASIS section/table,
  feature requirement, implementation path, and executable test. Negative
  tests MUST cover applicable MUST NOT/SHALL NOT conditions.
- **KMIPKIT-0049-FR-010**: Rust documentation MUST include tested examples for
  plaintext, wrapped, transparent, generic-fallback, and secret-lifetime use.
  C, Java, and Python parity remains assigned to KMIPKIT-0053 and MUST NOT be
  claimed complete by this feature.
- **KMIPKIT-0049-FR-011**: Typed Key Block and Key Wrapping Data setters MUST
  affect conversion back to generic TTLV after parsing. Untouched unknown
  fields MUST retain their values and relative order. `with_raw()` MUST expose
  the current structure of both parsed models and builders, including
  incomplete builders; this inspection view MUST NOT bypass validation during
  final conversion. Clearing an optional field through a setter MUST remove
  the source field from both the inspection view and converted tree. This
  requirement defines local API behavior and does not claim that KMIP specifies
  a builder mutation model.
- **KMIPKIT-0049-FR-012**: The Rust protocol API MUST provide typed construction
  and borrowed reading for the Password Credential, Device Credential, One
  Time Password Credential, and Hashed Password Credential system objects
  defined by KMIP 3.0 §2.1.3, Tables 5–12, Object Type §11.35, Table 582, and
  tag assignments §11.58, Table 605. Models MUST preserve unknown children and
  extensions, enforce known field types, requiredness, order, and the local
  conditional rules stated in the source, and remain compatible with generic
  `Item`/`Structure` access. Credential attributes, including Password Link,
  are outside the object Structure; checks that require both the object and its
  Attribute Set are assigned to KMIPKIT-0051.
- **KMIPKIT-0049-FR-013**: KMIP-owned copies of system credential values MUST be
  redacted from formatting and errors and zeroized on release. Constructors
  MUST accept secret-bearing values through the existing secret owner types;
  borrowed readers MUST be callback-scoped. These models MUST NOT calculate,
  verify, or transform passwords, salts, OTPs, or hashes. Edition-specific
  acceptance is enforced by the request/version validation assigned to
  KMIPKIT-0051; this model layer preserves and validates structure only.
- **KMIPKIT-0049-FR-014**: A typed Secret Data object MUST contain its required
  Key Value within Key Block as specified by KMIP 2.1 §2.7 and Table 8; readers
  MUST reject an absent Key Value. Split Key validation MUST enforce the
  explicit Key Part Identifier bounds and Polynomial Sharing Prime Field
  condition from §2.8 and Table 9, but MUST NOT impose an unstated numeric
  bound on Split Key Threshold.

### Normative Traceability

The authoritative source is
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, identified as
`KMIPKIT-SRC-spec` in the pinned catalog. The per-record matrix is
`gap-matrix.md`; normative requirement IDs and test-case links are in
`traceability.md`. Scope includes §§2.1–2.9 (Tables 2–10), §§3.1–3.12
(Tables 12–26, including Encryption Key Information and MAC/Signature Key
Information), §10.1.2, and relevant Object Type, Key Format Type, Key
Compression Type, and Cryptographic Algorithm enumerations in §11. The
operation integration tests cite §6.1.25 (Import) and §6.1.43 (Register).
The approved KMIP 3.0 addendum also uses the pinned
`specification/oasis/kmip-3.0/upstream/kmip-spec-v3.0.html`: §§2.1.3.1–2.1.3.4,
Tables 5–12; §11.35, Table 582; and §11.58, Table 605. The OASIS Usage Guide is
informative and does not add requirements.

| Requirement | Normative source | Verification |
|---|---|---|
| FR-001 | §§2.1–2.9, Tables 2–10; Object Type §11.34; pinned tag allocations | Every object record, tag association, required/optional field, and direct Register/Import Item shape tested against catalog and pinned source |
| FR-002–004, FR-011 | §§3.1–3.3, Tables 12–17; field ordering §10.1.2; FR-011 is a local typed-API contract | Typed structure vectors, malformed-field negatives, exact-order and round-trip tests; parse-mutate-convert cases preserve extensions and builder views remain inspectable |
| FR-005–006 | §§6.1.25, 6.1.43 and generic TTLV §§5.1–5.4; API compatibility policy | Register/Import fake-transport tests; generic conversion property tests |
| FR-007–008 | AGENTS.md security boundary; TTLV ownership contract; §§3.1–3.3 wrapping schema | Redaction, zeroization, borrowed-lifetime, and no-crypto-operation tests |
| FR-009–010 | Constitution §§I–IV; pinned catalog and API manifest rules | Requirement-to-test audit, Rust consumer examples, manifest and binding-parity audit |
| FR-012–013 | KMIP 3.0 §§2.1.3.1–2.1.3.4, Tables 5–12; §11.35, Table 582; §11.58, Table 605 | Four typed system credential object schemas, required/conditional field tests, order, unknown-child preservation, redaction, and zeroizing owned values |

## Scope and Exclusions

- In scope: Rust models for all nine KMIP 2.1 object structures, Key Block,
  Key Value, Key Material, wrapping structures, additive Register/Import
  integration, and owned-secret handling. The directly approved 2026-11-08
  addendum also includes the four KMIP 3.0 system credential object Structures.
- Existing generic TTLV and operation APIs remain available and lossless.
- This feature does not add operations, response-specific object accessors,
  response `.raw()` APIs, or four-language typed parity; those are assigned
  to KMIPKIT-0052 and KMIPKIT-0053 respectively.
- This feature does not perform cryptographic operations, parse or validate
  certificate cryptography, decompose PGP keys, unwrap keys, infer cryptographic
  choices, or determine server policy/capabilities.
- The credential object layer does not inspect sibling Attributes. In
  particular, Password Link resolution and the rule selecting between the
  Password Link attribute and in-structure password mechanisms require the
  object plus its Attribute Set and belong to KMIPKIT-0051 validation.
- Full `validate()` reporting and universal pre-send rejection are assigned
  to KMIPKIT-0051. Constructors still enforce their own type-safe and
  source-defined structural contract.
- No claim of server interoperability, profile conformance, certification,
  or FIPS status is made.

## Success Criteria

- **KMIPKIT-0049-SC-001**: The inventory accounts for all nine object types,
  every structure member in §§2.1–3.12, all transparent key material forms,
  and their source-defined requiredness and conditional cases.
- **KMIPKIT-0049-SC-002**: Tests prove exact tags, TTLV types, field order,
  optionality, and round-trip preservation for every typed Structure; negative
  cases cover invalid requiredness, conditional fields, and wrapped/plain
  representation mismatches.
- **KMIPKIT-0049-SC-003**: Unknown Object Type, Key Format Type, unknown child
  Items, and vendor extension values survive generic or typed-fallback
  conversion without loss.
- **KMIPKIT-0049-SC-004**: Secret-bearing owned data, including a standalone
  wrapped `KeyBlockValue` before builder transfer, is absent from Debug and
  errors and KMIPKit-owned copies are zeroized on drop; lifetime documentation
  states what is outside that guarantee.
- **KMIPKIT-0049-SC-005**: Register/Import consumers can use typed objects
  without breaking existing generic constructors, and their emitted messages
  match the existing operation contract.
- **KMIPKIT-0049-SC-006**: Red, Green, and Refactor commits are distinct;
  local required checks, Rust export audit, deterministic generation checks,
  and requirement traceability pass before review.
- **KMIPKIT-0049-SC-007**: All four KMIP 3.0 system credential types map to
  Object Types 0x0D–0x10 and the exact §11.58 Table 605 Structure tags; tests
  cover source-defined member types, requiredness, order, Password's
  object/Attribute Set boundary, unknown values, and redaction.
- **KMIPKIT-0049-SC-008**: Credential object values are built only from
  `SecretText`/`SecretBytes` owners, borrowed through callback-scoped readers,
  and preserved in generic TTLV without algorithm computation or version
  claims; the client validator enforces the active KMIP edition.

## Key Entities

- **Managed Object**: One of the nine KMIP 2.1 object Structures, or a generic
  unknown-object fallback paired with its raw-preserving Object Type.
- **Key Block**: The key-format metadata, optional key value and cryptographic
  metadata, and optional wrapping data associated with a managed key object.
- **Key Value**: Plaintext key material and optional associated Attributes;
  distinct from the wrapped Key Value Byte String carried by a Key Block.
- **Key Material**: Opaque source-encoded bytes or one of the transparent
  key-material Structures, owned or borrowed according to API lifetime.
- **Key Wrapping Data**: Caller-provided metadata describing wrapping method,
  key identifiers, parameters, signature/MAC, IV/counter/nonce, and encoding.
- **Secret Owner**: The KMIPKit-owned TTLV payload storage that redacts and
  zeroizes its payload on release.

## Assumptions

- The user's directly approved Phase 5 plan authorizes this specification and
  its sequence. This feature branch starts from the pushed HEAD of
  `feature/KMIPKIT-0048-typed-attributes-release` (PR #140) and stacks toward
  `release/1.0.0`, as an explicit exception to the constitution's release-base
  default.
- The source's nine Object Type structures are Certificate, Certificate
  Request, Opaque Object, PGP Key, Private Key, Public Key, Secret Data, Split
  Key, and Symmetric Key; the version's catalog and pinned OASIS tables are
  authoritative for exact contents.
- KMIPKit's existing `Value`/`Item` owner already recursively zeroizes its
  payload storage; typed owners will reuse that contract instead of promising
  cleanup of caller or foreign-runtime copies.
- Object Type and Key Format Type values remain open raw values. Unknown or
  extension variants use a generic representation.
- Typed response accessors and cross-language parity are delivered in 0052
  and 0053, after this Rust object model is stable.
