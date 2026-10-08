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

- [ ] T004 Verify the approved/merged KMIPKIT-0016 specification revision and base `feature/KMIPKIT-0016-attribute-operations` on the latest `release/1.0.0`; record the approved spec revision and release base commit in `specs/016-attribute-operations/traceability.md`.
- [ ] T005 Verify the release branch contains the approved KMIPKIT-0014 `AttributeEntry` implementation, the KMIPKIT-0013 transport implementation, and the KMIPKIT-0015 integration changes; record each dependency commit in `specs/016-attribute-operations/traceability.md` and stop implementation if any dependency is absent.
- [ ] T006 [P] Reconfirm the seven source sections and table numbers against `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` and record any corrected reference in `specs/016-attribute-operations/traceability.md`.

---

## Phase 2: Foundational

**Purpose**: Establish reusable attribute values, request fixtures, and requirement mapping before operation stories.

**Dependency gate**: Complete T005 successfully before any implementation. If KMIPKIT-0014 has not supplied `AttributeEntry`, do not duplicate or change its contract here; wait for its implementation to merge and rebase.

- [ ] T007 [FR-001/FR-002/FR-011/SC-001] Add a Red Table 150 conformance vector in `crates/kmipkit-protocol/tests/unit/vendor_attribute_tests.rs` and shared fixtures in `crates/kmipkit-protocol/tests/support/attribute_fixtures.rs`; assert the Vendor Attribute structure tag and ordered Vendor Identification (required Text String), Attribute Name (required Text String), and Attribute Value (required generic Item value) members from OASIS KMIP v2.1 §4.60 Table 150. The Notify exception is outside this client-initiated operation scope. Preserve the mapped `KMIPKIT-TEST-CN01-2-42` / `TC-I18N-3-21` assignment, but do not claim that official case passed while its XML fixture remains unavailable.
- [ ] T008 [P] Verify the approved design-time catalog mapping covers each applicable source clause and OASIS test case for KMIPKIT-0016 in `specs/016-attribute-operations/traceability.md`; stop if any mapping remains unassigned.
- [ ] T009 [P] Verify `feature_spec` is `KMIPKIT-0016` for the seven operation elements, the Vendor Attribute element and its three Table 150 members, and every applicable normative requirement and source-linked test case in `specification/catalog/kmip-2.1.json`; add source-backed element links where applicable, and defer implementation/verification paths until code/tests exist under T057 (FR-014).
- [ ] T010 Verify FR-001, FR-002, and FR-011 use the existing bounded generic TTLV and message/result types in `crates/kmipkit-protocol/src/attribute.rs` and `crates/kmipkit-protocol/src/lib.rs`; do not modify shared production models in this feature, and stop for a separately approved spec change if their contracts are insufficient.

**Checkpoint**: Dependency gate passes; attribute fixtures and catalog-to-requirement mappings are ready. Commit the foundational test vectors before operation implementation.

---

## Phase 2A: Source-backed mutation policy registry

**Purpose**: Make unconditional standard-attribute prohibitions explicit, reviewable, and reproducible before implementing mutation behavior.

- [ ] T011 [P] [FR-015] Add and run Red validator tests in `tools/normative_catalog/tests/test_validate.py` requiring every cataloged standard attribute to declare verbatim `source_initially_set_by`, `source_modifiable_by_client`, and `source_deletable_by_client` source-cell text, exact `Yes`/`No` `source_always_required`, an exact `source_policy_table` identifier, and explicit `source_operation_restrictions`/`source_conditional_rules` lists (empty when none apply) with source references for every entry. Require the separate Vendor Attribute element to declare one §4.60 `source_value_policies` entry with `source_refs`, `structure_table`, exact `source_text`, `value_predicate.member_element_id`, `value_predicate.equals`, `value_predicate.source_indicates_origin`, and `prohibited_client_operations`; reject missing or contradictory metadata without inventing a normative requirement ID for the prose.
- [ ] T012 [FR-015] Populate the canonical catalog's 62 named standard-attribute records with those policy fields transcribed from the pinned OASIS KMIP 2.1 §4 definitions; preserve all source text exactly, do not collapse conditional values into booleans, and retain exact second-table identifiers. Populate the separate §4.60 `source_value_policies` entry on the Vendor Attribute catalog element with its §4.60 source reference, Table 150 structure reference, verbatim rule prose, Vendor Identification `y` predicate, server-created source meaning, and all prohibited client actions; link the three Table 150 members and source-linked test case. Update the validator, run the focused tests Green, and do not edit upstream files.
- [ ] T013 [P] [FR-015/SC-008] Add and run Red tests for deterministic generation and `--check` drift detection of the internal attribute-policy lookup in `tools/normative_catalog/tests/test_generate_attribute_policy.py`; test every standard record, conditional-rule preservation, unknown names without generated entries, the separate Vendor Attribute `Vendor Identification=y` rule, and that other identifiers such as `x` do not match that prohibition; capture failing Red evidence.
- [ ] T014 [FR-015] Implement the pinned deterministic generator in `tools/normative_catalog/generate_attribute_policy.py`, generate `crates/kmipkit-protocol/src/generated/attribute_policy.rs` from the canonical catalog, and wire it as a private protocol module; preserve qualified source values without treating them as unconditional prohibitions and generate the separate §4.60 value-aware Vendor Attribute rule. Run the focused generator tests Green. Never hand-edit generated output.
- [ ] T015 [FR-015/SC-008] [Refactor] Add the generator `--check` step to repository CI, verify that repeated generation is byte-for-byte deterministic, rerun the generator and focused tests without changing behavior, and record Refactor evidence in a distinct commit with command results and exact source coverage in `specs/016-attribute-operations/traceability.md`.

**Checkpoint**: The catalog validates complete source-backed metadata for every standard attribute, and generated lookup output is current and deterministic. Mutation implementation cannot proceed if this gate fails.

## Phase 3: User Story 1 — Read object attributes (Priority: P1)

**Goal**: Call Get Attributes and Get Attribute List while preserving omission, repeated values, name order, and operation results.

**Independent Test**: A fake transport exchanges each read request once and returns vectors covering all-name reads, selected names, absent names, repeated instances, repeated returned names, and operation errors.

### Tests for User Story 1 — Red

- [ ] T016 [P] [US1] Add FR-006/FR-010/SC-002/SC-004 OASIS-table request/response and malformed-payload tests for Get Attributes in `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs` using §6.1.20 Tables 223–225.
- [ ] T017 [P] [US1] Add FR-007/FR-010/SC-002/SC-004 OASIS tests proving Table 226's UID-only request, §6.1.21 full-name behavior, Table 227's required response references/cardinality, repeated names, and malformed response rejection in `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs` using §6.1.21 Tables 226–228.
- [ ] T018 [P] [US1] Add FR-006/FR-007/FR-010/FR-012/SC-007 fake-transport execution tests for both read operations in `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`, including one-exchange behavior and unchanged Result Status, every applicable Result Reason from Tables 225 and 228, and Result Message.
- [ ] T019 [US1] Run `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs`, `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`; record the failing Red evidence in the implementation PR before adding production behavior.

### Implementation for User Story 1 — Green

- [ ] T020 [P] [US1] Implement typed Get Attributes request/response encoding and decoding for FR-001/FR-002/FR-006/SC-001 in `crates/kmipkit-protocol/src/get_attributes.rs`.
- [ ] T021 [P] [US1] Implement the UID-only Get Attribute List request, full-name operation result, and required one-or-more Attribute Reference response for FR-001/FR-002/FR-007/SC-001 in `crates/kmipkit-protocol/src/get_attribute_list.rs`.
- [ ] T022 [US1] Expose both operation models for FR-001/FR-002/FR-011 from `crates/kmipkit-protocol/src/lib.rs` while retaining unknown and repeated values in wire order.
- [ ] T023 [US1] Add fake-transport dispatch and response handling for FR-001/FR-006/FR-007/FR-010/FR-012 for both read operations in `crates/kmipkit-client/src/execute.rs`.
- [ ] T024 [US1] Run the focused tests in `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs`, `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`; confirm they pass and record Green evidence in a separate implementation commit.

### Refactor for User Story 1

- [ ] T025 [US1] [Refactor] Refactor shared read-operation conversion and typed execution without changing vectors or server-result behavior in `crates/kmipkit-protocol/src/get_attributes.rs`, `crates/kmipkit-protocol/src/get_attribute_list.rs`, and `crates/kmipkit-client/src/execute.rs`.
- [ ] T026 [US1] Re-run `crates/kmipkit-protocol/tests/unit/get_attributes_tests.rs`, `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_read_execution_tests.rs`; record Refactor evidence in a separate implementation commit.

**Checkpoint**: Both read operations pass independently, including absent and repeated-value cases.

---

## Phase 4: User Story 2 — Change object attributes (Priority: P1)

**Goal**: Express Add, Adjust, Delete, Modify, and Set as distinct requests; reject unconditional source-defined client prohibitions locally while leaving remote-state-dependent policy to the server.

**Independent Test**: Table-driven protocol and fake-transport tests prove each request shape, optional selector, caller value, and operation-specific server error is preserved.

### Tests for User Story 2 — Red

- [ ] T027 [P] [US2] Add FR-003/FR-010/SC-002/SC-003 Add Attribute request vectors, including exact caller-supplied request values, in `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs` using §6.1.2 Tables 167–169; cover server error preservation in T032.
- [ ] T028 [P] [US2] Add FR-004/FR-010/SC-002/SC-003 Adjust Attribute vectors for every assigned and extension Adjustment Type, reserved enum boundaries, omitted parameter defaults, absent-current-value defaults for numeric, interval, Boolean, and other types, and server operation errors in `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs` using §6.1.3 Tables 170–172 and §11.1 Tables 428–429.
- [ ] T029 [P] [US2] Add FR-005/FR-010/SC-002/SC-003 Delete Attribute vectors for Current Attribute selection, omitted Current Attribute, both selectors omitted, and unchanged request shape in `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs` using §6.1.13 Tables 202–204; cover server errors in T032.
- [ ] T030 [P] [US2] Add FR-008/FR-010/SC-002/SC-003 Modify Attribute vectors for exact Current/New values, omitted Current Attribute, multiple instances, and operation errors in `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs` using §6.1.34 Tables 265–267.
- [ ] T031 [P] [US2] Add FR-009/FR-010/SC-002/SC-003 Set Attribute vectors for New Attribute preservation, zero/single/multiple existing values, and operation errors in `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs` using §6.1.51 Tables 322–324.
- [ ] T032 [US2] Add FR-010/FR-012/FR-015/SC-003/SC-007/SC-008 fake-transport tests for five mutation operations in `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`: test New Attribute targets for Add/Set, both Current and New Attribute targets for Modify, Attribute Reference for Adjust, and supplied Current Attribute/Attribute Reference selectors for Delete. Assert one exchange for allowed requests; zero exchanges and payload-free `NotSent` for standard prohibitions, any Add/Modify New Attribute named Usage Limits (Count is required by §7.40 Table 392 and barred by §4.59), and inspectable Vendor Attribute values with Vendor Identification `y` in Add/Modify/Set/Delete. Include Modify's Current Attribute case and Delete's Current Attribute case. Assert one exchange and no local rejection based on the §4.60 `y` rule for Vendor Attribute values with Vendor Identification `x` or another value; assert no inference for Adjust or reference-only Delete, no local state mutation or retry, and unchanged Result Status/Reason/Message for server-authoritative outcomes. Unknown names and remote-state-dependent Usage Limits Total rules must not trigger a preflight exchange.
- [ ] T033 [US2] Run `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`; record the failing Red evidence in the implementation PR before adding production behavior.

### Implementation for User Story 2 — Green

- [ ] T034 [P] [US2] Implement Add Attribute request/response models for FR-001/FR-002/FR-003/SC-001/SC-003 in `crates/kmipkit-protocol/src/add_attribute.rs`.
- [ ] T035 [P] [US2] Implement Adjust Attribute request/response models and future Adjustment Type preservation for FR-001/FR-002/FR-004/FR-011/SC-001/SC-003/SC-005 in `crates/kmipkit-protocol/src/adjust_attribute.rs`.
- [ ] T036 [P] [US2] Implement Delete Attribute request/response models for FR-001/FR-002/FR-005/SC-001/SC-003 in `crates/kmipkit-protocol/src/delete_attribute.rs`.
- [ ] T037 [P] [US2] Implement Modify Attribute request/response models for FR-001/FR-002/FR-008/SC-001/SC-003 in `crates/kmipkit-protocol/src/modify_attribute.rs`.
- [ ] T038 [P] [US2] Implement Set Attribute request/response models for FR-001/FR-002/FR-009/SC-001/SC-003 in `crates/kmipkit-protocol/src/set_attribute.rs`.
- [ ] T039 [US2] Export the five mutation operation models for FR-001/FR-002 from `crates/kmipkit-protocol/src/lib.rs` without collapsing their distinct semantics.
- [ ] T040 [US2] Add fake-transport dispatch and typed result conversion for FR-001/FR-010/FR-012/FR-015 for the five mutation operations in `crates/kmipkit-client/src/execute.rs`; consult the generated standard and Vendor Attribute policies, reject unconditional prohibitions as payload-free `NotSent` before invoking transport, reject Add/Modify New Attribute `Usage Limits` and inspectable Vendor Attribute `Vendor Identification=y`, and do not infer vendor identifiers for Adjust/reference-only Delete or preflight unknown/state-dependent cases.
- [ ] T041 [US2] Run `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`; record Green evidence in a separate implementation commit.

### Refactor for User Story 2

- [ ] T042 [US2] [Refactor] Refactor shared mutation conversion and execution policy checks while preserving operation-specific field order/cardinality and source-backed rejection behavior in `crates/kmipkit-protocol/src/attribute.rs`, the five operation modules under `crates/kmipkit-protocol/src/`, and `crates/kmipkit-client/src/execute.rs`.
- [ ] T043 [US2] Re-run `crates/kmipkit-protocol/tests/unit/add_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/adjust_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/delete_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/modify_attribute_tests.rs`, `crates/kmipkit-protocol/tests/unit/set_attribute_tests.rs`, and `crates/kmipkit-client/tests/unit/attribute_mutation_execution_tests.rs`; record Refactor evidence in a separate implementation commit.

**Checkpoint**: All five mutation operations preserve caller intent and server result semantics.

---

## Phase 5: User Story 3 — Preserve attribute identity, values, and failures (Priority: P1)

**Goal**: Retain standard, vendor, repeated, and future attribute data losslessly, preserve delivery/result contracts, and redact sensitive values from diagnostics.

**Independent Test**: Generic TTLV and client tests demonstrate lossless round trips for unknown names/tags/values, unknown Adjustment Type values, order preservation, bounded malformed-response rejection, and redacted errors.

### Tests for User Story 3 — Red

- [ ] T044 [P] [US3] Add FR-011/SC-005 round-trip cases for unknown Attribute Names, allocated and §11.56 extension tags, repeated attributes, unknown Enumeration values, and valid Adjustment Type extensions in `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`; add negative Reserved-tag/enum cases.
- [ ] T045 [P] [US3] Add FR-013/SC-002 malformed and over-limit response cases for all seven operations, including received Reserved Tags rejected before generic `Item` construction under OASIS KMIP 2.1 Chapter 11/§11.56 allocation classification and accepted KMIPKit policy ADR-0011, in `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`.
- [ ] T046 [P] [US3] Add FR-013 redaction assertions for attribute values in `crates/kmipkit-protocol/tests/attribute_redaction.rs`.
- [ ] T047 [P] [US3] Add FR-012/SC-007 delivery-state, pending-result, and no-retry assertions across all seven operations in `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`.
- [ ] T048 [US3] Run `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`, `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`, `crates/kmipkit-protocol/tests/attribute_redaction.rs`, and `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`; record failing Red evidence in the implementation PR.

### Implementation for User Story 3 — Green

- [ ] T049 [US3] Preserve exact Attribute Names, complete tagged values, repeated entries, and wire order for FR-011 in `crates/kmipkit-protocol/src/attribute.rs`.
- [ ] T050 [US3] Preserve valid unknown Adjustment Type extension values and reject Reserved enum values without synthesizing tags for FR-004/FR-011 in `crates/kmipkit-protocol/src/adjust_attribute.rs`.
- [ ] T051 [US3] Route malformed/over-limit responses through existing bounded decoder and shared error handling for FR-013 in `crates/kmipkit-client/src/execute.rs`.
- [ ] T052 [US3] Ensure typed attribute Debug/error formatting redacts values for FR-013 in `crates/kmipkit-protocol/src/attribute.rs` and operation error conversions in `crates/kmipkit-client/src/execute.rs`.
- [ ] T053 [US3] Run the tests in `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`, `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`, `crates/kmipkit-protocol/tests/attribute_redaction.rs`, and `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`; record Green evidence in a separate implementation commit.

### Refactor for User Story 3

- [ ] T054 [US3] [Refactor] Refactor shared preservation, redaction, and client result conversion without reducing unknown-value coverage in `crates/kmipkit-protocol/src/attribute.rs` and `crates/kmipkit-client/src/execute.rs`.
- [ ] T055 [US3] Re-run `crates/kmipkit-protocol/tests/unit/attribute_roundtrip_tests.rs`, `crates/kmipkit-protocol/tests/unit/attribute_limits_tests.rs`, `crates/kmipkit-protocol/tests/attribute_redaction.rs`, and `crates/kmipkit-client/tests/unit/attribute_execution_contract_tests.rs`; record Refactor evidence in a separate implementation commit.

**Checkpoint**: All three user stories pass with no value loss, unbounded allocation, local state mutation, or secret-bearing diagnostics.

---

## Phase 6: Polish, Traceability, and Conformance

**Purpose**: Close documentation, catalog, generation, quality, compatibility, and review evidence.

- [ ] T056 [P] Update executable Rust examples and user documentation for all seven operations in `docs/guide/en/attribute-operations.md` and `docs/guide/es/attribute-operations.md`.
- [ ] T057 Complete FR-001–FR-015/SC-001–SC-008 requirement-to-OASIS-to-catalog-to-code-to-test mappings in `specs/016-attribute-operations/traceability.md` for SC-006.
- [ ] T058 Regenerate catalog coverage output with `python tools/normative_catalog/report.py --repo-root . --write` and commit generated output only from the pinned tool in `specification/catalog/coverage-report.md`.
- [ ] T059 Run `python tools/normative_catalog/report.py --repo-root . --check` and resolve any unmapped applicable clause before review.
- [ ] T060 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused protocol/client tests, and `cargo llvm-cov --workspace --all-features`; record results and coverage exclusions in `specs/016-attribute-operations/traceability.md`.
- [ ] T061 Verify compatibility and inspect the complete diff for generated-file provenance, unsafe code, raw-body/secret logging, and deviations from the seven-operation scope in `specs/016-attribute-operations/traceability.md`.
- [ ] T062 Rebase/update the feature branch from the latest `release/1.0.0`, resolve conflicts, and rerun all required checks on the synchronized tree; record the final base/head SHAs in `specs/016-attribute-operations/traceability.md`.
- [ ] T063 Open or update the draft PR from the synchronized feature branch using the terminal; include scope, reason, distinct Red/Green/Refactor evidence, verification results, risks, and known limitations.
- [ ] T064 Run the full supported-platform CI matrix defined in `.github/workflows/ci.yml` after T062 and attach final CI evidence to the draft PR.

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
- `AttributeEntry` contract is owned by KMIPKIT-0014; this feature may reuse it but must not silently redefine it.
- Implementation does not claim server support for every attribute or policy.
- The pinned OASIS upstream copies are immutable; only project-authored catalog, mapping, tests, and documentation may change.
