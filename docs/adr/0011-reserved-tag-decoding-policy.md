# ADR-0011: Reject Received Reserved TTLV Tags

**Status**: Proposed
**Date**: 2026-10-04
**Decision owner**: KMIPKit maintainer
**Related discrepancy**: `KMIPKIT-DISC-037`

## Context

The KMIP 2.1 Chapter 11 introduction says implementations SHALL NOT use Tag Values marked Reserved. Section 11.56 lists Reserved ranges that can appear in data received from a peer. The normative catalog records an unresolved project question (`KMIPKIT-DISC-037`): whether a generic decoder should reject a received Reserved Tag or retain it opaquely while preventing it from being emitted as a supported Tag.

The approved KMIPKIT-0004 public generic tree requires an allocation-checked `Tag` for every `Item`. It has no opaque Reserved-tag node. The allocation policy in accepted ADR-0010 rejects values classified Reserved after applying its exact-entry precedence rule. Unknown Tags in the 0x54 Extensions range remain accepted by that policy.

## Proposed Decision

1. A TTLV decoder MUST reject a received Tag classified as Reserved before constructing the public generic `Item` tree.
2. The error MUST be payload-free and may expose only safe structural metadata such as the byte offset and numeric Tag.
3. An allocation-checked Tag accepted by ADR-0010, including an unknown Tag in the 0x54 extension range, remains eligible for generic decoding.
4. Exact individual catalog assignments and Reserved classifications continue to follow the accepted precedence rule in ADR-0010.
5. The codec MUST NOT create or expose an opaque path that could accidentally re-encode a Reserved Tag as supported data.

This is a proposed KMIPKit policy. It does not claim that OASIS explicitly requires decoders to reject received Reserved Tags, and it does not change the OASIS registry classification.

## Rationale

Rejection is the smallest policy consistent with the existing public model and the normative prohibition on implementation use. Preserving a Reserved Tag would require a separate raw wire representation, conversion rules, and safeguards against emitting prohibited values. It could not be retained in the current public generic tree without weakening the checked-Tag invariant. Unknown extension Tags remain forward-compatible through the accepted 0x54 extension range.

## Consequences

- Decoder tests must distinguish assigned Tags, accepted extension Tags, and Reserved/unallocated Tags.
- A peer response containing a Reserved Tag fails decoding as a whole; no partial public tree is returned.
- `KMIPKIT-DISC-037` may be closed with this ADR's decision ID after human review and catalog validation.
- If interoperability evidence later requires opaque receipt, that behavior needs a new reviewed specification and a separate wire-level representation; it must not be added implicitly to `Item`.

## Alternatives Considered

- **Retain Reserved Tags opaquely**: Rejected for 1.0 because the public model cannot represent them and this would add a second tree/API boundary.
- **Preserve arbitrary raw TTLV bytes and defer the decision**: Rejected because it would retain untrusted, potentially secret payloads outside the model's explicit redaction and zeroization behavior, and could lead to accidental re-emission.
- **Reject only on encoding, retain on decoding**: Rejected because no safe public destination exists in the current generic model.

## Implementation Gate

This ADR remains Proposed until reviewed and accepted under repository governance. KMIPKIT-0005 implementation must not implement the Reserved-tag branch before acceptance. When accepted, update the `KMIPKIT-DISC-037` catalog record's state and decision reference using the reviewed catalog workflow; regenerate and validate all generated artifacts in the same PR.
