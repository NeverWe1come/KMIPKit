# Research: KMIP 2.1 Credential Sessions

## Decisions

### Normative operation contracts

Use the pinned OASIS KMIP Specification v2.1 as the authority: Delegated Login §6.1.12, Tables 199–201; Login §6.1.30, Tables 253–255; Logout §6.1.31, Tables 256–258. Use §7.35/Table 387 for Rights and §7.39/Table 391 plus §11.57 for Ticket and Ticket Type. The approved spec's field requiredness and source decisions are binding. Usage Guide descriptions are informative only.

### Ticket representation and secrecy

Represent the operation Ticket as its §7.39 structure, with an open raw Ticket Type Enumeration and a secret Ticket Value byte string. Reuse the existing secret ownership/zeroization patterns instead of adding a cryptographic provider or logging exception. Keep both fields structurally inspectable while redacting the value from Debug, errors, and diagnostics.

Alternatives considered: keeping the whole ticket opaque would lose useful typed structure; storing Ticket Value in an ordinary byte vector would violate the repository secret-handling rules.

### Nested request fields

Preserve Rights and Usage Limits as generic structurally valid TTLV structures, including order and unknown members. Preserve Lease Time as the exact approved Interval-or-Date-Time wire value. Request Count uses the protocol Integer representation. Do not validate rights against server policy or infer remote state.

Alternative considered: interpreting and restricting nested fields to locally known values would reject forward-compatible data and invent policy.

### Client execution and state

Use the existing explicit ClientRequest, typed batch response, result, delivery-state, and fake-transport contracts. Each operation is one caller-selected request. Pending remains Pending with its correlation bytes; no Poll or retry is automatic.

A returned ticket is not installed into client authentication state. Callers decide whether and how to supply it to a later operation. Login and Delegated Login therefore do not mutate the TLS identity or hidden session state.

Alternative considered: automatic ticket reuse would create implicit credential state, complicate secret ownership, and make the request behavior depend on an unrequested client policy.

### Cross-language boundary

The Rust protocol and client work can be implemented independently. KMIPKIT-0043 owns the canonical public API manifest and generator contract for cross-language signatures and parity artifacts; use the approved manifest scope in specification/api/public-api.json and docs/development/api-manifest.md. KMIPKIT-0043 is a dependency only for C, Java, and Python parity and generated artifacts, not for standalone Rust operation models or the typed Rust client.

### Conformance evidence

Use the official test-case inventory already recorded in spec.md. The pinned TC-LOGIN-1-21.xml fixture exists at specification/oasis/kmip-2.1/fixtures/TC-LOGIN-1-21.xml. Catalog decision KMIPKIT-DEC-012 maps Login test case 2.48 to the displayed filename despite the malformed raw upstream href; keep the pinned HTML immutable. Delegated Login cases have catalog IDs but no listed fixture in the approved scope. Logout has no direct catalog test-case ID. Derived tests must be identified as project tests and must not be reported as official fixture passes.

## No new dependency

The existing TTLV model, protocol result model, SecretBytes, typed client execution, and fake transport cover the implementation needs. No new runtime dependency is justified.
