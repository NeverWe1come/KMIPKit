# Implementation Evidence: KMIP 2.1 Credential Sessions

## T005 — Delegated Login RED

The source-derived tests cover OASIS KMIP Specification v2.1 §6.1.12 Tables 199–201 and the shared Ticket structure in §7.39 Table 391. The fake-transport tests assert Table 199 field selection, caller-owned Ticket handling, Table 201 result preservation, Pending correlation without implicit Poll, one exchange/no retry, and no hidden authentication-state mutation. Unknown Ticket Type values and Ticket extensions remain covered.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Baseline | `25a8770c` | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features` | PASS: protocol 547 unit tests plus integration/doc tests; client 420 unit tests plus UI, integration, and doc tests. |
| Red | `b67a7804` | `cargo test -p kmipkit-protocol --lib delegated_login_operation_tests`; `cargo test -p kmipkit-client --lib delegated_login_execution_tests` | Expected compile failures confirmed before implementation: Delegated Login protocol models and the typed request/outcome/client operation API were absent. The separate Red commit records these results. |
| Red formatting | `b67a7804` | `cargo fmt --all --check`; `git diff --check` | PASS. |
| Green | This commit | `cargo test -p kmipkit-protocol --lib delegated_login_operation_tests`; `cargo test -p kmipkit-client --lib delegated_login_execution_tests` | PASS: protocol 5/5 and client 6/6 Delegated Login tests. |
| Green | This commit | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features` | PASS: protocol 552 unit tests plus integration/doc tests; client 426 unit tests plus UI, integration, and doc tests. |
| Green quality gates | This commit | `cargo clippy -p kmipkit-protocol -p kmipkit-client -p kmipkit --all-targets --all-features -- -D warnings`; `cargo fmt --all --check`; `git diff --check` | PASS. |

These results establish the Delegated Login model and typed client path against source-derived tests. They do not indicate official KMIP conformance, cross-language API parity, or completion of KMIPKIT-024.
