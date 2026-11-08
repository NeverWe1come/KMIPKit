# Feature Specification: Typed KMIP Object Attributes

**Feature Branch**: `feature/KMIPKIT-0048-typed-attributes`  
**Created**: 2026-11-05  
**Status**: Approved by maintainer plan authorization  
**Input**: Approved Phase 5 plan, item 0048: typed construction and reading for catalogued KMIP 2.1 attributes through `AttributeSet`, `CurrentAttribute`, and `NewAttribute`.

## User Scenarios & Testing

### User Story 1 - Build and read standard attributes (Priority: P1)

As a Rust client author, I can construct and inspect standard KMIP attributes using attribute-specific types and value categories instead of manually matching raw tags and TTLV variants.

**Why this priority**: Object registration, retrieval, and attribute operations are central KMIP client workflows. A typed surface reduces tag/value mismatches while keeping the existing generic path available.

**Independent Test**: A Rust consumer constructs, inserts, retrieves, and reads examples from each catalogued TTLV value category; tests prove every supported direct attribute maps to its pinned tag and encoding.

**Acceptance Scenarios**:

1. **Given** a catalogued attribute definition, **When** the caller builds its typed value, **Then** the resulting Item uses the source-assigned tag and exact TTLV value type.
2. **Given** an `AttributeSet` with repeated instances permitted by its source table, **When** the caller reads that attribute, **Then** all instances are returned in wire order without deduplication.
3. **Given** a generic Item or an unknown assigned/extension attribute, **When** the caller uses existing generic APIs, **Then** its tag, value, order, and repetitions remain available unchanged.

### User Story 2 - Use typed attributes in operation wrappers (Priority: P1)

As a Rust client author, I can use a typed attribute with `CurrentAttribute` or `NewAttribute`, while retaining the existing generic constructors for compatibility and vendor extensions.

**Independent Test**: A Rust consumer wraps and reads typed Current/New Attributes for scalar and Structure values and verifies that invalid tag/type combinations fail without disclosing attribute contents.

**Acceptance Scenarios**:

1. **Given** a typed attribute value, **When** it is wrapped as Current or New Attribute, **Then** the wrapper contains exactly that direct attribute Item.
2. **Given** a wrapper made with the existing generic constructor, **When** a typed view is requested, **Then** it succeeds only for a matching known tag and TTLV encoding.

### Edge Cases

- Attributes with source-permitted repeated instances retain every occurrence and original order.
- Generic and typed `AttributeSet` collection APIs preserve all inserted duplicates; existing operation-level cardinality validation remains responsible for rejecting source-prohibited duplicates before use.
- Open Enumeration values preserve unknown and vendor values; typed convenience does not restrict them to known standard constants.
- A mismatched tag or TTLV type fails with a redacted, payload-free error.
- Complex Structure-valued attributes expose a typed Structure value category and retain generic child Items; this feature does not synthesize missing child fields or infer object/server policy.
- The normative catalog lists 63 attribute records, but “Certificate Attributes” (§4.6, Table 39) has no standalone TTLV tag or `source_encoding` and describes certificate contents. It is not a direct Attribute Item and is excluded from the typed Item API. The remaining 62 records have a direct assigned/special tag and a defined TTLV value category.

## Requirements

### Functional Requirements

- **KMIPKIT-0048-FR-001**: The Rust protocol API MUST expose a typed definition for each of the 62 catalogued direct Object Attributes with a concrete tag and `source_encoding`, including the separately encoded Vendor Attribute Structure.
- **KMIPKIT-0048-FR-002**: Each typed definition MUST map to the exact tag and TTLV value category in its pinned KMIP Specification v2.1 §4 attribute table. Enumeration values remain open raw `u32` values; this layer MUST NOT reject unknown or vendor values.
- **KMIPKIT-0048-FR-003**: `AttributeSet` MUST support typed insertion and reading for every definition. Reading MUST preserve all matching occurrences and their original order. Typed collection APIs MUST retain the existing lossless insertion behavior; existing operation-level cardinality validation remains authoritative. Generic insertion/iteration APIs remain available.
- **KMIPKIT-0048-FR-004**: `CurrentAttribute` and `NewAttribute` MUST support typed construction and typed views while preserving their existing generic `Item` constructor and accessor.
- **KMIPKIT-0048-FR-005**: Typed reads MUST distinguish tag mismatch from TTLV value-type mismatch using a redacted error; `Debug` and error formatting MUST NOT expose attribute values or raw KMIP bodies.
- **KMIPKIT-0048-FR-006**: Scalar and Structure values MUST round-trip without lossy conversion. Structure-valued attributes retain child order, unknown children, and repetitions through the existing TTLV model.
- **KMIPKIT-0048-FR-007**: Typed convenience MUST NOT change source-backed multiplicity checks, Vendor Attribute validation, mutation preflight, server-authoritative behavior, or any existing wire encoding.
- **KMIPKIT-0048-FR-008**: The 62 definitions and their API declarations MUST be reconciled with the normative catalog and public API manifest through deterministic tooling; generated outputs MUST use the repository generator and pass `--check`.
- **KMIPKIT-0048-FR-009**: Rust rustdoc and English/Spanish user guides MUST include executable examples for typed scalar, repeated, and Structure-valued attributes, and explain generic fallback and open values.
- **KMIPKIT-0048-FR-010**: C, Java, and Python binding parity for the typed attribute surface is assigned to KMIPKIT-0053; this specification MUST not claim those bindings are complete.
- **KMIPKIT-0048-FR-011**: The approved Phase 5 audit correction MUST normalize only the 14 semantic Link Type assignments shared by KMIP 2.1 §11.28, Table 465, and KMIP 3.0 §§4.35.1–4.35.20 / §11.58, using explicit tag mappings. It MUST reject 3.0-only link attributes, identifier encodings with no counterpart in the destination version, duplicate required fields, and extra unrecognized children rather than discard or coerce them. Enumeration and Integer identifiers MUST round-trip unchanged. Text String identifiers MUST be rejected when converting to KMIP 3.0 because the KMIP 3.0 link attribute encoding in §4.35 does not list Text String.

### Normative Traceability

The detailed per-attribute matrix is in `gap-matrix.md`; every direct attribute row cites its exact §4 subsection and source table from the pinned `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` and checked-in catalog. Wrapper structure rules are KMIP Specification v2.1 §§5.5–5.7, Tables 161–163; Structure encoding/order is §10.1.2. Generic Item and Attribute Structures are governed by §§5.1–5.4 and Tables 157–160.

## Success Criteria

- **KMIPKIT-0048-SC-001**: All 62 direct attribute definitions have an exact tag, source section/table, and TTLV value category verified against the pinned catalog and upstream source.
- **KMIPKIT-0048-SC-002**: Tests cover each generated definition and every TTLV value category, including open Enumeration values, invalid tag/type pairs, and the Certificate Attributes exclusion.
- **KMIPKIT-0048-SC-003**: Repeated instances permitted by the source are observed in original order; the collection retains duplicates losslessly and operation-level validation continues to reject prohibited uses.
- **KMIPKIT-0048-SC-004**: Existing generic API behavior and serialized TTLV vectors remain unchanged.
- **KMIPKIT-0048-SC-005**: All public Rust routes are accounted for by exact manifest mappings or justified exclusions, and generation is deterministic.
- **KMIPKIT-0048-SC-006**: Each applicable normative/API requirement links to source, implementation, and executable verification before review; binding parity is explicitly tracked for 0053.
- **KMIPKIT-0048-SC-007**: Every shared Link Type value maps to its exact KMIP 3.0 link attribute tag; all 20 KMIP 3.0 Link tags are allocation-checked, and six attributes without a KMIP 2.1 equivalent are rejected without changing the input.

## Key Entities

- **Typed Attribute Definition**: A generated Rust marker that fixes one KMIP Object Attribute's tag and source-defined TTLV value category.
- **Typed Attribute**: One owned direct TTLV Item coupled to its attribute definition; it can be inserted into an `AttributeSet` or wrapped as a Current/New Attribute.
- **Typed Attribute View**: A borrowed view of an existing Item that exposes the matching value category without copying or losing child data.
- **Attribute Set**: An ordered set of direct object attribute Items whose source-defined multiplicity policy remains authoritative.

## Assumptions and Scope

- This feature adds typed Rust APIs over existing TTLV Items and does not add protocol operations or change KMIP 2.1 wire behavior.
- The catalog and immutable OASIS 2.1 source copy are the source of truth. Where the catalog counts a record that is not a direct tagged Attribute, the standard's encoded structure determines applicability.
- The Phase 5 audit correction also uses the checksum-pinned KMIP 3.0 source and the reviewed versioned-tag catalog for LinkNormalizer; it does not reinterpret the KMIP 2.1 attribute inventory.
- Detailed nested builders for managed cryptographic objects remain in KMIPKIT-0049. Here, Structure-valued attribute payloads use the existing lossless TTLV Structure representation.
- Equivalent C, Java, and Python convenience APIs are delivered and verified under KMIPKIT-0053, after the Rust contract stabilizes.
- Unknown tags, unknown Enumeration values, extension children, and generic Item access remain supported.
