# Tasks: KMIP Shared Error Contract

**Input**: Approved design documents in specs/003-core-types-errors/

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/error-contract.md

**Organization**: Tasks follow the three independently testable user stories. Foundation work remains sequential under the approved one-agent foundation rule.

## Format: [ID] [P?] [Story] Description

- [P] marks a task safe to run in parallel. None are marked parallel because this is shared foundation work.
- [Story] maps a user-story task to spec.md.

## Phase 1: Setup and Prerequisites

**Purpose**: Confirm reviewed source material before implementation.

- [x] T001 Rebase feature/KMIPKIT-0003-core-types-errors onto active release/1.0.0 after KMIPKIT-0002 is merged; confirm specification/catalog/kmip-2.1.json and specification/catalog/coverage-report.md include Result Status and Result Reason before implementation begins. Evidence: the merged catalog contains the Result Status and Result Reason enumeration records at Specification §§11.47 and 11.46; the generated report includes both stable enumeration IDs and their child value IDs/source sections.

## Phase 2: Foundational

**Purpose**: Establish normative traceability before implementation.

- [x] T002 Create specification/compliance/requirements/KMIPKIT-0003.csv using the merged inventory record schema, with stable IDs, exact OASIS clauses, normative level, scope, implementation locations, test IDs, and status.

## Phase 3: User Story 1 - Inspect a KMIP Operation Result (Priority: P1)

**Goal**: Preserve and inspect known and unknown KMIP result values and message presence.

**Independent Test**: Run protocol result contract tests without a server and verify generated known values, unknown-value retention, optional message distinction and exact text, and Failure/Success reason validation.

- [x] T003 [US1] Add external protocol contract tests in crates/kmipkit-protocol/tests/result_contract.rs for unknown numeric values, absent/present-empty/present-nonempty Result Message (assert exact UTF-8 text bytes including arbitrary server text), Failure/Success reason invariants, safe ResultMessage Debug output, KmipOperationResult Display/Debug redaction, and ProtocolError cause sanitization/source-chain redaction. Use a drop-probe source and assert it is destroyed during construction; commit as Red and record the expected missing-behavior failure. Red evidence: `cargo test -p kmipkit-protocol --test result_contract` failed at the unresolved public imports, as expected before implementation.
- [x] T004 [US1] Implement ResultStatus, ResultReason, optional ResultMessage, KmipOperationResult, and safe ProtocolError types in crates/kmipkit-protocol/src/result.rs and crates/kmipkit-protocol/src/error.rs using generated catalog values; consume and drop arbitrary source errors during construction, implement redacted ResultMessage Debug and KmipOperationResult Display/Debug, export them from crates/kmipkit-protocol/src/lib.rs, and commit the minimal passing behavior as Green. Green evidence: the generated result mapping check passed and `cargo test -p kmipkit-protocol --test result_contract` passed (11 tests).
- [x] T005 [US1] Confirm all generated Result Status and Result Reason assignments are covered in crates/kmipkit-protocol/tests/result_contract.rs and link each case to KMIPKIT-0003-NR-001 and NR-002. The two catalog-driven tests enumerate every assigned value and verify name/raw-value round trips; NR-001 and NR-002 reference those tests in the compliance CSV.

## Phase 4: User Story 2 - Assess Request Delivery after Failure (Priority: P1)

**Goal**: Report the strongest send/receive evidence for local request-handling failures.

**Independent Test**: Run delivery-state tests through fake transport boundaries and verify each state without network access.

- [x] T006 [US2] Add external delivery tests in crates/kmipkit-transport/tests/delivery_state.rs proving a zero-byte read remains PossiblySent and receipt of the first response byte advances to ResponseStarted, as well as NotSent/write-started boundaries. Add TransportError cause sanitization/source-chain tests with a drop probe that proves the original source is destroyed; commit as Red before production changes. Red evidence: `cargo test -p kmipkit-transport --test delivery_state` failed on unresolved public imports before the transport contract existed.
- [x] T007 [US2] Implement RequestDeliveryState and a safe TransportError in crates/kmipkit-transport/src/error.rs; consume and drop arbitrary source errors during construction, expose the state at the crate boundary, and make T006 pass by committing the minimal behavior as Green. Green evidence: `cargo test -p kmipkit-transport --all-features` passed (5 integration tests); targeted Clippy passed.

## Phase 5: User Story 3 - Diagnose Failures without Disclosing Secrets (Priority: P1)

**Goal**: Preserve safe error categories while default reporting stays redacted.

**Independent Test**: Construct each local failure category with a synthetic source containing a secret sentinel; verify the safe category remains available and the sentinel is absent from Display, Debug, and the full public source chain.

- [ ] T008 [US3] Add client error tests in crates/kmipkit-client/tests/error_contract.rs for local category distinction, safe cause-category retention, unsafe source sanitization, and safe Display/Debug. Use distinct source sentinels for credentials, private keys, secret key material, one-time passwords, tickets, and raw KMIP body bytes; walk every `source()` link and assert each sentinel is unreachable, and use a drop probe to prove the original local source is not retained privately. Construct a complete server-result client error containing a Result Message sentinel and assert redaction from its Display/Debug before T009; commit as Red.
- [ ] T009 [US3] Implement the composite client error in crates/kmipkit-client/src/error.rs, retaining safe cause categories and transport delivery state while consuming and dropping arbitrary source errors before retention; make T008 pass and commit Green.
- [ ] T010 [US3] Add a public API integration test in crates/kmipkit/tests/error_api.rs that imports the intended result and error types from the facade and verifies server-result fidelity, delivery mapping, and no delivery state for a complete KMIP response; commit as Red and record the expected missing-export failure.
- [ ] T011 [US3] Re-export the supported result and error types from crates/kmipkit/src/lib.rs; make T010 pass and commit Green.

## Phase 6: Polish and Cross-Cutting Concerns

**Purpose**: Refactor, audit disclosure paths, document, and verify all stories.

- [ ] T012 Audit the affected protocol, transport, client, and facade result/error paths, including complete server-result handling, for logger dependencies and logging call sites (`log`, `tracing`, stdout/stderr); record findings in the PR. If no logger or call sites exist, verify this feature adds none. If any do exist, add captured-output sentinels for credentials, private keys, secret key material, one-time passwords, tickets, Result Message, causes, and raw bodies and prove they are omitted.
- [ ] T013 Refactor crates/kmipkit-protocol/src/result.rs, crates/kmipkit-protocol/src/error.rs, crates/kmipkit-transport/src/error.rs, and crates/kmipkit-client/src/error.rs for clear ownership and public documentation; keep all tests passing and commit separately as Refactor.
- [ ] T014 Complete specification-to-code/test mappings in specification/compliance/requirements/KMIPKIT-0003.csv and update docs/architecture/public-api.md and docs/development/testing.md.
- [ ] T015 Run cargo fmt --all --check, cargo clippy --workspace --all-targets --all-features -- -D warnings, cargo test --workspace --all-features, and cargo doc --workspace --all-features --no-deps from the Cargo.toml workspace root; record exact results.
- [ ] T016 Run cargo llvm-cov for protocol, transport, client, and workspace thresholds; verify changed code at least 95 percent, protocol 95 percent, transport 85 percent, workspace 90 percent, 100 percent applicable normative traceability, current generated files, and safe error formatting from the Cargo.toml workspace root.

## Dependencies and Execution Order

- T001 blocks implementation until KMIPKIT-0002 is merged and reviewed generated catalog data is present.
- T002 follows the normative record schema established by the inventory feature.
- T003 precedes T004; T006 precedes T007; T008 precedes T009; T010 precedes T011.
- User Story 2 follows the result-value foundation; User Story 3 follows both result and delivery contracts.
- T012-T016 depend on all story behavior. T013 is the distinct Refactor commit after Red/Green evidence.
- Foundation work is sequential; no parallel opportunities are identified.

## Requirement Coverage

- KMIPKIT-0003-NR-001: T003-T005, T014, T016.
- KMIPKIT-0003-NR-002: T003-T005, T014, T016.
- KMIPKIT-0003-NR-003: T003-T005, T010, T014, T016.
- KMIPKIT-0003-FR-001: T008-T011, T014-T016.
- KMIPKIT-0003-FR-002: T003-T005, T014, T016.
- KMIPKIT-0003-FR-003: T003-T005, T010, T014-T016.
- KMIPKIT-0003-FR-004: T006-T007, T010-T011, T014-T016.
- KMIPKIT-0003-FR-005: T008-T009, T012-T016.
- KMIPKIT-0003-FR-006: T003, T008-T009, T012-T016.
- KMIPKIT-0003-FR-007: T003-T005, T010-T011, T014-T016.
- KMIPKIT-0003-FR-008: T003-T005, T014, T016.
- SC-001: T008-T011, T014-T016.
- SC-002: T003-T005, T014, T016.
- SC-003: T006-T007, T010-T011, T014-T016.
- SC-004: T003, T008-T009, T012-T016.
- SC-005: T002, T005, T014, T016.

## MVP

Complete User Story 1 first: preserve KMIP result values and enforce the Success/Failure reason invariant. Continue with delivery certainty and safe client error reporting before declaring this feature complete.

## Notes

- Every story follows strict Red then Green; Refactor is a separate commit after all story tests pass.
- Do not begin implementation before T001 is satisfied.
- This foundation feature stays single-agent and does not modify FFI or language bindings.
