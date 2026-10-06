# Tasks: KMIP 2.1 Credentials and Attestation

**Input**: Design documents in `specs/008-credentials-attestation/`.<br>
**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/rust-credentials.md`, and `quickstart.md`.<br>
**Scope**: T001–T004 are complete. The remaining tasks implement and verify KMIPKIT-0008; no credential functionality is complete until those implementation and review tasks pass.

## Phase 1: Approval and dependency gates

**Purpose**: Resolve normative decisions and stabilize the exact shared contracts before any secret-bearing implementation.

- [x] T001 Initial independent QA assessed `checklists/requirements.md` against the earlier draft and recorded findings in `review-notes.md`. This is historical preliminary review; T004 records the final review of the refreshed exact revision.
- [x] T002 Record and reconcile each decision disposition in `spec.md` and `research.md`: OD-001 reflects the completed KMIPKIT-0002 T045–T047 catalog correction while preserving open DISC-041 and the absence of explicit catalog-owner sign-off; OD-002 adopts a conservative typed-value rule requiring at least one of the four §9.11 identifier members, while preserving all six Table 412 members and leaving actual uniqueness with the caller; OD-003 retains its temporal limit; OD-004 is resolved by the permanent in-memory scope; OD-005 is limited to future Authentication selection; and OD-006 follows the existing informative classification of `KMIPKIT-CLAUSE-SPEC-9.11-008`. The accepted catalog and generated report were inspected but not edited. Keep this feature's OD-006 distinct from KMIPKIT-0007 OD-006.
- [x] T003 Verify the accepted dependencies and record their exact release evidence: KMIPKIT-0005 PR #30 and ADR-0012, KMIPKIT-0006 PR #32 and opaque Authentication header model, and KMIPKIT-0007 PR #40 and its request/execution scope. Record that 0007 does not define credential selection, defaults, or precedence; its separate OD-006 and ADR-0012 require the first future secret-bearing sender to own the candidate callsite lifecycle test. KMIPKIT-0008 does not add a Credential writer or secret-bearing send path; it may only add the non-secret indicator to 0007's existing header builder. Evidence and artifact hashes are in `research.md`.
- [x] T004 Confirm this branch is based on `release/1.0.0` commit `ae87b89d43957e4fc028e785dc181e69b0165dac`; freeze the bounded interface: KMIPKIT-0008 adds no Client/ClientBatch Authentication selection API, no new writer/permit/transport callsite, and no Credential payload send path. It adds the capability indicator only to the existing `Client::execute` Request Header builder and must preserve the single-writer/single-permit boundary. Record the OASIS single Request Header / repeated Batch Item layout from §§8.1–8.3 (the header's message-wide Authentication scope is a structural inference), while keeping defaults, replacement, omission, and precedence under OD-005 for a future Authentication execution-integration feature. Apply OD-002's conservative rule requiring at least one of the four §9.11 identifier fields in typed Device values, preserve caller-owned actual uniqueness and the unspecified comparison scope, and retain the OD-003 timestamp-monotonicity exclusion. Reconcile `spec.md`, `contracts/rust-credentials.md`, `plan.md`, and `quickstart.md`; independent QA reviewed exact HEAD `7943d090c129482c022e1b9a5197ac2e3433aa3a` against all checklist items with no substantive blockers; record the delegated maintainer authorization before coding. The unimplemented Authentication execution handoff is not a prerequisite for standalone in-memory models; any future credential send path still needs its own approved scope and ADR-0012 candidate-callsite lifecycle test.

## Phase 2: Setup and shared model contracts

**Purpose**: Establish safe Rust model and test conventions after all Phase 1 gates pass.

- [x] T005 Review the existing test dependency before deciding whether KMIPKIT-0008 needs another one. The independent review confirmed QuickCheck 1.1.0 is already pinned, no direct dependency or manifest change is needed, and the test plan must account for its transitive `rand`/`getrandom` graph and non-portable `SmallRng` sequences. Evidence and references are in `dependency-review.md`.
- [x] T006 Add traceability to `specification/compliance/requirements/KMIPKIT-0008.csv`: all 11 catalog requirements assigned to KMIPKIT-0008 are present with exact clause IDs, scopes, implementation locations, and planned executable test paths; the existing 0006 Attestation Capable Indicator default is linked separately; server-only duties and deferred timestamp policy are classified; project policies, including exact Nonce byte preservation, are identified separately from OASIS rows.
- [x] T007 Add the credential module skeleton in `crates/kmipkit-protocol/src/credential/mod.rs` and public exports in `crates/kmipkit-protocol/src/lib.rs`; keep `#![forbid(unsafe_code)]` and add no production writer or transport path. Added as part of the first Green commit after the Authentication and Credential RED commits.

## Phase 3: User Story 1 — Build a typed Authentication value (P1)

**Goal**: Represent absent or non-empty Authentication, repeatable ordered Credential entries, each assigned variant, and unknown values losslessly.

**Independent Test**: External contract tests cover absent, present-empty rejection, one/multiple credentials, each named discriminator, unknown type, and exact opaque TTLV preservation.

### Tests — Red

- [x] T008 [US1] [RED COMMIT] Add Authentication absence/present-empty, repeat/order, and server-duty non-enforcement contract tests in `crates/kmipkit-protocol/tests/credential_contract.rs` (FR-001, FR-002; §9.4/Table 403). The focused Rust 1.94 test command fails at compile time exactly because the public `Authentication` and `Credential` APIs do not exist yet; evidence is recorded in `review-notes.md`.
- [x] T009 [US1] [RED COMMIT] Add generic-tree conversion/property tests for all seven Credential Type values, unknown raw values, Extensions, unknown children, and stable field order in `crates/kmipkit-protocol/tests/credential_roundtrip.rs` (FR-003, FR-004, FR-011, SC-002; §9.11/Tables 410–416, §11.11/Table 442). The focused Rust 1.94 command fails only because the public `Credential` API is not implemented yet; evidence is recorded in `review-notes.md`.

### Implementation — Green

- [x] T010 [US1] [GREEN COMMIT] Implement non-empty `Authentication` and the Credential discriminator/value model in `crates/kmipkit-protocol/src/credential/authentication.rs` and `value.rs` (FR-001–FR-003). Focused Authentication tests and Clippy pass; exact evidence is recorded in `review-notes.md`.
- [x] T011 [US1] [GREEN COMMIT] Implement typed conversion and raw unknown/opaque preservation in `crates/kmipkit-protocol/src/credential/conversion.rs` and `value.rs` (FR-003, FR-004, FR-011, SC-002).

### Refactor

- [x] T012 [US1] [REFACTOR COMMIT] Extract common ordered-structure validation and document public Authentication/Credential contracts in `crates/kmipkit-protocol/src/credential/validation.rs` and `crates/kmipkit-protocol/src/credential/mod.rs` (FR-014).

## Phase 4: User Story 2 — Construct credential variants safely (P1)

**Goal**: Validate known table-defined values, preserve Nonce/hash data, and protect sensitive values without local cryptographic behavior or unapproved transmission.

**Independent Test**: Table-driven positive/negative and secret sentinel tests cover all variants. Represent and preserve all six Device fields; typed Device values require at least one of the four §9.11 identifier members, while Password and Device Identifier are tested as supplementary fields and generic TTLV preserves unvalidated trees. Keep field presence distinct from text content and do not test uniqueness enforcement. Preserve caller Timestamp/hash bytes and effective SHA-256 default, but do not add hash calculation or monotonicity checking/tests pending OD-003 review. An execute-boundary capture test verifies the non-secret indicator is emitted and no Authentication/Credential payload is added.

### Tests — Red

- [x] T013 [US2] [RED COMMIT] Add failing tests for Username and Password, OTP, Ticket, and Device field types/requiredness in `crates/kmipkit-protocol/tests/credential_contract.rs`; reject a typed Device with none of the four §9.11 identifier members, preserve each of the six Table 412 fields, test Password and Device Identifier as supplementary fields alongside an identifier, and distinguish field presence from text length (FR-004, FR-005; Tables 411, 412, 414, 416). Ensure the public contract documents caller responsibility for actual uniqueness without inventing comparison scope or adding a client-side uniqueness test. Test generic TTLV preservation separately for unvalidated Device trees.
- [x] T014 [US2] [RED COMMIT] Add failing Hashed Password tests for required fields, algorithm absent/effective SHA-256/explicit/unknown values, and exact caller timestamp/hash-byte preservation in `crates/kmipkit-protocol/tests/credential_contract.rs`. Do not add monotonicity checking/tests until OD-003 review settles owner, comparison scope, and clock behavior (FR-007; §9.11/Table 415).
- [x] T015 [US2] [RED COMMIT] Add failing Attestation and Nonce tests for required Nonce/Type, neither/either/both evidence fields, and exact server Nonce byte preservation in `crates/kmipkit-protocol/tests/credential_contract.rs` (FR-006; §§9.11 and 9.14, Tables 413 and 419).
- [x] T016 [US2] [RED COMMIT] Add secret sentinel tests proving no credential, OTP, ticket, Nonce, or attestation value appears in Debug, Display, validation errors, or captured logs in `crates/kmipkit-protocol/tests/credential_redaction.rs`; include owned-memory lifecycle evidence only after the separately approved secret lifecycle contract is fixed (FR-010; secret-lifecycle dependency).

### Implementation — Green

- [x] T017 [US2] [GREEN COMMIT] Implement Username and Password, Device, OTP, Ticket, and opaque Extensions value types and table-derived structural validation in `crates/kmipkit-protocol/src/credential/variants.rs` (FR-003–FR-005, FR-011).
- [ ] T018 [US2] [GREEN COMMIT] Implement caller-supplied Hashed Password bytes/timestamp, raw algorithm preservation, and non-cryptographic effective SHA-256 default in `crates/kmipkit-protocol/src/credential/hashed_password.rs` (FR-007).
- [ ] T019 [US2] [GREEN COMMIT] Implement Attestation Credential and server-sourced Nonce types with raw field preservation in `crates/kmipkit-protocol/src/credential/attestation.rs` and `nonce.rs` (FR-006).
- [ ] T020 [US2] [GREEN COMMIT] Implement redacted secret wrappers and KMIPKit-owned zeroization using the reviewed ownership contract in `crates/kmipkit-protocol/src/credential/secret.rs`; add no production credential encode/send callsite in this feature (FR-010, FR-012).

### Refactor

- [ ] T021 [US2] [REFACTOR COMMIT] Consolidate common secret-safe formatting and validation errors across every variant in `crates/kmipkit-protocol/src/credential/secret.rs` and `validation.rs` while preserving all Red/Green behavior (FR-010, FR-014).

## Phase 5: User Story 3 — Report attestation construction capability (P1)

**Goal**: Emit the Attestation Capable Indicator truthfully from the exposed credential construction ability using the existing request-header builder.

**Independent Test**: A fake-transport `Client::execute` test captures the outbound header and verifies True when the released credential API can construct Attestation Credential. It asserts no Authentication or Credential payload is emitted. Existing protocol-view tests cover omitted/effective-false input behavior.

### Tests — Red

- [ ] T022 [US3] [RED COMMIT] Add a failing fake-transport request-capture test in `crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs`, register the test module in `crates/kmipkit-client/src/lib.rs`, and assert the existing outbound header advertises True, Authentication stays absent, and no Credential payload is emitted. Retain protocol tests for absent/effective-false parsed headers (FR-008, SC-004; §9.3/Table 402).

### Implementation — Green

- [ ] T023 [US3] [GREEN COMMIT] Add the Attestation Capable Indicator to the existing `build_request_message` in `crates/kmipkit-client/src/execute.rs`, derived from the public Attestation Credential constructor capability; do not add a writer, permit, Authentication selection, or Credential payload path (FR-008, SC-004).

### Refactor

- [ ] T024 [US3] [REFACTOR COMMIT] Document indicator semantics and ensure no caller configuration can contradict the shipped capability; preserve the existing one-writer/one-permit audit in `crates/kmipkit-client` (FR-008, FR-009, SC-004).

## Phase 6: Documentation, verification, and review

**Purpose**: Complete traceability and establish review evidence for a draft implementation PR.

- [ ] T025 Update `docs/architecture/public-api.md`, `docs/architecture/transport-security.md`, and English/Spanish user documentation with approved credential ownership, the effective SHA-256 default and optional-field absence, the caller's Device identifier uniqueness responsibility without an invented comparison scope, observable Attestation Capable Indicator=True behavior, redaction, and runtime zeroization limits (FR-005, FR-008–FR-010). Do not document KMIPKit Authentication selection/default/override behavior here; that remains under OD-005.
- [ ] T026 Add and run contract tests for the acceptance scenarios in `specs/008-credentials-attestation/quickstart.md`; construct credential/authentication values in memory, capture only the non-secret indicator in the fake execute path, and include no real secret fixtures (SC-001–SC-006).
- [ ] T027 Run focused protocol tests, `cargo fmt --all --check`, workspace Clippy/tests/docs, catalog validation/report regeneration, and immutable-source checks; demonstrate at least 95% changed/protocol code line coverage and at least 90% workspace coverage or document the exact blocking evidence (SC-001–SC-006).
- [ ] T028 Run independent QA and security reviews sequentially against spec, catalog, code, tests, traceability, and secret lifecycle; fix findings and record exact revision/commands. Confirm this feature introduced no production credential send path; any future send path must move to a separate approved feature with its own candidate-callsite lifecycle evidence (SC-003, SC-005, SC-006).
- [ ] T029 Rebase onto current `release/1.0.0`, run supported Linux/Windows/macOS CI, verify generated output is current, and create/verify a draft PR through terminal. Include distinct Red, Green, Refactor commits and their exact evidence.

## Dependencies and execution order

- **T001–T004** are sequential approval/dependency gates and block every code task.
- **T005–T007** establish reviewed test and API foundations after the gates pass.
- **User Story 1 (T008–T012)** precedes variant implementation because every variant uses its Credential and Authentication containers.
- **User Story 2 (T013–T021)** depends on the raw-preserving contracts from User Story 1; independent table cases can be authored in parallel after T008–T009, but each Green task follows its Red commit.
- **User Story 3 (T022–T024)** depends on the attestation type from T019 and the existing 0007 execute header builder. The 0006 parsed header view remains read-only.
- **T025–T029** require all stories and traceability to be complete.

## Parallel opportunities

After T001–T007 pass, separate test authors may prepare isolated Red test files for Username/Password/OTP/Ticket, Hashed Password, and Attestation/Nonce. Green work remains with one implementer while module interfaces are stabilized; QA and security reviews run in separate turns after implementation to avoid shared-edit conflicts.

## Implementation strategy

1. Keep the current PR documentation-only and leave every implementation checkbox unchecked.
2. After the spec gate, implement Authentication/Credential common models first, then variant models, then the Attestation Indicator in the existing request builder.
3. Use one responsible implementer for shared protocol modules; maintain distinct Red, Green, Refactor commits.
4. KMIPKIT-0008 never exposes secret-bearing transmission. Any future client feature that adds a send path must have separate approval and owner-through-transport lifecycle evidence.
5. Finish with generated traceability, docs, CI, coverage, and independent sequential QA/security review before opening an implementation draft PR.
