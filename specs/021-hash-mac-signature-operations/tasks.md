# Tasks: KMIP 2.1 Hash, MAC, and Signature Operations

**Input**: Design documents from `specs/021-hash-mac-signature-operations/`

**Prerequisites**: Satisfied for this implementation: KMIPKIT-0019's shared `OperationData`, Cryptographic Parameters, and multipart types are present in the `release/1.0.0` base, and KMIPKIT-0021 was approved before implementation. The feature branch starts from `release/1.0.0` at `db51b7a1c93edbe81f89a9faf7902aa543caec3f`; T001–T003 evidence is recorded below.

**Tests**: Required by the feature and repository TDD rules. Every story has distinct Red, Green, and Refactor commits. Red tests must fail for the intended missing operation behavior before production edits.

## Format: `[ID] [P?] [Story] Description`

- `[P]` is used only when a task can proceed on different files without incomplete dependencies.
- Story labels map to `spec.md`: `[US1]` Hash; `[US2]` MAC and Sign; `[US3]` MAC Verify and Signature Verify.
- Every task names the exact files to change or verify.

## Phase 1: Setup and prerequisite gate

**Purpose**: Start implementation from the right release state and reuse #0019 contracts.

- [x] T001 Confirm KMIPKIT-0019 implementation is merged and `release/1.0.0` exports the shared `OperationData`, Cryptographic Parameters, Correlation Value, Init Indicator, and Final Indicator contracts used by `specs/021-hash-mac-signature-operations/plan.md`.
- [x] T002 Create the implementation branch and worktree from current `release/1.0.0`; record the base SHA and clean-worktree result in the implementation PR. Base: `db51b7a1c93edbe81f89a9faf7902aa543caec3f`; branch: `feature/KMIPKIT-0021-hash-mac-signature-implementation`.
- [x] T003 Confirm the pinned OASIS source hashes in `specification/oasis/kmip-2.1/CHECKSUMS.sha256` and confirm no diff exists under `specification/oasis/kmip-2.1/upstream/`.

## Phase 2: Shared operation execution foundation

**Purpose**: Establish shared red tests and request/response dispatch before completing independent operation slices.

- [x] T004 Add failing client tests in `crates/kmipkit-client/tests/unit/hash_mac_signature_execution_tests.rs` proving five operation codes, one exchange per explicit call, batch item correlation, common result preservation, and no automatic retry.
- [x] T005 Add failing protocol tests in `crates/kmipkit-protocol/tests/unit/cryptographic_operation_contract_tests.rs` for redacted byte wrappers, generic Cryptographic Parameters preservation, unknown enumeration values, and shared multipart field conversion.
- [x] T006 Implement only the minimal shared execution dispatch scaffolding in `crates/kmipkit-client/src/execute.rs` and public exports in `crates/kmipkit-client/src/lib.rs` needed for the operation-specific requests; preserve existing validation and response ownership.
- [x] T007 Refactor the shared operation test helpers in `crates/kmipkit-client/tests/unit/hash_mac_signature_execution_tests.rs` and `crates/kmipkit-protocol/tests/unit/cryptographic_operation_contract_tests.rs` without changing behavior; record a separate Refactor commit after the Green commit.

## Phase 3: User Story 1 — Request a hash from a KMIP server (Priority: P1)

**Goal**: Add a typed Hash request/result and a synchronous client method that preserves Hash data and multipart fields without local hashing.

**Independent Test**: Protocol tests cover Tables 235–237 and Hashing Algorithm values. Fake transport captures a single-part Hash and explicit multipart calls, and verifies exact payloads, response/result shape, one exchange, and no local computation.

### Red commit

- [x] T008 [US1] Add failing Hash request tests in `crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs` for required Cryptographic Parameters, single-part Data, absent multipart Data, optional Correlation/Init/Final fields, field order, and invalid missing input.
- [x] T009 [US1] Add failing Hash response/error tests in `crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs` for Table 236 cardinality, Data bytes, Correlation Value, non-success results, malformed types, and generic unknown fields.
- [x] T010 [US1] Add failing Hash execution tests in `crates/kmipkit-client/tests/unit/hash_execution_tests.rs` for operation code, one exchange, batch association, Pending/failure results, delivery evidence, and no retry.
- [x] T011 [US1] Add failing Hashing Algorithm round-trip tests in `crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs` for every standard value, Extensions range, and unknown value.

### Green commit

- [x] T012 [US1] Implement `HashRequest`, `HashResponse`, and `HashError` in `crates/kmipkit-protocol/src/hash.rs` with table-based local validation, redacted formatting, source response retention, and payload conversion.
- [x] T013 [US1] Export Hash protocol types from `crates/kmipkit-protocol/src/lib.rs` and integrate Hash request/response dispatch and convenience methods in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`.
- [x] T014 [US1] Implement Hash fake-transport tests in `crates/kmipkit-client/tests/unit/hash_execution_tests.rs` and verify all Red cases pass without invoking local hashing.

### Refactor commit

- [x] T015 [US1] Refactor Hash payload/error helpers in `crates/kmipkit-protocol/src/hash.rs` and `crates/kmipkit-protocol/tests/unit/hash_operation_tests.rs`; retain distinct Red/Green/Refactor commits and exact catalog traceability.

**Checkpoint**: Hash passes all source-derived tests independently and uses the common execution path once per call.

## Phase 4: User Story 2 — Request MAC or signature from a KMIP server (Priority: P1)

**Goal**: Add typed MAC and Sign request/result models with explicit key/parameter choices, opaque input/output bytes, and caller-controlled multipart use.

**Independent Test**: Tests cover request and response tables 259–261 and 334–336, optional key/parameters, MAC Data, Sign Data versus Digested Data, output bytes, errors, generic values, redaction, one exchange, and no retry.

### Red commit

- [x] T016 [US2] Add failing MAC payload tests in `crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs` for optional request Unique Identifier/parameters, single-part Data, absent multipart Data, optional Correlation/Init/Final, Table 260 output fields, and successful-response Unique Identifier missing/duplicate/type validation.
- [x] T017 [US2] Add failing Sign payload tests in `crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs` for optional request Unique Identifier/parameters, Data required unless single-part Digested Data is supplied, no multipart Data, optional Digested Data, Table 335 output fields, and successful-response Unique Identifier missing/duplicate/type validation.
- [x] T018 [US2] Add failing client tests in `crates/kmipkit-client/tests/unit/mac_execution_tests.rs` and `crates/kmipkit-client/tests/unit/sign_execution_tests.rs` for correct operation code, one exchange, server-result preservation, batch behavior, and no retries.
- [x] T019 [US2] Add failing enum/round-trip and redaction tests in `crates/kmipkit-protocol/tests/unit/mac_operation_tests.rs` and `crates/kmipkit-protocol/tests/unit/sign_operation_tests.rs` for all relevant Cryptographic Algorithm, Digital Signature Algorithm, MAC Data, and Signature Data values and sentinel-byte diagnostics.

### Green commit

- [x] T020 [US2] Implement `MacRequest`, `MacResponse`, and `MacError` in `crates/kmipkit-protocol/src/mac.rs` with Table 259–261 behavior and redacted byte handling.
- [x] T021 [US2] Implement `SignRequest`, `SignResponse`, and `SignError` in `crates/kmipkit-protocol/src/sign.rs` with Data/Digested Data, multipart, and Table 334–336 behavior.
- [x] T022 [US2] Export MAC and Sign types in `crates/kmipkit-protocol/src/lib.rs`; integrate MAC and Sign request, response, and client convenience dispatch in `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`.
- [x] T023 [US2] Complete MAC and Sign fake-transport tests in `crates/kmipkit-client/tests/unit/mac_execution_tests.rs` and `crates/kmipkit-client/tests/unit/sign_execution_tests.rs` and make the Red cases pass.

### Refactor commit

- [x] T024 [US2] Refactor shared MAC/Sign parameter and byte handling in `crates/kmipkit-protocol/src/mac.rs`, `crates/kmipkit-protocol/src/sign.rs`, and their unit tests without changing specified wire behavior.

**Checkpoint**: MAC and Sign are independently constructible and testable, with explicit caller parameters and no local cryptographic work.

## Phase 5: User Story 3 — Verify a MAC or signature remotely (Priority: P1)

**Goal**: Add typed verification requests and responses, including lossless Validity Indicator values and the open final multipart response rule.

**Independent Test**: Tests cover Tables 262–264 and 337–339, verification inputs, Valid/Invalid/Unknown, optional recovered data, the five discrepancy cases, errors, result semantics, and generic TTLV retention.

### Red commit

- [x] T025 [US3] Add failing MAC Verify request tests in `crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs` for optional key/parameters/original Data, required single-part MAC Data, absent multipart MAC Data, and multipart fields.
- [x] T026 [US3] Add failing Signature Verify request tests in `crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs` for optional key/parameters/Data/Digested Data, required single-part Signature Data, and absent Signature Data on multipart requests.
- [x] T027 [US3] Add failing verification response tests in `crates/kmipkit-protocol/tests/unit/mac_verify_operation_tests.rs` and `crates/kmipkit-protocol/tests/unit/signature_verify_operation_tests.rs` for Valid/Invalid/Unknown/extension values, required single-part response indicator, both final multipart forms, non-final indicator rejection, required response Unique Identifier missing/duplicate/type validation, recovered Data, and generic TTLV retention.
- [x] T028 [US3] Add failing fake-client tests in `crates/kmipkit-client/tests/unit/mac_verify_execution_tests.rs` and `crates/kmipkit-client/tests/unit/signature_verify_execution_tests.rs` for exactly one exchange, invalid/unknown results as operation results, delivery evidence, errors, and no retry.

### Green commit

- [x] T029 [US3] Implement `MacVerifyRequest`, `MacVerifyResponse`, and `MacVerifyError` in `crates/kmipkit-protocol/src/mac_verify.rs`, including `KMIPKIT-DISC-048` cardinality handling.
- [x] T030 [US3] Implement `SignatureVerifyRequest`, `SignatureVerifyResponse`, and `SignatureVerifyError` in `crates/kmipkit-protocol/src/signature_verify.rs`, including recovered Data and `KMIPKIT-DISC-048` handling.
- [x] T031 [US3] Export verification types in `crates/kmipkit-protocol/src/lib.rs`; integrate both verification operations into `crates/kmipkit-client/src/execute.rs` and `crates/kmipkit-client/src/lib.rs`.
- [x] T032 [US3] Complete fake-client tests in `crates/kmipkit-client/tests/unit/mac_verify_execution_tests.rs` and `crates/kmipkit-client/tests/unit/signature_verify_execution_tests.rs`; verify Invalid and Unknown remain operation results.

### Refactor commit

- [x] T033 [US3] Refactor shared Validity Indicator and response-shape handling in `crates/kmipkit-protocol/src/mac_verify.rs`, `crates/kmipkit-protocol/src/signature_verify.rs`, and verification unit tests without choosing a `KMIPKIT-DISC-048` server interpretation.

**Checkpoint**: Verification operations preserve their response semantics; only final multipart indicator presence remains intentionally tolerant under the open discrepancy.

## Phase 6: Polish and cross-cutting gates

**Purpose**: Close documentation, traceability, security, quality, and compatibility evidence for review.

- [x] T034 Add executable English Rust guide examples in `docs/user-guide/en/cryptographic-operations.md` covering five operations, explicit parameters, multipart use, results, and no local cryptography. **GREEN evidence:** the bilingual user-guide runner compiled the English example against the public client/protocol APIs.
- [x] T035 Add equivalent executable Spanish Rust guide examples in `docs/user-guide/es/operaciones-criptograficas.md` covering five operations, explicit parameters, multipart use, results, and `KMIPKIT-DISC-048`. **GREEN evidence:** the bilingual user-guide runner compiled the Spanish example against the same public APIs.
- [x] T036 Add integration tests in `crates/kmipkit-protocol/tests/cryptographic_operations_guide_examples.rs` to compile both guide examples and prove their sample payload values remain redacted. **RED evidence:** `cargo test -p kmipkit-protocol --test cryptographic_operations_guide_examples --all-features` exited 1 while compiling the new target because both planned guide paths are missing; no unrelated syntax or registration errors occurred. **GREEN evidence:** the same test target passed 2/2; `python3 scripts/test_user_guide_examples.py --guide docs/user-guide/en/cryptographic-operations.md --guide docs/user-guide/es/operaciones-criptograficas.md` compiled both marked guide examples, exit 0.
- [x] T037 Update `specification/catalog/kmip-2.1.json` implementation and verification references for all six applicable client requirement IDs, five operation elements, enum/data elements, and test paths; keep server-only and retired IDs unassigned. **Evidence:** `python tools/normative_catalog/validate.py` passed (`sources=4 clauses=1411 records=4029`); 15 `test_feature_traceability.py` tests passed with one existing skip; a direct reference audit resolved all K0021 paths and named tests.
- [x] T038 Regenerate and verify `specification/catalog/coverage-report.md` using `python tools/normative_catalog/report.py --write` and `--check`; validate with `python tools/normative_catalog/validate.py`. **Evidence:** `--write` generated the report; `--check` verified it current; validation passed (`sources=4 clauses=1411 records=4029`). The generated diff removes the six assigned requirements and 119 assigned elements from the unassigned sections while preserving server-only/retired inventory records.
- [x] T039 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`; resolve warnings without broad lint suppressions. **Evidence:** both commands passed in WSL Ubuntu 26.04 with Rust 1.94; no new lint allowances were added.
- [x] T040 Run focused and workspace tests, `cargo llvm-cov --workspace --all-features`, and confirm coverage thresholds in `AGENTS.md` for changed protocol/client code and the workspace. **Evidence:** terminal-created draft PR #78 CI run [38082396564](https://github.com/NeverWe1come/KMIPKit/actions/runs/38082396564) passed all coverage producers and the selected-scope coverage gate on Linux, Windows, and macOS. The gate reported changed Rust 1573/1620 (97.10%), TTLV 731/737 (99.19%), protocol 8347/8641 (96.60%), transport 3713/3902 (95.16%), FFI 2035/2137 (95.23%), and workspace 20121/20911 (96.22%); Java 92.24%, Python 98.72%, and JNI 90.93% also exceeded their gates. The Python script suite passed 255 tests with 26 environment-dependent skips. A fresh full Rust coverage export in WSL had earlier failed on a generated `trybuild` scratch object; that local tooling failure is superseded by the complete PR CI coverage run. The coverage-gate parser now accepts LLVM's grouped function-instantiation summary when file segments union distinct physical lines, with regression coverage preserving conservative summary-uncovered accounting.
- [x] T041 Run the pinned generator drift, dependency/license/supply-chain, parser/FFI, and security checks that apply to the changed modules; confirm no changes under `specification/oasis/kmip-2.1/upstream/`. **Evidence:** deterministic catalog/API/fixture generator checks, catalog validation/report, 1,411-candidate source audit, and immutable-source check passed; the diff changes no pinned upstream, FFI, or binding files. `pwsh -NoProfile -File .\scripts\Test-DependencyPolicy.ps1` passed with cargo-deny 0.20.2, verified root and fuzz workspaces against RustSec commit `7eebec69c352c7191b1f13eb95dd510eeca5d1de` (2026-10-09), ran the configured and negative-fixture checks, and confirmed both lockfiles unchanged. PR #78 CI run [38080294447](https://github.com/NeverWe1come/KMIPKit/actions/runs/38080294447) passed the parser regression suite, all platform jobs, C/JNI sanitizers, fuzz smoke, and dependency policy. The formal Codex Security diff scan completed against `db51b7a1c93edbe81f89a9faf7902aa543caec3f..b193c8d62bae9a40a8b3d81261012c65129a15ac` with complete coverage and zero findings (scan `4c4f4588-88dc-4486-95d6-b13a2ce7749f`). The qualified human security audit remains a separate release gate before 1.0.0.
- [x] T042 Update the traceability implementation/verification references and `specs/021-hash-mac-signature-operations/quickstart.md` with actual outputs and open gates; do not claim unavailable official fixtures passed.
- [x] T043 Obtain sequential independent QA and security reviews, address findings, rebase on current `release/1.0.0`, open/update a draft PR from the terminal to trigger PR-only CI, and verify Linux, Windows, and macOS checks. **Evidence:** sequential QA and security reviews completed; the PR base is current `release/1.0.0` at `db51b7a1c93edbe81f89a9faf7902aa543caec3f`; draft PR #78 was updated from the terminal. CI run [38082396564](https://github.com/NeverWe1come/KMIPKit/actions/runs/38082396564) completed successfully, including all Rust OS/toolchain matrices, the coverage gate, and the widened progress-test deadlines. The first macOS attempt of earlier run 38081026494 exposed the narrow test-only total deadline; that run's retry passed before this branch widened the test margins.

## Dependencies & Execution Order

### Phase dependencies

- **Phase 1**: Must establish the merged KMIPKIT-0019 shared types and current release base before any #21 implementation work.
- **Phase 2**: Shared execution and data contracts; blocks all operation stories.
- **Phase 3 (US1)**: Hash protocol request/response, client dispatch, and tests.
- **Phase 4 (US2)**: MAC and Sign; follows shared execution foundation. MAC and Sign protocol model tests can be authored separately, but shared `execute.rs` integration is serialized.
- **Phase 5 (US3)**: MAC Verify and Signature Verify; follows shared execution foundation and uses the Validity Indicator conflict contract.
- **Phase 6**: Depends on all three stories and complete Red/Green/Refactor evidence.

### User story dependencies

- **US1** depends on Phase 2 and KMIPKIT-0019 shared models.
- **US2** depends on Phase 2 and KMIPKIT-0019 shared models; it does not depend on Hash implementation.
- **US3** depends on Phase 2 and KMIPKIT-0019 shared models; it does not depend on Hash, MAC, or Sign implementation, but uses the shared enumeration and response view types.
- Keep one implementer active for this specification to avoid simultaneous edits to `execute.rs`, protocol exports, and coverage/traceability records. QA and security review remain sequential.

### Parallel opportunities

- Protocol Red test files for different operation stories can be authored independently after shared constructors/types are frozen, but this feature uses one active implementer for the shared execution boundary.
- After implementation is complete, English and Spanish guide drafts can proceed in parallel in their separate files; catalog and test references must be reconciled before review.
- Independent QA and security reviews must be performed one after the other, then fixes are applied before CI and PR update.

## Implementation Strategy

Implement shared contracts first, then complete each user story in strict Red, Green, Refactor development commits. Validate US1 (Hash) independently, then US2 (MAC/Sign), then US3 (verification). Finish with bilingual guides, catalog/generator checks, coverage/security evidence, cross-platform CI, and a terminal-created draft PR. Do not report formal OASIS conformance, profile support, or official fixture passes from unavailable fixtures.
