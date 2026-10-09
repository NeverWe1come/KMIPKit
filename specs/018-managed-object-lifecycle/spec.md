# Feature Specification: KMIP 2.1 Managed-Object State Transitions

**Feature Branch**: `feature/KMIPKIT-0018-managed-object-lifecycle`
**Created**: 2026-10-09
**Status**: Draft for independent human review
**Input**: KMIPKit roadmap Phase 2: cover remaining client-initiated KMIP 2.1 operations through bounded, traceable protocol-family specifications.

## Normative scope

This specification covers exactly four KMIP 2.1 client-to-server operations:

| Operation | Catalog element | Normative source | Request, response, and error tables |
| --- | --- | --- | --- |
| Activate | `KMIPKIT-ELEM-OP-C2S-ACTIVATE` | OASIS KMIP Specification v2.1 §6.1.1 | Tables 164–166 |
| Archive | `KMIPKIT-ELEM-OP-C2S-ARCHIVE` | §6.1.4 | Tables 173–175 |
| Destroy | `KMIPKIT-ELEM-OP-C2S-DESTROY` | §6.1.15 | Tables 208–210 |
| Recover | `KMIPKIT-ELEM-OP-C2S-RECOVER` | §6.1.42 | Tables 288–290 |

The common message and batch rules come from OASIS KMIP Specification v2.1 §§8.1–8.6 and 9.1–9.2, 9.5–9.6, 9.12, 9.16, and 9.19–9.21, together with the approved KMIPKIT-0006 message/batch model, KMIPKIT-0007 client execution contract, and KMIPKIT-0009 asynchronous-result model. The pinned OASIS source and checked-in normative catalog are authoritative; Usage Guide text is informative.

The catalog contains one applicable client MAY for Archive (`KMIPKIT-REQ-SPEC-6.1.4-001`) and two for Recover (`KMIPKIT-REQ-SPEC-6.1.42-001-001` and `KMIPKIT-REQ-SPEC-6.1.42-001-002`). The catalog classifies the Activate and Destroy prose clauses as server-only; those clauses do not create client-side object-state requirements. The operations remain in the client's 1.0 scope because their client request/response payloads are defined in the tables above. No requirement-specific official Test Cases IDs are linked for the Archive and Recover client requirements in the catalog; derived tests must not be described as official-case passes.

Each operation table defines an optional request Unique Identifier and a required successful-response Unique Identifier. The allowed Unique Identifier encodings are defined in §4.58 Tables 145–146; §11.56 assigns the Unique Identifier tag. These field rules support all four operation tables and do not expand the operation scope. Existing message, batch, ID Placeholder, delivery-state, and Pending rules apply without being redefined here.

## Product boundaries and exclusions

- Encoding is TTLV only. This is a Rust protocol/client slice using the accepted request, response, message, batch, result, and transport foundations.
- The scope is limited to Activate, Archive, Destroy, and Recover. Register/import, Revoke, Re-certify, Re-key, other object operations, cryptographic processing, and server-initiated operations are excluded and require their own specification coverage.
- C ABI, Java, Python, and complete high-level API parity remain in the later 1.0 API specifications. This specification does not claim those language surfaces are complete.
- KMIPKit represents the client request and server result. It does not locally perform or simulate a server state transition, infer remote object state, or claim that a successful Archive response proves archival completion or that a Destroy response proves metadata removal.
- A Recover result may be Pending only under the approved asynchronous request/response rules. The caller may explicitly Poll as supported by KMIPKIT-0009 and may explicitly Get after Recover completes, as allowed by `KMIPKIT-REQ-SPEC-6.1.42-001-002`. This feature adds no automatic polling, follow-up Get, retry, or recovery workflow.
- Each explicit invocation performs at most one exchange. Existing delivery evidence distinguishes NotSent, PossiblySent, and ResponseStarted; no request is automatically retried.
- This specification does not change the KMIP version, TLS policy, transport boundary, public language scope, or immutable upstream OASIS files.

## User Scenarios & Testing

### User Story 1 — Activate or destroy an identified object (Priority: P1)

As a KMIPKit caller, I can explicitly request Activate or Destroy and receive the KMIP server's result, so the library expresses the protocol operation without pretending that it owns remote object state.

**Why this priority**: These operations complete basic server-managed object lifecycle coverage and must preserve the caller's exact choice of operation.

**Independent Test**: A deterministic fake transport verifies each request against its OASIS request table, returns valid and invalid success payloads and table-defined errors, and confirms that no local state change or retry occurs.

**Acceptance Scenarios**:

1. **Given** an Activate request with or without its optional Unique Identifier, **When** it is encoded, **Then** the request follows §6.1.1 Table 164 and preserves omission or the supplied identifier.
2. **Given** a Destroy request with or without its optional Unique Identifier, **When** it is encoded, **Then** the request follows §6.1.15 Table 208 and preserves omission or the supplied identifier.
3. **Given** a successful Activate or Destroy response, **When** it is decoded, **Then** the required response Unique Identifier is returned with that operation's typed outcome.
4. **Given** either operation fails or returns a malformed successful payload, **When** the result is processed, **Then** the server result is preserved or a protocol error is reported, and KMIPKit does not infer or mutate remote object state.
5. **Given** either request permits an asynchronous result under the shared batch rules, **When** the server returns Pending, **Then** the caller receives the correlated Pending outcome and KMIPKit does not poll or retry automatically.

### User Story 2 — Request object archival (Priority: P2)

As a KMIPKit caller, I can request archival of a managed object and observe the server's result, so I can express an archival preference while leaving archival policy and timing to the server.

**Why this priority**: Archive is the client's explicit signal that an object is no longer expected to be accessed and should be archived when server policy permits.

**Independent Test**: A deterministic fake transport verifies the optional identifier and response shape from §6.1.4 Tables 173–175 and confirms that neither request nor success response is described as proof of completed archival.

**Acceptance Scenarios**:

1. **Given** an Archive request with an optional Unique Identifier, **When** it is encoded, **Then** the identifier is preserved exactly when supplied and remains absent when omitted.
2. **Given** a successful Archive response, **When** it is decoded, **Then** the required Unique Identifier is returned and the client does not report that archival has completed.
3. **Given** the server rejects an Archive request or returns Pending when the shared asynchronous rules permit it, **When** the response is processed, **Then** the result and any correlation value are preserved without a retry or automatic poll.

### User Story 3 — Recover an archived object (Priority: P2)

As a KMIPKit caller, I can explicitly request recovery and inspect either a completed result or a Pending result, so I can decide whether and when to poll or retrieve the recovered object.

**Why this priority**: Recovery completes the archived-object workflow while keeping asynchronous work and follow-up retrieval under caller control.

**Independent Test**: Fake-transport tests cover a successful Recover result, a permitted Pending result with exact correlation bytes, operation errors, malformed payloads, and one-exchange/no-retry behavior.

**Acceptance Scenarios**:

1. **Given** a Recover request with an optional Unique Identifier, **When** it is encoded, **Then** the request follows §6.1.42 Table 288 and preserves its supplied or omitted identifier.
2. **Given** Recover completes successfully, **When** its response is decoded, **Then** the required Unique Identifier and typed operation result are returned; any later Get remains a separate caller action.
3. **Given** an asynchronous Recover request receives Pending, **When** the result is decoded, **Then** the Pending outcome and exact Asynchronous Correlation Value remain available through the shared result model for an explicit Poll.
4. **Given** Recover returns a failure, malformed success payload, or transport failure, **When** the result is surfaced, **Then** the common status/error and delivery-state contracts apply and the client does not automatically poll or retry.

### Edge Cases

- An optional Unique Identifier is omitted; the request remains omitted and existing batch ID Placeholder semantics are preserved.
- A request contains an ID Placeholder whose validity depends on an earlier batch item; the shared ordered batch and response-correlation rules determine its handling.
- A successful operation response omits or mis-types its required Unique Identifier; the client rejects the payload as a protocol error.
- A server returns a non-success Result Status or a Pending result; the common result model preserves the status, operation-specific Result Reason, permitted message, and asynchronous correlation value without treating failure or Pending as success.
- A Recover request is sent without the request conditions that allow asynchronous outcomes; a Pending response is rejected under the shared KMIPKIT-0007 contract.
- A timeout or disconnect occurs after a request may have been sent; delivery state is preserved and the operation is not retried.
- Unknown raw enum values and supported extension data remain subject to the shared generic TTLV and result-preservation rules; this feature does not invent server policy from unknown values.

## Requirements

### Functional Requirements

- **KMIPKIT-0018-FR-001**: The client MUST provide typed request and response models for Activate, Archive, Destroy, and Recover, each associated with its exact KMIP operation identifier and the corresponding request, response, and error tables cited in the normative scope.
- **KMIPKIT-0018-FR-002**: The client MUST preserve the optional request Unique Identifier, including omission, and MUST require and return the Unique Identifier required by each successful response table. Existing valid batch ID Placeholder semantics MUST remain available.
- **KMIPKIT-0018-FR-003**: Activate and Destroy MUST expose the request and response defined in §6.1.1 Tables 164–165 and §6.1.15 Tables 208–209. The client MUST NOT implement the server-only object-state effects as local mutation or claim that a response establishes additional remote state.
- **KMIPKIT-0018-FR-004**: Archive MUST expose the client MAY in `KMIPKIT-REQ-SPEC-6.1.4-001` by allowing a caller to request archival under §6.1.4 Tables 173–174. The API and documentation MUST describe this as a preference request and MUST NOT claim that a successful response proves archival has completed.
- **KMIPKIT-0018-FR-005**: Recover MUST expose the operation under §6.1.42 Tables 288–289 and trace `KMIPKIT-REQ-SPEC-6.1.42-001-001` and `KMIPKIT-REQ-SPEC-6.1.42-001-002`. It MUST surface Pending only under the existing asynchronous contract; callers may explicitly Poll or Get using the shared APIs, and KMIPKit MUST NOT perform either follow-up automatically.
- **KMIPKIT-0018-FR-006**: For all four operations, Success, Failure, and a Pending result permitted by the shared asynchronous rules MUST follow the shared message, result, and client-execution contracts. Operation-specific Result Reason, permitted Result Message, asynchronous correlation bytes, malformed-success behavior, and transport delivery evidence MUST be preserved or rejected according to those contracts. A malformed success MUST NOT produce a typed successful outcome.
- **KMIPKIT-0018-FR-007**: Each explicit operation invocation MUST perform at most one request exchange. The client MUST NOT automatically retry, poll, or issue a follow-up Get.
- **KMIPKIT-0018-FR-008**: The feature MUST preserve unknown values and accepted extension data through the existing generic TTLV and response model and MUST NOT infer unlisted server policy or change the established tag-allocation policy.
- **KMIPKIT-0018-FR-009**: Every applicable client requirement and operation-table obligation MUST have a stable traceability link to the pinned OASIS section/table, this specification, implementation, and executable verification. Server-only clauses for Activate and Destroy MUST be identified as server behavior and MUST NOT be recorded as client requirements.
- **KMIPKIT-0018-FR-010**: Operation values, response bodies, and any secret-bearing data MUST NOT be logged, formatted, or included in errors; existing secret redaction and memory ownership rules remain in force.

### Key Entities

- **Managed Object Identifier**: The optional request and required successful-response identifier that selects or names the object under the operation-specific tables.
- **Lifecycle Operation Request**: One explicit Activate, Archive, Destroy, or Recover request with the fields permitted by its request table.
- **Lifecycle Operation Result**: The correlated Success, Failure, or permitted Pending outcome, including its operation-specific response payload and shared result metadata.
- **Asynchronous Correlation Value**: The opaque exact value used by the existing client to associate a later explicit Poll with a Pending Recover result.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0018-SC-001**: All four operations have typed request and response representations that accept valid table-shaped messages and reject malformed successful payloads.
- **KMIPKIT-0018-SC-002**: Every applicable client requirement and operation-table obligation in the catalog is mapped to this specification, implementation, and at least one executable test (100% traceability).
- **KMIPKIT-0018-SC-003**: Deterministic tests demonstrate exact operation correlation, error preservation, Recover Pending behavior when allowed, and no more than one exchange for each explicit invocation.
- **KMIPKIT-0018-SC-004**: English and Spanish user guides describe all four operations without claiming client-side state mutation, automatic follow-up, or stronger server guarantees than the response establishes.

## Assumptions

- KMIPKIT-0006, KMIPKIT-0007, and KMIPKIT-0009 provide the shared message, batch, ID Placeholder, result, delivery-state, and asynchronous outcome contracts needed by these operations.
- The client's role is to encode the request and expose the server's result. Object-state transitions, archival timing, access control, and server policy remain server responsibilities unless the catalog identifies a client requirement.
- Recover followed by Poll or Get consists of separate caller-controlled operations; this specification adds no composite recovery workflow.
- The pinned KMIP 2.1 sources and checked-in catalog are present locally; implementation and tests do not fetch OASIS materials at build time.
