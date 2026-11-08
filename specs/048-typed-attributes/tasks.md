# KMIPKIT-0048 Tasks

## Phase 1: Specification and design

- [x] T001 Record maintainer authorization, normative scope, and explicit exclusions in `specs/048-typed-attributes/approval-record.md` and `specs/048-typed-attributes/spec.md`.
- [x] T002 Compare all 63 §4 catalog records to the pinned OASIS source and record 62 direct tagged attributes plus the Certificate Attributes exclusion in `specs/048-typed-attributes/gap-matrix.md`.
- [x] T003 Define API shape, source mapping, and verification strategy in `specs/048-typed-attributes/plan.md` and `specs/048-typed-attributes/traceability.md`.

## Phase 2: Red - failing contracts for both user stories

- [x] T004 [US1] Add failing tests for the 62 direct definitions, exact tags, encodings, multiplicity, and Certificate Attributes exclusion in `tools/normative_catalog/tests/test_generate_typed_attributes.py`, exercising `tools/normative_catalog/generate_attribute_types.py` (FR-001, FR-002, FR-008; Red commit).
- [x] T005 [US1] Add failing Rust tests for scalar, enum, byte-string, interval, Structure, RNG Parameters, and Unique Identifier typed values in `crates/kmipkit-protocol/tests/unit/typed_attribute_tests.rs` (FR-002, FR-005, FR-006; Red commit).
- [x] T006 [US1] Add failing tests for repeated-value order, lossless singleton-duplicate collection, operation-level cardinality rejection, unknown generic Items, and Structure child preservation in `crates/kmipkit-protocol/tests/unit/typed_attribute_tests.rs` (FR-003, FR-006, FR-007; Red commit).
- [x] T007 [US2] Add failing typed Current/New wrapper construction and tag/type mismatch tests in `crates/kmipkit-protocol/tests/unit/typed_attribute_tests.rs` (FR-004, FR-005; Red commit).

## Phase 3: Green - typed construction and reading

- [x] T008 [US1] Extend `tools/normative_catalog/generate_attribute_types.py` to deterministically render typed markers alongside existing TTLV type metadata into `crates/kmipkit-protocol/src/typed_attributes_generated.rs` (FR-001, FR-002, FR-008; Green commit).
- [x] T009 [US1] Add the sealed typed definition contract, owned attributes, borrowed callback views, and redacted conversion errors in `crates/kmipkit-protocol/src/typed_attributes.rs` (FR-002, FR-005, FR-006).
- [x] T010 [US1] Add typed insertion and ordered typed queries to `AttributeSet` in `crates/kmipkit-protocol/src/attribute.rs` without removing generic APIs (FR-003, FR-007).
- [x] T011 [US2] Add typed constructors and views to `CurrentAttribute` and `NewAttribute` in `crates/kmipkit-protocol/src/attribute.rs` (FR-004, FR-005; Green commit).
- [x] T012 [US2] Reconcile every new public route and exact exclusion in the reviewed API manifest and regenerate outputs with `tools/api_manifest/` tooling (FR-008).
- [x] T013 [US2] Add executable Rust examples and open-value/generic-fallback guidance to `docs/user-guide/en/attribute-operations.md` and `docs/user-guide/es/attribute-operations.md` (FR-009, FR-010).

## Phase 4: Refactor and verification

- [x] T014 Refactor the generator and shared conversion code without changing behavior; record distinct Red, Green, and Refactor commit evidence in `specs/048-typed-attributes/implementation-evidence.md` (FR-001–FR-010; Refactor commit).
- [x] T015 Run format, focused/workspace tests, Clippy, generator `--check`, manifest audit, and applicable coverage/security checks; record exact results in `specs/048-typed-attributes/implementation-evidence.md` (FR-001–FR-010).
- [x] T016 Push `feature/KMIPKIT-0048-typed-attributes` and open draft stacked PR #115 targeting `feature/KMIPKIT-0047-named-values`.

## Phase 5 audit correction: cross-version LinkNormalizer

- [x] T017 [Red] Add failing tests for the exact 14 KMIP 2.1 Link Type to KMIP 3.0 tag mappings, all 20 versioned Link tag allocations, identifier round trips, and rejection of unmapped fields/tags (FR-011; Red commits `a20fe669a`, `8ce6503b9`, `2a8dc8b7b`, and `587d9086a`).
- [x] T018 [Green] Add the checksum-pinned KMIP 3.0 Link tag catalog, regenerate the supported-version TTLV allocation union, and implement explicit lossless LinkNormalizer mappings (FR-011; Green commit `3296bc6bd`).
- [ ] T019 [Refactor] Correct KMIPKIT-0071 references and migration documentation, update this spec's traceability and evidence, then run focused, generator, formatting, lint, API, coverage, and hosted CI checks (FR-011).
- [x] T020 [Red] Require the exact 60 assigned KMIP 3.0 tags from §11.58, Table 605, including credential and KEM tags, and fail when the reviewed catalog contains only the 20 link tags (FR-012; Red commit `36abc5940`).
- [x] T021 [Green] Extend the versioned tag catalog and deterministic allocator to all exact Table 605 assignments without deriving assignments from ranges; regenerate the allocation union (FR-012; Green commit `0fd8ea58e`).
- [x] T022 [Refactor] Update KMIPKIT-0071 traceability, catalog documentation, and implementation evidence; run tag/API generation checks, focused tests, format, and lint (FR-012; Refactor commit `c68aa922b`).
- [ ] T023 [Verification] Pass the applicable coverage gates and all hosted CI checks on the final PR head (FR-012).

## Dependencies and independent criteria

T001–T003 define the approved contract before implementation. T004–T007
establish Red for both stories before any production implementation. T008–T010
deliver independently testable typed set construction and reading; T011 proves
operation wrapper use. T012–T014 reconcile public routes, documentation, and
Refactor evidence. T015–T016 close the stacked PR. No implementation task is
parallelized because the generator, Rust API, and manifest share one mapping
contract.

## Requirement-to-task coverage

| Requirement | Tasks |
|---|---|
| KMIPKIT-0048-FR-001 | T004, T008, T014, T015 |
| KMIPKIT-0048-FR-002 | T004, T005, T008, T009, T014, T015 |
| KMIPKIT-0048-FR-003 | T006, T010, T014, T015 |
| KMIPKIT-0048-FR-004 | T007, T011, T014, T015 |
| KMIPKIT-0048-FR-005 | T005, T007, T009, T011, T014, T015 |
| KMIPKIT-0048-FR-006 | T005, T006, T009, T014, T015 |
| KMIPKIT-0048-FR-007 | T006, T010, T014, T015 |
| KMIPKIT-0048-FR-008 | T004, T008, T012, T014, T015 |
| KMIPKIT-0048-FR-009 | T013, T014, T015 |
| KMIPKIT-0048-FR-010 | T013, T014, T015 |
| KMIPKIT-0048-FR-011 | T017, T018, T019 |
| KMIPKIT-0048-FR-012 | T020, T021, T022, T023 |
