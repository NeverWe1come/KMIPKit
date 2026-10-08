# Transport Requirements Checklist: Production TLS and HTTPS Transports

**Purpose**: Review the completeness and clarity of the raw TLS, HTTPS, timeout, and typed-client requirements.
**Created**: 2026-10-07
**Feature**: [spec.md](../spec.md)

**Note**: This checklist evaluates requirements quality, not implementation behavior.
**Review Ownership**: Reviewer-owned. Leave unchecked until reviewed independently.

## Requirement Completeness

- [ ] CHK001 Do the raw TTLV/TLS and HTTPS scenarios each define configuration, successful exchange, malformed input, transport failure, and connection invalidation outcomes?
- [ ] CHK002 Does the typed-client story state exactly which currently implemented request variants it can execute and preserve the existing closed request boundary?
- [ ] CHK003 Are endpoint, path, trust-source, TLS-name, certificate, key, and CRL inputs all represented with their required/default/optional status?
- [ ] CHK004 Are every applicable HTTPS Client §5.3.1 requirement and its stable catalog ID linked to observable behavior and a verification path?
- [ ] CHK005 Does the specification clearly exclude server-initiated operations, other encodings, and full profile claims?

## Requirement Clarity and Consistency

- [ ] CHK006 Are connect, read, write, and total timeouts precisely defined, including the start point, progress/reset behavior, per-client/request overrides, and precedence?
- [ ] CHK007 Is zero duration distinguishable from explicit unbounded configuration in both the public API and failure semantics?
- [ ] CHK008 Are DNS resolution, TCP connect, and TLS handshake included in the connect deadline and total deadline without sending KMIP bytes early?
- [ ] CHK009 Are the raw low-level caller-byte contract and typed client's validated encode path clearly distinguished?
- [ ] CHK010 Are HTTP parser responsibilities separated from KMIPKit's status/header/body checks without promising detection the parser cannot provide?
- [ ] CHK011 Are Hyper/Tokio, the standard-library system resolver, and the per-client worker lifecycle linked to the accepted ADR changes rather than treated as implicit ADR-0005 behavior?

## Acceptance Criteria and Edge Coverage

- [ ] CHK012 Can each success criterion be objectively verified on Linux, Windows, and macOS without a live external KMIP server?
- [ ] CHK013 Do response-cap and TTLV-length criteria establish checking before allocation/growth, including arithmetic overflow and exact boundary behavior?
- [ ] CHK014 Do timeout, HTTPS reuse, and raw-TLS close-after-frame criteria define whether a timed-out or malformed request can ever be replayed or a surplus frame misattributed?
- [ ] CHK015 Do HTTPS criteria cover duplicate/conflicting headers, transfer/content encoding, non-200 status, truncation, and parser errors?
- [ ] CHK016 Are OS-owned resolver routing/retry/cache behavior, the KMIPKit governor and candidate caps, timeout/cancellation limits, and cross-platform system-resolver smoke tests sufficiently specified?
- [ ] CHK017 Are all failure results tied to `NotSent`, `PossiblySent`, or `ResponseStarted` with no raw payload or dependency error text?
- [ ] CHK021 Does the delivery-state commit occur at a defined dispatch boundary that does not depend on distinguishing HTTP headers from body bytes below Hyper?
- [ ] CHK022 Are direct production-adapter timeout overrides specified and tested separately from typed operation options?
- [ ] CHK023 Does the accepted raw-TLS close-after-frame exception reconcile the reusable-connection statements in both canonical architecture documents before implementation?
- [ ] CHK024 Can an unsolicited HTTP response on a reused connection be proven unable to satisfy a later KMIP exchange?
- [ ] CHK025 Does response-byte observation have explicit precedence when it races timeout finalization?
- [ ] CHK026 Are the full-handshake certificate checks and one-hour TLS resumption trust snapshot consistent across spec, plan, data model, contracts, and tasks?
- [ ] CHK027 Does the production client retain the immutable KMIPKIT-0012 `ClientConfiguration`
  separately from transport configuration, reject a foreign-registry `ClientRequestMessageExtension`
  before request construction/encoding/exchange as sanitized `InvalidInput`/`NotSent`, and preserve
  same-client behavior in a public Rust integration test?

## Normative Scope

- [ ] CHK018 Does each OASIS normative requirement cite the pinned document/version/section and the exact existing catalog ID?
- [ ] CHK019 Are TLS/mTLS product policy requirements kept separate from server-only OASIS clauses and profile-conditional duties?
- [ ] CHK020 Are profile test cases described as evidence and not incorrectly presented as independent normative requirements?

## Notes

- The author quality checklist is separate from this reviewer-owned checklist.
- Record findings inline or in the review response; do not use this checklist to claim implementation completion.
