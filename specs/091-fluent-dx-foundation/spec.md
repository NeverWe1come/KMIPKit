# Feature Specification: KMIPKit Fluent API Foundation (KMIPKIT-0091)

- **Feature Branch**: `feature/KMIPKIT-0091-fluent-dx`
- **Base**: `release/1.0.0` at `9692057c9ade474fb0c4e0b7130539da368eb794`
- **Status**: Draft for maintainer review; implementation authorization is the user request, subject to this spec's approval
- **Owning identifier**: KMIPKIT-0091

## 1. Purpose

Add a task-oriented, fluent entry point for the most common synchronous client
journey: create a symmetric key, retrieve an object by its identifier, and
destroy an object by its identifier. The same capability is exposed in Rust,
C, Java, and Python while preserving their existing public APIs. Users select
security-relevant key properties explicitly and do not need to construct
KMIP Request Payloads, message headers, batch items, or numeric protocol tags
for this journey.

This is the foundation delivery of the larger Fluent DX plan. Key-pair and
registered-object creation, cryptographic operations, search, other lifecycle
operations, and progressive examples are separate follow-on specifications.

## 2. Normative and Project References

- OASIS KMIP Specification v2.1: §4 Object Attributes; §6.1.13 Create;
  §6.1.15 Destroy; §6.1.19 Get; §7.1 Attribute; §7.34 Name; §11.56 Tag
  Enumeration.
- Approved KMIPKit specifications: KMIPKIT-0007 client execution,
  KMIPKIT-0014 server-generated creation, KMIPKIT-0047 named protocol values,
  KMIPKIT-0048 typed attributes, KMIPKIT-0049 managed objects, KMIPKIT-0051
  pre-send validation, KMIPKIT-0052 typed responses, KMIPKIT-0053 binding
  parity, and KMIPKIT-0069 multi-version foundation.
- Accepted ADRs: ADR-0014 public transport exchange boundary and ADR-0022
  multi-version architecture.
- Product invariants: `.specify/memory/constitution.md` and `AGENTS.md`.

## 3. User Scenarios

### User Story 1 - Create and identify a symmetric key (P1)

An application developer supplies an approved algorithm, key length, and
usage mask; optionally supplies a human-readable name and catalogued
attributes; executes one Create operation; and obtains the server-returned
Unique Identifier from the typed outcome.

**Independent test**: A four-language consumer sends Create through a
loopback TLS 1.3/mTLS fixture, which validates the decoded request and returns
a valid typed success response with a deterministic synthetic identifier.

**Why this priority**: Key creation is the foundation for the stated goal of
letting application developers complete common KMIP tasks without manually
assembling protocol structures.

**Acceptance scenarios**:

1. **Given** a client and explicit algorithm, length, and usage mask,
   **When** the caller executes Create, **Then** the fixture receives one
   typed Create with those values and the caller can read the returned ID.
2. **Given** invalid or missing required key properties, **When** the caller
   builds or executes the request, **Then** validation fails before network
   transmission.

### User Story 2 - Retrieve by identifier (P1)

The developer passes the returned identifier to a typed Get convenience
method and inspects the typed response, without constructing a Get payload.

**Why this priority**: Get completes the first useful lifecycle task and
proves that the Create result can flow into another operation.

**Independent test**: Each language sends a typed Get to the fixture and
parses a valid response containing a synthetic managed object.

**Acceptance scenario**: **Given** a valid identifier, **When** the caller
executes Get, **Then** the fixture receives that identifier and the caller
receives the existing typed object response.

### User Story 3 - Destroy by identifier (P1)

The developer passes an identifier to a typed Destroy convenience method and
receives the explicit KMIP operation outcome.

**Why this priority**: Destroy completes the common key lifecycle without
requiring callers to manually create a batch or interpret a raw response.

**Independent test**: Each language sends a typed Destroy to the fixture and
observes its typed success, pending, or failure result.

**Acceptance scenario**: **Given** a valid identifier, **When** the caller
executes Destroy, **Then** the caller can distinguish Success, Pending, and
Failure through the existing response contract.

### User Story 4 - Add a catalogued or expert attribute (P2)

The developer sets common catalogued attributes with typed semantic values.
An expert may supply a generic TTLV Item for an unknown attribute. A
Vendor Attribute is constructed through a dedicated helper that produces its
required §4.60 structure; extension tags are not inferred to be attributes.

**Why this priority**: Expert customization is necessary for a fluent API to
remain useful when an application needs protocol attributes beyond the common
defaults.

**Independent test**: Shared cases exercise accepted catalogued values,
wrong-type rejection, explicit unknown Items, and valid/invalid Vendor
Attribute structures in all four bindings.

**Acceptance scenarios**:

1. **Given** a catalogued attribute and a semantic value of the wrong type,
   **When** the caller adds it, **Then** construction fails locally.
2. **Given** an unknown attribute tag, **When** the caller supplies an
   explicit TTLV Item, **Then** the item is preserved; without a typed Item,
   construction fails.
3. **Given** Vendor Attribute components, **When** the caller uses the
   dedicated helper, **Then** KMIPKit creates the valid required structure and
   rejects malformed components.

### Edge Cases

- Missing or invalid algorithm, length, or usage mask fails before sending.
- An absent Name is distinct from a present Name; the convenience setter
  always emits KMIP Name Type `Uninterpreted Text String`.
- Repeated attributes remain representable when KMIP permits them; operation
  cardinality rules are validated by the existing typed request path.
- A catalogued tag with an incorrect TTLV type, or an unknown tag without an
  explicit Item, fails locally.
- A `0x54xxxx` message extension tag cannot be added through the Object
  Attribute path.
- Null, malformed, or unsupported Unique Identifiers fail locally for Get
  and Destroy.
- Pending keeps its correlation value available and does not trigger Poll;
  transport failure does not trigger a retry.

### Key Entities

- **Symmetric-key request**: Algorithm, length, usage mask, optional Name, and
  zero or more catalogued or explicitly typed expert attributes.
- **Unique Identifier**: The typed KMIP identifier used to retrieve or destroy
  one object.
- **Operation response**: Existing typed response data and explicit
  Success/Pending/Failure outcome, including Pending correlation when present.

## 4. Functional Requirements

- **KMIPKIT-0091-FR-001**: Rust MUST expose `Client::request(&mut self)` as a
  scoped fluent namespace. Its Create Symmetric Key, Get, and Destroy builders
  MUST build the existing typed request types, execute through existing typed
  client methods, and provide conversion to an existing typed batch item.
- **KMIPKIT-0091-FR-002**: Create Symmetric Key MUST require an explicit
  Cryptographic Algorithm, Cryptographic Length, and Cryptographic Usage
  Mask, supplied directly or through a named supported preset. A preset MUST
  set only its documented algorithm and length; it MUST NOT choose usage,
  extractability, sensitivity, wrapping, mode, padding, or server policy.
- **KMIPKIT-0091-FR-003**: The builder MUST provide a convenience Name method
  that constructs a valid KMIP Name with a documented default Name Type.
  Repeated Name attributes MUST remain representable through the lower-level
  attribute method.
- **KMIPKIT-0091-FR-004**: Catalogued attributes supplied as semantic values
  MUST be encoded with the catalogued TTLV type and rejected before network
  transmission if the value cannot be represented or violates the existing
  attribute policy. The builder MUST preserve repeated attributes where KMIP
  permits them and MUST leave operation-specific cardinality checks to the
  existing typed request validation.
- **KMIPKIT-0091-FR-005**: An unknown attribute MUST require an explicit
  generic TTLV Item including its type. A Vendor Attribute MUST use a
  dedicated constructor that produces and validates the required
  Vendor Identification, Attribute Name, and Attribute Value structure.
  Numeric `0x54xxxx` message-extension tags MUST NOT be accepted as ordinary
  Object Attributes by inference.
- **KMIPKIT-0091-FR-006**: Get and Destroy convenience builders MUST require
  an explicit Unique Identifier for this initial single-object journey.
  They MUST NOT silently omit the identifier or infer remote object state.
- **KMIPKIT-0091-FR-007**: A fluent `execute` call MUST make the existing
  Success, Pending, and Failure outcome visible through the existing typed
  response API. Pending correlation data MUST remain available. The fluent
  API MUST NOT poll or retry automatically.
- **KMIPKIT-0091-FR-008**: Per-exchange transport options MUST use the existing
  `RequestOptions` contract. Authentication and shared Request Header fields
  MUST remain explicit batch-level operations and MUST NOT be placed in
  `RequestOptions`.
- **KMIPKIT-0091-FR-009**: C MUST expose task-level functions for the three
  initial operations and an opaque builder for Create Symmetric Key. Functions
  MUST use the stable C ABI status codes, versioned input records where a
  record is needed, explicit ownership, and the existing release functions.
  No parallel error-handle model is introduced.
- **KMIPKIT-0091-FR-010**: Java MUST expose an idiomatic `org.kmipkit.client`
  facade and `org.kmipkit.fluent` builders. Existing `org.kmipkit.extensions`
  classes and symbols MUST continue to work.
- **KMIPKIT-0091-FR-011**: Python MUST expose `kmipkit.fluent` builders and
  named-argument convenience functions on its public Client facade. Existing
  `kmipkit.extensions` and Client APIs MUST continue to work.
- **KMIPKIT-0091-FR-012**: Rust MUST own fluent construction, semantic
  validation, and request conversion. C, Java, and Python MUST route through
  the existing Rust implementation and stable C ABI; they MUST NOT implement
  independent KMIP serialization or validation rules.
- **KMIPKIT-0091-FR-013**: Fluent execution MUST accept only existing typed
  request/batch-item routes. No API in this feature may transmit an arbitrary
  generic TTLV Request Message, raw request bytes, or a caller-supplied
  transport.
- **KMIPKIT-0091-FR-014**: The default effective version MUST remain KMIP
  2.1. Explicit version selection MUST use existing version-gated routes and
  reject unsupported builder/operation combinations before writing bytes.
- **KMIPKIT-0091-FR-015**: Secret-bearing builder inputs and outputs MUST
  follow existing zeroization, redaction, ownership-transfer, and foreign
  runtime copy limitations. Errors and formatting MUST NOT reveal credentials,
  key material, raw request bodies, or attribute values designated secret by
  existing policy.
- **KMIPKIT-0091-FR-016**: The public API manifest, generated declarations,
  exports, Rust/foreign consumers, English and Spanish user guides, and
  requirement traceability MUST describe the same public surface.

## 5. API Shape

The following examples define semantic naming. Language-specific spelling
follows established conventions.

```rust
client.request()
    .create_symmetric_key()
    .aes_256()
    .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
    .name("payments-key")
    .execute(&limits)?;

client.request().get(&unique_identifier).execute(&limits)?;
client.request().destroy(&unique_identifier).execute(&limits)?;
```

Every bound builder also provides `build()` for the existing typed request,
`to_batch_item()` for composition into an existing ClientBatch, and
`with_options(RequestOptions)` for exchange-local options. `execute()`
returns the existing typed one-item response, retaining all KMIP outcomes.
Advanced attributes use typed catalog values; unrecognized values require an
explicit TTLV Item. The Vendor Attribute helper takes vendor identification,
attribute name, and an explicit TTLV Item value.

C exposes `kmipkit_client_create_symmetric_key_v1`,
`kmipkit_client_get_by_id_v1`, and `kmipkit_client_destroy_by_id_v1` task-level
functions, plus an opaque symmetric-key builder. Each direct function uses the
client's configured exchange-timeout policy and existing codec limits, and
returns an owned `kmipkit_client_batch_response_t` containing exactly one
operation response. Callers inspect it with the existing response accessors,
including the operation outcome and Pending correlation data. Explicit
exchange-timeout overrides are available through corresponding
`_with_options_v1` variants that take a valid, non-null
`kmipkit_request_options_v1_t`; a null options pointer is invalid under the
existing C ABI contract.

The versioned simple Create input is
`kmipkit_symmetric_key_request_v1_t`: `struct_size`, `abi_version`, raw
Cryptographic Algorithm (`uint32_t`), signed Cryptographic Length (`int32_t`),
Cryptographic Usage Mask (`int32_t`), `name_present` (`uint8_t`), reserved
zeroed bytes, and a `kmipkit_byte_span_v1_t` Name Value. When
`name_present == 0`, the span must be empty; when it is `1`, the span must
refer to valid UTF-8 for the duration of the call. The semantic helper uses
`KMIPKIT_NAME_TYPE_UNINTERPRETED_TEXT_STRING`. Get and Destroy take a pointer
to the existing versioned `kmipkit_unique_identifier_v1_t`. The direct
function forms are
`kmipkit_client_create_symmetric_key_v1(client, request, limits,
out_response)`,
`kmipkit_client_get_by_id_v1(client, unique_identifier, limits,
out_response)`, and
`kmipkit_client_destroy_by_id_v1(client, unique_identifier, limits,
out_response)`; `_with_options_v1` variants add an `options` pointer before
`out_response`.

The opaque builder API is `kmipkit_symmetric_key_builder_create`,
`_set_algorithm`, `_set_length`, `_set_usage_mask`, `_set_name`,
`_add_catalogued_attribute`, `_add_attribute_item`, `_add_vendor_attribute`,
`_build_batch_item`, `_execute_v1`, `_execute_with_options_v1`, and `_release`.
`_add_catalogued_attribute` accepts a generated
`KMIPKIT_STANDARD_ATTRIBUTE_*` tag plus a typed TTLV Value and rejects type
mismatches; `_add_attribute_item` accepts an explicit TTLV Item for unknown
tags. The Vendor Attribute function accepts vendor identification and
attribute name bytes plus an explicit TTLV Item as its Attribute Value.
`_build_batch_item` leaves the builder usable and returns an independently
owned batch item. Borrowed inputs are copied on success and ownership is never
transferred unless the function name or existing ABI contract says so.
`_execute_v1` uses client-configured timeout policy; its options variant
requires a valid, non-null v1 options record.

Java uses `client.request().createSymmetricKey()` with typed setters and
`execute()`. Python uses `client.request().create_symmetric_key()` and a
named-argument convenience form. Both expose `build`, `to_batch_item`, and
exchange-local options with their normal result and resource-lifetime types.

## 6. Scope and Exclusions

This feature covers the API contract and implementation for Create Symmetric
Key, Get by Unique Identifier, Destroy by Unique Identifier, common
catalogued attributes, explicit generic Items, and structurally valid Vendor
Attributes in KMIP 2.1 by default. Only presets whose operations and values
are already in the approved versioned surface may be offered. Selection of
key usage, extraction policy, sensitivity, mode, padding, and cryptographic
policy remains explicit.

Create Key Pair, Create Split Key, Register, Encrypt/Decrypt, Sign/Verify,
MAC, KEM, Locate, Rekey, Revoke, attribute mutation operations, arbitrary
Request Message sending, automatic retries, automatic polling, undocumented
presets, KMIP 1.x, and new KMIP 3.0 routes are outside this delivery and
require their own specifications or existing approved feature scope.

## 7. Acceptance and Verification

- TDD tests demonstrate Red before behavior implementation for semantic
  attribute conversion, each task-level operation, outcome propagation, and
  cross-language ownership.
- Rust tests cover valid/invalid semantic values, unknown values as explicit
  Items, Vendor Attribute structure/order, required key parameters, direct
  execution, batch conversion, and pre-send version/validation failures.
- C consumer tests cover each helper and opaque builder, input record size and
  ABI version, null/count boundaries, status and error snapshots, ownership
  transfer, release, and secret redaction.
- Java and Python external consumers complete the same Create/Get/Destroy
  fixture journey and test typed failures, Pending correlation, resource
  cleanup, and legacy API compatibility.
- A loopback-only TLS 1.3/mTLS responder validates actual encoded Create,
  Get, and Destroy requests and returns matching valid typed responses. No
  network test uses a real KMS or real secret.
- Existing execute-boundary, request validation, version-gate, lifecycle
  redaction, and response tests remain passing.
- The API manifest generator and export audits pass. Workspace formatting,
  lint, tests, traceability, and relevant coverage gates pass before the
  feature PR is ready for review.

## 8. Assumptions

- This is a new public API in an unreleased 1.0 line; current public functions,
  symbols, and packages remain source/ABI compatible.
- The first delivery uses the existing typed protocol and client operations;
  it adds ergonomics, not new KMIP wire capability.
- Pending remains a caller-managed protocol outcome. A successful Create
  yields its identifier only from the valid typed success response.
- The C direct functions and builder coexist: direct functions cover ordinary
  parameter sets; the builder covers optional catalogued and expert fields.

## 9. Success Criteria

- **KMIPKIT-0091-SC-001**: A consumer in each of Rust, C, Java, and Python
  completes Create/Get/Destroy against the controlled TLS fixture using
  semantic operation APIs and without manually creating a Request Payload,
  batch, or numeric protocol tag.
- **KMIPKIT-0091-SC-002**: All four consumers observe the same required-field
  validation, default Name Type, typed operation outcomes, and Pending
  correlation behavior.
- **KMIPKIT-0091-SC-003**: Existing public API and ABI compatibility checks
  pass, and each declared new export is present in the generated public API
  manifest and its relevant binding inventory.
- **KMIPKIT-0091-SC-004**: The complete required format, lint, test, coverage,
  traceability, manifest, and export-audit checks pass before the draft PR is
  ready for review.
