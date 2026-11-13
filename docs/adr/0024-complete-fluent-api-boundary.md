# ADR-0024: Complete Fluent API Boundary

**Status**: Proposed for human review
**Date**: 2026-11-11
**Specification**: [KMIPKIT-0093](../../specs/093-fluent-complete-api/spec.md)

## Context

The maintainer directed KMIPKit to implement the complete fluent master plan
in the 1.x product line. KMIPKIT-0091 supplies a first fluent client path;
KMIPKIT-0092 specifies three more object operations. The requested plan adds
all catalogued operations, expert tag placement, and three response encodings
in four languages. Existing ADR-0012/0014 limit high-level transmission to
validated typed requests, while ADR-0018 retains one zeroizing response tree.

## Decision

1. Operation-specific fluent objects are adapters over existing typed Rust
   requests and results. A typed operation may carry explicit expert Items,
   but the public `Client::execute` type remains closed and the final payload
   passes source-backed validation before the private encoder.
2. The client request namespace is for client-to-server operations. The
   server-to-client namespace receives objects from the listener and returns
   typed handler responses. Both share the catalog and value types, without
   manufacturing a client-side route for a server-only operation.
3. The retained semantic response tree supplies explicit TTLV hex, JSON, and
   XML inspection. Rendering never happens in diagnostics, `Debug`, or error
   messages. Rendered copies are caller-owned; Java/Python runtime copies
   cannot be promised to zeroize. Response properties remain absent unless
   present on the wire.
4. Version-specific symbols come only from pinned OASIS assignments. The
   KMIP 2.1 default and the explicit KMIP 3.0 working-draft route are retained.
   The master plan's KEM codes are corrected to `0x41`/`0x42`.
5. The current procedural and typed APIs remain supported. Generated names
   are additive and preserve existing ABI/source symbols.

## Consequences

Each language presents a simple operation object while Rust remains the only
KMIP construction and validation owner. Positional expert insertion raises
the validation and secret-handling burden and therefore requires negative
tests and full source traceability before its PR is ready. Rendering a secret
response into a `String` creates a caller-managed copy; documentation must
state this clearly.
