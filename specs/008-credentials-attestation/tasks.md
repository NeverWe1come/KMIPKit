# Tasks: KMIP 2.1 Credentials and Attestation

**Input**: Design documents in `specs/008-credentials-attestation/`.<br>
**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/rust-credentials.md`, and `quickstart.md`.<br>
**Scope**: All implementation tasks remain blocked until T001–T004 are evidenced. This branch is documentation-only; it does not implement credential handling.

## Phase 1: Approval and dependency gates

**Purpose**: Resolve normative decisions and stabilize the exact shared contracts before any secret-bearing implementation.

- [x] T001 Independent QA assessed `checklists/requirements.md` against this draft `spec.md`, recorded findings in `review-notes.md`, and re-reviewed the corrections with no remaining findings. This is preliminary review only; final approval follows T002–T004 and all resulting edits.
- [ ] T002 Record this specification's decision dispositions in `spec.md` and `research.md`: OD-001 remains open for a separate catalog-owner workflow; OD-002 and OD-003 remain open with their validation/test limits; OD-004 is resolved by scope; OD-005 remains open for execution integration; and OD-006 is resolved by the existing informative classification of `KMIPKIT-CLAUSE-SPEC-9.11-008`. The separate OD-001 catalog workflow must track the lowercase §9.4 source keyword/classification correction and review evidence, the `KMIPKIT-REQ-SPEC-9.11-001` summary correction, and reviewed element links. Do not edit `specification/catalog/kmip-2.1.json` or generated output in this feature task. Keep this feature's OD-006 distinct from KMIPKIT-0007 OD-006.
- [ ] T003 Verify the exact accepted/merged KMIPKIT-0005 wire codec and secret lifecycle gates, KMIPKIT-0006 Authentication header and typed message model, and KMIPKIT-0007 request/execution handoff. Separately record the accepted KMIPKIT-0007 OD-006 disposition about future secret-bearing request lifecycle-test ownership under ADR-0012; it is not KMIPKIT-0008's OD-006. KMIPKIT-0008 is permanently in-memory and MUST NOT add a production credential writer or send path; any later send path requires a separate approved feature owning its candidate callsite and lifecycle test.
- [ ] T004 Rebase this branch from the active `release/1.0.0` after dependency merges and freeze the exact KMIPKIT-0007 execution handoff for inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch. This handoff gates execution integration only; standalone in-memory Credential and Authentication models do not need an execution API. Update this spec, contract, and tasks together if the boundary changes. After separate catalog and interface workflows, obtain independent review and human approval of the exact final spec revision and updated checklist before coding.

## Phase 2: Setup and shared model contracts

**Purpose**: Establish safe Rust model and test conventions after all Phase 1 gates pass.

- [ ] T005 Add a dependency review before introducing any new test dependency in `specs/008-credentials-attestation/dependency-review.md`; include license, maintenance/security history, MSRV, platforms, transitive footprint, and alternatives; obtain independent review before editing Cargo manifests.
- [ ] T006 Add traceability to `specification/compliance/requirements/KMIPKIT-0008.csv`: map every in-scope OASIS requirement to exact source clause/catalog IDs, code paths, and executable test paths; record project policies separately with their canonical decision/source and tests. Do not attribute project policies to OASIS. Explicitly exclude server duties and retain resolved OD dispositions.
- [ ] T007 Add the credential module skeleton in `crates/kmipkit-protocol/src/credential/mod.rs` and public exports in `crates/kmipkit-protocol/src/lib.rs`; keep `#![forbid(unsafe_code)]` and add no production writer or transport path.

## Phase 3: User Story 1 — Build a typed Authentication value (P1)

**Goal**: Represent absent or non-empty Authentication, repeatable ordered Credential entries, each assigned variant, and unknown values losslessly.

**Independent Test**: External contract tests cover absent, present-empty rejection, one/multiple credentials, each named discriminator, unknown type, and exact opaque TTLV preservation.

### Tests — Red

- [ ] T008 [US1] [RED COMMIT] Add failing Authentication absent/present-empty, repeat/order, and server-duty non-enforcement tests in `crates/kmipkit-protocol/tests/credential_contract.rs` (FR-001, FR-002; §9.4/Table 403).
- [ ] T009 [US1] [RED COMMIT] Add failing generic-tree conversion/property tests for all seven Credential Type values, unknown raw values, Extensions, unknown children, and stable field order in `crates/kmipkit-protocol/tests/credential_roundtrip.rs` (FR-003, FR-004, FR-011, SC-002; §9.11/Tables 410–416, §11.11/Table 442).

### Implementation — Green

- [ ] T010 [US1] [GREEN COMMIT] Implement non-empty `Authentication` and the Credential discriminator/value model in `crates/kmipkit-protocol/src/credential/authentication.rs` and `credential.rs` (FR-001–FR-003).
- [ ] T011 [US1] [GREEN COMMIT] Implement typed conversion and raw unknown/opaque preservation in `crates/kmipkit-protocol/src/credential/conversion.rs` and `credential.rs` (FR-003, FR-004, FR-011, SC-002).

### Refactor

- [ ] T012 [US1] [REFACTOR COMMIT] Extract common ordered-structure validation and document public Authentication/Credential contracts in `crates/kmipkit-protocol/src/credential/validation.rs` and `crates/kmipkit-protocol/src/credential/mod.rs` (FR-014).

## Phase 4: User Story 2 — Construct credential variants safely (P1)

**Goal**: Validate known table-defined values, preserve Nonce/hash data, and protect sensitive values without local cryptographic behavior or unapproved transmission.

**Independent Test**: Table-driven positive/negative and secret sentinel tests cover all variants. Represent and preserve every Device field, but gate only empty/minimum-field validation pending OD-002 review. Preserve caller Timestamp/hash bytes and effective SHA-256 default, but do not add hash calculation or monotonicity checking/tests pending OD-003 review. This feature remains in-memory and adds no production credential send path.

### Tests — Red

- [ ] T013 [US2] [RED COMMIT] Add failing tests for Username and Password, OTP, Ticket, and Device field types/requiredness in `crates/kmipkit-protocol/tests/credential_contract.rs`; gate Device empty/minimum-field assertions until OD-002 review settles the covered field set (FR-004, FR-005; Tables 411, 412, 414, 416).
- [ ] T014 [US2] [RED COMMIT] Add failing Hashed Password tests for required fields, algorithm absent/effective SHA-256/explicit/unknown values, and exact caller timestamp/hash-byte preservation in `crates/kmipkit-protocol/tests/credential_contract.rs`. Do not add monotonicity checking/tests until OD-003 review settles owner, comparison scope, and clock behavior (FR-007; §9.11/Table 415).
- [ ] T015 [US2] [RED COMMIT] Add failing Attestation and Nonce tests for required Nonce/Type, neither/either/both evidence fields, and exact server Nonce byte preservation in `crates/kmipkit-protocol/tests/credential_contract.rs` (FR-006; §§9.11 and 9.14, Tables 413 and 419).
- [ ] T016 [US2] [RED COMMIT] Add secret sentinel tests proving no credential, OTP, ticket, Nonce, or attestation value appears in Debug, Display, validation errors, or captured logs in `crates/kmipkit-protocol/tests/credential_redaction.rs`; include owned-memory lifecycle evidence only after the separately approved secret lifecycle contract is fixed (FR-010; secret-lifecycle dependency).

### Implementation — Green

- [ ] T017 [US2] [GREEN COMMIT] Implement Username and Password, Device, OTP, Ticket, and opaque Extensions value types and table-derived structural validation in `crates/kmipkit-protocol/src/credential/variants.rs` (FR-003–FR-005, FR-011).
- [ ] T018 [US2] [GREEN COMMIT] Implement caller-supplied Hashed Password bytes/timestamp, raw algorithm preservation, and non-cryptographic effective SHA-256 default in `crates/kmipkit-protocol/src/credential/hashed_password.rs` (FR-007).
- [ ] T019 [US2] [GREEN COMMIT] Implement Attestation Credential and server-sourced Nonce types with raw field preservation in `crates/kmipkit-protocol/src/credential/attestation.rs` and `nonce.rs` (FR-006).
- [ ] T020 [US2] [GREEN COMMIT] Implement redacted secret wrappers and KMIPKit-owned zeroization using the reviewed ownership contract in `crates/kmipkit-protocol/src/credential/secret.rs`; add no production credential encode/send callsite in this feature (FR-010, FR-012).

### Refactor

- [ ] T021 [US2] [REFACTOR COMMIT] Consolidate common secret-safe formatting and validation errors across every variant in `crates/kmipkit-protocol/src/credential/secret.rs` and `validation.rs` while preserving all Red/Green behavior (FR-010, FR-014).

## Phase 5: User Story 3 — Report attestation construction capability (P1)

**Goal**: Derive the request-header Attestation Capable Indicator truthfully from the exposed credential construction ability.

**Independent Test**: Header tests exercise true, false, and omitted/effective-false behavior based on the capability that the released typed builder actually provides.

### Tests — Red

- [ ] T022 [US3] [RED COMMIT] Add failing capability/indicator tests in `crates/kmipkit-protocol/tests/attestation_indicator.rs`, including absent default false and no evidence-generation claim (FR-008, SC-004; §9.3/Table 402).

### Implementation — Green

- [ ] T023 [US3] [GREEN COMMIT] Integrate the indicator with the accepted KMIPKIT-0006 header type in `crates/kmipkit-protocol/src/message/header.rs` and derive truth from the public Attestation Credential constructor capability (FR-008, SC-004).

### Refactor

- [ ] T024 [US3] [REFACTOR COMMIT] Document indicator semantics and prevent caller configuration from contradicting actual construction capability in `crates/kmipkit-protocol/src/message/header.rs` (FR-008, FR-009, SC-004).

## Phase 6: Documentation, verification, and review

**Purpose**: Complete traceability and establish review evidence for a draft implementation PR.

- [ ] T025 Update `docs/architecture/public-api.md`, `docs/architecture/transport-security.md`, and English/Spanish user documentation with approved credential ownership, default/override, redaction, and runtime zeroization limits (FR-009, FR-010, FR-013).
- [ ] T026 Add and run executable examples from `specs/008-credentials-attestation/quickstart.md`; ensure the examples construct in-memory values and include no real secret fixtures (SC-001–SC-006).
- [ ] T027 Run focused protocol tests, `cargo fmt --all --check`, workspace Clippy/tests/docs, catalog validation/report regeneration, and immutable-source checks; demonstrate at least 95% changed/protocol code line coverage and at least 90% workspace coverage or document the exact blocking evidence (SC-001–SC-006).
- [ ] T028 Run independent QA and security reviews sequentially against spec, catalog, code, tests, traceability, and secret lifecycle; fix findings and record exact revision/commands. Confirm this feature introduced no production credential send path; any future send path must move to a separate approved feature with its own candidate-callsite lifecycle evidence (SC-003, SC-005, SC-006).
- [ ] T029 Rebase onto current `release/1.0.0`, run supported Linux/Windows/macOS CI, verify generated output is current, and create/verify a draft PR through terminal. Include distinct Red, Green, Refactor commits and their exact evidence.

## Dependencies and execution order

- **T001–T004** are sequential approval/dependency gates and block every code task.
- **T005–T007** establish reviewed test and API foundations after the gates pass.
- **User Story 1 (T008–T012)** precedes variant implementation because every variant uses its Credential and Authentication containers.
- **User Story 2 (T013–T021)** depends on the raw-preserving contracts from User Story 1; independent table cases can be authored in parallel after T008–T009, but each Green task follows its Red commit.
- **User Story 3 (T022–T024)** depends on the attestation type from T019 and the accepted header interface from 0006.
- **T025–T029** require all stories and traceability to be complete.

## Parallel opportunities

After T001–T007 pass, separate test authors may prepare isolated Red test files for Username/Password/OTP/Ticket, Hashed Password, and Attestation/Nonce. Green work remains with one implementer while module interfaces are stabilized; QA and security reviews run in separate turns after implementation to avoid shared-edit conflicts.

## Implementation strategy

1. Keep the current PR documentation-only and leave every implementation checkbox unchecked.
2. After approvals/gates, implement Authentication/Credential common models first, then variant models, then Attestation Indicator integration.
3. Use one responsible implementer for shared protocol modules; maintain distinct Red, Green, Refactor commits.
4. KMIPKIT-0008 never exposes secret-bearing transmission. Any future client feature that adds a send path must have separate approval and owner-through-transport lifecycle evidence.
5. Finish with generated traceability, docs, CI, coverage, and independent sequential QA/security review before opening an implementation draft PR.
