# Tasks: KMIP 2.1 Normative Inventory

**Input**: Design documents from `specs/002-normative-inventory/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/catalog-format.md`

**Tests**: Required by the feature scope and KMIPKit testing policy. Write failing validator/report tests before implementation and retain Red, Green, Refactor as distinct commits.

**Organization**: Tasks are grouped by the three independently reviewable inventory outcomes; all data changes are serialized because they share one canonical catalog.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel without editing the same files or depending on the same mutable catalog.
- **[Story]**: User story from `spec.md`.
- Every catalog and generated-report task cites source sections in the record data.

## Phase 1: Setup

**Purpose**: Establish the pinned evidence baseline and deterministic tooling layout.

- [x] T001 Record and independently verify the four pinned source IDs, stages, dates, local paths, and SHA-256 values from `specification/oasis/kmip-2.1/SOURCES.md` and `CHECKSUMS.sha256`.
- [x] T002 Create the catalog, validator, report, and unit-test paths specified in `plan.md`; do not modify or regenerate files under `specification/oasis/`.

## Phase 2: Foundational Contracts, Clause Audit, and Validation

**Purpose**: Define the catalog contract and completeness denominator before entering domain records.

### Red

- [x] T003 [P] Add failing tests for source metadata, source-clause ledger dispositions, `tag_ranges`, policy records, stable IDs, top-level record collections, and required source references in `tools/normative_catalog/tests/test_validate.py`.
- [ ] T004 [P] Add failing tests for duplicate JSON keys, unknown fields, invalid UTF-8, unpaired escaped surrogates, unresolved references, invalid scope/direction, checksum mismatch, absolute/traversal/symlink/reparse paths, JSON/HTML size-depth-node-record-string and global token/member limits, and immutable-base changes including add/delete/rename/mode/symlink changes in `tools/normative_catalog/tests/test_validate.py` and `test_immutable_sources.py`.
- [ ] T005 [P] Add failing report tests for unassigned coverage, source-count mismatch, absent fixture status, negative-test flags, SHOULD/SHOULD NOT/RECOMMENDED deviations without an accepted decision, malicious Markdown/HTML/link payloads (including parentheses, backslashes, line breaks, and image syntax), control characters, and locale-independent output in `tools/normative_catalog/tests/test_report.py`.
- [ ] T006 [P] Add failing source-audit tests for the documented case-insensitive token vocabulary and block boundaries, inline text split across HTML nodes, declared charset handling independent of locale, malformed/unsupported charset, stable structural IDs, limits, non-fetching parsing, exact candidate/ledger equality, and independently reviewed missed-block fixtures in `tools/normative_catalog/tests/test_audit_sources.py`.

### Green

- [x] T007 Define the checked-in record contract in `specification/catalog/README.md` and this feature contract; include `source_clauses`, `tag_ranges`, and `policies` in `specification/catalog/kmip-2.1.json`.
- [ ] T008 Implement `tools/normative_catalog/check_immutable_sources.py` first, requiring the exact PR base commit SHA and failing closed; run it before `audit_sources.py`. Compare the full pinned Git tree and manifest, then implement a non-fetching auditor that reads only allowlisted Git blobs, enforces HTML limits, and emits documented candidate locators. Wire this order into `.github/workflows/ci.yml` with `contents: read`, no secrets, and the PR event base SHA after rebasing on the CI foundation.
- [ ] T009 Implement strict offline parsing and semantic validation in `tools/normative_catalog/validate.py`, including a bounded preflight scan before object construction, duplicate-key detection, source/fixture Git-tree metadata lookup without following catalog paths, anchored IDs, relationships, source counts, and requirement-level rules; use Python standard library only.
- [x] T010 Implement `tools/normative_catalog/report.py` with separate table-text and link-label encoders, allowlisted constant link destinations, deterministic sort keys, fixed sections, and `--check`/`--write`; add a minimal valid catalog fixture and make the Red tests pass.

### Refactor

- [ ] T011 Refactor shared identifier, citation, source-integrity, and aggregate-count checks into small documented helpers; preserve all test behavior and byte-identical report output.

**Checkpoint**: Small source-audit fixtures prove the documented extraction rules and candidate/ledger set equality; structural validation rejects malformed data. Full-source equality is deferred until T022–T023 populate the ledger and T027 independently checks every source section/table. No inventory completeness claim is made until all catalog sections are populated.
## Phase 3: User Story 1 - Review Client Protocol Coverage (Priority: P1)

**Goal**: Represent every operation and protocol element, including direction and 1.0/1.1 disposition.

**Independent Test**: Compare catalog counts and source sections with Specification §§1–12; validator reports exact reconciliation for all named table categories.

### Red

- [ ] T012 [P] Add failing tests for 57 client-to-server and 5 server-to-client operation records, exact section references, direction, scope state, required request/response payload links, and async Poll/Cancel response classification in `tools/normative_catalog/tests/test_validate.py`.
- [ ] T013 [P] Add failing count, exact-value, and uniqueness tests for data types, objects, structures, every message field/nested member and credential form, attributes, every enumeration definition/value, every bitmask/defined bit value, options, results, and the 374 Table 487 single-value rows (354 non-reserved values and 20 reserved values) plus five range rows.

### Green

- [ ] T014 Populate operation records from Specification §§6.1–6.2, including all 57 §6.1 operation names, payload-section references, and the five §6.2 server-initiated operations marked for 1.1.
- [ ] T015 Populate all typed protocol elements and wire values from Specification §§1–5 and 10–12, including every enumeration value, bitmask bit, option, and result; distinguish reserved/unused/range/extension entries from usable named values.
- [ ] T016 Populate common messages, every message field and nested structure member, credential forms, options, result values, extensions, and protocol asynchronous behavior from Specification §§7–14; record client/server direction per field where required.
- [ ] T017 Link operation and protocol-element records to official Test Cases IDs only when the pinned HTML establishes the link; preserve the original case label and malformed link separately.
- [ ] T018 Make all operation, uniqueness, and source-count Red tests pass; do not alter the pinned OASIS sources.

### Refactor

- [ ] T019 Refactor catalog ordering and element relationships so the serialized data is deterministic and all shared structures retain explicit direction and source references.

**Checkpoint**: Every operation and named protocol-element category reconciles with the pinned sources and the report distinguishes 1.0 from 1.1.

## Phase 4: User Story 2 - Trace Normative Requirements and Evidence (Priority: P1)

**Goal**: Give every applicable client normative clause a stable requirement record linked to evidence and later implementation/test assignments.

**Independent Test**: The clause ledger and generated report demonstrate 100% classification of reviewed normative clauses and expose all unassigned records.

### Red

- [ ] T020 [P] Add failing tests requiring source-clause ledger IDs, exact source keyword, canonical strength mapping for all ten OASIS keywords (including REQUIRED and RECOMMENDED), section, role/direction, condition, scope, and verification/evidence fields for every normative requirement in `tools/normative_catalog/tests/test_validate.py`.
- [ ] T021 [P] Add failing tests that prohibit silent omissions, require negative-verification markers for prohibitions, require accepted decisions for deviations from SHOULD, SHOULD NOT, and RECOMMENDED requirements, and retain MAY/OPTIONAL capabilities.

### Green

- [ ] T022 Populate the normative clause ledger for applicable client obligations in Specification §§1–14, including shared TTLV/message rules, operations, versioning, security, conformance, and extension behavior.
- [ ] T023 Populate applicable normative additions and conditions from every client profile clause and profile conformance clause in Profiles §§2–6; separately mark XML/JSON and conditional domain profiles against the 1.0 boundary.
- [ ] T024 Populate all 110 Test Cases CN01 section identifiers and all 93 Profiles fixture references, including mandatory/optional status where explicit and fixture availability; do not infer undocumented fixture workflows.
- [ ] T025 Add stable record relationships between requirements, elements, profiles, and test evidence; keep unassigned feature, implementation, and verification references empty until later specifications implement them.
- [ ] T026 Make all normative-strength, source-coverage, negative-test, SHOULD-deviation, and relationship Red tests pass.

### Refactor

- [ ] T027 Independently review every Specification and Profiles section and table against the source HTML and emitted locator set; record section-by-section completion evidence, reconcile missed/duplicate candidates, conditional clauses, and paraphrase boundaries without copying long OASIS prose.
- [ ] T028 Refactor report sections to list unassigned records by requirement ID, normative level, scope, and source; retain stable sort order and byte-identical output.

**Checkpoint**: Every reviewed normative client clause maps to a stable record or a documented, reviewed exclusion; every evidence gap is visible.

## Phase 5: User Story 3 - Identify Profile Boundaries and Source Issues (Priority: P2)

**Goal**: Surface every client profile, profile-selection prerequisite, source inconsistency, and downstream blocker.

**Independent Test**: Compare profile and discrepancy records with Profiles §§3–6 and Specification §11.5; verify no unresolved issue is silently assigned a behavior.

### Red

- [ ] T029 [P] Add failing tests requiring all client profiles, conformance clause references, dependencies, test IDs, transport/encoding fields, applicability, and separate claim states in `tools/normative_catalog/tests/test_validate.py`.
- [ ] T030 [P] Add failing tests for the §11.5 Continue wording conflict, Profiles §5.3.1 HTTPS XML/JSON Content-Type versus binary TTLV body conflict, profile cross-reference defects, case-label/link discrepancies, and absent XML fixtures in `tools/normative_catalog/tests/test_validate.py`.

### Green

- [ ] T031 Populate all client and server profile records from Profiles §§5–6, tagging server-only profiles and keeping applicability, target selection, evidence completion, and conformance claim state distinct.
- [ ] T032 Add discrepancy records for Batch Error Continuation, Profiles §5.3.1 HTTPS encoding/content-type conflict, reserved-tag preservation policy, profile cross-references, profile case-label/link mismatches, malformed Test Cases links, and absent linked XML fixtures.
- [ ] T033 Add distinct project-policy records for unknown/future/vendor values and generate `specification/catalog/coverage-report.md` with count reconciliation, unassigned requirements, profile states, missing fixtures, open discrepancies, policy provenance, and source checksums.
- [ ] T034 Make all profile, discrepancy, fixture-state, and report tests pass; verify that open discrepancies appear as explicit blockers for dependent implementation without causing the inventory validator to choose an interpretation.

### Refactor

- [ ] T035 Cross-check each profile and discrepancy record against the pinned HTML, normalize labels only in separate display fields, and keep source IDs and citations unchanged.

**Checkpoint**: Every client profile and identified discrepancy has a traceable record; the catalog makes no unsupported compliance claim.

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Verify generation, documentation, security boundaries, and release readiness for the inventory PR.

- [ ] T036 Regenerate the Markdown report twice and verify byte-identical output, LF line endings, and a clean `--check` result.
- [ ] T037 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo llvm-cov --workspace --all-features`, the complete Python unit suite, source checksum verification, workflow checks, and `git diff --check`; verify no network calls or OASIS edits are present, and verify the CI diff guard rejects any source/manifest change against the exact PR base SHA.
- [ ] T038 Update `specification/catalog/README.md` and `docs/compliance/conformance.md` to explain the catalog/report, stable IDs, scope states, evidence gaps, and discrepancy lifecycle.
- [ ] T039 Run an independent QA review against every acceptance criterion and requirement; record check outputs and residual limitations in the draft PR.
- [ ] T040 Run an independent security review of file parsing, path handling, deterministic generation, dependencies, and untrusted catalog references; resolve actionable findings.
- [ ] T041 Prepare and push a draft PR from `feature/KMIPKIT-0002-normative-inventory`; include Red, Green, Refactor evidence, source counts, checksum results, coverage report, risks, and the explicit no-profile-claim limitation.

## Requirement-to-Task Traceability

| Requirement | Tasks |
|---|---|
| FR-001, FR-020 | T001, T008, T037, T039, T041 |
| FR-002, FR-006 | T012, T014, T016, T018, T019 |
| FR-003, FR-019 | T013, T015, T016, T019, T033 |
| FR-004, FR-021 | T003, T006, T007, T008, T020, T022, T027 |
| FR-005 | T020, T021, T022, T026 |
| FR-007, FR-008 | T023, T029, T031, T034, T035 |
| FR-009 | T017, T024, T025, T034 |
| FR-010 | T030, T032, T034, T035 |
| FR-011 | T003, T033 |
| FR-012 | T005, T028, T033 |
| FR-013, FR-018 | T002, T006, T007, T008, T009, T010, T011, T019, T036, T037 |
| FR-014 | T038 |
| FR-015, FR-016 | T002, T004, T007, T009, T040 |
| FR-017 | T004, T005, T007, T009, T010, T040 |
| FR-022 | T004, T006, T008, T040 |
| FR-001 through FR-022 (cross-cutting review and PR readiness) | T039, T041 |

## Success-Criterion-to-Task Traceability

| Success Criterion | Tasks |
|---|---|
| SC-001 | T012, T014, T018 |
| SC-002 | T013, T015, T016, T033 |
| SC-003 | T006, T020, T022, T027 |
| SC-004 | T017, T024, T029, T031, T032 |
| SC-005 | T005, T025, T028, T033 |
| SC-006 | T005, T010, T036 |
| SC-007 | T004, T005, T009, T010 |
| SC-008 | T004, T008, T037 |
| SC-009 | T001, T020, T022, T023, T027, T038 |

## Dependencies and Execution Order

### Phase dependencies

- Setup and foundational contracts precede domain catalog entry.
- User Story 1 creates protocol-element references consumed by Stories 2 and 3.
- User Story 2 creates requirement/evidence relationships consumed by Story 3 and the final report.
- User Story 3 completes the profile/discrepancy state used by final validation.
- Polish begins only after all source records are populated.

### User story dependencies

- **US1** depends on the validator contract and source manifest.
- **US2** depends on stable element, source, profile, and test identifiers from US1.
- **US3** depends on requirement/evidence records from US2.

Catalog data is a shared file, so domain story work is serialized under one implementer; tests for separate validator behaviors may be prepared in parallel only when they touch separate test modules.

## Red-Green-Refactor Commit Evidence

Create distinct signed commits for each cycle and list the failing/passing command in the PR:

1. Red commits add contract and focused behavior tests and record their expected failures.
2. Green commits add the minimum validator, catalog, or report behavior to pass the corresponding tests.
3. Refactor commits improve record organization and validation/report structure while rerunning the same tests.

Do not combine Red, Green, and Refactor into one commit.
