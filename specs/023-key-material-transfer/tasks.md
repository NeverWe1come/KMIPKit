# Tasks: KMIP 2.1 Key Derivation and Transfer

**Input**: Design documents from `specs/023-key-material-transfer/`.

**Prerequisites**: Approved [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/public-rust.md](contracts/public-rust.md), and [traceability.md](traceability.md).

**TDD and commit evidence**: Each operation story requires a distinct Red test commit, Green implementation commit, and Refactor commit. Join Split Key tests include the approved 2026-10-11 threshold context rule. PR evidence follows [quickstart.md](quickstart.md).

**Organization**: Tasks are grouped by the four P1 stories. Operation payload files are independent; client dispatcher, public exports, catalog, and generated report are serialized.

## Phase 1: Setup and prerequisite gate

**Purpose**: Confirm the worktree, immutable sources, common types, and public API boundary.

- [x] T001 Confirm the managed worktree is on `feature/KMIPKIT-0037-key-material-transfer-implementation`, based initially on active `release/1.0.0` at `af172974cd5c6f56f7f938574e1546d4e295a9ef`, and record that the worktree was clean at creation; identify the occupied prescribed branch in implementation evidence. Rebased cleanly onto `origin/release/1.0.0` at `a99e419fdb7d61ed52bb4637796e5e18bbc3009c` on 2026-10-11.
- [x] T002 Verify source checksums in `specification/oasis/kmip-2.1/CHECKSUMS.sha256` and confirm no changes under `specification/oasis/kmip-2.1/upstream/`.
- [x] T003 Confirm shared object/attribute, generic TTLV, operation-data, Cryptographic Parameters, result, batch, client, and secret ownership APIs in `crates/kmipkit-protocol/src/` and `crates/kmipkit-client/src/execute.rs`; reserve four-language mappings for KMIPKIT-0043.

## Phase 2: Shared execution and secret contract tests

**Purpose**: Establish common failing tests before production dispatch/model changes.

- [x] T004 [P] Add failing protocol tests in `crates/kmipkit-protocol/tests/unit/key_transfer_operation_contract_tests.rs` for open result/error values, unknown child retention, secret redaction, and owned-byte handling; commit Red evidence (`882bf416`). Final harness registration in `crates/kmipkit-protocol/src/lib.rs` makes the shared suite executable; `cargo test -p kmipkit-protocol --lib key_transfer_operation_contract_tests` passed 4 tests on 2026-10-11.
- [x] T005 [P] Add failing fake-transport tests in `crates/kmipkit-client/tests/unit/key_transfer_operations_execution_tests.rs` for all four operation codes, one exchange, result/batch association, Pending, delivery-state failures, and no retry; commit before implementation (`ba6fd674`). Final harness registration in `crates/kmipkit-client/src/execute.rs` makes the shared suite executable; after correcting invalid/stale synthetic fixtures, `cargo test -p kmipkit-client --lib key_transfer_operations_execution_tests` passed 4 tests on 2026-10-11, including completed typed responses and all public response-view accessors.

## Phase 3: User Story 1 — Derive Key (Priority: P1)

**Goal**: Request server-side derivation with caller-selected input, method, algorithm, length, and output attributes.

**Independent Test**: Tables 205–207 and §7.12/Table 364; ID Placeholder prohibition; explicit source IDs and algorithm details; one exchange and no local derivation.

- [x] T006 [P] [US1] Add failing Derive Key payload and fake-client tests in `crates/kmipkit-protocol/tests/unit/derive_key_operation_tests.rs` and `crates/kmipkit-client/tests/unit/derive_key_execution_tests.rs` for §6.1.14 Tables 205–207, §7.12 Table 364 and traceability rows, plus the both-inputs server-error passthrough without local pre-rejection; commit Red. Current protocol focal suite passes 9 tests, including explicit parameter preservation and malformed success IDs.
- [x] T007 [US1] Implement `DeriveKeyRequest/Response/Error` in `crates/kmipkit-protocol/src/derive_key.rs`, export it, and integrate `ClientRequest`/operation code/response/convenience dispatch in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T006 passes.
- [x] T008 [US1] Refactor Derive Key payload/dispatch/tests in `crates/kmipkit-protocol/src/derive_key.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files without changing caller-controlled semantics; commit Refactor separately.

## Phase 4: User Story 2 — Export (Priority: P1)

**Goal**: Request a server-side export with explicit optional format/wrapping/compression choices.

**Independent Test**: Tables 217–219; all response object/attribute fields; secret redaction; no local transformation.

- [x] T009 [P] [US2] Add failing Export request, success/error response, and fake-client tests in `crates/kmipkit-protocol/tests/unit/export_operation_tests.rs` and `crates/kmipkit-client/tests/unit/export_execution_tests.rs` for §6.1.18 Tables 217–219 and its traceability rows; commit Red. Current protocol focal suite passes 6 tests.
- [x] T010 [US2] Implement `ExportRequest/Response/Error` in `crates/kmipkit-protocol/src/export.rs`, export it, and integrate dispatch/client convenience methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T009 passes.
- [x] T011 [US2] Refactor Export model/dispatch/tests in `crates/kmipkit-protocol/src/export.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; commit Refactor separately.

## Phase 5: User Story 3 — Import (Priority: P1)

**Goal**: Send an object and caller-provided attributes for server-side import, with replacement only by explicit intent.

**Independent Test**: Tables 238–240; required identifier/type/attributes/object; conditional wrapping; explicit Replace Existing; one exchange; no local unwrap/import.

- [x] T012 [P] [US3] Add failing Import request, response/error, and fake-client tests in `crates/kmipkit-protocol/tests/unit/import_operation_tests.rs` and `crates/kmipkit-client/tests/unit/import_execution_tests.rs` for §6.1.25 Tables 238–240 and its traceability rows; commit Red. Current protocol focal suite passes 9 tests.
- [x] T013 [US3] Implement `ImportRequest/Response/Error` in `crates/kmipkit-protocol/src/import.rs`, export it, and integrate dispatch/client methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T012 passes.
- [x] T014 [US3] Refactor Import payload/dispatch/tests in `crates/kmipkit-protocol/src/import.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files without local unwrapping; commit Refactor separately.

## Phase 6: User Story 4 — Join Split Key (Priority: P1)

**Goal**: Combine an ordered set of caller-identified split keys after validating the caller-known threshold context.

**Independent Test**: Tables 244–246; one caller-known threshold per identifier occurrence; ID order/repetitions preserved; entry count >= maximum threshold before network transmission; no lookup; one exchange.

- [x] T015 [P] [US4] Add failing Join Split Key protocol and fake-client tests in `crates/kmipkit-protocol/tests/unit/join_split_key_operation_tests.rs` and `crates/kmipkit-client/tests/unit/join_split_key_execution_tests.rs` for missing/misaligned threshold context, count below/equal/above max, different thresholds, repeated field/value entries, exact wire ID order, and §6.1.27 Tables 244–246; commit Red. Current protocol focal suite passes 9 tests.
- [x] T016 [US4] Implement `SplitKeyReference` and `JoinSplitKeyRequest/Response/Error` in `crates/kmipkit-protocol/src/join_split_key.rs`; require explicit non-wire per-entry threshold context, validate request entry count >= maximum threshold, preserve all ID occurrences, and integrate client dispatch in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`; commit Green after T015 passes (`2ff342bb`).
- [x] T017 [US4] Refactor Join Split Key model/validation/dispatch/tests in `crates/kmipkit-protocol/src/join_split_key.rs`, `crates/kmipkit-client/src/execute.rs`, and both test files; preserve the 2026-10-11 interpretation and commit Refactor separately (`6b0ad9a3`).

## Phase 7: Cross-language parity, documentation, and release evidence

**Purpose**: Complete manifest-backed public mapping, traceability, docs, generated outputs, and release evidence.

- [ ] T018 **Blocked on KMIPKIT-0043 operation mappings.** The read-only 2026-10-11 check of its v2 manifest found these operation IDs, but T046, T047, T050, and T052 remain unchecked and the operation-dependencies contract keeps mappings blocked pending canonical approved Rust mapping and parity evidence. Do not edit or close tasks on that branch from this feature. Complete those four mappings only after the owning 0043 task provides them; then reconcile `specification/api/public-api.json` and verify generated declarations in `crates/kmipkit-ffi/`, `bindings/c/`, `bindings/java/`, and `bindings/python/` use the pinned generator.
- [x] T019 Add executable English and Spanish examples in `docs/user-guide/en/key-derivation-and-transfer.md` and `docs/user-guide/es/derivacion-y-transferencia-de-claves.md`; compile/run them with the approved guide mechanism and use synthetic redacted material. Evidence: on 2026-10-11 the targeted guide command compiled 2 marked Rust examples and the default `python scripts/test_user_guide_examples.py` compiled all 16 marked examples, both exit 0.
- [x] T020 Update traceability for all 18 stable ledger IDs and every mandatory/conditional payload-table row in `specs/023-key-material-transfer/traceability.md` and `specification/catalog/kmip-2.1.json`. Evidence: Red test commit `1ec2ec63` first failed on missing feature assignments and executable refs; Green catalog/traceability update passes `python -m unittest tools.normative_catalog.tests.test_key_derivation_transfer_traceability` (3 tests) and `python tools/normative_catalog/validate.py` (4044 records valid). Catalog notes retain source/server-authoritative conditions and do not claim official fixtures ran.
- [ ] T021 Regenerate and validate `specification/catalog/coverage-report.md` and affected generated operation artifacts using pinned tools; do not edit generated outputs manually. Partial evidence: `python tools/normative_catalog/report.py --write` generated the report and `--check` passed. Operation binding/generated declarations remain gated on T018 and are not manually changed.
- [ ] T022 Run focused operation tests, formatter, workspace Clippy/tests/coverage, dependency/security/parser/FFI checks, and supported-platform CI; record actual commands/results in `specs/023-key-material-transfer/quickstart.md`. The follow-up's focused operation tests, formatter, workspace Clippy/tests, normative catalog checks, independent QA, and exact-range security scan are recorded there. The local Windows-only production projection is not the CI aggregate and does not include the Linux C consumer report. Prior release-line evidence: PR #93 run [38156503260](https://github.com/NeverWe1come/KMIPKit/actions/runs/38156503260) passed supported-platform tests, Linux FFI sanitizers/C consumer/fuzz smoke, and dependency/parser checks; only scheduled release/nightly jobs were skipped. PR #98 coverage run [38211113445](https://github.com/NeverWe1come/KMIPKit/actions/runs/38211113445/job/114687344804) measured 25,811/26,709 = 96.64%. Keep this task open until the follow-up PR's actual required coverage, platform, dependency, parser, and FFI checks pass.
- [ ] T023 Independent QA and security reviews are complete with no remaining findings on the final diff; Red/Green/Refactor evidence is in `quickstart.md`. The branch was based on the current fetched `release/1.0.0` at `7a430d5f`. Create the draft PR, include traceability/catalog and generator evidence, parity status, fixture limitations, risks, and verification results, then close this task after the PR exists.

## Dependencies & Execution Order

- T001–T003 precede implementation; T004–T005 establish shared Red evidence first.
- Each story's Red task precedes Green; its Refactor commit follows the passing Green state.
- Four Rust operation model/test slices are independent. Dispatcher, protocol exports, catalog, and generated report are shared serialized edits.
- Rust operations may start without KMIPKIT-0043. T018 waits for the canonical manifest and its operation-specific mapping work; that gate applies to four-language parity only.
- Documentation, traceability, generator, and release checks follow operation behavior.

## Parallel Opportunities

- Protocol Red tests and model work for distinct operations can be prepared on separate files after shared API conventions are fixed.
- Derive Key, Export, Import, and Join Split Key table vectors are operation-specific.
- Do not parallel-edit `crates/kmipkit-client/src/execute.rs`, `crates/kmipkit-protocol/src/lib.rs`, `specification/catalog/kmip-2.1.json`, or the generated coverage report.

## Implementation Strategy

Deliver each operation as an independently testable typed request/response/error slice with distinct Red, Green, and Refactor commits. Start with Derive Key to exercise reuse of operation-data and Cryptographic Parameters, then complete other operations while serializing shared client integration. Join Split Key's context/threshold contract is fixed by the approved 2026-10-11 interpretation. Close with 0043 parity mappings, bilingual docs, complete requirement/table traceability, generated outputs, coverage/security/platform evidence, and a human-reviewed draft PR. Do not equate source-derived table vectors with official fixture passes or formal profile/certification.
