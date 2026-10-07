# Implementation Tasks: Server-Generated Object Creation

**Feature**: KMIPKIT-0014

**Status**: Draft for independent review. Implementation requires accepted spec and merged dependencies.
**Dependency gates**: Client execution changes wait for KMIPKIT-0010, KMIPKIT-0012, and KMIPKIT-0013 to merge. Public API manifest and generated language changes belong to a later API parity specification.

## Phase 1: Normative setup

- [ ] T001 Complete specs/014-server-generated-creation/traceability.md for every applicable OASIS clause, field, cardinality, error, stable requirement ID, implementation path, and executable test; record TC-CREATE-SD-1-21 (§2.12) as applicable to Create/Secret Data, verify the linked XML, pin a local fixture under tests/vectors/oasis/kmip-v2.1/ with exact source provenance and checksum (never under upstream/), and record that no direct Create Key Pair or Create Split Key case appears in the pinned work product.
- [ ] T002 Assign all three operation elements and applicable requirement records to KMIPKIT-0014 in specification/catalog/kmip-2.1.json, regenerate the coverage report with the pinned tool, and verify generated output.
- [ ] T003 Add Red tests for ordered and repeated Table 150 Attribute Structures, exact unknown Attribute Name Text Strings, generic Attribute Value types, absent/present-empty structures, malformed names/values, and redacted AttributeEntry Debug output with a byte-value sentinel in crates/kmipkit-protocol/tests/unit/attribute_tests.rs; commit as Red.
- [ ] T004 Implement the ordered Table 150 AttributeEntry model with exact Attribute Name strings, generic Attribute Values, and redacted public Debug output in crates/kmipkit-protocol/src/attribute.rs; export it and AttributeSet from crates/kmipkit-protocol/src/lib.rs; commit as Green.
- [ ] T005 Refactor attribute validation in crates/kmipkit-protocol/src/attribute.rs and redacted diagnostics; keep tests passing and commit separately as Refactor.

## Phase 2: User Story 1 - Create (P1)

- [ ] T006 [US1] Write Red protocol tests for all Table 186-188 fields, optionality, errors, and malformed responses in crates/kmipkit-protocol/tests/unit/create_tests.rs; prove no cryptographic attribute or parameter is synthesized, malformed-value diagnostics redact a sentinel, and the official TC-CREATE-SD-1-21 local fixture encodes/decodes to its specified Create/Secret Data exchange; commit as Red.
- [ ] T007 [US1] Implement CreateRequest and CreateResponse conversion in crates/kmipkit-protocol/src/create.rs and public exports; commit as Green.
- [ ] T008 [US1] Refactor Create parsing in crates/kmipkit-protocol/src/create.rs and validation while preserving unknown generic values; commit separately as Refactor.
- [ ] T009 [US1] Write Red fake-transport tests for one exchange, result association, success, Failure, Pending with exact operation identity/Result/correlation assertions, delivery-state errors, registered repeated Message Extensions, and no retry. Resolve KMIPKIT-0007 OD-006 with an operation-specific Client::execute lifecycle test: keep a secret-bearing request sentinel readable through simulated partial writes; after success and an injected post-write error, use the approved test observer to prove initialized request-owner bytes are zero before release; assert delivery state. Verify the secret sentinel and a raw response-body sentinel are absent from Debug, Display, returned errors, and captured logs. Add the tests to crates/kmipkit-client/tests/unit/create_execution_tests.rs and commit as Red.
- [ ] T010 [US1] Add ClientRequest::Create, typed ClientBatchOutcome support, and Client::create through the single writer in crates/kmipkit-client/src/execute.rs. Generalize PendingOutcome to retain operation identity, exact result, and correlation without a Discover Versions wrapper; commit as Green after KMIPKIT-0010, KMIPKIT-0012, and KMIPKIT-0013 merge.
- [ ] T011 [US1] Refactor Create dispatch in crates/kmipkit-client/src/execute.rs, redacted errors, and no-retry behavior; commit separately as Refactor.

## Phase 3: User Story 2 - Create Key Pair (P1)

- [ ] T012 [P] [US2] Write Red protocol tests for Tables 189-192, separate common/private/public groups, all four Table 191 values, absent effective values, common fallback, key-specific overrides, equal overrides differing from common, one-sided effective values, conflicting effective values, repeated attributes, malformed responses, no synthesized cryptographic choices, and redacted AttributeEntry Debug output in crates/kmipkit-protocol/tests/unit/create_key_pair_tests.rs; commit as Red.
- [ ] T013 [US2] Implement CreateKeyPairRequest and CreateKeyPairResponse in crates/kmipkit-protocol/src/create_key_pair.rs; preserve each group for server-side union behavior and validate Table 191 against effective values after §6.1.9 key-specific-over-common precedence exactly as defined in spec.md; commit as Green.
- [ ] T014 [US2] Refactor Table 191 validation and response parsing in crates/kmipkit-protocol/src/create_key_pair.rs; commit separately as Refactor.
- [ ] T015 [US2] Write Red fake-transport tests for both identifiers, errors, Pending with exact operation identity/Result/correlation assertions, batch behavior, registered repeated Message Extensions, and no retry. Resolve KMIPKIT-0007 OD-006 with an operation-specific Client::execute lifecycle test: keep a secret-bearing key-attribute sentinel readable through simulated partial writes; after success and an injected post-write error, use the approved test observer to prove initialized request-owner bytes are zero before release; assert delivery state. Verify secret and raw response-body sentinels are absent from Debug, Display, returned errors, and captured logs. Add the tests to crates/kmipkit-client/tests/unit/create_key_pair_execution_tests.rs and commit as Red.
- [ ] T016 [US2] Add ClientRequest::CreateKeyPair, its typed ClientBatchOutcome, and Client::create_key_pair through the single writer in crates/kmipkit-client/src/execute.rs; commit as Green after KMIPKIT-0010, KMIPKIT-0012, and KMIPKIT-0013 merge.
- [ ] T017 [US2] Refactor key-pair dispatch and result handling in crates/kmipkit-client/src/execute.rs; commit separately as Refactor.

## Phase 4: User Story 3 - Create Split Key (P2)

- [ ] T018 [P] [US3] Write Red protocol tests for every Table 193 field, optional input identifier, repeated Table 194 identifiers, Table 195 errors, malformed cardinalities, and no synthesized split method/part count/threshold in crates/kmipkit-protocol/tests/unit/create_split_key_tests.rs; commit as Red.
- [ ] T019 [US3] Implement CreateSplitKeyRequest and CreateSplitKeyResponse in crates/kmipkit-protocol/src/create_split_key.rs; preserve identifiers and wire order; commit as Green.
- [ ] T020 [US3] Refactor validation in crates/kmipkit-protocol/src/create_split_key.rs without implying request attributes override attributes of an input key; commit separately as Refactor.
- [ ] T021 [US3] Write Red fake-transport tests for one exchange, identifiers, errors, Pending with exact operation identity/Result/correlation assertions, delivery state, registered repeated Message Extensions, and no retry. Resolve KMIPKIT-0007 OD-006 with an operation-specific Client::execute lifecycle test: keep a secret-bearing request sentinel readable through simulated partial writes; after success and an injected post-write error, use the approved test observer to prove initialized request-owner bytes are zero before release; assert delivery state. Verify secret and raw response-body sentinels are absent from Debug, Display, returned errors, and captured logs. Verify Maximum Response Size is min(local response-byte limit, i32::MAX). Also return a response one byte over the configured local limit while ignoring the advertised peer value and prove the client rejects it before decoder entry using KMIPKIT-0007's approved decoder-counter seam; assert an exact-limit response remains accepted. Add the tests to crates/kmipkit-client/tests/unit/create_split_key_execution_tests.rs and commit as Red.
- [ ] T022 [US3] Add ClientRequest::CreateSplitKey, its typed ClientBatchOutcome, Client::create_split_key, and §9.12 Maximum Response Size derivation for any batch containing Create Split Key through the single writer in crates/kmipkit-client/src/execute.rs; commit as Green after KMIPKIT-0010, KMIPKIT-0012, and KMIPKIT-0013 merge.
- [ ] T023 [US3] Refactor split-key dispatch and result handling in crates/kmipkit-client/src/execute.rs; commit separately as Refactor.

## Phase 5: Traceability and validation

- [ ] T024 Update English and Spanish client guides with executable Rust examples for all three operations in docs/user-guide/en/client-execution.md and docs/user-guide/es/client-execution.md.
- [ ] T025 Update traceability.md with final symbols and test names; confirm every applicable clause and requirement links to code and executable evidence.
- [ ] T026 Run protocol/client and workspace coverage gates, format, Clippy, workspace tests, docs, catalog generation, dependency policy, immutable-source/generated-output checks, and Linux/Windows/macOS CI from the repository Cargo.toml and .github/workflows/ci.yml; record command results.
- [ ] T027 Run Spec Kit convergence on specs/014-server-generated-creation/tasks.md, resolve critical gaps, and include distinct Red, Green, Refactor commit evidence in the PR.
- [ ] T028 Obtain independent QA and security reviews of the implementation and evidence, resolve findings, retain reviewer records, and create a draft PR with scope, risks, limitations, and verification evidence.

## Dependencies and sequencing

T003-T005 establish the shared attribute model. Protocol slices T006-T008, T012-T014, and T018-T020 use independent modules and may proceed in parallel after the foundation. Changes to execute.rs are serialized. Client tasks T010, T016, and T022 wait for KMIPKIT-0010, KMIPKIT-0012, and KMIPKIT-0013. The public API manifest and generated language adapters are out of scope and wait for the API parity specification after all operation APIs are frozen. This feature remains incomplete until traceability, all validation gates, and independent QA/security reviews pass.
