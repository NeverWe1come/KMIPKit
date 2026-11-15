# KMIPKIT-0114 C Lifecycle Direct Helpers

**Status**: Approved

The maintainer's standing authorization to execute the complete KMIPKit plan
without further approval requests applies to this delivery.

## Purpose

Provide task-oriented C entry points for the supported object lifecycle
operations that already have typed Rust client APIs. A C caller supplies the
object identifier, typed attributes or revocation reason, client, and codec
limits; KMIPKit builds the existing typed request and performs one exchange.

## Scope

This slice adds direct C client helpers for Activate, Archive, Locate, Recover,
and Revoke. It reuses the typed request models, validation, version gates,
transport, response handles, and error mapping already owned by KMIPKit. Each
operation also has a timeout-options variant. Existing C symbols and ABI
layouts remain unchanged.

This slice does not add lifecycle state simulation, Poll, automatic retry,
arbitrary TTLV request submission, new transports, or new KMIP semantics.
Asynchronous Request Header configuration remains available through the
existing explicit batch API; timeout options do not set Request Header fields.

## Normative references

- OASIS *KMIP Specification v2.1*, §6.1.1, Activate Operation.
- OASIS *KMIP Specification v2.1*, §6.1.4, Archive Operation.
- OASIS *KMIP Specification v2.1*, §6.1.28, Locate Operation.
- OASIS *KMIP Specification v2.1*, §6.1.42, Recover Operation.
- OASIS *KMIP Specification v2.1*, §6.1.44, Revoke Operation.
- KMIPKIT-0018, Managed Object Lifecycle, for the approved typed request
  models, validation, protocol version gates, and delivery-state behavior.
- KMIPKIT-0093-FR-004, FR-005, FR-011, and FR-012.

## Functional requirements

- **KMIPKIT-0114-FR-001**: The C ABI MUST expose direct helpers for Activate,
  Archive, Locate, Recover, and Revoke. Each helper MUST build the existing
  typed request variant and MUST invoke the existing `kmipkit_client`
  execution path.
- **KMIPKIT-0114-FR-002**: Identifier inputs MUST use the existing versioned
  `UniqueIdentifier` record. Locate filters MUST use the existing typed
  `AttributeSet` handle. Revoke MUST use the existing typed revocation-reason
  handle and MUST preserve an optional compromise-occurrence date. Helpers
  MUST NOT require numeric operation codes or accept an arbitrary request
  message or transport.
- **KMIPKIT-0114-FR-003**: Each direct helper MUST execute exactly one request
  exchange and return the existing owned client batch response handle. The
  timeout variant MUST accept only exchange-local `RequestOptions`; it MUST
  NOT alter Request Header fields. No helper may retry, poll, or simulate
  remote object state.
- **KMIPKIT-0114-FR-004**: The ABI MUST return the existing fixed-width
  `int32_t` status values, preserve the existing output-handle ownership
  rules, and retain the client's delivery-state and redaction behavior.
- **KMIPKIT-0114-FR-005**: C consumer tests MUST compile and call every new
  symbol. Controlled TLS fixture tests MUST decode the actual request and
  provide encoded typed responses for every helper, including negative local
  validation cases that prove no request was sent.

## Acceptance criteria

1. Activate, Archive, Locate, Recover, and Revoke each have a direct C helper
   and a separate exchange-options helper.
2. Each helper reaches the corresponding typed Rust client method and returns
   the existing response handle; invalid inputs fail before transport.
3. A compiled C consumer calls all ten symbols and verifies invalid-handle
   status behavior.
4. The TLS fixture observes the correct operation and semantic fields and
   returns a real encoded KMIP response that the C caller reads as a typed
   response.
5. The API manifest and all generated declarations are current; ABI, coverage,
   traceability, format, Clippy, and relevant Rust/C tests pass.

