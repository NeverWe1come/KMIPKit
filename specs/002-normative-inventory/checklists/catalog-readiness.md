# Inventory Review Checklist: KMIP 2.1 Normative Coverage

**Purpose**: Reviewer checklist for source completeness and traceability before the catalog is used to drive protocol specifications.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)
**Review Ownership**: Reviewer-owned. Leave unchecked until each criterion is independently verified.

## Source Integrity

- [x] CHK001 All four local OASIS work products match the recorded identity, stage, publication date, and SHA-256.
- [x] CHK002 No upstream OASIS source or generated output was edited by hand.
- [x] CHK003 Catalog validation and report generation succeed without network access.

## Protocol Element Coverage

- [x] CHK004 All 57 client-to-server and 5 server-to-client operations have exact sections, directions, and scope dispositions.
- [x] CHK005 Data types, objects, structures, fields, credentials, attributes, every enumeration value, every defined bitmask bit, options, results, and tag-table counts reconcile to pinned source tables, with all five tag-range rows kept separate.
- [x] CHK006 Reserved, unused, extension-range, and server-direction values remain distinct from usable 1.0 client values.
- [x] CHK007 Asynchronous results and client Poll/Cancel flows are not misclassified as server-initiated operations.

## Normative Traceability

- [x] CHK008 Every normative-text candidate in pinned Specification and Profiles HTML appears in the source-clause ledger with a linked requirement or reviewed disposition.
- [x] CHK009 Each requirement includes source section, level, role/direction, condition, scope, and evidence state.
- [x] CHK010 Prohibitions have negative-verification markers; deviations from SHOULD, SHOULD NOT, and RECOMMENDED point to accepted decisions; MAY/OPTIONAL capabilities remain represented.
- [x] CHK011 Every unassigned specification, implementation, or verification link appears in the generated report.

## Profiles and Evidence

- [x] CHK012 Every client profile and conformance clause is represented with 1.0 applicability and a separate claim state.
- [x] CHK013 Test-case identifiers and fixture availability are recorded without inferring absent fixture contents.
- [x] CHK014 No profile conformance claim is emitted by the inventory without completed clauses and evidence.

## Source Discrepancies and Determinism

- [x] CHK015 Batch Error Continuation wording, Profiles §5.3.1 HTTPS content-type/body conflict, and suspect profile/test links are recorded with source locations and alternatives.
- [x] CHK016 Open discrepancies remain unresolved in catalog data and block only dependent implementation decisions.
- [x] CHK017 Source files and fixture paths are allowlisted/root-confined; raw `href` values are never followed.
- [x] CHK018 Repeated report generation from identical and permuted catalog inputs produces byte-identical output.
- [x] CHK019 An independent section-by-section review records that every Specification and Profiles section/table was checked for normative candidates missed by the extractor.
