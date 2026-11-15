# Task 22 — Common Operation Result Parser

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

Start commit: `bcb1085163cc8e8baea1579dfcf17a9e11c420fb`

## Scope

Centralized parsing of the common Result Status, Result Reason, and Result
Message fields for Encrypt and Decrypt. The private
`result::parse_operation_result` helper validates the existing
`KmipOperationResult` invariant and retains unknown raw values. It is also used
by the existing asynchronous result adapter, which keeps its existing error
mapping. Encrypt and Decrypt map common parse errors into their existing
operation-specific variants and retain their operation-specific successful
payload parsing and safe error text.

The completed-response converters still inspect the parsed status and return
their existing payload-free `PendingOutcomeRequired` error for Operation
Pending. Callers must route Pending through the shared `PendingOutcome` path
before invoking either completed-response converter. This task does not create
PendingOutcome values or add client dispatch.

## Red, Green, Refactor commits

| Phase | Commit | Evidence |
| --- | --- | --- |
| Red / characterization | `b0339be685aa619e26f6ab6c4317a711bb9fb0c6` | Added tests before the parser refactor for exact unknown-reason and message retention through the existing asynchronous helper and both converters, the Failure reason invariant, and Pending preservation/rejection. The focused test target passed 3 tests against the pre-refactor behavior; this is a behavior-preserving extraction, so no artificial failing behavior or production stub was introduced. |
| Green | `d0b2d2517528a51ef47773ec3aa18f9c7c955303` | Added the reusable parser in `result.rs` and routed the asynchronous adapter through it. Common result tests (3), asynchronous tests (2), and Discover Versions result-error tests (2) passed. |
| Refactor | Included in the signed commit containing this report | Updated Encrypt and Decrypt converters to consume the shared parser, with local error mapping and the existing Pending guard. Updated T022 evidence and requirement traceability. |

All three development commits use DCO sign-offs.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-protocol --lib encrypt_response_tests --offline` | 2 passed |
| `cargo test -p kmipkit-protocol --lib decrypt_response_tests --offline` | 2 passed |
| `cargo test -p kmipkit-protocol --lib operation_failure_tests --offline` | 2 passed |
| `cargo test -p kmipkit-protocol --lib pending_encrypt_response_shape_tests --offline` | 3 passed; both completed converters reject Pending |
| `cargo test -p kmipkit-protocol --lib common_operation_result_tests --offline` | 3 passed |
| `cargo test -p kmipkit-protocol --all-features --offline --quiet` | 312 unit tests, all protocol integration suites, and doc tests passed |
| `cargo fmt --all --check` | passed |
| `cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo doc -p kmipkit-protocol --no-deps --all-features --offline` | passed |
| `git diff --check` | passed after report and traceability edits |

The first Clippy run flagged missing backticks around `PendingOutcome` in a
test module's documentation. That wording was corrected, and the final Clippy
run passed.

## Traceability and limits

Updated `KMIPKIT-REQ-SPEC-6.1-001-002` in
`specs/019-cryptographic-operations/traceability.md` to map the shared parser,
Encrypt/Decrypt converter behavior, and focused tests. Updated T022 with the
characterization, Green, and Refactor evidence.

No OASIS upstream or generated files were edited. The XML fixture adapter,
28-item fixture accounting, client PendingOutcome creation/dispatch, multipart
client policy, full workspace checks, coverage, interoperability, and release
gates remain outside T022.
