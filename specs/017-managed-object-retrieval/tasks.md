---
description: "Implementation tasks for KMIPKIT-0017 Get and Locate"
---

# Tasks: KMIP 2.1 Get and Locate

**Input**: Approved specification and design documents in specs/017-managed-object-retrieval/<br>
**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/public-rust.md, quickstart.md, traceability.md, and an approved/merged specification PR.<br>
**Testing**: Strict Red, Green, Refactor TDD. Record Red, Green, and any justified Refactor in distinct commits. Do not manufacture a failing test or refactor when no behavior change is warranted.

## Implementation gate

The revised KMIPKIT-0017 specification and catalog-disposition changes are merged into the active release/1.0.0 branch through PRs #63 and #79; the release-contract refresh merged in PR #81 at `fd6b7782`. T001 records the implementation worktree baseline and verifies the shared client contracts before T002 and later implementation tasks. KMIPKIT-DEC-003 through -005 resolve the prior catalog actor and PKCS#12 questions; executable evidence is still required before making conformance claims. The refreshed design records that PR #78 extended shared dispatch/typed outcomes and added callback-scoped full response TTLV access, while leaving the current validated direct-item `AttributeSet` contract unchanged.

## Phase 1: Setup

**Purpose**: Start from the approved release and establish deterministic operation test fixtures.

- [x] T001 Create the implementation worktree from active release/1.0.0 and record its commit; verify the client extension points for additive Get/Locate dispatch and typed-outcome integration, callback lifetime and owned-copy behavior for response TTLV, and the current validated direct-item `AttributeSet` contract against specs/017-managed-object-retrieval/traceability.md
- [x] T002 Add deterministic generic TTLV test builders for Get and Locate request/response payloads in crates/kmipkit-protocol/tests/support/
- [x] T003 Add fake-transport helpers that capture one request batch and return controlled response batches in crates/kmipkit-client/tests/support/

## Phase 2: Foundational ownership contract

**Purpose**: Establish the fallible, zeroizing TTLV item-copy primitive required to retain Get's callback-scoped Any Object. Get/Locate client dispatch and typed outcomes are added with their operation models and decoders in T017 and T027.

- [x] T004 Add failing public-surface tests for `Item::try_clone` deep-copy/roundtrip behavior plus Get/Locate request exports and model accessors in crates/kmipkit-ttlv/tests/public_item_contract.rs and crates/kmipkit-protocol/tests/public_operation_contract.rs
- [x] T005 Add a failing client-boundary integration contract for request variants, operation identifiers, completed-only outcome accessors, unified `ClientResponseView` accessors, and valid Pending response mapping in crates/kmipkit-client/tests/operation_boundary.rs. Its compile-time Red remains until the Get/Locate models and client mappings are implemented in T014–T017 and T024–T027; execute its runtime scenario in T028.
- [x] T006 Record focused Red commands/results and commit the contract tests before production edits in the KMIPKIT-0017 implementation branch
- [x] T007 Implement the fallible zeroizing `Item::try_clone` in crates/kmipkit-ttlv/src/item.rs
- [x] T008 Run `public_item_contract` and focused TTLV tests, record Green evidence, and commit the `Item::try_clone` Green implementation separately from the Red tests in the KMIPKIT-0017 implementation branch. The combined client-boundary test remains deferred to T028 because its typed response models are introduced in the operation stories.
- [x] T009 Review `Item::try_clone` for a justified behavior-preserving Refactor; if warranted, test and commit it separately in crates/kmipkit-ttlv/src/item.rs

## Phase 3: User Story 1 — Retrieve one managed object (Priority: P1)

**Goal**: Encode caller-selected Get fields and expose a successful object as opaque generic TTLV.<br>
**Independent test**: Table 220/221 vectors validate field presence, order, required response fields, and Any Object preservation; fake transport validates one exchange, result/error preservation, and redacted failures.

### Tests for User Story 1

- [x] T010 [US1] Add failing Table 220 tests for each optional field, omission, explicit Enumeration values, and request field order in crates/kmipkit-protocol/tests/unit/get_tests.rs
- [x] T011 [US1] Add failing Table 221/222 tests for success/error outcomes, required-field cardinality and types, applicable Result Reasons, unknown values, and nested Any Object preservation including repeated fields in crates/kmipkit-protocol/tests/unit/get_tests.rs
- [x] T012 [US1] Add failing fake-transport tests for one exchange, Pending/result preservation, delivery state, and no automatic retry; include a Get with omitted Unique Identifier after a preceding batch operation
  establishes server-side ID Placeholder, and assert batch order and no client-side identifier synthesis in
  crates/kmipkit-client/tests/unit/object_read_execution_tests.rs
- [x] T013 Record Red commands/results and commit Get tests before production changes in the KMIPKIT-0017 implementation branch

T013 Red evidence: T010 commit `1badf7dd` adds the Table 220 request cases; `cargo test -p kmipkit-protocol --lib get_tests` failed only on absent Get request/selector APIs. T011 commits `6c689908` and `a3c148ee` add Table 221/222 response cases and remove payload formatting from assertion diagnostics; the same focused command failed only on absent Get response/request APIs. T012 commit `099d3882` adds client fake-transport cases; `cargo test -p kmipkit-client --lib object_read_execution_tests` failed with 15 missing Get request/mapping API diagnostics. In each Red, `cargo fmt --all --check` and `git diff --check` passed. All three Red changes were reviewed independently; T011's P2 diagnostic finding was fixed and re-reviewed before the branch was pushed. No production changes were made in T010–T013.

Red contract correction: commit `6236b76d` changes the T012 success assertion to compare `GetResponse::object_type()` with `Some(ObjectType::from_raw(2))`, matching the approved public Rust contract and protocol response tests. Re-running `cargo test -p kmipkit-client --lib object_read_execution_tests` still fails only because the Get request and client mapping APIs are absent (15 missing-API diagnostics); `cargo fmt --all --check` and `git diff --check` pass.

Red fixture correction: commit `377b66e9` binds each `StructureView` before borrowing its children and uses the concrete `GetError` in the Get test helper return types; all behavior assertions are retained. With the production draft stashed, `cargo test -p kmipkit-protocol --lib get_tests` fails only on missing Get error/request/response/selector exports, and `cargo test -p kmipkit-client --lib object_read_execution_tests` fails only on 15 missing Get request/operation/mapping APIs. `cargo fmt --all` formats the corrected tests; `cargo fmt --all --check` and `git diff --check` are recorded after this evidence update.

Unknown-status fixture correction: commit `675eb5f4` gives the unknown-status response a valid Table 221 payload required by shared `ResponseMessage` validation, while retaining assertions that the raw unknown status/reason are preserved and success-only accessors remain empty. With the production draft stashed, `cargo test -p kmipkit-protocol --lib get_tests` exits 101 with only E0432 for missing Get error/request/response/selector exports from `get_tests.rs`; no fixture or implementation diagnostic remains. Formatting and diff checks are recorded after this evidence update.

Typed identifier assertion correction: commit `f392a60e` updates the two client assertions to compare the existing `UniqueIdentifier::TextString` representation specified by data-model.md and exercised across all three wire forms by `get_tests.rs`. With the production draft stashed, `cargo test -p kmipkit-client --lib object_read_execution_tests` exits 101 with only the expected 15 missing Get request/operation/outcome/response-view API diagnostics; the type-mismatch diagnostics are gone. Formatting and diff checks are recorded after this evidence update.

### Implementation for User Story 1

- [x] T014 [US1] Implement raw-preserving Get selectors, builder/accessors, and ordered Table 220 payload conversion in crates/kmipkit-protocol/src/get.rs. Green commit: `182533ff`.
- [x] T015 [US1] Implement typed Table 221 conversion with sanitized missing/duplicate/mistyped-field errors and owned, redacted, zeroizing Any Object storage copied with `Item::try_clone` from the callback-scoped TTLV view in crates/kmipkit-protocol/src/get.rs. Green commit: `182533ff`.
- [x] T016 [US1] Export Get request/response/error types and public rustdoc from crates/kmipkit-protocol/src/lib.rs. Green commit: `182533ff`.
- [x] T017 [US1] Add Get request serialization, completed and Pending response decoding, typed outcome storage, direct completed accessor, and unified response-view mapping in crates/kmipkit-client/src/execute.rs. Green commit: `182533ff`.
- [x] T018 Run focused Get protocol/client tests, record Green evidence, and commit the Green implementation separately in the KMIPKIT-0017 implementation branch. **Green evidence (2026-10-11):** DCO Green commit `182533ff` contains the Get production implementation; `cargo test -p kmipkit-protocol --lib get_tests` passed 17/17 and `cargo test -p kmipkit-client --lib object_read_execution_tests` passed 5/5. `cargo fmt --all --check` and `git diff --check` passed. Library lint commands `cargo clippy -p kmipkit-protocol --lib --all-features -- -D warnings` and `cargo clippy -p kmipkit-client --lib --all-features -- -D warnings` passed; the test-only lint cleanup is DCO commit `e39d4022`. The combined `operation_boundary` target remains deferred to T028 because its contract imports Locate APIs not implemented until T024–T027. An additional all-target package Clippy attempt reached the existing Locate public-surface Red test and failed on missing Locate exports and its temporary-view lifetime; Locate is outside this Get increment. T019 review disposition: no separate Refactor commit is justified by Get behavior tests; the small Get dispatch helper extraction is already part of T017 to satisfy the existing 100-line Clippy limit. The independent QA review approved Get with no actionable findings. Codex Security diff scan `ec54d7a1-619d-48d7-a37d-eb301fdd0e88` completed with no candidates across the five changed source files; this does not substitute for the qualified human review required before release.
- [x] T019 Perform a behavior-preserving Get Refactor only where test evidence justifies it; run tests and commit any change separately in crates/kmipkit-protocol/src/get.rs and crates/kmipkit-client/src/execute.rs. **Disposition (2026-10-11):** reviewed the Green implementation and focused tests; no additional behavior-preserving refactor is justified. No separate code commit was manufactured.

## Phase 4: User Story 2 — Find managed objects (Priority: P1)

**Goal**: Send ordered server-side search criteria and expose optional Located Items plus repeated identifiers without local filtering.<br>
**Independent test**: Table 247/248 vectors validate exact request order, empty required Attributes, optional values, and zero-or-more response IDs; fake transport validates batch and result order.

### Tests for User Story 2

- [ ] T020 [US2] Add failing Table 247 tests for empty/nonempty required Attributes, each optional field, explicit zero, Online/Archival/Destroyed mask bits 0x1/0x2/0x4, unknown mask bit 0x8000_0000 round-trip, and exact Maximum Items/Offset Items/Storage Status Mask/Object Group Member/Attributes order in crates/kmipkit-protocol/tests/unit/locate_tests.rs; build populated criteria with `AttributeSet::try_new`/`try_push`, assert catalogued TTLV-type and Vendor Attribute shape/order rejection, and preserve valid partial structured criteria, repeated date values for ranges, Cryptographic Usage Mask, and Usage Limits without local matching
- [ ] T021 [US2] Add failing Table 248/249 tests for omitted Located Items, zero/one/repeated identifiers, duplicates, raw identifier forms, response order, and applicable Result Reasons in crates/kmipkit-protocol/tests/unit/locate_tests.rs
- [ ] T022 [US2] Add failing fake-transport tests for batch order, unchanged criteria, and no local ID Placeholder assumptions in crates/kmipkit-client/tests/unit/object_read_execution_tests.rs
- [ ] T023 Record Red commands/results and commit Locate tests before production changes in the KMIPKIT-0017 implementation branch

### Implementation for User Story 2

- [ ] T024 [US2] Implement Locate selectors, required Attributes Structure, builder/accessors, and ordered Table 247 payload conversion in crates/kmipkit-protocol/src/locate.rs
- [ ] T025 [US2] Implement Table 248 conversion for optional Integer and zero-or-more ordered Unique Identifiers in crates/kmipkit-protocol/src/locate.rs
- [ ] T026 [US2] Export Locate request/response/error types and public rustdoc from crates/kmipkit-protocol/src/lib.rs
- [ ] T027 [US2] Add Locate request serialization, completed and Pending response decoding, typed outcome storage, direct completed accessor, and unified response-view mapping in crates/kmipkit-client/src/execute.rs
- [ ] T028 Run focused Locate protocol/client tests and the combined `operation_boundary` integration contract after both Get and Locate client mappings exist; record Green evidence and commit the Locate/client Green implementation separately in the KMIPKIT-0017 implementation branch
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
- Setup T001–T003 precedes contract Red tests T004–T006. T007–T009 establish only the TTLV ownership primitive; T017 adds Get client dispatch with Get models, and T027 extends dispatch with Locate models. T005's runtime scenario therefore runs at T028, after both operation paths exist; do not add temporary public models to make it pass earlier.
- US1 T010–T019 and US2 T020–T029 depend on the shared foundation. Their protocol files are separate, but shared dispatch and public exports overlap; implement sequentially with one active implementer. No tasks are marked parallel.
- US3 T030–T036 follows both request/response paths.
- Polish T037–T046 follows all stories.
- Only a human approves or merges the implementation PR.

## Implementation strategy

Deliver Get and Locate in the same Rust implementation increment, sequentially by story. Complete secret/malformed/result guarantees before opening the implementation PR. This specification does not claim C, Java, or Python parity, two-server interoperability, profile support, or certification; those are independent 1.0 gates.

## Parallel execution

No implementation tasks are marked parallel. Each operation has its own protocol module, but both touch client dispatch, public exports, result handling, catalog references, and shared verification. Keep ownership sequential.
