# Tasks: Local Cosmian KMS Integration Tests

**Status**: In progress. The maintainer approved expanded operation coverage on
2026-10-09. Coverage and hosted CI gates remain open.

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
- [x] T019 Update the specification, implementation plan, and traceability scope for ten selected typed operations; cite the exact OASIS v2.1 section and tables. Record Adjust Attribute separately as unsupported by Cosmian 5.28.0.
- [x] T020 Add diagnostic test-first cases for Create, Create Key Pair, and Create Split Key in `crates/kmipkit-client/tests/cosmian_kms.rs`; run each through KMIPKit's typed client over the existing raw-TLS mTLS transport.
- [x] T021 Add Get Attributes and Get Attribute List cases using a fresh server-generated object per test; assert result success and the requested/listed attributes in the typed result.
- [x] T022 Add Add Attribute, Delete Attribute, Modify Attribute, and Set Attribute cases using independent server-generated objects; assert operation success and returned identifiers. Adjust Attribute is excluded after Cosmian rejects it as unsupported.
- [x] T023 Run the complete ignored Cosmian suite against the pinned local image, record each operation's result independently, and fix only test input/setup defects in this integration PR. Record and stop at the out-of-scope Create builder defect.
- [x] T024 Update `specs/015-cosmian-kms-integration/traceability.md`, `quickstart.md`, `tests/integration/cosmian/README.md`, and bilingual client integration documentation with exact commands, operation clauses, and live outcomes.
- [ ] T025 Run focused client/transport suites, `cargo fmt --all --check`, workspace Clippy, workspace tests, catalog checks, and the required coverage/hosted-platform CI gates; record commands and outcomes. Local client/transport tests, format, Clippy, workspace tests, Compose/syntax checks, immutable-source/audit/catalog/generated-output checks pass. Local `cargo llvm-cov --workspace --all-features --summary-only` reports 83.80% line coverage, below the 90% workspace threshold; hosted multi-platform coverage aggregation remains pending.
- [ ] T026 Review every FR and SC against the expanded tests; leave PR #56 open as draft until coverage and CI gates pass.
