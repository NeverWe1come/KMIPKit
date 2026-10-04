# Tasks: KMIP 2.1 Message and Batch Model

**Input**: Design documents in `specs/006-message-batch-model/`.
**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/rust-message-model.md`.
**Scope**: Implement only after the approved specification and the KMIPKIT-0005 codec foundation gates below are satisfied. The current draft branch performs no protocol implementation.

## Phase 1: Preconditions and setup

**Purpose**: Confirm approved scope and stable dependency interfaces before Red tests.

- [ ] T001 Review/accept the KMIPKIT-0006 specification and its requirement-quality checklist; record exact approved revision in the implementation PR.
- [ ] T002 Confirm KMIPKIT-0005 correction/ADR-0011 and codec foundation are accepted and merged to `release/1.0.0`; rebase this feature branch on the current release head.
- [ ] T003 [P] Review existing `Structure`, `StructureView`, and `Item::with_value` APIs; confirm a public `Structure::view()` can safely support validation without payload copies (FR-017; SC-006).

## Phase 2: Foundational contracts

**Purpose**: Add only the view API that permits protocol validation without transferring payload ownership.

- [ ] T004 [RED COMMIT] Add failing API tests for scoped Structure views, child-order preservation, and inability to mutate or take payload ownership in `crates/kmipkit-ttlv/tests/structure_view.rs` (FR-017; SC-006).
- [ ] T005 [GREEN COMMIT] Add the smallest safe `Structure::view()` implementation and public documentation in `crates/kmipkit-ttlv/src/structure.rs` and `crates/kmipkit-ttlv/src/lib.rs` (FR-017; SC-006).
- [ ] T006 [REFACTOR COMMIT] Refine view API documentation and tests; retain `#![forbid(unsafe_code)}` and existing zeroization behavior (FR-017; SC-006).

**Checkpoint**: The read-only view API is merged on the feature branch before message model work.

## Phase 3: User Story 1 — Structurally valid messages (P1)

**Goal**: Validate request/response envelope fields and provide lossless typed views over the owned TTLV tree.

- [ ] T007 [RED COMMIT] Add external tests for Request/Response Message envelope order, header and batch-item field order, requiredness, option/default presence, and parse-time preservation of assigned, extension-range, and otherwise unassigned Asynchronous Indicator and Batch Error Continuation raw Enumeration values; signed-Integer count bounds, version representation, and Attestation Capable Indicator default in `crates/kmipkit-protocol/tests/message_contract.rs` (FR-001, FR-002, FR-003, FR-006, FR-007, FR-012, FR-018, FR-019, FR-022; SC-001, SC-002). Do not test outbound rejection here; `KMIPKIT-0007-client-execution` owns outbound acceptance and negative tests for unassigned option values outside Tables 432 and 435. Cite OASIS KMIP Specification v2.1 §§8.1–8.6, 9.2–9.8, 9.10, 9.16, and Tables 394–399, 432, and 435 in the test documentation.
- [ ] T008 [RED COMMIT] Add round-trip ownership tests for unknown allocated fields and raw Enumeration values (including unknown request-option values), Server Correlation Value preservation in generic TTLV, repeated/schema-authorized values, opaque subtrees, source order, and secret/result-message redaction in `crates/kmipkit-protocol/tests/message_conversion.rs` (FR-013, FR-014, FR-017, FR-022; SC-003, SC-004, SC-006). Cite OASIS KMIP Specification v2.1 §§9.2, 9.6, 9.10, and 10.1.2 and Tables 395–399, 432, and 435.
- [ ] T009 [GREEN COMMIT] Implement `RequestMessage`, `ResponseMessage`, and safe message errors in `crates/kmipkit-protocol/src/message/mod.rs` and `crates/kmipkit-protocol/src/message/validation.rs` (FR-001 through FR-003).
- [ ] T010 [GREEN COMMIT] Implement typed request/response header views and defaults in `crates/kmipkit-protocol/src/message/header.rs`; preserve raw request-option Enumeration values and reuse existing result types from `crates/kmipkit-protocol/src/result.rs`. Outbound option-value validation remains KMIPKIT-0007 work (FR-005 through FR-010, FR-012, FR-018, FR-019, FR-022).
- [ ] T011 [GREEN COMMIT] Implement request/response batch item views and lossless owned-tree conversion in `crates/kmipkit-protocol/src/message/batch.rs` and `crates/kmipkit-protocol/src/message/conversion.rs` (FR-001, FR-002, FR-004, FR-005, FR-008, FR-011, FR-013, FR-022).
- [ ] T012 [REFACTOR COMMIT] Split focused private helpers, document every public type/method, and add the compiled public usage example from `specs/006-message-batch-model/quickstart.md` as a doctest (FR-001, FR-017; SC-001, SC-006).

## Phase 4: User Story 2 — Preserve batch item identity (P1)

**Goal**: Preserve unique request item IDs and exact response item association through model conversion without runtime matching.

- [ ] T013 [RED COMMIT] Add tests for optional single-item IDs, required multi-item request IDs and KMIPKit project-invariant pairwise uniqueness, response ID preservation, property-based ID/order preservation, and payload-free failures in `crates/kmipkit-protocol/tests/message_validation.rs` (FR-004; SC-002, SC-003). Cite OASIS KMIP Specification v2.1 §§8.3, 8.6, 9.21 and Tables 396, 399.
- [ ] T014 [GREEN COMMIT] Implement Table 396 ID presence and request-local uniqueness validation plus raw-byte access in `crates/kmipkit-protocol/src/message/batch.rs` and `crates/kmipkit-protocol/src/message/validation.rs` (FR-004).
- [ ] T015 [REFACTOR COMMIT] Ensure the model never uses Client Correlation Value as a per-item key; document boundary to later client matching (FR-008).

## Phase 5: User Story 3 — Represent asynchronous results (P1)

**Goal**: Represent Pending response fields without scheduling Poll, Cancel, retry, or wait work.

- [ ] T016 [RED COMMIT] Add tests for omitted/explicit Asynchronous Indicator defaults, mixed Pending/completed results, unknown response enum preservation, missing Pending correlation, Result Message rejection for Success/Pending, request repeated versus response singleton Message Extension cardinality (including duplicate-response rejection), malformed Message Extension fields/types/order/vendor characters, and sentinel redaction in `crates/kmipkit-protocol/tests/message_validation.rs` (FR-009, FR-010, FR-011, FR-014, FR-020, FR-021; SC-004, SC-006). Cite OASIS KMIP Specification v2.1 §§8, 8.6, 9.1–9.2, 9.13, and Tables 396, 399, 418.
- [ ] T017 [GREEN COMMIT] Implement Pending correlation and Result Message status validation, raw-preserving indicator views, and Message Extension shape validation in `crates/kmipkit-protocol/src/message/header.rs`, `crates/kmipkit-protocol/src/message/batch.rs`, and `crates/kmipkit-protocol/src/message/validation.rs` (FR-009, FR-010, FR-011, FR-020, FR-021).
- [ ] T018 [REFACTOR COMMIT] Narrow errors and `Debug` output to safe metadata; keep Poll, Cancel, retry, and waiting outside the message model (FR-014, FR-015; SC-006).

## Phase 6: Traceability, documentation, and verification

**Purpose**: Close normative and project-policy traceability with generated evidence.

- [ ] T019 Add `specification/compliance/requirements/KMIPKIT-0006.csv` mapping each in-scope requirement/clause to implementation and executable tests (FR-016, SC-005); assign `KMIPKIT-REQ-SPEC-8-003-002` to mixed-response handling, mark `KMIPKIT-REQ-SPEC-9.8-001-002/-003` server-only, assign `KMIPKIT-REQ-SPEC-9.12-001-002/-003` and outbound option-value validation under Tables 432/435 to KMIPKIT-0007 with their boundary and acceptance/negative tests, and defer Poll/Cancel correlation use to KMIPKIT-0009. Assign `KMIPKIT-DISC-039`/§6.1.41 Query Asynchronous Requests response mapping to KMIPKIT-0009 to review the exact normative-source conflict and document and test a decision; do not assume a resolution in KMIPKIT-0006.
- [ ] T020 Update the reviewed source catalog through the audited catalog workflow when assigning element/requirement traceability; regenerate `specification/catalog/coverage-report.md` with the pinned report tool and never edit generated output manually (FR-016; SC-005).
- [ ] T021 Update `docs/architecture/public-api.md` and English/Spanish user documentation; keep implementation details out of user workflows (FR-002; SC-001).
- [ ] T022 Run the required format, Clippy, focused and workspace tests, doctests, catalog checks, source-immutability checks, and `cargo llvm-cov`; verify at least 95% line coverage for changed code, at least 95% for `kmipkit-ttlv` and protocol/model code, and at least 90% for the Rust workspace overall (SC-007). This feature adds no transport or FFI code.
- [ ] T023 Run security and QA review against the specification, data model, tasks, tests, and generated traceability; verify FR-015 scope exclusions and follow-on ownership, including the KMIPKIT-0007 deterministic fake-transport test for one response batch containing both completed and Pending items, accepting Pending when the request indicator permits asynchronous results and rejecting it otherwise for `KMIPKIT-REQ-SPEC-8-003-002`; resolve findings and record Red/Green/Refactor commit evidence.
- [ ] T024 Rebase onto current `release/1.0.0`, run supported Linux/Windows/macOS CI, verify generated output is clean, and open a draft PR using the Git terminal workflow.

## Commit discipline

TDD tasks tagged RED, GREEN, and REFACTOR MUST land in separate development commits in that order. Each commit message identifies its phase and feature ID. The draft PR body records the exact commands, expected/observed Red failures, Green results, Refactor results, coverage evidence, security/QA findings, and any unavailable platform evidence. Do not start these implementation tasks while T002 remains incomplete.
