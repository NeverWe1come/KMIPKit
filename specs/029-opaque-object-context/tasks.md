# Tasks: Explicit Opaque Object cryptographic context for Register

**Input**: Design documents from `/specs/029-opaque-object-context/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/register-opaque-context.md

**Tests**: Required by spec and project TDD rules. Each Red, Green, and Refactor stage is committed separately with DCO sign-off.

**Organization**: Tasks are grouped by user story; foundational catalog work and cross-cutting binding parity are sequenced around the release dependencies.

## Phase 1: Setup

**Purpose**: Confirm code-generation and traceability paths before behavior changes.

- [x] T001 Identify the pinned catalog validation/generation commands and record them in the PR description; inspect `specification/catalog/kmip-2.1.json` and its generated reports without editing generated files.
- [x] T002 Confirm current manifest dependency status for KMIPKIT-0043 and identify the pinned generator entry points for C, Java, and Python. Rust core work may proceed; binding generation must wait until the manifest contract is available on `release/1.0.0`.

## Phase 2: Foundational tests

**Purpose**: Establish failing source-to-requirement evidence for both catalog IDs before implementation.

- [x] T003 Add a failing catalog validator test proving KMIPKIT-REQ-SPEC-2.3-002 and KMIPKIT-REQ-SPEC-2.3-003 are assigned to this spec and link to the correct Table 4 structure-member elements. Assert that those two elements record `source_requiredness: Yes`, the Enumeration/Byte String encodings, and their §2.3 source references; require the 2.3-003 summary to name both child fields rather than claim the enclosing Opaque Object row is required. Preserve the clause-audit locator for the Table 4 `REQUIRED` header row. Require related existing Register IDs 6.1.43-001, 6.1.43-007, 6.1.43-009-001/002, and 6.1.43-010-001/002 to retain their existing owner while linking the new Opaque Object implementation and verification references.
- [x] T004 Run the focused catalog test and commit the failing Red evidence separately with DCO sign-off.

**Checkpoint**: Focused tests fail only for the missing implementation/traceability behavior; no production or generated source changes yet.

## Phase 3: User Story 1 — Register a generic Opaque Object (Priority: P1)

**Goal**: Preserve generic opaque behavior while requiring the valid Table 4 structure.

**Independent Test**: A default-context Register request with no crypto attributes and valid Table 4 child fields serializes unchanged; absent or mistyped Table 4 children fail locally before sending.

### Tests for User Story 1 (TDD Red)

- [x] T005 Add failing protocol tests in `crates/kmipkit-protocol/tests/unit/register_operation_tests.rs` for the Opaque Object tag and Structure value, required Opaque Data Type (Enumeration), required Opaque Data Value (Byte String), wrong child types, and generic-context acceptance without crypto attributes.
- [x] T006 Add failing preservation tests for unknown Opaque Data Type values, unknown Opaque Object children, and ordered caller attributes; run the focused protocol tests and commit Red evidence separately with DCO sign-off.

### Implementation for User Story 1 (TDD Green and Refactor)

- [x] T007 Implement the Table 4 Opaque Object child validation in `crates/kmipkit-protocol/src/register.rs`, preserving unknown values and caller order without inspecting Opaque Data Value; run focused protocol tests and commit Green evidence separately.
- [x] T008 Refactor Register validation into small helpers, retain sanitized errors and existing behavior for other Object Types, run focused tests plus fmt/Clippy for changed code, and commit Refactor evidence separately.

**Checkpoint**: Generic Opaque Object requests remain free of crypto-attribute requirements and valid Table 4 values are preserved.

## Phase 4: User Story 2 — Register an Opaque Object as managed cryptographic material (Priority: P1)

**Goal**: Make the §2.3 context choice explicit and apply the applicable Table 293 rules before transport.

**Independent Test**: Valid selected-context requests preserve the caller's direct attributes; each missing attribute, partial Algorithm/Length pair, or incompatible Object Type fails with zero transport writes and NotSent.

### Tests for User Story 2 (TDD Red)

- [x] T009 Add failing Register model tests for default Generic context, explicit Managed Cryptographic Object context, selector rejection for non-Opaque Object Types, KMIPKIT-REQ-SPEC-6.1.43-007/009/010 coverage: required Cryptographic Usage Mask, required Algorithm and Length, the Key Block omission boundary, and pair consistency in `crates/kmipkit-protocol/tests/unit/register_operation_tests.rs`.
- [x] T010 Add failing client fake-transport tests in `crates/kmipkit-client/tests/` proving invalid selected-context requests report NotSent and produce zero writes; verify valid selected-context requests send the unchanged Table 291 payload without a context marker. Run the focused tests and commit Red evidence separately with DCO sign-off. Independent scoped review confirmed exact TTLV tags, types, values, and order; only the expected absent API symbols fail compilation. A follow-up regression test also verifies that selecting even the generic Opaque Object context for a Certificate fails with Validation/NotSent and zero writes; a mutation check that bypassed the context/type guard failed at the expected validation assertion.

### Implementation for User Story 2 (TDD Green and Refactor)

- [x] T011 Add the explicit context type and defaulted selector to the Register request model in `crates/kmipkit-protocol/src/register.rs`; apply Table 293 only to Object Type Opaque Object when the caller selects Managed Cryptographic Object context. Add reusable non-allocating local-shape validation and invoke it in the client preflight (`crates/kmipkit-client/src/execute.rs`) so caller-invalid requests surface as Validation/InvalidInput before transport, consistent with the client contract. Do not infer Key Block data from Opaque Data Value. Run focused protocol and client tests and commit Green evidence separately. Commit `3d169156` passed independent review; one rustdoc wording nit is assigned to T012.
- [x] T012 Refactor shared Table 293 checks without changing other object-type behavior; ensure errors remain payload-free and validation occurs before transport execution. Run focused tests, fmt, and Clippy and commit Refactor evidence separately. Commit `7a97fcda` passed independent review with no findings.

**Checkpoint**: Both stories pass independently, no new KMIP wire field exists, and local errors have NotSent delivery evidence.

## Phase 5: Normative catalog and binding parity

**Purpose**: Close requirement traceability and expose equivalent 1.0 capability.

- [x] T013 Update project-authored catalog inputs to assign requirements 2.3-002 and 2.3-003 to this spec, set reviewed client direction and status, link 2.3-003 to the two required Table 4 structure-member elements, correct its summary, and add Opaque Object implementation/verification references to those elements and the existing Register requirements without changing their owner. Keep the source-clause locator aligned with the pinned keyword audit's `REQUIRED` header row. Leave upstream OASIS files unchanged. Commit `1df20461` passed independent source/catalog review with no findings.
- [x] T014 Regenerate the coverage report and all affected deterministic traceability artifacts with the pinned repository tool; verify regeneration produces no subsequent diff. `report.py --write` followed by `--check` passed twice; catalog validation passed (4 sources, 1,411 clauses, 4,044 records), and all 19 report tests passed (one expected skip). The focused catalog regression passed in isolation after a transient Windows safe-path lock failure during an earlier parallel run. Generated output committed as `d32e67a3`.
- [ ] T015 After KMIPKIT-0043's public API manifest is integrated into the release branch, add failing parity tests for the selector and validation behavior in C, Java, and Python before adding manifest entries or generated output.
- [ ] T016 Extend the public API manifest and run its pinned generator to produce equivalent binding surfaces; do not edit generated files by hand. Run generated C consumer, Java JNI, and Python CFFI parity tests; commit the manifest and generated changes together.
- [ ] T017 Refactor public names and errors to match the accepted manifest, update compatibility checks, and record each binding's test evidence without claiming unsupported server acceptance.

## Phase 6: Polish and cross-cutting concerns

- [x] T018 Update English and Spanish user and API documentation with the default, explicit crypto context, Table 293 requirements, wire behavior, error behavior, and server-policy limitation; add executable examples where supported. Both guides link to a shared compiled Rust example covering generic and managed contexts. The Opaque Data Type sample uses the extension range from §11.35/Table 469. The guide contract tests passed (3 tests), including exact source parity for both snippets, and `cargo run -p kmipkit-protocol --example register_opaque_context` passed.
- [x] T019 Add exact source-to-code-to-test links for both requirement IDs and applicable Table 293 rows; run the catalog traceability validator and all affected generated-artifact checks. `traceability.md` maps all eight scoped IDs to exact OASIS sections, implementation files, and positive/negative test symbols; its regression test verifies every cited source and test symbol exists. For §6.1.43/Table 291, negative evidence checks the constructor and private fields make omission of Object Type or Attributes unrepresentable. The traceability test, catalog validator (`4 sources, 1,411 clauses, 4,044 records`), generated report check, and full normative catalog suite (`222 tests, 7 skipped`) passed. A parallel report-check attempt once hit a source-read error; the validator and an isolated report check then passed.
- [ ] T020 Run protocol/client/binding test suites, Rust format, Clippy, workspace tests, coverage thresholds for changed code, dependency/security checks, and supported CI platform checks. Record each command and result in the PR. **Partial verification (2026-10-13):** Windows `cargo test -p kmipkit-protocol -p kmipkit-client --all-features`, `cargo fmt --all --check`, workspace Clippy, rustdoc, and the full workspace `cargo llvm-cov --workspace --all-features --summary-only --locked` passed; focused coverage is `register.rs 98.03%`, `kmipkit-protocol 96.17%`, and `kmipkit-ttlv 98.29%`. The raw Rust workspace line total is 89.91% without the separately collected C consumer report; the Linux C ABI consumer passed with 93.59% line coverage, and the official aggregate gate passed on this feature in PR #103 CI run [38405257343](https://github.com/NeverWe1come/KMIPKit/actions/runs/38405257343) across Linux, Windows, and macOS. Repeating the same coverage command on the release base measured 89.89%, showing this single-Windows raw figure predates this feature without establishing the official aggregate gate result; the Opaque Object context regression test covers the new Register context path, and the official aggregate workspace gate result is recorded below. WSL Ubuntu 26.04 with Rust 1.94 passed `cargo test --workspace --all-features`. The full normative catalog suite passed `222 tests (7 skipped)`, scripts suite `255 (26 skipped)`, API extension-fixture suite `16 (1 skipped)`, API manifest generator check, extension-fixture generator check, and `scripts/Test-DependencyPolicy.ps1` with fresh RustSec evidence. T015–T017 and binding parity tests remain gated on open, unmerged PR #100; its reported CI checks are green, but the manifest is not yet available on `release/1.0.0`. The latest focused client/protocol suites, formatter, Clippy, feature-guide contract tests, and feature traceability validators passed. Final feature CI remains pending.
**Coverage scope clarification (2026-10-13):** The 89.91% result above is a single-Windows raw `cargo llvm-cov` summary; it is not the repository's official 90% workspace gate. `docs/development/testing.md` requires the gate to aggregate Linux, Windows, and macOS LLVM reports. The same-base KMIPKIT-0043 PR #100 completed its official selected-scope coverage gate successfully in CI run [38401743573](https://github.com/NeverWe1come/KMIPKit/actions/runs/38401743573), but that is not evidence for this feature's changed lines. The official gate passed for this feature in PR #103 run [38405257343](https://github.com/NeverWe1come/KMIPKit/actions/runs/38405257343); rerun it after binding parity before closing T020.

- [x] T021 Perform convergence, independent QA and security review, resolve findings, rebase/update from `release/1.0.0`, and open or update a draft PR with Red/Green/Refactor commits, risks, evidence, and remaining dependency notes. Do not merge or publish. Evidence: convergence added T022-T024; independent QA and security reviews completed and findings resolved; branch is based on current release/1.0.0 at 1487f70; draft PR #103 is open.

## Dependencies & Execution Order

### Phase Dependencies

- Setup and foundational Red evidence precede story implementation.
- User Story 1 and User Story 2 are sequential because they share `register.rs` and its validator.
- Normative catalog inputs and generated traceability outputs change together.
- Binding work (T015–T017) depends on KMIPKIT-0043's accepted manifest being available on the release branch; it is not a reason to omit protocol tests or Rust implementation.
- Polish depends on both stories and traceability/binding completion.

### User Story Dependencies

- **US1 (P1)**: Begins after T001–T004; validates generic Opaque Object structure and preserves default behavior.
- **US2 (P1)**: Begins after US1; adds the explicit context and conditional Table 293 checks.
- **Language parity**: Follows both stories and the KMIPKIT-0043 manifest integration.

### Parallel Opportunities

- No implementation tasks are marked parallel because protocol and binding contract changes share traceability and public API decisions. Independent reviewers may inspect different artifacts after the implementation commits are available.

## Implementation Strategy

1. Establish Red evidence in test-only commits.
2. Deliver generic Opaque Object Table 4 behavior and validate it independently.
3. Add explicit Managed Cryptographic Object context and conditional Table 293 validation.
4. Correct the project-authored catalog inputs and regenerate traceability.
5. Add manifest-driven language parity once its dependency is present.
6. Complete docs, coverage, CI, security/QA review, convergence, and draft PR preparation.

## Phase 7: Convergence

- [ ] T022 Complete the C, Java, and Python context selector and validation parity in T015–T017 after KMIPKIT-0043's manifest is integrated into `release/1.0.0`; regenerate outputs only through the pinned generator and run each binding's parity suite per FR-007 and SC-004 (partial).
- [x] T023 Run and pass the official 90% Rust workspace coverage gate for this feature using the required Linux, Windows, and macOS reports. Single-host `cargo llvm-cov` summaries (89.89% on the release checkout and 89.91% here) are diagnostic only and do not establish that the aggregate gate fails. If the feature's official aggregate fails, preserve the threshold and scope any unrelated workspace remediation separately; do not exclude code or weaken the gate (passed in PR #103 CI run [38405257343](https://github.com/NeverWe1come/KMIPKit/actions/runs/38405257343); Constitution §6).
- [ ] T024 After T022 and T023 are complete, run the final feature CI matrix on Linux, Windows, and macOS, complete convergence plus independent QA/security review, update from `release/1.0.0`, and prepare the draft PR with evidence per T020–T021 (partial; plan: final validation and 1.0 acceptance).
