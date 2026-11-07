# Feature Specification: Named KMIP Protocol Values

**Feature Branch**: `feature/KMIPKIT-0047-named-values`
**Created**: 2026-11-05
**Status**: Approved
**Input**: The approved KMIPKit Phase 5 plan; the maintainer explicitly authorized feature branches to start from the current KMIPKIT-0043 branch and stack toward `release/1.0.0`.

## Purpose

Make assigned KMIP 2.1 Enumeration and Bit Mask values discoverable by name in
Rust, C, Java, and Python while preserving the exact numeric value for unknown,
future, and vendor-defined values.

## User Stories and Acceptance Scenarios

### User Story 1 — Construct requests with named values (P1)

As an application developer, I can select an assigned KMIP value by a
generated symbolic name instead of memorizing its numeric value.

**Acceptance scenarios**

1. Every assigned KMIP 2.1 value in the reviewed catalog has a stable named
   constant in each supported language.
2. Constructing the corresponding typed model from a named value writes the
   exact catalog numeric value.
3. Unknown raw values remain representable and are not rejected because they
   have no generated name.

### User Story 2 — Inspect server values and bit masks (P1)

As an application developer, I can get the standard name for a known raw
Enumeration and the names of known bits in a mask without losing unknown values
or bits.

**Acceptance scenarios**

1. Name lookup returns the catalog name for every assigned Enumeration value.
2. Unknown Enumeration values remain available as raw numbers and name lookup
   reports no known name.
3. Bit-mask name lookup reports every set assigned bit and preserves the full
   raw mask, including unassigned bits.

## Catalog allocation audit

The approved catalog contains 723 Enumeration value rows and 44 Bit Mask
value rows, but those totals include non-assigned entries. Only **644
Enumeration values and 33 Bit Mask bits are allocated numeric values** that
can be emitted as usable named constants. The remaining rows are reserved or
extension placeholders (including wildcard values such as `8XXXXXXX`) and
must not be presented as valid constants. This distinction follows the
allocation field and wire-value representation in the catalog, whose entries
link to the exact normative source. `from_raw` continues to preserve values
that are unknown, reserved, future, or vendor-defined.

## Functional Requirements

- **KMIPKIT-0047-FR-001**: Generate language-accessible named constants for all
  64 catalogued KMIP 2.1 Enumerations and each of their 644 assigned values.
- **KMIPKIT-0047-FR-002**: Generate language-accessible named constants for
  all 3 catalogued Bit Masks and each of their 33 assigned bits.
- **KMIPKIT-0047-FR-003**: Provide name lookup for assigned Enumeration values;
  unknown numeric values MUST remain representable and MUST return no assigned
  name.
- **KMIPKIT-0047-FR-004**: Provide bit-name lookup that reports all assigned
  bits present in a raw mask and preserves all unknown bits in the raw value.
- **KMIPKIT-0047-FR-005**: Rust, C, Java, and Python MUST expose equivalent
  named-value capabilities through their existing package surfaces.
- **KMIPKIT-0047-FR-006**: Generated values MUST come only from the checked-in
  KMIP 2.1 catalog; generation MUST be deterministic and must not download or
  scrape OASIS sources.
- **KMIPKIT-0047-FR-007**: New constants and lookups MUST NOT change wire
  encoding, operation behavior, existing public paths, or unknown-value
  preservation.

## Normative Sources and Traceability

The authoritative source is the pinned KMIP Specification v2.1 in
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. Enumeration
and Bit Mask values are defined in §§11–12; each generated value is traced to
its exact catalog `element_id` and `source_refs` entry. The catalog's four
source documents and pinned digests remain authoritative; Usage Guide text is
not used to add requirements.

| Requirement | Normative/catalog source | Verification |
|---|---|---|
| FR-001 | Catalog elements with kind `enumeration` and `enumeration_value`; each value's `source_refs` points to its exact KMIP 2.1 section | Generated count/value tests and four-language API tests |
| FR-002 | Catalog elements with kind `bitmask` and `bitmask_value`; each value's `source_refs` points to its exact KMIP 2.1 section | Generated count/bit tests and four-language API tests |
| FR-003–004 | KMIP 2.1 §§11–12 and KMIPKIT unknown/future value preservation policy | Known/unknown raw value and unknown-bit tests |
| FR-005–007 | Constitution §§3, 4, 9, 10; KMIPKIT-0043 manifest and generation contract | Manifest parity, deterministic generation, and regression tests |

## Scope and Exclusions

- In scope: symbolic constants and name inspection for the catalogued KMIP 2.1
  Enumerations and Bit Masks, in the four 1.0 languages.
- Existing raw-value constructors and generic TTLV interfaces remain supported.
- This feature does not infer server support, profile conformance, or
  cryptographic policy from the presence of a constant.
- This feature does not add protocol operations or modify wire behavior.

## Success Criteria

- **KMIPKIT-0047-SC-001**: Generated output accounts for 64 Enumerations,
  644 assigned Enumeration values, 3 Bit Masks, and 33 assigned mask bits;
  the catalog audit separately accounts for all 723 and 44 total rows,
  including reserved and extension placeholders.
- **KMIPKIT-0047-SC-002**: Every assigned value resolves to its exact raw
  catalog value and stable name in Rust, C, Java, and Python.
- **KMIPKIT-0047-SC-003**: Unknown Enumeration values and mask bits survive
  construction, inspection, and serialization unchanged.
- **KMIPKIT-0047-SC-004**: Re-running the pinned generator produces no diff;
  every new public declaration is reconciled in the API manifest.
