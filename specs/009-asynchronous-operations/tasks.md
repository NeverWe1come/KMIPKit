# Tasks: KMIP 2.1 Client Asynchronous Operations

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md)
**Prerequisites**: Direct human authorization for autonomous implementation; current release branch contains accepted KMIPKIT-0006/0007 contracts. Reviewer-owned checklists remain review evidence and their unchecked items are not marked complete.
**Testing**: Required by the feature specification. Tests precede production code, with distinct Red, Green, and Refactor commits.

**Implementation record**: The Spec Kit prerequisite check passed with
`SPECIFY_FEATURE_DIRECTORY=specs/009-asynchronous-operations`. At the time of
implementation, the reviewer-owned checklists still had 20 requirements, 16
protocol-review, and 8 readiness items unchecked. Autonomous continuation was
explicitly authorized implementation without further approval prompts. T001 records that authorization and does not claim that a human reviewed or approved the specification or reviewer-owned checklists; checklist files remain unchanged and their owners retain review responsibility.

The distinct TDD commits are `94864d4` (protocol Red), `191d489` (protocol
Green), `5a0220d` (client Red), `fc3c295` (client Green), and `b1a3986`
(Refactor).

**Local verification** (2026-10-06, WSL Ubuntu-26.04, Rust 1.94):

- `cargo +1.94.0 fmt --all --check`, workspace Clippy with `-D warnings`,
  `cargo +1.94.0 test --workspace --all-features`, and workspace rustdoc with
  `RUSTDOCFLAGS=-D warnings` passed. The final focused client/protocol fmt,
  Clippy, and all-feature test rerun also passed (168 client unit tests, 60
  protocol unit tests, integration tests, and doctests).
- `cargo +1.94.0 llvm-cov --workspace --all-features` passed. LLVM line
  summaries measured 4,417/4,607 workspace lines (95.88%), 2,554/2,646
  TTLV/protocol lines (96.52%), and 88/88 transport/FFI lines (100%). The five
  asynchronous protocol model files each exceed 95% whole-file line coverage.
- A single-platform changed-line diagnostic measured 693/711 executable or
  summary-only changed lines (97.47%), with zero summary-only residual on
  changed source files; `execute.rs` was 292/300 (97.33%), and each changed
  asynchronous protocol source was at least 95.71%. This is local evidence,
  not the required three-platform CI gate. The unmodified local parser also
  reports an LLVM function-region/file-segment mismatch at unchanged
  `kmipkit-ttlv/src/item.rs:14`; the diagnostic excluded only that
  non-changed-source mismatch.
- `scripts/tests` ran 156 tests (25 cargo-deny fixture tests skipped because
  `CARGO_DENY` was not configured). The normative catalog suite passed 169
  tests with 7 platform-dependent skips after correcting stale KMIPKIT-0007
  test references. `scripts/tests/Test-Wsl.ps1` passed all 8 tests.
- Immutable OASIS source validation, the 1,411-candidate source audit,
  catalog validation (4 sources, 1,411 clauses, 4,024 records), coverage
  report verification, generated TTLV tag/result checks, and the PowerShell
  dependency-policy check passed. The dependency-policy check verified
  cargo-deny 0.20.2 and unchanged root/fuzz lockfile hashes.

**Review-fix verification** (2026-10-06, native Windows worktree, Rust 1.94):
`cargo +1.94.0 fmt --all --check`, workspace Clippy with `-D warnings`,
`cargo +1.94.0 test --workspace --all-features`, and
`cargo +1.94.0 doc --workspace --all-features --no-deps` passed. The workspace
run included 174 client unit tests, 61 protocol unit tests, integration tests,
and doctests. `cargo llvm-cov -p kmipkit-protocol --all-features --summary-only`
reported 96.12% protocol line coverage and 95.40% for changed
`process.rs` (97.10% regions); its uncovered Missing Pending Correlation Value
and Missing Response Payload returns cannot be reached through the validated
public response-item view because generic message validation rejects those
shapes first. This local Windows run does not replace the implementation
branch's three-platform CI or three-platform coverage aggregation.

T035 remains open for supported-platform CI and the exact three-platform coverage aggregation. T001 is complete as an authorization record only; no reviewer-owned checklist approval is claimed. T037 is complete: independent QA re-review found no remaining Critical, Important, or P2 findings; the Codex Security diff scan `035c0478-7289-407f-bbb7-73aefd61e43d` covered `5c0cc5e4de73f8ba4d3583693a3d1a0c21dfa842..01dc2ecf6ac55fd39b95ad4e15ab4400149892af`, reported no findings, and had complete coverage. The initial Process Pending and zeroization-test findings were corrected with distinct Red/Green/Refactor commits. The scan does not replace the qualified human security review required before 1.0.

## Phase 0: Review and readiness gates

**Purpose**: Record implementation authorization, source treatment, and interface dispositions while preserving reviewer-owned checklist responsibility.

- [x] T001 Record the direct human authorization for autonomous implementation without further approval prompts; do not represent it as approval of `spec.md`, `plan.md`, or the reviewer-owned protocol checklist.
- [x] T002 [P] Confirm KMIPKIT-0006/0007 and ADR-0014 contracts at the active release branch; verify batch association, Pending correlation access, limits, no-retry, and delivery-state behavior.
- [x] T003 Record the exact accepted disposition for `KMIPKIT-DISC-039`; until resolved, retain generic Query responses and keep typed Table 286 interpretation excluded.
- [x] T004 Record a catalog workflow item for the missing Process Table 278 client requirement ID; do not edit generated catalog output in this feature.
- [x] T005 Confirm proposed generic Poll completion can preserve any original operation payload without casting it to the initial Discover Versions type.

**Gate**: T001–T005 complete before any test or production-code commit.

## Phase 1: Shared test fixtures and traceability

**Purpose**: Establish source-derived, red tests and reusable operation wire fixtures.

- [x] T006 Add pinned-source-derived TTLV fixture builders for asynchronous operations in `crates/kmipkit-protocol/tests/support/async_operation_fixtures.rs`; cite exact OASIS clause/table IDs in fixture docs (KMIPKIT-0009-FR-001–KMIPKIT-0009-FR-007).
- [x] T007 Add stable feature requirement IDs and exact OASIS source citations to protocol/client test headers and this feature's traceability table; do not edit generated catalog output (KMIPKIT-0009-FR-012).
- [x] T008 Add malformed/unknown-value test matrix for required fields, wrong types, repeated/unknown values, and configured decode limits in protocol test support; derived cases cite §§6.1.5, 6.1.38, 6.1.39, and 6.1.41, Tables 176–178, 276–280, and 285–287 (KMIPKIT-0009-FR-001–KMIPKIT-0009-FR-007).

## Phase 2: User Story 1 — Continue or inspect one pending operation (Priority: P1)

**Goal**: Caller explicitly submits Poll or Cancel with exact pending correlation bytes and receives a truthful one-exchange outcome.

**Independent Test**: Fake transport confirms one Poll/Cancel exchange, exact encoded correlation bytes, correct outcome, redaction, and no automatic resend.

### Red — tests first

- [x] T009 [US1] Add failing Poll payload field/order and exact binary correlation tests, including arbitrary-byte property cases within codec limits, in `crates/kmipkit-protocol/tests/unit/poll_tests.rs`; cite §§6.1.38/Table 276, 8.6/Table 399, 9.1/Table 400, and 9.19/Table 424 (KMIPKIT-0009-FR-001–KMIPKIT-0009-FR-003).
- [x] T010 [US1] Add failing Poll Pending/no-payload, borrowed-only correlation access, no ordinary unzeroized duplicate, KMIPKit-owned zeroization-on-drop, completed-success generic payload, and terminal-Failure status/reason-without-payload tests in `crates/kmipkit-client/tests/unit/poll_execution_tests.rs`; cite §6.1.38/Table 276 and §8.6/Table 399, prove one exchange, and prove no auto-poll on Pending (KMIPKIT-0009-FR-002, KMIPKIT-0009-FR-003, KMIPKIT-0009-FR-008, KMIPKIT-0009-FR-009, KMIPKIT-0009-FR-010).
- [x] T011 [US1] Add failing Cancel request, echoed correlation, known/unknown Cancellation Result tests in `crates/kmipkit-protocol/tests/unit/cancel_tests.rs`; cite §6.1.5/Tables 176–178 and §11.7 (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-004).
- [x] T012 [US1] Add failing Cancel fake-transport delivery, mismatch, Pending-response rejection even when the request permits async results, request/echo redaction and KMIPKit-owned temporary-copy zeroization, and no-retry tests in `crates/kmipkit-client/tests/unit/cancel_execution_tests.rs`; cite §6.1.5/Tables 176–178 (KMIPKIT-0009-FR-004, KMIPKIT-0009-FR-008, KMIPKIT-0009-FR-009). Follow-up QA tests observe the encoded request owner after Cancel success and exchange-error paths; they make no claim about the separate caller-owned capture buffer.

### Green — minimum implementation

- [x] T013 [US1] Implement Poll typed request/response views in `crates/kmipkit-protocol/src/poll.rs` and export them from `crates/kmipkit-protocol/src/lib.rs` (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-003).
- [x] T014 [US1] Implement Cancel typed request/response and forward-compatible Cancellation Result values in `crates/kmipkit-protocol/src/cancel.rs` and protocol exports (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-004).
- [x] T015 [US1] Extend pending outcome/follow-up encoding in `crates/kmipkit-client/src/execute.rs` or a focused asynchronous execution module; keep exact bytes owned and redacted (KMIPKIT-0009-FR-002, KMIPKIT-0009-FR-009).
- [x] T016 [US1] Implement one-exchange Poll/Cancel execution, generic Poll completion payload, validation, response association, and delivery-aware errors in `crates/kmipkit-client/src/` (KMIPKIT-0009-FR-003, KMIPKIT-0009-FR-008, KMIPKIT-0009-FR-010).

### Refactor

- [x] T017 [US1] Remove duplicated parsing/encoding without weakening operation-specific Pending rules; document public types and preserve unknown TTLV (KMIPKIT-0009-FR-003, KMIPKIT-0009-FR-004, KMIPKIT-0009-FR-010).
- [x] T018 [US1] Refactor shared correlation-byte test fixtures after Red coverage is established; preserve the arbitrary-byte properties from T009 and borrowed-accessor, no-duplicate, redaction, and zeroization assertions from T010/T012 (KMIPKIT-0009-FR-002, KMIPKIT-0009-FR-009).

## Phase 3: User Story 2 — Request server processing (Priority: P1)

**Goal**: Caller explicitly sends Process and gets a result distinct from the original Pending operation.

**Independent Test**: Verify required exact correlation, an empty payload on every non-Failure response (including Pending), no payload on Failure, response association, Batch Order implications documented, and one exchange only.

### Red — tests first

- [x] T019 [US2] Add failing Process field/order, empty response payload, and error/result tests in `crates/kmipkit-protocol/tests/unit/process_tests.rs`; cite §6.1.39/Tables 278–280 (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-005).
- [x] T020 [US2] Add failing Process execution tests in `crates/kmipkit-client/tests/unit/process_execution_tests.rs`, including caller-selected Pending handling, independent delivery state, and one exchange; cite §6.1.39/Tables 278–280 and §8.6/Table 399 (KMIPKIT-0009-FR-005, KMIPKIT-0009-FR-008, KMIPKIT-0009-FR-009).

### Green — minimum implementation

- [x] T021 [US2] Implement Process request and response models in `crates/kmipkit-protocol/src/process.rs` and export them (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-005).
- [x] T022 [US2] Add explicit Process execution to `crates/kmipkit-client/src/` without changing Poll/Cancel into wait helpers or asserting server-side effects (KMIPKIT-0009-FR-005, KMIPKIT-0009-FR-008).

### Refactor

- [x] T023 [US2] Verify that every non-Failure Process response payload is present and empty, including Pending, while Failure has no payload under §8.6/Table 399 and §6.1.39/Table 279; document Batch Order Option's default behavior without claiming other items are controlled (KMIPKIT-0009-FR-005, KMIPKIT-0009-FR-008).
- [x] T024 [US2] Add negative response tests for unexpected Process payload members on Success and Pending, unrequested response batch IDs, and unpermitted asynchronous outcomes; the Pending case is valid at the generic §8.6/Table 399 message boundary but rejected by the Process model under §6.1.39/Table 279, with one client exchange (KMIPKIT-0009-FR-005, KMIPKIT-0009-FR-010).

## Phase 4: User Story 3 — Query outstanding asynchronous requests (Priority: P2)

**Goal**: Typed optional filters and generic response access while DISC-039 is unresolved.

**Independent Test**: Encode absent/empty/repeated filters and round-trip opaque response subtrees losslessly.

### Red — tests first

- [x] T025 [US3] Add failing Query request filter field/order/repetition and correlation-sentinel redaction tests in `crates/kmipkit-protocol/tests/unit/query_async_requests_tests.rs`; cite §6.1.41/Table 285 and §7.1/Table 352 (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-006, KMIPKIT-0009-FR-009).
- [x] T026 [US3] Add failing generic Query response round-trip tests in `crates/kmipkit-protocol/tests/unit/query_async_response_tests.rs`; assert no typed Table 286 mapping or conformance claim while DISC-039 is open (KMIPKIT-0009-FR-007).
- [x] T027 [US3] Add failing Query one-exchange/response-association, caller-input non-retention, redaction, no ordinary unzeroized duplicate, and KMIPKit-owned filter-copy zeroization tests on success/error paths in `crates/kmipkit-client/tests/unit/query_async_execution_tests.rs`; cite §6.1.41/Tables 285–287 and §8.6/Table 399, without asserting a typed Table 286 schema (KMIPKIT-0009-FR-006–KMIPKIT-0009-FR-010). Follow-up tests use the existing observer specifically for the encoded request owner on success and exchange error; the Query model-drop AST test and separate caller-owned-buffer assertion cover the input lifetime boundary.

### Green — minimum implementation

- [x] T028 [US3] Implement typed Query Asynchronous Requests filters in `crates/kmipkit-protocol/src/query_async_requests.rs` and export the model (KMIPKIT-0009-FR-001, KMIPKIT-0009-FR-006).
- [x] T029 [US3] Add Query response generic TTLV exposure and execution in `crates/kmipkit-client/src/` without inferring the disputed typed schema (KMIPKIT-0009-FR-007, KMIPKIT-0009-FR-008, KMIPKIT-0009-FR-010).

### Refactor

- [x] T030 [US3] Refactor Query filter and response fixtures after Red coverage is established; preserve lossless unknown/repeated-value and codec-limit assertions from T025/T026 and caller-input lifetime/redaction/zeroization assertions from T027 (KMIPKIT-0009-FR-006, KMIPKIT-0009-FR-007, KMIPKIT-0009-FR-009).
- [x] T031 [US3] Update research/traceability only if reviewed evidence changes; a typed response mapping requires an approved scope change and resolved discrepancy record (KMIPKIT-0009-FR-007, KMIPKIT-0009-FR-012).

## Phase 5: Cross-cutting quality and review

- [x] T032 [P] Add Rustdoc examples only after public names are final, in `crates/kmipkit-protocol/src/` and `crates/kmipkit-client/src/`; compile examples with workspace docs.
- [x] T033 [P] Update English/Spanish user documentation and `docs/roadmap.md` with explicit one-shot behavior, generic-result limits, server policy caveat, and DISC-039 status (KMIPKIT-0009-FR-008, KMIPKIT-0009-FR-011).
- [x] T034 Complete the requirement-to-code-to-test trace matrix, including explicit source/catalog gaps; verify 100% traceability for all claims made by this feature (KMIPKIT-0009-FR-012).
- [ ] T035 Run `cargo fmt --all --check`, workspace Clippy with `-D warnings`, workspace tests/docs, targeted protocol/client coverage, workspace coverage gates, immutable OASIS/catalog generation checks, dependency/security checks, and supported-platform CI.
- [x] T036 Run Spec Kit convergence; add tasks for any remaining gaps and repeat implementation/convergence until no gaps remain.
- [x] T037 Obtain independent QA and security review and record findings/corrections before creating a draft implementation PR; final QA and Codex Security review evidence is recorded above.

## Dependencies and execution order

- The direct human authorization recorded at T001 permits this implementation to proceed without further approval prompts. Scope changes and normative contradictions block only the affected behavior; reviewer-owned checklist items remain open until their owners review them.
- Phase 1 is shared setup and precedes operation work.
- User Stories 1–3 share the client request/response path and are intentionally sequential for one implementer. Protocol-only model tests can be isolated after shared tags/error conventions are agreed, but do not run parallel interface changes.
- Within each story, Red tests precede Green implementation, then Refactor and documentation. Keep Red, Green, and Refactor commits distinct and DCO signed.
- Phase 5 depends on all three user stories and the convergence review.

## Definition of completion for this feature

All unblocked scope, exact byte behavior, response-state semantics, error/delivery evidence, traceability, docs, tests, coverage, CI, QA, and security gates pass. If DISC-039 or the Process catalog ID remains unresolved, generic response access and typed Process behavior may be complete, but the associated typed Query mapping/catalog conformance gate is explicitly still open and release-wide traceability is not claimed closed.

## Phase 6: Convergence

- [x] T038 Update the Cancellation Result data model to list every assigned value and preserve extension values per KMIPKIT-0009-FR-004 (§11.7, Tables 437–438; partial).
