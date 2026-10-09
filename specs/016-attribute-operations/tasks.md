# Tasks: KMIP 2.1 Attribute Operations

**Input**: Design documents from `specs/016-attribute-operations/`

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/public-rust.md`, `traceability.md`

**Tests**: Required. KMIPKit mandates strict Red, Green, Refactor TDD and tests are executable protocol specifications.

**Organization**: T001–T003 close the specification-PR preparation gate before approval. Implementation tasks T004–T064 are not executable until the specification PR is approved and merged and the dependency gate in Phase 2 passes.

## Specification PR Preparation Gate (before approval)

**Purpose**: Close the Phase B design-time assignment for this family in the canonical KMIPKIT-0002 catalog before asking for specification approval.

- [x] T001 Assign the seven operation elements, applicable shared attribute-structure requirement IDs, the §4.60 Vendor Attribute element and Table 150 members, and every source-linked OASIS test-case ID to `KMIPKIT-0016` in `specification/catalog/kmip-2.1.json`; record reviewed exclusions.
- [x] T002 Regenerate `specification/catalog/coverage-report.md` with `python tools/normative_catalog/report.py --repo-root . --write` and verify it with `python tools/normative_catalog/report.py --repo-root . --check`.
- [x] T003 Update the pre-approval mapping section in `specs/016-attribute-operations/traceability.md` with the assigned catalog IDs and confirm that no applicable operation/shared attribute requirement or explicitly linked OASIS test case remains unassigned.

**Checkpoint**: Complete this gate in the specification PR before requesting approval. This gate is the feature-level Phase B inventory closure; the project-wide operation-family count remains provisional until all applicable operation elements have feature assignments.

## Phase 1: Setup

**Purpose**: Confirm the branch is based on the release containing the approved prerequisites and establish the normative input set.

- [x] T004 Verify the approved/merged KMIPKIT-0016 specification revision and base `feature/KMIPKIT-0016-attribute-operations-implementation` on the latest `release/1.0.0`; record the approved spec revision and release base commit in `specs/016-attribute-operations/traceability.md`. Evidence: PR #58 merge `0fa129e1248e735343704fb662926a401585ed7c`; implementation branch created at release commit `04586ab9606521b304b0834aad85387cf149b196`.
- [x] T005 Verify the active release branch contains the merged KMIPKIT-0013 production transport and focused correction validating the OASIS `Response Message` root tag (`0x42007B`); record both commits in `specs/016-attribute-operations/traceability.md` and stop implementation if either is absent. Also verify the merged KMIPKIT-0014 `AttributeSet` implementation is available for the Get Attributes response contract. Record the Cosmian live smoke test in KMIPKIT-0015 PR #56 as deferred interoperability evidence, not a prerequisite; its response-root code fix must land separately and remains required. Evidence: PR #55 commit `0e50e4b9859532cf7a0832deb2d510a3af4563de`, PR #59 commit `ba6421152f739b21204b0792f9e7fc1e12234c29`, PR #60 commit `04586ab9606521b304b0834aad85387cf149b196`; focused transport and protocol tests pass.
- [x] T006 [P] Reconfirm the seven source sections and table numbers against `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` and record any corrected reference in `specs/016-attribute-operations/traceability.md`. Evidence: independent source review confirmed Tables 167–169, 170–172, 202–204, 223–225, 226–228, 265–267, and 322–324 under the assigned sections; §11.1 Tables 428–429 also match. No correction was needed.

---

## Phase 2: Foundational

**Purpose**: Establish reusable attribute values, request fixtures, and requirement mapping before operation stories.

**Dependency gate**: Complete T005 successfully before any implementation. Follow the OASIS shapes: Attribute Reference supports both name and tag forms; Current/New Attribute wrap one direct §4 generic TTLV Item; Get Attributes response reuses KMIPKIT-0014's ordered direct-item `AttributeSet`. Do not wait for the KMIPKIT-0015 Cosmian deployment/live smoke test in PR #56, but require the response-root correction before this feature starts.

- [x] T007 [FR-001/FR-002/FR-011/SC-001] Add a Red Table 150 conformance vector in `crates/kmipkit-protocol/tests/unit/vendor_attribute_tests.rs` and shared fixtures in `crates/kmipkit-protocol/tests/support/attribute_fixtures.rs`; assert the Vendor Attribute's required Vendor Identification and Attribute Name Text String members plus its required direct Attribute Value Item, in OASIS KMIP v2.1 §4.60 Table 150 order. The Notify exception is outside this client-initiated operation scope. Preserve the mapped `KMIPKIT-TEST-CN01-2-42` / `TC-I18N-3-21` assignment, but do not claim that official case passed while its XML fixture remains unavailable. Evidence: Red `9c6c88b` failed on the not-yet-added shared test-fixture module; the existing AttributeSet behavior was already present, so no product-behavior failure is claimed. Green fixture `cbcd7e3`; Refactor `1da895c`; independent-tag review correction `10a4087`. `cargo test -p kmipkit-protocol vendor_attribute_table_150_preserves_required_members_in_order --locked --offline` passed (1 test); a temporary change of the fixture's outer tag to the other allocated Attribute tag `0x0042_0001` made the independent literal expectation fail, then the restored fixture passed. `cargo test -p kmipkit-protocol --all-features --locked --offline`, package Clippy, `cargo fmt --all --check`, and `git diff --check` passed. Independent QA found and verified the expected tag values are now pinned separately from the fixture.
- [x] T008 [P] Verify the approved design-time catalog mapping covers each applicable source clause and OASIS test case for KMIPKIT-0016 in `specs/016-attribute-operations/traceability.md`; stop if any mapping remains unassigned. Evidence: all 30 applicable requirement IDs and the five explicitly linked test cases are reconciled in the traceability matrix; the unavailable CN01 fixture and absent direct cases for Add/Adjust/Delete are called out without inferred coverage.
- [x] T009 [P] Verify `feature_spec` is `KMIPKIT-0016` for the seven operation elements, the Vendor Attribute element and its three Table 150 members, and every applicable normative requirement and source-linked test case in `specification/catalog/kmip-2.1.json`; add source-backed element links where applicable, and defer implementation/verification paths until code/tests exist under T057 (FR-014). Evidence: catalog inspection found 11 assigned elements and 30 assigned applicable requirements; all five source-linked cases point to those assigned elements. Implementation/verification refs remain deferred until T057.
- [x] T010 Verify FR-001, FR-002, and FR-011 can use existing bounded generic TTLV `Item`, message/result, and KMIPKIT-0014 `AttributeSet` contracts for both Attribute Reference variants, Current/New direct-Item wrappers, and Get Attributes response values. This is a read-only contract check; later Red tests and implementation tasks add 0016 operation types under TDD. Do not change existing generic `Item`, message/result, or `AttributeSet` contracts; if any are insufficient, stop for a separately approved spec change. Evidence: `Item`/`Value`/`Structure` represent the structures; existing batch views retain request payload, response payload, and result fields; `AttributeSet` preserves ordered direct items. The client still has only Discover Versions in its closed typed request enum, so attribute operations remain the later implementation tasks' scope.

**Checkpoint**: Dependency gate passes; attribute fixtures and catalog-to-requirement mappings are ready. Commit the foundational test vectors before operation implementation.

---

## Phase 2A: Source-backed mutation policy registry

**Purpose**: Make unconditional standard-attribute prohibitions explicit, reviewable, and reproducible before implementing mutation behavior.

- [x] T011 [P] [FR-015] Add and run Red validator tests in `tools/normative_catalog/tests/test_validate.py` requiring every cataloged standard attribute to declare verbatim `source_initially_set_by`, `source_modifiable_by_client`, and `source_deletable_by_client` source-cell text, exact `Yes`/`No` `source_always_required`, an exact `source_policy_table` identifier, and explicit `source_operation_restrictions`/`source_conditional_rules` lists (empty when none apply) with source references for every entry. Require the separate Vendor Attribute element to declare one §4.60 `source_value_policies` entry with `source_refs`, `structure_table`, exact `source_text`, `value_predicate.member_element_id`, `value_predicate.equals`, `value_predicate.source_indicates_origin`, and `prohibited_client_operations`; reject missing or contradictory metadata without inventing a normative requirement ID for the prose. Evidence: signed Red commits `d754414`, `3481158`, `278f252`, and `5b57da5`; the focused 10-test command ran in 63.076s and failed with 16 expected assertions because metadata is absent and the current validator accepts missing, removed, contradictory, and unsupported entries. Independent QA found the final Red set source-backed and ready for T012. Qualified cells remain verbatim conditional rules; the §4.60 creation action remains a source verb and is not mapped to a KMIP operation in this catalog task.
- [x] T012 [FR-015] Populate the canonical catalog's 62 named standard-attribute records with those policy fields transcribed from the pinned OASIS KMIP 2.1 §4 definitions; preserve all source text exactly, do not collapse conditional values into booleans, and retain exact second-table identifiers. Populate the separate §4.60 `source_value_policies` entry on the Vendor Attribute catalog element with its §4.60 source reference, Table 150 structure reference, verbatim rule prose, Vendor Identification `y` predicate, server-created source meaning, and all prohibited client actions; link the three Table 150 members and source-linked test case. Update the validator, run the focused tests Green, and do not edit upstream files. Evidence: signed Green implementation commit `2c52e74`; exact focused suite passed 10/10 in 99.667s; full catalog suite passed 186 tests (7 Windows platform skips); complete catalog validator, coverage report check, Python compilation, and `git diff --check` passed. Independent review's P2 type-scope finding was demonstrated Red (`7a43eff`), corrected Green (`0b4f43f`), and confirmed addressed on re-review. Full commands and traceability are recorded under T012 in `traceability.md`.
- [x] T013 [P] [FR-015/SC-008] Add and run Red tests for deterministic generation and `--check` drift detection of the internal attribute-policy lookup in `tools/normative_catalog/tests/test_generate_attribute_policy.py`; test every tag-addressable standard record, preserve all 62 canonical source-policy records, keep §4.6 Certificate Attributes / Table 40 catalog-only because it has no §11.56 Tag, preserve conditional rules, omit unknown tags, avoid unmapped name-form inference, and distinguish the separate Vendor Attribute `y` rule from other identifiers such as `x`; capture failing Red evidence. Evidence: signed Red commits `b1a4c7e`, `f34bb96`, `5a3e211`, and source-driven correction `7fbba61`. The focused command ran 10 tests and reported 10 expected failures because the T014 generator module/script is not implemented. Final independent review approved the 61-tag/62-source-row contract, record-level assertions, catalog-only Certificate Attributes treatment, unmapped name-form behavior, separate Vendor record, and Red evidence. Full commands, source rationale, and limitation are recorded under T013 in `traceability.md`.
- [x] T014 [FR-015] Implement the pinned deterministic generator in `tools/normative_catalog/generate_attribute_policy.py`, generate `crates/kmipkit-protocol/src/generated/attribute_policy.rs` from the canonical catalog, and wire it as a private protocol module; emit exactly the 61 standard policies with assigned §11.56 Item/Tag values, preserve qualified source values without treating them as unconditional prohibitions, retain §4.6 Certificate Attributes / Table 40 in the source catalog without inventing a runtime key, and generate the separate §4.60 value-aware Vendor Attribute rule. Run the focused generator tests Green. Never hand-edit generated output. Evidence: signed Green commit `fca0728`; review-driven lint fix `829c2a7`. `python -B -m unittest -v tools.normative_catalog.tests.test_generate_attribute_policy` passed (10 tests); repeated `--write` output had SHA-256 `3EACAA65E0F3AF0758EB80F1609F709DD020EC427A810BD6FCD47678E4663DE9`; `--check`, malformed-catalog no-output probe, `cargo check -p kmipkit-protocol`, workspace fmt check, package Clippy, generated-file rustfmt, and `git diff --check` passed. Independent implementation review passed after replacing broad dead-code allowance with a scoped `#[expect]`. Security diff scan `a8f6f2f8-76f0-4c8b-9193-e52e88ca0da8` completed with zero findings and complete coverage of the generator and generated module wiring. Detailed evidence is recorded under T014 in `traceability.md`.
- [x] T015 [FR-015/SC-008] [Refactor] Add the generator `--check` step to repository CI, verify that repeated generation is byte-for-byte deterministic, rerun the generator and focused tests without changing behavior, and record Refactor evidence in a distinct commit with command results and exact source coverage in `specs/016-attribute-operations/traceability.md`. Evidence: Red contract-test commit `3c8503a` failed because the normative-inventory job lacked the generated-policy check; Green CI commit `592a924` adds the `--check` step, and the contract test passes. Refactor evidence and full commands/results are recorded under T015 in `traceability.md`.

**Checkpoint**: The catalog validates complete source-backed metadata for every standard attribute, and generated lookup output is current and deterministic. Mutation implementation cannot proceed if this gate fails.

## Phase 3: User Story 1 — Read object attributes (Priority: P1)

**Goal**: Call Get Attributes and Get Attribute List while preserving omission, repeated values, name order, and operation results.

**Independent Test**: A fake transport exchanges each read request once and returns vectors covering all-name reads, selected names, absent names, repeated instances, repeated returned names, and operation errors.

### Tests for User Story 1 — Red

- [x] T016 [P] [US1] Add FR-006/FR-010/SC-002/SC-004 OASIS-table request/response and malformed-payload tests for Get Attributes in `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs` using §6.1.20 Tables 223–225. Evidence: signed Red commits `69de061`, `ea45bfc`, and `efa4f38`; final independent protocol-vector review passed.
- [x] T017 [P] [US1] Add FR-007/FR-010/SC-002/SC-004 OASIS tests proving Table 226's UID-only request, §6.1.21 full-name behavior, Table 227's required response references/cardinality, repeated names, and malformed response rejection in `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs` using §6.1.21 Tables 226–228. Evidence: signed Red commits `d8005a3` and `fd507fd`; final independent protocol-vector review passed.
- [x] T018 [P] [US1] Add FR-006/FR-007/FR-010/FR-012/SC-007 fake-transport execution tests for both read operations in `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`, including one-exchange behavior and unchanged Result Status, every applicable Result Reason from Tables 225 and 228, and Result Message. Evidence: signed Red commits `ad52ae3` and `86dc6db`; the final independent execution-test review passed and is summarized in `traceability.md`.
- [x] T019 [US1] Run `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs`, `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`; record the failing Red evidence in the implementation PR before adding production behavior. Evidence: all three registered modules were run with the locked offline Cargo commands and failed only on the expected missing models/client variants; exact command summaries and the no-match source discrepancy are recorded in `traceability.md`.

### Implementation for User Story 1 — Green

- [x] T020 [P] [US1] Implement typed Get Attributes request/response encoding and decoding for FR-001/FR-002/FR-006/SC-001 in `crates/kmipkit-protocol/src/get_attributes.rs`; enforce shared Vendor Attribute Table 150 member order and reject Reserved/unallocated outbound tag-form references. Green: `547c3185e894932371daa13f321edf3ad631da40`.
- [x] T021 [P] [US1] Implement the UID-only Get Attribute List request, full-name operation result, and required one-or-more Attribute Reference response for FR-001/FR-002/FR-007/SC-001 in `crates/kmipkit-protocol/src/get_attribute_list.rs`. Green: `789215c39f16900e79a4c2ca6a04c1294198199f`.
- [x] T022 [US1] Expose both operation models for FR-001/FR-002/FR-011 from `crates/kmipkit-protocol/src/lib.rs` while retaining unknown and repeated values in wire order. Green: `75f2032fcc7adb7ad08f7759560fe65cec511d4b`; workspace formatting follow-up: `5438d85a80459a65b4967824767ecbee180359d1`.
- [x] T023 [US1] Add fake-transport dispatch and response handling for FR-001/FR-006/FR-007/FR-010/FR-012 for both read operations in `crates/kmipkit-client/src/execute.rs`. Red: `ca17623c7bb29305576e5021dbddb9ed23e98f0f`; Green: `08ba69ab0429748ac4fb538e0c0494804a2a68a9`.
- [x] T024 [US1] Run the focused tests in `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs`, `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`; confirm they pass and record Green evidence in a separate implementation commit. Evidence: `traceability.md`, “Attribute read-operation Green evidence (T020–T024)”.

### Refactor for User Story 1

- [x] T025 [US1] [Refactor] Refactor shared read-operation conversion and typed execution without changing vectors or server-result behavior in `crates/kmipkit-protocol/src/get_attributes.rs`, `crates/kmipkit-protocol/src/get_attribute_list.rs`, and `crates/kmipkit-client/src/execute.rs`. Refactor: `5932eab603a053f47ff5e61f5f376e2afdce1cc2`.
- [x] T026 [US1] Re-run `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs`, `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`; record Refactor evidence in a separate implementation commit. Evidence: `traceability.md`, “Attribute read-operation Refactor evidence (T025–T026)”.

**Checkpoint**: Both read operations pass independently, including absent and repeated-value cases.

---

## Phase 4: User Story 2 — Change object attributes (Priority: P1)

**Goal**: Express Add, Adjust, Delete, Modify, and Set as distinct requests; reject unconditional source-defined client prohibitions locally while leaving remote-state-dependent policy to the server.

**Independent Test**: Table-driven protocol and fake-transport tests prove each request shape, optional selector, caller value, and operation-specific server error is preserved.

### Tests for User Story 2 — Red

- [x] T027 [P] [US2] Add FR-003/FR-010/SC-002/SC-003 Add Attribute request vectors, including exact caller-supplied request values, in `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs` using §6.1.2 Tables 167–169; cover server error preservation in T032. Red: `66f392414ca9d2aa3e15a1687c4eb44934e63762`; independent test review passed.
- [x] T028 [P] [US2] Add FR-004/FR-010/SC-002/SC-003 Adjust Attribute vectors for every assigned and extension Adjustment Type, reserved enum boundaries, omitted parameter defaults, absent-current-value defaults for numeric, interval, Boolean, and other types, and server operation errors in `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs` using §6.1.3 Tables 170–172 and §11.1 Tables 428–429. Red: `2667ca6`; type-category coverage: `a25cf49`; full Table 428 matrix and corrected target references: `a4606f5`, `abd6c32`, `08653f3`; independent test review passed. See `traceability.md`, “Adjust Attribute request/response Red evidence (T028)”.
- [x] T029 [P] [US2] Add FR-005/FR-010/SC-002/SC-003 Delete Attribute vectors for Current Attribute selection, omitted Current Attribute, both selectors omitted, and unchanged request shape in `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs` using §6.1.13 Tables 202–204; cover server errors in T032. Red: `46e47a6208ea684be9ac3425a27ee3e4f9607be0`; independent test review passed. See `traceability.md`, “Delete Attribute request/response Red evidence (T029)”.
- [x] T030 [P] [US2] Add FR-008/FR-010/SC-002/SC-003 Modify Attribute vectors for exact Current/New values, omitted Current Attribute, multiple instances, and operation errors in `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs` using §6.1.34 Tables 265–267. Red: `72b49c54260bbe39484f4947c9c46f7e614fba2a`; required-UID negative and Table 267 errors: `e3f2e58cb254e9105eb0b71fb097ea29a997570d`; independent test review passed. See `traceability.md`, “Modify Attribute request/response Red evidence (T030)”.
- [x] T031 [P] [US2] Add FR-009/FR-010/SC-002/SC-003 Set Attribute vectors for New Attribute preservation, zero/single/multiple existing values, and operation errors in `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs` using §6.1.51 Tables 322–324. Red: `65e5b5eaa9b482393aebe841adfb52e85de7f081`; independent test review passed. See `traceability.md`, “Set Attribute request/response Red evidence (T031)”.
- [x] T032 [US2] Add FR-010/FR-012/FR-015/SC-003/SC-007/SC-008 fake-transport tests for five mutation operations in `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`: test New Attribute direct-Item tags for Add/Set, both Current/New direct Items for Modify, both name/tag Attribute Reference variants for Adjust and Delete, and supplied Current Attribute for Delete. Assert one exchange for allowed requests; zero exchanges and payload-free `NotSent` for standard prohibitions, Add/Modify New Attribute Usage Limits, and every inspectable Vendor Attribute `Vendor Identification=y` case in supplied value or name-form reference. Include name-form Vendor Attribute `y` reference tests for Adjust and reference-only Delete; those requests are inspectable and must be locally rejected. Assert one exchange and no local rejection for tag-form references whose vendor identifier is unavailable or for Vendor Attribute values/references with identifiers other than `y`; assert no local state mutation or retry, and unchanged Result Status/Reason/Message for server-authoritative outcomes. Unknown names/tags and remote-state-dependent rules must not trigger a preflight exchange. Red commits: `4e5373675331e3020ddef33e5c7dfc0c979a8ebe`, corrected by `f7f16de9b99b40360444b6c0743f4104173d126a`; independent QA passed. See the T032 Red evidence in `traceability.md`.
- [x] T033 [US2] Run `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`; record the failing Red evidence in the implementation PR before adding production behavior. Evidence: focused protocol and client test builds fail only because mutation models and client constructors are not implemented yet; see T033 Red evidence in `traceability.md`.

### Implementation for User Story 2 — Green

- [x] T034 [P] [US2] Implement Add Attribute request/response models for FR-001/FR-002/FR-003/SC-001/SC-003 in `crates/kmipkit-protocol/src/add_attribute.rs`.
- [x] T035 [P] [US2] Implement Adjust Attribute request/response models and future Adjustment Type preservation for FR-001/FR-002/FR-004/FR-011/SC-001/SC-003/SC-005 in `crates/kmipkit-protocol/src/adjust_attribute.rs`.
- [x] T036 [P] [US2] Implement Delete Attribute request/response models for FR-001/FR-002/FR-005/SC-001/SC-003 in `crates/kmipkit-protocol/src/delete_attribute.rs`.
- [x] T037 [P] [US2] Implement Modify Attribute request/response models for FR-001/FR-002/FR-008/SC-001/SC-003 in `crates/kmipkit-protocol/src/modify_attribute.rs`.
- [x] T038 [P] [US2] Implement Set Attribute request/response models for FR-001/FR-002/FR-009/SC-001/SC-003 in `crates/kmipkit-protocol/src/set_attribute.rs`.
- [x] T039 [US2] Export the five mutation operation models for FR-001/FR-002 from `crates/kmipkit-protocol/src/lib.rs` without collapsing their distinct semantics.
- [x] T040 [US2] Add fake-transport dispatch and typed result conversion for FR-001/FR-010/FR-012/FR-015 for the five mutation operations in `crates/kmipkit-client/src/execute.rs`; consult generated standard and Vendor Attribute policies, reject unconditional prohibitions as payload-free `NotSent` before invoking transport, reject Add/Modify New Attribute Usage Limits and any supplied Vendor Attribute value or name-form Attribute Reference with Vendor Identification `y`, and do not infer the identifier for tag-form references or preflight unknown/state-dependent cases.
- [x] T041 [US2] Run `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`; record Green evidence in a separate implementation commit.

### Refactor for User Story 2

- [x] T042 [US2] [Refactor] Refactor shared mutation conversion and execution policy checks while preserving operation-specific field order/cardinality and source-backed rejection behavior in `crates/kmipkit-protocol/src/attribute.rs`, the five operation modules under `crates/kmipkit-protocol/src/`, and `crates/kmipkit-client/src/execute.rs`.
- [x] T043 [US2] Re-run `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`; record Refactor evidence in a separate implementation commit.

**Checkpoint**: All five mutation operations preserve caller intent and server result semantics.

---

## Phase 5: User Story 3 — Preserve attribute identity, values, and failures (Priority: P1)

**Goal**: Retain standard, vendor, repeated, and future attribute data losslessly, preserve delivery/result contracts, and redact sensitive values from diagnostics.

**Independent Test**: Generic TTLV and client tests demonstrate lossless round trips for unknown names/tags/values, unknown Adjustment Type values, order preservation, bounded malformed-response rejection, and redacted errors.

### Tests for User Story 3 — Red

- [x] T044 [P] [US3] Add FR-011/SC-005 round-trip cases for unknown Attribute Names, allocated and §11.56 extension tags, repeated attributes, unknown Enumeration values, and valid Adjustment Type extensions in `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`; add negative Reserved-tag/enum cases.
- [x] T045 [P] [US3] Add FR-013/SC-002 malformed and over-limit response cases for all seven operations, including received Reserved Tags rejected before generic `Item` construction under OASIS KMIP 2.1 Chapter 11/§11.56 allocation classification and accepted KMIPKit policy ADR-0011, in `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`.
- [x] T046 [P] [US3] Add FR-013 redaction assertions for attribute values in `crates/kmipkit-protocol/tests/attribute_redaction.rs`.
- [x] T047 [P] [US3] Add FR-012/SC-007 delivery-state, pending-result, and no-retry assertions across all seven operations in `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`.
- [ ] T048 [US3] Run `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`, `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`, `crates/kmipkit-protocol/tests/attribute_redaction.rs`, and `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`; record failing Red evidence in the implementation PR. Tests passed on the unchanged pre-existing implementation, so the required Red evidence is missing; do not close without an approved no-change disposition or task revision.

### Implementation for User Story 3 — Green

- [x] T049 [US3] Preserve exact Attribute Names, complete tagged values, repeated entries, and wire order for FR-011 in `crates/kmipkit-protocol/src/attribute.rs`.
- [x] T050 [US3] Preserve valid unknown Adjustment Type extension values and reject Reserved enum values without synthesizing tags for FR-004/FR-011 in `crates/kmipkit-protocol/src/adjust_attribute.rs`.
- [x] T051 [US3] Route malformed/over-limit responses through existing bounded decoder and shared error handling for FR-013 in `crates/kmipkit-client/src/execute.rs`.
- [x] T052 [US3] Ensure typed attribute Debug/error formatting redacts values for FR-013 in `crates/kmipkit-protocol/src/attribute.rs` and operation error conversions in `crates/kmipkit-client/src/execute.rs`.
- [x] T053 [US3] Run the tests in `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`, `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`, `crates/kmipkit-protocol/tests/attribute_redaction.rs`, and `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`; record Green evidence in a separate implementation commit.

### Refactor for User Story 3

- [ ] T054 [US3] [Refactor] Refactor shared preservation, redaction, and client result conversion without reducing unknown-value coverage in `crates/kmipkit-protocol/src/attribute.rs` and `crates/kmipkit-client/src/execute.rs`. Review found the existing shared paths already centralized and no behavior refactor was warranted; the no-change disposition needs human acceptance before this task can close.
- [x] T055 [US3] Re-run `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`, `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`, `crates/kmipkit-protocol/tests/attribute_redaction.rs`, and `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`; record Refactor evidence in a separate implementation commit.

**Checkpoint**: All three user stories pass with no value loss, unbounded allocation, local state mutation, or secret-bearing diagnostics.

---

## Phase 6: Polish, Traceability, and Conformance

**Purpose**: Close documentation, catalog, generation, quality, compatibility, and review evidence.

- [x] T056 [P] Update executable Rust examples and user documentation for all seven operations in `docs/user-guide/en/attribute-operations.md` and `docs/user-guide/es/attribute-operations.md`. Evidence: the bilingual guides document every request shape, preservation/result behavior, delivery policy, and include a shared example; `cargo test -p kmipkit-protocol --test attribute_operations_guide_examples --all-features --locked --offline` passed (1 test). The repository's established `docs/user-guide/` path is used because the task's `docs/guide/` directories do not exist.
- [x] T057 Complete FR-001–FR-015/SC-001–SC-008 requirement-to-OASIS-to-catalog-to-code-to-test mappings in `specs/016-attribute-operations/traceability.md` for SC-006. Evidence: FR and success-criteria matrices are complete; all 11 KMIPKIT-0016 catalog elements and 30 requirements now carry implementation and verification paths; 206 catalog tests passed (7 platform skips).
- [x] T058 Regenerate catalog coverage output with `python tools/normative_catalog/report.py --repo-root . --write` and commit generated output only from the pinned tool in `specification/catalog/coverage-report.md`. Evidence: generator wrote report successfully after catalog references were added.
- [x] T059 Run `python tools/normative_catalog/report.py --repo-root . --check` and resolve any unmapped applicable clause before review. Evidence: `--check` passed; `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py' -v` passed 206 tests with seven platform skips.
- [x] T060 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused protocol/client tests, and `cargo llvm-cov --workspace --all-features`; record results and coverage exclusions in `specs/016-attribute-operations/traceability.md`.
- [x] T061 Verify compatibility and inspect the complete diff for generated-file provenance, unsafe code, raw-body/secret logging, and deviations from the seven-operation scope in `specs/016-attribute-operations/traceability.md`. Evidence: independent security and QA review findings and dispositions are recorded in the T061 section; the coverage follow-up review of `2b9d443` and release merge `124ab15` found no code findings, and the added changes are test-only plus the required release synchronization.
- [x] T062 Confirm the feature branch includes the latest `release/1.0.0`, resolve any conflicts, and rerun required checks on the synchronized code tree; record the base/head SHAs in `specs/016-attribute-operations/traceability.md`. Evidence: release SHA `3448c197b964a4b3b9374d364b3e0d8bcc414864` is the merge base of synchronized code HEAD `124ab151e3ff9bd2b7be84f7f3cccf7a1b3b734d`, and `git rev-list --left-right --count origin/release/1.0.0...HEAD` returned `0 98`. The release merge completed without conflicts. On this synchronized tree, workspace tests, formatting, and workspace Clippy passed; full post-sync CI is tracked by T064.
- [x] T063 Open or update the draft PR from the synchronized feature branch using the terminal; include scope, reason, distinct Red/Green/Refactor evidence, verification results, risks, and known limitations. Draft PR: [#62](https://github.com/NeverWe1come/KMIPKit/pull/62). It is explicitly marked draft and not ready for approval or merge while the source conflict and T048/T054 dispositions remain open.
- [x] T064 Run the full supported-platform CI matrix defined in `.github/workflows/ci.yml` after T062 and attach final CI evidence to the draft PR. Evidence: post-sync GitHub Actions run [`37932798811`](https://github.com/NeverWe1come/KMIPKit/actions/runs/37932798811) passed every applicable core, script-contract, language-binding, sanitizer, fuzz-smoke, inventory, adapter-coverage, and aggregate-coverage job. The aggregate gate passed changed Rust 96.47%, TTLV 99.18%, protocol 96.55%, transport 95.16%, FFI 95.23%, Java 92.24%, Python 98.72%, JNI 90.93%, and workspace 96.13%; see `traceability.md`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No code dependencies; establish source set and release base.
- **Foundational (Phase 2)**: Requires all dependency checks in T005 to pass; blocks all user-story implementation.
- **User Stories (Phases 3–5)**: Depend on Phase 2 and run sequentially: US2 starts after US1, and US3 starts after both. They share the attribute and client execution contracts. The source-backed mutation registry gate in T011–T015 must also pass before mutation implementation.
- **Polish (Phase 6)**: Depends on all user stories.

### User Story Dependencies

- **US1 (P1)**: Starts after Phase 2; validates read semantics and shared Attribute results.
- **US2 (P1)**: Starts after US1 completes because both stories modify shared protocol exports and client execution.
- **US3 (P1)**: Starts after US1 and US2 complete; validates cross-operation preservation, bounds, redaction, delivery state, and retry behavior.

### Within Each User Story

- Write and run tests first; preserve the failing Red evidence.
- Implement the smallest behavior required to pass; preserve Green evidence in a separate commit.
- Refactor only after Green; preserve Refactor evidence in a separate commit.
- Keep operation-specific errors, field cardinality, and payload ordering explicit.

### Parallel Opportunities

- T006, T008, and T009 work in independent documentation/catalog files after the base is confirmed.
- Within US1, protocol vector tests T016–T017 can proceed in parallel; T018 uses shared fake-transport support.
- Within US2, the five operation-specific protocol test modules T027–T031 and operation modules T034–T038 can proceed in parallel if files are assigned separately. Policy catalog/generator tasks T011–T015 precede these mutation tests and implementation. Client execution changes remain serialized because they share `execute.rs`.
- Within US3, T044–T047 touch independent test files.
- English and Spanish user guides in T056 may be drafted independently.
- Stories are serialized; parallelism is limited to distinct files within the active story.

## Parallel Example: User Story 2

```text
Task: T027 Add Add Attribute vectors in crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs
Task: T028 Add Adjust Attribute vectors in crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs
Task: T029 Add Delete Attribute vectors in crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs
Task: T030 Add Modify Attribute vectors in crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs
Task: T031 Add Set Attribute vectors in crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs
```

## Implementation Strategy

### MVP First

Complete the dependency gate, then User Story 1 as the first independently testable increment. Continue to User Story 2 only after read models and execution pass. Complete User Story 3 before the feature can be considered conformant or ready for review.

### Incremental Delivery

1. Complete Phase 1 and Phase 2; stop if any dependency is not on the active release branch.
2. Implement and validate read operations (US1).
3. Implement and validate distinct mutation operations (US2).
4. Validate shared losslessness, limits, redaction, delivery, and no-retry contracts (US3).
5. Complete traceability, generation, documentation, coverage, and platform CI (Phase 6).

### Commit Evidence

The implementation PR must contain distinct Red, Green, and Refactor commits for each user-story increment. Its description must identify the tests and commands run for each phase and summarize any platform reruns without claiming an unexplained intermittent failure is fixed.

## Notes

- Every task has an exact repository path and stable task ID.
- Attribute Reference supports both Table 161 forms, Current/New Attribute wrap direct Items, and the Get Attributes response reuses KMIPKIT-0014's `AttributeSet` without changing its contract.
- Implementation does not claim server support for every attribute or policy.
- The pinned OASIS upstream copies are immutable; only project-authored catalog, mapping, tests, and documentation may change.
