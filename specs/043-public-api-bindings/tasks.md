# Tasks: Full Public API Manifest and Binding Parity

**Input**: Design documents from `/specs/043-public-api-bindings/`

**Prerequisites**: `spec.md` and `plan.md` required; research, data model, quickstart, contracts, and traceability define the implementation contract.

**Organization**: Tasks are grouped by user story and implementation phase. An operation task is blocked until the exact operation's separate feature specification is approved and its canonical Rust API exists.

**TDD and commit evidence**: Before changing behavior, add the failing test; implement until it passes; then refactor. Keep Red, Green, and Refactor evidence in distinct development commits for every behavior change, and record commands/results in the implementation PR.

**Task numbering note**: T076–T078 were added after implementation review found shared API requirements not covered by the original task breakdown. Their dependency edges make them prerequisites for adapter and operation mappings; earlier task IDs are retained to preserve existing evidence references.

## Phase 1: Manifest v2 and Inventory (US1)

**Purpose**: Establish a complete, strict, reviewable 1.0 API contract without duplicating protocol semantics.

- [X] T001 [US1] Write failing v2 schema tests for required top-level fields, format version, scope, the exact eight public package IDs, declaration/exclusion records, language/API-level sets, and strict unknown-field rejection in `tools/api_manifest/tests/test_manifest_v2.py`.
- [X] T002 [US1] Write failing operation-inventory tests for exact 56-row `client_1_0` catalog equality, numeric section ordering, duplicate/missing/extra operations, and explicit rejection of the `client_1_1` Set Endpoint Role operation in `tools/api_manifest/tests/test_catalog_parity.py`.
- [X] T003 [US1] Write failing readiness tests for blocked/declared/verified transitions, approved owner-spec/ADR resolution, exact candidate/reason fields for blocked observed exports, complete package declaration mapping, required parity scope, and release rejection when any operation/capability/declaration remains blocked in `tools/api_manifest/tests/test_operation_readiness.py`.
- [X] T004 [US1] Implement the v2 JSON Schema in `specification/api/public-api.schema.json`, preserving registry-slice declarations and adding full-1.0 package and public declaration inventories, exact exclusion records, requirement-ID patterns, capability/type/function mappings, and operation readiness constraints.
- [X] T005 [US1] Migrate `specification/api/public-api.json` to v2 with exactly the five published Rust crate IDs plus C, Java, and Python distribution-family IDs; seed the full currently public declaration inventory, mark observed exports with unapproved owners as blocked candidates without signatures/mappings, and preserve all existing registry declarations, requirement references, error categories, limits, security notes, and outputs.
- [X] T006 [US1] Implement catalog and cross-record validation in `tools/api_manifest/generate.py`; compare all 56 in-scope operation IDs/names/sections/scope values, enforce exact public-package coverage, verify declaration-to-capability and approval-source references, and reject undeclared exports or unapproved parity exceptions without network access.
- [X] T007 [US1] Add the explicit blocked/approved-owner mapping represented by `contracts/operation-dependencies.md` to the manifest and validator; do not infer feature IDs from draft worktrees.
- [X] T008 [US1] Refactor the v2 validator and renderer so JSON ordering, filesystem enumeration, locale, timestamps, and environment paths cannot affect output; keep destinations fixed in generator code/schema.
- [X] T009 [US1] Generate the existing registry outputs unchanged in behavior and add the v2 public API Rust, C, Java, Python, and parity outputs at the fixed destinations in `contracts/manifest-v2.md`; ensure they derive only from declared/verified entries and never generate API signatures from blocked declaration records.
- [X] T010 [US1] Update `docs/development/api-manifest.md` and `specs/012-vendor-extension-registry/contracts/public-api-manifest.md` to explain the registry slice's preservation inside manifest v2 and the new full-1.0 contract owner.
- [ ] T011 [US1] Verify deterministic generator --write and --check, missing/stale output detection, path traversal/symlink rejection, exact catalog equality, exact package inventory, and Rust/C/Java/Python export-surface audits; require each public declaration to have a manifest record or exact approved exclusion. The Rust report has 9,497 matched paths (7,614 verified and 1,883 conditional) with zero discrepancies. The binding audit is reopened: its checked-in snapshot predates T076 promotion and the expanded Python inventory of generated dataclass members/signatures and inherited package-facing constructors. Regenerate the report and reconcile every observed path before completing T011. The C ABI allowlist remains configured across Windows, Linux, and macOS.

**Checkpoint**: One manifest represents the full 1.0 inventory; no blocked operation is advertised as an available capability.

## Phase 2: Shared Rust and C Contract (US2)

**Purpose**: Map already implemented, approved shared capabilities through the stable C ABI.

- [X] T012 [US2] Write failing Rust/C parity fixtures for shared API-level discovery, client construction, errors, batch outcomes, generic TTLV, and extension registry in `tests/fixtures/api/`.
- [X] T013 [US2] Inventory canonical declarations across all five published Rust crates and map only formally Approved/Accepted shared capabilities in `contracts/shared-dependencies.md` whose implementations exist, linking each declaration to owner requirements and retaining public docs; keep candidate drafts blocked and classify only the ADR-0014 transport exception as Rust-only.
- [X] T014 [US2] Implement or generate fixed-width C declarations for the shared capabilities in `crates/kmipkit-ffi/` and `bindings/c/include/kmipkit.h`, retaining existing exported symbols and ABI behavior. The generated header, FFI implementations, and ABI audit are complete; see T014 evidence in `implementation-evidence.md`.
- [X] T015 [US2] Define and implement manifest-backed C handle ownership, retain/release behavior, UTF-8/span lengths, result/error status mappings, and closed/wrong-kind/null handle validation. Existing Arc-owned opaque handles, checked kind/null access, explicit-length UTF-8 validation, stable error snapshots, and Java/Python wrapper closure guards are verified; the C caller represents a closed handle by clearing its local pointer to NULL after release. Stale C pointers after release remain caller-precondition violations under the existing ABI. See T015 evidence in `implementation-evidence.md`.
- [X] T016 [US2] Expose and test version/capability query functions, stable error categories, and delivery-state reporting through the C ABI without exposing raw secret or request payloads.
- [X] T017 [US2] Add failing-then-passing C consumer tests compiled and called from C for success, validation error, resource limit, closed handle, release ordering, and existing ABI compatibility. Model a closed C handle by clearing the caller's local pointer to NULL after release and asserting that NULL is rejected; never call through a freed pointer. The added NULL-sentinel characterization passed on Linux, Windows, and macOS CI; see T017 evidence in `implementation-evidence.md`.
- [X] T018 [US2] Refactor common FFI lifecycle/error code to one reviewed path; catch Rust panics at all 129 exported C functions as redacted `KMIPKIT_ERROR_INTERNAL` status 7; retain Rust `forbid(unsafe_code)` outside `kmipkit-ffi` and document every unsafe block with a `SAFETY` rationale.
- [X] T019 [US2] Update C API documentation and generated symbol inventory from the complete v2 manifest.

### Phase 2 follow-up: shared API-level metadata and typed client ABI

- [ ] T076 [US2] Complete the shared C ABI foundation for the approved Rust client: construct validated transport configuration and a production Client from separate transport and extension configurations; build ordered typed ClientBatch values; execute one batch with canonical CodecLimits; and expose ordered common result/outcome metadata, exact Pending correlation bytes, and sanitized error/delivery state. Map every accepted endpoint, trust, certificate/private-key, revocation-list, timeout, and request-limit input supported by KMIPKIT-0013. Preserve ABI 1.x; use fixed-width forward-compatible discriminators; do not accept an arbitrary Transport, raw requests, generic TTLV as an execution request, or an operation-number-plus-generic-payload shortcut. Do not retry, poll, wait, or issue follow-up operations automatically. The C, Java, and Python facade implementations and partial manifest/header promotion now exist; this task stays open until every shared declaration has exact approved four-language signatures/mappings, the public C consumer and parity evidence pass, the binding-export audit has no gaps, and required CI is complete. Keep kmipkit_client_execute_with_error_ttlv_v1 blocked. Per-operation constructors and typed responses remain T031-T069. Cite KMIPKIT-0005-FR-007, KMIPKIT-0006-FR-005-010/014-015, KMIPKIT-0007-FR-001/003/006/008-013, KMIPKIT-0012-FR-001, KMIPKIT-0013-FR-001-006/008-018, and KMIPKIT-0043-FR-003/005/007/012/013/019/021.

T076 checkpoint: the Rust/C runtime foundation and Java/Python shared client facades are present, and the manifest/generated C header contain a partial promotion of the shared client surface. Local Windows and Linux C consumer evidence passes. The latest pinned export snapshot exactly accounts for all 264 observed C paths, including 27 exact C-only exclusions; Java still has 16 and Python 80 observed paths without manifest classifications. The generic error-TTLV helper remains blocked. Exact export reconciliation, complete four-language parity evidence, and required CI are still open; do not mark T076 complete.

**Checkpoint**: Every shared capability currently backed by an approved Rust implementation has a safe, documented C mapping and passing C consumer evidence.

## Phase 3: Java 17 and Python 3.12 Adapters (US2, US3)

**Purpose**: Expose the shared Rust/C capabilities idiomatically and with semantic parity.

- [X] T020 [US2] Write failing Java 17 JNI parity and lifecycle tests from the shared fixtures before adding generated/facade declarations in `bindings/java/`. Java mappings in the fixture resolve to executable JUnit tests; API-level discovery and ordered batch outcomes remain explicitly blocked. Rust/C consumers remain planned and are not claimed by T020; see `implementation-evidence.md`.
- [ ] T021 [US2] After T076, implement Java 17 shared typed/high-level/TTLV mappings over the C ABI, including client construction, batch execution/results, `AutoCloseable` ownership, JSpecify nullness, stable exception categories, and redacted diagnostics.
- [ ] T022 [US2] Refactor Java wrapper ownership so a live child view safely retains its owner, close is deterministic, and invalid/closed inputs fail before JNI calls.

T021 remains open. The shared Java client/batch facade maps construction and execution, response metadata and outcome kinds, asynchronous indicator, request timestamp, batch ordering, continuation values, correlation bytes, unique batch item identity, and the shared Client/ClientBatch/ClientBatchResponse release lifecycle over the C ABI. ClientBatch and ClientBatchItem preserve explicit AutoCloseable ownership. The latest pinned audit still has 16 Java exports without manifest classifications, all in transport builder/timeout/encoding declarations; full nullness/parity, the JNI transferred-handle preflight ownership review, complete four-language T076 parity, and final CI remain acceptance gaps. Local evidence: `mvn --batch-mode --file bindings/java/pom.xml verify` passed 129 tests (3 documented platform skips) and the configured JaCoCo gate at the recorded checkpoint. Individual operation mappings remain T031-T069 work.
- [ ] T023 [US2] Write failing Python 3.12 CFFI parity, lifecycle, typing, and package tests from the shared fixtures before implementing the facade in `bindings/python/`.
- [ ] T024 [US2] After T076, implement Python 3.12 shared typed/high-level/TTLV mappings over the C ABI, including client construction, batch execution/results, complete annotations, `py.typed`, context managers, stable exceptions, and redacted diagnostics.

T024 implementation checkpoint: the Python client facade exposes the shared client, batch options, response metadata and outcome kinds, and unique batch item identity. The latest pinned audit still has 80 Python exports without manifest classifications, including public value constructors, fields, enum values, exception constructors, and inherited handle constructors. Local evidence: `bindings/python/tests` passed 152 tests under the built Python 3.12 package at the recorded checkpoint. Public-manifest/export reconciliation and complete parity remain open with T076/T029/T071. The expanded export scanner also surfaces inherited raw-handle constructors on Client, Batch, and BatchResult; their ownership/security review is unresolved and they must not be treated as product construction paths or closed by a mapping.
- [ ] T077 [US2] After T021 and T024, expose the local three-level API taxonomy (`high-level`, `typed-protocol`, `generic-ttlv`) consistently in Rust, C, Java, and Python with no network calls; test its output against `api-levels.shared.v1`. Make clear that this local API-surface query is distinct from the KMIP Discover Versions operation and does not assert that every operation mapping is complete. Cite KMIPKIT-0043-FR-005/010.
- [ ] T025 [US2] Refactor Python wrapper ownership and integer conversions so closed handles, out-of-range fixed-width values, and unsupported item types fail consistently with C and Java.
- [X] T026 [US3] Implement Java native-library selection, hash verification, private atomic extraction, and loading per `docs/architecture/ffi-and-bindings.md`; do not make network requests.
- [X] T027 [US3] Implement Python native-library loading adjacent to the installed wheel, allow only the documented explicit administrator path, and reject ABI mismatch without a download fallback.
- [ ] T028 [US3] Test Java/Python package loading offline on every platform in the existing CI matrix, including missing library, wrong architecture/ABI, and redacted error paths.
- [ ] T029 [US2] After T076–T078, compare shared fixtures through Rust, C, Java, and Python for API-level metadata, defaults, input ranges, result/status, batch order, unknown values, error category, secret redaction, local Usage Allocation guard accounting, and resource cleanup.
- [X] T030 [US3] Update Java/Python package metadata and user documentation to state synchronized official versions, supported runtimes, native loading behavior, and current capability scope.

**Checkpoint**: Every implemented shared capability resolves to all four language surfaces and passes parity and offline native-loading checks.

## Phase 4: APIs from Formally Approved Operation Specifications (US2)

**Purpose**: Bind only operations whose owner specification is marked Approved/Accepted in the active release line and whose canonical Rust implementation exists.

- [ ] T031 [US2] Add C, Java, and Python mappings plus parity fixtures for Discover Versions from approved `specs/007-client-execution/spec.md`; cite §6.1.16.
- [ ] T032 [US2] Add mappings and parity fixtures for Create, Create Key Pair, and Create Split Key from accepted `specs/014-server-generated-creation/spec.md`; cite §§6.1.8–6.1.10.
- [ ] T033 [US2] Add mappings and parity fixtures for Hash, MAC, MAC Verify, Sign, and Signature Verify from approved `specs/021-hash-mac-signature-operations/spec.md`; cite §§6.1.24, 6.1.32–6.1.33, and 6.1.55–6.1.56.
- [ ] T034 [US2] For T031–T033, use strict Red/Green/Refactor TDD with separate development commits for each phase; update manifest declarations and stable traceability, and preserve ABI compatibility and each owner specification's semantics.

**Checkpoint**: Only operations with formally Approved/Accepted owner specifications may become declared or verified; the current release inventory has 37 operation rows linked to approved owners and 19 rows linked to candidate owners, which remain blocked.

## Phase 5: Linked but Unapproved API Dependencies (US1, US2)

**Purpose**: Keep the 19 mapped-but-unapproved operation dependencies and the shared feature gates visible without treating implementation authorization as spec approval.

> **Hard prerequisite for each task below**: the linked specification is formally marked Approved/Accepted in the active release line, its scope is confirmed for 1.0, and its canonical Rust contract and implementation are available. A status that only authorizes autonomous implementation does not satisfy this gate.

- [ ] T035 [US2] After formal approval of `specs/009-asynchronous-operations/spec.md`, add mappings and parity fixtures for Cancel, Poll, Process, and Query Asynchronous Requests; cite §§6.1.5, 6.1.38–6.1.39, and 6.1.41; preserve exact correlation bytes and no automatic follow-up; resolve the specification's `KMIPKIT-DISC-039` gate before typed Query response exposure.
- [ ] T036 [US2] After formal approval of `specs/010-profile-conformance/spec.md` and closure of its applicable profile gates, add manifest mappings and four-language parity evidence for the accepted profile-validation API; do not claim profile support before its normative clauses and tests pass.
- [ ] T037 [US2] After formal approval of `specs/016-attribute-operations/spec.md`, add mappings and parity fixtures for Add Attribute, Adjust Attribute, Delete Attribute, Get Attribute List, Get Attributes, Modify Attribute, and Set Attribute; cite §§6.1.2–6.1.3, 6.1.13, 6.1.20–6.1.21, 6.1.34, and 6.1.51.
- [ ] T038 [US2] After formal approval of `specs/017-managed-object-retrieval/spec.md`, add mappings and parity fixtures for Get and Locate; cite §§6.1.19 and 6.1.28.
- [ ] T039 [US2] After formal approval of `specs/018-managed-object-lifecycle/spec.md`, add mappings and parity fixtures for Activate, Archive, Destroy, and Recover; cite §§6.1.1, 6.1.4, 6.1.15, and 6.1.42.
- [ ] T040 [US2] After formal approval of `specs/019-cryptographic-operations/spec.md`, add mappings and parity fixtures for Decrypt and Encrypt; cite §§6.1.11 and 6.1.17.
- [ ] T041 [US2] After formal approval of `specs/020-query-ping/spec.md`, add mappings and parity fixtures for Ping and Query; cite §§6.1.36 and 6.1.40.
- [ ] T042 [US2] For each task T035–T041, perform Red/Green/Refactor commits; update the operation's candidate owner to approved owner only after formal acceptance; keep the row blocked until all language tests and traceability pass.

**Checkpoint**: The 19 candidate-linked operation rows and the 0009/0010 shared capabilities advance only when their own specs are approved and implemented; unresolved source or scope issues remain visible blockers.

## Phase 6: Newly Approved Operation Owners (US1, US2)

**Purpose**: Add four-language API mappings for operations whose owner specifications were approved after the initial inventory; do not duplicate their protocol contracts.

> **Hard prerequisite for each task below**: its linked owner specification is formally approved/accepted in the active release line, includes the operation in the 1.0 scope, its canonical Rust contract and implementation are available, and T076's typed client/batch ABI exists. Until the four-language mapping is verified, the manifest operation row remains blocked.
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
- [X] T068 [US2] Resolve Set Endpoint Role (§6.1.54) as explicitly deferred to 1.1 by the normative catalog and approved specification 025; keep it out of the 1.0 API while generic TTLV preserves its open Operation enumeration value.
- [ ] T069 [US2] Add four-language API mapping and parity evidence for Validate after its approved contract exists; cite §6.1.57.
- [ ] T078 [US2] Map the approved local Usage Allocation guard capability from KMIPKIT-0039-FR-009 to the Rust, C, Java, and Python facades and public API manifest; preserve the exact caller-requested allocation count, caller-supplied unit/cost, resolved object-ID binding, non-cloneable/single-owner lifecycle, typed Encrypt/MAC/Sign-only scope, checked local reservation transitions, and explicitly local-only accounting. The caller remains responsible for requesting allocation during the known protection-enabled window; the guard must not infer or validate remote Activation Date or Protect Stop Date. Add generated declarations, executable parity and negative-boundary tests, and requirement traceability for `KMIPKIT-REQ-SPEC-6.1.23-001-001`, `-001-002`, `-002-001`, and `-002-002`. Depend on T076, Java T021, Python T024, and the typed operation mappings in T033 (MAC/Sign) and T040 (Encrypt), plus T049 for allocation construction; do not duplicate operation request/response contracts or claim remote consumption. Cite KMIPKIT-0039-FR-002/004/009, KMIPKIT-0043-FR-004/005/010/017/021, and OASIS KMIP v2.1 §§4.2.1, 4.59, 6.1.23, and 11.60 Table 491.

**Checkpoint**: All 56 `client_1_0` operations have approved owners and all four mappings; no blocked row remains. Set Endpoint Role remains a cataloged `client_1_1` operation and is excluded from this release gate.

## Phase 7: Release Parity Gate (US1, US2, US3)

**Purpose**: Verify the complete API contract and prepare it for human release review.

- [ ] T070 [US1] Add `--check --require-complete` release validation that fails unless T076–T078, all 56 in-scope operations, and all shared capabilities are verified; all eight package families are version-synchronized; every exported declaration is inventoried or explicitly excluded; and every requirement/type/function/error/output reference resolves.
- [ ] T071 [US2] After T076–T078 and T031–T069, run the shared parity corpus through Rust, compiled C, Java 17, and Python 3.12 on the complete API surface, including negative inputs, unknown values, asynchronous outcomes, local Usage Allocation guard boundaries, ownership, and redaction.
- [ ] T072 [US3] Run offline native package load/install tests across the existing platform matrix and verify no executable runtime download path.
- [ ] T073 [US1] Reconcile the public manifest, eight package identities/versions, exported-symbol inventory, fixed generated outputs, operation dependency matrix, compliance traceability, API reference, and English/Spanish user documentation.
- [ ] T074 [US1] Run the repository-required formatting, lint, workspace, coverage, security, compatibility, and generated-file checks at the applicable AGENTS.md thresholds; verify separate ordered Red/Green/Refactor development commits and DCO sign-off, and record commands/results plus approved exclusions in the draft PR.
- [ ] T075 [US1] Confirm T076–T078 are complete; zero blocked manifest operations/capabilities; 56/56 `client_1_0` inventory coverage; complete package export accounting; 100% requirement traceability; synchronized package versions; no unauthorized Rust-only surface; and no profile/certification claim without evidence before requesting human review.
