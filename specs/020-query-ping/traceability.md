# KMIPKIT-0020 Traceability: Query and Ping

This is the design-time assignment for the specification PR. Implementation and verification paths stay pending until the separately authorized implementation change exists.

## Client normative requirements

| Stable requirement ID | Exact source | Normative statement / feature behavior | Spec | Planned implementation | Planned verification |
| --- | --- | --- | --- | --- | --- |
| `KMIPKIT-REQ-SPEC-6.1.40-014` | OASIS KMIP Specification v2.1 §6.1.40, Table 282; catalog clause `KMIPKIT-CLAUSE-SPEC-6.1.40-014` | Query request contains at least one Query Function. Empty list rejected before exchange. | FR-003; US2.1 | `crates/kmipkit-protocol/src/query.rs`; `crates/kmipkit-client/src/execute.rs` | `query_operation_tests::rejects_empty_query_function_list`; `query_execution_tests::empty_query_is_not_sent` |
| `KMIPKIT-REQ-SPEC-6.1.40-016` | OASIS KMIP Specification v2.1 §6.1.40, Table 282; catalog clause `KMIPKIT-CLAUSE-SPEC-6.1.40-016` | Client MAY repeat Query Function to request multiple information types. | FR-003; US2.1 | `crates/kmipkit-protocol/src/query.rs` | `query_operation_tests::preserves_repeated_query_functions_and_order`; `query_execution_tests::sends_multiple_query_functions_once` |

## Operation and structure assignments

| Catalog element | Source | Assigned specification coverage | Planned verification |
| --- | --- | --- | --- |
| `KMIPKIT-ELEM-OP-C2S-PING` | §6.1.36, Tables 271–272 | FR-001, FR-002, FR-008–FR-010 | `ping_operation_tests`; `ping_execution_tests` |
| `KMIPKIT-ELEM-OP-C2S-QUERY` | §6.1.40, Tables 281–284 | FR-001, FR-003–FR-009, FR-011–FR-012 | `query_operation_tests`; `query_execution_tests` |
| `KMIPKIT-ELEM-ENUMERATION-QUERY-FUNCTION` and all 14 assigned values plus extension range | §11.44, Table 476 | FR-003, FR-005 | `query_function_tests`; Query request round trips |
| Query response members in Table 283 | §6.1.40, Table 283 | FR-006–FR-007, FR-011 | Per-member cardinality, repetition, order, unknown-value, empty-payload, and empty Protection Storage Masks tests |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-23-OBJECT-GROUPS` and `KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP` | §7.23, Table 375 | FR-004 | Request round trip with absent, single, and repeated Object Group members |
| `KMIPKIT-ELEM-ATTRIBUTE-OBJECT-GROUP` | §4.35, Tables 99–100; §7.23, Table 375 | FR-004 | Attribute shape, Text String value, repetition, order, and request round trip |

Operation enum values assigned to this feature are KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PING-0000003B and KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-00000018. The optional Object Groups request structure, its repeated Object Group member, and the Object Group attribute are assigned to this feature and verified by the Query request tests. `KMIPKIT-ELEM-ENUMERATION-OBJECT-GROUP-MEMBER` (§11.33) is not assigned: it types a distinct Locate request option, not the Object Group attribute in Query. Neither operation element has an asynchronous-response assignment in the catalog.

The two normative client requirements above are the applicable standalone requirement records cataloged for this scope. Table 281–284 prose assigns additional conditional response obligations to the server; this client implementation does not implement server behavior. It decodes and exposes the response without claiming the server conformed merely because a response was received.

## Open source conflict

`KMIPKIT-DISC-047` records two conflicting statements in OASIS KMIP Specification v2.1 §6.1.40: the prose says the response payload is empty when there are no values to return, while Table 283 marks Protection Storage Masks as required and says a server may provide an empty list when unable or unwilling to provide that information. The client contract accepts both forms without choosing a server-conformance interpretation. This feature does not close the discrepancy; an approved OASIS erratum or project decision is required before asserting server conformance for the disputed case.

## OASIS test-case mapping and limitations

| Catalog test ID | Official label | Source | Relationship | Fixture status and claim |
| --- | --- | --- | --- | --- |
| `KMIPKIT-TEST-CN01-2-67` | `TC-PING-1-21` | Test Cases v2.1 §2.67 | Ping | Fixture unavailable; no official vector pass claim |
| `KMIPKIT-TEST-PROF-5-17-1` | `QS-M-1-12` | Profiles v2.1 §5.17.1 | Query | Fixture unavailable; mapping confidence weak and source/case label mismatch is recorded in the catalog |
| `KMIPKIT-TEST-PROF-5-3-3-1` | `MSGENC-HTTPS-M-1-21` | Profiles v2.1 §5.3.3.1 | Query, HTTPS profile | Fixture unavailable; HTTPS profile is not claimed by this TTLV operation specification |
| `KMIPKIT-TEST-PROF-5-4-4-1` | `MSGENC-XML-M-1-21` | Profiles v2.1 §5.4.4.1 | Query, XML profile | Fixture unavailable; XML is out of scope |
| `KMIPKIT-TEST-PROF-5-5-4-1` | `MSGENC-JSON-M-1-21` | Profiles v2.1 §5.5.4.1 | Query, JSON profile | Fixture unavailable; JSON is out of scope |

These source links inform coverage and test planning; their missing XML fixtures cannot be treated as test failures or passes. Query tests in this feature use source-derived TTLV expectations and explicitly identify their provenance. The catalog assigns both client requirements to KMIPKIT-0020; their status remains unassigned until executable implementation and verification references are recorded in the implementation PR.

## Catalog source dispositions retained

- Query's two client requirements retain their stable IDs and exact §6.1.40 clauses.
- The catalog's server-only and non-applicable clause dispositions in §§6.1.36 and 6.1.40 are not reclassified as client obligations.
- `KMIPKIT-DISC-002` is limited to HTTPS profile evidence; it does not gate ordinary TTLV Query/Ping.
- `KMIPKIT-DISC-039` applies to Query Asynchronous Requests (§6.1.41), not ordinary Query.
- No profile support, interoperability certification, or formal OASIS test-suite pass is claimed.