# KMIPKIT-0050 Implementation Tasks

## Phase 1: Source audit and contract

- [x] T001 Verify pinned KMIP v2.1 §4.16/Table 59, §§8.1–8.3/Tables 394–396, §9.4, §10.1.2, and relevant §6.1 operation tables; record exact catalog IDs and source checksums. Source SHA-256 and exact Table 59 order are in `research.md`; catalog IDs are in `spec.md` and `traceability.md`.
- [x] T002 Re-run the 56-operation inventory against `ClientOperation`, `ClientRequest`, direct `Client` methods, async controls, and `_with_options` forms; update `gap-matrix.md` with final symbols and operation owner specs. Confirmed 56 operations, 52 typed `ClientRequest` variants, 4 async direct controls, 14 missing direct methods, and 10 missing option forms at parent `815f3cc351ac234004a8c0cd532a1a8ecee5cde0`.
- [x] T003 Trace secret ownership from `Authentication` through `ClientBatch`, request construction, TTLV ownership, encoding, transport, and drop; record invariants before changing code. The tree and wire-owner lifetime are recorded in `research.md`; tests will verify success and partial-write error at the candidate callsite per ADR-0012 Decision 6.
- [x] T004 Complete requirements checklist review and Spec Kit analysis; resolve all critical scope, source, ordering, or auth-lifecycle gaps before writing production code. Source analysis corrected the draft Table 59 ordering and incorporated ADR-0012's candidate-callsite lifecycle gate; see `research.md` and the checked requirements checklist.

## Phase 2: Typed Cryptographic Parameters (US1)

### Red

- [ ] T005 Add protocol tests for all 18 Table 59 fields, raw future enum values, unknown member preservation, typed accessors, and `with_ttlv`/`into_ttlv`; verify they fail because the public typed types do not exist.
- [ ] T006 Add negative tests for wrong TTLV types, duplicate known singleton fields, incorrect known-member order, and sanitized diagnostics; verify expected failures.
- [ ] T007 Add typed integration tests showing each existing operation/key-information setter can accept `CryptographicParameters` while its generic `Structure` form still compiles.

### Green

- [ ] T008 Implement `CryptographicParameters`, `CryptographicParametersBuilder`, typed accessors, generic TTLV bridge, canonical Table 59 order, and extension preservation in `crates/kmipkit-protocol/src/cryptographic_parameters.rs`.
- [ ] T009 Validate known field types/cardinality/order and preserve unknown values; reuse existing §4.16 IV Length and Tag Length validation without introducing server-dependent assumptions.
- [ ] T010 Add additive typed setters to Encrypt, Decrypt, Derive Key, `DerivationParameters`, Sign, Signature Verify, MAC, MAC Verify, Encryption Key Information, and MAC Signature Key Information; preserve existing generic setter signatures.

### Refactor

- [ ] T011 Refactor parameter validation into a single shared schema mapping for builder, parser, and operations without cloning secret-bearing TTLV.
- [ ] T012 Add compile-tested Rust parameter examples and document raw enumeration, unknown-field, and secret-byte lifetime behavior in both user guides.

## Phase 3: Per-message Authentication (US2)

### Red

- [ ] T013 Add fake-transport tests for absent Authentication, one-item Authentication, and a multi-item batch sharing one Request Header Authentication; confirm expected compile or behavior failure before implementation.
- [ ] T014 Add tests for empty/malformed generic Authentication rejected by `Authentication::try_from_ttlv` before it can be attached to a batch, repeated Credential order, and error/debug/log redaction. The safe typed API cannot represent empty Authentication; separately assert zero transport writes for any batch preflight failure that returns `NotSent`.
- [ ] T015 Add tests for owned secret copy/clone/drop and encoded request lifetime using repository-supported zeroization observers or test hooks. The candidate-callsite integration test MUST prove initialized request-owner bytes remain live through transport success and partial-write error, are zeroized after return, preserve correct delivery state, and are not retried, as required by ADR-0012 Decision 6.

### Green

- [ ] T016 Add optional `Authentication` to `ClientBatch`, defaulting to absent, and implement a consuming `with_authentication` builder.
- [ ] T017 Encode Authentication exactly once in Request Header field order and validate before transport I/O; preserve one-message scope across all batch items.
- [ ] T018 Ensure profile-clone and transient serialization paths keep credential ownership/redaction/zeroization; do not add any global client credential state.

### Refactor

- [ ] T019 Consolidate common Request Header construction and sanitized failure handling without changing operation payload bytes or delivery-state semantics.
- [ ] T020 Update API docs and bilingual examples to distinguish KMIP Request Header Authentication from TLS client authentication and to show explicit per-batch ownership.

## Phase 4: Complete direct Client methods (US3)

### Red

- [ ] T021 Add a compile-time consumer naming all 56 operation methods, including `_with_options` forms, and record the expected failures for the current 14 + 10 gaps.
- [ ] T022 Add fake-transport parity tests for the 14 missing direct methods against their one-item batch equivalents, including failure/delivery and asynchronous status cases where applicable.
- [ ] T023 Add option pass-through tests for the 10 existing methods that lack `_with_options`.

### Green

- [ ] T024 Implement missing default and `_with_options` methods for Discover Versions, Delegated Login, Login, Logout, Get, Locate, Add Attribute, Adjust Attribute, Delete Attribute, Modify Attribute, Set Attribute, Get Attributes, Encrypt, and Decrypt.
- [ ] T025 Add `_with_options` variants for Get Constraints, Get Attribute List, Set Constraints, Set Defaults, Get Usage Allocation, Validate, RNG Retrieve, Log, Interop, and RNG Seed.
- [ ] T026 Route each wrapper through the existing typed request/batch exchange path and preserve one exchange, response mapping, `RequestDeliveryState`, timeout options, Pending/correlation data, and no implicit retries or Poll.

### Refactor

- [ ] T027 Remove duplicated wrapper setup with private typed single-item helpers where signatures and error semantics remain explicit and readable.
- [ ] T028 Complete the final 56-row direct method inventory in the API audit and Rust quickstart; compile all examples.

## Phase 5: Manifest, examples, and traceability

- [ ] T029 Reconcile every new public Rust path with the reviewed API manifest, exact capability IDs, and narrow exact exclusions; regenerate outputs only with `tools/api_manifest/generate.py`.
- [ ] T030 Update the English guide with typed parameters, request-scoped Authentication, and direct-method examples.
- [ ] T031 Update the Spanish guide with equivalent examples and identical security limitations.
- [ ] T032 Complete normative and project-policy traceability with implementation paths and executable tests; update `traceability.md` and evidence.

## Phase 6: Verification and draft PR

- [ ] T033 Run focused parameter, Authentication, direct method, and negative tests; record commands and results.
- [ ] T034 Run `cargo fmt --all --check`, workspace/client/protocol Clippy, and relevant client/protocol/workspace tests.
- [ ] T035 Run available Windows workspace coverage, changed-code coverage, public API audits, manifest `--check`, and traceability checks; record exact percentages and any missing cross-platform artifacts.
- [ ] T036 Review secret handling, raw request diagnostics, generic TTLV preservation, all 56 mappings, and spec/plan/task agreement; resolve findings.
- [ ] T037 Run independent security and QA reviews, address findings, and record their scope and results. The ADR-0012 candidate-callsite lifecycle test must pass CI before the PR can be merged or enabled; retain draft status if hosted CI remains unavailable.
- [ ] T038 Commit Red, Green, and Refactor evidence in distinct commits for each implementation group.
- [ ] T039 Open a draft PR with `gh` from `feature/KMIPKIT-0050-parameters-auth-client` to `feature/KMIPKIT-0049-managed-objects`; verify URL, state, base, head, and commit in implementation evidence.

## Dependencies

- T001–T004 precede implementation.
- US1 parameter type tests/implementation (T005–T012) precede typed setter integration and can proceed before Authentication.
- US2 Authentication (T013–T020) depends on T003's secret-lifetime audit and uses the existing Credential models.
- US3 direct methods (T021–T028) reuses the existing operation types and common client execution path; it can be implemented after its inventory Red test.
- T029–T039 require all three stories complete. Security/QA review precedes PR creation.
