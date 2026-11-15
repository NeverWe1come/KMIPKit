# Tasks: KMIP 2.1 Query and Ping

**Input**: Design documents in `specs/020-query-ping/`

**Prerequisites**: Approved `spec.md`, `plan.md`, `data-model.md`, and public Rust contract.

**Execution rule**: These are implementation tasks for a later approved feature change. Do not start them from the spec-preparation PR. Commit tests, production code, and justified refactoring separately as Red, Green, and Refactor evidence.

## Phase 1: Shared verification setup

**Purpose**: Confirm the approved source contract and prepare source-derived fixtures before operation work.

- [x] T001 Confirm the active release base, Query/Ping OASIS references, both client requirement IDs, and assigned catalog elements in `specs/020-query-ping/traceability.md`.
- [x] T002 Add Query and Ping source-derived TTLV fixture helpers with independent tag/value expectations in `crates/kmipkit-protocol/tests/support/query_ping_fixtures.rs`.
- [x] T003 Add catalog implementation/test references for the approved Query/Ping scope, including the Object Groups member and Object Group attribute, in `specification/catalog/kmip-2.1.json` and regenerate `specification/catalog/coverage-report.md` with `tools/normative_catalog/report.py`.

## Phase 2: User Story 1 — Ping (Priority: P1)

**Goal**: Provide one explicit empty-payload Ping exchange with common result and delivery evidence.

**Independent Test**: Fake transport observes exactly one empty Ping request; empty Success returns success; KMIP/protocol/transport failure preserves status and delivery state without retry.

### Tests first (Red)

- [x] T004 [P] [US1] Add Ping request/response shape, empty payload, malformed message, and source-derived TTLV tests in `crates/kmipkit-protocol/tests/unit/ping_operation_tests.rs`.
- [x] T005 [P] [US1] Add fake-transport success, failure, delivery-state, one-exchange, and no-retry tests in `crates/kmipkit-client/tests/unit/ping_execution_tests.rs`.
- [x] T006 [US1] Run focused Ping tests before implementation and record the expected Red result in the Red commit.

### Implementation (Green)

- [x] T007 [P] [US1] Implement typed Ping request/response payload models and conversions in `crates/kmipkit-protocol/src/ping.rs`; export public types from `crates/kmipkit-protocol/src/lib.rs`.
- [x] T008 [US1] Add Ping request/response dispatch and `Client::ping` to `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`, using the shared execution path once.
- [x] T009 [US1] Run focused Ping protocol and fake-transport tests and record Green evidence.

### Refactor

- [x] T010 [US1] Review Ping conversion and dispatch for duplicated shared logic; refactor only where justified and rerun the focused suites, recording Refactor evidence.

## Phase 3: User Story 2 — Query (Priority: P1)

**Goal**: Let callers select one or more Query Functions and inspect the server response without loss or inferred capability claims.

**Independent Test**: Fake transport verifies required/repeated Query Function behavior, absent/single/repeated Object Group attributes, all Table 283 response members/cardinalities, both source-described successful response forms, failure, unknown values, and a single exchange.

### Tests first (Red)

- [x] T011 [P] [US2] Add Query request/response model tests for all 14 standard Query Function values, valid extension/future values, repeated function ordering, optional Object Groups with zero, one, and repeated Object Group Text String attributes, empty-function rejection, Table 283 cardinalities, unknown nested Items, empty payload and structured response forms, and an empty Protection Storage Masks list in `crates/kmipkit-protocol/tests/unit/query_operation_tests.rs`.
- [x] T012 [P] [US2] Add fake-transport tests for Query success/failure, all returned members, one exchange, no retry/follow-up, result and delivery preservation, and request/error redaction in `crates/kmipkit-client/tests/unit/query_execution_tests.rs`.
- [x] T013 [US2] Run focused Query tests before implementation and record the expected Red result in the Red commit.

### Implementation (Green)

- [x] T014 [US2] Implement open Query Function values and typed Query request/response models with Table 282–283 cardinality and generic nested Item preservation in `crates/kmipkit-protocol/src/query.rs`.
- [x] T015 [US2] Add Query request/response conversion, operation dispatch, and `Client::query` on the common exchange path in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`.
- [x] T016 [US2] Run focused Query protocol and fake-transport tests and record Green evidence.

### Refactor

- [x] T017 [US2] Review Query model/decoder helpers for invariant duplication and loss of optionality/order; make only justified refactors and rerun focused suites, recording Refactor evidence.

## Phase 4: Documentation and cross-cutting verification

- [x] T018 [P] Add executable Rust usage examples for Query and Ping in `crates/kmipkit-protocol/tests/query_ping_guide_examples.rs`.
- [x] T019 [P] Document Query/Ping request choices, server-reported capability limits, Ping semantics, errors, and no-retry behavior in `docs/user-guide/en/query-ping.md` and `docs/user-guide/es/operaciones-query-ping.md`.
- [x] T020 Complete `specs/020-query-ping/traceability.md` with final code and executable test references for each applicable requirement and response element.
- [x] T021 Add regression checks that Query/Ping Debug, Display, and error source chains do not expose raw KMIP bodies in `crates/kmipkit-client/tests/unit/query_ping_redaction_tests.rs`.
- [x] T022 Run catalog validation, generated-report check, immutable OASIS source check, formatting, Clippy, focused and workspace tests, coverage, documentation tests, and the full required CI matrix; meet the repository gates (95% protocol/model and changed code, 85% transport/FFI/bindings, 90% workspace) and record platform and coverage results in the implementation PR.
  - CI run 37984606997 passed on Ubuntu, Windows, and macOS, including Rust stable and 1.94, C consumer, Java/JNI, Python, sanitizers, fuzz smoke, normative inventory, documentation contracts, and coverage aggregation. Aggregate coverage: changed Rust 556/571 (97.37%), TTLV 724/730 (99.18%), protocol 6135/6353 (96.57%), transport 3713/3902 (95.16%), FFI 2035/2137 (95.23%), Java 618/670 (92.24%), Python 773/783 (98.72%), JNI 1193/1312 (90.93%), workspace 16807/17476 (96.17%).

## Phase 5: QA corrections

### Tests first (Red)

- [x] T023 Add a decoder regression test proving `Server Information` rejects a non-Structure TTLV value, citing KMIP v2.1 §6.1.40, Table 283 and §7.37, Table 389.
- [x] T024 Add fake-transport regressions proving Ping and Query retain typed Pending outcomes and exact correlation bytes when the typed batch's Asynchronous Indicator permits Pending, with one exchange and no automatic follow-up; cite KMIP v2.1 §§9.1–9.2, §11.3, Tables 400–401 and 431–432 and KMIPKIT-0007 FR-008.
  - Red evidence: commit `2dec86c`; the Query decoder test failed because Text String was accepted for Server Information, and both Pending tests failed because dispatch returned completed Ping/Query variants.

### Implementation (Green)

- [x] T025 Validate Server Information's outer TTLV value as a Structure before preserving its generic Item.
- [x] T026 Route typed Ping and Query response conversion through the common Pending outcome helper, preserving operation, typed response, and the exact correlation value in zeroizing storage.
  - Green evidence: the focused protocol regression passed (1/1); the focused client Pending regressions passed (2/2) after the implementation change.

### Refactor and traceability

- [x] T027 Reconcile the Query/Ping design and acceptance criteria with the inherited KMIPKIT-0007 asynchronous batch contract; map the typed Pending paths and malformed Server Information test in traceability. Review the shared helper reuse and make no further production refactor because the minimal green change already follows the common path.
  - Refactor evidence: documentation and traceability align with the implemented shared path; focused suites remain green. No generated catalog artifact changed.

## Dependencies and execution order

- T001–T003 establish the source and verification baseline before either story.
- US1 and US2 are independent after setup and may be implemented in separate test/code commits or serially in one branch.
- Each story's Red tests must precede its production changes; its Green tests must pass before a justified Refactor review.
- T020–T022 follow both stories. No C, Java, Python, JSON/XML, server-initiated operation, or transport implementation belongs to this specification.
