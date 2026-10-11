# Traceability: KMIP 2.1 Credential Sessions

This initial mapping assigns stable operation, payload, and error-table trace IDs before implementation. The operation and tag IDs below already exist in specification/catalog/kmip-2.1.json. The spec-scoped table IDs fill gaps where the catalog has no row-level payload/error requirement ID. Implementation and verification references remain planned until the implementation PR.

## Operation and table contracts

| Stable trace ID | Catalog element(s) | Exact OASIS source | Contract | Planned implementation | Planned verification |
| --- | --- | --- | --- | --- | --- |
| KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELEGATED-LOGIN-0000002F | Operation and value from §6.1.12/§11.36 | §6.1.12; Tables 199–201 | Client-to-server Delegated Login request/response and operation errors | protocol delegated_login.rs; client execute.rs | delegated_login_operation_tests; delegated_login_execution_tests |
| KMIPKIT-024-T199-REQUEST | KMIPKIT-ELEM-TAG-420049, KMIPKIT-ELEM-TAG-42014C, KMIPKIT-ELEM-TAG-420095, KMIPKIT-ELEM-TAG-42014D | §6.1.12, Table 199; §7.35/Table 387 | Optional Lease Time, Request Count, Usage Limits; required Rights; exact order and presence | delegated_login.rs | request requiredness, optionality, order, and unknown nested values |
| KMIPKIT-024-T200-RESPONSE | KMIPKIT-ELEM-TAG-420149, KMIPKIT-ELEM-OPERATION-STRUCTURE-7-39-TICKET, KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-TYPE, KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-VALUE, KMIPKIT-ELEM-ENUMERATION-TICKET-TYPE | §6.1.12, Table 200; §7.39/Table 391; §11.57 | Required Ticket structure; raw Ticket Type and secret Ticket Value | ticket.rs; delegated_login.rs | required field, nested structure, unknown enumeration, redaction, zeroization |
| KMIPKIT-024-T201-ERROR | KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN; KMIPKIT-ELEM-TAG-42007F Result Status, KMIPKIT-ELEM-TAG-42007E Result Reason, and KMIPKIT-ELEM-TAG-42007D Result Message | §6.1.12, Table 201 | Preserve listed Failure alternatives, duplicate source names, generic unknown reasons, and common result metadata | delegated_login.rs; execute.rs | operation-failure result and reason preservation, no result fabrication |
| KMIPKIT-ELEM-OP-C2S-LOGIN; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGIN-0000002D | Operation and value from §6.1.30/§11.36 | §6.1.30; Tables 253–255 | Client-to-server Login request/response and operation errors | protocol login.rs; client execute.rs | login_operation_tests; login_execution_tests |
| KMIPKIT-024-T253-REQUEST | KMIPKIT-ELEM-TAG-420049, KMIPKIT-ELEM-TAG-42014C, KMIPKIT-ELEM-TAG-420095 | §6.1.30, Table 253 | Optional Lease Time, Request Count, and Usage Limits; empty request permitted | login.rs | empty and optional-field round trips, order and extension preservation |
| KMIPKIT-024-T254-RESPONSE | KMIPKIT-ELEM-TAG-420149, KMIPKIT-ELEM-OPERATION-STRUCTURE-7-39-TICKET, KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-TYPE, KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-VALUE, KMIPKIT-ELEM-ENUMERATION-TICKET-TYPE | §6.1.30, Table 254; §7.39/Table 391; §11.57 | Required Ticket structure; raw Ticket Type and secret Ticket Value | ticket.rs; login.rs | official fixture and derived response cases, redaction, unknown-value preservation |
| KMIPKIT-024-T255-ERROR | KMIPKIT-ELEM-OP-C2S-LOGIN; KMIPKIT-ELEM-TAG-42007F Result Status, KMIPKIT-ELEM-TAG-42007E Result Reason, and KMIPKIT-ELEM-TAG-42007D Result Message | §6.1.30, Table 255 | Preserve listed Failure alternatives and common result metadata | login.rs; execute.rs | Operation Failed, all table reasons, malformed response, no retry |
| KMIPKIT-ELEM-OP-C2S-LOGOUT; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGOUT-0000002E | Operation and value from §6.1.31/§11.36 | §6.1.31; Tables 256–258 | Client-to-server Logout request/response and operation errors | protocol logout.rs; client execute.rs | logout_operation_tests; logout_execution_tests |
| KMIPKIT-024-T256-REQUEST | KMIPKIT-ELEM-TAG-420149, KMIPKIT-ELEM-OPERATION-STRUCTURE-7-39-TICKET, KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-TYPE, KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-VALUE, KMIPKIT-ELEM-ENUMERATION-TICKET-TYPE | §6.1.31, Table 256; §7.39/Table 391; §11.57 | Required Ticket structure; do not expose Ticket Value in diagnostics | logout.rs | missing Ticket rejects before transmission; exact secret-safe encoding |
| KMIPKIT-024-T257-RESPONSE | KMIPKIT-ELEM-OP-C2S-LOGOUT | §6.1.31, Table 257 | Empty operation-specific success payload | logout.rs | empty payload round trip; reject malformed payload without loss |
| KMIPKIT-024-T258-ERROR | KMIPKIT-ELEM-OP-C2S-LOGOUT; KMIPKIT-ELEM-TAG-42007F Result Status, KMIPKIT-ELEM-TAG-42007E Result Reason, and KMIPKIT-ELEM-TAG-42007D Result Message | §6.1.31, Table 258 | Preserve listed Failure alternatives and common result metadata | logout.rs; execute.rs | Operation Failed, all table reasons, delivery evidence, no retry |

## Existing normative cases

| Stable catalog ID | Exact source | Planned verification and limitation |
| --- | --- | --- |
| KMIPKIT-TEST-CN01-2-21 through KMIPKIT-TEST-CN01-2-33 | OASIS Test Cases v2.1 CN01 §§2.21–2.33 | Map Delegated Login scenarios to source-derived/project tests where pinned fixture files are unavailable; do not claim official fixture execution. |
| KMIPKIT-TEST-CN01-2-48 | OASIS Test Cases v2.1 CN01 §2.48, TC-LOGIN-1-21.xml | Use the pinned fixture at specification/oasis/kmip-2.1/fixtures/TC-LOGIN-1-21.xml; claim a pass only after running the actual fixture test. |
| KMIPKIT-TEST-CN01-2-49 and KMIPKIT-TEST-CN01-2-50 | OASIS Test Cases v2.1 CN01 §§2.49–2.50 | Derive and trace tests to exact case statements; no fixture pass claim without a fixture. |
| No direct catalog test ID | Logout §6.1.31 | Use project-derived positive/negative tests; label them as project tests, not official OASIS cases. |

## Requirement handling

The approved spec records no separate catalog client normative clause IDs for these three operations. Payload requiredness and result-table alternatives are mapped above to stable operation/tag IDs and spec-scoped table IDs. Server-side permission enforcement described in operation prose remains server behavior; this client exposes caller-controlled payloads and preserves returned results without claiming the server accepted a right or ticket.
