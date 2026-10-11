# Implementation Evidence: KMIP 2.1 Credential Sessions

## T005 — Delegated Login RED

The source-derived tests cover OASIS KMIP Specification v2.1 §6.1.12 Tables 199–201 and the shared Ticket structure in §7.39 Table 391. The fake-transport tests assert Table 199 field selection, caller-owned Ticket handling, Table 201 result preservation, Pending correlation without implicit Poll, one exchange/no retry, and no hidden authentication-state mutation. Unknown Ticket Type values and Ticket extensions remain covered.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Baseline | Existing worktree HEAD `25a8770c` | `cargo test -p kmipkit-protocol --all-features`; `cargo test -p kmipkit-client --all-features` | PASS: protocol 547 unit tests plus integration/doc tests; client 420 unit tests plus UI, integration, and doc tests. |
| Red | Pending commit | `cargo test -p kmipkit-protocol --lib delegated_login_operation_tests` | Expected compile failure: `DelegatedLoginError`, `DelegatedLoginRequest`, and `DelegatedLoginResponse` are not yet exported. |
| Red | Pending commit | `cargo test -p kmipkit-client --lib delegated_login_execution_tests` | Expected compile failures: `DelegatedLoginRequest`, `ClientRequest::delegated_login`, typed outcome access, and `ClientOperation::DelegatedLogin` do not yet exist. |
| Red formatting | Pending commit | `cargo fmt --all --check`; `git diff --check` | PASS. |

These are RED-stage results only; they do not indicate implementation completion or protocol conformance.
