# Feature Specification: KMIP TTLV Wire Codec

**Feature Branch**: `feature/KMIPKIT-0005-ttlv-wire-codec`
**Created**: 2026-10-04
**Status**: Draft — KMIPKIT-0004 is merged into `release/1.0.0` at `cf6c4c0d87c4de7dc159aba046a8fe5638ccc6bf`; implementation remains gated on approval of this feature and the proposed Reserved-tag policy in ADR-0011.
**Input**: KMIPKit roadmap: implement the strict TTLV encoder and decoder with bounded resource use after the generic TTLV value model.

## Clarification Record

This specification uses the pinned local OASIS KMIP Specification v2.1 source. It does not fetch or modify upstream material.

- **Normative padding interpretation**: §10.1.3 defines Item Length as the Item Value length. §10.1.5 explicitly excludes padding from Item Length for Integer, Enumeration, Text String, Byte String, and Interval; Structure Item Length includes all child encodings and their padding. §10.1.2 explicitly includes Big Integer sign-extension padding in Item Length. The decoder therefore computes these cases separately. OASIS does not assign a required byte value to the four-byte or trailing string/byte padding; the decoder accepts any padding octets of the required length, while the encoder emits zero octets for deterministic canonical output. This zero-fill is a KMIPKit canonicalization rule, not an OASIS requirement.
- **Open reserved-tag policy**: `KMIPKIT-DISC-037` remains open in `specification/catalog/kmip-2.1.json`. It lists two alternatives for a received Reserved tag: reject it, or preserve it through a separate opaque representation. The 004 public generic tree accepts allocation-checked Tags and does not define an opaque Reserved-tag node. This specification proposes rejection in ADR-0011 but does not claim an OASIS interpretation. The decision remains an implementation blocker until the proposed ADR is reviewed.
- **Depth configuration boundary**: security defaults are 16 MiB per message, 64 Structure levels, and 100,000 elements. The existing 004 model caps constructed trees at 64 levels. The codec limit may be configured downward; callers cannot raise it above 64 under this feature.
- **Big Integer empty value**: OASIS §10.1.2 requires a Big Integer to be represented as a big-endian two's-complement byte sequence and requires its length to be a multiple of eight, but does not state a minimum length. KMIPKit will reject a zero-length Big Integer as a project validity rule because an empty octet sequence cannot represent a two's-complement integer. This is not presented as an explicit OASIS minimum-length clause.
- **Item Length representability**: Every Item Length field is unsigned 32-bit under §10.1.3. A raised per-call message limit never permits any individual Item Value to exceed `u32::MAX`; implementation must check this independently of platform size and caller limits.
- **Dependency**: KMIPKIT-0004 implementation PR #14 has landed in `release/1.0.0` at `cf6c4c0d87c4de7dc159aba046a8fe5638ccc6bf`. This branch is updated from that release head and the public `Item`, `Structure`, `Tag`, and `Value` API is available for contract review.

## User Scenarios & Testing

### User Story 1 — Encode generic KMIP values as TTLV (Priority: P1)

A KMIP client developer needs to turn a valid generic TTLV item into the standard binary representation so requests can be sent by a transport. Encoding must use the item's tag and type, the correct length rules, and the value representation for all eleven KMIP 2.1 Item Types. Structure children remain in the order already represented by the model.

**Why this priority**: No client request can be transmitted as KMIP TTLV until a complete, deterministic encoder exists.

**Independent Test**: Encode table-driven known vectors for each assigned Item Type and compare every output byte with a reviewed expected vector derived from OASIS v2.1.

**Acceptance Scenarios**:

1. **Given** any of the eleven assigned Item Types, **When** a generic item is encoded, **Then** the 3-byte Tag, 1-byte Item Type, and 4-byte Item Length precede the correctly represented value in network byte order.
2. **Given** a Structure with nested, repeated, or caller-ordered children, **When** it is encoded, **Then** every child appears once and in the model's original order, and the Structure length includes the complete child encodings.
3. **Given** a Big Integer whose byte count is not a multiple of eight, **When** it is encoded, **Then** the minimum leading sign-extension bytes are added and counted in the Item Length.
4. **Given** an Integer, Enumeration, or Interval, **When** it is encoded, **Then** exactly four padding bytes follow the value and are excluded from Item Length.
5. **Given** a Text String or Byte String, **When** it is encoded, **Then** the minimum number of following padding bytes aligns the complete item to an eight-byte boundary and those bytes are excluded from Item Length.
6. **Given** a model tree that fails validation or size/limit preflight, **When** encoding fails, **Then** no value payload has been copied into an output buffer. Once payload copying begins, no fallible operation remains.

### User Story 2 — Decode bounded TTLV input into generic values (Priority: P1)

A KMIP client developer needs to parse a complete TTLV item returned by a server into a generic tree without losing supported tags, extension tags, unknown Enumeration values, Integer bit patterns, repeated children, or Structure order. The decoder must reject malformed data instead of constructing a partial public item.

**Why this priority**: A client must safely interpret successful, failed, and asynchronous KMIP responses while preserving values not yet understood by higher layers.

**Independent Test**: Decode reviewed OASIS vectors for all assigned Item Types, inspect the resulting generic values, and re-encode them to canonical TTLV bytes.

**Acceptance Scenarios**:

1. **Given** a valid single-item TTLV byte slice within configured limits, **When** it is decoded, **Then** one generic item is returned and no trailing bytes are silently ignored.
2. **Given** nested Structures, repeated tags, extension tags accepted by the generic model, unknown Enumeration values, or unknown mask bits, **When** decoded, **Then** their tag/value bits and child order remain unchanged in the generic model.
3. **Given** a truncated header or value, invalid fixed-width length, invalid UTF-8 Text String, invalid Boolean representation, inconsistent Structure boundary, unsupported Item Type code, or trailing bytes, **When** decoded, **Then** a payload-free decoding error is returned without a partial item.
4. **Given** an encoded message contains a Reserved tag, **When** decoded, **Then** behavior follows proposed ADR-0011, which rejects the tag before generic Item construction; this scenario remains gated on ADR review.

### User Story 3 — Bound work and memory for untrusted TTLV (Priority: P1)

A KMIP client developer needs per-call resource limits so malformed or hostile server responses cannot trigger excessive allocation or unbounded recursive work. Errors must identify the failure class and safe location metadata without disclosing input bytes or decoded payloads.

**Why this priority**: The decoder processes network-controlled bytes and is part of the library's security boundary.

**Independent Test**: Exercise each default limit at its boundary and one unit over, plus caller-lowered limits, and verify oversized declared lengths fail before allocating the declared body.

**Acceptance Scenarios**:

1. **Given** defaults are used, **When** an input exceeds 16 MiB, contains more than 64 nested Structure levels, or contains more than 100,000 items, **Then** decoding fails with a bounded resource-limit error.
2. **Given** a caller supplies lower per-call limits, **When** input exceeds one of those limits, **Then** decoding fails at the first violating boundary and leaves no partial public tree.
3. **Given** a header declares a length that overflows arithmetic, exceeds the remaining input, exceeds the configured message bound, or cannot fit a supported Item Type, **When** decoded, **Then** it is rejected before allocation based on that declared length.
4. **Given** any codec error is formatted, **When** Debug, Display, or source-chain diagnostics are used, **Then** raw TTLV bytes and payload contents are absent.

### Edge Cases

- A tag uses the first or last byte of the 24-bit Tag range; raw tag bytes are decoded big-endian.
- Item Length is zero for permitted empty Text String, Byte String, and Structure values. A zero-length Big Integer is rejected by the explicit KMIPKit project validity rule above; zero is also rejected for every fixed-width type.
- Structure children exactly fill, underfill, or overrun their parent Item Length.
- Text String padding is calculated from UTF-8 octet length, not character count; Text String value bytes must be valid UTF-8.
- Big Integer encoding rejects an empty in-memory value. For a non-empty value, padding count is zero when already aligned and otherwise is the minimum needed; padding is signed extension based on the first value octet.
- A value's logical length and its padding have different Item Length treatment by Item Type; String/Byte String and fixed 4-byte values exclude their padding, Big Integer and Structure include their specified bytes.
- Padding byte contents are not rejected solely for being nonzero because the cited OASIS clauses specify padding size/placement but not a byte value. The encoder emits zero octets for padding that is not part of the Big Integer value; the decoder may normalize accepted nonzero padding on re-encoding.
- Boolean accepts only the exact eight-byte encodings for False and True.
- Unsupported Item Type codes are rejected because the current generic model represents the eleven Item Types assigned by KMIP 2.1 only.
- The maximum configured Structure depth cannot exceed the 004 model's 64-level construction cap unless that model contract is separately changed.
- Reserved-tag receipt follows the rejection proposal in ADR-0011 and must not be conflated with preserving unknown Tags in the extension range.

## Requirements

### Functional Requirements

- **KMIPKIT-0005-FR-001**: The encoder MUST emit each Item as a 3-byte unsigned Tag, 1-byte Item Type, 4-byte unsigned Item Length, and Item Value, with numeric fields in big-endian order, as defined by OASIS KMIP Specification v2.1 §§10.1.1–10.1.4.
- **KMIPKIT-0005-FR-002**: The encoder MUST support all eleven Item Types assigned by OASIS KMIP Specification v2.1 §11.23 and MUST apply the type-specific representations in §10.1.2: signed 32-bit Integer, signed 64-bit Long Integer, big-endian two’s-complement Big Integer, unsigned 32-bit Enumeration, the exact eight-byte Boolean values, UTF-8 Text String, Byte String, signed 64-bit Date Time and Date Time Extended, and unsigned 32-bit Interval. It MUST apply the permitted Item Lengths in §10.1.3 and padding rules in §10.1.5. As a KMIPKit project validity rule, it MUST reject empty Big Integer values; OASIS does not state this minimum explicitly.
- **KMIPKIT-0005-FR-003**: The encoder MUST preserve the generic model's Structure child order and repeated Tags. It MUST NOT reorder children or claim schema-level validity; callers constructing a specification-defined Structure remain responsible for supplying fields in that structure's §10.1.2-defined order until typed protocol models enforce it.
- **KMIPKIT-0005-FR-004**: The decoder MUST decode one complete TTLV Item from the supplied byte slice and MUST reject trailing bytes rather than silently ignore or concatenate them.
- **KMIPKIT-0005-FR-005**: The decoder MUST preserve values and ordering represented by the generic model, including accepted extension Tags, unknown Enumeration values, unknown Integer bits, repeated child Tags, and exact Big Integer Item Value octets. It MUST reject unsupported Item Type codes because the 004 model has no representation for them.
- **KMIPKIT-0005-FR-006**: The decoder MUST validate header completeness, type-specific Item Length constraints (including rejection of an empty Big Integer under the stated KMIPKit project validity rule), UTF-8 validity, Boolean encodings, Structure boundaries, padding counts, and checked length arithmetic before accepting a generic item. It MUST not interpret reserved padding byte contents as invalid where OASIS does not specify a required value.
- **KMIPKIT-0005-FR-007**: Encoding and decoding MUST enforce default per-message byte, Structure-depth, and item-count limits of 16 MiB, 64 levels, and 100,000 items respectively. A caller MUST be able to configure per-call limits without global mutable state. Message and item-count limits may be raised or lowered subject to representable/API limits; depth may be lowered but MUST NOT exceed the 004 model's 64-level construction bound. Regardless of caller limits, each Item Value length MUST fit the unsigned 32-bit Item Length field.
- **KMIPKIT-0005-FR-008**: The decoder MUST check declared lengths and cumulative Structure lengths against the U32 wire representation, available bytes, parent boundaries, and configured limits before reserving or allocating storage based on those lengths.
- **KMIPKIT-0005-FR-009**: Codec errors MUST preserve useful source/location context while never formatting, logging, or exposing raw TTLV bodies or value payloads.
- **KMIPKIT-0005-FR-010**: The decoder MUST reject a received Tag classified as Reserved under §11.56 before constructing a public generic Item, as proposed in ADR-0011. This project decision remains gated on review/acceptance of that ADR and is not an OASIS clarification. Other Tags rejected by the 004 allocation-checked Tag API MUST return an error and MUST NOT enter the public generic tree.
- **KMIPKIT-0005-FR-011**: Every normative requirement applicable to this codec MUST be linked to its exact OASIS source, stable catalog/project requirement ID, implementation location, and executable verification before feature completion. The schema-order catalog requirement `KMIPKIT-REQ-SPEC-10.1.2-001` is not implemented by generic order preservation alone. It MUST remain open until every applicable client 1.0 Structure is assigned to an approved typed protocol specification and has implementation plus executable order-verification references in the catalog; merely naming future specifications is insufficient. This global traceability gate does not block implementation of the generic codec, which preserves caller order but cannot validate operation schemas.
- **KMIPKIT-0005-FR-012**: The encoder MUST validate the complete Item tree, calculate all lengths and enforce limits, and reserve the complete output capacity before copying any value payload. After the first payload byte is copied, encoding MUST have no fallible exit. If implementation cannot guarantee that invariant, every error path MUST zeroize the partial output buffer before releasing it.

### Normative Traceability

| Requirement ID | OASIS source and clause | Normative statement or definition | Codec coverage and verification |
|---|---|---|---|
| KMIPKIT-0005-NR-001 | OASIS KMIP Specification v2.1, §10.1.1 | Tag is a three-byte unsigned integer transmitted big-endian. | Encode/decode vectors at 24-bit boundaries; implementation/test links are added by the implementation PR. |
| KMIPKIT-0005-NR-002 | OASIS KMIP Specification v2.1, §§10.1.2 and 11.23 | Item Type byte selects the defined value representation; all fields of a specified Structure are encoded in their definition order. | Vectors cover all eleven types; generic codec preserves caller order. Schema-order enforcement is not claimed here. Catalog requirement `KMIPKIT-REQ-SPEC-10.1.2-001` remains unassigned until every applicable client 1.0 Structure has approved typed-spec ownership, implementation, and executable order-verification references; planned specification names alone do not complete coverage. |
| KMIPKIT-0005-NR-003 | OASIS KMIP Specification v2.1, §10.1.2 | Big Integer is a big-endian two's-complement byte sequence; if not a multiple of eight bytes it receives the minimum leading sign-extended padding, included in Item Length. | Vectors cover positive/negative sign extension and aligned values. Catalog IDs: `KMIPKIT-REQ-SPEC-10.1.2-002-001`, `KMIPKIT-REQ-SPEC-10.1.2-002-002`. |
| KMIPKIT-0005-NR-004 | OASIS KMIP Specification v2.1, §10.1.3 | Item Length is a 32-bit big-endian value counting Item Value bytes; allowed lengths depend on Item Type. | Exact length vectors, fixed-width invalid-length negatives, overflow and boundary tests. |
| KMIPKIT-0005-NR-005 | OASIS KMIP Specification v2.1, §10.1.5 | Structure length includes encoded sub-items and padding; Integer, Enumeration, Text String, Byte String, and Interval lengths exclude their following padding; string/byte padding is minimal and 4-byte values receive four following padding bytes. | Exact byte vectors and decoder boundary tests. Catalog IDs: `KMIPKIT-REQ-SPEC-10.1.5-001-001`, `KMIPKIT-REQ-SPEC-10.1.5-001-002`. |
| KMIPKIT-0005-NR-006 | OASIS KMIP Specification v2.1, Chapter 11 introduction and §11.56 | Implementations SHALL NOT use Tags marked Reserved; §11.56 assigns the 0x42 and 0x54 prefixes to specification and extension Tags. | Encoder uses the checked Tag policy from 004 and MUST NOT emit a Reserved Tag. Inbound Reserved-tag disposition follows proposed ADR-0011; other allocation-rejected Tags are rejected by the checked Tag API. Catalog IDs `KMIPKIT-REQ-SPEC-11-001` and `KMIPKIT-REQ-SPEC-11.56-001`. |

### Key Entities

- **Codec Limits**: Per-call byte, nesting-depth, and item-count maxima used to bound encoding and decoding.
- **Encoded Item**: The canonical byte sequence for one generic Item, including its fixed header and type-specific padding.
- **Decoded Item**: One generic Item tree produced only after the entire input slice passes framing, type, length, value, padding-count, and resource checks.
- **Codec Error**: A payload-free error class with safe offset/tag/type context where available.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0005-SC-001**: Golden vectors cover 100% of the eleven assigned KMIP 2.1 Item Types and compare exact Tag, Type, Length, Value, and padding bytes.
- **KMIPKIT-0005-SC-002**: Every canonical golden vector decodes into the expected generic value and re-encodes to the same canonical TTLV bytes. Noncanonical padding octets may be normalized to the encoder’s zero-filled padding.
- **KMIPKIT-0005-SC-003**: Every listed malformed-input category has a negative test that returns a structured error without panic, partial item, or payload disclosure.
- **KMIPKIT-0005-SC-004**: Exact-boundary and one-over tests cover default and configured message-byte, Structure-depth, and item-count limits; declared oversized values are rejected before body-sized allocation.
- **KMIPKIT-0005-SC-005**: Changed codec and protocol/model lines meet the repository's 95% line-coverage gate, and all in-scope normative requirements have implementation and executable-verification links before the implementation PR is ready.

## Assumptions

- The only 1.0 wire encoding is TTLV; JSON and XML remain out of scope.
- The codec is a pure Rust library layer over the generic model from KMIPKIT-0004. It performs no socket, TLS, HTTP, or KMIP operation/schema work.
- One call decodes or encodes exactly one complete Item. Stream framing and transport reads remain in the transport layer.
- A decoded item is structurally valid TTLV under the codec's implemented Item Types, but this alone does not prove that its tag is valid in a particular operation or that a server accepts it.
- The generic value model's allocation-checked Tag and 64-level cap are upstream API contracts merged in PR #14.
- Canonical re-encoding is expected; byte-for-byte preservation is guaranteed only for fields the generic model represents, not for unsupported Item Types or discarded padding bytes.
- Decoder handling of Reserved tags follows the proposed project policy in ADR-0011; acceptance of that policy remains open.

## Implementation Gates

Do not start implementation until all of the following are true:

1. This feature specification and implementation plan are approved under repository governance.
2. Proposed ADR-0011 receives review/acceptance, resolving `KMIPKIT-DISC-037` by rejecting received Reserved Tags before generic model construction.
3. Reconfirm the accepted ADR-0010 Tag allocation policy and use the merged 004 public APIs.
4. `CodecLimits.max_structure_depth` remains configurable from 0 through the model's hard maximum of 64; no implementation may claim support above that limit without a separate reviewed change.
