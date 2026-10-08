# KMIPKIT-0013 delegated authorization and design gate

**Recorded**: 2026-10-07
**Feature branch**: `feature/KMIPKIT-0013-production-transport`
**Release base**: `99778e2df019e9ec71d6866c64a6a440b6e65148` (`release/1.0.0`, after PR #48)

## Authorization and acceptance

The maintainer directly instructed Codex in this conversation to continue the
KMIPKit roadmap autonomously and not request further approvals or manual
intervention. This delegates acceptance of the bounded KMIPKIT-0013 design
and ADR-0015 decisions recorded here.

This record does not claim that the maintainer personally inspected or
line-reviewed these exact artifacts. The authorization does not waive human
review and approval of the eventual draft PR, human-only merging, or the
qualified independent human security audit required before 1.0.0. Agent
security/design review is not that qualified human audit. No implementation
or release readiness is claimed by accepting this design.

Under that delegated authorization:

- The KMIPKIT-0013 specification and ADR-0015 are accepted for implementation.
- ADR-0015 partially supersedes ADR-0005's transport implementation choices;
  ADR-0005's TLS 1.3, mTLS, and no-retry security policy remains in force.
- The canonical connection model is amended: HTTPS may reuse a healthy HTTP/1
  connection; raw TLS closes after exactly one response frame.
- TLS 1.3 session tickets are configuration-local, capped at 16, expire after
  one hour of local monotonic time, and inherit the peer and trust/CRL decision
  from the verified full handshake. Rebuilding the client applies changed
  trust inputs with an empty ticket cache.
- DNS `max_active_requests` is explicitly a per-multiplexed-upstream-connection
  bound, not an aggregate per-client limit. HTTPS sends exactly one `Host`
  header derived from endpoint authority, independent of request target and
  TLS verification-name override.

The canonical updates are in `docs/design/project-definition.md`,
`docs/architecture/transport-security.md`, `docs/adr/0005-transport-and-tls.md`,
and accepted `docs/adr/0015-asynchronous-transport-worker.md`.

## Independent design reviews

- **Original QA/specification review: PASS.** The reviewer checked the requirement/task
  map and final design corrections. Findings about buffer-cleanup task mapping,
  the missing HTTPS `Host` tests, IPv6 `Host` serialization, and rebuilt-client
  trust-cache behavior were incorporated. The original final check confirmed
  FR-001–FR-017 coverage and all eight success criteria linked to planned
  verification; it did not include the later-discovered registry binding
  requirement recorded below. ADR-0015 and canonical updates were checked for
  consistency; no ADR acceptance language remained pending at that review.
- **Original security/design review: PASS.** The reviewer checked DNS bounds, HTTPS
  `Host`, session-resumption trust lifetime, request/response delivery races,
  `SendRequest::ready()` ordering, and the canonical security policy. The DNS
  limit is scoped to each multiplexed upstream connection; the resumed-session
  trust snapshot is bounded to one hour; and full handshakes perform chain,
  validity, hostname, and configured CRL checks. The reviewer confirmed there
  were no material security or specification contradictions in the reviewed
  transport design; the later registry binding clarification is recorded
  below.

Both reviews are independent of the implementer's analysis, but neither is a
human PR approval or the required qualified human security audit. Reviewer-
owned implementation checklists remain unchecked until implementation
evidence exists.

## Cross-artifact analysis and implementation gate

The initial Spec Kit cross-artifact analysis found 17 functional requirements
with task coverage, all eight buildable success criteria assigned to
verification/evidence tasks, and 62 ordered tasks (`T001`–`T062`). Its
initially identified consistency and coverage findings were fixed before
acceptance. A later independent QA follow-up identified the KMIPKIT-0012
registry-to-production-client binding gap; the amendment and its coverage are
recorded below. T001 records the original gate and canonical updates; code and
dependency-manifest changes remain gated by dependency review and the
Red/Green/Refactor tasks.

No implementation tests were run during this specification review. The pinned
OASIS upstream source copies remain unchanged.

## Cross-specification QA follow-up: client registry binding

The 2026-10-07 “registry provenance seal” section of
`specs/012-vendor-extension-registry/verification.md` records that each
registry-validated outbound value carries private provenance for the immutable
registry that validated it. It identifies the remaining
KMIPKIT-0013 responsibility: the production client must retain its
`ClientConfiguration`, compare attached extension provenance at the beginning
of execution, reject a mismatch as sanitized `InvalidInput`/`NotSent` before
request construction, and cover the behavior with a public client test.

This accepted KMIPKIT-0013 contract clarification adds FR-018, expands SC-007
and User Story 4, specifies separate ownership of `ClientConfiguration` and
transport configuration in the plan/data model, and records the corresponding
delivery-state rule. T046a is the Red public Rust integration task; T047 and
T048 include its Green and Refactor evidence. The same-configuration case is
required to preserve the existing valid `ClientRequestMessageExtension`
encoding and execution path.
`TransportConfig` remains transport-only and gains no registry field. The
task coverage map links FR-018 and SC-007 to these tests. The revised package
contains 18 functional requirements and 63 task entries. Existing FR/SC
identifiers and T001–T062 identifiers remain stable; T046a is the only added
task identifier. No transport behavior is added by this clarification, and
no code or tests were run for this documentation update.

## Resolver architecture correction: native system resolution

**Recorded**: 2026-10-08

The maintainer's standing direct instruction to continue autonomously and not
request manual approval also delegates acceptance of the narrowly scoped
resolver correction described here. It does not waive the human-only merge,
eventual qualified human security audit, or release-publication constraints
recorded above.

Independent architecture review confirmed that Hickory 0.26.3 flattens
configured upstream DNS servers and cannot preserve per-interface split-DNS
and VPN routing. The same review examined a replacement using
`std::net::ToSocketAddrs` within Tokio `spawn_blocking` and found it coherent
under the conditions captured in ADR-0016: acquire a shared permit before
submission; keep that permit until the native call exits; scope the 32-permit
limit to one loaded KMIPKit library instance; make admission fail fast; retain
at most 16 OS-ordered addresses; and discard late results before candidate
TCP/TLS connection or KMIP dispatch. A started native lookup cannot be
interrupted and OS DNS work may continue after the KMIPKit caller returns.

Under the delegated authorization:

- ADR-0016 is accepted and supersedes only ADR-0015's resolver choice and
  packet/cache limits. ADR-0015 remains accepted for its other decisions.
- KMIPKIT-0013 FR-013, research, plan, data model, timeout contract, tests, and
  dependency review will use the native system resolver contract. The
  dependency-review record is completed by T003a after T002a changes the lockfile.
- T002a removes Hickory from production dependencies; cross-platform
  system-resolver behavior remains an implementation and CI verification gate.
- The 32-permit shared governor is a narrow exception to the preference
  against global mutable state and must receive independent security/lifecycle
  review.
- T012a, T002a, and T003a extend the implementation plan from 63 to 66 task
  entries; Red/Green/Refactor boundaries remain separate.

No platform-specific split-DNS test environment is available in CI by default;
the feature claims delegation to the operating system's resolver, while
deterministic injected-resolver tests verify KMIPKit-owned admission,
cancellation, result isolation, candidate ordering/caps, and bounded shutdown.
This design acceptance does not claim implementation completion or an external
human security review.
