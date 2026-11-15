# Feature Specification: KMIPKit Fluent Object Creation (KMIPKIT-0092)

**Feature Branch**: `feature/KMIPKIT-0092-fluent-objects`  
**Base**: `release/1.0.0` at `2b01ab516e50035eee42fc206396c313dae8d9fb`  
**Status**: Draft; implementation is gated on human approval of this specification.  
**Owning identifier**: KMIPKIT-0092

## 1. Purpose

Extend the KMIPKit task-oriented API with public/private key-pair creation,
split-key creation, and registration of caller-supplied managed objects. A
programmer should select the key properties or object they intend to create or
register without manually assembling operation payloads, message headers, or
batch items. Existing typed KMIP requests remain the source of protocol
behavior, and all four supported languages expose the same capabilities.

This is the second delivery of the Fluent DX plan. It builds on the approved
KMIPKIT-0091 foundation. Cryptographic operations, search, additional lifecycle
operations, and end-to-end examples remain in later specifications.

## 2. Normative and Project References

- OASIS KMIP Specification v2.1: §§2.1–2.9 and 3.1–3.12 for the approved
  managed-object and key-material structures; §§4–5 for object attributes
  and attribute groups; §4.13 Tables 53–54 and §4.14 Tables 55–56 for
  cryptographic length and domain parameters;
  §6.1.9 Tables 189–192 for Create Key Pair; §6.1.10
  Tables 193–195 for Create Split Key; §6.1.43 Tables 291–294 for Register;
  §§8.1–8.6 and 9.1–9.21 for message, batch, response, and result behavior.
- Approved KMIPKit specifications: KMIPKIT-0007 client execution,
  KMIPKIT-0014 server-generated object creation, KMIPKIT-0022 object
  lifecycle and Register, KMIPKIT-0047 named protocol values, KMIPKIT-0048
  typed attributes, KMIPKIT-0049 managed objects and secret ownership,
  KMIPKIT-0051 pre-send validation, KMIPKIT-0052 typed responses,
  KMIPKIT-0053 binding parity, KMIPKIT-0069 version support, and
  KMIPKIT-0091 Fluent API Foundation.
- Accepted ADRs: ADR-0014 public transport exchange boundary and ADR-0022
  multi-version architecture.
- Product invariants: `.specify/memory/constitution.md` and `AGENTS.md`.

The immutable local OASIS sources and checked-in normative catalog are
authoritative. No Usage Guide example creates a new normative requirement.
This feature adds task-oriented access to existing operation behavior; it does
not expand the underlying KMIP operation, object-format, version, or transport
support approved by the referenced specifications.

## 3. User Scenarios

### User Story 1 — Create a public/private key pair (P1)

An application developer asks the KMIP server to create a public/private key
pair, chooses an approved algorithm and its required parameters, selects the
usage of each key, optionally supplies a name and other attributes, and
receives both identifiers from the typed result.

**Why this priority**: A key pair is a common foundation for signing,
verification, and key agreement, and its common and key-specific attributes
have protocol-defined behavior that a task-oriented API must preserve.

**Independent test**: A four-language consumer sends Create Key Pair through a
controlled TLS 1.3/mTLS fixture. The fixture verifies the decoded operation and
returns a typed response with deterministic private and public identifiers.

**Acceptance scenarios**:

1. Given explicit algorithm, required parameters, and caller-selected usage
   masks, when the caller executes Create Key Pair, then the fixture receives
   one valid typed operation and the caller can inspect both returned IDs.
2. Given common, private-key, and public-key attributes, when the request is
   built, then each group and its order are preserved and KMIPKIT-0014's
   Table 191 checks run before transmission.
3. Given a missing, inconsistent, unsupported, or version-incompatible
   selection, when the caller builds or executes, then validation fails before
   network transmission and no algorithm, parameter, or usage is substituted.

### User Story 2 — Create split-key parts (P1)

A developer asks the server to split a supported key object, explicitly
chooses the split method, number of parts, and threshold, optionally identifies
the input object, and receives every returned identifier in protocol order.

**Why this priority**: Split-key controls and repeated result identifiers are
easy to lose in an oversimplified facade and are required for a useful,
lossless API.

**Independent test**: A controlled TLS fixture verifies the request fields and
returns one and multiple typed identifiers; the consumer observes every value
in order.

**Acceptance scenarios**:

1. Given explicit split parameters, when the caller executes, then those exact
   values are sent and all returned IDs remain available in wire order.
2. Given Polynomial Sharing Prime Field, when the caller omits the required by
   KMIPKit policy Prime Field Size, then construction fails locally.
3. Given no input identifier, when the caller executes, then the identifier
   remains absent; KMIPKit does not infer an object or perform split-key
   cryptography locally.

### User Story 3 — Register a supplied managed object (P1)

A developer selects one of the nine KMIP 2.1 managed-object forms already
represented by KMIPKIT-0049, supplies its semantic key material or object
fields, adds its attributes and optional protection-storage choices, and
receives the server's identifier. Common key and certificate forms have
task-oriented constructors; the other approved forms remain available through
their existing typed object constructors.

**Why this priority**: Applications often generate or receive key material
outside the KMIP server and need to register it without hand-assembling a
Register payload or duplicating object-type and Key Block relationships.

**Independent test**: A controlled TLS fixture verifies Register of plaintext
and wrapped test objects in the forms already represented by KMIPKIT-0049,
then returns a typed success containing a deterministic identifier.

**Acceptance scenarios**:

1. Given a typed managed object and its attributes, when Register is built,
   then its Object Type, object structure, Key Format Type, key material, and
   attribute groups remain correctly associated.
2. Given a malformed, mismatched, unsupported-version, or incomplete object,
   when the caller builds or executes, then local validation rejects it before
   network transmission.
3. Given secret-bearing key material, when an owned KMIPKit object is released,
   then KMIPKit-owned copies follow the existing zeroization and redaction
   rules; Java and Python documentation states the limits of runtime-managed
   copies.
4. Given a server failure, Pending result, or transport error, when the
   operation completes, then the existing typed outcome and delivery-state
   contracts remain visible without an automatic poll or retry.

### Edge Cases

- Common, Private Key, and Public Key Attribute groups remain distinct;
  repeated attributes are preserved where the source tables permit them.
- For every Table 191 attribute, Create Key Pair applies the existing rule:
  each key-specific value overrides the common value, both effective values
  must be absent or equal, and no value is copied or synthesized.
- Optional structures remain distinct from absent structures. Create Split
  Key's required Attributes structure remains present even when empty.
- A missing Prime Field Size for Polynomial Sharing Prime Field is rejected
  according to the existing KMIPKit client policy. Other split-method values
  and integer bounds are handled by the existing typed request contract.
- Register preserves the exact supported Key Format Type representation and
  rejects mismatched object/type structures, missing required attributes, or
  invalid conditional algorithm/length pairs before sending.
- Unknown/vendor object types remain available only through the existing
  explicit typed generic-object route. This feature does not infer an object
  type from a TTLV tag or add arbitrary TTLV message transmission.
- Pending does not trigger Poll; transport errors do not trigger retries.
  Result correlation and delivery state remain visible through the existing
  contract.
- Secrets, credentials, raw protocol bodies, and secret attributes are not
  formatted, logged, or exposed through errors.

## 4. Functional Requirements

- **KMIPKIT-0092-FR-001**: Rust MUST expose fluent Create Key Pair, Create
  Split Key, and Register builders that produce the existing typed KMIPKit
  request models, execute only through existing typed client methods, and can
  convert to the existing typed batch-item path.
- **KMIPKIT-0092-FR-002**: Create Key Pair MUST expose its common, private, and
  public attribute and protection-storage groups without merging or losing
  ordering. It MUST enforce KMIPKIT-0014's §6.1.9 Table 191 consistency rule
  and MUST NOT invent or copy attribute values.
- **KMIPKIT-0092-FR-003**: The initial public/private key-pair presets MUST be
  RSA with a 3072-bit length, ECDSA with Recommended Curve P-256, and ECDH
  with Recommended Curve P-256. These presets use the existing KMIP 2.1
  Create Key Pair route and set only their documented algorithm, length, or
  curve. The caller MUST select each key's usage independently. Custom typed
  parameters remain available. No preset may select TDES, ECB, RSA PKCS#1
  v1.5, or a KMIP 3.0 route.
- **KMIPKIT-0092-FR-004**: Create Split Key MUST require explicit split method,
  object type, number of parts, threshold, and Attributes. Assigned object
  types and split methods MUST have symbolic named values in the common
  builder. It MUST preserve the optional input Unique Identifier, Prime Field
  Size, and protection-storage masks, apply existing request validation, and
  expose every returned Unique Identifier in wire order.
- **KMIPKIT-0092-FR-005**: Register MUST accept semantic constructors for the
  nine KMIP 2.1 object forms approved by KMIPKIT-0049: Certificate, Symmetric
  Key, Public Key, Private Key, Split Key, Secret Data, Opaque Object, PGP Key,
  and Certificate Request. The API MUST preserve object type, key format,
  associated attributes, secret ownership, and existing conditional Register
  validation. It MUST NOT add new object or key-material formats in this
  feature. Unknown/vendor object values remain explicit generic expert inputs.
- **KMIPKIT-0092-FR-006**: Secret-bearing values MUST use existing owned secret
  containers and zeroization behavior. Java and Python documentation MUST
  identify copies controlled by their runtimes and MUST NOT claim guaranteed
  erasure of runtime-managed copies.
- **KMIPKIT-0092-FR-007**: C MUST expose direct task-level functions for the
  simple operation forms and opaque builders for advanced inputs, using the
  existing fixed-width status codes, ownership, release functions, and
  versioned records where required. It MUST NOT introduce a parallel error
  handle model.
- **KMIPKIT-0092-FR-008**: Java MUST provide idiomatic entry points and
  builders in `org.kmipkit.client` and `org.kmipkit.fluent`; Python MUST provide
  corresponding builders in `kmipkit.fluent` and named-argument convenience
  methods on its public Client facade. Existing APIs remain source-compatible.
- **KMIPKIT-0092-FR-009**: Rust MUST own fluent construction, semantic
  validation, and typed conversion. C, Java, and Python MUST route through the
  existing Rust implementation and C ABI and MUST NOT duplicate KMIP
  serialization or validation rules.
- **KMIPKIT-0092-FR-010**: Fluent execution MUST use existing typed request
  and batch-item routes only. It MUST NOT transmit an arbitrary generic TTLV
  Request Message, raw request bytes, or caller-supplied transport.
- **KMIPKIT-0092-FR-011**: Each explicit execution MUST perform at most one
  exchange. Success, Failure, Pending, correlation, operation errors, and
  delivery state MUST follow KMIPKIT-0007 and KMIPKIT-0052. No Poll or retry
  may occur automatically.
- **KMIPKIT-0092-FR-012**: KMIP 2.1 remains the default effective version.
  Explicit versions MUST use existing version-gated routes; unsupported
  operation, preset, object, or encoding combinations MUST fail before
  network transmission.
- **KMIPKIT-0092-FR-013**: Existing APIs MUST remain compatible. The public
  API manifest and generated exports, C/JNI/CFFI ownership behavior,
  documentation in English and Spanish, executable consumer examples, and
  requirement traceability MUST describe the same supported surface.
- **KMIPKIT-0092-FR-014**: Every applicable OASIS client requirement and
  operation-table obligation MUST map to an exact pinned source, a stable
  feature requirement, implementation, and executable verification. Server
  behavior MUST NOT be represented as a client guarantee or formal
  conformance claim.
- **KMIPKIT-0092-FR-015**: The common fluent path MUST use semantic names for
  assigned algorithms, usage-mask bits, split methods, object types, key
  formats, and object forms. It MUST NOT require raw numeric KMIP values or
  tags. Unknown and vendor values remain available through the existing
  explicit typed expert APIs and MUST NOT be inferred from numeric ranges.

## 5. API Shape

Language-specific spelling follows existing conventions. The following
illustrates the required caller choices without prescribing builder internals:

```rust
client.request()
    .create_key_pair()
    .ecdsa_p256()
    .private_usage(CryptographicUsage::Sign)
    .public_usage(CryptographicUsage::Verify)
    .name("service-signing-key")
    .execute(&limits)?;

client.request()
    .create_key_pair()
    .ecdh_p256()
    .private_usage(CryptographicUsage::DeriveKey)
    .public_usage(CryptographicUsage::DeriveKey)
    .execute(&limits)?;

client.request()
    .create_split_key()
    .object_type(ManagedObjectType::SymmetricKey)
    .method(SplitKeyMethod::Xor)
    .parts(3)
    .threshold(2)
    .execute(&limits)?;

client.request()
    .register()
    .symmetric_key(key_material, KeyFormatType::TransparentSymmetricKey)
    .name("imported-data-key")
    .execute(&limits)?;
```

Builders also provide `build()`, `to_batch_item()`, and exchange-local
`with_options(RequestOptions)` where the operation contract permits them.
Authentication and shared Request Header fields remain explicit batch-level
operations. Raw generic Items remain confined to the already existing typed
expert/object APIs and are never accepted as a request message to transmit.

## 6. Key Entities

- **Key-pair creation request**: Explicit algorithm and required parameters,
  caller-selected usage for each key, three distinct attribute groups, and
  optional protection-storage choices.
- **Split-key request**: Explicit object type, method, part count, threshold,
  required Attributes, and optional input identifier and method-specific
  parameters.
- **Registered managed object**: An existing KMIPKIT-0049 typed object with
  its exact Object Type, Key Format Type, Key Material, attributes, and secret
  owner.
- **Operation outcome**: Existing typed Success, Failure, or Pending data,
  including repeated identifiers, correlation, and delivery state as
  applicable.

## 7. Success Criteria

### Measurable Outcomes

- **KMIPKIT-0092-SC-001**: In Rust, C, Java, and Python, a consumer completes
  key-pair creation, split-key creation, and Register against the controlled
  TLS fixture without manually composing Request Payloads, message headers,
  or batch items.
- **KMIPKIT-0092-SC-002**: The fixture observes the caller's exact cryptographic
  choices, split parameters, object format, and attributes; results preserve
  every required or repeated identifier and distinguish Success, Pending, and
  Failure.
- **KMIPKIT-0092-SC-003**: Invalid local inputs fail before transmission in
  all four languages, and each explicit operation performs no automatic Poll
  or retry.
- **KMIPKIT-0092-SC-004**: All applicable requirements and operation-table
  obligations have complete, executable traceability; changed production
  code meets the repository's coverage gates.
- **KMIPKIT-0092-SC-005**: Existing public APIs and generated declarations
  remain compatible, and the documented secret-memory boundaries match the
  tested Rust, C, JNI, and CFFI ownership behavior.

## 8. Assumptions and Exclusions

- KMIPKIT-0091's request namespace, result contract, attribute helpers,
  version handling, and transport options are available before this delivery
  is integrated. This specification does not reimplement them.
- The approved algorithms, structures, and key-material formats are exactly
  those already supported by the pinned KMIP 2.1 catalog and KMIPKIT-0014,
  KMIPKIT-0022, KMIPKIT-0047, and KMIPKIT-0049. This feature does not grant a
  new KMIP version or cryptographic operation.
- The KMIP server generates the pair and split parts and decides whether it
  accepts Register. KMIPKit does not perform local key generation, split-key
  cryptography, signing, wrapping, unwrapping, or format conversion.
- Cryptographic operations, Locate, Rekey, Revoke, other attribute lifecycle
  operations, batch Placeholder examples, and full progressive user journeys
  remain out of scope for this delivery.
- Java/Python runtime copies may not be deterministically erased; KMIPKit
  guarantees apply only to memory it owns.
