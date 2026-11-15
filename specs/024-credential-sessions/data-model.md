# Data Model: KMIP 2.1 Credential Sessions

All wire data is represented by the existing structurally valid TTLV model. Typed operation models preserve optionality, order, and unknown values. Normative field contracts are in the pinned OASIS KMIP v2.1 specification and the approved spec.md.

## Operation Ticket

Source: §7.39, Table 391; Ticket Type values: §11.57.

| Field | Wire form | Requirement | Model rule |
| --- | --- | --- | --- |
| Ticket Type | Enumeration | Required | Keep the raw unsigned value; unknown/vendor values round-trip. |
| Ticket Value | Byte String | Required | Secret-owned bytes; redact formatting/errors and zeroize memory owned by KMIPKit. |

The Ticket is shared by Delegated Login and Login responses and the Logout request. It is not automatically installed as a future authentication credential.

## Delegated Login

Source: §6.1.12, Tables 199–201.

| Side | Field | Wire form | Requiredness |
| --- | --- | --- | --- |
| Request | Lease Time | Interval or Date Time per approved table contract | Optional |
| Request | Request Count | Integer | Optional |
| Request | Usage Limits | Structure | Optional |
| Request | Rights | Structure containing the Rights list | Required |
| Response | Ticket | §7.39 Ticket structure | Required |
| Error | Result Status and Result Reason | Common batch result | Table 201 failure alternatives preserved |

The client serializes supplied fields in protocol order. Missing Rights is rejected locally with NotSent evidence. Returned Ticket is caller-visible but secret-safe.

## Login

Source: §6.1.30, Tables 253–255.

| Side | Field | Wire form | Requiredness |
| --- | --- | --- | --- |
| Request | Lease Time | Interval or Date Time per approved table contract | Optional |
| Request | Request Count | Integer | Optional |
| Request | Usage Limits | Structure | Optional |
| Response | Ticket | §7.39 Ticket structure | Required |
| Error | Result Status and Result Reason | Common batch result | Table 255 failure alternatives preserved |

The request can be empty. Optional absence is preserved and is not converted to a default.

## Logout

Source: §6.1.31, Tables 256–258.

| Side | Field | Wire form | Requiredness |
| --- | --- | --- | --- |
| Request | Ticket | §7.39 Ticket structure | Required |
| Response | Operation payload | Empty | No operation-specific response field |
| Error | Result Status and Result Reason | Common batch result | Table 258 failure alternatives preserved |

Missing Ticket is rejected locally with NotSent evidence. Ticket Value remains secret throughout encoding, execution, and error handling.

## Common result and extension data

Every client response retains the existing KMIP result status, reason, permitted message, response batch association, Pending/correlation state, and request delivery state. Typed accessors coexist with generic TTLV access. Unknown tags, enum values, and extension data remain available; no server result or state is fabricated.
