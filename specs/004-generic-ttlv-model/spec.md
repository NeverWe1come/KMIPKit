# Feature Specification: KMIP Generic TTLV Value Model

**Feature Branch**: **feature/KMIPKIT-0004-generic-ttlv-model**

**Created**: 2026-10-04

**Status**: Draft — proposed scope and tag-range interpretation for maintainer review; implementation is not approved by this artifact.

**Input**: User roadmap: "lossless generic TTLV tree" within the KMIPKit 1.0 KMIP 2.1 client foundation.

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

**Independent Test**: Construct a Structure with repeated allocation-checked tags and unknown vendor-extension tags, unknown enumeration values, high-bit mask values, and out-of-order children; inspect it and verify exact tag values, payload values, and child order. Separately verify that arbitrary raw 24-bit tags are retained by the Raw Tag type but cannot enter a generic item unless they pass the tag-allocation check.

**Acceptance Scenarios**:

1. **Given** a 24-bit tag in the KMIP 2.1 vendor-extension range that is not known to this KMIPKit release, **When** an item is constructed and inspected, **Then** the tag value is returned exactly without truncation or replacement.
2. **Given** any 24-bit value, **When** it is represented as a Raw Tag, **Then** all 24 bits are retained, while constructing a public item from it requires successful allocation checking under the KMIP 2.1 tag catalog.
3. **Given** a tag explicitly listed as Reserved or inside an unused/reserved residual range, **When** it is used to construct a public item, **Then** construction fails; exact individually assigned tag entries take precedence over the aggregate reserved range notation in §11.56.
4. **Given** an unknown 32-bit Enumeration value or an Integer used as a bitmask, **When** the value is inspected, **Then** every bit is unchanged.
5. **Given** a Structure with repeated child tags and a caller-defined order, **When** its children are traversed, **Then** all children appear once in the original order.

### User Story 3 - Inspect generic values without leaking contents through default diagnostics (Priority: P1)

An application developer needs to use Debug output while working with generic KMIP values without exposing strings, byte strings, integer payloads, or nested content that may contain credentials, secret key material, tickets, or raw message data.

**Why this priority**: Generic values are untrusted and may contain secrets; default diagnostic formatting must not become a disclosure path.

**Independent Test**: Place distinct payload sentinels in every value family and nested Structure. Format each public value type and variant directly, as well as its containing item and tree, through default Debug, any provided Display, and representative model errors. Verify no payload sentinel appears while the item type, tag, and Structure child count remain available. Add a compile-time API check that the public model exposes no general-purpose serialization implementation that emits value contents, and Drop probes that verify KMIPKit-owned memory is zeroized for every variant and nested values.

**Acceptance Scenarios**:

1. **Given** any public generic value type or variant contains a payload, **When** it or a containing item/tree is formatted with default Debug, **Then** the payload is absent, including strings, bytes, Big Integer octets, booleans, and numeric values.
2. **Given** an application explicitly accesses a value through the public API, **When** it exposes the value through the closure-scoped secret API, **Then** the original value is available only for that controlled access scope without implicit formatting or serialization.
3. **Given** a public value type has a Display implementation, **When** it is formatted, **Then** its payload remains redacted; otherwise the type has no Display implementation. No automatic serialization implementation emits generic value contents.
4. **Given** a model error is produced while constructing or inspecting an item/tree, **When** its available diagnostic formatting is used, **Then** it does not reveal payload sentinels or raw KMIP message bodies.
5. **Given** an application constructs or inspects values while library logging is enabled, **When** KMIPKit emits log output, **Then** no value payload or raw KMIP message body appears in that output.
6. **Given** a generic value owns secret payload data, **When** the value or containing Structure is dropped, **Then** all payload memory owned by KMIPKit is zeroized before deallocation.

### Edge Cases

- A Raw Tag is the maximum representable 24-bit value.
- An unknown tag value is retained by Raw Tag without truncation; representing it there does not assert that it is valid or permitted in a public generic tree or on the KMIP wire.
- An unknown vendor-extension tag in the range designated by KMIP 2.1 is permitted in a generic item after allocation checking; an arbitrary raw tag must pass the catalog-backed allocation check before item construction.
- Enumeration values use the full 32-bit unsigned range, including extension values.
- Integer bitmasks set the highest bit and several unknown bits.
- A Structure is empty, contains repeated tags, or contains nested Structures.
- Any payload-bearing variant may hold secret material; nested secret values are cleared when the model drops their KMIPKit-owned memory.
- Text String values contain non-ASCII Unicode characters and are not Unicode-normalized.
- Byte String and Big Integer values are empty or contain leading zero/sign-extension octets; codec validity rules are specified separately.
- Diagnostic formatting of any public leaf value, value enum, item, or nested Structure does not reveal contained values; error formatting does not include values or raw message bodies.

## Requirements

### Functional Requirements

- **KMIPKIT-0004-FR-001**: The generic value model MUST represent every Item Type assigned by KMIP 2.1 §11.23: Structure, Integer, Long Integer, Big Integer, Enumeration, Boolean, Text String, Byte String, Date Time, Interval, and Date Time Extended.
- **KMIPKIT-0004-FR-002**: A Raw Tag MUST retain an unsigned 24-bit value and reject values outside that range rather than truncate them. Every public generic item MUST expose a tag whose KMIP 2.1 allocation has been checked; this check does not establish TTLV wire or protocol validity.
- **KMIPKIT-0004-FR-003**: The model MUST distinguish Raw Tag from allocation-checked Tag. A Raw Tag MUST retain all 24 bits without claiming validity. A public generic item or Structure MUST accept only an allocation-checked Tag that passes the KMIP 2.1 catalog-backed tag-allocation gate. The gate MUST accept individually listed assigned values and the OASIS vendor-extension range, MUST reject individually listed Reserved values and unused/reserved residual ranges, and MUST treat an exact individual catalog entry as authoritative over an aggregate range entry. Unknown vendor-extension tags MUST remain usable without a known-tag lookup; other unknown or prohibited values remain representable only as Raw Tag. This range interpretation is proposed for maintainer review and is not an OASIS clarification.
- **KMIPKIT-0004-FR-004**: Enumeration values MUST be retained as their full unsigned 32-bit value, including values not assigned in this KMIP 2.1 catalog, without asserting that an unknown value is valid for wire use. Integer and Long Integer values MUST retain their exact signed values and bit patterns; Interval MUST retain its unsigned 32-bit value.
- **KMIPKIT-0004-FR-005**: Big Integer values MUST retain their exact two’s-complement byte sequence, including any sign-extension octets that are part of the Item Value. Byte String values MUST retain every byte. Text String values MUST retain their Unicode text without normalization.
- **KMIPKIT-0004-FR-006**: A Structure MUST retain all children in caller-specified order, including repeated Tag values. The generic model MUST NOT apply operation-specific field ordering, cardinality, or semantic validation.
- **KMIPKIT-0004-FR-007**: Boolean values MUST be represented as False or True. Date Time and Date Time Extended values MUST retain their signed 64-bit representation without timezone conversion or precision reduction.
- **KMIPKIT-0004-FR-008**: The value variant MUST determine the Item Type so a generic item cannot pair a known Item Type with a mismatched value variant.
- **KMIPKIT-0004-FR-009**: Default Debug formatting for every public payload-bearing value type and variant, generic item, or nested Structure MUST NOT reveal Text String contents, Byte String or Big Integer bytes, Boolean values, numeric payloads, Date Time values, or nested item values. It MAY expose Tag, Item Type, and Structure child counts. Explicit closure-scoped value exposure remains available to callers.
- **KMIPKIT-0004-FR-010**: Public model types MUST NOT implement general-purpose automatic serialization that emits value contents or log those contents. Explicit TTLV encoding by a caller-invoked codec is outside this feature. Any provided Display formatting MUST redact payloads; otherwise payload-bearing types MUST have no Display implementation. Error formatting MUST NOT expose value contents or raw KMIP message bodies.
- **KMIPKIT-0004-FR-011**: Every applicable normative protocol requirement MUST link to the exact OASIS document and clauses. Every KMIPKit design or security requirement MUST link to its governing project authority. Before implementation completion, every in-scope requirement MUST also link to its implementation location and executable verification.
- **KMIPKIT-0004-FR-012**: Every payload-bearing value, including values nested in Structures, MUST use a dedicated secret type that cannot be formatted or automatically serialized with its contents. Value exposure MUST require an explicit closure-scoped operation. KMIPKit MUST zeroize all payload memory it owns when that value is dropped, including nested values. Documentation MUST state that copies made by callers or managed language runtimes cannot be cleared by KMIPKit.

### Normative Traceability

The OASIS clauses define the protocol representation; the generic in-memory model and preservation behavior below are KMIPKit requirements layered on those definitions.

| Requirement ID | OASIS source and clause | Normative statement or definition | KMIPKit behavior | Related functional requirement(s) |
|---|---|---|---|---|
| KMIPKIT-0004-NR-001 | OASIS KMIP Specification v2.1, §10.1.1 Tag | An Item Tag is an unsigned three-byte binary integer transmitted big-endian. | Store the complete 24-bit Raw Tag value without truncation; validate a tag before it can be used by a public generic item. Wire encoding is outside this feature. | FR-002 |
| KMIPKIT-0004-NR-002 | OASIS KMIP Specification v2.1, §§10.1.2 and 11.23 | Item Type is a one-byte code; §11.23 assigns the KMIP 2.1 item types and §10.1.2 describes their value representations. | Represent all eleven assigned item types as distinct generic values. | FR-001, FR-007, FR-008 |
| KMIPKIT-0004-NR-003 | OASIS KMIP Specification v2.1, §10.1.2, Enumeration | Enumeration is a four-byte unsigned integer; permitted extension values use the high nibble 8. | Preserve the full raw unsigned value and do not coerce unknown values to a known enumeration. | FR-004 |
| KMIPKIT-0004-NR-004 | OASIS KMIP Specification v2.1, §§12.1–12.3 Bit Masks | The specification defines KMIP bit masks as sets of bit values. | Keep generic Integer bit patterns unchanged; this model does not assign operation-specific meaning to mask bits. | FR-004 |
| KMIPKIT-0004-NR-005 | OASIS KMIP Specification v2.1, Chapter 11 introduction and §11.56 Tag Enumeration | Implementations SHALL NOT use Tag Values marked Reserved (Chapter 11 introduction). §11.56 states that Tags SHALL begin with 42 or 54 hex in the first byte; 42 identifies specification tags and 54 identifies extensions. | The public in-memory model checks tag allocation only; this does not establish TTLV wire or protocol validity. A Raw Tag remains separate and cannot be inserted into a generic item until it passes the tag-allocation gate. | FR-002, FR-003 |
| KMIPKIT-0004-NR-006 | OASIS KMIP Specification v2.1, §§10.1.2–10.1.5 | Type-specific lengths, value encoding, and padding rules define wire validity; §10.1.2 requires fields in Specification-defined Structures to be encoded in their described order. | This model preserves semantic values and Big Integer value octets but does not preserve framing or arbitrary original padding. It preserves caller order and does not validate schema-specific field order; protocol validation must establish known Structure order before transmission. Encoder/decoder conformance is a later feature. | FR-005, FR-006 |

Normative source: the immutable local copy at specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html. Unknown-tag, unknown-enumeration, raw-bit preservation, value-redaction, and generic-tree behavior are KMIPKit requirements; they are not presented as additional OASIS requirements.

### Project Policy Traceability

| Requirement ID | Governing project authority | Application |
|---|---|---|
| KMIPKIT-0004-FR-002, FR-003, FR-004, FR-005, FR-006 | `AGENTS.md` §9 “Public API and compatibility”; this specification, bounded by OASIS sources in NR-001, NR-003–NR-006 | Keep raw and allocation-checked tags distinct; preserve unknown enum values, bit patterns, and exact payloads; preserve caller order without claiming schema-specific protocol validity. |
| KMIPKIT-0004-FR-008 | `docs/architecture/public-api.md`, sections “Typed protocol” and “Generic TTLV” | Keep the generic public API structurally typed and prevent a known Item Type/value mismatch. |
| KMIPKIT-0004-FR-009, FR-010 | `AGENTS.md` §8 “Security invariants” and §9 “Public API and compatibility” | Redact secrets and message contents from formatting, logs, serialization, and errors; retain explicit value access. |
| KMIPKIT-0004-FR-012 | `AGENTS.md` §8 “Security invariants”; `docs/architecture/public-api.md`, “Secrets” | Use dedicated secret types, closure-scoped access, and zeroization of KMIPKit-owned memory. |
| KMIPKIT-0004-FR-011 | `.specify/memory/constitution.md`, Principle I “Specification and traceability”; `AGENTS.md` §5 “OASIS requirements and conformance” | Keep normative and project-policy requirements traceable through implementation and executable verification. |

For the proposed tag-allocation gate, KMIPKit uses the exact per-value allocation in the normative catalog when one exists: individually listed assigned tags are accepted and individually listed Reserved tags are rejected. The aggregate `420XXX – 42FFFF` row in §11.56 is proposed as the residual allocation for values without an individual entry, so it does not invalidate individually listed assigned values. The `540000 – 54FFFF` extension range is accepted; other unused or reserved allocations are rejected. This is a proposed KMIPKit interpretation for maintainer review, not an OASIS clarification. The separate open inventory discrepancy KMIPKIT-DISC-037 concerns decoder behavior when a reserved tag is received and remains outside this in-memory model feature.

## Key Entities

- **Generic TTLV item**: One item with an allocation-checked Tag and a secret-wrapped value whose variant identifies its Item Type. This in-memory check does not validate encoded lengths, padding, or protocol semantics.
- **Raw Tag**: An unsigned 24-bit value preserving all bits without asserting OASIS validity. It is not interchangeable with an allocation-checked Tag and cannot be used to construct a public generic item until it passes the tag-allocation gate.
- **Allocation-checked Tag**: A Tag whose allocation is accepted by the proposed catalog-backed gate. This says nothing about TTLV wire validity or protocol-level Structure ordering.
- **Generic value**: One of the eleven KMIP 2.1 Item Type values, preserving its type-appropriate value.
- **Structure**: An ordered sequence of generic items; repeated Tag values are permitted.
- **Numeric value**: An exact Integer, Long Integer, Enumeration, or Interval value, with Enumeration and bitmask bits retained without lookup.
- **Big Integer value**: Its two’s-complement octets as represented by the Item Value, including sign-extension octets.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0004-SC-001**: Applications can construct and inspect all eleven KMIP 2.1 Item Types with no type/value mismatch.
- **KMIPKIT-0004-SC-002**: 100% of constructed Raw Tags, allocation-checked Tags, Integer, Long Integer, Big Integer, Enumeration, Boolean, Text String, Byte String, Date Time, Date Time Extended, and Interval values are returned unchanged during closure-scoped exposure; tags that fail allocation checks cannot be inserted into a generic item.
- **KMIPKIT-0004-SC-003**: 100% of Structure children, including duplicate Tags, are returned in caller-specified order.
- **KMIPKIT-0004-SC-004**: No payload sentinel from any public value type or variant appears in default Debug, any provided Display, model-error formatting, or KMIPKit-emitted logs, whether formatted directly or through a containing item or nested Structure. The public model has no general-purpose automatic serialization that emits value contents.
- **KMIPKIT-0004-SC-005**: 100% of in-scope FR and NR identifiers link to their applicable OASIS or project authority, implementation location, and executable verification before implementation completion.
- **KMIPKIT-0004-SC-006**: Every payload-bearing value variant is held by a dedicated zeroizing secret type; Drop probes confirm KMIPKit-owned payload memory is cleared, including nested Structure values, compile-time API tests confirm exposed references cannot outlive the closure scope, and API documentation states that KMIPKit cannot clear copies retained by callers or managed language runtimes.

## Assumptions

- KMIPKit 1.0 uses KMIP 2.1 and TTLV only.
- The KMIPKIT-0003 shared result/error contract is merged before implementation of this feature; the generic TTLV crate remains below protocol and transport layers.
- The model is an in-memory representation. It does not encode or decode TTLV. It checks 24-bit tag representation, the proposed KMIP 2.1 tag allocation, and the relationship between value variants and Item Types; it does not validate encoded lengths, padding, a KMIP-defined Structure's field order/cardinality, or operation semantics. A generic tree alone cannot be claimed TTLV wire-valid or protocol-valid. Higher-level protocol/client validation must establish these properties before transmission as a known KMIP Structure.
- The future encoder specification will apply the same proposed KMIP 2.1 tag-allocation gate and the exact length, endianness, and padding requirements from §§10.1.1–10.1.5. Decoder handling of received Reserved tags remains open under KMIPKIT-DISC-037; this feature does not prescribe whether a decoder rejects such a tag or preserves it through a separate raw representation. A decoded value may enter this public generic tree only after it satisfies the tree's tag-allocation gate.
- The model does not convert Enumeration values into closed Rust enums or interpret bitmask semantics. Only the eleven Item Types assigned in §11.23 are in scope; an unrecognized Item Type code has no defined KMIP 2.1 value semantics and is not represented by this feature.
- Secret-value handling MUST reuse an approved project secret type and zeroization mechanism when available. A new direct dependency is allowed only when the implementation plan documents why it is needed for secure zeroization or the required API, with license and supply-chain checks.

## Exclusions

- TTLV byte encoding, decoding, framing, wire padding, and decoder resource limits.
- Validation of KMIP-defined Structure field order, KMIP operation fields, profiles, or object semantics.
- Message headers, batch processing, request correlation, transport, TLS, or client execution.
- C ABI, Java JNI, Python CFFI, and high-level operation APIs.
- Opaque items carrying an unrecognized Item Type code.
- JSON or XML encoding.
