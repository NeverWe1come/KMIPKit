# Feature Specification: KMIP 2.1 Interop and diagnostics

**Feature Branch**: feature/KMIPKIT-0042-diagnostics-interop
**Created**: 2026-10-10
**Status**: Draft for human review
**Input**: Specify the remaining client-initiated KMIP 2.1 operations in this family for the 1.0.0 release.

## Normative scope

OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020) is authoritative. Its pinned local copy is under specification/oasis/kmip-2.1/upstream. The Test Cases and Profiles documents provide evidence where applicable; the Usage Guide is informative.

| Operation | Catalog ID | Section | Request table | Response table | Error table |
| --- | --- | --- | ---: | ---: | ---: |
| Interop | KMIPKIT-ELEM-OP-C2S-INTEROP | §6.1.26 | 241 | 242 | 243 |
| Log | KMIPKIT-ELEM-OP-C2S-LOG | §6.1.29 | 250 | 251 | 252 |
| Validate | KMIPKIT-ELEM-OP-C2S-VALIDATE | §6.1.57 | 340 | 341 | 342 |

## Operation errors

Each operation error table is cited below with its Table number. Result Reason alternatives are preserved as listed in the pinned source; duplicate names reflect duplicate entries in the source table. The common result and transport-delivery contract remains applicable.

| Operation | OASIS error table | Result Status | Result Reason values |
| --- | ---: | --- | --- |
| Interop | 243 | Operation Failed | Invalid Field, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Log | 252 | Operation Failed | Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Validate | 342 | Operation Failed | Invalid Field, Invalid Object Type, Object Not Found, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |

## Product boundaries and dependencies

The feature uses the approved TTLV codec, typed messages/batches, client execution, TLS/HTTPS transports, object model, credentials, and public-language bindings. It adds no local cryptographic algorithm, server-initiated operation, encoding, transport, or implicit server-policy choice. A caller selects each operation explicitly. One call sends at most one request. The existing typed-batch contract preserves protocol Pending and correlation bytes when the caller enables asynchronous results. Unknown and vendor values remain lossless. Raw KMIP bodies and secret values never appear in logs or errors. Rust, C, Java, and Python must offer equivalent 1.0.0 capability.

## User Scenarios & Testing

### User Story 1 — Interop (Priority: P1)

As a KMIPKit caller, I can send the §6.1.26 Interop request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 241 request and returns a Table 242 success and Table 243 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Interop request, when encoded and sent, then exactly one operation with the Table 241 fields reaches the transport.
2. Given a valid Table 242 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 243 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Required Interop Function and Identifier, empty success payload; no interoperability claim follows from implementing this operation.

**Payload schema** (OASIS Specification v2.1 §6.1.26, Tables 241–242):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Interop Function | Yes | The function to be performed |
| Request | Interop Identifier | Yes | The identifier if the test case to be submitted. |
| Response | Empty payload | — | No operation-specific field. |

---

### User Story 2 — Log (Priority: P1)

As a KMIPKit caller, I can send the §6.1.29 Log request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 250 request and returns a Table 251 success and Table 252 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Log request, when encoded and sent, then exactly one operation with the Table 250 fields reaches the transport.
2. Given a valid Table 251 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 252 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Required Log Message and empty success payload. Approved KMIPKIT-DISC-017 assigns the mistaken Query error introduction to Log. Caller must not submit secrets.

**Payload schema** (OASIS Specification v2.1 §6.1.29, Tables 250–251):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Log Message | Yes | The message to log |
| Response | Empty payload | — | No operation-specific field. |

---

### User Story 3 — Validate (Priority: P1)

As a KMIPKit caller, I can send the §6.1.57 Validate request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 340 request and returns a Table 341 success and Table 342 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Validate request, when encoded and sent, then exactly one operation with the Table 340 fields reaches the transport.
2. Given a valid Table 341 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 342 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: One certificate chain using certificates, identifiers, or both; preserve order, optional Validity Date, and Validity Indicator without local validation.

**Payload schema** (OASIS Specification v2.1 §6.1.57, Tables 340–341):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Certificate | No, MAY be repeated | One or more Certificates. |
| Request | Unique Identifier | No, MAY be repeated | One or more Unique Identifiers of Certificate Objects. |
| Request | Validity Date | No | A Date-Time object indicating when the certificate chain needs to be valid. If omitted, the current date and time SHALL be assumed. |
| Response | Validity Indicator | Yes | An Enumeration object indicating whether the certificate chain is valid, invalid, or unknown. |

---

### Edge Cases

- Missing required or incompatible conditional request fields fail before transmission with NotSent delivery evidence.
- Optional absence, repeated field order, unknown values, and opaque structures remain distinct; server results are not fabricated.
- KMIP failure results and malformed network input are handled through the shared result/error and TTLV resource-limit contracts.
- Tickets, credentials, keys, seeds, and raw KMIP bodies never appear in diagnostics.

## Requirements

### Functional Requirements

- **FR-001**: Expose typed request/response models and explicit client calls for Interop, Log, Validate using the common batch and delivery-state contract.
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
| KMIPKIT-REQ-SPEC-6.1.57-001 | §6.1.57 | SHALL | A Validate Certificate request must contain only one certificate chain. |
| KMIPKIT-REQ-SPEC-6.1.57-002-001 | §6.1.57 | MAY | A client may build the chain from certificate objects, Managed Certificate identifiers, or both. |
| KMIPKIT-REQ-SPEC-6.1.57-002-002 | §6.1.57 | MAY | A client may specify the date at which the chain must be valid. |
| KMIPKIT-REQ-SPEC-6.1.57-004 | §6.1.57 | MAY | A client may include repeated Certificate objects in the chain. |
| KMIPKIT-REQ-SPEC-6.1.57-005 | §6.1.57 | MAY | A client may include repeated Unique Identifiers for Managed Certificate objects in the chain. |
| KMIPKIT-REQ-SPEC-6.1.57-006 | §6.1.57 | SHALL | If Validity Date is omitted, the chain is evaluated at the current date and time. |

### Key Entities

- Operation request: typed fields and preserved structurally valid extensions.
- Operation response: typed fields and preserved unknown data.
- KMIP result: status, reason, permitted message, Pending correlation, and delivery evidence.
- Secret value: ticket, credential, key, seed, or other sensitive bytes subject to redaction and owned-memory zeroization.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 3 operations have typed request/response, an explicit client call, generic TTLV access, and Rust/C/Java/Python parity before 1.0.0.
- **SC-002**: All Table-defined fields and error forms and all 6 catalog client requirement IDs have positive/relevant negative verification and exact source-to-test links.
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
| Interop | No direct catalog case ID |
| Log | No direct catalog case ID |
| Validate | No direct catalog case ID |

### Maintainer-approved interpretations (2026-10-10)

These are project interpretations, not official OASIS errata. The implementation PR must record them as catalog decisions without modifying pinned upstream copies.

| Operation | Discrepancy | Disposition |
| --- | --- | --- |
| Log | KMIPKIT-DISC-017 | Required Log Message and empty success payload. Approved KMIPKIT-DISC-017 assigns the mistaken Query error introduction to Log. Caller must not submit secrets. |
