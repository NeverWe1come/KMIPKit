---
description: "Implementation tasks for KMIP 2.1 Policy and Usage Controls"
---

# Tasks: KMIP 2.1 Policy and Usage Controls

**Input**: Design documents in `specs/025-policy-controls/`.

**Prerequisites**: Approved `spec.md`, `plan.md`, `research.md`, `data-model.md`, `traceability.md`, and `contracts/`.

**Tests**: Required by the approved acceptance criteria and `AGENTS.md`. Each operation and the usage guard have distinct Red, Green, and Refactor evidence commits. The listed official fixture inventory has no direct OASIS case for these operations; derive and label tests as project tests.

**Organization**: Tasks are grouped by four in-scope P1 user stories, the usage-allocation guard, and the Set Endpoint Role 1.0 scope resolution. Shared client dispatcher edits are integrated serially. Cross-language parity follows the approved KMIPKIT-0043 manifest contract.

## Format and path conventions

Each task uses `[ ] T### [US#] action in exact/repository/path`. Every implementation story has explicit Red, Green, and Refactor commits. Unit test modules live under `tests/unit/` and are registered through the existing crate test module declarations (`crates/kmipkit-protocol/src/lib.rs` or `crates/kmipkit-client/src/lib.rs`/`execute.rs`).

## Phase 1: Setup

**Purpose**: Confirm the approved source and release baseline. No project initialization or new runtime dependency is needed.

- [x] T001 Verify source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256`; compare the pinned §6.1.22, §6.1.23, §6.1.52, §6.1.53, §6.1.54, §4.59, §7.7, §7.11, §11.19, and §11.60 source passages with `specs/025-policy-controls/spec.md` and `traceability.md`; confirm the approved release baseline and record exact commands before coding. Evidence refreshed after rebasing: all four checksum entries returned OK; `python tools/normative_catalog/validate.py` returned `Catalog valid: sources=4 clauses=1411 records=4044`; `git merge-base HEAD origin/release/1.0.0` and `git rev-parse origin/release/1.0.0` both returned `4fba50ecbff0a6d2ab662bab4ba69031b80ba3c6`. OASIS §7.40 Table 392 specifies Usage Limits Count as Long Integer, so corrected the feature contracts and tasks to use `i64`/TTLV Long Integer; §11.19 Table 452 is the Endpoint Role Enumeration source.

## Phase 2: User Story 1 — Get Constraints (Priority: P1)

**Goal**: Send the empty §6.1.22 request and return its required Constraints structure and common result without losing extensions.

**Independent Test**: A fake transport sees exactly one Table 229 request; Table 230 success returns required Constraints, and Table 231 failure preserves result metadata and delivery.

### Red commit

- [x] T002 [US1] Add failing protocol and fake-client tests in `crates/kmipkit-protocol/tests/unit/get_constraints_tests.rs` and `crates/kmipkit-client/tests/unit/get_constraints_execution_tests.rs`; register modules in `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit-client/src/lib.rs`; cite OASIS KMIP Specification v2.1 §6.1.22, Tables 229–231 in test documentation; cover empty request, required ordered Constraints, unknown nested members, malformed response, Table 231 reasons, Pending/correlation, one exchange, NotSent validation, redaction, and no retry; record failing command/output in a separate Red commit. Red evidence: `cargo test -p kmipkit-protocol -p kmipkit-client --lib get_constraints` exited 1 with E0432/E0599 because `GetConstraintsRequest`/`GetConstraintsResponse`, `ClientRequest::get_constraints()`, `Client::get_constraints()`, `ClientOperation::GetConstraints`, `ClientBatchOutcome::GetConstraints`, and the response accessor are absent. The first run exposed an unrelated test fixture `Structure::clone()` typo; corrected it before recording this expected failure.

### Green commit

- [x] T003 [US1] Implement `GetConstraintsRequest`/`GetConstraintsResponse` in `crates/kmipkit-protocol/src/get_constraints.rs`, export from `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit/src/lib.rs`, and add the typed request/response path in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T002 and commit separately as Green. QA found unknown sibling payload fields were rejected contrary to FR-002/FR-003. Follow-up Red commit `c7e45aa1` recorded E0599 for absent `with_ttlv`; Green commit retains and exposes the complete response item and accepts unknown siblings while still requiring exactly one correctly typed `Constraints`. Verified `cargo test -p kmipkit-protocol -p kmipkit-client --lib get_constraints` (7 protocol, 6 client passed), focused Clippy with `-D warnings`, `cargo fmt --all --check`, and `git diff --check`. Original fixture repairs: ordered assertions replace unsupported `Structure` equality; absent Success payload is tested at the shared validator boundary.

### Refactor commit

- [x] T004 [US1] Refactor Get Constraints encoding, required-field validation, and focused assertions in `crates/kmipkit-protocol/src/get_constraints.rs`, `crates/kmipkit-protocol/tests/unit/get_constraints_tests.rs`, and `crates/kmipkit-client/tests/unit/get_constraints_execution_tests.rs` without changing wire behavior; rerun focused tests and commit separately as Refactor. Refactored cardinality validation to select exactly one Constraints member; lifetime test inspects preserved unknown siblings after dropping the source message. Verified `cargo test -p kmipkit-protocol -p kmipkit-client --lib get_constraints` (7 protocol, 6 client), focused Clippy with `-D warnings`, `cargo fmt --all --check`, and `git diff --check`. Independent QA approved HEAD `61576075` with no blockers and reproduced tests/Clippy.

**Checkpoint**: Empty request and required Constraints result satisfy Tables 229–231.

## Phase 3: User Story 2 — Get Usage Allocation and explicit guard (Priority: P1)

**Goal**: Implement §6.1.23 allocation exchange and the approved local caller-supplied guard for known Usage-Limited protection use.

**Independent Test**: The fake transport verifies Tables 232–234; guard tests prove object/unit/cost boundaries and delivery-state reservation behavior without claiming remote consumption.

### Allocation Red commit

- [x] T005 [US2] Add failing model/fake-client tests in `crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs` and `crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs`; register modules in `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit-client/src/lib.rs`; cite OASIS KMIP Specification v2.1 §6.1.23, Tables 232–234, §7.40, Table 392, and §4.58, Table 145; cover optional identifier, required Long Integer (`i64`) count, required response identifier, malformed responses, Table 234 failures, Pending, unknown reasons, one exchange, delivery, and no retry. QA identified and corrected the original invalid-identifier case: Table 145 permits Text String, Enumeration, and Integer, so the final Red tests use Boolean as invalid and preserve Integer/open Enumeration as positive cases; the follow-up also fixes a fixture temporary borrow. Verified the corrected Red in a clean detached worktree at `d0d413f5`: `cargo test -p kmipkit-protocol -p kmipkit-client --lib get_usage_allocation` exited 1 with E0432 unresolved imports for the not-yet-implemented request/response types, as expected. Independent QA approved the contract and citation.

### Allocation Green commit

- [x] T006 [US2] Implement `GetUsageAllocationRequest`/`GetUsageAllocationResponse` in `crates/kmipkit-protocol/src/get_usage_allocation.rs`, export through `crates/kmipkit-protocol/src/lib.rs` and `crates/kmipkit/src/lib.rs`, and wire typed execution/results in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T005 and commit separately as Green. Green evidence: `cargo test -p kmipkit-protocol -p kmipkit-client --lib get_usage_allocation` passed (6 protocol, 5 client); `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` passed. Response retains the complete response TTLV, accepts unknown siblings and open Unique Identifier wire forms, and exposes common Failure/Pending results without polling or retry. Independent QA found no normative or implementation blockers.

### Allocation Refactor commit

- [x] T007 [US2] Refactor allocation request/response field handling in `crates/kmipkit-protocol/src/get_usage_allocation.rs`; preserve Long Integer wire type, optionality, and response ID; rerun tests in `crates/kmipkit-protocol/tests/unit/get_usage_allocation_tests.rs` and `crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs`, then commit separately as Refactor. Simplified required-response-identifier extraction while preserving distinct missing-payload and malformed-payload errors. Verified the 11 focused Get Usage Allocation tests, focused Clippy with `-D warnings`, formatting, and `git diff --check`; independent QA approved without behavior changes.

### Guard Red commit

- [x] T008 [US2] Add failing `UsageAllocationContext` tests in `crates/kmipkit-client/tests/unit/usage_allocation_guard_tests.rs` and compile-fail fixture `crates/kmipkit-client/tests/ui/usage_allocation_guard_not_clone.rs`, exercised by `crates/kmipkit-client/tests/usage_allocation_guard_boundary.rs`; cite OASIS KMIP Specification v2.1 §§4.2.1, 4.59, and 6.1.23, Tables 148 and 232–233 plus §11.60, Table 491; cover context construction only from complete Success, caller-supplied count/unit, returned/resolved object ID, non-cloneability, Encrypt/MAC/Sign scope, positive cost, exact-budget acceptance, over-budget/missing (including batch ID Placeholder omission)/mismatched ID rejection with NotSent and no transport, and no accounting for Decrypt/Verify/Hash or ordinary guarded-scope Encrypt/MAC/Sign calls; record Red output separately. Red evidence: `cargo test -p kmipkit-client --lib usage_allocation_guard_tests` exited 101 only for absent `UsageAllocationContext` and guarded client methods; `cargo test -p kmipkit-client --test usage_allocation_guard_boundary` passed its expected compile-fail case; `cargo fmt --all --check` and `git diff --check` passed. Independent QA approved the final Red contract, including §4.2.1 ID Placeholder omission, §11.60 extension unit `0x8000_0054`, async Pending reservation, valid SHA-256 fixture, and ordinary unguarded protection accounting.

### Guard Green commit

- [x] T009 [US2] Implement the non-cloneable caller-owned context and guarded one-item typed protection path in `crates/kmipkit-client/src/usage_allocation.rs`, `crates/kmipkit-client/src/execute.rs`, `crates/kmipkit-client/src/lib.rs`, `crates/kmipkit-protocol/src/mac.rs`, `crates/kmipkit-protocol/src/sign.rs`, and `crates/kmipkit/src/lib.rs`; expose only explicit unit/cost inputs, preserve open values, reject invalid context/cost/ID before send, use positive `i64` local costs and checked arithmetic consistent with §7.40/Table 392, and make T008 pass; commit separately as Green. Green evidence: all 12 `usage_allocation_guard_tests` pass; `cargo test -p kmipkit-client --test usage_allocation_guard_boundary` passes with a compile-fail specifically proving the context lacks `Clone`; focused Clippy across protocol/client/facade with all targets/features and `-D warnings`, `cargo fmt --all --check`, and `git diff --check` pass. Coverage includes full-range `i64::MAX` reservation and overflow rejection before transport. Independent QA approved with no blockers.

### Guard Refactor commit

- [x] T010 [US2] Refactor checked local reservation accounting and delivery-state transitions in `crates/kmipkit-client/src/usage_allocation.rs`, `crates/kmipkit-client/src/execute.rs`, and `crates/kmipkit-client/tests/unit/usage_allocation_guard_tests.rs`; prove `NotSent` releases while `PossiblySent`, `ResponseStarted`, complete Success/Failure, and Pending retain locally, rerun focused tests, and commit separately as Refactor. Centralized the error-settlement transition in `UsageAllocationContext::settle_after_error`; only exact `Some(NotSent)` releases, and all other delivery evidence retains. Verified 12 focused tests (including exact Pending correlation bytes), trybuild non-Clone boundary, focused Clippy `-D warnings`, formatting, and `git diff --check`. Independent QA approved the behavior-preserving refactor.

**Checkpoint**: A successful allocation may seed one explicit local context; guarded requests cannot exceed its caller-known local allowance. No remote usage is inferred.

## Phase 4: User Story 3 — Set Constraints (Priority: P1)

**Goal**: Send required Constraints using approved KMIPKIT-DISC-020 interpretation and preserve the common response.

**Independent Test**: A fake transport captures Table 325 and returns empty Table 326 success or Table 327 failure.

### Red commit

- [x] T011 [US3] Add failing protocol/client tests in `crates/kmipkit-protocol/tests/unit/set_constraints_tests.rs` and `crates/kmipkit-client/tests/unit/set_constraints_execution_tests.rs`; register modules through protocol/client test module declarations; cite OASIS KMIP Specification v2.1 §6.1.52, Tables 325–327; cover required Constraints, order/unknown nested data, approved DISC-020 error assignment, empty success, malformed response, Pending, delivery state, and no retry; record Red output separately. Red evidence: `cargo test -p kmipkit-protocol --lib set_constraints_tests` exited 1 with E0432 for absent `SetConstraintsRequest`/`SetConstraintsResponse`; `cargo test -p kmipkit-client --lib set_constraints_execution_tests` exited 1 with E0432/E0599 for absent request type, `ClientRequest::set_constraints`, and response accessor. `cargo fmt --all --check` and `git diff --check` passed. Independent QA approved the corrected test contract: request order checks typed values/nested data, unknown response fields retain their value, Pending correlation is asserted from the response, and there is one exchange with no retry. A follow-up test correction constructs a raw malformed message to assert common rejection of missing Response Payload, because `response_message` intentionally panics on invalid fixtures.

### Green commit

- [x] T012 [US3] Implement `SetConstraintsRequest`/`SetConstraintsResponse` in `crates/kmipkit-protocol/src/set_constraints.rs`, export via protocol/facade `lib.rs`, and add typed execution in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; pass T011 and commit separately as Green. Success requires the common Response Payload Structure and preserves its full TTLV, including unknown vendor fields, while applying Table 326's empty known payload. The client exposes both `ClientRequest::set_constraints` and `Client::set_constraints`; Pending remains explicit with no polling/retry. Verified 6 protocol and 5 client focused tests, focused Clippy (`-D warnings`), formatting, and `git diff --check`. Independent QA approved the implementation and API wiring.

### Refactor commit

- [x] T013 [US3] Refactor Set Constraints encoding and Table 327 result tests in `crates/kmipkit-protocol/src/set_constraints.rs`, `crates/kmipkit-protocol/tests/unit/set_constraints_tests.rs`, and `crates/kmipkit-client/tests/unit/set_constraints_execution_tests.rs` without changing Table 325 behavior; rerun focused tests and commit separately as Refactor. Kept the production encoder unchanged after review found no beneficial wire-code cleanup; refactored the distinct Table 327 reason inventory and added owned-response lifetime plus request/response Debug-redaction coverage. Verified 8 protocol and 5 client tests, focused Clippy (`-D warnings`), formatting, and `git diff --check`. Independent QA approved with no behavior change.

## Phase 5: User Story 4 — Set Defaults (Priority: P1)

**Goal**: Preserve the semantic difference between absent Defaults Information (remove all Object Defaults) and a supplied Structure.

**Independent Test**: Fake transport verifies Table 328 omission vs presence and empty Table 329 success or Table 330 error.

### Red commit

- [x] T014 [US4] Add failing protocol/client tests in `crates/kmipkit-protocol/tests/unit/set_defaults_tests.rs` and `crates/kmipkit-client/tests/unit/set_defaults_execution_tests.rs`; register modules through protocol/client test module declarations; cite OASIS KMIP Specification v2.1 §6.1.53, Tables 328–330 and §7.11; cover absent vs empty/present Defaults Information, unknown nested elements, Table 330's approved DISC-020 assignment, malformed success Response Payload, Pending, delivery, and no retry; record Red output separately. Red evidence: `cargo test -p kmipkit-protocol --lib set_defaults_tests` exited 1 with E0432 for absent `SetDefaultsRequest`/`SetDefaultsResponse`; `cargo test -p kmipkit-client --lib set_defaults_execution_tests` exited 1 with E0432/E0599 for absent request type, `ClientRequest::set_defaults`, and `ClientBatchOutcome::SetDefaults`. `cargo fmt --all --check` and `git diff --check` passed. The malformed Success test builds a raw response and confirms the common §8.6/Table 399 validator rejects missing Response Payload. Independent QA approved the tests and corrected normative attribution: Table 328 defines omission semantics; DISC-020 applies to the Table 330 error introduction.

### Green commit

- [x] T015 [US4] Implement `SetDefaultsRequest`/`SetDefaultsResponse` in `crates/kmipkit-protocol/src/set_defaults.rs`, export via protocol/facade `lib.rs`, and add explicit execution in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; preserve omission as omission, pass T014, and commit separately as Green. Table 328 omission is encoded as no field; `Some(Structure::new())` remains one empty Defaults Information Structure. Typed success retains the full response payload and unknown fields. Client exposes batch and direct convenience APIs; Pending uses one exchange with exact correlation and no implicit Poll/retry. Verified 7 protocol and 7 client focused tests, focused Clippy (`-D warnings`), formatting, and `git diff --check`. Independent QA approved; source attribution follows Table 328 for omission and DISC-020 for Table 330's truncated error introduction.

### Refactor commit

- [x] T016 [US4] Refactor Set Defaults request/result handling in `crates/kmipkit-protocol/src/set_defaults.rs`, `crates/kmipkit-protocol/tests/unit/set_defaults_tests.rs`, and `crates/kmipkit-client/tests/unit/set_defaults_execution_tests.rs` while preserving absent-vs-empty behavior; rerun focused tests and commit separately as Refactor. Kept production encoding unchanged after review found no beneficial wire-code cleanup; deduplicated Table 330's reason inventory and added tests for response ownership after source drop and Debug redaction. Verified 9 protocol and 7 client focused tests, focused Clippy (`-D warnings`), formatting, and `git diff --check`. Independent QA approved without behavior changes.

## Phase 6: Scope resolution — Set Endpoint Role deferred from 1.0

**Goal**: Resolve the conflict between §6.1.54's mandatory post-success role transition and KMIPKit's client-only 1.0 boundary; do not ship a partial operation.

**Independent Test**: Confirm that the 1.0 typed API and module registrations do not expose Set Endpoint Role, and that the normative catalog retains OASIS records while reflecting its 1.1 disposition.

### Normative review (Red evidence)

- [x] T017 Review the client-to-server Set Endpoint Role contract in OASIS KMIP 2.1 §6.1.54, Tables 331–333 and §11.19, Tables 451–452; identify whether typed request/response tests can satisfy the full success behavior inside the 1.0 client-only boundary. Initial Red tests demonstrated the API was not implemented (Red commit `07a118a2`); review then confirmed that successful completion requires the server to assume the Client role and KMIPKit to assume Server role over the current channel. §6.2.5 defines the reverse-direction operation. Independent QA confirmed no client-only interpretation that implements the full role transition. Decision: defer both directions to 1.1; do not implement a payload-only partial operation.

### Scope disposition (Green)

- [x] T018 Record the 1.0 deferral and exact OASIS rationale in `spec.md`, `plan.md`, `research.md`, `data-model.md`, `traceability.md`, `contracts/operations.md`, `quickstart.md`, and the interoperability matrix. The spec states that a separate approved 1.1 specification is required for bidirectional channel behavior; T020 updates catalog release-scope metadata while preserving OASIS records.

### Cleanup and verification (Refactor)

- [x] T019 Remove the exploratory Set Endpoint Role tests and their test-module registrations; verify there is no typed model, client method, dispatcher variant, or test module for this deferred operation in 1.0. Keep generic TTLV unknown-tag/enum preservation unchanged. Independent QA reviewed the code/API/module search and confirmed no 1.0 typed-surface remnants.

## Phase 7: Traceability, parity, and release evidence

**Purpose**: Complete catalog traceability, cross-language capability, examples, interop record, and quality gates.

- [x] T020 Update `specification/catalog/kmip-2.1.json` with feature ID, implementation/verification references for the four in-scope operation/tag/structure/enum elements and all four §6.1.23 client requirements. Retain Set Endpoint Role's normative record but mark its implementation scope `client_1_1` with the approved deferral rationale. Update `specs/025-policy-controls/traceability.md`, then run the pinned catalog validator and generated coverage report checks; do not edit generated report output by hand. Tags retain their original `KMIPKIT-0004` allocation ownership while gaining 025 implementation and verification references. Verified: `python tools/normative_catalog/validate.py` reports `Catalog valid: sources=4 clauses=1411 records=4044`; report `--write` and `--check` succeed; all 213 normative-catalog unit tests pass (7 platform skips). The operation-inventory regression confirms the normative Set Endpoint Role records remain present but scoped to `client_1_1`.
- [ ] T021 Add failing C ABI and Java/Python parity tests for the four in-scope operations and explicit caller guard in `bindings/c/tests/policy_controls.c`, `bindings/java/src/test/java/org/kmipkit/PolicyControlsParityTest.java`, `bindings/python/tests/test_policy_controls.py`, and `crates/kmipkit-ffi/tests/c_api_coverage.rs`; register the C test in `bindings/c/CMakeLists.txt`; do not begin until KMIPKIT-0043's manifest contract is approved and merged; record a separate Red evidence commit.
- [ ] T022 Add approved entries to `specification/api/public-api.json`, implement C/JNI/Java/Python adapters in `crates/kmipkit-ffi/src/lib.rs`, `bindings/java/native/kmipkit_jni.cpp`, `bindings/java/src/main/java/org/kmipkit/PolicyControls.java`, and `bindings/python/src/kmipkit/policy_controls.py`; regenerate C header and language metadata with `python tools/api_manifest/generate.py --write`; pass T021 and commit the manifest/adapters/generated outputs together as separate Green evidence.
- [ ] T023 Refactor language parity adapters/tests in `crates/kmipkit-ffi/src/lib.rs`, `bindings/java/native/kmipkit_jni.cpp`, `bindings/java/src/main/java/org/kmipkit/PolicyControls.java`, `bindings/python/src/kmipkit/policy_controls.py`, and the parity test files without changing the manifest contract; run `python tools/api_manifest/generate.py --check` and binding tests, then commit separately as Refactor.
- [x] T024 Add English/Spanish operation and guard examples in `docs/user-guide/en/policy-controls.md` and `docs/user-guide/es/controles-de-uso.md`; compile-check a Rust mirror and verify its linkage to both guides in `crates/kmipkit-client/tests/unit/policy_controls_guide_examples.rs`; document caller cost/unit responsibility and local-only reservation limits without claiming conformance or remote consumption. The test compiles the mirrored code rather than extracting Markdown fences. Verified two guide tests pass; one exercises absent versus explicitly empty Defaults Information.
- [x] T025 Update `docs/development/kmip-interoperability.md` with the four operation rows and exact integration evidence fields; leave results Pending until measured against identified server/version configurations; do not claim support based only on safe rejection. The Usage Allocation guard is local-only and is not a server interoperability result. Verified the matrix includes all four OASIS operation rows, leaves all server results Pending, and specifies revision, product/build/digest, listener configuration, transport/TLS, certificates, operation/request, response/delivery, UTC date, outcome, and workaround evidence.
- [x] T026 Run all focused tests and commands in `specs/025-policy-controls/quickstart.md`, plus the required Rust workspace, documentation, catalog, manifest, dependency-policy, and user-guide checks; record exact outcomes and limitations. On Windows x86_64, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked`, docs, generated catalog/manifest checks, dependency policy, and bilingual examples passed. The follow-up tests cover every reachable changed Rust line identified after hosted run 38174854498 failed the changed-code gate at 529/686 lines (77.1%); that earlier run's other checks passed. A local cross-platform report comparison measured 682/686 lines (99.42%), leaving only four defensive branches unreachable through public validated inputs: `get_usage_allocation.rs` push failures after fixed valid fields (`:72`, `:78`), and the missing-success-payload conversions in `set_constraints.rs:97` and `set_defaults.rs:124`, which the common response validator rejects earlier. This local comparison is not the hosted result. The updated commit `bbfc1d73` passed hosted GitHub Actions run 38177656469: all required core matrix, three-platform Rust/FFI coverage, selected-scope coverage gate, C, JNI, Java, Python, script, and traceability checks completed successfully. `cargo llvm-cov --workspace --all-features` could not complete locally because three unrelated DNS tests could not bind loopback sockets (`WSAEACCES`); hosted platform coverage passed. Live KMIP 2.1 integration probes remain unavailable locally: Docker's Linux daemon is absent and PyKMIP supports at most KMIP 2.0. The two-independent-KMIP-2.1-implementation release gate remains open; no P025 operation is represented as server-tested.

## Dependencies and execution order

- T001 precedes implementation; it verifies immutable sources and the release baseline.
- US1 (T002–T004) establishes the first shared dispatcher integration.
- US2 allocation (T005–T007) follows US1 because `execute.rs` and crate exports are shared. The guard (T008–T010) follows allocation and consumes its resolved success result.
- US3 and US4 follow the prior shared client integration sequence to avoid concurrent conflicting edits to `crates/kmipkit-client/src/execute.rs`, client exports, and protocol exports. Their payload models and isolated tests remain separable after their contracts are stable. Set Endpoint Role is a documented scope disposition, not an implementation user story.
- T020–T026 depend on the four in-scope operation stories, usage guard, and documented Set Endpoint Role deferral. Cross-language T021–T023 additionally require KMIPKIT-0043's approved manifest contract/merge; this does not block Rust implementation or documentation preparation.
- T025 and external integration evidence can be prepared in parallel with parity after the Rust behavior is stable, but the interop matrix remains Pending until real measurements are recorded.

## Independent acceptance criteria

- **US1**: Tables 229–231, including empty request, required Constraints, unknown nested fields, error reasons, Pending, and one-exchange behavior.
- **US2 allocation**: Tables 232–234, exact Long Integer count, optional/requested identifier behavior, required resolved response ID, and common result/delivery behavior.
- **US2 guard**: successful result plus explicit exact count/unit/ID context; positive caller cost only; matching ID; no over-budget transmission; delivery reservations; local-only accounting.
- **US3**: Tables 325–327 and approved KMIPKIT-DISC-020 disposition.
- **US4**: Tables 328–330; Table 328 says absent Defaults Information means removal; DISC-020 applies to Table 330 error attribution.
- **Scope disposition**: Set Endpoint Role (§6.1.54) and reverse direction (§6.2.5) are not advertised as 1.0 capabilities; the normative catalog retains both OASIS operations and marks the client-to-server operation as deferred to 1.1 with rationale.
