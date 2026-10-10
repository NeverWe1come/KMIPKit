# Feature Specification: KMIP 2.1 Policy and usage controls

**Feature Branch**: feature/KMIPKIT-0039-policy-controls
**Created**: 2026-10-10
**Status**: Approved for implementation — 2026-10-11
**Approval evidence**: The maintainer approved specifications 022–028 for future implementation in a direct instruction on 2026-10-11. This approval does not start implementation.
**Input**: Specify the remaining client-initiated KMIP 2.1 operations in this family for the 1.0.0 release.

## Normative scope

OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020) is authoritative. Its pinned local copy is under specification/oasis/kmip-2.1/upstream. The Test Cases and Profiles documents provide evidence where applicable; the Usage Guide is informative.

| Operation | Catalog ID | Section | Request table | Response table | Error table |
| --- | --- | --- | ---: | ---: | ---: |
| Get Constraints | KMIPKIT-ELEM-OP-C2S-GET-CONSTRAINTS | §6.1.22 | 229 | 230 | 231 |
| Get Usage Allocation | KMIPKIT-ELEM-OP-C2S-GET-USAGE-ALLOCATION | §6.1.23 | 232 | 233 | 234 |
| Set Constraints | KMIPKIT-ELEM-OP-C2S-SET-CONSTRAINTS | §6.1.52 | 325 | 326 | 327 |
| Set Defaults | KMIPKIT-ELEM-OP-C2S-SET-DEFAULTS | §6.1.53 | 328 | 329 | 330 |
| Set Endpoint Role | KMIPKIT-ELEM-OP-C2S-SET-ENDPOINT-ROLE | §6.1.54 | 331 | 332 | 333 |

## Operation errors

Each operation error table is cited below with its Table number. Result Reason alternatives are preserved as listed in the pinned source; duplicate names reflect duplicate entries in the source table. The common result and transport-delivery contract remains applicable.

| Operation | OASIS error table | Result Status | Result Reason values |
| --- | ---: | --- | --- |
| Get Constraints | 231 | Operation Failed | Invalid Field, Invalid Object Type, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Get Usage Allocation | 234 | Operation Failed | Attribute Not Found, Invalid Message, Invalid Object Type, Object Not Found, Usage Limit Exceeded, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Set Constraints | 327 | Operation Failed | Invalid Field, Invalid Object Type, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Set Defaults | 330 | Operation Failed | Invalid Field, Invalid Object Type, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Set Endpoint Role | 333 | Operation Failed | Permission Denied, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |

## Product boundaries and dependencies

The feature uses the approved TTLV codec, typed messages/batches, client execution, TLS/HTTPS transports, object model, credentials, and public-language bindings. It adds no local cryptographic algorithm, server-initiated operation, encoding, transport, or implicit server-policy choice. A caller selects each operation explicitly. One call sends at most one request. The existing typed-batch contract preserves protocol Pending and correlation bytes when the caller enables asynchronous results. Unknown and vendor values remain lossless. Raw KMIP bodies and secret values never appear in logs or errors. Rust, C, Java, and Python must offer equivalent 1.0.0 capability.

## User Scenarios & Testing

### User Story 1 — Get Constraints (Priority: P1)

As a KMIPKit caller, I can send the §6.1.22 Get Constraints request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 229 request and returns a Table 230 success and Table 231 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Get Constraints request, when encoded and sent, then exactly one operation with the Table 229 fields reaches the transport.
2. Given a valid Table 230 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 231 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Empty request and required Constraints response; approved KMIPKIT-DISC-016 assigns the erroneous error introduction to this operation.

**Payload schema** (OASIS Specification v2.1 §6.1.22, Tables 229–230):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Empty payload | — | No operation-specific field. |
| Response | Constraints | Yes | The set of Constraints that are being applied during operations. |

---

### User Story 2 — Get Usage Allocation (Priority: P1)

As a KMIPKit caller, I can send the §6.1.23 Get Usage Allocation request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 232 request and returns a Table 233 success and Table 234 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Get Usage Allocation request, when encoded and sent, then exactly one operation with the Table 232 fields reaches the transport.
2. Given a valid Table 233 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 234 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Usage Limits Count is required; protection-use and exhaustion rules in §6.1.23 apply.

**Payload schema** (OASIS Specification v2.1 §6.1.23, Tables 232–233):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the object whose usage allocation is being requested. If omitted, then the ID Placeholder is substituted by the server. |
| Request | Usage Limits Count | Yes | The number of Usage Limits Units to be protected. |
| Response | Unique Identifier | Yes | The Unique Identifier of the object. |

---

### User Story 3 — Set Constraints (Priority: P1)

As a KMIPKit caller, I can send the §6.1.52 Set Constraints request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 325 request and returns a Table 326 success and Table 327 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Set Constraints request, when encoded and sent, then exactly one operation with the Table 325 fields reaches the transport.
2. Given a valid Table 326 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 327 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Constraints is required; approved KMIPKIT-DISC-020 assigns the truncated error introduction to this operation.

**Payload schema** (OASIS Specification v2.1 §6.1.52, Tables 325–326):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Constraints | Yes | The set of Constraints to apply during operations. |
| Response | Empty payload | — | No operation-specific field. |

---

### User Story 4 — Set Defaults (Priority: P1)

As a KMIPKit caller, I can send the §6.1.53 Set Defaults request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 328 request and returns a Table 329 success and Table 330 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Set Defaults request, when encoded and sent, then exactly one operation with the Table 328 fields reaches the transport.
2. Given a valid Table 329 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 330 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Absent Defaults Information explicitly requests removal of all Object Defaults; approved KMIPKIT-DISC-020 applies.

**Payload schema** (OASIS Specification v2.1 §6.1.53, Tables 328–329):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Defaults Information | No | The set of Object Defaults to begin using. If no Defaults Information is supplied, the semantic is to remove all Object Defaults from the server. |
| Response | Empty payload | — | No operation-specific field. |

---

### User Story 5 — Set Endpoint Role (Priority: P1)

As a KMIPKit caller, I can send the §6.1.54 Set Endpoint Role request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 331 request and returns a Table 332 success and Table 333 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Set Endpoint Role request, when encoded and sent, then exactly one operation with the Table 331 fields reaches the transport.
2. Given a valid Table 332 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 333 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Only client-to-server §6.1.54 is covered; server-initiated §6.2.5 is outside 1.0.

**Payload schema** (OASIS Specification v2.1 §6.1.54, Tables 331–332):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Endpoint Role | Yes | The endpoint role for the server to apply. |
| Response | Endpoint Role | Yes | The accepted endpoint role as applied by the server. |

---

### Edge Cases

- Missing required or incompatible conditional request fields fail before transmission with NotSent delivery evidence.
- Optional absence, repeated field order, unknown values, and opaque structures remain distinct; server results are not fabricated.
- KMIP failure results and malformed network input are handled through the shared result/error and TTLV resource-limit contracts.
- Tickets, credentials, keys, seeds, and raw KMIP bodies never appear in diagnostics.

## Requirements

### Functional Requirements

- **FR-001**: Expose typed request/response models and explicit client calls for Get Constraints, Get Usage Allocation, Set Constraints, Set Defaults, Set Endpoint Role using the common batch and delivery-state contract.
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
| KMIPKIT-REQ-SPEC-6.1.23-001-001 | §6.1.23 | SHALL NOT | Obtain an allocation before using a Usage-Limited object for cryptographic protection. |
| KMIPKIT-REQ-SPEC-6.1.23-001-002 | §6.1.23 | SHALL | Request allocation only while protection is enabled for the object. |
| KMIPKIT-REQ-SPEC-6.1.23-002-001 | §6.1.23 | SHALL | Specify the number of protection units needed. |
| KMIPKIT-REQ-SPEC-6.1.23-002-002 | §6.1.23 | SHALL NOT | Do not continue protection use after the allocation is consumed until another allocation is obtained. |

### Key Entities

- Operation request: typed fields and preserved structurally valid extensions.
- Operation response: typed fields and preserved unknown data.
- KMIP result: status, reason, permitted message, Pending correlation, and delivery evidence.
- Secret value: ticket, credential, key, seed, or other sensitive bytes subject to redaction and owned-memory zeroization.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 5 operations have typed request/response, an explicit client call, generic TTLV access, and Rust/C/Java/Python parity before 1.0.0.
- **SC-002**: All Table-defined fields and error forms and all 4 catalog client requirement IDs have positive/relevant negative verification and exact source-to-test links.
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
| Get Constraints | No direct catalog case ID |
| Get Usage Allocation | No direct catalog case ID |
| Set Constraints | No direct catalog case ID |
| Set Defaults | No direct catalog case ID |
| Set Endpoint Role | No direct catalog case ID |

### Maintainer-approved interpretations (2026-10-10)

These are project interpretations, not official OASIS errata. The implementation PR must record them as catalog decisions without modifying pinned upstream copies.

| Operation | Discrepancy | Disposition |
| --- | --- | --- |
| Get Constraints | KMIPKIT-DISC-016 | Empty request and required Constraints response; approved KMIPKIT-DISC-016 assigns the erroneous error introduction to this operation. |
| Set Constraints | KMIPKIT-DISC-020 | Constraints is required; approved KMIPKIT-DISC-020 assigns the truncated error introduction to this operation. |
| Set Defaults | KMIPKIT-DISC-020 | Absent Defaults Information explicitly requests removal of all Object Defaults; approved KMIPKIT-DISC-020 applies. |
