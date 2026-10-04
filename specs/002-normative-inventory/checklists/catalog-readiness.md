# Inventory Review Checklist: KMIP 2.1 Normative Coverage

**Purpose**: Reviewer checklist for source completeness and traceability before the catalog is used to drive protocol specifications.
**Created**: 2026-10-04
**Feature**: [spec.md](../spec.md)
**Review Ownership**: Reviewer-owned. Leave unchecked until each criterion is independently verified.

## Source Integrity

- [ ] CHK001 All four local OASIS work products match the recorded identity, stage, publication date, and SHA-256.
- [ ] CHK002 No upstream OASIS source or generated output was edited by hand.
- [ ] CHK003 Catalog validation and report generation succeed without network access.

## Protocol Element Coverage

- [ ] CHK004 All 57 client-to-server and 5 server-to-client operations have exact sections, directions, and scope dispositions.
- [ ] CHK005 Data types, objects, structures, fields, credentials, attributes, every enumeration value, every defined bitmask bit, options, results, and tag-table counts reconcile to pinned source tables, with all five tag-range rows kept separate.
- [ ] CHK006 Reserved, unused, extension-range, and server-direction values remain distinct from usable 1.0 client values.
- [ ] CHK007 Asynchronous results and client Poll/Cancel flows are not misclassified as server-initiated operations.

## Normative Traceability

- [ ] CHK008 Every normative-text candidate in pinned Specification and Profiles HTML appears in the source-clause ledger with a linked requirement or reviewed disposition.
- [ ] CHK009 Each requirement includes source section, level, role/direction, condition, scope, and evidence state.
- [ ] CHK010 Prohibitions have negative-verification markers; deviations from SHOULD, SHOULD NOT, and RECOMMENDED point to accepted decisions; MAY/OPTIONAL capabilities remain represented.
- [ ] CHK011 Every unassigned specification, implementation, or verification link appears in the generated report.

## Profiles and Evidence

- [ ] CHK012 Every client profile and conformance clause is represented with 1.0 applicability and a separate claim state.
- [ ] CHK013 Test-case identifiers and fixture availability are recorded without inferring absent fixture contents.
- [ ] CHK014 No profile conformance claim is emitted by the inventory without completed clauses and evidence.

## Source Discrepancies and Determinism

- [ ] CHK015 Batch Error Continuation wording, Profiles §5.3.1 HTTPS content-type/body conflict, and suspect profile/test links are recorded with source locations and alternatives.
- [ ] CHK016 Open discrepancies remain unresolved in catalog data and block only dependent implementation decisions.
- [ ] CHK017 Source files and fixture paths are allowlisted/root-confined; raw `href` values are never followed.
- [ ] CHK018 Repeated report generation from identical and permuted catalog inputs produces byte-identical output.
- [ ] CHK019 An independent section-by-section review records that every Specification and Profiles section/table was checked for normative candidates missed by the extractor.
