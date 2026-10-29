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
- [x] T041 Add ignored RNG Retrieve/Seed and the applicable cryptographic algorithm/parameter cases, including HMAC variants, Hash/SHA-224 discrepancy and matching encrypt/decrypt or sign/MAC verification pairs. Offline builders cover all 1,466 crypto rows and all 246 exact finding probes. The reconciled pinned live matrix plus targeted follow-ups records 650 crypto interoperability successes; 20 other typed crypto responses remain finding observations, 383 target rejections, 325 setup rejections, 81 response-validation failures, and 7 Hash multipart semantic mismatches. Create Key Pair contributes 57/59 observed interoperation cases, including 24 source-discrepancy probes whose typed responses contained both validated key identifiers. The targeted follow-up measured 12 ConfigurableKEM Encrypt/Decrypt combinations as typed results and three RSA/AES probes as target rejections. The later review fixed three AES-ECB fixture lengths for Encrypt and Decrypt and CoverCrypt result framing; the three standalone Encrypt ECB rows now pass and Encrypt records 287/300 interoperable cases; two pure-classical ConfigurableKEM setup cases remain rejected before Encrypt/Decrypt is sent, and the RSA/AES Decrypt probe is a target rejection with an ordinary RSA-OAEP fixture. All six valid RNG rows returned typed responses missing Operation and are classified as response-validation failures; six standards-invalid RNG variants remain unsent exclusions. No target rejection, response-validation failure, or semantic mismatch was converted to success. FR-011–FR-016; SC-003–SC-008.
- [x] T042 Record Refactor: split `cosmian_kms` into focused lifecycle, cryptographic, random, key-management and transfer modules with shared typed-client/fixture/sanitized-outcome helpers. Offline parity checks cover every executable matrix row and all network tests remain ignored by default. Signed-off Red/Green/Refactor history is retained. FR-004, FR-008–FR-016; SC-002–SC-008.
- [x] T043 Update `quickstart.md`, the local integration README, `docs/development/kmip-interoperability.md` and English/Spanish user-guide entry points with full/selected commands, source gaps and measured outcomes. Historical selected results remain separate. The manifest/report link each case and count successful typed postconditions as observed interoperability while keeping source-only values separate from KMIP 2.1 conformance claims. FR-004, FR-006, FR-010, FR-016; SC-002–SC-007.
- [x] T044 Execute and reconcile the exhaustive Cosmian 5.28.0 matrix and final local checks. The
  complete serial run with rustls 0.23.46 ended with 101 passed and 9 failed test functions after
  6,383.81 seconds; all outcomes remain per-case in the ledger. The targeted follow-up measured the
  15 former cipher fixture blockers (12 typed ConfigurableKEM results and three AES/RSA target
  rejections), exit code 0; both runs completed Compose teardown with no services. A later focused
  Get/Export run selected 66 legacy PKCS#12 cases: 62 target requests passed typed postconditions
  and four password-linked Import setups were rejected before the target request. Adding the missing
  vendor format selector resolved the 50 legacy-PKCS#12 response-validation findings. Offline
  Cosmian tests: 118 passed, 110 live tests ignored. Workspace tests and Clippy pass; workspace
  coverage is 92.93%; requirement/Compose contracts pass. Hosted CI run 39049893049 passed on the
  exact pre-correction matrix head 5c3e4952be02422bff7149256efc617cea12692a; this branch adds a
  focused source-citation regression test, and the corrected citations must pass ordinary PR CI on
  the updated head. Keep
  PR #56 in draft; the independent-server release gate remains open. FR-001–FR-016; SC-001–SC-009.

## KMIP 2.1 multipart Hash correction — authorized 2026-10-17

- [x] T046 Add the approved FR-017 scope amendment and Red/Green/Refactor coverage. Red commit
  b3829db1 adds protocol/client contract tests that failed against the old multipart validator;
  Green commit 29ac4b51 makes multipart cryptographic output optional/preservable, allows
  caller-provided input data on multipart requests while retaining single-part output checks and
  framing validation, and fixes the Hash test to send a, b, and c while reusing the first response's
  Correlation Value. OASIS KMIP v2.1 §6.1 defines multipart sequencing, §7.8 defines Correlation
  Value reuse, and Hash §6.1.24, Table 235 marks Data as not required for multipart Hash, while
  §6.1 says Data is always required except when Init or Final Indicator is true. The project makes
  no conformance claim for the disputed intermediate omission, and the test supplies Data on every
  part. The selected pinned Cosmian
  5.28.0 run accepts the typed responses but all seven multipart algorithms return a final digest
  different from the independent abc known-answer vector; three SHA-224 cases remain Operation Not
  Supported as the pinned source omits SHA-224 despite its documentation. The seven multipart rows
  are semantic_mismatch; the three SHA-224 rows are target rejections. All 10 observations are
  retained in the manifest; no failure is counted as interoperability. Verification passed: cargo
  fmt --all --check, protocol unit tests (723), focused client unit tests (12), offline Cosmian
  tests (118 passed, 110 live ignored), full workspace tests, Clippy, and workspace coverage (92.93%
  overall; every touched protocol model is 95.36% or higher). FR-017; SC-010.

## Static-analysis follow-up — 2026-10-29

- [ ] T047 Review and resolve the two CodeQL alerts on the Cosmian test matrix. Red commit
  `8d7c32168` added a freshness regression that failed against the fixed nonce. Green commit
  `c014b2e97` uses bounded AWS-LC system randomness for caller nonces and replaces dynamic
  preflight error output with fixed, redacted diagnostics. `cargo fmt --all --check`, the focused
  regression, the full Cosmian test binary (122 passed, 110 live tests ignored), and client Clippy
  passed locally. The source review found no production secret exposure or nonce reuse under the
  same case key; hosted CodeQL must confirm the diagnostic and nonce alerts are gone on the updated
  PR head. Details are in `task-6-report.md`.
