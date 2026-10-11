# Traceability: KMIP 2.1 Policy and Usage Controls

The approved spec and this table link normative operation payloads and requirements to planned implementation and verification. Spec-scoped table IDs fill gaps where the catalog has no row-level payload requirement. No direct OASIS Test Cases fixture is cataloged for the five operations.

## Operation/payload traceability

| Stable trace ID | Catalog element IDs | Exact source | Contract | Planned implementation | Planned verification |
| --- | --- | --- | --- | --- | --- |
| `KMIPKIT-025-T229-REQUEST` | `KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS`; `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-CONSTRAINTS-00000038` | Specification §6.1.22, Table 229 | Empty operation payload | `get_constraints.rs`; client `execute.rs` | Empty request vector and operation identity in fake transport |
| `KMIPKIT-025-T230-RESPONSE` | `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-7-CONSTRAINTS`; `KMIPKIT-ELEM-TAG-420168` | §6.1.22, Table 230; §7.7 | Required ordered Constraints Structure, preserve unknown children | `get_constraints.rs` | Required-field, roundtrip, unknown nested tag, malformed response |
| `KMIPKIT-025-T231-ERROR` | Get Constraints operation ID; common Result Status/Reason/Message tags | §6.1.22, Table 231 | Preserve Failure and every listed Result Reason | `get_constraints.rs`; `execute.rs` | One test per listed reason plus unknown result reason |
| `KMIPKIT-025-T232-REQUEST` | `KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION`; `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-USAGE-ALLOCATION-00000011`; `KMIPKIT-ELEM-TAG-420094`; `KMIPKIT-ELEM-TAG-420096` | §6.1.23, Table 232; §4.59, Table 148; §4.58, Table 145 | Optional Unique Identifier; required Usage Limits Count Long Integer | `get_usage_allocation.rs`; client `execute.rs` | `get_usage_allocation_tests.rs`; `get_usage_allocation_execution_tests.rs`: optional omission, required count, Long Integer encoding and field order |
| `KMIPKIT-025-T233-RESPONSE` | Get Usage Allocation operation ID; `KMIPKIT-ELEM-TAG-420094` | §6.1.23, Table 233; §4.58, Table 145 | Required resolved Unique Identifier | `get_usage_allocation.rs`; client `execute.rs` | Missing/duplicate/wrong-type ID rejected; text, integer, and open enumeration IDs preserved; unknown siblings retained |
| `KMIPKIT-025-T234-ERROR` | Get Usage Allocation operation ID; common result tags | §6.1.23, Table 234 | Preserve Failure and listed/unknown Result Reason | `get_usage_allocation.rs`; client `execute.rs` | All listed reason cases, unknown reasons, Pending correlation, one exchange, delivery evidence, and no retry in `get_usage_allocation_execution_tests.rs` |
| `KMIPKIT-025-T325-REQUEST` | `KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS`; `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-CONSTRAINTS-00000037`; `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-7-CONSTRAINTS`; `KMIPKIT-ELEM-TAG-420168` | §6.1.52, Table 325; §7.7 | Required Constraints Structure | `set_constraints.rs` | Missing field NotSent, ordered Structure preservation |
| `KMIPKIT-025-T326-RESPONSE` | Set Constraints operation ID | §6.1.52, Table 326 | Empty operation success payload | `set_constraints.rs` | Empty success and malformed nonempty response behavior |
| `KMIPKIT-025-T327-ERROR` | Set Constraints operation ID; common result tags | §6.1.52, Table 327; approved KMIPKIT-DISC-020 | Preserve error alternatives under approved source disposition | `set_constraints.rs`; `execute.rs` | Every listed reason and exact trace to DISC-020 |
| `KMIPKIT-025-T328-REQUEST` | `KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS`; `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-DEFAULTS-00000036`; `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-11-DEFAULTS-INFORMATION`; `KMIPKIT-ELEM-TAG-420152` | §6.1.53, Table 328; §7.11 | Optional Defaults Information; Table 328 explicitly says omission removes all Object Defaults | `set_defaults.rs` | Omitted vs empty distinction |
| `KMIPKIT-025-T329-RESPONSE` | Set Defaults operation ID | §6.1.53, Table 329 | Empty operation success payload | `set_defaults.rs` | Empty success and malformed payload |
| `KMIPKIT-025-T330-ERROR` | Set Defaults operation ID; common result tags | §6.1.53, Table 330; approved KMIPKIT-DISC-020 | Preserve Failure and listed reasons; disposition assigns truncated error introduction to Table 330 | `set_defaults.rs`; `execute.rs` | Table reason cases, no fabricated success |
| `KMIPKIT-025-T331-REQUEST` | `KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE`; `KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ENDPOINT-ROLE-00000032`; `KMIPKIT-ELEM-TAG-420151`; `KMIPKIT-ELEM-ENUMERATION-ENDPOINT-ROLE` | §6.1.54, Table 331; §11.19 | Required open Endpoint Role Enumeration | `set_endpoint_role.rs` | Known, unknown, and vendor enum roundtrips; missing-field NotSent |
| `KMIPKIT-025-T332-RESPONSE` | Set Endpoint Role C→S operation ID; tag/enum IDs above | §6.1.54, Table 332 | Required accepted Endpoint Role, preserve unknown value | `set_endpoint_role.rs` | Required field and unknown enum value |
| `KMIPKIT-025-T333-ERROR` | Set Endpoint Role C→S operation ID; common result tags | §6.1.54, Table 333 | Preserve operation failure alternatives | `set_endpoint_role.rs`; `execute.rs` | Table reason cases and result metadata |

## §6.1.23 client requirement traceability

| Catalog requirement ID | Normative source and obligation | Planned implementation | Planned verification |
| --- | --- | --- | --- |
| `KMIPKIT-REQ-SPEC-6.1.23-001-001` | Specification §6.1.23: a client SHALL NOT use a Usage-Limited object for protection unless an allocation has been obtained | Explicit successful-allocation context plus caller route-through-guard contract; no metadata inference | Cannot create context from Failure/Pending/malformed allocation; guarded calls require context and matching object; document unguarded/external boundary |
| `KMIPKIT-REQ-SPEC-6.1.23-001-002` | §6.1.23: request allocation only after Activation Date and before Protect Stop Date | Caller responsible for known object timing; client does not infer/read remote dates | Document this local information boundary; no test fabricates remote-date validation |
| `KMIPKIT-REQ-SPEC-6.1.23-002-001` | §6.1.23/Table 232: include required number of Usage Limits Units | Required Integer in typed request; context receives exact count | Missing count/invalid local input NotSent; encoded request exact count |
| `KMIPKIT-REQ-SPEC-6.1.23-002-002` | §6.1.23: do not continue after the entire allocation has been consumed until another allocation is obtained | Check caller cost against local available units before guarded protection; retain ambiguous reservations | Exact balance accepted; over-budget rejected without transport; NotSent releases, every possibly delivered outcome reserves |

The guard covers only typed Encrypt, MAC, and Sign calls routed through one caller-owned instance. It does not count Decrypt, MAC Verify, Signature Verify, Hash, unguarded/raw requests, other processes, or external server use. It never reports local reservation as remote consumption.

## Common client/batch behaviors

| Stable trace ID | Source | Contract | Planned verification |
| --- | --- | --- | --- |
| `KMIPKIT-025-COMMON-RESULT` | Shared KMIP response/batch contract; each operation error table | Preserve Success, Failure, permitted Result Message, Pending, and correlation; no retry/Poll | Fake transport response matrix per operation; unknown result values |
| `KMIPKIT-025-COMMON-DELIVERY` | AGENTS.md transport delivery invariant | Validation is NotSent; ambiguous delivery remains PossiblySent/ResponseStarted | Fake-transport call-count and error-delivery assertions |
| `KMIPKIT-025-OPEN-VALUES` | Constitution public API compatibility; §11.19 and §11.60 | Open Enumeration and unknown nested fields survive typed conversions | Property/roundtrip tests over known, unknown, and extension values |

## Official test-case inventory

No direct OASIS Test Cases catalog IDs or pinned fixtures are listed for these five operations in the approved source inventory. Add project-derived tests from the exact specification tables and label them as project tests; do not call them official OASIS conformance cases. Official profile/conformance claims remain blocked until the complete applicable source test set is evidenced.
