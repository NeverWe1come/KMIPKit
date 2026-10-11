# Tasks: KMIP 2.1 Credential Sessions

**Input**: Design documents from specs/024-credential-sessions/

**Prerequisites**: Approved spec.md, plan.md, research.md, data-model.md, contracts/, traceability.md

**Tests**: Required by the approved acceptance criteria and AGENTS.md. Every user-story implementation follows separate Red, Green, and Refactor development commits.

**Organization**: Tasks are grouped by the three P1 user stories. The shared Ticket model is foundational.

## Format and paths

Every task uses the repository checklist form: checkbox, sequential task ID, optional parallel marker, story label for story tasks, and an exact repository path.

## Phase 1: Setup

**Purpose**: Confirm the pinned source and accepted release baseline; no project initialization or new dependency is required.

- [x] T001 Verify the OASIS source checksum and release-branch foundation references in specification/oasis/kmip-2.1/CHECKSUMS.sha256, specs/024-credential-sessions/spec.md, and specs/024-credential-sessions/plan.md; record any source change before coding.

## Phase 2: Foundational Ticket model

**Purpose**: Implement the shared §7.39 Ticket model required by Delegated Login, Login, and Logout before the operation stories.

- [x] T002 Add failing Ticket model tests in crates/kmipkit-protocol/tests/unit/ticket_model_tests.rs and register the test module in crates/kmipkit-protocol/src/lib.rs for required Ticket Type and Ticket Value, open-enum and property-based TTLV round trips, malformed nested values, redacted formatting, and secret ownership; run the focused cargo test and commit the Red evidence separately.
- [x] T003 Implement the typed Ticket structure and secret-safe Ticket Value in crates/kmipkit-protocol/src/ticket.rs; export it from crates/kmipkit-protocol/src/lib.rs and crates/kmipkit/src/lib.rs and make every T002 Red test pass; commit the Green evidence separately.
- [ ] T004 Refactor Ticket parsing, redaction, and zeroization paths in crates/kmipkit-protocol/src/ticket.rs and crates/kmipkit-protocol/tests/unit/ticket_model_tests.rs without changing wire behavior; rerun focused tests and commit the Refactor evidence separately.

**Checkpoint**: Ticket fields round-trip and remain secret-safe. Operation-specific work can proceed.

## Phase 3: User Story 1 — Delegated Login (Priority: P1)

**Goal**: Expose an explicit typed Delegated Login exchange with Rights and optional lease/count/usage fields.

**Independent Test**: Fake transport captures one Table 199 request and returns Table 200 success or Table 201 failure; verify field presence/order, result metadata, delivery evidence, one exchange, and no retry.

### Red commit

- [ ] T005 Add failing protocol and fake-transport tests in crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs and crates/kmipkit-client/tests/unit/delegated_login_execution_tests.rs; register them through crates/kmipkit-protocol/src/lib.rs and crates/kmipkit-client/src/execute.rs; cover required Rights, optional fields and both Lease Time forms, Ticket response, Table 201 reasons, malformed payload, unknown tags/values, Pending/correlation without Poll, one exchange, no retry, and no hidden auth-state mutation; record the failing command/output in a separate Red commit.

### Green commit

- [ ] T006 Implement DelegatedLoginRequest, DelegatedLoginResponse, and sanitized errors in crates/kmipkit-protocol/src/delegated_login.rs; export the models from crates/kmipkit-protocol/src/lib.rs and integrate the explicit request/response in crates/kmipkit-client/src/execute.rs, crates/kmipkit-client/src/lib.rs, and crates/kmipkit/src/lib.rs; make T005 pass and commit separately as Green.

### Refactor commit

- [ ] T007 Refactor Delegated Login payload handling and tests in crates/kmipkit-protocol/src/delegated_login.rs and crates/kmipkit-protocol/tests/unit/delegated_login_operation_tests.rs; preserve field order, unknown data, secret redaction, and behavior, rerun the focused tests, and commit separately as Refactor.

**Checkpoint**: Delegated Login can be constructed, executed once, and inspected without automatic session state.

## Phase 4: User Story 2 — Login (Priority: P1)

**Goal**: Expose an explicit typed Login exchange with absent optional fields preserved and a caller-owned Ticket result.

**Independent Test**: Fake transport exercises the pinned TC-LOGIN-1-21 fixture and Tables 253–255; verify result semantics, one exchange, delivery evidence, redaction, and no retry.

### Red commit

- [ ] T008 Add failing protocol and fake-transport tests in crates/kmipkit-protocol/tests/unit/login_operation_tests.rs and crates/kmipkit-client/tests/unit/login_execution_tests.rs; register them through crates/kmipkit-protocol/src/lib.rs and crates/kmipkit-client/src/execute.rs; cover empty/optional Table 253 requests, Table 254 Ticket parsing, Table 255 reasons, fixture case KMIPKIT-TEST-CN01-2-48, malformed/unknown data, Pending/correlation without Poll, no retry, and no hidden auth-state mutation; record the failing command/output in a separate Red commit.

### Green commit

- [ ] T009 Implement LoginRequest and LoginResponse in crates/kmipkit-protocol/src/login.rs; export the models from crates/kmipkit-protocol/src/lib.rs and integrate Login in crates/kmipkit-client/src/execute.rs, crates/kmipkit-client/src/lib.rs, and crates/kmipkit/src/lib.rs; make T008 pass and commit separately as Green.

### Refactor commit

- [ ] T010 Refactor Login request/response handling and tests in crates/kmipkit-protocol/src/login.rs and crates/kmipkit-protocol/tests/unit/login_operation_tests.rs without changing wire behavior; rerun fixture and focused tests, then commit separately as Refactor.

**Checkpoint**: Login returns a secret-safe Ticket without changing future client authentication state.

## Phase 5: User Story 3 — Logout (Priority: P1)

**Goal**: Expose an explicit typed Logout exchange requiring a caller-supplied Ticket.

**Independent Test**: Fake transport captures one Table 256 request, then returns an empty Table 257 success or a Table 258 failure; verify secret-safe encoding and no retry.

### Red commit

- [ ] T011 Add failing protocol and fake-transport tests in crates/kmipkit-protocol/tests/unit/logout_operation_tests.rs and crates/kmipkit-client/tests/unit/logout_execution_tests.rs; register them through crates/kmipkit-protocol/src/lib.rs and crates/kmipkit-client/src/execute.rs; cover required Ticket, empty success payload, Table 258 reasons, malformed/unknown response, Pending/correlation without Poll, NotSent validation, delivery state, no retry, and no hidden auth-state mutation; record the failing command/output in a separate Red commit.

### Green commit

- [ ] T012 Implement LogoutRequest and LogoutResponse in crates/kmipkit-protocol/src/logout.rs; export the models from crates/kmipkit-protocol/src/lib.rs and integrate Logout in crates/kmipkit-client/src/execute.rs, crates/kmipkit-client/src/lib.rs, and crates/kmipkit/src/lib.rs; make T011 pass and commit separately as Green.

### Refactor commit

- [ ] T013 Refactor Logout encoding, response validation, and tests in crates/kmipkit-protocol/src/logout.rs and crates/kmipkit-protocol/tests/unit/logout_operation_tests.rs without changing wire behavior; rerun focused tests and commit separately as Refactor.

**Checkpoint**: Logout can only use an explicit caller-provided Ticket and returns the common result without hidden session changes.

## Phase 6: Polish and cross-cutting concerns

**Purpose**: Close traceability, documentation, parity, security, and release evidence.

- [ ] T014 Update specs/024-credential-sessions/traceability.md and specification/catalog/kmip-2.1.json with implementation/verification references for all three operation elements, operation values, payload tag/structure/enumeration elements, common result tags, table trace IDs, and applicable test cases; run tools/normative_catalog/validate.py and tools/normative_catalog/report.py --write followed by --check.
- [ ] T015 Add failing parity and ABI tests in bindings/c/tests/credential_sessions.c, bindings/java/src/test/java/org/kmipkit/CredentialSessionsParityTest.java, bindings/python/tests/test_credential_sessions.py, and crates/kmipkit-ffi/tests/c_api_coverage.rs; register the C test in bindings/c/CMakeLists.txt and commit the Red evidence separately after KMIPKIT-0043 approves and merges the canonical API contract.
- [ ] T016 Add the three approved operation entries to specification/api/public-api.json, implement the C ABI adapter in crates/kmipkit-ffi/src/lib.rs, the JNI bridge in bindings/java/native/kmipkit_jni.cpp and Java surface in bindings/java/src/main/java/org/kmipkit/CredentialSessionOperations.java, and the Python surface in bindings/python/src/kmipkit/credential_sessions.py; regenerate C header/Java/Python manifest outputs with tools/api_manifest/generate.py --write, and make T015 pass; keep the manifest, adapters, tests, generated files, and traceability in the coordinated API change and commit Green separately.
- [ ] T017 Refactor operation bindings and parity fixtures in crates/kmipkit-ffi/src/lib.rs, bindings/java/native/kmipkit_jni.cpp, bindings/java/src/main/java/org/kmipkit/CredentialSessionOperations.java, bindings/python/src/kmipkit/credential_sessions.py, and bindings/c/tests/credential_sessions.c without changing public behavior; run parity tests and tools/api_manifest/generate.py --check, then commit Refactor separately.
- [ ] T018 Add and compile examples in docs/user-guide/en/credential-sessions.md and docs/user-guide/es/sesiones-credenciales.md; register compilation coverage in crates/kmipkit-protocol/tests/credential_sessions_guide_examples.rs; ensure examples explicitly pass Ticket values, redact secrets, and do not claim official conformance for unavailable cases.
- [ ] T019 Run the focused protocol/client tests, fixture checks, and all commands listed in specs/024-credential-sessions/quickstart.md; run applicable generated-artifact/security checks; record exact results and limitations in the PR.
- [ ] T020 Add these operations to docs/development/kmip-interoperability.md and the two-independent-implementation integration matrix; record server/version and results before claiming interoperability or 1.0.0 completion.

## Dependencies and execution order

- Phase 1 verifies source and baseline; it does not create new infrastructure.
- Phase 2 provides the shared Ticket type and blocks the three operation stories.
- User Stories 1–3 share protocol exports and the central client dispatcher in crates/kmipkit-client/src/execute.rs. Their model/test files are operation-specific, but integrate central files in a coordinated sequence.
- T015–T017 C/Java/Python parity slice depends on merged KMIPKIT-0043 and its approved public API manifest contract; protocol models and the Rust client work in T002–T013 do not.
- Phase 6 depends on all three Rust stories. T020 is an overall 1.0 release gate and must not be claimed until both independent server integrations pass.

## Parallel opportunities

- The three operation test modules and protocol model modules use distinct files and can be developed as isolated Red/model work after the shared Ticket contract is stable.
- Client executor dispatch, response view, and crate export files are shared hotspots and must be integrated serially.
- Cross-language generation is independent of the Rust protocol models once the manifest slice is approved; it must stay coordinated with the manifest owner.

## Implementation strategy

Deliver the shared secret-safe Ticket model first, then complete each operation through separate Red, Green, and Refactor commits. Delegated Login is the MVP story because it covers the required Rights list and Ticket response; Login and Logout then reuse the same Ticket type. Finish traceability and documentation, add language parity from the manifest, and treat the two-server test as a release gate rather than an unsupported claim.

## Independent test criteria

- **US1**: Table 199–201 model and fake-client tests pass, including required Rights, optional fields, response Ticket, result/reason, Pending, delivery, one exchange, and no retry.
- **US2**: Table 253–255 tests and the pinned TC-LOGIN-1-21 fixture pass, with Ticket Value redacted.
- **US3**: Table 256–258 tests pass; a missing Ticket is rejected before transmission and success payload is empty.
