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
- [x] T015 Add failing parity tests for the selector and validation behavior in C, Java, and Python before manifest entries or generated output. Red evidence is committed in `8ccc9cef`, `9d8ab569`, `439a8ef0`, `0068a6b2`, and `4b5bd21f`; the focused binding and inventory tests failed on the missing declarations before implementation.
- [x] T016 Extend the public API manifest and run its pinned generator to produce equivalent binding surfaces; generated files were produced only by `tools/api_manifest/generate.py`. C public-header consumer compilation passed, the C ABI consumer suite passed (2), Java Gradle tests passed, Python Register-context and generated-manifest tests passed (8), and the complete API manifest/generator suite passed (253, 3 skipped). Manifest and generated outputs are included with the implementation commit.
- [x] T017 Align Python factory and selector paths with the accepted manifest, add explicit native-handle release parity (Rust drop / C release / Java-Python close), update the bilingual ownership guidance and compatibility inventory, and record per-binding evidence. No server-acceptance claim is made.

## Phase 6: Polish and cross-cutting concerns

- [x] T018 Update English and Spanish user and API documentation with the default, explicit crypto context, Table 293 requirements, wire behavior, error behavior, and server-policy limitation; add executable examples where supported. Both guides link to a shared compiled Rust example covering generic and managed contexts and document C, Java, and Python ownership transfer and release methods. The Opaque Data Type sample uses the extension range from §11.35/Table 469. The guide contract tests passed (3 tests), including exact source parity for both snippets, and `cargo run -p kmipkit-protocol --example register_opaque_context` passed.
- [x] T019 Add exact source-to-code-to-test links for both requirement IDs and applicable Table 293 rows; run the catalog traceability validator and all affected generated-artifact checks. `traceability.md` maps all eight scoped IDs to exact OASIS sections, implementation files, and positive/negative test symbols; its regression test verifies every cited source and test symbol exists. For §6.1.43/Table 291, negative evidence checks the constructor and private fields make omission of Object Type or Attributes unrepresentable. The traceability test, catalog validator (`4 sources, 1,411 clauses, 4,044 records`), generated report check, and full normative catalog suite (`222 tests, 7 skipped`) passed. A parallel report-check attempt once hit a source-read error; the validator and an isolated report check then passed.
- [ ] T020 Run protocol/client/binding test suites, Rust format, Clippy, workspace tests, coverage thresholds for changed code, dependency/security checks, and supported CI platform checks. Record each command and result in the PR. **Earlier verification (2026-10-13, predates the current binding changes):** Windows `cargo test -p kmipkit-protocol -p kmipkit-client --all-features`, `cargo fmt --all --check`, workspace Clippy, rustdoc, and the full workspace `cargo llvm-cov --workspace --all-features --summary-only --locked` passed; focused coverage is `register.rs 98.03%`, `kmipkit-protocol 96.17%`, and `kmipkit-ttlv 98.29%`. The raw Rust workspace line total is 89.91% without the separately collected C consumer report; the Linux C ABI consumer passed with 93.59% line coverage, and the official aggregate gate passed on this feature in PR #103 CI run [38405257343](https://github.com/NeverWe1come/KMIPKit/actions/runs/38405257343) across Linux, Windows, and macOS. Repeating the same coverage command on the release base measured 89.89%, showing this single-Windows raw figure predates this feature without establishing the official aggregate gate result; the Opaque Object context regression test covers the new Register context path, and the official aggregate workspace gate result is recorded below. WSL Ubuntu 26.04 with Rust 1.94 passed `cargo test --workspace --all-features`. The full normative catalog suite passed `222 tests (7 skipped)`, scripts suite `255 (26 skipped)`, API extension-fixture suite `16 (1 skipped)`, API manifest generator check, extension-fixture generator check, and `scripts/Test-DependencyPolicy.ps1` with fresh RustSec evidence. KMIPKIT-0043 was merged by PR #100 as `b499366b`; T015–T017 are complete locally on this branch. The latest C ABI consumer suite passed (2 tests), the public-header C consumer compiled with `-Wall -Wextra -Werror`, Java Gradle tests passed, focused Python tests passed (8), and the full API manifest/generator suite passed (253 tests, 3 skipped). The feature-specific binding audit has no missing or mismatched 0044 exports. The full audit still reports 92 Java paths and 109 Python paths assigned to the pre-existing 0043 inventory, plus an unresolved `kmipkit.errors.IntEnum` import. Current branch changes have not yet been pushed through CI; final feature CI remains pending.
**Coverage scope clarification (2026-10-13):** The 89.91% result above is a single-Windows raw `cargo llvm-cov` summary; it is not the repository's official 90% workspace gate. `docs/development/testing.md` requires the gate to aggregate Linux, Windows, and macOS LLVM reports. The same-base KMIPKIT-0043 PR #100 completed its official selected-scope coverage gate successfully in CI run [38401743573](https://github.com/NeverWe1come/KMIPKit/actions/runs/38401743573), but that is not evidence for this feature's changed lines. The official gate passed for this feature in PR #103 run [38405257343](https://github.com/NeverWe1come/KMIPKit/actions/runs/38405257343); rerun it against the current binding-parity branch before closing T020.

**Latest local verification (2026-10-13):** Stable and Rust 1.94 workspace Clippy pass with `--all-targets --all-features --locked -- -D warnings`; `cargo fmt --all --check`, `git diff --check`, `tools/api_manifest/generate.py --check` (12 outputs), shared Rust inventory check, and all 20 compiled guide examples pass. The regenerated Rust export audit reports 9,503 matched declarations, 22 exact approved exclusions, 9,525 observed exports, and zero mismatches or missing paths. Requirement traceability tests pass (2); the feature traceability suite passes (15, one platform-permission skip); the full API manifest suite passes (254 tests, 3 Windows symlink-permission skips). Final multi-platform PR CI is still pending, so T020 and T024 remain open.

- [ ] T021 Perform convergence, independent QA and security review, resolve findings, rebase/update from `release/1.0.0`, and update the draft PR with Red/Green/Refactor commits, risks, evidence, and remaining dependency notes. Convergence on 2026-10-13 found no additional implementation gaps; fresh independent QA and security reviews found no actionable 0044-specific findings. The release branch is already an ancestor of this feature branch. Remaining work is to publish the latest commits, record the fresh CI evidence, and update the PR summary. Draft PR #103 remains open.

## Dependencies & Execution Order

### Phase Dependencies

- Setup and foundational Red evidence precede story implementation.
- User Story 1 and User Story 2 are sequential because they share `register.rs` and its validator.
- Normative catalog inputs and generated traceability outputs change together.
- Binding work (T015–T017) began after KMIPKIT-0043's manifest was integrated into `release/1.0.0` as `b499366b`; it is not a reason to omit protocol tests or Rust implementation.
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

- [x] T022 Complete the C, Java, and Python context selector and validation parity in T015–T017; KMIPKIT-0043 is already integrated into `release/1.0.0`. The pinned generator check, C public-header consumer, C ABI suite, Java tests, Python focused parity tests, and complete API manifest suite pass. Feature-specific export comparison has zero missing paths or kind mismatches; remaining full-audit findings are the separate 0043 inventory backlog.
- [x] T023 Run and pass the official 90% Rust workspace coverage gate for this feature using the required Linux, Windows, and macOS reports. Single-host `cargo llvm-cov` summaries (89.89% on the release checkout and 89.91% here) are diagnostic only and do not establish that the aggregate gate fails. If the feature's official aggregate fails, preserve the threshold and scope any unrelated workspace remediation separately; do not exclude code or weaken the gate (passed in PR #103 CI run [38405257343](https://github.com/NeverWe1come/KMIPKit/actions/runs/38405257343); Constitution §6).
- [ ] T024 After T022 and T023 are complete, run the final feature CI matrix on Linux, Windows, and macOS, complete convergence plus independent QA/security review, update from `release/1.0.0`, and prepare the draft PR with evidence per T020–T021 (partial; convergence and independent reviews completed 2026-10-13 with no findings; latest commits and final CI remain pending).
