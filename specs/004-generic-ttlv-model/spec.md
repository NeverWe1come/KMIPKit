# Feature Specification: KMIP Generic TTLV Value Model

**Feature Branch**: **feature/KMIPKIT-0004-generic-ttlv-model-implementation**

**Created**: 2026-10-04

**Status**: Approved — design PR #10 was merged into `release/1.0.0` at `3638c6c7929992e8ced59a3903a0a6640f847069`; the required KMIPKIT-0003 implementation was merged at `b52df30648312f8c7703f511afe80a412cda66cd`.

**Input**: User roadmap: "lossless generic TTLV tree" within the KMIPKit 1.0 KMIP 2.1 client foundation.

## Clarifications

### Session 2026-10-04

- Q: Which tag-allocation entry controls when §11.56's individually assigned tags overlap its aggregate Reserved range? A: The accepted KMIPKit policy gives precedence to individually listed catalog entries and applies the aggregate Reserved range only to residual values without an individual assignment; it accepts the §11.56 `0x540000–0x54FFFF` Extensions range at the allocation gate. This is a project policy recorded in accepted ADR-0010, not an OASIS clarification.
- Q: What does closure-scoped value exposure guarantee, and how is zeroization verified? A: The API lends borrowed views that cannot escape by reference; callers may intentionally copy or format values, and KMIPKit cannot clear those copies. A safe test-only zeroization probe verifies the Drop path calls the zeroizer before release; tests do not inspect deallocated memory or use unsafe code.
- Q: How are Big Integer octets represented before wire validation? A: Preserve the exact Item Value octets without mathematical-integer normalization; an empty sequence is permitted in this in-memory model but is not thereby declared valid for TTLV wire encoding.
- Q: Which serialization, diagnostics, and logging surfaces are in scope here? A: This Rust model provides no `serde::Serialize` implementation; no other automatic payload serializer is supported in this feature. Debug, optional Display, and model-local errors redact payloads. The model adds no logging call sites; transport/client raw-body logging checks remain with those layers.

## User Scenarios & Testing

### User Story 1 - Construct and inspect any KMIP 2.1 TTLV value (Priority: P1)

A Rust application developer needs to construct and inspect a generic KMIP value without depending on operation-specific models. The model must distinguish each KMIP 2.1 TTLV item type and expose the corresponding value without coercion.

**Why this priority**: Typed protocol models and the generic public escape hatch both depend on a stable value representation.

**Independent Test**: Construct one value of every assigned KMIP 2.1 TTLV item type and use the explicit closure-scoped exposure API to verify the same tag, type, and value are observable.

**Acceptance Scenarios**:

1. **Given** an application creates each assigned KMIP 2.1 TTLV item type, **When** it inspects the item and explicitly exposes its value, **Then** the item reports the same item type and unchanged value.
2. **Given** a Big Integer value containing its signed two’s-complement octets, **When** the application inspects it, **Then** those octets remain unchanged.
3. **Given** an item with a Text String or Byte String value, **When** the application inspects it, **Then** text is not normalized and bytes are not altered.

### User Story 2 - Preserve tags, numeric values, and structure order (Priority: P1)

An application developer needs generic items to retain tag numbers, unknown enumeration values, bitmask bits, repeated child tags, and the order of Structure children. The model must not reinterpret values using an incomplete KMIP catalog.

**Why this priority**: A forward-compatible generic API must not discard data merely because a higher-level KMIP model does not recognize it.

**Independent Test**: Construct a Structure with repeated allocation-checked tags and unknown extension tags, unknown enumeration values, high-bit mask values, and out-of-order children; inspect it and verify exact tag values, payload values, and child order. Separately verify that arbitrary raw 24-bit tags are retained by the Raw Tag type but cannot enter a generic item unless they pass the tag-allocation check.

**Acceptance Scenarios**:

1. **Given** a 24-bit tag in the §11.56 Extensions range that is not known to this KMIPKit release, **When** an item is constructed and inspected, **Then** the tag value is returned exactly without truncation or replacement.
2. **Given** any 24-bit value, **When** it is represented as a Raw Tag, **Then** all 24 bits are retained, while constructing a public item from it requires successful allocation checking under the KMIP 2.1 tag catalog; a failed check leaves the Raw Tag available.
3. **Given** a tag explicitly listed as Reserved or inside an unused/reserved residual range, **When** it is used to construct a public item, **Then** construction fails; exact individually assigned tag entries take precedence over the aggregate reserved range notation in §11.56.
4. **Given** an unknown 32-bit Enumeration value or an Integer used as a bitmask, **When** the value is inspected, **Then** every bit is unchanged.
5. **Given** a Structure with repeated child tags and a caller-defined order, **When** its children are traversed, **Then** all children appear once in the original order.
6. **Given** a Structure at the maximum supported nesting depth, **When** an item would create a deeper tree, **Then** construction fails with a payload-free model error and leaves the existing Structure usable.

### User Story 3 - Inspect generic values without leaking contents through default diagnostics (Priority: P1)

An application developer needs to use Debug output while working with generic KMIP values without exposing strings, byte strings, integer payloads, or nested content that may contain credentials, secret key material, tickets, or raw message data.

**Why this priority**: Generic values are untrusted and may contain secrets; default diagnostic formatting must not become a disclosure path.

**Independent Test**: Place distinct payload sentinels in every value family and nested Structure. Format each public value type, `ValueView`, and variant directly, as well as its containing item and tree, through default Debug, any provided Display, and representative model errors. Verify no payload sentinel appears while the item type, tag, and Structure child count remain available. Add compile-fail API checks that `Value`, `ValueView`, `Item`, and `Structure` do not implement `serde::Serialize`, that a borrowed view cannot escape its closure, and that owning types do not implement `Clone` or `Copy`; runtime checks confirm callers can explicitly copy values; safe zeroization probes verify the Drop path and recursive live-object zeroization behavior.

**Acceptance Scenarios**:

1. **Given** any public generic value type or variant contains a payload, **When** it or a containing item/tree is formatted with default Debug, **Then** the payload is absent, including strings, bytes, Big Integer octets, booleans, and numeric values.
2. **Given** an application exposes a value through the closure-scoped API, **When** it reads a payload, **Then** the callback receives only borrowed views that cannot escape by reference; explicit copies are allowed and are outside KMIPKit's zeroization guarantee.
3. **Given** a public value type has a Display implementation, **When** it is formatted, **Then** its payload remains redacted; otherwise the type has no Display implementation. Public value, `ValueView`, item, and tree types do not implement `serde::Serialize`; no other automatic payload serializer is supported in this feature.
4. **Given** a model error is produced while constructing or inspecting an item/tree, **When** its available diagnostic formatting is used, **Then** it does not reveal payload sentinels. This in-memory model does not produce or format raw KMIP message bodies.
5. **Given** a generic value owns secret payload data, **When** the value or containing Structure is dropped, **Then** the current payload storage, including the backing capacity of owned String and byte buffers, is zeroized before release; no unsafe inspection of freed memory is required to verify the Drop path.

### Edge Cases

- A Raw Tag is the maximum representable 24-bit value.
- An unknown tag value is retained by Raw Tag without truncation; representing it there does not assert that it is valid or permitted in a public generic tree or on the KMIP wire.
- An unknown tag in the §11.56 Extensions range is permitted in a generic item after allocation checking; an arbitrary Raw Tag must pass the catalog-backed allocation check before item construction.
- Enumeration values use the full 32-bit unsigned range, including extension values.
- Integer bitmasks set the highest bit and several unknown bits.
- A Structure is empty, contains repeated tags, or contains nested Structures.
- A Structure may be nested through 64 levels; constructing a deeper tree fails before the value is accepted.
- Any payload-bearing variant may hold secret material; nested secret values and their current owned buffers are zeroized when the model drops them. Previous caller-side copies or allocations are outside this guarantee.
- Text String values contain non-ASCII Unicode characters and are not Unicode-normalized.
- Byte String values may be empty. Big Integer stores the exact Item Value octets without mathematical-integer normalization; empty values and alternate sign-extension encodings are accepted by this in-memory model, while TTLV wire validity is established by a separate codec feature.
- Diagnostic formatting of any public leaf value, `ValueView`, item, or nested Structure does not reveal contained values; error formatting does not include values or raw message bodies.

## Requirements

### Functional Requirements

- **KMIPKIT-0004-FR-001**: The generic value model MUST represent every Item Type assigned by KMIP 2.1 §11.23: Structure, Integer, Long Integer, Big Integer, Enumeration, Boolean, Text String, Byte String, Date Time, Interval, and Date Time Extended.
- **KMIPKIT-0004-FR-002**: A Raw Tag MUST retain an unsigned 24-bit value and reject values outside that range rather than truncate them. Every public generic item MUST expose a tag whose KMIP 2.1 allocation has been checked; this check does not establish TTLV wire or protocol validity.
- **KMIPKIT-0004-FR-003**: The model MUST distinguish Raw Tag from allocation-checked Tag. A Raw Tag MUST retain all 24 bits without claiming validity. A public generic item or Structure MUST accept only an allocation-checked Tag that passes the KMIP 2.1 catalog-backed tag-allocation gate. The gate MUST accept individually listed assigned values and the §11.56 `0x540000–0x54FFFF` Extensions range, MUST reject individually listed Reserved values and unused/reserved residual ranges, and MUST treat an exact individual catalog entry as authoritative over an aggregate range entry. Unknown extension tags MUST remain usable without a known-tag lookup; other unknown or prohibited values remain representable only as Raw Tag. This is an accepted KMIPKit project policy recorded in ADR-0010, not an OASIS clarification.
- **KMIPKIT-0004-FR-004**: Enumeration values MUST be retained as their full unsigned 32-bit value, including values not assigned in this KMIP 2.1 catalog, without asserting that an unknown value is valid for wire use. Integer and Long Integer values MUST retain their exact signed values and bit patterns; Interval MUST retain its unsigned 32-bit value.
- **KMIPKIT-0004-FR-005**: Big Integer values MUST retain the exact Item Value octets, including sign-extension octets, without mathematical-integer normalization; an empty octet sequence is permitted in this in-memory model and MUST NOT be claimed wire-valid by this feature. Byte String values MUST retain every byte. Text String values MUST retain their Unicode text without normalization.
- **KMIPKIT-0004-FR-006**: A Structure MUST retain all children in caller-specified order, including repeated Tag values. The generic model MUST NOT apply operation-specific field ordering, cardinality, or semantic validation.
- **KMIPKIT-0004-FR-007**: Boolean values MUST be represented as False or True. Date Time and Date Time Extended values MUST retain their signed 64-bit representation without timezone conversion or precision reduction.
- **KMIPKIT-0004-FR-008**: The value variant MUST determine the Item Type so a generic item cannot pair a known Item Type with a mismatched value variant.
- **KMIPKIT-0004-FR-009**: Default Debug formatting for every public payload-bearing value type and variant, `ValueView`, `StructureView`, generic item, or nested Structure MUST NOT reveal Text String contents, Byte String or Big Integer bytes, Boolean values, numeric payloads, Date Time values, or nested item values. It MAY expose Tag, Item Type, and Structure child counts. Explicit closure-scoped value exposure remains available to callers.
- **KMIPKIT-0004-FR-010**: Public `Value`, `ValueView`, `StructureView`, `Item`, and `Structure` types MUST NOT implement `serde::Serialize`. KMIPKit defines no other automatic payload serializer for this model; adding another serializer later requires a separate reviewed specification. Explicit TTLV encoding by a caller-invoked codec is outside this feature. Any provided Display formatting MUST redact payloads; otherwise payload-bearing types MUST have no Display implementation. Model-local error formatting MUST NOT expose value contents. This in-memory model MUST add no payload-bearing logging call sites; raw KMIP body redaction is verified in transport/client specifications.
- **KMIPKIT-0004-FR-011**: Every applicable normative protocol requirement MUST link to the exact OASIS document and clauses. Every KMIPKit design or security requirement MUST link to its governing project authority. Before implementation completion, every in-scope requirement MUST also link to its implementation location and executable verification.
- **KMIPKIT-0004-FR-012**: Every payload-bearing value, including values nested in Structures, MUST use a dedicated secret type that cannot be formatted or automatically serialized with its contents, and public owning value/item/tree types MUST NOT implement `Clone` or `Copy`. Value exposure MUST lend borrowed views through an explicit closure-scoped operation; borrowed references MUST NOT escape the callback, while explicit caller copies remain permitted. Secret payloads MUST remain in stable owned allocations when a Structure's child vector grows. Drop MUST zeroize current KMIPKit-owned payload storage, including the backing capacity of owned String and byte buffers and nested values, before release. Documentation MUST state that caller copies, caller-side prior allocations/reallocations, temporary stack/register copies, and managed-language runtime copies cannot be cleared by KMIPKit.
- **KMIPKIT-0004-FR-013**: Errors created by the TTLV value model MUST use safe, value-free diagnostics and MUST NOT expose payload contents. Because `kmipkit-ttlv` is below protocol/client crates, its model error type MUST remain local and MUST NOT introduce a dependency cycle; higher layers may map or wrap it through the shared error model.
- **KMIPKIT-0004-FR-014**: A generic Structure tree MUST NOT exceed 64 nested Structure levels. Construction that would exceed this limit MUST fail with a model-local, payload-free error before the tree is accepted. This is a KMIPKit in-memory safety limit, not an OASIS protocol limit.

### Normative Traceability

The OASIS clauses define the protocol representation; the generic in-memory model and preservation behavior below are KMIPKit requirements layered on those definitions.

| Requirement ID | OASIS source and clause | Normative statement or definition | KMIPKit behavior | Related functional requirement(s) |
|---|---|---|---|---|
| KMIPKIT-0004-NR-001 | OASIS KMIP Specification v2.1, §10.1.1 Tag | An Item Tag is an unsigned three-byte binary integer transmitted big-endian. | Store the complete 24-bit Raw Tag value without truncation; validate a tag before it can be used by a public generic item. Wire encoding is outside this feature. | FR-002 |
| KMIPKIT-0004-NR-002 | OASIS KMIP Specification v2.1, §§10.1.2 and 11.23 | Item Type is a one-byte code; §11.23 assigns the KMIP 2.1 item types and §10.1.2 describes their value representations. | Represent all eleven assigned item types as distinct generic values. | FR-001, FR-007, FR-008 |
| KMIPKIT-0004-NR-003 | OASIS KMIP Specification v2.1, §10.1.2, Enumeration | Enumeration is a four-byte unsigned integer; permitted extension values use the high nibble 8. | Preserve the full raw unsigned value and do not coerce unknown values to a known enumeration. | FR-004 |
| KMIPKIT-0004-NR-004 | OASIS KMIP Specification v2.1, §§12.1–12.3 Bit Masks | The specification defines KMIP bit masks as sets of bit values. | Keep generic Integer bit patterns unchanged; this model does not assign operation-specific meaning to mask bits. | FR-004 |
| KMIPKIT-0004-NR-005 | OASIS KMIP Specification v2.1, Chapter 11 introduction and §11.56 Tag Enumeration | Implementations SHALL NOT use Tag Values marked Reserved (Chapter 11 introduction). §11.56 states that Tags SHALL begin with 42 or 54 hex in the first byte; 42 identifies specification tags and 54 identifies extensions. | The public in-memory model checks tag allocation only; this does not establish TTLV wire or protocol validity. A Raw Tag remains separate and cannot be inserted into a generic item until it passes the tag-allocation gate. | FR-002, FR-003 |
| KMIPKIT-0004-NR-006 | OASIS KMIP Specification v2.1, §§10.1.2–10.1.5 | Type-specific value representations, lengths, and padding define item-level TTLV representation constraints; §10.1.2 requires fields in Specification-defined Structures to be encoded in their described order. | This model preserves semantic values and Big Integer Item Value octets but does not preserve framing or arbitrary original padding. It preserves caller order and does not validate schema-specific field order; protocol validation must establish known Structure order before transmission. Encoder/decoder conformance is a later feature. | FR-005, FR-006 |
| KMIPKIT-0004-NR-007 | OASIS KMIP Specification v2.1, §10.1.2 Type | Integer is a 32-bit signed two's-complement value; Long Integer is a 64-bit signed two's-complement value; Date Time and Date Time Extended are 64-bit signed values; Interval is a 32-bit unsigned value. The clause defines their wire representations as big-endian. | Preserve each numeric value at its defined signedness and width without conversion or precision loss. This requirement traces value semantics only; byte encoding remains outside this feature. | FR-004, FR-007 |

Normative source: the immutable local copy at specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html. Unknown-tag, unknown-enumeration, raw-bit preservation, value-redaction, and generic-tree behavior are KMIPKit requirements; they are not presented as additional OASIS requirements.

### Project Policy Traceability

| Requirement ID | Governing project authority | Application |
|---|---|---|
| KMIPKIT-0004-FR-002, FR-003, FR-004, FR-005, FR-006, FR-007 | `AGENTS.md` §9 “Public API and compatibility”; this specification, bounded by OASIS sources in NR-001, NR-003–NR-007 | Keep raw and allocation-checked tags distinct; preserve unknown enum values, bit patterns, and exact payloads; preserve caller order without claiming schema-specific protocol validity. |
| KMIPKIT-0004-FR-008 | `docs/architecture/public-api.md`, sections “Typed protocol” and “Generic TTLV” | Keep the generic public API structurally typed and prevent a known Item Type/value mismatch. |
| KMIPKIT-0004-FR-009, FR-010 | `AGENTS.md` §8 “Security invariants” and §9 “Public API and compatibility” | Redact secrets and message contents from formatting, logs, serialization, and errors; retain explicit value access. |
| KMIPKIT-0004-FR-012 | `AGENTS.md` §8 “Security invariants”; `docs/architecture/public-api.md`, “Secrets” | Use dedicated secret types, closure-scoped access, and zeroization of KMIPKit-owned memory. |
| KMIPKIT-0004-FR-013 | `.specify/memory/constitution.md`, Principle III “One core, explicit language boundaries”; `docs/adr/0003-layered-cargo-workspace.md` | Keep the TTLV model error local to the lower layer and avoid protocol/client dependency cycles while preserving safe diagnostics. |
| KMIPKIT-0004-FR-011 | `.specify/memory/constitution.md`, Principle I “Specification and traceability”; `AGENTS.md` §5 “OASIS requirements and conformance” | Keep normative and project-policy requirements traceable through implementation and executable verification. |
| KMIPKIT-0004-FR-014 | `AGENTS.md` §8 “Security invariants”; `.specify/memory/constitution.md`, Principle IV “Secure defaults and lossless protocol handling” | Bound in-memory nesting to prevent unbounded recursive formatting and zeroization work; this 64-level bound is a KMIPKit safety limit, not an OASIS requirement. |

For the accepted tag-allocation gate, KMIPKit uses the exact per-value allocation in the normative catalog when one exists: individually listed assigned tags are accepted and individually listed Reserved tags are rejected. The aggregate `420XXX – 42FFFF` row in §11.56 applies only to residual values without an individual entry, so it does not invalidate individually listed assigned values. The §11.56 `540000 – 54FFFF` Extensions range is accepted at this allocation gate; passing the gate does not validate extension semantics or context. Other unused or reserved allocations are rejected. This is accepted KMIPKit project policy recorded in ADR-0010, not an OASIS clarification. The separate open inventory discrepancy KMIPKIT-DISC-037 concerns decoder behavior when a reserved tag is received and remains outside this in-memory model feature.

### Accepted Project Decision: Overlapping Tag Allocation Notation

The pinned OASIS KMIP Specification v2.1 OASIS Standard dated 14 December 2020 (`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`) has two normative statements relevant to the gate: the Chapter 11 introduction says implementations SHALL NOT use tag values marked Reserved, while §11.56 lists individually assigned tags such as `0x420174`–`0x420176` and also labels `420XXX – 42FFFF` as Reserved. The aggregate range numerically overlaps those individual assignments. The document does not explicitly state which entry has precedence.

The observable alternatives are:

1. **Individual entries take precedence**: accept individually assigned tags, reject individually listed Reserved tags, and treat the remainder of the aggregate range as Reserved. This preserves the tags that §11.56 explicitly assigns for standard KMIP items.
2. **The aggregate range takes precedence**: reject every tag in `420XXX – 42FFFF`, including values that §11.56 also lists individually as assigned. This would make those standard tag assignments unusable in the public generic model.

KMIPKit accepts the first interpretation as project policy: exact per-tag entries control, and the aggregate range applies only to residual values without an individual catalog entry. This gives effect to the assigned tags while retaining the residual Reserved range. It is not an OASIS resolution and changes no normative requirement. The decision is recorded in accepted ADR-0010. The implementation must include table-driven tests for assigned, individually Reserved, residual Reserved, and extension-range values.

## Key Entities

- **Generic TTLV item**: One item with an allocation-checked Tag and a secret-wrapped value whose variant identifies its Item Type. This in-memory check does not validate encoded lengths, padding, or protocol semantics.
- **Raw Tag**: An unsigned 24-bit value preserving all bits without asserting OASIS validity. It is not interchangeable with an allocation-checked Tag and cannot be used to construct a public generic item until it passes the tag-allocation gate.
- **Allocation-checked Tag**: A Tag whose allocation is accepted by the catalog-backed gate defined by accepted ADR-0010. This says nothing about TTLV wire validity, extension semantics, or protocol-level Structure ordering.
- **Generic value**: One of the eleven KMIP 2.1 Item Type values, preserving its type-appropriate value.
- **Structure**: An ordered sequence of generic items; repeated Tag values are permitted.
- **Structure depth**: At most 64 nested Structure levels. This is a KMIPKit model safety bound and not a protocol constraint.
- **Numeric value**: An exact Integer, Long Integer, Enumeration, or Interval value, with Enumeration and bitmask bits retained without lookup.
- **Big Integer value**: Its two’s-complement octets as represented by the Item Value, including sign-extension octets.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0004-SC-001**: Applications can construct and inspect all eleven KMIP 2.1 Item Types with no type/value mismatch.
- **KMIPKIT-0004-SC-002**: 100% of constructed Raw Tags, allocation-checked Tags, Integer, Long Integer, Big Integer, Enumeration, Boolean, Text String, Byte String, Date Time, Date Time Extended, and Interval values are returned unchanged during closure-scoped exposure; tags that fail allocation checks cannot be inserted into a generic item.
- **KMIPKIT-0004-SC-003**: 100% of Structure children, including duplicate Tags, are returned in caller-specified order.
- **KMIPKIT-0004-SC-004**: No payload sentinel from public `Value`, `ValueView`, `StructureView`, `Item`, `Structure`, or model errors appears in default Debug, any provided Display, or model-error formatting, whether formatted directly or through a containing item or nested Structure. Compile-fail API checks confirm these public payload surfaces do not implement `serde::Serialize`. The model adds no logging call sites.
- **KMIPKIT-0004-SC-005**: 100% of in-scope FR and NR identifiers link to their applicable OASIS or project authority, implementation location, and executable verification before implementation completion.
- **KMIPKIT-0004-SC-006**: Every payload-bearing value variant is held by a dedicated zeroizing secret type in a stable allocation. A safe `Zeroize` spy confirms the secret wrapper's Drop path invokes the trait; separate live-object unit tests confirm each payload variant and nested Structure zeroizes its fields. A source review of the exact locked dependency confirms current String/Vec backing-capacity behavior before release, and a safe address-stability test confirms adding Structure children does not relocate existing payload storage. Compile-fail API tests confirm borrowed references cannot outlive the closure scope; runtime tests confirm explicit caller copies are possible; documentation states that caller copies, historical caller allocations, and runtime copies cannot be cleared by KMIPKit.
- **KMIPKIT-0004-SC-007**: Structures at depths 1 through 64 can be constructed and inspected; an attempted 65th nested Structure is rejected without disclosing payloads or overflowing the call stack.

## Assumptions

- KMIPKit 1.0 uses KMIP 2.1 and TTLV only.
- The KMIPKIT-0003 implementation PR, including the shared result/error contract, is merged before implementation of this feature; the generic TTLV crate remains below protocol and transport layers. The TTLV crate defines its own safe model errors because it cannot depend upward on protocol/client crates.
- The model is an in-memory representation. It does not encode or decode TTLV. It checks 24-bit tag representation, the accepted KMIPKit tag-allocation policy, the relationship between value variants and Item Types, and a KMIPKit maximum Structure depth of 64; it does not validate encoded lengths, padding, a KMIP-defined Structure's field order/cardinality, or operation semantics. A generic tree alone cannot be claimed TTLV wire-valid or protocol-valid. Higher-level protocol/client validation must establish these properties before transmission as a known KMIP Structure. A follow-on codec specification covers wire framing, exact lengths, padding, and decoder limits.
- The future encoder specification will apply the accepted tag-allocation policy together with the exact length, endianness, and padding requirements from §§10.1.1–10.1.5. Decoder handling of received Reserved tags remains open under KMIPKIT-DISC-037; this feature does not prescribe whether a decoder rejects such a tag or preserves it through a separate raw representation. A decoded value may enter this public generic tree only after it satisfies the tree's tag-allocation gate.
- The model does not convert Enumeration values into closed Rust enums or interpret bitmask semantics. Only the eleven Item Types assigned in §11.23 are in scope; an unrecognized Item Type code has no defined KMIP 2.1 value semantics and is not represented by this feature.
- Secret-value handling MUST use a private project wrapper backed by the `zeroize` crate's safe `Zeroize` trait and Drop path. Pin the reviewed dependency through Cargo.lock; do not enable derive or serialization features unless a separate documented need is accepted. The selected version and license/MSRV compatibility are recorded in research.md and verified by dependency/license checks.

## Exclusions

- TTLV byte encoding, decoding, framing, wire padding, and decoder resource limits.
- Validation of KMIP-defined Structure field order, KMIP operation fields, profiles, or object semantics.
- Message headers, batch processing, request correlation, transport, TLS, or client execution.
- C ABI, Java JNI, Python CFFI, and high-level operation APIs.
- Opaque items carrying an unrecognized Item Type code.
- JSON or XML encoding.
