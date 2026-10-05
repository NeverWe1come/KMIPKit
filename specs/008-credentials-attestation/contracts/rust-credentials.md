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
- `HashedPasswordCredential` accepts caller-provided hash bytes and Timestamp. An absent algorithm reports effective SHA-256 while retaining absence; explicit algorithm values are preserved raw.
- `AttestationCredential` requires Nonce and Attestation Type and at least one evidence field. It preserves both evidence fields if provided and preserves server Nonce bytes exactly.
- Device minimum-field validation cannot be finalized until OD-002 closes; global uniqueness is not locally checked.
- Authentication placement/default override behavior cannot be wired to `Client::execute` until OD-005 and the KMIPKIT-0007 contract are accepted.

## Secret contract

Sensitive inputs are wrapped in redacted types, and KMIPKit-owned copies are zeroized on drop/consumption under the approved secret lifecycle. The guarantee covers initialized bytes before owner deallocation, not spare or uninitialized capacity, buffers left by pre-transfer reallocations, caller/dependency copies, borrowed-view copies, temporary stack/register copies, or foreign-runtime copies; capacity is covered only when initialized and cleanup is verified. Do not expose a raw debug escape hatch. This feature adds no production credential writer or send path regardless of OD-004. Any later send path belongs to a separate approved feature that owns the candidate callsite and its owner-through-transport lifecycle test.

## Test contract

Tests are derived from Specification §§9.3, 9.4, 9.11, 9.14, and 11.11 and Tables 402–416, 419, and 442. No official test vector is claimed. Property tests cover unknown raw enum values and opaque child preservation with deterministic bounded input. Malformed input tests check missing required values, duplicate singleton fields, wrong TTLV types, empty Authentication, redaction, and exact Nonce/credential byte preservation.
