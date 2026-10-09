# KMIPKIT-0016 Normative Traceability

This design-stage matrix maps every functional requirement to its exact pinned OASIS source and planned verification. The implementation PR must replace planned paths with final code/test references and include every applicable catalog requirement and test-case identifier.

| Requirement | OASIS source | Planned implementation | Planned verification |
| --- | --- | --- | --- |
| FR-001, FR-002 | §4; §§5.1, 5.5–5.7; KMIPKIT-REQ-SPEC-4-001-001; KMIPKIT-REQ-SPEC-4.60-001 (Table 150 Vendor Attribute encoding); §§6.1.2/3/13/20/21/34/51, request and response tables cited by operation | `crates/kmipkit-protocol/src/attribute.rs`; seven operation modules; existing KMIPKIT-0014 `AttributeSet` for Get Attributes response | `crates/kmipkit-protocol/tests/unit/attribute_*_tests.rs`, including both Attribute Reference forms, direct Current/New Items, multi-instance preservation, and Table 150 Vendor Attribute encoding |
| FR-003 | §6.1.2, Tables 167–169; §§4.1–4.63, 4.59–4.60; §7.40 Table 392; KMIPKIT-REQ-SPEC-4.28-001-002 | `add_attribute.rs`; generated standard/Vendor Attribute policy | Add request shape, local `NotSent` rejection for standard prohibitions, Usage Limits, and inspectable Vendor Attribute `y`, plus server Result Reason preservation |
| FR-004 | §6.1.3, Tables 170–172; §§4.1–4.63, 4.60; §11.1, Tables 428–429; KMIPKIT-0004/ADR-0010 | `adjust_attribute.rs`; generated standard-attribute policy | Both Attribute Reference forms, `y` rejection for inspectable name form, server authority for tag form, enum validation, and adjustment defaults |
| FR-005 | §6.1.13, Tables 202–204; §§4.1–4.63, 4.60, 5.5–5.6 | `delete_attribute.rs`; generated standard/Vendor Attribute policy | Both optional selectors including both omitted, reference-only all-instance, local standard policy, `y` rejection for supplied Current Item or name-form reference, and tag-form/server-state outcomes |
| FR-006 | §6.1.20, Tables 223–225; KMIPKIT-0014 `AttributeSet` contract | `get_attributes.rs`; existing `AttributeSet` | Omitted/duplicate/missing/repeated Attribute Reference and direct response Item tests |
| FR-007 | §6.1.21 prose and Tables 226–228 | `get_attribute_list.rs` | UID-only request, full-name result, required repeated response references, and error tests |
| FR-008 | §6.1.34, Tables 265–267; §§4.1–4.63, 4.59–4.60, 5.6–5.7; §7.40 Table 392 | `modify_attribute.rs`; generated standard/Vendor Attribute policy | Exact old/new values, omitted selector, local standard/Usage Limits/Vendor Attribute `y` rejection, and ambiguity/server-state tests |
| FR-009 | §6.1.51, Tables 322–324; §§4.1–4.63, 4.60; KMIPKIT-REQ-SPEC-4.28-001-002 | `set_attribute.rs`; generated standard/Vendor Attribute policy | Absent, single, multi-instance, local unconditional and Vendor Attribute `y` prohibition, and remaining server error tests |
| FR-010 | Tables 169, 172, 204, 225, 228, 267, 324; §§9.1–9.2 | Existing shared result model | Operation-specific Result Status/Reason/Message tests |
| FR-011 | §§4, 5.1, 5.5–5.7, 11.1 Table 429; KMIPKIT-0004/ADR-0010 | `AttributeReference`, direct Current/New generic Items, KMIPKIT-0014 `AttributeSet`, and generic TTLV value model | Round-trip both reference forms, allocated/extension tags and unknown values; reject Reserved outbound tags and Adjustment Type values outside assigned/extension ranges |
| FR-012 | §§8.1–8.6, 9.19–9.21; KMIPKIT-0007/-0009 accepted contracts | `crates/kmipkit-client/src/execute.rs` | Fake transport one-exchange, delivery-state, and pending tests |
| FR-013 | AGENTS.md §8; accepted decoder limits; OASIS KMIP Specification v2.1 Chapter 11 introduction and §11.56; accepted ADR-0011 (project policy) | Existing bounded decoder and Reserved-tag rejection path | Malformed/over-limit, redaction, and Reserved-tag rejection tests; do not attribute receiver rejection to OASIS |
| FR-014 | All sources above and seven catalog operation elements | `specification/catalog/kmip-2.1.json` | Applicable operation/shared-attribute requirements and source-linked OASIS test cases assigned to KMIPKIT-0016 before specification approval; catalog validator and generated-report check; code/test references completed during implementation |
| FR-015 | §§4.1–4.63, 4.59–4.60 prose (Table 150); §§5.5–5.7; §§6.1.2, 6.1.3, 6.1.13, 6.1.51; §7.40 Table 392; KMIPKIT-REQ-SPEC-4.28-001-002, KMIPKIT-REQ-SPEC-4.59-002/-003-001/-003-002 | Canonical catalog, deterministic policy generator, and mutation execution validation | Exhaustive table-driven registry checks; local errors are payload-free `NotSent` with zero fake-transport exchanges; unknown names/tags and remote-state-dependent rules preserve server results; Vendor Attribute `y` is checked for direct values and name-form references |

## Implementation preflight evidence

- **Approved specification revision**: KMIPKIT-0016 was approved by the merge of [specification PR #58](https://github.com/NeverWe1come/KMIPKit/pull/58), merge commit `0fa129e1248e735343704fb662926a401585ed7c`; the merged feature branch revision was `32b4644a7ef1a74addcb7ecbe1dd8b0a33c5b866`.
- **Release base**: `release/1.0.0` is at `04586ab9606521b304b0834aad85387cf149b196` (PR #60 merge). Dedicated implementation branch `feature/KMIPKIT-0016-attribute-operations-implementation` was created at this exact release commit; `git merge-base --is-ancestor origin/release/1.0.0 HEAD` passed.
- **Production transport**: [PR #55](https://github.com/NeverWe1come/KMIPKit/pull/55) merged as `0e50e4b9859532cf7a0832deb2d510a3af4563de`.
- **Response-root correction**: [PR #59](https://github.com/NeverWe1come/KMIPKit/pull/59) merged as `ba6421152f739b21204b0792f9e7fc1e12234c29`. Release code validates the decoded `ResponseMessage`; `crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_a_request_message_as_the_response_root` rejects a Request Message root, with the source citation to §8.4 Table 397 and §11.56 mapping Response Message to `0x42007B`. The focused regression passed locally with `cargo test -p kmipkit-transport --test raw_tls raw_tls_rejects_a_request_message_as_the_response_root --locked --offline` (1 passed).
- **AttributeSet dependency**: [PR #60](https://github.com/NeverWe1come/KMIPKit/pull/60) merged as `04586ab9606521b304b0834aad85387cf149b196`. `crates/kmipkit-protocol/src/attribute.rs` provides the ordered direct-item `AttributeSet` used by the Get Attributes contract. Its seven focused attribute tests passed locally with `cargo test -p kmipkit-protocol attribute --locked --offline` (7 passed).
- **Deferred interoperability evidence**: KMIPKIT-0015 PR #56 remains separate, optional live Cosmian evidence and is not a prerequisite for typed operation implementation. Its response-root correction was delivered independently in PR #59 above.
- **OASIS source re-audit**: Independent read-only review and local source search compared all seven operation sections and request/response/error table captions in the immutable upstream KMIP 2.1 HTML. The references in the normative-scope table and FR matrix match: §6.1.2 Tables 167–169; §6.1.3 Tables 170–172; §6.1.13 Tables 202–204; §6.1.20 Tables 223–225; §6.1.21 Tables 226–228; §6.1.34 Tables 265–267; §6.1.51 Tables 322–324. Adjustment Type references §11.1 Tables 428–429 also match. No reference correction was needed.

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

Shared message and result requirements outside §§4–5.7 retain their existing feature assignments or inventory dispositions; they are not silently reassigned by this operation-family specification. Get Attributes response uses KMIPKIT-0014's existing direct-item `AttributeSet`; Attribute Reference name/tag forms and Current/New wrappers remain separate shapes.

### Reviewed exclusions

- `KMIPKIT-REQ-SPEC-4.28-001-001` (Key Value Present in Register) and `KMIPKIT-REQ-SPEC-4.28-001-003` (Key Value Present as a Locate criterion) are excluded because Register and Locate are not among these seven operations; their operation-family specifications own the behavior.
- Lease Time's §4.30 lease-expiry behavior governs whether an object may continue to be used, not these attribute request/response contracts. The catalog attribute record remains available to generic attribute operations; any independently applicable §4 source-defined mutability flag is still captured in the standard-attribute policy registry.
- Usage Limits state-dependent Total restrictions are assigned above but are not guessed locally because their triggers depend on remote protection/use/allocation state.
- §4.60 Vendor Attribute encoding (`KMIPKIT-REQ-SPEC-4.60-001`) is mapped to FR-001/FR-002 and the Table 150 required-member vector in T007. Separately, §4.60 prose prohibiting client actions on server-created Vendor Attributes with Vendor Identification `y` is traced to FR-015 and tested wherever the identifier is supplied in a Vendor Attribute Item or name-form Attribute Reference. A tag-form Attribute Reference does not expose the vendor identifier, so the client does not infer it.

## Normative source inventory

- OASIS KMIP Specification v2.1 §4: Object Attributes, including multi-instance, single-instance, read-only, and required-value behavior.
- OASIS KMIP Specification v2.1 §§5.1, 5.5–5.7: Attributes, Attribute Reference, Current Attribute, and New Attribute.
- OASIS KMIP Specification v2.1 §6.1.2 Tables 167–169; §6.1.3 Tables 170–172; §6.1.13 Tables 202–204; §6.1.20 Tables 223–225; §6.1.21 Tables 226–228; §6.1.34 Tables 265–267; §6.1.51 Tables 322–324.
- OASIS KMIP Specification v2.1 §11.1 Tables 428–429: Adjustment Type values and descriptions.
- Shared message/result requirements: applicable structures in §§8.1–8.6, 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21.

The exact upstream source files remain immutable under `specification/oasis/kmip-2.1/upstream/`.
