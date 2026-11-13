# KMIPKIT-0115 Delivery Tasks

## Phase 1: Setup

- [x] T001 Confirm KMIP 2.1 clauses, existing typed constructors, handle transfer rules, and language test commands in `specs/115-java-python-lifecycle-facade/research.md`.

## Phase 2: Foundational Contract Tests

- [x] T002 [P] [RED COMMIT] Add failing Java shape and controlled mTLS tests for the five operations in `bindings/java/src/test/java/org/kmipkit/fluent/FluentLifecycleApiShapeTest.java` and `FluentLifecycleNativeIntegrationTest.java`.
- [x] T003 [P] [RED COMMIT] Add failing Python builder, direct-method, and controlled mTLS tests for the five operations in `bindings/python/tests/test_fluent_lifecycle.py` (record both language failures in the Red commit).

## Phase 3: User Story 1 - Java lifecycle facade (Priority: P1)

**Independent Test**: Java builds all five typed operations, rejects invalid inputs before transport, and decodes fixture responses.

- [ ] T004 [US1] Implement Java `OperationBuilder`, `LocateBuilder`, `RevokeBuilder`, and `FluentRequest` constructors in `bindings/java/src/main/java/org/kmipkit/fluent/`.
- [ ] T005 [US1] Add direct methods and exchange-options overloads in `bindings/java/src/main/java/org/kmipkit/client/KmipClient.java`; pass the Java lifecycle tests.

## Phase 4: User Story 2 - Python lifecycle facade (Priority: P1)

**Independent Test**: Python builds all five typed operations, rejects invalid inputs before transport, closes temporary handles, and decodes fixture responses.

- [ ] T006 [US2] Implement matching operation, Locate, and Revoke builders in `bindings/python/src/kmipkit/fluent.py` and named direct methods in `bindings/python/src/kmipkit/client.py`.
- [ ] T007 [US2] Add bilingual lifecycle examples and Java/Python ownership notes in `docs/user-guide/en/lifecycle-operations.md` and `docs/user-guide/es/operaciones-ciclo-vida.md`.

## Phase 5: Polish and Cross-Cutting Checks

- [ ] T008 Register all new declarations under KMIPKIT-0115 in `specification/api/public-api.json`, regenerate outputs with `tools/api_manifest/generate.py`, and update `specs/115-java-python-lifecycle-facade/traceability.md`.
- [ ] T009 Refactor shared builder execution only after green behavior and commit the cleanup separately.
- [ ] T010 Run focused and full Java/Python checks, manifest/export audits, traceability, and exact-head CI; record results in `specs/115-java-python-lifecycle-facade/implementation-evidence.md`.
