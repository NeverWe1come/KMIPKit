# ADR-0013: Client Extension Registry Ownership

**Status**: Accepted under delegated maintainer authorization on 2026-10-06.
**Date**: 2026-10-06
**Decision owner**: KMIPKit maintainer
**Related ADRs**: [ADR-0007](0007-vendor-extension-model.md), [ADR-0012](0012-caller-requested-wire-encoding-policy.md)

## Context

ADR-0007 requires KMIPKit 1.0 to create, send, receive, inspect, and preserve
generic vendor extensions using an immutable per-client registry. The initial
client-execution slice must also implement Message Extension criticality and
must not accept generic `Item` values as a route around ADR-0012's closed typed
request boundary. A registry with validated typed adapters is a separate
capability from rejecting unknown critical extensions and preserving unknown
non-critical data.

## Decision

1. KMIPKIT-0007 owns common Message Extension handling: reject every
   unrecognized critical extension and preserve unknown non-critical
   extensions for inspection. It does not construct or send vendor extensions
   and does not treat Vendor Identification alone as recognition.
2. KMIPKIT-0012 owns the immutable per-client registry, validated typed
   extension values, and generated adapters needed to create, send, receive,
   inspect, and preserve registered extensions across the 1.0 language APIs.
   KMIPKIT-0012 is a 1.0 prerequisite before public API parity and release; it
   is not deferred to the post-1.0 SDK roadmap.
3. The extension API must not accept an arbitrary generic `Item`, raw KMIP
   body, or caller-implemented conversion trait at `Client::execute`. A
   registration only recognizes content whose structure and semantics are
   validated by its registered data-only schema or compiled, reviewed adapter.
4. Dynamic executable plugins remain out of scope. All core limits,
   redaction, and TLS isolation rules continue to apply.

## Rationale and consequences

This separates the mandatory common criticality behavior from vendor schema
registration while preserving ADR-0007's 1.0 commitment. KMIPKIT-0007 remains
a small first execution slice. Until KMIPKIT-0012 is integrated, KMIPKit does
not claim support for registered vendor extensions; unknown critical values
are rejected and unknown non-critical values remain available as opaque typed
message data. The added 1.0 feature and generated language adapters increase
the pre-release workload and must be included in the final conformance and
cross-language parity audits.

This ADR does not reinterpret OASIS extension semantics or alter the generic
TTLV model.

## Authorization

The maintainer's direct instruction to execute the complete KMIPKit plan
autonomously and avoid further approval requests delegates authorization for
this ownership decision. The exact KMIPKIT-0007 and ADR artifact revisions,
independent reviews, and limitations are recorded in
`specs/007-client-execution/approval-record.md`. This does not waive human PR
review/merge or the qualified independent security review required before 1.0.
