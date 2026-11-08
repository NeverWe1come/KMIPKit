# Tasks: Typed Managed Objects and Key Material

**Input**: Design documents from `specs/049-managed-objects/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/managed-objects.md`, `traceability.md`

**Organization**: Tasks are grouped by user story. Tests are mandatory and must fail before production implementation. Red, Green, and Refactor evidence will be committed separately.

## Phase 1: Setup

**Purpose**: Establish reproducible source and branch evidence.

- [x] T001 Record the pinned OASIS/catalog baseline, predecessor PR #115, and authorized branch-base exception in `specs/049-managed-objects/implementation-evidence.md`.
- [x] T002 [P] Confirm the 53 operative requirement IDs and conditional exclusions in `specs/049-managed-objects/traceability.md` against `specification/catalog/kmip-2.1.json` and the pinned source; record source sections/tables and audit command in `specs/049-managed-objects/implementation-evidence.md`.

## Phase 2: Foundational typed-structure contract

**Purpose**: Shared lossless conversion, schema order, validation errors, and safe secret behavior used by all stories.

- [x] T003 Add failing contract tests for ordered typed-structure conversion, unknown/repeated child preservation, non-secret structural diagnostics, and redacted formatting in `crates/kmipkit-protocol/tests/managed_object_contract.rs`.
- [x] T004 Implement the private schema/field-order and typed conversion helpers in `crates/kmipkit-protocol/src/managed_object.rs`; preserve source order, generic unknown children, and `Value` ownership/zeroization.
- [x] T005 Refactor the shared helper API and document its schema and ownership invariants in `crates/kmipkit-protocol/src/managed_object.rs`; retain the passing tests from T003.

## Phase 3: User Story 2 — Key structures and material (Priority: P1)

**Goal**: Construct and inspect Key Block, Key Value, all source-defined Key Material forms, and wrapping metadata without performing cryptography.

**Independent Test**: Run the focused key-structure integration tests; verify exact source-defined field order/types/requiredness, wrapped/plain distinction, lossless conversion, and secret redaction.

### Red — tests first

- [x] T006 [US2] Add failing tests for Key Block required/optional fields, conditional algorithm/length pairing, wrapped/plain Key Value forms, and order in `crates/kmipkit-protocol/tests/key_structures.rs`.
- [x] T007 [US2] Add failing tests for Key Value and raw/opaque/PKCS/extension material representations, preserving byte-string versus Structure forms in `crates/kmipkit-protocol/tests/key_material.rs`.
- [x] T008 [US2] Add failing tests for Transparent Symmetric, DSA Private/Public, RSA Private/Public, DH Private/Public, and EC Private/Public Key Structures, required members, RSA private-key alternatives, and field order in `crates/kmipkit-protocol/tests/transparent_key_material.rs`.
- [x] T009 [US2] Add failing tests for Key Wrapping Data, Encryption Key Information, MAC/Signature Key Information, required identifiers, optional fields, and No Encoding constraints in `crates/kmipkit-protocol/tests/key_wrapping.rs`.

### Green — minimum implementation

- [x] T010 [US2] Implement typed Key Block and plaintext/wrapped Key Value construction and borrowed inspection in `crates/kmipkit-protocol/src/key_block.rs`; own wrapped bytes through `SecretBytes` from construction so dropping a standalone value zeroizes its payload before builder transfer.
- [x] T011 [US2] Implement open Key Material format handling, raw/opaque/PKCS and extension fallbacks, and typed material Structure conversion in `crates/kmipkit-protocol/src/key_material.rs`.
- [x] T012 [US2] Implement named types for Transparent Symmetric, DSA Private/Public, RSA Private/Public, DH Private/Public, and EC Private/Public Key Structures, including each source-defined field type and RSA private-key alternatives, in `crates/kmipkit-protocol/src/key_material.rs`.
- [x] T013 [US2] Implement typed Key Wrapping Data and its child Structures in `crates/kmipkit-protocol/src/key_wrapping.rs`.
- [x] T014 [US2] Export the key-structure API and errors from `crates/kmipkit-protocol/src/lib.rs`; add focused passing examples to `crates/kmipkit-protocol/tests/key_structures.rs`.

### Refactor — preserve behavior and API clarity

- [x] T015 [US2] Refactor common constructors, errors, and conversions across `crates/kmipkit-protocol/src/key_block.rs`, `crates/kmipkit-protocol/src/key_material.rs`, and `crates/kmipkit-protocol/src/key_wrapping.rs` without changing wire output.
- [x] T016 [US2] Add negative tests that confirm diagnostics and `Debug` output never contain secret bytes in `crates/kmipkit-protocol/tests/key_material.rs` and `crates/kmipkit-protocol/tests/key_wrapping.rs`.

## Phase 4: User Story 1 — Managed-object types (Priority: P1)

**Goal**: Construct and inspect all nine known object Structures, preserving generic fallback and every unknown descendant.

**Independent Test**: Run the managed-object tests for all nine types and unknown Object Type; assert exact tags, TTLV types, field order, requiredness, and round trips.

### Red — tests first

- [x] T017 [US1] Add failing vectors for Certificate (`0x00420013`), Certificate Request (`0x00420018`), and Opaque Object (`0x0042005B`) schemas, including their direct object-specific Item tags, in `crates/kmipkit-protocol/tests/managed_objects.rs`.
- [x] T018 [US1] Add failing vectors for PGP Key (`0x004200BD`), Private Key (`0x00420064`), Public Key (`0x0042006D`), and Symmetric Key (`0x0042008F`) schemas and tag/Object Type associations in `crates/kmipkit-protocol/tests/managed_objects.rs`.
- [x] T019 [US1] Add failing vectors for Secret Data and Split Key conditional fields and numeric relationships in `crates/kmipkit-protocol/tests/managed_objects.rs`.
- [x] T020 [US1] Add failing tests for unknown Object Type and vendor/repeated child preservation in `crates/kmipkit-protocol/tests/managed_objects.rs`.

### Green — minimum implementation

- [x] T021 [US1] Implement typed Certificate, Certificate Request, and Opaque Object models that emit their object-specific tags in `crates/kmipkit-protocol/src/managed_object.rs`.
- [x] T022 [US1] Implement typed PGP Key, Private Key, Public Key, and Symmetric Key models in `crates/kmipkit-protocol/src/managed_object.rs`.
- [x] T023 [US1] Implement typed Secret Data and Split Key models, including conditional Prime Field Size and Key Part Identifier bounds in `crates/kmipkit-protocol/src/managed_object.rs`.
- [x] T024 [US1] Implement the open typed/generic object dispatch, `Item` conversion, and public exports in `crates/kmipkit-protocol/src/managed_object.rs` and `crates/kmipkit-protocol/src/lib.rs`.

### Refactor — preserve behavior and API clarity

- [x] T025 [US1] Refactor object schema declarations and borrowed accessors in `crates/kmipkit-protocol/src/managed_object.rs`; confirm every known object emits OASIS order independent of builder call order.

## Phase 5: User Story 3 — Register/Import integration and secret lifecycle (Priority: P1)

**Goal**: Pass typed managed objects through existing operations while preserving generic APIs and KMIPKit-owned secret cleanup.

**Independent Test**: Use the fake transport to compare Register/Import TTLV output and verify redacted errors/debug output and owner drop behavior.

### Red — tests first

- [x] T026 [US3] Add failing fake-transport tests for typed Register objects, direct object-specific Item tags, and unchanged generic Register construction in `crates/kmipkit-client/tests/unit/register_execution_tests.rs`.
- [x] T027 [US3] Add failing fake-transport tests for typed Import objects, wrapped Key Value, Attributes, direct object-specific Item tags, and unchanged generic Import construction in `crates/kmipkit-client/tests/unit/import_execution_tests.rs`.
- [x] T028 [US3] Add failing tests for secret redaction, ownership transfer without a hidden copy, and borrow lifetimes in `crates/kmipkit-protocol/tests/managed_object_secrets.rs`; rely on the existing `crates/kmipkit-ttlv/tests/value_zeroization.rs` executable contract for drop-time zeroization rather than asserting freed memory from an integration test.

### Green — minimum implementation

- [x] T029 [US3] Add source-compatible typed-object helpers to `RegisterRequest` in `crates/kmipkit-protocol/src/register.rs`.
- [x] T030 [US3] Add source-compatible typed-object helpers to `ImportRequest` in `crates/kmipkit-protocol/src/import.rs`.
- [x] T031 [US3] Reuse existing TTLV secret ownership and add fake-transport/formatting behavior required by `crates/kmipkit-protocol/tests/managed_object_secrets.rs` without duplicating secret storage.

### Refactor — preserve behavior and API clarity

- [x] T032 [US3] Refactor typed operation helpers and ownership documentation in `crates/kmipkit-protocol/src/register.rs`, `crates/kmipkit-protocol/src/import.rs`, and `crates/kmipkit-protocol/src/managed_object.rs`; verify generic APIs remain source-compatible.

## Phase 6: Polish and cross-cutting verification

**Purpose**: Close traceability, manifest, examples, documentation, and local quality gates.

- [x] T033 Reconcile every new public Rust route with the reviewed API manifest input and regenerate outputs only with `tools/api_manifest/generate.py`; record the manifest audit in `specs/049-managed-objects/implementation-evidence.md`.
- [x] T034 Add compiling Rust examples for plaintext, wrapped, transparent, generic fallback, and secret lifetime to `crates/kmipkit-protocol/tests/managed_object_examples.rs` and `specs/049-managed-objects/quickstart.md`.
- [x] T035 Update the English user guide with typed managed-object and key material examples in `docs/user-guide/en/`.
- [x] T036 Update the Spanish user guide with equivalent examples and secret-lifetime limits in `docs/user-guide/es/`.
- [x] T037 Complete the 53-requirement implementation-to-test mapping and conditional-exclusion evidence in `specs/049-managed-objects/traceability.md` and `specs/049-managed-objects/implementation-evidence.md`.
- [x] T038 Run `cargo fmt --all --check`, focused protocol tests, `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D warnings`, and `cargo test --offline -p kmipkit-protocol --all-features`; record exact results in `specs/049-managed-objects/implementation-evidence.md`.
- [x] T039 Run `python -B tools/api_manifest/generate.py --check`, Rust and binding export audits, and applicable deterministic generated-output checks; record results in `specs/049-managed-objects/implementation-evidence.md`.
- [x] T040 Run the applicable Windows workspace coverage gates, including protocol, changed-line, and global thresholds; document generated-code exclusions and exact results in `specs/049-managed-objects/implementation-evidence.md`. Windows-local results pass: protocol 96.24%, Rust workspace 94.01%, changed production code 95.78%. Cross-platform aggregate evidence remains outstanding.
- [x] T041 Review the final diff against the spec, contracts, pinned OASIS tables, and every traceability row; resolve all findings and record the Red/Green/Refactor commit hashes in `specs/049-managed-objects/implementation-evidence.md`.
- [x] T042 Run a security-focused diff review and an acceptance-criteria QA review; record scope, findings, and remediation evidence in `specs/049-managed-objects/implementation-evidence.md`.
- [x] T043 Open the stacked draft PR from `feature/KMIPKIT-0049-managed-objects` to `feature/KMIPKIT-0048-typed-attributes` using `gh`; verify its state, base, head, and URL in the evidence file. At the 2026-11-07 pre-refresh snapshot, PR #116 was open/draft with base OID `a4fffdbac60563b0aeac5795902230df8e00039a` and head OID `28d209fd0a159376a8b7ca2b58c71704b37cc9a2`; the following evidence refresh is documentation-only. Keep it draft while hosted cross-platform controls remain outstanding.

## Dependencies and execution order

- T001–T005 establish source evidence and common typed-structure rules before model work.
- User Story 2 (T006–T016) precedes User Story 1 (T017–T025) because managed key objects depend on Key Block and Key Material types.
- User Story 3 (T026–T032) follows both model stories because Register/Import integration consumes their public types.
- Polish tasks T033–T043 require the complete public API and tests; the security and QA review must precede PR creation.
- Work remains sequential on this branch to preserve the explicit Red, Green, Refactor commit evidence and avoid shared-file conflicts.

## Implementation strategy

1. Commit the failing foundation and key-model tests as Red; run them and record expected compile/test failures.
2. Implement only enough code to make those tests pass and commit Green.
3. Refactor while keeping tests green and commit Refactor.
4. Repeat Red/Green/Refactor for managed objects and operation integration, updating the traceability matrix with each requirement.
5. Finish manifest generation, tested examples, bilingual docs, audits, coverage, and review before opening the stacked draft PR against PR #115.

## Independent test criteria

- **US1**: Nine known managed-object schemas and unknown fallback round-trip with exact field order and preserved extensions.
- **US2**: Plaintext/wrapped Key Block forms, all Key Material encodings, transparent structures, and wrapping metadata match pinned source rules without crypto operations.
- **US3**: Typed Register/Import requests emit the existing wire contract; generic constructors remain available; errors/debug output redact secrets and owned payloads retain zeroization semantics.
