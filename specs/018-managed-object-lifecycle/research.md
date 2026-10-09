# Research: KMIP 2.1 Managed-Object State Transitions

## Decision: Scope this family to Activate, Archive, Destroy, and Recover

**Rationale**: The release catalog classifies all four as client-to-server operations in `client_1_0`. Their request/response payloads have the same small outer shape, while Recover completes the archived-object workflow deferred by managed-object retrieval. Register introduces an object payload and attribute validation; Revoke and Re-key have distinct reason, date, or cryptographic parameters and should receive separate family specifications.

**Alternatives considered**: Add Register or Revoke to this family. Rejected because they have materially different payloads and normative rules that would obscure these four state-oriented operations.

## Decision: Keep server state authoritative

**Evidence**: The pinned OASIS KMIP 2.1 Specification defines request, response, and error payloads in §6.1.1 Tables 164–166 (Activate), §6.1.4 Tables 173–175 (Archive), §6.1.15 Tables 208–210 (Destroy), and §6.1.42 Tables 288–290 (Recover). §4.58 Tables 145–146 define permitted Unique Identifier encodings; §11.56 Table 487 assigns tag `0x420094` to Unique Identifier. The checked-in catalog classifies the Activate and Destroy prose clauses as `server_only`; Archive and Recover have the three client MAY requirements recorded in this specification.

**Decision**: Implement the client wire models and result handling. Do not implement remote object state, local state mutation, server policy, or stronger claims than the KMIP result establishes. Archive communicates a preference. Activate and Destroy do not transfer server responsibilities to the client.

## Decision: Reuse shared asynchronous outcomes

**Evidence**: KMIPKIT-0006 and KMIPKIT-0007 define batch and Pending behavior; KMIPKIT-0009 defines explicit Poll, Cancel, Process, and Query operations. The Recover requirements identify Poll and Get as optional follow-up actions.

**Decision**: Preserve any Pending outcome that the shared request/response rules permit, including its exact correlation bytes. Only the caller may issue Poll or Get; this feature adds no automatic polling, retry, or follow-up retrieval.

## Decision: Preserve existing identity, error, and decoder contracts

**Evidence**: All four request tables permit an optional Unique Identifier and their successful response tables require one. Shared KMIPKit contracts already define ID Placeholder use, batch correlation, error redaction, request delivery state, and bounded TTLV decoding.

**Decision**: Reuse the existing typed Unique Identifier, batch, result, and delivery-state models. Do not add local identifier generation or operation-specific error text handling that exposes raw payloads.

## Decision: Use derived tests without overstating official conformance

**Evidence**: The release catalog links no requirement-specific official Test Cases IDs to the Archive and Recover client MAY requirements. The test-case availability discrepancy is tracked in the catalog.

**Decision**: Add exact source-derived positive and negative vectors, fake-transport tests, and malformed-payload tests. Mark their provenance as derived; do not call them official OASIS test cases.

## Source hierarchy

The immutable pinned KMIP 2.1 OASIS Specification and the checked-in normative catalog are the normative sources. Usage Guide prose and examples are informative and cannot add client requirements. No network lookup or new ADR is needed for this bounded family.
