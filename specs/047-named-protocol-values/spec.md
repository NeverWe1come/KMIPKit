# Feature Specification: Named KMIP Protocol Values

**Feature Branch**: `feature/KMIPKIT-0047-named-values`
**Created**: 2026-11-05
**Status**: Approved
**Input**: The approved KMIPKit Phase 5 plan; the maintainer explicitly authorized feature branches to start from the current KMIPKIT-0043 branch and stack toward `release/1.0.0`.

## Purpose

Make assigned KMIP 2.1 Enumeration and Bit Mask values discoverable by name in
Rust, C, Java, and Python while preserving the exact numeric value for unknown,
future, and vendor-defined values. A separate, reviewable catalog also records
the operation additions in KMIP 2.1 and the KMIP 3.0 Operation, Object Class,
and Object Type assignments used by version-aware code.

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
  KMIP 2.1 catalog or the checksum-verified version-delta catalog; generation
  MUST be deterministic and must not download or scrape OASIS sources.
- **KMIPKIT-0047-FR-007**: Named constants and lookups MUST NOT change wire
  encoding, request formation, existing public paths, or unknown-value
  preservation. Version-introduction metadata is governed by FR-010.
- **KMIPKIT-0047-FR-008**: The version-delta catalog MUST use exact assigned
  values from pinned OASIS sources and record the edition, section, table, and
  source checksum for every group. Reserved and wildcard allocations MUST NOT
  be emitted as named constants.
- **KMIPKIT-0047-FR-009**: Rust, C, Java, and Python MUST expose the reviewed
  KMIP 2.1 Operation additions and KMIP 3.0 Operation, Object Class, and Object
  Type additions through deterministic generated values and lookups.
- **KMIPKIT-0047-FR-010**: Version-aware operation lookup MUST return no
  introduction span for unassigned codes; numeric ranges MUST NOT make
  reserved or unassigned operation values valid.
- **KMIPKIT-0047-FR-011**: Version-aware tag lookup MUST use the exact tag
  assignments introduced by each pinned KMIP edition. It MUST NOT classify
  previously assigned KMIP 2.0/2.1 tags as KMIP 3.0 tags based on an adjacent
  numeric range.

## Normative Sources and Traceability

The authoritative source for KMIP 2.1 Enumeration and Bit Mask values is the
pinned KMIP Specification v2.1 in
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. These values
are defined in §§11–12; each generated value is traced to its exact catalog
`element_id` and `source_refs` entry. Edition deltas are recorded separately in
`specification/catalog/kmip-versioned-values.json`, with exact source checksums
and section/table citations to KMIP 2.1, KMIP 3.0, and the pinned KMIP 2.0
baseline. Reserved and wildcard allocations are omitted. Usage Guide text is
not used to add requirements.

KMIP tag version introductions are checked against exact assignments in KMIP
2.1 §11.56 and KMIP 3.0 §11.58, Table 605. For example, `Object Defaults`
(`0x420153`) and `Protection Storage Masks` (`0x42015F`) remain valid tags in
KMIP 2.1, while `NIST Security Category` (`0x4201C2`) is introduced in KMIP
3.0. Version gates use explicit assignments and do not infer edition from a
tag-number interval.

| Requirement | Normative/catalog source | Verification |
|---|---|---|
| FR-001 | Catalog elements with kind `enumeration` and `enumeration_value`; each value's `source_refs` points to its exact KMIP 2.1 section | Generated count/value tests and four-language API tests |
| FR-002 | Catalog elements with kind `bitmask` and `bitmask_value`; each value's `source_refs` points to its exact KMIP 2.1 section | Generated count/bit tests and four-language API tests |
| FR-003–004 | KMIP 2.1 §§11–12 and KMIPKIT unknown/future value preservation policy | Known/unknown raw value and unknown-bit tests |
| FR-005–007 | Constitution §§3, 4, 9, 10; KMIPKIT-0043 manifest and generation contract | Manifest parity, deterministic generation, and regression tests |
| FR-008–009 | KMIP 2.1 §11.36 Table 470; KMIP 3.0 §§11.34–11.35 Tables 581–582 and §11.37 Table 583; `specification/catalog/kmip-versioned-values.json` verifies pinned source checksums | Exact catalog assignment tests, four-language generation tests, and `VersionGate` tests |
| FR-010 | KMIP 2.0/2.1/3.0 Operation Enumerations; assigned values in the checked-in versioned catalog | Assigned, reserved, and unknown operation code tests |
| FR-011 | KMIP 2.1 §11.56; KMIP 3.0 §11.58 Table 605 and checksum-pinned assignments | `VersionGate` exact tag-span tests for legacy and 3.0 tags |

## Scope and Exclusions

- In scope: symbolic constants and name inspection for the catalogued KMIP 2.1
  Enumerations and Bit Masks, in the four 1.0 languages.
- Also in scope: exact version deltas for KMIP 2.1 Operations and KMIP 3.0
  Operations, Object Class, and Object Type used by version gates and typed
  models, plus exact KMIP 3.0 tag assignments used by version gates. These
  deltas do not imply complete KMIP 3.0 protocol support.
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
- **KMIPKIT-0047-SC-005**: The exact KMIP 2.1/3.0 delta tables are generated in
  all four languages; reserved or unassigned codes have no generated name and
  receive no version span.
- **KMIPKIT-0047-SC-006**: KMIP 2.0/2.1 tags retain their existing version
  spans; only exact KMIP 3.0 Table 605 assignments receive a KMIP 3.0 span.
