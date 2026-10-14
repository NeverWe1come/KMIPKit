# Tasks: Rustls Advisory Remediation

**Input**: Design documents from `/specs/045-rustls-advisory-remediation/`

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `clarifications.md`, and `quickstart.md`.

**Tests**: Required by the approved feature specification and repository TDD rules. Preserve distinct DCO-signed Red, Green, and Refactor commits.

## Phase 1: Setup

**Purpose**: Establish the release baseline and capture the affected dependency evidence.

- [x] T001 Confirm the feature branch starts at `origin/release/1.0.0` (`550a8648`), record the baseline Cargo manifests and lockfile state, and verify that current TLS features are `aws_lc_rs` and `std` in `Cargo.toml`.
- [x] T002 Run the current online dependency-policy command; record its fail-closed result identifying RUSTSEC-2026-0346 through -0349 on `rustls@0.23.45`, without changing the dependency or adding an exception.

## Phase 2: Foundational regression contract (Red)

**Purpose**: Prove the repository currently permits the affected pin before changing the dependency.

- [x] T003 Add `scripts/tests/test_rustls_advisory_pin.py` with a contract asserting that the workspace Rustls version is an exact stable 0.23.x pin at or above 0.23.46, and that the matching root lockfile entry has the same version. Run the focused test against 0.23.45 and retain its expected failure output.
- [x] T004 Commit only the failing regression contract and its Red evidence as a separate DCO-signed commit; do not alter Cargo manifests or lockfiles in this commit (`test(deps): expose patched rustls floor requirement`).

## Phase 3: User Story 1 — Use the patched Rustls release (Priority: P1)

**Goal**: Remove the four known Rustls advisory findings while preserving the current TLS backend and feature behavior.

**Independent Test**: The regression contract passes, Rustls resolves to the patched exact release, and fresh policy checks pass for both workspaces without exceptions.

### Implementation and tests for User Story 1 (TDD Green)

- [x] T005 Update the exact Rustls workspace pin to `=0.23.46` in `Cargo.toml`, use Cargo to update `Cargo.lock`, and verify no package version other than Rustls changed. Run the focused regression contract and relevant `kmipkit-transport` tests, verify the TLS feature set remains unchanged, then commit the dependency change separately with DCO sign-off (`feat(deps): update rustls to patched 0.23.46`).
- [ ] T006 Run `pwsh -File .\scripts\Test-DependencyPolicy.ps1` and require successful online RustSec refresh and passing root/fuzz checks without an exception. Record both workspace database revisions and commit the Green verification evidence separately with DCO sign-off.

### Refactor and traceability

- [ ] T007 Add the final advisory/test/policy evidence to `specs/045-rustls-advisory-remediation/implementation-evidence.md`, link FR-001 through FR-005 in `specification/compliance/requirements/KMIPKIT-0045.csv`, and update `docs/security/dependency-policy.md` only if needed to explain the concrete remediation. Keep behavioral and policy code unchanged; commit this documentation and traceability Refactor separately.

## Phase 4: Polish and independent review

- [ ] T008 Run the focused Python contract, transport tests, formatting, Clippy, workspace tests, coverage, documentation checks, deterministic generators, dependency-policy runner, and relevant script contracts; record actual results and any external environment limitation.
- [ ] T009 Obtain independent QA and security review of the changed test, manifest/lockfile diff, advisory evidence, and unchanged TLS features; resolve all actionable findings and repeat affected checks.
- [ ] T010 Update from the active `release/1.0.0` branch without force-pushing, rerun affected checks, and create a detailed terminal-created draft PR with DCO Red/Green/Refactor commit hashes, traceability, CI evidence, risks, and limitations. Leave it open for human review.

## Requirement-to-task coverage

| Requirement | Tasks |
|---|---|
| FR-001 | T002, T003, T005, T006 |
| FR-002 | T001, T003, T005, T008 |
| FR-003 | T002, T006, T008 |
| FR-004 | T001, T005, T008 |
| FR-005 | T003, T004, T006, T007, T009, T010 |
| SC-001 | T002, T005, T006 |
| SC-002 | T002, T006, T008 |
| SC-003 | T003, T005, T008, T009, T010 |
| SC-004 | T001, T005, T008 |
