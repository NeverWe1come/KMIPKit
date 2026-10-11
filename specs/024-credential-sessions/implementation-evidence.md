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
