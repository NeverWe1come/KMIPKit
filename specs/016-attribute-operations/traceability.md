# KMIPKIT-0016 Normative Traceability

This design-stage matrix maps every functional requirement to its exact pinned OASIS source and planned verification. The implementation PR must replace planned paths with final code/test references and include every applicable catalog requirement and test-case identifier.

| Requirement | OASIS source | Planned implementation | Planned verification |
| --- | --- | --- | --- |
| FR-001, FR-002 | §§5.1, 5.5–5.7; §§6.1.2/3/13/20/21/34/51, request and response tables cited by operation | `crates/kmipkit-protocol/src/attribute.rs`; seven operation modules | `crates/kmipkit-protocol/tests/unit/attribute_*_tests.rs` |
| FR-003 | §6.1.2, Tables 167–169; §4 | `add_attribute.rs` | Add request shape and server Result Reason preservation; read-only is server policy, not client prevalidation |
| FR-004 | §6.1.3, Tables 170–172; §11.1, Tables 428–429; KMIPKIT-0004/ADR-0010 | `adjust_attribute.rs` | Assigned/extension enum validation, omitted-parameter defaults, absent-current-value defaults by type, single/multi-instance, and server error preservation |
| FR-005 | §6.1.13, Tables 202–204; §§4, 5.5–5.6 | `delete_attribute.rs` | Both optional selectors including both omitted, reference-only all-instance, required/read-only server results |
| FR-006 | §6.1.20, Tables 223–225 | `get_attributes.rs` | Omitted/duplicate/missing/repeated name and error tests |
| FR-007 | §6.1.21 prose and Tables 226–228 | `get_attribute_list.rs` | UID-only request, full-name result, required repeated response references, and error tests |
| FR-008 | §6.1.34, Tables 265–267; §§4, 5.6–5.7 | `modify_attribute.rs` | Exact old/new values, omitted selector, and ambiguity error tests |
| FR-009 | §6.1.51, Tables 322–324; §4 | `set_attribute.rs` | Absent, single, multi-instance and error tests |
| FR-010 | Tables 169, 172, 204, 225, 228, 267, 324; §§9.1–9.2 | Existing shared result model | Operation-specific Result Status/Reason/Message tests |
| FR-011 | §§4, 5.1, 5.5–5.7, 11.1 Table 429; KMIPKIT-0004/ADR-0010 | Shared `AttributeEntry` and generic TTLV value model | Round-trip allocated/extension tags and unknown values; reject Reserved tags and Adjustment Type values outside assigned/extension ranges |
| FR-012 | §§8.1–8.6, 9.19–9.21; KMIPKIT-0007/-0009 accepted contracts | `crates/kmipkit-client/src/execute.rs` | Fake transport one-exchange, delivery-state, and pending tests |
| FR-013 | AGENTS.md §8; accepted decoder limits | Existing redaction and bounded decoder path | Malformed/over-limit and redaction tests |
| FR-014 | All sources above and seven catalog operation elements | `specification/catalog/kmip-2.1.json` | Catalog validator, generated-report check, and full requirement matrix |

## Normative source inventory

- OASIS KMIP Specification v2.1 §4: Object Attributes, including multi-instance, single-instance, read-only, and required-value behavior.
- OASIS KMIP Specification v2.1 §§5.1, 5.5–5.7: Attributes, Attribute Reference, Current Attribute, and New Attribute.
- OASIS KMIP Specification v2.1 §6.1.2 Tables 167–169; §6.1.3 Tables 170–172; §6.1.13 Tables 202–204; §6.1.20 Tables 223–225; §6.1.21 Tables 226–228; §6.1.34 Tables 265–267; §6.1.51 Tables 322–324.
- OASIS KMIP Specification v2.1 §11.1 Tables 428–429: Adjustment Type values and descriptions.
- Shared message/result requirements: applicable structures in §§8.1–8.6, 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21.

The exact upstream source files remain immutable under `specification/oasis/kmip-2.1/upstream/`.
