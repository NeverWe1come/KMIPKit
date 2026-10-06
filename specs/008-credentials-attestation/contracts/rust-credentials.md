# Proposed Rust Contract: Credentials and Attestation

This is a design contract for review, not an existing API. Exact names may change during the approved implementation plan, but the behavioral invariants below may not.

## Public types

```rust,ignore
pub struct Authentication { /* non-empty ordered credentials */ }
pub struct Credential { /* raw-preserving type and value */ }
pub enum CredentialValue {
    UsernameAndPassword(UsernameAndPasswordCredential),
    Device(DeviceCredential),
    Attestation(AttestationCredential),
    OneTimePassword(OneTimePasswordCredential),
    HashedPassword(HashedPasswordCredential),
    Ticket(TicketCredential),
    Extensions(OpaqueTtlv),
    Unknown { credential_type: u32, value: OpaqueTtlv },
}
pub struct Nonce { pub id: SecretBytes, pub value: SecretBytes }
pub enum RawEnumeration { Known(u32), Unknown(u32) }
```

The snippet is illustrative. It must be reconciled with the `kmipkit-ttlv` and KMIPKIT-0006 public contracts before implementation. `Authentication::new` rejects an empty list. Typed values validate field structure without discarding the retained ordered generic tree. Unknown types and enum values remain lossless. No `Debug`, `Display`, or error includes credential payload contents.

## Construction and conversion

- Provide caller-facing constructors for each known variant and validate only source-backed structural requirements plus resolved project policy.
- Convert to/from generic TTLV without computing cryptographic values. Preserve unknown fields and source order when converting an existing tree.
- `HashedPasswordCredential` requires and preserves caller-provided hash bytes and Timestamp. An absent algorithm reports effective SHA-256 while retaining absence; explicit algorithm values are preserved raw. No hash calculation or monotonicity check/test is specified until OD-003 review settles owner, comparison scope, and clock behavior.
- `AttestationCredential` requires Nonce and Attestation Type and at least one evidence field. It preserves both evidence fields if provided and preserves server Nonce bytes exactly.
- Every Table 412 Device field is representable and preserved. As a conservative typed-client interpretation (not wording stated verbatim by §9.11), a typed Device Credential Value must contain at least one of Device Serial Number, Network Identifier, Machine Identifier, or Media Identifier; Password or Device Identifier alone cannot satisfy the §9.11 uniqueness SHALL. Field presence does not impose non-empty text. The caller is responsible for supplying one or a combination of those identifiers that is actually unique. The comparison scope is unspecified by the source, and KMIPKit does not verify uniqueness from client-local data. Generic TTLV can retain unvalidated trees.
- `Authentication` is a standalone in-memory value and does not require an execution API. Wiring it into execution is gated by OD-005 and an explicit KMIPKIT-0007 handoff settling inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch.

## Secret contract

Sensitive inputs are wrapped in redacted types, and KMIPKit-owned copies are zeroized on drop/consumption under the approved secret lifecycle. The guarantee covers initialized bytes before owner deallocation, not spare or uninitialized capacity, buffers left by pre-transfer reallocations, caller/dependency copies, borrowed-view copies, temporary stack/register copies, or foreign-runtime copies; capacity is covered only when initialized and cleanup is verified. Do not expose a raw debug escape hatch. Credential and Authentication values remain in-memory; KMIPKIT-0008 adds no Credential writer, Authentication selection, or secret-bearing send path. Its request-header changes set only the non-secret Attestation Capable Indicator in the existing synchronous and asynchronous builders. Any later secret-bearing path belongs to a separate approved feature that owns the candidate callsite and its owner-through-transport lifecycle test.

## Test contract

Tests are derived from Specification §§9.3, 9.4, 9.11, 9.14, and 11.11 and Tables 402–416, 419, and 442. No official test vector is claimed. Property tests cover unknown raw enum values and opaque child preservation with deterministic bounded input. Malformed input tests check missing required values, duplicate singleton fields, wrong TTLV types, empty Authentication, redaction, and exact Nonce/credential byte preservation. Typed Device tests reject values with none of the four §9.11 identifier members; test Password and Device Identifier preservation alongside at least one identifier member, and preserve field presence independently of text length. Generic TTLV tests retain unvalidated trees. Do not test client-side uniqueness enforcement because comparison scope is unspecified. The execute-boundary test verifies indicator=True through the existing writer with Authentication and Credential payload absent. Do not test or claim client enforcement of §9.4's lowercase “must” sentence or the informative OTP single-authentication wording. Do not implement Hashed Password timestamp monotonicity (OD-003).
