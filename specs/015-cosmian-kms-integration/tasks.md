# Tasks: Local Cosmian KMS Integration Tests

**Status**: In progress. The maintainer authorized the exhaustive 35-path matrix amendment on 2026-10-16. T001–T034 below are historical baseline work; they do not close the new matrix tasks. The maintainer approved earlier expanded operation coverage on
2026-10-09. The original ten-operation suite passed after syncing PR #56 with
`release/1.0.0`; on 2026-10-15 the scope expanded to twenty operation cases.
The latest live run passed 20/20 test cases: eighteen successful typed
operation exchanges and two expected limitation probes (Query and Ping). The
rerun included final test-helper and secret-buffer changes, and the local
Cosmian Compose deployment was stopped and verified empty afterward. Local
workspace lint, tests, and coverage passed; hosted CI for this expanded head
completed on 2026-10-15. All matrix and selected-scope coverage jobs passed;
the dependency-policy job failed on four newly issued RustSec advisories for
the existing `rustls 0.23.45` dependency. That dependency remediation is outside
KMIPKIT-0015 and requires a separately approved scope before the draft PR can
clear every review gate.

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md)

## Phase 1: Specification and test-first scaffold

- [x] T001 Write the bounded feature specification and exact OASIS clause references in `specs/015-cosmian-kms-integration/spec.md`.
- [x] T002 Record transport choice, local isolation, and the original deferred-verification plan in `specs/015-cosmian-kms-integration/research.md` and `plan.md`; update the evidence after live-test authorization.
- [x] T003 [US2] Author the ignored typed Discover Versions integration test first in `crates/kmipkit-client/tests/cosmian_kms.rs`. The user later authorized executing it.

## Phase 2: Local test infrastructure

- [x] T004 [US1] Add the pinned local Cosmian KMS configuration and loopback-only raw KMIP socket in `tests/integration/cosmian/compose.yaml` and `cosmian-kms.toml`.
- [x] T005 [US1] Add ephemeral CA/server/client certificate generation and mount only server credentials plus public client CA into Cosmian in `tests/integration/cosmian/generate-pki.sh`, `server.ext`, and `client.ext`.
- [x] T006 [US1] Exclude generated credentials from version control in `.gitignore` and add local lifecycle actions in `scripts/integration/cosmian-kms.ps1`.
- [x] T007 [US1, US2] Document start, status, logs, explicit test, and stop commands plus the one-operation scope in `tests/integration/cosmian/README.md` and `specs/015-cosmian-kms-integration/quickstart.md`.
- [x] T008 [US2] Link requirements to OASIS sections, code and live test evidence in `specs/015-cosmian-kms-integration/traceability.md`.

## Phase 3: Configuration and deployment-only verification

- [x] T009 Validate PowerShell syntax, execute the PKI generator during deployment, run `docker compose config`, `cargo fmt --all --check`, and `git diff --check` before the live KMIP test was authorized.
- [x] T010 Deploy Cosmian KMS 5.28.0 locally and inspect its container status/logs; then send one explicitly authorized Discover Versions request.
- [x] T011 Verify generated private keys are ignored and the Cosmian container mounts only its server credentials and public client CA. Leave the local server running for repeatable testing.

## Live interoperability investigation and verification

- [x] T012 Execute `cargo test -p kmipkit-client --test cosmian_kms -- --ignored`; record the initial failure, successful rerun, and image digest.
- [x] T013 Diagnose the failed response as KMIPKit expecting the Request Message root tag on the response path; add regression coverage and correct it to require the OASIS Response Message tag. Do not weaken the live assertion.
- [x] T014 Verify the focused raw-TLS and production-client suites and the live Cosmian test.
- [x] T015 Retain distinct Red, Green, and Refactor commits: `c74dbd1` (Red), `7094217` (Green), and `30c8fea` (Refactor).
- [x] T016 Run local workspace verification: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-features --locked --quiet`, and `git diff --check` all passed. The explicitly selected Cosmian integration test passed (1/1).
- [x] T017 Run the KMIPKIT-0013 continuation branch's required local checks and retain the original one-operation live evidence before the release branch advanced.

## Expanded typed-operation integration coverage

- [x] T018 Synchronize the feature branch with `release/1.0.0`, resolve the overlapping Response Message fixture changes, and retarget PR #56 to `release/1.0.0`.
- [x] T019 Update the specification, implementation plan, and traceability scope for ten selected typed operations; cite the exact OASIS v2.1 section and tables. The historical selected run treated Adjust Attribute as unsupported; the exhaustive probe later classified its malformed error response separately in the matrix.
- [x] T020 Add diagnostic test-first cases for Create, Create Key Pair, and Create Split Key in `crates/kmipkit-client/tests/cosmian_kms.rs`; run each through KMIPKit's typed client over the existing raw-TLS mTLS transport.
- [x] T021 Add Get Attributes and Get Attribute List cases using a fresh server-generated object per test; assert result success and the requested/listed attributes in the typed result.
- [x] T022 Add Add Attribute, Delete Attribute, Modify Attribute, and Set Attribute cases using independent server-generated objects; assert operation success and returned identifiers. Adjust Attribute stayed outside the advertised support set; its exhaustive probe is retained with the measured malformed response.
- [x] T023 Run the complete ignored Cosmian suite against the pinned local image, record the initial result and the rerun after syncing the separately reviewed KMIPKIT-0014 fixes from `release/1.0.0`. Keep production protocol fixes out of this integration PR.
- [x] T024 Update `specs/015-cosmian-kms-integration/traceability.md`, `quickstart.md`, `tests/integration/cosmian/README.md`, and bilingual client integration documentation with exact commands, operation clauses, and live outcomes.
- [x] T025 Run focused client/transport suites, `cargo fmt --all --check`, workspace Clippy, workspace tests, and local coverage; record commands and outcomes. Local client/transport tests, format, Clippy, workspace tests, Compose/syntax checks, immutable-source/audit/catalog/generated-output checks pass. The 2026-10-15 local `cargo llvm-cov --workspace --all-features --summary-only` report is 92.92% workspace line coverage; `kmipkit-protocol` is 96.17% and `kmipkit-ttlv` is 98.29%.
- [x] T026 Review every FR and SC against the expanded tests. The historical selected live suite passed 10/10 after syncing current release; the exhaustive suite is reported separately as 101 passed and 9 failed functions on the latest rustls 0.23.46 run. Hosted CI run `39007889527` passed on commit `056f45a5`; keep PR #56 in draft while measured interoperability findings and the second-server release gate remain open.
- [x] T027 Record the 10/10 Cosmian live result after syncing KMIPKIT-0014 corrections from release and update operation-level traceability and run instructions. Verified on merge head `a1945f4e`, then repeated after immutable image pinning on `9a6a6341` with `scripts/integration/cosmian-kms.ps1 -Action test`: 10 passed, 0 failed on both runs.

## KMIP 2.1 cryptographic and server-query coverage

- [x] T028 Add ignored typed tests for Activate, Query, Ping, Encrypt, Decrypt, Hash, MAC, MAC Verify, Sign, and Signature Verify, with exact OASIS v2.1 clause/table citations and isolated server-generated keys.
- [x] T029 Add regression tests for the normative MAC Data, Signature Data, and Validity Indicator TTLV tag assignments from OASIS KMIP v2.1 §11.56, Table 487; confirm they fail before changing production constants.
- [x] T030 Correct the three shared KMIP protocol tag constants and validate the typed cryptographic operations against Cosmian 5.28.0. Keep the implementation correction within the approved KMIPKIT-0021 scope.
- [x] T031 Preserve the historical Query/Ping baseline and reconcile it with the exhaustive probes: Query omits the Table 283 required Protection Storage Masks field; Ping and Adjust Attribute currently classify as malformed Operation Failed / Invalid Message responses (Status 1, Reason 4) because their error Batch Items omit Operation.
- [x] T032 Rerun the complete 20-case serial Cosmian suite after final test-helper and secret-buffer changes, capture the exact result, and stop/remove the local Cosmian Compose service after the run. Final merged head `5b86ab87`: 20 passed, 0 failed; Compose status was empty afterward.
- [x] T033 Update this task record, plan, specification, traceability, quickstart, and integration README with final run and server teardown evidence; run repository verification and retain distinct Red, Green, and Refactor commits.
- [x] T034 Run the hosted multi-platform CI and aggregate coverage gates on the expanded draft PR head and record the result. Run `38809840838` on `7df810ab`: core, bindings, scripts, traceability, immutable-source inventory, sanitizers, and the selected-scope coverage gate passed. Dependency policy failed because `rustls 0.23.45` has four RustSec advisories without registered exceptions; details and the out-of-scope follow-up are recorded in `traceability.md`. Keep the PR in draft until that gate is addressed under an approved dependency-security scope.


## Exhaustive source-derived matrix — approved 2026-10-16

The approved scope expands the earlier selected integration to every applicable object, format, encoding, algorithm, and parameter combination represented by Cosmian 5.28.0 documentation and pinned source, while retaining source disagreements and unrepresentable rows as findings. The per-case ledger is `tests/integration/cosmian/capability-matrix.json`; the evidence report is `specs/015-cosmian-kms-integration/task-6-report.md`. Counts below distinguish live typed interoperability from any other server response or test assertion.

- [x] T035 Amend `spec.md`, `research.md`, `plan.md`, `tasks.md`, `traceability.md` and `docs/development/kmip-interoperability.md` before test implementation. Establish the exact 35-path set/34-row grouping, normative data registry, valid intersections, Import/Export/PFX/encoding scope, source discrepancies, isolation and per-case reporting. Covers FR-011–FR-016 and the amended SCs at specification level.
- [x] T036 Record Red: the offline `cosmian_capability_manifest_is_complete` test and initially empty inventory failed on missing inventory in a signed-off Red commit. Green inventory now validates all 6,429 records, unique IDs, exact source links, 35 advertised operation paths (Set/Modify separate), Query and separate probes. Verified with `cargo test --offline -p kmipkit-client --test cosmian_kms cosmian_capability_manifest_is_complete -- --exact`. FR-011, FR-016; SC-003, SC-005.
- [x] T037 Build the pinned inventory and pass its validator. The 6,429 rows comprise 6,417 executable cases and 12 normative exclusions; every executable row now has an individual live classification or explicit fixture/setup blocker. Inventory includes applicable object, default/explicit format, vendor/fixture encoding, algorithm and parameter combinations with OASIS citations and source gaps/discrepancies. FR-011–FR-014, FR-016; SC-003, SC-005–SC-007.
- [x] T038 Add ignored independent lifecycle/object cases for Certify, Check, Destroy, Get, Locate, Register, Revoke and Validate, plus expanded operations and the separate Query/Ping/Adjust/Re-certify probes. The full run and focused follow-up runs record lifecycle results in the manifest; each case provisions its own state and cleanup ledger. FR-008–FR-012, FR-015–FR-016; SC-003–SC-005, SC-008.
- [x] T039 Add ignored Import/Export defaults, explicit formats, applicable PEM/DER paths and PrivateKey/PFX cases, retaining the PFX format-table omission. The full matrix records 4,640 transfer cases and distinguishes target replies, setup failures and response-validation failures. Password Link discrepancy and output-format postconditions are recorded per case. FR-005, FR-008, FR-012–FR-013, FR-015–FR-016; SC-005–SC-006, SC-008.
- [x] T040 Add ignored Derive Key, Join Split Key, Re-key and Re-key Key Pair cases plus source-confirmed generation/split/derivation/wrapping variants. The 216 key-management records have per-case outcomes: 12 interoperability successes, 201 target rejections and 3 setup rejections. FR-011–FR-016; SC-003–SC-008.
- [x] T041 Add ignored RNG Retrieve/Seed and the applicable cryptographic algorithm/parameter cases, including HMAC variants, Hash/SHA-224 discrepancy and matching encrypt/decrypt or sign/MAC verification pairs. Offline builders cover all 1,466 crypto rows; 15 exact fixture limitations remain explicitly `fixture_unavailable`. The pinned serial live run plus targeted probes yielded 616 crypto interoperability successes; 32 other typed crypto responses remain finding observations, 384 target rejections, 331 setup rejections, 88 response-validation failures, and 15 unavailable fixtures. All six valid RNG rows returned malformed typed responses; six standards-invalid RNG variants remain unsent exclusions. No failure was converted to success. FR-011–FR-016; SC-003–SC-008.
- [x] T042 Record Refactor: split `cosmian_kms` into focused lifecycle, cryptographic, random, key-management and transfer modules with shared typed-client/fixture/sanitized-outcome helpers. Offline parity checks cover every executable matrix row and all network tests remain ignored by default. Signed-off Red/Green/Refactor history is retained. FR-004, FR-008–FR-016; SC-002–SC-008.
- [x] T043 Update `quickstart.md`, the local integration README, `docs/development/kmip-interoperability.md` and English/Spanish user-guide entry points with full/selected commands, source gaps and measured outcomes. Historical selected results remain separate. The manifest/report link each case and count only successful typed postconditions as interoperability. FR-004, FR-006, FR-010, FR-016; SC-002–SC-007.
- [ ] T044 The latest complete ignored suite ran serially against the pinned local Cosmian 5.28.0 image with rustls 0.23.46 and captured per-case results; Compose teardown completed and `ps --all` was empty. It ended with 101 passed and 9 failed test functions after 6,383.81 seconds; the failures remain visible as target rejections, malformed/invalid responses, or operation-specific postcondition failures. Latest offline target tests pass (108 passed, 110 live tests ignored). Final-head workspace tests, Clippy, 92.93% workspace coverage, traceability, immutable-source, source audit, catalog/generated artifacts, script contracts, integration syntax/Compose teardown and fresh RustSec dependency policy pass locally. Hosted CI run `39007889527` passed on commit `056f45a5`; verify CI again after this final evidence update and keep PR #56 in draft. FR-001–FR-016; SC-001–SC-009.
- [x] T045 Correct the clean-checkout regression exposed by run `39006703175`: the offline probe inventory and certificate mapping tests must not require ignored runtime-generated Cosmian CSR/certificate fixtures. Red: Linux/macOS/Windows CI failed on missing fixtures. Green: commit `8e81b1bb` checks all 30 PQC CSR dependencies as a complete-or-absent set; with no fixtures, all 107 offline tests pass and preflight is 201 runnable / 45 `NotSent`; with generated fixtures it remains 231 / 15. Targeted Clippy and formatting pass. FR-016; SC-005.
