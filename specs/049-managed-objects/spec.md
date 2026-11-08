# Feature Specification: Typed Managed Objects and Key Material

**Feature Branch**: `feature/KMIPKIT-0049-managed-objects`
**Created**: 2026-11-05
**Status**: Approved by maintainer plan authorization
**Input**: Approved KMIPKit Phase 5 plan, item 0049: typed construction and reading for all nine KMIP 2.1 managed-object types, key structures, wrapping structures, and explicit secret ownership and cleanup.

## Purpose

Let Rust client authors construct and inspect KMIP 2.1 managed objects and key
material through named, lossless types. Preserve generic TTLV access for future
and vendor extensions, and make the lifetime of KMIPKit-owned secret material
explicit.

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

### Edge Cases

- The `Object Type` and `Key Format Type` are open numeric values. Unknown and
  extension values remain usable through a generic fallback.
- `Key Value` in `Key Block` can be absent, a plaintext Structure, or wrapped
  Byte String according to the source schema; the typed model does not
  synthesize missing fields.
- `Key Material` encoding is selected explicitly by the caller's Key Format
  Type. KMIPKit does not infer algorithm, size, usage, or protection policy.
- A Split Key's Prime Field Size is required only for Polynomial Sharing
  Prime Field; Key Part Identifier must be from 1 through Split Key Parts.
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
  §§3.4–3.12, Tables 18–26. Unknown and extension key formats MUST remain
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
The OASIS Usage Guide is informative and does not add requirements.

| Requirement | Normative source | Verification |
|---|---|---|
| FR-001 | §§2.1–2.9, Tables 2–10; Object Type §11.34; pinned tag allocations | Every object record, tag association, required/optional field, and direct Register/Import Item shape tested against catalog and pinned source |
| FR-002–004 | §§3.1–3.12, Tables 12–26; field ordering §10.1.2 | Typed structure vectors, malformed-field negatives, exact-order and round-trip tests |
| FR-005–006 | §§6.1.25, 6.1.43 and generic TTLV §§5.1–5.4; API compatibility policy | Register/Import fake-transport tests; generic conversion property tests |
| FR-007–008 | AGENTS.md security boundary; TTLV ownership contract; §§3.1–3.3 wrapping schema | Redaction, zeroization, borrowed-lifetime, and no-crypto-operation tests |
| FR-009–010 | Constitution §§I–IV; pinned catalog and API manifest rules | Requirement-to-test audit, Rust consumer examples, manifest and binding-parity audit |

## Scope and Exclusions

- In scope: Rust models for all nine KMIP 2.1 object structures, Key Block,
  Key Value, Key Material, wrapping structures, additive Register/Import
  integration, and owned-secret handling.
- Existing generic TTLV and operation APIs remain available and lossless.
- This feature does not add operations, response-specific object accessors,
  response `.raw()` APIs, or four-language typed parity; those are assigned
  to KMIPKIT-0052 and KMIPKIT-0053 respectively.
- This feature does not perform cryptographic operations, parse or validate
  certificate cryptography, decompose PGP keys, unwrap keys, infer cryptographic
  choices, or determine server policy/capabilities.
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
  `feature/KMIPKIT-0048-typed-attributes` (PR #115) and stacks toward
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
