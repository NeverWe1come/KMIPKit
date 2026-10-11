# Feature Specification: KMIP 2.1 Credential sessions

**Feature Branch**: feature/KMIPKIT-0038-credential-sessions
**Created**: 2026-10-10
**Status**: Approved for implementation — 2026-10-11
**Approval evidence**: The maintainer approved specifications 022–028 for future implementation in a direct instruction on 2026-10-11. This approval does not start implementation.
**Input**: Specify the remaining client-initiated KMIP 2.1 operations in this family for the 1.0.0 release.

## Normative scope

OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020) is authoritative. Its pinned local copy is under specification/oasis/kmip-2.1/upstream. The Test Cases and Profiles documents provide evidence where applicable; the Usage Guide is informative.

| Operation | Catalog ID | Section | Request table | Response table | Error table |
| --- | --- | --- | ---: | ---: | ---: |
| Delegated Login | KMIPKIT-ELEM-OP-C2S-DELEGATED-LOGIN | §6.1.12 | 199 | 200 | 201 |
| Login | KMIPKIT-ELEM-OP-C2S-LOGIN | §6.1.30 | 253 | 254 | 255 |
| Logout | KMIPKIT-ELEM-OP-C2S-LOGOUT | §6.1.31 | 256 | 257 | 258 |

## Operation errors

Each operation error table is cited below with its Table number. Result Reason alternatives are preserved as listed in the pinned source; duplicate names reflect duplicate entries in the source table. The common result and transport-delivery contract remains applicable.

| Operation | OASIS error table | Result Status | Result Reason values |
| --- | ---: | --- | --- |
| Delegated Login | 201 | Operation Failed | Invalid Field, Permission Denied, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Login | 255 | Operation Failed | Invalid Field, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Logout | 258 | Operation Failed | Invalid Ticket, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |

## Product boundaries and dependencies

The feature uses the approved TTLV codec, typed messages/batches, client execution, TLS/HTTPS transports, object model, credentials, and public-language bindings. It adds no local cryptographic algorithm, server-initiated operation, encoding, transport, or implicit server-policy choice. A caller selects each operation explicitly. One call sends at most one request. The existing typed-batch contract preserves protocol Pending and correlation bytes when the caller enables asynchronous results. Unknown and vendor values remain lossless. Raw KMIP bodies and secret values never appear in logs or errors. Rust, C, Java, and Python must offer equivalent 1.0.0 capability.

## User Scenarios & Testing

### User Story 1 — Delegated Login (Priority: P1)

As a KMIPKit caller, I can send the §6.1.12 Delegated Login request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 199 request and returns a Table 200 success and Table 201 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Delegated Login request, when encoded and sent, then exactly one operation with the Table 199 fields reaches the transport.
2. Given a valid Table 200 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 201 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Rights is required. Returned Ticket is secret; Lease Time, Request Count, and Usage Limits are optional.

**Payload schema** (OASIS Specification v2.1 §6.1.12, Tables 199–200):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Lease Time | No | The lease time Interval or Date Time for the ticket. |
| Request | Request Count | No | The integer count of the number of requests that can be made with the ticket |
| Request | Usage Limits | No | The usage limits for operations performed. |
| Request | Rights | Yes | List of Rights granted to the ticket holder which may only perform operations allowed by at least one of the contained Right structures. |
| Response | Ticket | Yes | The Ticket that is returned |

---

### User Story 2 — Login (Priority: P1)

As a KMIPKit caller, I can send the §6.1.30 Login request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 253 request and returns a Table 254 success and Table 255 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Login request, when encoded and sent, then exactly one operation with the Table 253 fields reaches the transport.
2. Given a valid Table 254 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 255 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Returned Ticket is secret; optional lease, count, and limits remain distinct. For TC-LOGIN-1-21, use the displayed `.xml` target despite the missing period in the raw href; the pinned source HTML remains unchanged (KMIPKIT-DEC-012).

**Payload schema** (OASIS Specification v2.1 §6.1.30, Tables 253–254):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Lease Time | No | The lease time Interval or Date Time for the ticket |
| Request | Request Count | No | The integer count of the number of requests that can be made with the ticket |
| Request | Usage Limits | No | The usage limits for the operations performed |
| Response | Ticket | Yes | The ticket that is returned |

---

### User Story 3 — Logout (Priority: P1)

As a KMIPKit caller, I can send the §6.1.31 Logout request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 256 request and returns a Table 257 success and Table 258 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Logout request, when encoded and sent, then exactly one operation with the Table 256 fields reaches the transport.
2. Given a valid Table 257 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 258 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Ticket is required and secret; success payload is empty.

**Payload schema** (OASIS Specification v2.1 §6.1.31, Tables 256–257):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Ticket | Yes | The ticket to be invalidated |
| Response | Empty payload | — | No operation-specific field. |

---

### Edge Cases

- Missing required or incompatible conditional request fields fail before transmission with NotSent delivery evidence.
- Optional absence, repeated field order, unknown values, and opaque structures remain distinct; server results are not fabricated.
- KMIP failure results and malformed network input are handled through the shared result/error and TTLV resource-limit contracts.
- Tickets, credentials, keys, seeds, and raw KMIP bodies never appear in diagnostics.

## Requirements

### Functional Requirements

- **FR-001**: Expose typed request/response models and explicit client calls for Delegated Login, Login, Logout using the common batch and delivery-state contract.
- **FR-002**: Validate payload structure and applicable client-side MUST/SHALL/SHALL NOT requirements that can be checked from local request data. Do not turn server-side requirements into client preconditions or infer remote object state or policy. Preserve unknown tags, enum values, bitmask bits, and extensions instead of rejecting them solely because KMIPKit does not recognize them.
- **FR-003**: Preserve Success, Failure, Pending, Result Reason, permitted Result Message, correlation, unknown values, and extension data losslessly. Never retry or poll automatically.
- **FR-004**: Expose equivalent Rust, C, Java, and Python capability by 1.0.0; retain generic structurally valid TTLV access alongside typed forms.
- **FR-005**: Redact secrets and raw bodies; zeroize KMIPKit-owned secret memory and document Java/Python runtime-copy limits.
- **FR-006**: Implement every applicable client requirement below, negatively verify MUST NOT/SHALL NOT, and record any SHOULD deviation for review.
- **FR-007**: Do not claim profile support, official test-vector passes, interoperability, or certification without corresponding pinned evidence.
- **FR-008**: Map every mandatory or conditional payload-table row to a stable catalog element or requirement ID and to verification; assign a stable spec-scoped trace ID when no suitable catalog ID exists.

### Client normative requirement ledger

Stable IDs and summaries below come from the checked-in normative catalog. The implementation PR must link each ID to code and verification and regenerate traceability with the pinned repository tool. Source sections are exact; payload tables above remain binding for operations with no separate client-clause records.

| Requirement ID | OASIS Specification section | Keyword | Client obligation or capability |
| --- | --- | --- | --- |
| No separate catalog client clause | — | — | Follow the cited payload and error tables. |

### Key Entities

- Operation request: typed fields and preserved structurally valid extensions.
- Operation response: typed fields and preserved unknown data.
- KMIP result: status, reason, permitted message, Pending correlation, and delivery evidence.
- Secret value: ticket, credential, key, seed, or other sensitive bytes subject to redaction and owned-memory zeroization.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 3 operations have typed request/response, an explicit client call, generic TTLV access, and Rust/C/Java/Python parity before 1.0.0.
- **SC-002**: All Table-defined fields and error forms and all 0 catalog client requirement IDs have positive/relevant negative verification and exact source-to-test links.
- **SC-003**: Fake-transport cases confirm one exchange and preserved Success, Failure, Pending, unknown values, malformed response behavior, and delivery state.
- **SC-004**: Required repository coverage, formatting, lint, security, generated-artifact, and two-independent-server integration gates pass before release.

## Assumptions and source decisions

- The accepted common message, batch, TTLV, TLS, security, and binding contracts remain authoritative. This specification adds only the named operations.
- Server-side obligations are not silently reclassified as client requirements; server policy may reject valid requests.
- Pinned official XML fixtures are incomplete. An unavailable fixture is documented rather than reported as a passing conformance vector.
- Other unresolved source discrepancies must receive an explicit disposition before implementation of affected behavior.

### Test-case inventory

| Operation | Catalog case IDs |
| --- | --- |
| Delegated Login | KMIPKIT-TEST-CN01-2-21, KMIPKIT-TEST-CN01-2-22, KMIPKIT-TEST-CN01-2-23, KMIPKIT-TEST-CN01-2-24, KMIPKIT-TEST-CN01-2-25, KMIPKIT-TEST-CN01-2-26, KMIPKIT-TEST-CN01-2-27, KMIPKIT-TEST-CN01-2-28, KMIPKIT-TEST-CN01-2-29, KMIPKIT-TEST-CN01-2-30, KMIPKIT-TEST-CN01-2-31, KMIPKIT-TEST-CN01-2-32, KMIPKIT-TEST-CN01-2-33 |
| Login | KMIPKIT-TEST-CN01-2-48 (fixture: `specification/oasis/kmip-2.1/fixtures/TC-LOGIN-1-21.xml`), KMIPKIT-TEST-CN01-2-49, KMIPKIT-TEST-CN01-2-50 |
| Logout | No direct catalog case ID |
