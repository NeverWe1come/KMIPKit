# Feature Specification: Java and Python Fluent Lifecycle Facade

**Feature Branch**: `feature/KMIPKIT-0115-java-python-lifecycle`
**Created**: 2026-11-13
**Status**: Approved under the user's standing authorization to implement the complete Fluent DX plan.
**Input**: User authorized autonomous completion of the approved KMIPKit Fluent DX plan across Rust, C, Java, and Python.

## User Scenarios & Testing

### User Story 1 - Complete common lifecycle tasks from Java (Priority: P1)

A Java application can use task-named builders and direct client helpers to
activate, archive, locate, recover, and revoke managed objects. The application
uses typed identifiers, typed Attribute collections, and typed revocation
reasons; it does not assemble operation codes or TTLV messages.

**Why this priority**: Java currently exposes only basic Get and Destroy in
its fluent facade, leaving ordinary object lifecycle workflows to manual batch
construction.

**Independent Test**: A compiled Java consumer builds all five request items,
rejects invalid inputs locally, and completes each operation against the
controlled mutual-TLS fixture while reading the typed response result.

**Acceptance Scenarios**:

1. **Given** a connected client and typed identifier, **When** the caller builds
   Activate, Archive, or Recover, **Then** the builder creates the matching
   typed request and one explicit execution returns the server result.
2. **Given** a typed Attribute collection and optional Locate filters,
   **When** the caller executes Locate, **Then** only supplied filters appear
   in the request and the response exposes the returned identifiers.
3. **Given** a typed revocation reason, **When** the caller executes Revoke,
   **Then** the reason, optional message, and optional compromise date are
   preserved and the message is absent from diagnostics.

### User Story 2 - Complete common lifecycle tasks from Python (Priority: P1)

A Python application can use `client.request()` builders and named direct
client methods for the same five operations, with the same input, execution,
and result behavior as Java.

**Why this priority**: Python currently exposes only basic Get and Destroy in
its fluent facade, leaving common lifecycle operations without task-level
entry points.

**Independent Test**: A Python consumer constructs all five typed requests,
exercises local validation and ownership cleanup, and completes the operations
against the controlled mutual-TLS fixture with typed responses.

**Acceptance Scenarios**:

1. **Given** a typed identifier, **When** the caller uses
   `client.request().activate(...)`, `.archive(...)`, or `.recover(...)`,
   **Then** the builder creates and executes the corresponding typed request.
2. **Given** typed Attributes and optional Locate filters, **When** the caller
   executes Locate, **Then** the request preserves their types and presence,
   and the response returns the fixture's identifiers.
3. **Given** a typed revocation reason, **When** the caller executes Revoke,
   **Then** its reason, optional message, and optional compromise date are
   preserved without exposing the message in representations or errors.

## Edge Cases

- An absent identifier remains absent only where the existing typed request
  model permits it; the `*_by_id` convenience methods require an identifier.
- Locate requires an Attribute collection, including an intentionally empty
  collection; omitted optional filters remain omitted.
- Invalid option records, invalid filter ranges, wrong handle kinds, and
  malformed reason inputs fail before transport and leave no response handle.
- Pending and failure outcomes remain visible through existing typed response
  accessors; the facade performs no Poll, retry, or state simulation.
- Request Options affect only exchange-local timeouts. The asynchronous
  Request Header indicator remains a separate explicit builder setting.

## Requirements

### Functional Requirements

- **KMIPKIT-0115-FR-001**: Java and Python fluent request namespaces MUST
  expose Activate, Archive, Locate, Recover, and Revoke builders using the
  existing typed request-item constructors.
- **KMIPKIT-0115-FR-002**: Each builder MUST provide local validation,
  `build`/`toBatchItem`, and one-exchange `execute` behavior through the
  existing client; it MUST preserve the existing Success, Pending, Failure,
  delivery-state, and correlation result contracts.
- **KMIPKIT-0115-FR-003**: Locate MUST require the existing typed Attribute
  collection and MUST preserve the exact presence and values of maximum items,
  offset, storage-status mask, and object-group-member filters.
- **KMIPKIT-0115-FR-004**: Revoke MUST require an explicit typed revocation
  reason and MUST preserve the optional reason message and compromise date.
  Reason messages MUST NOT appear in `toString`, `repr`, `Debug`, or errors.
- **KMIPKIT-0115-FR-005**: Java `KmipClient` and Python `Client` MUST expose
  named direct helpers for all five operations, including overloads or named
  `options` for exchange-local timeout overrides.
- **KMIPKIT-0115-FR-006**: Request Options and the Request Header asynchronous
  indicator MUST remain separate. No helper may retry, Poll, simulate remote
  state, or submit arbitrary TTLV messages.
- **KMIPKIT-0115-FR-007**: C-owned handles and Java/Python wrapper copies MUST
  follow existing transfer and release rules. Documentation MUST state the
  existing Java/Python runtime-copy limitations for caller-owned text.
- **KMIPKIT-0115-FR-008**: Every new public declaration MUST be registered
  under this spec in `specification/api/public-api.json`; generated outputs
  MUST be regenerated and audited with the repository tool.
- **KMIPKIT-0115-FR-009**: Controlled mutual-TLS tests MUST decode all five
  request types in both languages and return encoded typed responses; invalid
  local input tests MUST prove no request was sent.

## Normative References

- OASIS *KMIP Specification v2.1*, §6.1.1, Activate Operation.
- OASIS *KMIP Specification v2.1*, §6.1.4, Archive Operation.
- OASIS *KMIP Specification v2.1*, §6.1.28, Locate Operation.
- OASIS *KMIP Specification v2.1*, §6.1.42, Recover Operation.
- OASIS *KMIP Specification v2.1*, §6.1.44, Revoke Operation.
- KMIPKIT-0093-FR-004 through FR-008 and FR-011 through FR-013.
- KMIPKIT-0101-FR-001 through FR-005 for the matching Rust operation
  builders and existing typed semantics.
- KMIPKIT-0114-FR-001 through FR-005 for the new C lifecycle entry points.

## Key Entities

- **Operation Builder**: A language-level value that owns or borrows typed
  operation inputs, builds one typed batch item, and executes through the
  current client.
- **Locate Attributes**: An ordered typed Attribute collection with optional
  paging and object-selection filters.
- **Revocation Inputs**: An explicit raw-preserving reason code, optional
  message, and optional compromise-occurrence date.
- **Operation Result**: The existing typed batch result, including the server
  outcome and transport delivery evidence.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0115-SC-001**: Java and Python each build and execute all five
  operations without manually constructing a batch or TTLV message.
- **KMIPKIT-0115-SC-002**: Every builder has positive request-shape coverage
  and negative local-validation coverage; all ten cross-language operations
  pass against controlled mutual-TLS responses.
- **KMIPKIT-0115-SC-003**: Existing language-package tests, public API
  generation, and export audits pass without removing or renaming declarations.
- **KMIPKIT-0115-SC-004**: The new production declarations meet the repository's
  90% line-coverage and complete traceability gates.

## Assumptions

- KMIP behavior, validation, and response interpretation remain owned by the
  existing Rust typed request and response implementations.
- The existing Java and Python `ClientBatchItem` constructors, `AttributeSet`,
  `RevokeRequest`, and `RequestOptions` are sufficient; this feature adds no
  native request routes or ABI layouts.
- Rekey, Rekey Key Pair, Re-certify, and attribute mutation/read operations
  remain separate follow-on slices because their request structures and
  semantic inputs differ from the five operations in this feature.
