# KMIPKIT-0050 Implementation Tasks

## Phase 1: Source audit and contract

- [x] T001 Verify pinned KMIP v2.1 §4.16/Table 59, §§8.1–8.3/Tables 394–396, §9.4, §10.1.2, and relevant §6.1 operation tables; record exact catalog IDs and source checksums. Source SHA-256 and exact Table 59 order are in `research.md`; catalog IDs are in `spec.md` and `traceability.md`.
- [x] T002 Re-run the 56-operation inventory against `ClientOperation`, `ClientRequest`, direct `Client` methods, async controls, and `_with_options` forms; update `gap-matrix.md` with final symbols and operation owner specs. Confirmed 56 operations, 52 typed `ClientRequest` variants, 4 async direct controls, 14 missing direct methods, and 10 missing option forms at parent `815f3cc351ac234004a8c0cd532a1a8ecee5cde0`.
- [x] T003 Trace secret ownership from `Authentication` through `ClientBatch`, request construction, TTLV ownership, encoding, transport, and drop; record invariants before changing code. The tree and wire-owner lifetime are recorded in `research.md`; tests will verify success and partial-write error at the candidate callsite per ADR-0012 Decision 6.
- [x] T004 Complete requirements checklist review and Spec Kit analysis; resolve all critical scope, source, ordering, or auth-lifecycle gaps before writing production code. Source analysis corrected the draft Table 59 ordering and incorporated ADR-0012's candidate-callsite lifecycle gate; see `research.md` and the checked requirements checklist.

## Phase 2: Typed Cryptographic Parameters (US1)

### Red

- [x] T005 Add protocol tests for all 18 Table 59 fields, raw future enum values, unknown member preservation, typed accessors, and `with_ttlv`/`into_ttlv`; Red evidence is in `c4028f6e6` and `1b5381e3c`.
- [x] T006 Add negative tests for wrong TTLV types, duplicate known singleton fields, incorrect known-member order, and sanitized diagnostics; verified in `crates/kmipkit-protocol/tests/unit/cryptographic_parameters_tests.rs`.
- [x] T007 Add typed integration tests showing each existing operation/key-information setter can accept `CryptographicParameters` while its generic `Structure` form still compiles; verified in `crates/kmipkit-protocol/tests/cryptographic_parameters_typed_setters.rs`.

### Green

- [x] T008 Implement `CryptographicParameters`, `CryptographicParametersBuilder`, typed accessors, generic TTLV bridge, canonical Table 59 order, and extension preservation in `crates/kmipkit-protocol/src/cryptographic_parameters.rs` (`823eb040c`).
- [x] T009 Validate known field types/cardinality/order and preserve unknown values; reuse existing §4.16 IV Length and Tag Length validation without introducing server-dependent assumptions.
- [x] T010 Add additive typed setters to Encrypt, Decrypt, Derive Key, `DerivationParameters`, Sign, Signature Verify, MAC, MAC Verify, Encryption Key Information, and MAC Signature Key Information; preserve existing generic setter signatures (`50f0c045f`, `cc4cca605`).

### Refactor

- [x] T011 Refactor parameter validation into a single shared schema mapping for builder, parser, and operations without cloning secret-bearing TTLV.
- [x] T012 Add compile-tested Rust parameter examples and document raw enumeration, unknown-field, and secret-byte lifetime behavior in both user guides; all 22 marked Rust examples compile.

## Phase 3: Per-message Authentication (US2)

### Red

- [x] T013 Add fake-transport tests for absent Authentication, one-item Authentication, and a multi-item batch sharing one Request Header Authentication (`72bb37335`).
- [x] T014 Add tests for empty/malformed generic Authentication rejected by `Authentication::try_from_ttlv` before it can be attached to a batch, repeated Credential order, and error/debug/log redaction. The safe typed API cannot represent empty Authentication; zero transport writes are asserted for preflight failures (`72bb37335`).
- [x] T015 Add tests for owned secret copy/clone/drop and encoded request lifetime using the approved zeroization observers; the candidate-callsite tests cover success, partial-write error, delivery state, cleanup, and no retry (`311374bb3`).

### Green

- [x] T016 Add optional `Authentication` to `ClientBatch`, defaulting to absent, and implement a consuming `with_authentication` builder (`45dc27944`).
- [x] T017 Encode Authentication exactly once in Request Header field order and validate before transport I/O; preserve one-message scope across all batch items.
- [x] T018 Ensure profile-clone and transient serialization paths keep credential ownership/redaction/zeroization; do not add any global client credential state.

### Refactor

- [x] T019 Consolidate common Request Header construction and sanitized failure handling without changing operation payload bytes or delivery-state semantics (`f000939e5`).
- [x] T020 Update API docs and bilingual examples to distinguish KMIP Request Header Authentication from TLS client authentication and to show explicit per-batch ownership.

## Phase 4: Complete direct Client methods (US3)

### Red

- [x] T021 Add a compile-time consumer naming all 56 operation methods, including `_with_options` forms for the 52 typed payload operations and four asynchronous controls (`3c5927b3e`; final assertions in `crates/kmipkit-client/tests/unit/direct_method_surface_tests.rs`).
- [x] T022 Add fake-transport parity tests for all 14 missing direct methods against their one-item batch equivalents; encoded requests and `NotSent` state match in `crates/kmipkit-client/tests/unit/direct_method_behavior_tests.rs` (`3b8c5a025`, `f0a98a151`).
- [x] T023 Add option pass-through tests for the ten existing methods that lacked `_with_options`; exact options are observed for all ten in `direct_method_behavior_tests.rs` (`f0a98a151`).

### Green

- [x] T024 Implement missing default and `_with_options` methods for Discover Versions, Delegated Login, Login, Logout, Get, Locate, Add Attribute, Adjust Attribute, Delete Attribute, Modify Attribute, Set Attribute, Get Attributes, Encrypt, and Decrypt (`c3faccac9`).
- [x] T025 Add `_with_options` variants for Get Constraints, Get Attribute List, Set Constraints, Set Defaults, Get Usage Allocation, Validate, RNG Retrieve, Log, Interop, and RNG Seed (`c3faccac9`).
- [x] T026 Route each wrapper through the existing typed request/batch exchange path and preserve one exchange, response mapping, `RequestDeliveryState`, timeout options, Pending/correlation data, and no implicit retries or Poll.

### Refactor

- [x] T027 Remove duplicated wrapper setup with private typed single-item helpers where signatures and error semantics remain explicit and readable (`d70a1071b`).
- [x] T028 Complete the final 56-row direct method inventory in `gap-matrix.md` and link it from the Rust quickstart; compile all user-guide examples.

## Phase 5: Manifest, examples, and traceability

- [x] T029 Reconcile every new public Rust path with the reviewed API manifest, exact capability IDs, and narrow exact exclusions; outputs use only `tools/api_manifest/generate.py`.
- [x] T030 Update the English guide with typed parameters, request-scoped Authentication, and direct-method examples.
- [x] T031 Update the Spanish guide with equivalent examples and identical security limitations.
- [x] T032 Complete normative and project-policy traceability with implementation paths and executable tests; update `traceability.md`, `gap-matrix.md`, and evidence.

## Phase 6: Verification and draft PR

- [x] T033 Run focused parameter, Authentication, direct method, and negative tests; record commands and results in `implementation-evidence.md`. The parameter contract target passed all 9 tests; the full workspace test and coverage runs passed.
- [x] T034 Run `cargo fmt --all --check`, workspace/client/protocol Clippy, and relevant client/protocol/workspace tests; record results in `implementation-evidence.md`.
- [x] T035 Run available Windows workspace coverage, changed-code coverage, public API audits, manifest `--check`, and traceability checks; record exact percentages and missing platform artifacts in `implementation-evidence.md`. After applying the gate's documented generated-source exclusions, all local Windows Rust thresholds pass, including protocol at 96.13% and changed executable lines at 95.91%. Subsequent Linux workspace and adapter coverage on the synchronized stack is recorded in `specs/053-api-parity-facade/implementation-evidence.md`; Windows/macOS adapter artifacts and the complete cross-platform aggregate remain unverified, so the PR remains draft.
- [x] T036 Review secret handling, raw request diagnostics, generic TTLV preservation, all 56 mappings, and spec/plan/task agreement; no unresolved implementation finding in the coordinator review. Independent review results are recorded under T037.
- [x] T037 Run independent security and QA reviews, address findings, and record their scope and results. Both reviews found no confirmed defect after the coverage calculation was reconciled. The security reviewer noted an unconfirmed local-input memory amplification concern because Authentication is cloned before encoder limits; no remote path or exploitability was established. Both ADR-0012 candidate-callsite lifecycle tests pass locally (`cargo test --offline -p kmipkit-client --lib authentication_owner`); hosted CI must still run and pass them before merge, so retain draft status while that result is unavailable.
- [x] T038 Commit Red, Green, and Refactor evidence in distinct commits for each implementation group; commit IDs are listed in `implementation-evidence.md`.
- [x] T039 Open a draft PR with `gh` from `feature/KMIPKIT-0050-parameters-auth-client` to `feature/KMIPKIT-0049-managed-objects`; verify URL, state, base, head, and commit in implementation evidence. At the initial 2026-11-07 audit snapshot, PR #118 was open/draft at base OID `28d209fd0a159376a8b7ca2b58c71704b37cc9a2` and head OID `ecbd9c978c625ba3648e49f74149287b54ae6c09`; the original PR-open head `4f23fba09f791dbbd7f510188151a3683fef1dfb` is historical. The #116 evidence refresh (`77b3d91ca`) was then merged here through documentation-only merge `79277e78f`; hosted platform/CI gates remain outstanding.

## Dependencies

- T001–T004 precede implementation.
- US1 parameter type tests/implementation (T005–T012) precede typed setter integration and can proceed before Authentication.
- US2 Authentication (T013–T020) depends on T003's secret-lifetime audit and uses the existing Credential models.
- US3 direct methods (T021–T028) reuses the existing operation types and common client execution path; it can be implemented after its inventory Red test.
- T029–T039 require all three stories complete. Security/QA review precedes PR creation.
