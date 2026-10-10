---
description: "Implementation tasks for KMIPKIT-0017 Get and Locate"
---

# Tasks: KMIP 2.1 Get and Locate

**Input**: Approved specification and design documents in specs/017-managed-object-retrieval/<br>
**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/public-rust.md, quickstart.md, traceability.md, and an approved/merged specification PR.<br>
**Testing**: Strict Red, Green, Refactor TDD. Record Red, Green, and any justified Refactor in distinct commits. Do not manufacture a failing test or refactor when no behavior change is warranted.

## Implementation gate

Do not execute implementation tasks until the revised KMIPKIT-0017 specification and catalog-disposition PR is approved and merged into the active release/1.0.0 branch. Create the implementation worktree from that release branch. KMIPKIT-DEC-003 through -005 resolve the prior catalog actor and PKCS#12 questions; still require executable evidence before making conformance claims. If the release has changed client dispatch, response ownership, or AttributeSet contracts, refresh this design and obtain review before writing implementation tests.

## Phase 1: Setup

**Purpose**: Start from the approved release and establish deterministic operation test fixtures.

- [ ] T001 Create the implementation worktree from active release/1.0.0 and verify approved release APIs against specs/017-managed-object-retrieval/traceability.md
- [ ] T002 Add deterministic generic TTLV test builders for Get and Locate request/response payloads in crates/kmipkit-protocol/tests/support/
- [ ] T003 Add fake-transport helpers that capture one request batch and return controlled response batches in crates/kmipkit-client/tests/support/

## Phase 2: Foundational contracts

**Purpose**: Establish closed typed dispatch and typed-outcome boundaries used by both stories.

- [ ] T004 Add failing public-surface tests for Get/Locate request exports and model accessors in crates/kmipkit-protocol/tests/public_operation_contract.rs
- [ ] T005 Add failing client-boundary tests for request variants, operation identifiers, and typed response accessors in crates/kmipkit-client/tests/operation_boundary.rs
- [ ] T006 Record focused Red commands/results and commit the contract tests before production edits in the KMIPKIT-0017 implementation branch
- [ ] T007 Implement only the shared Get/Locate request variants, operation IDs, outcome slots, and typed accessors in crates/kmipkit-client/src/execute.rs
- [ ] T008 Run the focused boundary tests, record Green evidence, and commit the shared client surface separately in the KMIPKIT-0017 implementation branch
- [ ] T009 Review shared dispatch for a justified behavior-preserving Refactor; if warranted, test and commit it separately in crates/kmipkit-client/src/execute.rs

## Phase 3: User Story 1 — Retrieve one managed object (Priority: P1)

**Goal**: Encode caller-selected Get fields and expose a successful object as opaque generic TTLV.<br>
**Independent test**: Table 220/221 vectors validate field presence, order, required response fields, and Any Object preservation; fake transport validates one exchange, result/error preservation, and redacted failures.

### Tests for User Story 1

- [ ] T010 [US1] Add failing Table 220 tests for each optional field, omission, explicit Enumeration values, and request field order in crates/kmipkit-protocol/tests/unit/get_tests.rs
- [ ] T011 [US1] Add failing Table 221/222 tests for success/error outcomes, required-field cardinality and types, applicable Result Reasons, unknown values, and nested Any Object preservation including repeated fields in crates/kmipkit-protocol/tests/unit/get_tests.rs
- [ ] T012 [US1] Add failing fake-transport tests for one exchange, Pending/result preservation, delivery state, and no automatic retry; include a Get with omitted Unique Identifier after a preceding batch operation
  establishes server-side ID Placeholder, and assert batch order and no client-side identifier synthesis in
  crates/kmipkit-client/tests/unit/object_read_execution_tests.rs
- [ ] T013 Record Red commands/results and commit Get tests before production changes in the KMIPKIT-0017 implementation branch

### Implementation for User Story 1

- [ ] T014 [US1] Implement raw-preserving Get selectors, builder/accessors, and ordered Table 220 payload conversion in crates/kmipkit-protocol/src/get.rs
- [ ] T015 [US1] Implement typed Table 221 conversion with sanitized missing/duplicate/mistyped-field errors and opaque Any Object ownership in crates/kmipkit-protocol/src/get.rs
- [ ] T016 [US1] Export Get request/response/error types and public rustdoc from crates/kmipkit-protocol/src/lib.rs
- [ ] T017 [US1] Add Get request serialization, response decoding, typed outcome storage, and matching accessor in crates/kmipkit-client/src/execute.rs
- [ ] T018 Run focused Get protocol/client tests, record Green evidence, and commit the Green implementation separately in the KMIPKIT-0017 implementation branch
- [ ] T019 Perform a behavior-preserving Get Refactor only where test evidence justifies it; run tests and commit any change separately in crates/kmipkit-protocol/src/get.rs and crates/kmipkit-client/src/execute.rs

## Phase 4: User Story 2 — Find managed objects (Priority: P1)

**Goal**: Send ordered server-side search criteria and expose optional Located Items plus repeated identifiers without local filtering.<br>
**Independent test**: Table 247/248 vectors validate exact request order, empty required Attributes, optional values, and zero-or-more response IDs; fake transport validates batch and result order.

### Tests for User Story 2

- [ ] T020 [US2] Add failing Table 247 tests for empty/nonempty required Attributes, each optional field, explicit zero, Online/Archival/Destroyed mask bits 0x1/0x2/0x4, unknown mask bit 0x8000_0000 round-trip, and exact Maximum Items/Offset Items/Storage Status Mask/Object Group Member/Attributes order in crates/kmipkit-protocol/tests/unit/locate_tests.rs; also preserve partial structured criteria, repeated date values for ranges, Cryptographic Usage Mask, and Usage Limits without local matching
- [ ] T021 [US2] Add failing Table 248/249 tests for omitted Located Items, zero/one/repeated identifiers, duplicates, raw identifier forms, response order, and applicable Result Reasons in crates/kmipkit-protocol/tests/unit/locate_tests.rs
- [ ] T022 [US2] Add failing fake-transport tests for batch order, unchanged criteria, and no local ID Placeholder assumptions in crates/kmipkit-client/tests/unit/object_read_execution_tests.rs
- [ ] T023 Record Red commands/results and commit Locate tests before production changes in the KMIPKIT-0017 implementation branch

### Implementation for User Story 2

- [ ] T024 [US2] Implement Locate selectors, required Attributes Structure, builder/accessors, and ordered Table 247 payload conversion in crates/kmipkit-protocol/src/locate.rs
- [ ] T025 [US2] Implement Table 248 conversion for optional Integer and zero-or-more ordered Unique Identifiers in crates/kmipkit-protocol/src/locate.rs
- [ ] T026 [US2] Export Locate request/response/error types and public rustdoc from crates/kmipkit-protocol/src/lib.rs
- [ ] T027 [US2] Add Locate request serialization, response decoding, typed outcome storage, and matching accessor in crates/kmipkit-client/src/execute.rs
- [ ] T028 Run focused Locate protocol/client tests, record Green evidence, and commit the Green implementation separately in the KMIPKIT-0017 implementation branch
- [ ] T029 Perform a behavior-preserving Locate Refactor only where test evidence justifies it; run tests and commit any change separately in crates/kmipkit-protocol/src/locate.rs and crates/kmipkit-client/src/execute.rs

## Phase 5: User Story 3 — Preserve results and protect object contents (Priority: P1)

**Goal**: Apply shared secret, malformed-input, future-value, operation-error, Pending, and delivery guarantees to both operations.<br>
**Independent test**: Sentinel values never appear in diagnostics; malformed/over-limit responses fail safely; unknown/repeated values stay lossless; server obligations are not misrepresented as client behavior.

- [ ] T030 [US3] Add failing redaction and KMIPKit-owned payload-zeroization tests for nested Get objects and wrapping fields in crates/kmipkit-protocol/tests/get_secret_handling.rs
- [ ] T031 [US3] Add failing malformed/duplicate/mistyped/truncated/over-limit vectors with sanitized-error assertions in crates/kmipkit-protocol/tests/object_read_negative_vectors.rs
- [ ] T032 [US3] Add negative client-behavior vectors proving omitted masks stay omitted, explicit standard and unknown mask bits round-trip unchanged, and response identifiers are not silently filtered in crates/kmipkit-client/tests/unit/object_read_execution_tests.rs
- [ ] T033 Record Red commands/results and commit US3 tests before any required production fix in the KMIPKIT-0017 implementation branch
- [ ] T034 Implement only security or protocol fixes demonstrated necessary by Red tests, reusing shared TTLV limits and zeroization in crates/kmipkit-protocol/src/get.rs, crates/kmipkit-protocol/src/locate.rs, or crates/kmipkit-client/src/execute.rs
- [ ] T035 Run focused security/negative tests, record Green evidence, and commit required fixes separately; if release behavior already satisfies a test, record a no-change disposition in specs/017-managed-object-retrieval/traceability.md
- [ ] T036 Perform only justified behavior-preserving security/ownership refactors and keep any Refactor evidence in a distinct commit in the KMIPKIT-0017 implementation branch

## Phase 6: Polish and cross-cutting traceability

- [ ] T037 Use the pinned TC-PKCS12-1-21 and TC-PKCS12-2-21 XML for in-scope Get vectors, and add derived local vectors for the five remaining unavailable cases in crates/kmipkit-protocol/tests/fixtures/; distinguish partial item coverage from full official-case passes
- [ ] T038 Compile and run the Rust examples in specs/017-managed-object-retrieval/quickstart.md from an integration test in crates/kmipkit-client/tests/
- [ ] T039 Write the English user guide with tested Get/Locate examples in docs/user-guide/en/object-retrieval.md
- [ ] T040 Write the Spanish user guide with equivalent examples and security notes in docs/user-guide/es/object-retrieval.md
- [ ] T041 Update implementation_refs and verification_refs for demonstrated client requirements in specification/catalog/kmip-2.1.json and regenerate specification/catalog/coverage-report.md using python -B tools/normative_catalog/report.py --write
- [ ] T042 Run catalog validation, immutable-source checks, and traceability suites using python -B tools/normative_catalog/validate.py --repo-root ., python -B tools/normative_catalog/check_immutable_sources.py --repo-root ., and python -B -m unittest discover -s tools/normative_catalog/tests -v
- [ ] T043 Run cargo fmt --all --check, cargo clippy --workspace --all-targets --all-features -- -D warnings, cargo test --workspace --all-features, and repository coverage checks; meet protocol/model 95%, changed code 95%, and workspace 90% gates without unjustified exclusions
- [ ] T044 Run supported-platform CI, resolve findings, and record the exact successful run in the KMIPKIT-0017 implementation PR
- [ ] T045 Converge implementation against every approved FR, SC, applicable client requirement, and checklist; update specs/017-managed-object-retrieval/tasks.md until no in-scope gap remains
- [ ] T046 Open or update a draft implementation PR describing scope, Red/Green/Refactor evidence, generated changes, security effects, coverage, CI, fixture gaps, and server-behavior limits

## Dependencies and execution order

- Specification approval and merge gate T001.
- Setup T001–T003 precedes shared client contracts T004–T009.
- US1 T010–T019 and US2 T020–T029 depend on the shared foundation. Their protocol files are separate, but shared dispatch and public exports overlap; implement sequentially with one active implementer. No tasks are marked parallel.
- US3 T030–T036 follows both request/response paths.
- Polish T037–T046 follows all stories.
- Only a human approves or merges the implementation PR.

## Implementation strategy

Deliver Get and Locate in the same Rust implementation increment, sequentially by story. Complete secret/malformed/result guarantees before opening the implementation PR. This specification does not claim C, Java, or Python parity, two-server interoperability, profile support, or certification; those are independent 1.0 gates.

## Parallel execution

No implementation tasks are marked parallel. Each operation has its own protocol module, but both touch client dispatch, public exports, result handling, catalog references, and shared verification. Keep ownership sequential.
