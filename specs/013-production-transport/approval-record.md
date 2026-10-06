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

- **QA/specification review: PASS.** The reviewer checked the requirement/task
  map and final design corrections. Findings about buffer-cleanup task mapping,
  the missing HTTPS `Host` tests, IPv6 `Host` serialization, and rebuilt-client
  trust-cache behavior were incorporated. The final check confirmed FR-001–
  FR-017 coverage, all eight success criteria linked to planned verification,
  and no remaining material QA gaps. ADR-0015 and canonical updates were
  checked for consistency; no acceptance language remains pending.
- **Security/design review: PASS.** The reviewer checked DNS bounds, HTTPS
  `Host`, session-resumption trust lifetime, request/response delivery races,
  `SendRequest::ready()` ordering, and the canonical security policy. The DNS
  limit is scoped to each multiplexed upstream connection; the resumed-session
  trust snapshot is bounded to one hour; and full handshakes perform chain,
  validity, hostname, and configured CRL checks. The reviewer confirmed there
  are no remaining material security or specification contradictions.

Both reviews are independent of the implementer's analysis, but neither is a
human PR approval or the required qualified human security audit. Reviewer-
owned implementation checklists remain unchecked until implementation
evidence exists.

## Cross-artifact analysis and implementation gate

The final Spec Kit cross-artifact analysis found 17 functional requirements
with task coverage, all eight buildable success criteria assigned to
verification/evidence tasks, and 62 ordered tasks (`T001`–`T062`). The
initially identified consistency and coverage findings were fixed before
acceptance. No critical design-analysis issue remains. T001 records this gate
and the canonical updates; code and dependency-manifest changes remain gated
by dependency review and the Red/Green/Refactor tasks.

No implementation tests were run during this specification review. The pinned
OASIS upstream source copies remain unchanged.
