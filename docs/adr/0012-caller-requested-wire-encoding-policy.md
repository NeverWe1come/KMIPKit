# ADR-0012: Conditional Caller-Requested TTLV Wire Encoding

**Status**: Proposed
**Date**: 2026-10-05
**Decision owner**: KMIPKit maintainer
**Related specification**: [KMIPKIT-0005 TTLV Wire Codec](../../specs/005-ttlv-wire-codec/spec.md)

## Context

`AGENTS.md` §8 currently prohibits logging, formatting, serializing, or including credentials, private keys, secret key material, OTPs, tickets, and raw KMIP bodies in errors. KMIPKIT-0005 proposes generating outbound TTLV that may contain such values. ADR-0011 concerns the disposition of received Reserved Tags and does not authorize wire encoding.

The current rule remains in force. This Proposed ADR grants no exception. It records a narrow policy proposal for review alongside approval of the KMIPKIT-0005 feature specification. No OASIS clause creates this project-policy exception.

## Proposed Decision

1. Only after a human accepts this ADR and approves the KMIPKIT-0005 feature specification may the feature produce temporary outbound TTLV solely to carry a KMIP operation explicitly requested by the caller.
2. KMIPKit must own the outbound bytes in a zeroizing buffer and retain that owner through the transport write. KMIPKit clears the storage it owns when the owner is dropped; this does not guarantee erasure of caller, TLS-library, or runtime copies.
3. This proposal does not permit diagnostic or general-purpose serialization, serialization traits, logging, formatting, inclusion in errors, persistence, or unnecessary copies.
4. This proposal does not permit arbitrary inbound raw-byte retention or re-emission. Inbound messages must be handled through the decoded model; no opaque raw-body path is authorized by this ADR.
5. The final API must enforce that outbound TTLV is generated only for an explicitly caller-requested operation and must not expose general-purpose serialization. The proposed standalone `encode(&Item)` contract does not carry operation intent; resolve its visibility and invocation path before approving KMIPKIT-0005. The first client feature/spec that sends secret-bearing TTLV owns the request-path integration test, which must pass before any client sends such data. If the boundary cannot be enforced, do not approve or implement this exception.

## Rationale

A KMIP client needs a wire representation to carry a requested operation, but temporary transport bytes have a different lifecycle from diagnostics, persistent data, or a general serialization format. Restricting the proposal to one requested outbound operation and a KMIPKit-owned zeroizing buffer minimizes exposure while preserving the existing redaction and no-persistence rules. The API boundary must be reviewed because a generic encoder call alone cannot establish caller intent.

## Consequences

- Until this ADR is accepted by a human and KMIPKIT-0005 is approved, the existing prohibition remains unchanged and no secret-bearing wire-encoding exception is authorized.
- Acceptance of this ADR alone does not approve KMIPKIT-0005 or its implementation.
- The feature specification and implementation contract must preserve the limits above and provide policy/owner verification separate from OASIS conformance tests.
- KMIPKIT-0005 owns codec-level policy/owner tests. The first client feature/spec owns the integration test for the approved request-only invocation path.
- An inability to enforce request-only encoding blocks the proposed exception; it does not relax the prohibition.

## Alternatives Considered

- **Treat encoding as already exempt from the rule**: Rejected because `AGENTS.md` §8 explicitly includes serialization of raw KMIP bodies and no accepted ADR authorizes an exception.
- **Permit general-purpose serialization of secret-bearing values**: Rejected because it would broaden persistence, copying, logging, and caller-controlled output paths.
- **Retain or replay arbitrary inbound raw bodies**: Rejected because it would bypass model validation and create an uncontrolled secret-retention and re-emission path.

## Implementation Gate

This ADR remains Proposed until human review and acceptance. The FR-013 exception is usable only after both this ADR is accepted and the KMIPKIT-0005 specification is approved. Before implementation, resolve a request-bound API path that does not expose general-purpose serialization. The first client feature/spec must own and pass the request-path integration test before any client sends secret-bearing TTLV. Do not change `AGENTS.md` or treat ADR-0011 as authority for this policy.
