# Feature Specification: KMIP 2.1 Query and Ping

**Feature Branch**: `feature/KMIPKIT-0020-query-ping`

**Created**: 2026-10-09

**Status**: Draft for review

**Input**: KMIPKit roadmap Phase D: specify bounded client-initiated Query and Ping operations for KMIP 2.1 over TTLV.

## Normative scope

This specification covers exactly two client-to-server operations:

| Operation | Catalog element | Normative source | Request/response/error tables |
| --- | --- | --- | --- |
| Ping | `KMIPKIT-ELEM-OP-C2S-PING` | OASIS KMIP Specification v2.1 §6.1.36 | Tables 271–272 |
| Query | `KMIPKIT-ELEM-OP-C2S-QUERY` | §6.1.40 | Tables 281–284; Query Function enumeration §11.44, Table 476 |

The Query request contains the required, repeatable Query Function field and optional Object Groups (Table 282). Object Groups is the §7.23 structure (Table 375), whose `Object Group` member is an optional, repeatable `Object Group` attribute; that attribute is a Text String (§4.35, Tables 99–100). Query Function values are the 14 standard values in §11.44 (Table 476), from Query Operations (1) through Query Storage Protection Masks (14), plus values in the OASIS extension range. The response fields and their occurrence rules are those in Table 283; Protection Storage Masks is a required response field whose list the server MAY return empty. Section 6.1.40 also says the response payload is empty if there are no values to return, which conflicts with Table 283's required field. The catalog records this as open `KMIPKIT-DISC-045`; the client model accepts both source-described forms without claiming which form a server must use. Query errors follow §6.1.40.1, Table 284: Operation Failed may carry Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large, or Unknown Object Group. Ping request and response payloads are empty (Tables 271–272); common KMIP message and batch semantics continue to apply.

The normative inventory records two client requirements for Query: `KMIPKIT-REQ-SPEC-6.1.40-014` (at least one Query Function) and `KMIPKIT-REQ-SPEC-6.1.40-016` (the client MAY repeat Query Function). Each exact source citation and assignment appears in [traceability.md](traceability.md). Table response clauses that place obligations on the responding server remain server requirements; this feature specifies the client's request construction and lossless handling of the response. The Usage Guide is informative.

## Product boundaries and exclusions

- The feature is limited to KMIP 2.1, TTLV, client-initiated calls, the existing request/batch model, and existing raw TLS and HTTPS transports.
- The deliverable is an additive Rust protocol model and client API. C ABI, JNI, CFFI, JSON, XML, new transports, and server-initiated operations are out of scope.
- A Query response reports data returned by the server. KMIPKit does not infer, locally verify, or promise that a listed operation, object type, extension, profile, validation, capability, or default is usable under a particular server policy.
- Unknown and vendor Query Function values, operation/object enumeration values, extension fields, and other structurally valid response Items remain representable and are not silently discarded.
- This feature does not implement server behavior, including Ping logging policy, Query response selection, Query Extension List/Map precedence, or server support decisions.
- An explicit operation call performs at most one request exchange. There is no automatic retry, polling, follow-up Query, or other hidden request.
- No OASIS upstream copy, accepted architecture boundary, TLS policy, or shared transport behavior changes.

## User Scenarios & Testing

### User Story 1 — Check whether a server responds (Priority: P1)

As a KMIPKit caller, I can issue Ping and distinguish a successful KMIP response from a transport or protocol failure, so I can check server responsiveness without implying broader health.

**Why this priority**: Ping is the smallest complete client operation and provides a direct, bounded responsiveness check.

**Independent Test**: A fake transport captures one empty Ping request and returns an empty successful Ping response; failure and transport cases retain the shared result and delivery state.

**Acceptance Scenarios**:

1. **Given** a configured client, **When** the caller invokes Ping, **Then** KMIPKit sends one client-initiated Ping with an empty request payload.
2. **Given** a valid empty Ping response with Success, **When** the caller receives the result, **Then** Ping completes successfully and does not claim service health beyond the received response.
3. **Given** a KMIP failure, malformed response, or transport error, **When** Ping completes, **Then** the common status/error and delivery evidence are preserved, with no automatic retry.

### User Story 2 — Inspect server capabilities (Priority: P1)

As a KMIPKit caller, I can request one or more kinds of server information and inspect exactly what the server returns, so I can make my own capability and policy decisions.

**Why this priority**: Query exposes the server's declared support for operations, object types, and other KMIP information while keeping policy decisions with the caller.

**Independent Test**: Fake-transport tests cover every standard Query Function, repeated functions, absent/single/repeated Object Group attributes, each Table 283 response member, both source-described successful response forms, errors, unknown values, and a single exchange.

**Acceptance Scenarios**:

1. **Given** a Query request, **When** it is encoded, **Then** it contains at least one Query Function; repeated functions and their order are preserved.
2. **Given** optional Object Groups with zero or more Object Group attributes, **When** the caller sends Query, **Then** each Text String attribute is represented in order without adding an implicit object group.
3. **Given** a successful Query response, **When** it is decoded, **Then** every present Table 283 field, its permitted repetitions, and its structurally valid nested Items remain accessible.
4. **Given** a successful Query response, **When** it uses either the empty payload form described by §6.1.40 or the structured Table 283 form with an empty Protection Storage Masks list, **Then** KMIPKit preserves that form and fabricates no information.
5. **Given** a response contains future, vendor, or otherwise unknown enum values or extension Items, **When** it is decoded, **Then** the original values remain available to the caller.
6. **Given** a Query failure or transport error, **When** the operation completes, **Then** the common status, reason, permitted message, and delivery state are retained and the request is not retried.
7. **Given** Query Extension List and Query Extension Map are both requested, **When** the client constructs the request, **Then** it transmits the caller's requested functions unchanged and leaves the standard's response precedence to the server.

## Requirements

### Functional Requirements

- **FR-001**: The Rust client MUST expose explicit Ping and Query entry points that use the existing client execution and batch semantics.
- **FR-002**: A Ping request and successful Ping response MUST use the empty operation payloads defined by §6.1.36, Tables 271–272.
- **FR-003**: A Query request MUST contain at least one Query Function and MAY contain repeated Query Function values, as defined by §6.1.40, Table 282 and §11.44, Table 476.
- **FR-004**: The Query request model MUST represent optional Object Groups as defined by §6.1.40, Table 282, including its zero-or-more repeated `Object Group` attributes from §7.23, Table 375. Each attribute's Text String value follows §4.35, Tables 99–100.
- **FR-005**: The Query API MUST expose all standard Query Function values and allow valid extension values; future/unknown enumeration values MUST remain lossless under the shared public-value policy.
- **FR-006**: The typed Query response MUST expose every member defined by §6.1.40, Table 283 with the standard's optionality and repeatability. Structurally valid nested and unknown Items MUST remain inspectable.
- **FR-007**: The Query response decoder MUST accept both forms described by the conflicting text in §6.1.40 and Table 283: an empty response payload when there are no values to return, and a structured response containing the required Protection Storage Masks field, whose list may be empty. This tolerant client behavior does not resolve the server-conformance conflict recorded as `KMIPKIT-DISC-045`.
- **FR-008**: Query and Ping MUST preserve the shared KMIP result status, result reason, permitted result message, batch correlation, and transport delivery state.
- **FR-009**: Each explicit Query or Ping invocation MUST perform at most one exchange and MUST NOT retry, poll, or issue a follow-up operation automatically.
- **FR-010**: Ping success MUST be documented as evidence that the server returned a successful Ping response, not as a guarantee of general service health.
- **FR-011**: Query results MUST be presented as server-reported information. KMIPKit MUST NOT convert them into claims that a capability is enabled, authorized, or usable.
- **FR-012**: Public diagnostics MUST follow the existing redaction rules and MUST NOT include raw KMIP request or response bodies.

### Key Entities

- **Query Function**: A known KMIP Query Function enumeration value or a losslessly preserved valid extension/future value.
- **Query Request**: One or more Query Function values and optional Object Groups.
- **Query Response**: The optional and repeatable information members defined by Table 283, preserving wire values and nested Items.
- **Ping Result**: The shared KMIP result for an empty Ping operation payload.
- **Delivery State**: The existing NotSent, PossiblySent, or ResponseStarted evidence for the request exchange.

### Edge Cases

- An empty Query Function collection is rejected before transmission with delivery state NotSent.
- Repeated Query Function values are not deduplicated or reordered by the client.
- Query response fields that are absent stay absent; repeated fields remain repeated and in received order.
- The two contradictory §6.1.40/Table 283 forms are both accepted by the client; this is not a server-conformance determination (`KMIPKIT-DISC-045`).
- Unknown enum values and extension data do not become errors solely because the current typed API does not assign them a named variant.
- A server failure never produces a fabricated typed-success payload.
- Transport failure does not trigger an implicit retry, even when delivery may have occurred.

## Success Criteria

- **SC-001**: The Query and Ping operation elements, their operation-enum values, Query Function enumeration and its 14 standard values plus extension range, Object Groups structure/member and Object Group attribute, and both client requirement IDs are assigned to KMIPKIT-0020 in the normative catalog. `KMIPKIT-DISC-045` records the unresolved response-shape conflict. The generated coverage report passes its pinned consistency check and continues to show implementation and verification evidence as pending until the implementation PR.
- **SC-002**: Focused protocol and fake-transport tests cover every Ping and Query acceptance scenario, including empty Query rejection, repeated functions, all 14 standard Query Function values, optional/repeated Object Group attributes, optional/repeated response fields, both source-described successful response forms, failures, unknown values, and no retry.
- **SC-003**: No Query or Ping operation-specific value is lost across typed model conversion to and from TTLV.
- **SC-004**: No conformance claim relies on an unavailable fixture; all referenced official test-case fixture availability and mapping caveats are documented.

## Assumptions

- The accepted KMIPKit common message, batch, TTLV, result, client execution, and transport contracts remain authoritative.
- The active branch is based on `release/1.0.0`; KMIPKIT-0019 has its own draft PR, so this feature uses the next reserved spec identifier, KMIPKIT-0020.
- All standard Query Function names/values, Object Groups shape, and Query response cardinalities are taken from the pinned OASIS Specification v2.1, not inferred from a server or Usage Guide. The conflicting Query response-shape statements remain recorded as `KMIPKIT-DISC-045`; the decoder is specified to tolerate both.
- The referenced official Ping and Query test fixtures are unavailable in the pinned repository snapshot. Their absence is not a failed test result.