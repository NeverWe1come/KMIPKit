# Cosmian 5.28.0 Exhaustive KMIP Integration Matrix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expand KMIPKIT-0015 so the ignored Rust integration suite exercises every valid KMIP 2.1 operation/object/format/encoding/algorithm/parameter combination Cosmian KMS 5.28.0 documents or implements, and records each live outcome without claiming unsupported interoperability.

**Architecture:** Keep Cosmian 5.28.0 and its image digest as the fixed target. Add a checked-in capability manifest as the case inventory, validate that inventory with non-network tests, and execute each row through KMIPKit's typed Rust `Client` over the existing raw TTLV TLS 1.3/mTLS deployment. Organize cases by operation family, provision independent disposable objects and fixtures per row, and update the approved feature specification, traceability, run guides, and observed results in the same change. Do not modify production protocol behavior in this feature; if the typed API cannot express a required request or a real defect is exposed, document the finding and seek its separately approved specification.

**Tech Stack:** Rust 2024 workspace, KMIPKit typed client and protocol models, pinned Cosmian KMS 5.28.0 Docker Compose deployment, PowerShell integration runner, JSON capability manifest validated with the workspace-pinned `serde_json`.

**Spec:** `specs/015-cosmian-kms-integration/spec.md`, amended to implement `docs/superpowers/specs/2026-10-16-cosmian-exhaustive-matrix-design.md`.

## Global Constraints

- The protocol is OASIS KMIP 2.1 only for the 1.x line; encoding is TTLV only.
- The 1.0 transport under test is raw TTLV over TLS 1.3 with mutual TLS, using rustls with aws-lc-rs.
- Rust is Edition 2024; initial MSRV is 1.94.
- All live integration tests remain ignored by default and run only against the loopback-published, image-digest-pinned Cosmian KMS 5.28.0 deployment.
- Every operation and normative combination cites the exact OASIS KMIP v2.1 section/table and receives stable KMIPKIT-0015 traceability IDs.
- Exclude standards-invalid intersections with exact normative reasons. Retain standards-valid combinations documented or implemented by the pinned release; documented combinations rejected or omitted by source remain explicit discrepancy probes, including Hash/SHA-224. Keep both citations and actual outcomes; expected rejection is never successful interoperability.
- Never print credentials, private keys, secret material, raw TTLV bodies, or local TLS private keys. Do not retry KMIP requests.
- Import only fixtures generated for this disposable test deployment; Destroy may target only a uniquely identified object created by the same test case.
- Do not edit immutable OASIS source copies or manually edit generated files. Do not change production APIs or protocol behavior without a separate approved specification.
- Keep distinct DCO-signed Red, Green, and Refactor development commits with actual commands and results recorded in the PR.
- After the authorized local live suite completes, including a failing run, stop and remove the Cosmian Compose deployment and confirm no service remains running.

---

## File Structure

| File | Responsibility |
|---|---|
| `specs/015-cosmian-kms-integration/spec.md` | Expand KMIPKIT-0015 scope, acceptance criteria, exclusions, and OASIS/vendor evidence for the complete matrix. |
| `specs/015-cosmian-kms-integration/tasks.md` | Track the approved implementation and record the final live run and teardown. |
| `specs/015-cosmian-kms-integration/plan.md` | Replace the selected-operation design with the exhaustive manifest-driven test architecture and verification sequence. |
| `specs/015-cosmian-kms-integration/research.md` | Record version-pinned support-source decisions, including Query, Set/Modify, and the PFX format-table omission. |
| `specs/015-cosmian-kms-integration/traceability.md` | Map each stable FR and OASIS clause to matrix rows, test names, and exact live observations. |
| `specs/015-cosmian-kms-integration/quickstart.md` | Give commands for the complete ignored matrix, one selected operation family, and teardown. |
| `tests/integration/cosmian/capability-matrix.json` | Store one stable row per valid intersection, including case ID, operation, object, algorithm and parameters, format and encoding, setup dependencies, source citations, expected classification, test family, and observed live result. |
| `crates/kmipkit-client/tests/cosmian_kms.rs` | Preserve the Cargo integration target name, shared verified-mTLS client setup, and imports for focused case modules. |
| `crates/kmipkit-client/tests/cosmian_kms/common.rs` | Hold shared case execution, unique identifier, fixture, sanitized outcome, and response assertion helpers. |
| `crates/kmipkit-client/tests/cosmian_kms/{lifecycle,transfer,key_management,random,cryptographic}.rs` | Hold ignored tests grouped by responsibility; each manifest row provisions its own state and reports its stable case ID. |
| `crates/kmipkit-client/Cargo.toml` | Add the already workspace-pinned `serde_json` as a dev dependency only if required by the manifest validator. |
| `tests/integration/cosmian/generate-pki.sh` | Generate any test-only certificate/key/PKCS#12 fixtures required by the Import/Export matrix into the existing ignored local directory. |
| `tests/integration/cosmian/README.md` | Describe matrix dimensions, local run commands, known/documented source gaps, and exact result classifications. |
| `docs/development/kmip-interoperability.md` | Reconcile the published Cosmian operation matrix with the separate live results, including empirical Modify Attribute coverage. |
| `docs/user-guide/en/production-transports.md`, `docs/user-guide/es/transportes-produccion.md` | Keep the English and Spanish local integration entry points accurate and link them to the expanded run guide. |

## Tasks

### Task 1: Amend KMIPKIT-0015 before implementation

**Files:** Modify `specs/015-cosmian-kms-integration/spec.md`, `research.md`, `plan.md`, `tasks.md`, `traceability.md`, and `docs/development/kmip-interoperability.md`.

**Interfaces:** The approved design is the scope input. The operation request/response citations for the sixteen not-yet-tested paths are: Certify §6.1.6 Tables 179–181; Check §6.1.7 Tables 183–185; Derive Key §6.1.14 Tables 205–207; Destroy §6.1.15 Tables 208–210; Export §6.1.18 Tables 217–219; Get §6.1.19 Tables 220–222; Import §6.1.25 Tables 238–240; Join Split Key §6.1.27 Tables 244–246; Locate §6.1.28 Tables 247–249; Re-key §6.1.46 Tables 303–307; Re-key Key Pair §6.1.47 Tables 310–312; Register §6.1.43 Tables 291–294; Revoke §6.1.44 Tables 295–297; RNG Retrieve §6.1.49 Tables 316–318; RNG Seed §6.1.50 Tables 319–321; Validate §6.1.57 Tables 340–342. Existing operation citations remain in the spec. Object, key format, encoding, algorithm, and parameter citations must be copied from the pinned OASIS v2.1 source files before their rows are accepted.

- [ ] Add stable requirements `KMIPKIT-0015-FR-011` through `FR-016`: all advertised operation paths (treat Set and Modify separately and Query as a known response defect); all valid managed-object intersections; all applicable Import/Export formats and encodings including source-confirmed PKCS#12/PFX; all source-confirmed algorithm and parameter variants; isolated per-case setup/cleanup; and complete case-level traceability/reporting.
- [ ] Replace the old exclusions of Import, Destroy, Get, Locate, and other supported paths. Keep Ping and Adjust Attribute as separately classified probes because the pinned release does not advertise/implement them as supported paths.
- [ ] State that Cosmian's combined `Set Attribute (Modify)` support row is one published row covering two KMIP operations, while the existing tests independently observed successful Set and Modify exchanges. Update `docs/development/kmip-interoperability.md` so it reports the published grouping and the live evidence without treating either as a broader conformance claim.
- [ ] State the exact PKCS#12 discrepancy: `objects.md` and `import.rs` describe/implement the `PrivateKey` PFX import path, while `formats.md` omits it; include the case and preserve that source note in the matrix.
- [ ] Cite each expanded operation and every new normative data dimension with exact OASIS v2.1 clauses/tables, and assign stable requirement IDs; do not infer normative requirements from Cosmian documentation.
- [ ] Revise success criteria so they require complete case inventory, typed KMIPKit requests, individually classified live outcomes, all ignored tests, no fake green interoperability claims, and stopped Compose services after the live run.
- [ ] Cross-check every new FR/SC against the approved matrix design and the existing `ClientRequest` API; do not change public API scope.
- [ ] Run the actual CI traceability contract commands from `.github/workflows/ci.yml`: `python3 -B -m unittest scripts.tests.test_requirement_traceability -v` and `python3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v`; inspect the full spec diff before test implementation. `docs/development/testing.md` does not document these checks.

### Task 2: Add a failing inventory-completeness check (Red)

**Files:** Create `tests/integration/cosmian/capability-matrix.json`; modify `crates/kmipkit-client/Cargo.toml` only if the existing workspace-pinned `serde_json` is not already available as a client dev dependency; add a manifest validator in `crates/kmipkit-client/tests/cosmian_kms.rs` or `tests/cosmian_kms/common.rs`.

**Interfaces:** Each JSON row has the fields `case_id`, `operation`, `object_type`, `algorithm`, `parameters`, `key_format`, `encoding`, `setup`, `oasis_source`, `cosmian_source`, `expected_outcome`, `test_family`, and `observed_result`. Use JSON `null` only when a dimension does not apply. `case_id` is stable and unique. The source inventory is pinned to Cosmian tag `5.28.0` and image digest `sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`.

- [ ] Add a test that deserializes the manifest and verifies JSON structure, required fields, unique IDs, valid expected-outcome values, and exact citation fields.
- [ ] In the same Red change, make the validator assert this explicit KMIP 2.1 supported path set: Activate, Add Attribute, Certify, Check, Create, Create Key Pair, Create Split Key, Decrypt, Delete Attribute, Derive Key, Destroy, Discover Versions, Encrypt, Export, Get, Get Attribute List, Get Attributes, Hash, Import, Join Split Key, Locate, MAC, MAC Verify, Modify Attribute, Query, RNG Retrieve, RNG Seed, Re-key, Re-key Key Pair, Register, Revoke, Set Attribute, Sign, Signature Verify, and Validate. Distinguish 34 support-table rows from 35 operation paths because `Set Attribute (Modify)` names two paths; keep Query present with its known response-shape failure classification. Ping is not in this expected supported set.
- [ ] Create the manifest initially as an empty JSON array so the integration target compiles; run the exact test `cargo test -p kmipkit-client --test cosmian_kms cosmian_capability_manifest_is_complete -- --exact` and capture the expected assertion failure listing missing operation paths/cases, without contacting the live server.
- [ ] Commit the failing inventory test with DCO sign-off and a `test:` message. This is the explicit Red evidence; do not label a passing pre-existing operation test as Red.

### Task 3: Build the pinned source inventory and make the manifest validator pass (Green)

**Files:** Modify `tests/integration/cosmian/capability-matrix.json`, validator in `crates/kmipkit-client/tests/cosmian_kms.rs`, and `specs/015-cosmian-kms-integration/research.md`.

**Interfaces:** Use OASIS KMIP v2.1 to determine normative validity and Cosmian KMS `5.28.0` documentation/`crate/server/src/core/operations/` to inventory vendor combinations. Exclude standards-invalid intersections and unsupported capabilities with exact source reasons, but retain standards-valid combinations explicitly documented by Cosmian as discrepancy probes when implementation rejects or omits them. A source/documentation gap must not silently remove a valid advertised case. Passing a check of expected rejection never counts as successful interoperability.

- [ ] Enumerate each advertised operation path and the compatible managed object types from `support.md`, `objects.md`, and pinned operation implementations. Include operation-specific object exclusions with their source evidence.
- [ ] Enumerate applicable `formats.md` defaults, every nonempty listed Import/Export format and source-implemented formats. Cover applicable PEM/DER fixture or documented vendor paths, recording normalization and actual wire representation separately. Preserve normative DER requirements (§2.1, Table 2; §11.25, Table 460); PEM is not a KMIP Key Format Type (§11.25, Table 461). Exclude any standards-invalid wire representation with its normative reason and retain the documentation discrepancy; do not send an invalid request. Add PrivateKey/PKCS#12 PFX import from `objects.md`/`import.rs` with its source-omission note.
- [ ] Enumerate each algorithm and parameter combination from the 5.28.0 algorithms document and operation source. Inspect operation code where documentation is incomplete, including HMAC variants in `mac.rs` and supported derivation parameters in `derive_key.rs`.
- [ ] For every algorithm, format, object and encoding dimension, retain each standards-valid documented or source-implemented intersection with exact OASIS and Cosmian file/section references. Include documented-but-source-gap rows such as Hash/SHA-224 as explicit discrepancy probes with both sources and individual live outcomes; source rejection or omission alone cannot exclude them. Classify any rejection as failed interoperability even if an expected-finding assertion passes. Exclude standards-invalid combinations with normative reasons.
- [ ] Add rows for every already-working operation/combination as well as the sixteen missing operation paths; this prevents the old selected-operation list from appearing exhaustive.
- [ ] Run the inventory test and confirm it now passes; run `cargo fmt --all --check` and `git diff --check`.
- [ ] Commit the complete passing source inventory with DCO sign-off and a `test:`/`feat(test):` message. This is the explicit Green evidence for the inventory contract.

### Task 4: Implement object and lifecycle matrix cases through the typed client

**Files:** Create `crates/kmipkit-client/tests/cosmian_kms/common.rs` and `lifecycle.rs`; modify the `cosmian_kms.rs` target root and manifest rows.

**Interfaces:** Shared case execution accepts a manifest case ID, creates a fresh `Client` using the existing certificate validation and mTLS configuration, sends one typed `ClientRequest`, and returns only sanitized typed response fields or KMIP status/reason. Each case creates its own server state; no case depends on test order.

- [ ] Move or expose only the current client/response helpers needed by focused modules; retain all 20 existing operation tests and their currently verified behavior.
- [ ] Add typed tests for Certify, Check, Get, Locate, Register, Revoke, Validate, and Destroy, using only object types and attributes present in their manifest rows.
- [ ] For every row, use a uniquely created server-side object; `Destroy` may target only the object created by that exact row. Do not share object IDs across cases.
- [ ] Assert successful typed result plus the operation-specific required response fields. If Cosmian rejects an advertised combination or KMIPKit rejects a response, report the case ID and sanitized status/shape as an interoperability failure; do not turn it into a passing limitation probe.
- [ ] Run the selected ignored lifecycle family against local Cosmian 5.28.0 with `--test-threads=1`; record the command and every case result.
- [ ] Commit the lifecycle tests and any test-only helper refactor with DCO sign-off after their assertions and response parsing are stable.

### Task 5: Implement import/export, key conversion, and split-key matrix cases

**Files:** Create or modify `crates/kmipkit-client/tests/cosmian_kms/transfer.rs` and `key_management.rs`; modify `tests/integration/cosmian/generate-pki.sh`, `.gitignore` only if required, and manifest rows.

**Interfaces:** Fixture generation writes only under ignored `.local/cosmian-kms/`; use KMIPKit `SecretBytes` for imported/exported secret buffers and never include buffer contents in output. Typed request models are the existing `ImportRequest`, `ExportRequest`, `JoinSplitKeyRequest`, `RekeyRequest`, `RekeyKeyPairRequest`, and `DeriveKeyRequest`.

- [ ] Add a manifest-driven typed Import/Export case for every standards-valid documented or source-implemented object/format/encoding/algorithm intersection. Cover applicable PEM/DER fixture/documentation paths without inventing a PEM Key Format Type or violating normative DER wire requirements; record fixture normalization and actual wire representation/format separately without recording payload bytes. Mark an inapplicable encoding dimension explicitly. Retain standards-valid documented source gaps as discrepancy probes and exclude standards-invalid wire claims with normative reasons.
- [ ] Generate an isolated test PFX/PKCS#12 fixture from test-only certificates and keys. Import it as the documented `PrivateKey` object, verify returned object identities/types, and retain the `formats.md` omission note in case and run documentation.
- [ ] Add typed cases for Re-key, Re-key Key Pair, Join Split Key, and Derive Key for every supported object/algorithm/parameter intersection in the manifest. Each row creates its own source and split components and asserts operation-specific outputs.
- [ ] Ensure cleanup and Destroy rows only refer to objects created by their own case; Compose teardown still removes all state after the full live run.
- [ ] Run the selected ignored transfer and key-management families serially against Cosmian. Record unsupported or malformed responses without weakening expected successful assertions.
- [ ] Commit the typed transfer/key-management cases and fixture code with DCO sign-off and redacted output review.

### Task 6: Implement random and cryptographic algorithm/parameter matrices

**Files:** Create or modify `crates/kmipkit-client/tests/cosmian_kms/random.rs` and `cryptographic.rs`; modify manifest rows and shared helper code only where an operation fixture requires it.

**Interfaces:** Use existing typed `ClientRequest` variants for RNG Retrieve, RNG Seed, Encrypt, Decrypt, Hash, MAC, MAC Verify, Sign, Signature Verify, Create, Create Key Pair, and Certify. Every row supplies explicit algorithm and parameters from the manifest; no algorithm default is inferred by the client.

- [ ] Add RNG Retrieve and RNG Seed cases for each valid documented parameter set. Use test-only seed input and assert only the typed result shape and requested length; never log output bytes.
- [ ] Add cryptographic round-trip/verification pairs across every applicable standards-valid documented or source-implemented algorithm/parameter combination, using matching test-owned keys, explicit modes/padding/digest/key sizes and `SecretBytes`. Documented source gaps remain individually classified discrepancy probes rather than omitted variants or successful interoperability.
- [ ] Expand Create/Create Key Pair/Certify coverage to each supported algorithm and key parameter combination needed by another matrix row; keep each row's prerequisites local to that row.
- [ ] Expand existing Hash, MAC, MAC Verify, Encrypt, Decrypt, Sign, Signature Verify tests over all applicable valid algorithms and parameter variants, including source-confirmed HMAC values absent from the algorithms page.
- [ ] Retain Query as a failing interoperability result because its successful response omits Table 283's required Protection Storage Masks. Retain Ping/Adjust Attribute as separate unadvertised probes; the exhaustive run measured malformed Operation Failed / Invalid Message responses because the error Batch Items omit Operation. Retain Re-certify as a separate safe source/documentation discrepancy probe (or explicit blocker), outside the 35 advertised supported paths.
- [ ] Run the random and cryptographic case families serially against the pinned server; capture case-level results and sanitized response evidence.
- [ ] Commit the typed algorithm-matrix cases with DCO sign-off.

### Task 7: Refactor the suite and validate matrix/test parity (Refactor)

**Files:** Modify `crates/kmipkit-client/tests/cosmian_kms.rs` and `crates/kmipkit-client/tests/cosmian_kms/*.rs`; add or update a parity validator in the test target; modify `tests/integration/cosmian/capability-matrix.json` only to correct verified inventory/results.

**Interfaces:** Every manifest `case_id` maps to exactly one exercised test case. Every test case maps back to exactly one manifest row. Failure diagnostics contain case IDs and non-sensitive result metadata only.

- [ ] Organize imports and helpers by operation family; remove duplicated client creation/response assertions only when behavior remains identical.
- [ ] Add parity assertions that fail when a matrix case has no executable case or a case references an unknown ID. Assert that operation, object, format, encoding, and algorithm dimensions used by a case match its manifest row.
- [ ] Run all offline manifest/parity checks and the complete ignored suite with `--test-threads=1` against the pinned local deployment.
- [ ] Review test output and error handling for leaked certificates, credentials, secret buffers, raw TTLV, retries, or shared server objects; correct any leak before committing.
- [ ] Commit the behavior-preserving Refactor with DCO sign-off and record separate Red, Green, and Refactor commit IDs and command results in `tasks.md`.

### Task 8: Update contributor documentation and observed interoperability evidence

**Files:** Modify `specs/015-cosmian-kms-integration/{spec.md,plan.md,research.md,tasks.md,traceability.md,quickstart.md}`, `tests/integration/cosmian/README.md`, `docs/development/kmip-interoperability.md`, and the English/Spanish production transport guides.

- [ ] Replace references to a twenty-operation selected suite with a precise description of the case manifest and its expanded dimensions.
- [ ] Document commands to start the pinned deployment, run all ignored cases serially, select a named test family, inspect sanitized results, stop the Compose services, and confirm empty status.
- [ ] Record every manifest case's observed outcome separately: successful typed response, Query response-shape nonconformance, or any actual incompatibility. Do not present expected rejection or a limitation assertion as successful operation interoperability.
- [ ] Update stable FR traceability to cite each OASIS clause and map every matrix ID to a test family and observed result. Preserve versioned Cosmian source URLs and the PFX omission.
- [ ] Update English and Spanish integration entry points and update every Cosmian row in `docs/development/kmip-interoperability.md` from “test pending” only when the matching live cases have actually run. Correct the Set/Modify note using both the published combined support row and existing separate live test evidence; link to the case matrix for combination-level results.
- [ ] Check all operation names and counts against the manifest and make no KMIP profile or certification claim.

### Task 9: Run the complete suite, shut down Cosmian, and close verification evidence

**Files:** Update `specs/015-cosmian-kms-integration/tasks.md`, `traceability.md`, `quickstart.md`, and `tests/integration/cosmian/README.md` with command evidence only after execution.

- [ ] Validate `docker compose config`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked --quiet`, and repository documentation/traceability checks.
- [ ] Start Cosmian only through `scripts/integration/cosmian-kms.ps1`; execute the full ignored target serially with `scripts/integration/cosmian-kms.ps1 -Action test` and retain per-case outcomes.
- [ ] Regardless of live test success or failure, run `scripts/integration/cosmian-kms.ps1 -Action down` and then `-Action status`; record evidence that no Compose service is running.
- [ ] Run `cargo llvm-cov --workspace --all-features --summary-only` and report its actual result without weakening the repository's coverage gates or attributing the pre-existing dependency advisory to this feature.
- [ ] Inspect `git diff --check`, ignored/generated artifacts, test logs, and final branch status; confirm generated PFX/private keys and database state are absent from tracked files.
- [ ] Leave the PR draft and do not merge or approve it. Record known Cosmian or KMIPKit incompatibilities and, when a production API/protocol change is needed, stop that portion and request a separate approved implementation specification.

## Self-Review

- **Spec coverage:** FR-011–FR-016 are addressed by Tasks 1–8. The exact sixteen missing operation clauses are listed in Task 1. Object, format, encoding, algorithm, and parameter cases are sourced and recorded in Tasks 2–3, executed in Tasks 4–6, parity-checked in Task 7, and reported in Task 8. Isolation and teardown are covered by Tasks 4–6 and 9.
- **Placeholder scan:** No TODO/TBD steps are present. Conditional branches state concrete stop conditions: no production fix is made without a separate approved specification; each vendor/source rejection is recorded with its case ID and exact sanitized result.
- **Type consistency:** Manifest fields and the typed request families are named in Tasks 2, 5, and 6. The test target remains `cosmian_kms`; helper modules are under `tests/cosmian_kms/`; case parity is verified against `case_id` in Task 7.
