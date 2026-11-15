# Tasks: KMIPKit Fluent Object Creation

**Input**: Design documents in this directory (`spec.md`, `plan.md`,
`research.md`, `data-model.md`, `contracts/public-api.md`, and `quickstart.md`).

**Prerequisites**: This specification must be approved and merged before
production work. KMIPKIT-0091 implementation PR #172 must also be merged into
`release/1.0.0`, because it provides the fluent namespace and outcome
contract. The implementation branch must be refreshed from the current
`release/1.0.0` after both gates are satisfied.

**Tests**: Required by the user-approved plan. Use strict Red, Green, Refactor
TDD and separate commits for each evidence phase.

## Phase 1: Setup

- [ ] T001 Confirm KMIPKIT-0092 approval, KMIPKIT-0091 merge, and the current `release/1.0.0` SHA before editing production files.
- [ ] T002 Rebase or recreate `feature/KMIPKIT-0092-fluent-objects` from the active `release/1.0.0` and record the base SHA in `specs/092-fluent-object-creation/plan.md`.
- [ ] T003 Run the existing workspace and binding baselines and record them in `specs/092-fluent-object-creation/implementation-evidence.md`.

## Phase 2: Foundational Contracts

**Purpose**: Establish shared builder and parity behavior before operation-specific work.

- [ ] T004 [P] Add Rust request builder boundary tests for typed request conversion and outcome preservation in `crates/kmipkit-client/tests/unit/fluent_request_tests.rs`.
- [ ] T005 [P] Add C ABI status, ownership, versioned-record, and release contract tests in `bindings/c/tests/public_client_api.c`.
- [ ] T006 Add Java and Python facade parity fixtures for supported object handles and runtime ownership in `bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java` and `bindings/python/tests/test_fluent.py`.
- [ ] T007 Implement shared Rust builder conversion helpers in `crates/kmipkit-client/src/fluent.rs` without creating new transport, serialization, or outcome behavior.

## Phase 3: User Story 1 — Create a Key Pair (P1)

**Goal**: Create an explicitly configured key pair in each language and inspect both server-returned IDs.

**Independent Test**: Use the controlled TLS fixture to verify the chosen algorithm, parameters, distinct usages, attribute groups, and both response identifiers.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add failing Rust builder tests for RSA 3072, ECDSA P-256, and ECDH P-256 presets and caller-selected usage masks in `crates/kmipkit-client/tests/unit/fluent_request_tests.rs`.
- [ ] T009 [P] [US1] Add failing Rust validation tests for Table 191 group precedence, absent values, equal overrides, differing values, and wrong types in `crates/kmipkit-protocol/tests/create_key_pair_fluent_tests.rs`.
- [ ] T010 [P] [US1] Add failing C consumer tests for direct Create Key Pair inputs, opaque builder ownership, and result identifiers in `bindings/c/tests/public_client_api.c`.
- [ ] T011 [P] [US1] Add failing Java and Python semantic API tests for preset and independent usage selection in `bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java` and `bindings/python/tests/test_fluent.py`.
- [ ] T012 [P] [US1] Add a TLS fixture consumer case for Create Key Pair in `tests/consumers/0053-parity/` and verify both returned IDs in all language suites.

### Implementation for User Story 1

- [ ] T013 [US1] Implement Create Key Pair fluent construction and the three approved presets in `crates/kmipkit-client/src/fluent.rs`, setting only algorithm/length/curve and preserving caller-selected usage.
- [ ] T014 [US1] Add C Create Key Pair direct inputs and opaque builder adapters in `crates/kmipkit-ffi/src/` and declarations in `bindings/c/include/kmipkit.h`.
- [ ] T015 [US1] Add Java Create Key Pair facade/builders in `bindings/java/src/main/java/org/kmipkit/client/` and `bindings/java/src/main/java/org/kmipkit/fluent/`.
- [ ] T016 [US1] Add Python Create Key Pair named-argument methods and builders in `bindings/python/src/kmipkit/client.py` and `bindings/python/src/kmipkit/fluent.py`.
- [ ] T017 [US1] Refactor shared key-pair attribute and preset handling in `crates/kmipkit-client/src/fluent.rs` while keeping the Table 191 validation path authoritative.

## Phase 4: User Story 2 — Create Split Key (P1)

**Goal**: Send explicit split parameters and retain all response identifiers in wire order.

**Independent Test**: The TLS fixture validates the method, type, part count, threshold, attributes, optional input identifier, and all response identifiers.

### Tests for User Story 2

- [ ] T018 [P] [US2] Add failing Rust builder tests for explicit method, parts, threshold, optional ID, and mandatory Prime Field Size policy in `crates/kmipkit-client/tests/unit/fluent_request_tests.rs`.
- [ ] T019 [P] [US2] Add failing C consumer tests for Split Key records, optional fields, and repeated result IDs in `bindings/c/tests/public_client_api.c`.
- [ ] T020 [P] [US2] Add failing Java and Python Split Key builder tests in `bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java` and `bindings/python/tests/test_fluent.py`.
- [ ] T021 [P] [US2] Add a TLS fixture consumer case proving all returned Split Key IDs and their order in `tests/consumers/0053-parity/`.

### Implementation for User Story 2

- [ ] T022 [US2] Implement the Split Key fluent builder in `crates/kmipkit-client/src/fluent.rs`, delegating required fields and conditional Prime Field Size validation to the existing typed request.
- [ ] T023 [US2] Add C Split Key direct inputs and opaque builder adapters in `crates/kmipkit-ffi/src/` and `bindings/c/include/kmipkit.h`.
- [ ] T024 [US2] Add Java Split Key facade/builders in `bindings/java/src/main/java/org/kmipkit/client/` and `bindings/java/src/main/java/org/kmipkit/fluent/`.
- [ ] T025 [US2] Add Python Split Key named-argument methods and builders in `bindings/python/src/kmipkit/client.py` and `bindings/python/src/kmipkit/fluent.py`.
- [ ] T026 [US2] Refactor repeated Split Key identifiers and response access in `crates/kmipkit-client/src/fluent.rs` without truncating or reordering results.

## Phase 5: User Story 3 — Register a Managed Object (P1)

**Goal**: Register an existing approved managed-object form through semantic fields and return its identifier.

**Independent Test**: The TLS fixture validates object type, Key Format Type, Key Material, attributes, and secret-safe ownership for every approved form.

### Tests for User Story 3

- [ ] T027 [P] [US3] Add failing Rust Register builder tests for all nine KMIPKIT-0049 object variants and all approved Key Material TTLV forms in `crates/kmipkit-client/tests/unit/fluent_request_tests.rs`.
- [ ] T028 [P] [US3] Add failing pre-send tests for object/type mismatch, conditional algorithm/length errors, and malformed managed objects in `crates/kmipkit-protocol/tests/register_fluent_tests.rs`.
- [ ] T029 [P] [US3] Add failing C consumer tests for semantic Register object construction, secret ownership, and handle release in `bindings/c/tests/public_client_api.c`.
- [ ] T030 [P] [US3] Add Java and Python Register tests for typed object inputs, close/context-manager lifetimes, redaction, and documented runtime-copy behavior in `bindings/java/src/test/java/org/kmipkit/fluent/FluentFacadeShapeTest.java` and `bindings/python/tests/test_fluent.py`.
- [ ] T031 [P] [US3] Add plaintext and wrapped Register consumer cases against the TLS fixture in `tests/consumers/0053-parity/`.

### Implementation for User Story 3

- [ ] T032 [US3] Implement semantic Register constructors for all nine approved object types in `crates/kmipkit-client/src/fluent.rs` using KMIPKIT-0049 typed models and existing Register validation.
- [ ] T033 [US3] Add C Register direct functions and opaque typed-object builders in `crates/kmipkit-ffi/src/` and `bindings/c/include/kmipkit.h` with existing secret owners and release rules.
- [ ] T034 [US3] Add Java Register builders and ownership documentation in `bindings/java/src/main/java/org/kmipkit/fluent/` and `bindings/java/src/main/java/org/kmipkit/client/`.
- [ ] T035 [US3] Add Python Register builders and ownership documentation in `bindings/python/src/kmipkit/fluent.py` and `bindings/python/src/kmipkit/client.py`.
- [ ] T036 [US3] Refactor typed Register adaptation in `crates/kmipkit-client/src/fluent.rs` and preserve generic expert fallback only through the explicit existing typed object path.

## Phase 6: Polish and Cross-Cutting Concerns

- [ ] T037 [P] Regenerate and audit Rust/C/Java/Python public declarations and manifest entries with the pinned generator in `specification/api/public-api.json` and its generated outputs.
- [ ] T038 [P] Add Create Key Pair, Create Split Key, and Register examples and secret-lifetime notes to `docs/user-guide/en/` and `docs/user-guide/es/`.
- [ ] T039 Update exact implementation/test symbols and results in `specs/092-fluent-object-creation/traceability.md` and `specs/092-fluent-object-creation/implementation-evidence.md`.
- [ ] T040 Run the required Rust, C, Java, Python, TLS fixture, coverage, traceability, generation, security, and compatibility checks and record results in `specs/092-fluent-object-creation/implementation-evidence.md`.
- [ ] T041 Open a `[WIP]` draft PR, fix every CI failure, and only remove `[WIP]` and mark ready after all CI checks are green.

## Dependencies and Execution Order

- T001–T003 must complete before implementation; T001 is blocked until both human approval of this spec and merge of PR #172.
- T004–T007 establish shared builder behavior before any story implementation.
- User stories are ordered US1 → US2 → US3 because each reuses the shared
  builder and result model; within a story, failing tests precede production
  changes and Refactor follows Green.
- T037–T041 require all three stories to be complete.

## Parallel Execution Examples

- After T007, independent US1 protocol/client tests (T008–T009), C tests
  (T010), foreign binding tests (T011), and fixture test scaffolding (T012)
  can be prepared in separate files. Production changes must wait for the
  failing test evidence.
- After US1 is complete, US2's language tests (T018–T021) can be prepared in
  parallel. Apply the same pattern to US3 tests (T027–T031).
- Final guide edits (T038) and manifest preparation (T037) can proceed in
  parallel once the public surface is stable.

## Implementation Strategy

Deliver the Create Key Pair story first as the smallest four-language
increment, then Split Key, then Register's managed-object and secret-ownership
surface. Every story must pass its controlled TLS journey in each language
before starting the next story. Do not implement production code before the
specification approval and KMIPKIT-0091 merge gates are satisfied.
