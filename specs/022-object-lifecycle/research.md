# Research: KMIP 2.1 Object Lifecycle and Certificates

## Decisions

### Reuse the current protocol and execution architecture

**Decision**: Implement operation-specific request, success-response, and sanitized model-error types in `kmipkit-protocol`; route them through the existing typed request/batch/response path in `kmipkit-client`.

**Rationale**: KMIPKit already owns TTLV, object/attribute, message, result, transport, redaction, and delivery-state behavior. A parallel model or direct wire encoder would duplicate shared semantics.

**Alternatives considered**: A second operation-specific transport path or generic-only request would duplicate execution or fail the approved typed API requirement.

### Treat operation tables and the approved ledger as payload authority

**Decision**: Build each request, response, and error shape from the exact §6.1.x clause/table named in [spec.md](spec.md). Use only stable client records and interpretations already approved there.

**Rationale**: Table field presence and conditionality are distinct from client normative text. Server-side requirements, examples, and policy are not client preconditions.

**Alternatives considered**: Applying informative Usage Guide examples as new requirements or inferring server behavior exceeds scope.

### Keep server errors in the common result contract

**Decision**: Preserve Result Status, Result Reason, allowed Result Message, Pending/correlation, extensions, and delivery evidence through existing common result structures. Operation-specific errors cover only local construction or response-shape failures.

**Rationale**: OASIS error tables list KMIP results; they do not require closed client enums or local operation behavior. Open values must remain lossless.

**Alternatives considered**: Closed per-operation reason enums would reject unknown/vendor values and duplicate shared handling.

### Do not maintain remote object or lease state

**Decision**: Return Lease Time and Last Change Date exactly as received. Do not cache leases, start timers, block application use, or query object state.

**Rationale**: This client library does not execute local cryptography, and the feature excludes hidden state and remote preflight.

**Alternatives considered**: Client-managed lease expiration would add hidden state and unrequested clock/follow-up policy.

### Keep Check batch recommendations explicit

**Decision**: Document how callers apply the recommended Batch Order Option=true and Batch Error Continuation Stop or Undo using the existing batch API. Never mutate caller options or silently override them in a single-operation method.

**Rationale**: This meets the operation guidance while preserving caller control.

### Gate only binding work on KMIPKIT-0043

**Decision**: Build canonical Rust models/client methods independently. Add no ad hoc binding signatures and do not edit generated language declarations until 0043's v2 manifest maps this approved Rust contract. Complete 0043 operation tasks T043, T044, T056, and T058–T063 for four-language parity.

**Rationale**: 0043 FR-003/005/006/021 make the approved operation contract and canonical Rust API the source for language mappings; 0043 owns manifest and binding parity. This does not block Rust protocol models.

## Normative source and fixture evidence

Pinned OASIS sections and tables are listed in [spec.md](spec.md). Re-key alone has direct catalog Test Case references, KMIPKIT-TEST-CN01-2-76 through -87. No direct case ID is listed for Certify, Check, Obtain Lease, Re-certify, Re-key Key Pair, Re-Provision, Register, or Revoke. Use local pinned fixtures when present and otherwise label vectors as derived from the source tables. A case title alone is not evidence of a fixture pass.

The approved interpretations KMIPKIT-DISC-040, KMIPKIT-DISC-025 / KMIPKIT-DEC-009, KMIPKIT-DEC-010, KMIPKIT-DISC-050 / KMIPKIT-DEC-013, and KMIPKIT-DISC-051 / KMIPKIT-DEC-014 remain authoritative as recorded by the approved specification and catalog. For Certify, the maintainer selected the §6.1.6 prose: omission of Certificate Request requires an explicit public-key Unique Identifier, and the conflicting Table 179 ID Placeholder sentence is not applied to the client request. The generated certificate may still supply the ID Placeholder for a following Get. Do not edit pinned upstream HTML.
