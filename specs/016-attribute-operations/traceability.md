# KMIPKIT-0016 Normative Traceability

This design-stage matrix maps every functional requirement to its exact pinned OASIS source and planned verification. The implementation PR must replace planned paths with final code/test references and include every applicable catalog requirement and test-case identifier.

| Requirement | OASIS source | Planned implementation | Planned verification |
| --- | --- | --- | --- |
| FR-001, FR-002 | §4; §§5.1, 5.5–5.7; KMIPKIT-REQ-SPEC-4-001-001; KMIPKIT-REQ-SPEC-4.60-001 (Table 150 Vendor Attribute encoding); §§6.1.2/3/13/20/21/34/51, request and response tables cited by operation | `crates/kmipkit-protocol/src/attribute.rs`; seven operation modules | `crates/kmipkit-protocol/tests/unit/attribute_*_tests.rs`, including distinct multi-instance value preservation and the Table 150 Vendor Attribute encoding vector |
| FR-003 | §6.1.2, Tables 167–169; §§4.1–4.63, 4.59–4.60; §7.40 Table 392; KMIPKIT-REQ-SPEC-4.28-001-002 | `add_attribute.rs`; generated standard/Vendor Attribute policy | Add request shape, local `NotSent` rejection for standard prohibitions, Usage Limits, and inspectable Vendor Attribute `y`, plus server Result Reason preservation |
| FR-004 | §6.1.3, Tables 170–172; §§4.1–4.63, 4.60; §11.1, Tables 428–429; KMIPKIT-0004/ADR-0010 | `adjust_attribute.rs`; generated standard-attribute policy | Assigned/extension enum validation, local standard-attribute rejection, omitted-parameter and absent-current-value defaults, and server result preservation for §4.60 because Attribute Reference omits Vendor Identification |
| FR-005 | §6.1.13, Tables 202–204; §§4.1–4.63, 4.60, 5.5–5.6 | `delete_attribute.rs`; generated standard/Vendor Attribute policy | Both optional selectors including both omitted, reference-only all-instance, local non-deletable/required-value rejection, Vendor Attribute `y` rejection when Current Attribute is supplied, and server results for uninspectable/state-dependent cases |
| FR-006 | §6.1.20, Tables 223–225 | `get_attributes.rs` | Omitted/duplicate/missing/repeated name and error tests |
| FR-007 | §6.1.21 prose and Tables 226–228 | `get_attribute_list.rs` | UID-only request, full-name result, required repeated response references, and error tests |
| FR-008 | §6.1.34, Tables 265–267; §§4.1–4.63, 4.59–4.60, 5.6–5.7; §7.40 Table 392 | `modify_attribute.rs`; generated standard/Vendor Attribute policy | Exact old/new values, omitted selector, local standard/Usage Limits/Vendor Attribute `y` rejection, and ambiguity/server-state tests |
| FR-009 | §6.1.51, Tables 322–324; §§4.1–4.63, 4.60; KMIPKIT-REQ-SPEC-4.28-001-002 | `set_attribute.rs`; generated standard/Vendor Attribute policy | Absent, single, multi-instance, local unconditional and Vendor Attribute `y` prohibition, and remaining server error tests |
| FR-010 | Tables 169, 172, 204, 225, 228, 267, 324; §§9.1–9.2 | Existing shared result model | Operation-specific Result Status/Reason/Message tests |
| FR-011 | §§4, 5.1, 5.5–5.7, 11.1 Table 429; KMIPKIT-0004/ADR-0010 | Shared `AttributeEntry` and generic TTLV value model | Round-trip allocated/extension tags and unknown values; reject Reserved outbound tags and Adjustment Type values outside assigned/extension ranges |
| FR-012 | §§8.1–8.6, 9.19–9.21; KMIPKIT-0007/-0009 accepted contracts | `crates/kmipkit-client/src/execute.rs` | Fake transport one-exchange, delivery-state, and pending tests |
| FR-013 | AGENTS.md §8; accepted decoder limits; OASIS KMIP Specification v2.1 Chapter 11 introduction and §11.56; accepted ADR-0011 (project policy) | Existing bounded decoder and Reserved-tag rejection path | Malformed/over-limit, redaction, and Reserved-tag rejection tests; do not attribute receiver rejection to OASIS |
| FR-014 | All sources above and seven catalog operation elements | `specification/catalog/kmip-2.1.json` | Applicable operation/shared-attribute requirements and source-linked OASIS test cases assigned to KMIPKIT-0016 before specification approval; catalog validator and generated-report check; code/test references completed during implementation |
| FR-015 | §§4.1–4.63, 4.59–4.60 prose (Table 150); §§6.1.2, 6.1.3, 6.1.13, 6.1.51; §7.40 Table 392; KMIPKIT-REQ-SPEC-4.28-001-002, KMIPKIT-REQ-SPEC-4.59-002/-003-001/-003-002 | Canonical catalog, deterministic policy generator, and mutation execution validation | Exhaustive table-driven registry checks; local errors are payload-free `NotSent` with zero fake-transport exchanges; unknown names and remote-state-dependent rules preserve server results; Vendor Attribute `y` is checked only when its value is supplied |

## Pre-approval catalog mapping

Phase B is the approved KMIPKIT-0002 normative inventory in `specs/002-normative-inventory/`. Its canonical design-time assignment inputs and generated coverage output are `specification/catalog/kmip-2.1.json` and `specification/catalog/coverage-report.md`. This specification PR closes the attribute-family assignment before approval. The implementation PR adds final code and executable-test references; it does not change these design-time assignments.

The seven operation elements are assigned to KMIPKIT-0016: Add Attribute, Adjust Attribute, Delete Attribute, Get Attributes, Get Attribute List, Modify Attribute, and Set Attribute. The catalog links each to its direct operation requirements and explicitly source-linked OASIS test cases. The Get Attribute List section has no separate normative requirement row in the current ledger; the operation element and its source-linked profile test remain assigned. The §4.60 Vendor Attribute element and its Table 150 structure members are also assigned to this feature. Its §4.60 prose rule has no separate normative-ledger requirement row, so it is traced through FR-015 without inventing an OASIS identifier. The feature contract defines the separate `source_value_policies` catalog entry as the canonical generator input; T011–T012 add that metadata and extend catalog validation after this specification is approved, because the current catalog schema rejects unknown element fields. The Vendor Attribute `Attribute Value` member's linked `KMIPKIT-TEST-CN01-2-42` (`TC-I18N-3-21`) is assigned, but its XML fixture is unavailable in the pinned test-case source; T007 adds a separate Table 150 encoding vector without claiming the official case passed.

The 30 applicable operation, shared attribute-structure, and Vendor Attribute requirement IDs assigned to KMIPKIT-0016 are:

- §4: `KMIPKIT-REQ-SPEC-4-001-001`, `KMIPKIT-REQ-SPEC-4-001-002`, `KMIPKIT-REQ-SPEC-4-001-003`, `KMIPKIT-REQ-SPEC-4-001-004`, and `KMIPKIT-REQ-SPEC-4-001-005`.
- §5.1: `KMIPKIT-REQ-SPEC-5.1-001` and `KMIPKIT-REQ-SPEC-5.1-002`.
- §5.5: `KMIPKIT-REQ-SPEC-5.5-001` and `KMIPKIT-REQ-SPEC-5.5-002`.
- §5.6: `KMIPKIT-REQ-SPEC-5.6-001`.
- §5.7: `KMIPKIT-REQ-SPEC-5.7-001`.
- §4.28: `KMIPKIT-REQ-SPEC-4.28-001-002` and `KMIPKIT-REQ-SPEC-4.28-002`.
- §4.59: `KMIPKIT-REQ-SPEC-4.59-002`, `KMIPKIT-REQ-SPEC-4.59-003-001`, and `KMIPKIT-REQ-SPEC-4.59-003-002`.
- §4.60: `KMIPKIT-REQ-SPEC-4.60-001` (Vendor Specific Attribute Item encoding).
- §6.1.2: `KMIPKIT-REQ-SPEC-6.1.2-001-001` and `KMIPKIT-REQ-SPEC-6.1.2-001-002`.
- §6.1.3: `KMIPKIT-REQ-SPEC-6.1.3-001`.
- §6.1.13: `KMIPKIT-REQ-SPEC-6.1.13-001-001` and `KMIPKIT-REQ-SPEC-6.1.13-001-002`.
- §6.1.20: `KMIPKIT-REQ-SPEC-6.1.20-001-001`, `KMIPKIT-REQ-SPEC-6.1.20-001-002`, and `KMIPKIT-REQ-SPEC-6.1.20-004`.
- §6.1.34: `KMIPKIT-REQ-SPEC-6.1.34-001-001`, `KMIPKIT-REQ-SPEC-6.1.34-001-002`, `KMIPKIT-REQ-SPEC-6.1.34-001-003`, and `KMIPKIT-REQ-SPEC-6.1.34-001-004`.
- §6.1.51: `KMIPKIT-REQ-SPEC-6.1.51-001`.

Explicitly linked OASIS test cases are `KMIPKIT-TEST-PROF-5-12-6-3` (`TL-M-3-21`, linked to Get Attribute List, Get Attributes, and Modify Attribute); `KMIPKIT-TEST-CN01-2-42` (`TC-I18N-3-21`, linked to the Vendor Attribute's Attribute Value member); and `KMIPKIT-TEST-CN01-2-92` (`TC-SETTATTR-1-21`), `KMIPKIT-TEST-CN01-2-93` (`TC-SETTATTR-2-21`), and `KMIPKIT-TEST-CN01-2-94` (`TC-SETTATTR-3-21`, all linked to Set Attribute). The TC-I18N-3-21 fixture is unavailable, so T007 adds an independent Table 150 vector without claiming that official case passed. No direct OASIS test-case row is linked to Add Attribute, Adjust Attribute, or Delete Attribute.

Shared message and result requirements outside §§4–5.7 retain their existing feature assignments or inventory dispositions; they are not silently reassigned by this operation-family specification.

### Reviewed exclusions

- `KMIPKIT-REQ-SPEC-4.28-001-001` (Key Value Present in Register) and `KMIPKIT-REQ-SPEC-4.28-001-003` (Key Value Present as a Locate criterion) are excluded because Register and Locate are not among these seven operations; their operation-family specifications own the behavior.
- Lease Time's §4.30 lease-expiry behavior governs whether an object may continue to be used, not these attribute request/response contracts. The catalog attribute record remains available to generic attribute operations; any independently applicable §4 source-defined mutability flag is still captured in the standard-attribute policy registry.
- Usage Limits state-dependent Total restrictions are assigned above but are not guessed locally because their triggers depend on remote protection/use/allocation state.
- §4.60 Vendor Attribute encoding (`KMIPKIT-REQ-SPEC-4.60-001`) is mapped to FR-001/FR-002 and the Table 150 required-member vector in T007. Separately, §4.60 prose prohibiting client actions on server-created Vendor Attributes with Vendor Identification `y` is traced to FR-015 and tested where the request includes the value; the client cannot infer that identifier from Adjust or reference-only Delete requests.

## Normative source inventory

- OASIS KMIP Specification v2.1 §4: Object Attributes, including multi-instance, single-instance, read-only, and required-value behavior.
- OASIS KMIP Specification v2.1 §§5.1, 5.5–5.7: Attributes, Attribute Reference, Current Attribute, and New Attribute.
- OASIS KMIP Specification v2.1 §6.1.2 Tables 167–169; §6.1.3 Tables 170–172; §6.1.13 Tables 202–204; §6.1.20 Tables 223–225; §6.1.21 Tables 226–228; §6.1.34 Tables 265–267; §6.1.51 Tables 322–324.
- OASIS KMIP Specification v2.1 §11.1 Tables 428–429: Adjustment Type values and descriptions.
- Shared message/result requirements: applicable structures in §§8.1–8.6, 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21.

The exact upstream source files remain immutable under `specification/oasis/kmip-2.1/upstream/`.
