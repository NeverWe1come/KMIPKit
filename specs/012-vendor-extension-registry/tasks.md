---
description: "Dependency-ordered TDD tasks for the KMIP 2.1 vendor extension registry"
---

# Tasks: KMIP 2.1 Vendor Extension Registry

**Input**: Design documents from specs/012-vendor-extension-registry/

**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Required by the feature specification, repository constitution, and KMIPKit TDD policy. Write tests first and prove the Red, Green, and Refactor stages with distinct commits.

**Organization**: Tasks are grouped by the three P1 user stories. Foundation and manifest tooling precede story work. Do not begin feature implementation until this specification is reviewed and integrated according to repository governance.

## Phase 1: Setup

**Purpose**: Establish the reviewed manifest inputs, generator test harness, and shared fixtures.

- [x] T001 [P] Add failing manifest-generator unit tests for deterministic output, check mode, malformed manifests, unknown required fields, unsafe paths, and symlink destinations in tools/api_manifest/tests/test_generate.py
- [x] T002 Add failing manifest-format validation cases and a valid minimal registry fixture, capture the expected failing run, and commit the generator Red stage in tools/api_manifest/tests/test_generate.py and tools/api_manifest/tests/fixtures/registry-manifest.json
- [x] T003 Define and correct the versioned registry manifest format and JSON Schema, including open/preserve-raw and closed/reject-unsupported numeric enum policies, in specification/api/public-api.schema.json and specs/012-vendor-extension-registry/contracts/public-api-manifest.md
- [x] T004 Add the registry-only public API manifest with requirement IDs, all eleven TTLV Item Types and their declared unsupported-value policy, signatures grounded in existing model methods or named bounded protocol wrappers, cross-language integer/Boolean validation and C handle errors, all ExtensionRegistryLimits defaults/hard maxima, ownership, redaction, and generated destinations in specification/api/public-api.json
- [ ] T005 Implement the Python 3.12 standard-library manifest generator and deterministic check mode in tools/api_manifest/generate.py
- [ ] T006 Run generator tests and check mode, preserve the passing evidence, and commit the generator Green stage in tools/api_manifest/tests/test_generate.py and tools/api_manifest/generate.py
- [ ] T007 Refactor the generator without output changes, rerun tests and check mode, and commit the generator Refactor stage in tools/api_manifest/generate.py
- [ ] T008 [P] Add shared valid, invalid, multiply-matching, unknown, and secret-bearing extension fixtures plus their schema and stable outcome encoding in tests/fixtures/extensions/; the multiply-matching payload contains two distinct discriminator paths and passes both definitions' schemas

## Phase 2: Foundational protocol and registry contracts

**Purpose**: Build and test the bounded schema/value model and immutable per-client registry primitives required by all stories.

- [ ] T009 [P] Write failing protocol tests for stable extension error categories, path-only redacted errors, and bounded resource-limit rejection in crates/kmipkit-protocol/tests/extension_error.rs
- [ ] T010 [P] Write failing client tests for immutable snapshots, per-client isolation, deterministic metadata views, and registration-order independence in crates/kmipkit-client/tests/extension_registry.rs
- [ ] T011 Commit the foundational protocol and client test-only Red stage with failing test output recorded in crates/kmipkit-protocol/tests/extension_error.rs and crates/kmipkit-client/tests/extension_registry.rs
- [ ] T012 Implement the protocol extension modules, fallible constructors, sealed validated value, Extension Information model, and public exports in crates/kmipkit-protocol/src/extension/
- [ ] T013 Implement the client-owned immutable registry, exact discriminator index, deterministic metadata views, and client configuration attachment in crates/kmipkit-client/src/extension_registry.rs
- [ ] T014 Implement complete schema validation with configured TTLV byte/depth/element limits and all schema-related ExtensionRegistryLimits; compile tag and sorted enum indexes plus order edges, count constraint members before clone/reserve, reject cyclic/duplicate/self order edges, and return path-only redacted limit errors in crates/kmipkit-protocol/src/extension/schema.rs
- [ ] T015 Pass all foundational tests and commit the Green stage with command/output evidence recorded in crates/kmipkit-protocol/tests/extension_error.rs and crates/kmipkit-client/tests/extension_registry.rs
- [ ] T016 Refactor protocol and registry internals without changing public behavior; retain forbid(unsafe_code) outside kmipkit-ffi and commit the Refactor stage in crates/kmipkit-protocol/src/extension/ and crates/kmipkit-client/src/extension_registry.rs

## Phase 3: User Story 1 - Register and construct a known extension (Priority: P1)

**Goal**: Let each client configure immutable vendor definitions and construct only schema-validated extension values.

**Independent Test**: Build isolated clients with valid and invalid definitions; verify deterministic metadata, stable error categories, schema-validated values, and that no generic Item or raw-body route enters typed execution.

### Tests for User Story 1

- [ ] T017 [P] [US1] Write failing protocol/client tests for Vendor Identification syntax, extension identity, compatibility ranges, duplicate identities/discriminators, conflicting definitions, definition-count limits, identity/Extension Information per-field and aggregate text-byte limits, discriminator scalar/per-registry bytes, and limit configuration below defaults plus above-default values only where default < hard maximum; reject hard maximum plus one for every field and count before clone/reserve in crates/kmipkit-protocol/tests/extension_definition.rs and crates/kmipkit-client/tests/extension_registry.rs
- [ ] T018 [P] [US1] Write failing schema tests for every declared node type, cardinality, order rule, length/range/enum/bitmask constraints, undeclared-child policy, malformed definitions, duplicate/self/cyclic order edges, aggregate-node/Structure-width/nesting/path-depth limits, per-rule and aggregate constraint-member limits, and checked-counter overflow at default/configured/hard boundaries; instrument worst-case repeated enum and ordered fields to assert <=13 tag comparisons per input Item, <=13 enum comparisons per enum Item, and one order-edge pass in crates/kmipkit-protocol/tests/extension_schema.rs
- [ ] T019 [P] [US1] Write failing protocol/client tests for all required and optional Extension Information fields from §7.13 Table 365, deterministic registry-local list/map representation that does not claim remote-server support, and no partial registry after any definition failure in crates/kmipkit-protocol/tests/extension_information.rs and crates/kmipkit-client/tests/extension_registry.rs
- [ ] T020 [P] [US1] Write compile-fail boundary tests for arbitrary Item values, raw bodies, caller conversion traits, and executable plugins, plus fake-transport tests for explicit criticality, two outbound Message Extensions in caller order, and secret-bearing extension sends with partial-write success and error paths in crates/kmipkit-client/tests/ui/ and crates/kmipkit-client/tests/extension_outbound.rs
- [ ] T021 Commit the User Story 1 test-only Red stage and preserve failing command evidence for limit boundaries and the secret-buffer lifecycle test required by KMIPKIT-0007-OD-006: verify a private test-only owner-drop observer sees the initialized encoded range zeroized immediately before release, the owner remains live through every partial write until transport success/error, failures before writing report NotSent, failures after a request prefix report PossiblySent, response-read failures after response bytes begin report ResponseStarted, and no retry occurs in crates/kmipkit-protocol/tests/extension_definition.rs, crates/kmipkit-client/tests/extension_registry.rs, and crates/kmipkit-client/tests/extension_outbound.rs

### Implementation for User Story 1

- [ ] T022 [P] [US1] Implement fallible ExtensionIdentity, Compatibility, and ExtensionDefinition constructors with stable categories; check identity/metadata and discriminator scalar byte limits before cloning or reserving storage in crates/kmipkit-protocol/src/extension/definition.rs
- [ ] T023 [P] [US1] Implement immutable schema builders for structures, scalar constraints, cardinality, order, and undeclared-child behavior in crates/kmipkit-protocol/src/extension/schema.rs
- [ ] T024 [US1] Before cloning or reserving, implement checked definition-set totals for UTF-8 bytes across every identity field and Extension Information Text String across all definitions, schema nodes, discriminator scalar bytes, and constraint members; enforce configured/hard limits and build deterministic registry indexes in crates/kmipkit-client/src/extension_registry.rs
- [ ] T025 [US1] Implement schema construction/validation that yields a sealed ValidatedExtensionValue only on complete success in crates/kmipkit-protocol/src/extension/value.rs
- [ ] T026 [US1] Attach the finalized registry to client configuration and add repeatable, ordered validated Message Extension attachment with caller-explicit Criticality Indicator to typed ClientBatchItem encoding in crates/kmipkit-client/src/lib.rs and crates/kmipkit-client/src/execute.rs
- [ ] T027 [US1] Pass User Story 1 tests including raised/lowered/default/hard limit boundaries, bounded enum/order validation, and secret-buffer lifetime/zeroization across transport success and failure; execute compile-fail boundary checks and commit the Green stage with evidence in crates/kmipkit-protocol/tests/extension_definition.rs and crates/kmipkit-client/tests/extension_registry.rs
- [ ] T028 [US1] Refactor builders and validation for clarity and bounded work, rerun all limit, lifecycle, and focused tests, and commit the Refactor stage in crates/kmipkit-protocol/src/extension/ and crates/kmipkit-client/src/extension_registry.rs
- [ ] T029 [US1] Add runnable Rust, C, Java, and Python registration/construction examples in the English and Spanish guides and binding packages in docs/user-guide/en/vendor-extensions.md, docs/user-guide/es/extensiones-de-fabricante.md, bindings/c/examples/, bindings/java/examples/, and bindings/python/examples/

## Phase 4: User Story 2 - Recognize and inspect a registered extension (Priority: P1)

**Goal**: Provide a fully validated typed view while preserving the original generic TTLV and existing criticality handling.

**Independent Test**: Process matching, absent, repeated, wrong-type, malformed, unknown-critical, and unknown-non-critical extensions through deterministic fake-transport scenarios and verify no partial typed values or data loss.

### Tests for User Story 2

- [ ] T030 [P] [US2] Write failing tests for exact discriminator lookup, missing/repeated/wrong-type paths, wide payloads with a last-child match, zero matches, full-schema rejection after a unique discriminator hit, registry rejection of duplicate keys, and an ambiguous payload with two distinct paths that each pass full schema validation; assert comparison bounds and no registration-order winner in crates/kmipkit-client/tests/extension_recognition.rs
- [ ] T031 [P] [US2] Write failing property-based generic TTLV round-trip and preservation tests for tags unknown to the extension schema but accepted by the KMIP allocation policy (including a vendor-extension tag), enum values, bitmask bits, repeated fields, child order, duplicate-tag path non-matches, shared-index entry/comparison boundaries, repeated-enum and ordering-edge work bounds, TTLV byte/depth/element boundaries, and configured schema/registry limit boundaries in crates/kmipkit-protocol/tests/extension_preservation.rs
- [ ] T032 [P] [US2] Write failing fake-transport response tests proving critical rejection and non-critical preservation remain owned by KMIPKIT-0007 in crates/kmipkit-client/tests/extension_execution.rs
- [ ] T033 [P] [US2] Write failing secret-redaction tests for schema errors, Debug, Display, and diagnostics in crates/kmipkit-protocol/tests/extension_redaction.rs
- [ ] T034 Commit the User Story 2 test-only Red stage and preserve failing command evidence in crates/kmipkit-client/tests/extension_recognition.rs and crates/kmipkit-protocol/tests/extension_preservation.rs

### Implementation for User Story 2

- [ ] T035 [P] [US2] Implement one bounded temporary radix-ordered payload tag index shared across discriminator paths; use lower/upper-bound searches, treat repeated path tags as non-matches, enforce the payload-entry and comparison budgets, stop after the second exact key hit before schema validation, and validate the complete schema only for a unique hit without normalizing the input subtree in crates/kmipkit-client/src/extension_registry.rs
- [ ] T036 [P] [US2] Implement recognized/unrecognized result types that retain the original generic TTLV subtree and expose no partial typed projection in crates/kmipkit-protocol/src/extension/value.rs
- [ ] T037 [US2] Integrate registry inspection with typed response/message mapping while retaining KMIPKIT-0007 criticality decisions in crates/kmipkit-client/src/execute.rs
- [ ] T038 [US2] Verify extension payload ownership follows existing zeroization behavior and redact all error/debug/display paths in crates/kmipkit-protocol/src/extension/ and crates/kmipkit-client/src/extension_registry.rs
- [ ] T039 [US2] Pass recognition, preservation, fake-transport, and redaction tests and commit the Green stage with evidence in crates/kmipkit-client/tests/extension_recognition.rs and crates/kmipkit-client/tests/extension_execution.rs
- [ ] T040 [US2] Refactor matching and response integration, rerun focused and property tests, and commit the Refactor stage in crates/kmipkit-client/src/extension_registry.rs and crates/kmipkit-protocol/src/extension/
- [ ] T041 [US2] Document Rust, C, Java, and Python recognition, unknown criticality, preservation, and runtime-copy limitations in docs/user-guide/en/vendor-extensions.md, docs/user-guide/es/extensiones-de-fabricante.md, and the language package examples

## Phase 5: User Story 3 - Share metadata and validation across languages (Priority: P1)

**Goal**: Expose equivalent registry behavior through Rust, C, Java 17, and Python 3.12 from the same reviewed manifest and shared fixtures.

**Independent Test**: Compile and call a real C consumer, run Java JNI and Python CFFI/Maturin tests, and compare normalized results/error categories against Rust for identical fixtures.

### Tests for User Story 3

- [ ] T042 [P] [US3] Write failing Rust manifest parity and generation-check tests for requirement IDs, ExtensionRegistryLimits fields/defaults/hard maxima, symbol ownership, destinations, and stable order in tools/api_manifest/tests/test_parity.py
- [ ] T043 [P] [US3] Write failing C consumer tests using shared fixtures for registration, every ExtensionRegistryLimits field/boundary including raising only fields whose default is below hard maximum, typed-pointer/uint64_t-length rejection before any input access, and no unbounded NUL scans; test NULL and wrong-kind live-handle rejection as `invalid_input` for handle-taking functions, release semantics, and the documented caller precondition for use-after-release/dangling/foreign pointers; also test lookup/schema comparison limits, repeated outbound order/criticality, inbound typed inspection, preservation of tags unknown to the extension schema but accepted by KMIP allocation rules (including a vendor-extension tag), and stable redacted errors in bindings/c/tests/extension_registry.c and tests/fixtures/extensions/
- [ ] T044 [P] [US3] Write failing Java 17 JNI tests using shared fixtures for registration, every ExtensionRegistryLimits field/boundary including raising only fields whose default is below hard maximum, lookup/schema comparison limits, repeated outbound order/criticality, inbound typed inspection, preservation of tags unknown to the extension schema but accepted by KMIP allocation rules (including a vendor-extension tag), metadata, closed-handle `invalid_input` lifecycle behavior, and redaction in bindings/java/src/test/java/org/kmipkit/ExtensionRegistryTest.java and tests/fixtures/extensions/
- [ ] T045 [P] [US3] Write failing Python 3.12 CFFI tests using shared fixtures for registration, every ExtensionRegistryLimits field/boundary including raising only fields whose default is below hard maximum, lookup/schema comparison limits, repeated outbound order/criticality, inbound typed inspection, preservation of tags unknown to the extension schema but accepted by KMIP allocation rules (including a vendor-extension tag), metadata, closed-handle `invalid_input` context-manager lifecycle behavior, and redaction in bindings/python/tests/test_extension_registry.py and tests/fixtures/extensions/
- [ ] T046 [P] [US3] Write failing parity-corpus checks requiring every shared fixture to declare normalized repeated outbound TTLV/order, inbound recognition and generic-preservation expectations, each default/lowered/hard/over-hard outcome and raised-default outcomes only where default < hard maximum, including identity/discriminator bytes, constraint members, payload-index records, lookup comparisons, per-item enum/schema comparisons and single-pass order-edge work, stable error category, and deterministic metadata for all four adapters in tools/api_manifest/tests/test_parity.py
- [ ] T047 Commit the User Story 3 test-only Red stage and preserve native and binding test failure evidence in bindings/c/tests/extension_registry.c, bindings/java/src/test/java/org/kmipkit/ExtensionRegistryTest.java, and bindings/python/tests/test_extension_registry.py

### Implementation for User Story 3

- [ ] T048 [P] [US3] Implement opaque fixed-width C handles and kmipkit_-prefixed registry functions for limits, outbound attachment, and inbound typed inspection/preservation; use typed pointers with uint64_t byte lengths for all variable-length inputs, reject over-limit values before dereference/read/copy, never perform unbounded NUL scans, and add SAFETY comments for each unsafe block in crates/kmipkit-ffi/src/extension_registry.rs
- [ ] T049 [P] [US3] Implement the compiled real-C-consumer test target and link it to the KMIPKit shared library in bindings/c/CMakeLists.txt and bindings/c/tests/
- [ ] T050 [P] [US3] Implement Java 17 handwritten facade, JNI lifecycle, ExtensionRegistryLimits, outbound attachment, inbound typed inspection/preservation, and stable error mapping in bindings/java/src/main/java/org/kmipkit/ExtensionRegistry.java and bindings/java/src/main/java/org/kmipkit/NativeExtensionRegistry.java
- [ ] T051 [P] [US3] Implement Python 3.12 CFFI/Maturin facade, deterministic close/context-manager behavior, ExtensionRegistryLimits, outbound attachment, inbound typed inspection/preservation, and stable error mapping in bindings/python/src/kmipkit/extensions.py
- [ ] T052 [US3] Generate Rust/C/Java/Python declarations and parity scaffolding from specification/api/public-api.json in tools/api_manifest/generate.py
- [ ] T053 [US3] Run the real C consumer, Java 17 JNI suite, Python 3.12 CFFI suite, and shared fixture parity tests; commit the Green stage with evidence in bindings/c/tests/, bindings/java/src/test/, and bindings/python/tests/
- [ ] T054 [US3] Refactor binding ownership and lifecycle code, rerun all parity suites, and commit the Refactor stage in crates/kmipkit-ffi/src/extension_registry.rs and bindings/
- [ ] T055 [US3] Document the manifest regeneration and check commands for maintainers in docs/development/api-manifest.md

## Phase 6: Polish, conformance, and release readiness

**Purpose**: Close traceability, coverage, generated-output, security, and documentation gates for this feature.

- [ ] T056 [P] Add exact-clause OASIS-derived vectors for §8.3 Table 396 repeated Message Extension order, §9.13 Table 418 structure, §7.13 Table 365, and §11.44 Table 476, plus a bounded schema-validation fuzz target, in tests/fixtures/extensions/oasis/, fuzz/fuzz_targets/extension_schema.rs, and fuzz/Cargo.toml
- [ ] T057 Add stable requirement-to-source/spec/code/test links for every KMIPKIT-0012-FR entry in specification/compliance/requirements/KMIPKIT-0012.csv
- [ ] T058 Update English and Spanish API/user documentation and execute every runnable registry example in docs/user-guide/en/vendor-extensions.md and docs/user-guide/es/extensiones-de-fabricante.md
- [ ] T059 Add CI regeneration check that fails on generated diffs and run it without modifying generated outputs by hand in .github/workflows/ci.yml
- [ ] T060 Enforce feature coverage gates of at least 95% TTLV/protocol and changed code, 85% FFI/adapters and any changed transport code, and the repository 90% overall gate; document only justified generated-code exclusions in docs/development/testing.md
- [ ] T061 Run formatting, Clippy, workspace tests, cargo llvm-cov, generator checks, property/negative/fuzz tests, FFI sanitizer jobs, C consumer, Java, Python, dependency-policy CI, and supported-platform CI; verify limit/lifecycle tests and record exact command results in specs/012-vendor-extension-registry/verification.md
- [ ] T062 Reconcile implementation, tests, docs, generated artifacts, and requirement traceability against every acceptance criterion; audit Rust, C, Java, and Python public signatures for forbidden raw-body, arbitrary-Item, callback, or executable-plugin routes in specs/012-vendor-extension-registry/verification.md
- [ ] T063 Complete independent QA and security reviews, resolve findings, and record evidence without treating agent review as human approval in specs/012-vendor-extension-registry/verification.md
- [ ] T064 Prepare the draft PR with Red/Green/Refactor commits, risks, verification evidence, and generated-artifact summary in the GitHub pull request for KMIPKIT-0012

## Dependencies & Execution Order

### Phase dependencies

- KMIPKIT-0006 generic message/batch models and KMIPKIT-0007 client execution/criticality behavior are required base dependencies; verify both are present on the active release base before T009 begins. This feature does not replace either owner.
- Setup tasks T001-T008 precede feature implementation; generator tests are written before generator code.
- Foundational tasks T009-T016 establish the bounded protocol and registry contracts and block all story implementation.
- User Story 1 tasks T017-T029 depend on the foundation and enable safe definition/value creation.
- User Story 2 tasks T030-T041 depend on User Story 1 validation types and integrate with KMIPKIT-0007 without taking over criticality behavior.
- User Story 3 tasks T042-T055 depend on stable registry semantics and the manifest contract; wrappers may proceed in parallel after Red tests and contracts are stable.
- Polish tasks T056-T064 depend on all three stories and generated output being complete.

### User story dependencies

- US1 follows the protocol/client foundation.
- US2 depends on US1's sealed validated value and registry lookup.
- US3 depends on stable Rust semantics and the reviewed registry manifest; its language wrappers may be implemented in parallel after interfaces are fixed.

### Within each story

- Write the tests and record failures before production implementation.
- Keep Red, Green, and Refactor evidence in distinct development commits.
- Re-run focused tests after refactoring; do not weaken the acceptance criteria to obtain a pass.
- Generated files are changed only by the pinned generator.

## Requirement coverage map

Every functional requirement and buildable success criterion has at least one planned test or implementation task. T057 creates the durable source/spec/code/test traceability record during implementation.

| Requirement | Planned tasks |
|---|---|
| KMIPKIT-0012-FR-001 | T010, T013, T026 |
| KMIPKIT-0012-FR-002 | T012, T017, T022-T023 |
| KMIPKIT-0012-FR-003 | T017, T024 |
| KMIPKIT-0012-FR-004 | T012, T020, T023, T062 |
| KMIPKIT-0012-FR-005 | T004, T020, T025, T026 |
| KMIPKIT-0012-FR-006 | T030, T035 |
| KMIPKIT-0012-FR-007 | T004, T031, T036 |
| KMIPKIT-0012-FR-008 | T032, T037 |
| KMIPKIT-0012-FR-009 | T012, T019 |
| KMIPKIT-0012-FR-010 | T004, T043-T055 |
| KMIPKIT-0012-FR-011 | T020-T021, T027-T028, T033, T038, T043-T045 |
| KMIPKIT-0012-FR-012 | T004, T009, T014, T017-T018, T024, T027, T030-T031, T035, T043-T046, T048, T050-T051, T056 |
| KMIPKIT-0012-FR-013 | T029, T041, T058 |
| KMIPKIT-0012-FR-014 | T001-T008, T042, T049, T052, T059 |
| KMIPKIT-0012-SC-001 | T043-T047, T053 |
| KMIPKIT-0012-SC-002 | T046, T053 |
| KMIPKIT-0012-SC-003 | T031, T036 |
| KMIPKIT-0012-SC-004 | T020, T062 |
| KMIPKIT-0012-SC-005 | T057, T062 |
| KMIPKIT-0012-SC-006 | T020-T021, T027-T028, T033, T038, T043-T045 |
| KMIPKIT-0012-SC-007 | T014, T017-T018, T024, T027, T030-T031, T035, T043-T046, T048, T050-T051, T061 |
| KMIPKIT-0012-SC-008 | T020-T021, T027-T028, T061 |

## Parallel opportunities

- T001-T002 and T007-T008 use separate test/fixture paths.
- Protocol tests T009 and client tests T010 can be authored in parallel.
- US1 protocol constructors T022-T023 can proceed in parallel after Red evidence.
- US2 bounded matching T035 and preservation result modeling T036 can proceed in parallel after contracts are stable.
- C, Java, and Python adapters T048-T051 can proceed in parallel after shared interfaces and fixtures are reviewed.

## Implementation strategy

1. Complete generator and shared-fixture setup.
2. Complete and review the bounded protocol/client foundation before story work.
3. Deliver US1, validate independently, then US2, then US3 with Red/Green/Refactor evidence for each.
4. Run cross-language parity and all security, traceability, coverage, and CI gates before preparing a draft PR.
5. Do not begin implementation until the specification is reviewed and integrated under the repository workflow.
