# Tasks: Full Public API Manifest and Binding Parity

**Input**: Design documents from `/specs/043-public-api-bindings/`

**Prerequisites**: `spec.md` and `plan.md` required; research, data model, quickstart, contracts, and traceability define the implementation contract.

**Organization**: Tasks are grouped by user story and implementation phase. An operation task is blocked until the exact operation's separate feature specification is approved and its canonical Rust API exists.

**TDD and commit evidence**: Before changing behavior, add the failing test; implement until it passes; then refactor. Keep Red, Green, and Refactor evidence in distinct development commits for every behavior change, and record commands/results in the implementation PR.

## Phase 1: Manifest v2 and Inventory (US1)

**Purpose**: Establish a complete, strict, reviewable 1.0 API contract without duplicating protocol semantics.

- [X] T001 [US1] Write failing v2 schema tests for required top-level fields, format version, scope, the exact eight public package IDs, declaration/exclusion records, language/API-level sets, and strict unknown-field rejection in `tools/api_manifest/tests/test_manifest_v2.py`.
- [X] T002 [US1] Write failing operation-inventory tests for exact 57-row catalog equality, numeric section ordering, duplicate/missing/extra operations, and rejection of out-of-scope operations in `tools/api_manifest/tests/test_catalog_parity.py`.
- [X] T003 [US1] Write failing readiness tests for blocked/declared/verified transitions, approved owner-spec/ADR resolution, exact candidate/reason fields for blocked observed exports, complete package declaration mapping, required parity scope, and release rejection when any operation/capability/declaration remains blocked in `tools/api_manifest/tests/test_operation_readiness.py`.
- [X] T004 [US1] Implement the v2 JSON Schema in `specification/api/public-api.schema.json`, preserving registry-slice declarations and adding full-1.0 package and public declaration inventories, exact exclusion records, requirement-ID patterns, capability/type/function mappings, and operation readiness constraints.
- [X] T005 [US1] Migrate `specification/api/public-api.json` to v2 with exactly the five published Rust crate IDs plus C, Java, and Python distribution-family IDs; seed the full currently public declaration inventory, mark observed exports with unapproved owners as blocked candidates without signatures/mappings, and preserve all existing registry declarations, requirement references, error categories, limits, security notes, and outputs.
- [X] T006 [US1] Implement catalog and cross-record validation in `tools/api_manifest/generate.py`; compare all 57 operation IDs/names/sections/scope values, enforce exact public-package coverage, verify declaration-to-capability and approval-source references, and reject undeclared exports or unapproved parity exceptions without network access.
- [ ] T007 [US1] Add the explicit blocked/approved-owner mapping represented by `contracts/operation-dependencies.md` to the manifest and validator; do not infer feature IDs from draft worktrees.
- [ ] T008 [US1] Refactor the v2 validator and renderer so JSON ordering, filesystem enumeration, locale, timestamps, and environment paths cannot affect output; keep destinations fixed in generator code/schema.
- [ ] T009 [US1] Generate the existing registry outputs unchanged in behavior and add the v2 public API Rust, C, Java, Python, and parity outputs at the fixed destinations in `contracts/manifest-v2.md`; ensure they derive only from declared/verified entries and never generate API signatures from blocked declaration records.
- [ ] T010 [US1] Update `docs/development/api-manifest.md` and `specs/012-vendor-extension-registry/contracts/public-api-manifest.md` to explain the registry slice's preservation inside manifest v2 and the new full-1.0 contract owner.
- [ ] T011 [US1] Verify deterministic `--write` and `--check`, missing/stale output detection, path traversal/symlink rejection, exact catalog equality, exact package inventory, and export-surface audits for Rust, C, Java, and Python; require every public declaration to resolve to a manifest record (including an explicit blocked record where appropriate) or exact approved exclusion.

**Checkpoint**: One manifest represents the full 1.0 inventory; no blocked operation is advertised as an available capability.

## Phase 2: Shared Rust and C Contract (US2)

**Purpose**: Map already implemented, approved shared capabilities through the stable C ABI.

- [ ] T012 [US2] Write failing Rust/C parity fixtures for shared API-level discovery, client construction, errors, batch outcomes, generic TTLV, and extension registry in `tests/fixtures/api/`.
- [ ] T013 [US2] Inventory canonical declarations across all five published Rust crates and map only formally Approved/Accepted shared capabilities in `contracts/shared-dependencies.md` whose implementations exist, linking each declaration to owner requirements and retaining public docs; keep candidate drafts blocked and classify only the ADR-0014 transport exception as Rust-only.
- [ ] T014 [US2] Implement or generate fixed-width C declarations for the shared capabilities in `crates/kmipkit-ffi/` and `bindings/c/include/kmipkit.h`, retaining existing exported symbols and ABI behavior.
- [ ] T015 [US2] Define and implement manifest-backed C handle ownership, retain/release behavior, UTF-8/span lengths, result/error status mappings, and closed/wrong-kind/null handle validation.
- [ ] T016 [US2] Expose and test version/capability query functions, stable error categories, and delivery-state reporting through the C ABI without exposing raw secret or request payloads.
- [ ] T017 [US2] Add failing-then-passing C consumer tests compiled and called from C for success, validation error, resource limit, closed handle, release ordering, and existing ABI compatibility.
- [ ] T018 [US2] Refactor common FFI lifecycle/error code to one reviewed path; retain Rust `forbid(unsafe_code)` outside `kmipkit-ffi` and document every unsafe block with a `SAFETY` rationale.
- [ ] T019 [US2] Update C API documentation and generated symbol inventory from the complete v2 manifest.

**Checkpoint**: Every shared capability currently backed by an approved Rust implementation has a safe, documented C mapping and passing C consumer evidence.

## Phase 3: Java 17 and Python 3.12 Adapters (US2, US3)

**Purpose**: Expose the shared Rust/C capabilities idiomatically and with semantic parity.

- [ ] T020 [US2] Write failing Java 17 JNI parity and lifecycle tests from the shared fixtures before adding generated/facade declarations in `bindings/java/`.
- [ ] T021 [US2] Implement Java 17 shared typed/high-level/TTLV mappings over the C ABI, including `AutoCloseable` ownership, JSpecify nullness, stable exception categories, and redacted diagnostics.
- [ ] T022 [US2] Refactor Java wrapper ownership so a live child view safely retains its owner, close is deterministic, and invalid/closed inputs fail before JNI calls.
- [ ] T023 [US2] Write failing Python 3.12 CFFI parity, lifecycle, typing, and package tests from the shared fixtures before implementing the facade in `bindings/python/`.
- [ ] T024 [US2] Implement Python 3.12 shared typed/high-level/TTLV mappings over the C ABI, with complete annotations, `py.typed`, context managers, stable exceptions, and redacted diagnostics.
- [ ] T025 [US2] Refactor Python wrapper ownership and integer conversions so closed handles, out-of-range fixed-width values, and unsupported item types fail consistently with C and Java.
- [ ] T026 [US3] Implement Java native-library selection, hash verification, private atomic extraction, and loading per `docs/architecture/ffi-and-bindings.md`; do not make network requests.
- [ ] T027 [US3] Implement Python native-library loading adjacent to the installed wheel, allow only the documented explicit administrator path, and reject ABI mismatch without a download fallback.
- [ ] T028 [US3] Test Java/Python package loading offline on every platform in the existing CI matrix, including missing library, wrong architecture/ABI, and redacted error paths.
- [ ] T029 [US2] Compare shared fixtures through Rust, C, Java, and Python for defaults, input ranges, result/status, order, unknown values, error category, secret redaction, and resource cleanup.
- [ ] T030 [US3] Update Java/Python package metadata and user documentation to state synchronized official versions, supported runtimes, native loading behavior, and current capability scope.

**Checkpoint**: Every implemented shared capability resolves to all four language surfaces and passes parity and offline native-loading checks.

## Phase 4: APIs from Formally Approved Operation Specifications (US2)

**Purpose**: Bind only operations whose owner specification is marked Approved/Accepted in the active release line and whose canonical Rust implementation exists.

- [ ] T031 [US2] Add C, Java, and Python mappings plus parity fixtures for Discover Versions from approved `specs/007-client-execution/spec.md`; cite §6.1.16.
- [ ] T032 [US2] Add mappings and parity fixtures for Create, Create Key Pair, and Create Split Key from accepted `specs/014-server-generated-creation/spec.md`; cite §§6.1.8–6.1.10.
- [ ] T033 [US2] Add mappings and parity fixtures for Hash, MAC, MAC Verify, Sign, and Signature Verify from approved `specs/021-hash-mac-signature-operations/spec.md`; cite §§6.1.24, 6.1.32–6.1.33, and 6.1.55–6.1.56.
- [ ] T034 [US2] For T031–T033, use strict Red/Green/Refactor TDD with separate development commits for each phase; update manifest declarations and stable traceability, and preserve ABI compatibility and each owner specification's semantics.

**Checkpoint**: Only the nine operations with formally Approved/Accepted owner specifications may become declared or verified; all other operation rows remain blocked.

## Phase 5: Linked but Unapproved API Dependencies (US1, US2)

**Purpose**: Make the 21 mapped-but-unapproved operation dependencies and the two shared feature gates visible without treating implementation authorization as spec approval.

> **Hard prerequisite for each task below**: the linked specification is formally marked Approved/Accepted in the active release line, its scope is confirmed for 1.0, and its canonical Rust contract and implementation are available. A status that only authorizes autonomous implementation does not satisfy this gate.

- [ ] T035 [US2] After formal approval of `specs/009-asynchronous-operations/spec.md`, add mappings and parity fixtures for Cancel, Poll, Process, and Query Asynchronous Requests; cite §§6.1.5, 6.1.38–6.1.39, and 6.1.41; preserve exact correlation bytes and no automatic follow-up; resolve the specification's `KMIPKIT-DISC-039` gate before typed Query response exposure.
- [ ] T036 [US2] After formal approval of `specs/010-profile-conformance/spec.md` and closure of its applicable profile gates, add manifest mappings and four-language parity evidence for the accepted profile-validation API; do not claim profile support before its normative clauses and tests pass.
- [ ] T037 [US2] After formal approval of `specs/016-attribute-operations/spec.md`, add mappings and parity fixtures for Add Attribute, Adjust Attribute, Delete Attribute, Get Attribute List, Get Attributes, Modify Attribute, and Set Attribute; cite §§6.1.2–6.1.3, 6.1.13, 6.1.20–6.1.21, 6.1.34, and 6.1.51.
- [ ] T038 [US2] After formal approval of `specs/017-managed-object-retrieval/spec.md`, add mappings and parity fixtures for Get and Locate; cite §§6.1.19 and 6.1.28.
- [ ] T039 [US2] After formal approval of `specs/018-managed-object-lifecycle/spec.md`, add mappings and parity fixtures for Activate, Archive, Destroy, and Recover; cite §§6.1.1, 6.1.4, 6.1.15, and 6.1.42.
- [ ] T040 [US2] After formal approval of `specs/019-cryptographic-operations/spec.md`, add mappings and parity fixtures for Decrypt and Encrypt; cite §§6.1.11 and 6.1.17.
- [ ] T041 [US2] After formal approval of `specs/020-query-ping/spec.md`, add mappings and parity fixtures for Ping and Query; cite §§6.1.36 and 6.1.40.
- [ ] T042 [US2] For each task T035–T041, perform Red/Green/Refactor commits; update the operation's candidate owner to approved owner only after formal acceptance; keep the row blocked until all language tests and traceability pass.

**Checkpoint**: The 21 candidate-linked operation rows and the 0009/0010 shared capabilities advance only when their own specs are approved and implemented; unresolved source or scope issues remain visible blockers.

## Phase 6: Operations without an Approved Owner Specification (US1, US2)

**Purpose**: Reserve exact operation-level gates for the remaining 27 operations without assigning feature numbers or duplicating their protocol contracts.

> **Hard prerequisite for each task below**: a separate operation specification is created, assigned by the maintainer, formally approved/accepted, and explicitly includes the operation in the 1.0 scope; its canonical Rust contract and implementation exist. Until then, the inventory row remains blocked and no API signature is added.
>
> **TDD requirement**: Every implementation task in this phase uses strict Red/Green/Refactor TDD and keeps the failing test, passing implementation, and refactor in distinct development commits.

- [ ] T043 [US2] Add four-language API mapping and parity evidence for Certify after its approved contract exists; cite §6.1.6.
- [ ] T044 [US2] Add four-language API mapping and parity evidence for Check after its approved contract exists; cite §6.1.7.
- [ ] T045 [US2] Add four-language API mapping and parity evidence for Delegated Login after its approved contract exists; cite §6.1.12.
- [ ] T046 [US2] Add four-language API mapping and parity evidence for Derive Key after its approved contract exists; cite §6.1.14.
- [ ] T047 [US2] Add four-language API mapping and parity evidence for Export after its approved contract exists; cite §6.1.18.
- [ ] T048 [US2] Add four-language API mapping and parity evidence for Get Constraints after its approved contract exists; cite §6.1.22.
- [ ] T049 [US2] Add four-language API mapping and parity evidence for Get Usage Allocation after its approved contract exists; cite §6.1.23.
- [ ] T050 [US2] Add four-language API mapping and parity evidence for Import after its approved contract exists; cite §6.1.25.
- [ ] T051 [US2] Add four-language API mapping and parity evidence for Interop after its approved contract exists; cite §6.1.26.
- [ ] T052 [US2] Add four-language API mapping and parity evidence for Join Split Key after its approved contract exists; cite §6.1.27.
- [ ] T053 [US2] Add four-language API mapping and parity evidence for Log after its approved contract exists; cite §6.1.29.
- [ ] T054 [US2] Add four-language API mapping and parity evidence for Login after its approved contract exists; cite §6.1.30.
- [ ] T055 [US2] Add four-language API mapping and parity evidence for Logout after its approved contract exists; cite §6.1.31.
- [ ] T056 [US2] Add four-language API mapping and parity evidence for Obtain Lease after its approved contract exists; cite §6.1.35.
- [ ] T057 [US2] Add four-language API mapping and parity evidence for PKCS#11 after its approved contract exists; cite §6.1.37.
- [ ] T058 [US2] Add four-language API mapping and parity evidence for Re-certify after its approved contract exists; cite §6.1.45.
- [ ] T059 [US2] Add four-language API mapping and parity evidence for Re-key after its approved contract exists; cite §6.1.46.
- [ ] T060 [US2] Add four-language API mapping and parity evidence for Re-key Key Pair after its approved contract exists; cite §6.1.47.
- [ ] T061 [US2] Add four-language API mapping and parity evidence for Re-Provision after its approved contract exists; cite §6.1.48.
- [ ] T062 [US2] Add four-language API mapping and parity evidence for Register after its approved contract exists; cite §6.1.43.
- [ ] T063 [US2] Add four-language API mapping and parity evidence for Revoke after its approved contract exists; cite §6.1.44.
- [ ] T064 [US2] Add four-language API mapping and parity evidence for RNG Retrieve after its approved contract exists; cite §6.1.49.
- [ ] T065 [US2] Add four-language API mapping and parity evidence for RNG Seed after its approved contract exists; cite §6.1.50.
- [ ] T066 [US2] Add four-language API mapping and parity evidence for Set Constraints after its approved contract exists; cite §6.1.52.
- [ ] T067 [US2] Add four-language API mapping and parity evidence for Set Defaults after its approved contract exists; cite §6.1.53.
- [ ] T068 [US2] Add four-language API mapping and parity evidence for Set Endpoint Role after its approved contract exists; cite §6.1.54.
- [ ] T069 [US2] Add four-language API mapping and parity evidence for Validate after its approved contract exists; cite §6.1.57.

**Checkpoint**: All 57 operations have approved owners and all four mappings; no blocked row remains.

## Phase 7: Release Parity Gate (US1, US2, US3)

**Purpose**: Verify the complete API contract and prepare it for human release review.

- [ ] T070 [US1] Add `--check --require-complete` release validation that fails unless all 57 operations and all shared capabilities are verified, all eight package families are version-synchronized, every exported declaration is inventoried or explicitly excluded, and every requirement/type/function/error/output reference resolves.
- [ ] T071 [US2] Run the shared parity corpus through Rust, compiled C, Java 17, and Python 3.12 on the complete API surface, including negative inputs, unknown values, asynchronous outcomes, ownership, and redaction.
- [ ] T072 [US3] Run offline native package load/install tests across the existing platform matrix and verify no executable runtime download path.
- [ ] T073 [US1] Reconcile the public manifest, eight package identities/versions, exported-symbol inventory, fixed generated outputs, operation dependency matrix, compliance traceability, API reference, and English/Spanish user documentation.
- [ ] T074 [US1] Run the repository-required formatting, lint, workspace, coverage, security, compatibility, and generated-file checks at the applicable AGENTS.md thresholds; verify separate ordered Red/Green/Refactor development commits and DCO sign-off, and record commands/results plus approved exclusions in the draft PR.
- [ ] T075 [US1] Confirm zero blocked manifest operations/capabilities, 57/57 inventory coverage, complete package export accounting, 100% requirement traceability, synchronized package versions, no unauthorized Rust-only surface, and no profile/certification claim without evidence before requesting human review.
