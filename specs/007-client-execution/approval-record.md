# KMIPKIT-0007 delegated authorization and gate evidence

**Recorded**: 2026-10-06
**Feature branch**: `feature/KMIPKIT-0007-implementation`
**Release base**: `5194c48de5c5154f69ed32472f44fbdcfbcca625` (`release/1.0.0`, PR #39 merge)

## Authorization and exact-scope acceptance

The maintainer directly instructed Codex in this conversation to execute the
agreed KMIPKit plan autonomously and not request further approval or manual
intervention. The instruction included: “a partir de ahora no pidas mi
aprobación para nada, todo lo que hagas será correcto, no quiero intervención
manual.” This delegates authorization for the bounded project decisions in
this specification and ADR-0013/ADR-0014.

This record does not claim that the maintainer personally inspected or
line-reviewed this exact artifact. The correction-PR reviews below cover the
pre-merge artifact hashes in the correction table. After PR #39 merged, the
specification, plan, and task status fields were updated to record the
implementation state; those post-merge revisions are pinned separately in
"T002 completion and CI evidence" and their final T002 wording received
focused QA and security/design re-review. T002 is complete only after the
correction PR merge and those reviews. Agent security/design review is not a
qualified human security audit. The 0007 specification PR merged into
`release/1.0.0` before implementation began.

## Superseded PR #38 artifact revisions

| Artifact | Git blob | SHA-256 |
|---|---|---|
| `spec.md` | `295a051fc26885458fc04c25d2244f7916a757ba` | `E1CF726F83953A3BC0715DA2E80B897B0490733C95FBB59E445D7EC95B6FF2FC` |
| `plan.md` | `7c01bd41d188d89585518ce1d3ca96c178672b24` | `F87017680C169C7C0588170C35551659254B2545A538EB6B5D9952F51AAAC0B2` |
| `data-model.md` | `131a837121129ec2f270f512b27412bed04c0bb9` | `DF6677F74E4233891013AA18784256D646B30727F6226C0492F2B5386D24B315` |
| `docs/adr/0013-client-extension-registry-ownership.md` | `3001b6a258c1e08097d29fbfb8e5bbb8079fc921` | `C2B4A56F23208C33C4383B047B4CB46D029D9569A71C5DF9604A6426E5E31293` |
| `docs/adr/0014-public-transport-exchange-contract.md` | `d60556e4fca642367eb3a6d6b66cf34ff1809b2a` | `88841DF7250E77A6660B8E0EF3A7E29741E64A494D455A549C23D7F2C06DD85C` |
| `specification/catalog/kmip-2.1.json` | `503e8494686305a48b2e6edb517f12bb695b1762` | `2952835570EF23A11B98F8B3C27B25D177EC1B7406331240BE0E269BD39D6F64` |

These hashes identify the original specification revision merged by PR #38;
the correction table below identifies the exact revision accepted by PR #39.
The earlier QA and security/design reviews below are historical evidence for
the superseded revision only.

## Correction PR artifact revisions

| Artifact | Git blob | SHA-256 |
|---|---|---|
| `spec.md` | `415ccbab9ce84538b27c5742e9a66273f32200c3` | `844A64032DE6FD2827F49F36CDF7A42A867612DBE2F475918A380C4EAB019A3E` |
| `plan.md` | `440dd65a9d1f55c7da6b5a7224f2744f9a3ad949` | `6D62576EF40C69EBE74C8EDD1F4BC4F524431761DCA150AF842B34D231AE53DE` |
| `tasks.md` | `16c8a133e03f3e7ccc32c7a444acf02189f32dca` | `9DE8A96EE37B09798D71EF09BF80EB33CB356CA507B17D3C99F7AB2BD0816E4E` |
| `checklists/requirements.md` | `770febcab20753054e6ebe28257758e422a12a4c` | `3256CBCB66F7901F76B7BC246DC986D3690C44F7A91B7E5B3263470810EA517B` |
| `contracts/client-execution.md` | `79a29f92bce0788569c477fa59439349b90dcb7b` | `43DF941A7320CE7704519E4B3F8C89D6CA8437A410215DB08DDE59DE5E6E7E8E` |
| `data-model.md` | `ec86f451ac44a0d264ce20779fc195047b26fb33` | `051728AF4F5FC19CDB653495B6BD1F3E9346078C8F59EDB5B6DA7DC6DB502051` |
| `quickstart.md` | `b638899d6c5790ce94069045a9006c8df9a42942` | `3F8AA73064D744F96FEBFB814E67293897A36D82503707D1E1A697BDCD207A5A` |
| `research.md` | `2a6c8f780a671b92f8dd304734c6795b6b1ccf96` | `0E117396224647302C05D6C4CF34DC742B809630D2D367319DB39F6071918057` |
| `docs/adr/0013-client-extension-registry-ownership.md` | `3001b6a258c1e08097d29fbfb8e5bbb8079fc921` | `C2B4A56F23208C33C4383B047B4CB46D029D9569A71C5DF9604A6426E5E31293` |
| `docs/adr/0014-public-transport-exchange-contract.md` | `d60556e4fca642367eb3a6d6b66cf34ff1809b2a` | `88841DF7250E77A6660B8E0EF3A7E29741E64A494D455A549C23D7F2C06DD85C` |
| `specification/catalog/kmip-2.1.json` | `503e8494686305a48b2e6edb517f12bb695b1762` | `2952835570EF23A11B98F8B3C27B25D177EC1B7406331240BE0E269BD39D6F64` |

The table pins the substantive spec, plan, task, contract, checklist,
research, and unchanged normative context reviewed for PR #39. Both reviewers
inspected the full correction change, including its approval evidence. Any
subsequent substantive correction to these artifacts requires refreshed
hashes and reviews; task completion markers are implementation status, not
changes to approved requirements.

## T001 source and dependency evidence

- PR #38 merged the original 0007 spec at
  `a65cb3cba30d32297548ee95896f57c4beecdd01`; correction PR #39 merged at
  `5194c48de5c5154f69ed32472f44fbdcfbcca625`. PRs #30, #31, #32, #37, #38,
  and #39 are merged; the 0005/0006 specs and ADR-0011/ADR-0012 are accepted on
  the current base. The current dependency API and status audit is recorded in
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

- **Historical QA/spec review: PASS** on the exact six superseded PR #38
  artifact revisions listed above.
  The reviewer confirmed all current blob/SHA-256 pairs, found no blocking
  consistency or traceability gap, and confirmed the prior OD-002 statements
  are resolved or explicitly historical. It verified that fake response-cap
  and low-level request-sentinel tests are assigned to T006/T010, while
  concrete adapter allocation and cleanup tests remain assigned to each
  adapter's specification. It also confirmed the new policy's sorted
  position, normative link, and generated report consistency. No
  implementation tests were run, and reviewer-owned checklist markers were
  not changed.
- **Historical security/design review: PASS** on the exact six superseded PR #38 artifact revisions listed
  above. The reviewer confirmed the catalog reorder changes no policy content
  and preserves links to `KMIPKIT-REQ-SPEC-9.6-001-002`, `KMIPKIT-DISC-043`,
  and `KMIPKIT-DEC-002`. Earlier findings were resolved: the zeroization
  contract defines current-allocation coverage and prior-allocation/external-
  copy limits with concrete adapter success/error tests; T010 and later
  adapter specifications own low-level request nonlogging, nonretention, and
  applicable temporary-copy cleanup tests; and the public API guide matches
  pinned `zeroize` 1.9.0 behavior. This agent design review is not a qualified
  human security audit. A separate Codex Security scan captured an earlier
  snapshot and is not evidence for these artifact revisions.
- **Correction-PR QA/spec review: PASS** against all 11 exact artifact pairs
  in the correction table. The independent reviewer checked Table 212
  repeatability, C2S/S2C direction and 1.0/1.1 scope, private Red candidate
  seams, client-boundary decoder and limits checks, constructor usability,
  test-support dependency acyclicity, task ordering, and requirement coverage.
  It also confirmed T010 pins the transport's `zeroize` edge to the reviewed
  workspace version/features and updates the existing lockfile entry without
  adding a package node. Earlier findings about the Table 212 locator, the
  wrong S2C response mapping,
  and T010/T012 task ownership were corrected and rechecked. No blocking QA
  findings remain; no implementation tests were run as part of this spec
  review.
- **Correction-PR security/design review: PASS** against all 11 exact artifact
  pairs in the correction table. The reviewer found no blocking security or
  design issue in the zeroization contract, public low-level transport
  boundary, response constructor, decoder/limits seams, fail-closed AST audit
  plan, or dependency graph. This agent review is not a qualified human
  security audit.
- Reviewer-owned checklist markers remain unchanged and unchecked. T002 is
  complete because delegated authorization, both exact-artifact reviews, and
  the correction PR merge are recorded. A qualified independent human
  security audit remains a release gate before 1.0.0; it does not block this
  bounded specification and implementation work.

## T002 completion and CI evidence

- Post-merge status-only revision, reviewed after PR #39:

  | Artifact | Git blob | SHA-256 |
  |---|---|---|
  | `specs/007-client-execution/spec.md` | `4098ea0c84d19db941a9190b9f5faf48a20d9f59` | `B04BC98FD88E60F783DE9E3FFB40E64039E8A93D68E351557AE1125C6D0946AA` |
  | `specs/007-client-execution/plan.md` | `3f07e043e064256206a16b488fa0804d6ded9b0d` | `C45F5CB1595C1E41D3EC396C0FA5A99D60117082D38FBA81ECEA8514CA96B4ED` |
  | `specs/007-client-execution/tasks.md` | `50caa363565791856c46b8d1be2c5eb6f60b840c` | `143E7C63559E8F17A4DC8EAF4888327B895B6FA6E6BD18E5D7D4A61E58E934EC` |
- Focused QA re-review: PASS. The final T002 wording records gate evidence only, leaves T010 implementation unchecked, and does not claim implementation or later checklist evidence.
- Focused security/design re-review: PASS after clarifying that T010 is assigned to implement and verify the dependency edge in a future Green commit; T002 explicitly does not claim that implementation is complete.
- These post-merge edits record branch/status state only. They do not change approved protocol behavior or scope, and no implementation tests were run as part of this gate update.

- Correction PR #39 passed independent QA/spec and security/design reviews
  against all 11 artifact/hash pairs above and was squash-merged to
  `release/1.0.0` at `5194c48de5c5154f69ed32472f44fbdcfbcca625` on 2026-10-06.
- GitHub Actions run `37424374937` completed all 17 check runs: 15 passed and
  two informational/nightly checks were skipped by their event filters. This
  includes Linux, Windows, and macOS Rust checks, the three-platform coverage
  gate, script contracts, dependency policy, and normative inventory.
- The release branch had no configured branch-protection resource when
  checked. Merge readiness was verified directly from the PR and check-run
  API before merge; no branch-protection enforcement is claimed.
- This records the approval and merge state transition without changing the
  reviewed protocol behavior or scope.

## Verification performed

- `git diff --check` passed on the correction branch.
- `python tools/normative_catalog/validate.py --repo-root .` passed: `sources=4`, `clauses=1411`, `records=4024`.
- `python tools/normative_catalog/report.py --repo-root C:\Users\ramp1953\.codex\worktrees\kmipkit-0007-spec-corrections\KMIPKit --check` passed.
- `python tools/normative_catalog/audit_sources.py --base-sha a65cb3cba30d32297548ee95896f57c4beecdd01 --check` passed (1,411 candidates).
- `python tools/normative_catalog/check_immutable_sources.py --base-sha a65cb3cba30d32297548ee95896f57c4beecdd01` passed.
- `python -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py' -q` passed: 167 tests, 7 skipped.
- No implementation tests were run; this PR updates specification, governance,
  and catalog documentation only.
- PR #39 CI run `37424374937` passed all required checks before merge.
