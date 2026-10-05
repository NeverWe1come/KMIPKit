# Feature Specification: KMIP 2.1 Message and Batch Model

**Feature Branch**: `feature/KMIPKIT-0006-message-batch-model`
**Created**: 2026-10-04
**Status**: Draft — this specification depends on the accepted generic TTLV model in KMIPKIT-0004. It is an in-memory protocol-model feature; wire encoding remains assigned to KMIPKIT-0005.
**Input**: KMIPKit roadmap: define ordered request/response messages, headers, batch items, correlation, and asynchronous result data for a KMIP 2.1 client.

## Scope and normative sources

This feature defines the in-memory message envelope and batch model used by a client-initiated KMIP 2.1 request/response exchange. It covers message headers, batch cardinality and options, per-item operation/result fields, correlation identifiers, and representation of pending asynchronous results. It preserves generic TTLV payloads for operation bodies, credentials, attestation data, and extensions; it does not define those payload schemas or execute network operations.

The normative source is the immutable local OASIS KMIP Specification v2.1 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. The tables and clauses below are the source of field shape and order:

| Source | Requirement or record | Treatment in this feature |
|---|---|---|
| §§8.1–8.6, Tables 394–399 | `KMIPKIT-REQ-SPEC-8.1-002`, `8.2-002`, `8.3-003`, `8.3-004`, `8-003-002`; clauses `KMIPKIT-CLAUSE-SPEC-8-001` through `-003`, `8.1-001/-002`, `8.2-001/-002`, `8.3-001` through `-004`, `8.4-001/-002`, `8.5-001/-002`, `8.6-001` through `-005` | Represent request and response shapes in OASIS field order. When the client sets Asynchronous Indicator, `KMIPKIT-REQ-SPEC-8-003-002` requires it to handle mixed synchronous/asynchronous response items. Server-side duties in §§8.4–8.6 are not restated as client MUSTs; their response shapes are inputs the client must represent. |
| §§9.1–9.2, Tables 400–401, and §11.3 | `KMIPKIT-REQ-SPEC-9.1-001`, `KMIPKIT-REQ-SPEC-9.2-001`; `KMIPKIT-CLAUSE-SPEC-9.1-001`, `9.2-001`, `11.3-001` through `-004` | Preserve async fields and expose correlation data. Poll/Cancel request construction and use of the asynchronous correlation value are assigned to `KMIPKIT-0009-asynchronous-operations`; server processing duties are not client requirements. |
| §§9.5–9.9, Tables 404–408 | `KMIPKIT-REQ-SPEC-9.5-001-001/-002`, `9.6-001-001` through `-003`, `9.7-001`, `9.8-001-001`, `9.9-001`; clauses `KMIPKIT-CLAUSE-SPEC-9.5-001`, `9.6-001`, `9.7-001`, `9.8-001`, `9.9-001/-002` | Enforce count, item shape, and option presence/defaults. `KMIPKIT-REQ-SPEC-9.8-001-002/-003` describe server execution permissions and are classified server-only in this change; the client model still exposes the §9.8 absent-means-True default. The semantics conflict for Batch Error Continuation remains gated by `KMIPKIT-DISC-001`. |
| §§9.10 and 9.13 | `KMIPKIT-REQ-SPEC-9.13-001-001` through `-005`; `KMIPKIT-CLAUSE-SPEC-9.10-001`, `9.13-001` | Preserve Server Correlation Value as response metadata for client-to-server exchanges and retain it in the generic tree if found in an out-of-scope request. Validate Message Extension shape. Response items contain at most one Message Extension. The per-client immutable registry is defined by `docs/architecture/extensions.md`; ADR-0007 requires generic extension preservation and criticality handling, while `KMIPKIT-0007-client-execution` implements recognition/actions. |
| §§9.12 and 9.20, Tables 417 and 425 | `KMIPKIT-REQ-SPEC-9.12-001-001` through `-003`; `KMIPKIT-REQ-SPEC-9.20-001-001/-002`; `KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP` | Represent optional Maximum Response Size and optional request Time Stamp. The Time Stamp field uses Date-Time encoding. `-001` records its optional request presence; `-002` permits a client without a real-time clock but with a countdown timer to use a timestamp to derive relative values from Date attributes. `KMIPKIT-0007-client-execution` owns response byte-limit enforcement, the §9.12 SHOULD to include a configured size for operations likely to return large replies, and outgoing Time Stamp policy. Whether the §9.20 countdown-timer MAY can source the outgoing Time Stamp remains unresolved under OD-004. |
| §§9.16–9.21, Tables 421–426 | `KMIPKIT-REQ-SPEC-9.19-002`, `9.21-001-001/-002`; `KMIPKIT-CLAUSE-SPEC-9.16-001`, `9.19-001`, `9.21-001` | Preserve raw protocol version and distinct correlation fields. Under the accepted KMIP 2.1-only 1.0 boundary in ADR-0002, `KMIPKIT-0007-client-execution` sends and accepts only version 2.1; verification includes an exact-2.1 property check plus representative non-2.1 mismatches and signed-32-bit boundaries. This is the documented product-scope exception tracked by `KMIPKIT-DISC-022`, not a claim of §9.16 same-major backward compatibility. Response Operation/ID echo verification and runtime matching are also owned by `KMIPKIT-0007-client-execution`. |
| §10.1.2 | `KMIPKIT-REQ-SPEC-10.1.2-001` | Define and verify each known message Structure's field order. TTLV byte encoding is out of scope. |
| §§9.3–9.4, 9.11 | `KMIPKIT-REQ-SPEC-9.3-001-001/-002`, `9.4-001-001` through `-003`, `9.4-002`, `9.11-001`, `9.11-004-001` through `-003`, `9.11-006`, `9.11-010-001/-002` | Represent header fields and preserve opaque structures. `KMIPKIT-0008-credentials-attestation` owns credential construction, authentication policy, Device Identifier handling, the hashed-password timestamp/hash algorithm/default rules, and truthful Attestation Capable Indicator behavior; it must define and test the applicable conditional-presence and credential rules. `KMIPKIT-0010-profile-conformance` owns profile-specific applicability/default semantics and tests for every profile it claims. No authentication or profile behavior is implemented or claimed by this message-model feature. |

The generated inventory report lists no official Test Cases ID linked to the message-level requirements below, and all 203 referenced XML fixtures are unavailable locally. This specification therefore requires derived table-driven conformance tests against the pinned Specification clauses and records any later OASIS vectors by exact source ID; it does not invent official test evidence. The inventory identifies 40 source discrepancies. `KMIPKIT-DISC-001` (OASIS §11.5) leaves the execution/rollback meaning of Continue and Undo unresolved; this model preserves the raw option and per-item results but does not infer rollback or execution effects. `KMIPKIT-DISC-022` (OASIS §9.16) records the conflict between same-major backward compatibility and the accepted KMIP 2.1-only 1.x boundary in ADR-0002. The approved product boundary controls KMIPKit 1.0: requests use version 2.1 and the client execution layer accepts only 2.1 responses, with rejection of other major/minor pairs tested in KMIPKIT-0007. This is a documented scope exception, not a resolution of the OASIS clause or a claim of same-major backward compatibility. This feature preserves the raw major/minor pair and does not implement runtime acceptance policy.

## Clarification Record

The approved roadmap, accepted ADRs, OASIS clauses, and current layer architecture resolve the initial boundaries without a user-specific choice:

- The message model owns in-memory request/response structures and schema order; KMIPKIT-0005 owns TTLV wire bytes, lengths, and padding.
- `kmipkit-protocol` owns typed structures, conversion to/from generic TTLV items, and structural validation. `KMIPKIT-0007-client-execution` owns mechanical defaults, paired request/response checks, declared response-size enforcement, extension criticality actions, and delivery orchestration.
- This feature represents asynchronous Pending data only. Poll/Cancel/Process operation requests, use of Asynchronous Correlation Value in Poll/Cancel, and `PendingOperation` execution belong to `KMIPKIT-0009-asynchronous-operations`. The 1.0 operation inventory must include every client-initiated operation, including Process (§6.1.39).
- OASIS §11.5 continuation effects remain an unresolved source/policy gate; the model preserves those input values without resolving their execution effects. ADR-0002's KMIP 2.1-only boundary determines the 1.0 version acceptance policy; KMIPKIT-0007 owns its runtime enforcement and tests, while `KMIPKIT-DISC-022` continues to document the resulting scope exception against §9.16. `KMIPKIT-0009-asynchronous-operations` owns review of the exact normative source conflict recorded as `KMIPKIT-DISC-039` for §6.1.41 Query Asynchronous Requests response mapping, and must document and test its decision; KMIPKIT-0006 assumes no resolution. The Process operation (§6.1.39) also remains assigned to later operation-family work.
- Authentication credentials and attestation are represented opaquely until `KMIPKIT-0008-credentials-attestation` defines their typed models, Device Identifier and hashed-password rules, authentication policy, conditional-presence checks, and indicator truthfulness; that feature must test the applicable credential variants, conditional fields, and hash/default behavior against exact OASIS clauses.
- Profile-specific field applicability, defaults, validation, and conformance claims are outside this feature and belong to `KMIPKIT-0010-profile-conformance`. That specification must map each claimed profile to exact OASIS Profile clauses and executable tests; no profile is claimed until all applicable requirements and tests pass.

## User Scenarios & Testing

### User Story 1 — Construct a structurally valid request or response (Priority: P1)

A client library user needs a stable representation of request/response headers and batch items so a request can be validated and later converted to TTLV without changing the OASIS field order or losing unknown values.

**Why this priority**: Every client operation uses the common message envelope, regardless of its operation-specific payload.

**Independent Test**: Construct representative request and response models without a server; convert to and from the generic TTLV tree and compare fields, ordering, repetitions, and raw values.

**Acceptance Scenarios**:

1. **Given** a request or response with a header and one or more Batch Items, **When** it is represented as a message, **Then** the message envelope, header fields, and item fields follow their OASIS table order and Batch Count equals the item count.
2. **Given** optional, repeated, unknown, or vendor extension fields, **When** a message is converted through the generic TTLV model, **Then** their values and relative order are preserved without being interpreted as standardized fields.
3. **Given** a response with Result Status, optional Reason/Message, and operation payload, **When** represented, **Then** the shared KMIPKIT-0003 result contract is reused and no result payload appears in default diagnostics.

### User Story 2 — Preserve batch item identity (Priority: P1)

A client library user needs each batch item's Unique Batch Item ID and result to remain attached when messages are converted or inspected; `KMIPKIT-0007-client-execution` uses these IDs to match responses to requests.

**Why this priority**: Without stable per-item IDs, `KMIPKIT-0007-client-execution` cannot safely correlate responses when Batch Order Option is false.

**Independent Test**: Convert deterministic multi-item requests and responses through the generic TTLV tree with response items in a different order; verify every ID remains attached to its original item. Runtime matching is tested by the later client feature.

**Acceptance Scenarios**:

1. **Given** a request with Batch Count greater than one, **When** it is validated, **Then** every request item has a Unique Batch Item ID as required by Table 396, and KMIPKit additionally requires the IDs to be pairwise distinct as a project invariant for unambiguous correlation.
2. **Given** a single-item request, **When** it is validated, **Then** Unique Batch Item ID may be absent or present.
3. **Given** response items are in a different order, **When** the response is represented, **Then** each exact Unique Batch Item ID remains attached to its original response item. If the request item had an ID, `KMIPKIT-0007-client-execution` later verifies the response echo and performs matching.
4. **Given** Client Correlation Value or Server Correlation Value fields, **When** a message header is inspected, **Then** these Text Strings remain distinct from the per-item Unique Batch Item ID, and Server Correlation Value is only response metadata for client-to-server exchanges.

### User Story 3 — Represent asynchronous results explicitly (Priority: P1)

A client that permits asynchronous replies needs to distinguish completed and pending batch items and retain the server-provided correlation bytes for explicit follow-up operations.

**Why this priority**: KMIP 2.1 permits asynchronous results, including batches with both pending and completed items.

**Independent Test**: Construct mixed response batches with complete, failed, and Pending items; verify raw status preservation, the Pending correlation value, and absence of any implicit Poll/Cancel behavior.

**Acceptance Scenarios**:

1. **Given** an omitted Asynchronous Indicator, **When** its effective value is inspected, **Then** Prohibited is reported while field absence remains preservable.
2. **Given** a response contains mixed Pending and completed items, **When** it is represented, **Then** each item remains independently inspectable; `KMIPKIT-0007-client-execution` checks whether the associated request allowed asynchronous results.
3. **Given** a Pending result, **When** the response model is built, **Then** its Asynchronous Correlation Value is required and preserved byte-for-byte.
4. **Given** a Pending result with an unknown status or unknown Enumeration value, **When** represented, **Then** the raw value is retained and no automatic retry, Poll, Cancel, or wait begins.

### Edge Cases

- Empty requests, empty responses, a Batch Count that disagrees with the item list, and counts outside the supported integer range.
- Single-item requests with an optional Unique Batch Item ID; multi-item requests with a missing or duplicate ID.
- Responses with reordered, missing, duplicated, or unexpected IDs.
- Absent option versus explicit OASIS default, including Batch Order Option and Batch Error Continuation Option.
- Unknown response Operation, Result Status, Result Reason, and generic TTLV Enumeration values are preserved by their respective message conversions. Unknown or unassigned Asynchronous Indicator and Batch Error Continuation values are preserved by request parsing and model conversion; outbound request validation for those option values belongs to `KMIPKIT-0007-client-execution`.
- Pending status without an Asynchronous Correlation Value; non-Pending status with an optional correlation field.
- Mixed Pending and completed items are structurally representable here; `KMIPKIT-0007-client-execution` rejects Pending outcomes when the associated request did not permit asynchronous responses.
- Malformed Message Extension children and Vendor Identification characters, including duplicate response Message Extensions; this protocol model preserves extensions without classifying them. An unregistered extension is unknown: the client layer rejects an unknown critical extension and may process an unknown non-critical extension as opaque content using the per-client registry in `docs/architecture/extensions.md`.
- Protocol Version values other than 2.1 are retained by this model. Under ADR-0002, `KMIPKIT-0007-client-execution` accepts only a 2.1 response, with a property test asserting acceptance iff the pair is `(2, 1)` plus representative mismatch and signed-32-bit boundary cases; the 2.1-only boundary is a documented scope exception to §9.16 recorded by `KMIPKIT-DISC-022`.
- Sensitive credentials, extension payloads, result messages, and operation payloads never appear in formatting or errors.

## Requirements

### Functional Requirements

- **KMIPKIT-0006-FR-001**: The protocol model MUST represent Request Message, Request Header, Request Batch Item, Response Message, Response Header, and Response Batch Item using the OASIS field order in §§8.1–8.6, Tables 394–399.
- **KMIPKIT-0006-FR-002**: The protocol model MUST represent each common header field required or optional in Tables 395 and 398. Operation payload, Authentication, Attestation, and Message Extension contents MAY remain generic TTLV structures until their dedicated specifications, and MUST be preserved without loss.
- **KMIPKIT-0006-FR-003**: Request and response messages MUST contain at least one Batch Item. Batch Count MUST equal the number of items; a single operation MUST use count one (§§8.1, 9.5). Counts that cannot be represented by the signed 32-bit TTLV Integer MUST be rejected.
- **KMIPKIT-0006-FR-004**: A request with more than one item MUST carry a Unique Batch Item ID on every item; a single-item request MAY omit it. Table 396 makes the request ID required when Batch Count exceeds one; §9.21 describes the field as optional generally and requires the response to echo an ID when the corresponding request supplied it. Present request IDs MUST be pairwise distinct within a message as a KMIPKit project invariant needed for unambiguous client correlation; OASIS does not state this uniqueness rule as a separate MUST. The response model MUST represent an optional Unique Batch Item ID without losing its bytes. Response Operation is required when the corresponding request item specified Operation (Table 399); this paired-message check and response ID echo/matching belong to `KMIPKIT-0007-client-execution` (§§8.3, 8.6, 9.21). This model does not implement runtime correlation.
- **KMIPKIT-0006-FR-005**: Request item Operation and Request Payload MUST be represented; response Operation, Result Status, conditional Result Reason, conditionally permitted Result Message, optional Asynchronous Correlation Value, and Response Payload MUST be represented. Shared result validation MUST use the KMIPKIT-0003 result types and invariants.
- **KMIPKIT-0006-FR-006**: Batch Order Option MUST be absent or represented as a Boolean; it MUST be rejected for a single-item batch, and its effective default MUST be True when absent (§9.8). The model MUST preserve whether the field was absent. The server's duty to execute items in request order when the option is True (`KMIPKIT-REQ-SPEC-9.8-001-002`) is not a client requirement and is not claimed as implemented here.
- **KMIPKIT-0006-FR-007**: Batch Error Continuation Option MUST be absent or represented by its raw Enumeration value; it MUST be rejected for a single-item batch and its effective default MUST be Stop when absent (§9.6). The model MUST preserve whether the field was absent and MUST NOT infer execution or rollback effects while `KMIPKIT-DISC-001` is unresolved.
- **KMIPKIT-0006-FR-008**: Client Correlation Value and Server Correlation Value MUST remain distinct Text String fields; Unique Batch Item ID and Asynchronous Correlation Value MUST remain distinct Byte String fields. All values MUST be preserved exactly. In 1.0 client-to-server exchanges, Server Correlation Value is response metadata; the generic model preserves it if present in an out-of-scope request, but the typed outgoing client request surface MUST NOT expose a setter for it. Server-to-client request use is deferred to 1.1. Client Correlation Value MUST NOT be treated as unique or used as the per-item match key (§§9.1, 9.9–9.10, 9.21).
- **KMIPKIT-0006-FR-009**: The Asynchronous Indicator MUST preserve raw Enumeration values and expose the effective default Prohibited when absent (§9.2). When the client sets it, it MUST handle mixed synchronous and asynchronous response items (`KMIPKIT-REQ-SPEC-8-003-002`, §8). The message model MUST represent mixed Pending and non-Pending response items without losing their per-item association; `KMIPKIT-0007-client-execution` MUST reject Pending results when the associated request did not permit asynchronous responses.
- **KMIPKIT-0006-FR-010**: A Pending response item MUST contain an Asynchronous Correlation Value. The model MUST expose its exact bytes for later explicit Poll/Cancel APIs and MUST NOT poll, cancel, retry, or wait automatically (§§8.6, 9.1, 9.19; AGENTS.md §8).
- **KMIPKIT-0006-FR-011**: Message Extensions MUST preserve their full generic values, order, Vendor Identification, and Criticality Indicator. Request items MAY contain repeated Message Extensions as permitted by Table 396; response items MUST contain at most one optional Message Extension because Table 399 does not authorize repetition. This model MUST NOT classify an extension as known or unknown. The per-client immutable registry in `docs/architecture/extensions.md` determines recognition; `KMIPKIT-0007-client-execution` MUST reject the entire message for an unrecognized critical extension and MAY process the remaining message when an unrecognized extension is non-critical (§9.13, Tables 396, 399; ADR-0007).
- **KMIPKIT-0006-FR-012**: Protocol Version MUST preserve the raw major/minor values. Requests created by the KMIPKit 1.0 client default to 2.1 under accepted ADR-0002. This model MUST NOT negotiate or accept/reject versions. `KMIPKIT-0007-client-execution` MUST enforce the ADR-0002 1.0 boundary by accepting only responses with Protocol Version 2.1 and rejecting every other major/minor pair. Verification MUST include a positive `(2, 1)` case, a property test asserting acceptance iff the pair equals `(2, 1)`, and deterministic mismatch/boundary cases including signed-32-bit limits. This is a documented product-scope exception to OASIS §9.16, tracked by `KMIPKIT-DISC-022`, and MUST NOT be described as same-major backward-compatible behavior.
- **KMIPKIT-0006-FR-013**: Request/response conversion to/from generic TTLV Structures, including `RequestMessage::try_from_ttlv`, MUST preserve unknown allocation-valid Tags, all raw Enumeration values, bitmask bits, opaque fields, explicitly repeatable fields, repeated unknown fields, and caller/source order. Parsing MUST NOT apply outbound request-value policy to Enumeration fields. It MUST NOT encode/decode wire bytes or claim wire validity. Duplicate known singleton fields are rejected; unknown tags are preserved only when accepted by the generic TTLV allocation policy, which this feature does not widen.
- **KMIPKIT-0006-FR-014**: Structural errors MUST be payload-free and default Debug/Display MUST redact raw TTLV, credential, extension, operation, and Result Message contents.
- **KMIPKIT-0006-FR-015**: The feature MUST add no network I/O, automatic retries, background tasks, TLS policy, credential cryptography, operation-specific payload schema, or server-initiated operation handling.
- **KMIPKIT-0006-FR-016**: Requirement traceability MUST link every in-scope OASIS requirement or project invariant to the exact catalog/source record, implementation location, and executable test before implementation completion. Server-only duties, `KMIPKIT-DISC-001`, `KMIPKIT-DISC-022`, `KMIPKIT-DISC-039`, and credential/attestation behavior deferred to later specs MUST be visibly classified rather than silently omitted.

- **KMIPKIT-0006-FR-017**: The generic TTLV model MUST expose a scoped, read-only Structure view sufficient for protocol validation and typed access without cloning or transferring owned payloads. The message model MUST store the validated generic tree and return it by ownership when converting back, preserving zeroization and source order.
- **KMIPKIT-0006-FR-018**: The request model MUST represent Attestation Capable Indicator presence and raw Boolean value and expose the OASIS effective default False when absent. `KMIPKIT-0008-credentials-attestation` MUST set it truthfully according to whether the client can create an Attestation Credential (§9.3).
- **KMIPKIT-0006-FR-019**: `RequestMessage::try_from_ttlv` and request-model conversions MUST preserve the raw Asynchronous Indicator and Batch Error Continuation Enumeration values, including assigned values, values in the OASIS extension allocation `0x80000000..=0x8FFFFFFF`, and other unassigned values; parsing MUST NOT apply outbound value rules. When emitting a client request, `KMIPKIT-0007-client-execution` MUST accept assigned values and values in that extension allocation, and MUST reject unassigned values outside those ranges with a payload-free error (Tables 432, 435; §§9.2, 9.6). Typed Enumeration wrappers remain raw-preserving, and generic TTLV continues to preserve every raw Enumeration value.
- **KMIPKIT-0006-FR-020**: Response Result Message MUST be absent when Result Status is Success or Operation Pending and MAY be present for the other Result Status values (Table 399). This rule is enforced by response-item validation in addition to the shared Result Reason invariants.
- **KMIPKIT-0006-FR-021**: Each Message Extension MUST contain one Vendor Identification Text String, one Criticality Indicator Boolean, and one Vendor Extension Structure in Table 418 order. Vendor Identification characters MUST be limited to `[A-Za-z0-9_.]`; the generic Vendor Extension subtree and any unknown allocation-valid fields MUST be preserved. The model MUST NOT classify extension knownness. The immutable per-client registry is defined in `docs/architecture/extensions.md`; `KMIPKIT-0007-client-execution` implements recognition and criticality actions required by §9.13 and ADR-0007 (Table 418).
- **KMIPKIT-0006-FR-022**: The generic request model MUST preserve Server Correlation Value if it is present in the owned TTLV tree. The typed outgoing 1.0 request API MUST NOT expose a setter or treat it as a supported client-request field; `KMIPKIT-0007-client-execution` MUST NOT emit it in client-initiated requests. Client-to-server Server Correlation Value is response metadata; request use belongs to server-initiated 1.1 work (§9.10).

### Follow-on ownership

The following IDs reserve traceable owners in the roadmap; each remains a separate draft specification and must complete the repository's specification workflow before implementation:

| Follow-on specification | Deferred requirements and verification owner |
|---|---|
| `KMIPKIT-0007-client-execution` | Paired response Operation/ID checks; `KMIPKIT-REQ-SPEC-8-003-002` mixed synchronous/asynchronous response handling and Pending permission against the request; extension-registry criticality behavior; `KMIPKIT-REQ-SPEC-9.12-001-002` response-size enforcement; `KMIPKIT-REQ-SPEC-9.12-001-003` large-response recommendation; outbound Asynchronous Indicator and Batch Error Continuation value validation; request Time Stamp passthrough/omission policy and OD-004 disposition of whether the §9.20 countdown-timer MAY can source the outgoing field; exclusion of Server Correlation Value from outgoing client requests; ADR-0002 protocol version enforcement (send and accept only 2.1; a property test that accepts iff the pair is `(2, 1)`, a positive 2.1 case, and representative mismatch/signed-32-bit boundary tests, documenting `KMIPKIT-DISC-022` as the §9.16 scope exception). Add a deterministic fake-transport test for one response batch containing both completed and Pending items, verifying Pending is accepted when the request indicator permits asynchronous results and rejected otherwise. Add acceptance tests for assigned and extension-range outbound option values plus negative tests rejecting unassigned values outside those ranges (Tables 432, 435; §§9.2, 9.6). Add exact-limit and over-limit response-size boundary tests, and test configured Maximum Response Size on operations identified as likely to return large responses. Add focused client tests for paired responses, registry, request Time Stamp preservation/omission, and version. Defer tests of countdown-derived outgoing Time Stamps until OD-004 is resolved. |
| `KMIPKIT-0008-credentials-attestation` | Credential construction and typed models; authentication policy; Device Identifier; hashed-password timestamp, algorithm, and defaults; conditional credential/header fields; attestation models; truthful Attestation Capable Indicator behavior. Add exact-clause tests for credential variants and their required/conditional fields, hash/default behavior, Device Identifier handling, and indicator truthfulness against supported Attestation Credential creation capability. |
| `KMIPKIT-0009-asynchronous-operations` | Poll/Cancel/Process operation models and responses; carry Asynchronous Correlation Value unchanged from Pending outcomes into explicit Poll/Cancel requests; own `KMIPKIT-DISC-039` and §6.1.41 Query Asynchronous Requests response mapping. Review the exact normative source conflict, document the resulting mapping decision, and add tests for that decision; KMIPKIT-0006 assumes no resolution. Add deterministic client tests that assert exact byte-for-byte correlation preservation in both Poll and Cancel requests and ensure no automatic polling/retry. |
| `KMIPKIT-0010-profile-conformance` | Profile-specific field applicability, conditional requirements/defaults, validation, and conformance claims. Inventory the exact OASIS Profiles clauses and linked requirements for each proposed profile; add executable tests for every applicable client requirement and official vector available for each claimed profile. Do not claim a profile until its applicable clauses and tests pass. |

### Key Entities

- **Protocol Version**: Raw major and minor integer values; version compatibility policy is separate from this message representation.
- **Request/Response Header**: OASIS-ordered common fields plus optional generic structures for fields assigned to later credential and attestation specifications. A validated message owns its original ordered generic tree; typed views inspect it without cloning payloads.
- **Batch Item**: One request or response operation with a raw-preserving Operation value, optional correlation ID, payload, and per-item result fields.
- **Unique Batch Item ID**: Byte String used to match one response item to its request item.
- **Client Correlation Value**: Optional Text String request metadata, distinct from the batch ID and asynchronous correlation value.
- **Server Correlation Value**: Server-generated Text String; response metadata for client-to-server exchanges and request metadata only for server-to-client operations, which are outside 1.0 scope.
- **Asynchronous Correlation Value**: Opaque server-provided Byte String attached to a Pending result for explicit future Poll/Cancel operations.
- **Batch Outcome**: Ordered response items preserving each individual result. Mapping to a prior request belongs to the client-execution layer.
- **Opaque Field**: Generic TTLV subtree retained without assigning credentials, attestation, vendor, or operation-specific meaning.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0006-SC-001**: 100% of message envelopes, headers, and batch-item fields in OASIS Tables 394–399 are represented in the request/response model or explicitly linked to a dedicated follow-on specification.
- **KMIPKIT-0006-SC-002**: The full table-driven structural case matrix passes, and 256 deterministic generated cases with 1–8 batch items each validate successfully; generated invalid count and missing/duplicate request-ID mutations are rejected before conversion returns a partial message.
- **KMIPKIT-0006-SC-003**: In 256 deterministic generated round-trip cases with 1–8 items and test-only ID samples of 1–32 bytes, every request and response item ID survives conversion with its original item association, independent of source order.
- **KMIPKIT-0006-SC-004**: In 256 deterministic generated cases per property, arbitrary raw `u32` Enumeration values, arbitrary `u32` Bit Mask values, allocation-valid unknown Tags, repeated fields, opaque subtrees, source order, and 1–1,024-byte asynchronous correlation values survive generic-tree round trips unchanged. Property-test bounds and seed are defined in `plan.md`; they limit generated test inputs, not supported protocol values.
- **KMIPKIT-0006-SC-005**: 100% of in-scope requirements have traceability records to implementation and executable verification; every exception has a visible source-discrepancy or deferred-spec record.
- **KMIPKIT-0006-SC-006**: No sentinel from credentials, extensions, operation payloads, or Result Message appears in default formatting or protocol-model errors.
- **KMIPKIT-0006-SC-007**: Line coverage is at least 95% for changed code, at least 95% for `kmipkit-ttlv` and protocol/model code, and at least 90% for the Rust workspace overall. This feature adds no transport or FFI code.

## Assumptions

- KMIPKIT-0004's lossless ordered generic TTLV tree is accepted and remains the message model's storage/conversion substrate.
- Message schema order comes from OASIS §§8.1–8.6 and §10.1.2; KMIPKIT-0005 owns byte encoding, decoding, padding, and decoder resource limits. KMIPKIT-0006 adds only the scoped read-only Structure view needed to validate the message tree.
- Existing `ResultStatus`, `ResultReason`, `ResultMessage`, and `KmipOperationResult` from KMIPKIT-0003 remain the shared per-operation result contract.
- `kmipkit-protocol` owns typed message structures and structural validation; `kmipkit-client` later supplies mechanical defaults and performs runtime delivery/correlation orchestration.
- `KMIPKIT-DISC-001` remains an unresolved execution-semantics question. `KMIPKIT-DISC-022` remains the documented §9.16 scope exception: ADR-0002 selects KMIP 2.1 only for 1.0, and KMIPKIT-0007 must enforce and test that boundary. KMIPKIT-0006 preserves version values without implementing runtime acceptance policy.
- All client-initiated operation types, including Process (§6.1.39), remain assigned to later operation-family specifications; this feature only provides the common message envelope.

## Exclusions

- TTLV byte encoding/decoding, transport framing, size/depth/element limits, TLS, HTTPS, and network I/O.
- Operation-specific request/response payload schemas and high-level operation builders.
- Credential construction, authentication algorithms, secret lifecycle beyond the existing generic TTLV model, and attestation generation.
- Resolving the rollback meaning of Batch Error Continuation values or adding same-major backward compatibility/version negotiation beyond the accepted KMIP 2.1-only 1.0 boundary.
- Executing Poll, Cancel, Process, background wait, connection retries, or server-initiated operations.
- C ABI, Java JNI, Python CFFI, and binding generation.
