# Fluent Attribute Read Bindings (KMIPKIT-0109)

**Feature Branch**: `feature/KMIPKIT-0109-fluent-attribute-read-bindings`  
**Base**: `feature/KMIPKIT-0108-fluent-attribute-ops` at `79a2141ee08ce9df503345c5afaeadee694ecd9a`  
**Status**: Approved for implementation under the maintainer-authorized complete KMIPKIT-0093 plan.  
**Owning identifier**: KMIPKIT-0109

## Purpose

Give C, Java, and Python callers a task-oriented way to issue Get Attributes
and Get Attribute List without assembling a batch themselves. The new language
facades use the existing typed KMIP requests, response accessors, client
exchange path, and Rust ownership rules. They complement the Rust builders
delivered by KMIPKIT-0108.

## User Scenarios and Testing

### Story 1 — Read selected attributes (P1)

An application selects an object identifier and ordered Attribute References,
then performs one Get Attributes exchange and inspects the actual typed
response. The response preserves a missing `Attributes` field versus a
present empty structure, server Failure, and Pending correlation.

**Independent test**: a consumer in each binding constructs the operation and
uses a controlled TLS 1.3/mTLS peer that decodes the request and returns a
valid encoded response. The test checks the operation, identifier, ordered
references, one exchange, typed result, and response fields.

### Story 2 — List available attribute names (P1)

An application selects an optional identifier or omits it for an enclosing
placeholder-aware batch, then obtains the ordered Attribute References in a
Get Attribute List response.

**Independent test**: each binding constructs either an operation-specific
batch item or a standalone request, verifies omission is preserved, and checks
the ordered typed response from a controlled transport.

### Edge Cases

- A duplicate Get Attributes reference is rejected before transport.
- A missing identifier remains absent in the built typed request and batch
  item; standalone execution follows the existing client rule for unresolved
  ID Placeholder values.
- Invalid handles, null arrays with nonzero lengths, and invalid references
  return the existing binding error without sending a request.
- Success responses do not synthesize absent fields; Failure responses retain
  their status and reason without fabricating operation payloads.
- Pending results retain their correlation value and do not trigger Poll or a
  second exchange.

## Normative and Architectural Sources

- OASIS KMIP Specification v2.1 §6.1.20, Tables 223–225: Get Attributes
  request and response payloads.
- OASIS KMIP Specification v2.1 §6.1.21, Tables 226–228: Get Attribute List
  request and response payloads.
- OASIS KMIP Specification v2.1 §§8–9: request/response messages and common
  operation result behavior.
- `specs/0108-fluent-attribute-reads/spec.md`: Rust builder contract and
  existing typed execution behavior.
- `specs/093-fluent-complete-api/spec.md`: KMIPKIT-0093-FR-004, FR-005,
  FR-008, FR-011 and the four-language parity requirement.
- ADR-0012: typed request execution boundary; ADR-0021: approved TLS and
  listener behavior.

## Functional Requirements

- **KMIPKIT-0109-FR-001**: C, Java, and Python MUST expose operation-specific
  task-oriented entry points for Get Attributes and Get Attribute List. Each
  entry point MUST construct the existing typed operation request and MUST NOT
  require callers to manually create a batch.
- **KMIPKIT-0109-FR-002**: Get Attributes MUST preserve the caller's ordered
  Attribute References and optional Unique Identifier. Duplicate references
  MUST be rejected by the existing typed request validator before transport.
- **KMIPKIT-0109-FR-003**: Get Attribute List MUST preserve an omitted
  Unique Identifier and MUST NOT synthesize an identifier or ID Placeholder.
- **KMIPKIT-0109-FR-004**: Both operation builders MUST support explicit
  validation, conversion to the existing typed batch item, exchange-local
  options, and an explicit Request Header Asynchronous Indicator where the
  language's current facade supports those choices.
- **KMIPKIT-0109-FR-005**: Standalone execution MUST use the existing client
  and perform at most one exchange. It MUST preserve Success, Pending, Failure,
  Result Reason, correlation, delivery state, and fields actually present in
  the response. It MUST NOT retry, Poll, fabricate response fields, or accept
  a caller-supplied raw message.
- **KMIPKIT-0109-FR-006**: C MUST use stable `kmipkit_` symbols, fixed-width
  types, opaque owned handles where needed, and existing `int32_t` status and
  release conventions. Java MUST expose idiomatic camelCase methods and
  Python MUST expose snake_case methods with named arguments and existing
  exception/ownership behavior.
- **KMIPKIT-0109-FR-007**: The public API manifest, generated exports,
  requirement traceability, and English/Spanish guides MUST describe the
  delivered capability. No generated file may be edited manually.

## Acceptance Criteria

1. Failing consumer tests exist for each language before implementation.
2. C ABI tests are compiled and invoked from C, including invalid inputs and
   one controlled TLS exchange for each operation.
3. Java and Python consumer tests verify idiomatic calls, ownership, exact
   request shape, ordered values, typed results, and one controlled TLS
   exchange for each operation.
4. Invalid builders fail before transport; valid standalone execution uses
   exactly one exchange; batch conversion preserves omitted identifiers.
5. Format, lint, language suites, coverage, generation, export audit, and
   exact-head CI pass before removing `[WIP]`.

## Explicit Limits

This delivery adds only the C, Java, and Python facades for Get Attributes and
Get Attribute List. It does not change Rust request or response models, wire
encodings, server-initiated behavior, protocol versions, automatic Polling,
retry policy, or generic TTLV send boundaries. Existing lower-level typed
request and batch APIs remain supported.

## Assumptions

- The approved KMIPKIT-0093 scope authorizes this additive binding work.
- Existing per-language typed result and response-view types are the canonical
  representation; no new result status enum is introduced.
- Java and Python runtimes may retain copies according to their existing
  memory-management rules; KMIPKit-owned native handles follow current release
  semantics.
