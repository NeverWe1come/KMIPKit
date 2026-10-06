# KMIPKIT-0007 delegated authorization and gate evidence

**Recorded**: 2026-10-06
**Feature branch**: `feature/KMIPKIT-0007-readiness-evidence`
**Release base**: `d4582e2bedd159f14d66205dae0219248eb9e7fc` (`origin/release/1.0.0`)

## Authorization and exact-scope acceptance

The maintainer directly instructed Codex in this conversation to execute the
agreed KMIPKit plan autonomously and not request further approval or manual
intervention. The instruction included: “a partir de ahora no pidas mi
aprobación para nada, todo lo que hagas será correcto, no quiero intervención
manual.” This delegates authorization for the bounded project decisions in
this specification and ADR-0013/ADR-0014.

This record does not claim that the maintainer personally inspected or
line-reviewed this exact artifact. Independent review evidence is listed
below and must cover the exact hashes recorded here before T002 is marked
complete. Agent security/design review is not a qualified human security
audit. The 0007 specification PR must be merged into `release/1.0.0` before
implementation begins.

## Exact artifact revisions

| Artifact | Git blob | SHA-256 |
|---|---|---|
| `spec.md` | `295a051fc26885458fc04c25d2244f7916a757ba` | `E1CF726F83953A3BC0715DA2E80B897B0490733C95FBB59E445D7EC95B6FF2FC` |
| `plan.md` | `7c01bd41d188d89585518ce1d3ca96c178672b24` | `F87017680C169C7C0588170C35551659254B2545A538EB6B5D9952F51AAAC0B2` |
| `data-model.md` | `131a837121129ec2f270f512b27412bed04c0bb9` | `DF6677F74E4233891013AA18784256D646B30727F6226C0492F2B5386D24B315` |
| `docs/adr/0013-client-extension-registry-ownership.md` | `3001b6a258c1e08097d29fbfb8e5bbb8079fc921` | `C2B4A56F23208C33C4383B047B4CB46D029D9569A71C5DF9604A6426E5E31293` |
| `docs/adr/0014-public-transport-exchange-contract.md` | `d60556e4fca642367eb3a6d6b66cf34ff1809b2a` | `88841DF7250E77A6660B8E0EF3A7E29741E64A494D455A549C23D7F2C06DD85C` |
| `specification/catalog/kmip-2.1.json` | `900ba49c6a18a445d3f81626bec1154c31e3015e` | `6C5DD69BC923F6019A2F1A24F319893D26030AA11D6A0B7990263E116ED3B2EB` |

These hashes identify the exact source artifacts that independent QA and
security/design review must cover in the proposed PR. The current artifact
revisions incorporate review findings about response allocation limits and
request-copy test coverage; the review evidence below remains pending for
these exact hashes. If any hashed file changes, recalculate its hashes and
repeat review for the changed exact revision before completing T002.

## T001 source and dependency evidence

- The active release base is PR #37's merge commit
  `d4582e2bedd159f14d66205dae0219248eb9e7fc`. PRs #30, #31, #32, and #37
  are merged; the 0005/0006 specs and ADR-0011/ADR-0012 are accepted on that
  base. The current dependency API and status audit is recorded in
  [`research.md`](research.md).
- OASIS Specification v2.1 source SHA-256 is
  `8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf`.
  The source-audit command examined 1,411 candidates, and the immutable-source
  check matched the full OASIS source tree to the release base.
- Catalog validation reported `sources=4`, `clauses=1411`, `records=4024`;
  generated coverage report verification passed. The §9.6 / Table 435 value
  discrepancy is separately cataloged as `KMIPKIT-DISC-043`; delegated
  project decision `KMIPKIT-DEC-002` chooses assigned values for outbound
  Batch Error Continuation in this slice and preserves raw decoded values.
  This is a KMIPKit policy choice, not an interpretation that removes the
  Table 435 extension allocation. `KMIPKIT-DISC-001` remains separate and
  open for Continue/Undo execution effects.

## Delegated design decisions

- **OD-001 / `KMIPKIT-DEC-002`**: use assigned Batch Error Continuation
  values for outbound requests in this slice; preserve raw decoded values.
- **OD-002 / ADR-0013**: 0007 owns common extension criticality handling;
  KMIPKIT-0012 owns the immutable per-client registry and validated typed
  adapters before the 1.0 public API is frozen.
- **OD-003 / ADR-0014**: `kmipkit-transport::Transport::exchange` is
  documented public low-level API, absent from the top-level facade and not
  injectable through a public `Client` constructor. It intentionally accepts
  caller-owned request bytes and returns a zeroizing `TransportResponse` as
  an escape hatch outside `Client::execute` validation and its request-owner
  guarantee. This explicitly excepts direct low-level responses from ADR-0012
  Decision 4; caller-owned response copies remain the caller's responsibility.
  Transport implementations must not log/retain request bytes, must zeroize
  initialized bytes in each current KMIPKit-owned temporary request/partial
  response allocation, and return only payload-free errors. Spare capacity,
  earlier allocations not cleared before reallocation, caller copies, and
  TLS/transport-library copies are outside this guarantee. 0007 specifies
  the exact cap and client-side check plus fake response/request tests; future
  adapter specifications own pre-allocation and allocation-growth cleanup
  tests on success and error, and applicable request nonlogging,
  nonretention, and temporary-copy tests.
- **OD-004**: preserve a caller-provided Date-Time request timestamp exactly
  or omit it; do not generate a countdown-derived outgoing value here.
- **OD-005**: require a pinned, fail-closed Rust AST regression audit with
  adversarial fixtures for the sole permit mint and writer callsite; this is
  not a language-level proof and remains subject to independent review.
- **OD-006**: the first later operation that can contain secret-bearing TTLV
  must pass an operation-specific owner-through-transport lifecycle test
  before its feature is approved or enabled.
- **Pending Asynchronous Correlation Value / FR-017**: classify it as
  capability-like sensitive metadata. Preserve exact bytes for explicit
  operations, expose only through a borrowed accessor, redact diagnostics and
  logs, and zeroize KMIPKit-owned storage at drop without an ordinary
  unzeroized duplicate.

## Independent review evidence

- **QA/spec review: PASS** on the exact six artifact revisions listed above.
  The reviewer confirmed their blob/SHA-256 pairs, found no blocking
  consistency or traceability gap, and confirmed the prior OD-002 statements
  are resolved or explicitly historical. The review also verified that fake
  response-cap and low-level request-sentinel tests are assigned to T006/T010,
  while concrete adapter allocation and cleanup tests remain assigned to each
  adapter's specification. No implementation tests were run, and
  reviewer-owned checklist markers were not changed.
- **Security/design review: PASS** on the exact six artifact revisions listed
  above. Two findings were resolved before this pass: the zeroization contract
  now defines current-allocation coverage and prior-allocation/external-copy
  limits, with concrete adapter success/error tests; and T010 plus later
  adapter specifications own low-level request nonlogging, nonretention, and
  applicable temporary-copy cleanup tests. The public API guide now matches
  pinned `zeroize` 1.9.0 behavior for the full current `Vec` capacity and its
  prior-reallocation limitation. This agent design review is not a qualified
  human security audit. A separate Codex Security scan captured an earlier
  snapshot and is not evidence for these artifact revisions.
- Reviewer-owned checklist items remain unchecked. T002 remains unchecked
  until the exact review/hash evidence is recorded and this specification PR
  is merged into `release/1.0.0`. A qualified independent human security audit
  remains required before 1.0.0.

## Verification performed

- `git diff --check` passed.
- `python tools/normative_catalog/validate.py` passed.
- `python tools/normative_catalog/report.py --check --repo-root
  C:\Users\ramp1953\.codex\worktrees\kmipkit-0007-t001-audit\KMIPKit`
  passed.
- `python tools/normative_catalog/audit_sources.py --base-sha d4582e2bedd159f14d66205dae0219248eb9e7fc --check` passed (1,411 candidates).
- `python tools/normative_catalog/check_immutable_sources.py --base-sha d4582e2bedd159f14d66205dae0219248eb9e7fc` passed.
- No implementation tests were run; this PR updates specification, governance,
  and catalog documentation only.
