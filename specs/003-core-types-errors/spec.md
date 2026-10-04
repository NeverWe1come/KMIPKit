# Feature Specification: KMIP Shared Error Contract

**Feature Branch**: `feature/KMIPKIT-0003-core-types-errors`

**Created**: 2026-10-04

**Status**: Approved — scope preauthorized by the maintainer's direct instruction to execute the roadmap autonomously.

**Input**: User description: "Implement the common KMIPKit foundation for errors and shared protocol result values, including lossless handling of unknown KMIP values, request delivery certainty, redaction, and safe cause categories."

## User Scenarios & Testing

### User Story 1 - Inspect a KMIP operation result (Priority: P1)

An application developer needs to distinguish a server result from a local failure and inspect its status, reason when present, and optional message without losing values the client does not recognize.

**Why this priority**: Applications must react to server results without confusing them with local failures or losing forward-compatible protocol data.

**Independent Test**: Supply successful, failed, pending, undone, and unknown result values and verify that each remains distinguishable and intact.

**Acceptance Scenarios**:

1. **Given** a failed result with a known status and reason, **When** an application inspects it, **Then** both values and any present message are available.
2. **Given** an unknown status or reason value, **When** an application inspects it, **Then** the original value is available and has not been coerced or discarded.
3. **Given** a result without Result Message, **When** an application inspects it, **Then** the message is absent.
4. **Given** a represented Success or Failure result, **When** its reason is inspected, **Then** Success has no Result Reason and Failure has one.

### User Story 2 - Assess request delivery after failure (Priority: P1)

An application developer receives a local failure while handling a request without a complete KMIP operation result and needs to know whether it was not sent, may have been sent, or response reception had begun. A complete KMIP operation result is reported separately. The library reports the strongest available evidence. Request execution and retry behavior belong to a later client feature.

**Why this priority**: Repeating a state-changing operation after an ambiguous network failure can cause unintended effects.

**Independent Test**: Exercise failures before transmission, during transmission, after a zero-byte read, and after the first response byte arrives; verify the reported delivery state.

**Acceptance Scenarios**:

1. **Given** failure before any request bytes are sent, **When** the application inspects the error, **Then** delivery is reported as not sent.
2. **Given** transmission began and no response byte has been received, **When** the application inspects the error, **Then** delivery is reported as possibly sent.
3. **Given** a read returns zero response bytes, **When** the application inspects the error, **Then** delivery remains possibly sent.
4. **Given** at least one response byte arrived but the response could not be completed, **When** the application inspects the error, **Then** response reception is reported as begun.

### User Story 3 - Diagnose failures without disclosing secrets (Priority: P1)

An application developer needs useful failure categories without credentials, key material, or raw KMIP bodies appearing in default error text or library logs.

**Why this priority**: Diagnostics must support investigation without turning failures into a secret disclosure path.

**Independent Test**: Construct each local failure category with a synthetic source containing a secret sentinel and verify its safe category is available, the sentinel is absent from the public source chain, and default Display/Debug output omits it.

**Acceptance Scenarios**:

1. **Given** a local failure with a source cause containing sensitive text, **When** the error and its public source chain are inspected, **Then** a safe cause category remains available and the original untrusted source text is unreachable.
2. **Given** an error associated with secret-bearing input, **When** default Display/Debug formatting or library logging occurs, **Then** the secret, Result Message text, and raw KMIP body are absent.

### Edge Cases

- Status or reason values are unknown to this client version.
- Result Message is absent, empty, or contains arbitrary server-provided text.
- The server returns pending or undone status; it must not be collapsed into success or failure.
- Failure occurs before transmission, after transmission begins, after a zero-byte read, or after at least one response byte arrives.
- A cause contains sensitive text; default formatting must not leak it.

## Requirements

### Functional Requirements

- **KMIPKIT-0003-FR-001**: Consumers MUST be able to distinguish KMIP server results from local validation, protocol-processing, and transport failures.
- **KMIPKIT-0003-FR-002**: Every received KMIP Result Status and Result Reason value MUST be retained exactly, including values unknown to this client version.
- **KMIPKIT-0003-FR-003**: The result contract MUST preserve whether Result Message is absent or present and preserve its text when present.
- **KMIPKIT-0003-FR-004**: A local request-handling failure without a complete KMIP operation result MUST expose exactly one delivery state: not sent, possibly sent, or response reception had begun. `ResponseStarted` requires evidence that at least one response byte was received; a read attempt that yields zero bytes remains `PossiblySent`. This state MUST reflect the strongest available evidence and MUST NOT imply that a retry is safe.
- **KMIPKIT-0003-FR-005**: Errors MUST preserve a safe cause category so consumers can identify the failure layer. Arbitrary source text and payloads MUST be discarded before retention; an unsafe original cause MUST NOT remain reachable through the public error or `source()` chain.
- **KMIPKIT-0003-FR-006**: Any Display/Debug implementation for ResultMessage, KmipOperationResult, and public client errors, and library-generated logs, MUST NOT include credentials, private keys, secret key material, one-time passwords, tickets, server Result Message text, or raw KMIP message bodies. This feature MUST NOT add automatic serialization of server message or error source text.
- **KMIPKIT-0003-FR-007**: Failure results MUST remain distinguishable from success, pending, and undone statuses. This feature MUST NOT assign operation-specific meaning to Result Reason values.
- **KMIPKIT-0003-FR-008**: A represented Failure result MUST include Result Reason, and a represented Success result MUST NOT include Result Reason. The model MUST reject these inconsistent combinations without inventing a value.

### Normative Traceability

| Requirement ID | Normative level | OASIS source and clause | Normative requirement | Implementation behavior |
|---|---|---|---|---|
| KMIPKIT-0003-NR-001 | MAY; assigned values | OASIS KMIP Specification v2.1, §9.19 Result Status and §11.47 Result Status Enumeration | §9.19 describes the response status and says the listed values MAY be set; §11.47 defines their assignments. | Represent each assigned status faithfully and retain unknown values under FR-002. |
| KMIPKIT-0003-NR-002 | SHALL; SHALL NOT | OASIS KMIP Specification v2.1, §9.18 Result Reason and §11.46 Result Reason Enumeration | Result Reason SHALL be present with Failure and set as specified; it SHALL NOT be present with Success. §11.46 defines assigned reasons. | Enforce the Failure/Success invariant and preserve present values, including unknown values under FR-002. |
| KMIPKIT-0003-NR-003 | MAY | OASIS KMIP Specification v2.1, §9.17 Result Message | Result Message MAY be returned as a descriptive Text String. | Preserve its presence and text; do not automatically format or log it. |

The normative source is the immutable checked-in copy at specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html. Preservation of unknown values and the safe display/logging policy are project-level requirements, not additional OASIS claims. The no-automatic-retry rule remains a client-wide boundary and is not implemented by this value/error contract. IDs remain stable as code and tests are added.
### Key Entities

- **Client failure**: A local validation, protocol-processing, or transport failure with a safe cause category and exactly one request delivery state.
- **KMIP operation result**: Server-reported Result Status, optional Result Reason, and optional Result Message.
- **Request delivery state**: The strongest available evidence about whether transmission did not begin, may have reached the server, or at least one response byte was received.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every represented failure is distinguishable as a KMIP server result or a local validation, protocol-processing, or transport failure.
- **SC-002**: All standard Result Status and Result Reason values are represented, and unknown values remain exact in every public API exposing them.
- **SC-003**: Every local request-handling failure without a complete KMIP operation result exposes one of the three delivery states, with deterministic tests proving zero-byte reads remain PossiblySent and the first received response byte advances to ResponseStarted.
- **SC-004**: Secret sentinel checks find no credentials, key material, OTPs, tickets, Result Message text, raw body bytes, or unsafe source text in default Display/Debug formatting, the public source chain, or library-generated logs.
- **SC-005**: Every applicable normative requirement has a stable ID and linked verification target before implementation is complete.

## Assumptions

- This feature establishes a shared contract for later protocol, transport, and binding specifications; it does not implement those layers.
- Complete typed response parsing and validation are delivered later. This contract defines value-level result invariants but does not parse wire bytes.
- The normative catalog owns generated lists of KMIP values; this feature specifies lossless handling without duplicating that catalog.
- Applications decide what action to take based on reported results and delivery state; the library never retries automatically.
- Result Message is untrusted server text. Consumers may inspect it explicitly, but default error formatting and library logs omit it.
- Error sources are untrusted. Only safe cause categories are retained; arbitrary source text and payloads are discarded before they can be reached through public error APIs.

## Exclusions

- TTLV encoding, decoding, or validation of arbitrary response bytes.
- Complete KMIP request/response models or operation-specific Result Reason rules.
- TLS, HTTPS, transport implementations, or the client request loop.
- C ABI, Java JNI, Python CFFI, and language-specific error translation.
- Generated definitions for the full normative catalog.
