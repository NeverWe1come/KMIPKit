---
description: "Dependency-ordered KMIPKIT-0051 implementation tasks"
---

# Tasks: KMIPKIT-0051 Pre-Send Validation

**Input**: `specs/051-pre-send-validation/spec.md`, `plan.md`, `gap-matrix.md`, `traceability.md`, `contracts/rust-api.md`

**Prerequisites**: Approved maintainer scope is recorded in `approval-record.md`; the feature is stacked on KMIPKIT-0050 / PR #118.

**Tests**: Strict Red, Green, Refactor TDD is required. Each R/G/R task below must be recorded in a separate development commit; all Red tasks must run and fail for the intended missing behavior before production code is written.

## Phase 1: Setup and source inventory

**Purpose**: Establish complete OASIS and code coverage before implementation.

- [x] T001 Verify the 56 supported `ClientOperation` variants against `specs/051-pre-send-validation/gap-matrix.md`, the KMIP v2.1 §6.1 Request Payload tables, and `crates/kmipkit-client/src/execute.rs`; retain the explicit 1.1 deferral for §6.1.54 Set Endpoint Role.
- [x] T002 Audit every field in the 56 Request Payload tables and record a stable `KMIPKIT-0051-R###` ID, field tag or object-schema tag rule, TTLV type, order, requiredness, repeatability, locally decidable conditions, exact nested-structure references, and raw/typed model boundary in `gap-matrix.md` and `conditions-audit.md`.
- [x] T003 Reconcile all 189 field rows with existing typed request fields, serializer order, current local validation, nearest test evidence, and shared schema use in `gap-matrix.md` and `model-audit.md`; record the source-backed MAC Data and Data Length serializer mismatches for test-first correction in T020–T022.

## Phase 2: Foundational validation model

**Purpose**: Define safe diagnostics and stable source references before any validator is added.

- [x] T004 [P] Add Red tests for deterministic reports and safe stable paths/codes/references in `crates/kmipkit-protocol/tests/unit/validation_report_tests.rs`.
- [x] T005 Implement public `ValidationReport`, `ValidationIssue`, `ValidationCode`, `ValidationSeverity`, path, and source-reference types in `crates/kmipkit-protocol/src/validation.rs` and re-export them from `crates/kmipkit-protocol/src/lib.rs`.
- [x] T006 Refactor path/report formatting helpers to keep metadata handling centralized without changing issue contents or ordering; record GREEN/REFACTOR evidence in separate commits for `crates/kmipkit-protocol/src/validation.rs` and `crates/kmipkit-protocol/tests/unit/validation_report_tests.rs`.

## Phase 3: User Story 1 — Inspect known schemas (Priority: P1)

**Goal**: Callers can validate typed requests and explicit-schema TTLV structures without guessing schemas or mutating request data.

**Independent Test**: Focused protocol tests cover valid/invalid type, order, requiredness, cardinality and locally decidable conditions; all 56 operation identities are accounted for; generic structures without identity do not get operation-specific guesses.

- [x] T007 [P] [US1] Add Red tests for explicit schema identities, unknown/unsupported identities, generic structures, field order/type/requiredness/cardinality, repeatable fields, and the unsupported client-initiated Set Endpoint Role scope in `crates/kmipkit-protocol/tests/unit/validation_schema_tests.rs`.
- [x] T008 [US1] Implement explicit Request Payload schema identities and descriptors for all 56 supported operations plus the explicitly deferred Set Endpoint Role identity. `tools/validation_schema/generate.py` deterministically generates descriptors from the audited field inventory and has a `--check` mode; it does not scrape OASIS or derive order from numeric tags.
- [x] T009 [US1] Implement borrowed `StructureView` validation against explicit operation Request Payload schemas, including value-free unsupported-schema reports, field type/order/requiredness/cardinality checks, and preservation of unknown fields in `crates/kmipkit-protocol/src/validation/schema.rs` and `validation_schema_tests.rs`.
- [ ] T009a [US1] Add explicit identities and borrowed recursive checks for reusable nested KMIP Structures identified by the audited schemas; only descend when the child schema is known, preserving unknown and vendor data.
- [ ] T010 [US1] Refactor schema descriptors and generic validation for one-responsibility modules without changing behavior in `crates/kmipkit-protocol/src/validation/`.
- [ ] T011 [P] [US1] Add Red tests for a common borrowed `ValidateRequest` contract and exhaustive validation identity for every supported request type in `crates/kmipkit-protocol/tests/unit/typed_request_validation_tests.rs`.
- [ ] T012 [US1] Implement the public borrowed request-validation trait and exhaustive 56-operation dispatch using the audited schemas in `crates/kmipkit-protocol/src/validation.rs`, operation request modules under `crates/kmipkit-protocol/src/`, and `crates/kmipkit-client/src/execute.rs`; for every operation, test either a local rule or a source-backed invariant-safe construction path.
- [ ] T013 [US1] Refactor typed request dispatch to remove duplication while preserving borrowed access, existing serializer behavior, and exhaustive-match guarantees in `crates/kmipkit-protocol/src/validation.rs` and `crates/kmipkit-client/src/execute.rs`.
- [ ] T014 [US1] Add Red tests for lifecycle and object operation rules (Activate, Archive, Certify, Check, Create, CreateKeyPair, CreateSplitKey, Destroy, Recover, Register, Revoke, Recertify, Rekey, RekeyKeyPair, ReProvision, Get) in their protocol unit tests under `crates/kmipkit-protocol/tests/unit/`.
- [ ] T015 [US1] Implement only table-backed lifecycle/object request rules in `crates/kmipkit-protocol/src/{activate,archive,certify,check,create,create_key_pair,create_split_key,destroy,recover,register,revoke,recertify,rekey,rekey_key_pair,reprovision,get}.rs`; extract checks from consuming serializers (including Certify, Create Split Key, and Register) into borrowed validators and reuse them from serialization.
- [ ] T016 [US1] Refactor lifecycle/object validators and update the same tests, preserving exact operation field order and no-clone behavior in `crates/kmipkit-protocol/src/{activate,archive,certify,check,create,create_key_pair,create_split_key,destroy,recover,register,revoke,recertify,rekey,rekey_key_pair,reprovision,get}.rs`.
- [ ] T017 [US1] Add Red tests for attribute, discovery, query, administrative, and diagnostic operations (AddAttribute, AdjustAttribute, DeleteAttribute, ModifyAttribute, SetAttribute, GetAttributes, GetAttributeList, Locate, GetUsageAllocation, GetConstraints, SetConstraints, SetDefaults, Query, DiscoverVersions, Interop, Ping, Log, Pkcs11) in `crates/kmipkit-protocol/tests/unit/`.
- [ ] T018 [US1] Implement table-backed local rules for attribute, discovery, query, administrative, and diagnostic operations in `crates/kmipkit-protocol/src/{add_attribute,adjust_attribute,delete_attribute,modify_attribute,set_attribute,get_attributes,get_attribute_list,locate,get_usage_allocation,get_constraints,set_constraints,set_defaults,query,discover_versions,interop,ping,log,pkcs11}.rs`.
- [ ] T019 [US1] Refactor these validators and tests while leaving server capabilities, authorization, state, and policy decisions to the server in the corresponding protocol modules and `crates/kmipkit-protocol/tests/unit/`.
- [ ] T020 [US1] Add Red tests for crypto, transfer, and entropy validation plus independent OASIS-literal serializer regressions for MAC Data (`0x4200C6`) and Data Length (`0x4200C4`) across request/response readers (Encrypt, Decrypt, DeriveKey, Export, Import, JoinSplitKey, Hash, Mac, MacVerify, Sign, SignatureVerify, RngRetrieve, RngSeed) in `crates/kmipkit-protocol/tests/unit/`.
- [ ] T021 [US1] Implement table-backed crypto, transfer, multipart, and entropy validation in `crates/kmipkit-protocol/src/{encrypt,decrypt,derive_key,export,import,join_split_key,hash,mac,mac_verify,sign,signature_verify,rng_retrieve,rng_seed}.rs`; extract consuming checks such as Derive Key field/method compatibility into borrowed validators and reuse KMIPKIT-0050 Cryptographic Parameters behavior without broadening it.
- [ ] T022 [US1] Refactor these validators while preserving secret redaction and existing TTLV serialization in the corresponding protocol modules and tests.
- [ ] T023 [US1] Add Red tests for session, asynchronous, lease, and certificate-chain requests (DelegatedLogin, Login, Logout, ObtainLease, Validate, Poll, Cancel, Process, QueryAsyncRequests) in `crates/kmipkit-protocol/tests/unit/`.
- [ ] T024 [US1] Implement table-backed validation for session, asynchronous, lease, and certificate-chain request types in `crates/kmipkit-protocol/src/{delegated_login,login,logout,obtain_lease,validate,poll,cancel,process,query_async_requests}.rs`; extract consuming checks such as Validate chain/certificate shape into borrowed validators and reuse KMIPKIT-0050 credential behavior without exposing credentials.
- [ ] T025 [US1] Refactor these validators and tests, preserving asynchronous correlation data, server policy boundaries, and redacted diagnostics in the corresponding protocol modules and tests.

## Phase 4: User Story 2 — Reject invalid requests before transport (Priority: P1)

**Goal**: Client execution shares the public validation result and rejects invalid requests with `NotSent` before encoding or transport exchange.

**Independent Test**: Scripted transport observes zero exchanges/writes for each invalid case and existing exact request bytes plus one exchange for valid cases.

- [ ] T026 [P] [US2] Add Red tests that execute invalid typed requests and invalid batches through `ScriptedTransport`, asserting sanitized errors, `RequestDeliveryState::NotSent`, zero exchanges, and zero writes in `crates/kmipkit-client/tests/unit/request_validation_tests.rs` and `crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs`.
- [ ] T027 [US2] Integrate the same request and batch validation into `Client::execute` before request message construction, encoding, and `transport.exchange` in `crates/kmipkit-client/src/execute.rs`; preserve existing delivery-state mapping.
- [ ] T028 [US2] Refactor preflight order and error adaptation; add byte-for-byte valid-request regression tests in `crates/kmipkit-client/tests/unit/request_validation_tests.rs` and `crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs`.

## Phase 5: User Story 3 — Preserve extension data and safe diagnostics (Priority: P2)

**Goal**: Unknown/future data remains available and no diagnostic exposes caller input or secret material.

**Independent Test**: Unknown tags and future enum/mask values round-trip unchanged; report, Display, and Debug output omit test secret canaries and encoded request bytes.

- [ ] T029 [P] [US3] Add Red tests for unknown tags, future enum and mask values, repeated extensions, credential/key canaries, and raw-body non-disclosure in `crates/kmipkit-protocol/tests/unit/validation_redaction_tests.rs` and `crates/kmipkit-client/tests/unit/request_validation_tests.rs`.
- [ ] T030 [US3] Implement lossless validation traversal and value-free report/error formatting in `crates/kmipkit-protocol/src/validation/` and `crates/kmipkit-client/src/execute.rs` without rejecting an item solely because its value/tag is unknown.
- [ ] T031 [US3] Refactor diagnostics and extension traversal while keeping all redaction and preservation tests green in the validation modules and focused test files.

## Phase 6: Polish and cross-cutting verification

**Purpose**: Close traceability, docs, source audit, and applicable quality checks.

- [ ] T032 Complete `specs/051-pre-send-validation/gap-matrix.md` with each audited field rule and its stable ID, implementation path, and executable test; complete 100% field-rule and functional-requirement mappings in `specs/051-pre-send-validation/traceability.md`.
- [ ] T033 Document Rust validation APIs, explicit schema use, server-side boundaries, and `NotSent` behavior in `docs/architecture/public-api.md`, crate rustdoc, and `docs/user-guide/en/` plus `docs/user-guide/es/`.
- [ ] T034 Run focused protocol/client validation tests, `cargo fmt --all --check`, workspace Clippy and tests, `python -B -m unittest scripts.tests.test_requirement_traceability -v`, `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v`, API manifest generation `--check`, and public Rust export audit; record exact results in `specs/051-pre-send-validation/traceability.md` and the draft PR description.

## Dependencies and execution order

- Setup/source inventory (T001–T003) precedes all implementation. Field-level OASIS review is a hard prerequisite because schema descriptors and tests must not be guessed.
- Foundational report types (T004–T006) precede schema-aware validation.
- User Story 1 schema registry (T007–T010) precedes typed request validation (T011–T025). Operation-family Red, Green, Refactor groups are serial within each family; different families may be reviewed independently after the shared interface is stable.
- User Story 2 pre-send integration (T026–T028) depends on the complete typed validator dispatch.
- User Story 3 preservation/redaction (T029–T031) depends on the report and traversal APIs; it may be tested while client integration is underway but must be green before final polish.
- T032–T034 depend on all implementation and tests being complete.

## Commit and implementation strategy

- Preserve distinct Red, Green, and Refactor commits for each TDD group: T004–T006; T007–T010; each operation-family triplet T014–T016, T017–T019, T020–T022, T023–T025; T026–T028; and T029–T031.
- Each Red commit contains only tests/spec evidence and records the intended failing command/output. Each Green commit contains the minimum production change needed to pass. Each Refactor commit keeps tests green and does not add behavior.
- No test in the matrix may be marked complete from serializer existence alone; cite the tested rule or a source-backed invariant that makes an invalid state unconstructible.
- The API-only validation behavior does not establish Baseline Client conformance or server acceptance.
