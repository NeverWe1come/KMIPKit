# ADR-0010: Tag allocation precedence for generic TTLV
Status: Proposed
Date: 2026-10-04

## Context

The pinned OASIS KMIP Specification v2.1 says implementations SHALL NOT use
values marked Reserved. Section 11.56 individually assigns tags
`0x420174`–`0x420176` and also labels the overlapping wildcard range
`420XXX–42FFFF` Reserved, without stating which entry takes precedence. The
reviewed normative catalog preserves both the individual records and the
aggregate range. The same section labels `0x540000–0x54FFFF` as Extensions.

The generic TTLV value-model specification needs a deterministic tag
allocation gate so accepted individual standard tags are usable while
reserved and unused residual values are rejected. This decision concerns the
in-memory allocation gate only; it does not decide how a decoder handles a
received Reserved tag, which remains tracked separately as KMIPKIT-DISC-037.

## Decision

Proposed policy for maintainer review:

1. An exact individual catalog record takes precedence over any overlapping
   aggregate range record.
2. Individually assigned tags are accepted, and individually Reserved tags
   are rejected.
3. Values in `420XXX–42FFFF` without an individual tag record are rejected as
   residual Reserved values.
4. The complete `0x540000–0x54FFFF` §11.56 Extensions range passes the
   allocation gate, without implying that a tag's extension semantics or
   context is valid.
5. Every other unused or reserved range is rejected. Raw Tag remains able to
   represent any unsigned 24-bit value but cannot enter the checked generic
   tree unless this gate accepts it.

This is a KMIPKit project policy, not an OASIS interpretation or clarification.
Implementation is gated on maintainer acceptance of this ADR and approval of
the KMIPKIT-0004 specification.

## Consequences

- The Rust tag table must be generated deterministically from the checked-in
  normative catalog; it must not be hand-maintained or scraped from OASIS.
- Tests must cover every individual catalog assignment, individually Reserved
  tags, residual boundaries, the full Extensions range, and unrelated raw
  values.
- Passing the allocation gate does not validate TTLV framing, lengths,
  padding, Structure order, or extension-specific semantics.
- A later ADR may supersede this policy if a reviewed source clarification or
  project requirement changes the decision.

## Alternatives considered

- Treat the aggregate Reserved range as dominant and reject all of
  `420XXX–42FFFF`. This would also reject individually assigned tags and make
  those standard items unusable in the generic model.
- Accept all values with first byte `0x42` or `0x54`. This would admit unused
  and Reserved values contrary to the tag-allocation records and Reserved
  constraint.
