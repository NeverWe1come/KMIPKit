# Implementation Evidence: KMIP 2.1 Credential Sessions

## T005 — Delegated Login RED

The source-derived tests cover OASIS KMIP Specification v2.1 §6.1.12 Tables 199–201 and the shared Ticket structure in §7.39 Table 391. The fake-transport tests assert Table 199 field selection, caller-owned Ticket handling, Table 201 result preservation, Pending correlation without implicit Poll, one exchange/no retry, and no hidden authentication-state mutation. Unknown Ticket Type values and Ticket extensions remain covered.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Baseline | `25a8770c` | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features` | PASS: protocol 547 unit tests plus integration/doc tests; client 420 unit tests plus UI, integration, and doc tests. |
| Red | `b67a7804` | `cargo test -p kmipkit-protocol --lib delegated_login_operation_tests`; `cargo test -p kmipkit-client --lib delegated_login_execution_tests` | Expected compile failures confirmed before implementation: Delegated Login protocol models and the typed request/outcome/client operation API were absent. The separate Red commit records these results. |
| Red formatting | `b67a7804` | `cargo fmt --all --check`; `git diff --check` | PASS. |
| Green | `f6b5e3c` | `cargo test -p kmipkit-protocol --lib delegated_login_operation_tests`; `cargo test -p kmipkit-client --lib delegated_login_execution_tests` | PASS: protocol 5/5 and client 6/6 Delegated Login tests. |
| Green | `f6b5e3c` | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features` | PASS: protocol 552 unit tests plus integration/doc tests; client 426 unit tests plus UI, integration, and doc tests. |
| Green quality gates | `f6b5e3c` | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |
| Refactor | This commit | `cargo test -p kmipkit-protocol --lib delegated_login_operation_tests` | PASS: 6/6 focused tests, including duplicate Ticket rejection and unknown extension preservation. The parser now separates TTLV value conversion from response field selection; request encoding calls the shared payload helper directly. |
| Refactor quality gates | This commit | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |

## T011 — Logout RED

The source-derived protocol tests cover OASIS KMIP Specification v2.1 §6.1.31 Tables 256–258 and the required Ticket from §7.39 Table 391. The client tests cover one explicit Ticket-bearing exchange, empty success payload, Table 258 failure, Pending correlation, malformed responses, local NotSent validation, and no retry. There is no direct official Logout fixture, and none is claimed.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | This commit | `cargo test -p kmipkit-protocol --lib logout_operation_tests` | Expected compile failure: `LogoutError`, `LogoutRequest`, and `LogoutResponse` are not implemented/exported. |
| Red | This commit | `cargo test -p kmipkit-client --lib logout_execution_tests` | Expected compile failures: Logout request construction, operation classification, and typed response accessors do not exist. |
| Red formatting | This commit | `cargo fmt --all --check`; `git diff --check` | PASS. |

## T012 — Logout GREEN

Logout requires a caller-supplied typed Ticket and encodes its original ordered TTLV without logging or changing ticket state. The typed response validates the empty success payload and retains shared Failure/Pending metadata. The fake transport confirms one exchange, delivery-state behavior, no retry, and no Ticket attached to the next independent request.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Focused Green | This commit | `cargo test -p kmipkit-protocol --lib logout_operation_tests`; `cargo test -p kmipkit-client --lib logout_execution_tests` | PASS: 5 protocol tests and 6 fake-transport tests for Tables 256–258, secret redaction, Pending, NotSent, and no hidden ticket reuse. |
| Green package checks | This commit | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features`; `cargo test -p kmipkit --all-features` | PASS: protocol (565 unit tests), client (438 unit tests), integration/UI/doc tests, and facade API tests. |
| Green quality gates | This commit | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |

## T013 — Logout REFACTOR

Common result-header conversion is separated from Logout's operation-specific empty-payload check. Added a test for preserving an unknown Table 258 Result Reason; the request wire shape and response behavior are unchanged.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Refactor | This commit | `cargo test -p kmipkit-protocol --lib logout_operation_tests`; `cargo test -p kmipkit-client --lib logout_execution_tests` | PASS: 6 protocol tests and 6 fake-transport tests. |
| Refactor quality gates | This commit | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |

These results establish the Delegated Login model and typed client path against source-derived tests. They do not indicate official KMIP conformance, cross-language API parity, or completion of KMIPKIT-024.

## T008 — Login RED

The tests use OASIS KMIP Specification v2.1 §6.1.30, Tables 253–255; Ticket uses §7.39, Table 391 and §11.57, Table 488. The pinned Test Cases CN01 §2.48 fixture is parsed directly from `specification/oasis/kmip-2.1/fixtures/TC-LOGIN-1-21.xml`; the assertions distinguish fixture consistency from a server interoperability run.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | This commit | `cargo test -p kmipkit-protocol --lib login_operation_tests` | Expected compile failure: `LoginError`, `LoginRequest`, and `LoginResponse` are not implemented/exported. |
| Red | This commit | `cargo test -p kmipkit-client --lib login_execution_tests` | Expected compile failures: Login request construction, operation classification, and typed response accessors do not exist. |
| Red formatting | This commit | `cargo fmt --all --check`; `git diff --check` | PASS. |

## T009 — Login GREEN

The implementation encodes the optional Table 253 fields in order, returns the required secret-safe Ticket for Table 254 success, preserves Table 255 failure metadata, and routes Login through one typed client exchange. Login does not store or attach a Ticket to later requests.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Focused Green | This commit | `cargo test -p kmipkit-protocol --lib login_operation_tests`; `cargo test -p kmipkit-client --lib login_execution_tests` | PASS: 12 protocol tests and 12 client tests, including the pinned CN01 §2.48 fixture values. |
| Green package checks | This commit | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features`; `cargo test -p kmipkit --all-features` | PASS: protocol and client package suites, integration/UI/doc tests, and facade API tests. |
| Green quality gates | This commit | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |

## T010 — Login REFACTOR

Ticket field selection now delegates the nested TTLV conversion to a dedicated helper, keeping duplicate detection separate from typed Ticket parsing. A duplicate-Ticket regression test verifies the prior rejection behavior and safe diagnostic.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Refactor | This commit | `cargo test -p kmipkit-protocol --lib login_operation_tests`; `cargo test -p kmipkit-client --lib login_execution_tests` | PASS: protocol/fixture tests and client Login execution tests pass. |
| Refactor quality gates | This commit | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |


## T014 — Catalog and traceability

Added implementation and verification references for Delegated Login, Login, Logout, their operation values, Ticket structure and members, and the shared payload/result tags. Existing KMIPKIT-0004 tag ownership and prior allocation references were preserved. The traceability document now reflects implemented Rust protocol/client modules and executable tests.

| Command | Result |
|---|---|
| `python tools/normative_catalog/validate.py` | PASS: 4,044 catalog records valid. |
| `python tools/normative_catalog/report.py --write` | PASS: regenerated the checked-in coverage report from catalog inputs. |
| `python tools/normative_catalog/report.py --check` | PASS: generated report is current and deterministic. |
| `git diff --check` | PASS. |


## T019 — Current verification and remaining release gates (2026-10-11)

The post-rebase Rust implementation was freshly rechecked on Windows after adding defensive parser, error-formatting, Ticket redaction, request-default, and typed response-view coverage. All seven focused protocol/client commands and every command in `quickstart.md` passed. The full workspace unit, fixture, fake-transport, UI/trybuild, integration, and doctest suites passed on this host. The separate Linux C consumer result recorded below is prior independent evidence. These are source-derived behavioral checks, not KMIP server interoperability results.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | PASS. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS. |
| `cargo test --workspace --all-features` | PASS: all workspace unit, integration, UI/trybuild, fixture, guide-example, and doc tests completed with exit code 0. |
| `python scripts/test_user_guide_examples.py` | PASS: 20 marked Rust guide examples compiled. |
| `python -B tools/api_manifest/generate.py --check` | PASS: six registry-manifest outputs are current. Credential-session operation mappings are not present in the release manifest. |
| `python -B -m unittest discover -s tools/api_manifest/tests -p 'test_*.py' -v` (with pinned `tools/api_manifest/requirements-test.txt` installed under ignored `target/`) | PASS: 51 tests; 3 symlink cases skipped because Windows did not grant symlink creation. |
| `python -B tools/normative_catalog/check_immutable_sources.py --repo-root . --base-sha 25f99278fe3fbc90da5569283107e19bb0fa012c` | PASS: pinned OASIS copies match the release base. |
| `python -B tools/normative_catalog/audit_sources.py --repo-root . --base-sha 25f99278fe3fbc90da5569283107e19bb0fa012c --check` | PASS: 1,411 normative source candidates audited. |
| Normative catalog validation, report, generated TTLV tags, result values, attribute types, and attribute policy checks | PASS: 4,044 catalog records valid; all generated outputs current. |
| `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` | PASS: 212 tests; 7 expected platform skips. |
| `pwsh -File .\scripts\Test-DependencyPolicy.ps1` | PASS: pinned cargo-deny 0.20.2 checked the root and fuzz workspaces, refreshed/verified the RustSec advisory database, and confirmed both lockfile hashes unchanged. |
| `python scripts/coverage_gate.py preflight --workspace .` | PASS: source scan found 107 executable or ambiguous function bodies across 163 production files. |
| `CARGO_TARGET_DIR=target/kmipkit-024-final-coverage cargo llvm-cov --workspace --all-features` | Tests/report completed; the isolated Windows line report measured 89.24%, below 90%, and LLVM warned that 11 functions had mismatched data while combining text reports from multiple test binaries. This is distinct from CI's raw JSON export. |
| CI coverage path: `cargo llvm-cov --workspace --all-features --locked --json --output-path ...`; `python scripts/coverage_gate.py normalize --workspace . --input ... --output ...` | PASS: exact raw-JSON export and normalization commands exited 0; no mismatched-data warning was emitted. The three-platform aggregate still requires CI. |
| `bash scripts/collect_ffi_coverage.sh coverage-ffi` with a fresh isolated Linux `CARGO_TARGET_DIR` | PASS: C consumer test passed; `crates/kmipkit-ffi/src/extension_registry.rs` measured 93.59% lines. Do not combine this Linux profile with Windows raw profiles; their LLVM profile formats differ. |
| `git diff --check` | PASS. |

The global coverage gate still needs its CI aggregate across the three Rust platforms plus the C consumer report. T015–T017 remain blocked until the approved KMIPKIT-0043 public API contract is merged; the current release manifest has no credential-session operation mappings. The 0043 implementation worktree is at 11/75 tasks and has no PR, so this dependency is not available to merge. T020 remains open: ports 5696 and 5697 have no local listeners, Docker is not running, and no two-server interoperability result is available. These checks do not establish 1.0.0 readiness, certification, or server interoperability.

Four-language manifest mappings remain blocked on the canonical KMIPKIT-0043 operation mapping tasks; T014 does not claim binding parity.


## T018 — Bilingual credential-session guide

Added English and Spanish guidance for Login, Delegated Login, and Logout, including explicit Ticket ownership, a caller-controlled owned copy for Logout, redacted diagnostics, the synthetic-only status of the example Ticket, and the current parity/fixture limitations. Registered Rust example checks and added both guides to the default compiler script and documentation index.

| Command | Result |
|---|---|
| `python scripts/test_user_guide_examples.py --guide docs/user-guide/en/credential-sessions.md --guide docs/user-guide/es/sesiones-credenciales.md` | PASS: 4 marked examples compiled. |
| `cargo test -p kmipkit-protocol --test credential_sessions_guide_examples --all-features` | PASS: 2/2 tests. |
| `python scripts/test_user_guide_examples.py` | PASS: all 18 marked user-guide examples compiled. |
| `cargo fmt --all --check` | PASS. |
| `git diff --check` | PASS. |
