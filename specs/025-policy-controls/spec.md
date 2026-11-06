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

Set Endpoint Role (§6.1.54, Tables 331–333) is documented in the OASIS
inventory but is deferred from KMIPKit 1.0.0; see the scope resolution below.

## Operation errors

Each operation error table is cited below with its Table number. Result Reason alternatives are preserved as listed in the pinned source; duplicate names reflect duplicate entries in the source table. The common result and transport-delivery contract remains applicable.

| Operation | OASIS error table | Result Status | Result Reason values |
| --- | ---: | --- | --- |
| Get Constraints | 231 | Operation Failed | Invalid Field, Invalid Object Type, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Get Usage Allocation | 234 | Operation Failed | Attribute Not Found, Invalid Message, Invalid Object Type, Object Not Found, Usage Limit Exceeded, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Set Constraints | 327 | Operation Failed | Invalid Field, Invalid Object Type, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Set Defaults | 330 | Operation Failed | Invalid Field, Invalid Object Type, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |

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
5. Given a successful allocation for a usage-limited object, when the caller creates the explicit guard with the returned Unique Identifier, caller-known Usage Limits Unit, and requested count, then the guard is bound to exactly that object and unit and starts with that local allowance.
6. Given a typed Encrypt, MAC, or Sign request routed through the guard, when the caller supplies a positive cost in that guard's unit, then the request is sent only if its explicit Unique Identifier matches the allocation and the cost does not exceed the known unreserved allowance.
7. Given a zero/negative cost, a different or absent object identifier, or a cost larger than the allowance, when the caller attempts guarded protection, then validation fails with NotSent evidence and the transport is not called.
8. Given a guarded request that is known not to have been sent, when execution returns NotSent, then its local reservation is released; given PossiblySent, ResponseStarted, a complete KMIP result, or Pending, then the reservation remains unavailable locally because the client cannot infer remote consumption.
9. Given an unguarded operation, raw TTLV value, or work from another process/client, when protection occurs, then it does not change this guard's local count and KMIPKit makes no claim that this guard prevents that external use.

**Specific rules**: Usage Limits Count is required and uses the Long Integer wire type defined in §7.40, Table 392; protection-use and exhaustion rules in §6.1.23 apply.

**Approved caller-guard interpretation**: For an object the caller knows has Usage Limits, the caller routes each protection request through the local usage-allocation guard described under Maintainer-approved interpretations below. The caller also ensures that Get Usage Allocation is requested only during the object's known protection-enabled window; KMIPKit does not fetch or infer Activation Date or Protect Stop Date. The caller supplies the usage unit from known object metadata and the cost for each guarded protection request; KMIPKit does not derive cost from payload size, operation type, or server state. The context is bound to the Unique Identifier returned by a successful allocation. It accounts only for requests routed through that context in this process. It does not track protection performed by another client, process, session, or an unguarded call, and it does not claim to know how much the server actually consumed.

**Payload schema** (OASIS Specification v2.1 §6.1.23, Tables 232–233):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the object whose usage allocation is being requested. If omitted, then the ID Placeholder is substituted by the server. |
| Request | Usage Limits Count | Yes | The number of Usage Limits Units to be protected; encode as TTLV Long Integer (`i64`) per §7.40, Table 392. |
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

**Specific rules**: OASIS §6.1.53, Table 328 explicitly states that omitting Defaults Information requests removal of all Object Defaults. KMIPKIT-DISC-020 applies to interpreting the truncated error introduction for Table 330, not to the Table 328 omission semantics.

**Payload schema** (OASIS Specification v2.1 §6.1.53, Tables 328–329):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Defaults Information | No | The set of Object Defaults to begin using. If no Defaults Information is supplied, the semantic is to remove all Object Defaults from the server. |
| Response | Empty payload | — | No operation-specific field. |

---

### Deferred operation — Set Endpoint Role

KMIP 2.1 §6.1.54, Tables 331–333, is not part of KMIPKit 1.0.0's supported
operation surface. The normative text states that after successful completion
the server assumes the Client role and the client assumes the Server role for
subsequent messages on the existing channel. §11.19, Tables 451–452, defines
the two roles but provides no client-only mode that suppresses this transition.
The corresponding server-to-client Set Endpoint Role operation is §6.2.5.

KMIPKit 1.0.0 does not accept server-initiated operations. Exposing §6.1.54 as
a complete client operation would therefore claim behavior the library cannot
perform after success. This operation and its role transition are deferred to
1.1, where a separate approved specification and ADR must cover both
directions. The OASIS operation remains in the normative inventory; it is not
advertised as implemented 1.0 capability.

---

### Edge Cases

- Missing required or incompatible conditional request fields fail before transmission with NotSent delivery evidence.
- Optional absence, repeated field order, unknown values, and opaque structures remain distinct; server results are not fabricated.
- KMIP failure results and malformed network input are handled through the shared result/error and TTLV resource-limit contracts.
- Tickets, credentials, keys, seeds, and raw KMIP bodies never appear in diagnostics.

## Requirements

### Functional Requirements

- **FR-001**: Expose typed request/response models and explicit client calls for Get Constraints, Get Usage Allocation, Set Constraints, and Set Defaults using the common batch and delivery-state contract. Set Endpoint Role (§6.1.54) is deferred to 1.1 because its normative successful result changes endpoint roles and requires server-initiated operation handling, outside the 1.0 boundary.
- **FR-002**: Validate payload structure and applicable client-side MUST/SHALL/SHALL NOT requirements that can be checked from local request data. Do not turn server-side requirements into client preconditions or infer remote object state or policy. When a caller explicitly supplies the approved usage-allocation context, enforce only its known local object, unit, allowance, and delivery reservations. Preserve unknown tags, enum values, bitmask bits, and extensions instead of rejecting them solely because KMIPKit does not recognize them.
- **FR-003**: Preserve Success, Failure, Pending, Result Reason, permitted Result Message, correlation, unknown values, and extension data losslessly. Never retry or poll automatically.
- **FR-004**: Expose equivalent Rust, C, Java, and Python capability by 1.0.0; retain generic structurally valid TTLV access alongside typed forms.
- **FR-005**: Redact secrets and raw bodies; zeroize KMIPKit-owned secret memory and document Java/Python runtime-copy limits.
- **FR-006**: Implement every applicable client requirement below, negatively verify MUST NOT/SHALL NOT, and record any SHOULD deviation for review.
- **FR-007**: Do not claim profile support, official test-vector passes, interoperability, or certification without corresponding pinned evidence.
- **FR-008**: Map every mandatory or conditional payload-table row to a stable catalog element or requirement ID and to verification; assign a stable spec-scoped trace ID when no suitable catalog ID exists.
- **FR-009**: Provide an explicit, non-cloneable local usage-allocation context for typed Encrypt, MAC, and Sign calls. Construct it only from a successful Get Usage Allocation exchange, the resolved response Unique Identifier, the caller-supplied Usage Limits Unit, and the exact requested count. Require a positive caller-supplied cost per guarded operation in that unit, reject a mismatched or absent object identifier and any cost above the locally available allowance before transport, and expose local reservation state without claiming server consumption. Do not infer metadata, units, costs, usage by external callers, or remaining remote allocation.

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

- **SC-001**: All 4 in-scope operations have typed request/response, an explicit client call, generic TTLV access, and Rust/C/Java/Python parity before 1.0.0. Set Endpoint Role is not included in this count or the 1.0 API surface.
- **SC-002**: All in-scope table-defined fields and error forms and all 4 catalog client requirement IDs have positive/relevant negative verification and exact source-to-test links, including tests for each §6.1.23 local guard boundary.
- **SC-003**: Fake-transport cases confirm one exchange and preserved Success, Failure, Pending, unknown values, malformed response behavior, and delivery state.
- **SC-004**: Required repository coverage, formatting, lint, security, generated-artifact, and two-independent-server integration gates pass before release.
- **SC-005**: For explicitly guarded typed protection calls, no request whose caller-supplied positive cost exceeds the known local allocation reaches transport; delivery tests prove only NotSent releases a reservation and no result is reported as server-consumption evidence.

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

### Maintainer-approved interpretations (2026-10-10)

These are project interpretations, not official OASIS errata. The implementation PR must record them as catalog decisions without modifying pinned upstream copies.

| Operation | Discrepancy | Disposition |
| --- | --- | --- |
| Get Constraints | KMIPKIT-DISC-016 | Empty request and required Constraints response; approved KMIPKIT-DISC-016 assigns the erroneous error introduction to this operation. |
| Set Constraints | KMIPKIT-DISC-020 | Constraints is required; approved KMIPKIT-DISC-020 assigns the truncated error introduction to this operation. |
| Set Defaults | KMIPKIT-DISC-020 | Table 328 normatively states that omission removes all Object Defaults. The approved KMIPKIT-DISC-020 disposition assigns the truncated error introduction to §6.1.53/Table 330. |

### 1.0 scope resolution — Set Endpoint Role (2026-10-11)

The implementation review found that the original 1.0 scope incorrectly treated
client initiation of §6.1.54 as sufficient to satisfy its full behavior. OASIS
§6.1.54 states without a value-dependent exception that successful completion
switches the endpoint roles; §11.19 defines what Client and Server roles do;
§6.2.5 describes the corresponding server-to-client operation. The KMIP 2.1
Baseline Client profile does not require §6.1.54. To preserve the approved
1.0 client-only boundary and avoid advertising a partial or misleading
implementation, defer §6.1.54 and §6.2.5 to 1.1. This is a release-scope
disposition, not an OASIS requirement change or an erratum. The normative
catalog continues to inventory the OASIS operation.

### Maintainer-approved interpretation — Usage Allocation guard (2026-10-11)

The maintainer approved an **explicit caller-supplied guard** for the client-side obligations in OASIS Specification v2.1 §6.1.23. This is a project interpretation of how KMIPKit exposes its client responsibilities; it is not OASIS errata. The source requires that a Usage-Limited Managed Object not be used for applying cryptographic protection before an allocation is obtained, that allocation be requested only after Activation Date and before Protect Stop Date, that the requested Usage Limits Count specify the number of units needed, and that the client stop applying protection after its entire allocation is consumed until it obtains a new allocation. The server assumes the entire requested allocation will be consumed. The exact clauses are `KMIPKIT-REQ-SPEC-6.1.23-001-001`, `KMIPKIT-REQ-SPEC-6.1.23-001-002`, `KMIPKIT-REQ-SPEC-6.1.23-002-001`, and `KMIPKIT-REQ-SPEC-6.1.23-002-002`.

The guard is explicit and local. It is associated with one successful Get Usage Allocation response, the concrete resolved Unique Identifier from that response, the caller-known Usage Limits Unit from object metadata, and the exact Usage Limits Count requested. The response does not echo the allocation amount or unit; callers must pass those request/metadata values to create the context. If Unique Identifier was omitted from the request, §6.1.23 uses the batch-scoped ID Placeholder to select the object; the response's concrete Unique Identifier identifies the selected object and becomes the context identifier. The literal ID Placeholder value (Enumeration value 1) is not a concrete resolved object identifier and cannot create a context or pass the guard. A guard cannot be created from a failed, Pending, malformed, or otherwise incomplete allocation result. Each context is non-cloneable and tracks only cost reservations made through that context instance in the current process.

Only typed Encrypt, MAC, and Sign requests are in the guard's protection-call scope. Decrypt, MAC Verify, Signature Verify, and Hash are outside this usage-allocation guard: §4.59 and §6.1.23 distinguish applying cryptographic protection from processing protected information, and do not allocate use for decryption or verification. A guarded operation must carry an explicit Unique Identifier equal to the allocation identifier; ID Placeholder, absent identifiers, or a different identifier are rejected before transport. The caller supplies a positive cost in the context's unit for every protected operation. KMIPKit never derives byte cost from Data length, assumes an object-use cost from an operation name, normalizes unknown units, or fetches/infers Usage Limits metadata. The exact caller-supplied unit and cost are retained as open values; unknown/vendor units remain usable if the caller can supply consistent costs.

The guard reserves the supplied cost before the exchange. If execution proves the request was `NotSent`, it releases that reservation. If transmission may have begun (`PossiblySent`), response bytes began (`ResponseStarted`), a complete response arrived, or the response is Pending or Failure, it retains the local reservation. KMIPKit cannot infer whether the server actually applied protection or consumed any amount, so the public result reports only local allowance/reservation accounting. A retained reservation is not evidence of remote consumption and cannot be reconciled automatically.

The caller must route each known Usage-Limited protection use through the guard it selected. Requests sent through the ordinary client API without a guard, generic TTLV access, calls from another client/process, and actions outside KMIPKit are outside this local count. The feature does not inspect remote object state, fetch Usage Limits or activation dates, track other consumers, provide a server-wide guarantee, or claim that unguarded use is prevented. There is no automatic retry, reallocation, consumption inference, or unit/cost inference.
