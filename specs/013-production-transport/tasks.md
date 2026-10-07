---
description: "Implementation tasks for production TLS and HTTPS transports"
---

# Tasks: Production TLS and HTTPS Transports

**Input**: Design documents from `specs/013-production-transport/`
**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, and `contracts/`
**Tests**: Required by the approved project plan and strict Red/Green/Refactor TDD. Every Red, Green, and Refactor stage uses a distinct development commit with command/result evidence.

**Execution gate**: T001 must be complete before code or dependency manifest changes. Record the maintainer's standing delegated authorization and independent design reviews there; keep the eventual implementation PR draft for human review/merge. Preserve the single active implementer until the shared worker, resolver, TLS, timeout, and configuration contracts are stable.

## Phase 1: Setup

**Purpose**: Establish review and dependency gates before implementation.

- [x] T001 Record the maintainer's standing delegated authorization and independent QA/security design reviews in `specs/013-production-transport/approval-record.md`; accept KMIPKIT-0013 and ADR-0015 under that authorization; supersede ADR-0005's HTTPS backend clause and explicitly amend the reusable-connection baseline in `docs/design/project-definition.md` and `docs/architecture/transport-security.md` to permit HTTPS reuse while raw TLS closes after one frame. Update the canonical TLS policy to record that per-client session tickets expire after one hour and inherit the trust/CRL decision from the full handshake. Complete this before code or dependency manifest changes.
- [x] T002 Add exact-pinned `tokio`, `hyper` (client + HTTP/1 only), `tokio-rustls`, `rustls` (AWS-LC provider only), `bytes`, `rustls-native-certs`, and `hickory-resolver` workspace dependencies with minimal features in `Cargo.toml` and `crates/kmipkit-transport/Cargo.toml`; update `Cargo.lock`; do not enable TLS 1.2, HTTP/2, compression, or git dependencies.
- [x] T003 (reopened for T004) Re-run the KMIPKIT-0011 dependency checks for `Cargo.lock`, licenses, advisories, feature tree, native build requirements, MSRV, and Linux/Windows/macOS targets after adding the exact-pinned test-support-only `rcgen` and `aws-lc-rs` build-helper dependencies; confirm AWS-LC is the only TLS provider and no production crate gains a test-fixture dependency. Evidence: `pwsh -File scripts/Test-DependencyPolicy.ps1` passed on Windows x86_64; the updated all-target feature tree has Hyper `client`/`http1`, rustls `aws_lc_rs`, rcgen `aws_lc_rs`, and `aws-lc-rs` `prebuilt-nasm`, with no `ring`, TLS 1.2, HTTP/2, proxy, or compression feature; the production normal-edge graph excludes `rcgen` and `kmipkit-test-support`. See `docs/development/kmipkit-0013-dependency-review.md#t004-test-fixture-dependency-review`.
- [x] T004 Add deterministic ephemeral-PKI and loopback DNS/transport fixtures in `crates/kmipkit-test-support/src/` using test-only `rcgen` with default features disabled and AWS-LC selected; add no public-network calls or real credentials and avoid a transport/test-support dependency cycle. Evidence: Red `f7d10d5`, Green `fabfce6`, Refactor `b6cf3ad`; edge-case coverage `9dbd95f`. `cargo test -p kmipkit-test-support --all-features --offline` passed; `cargo llvm-cov --workspace --all-features --offline` passed with `dns.rs` 98.24%, `pki.rs` 97.94%, and `local_transport.rs` 100% line coverage. Fixtures bind only loopback and use throwaway generated identities.
- [x] T005 Add dependency-feature assertions and runtime configuration-policy tests in `crates/kmipkit-transport/tests/` proving there is no Hyper HTTP/2 feature, no proxy/compression feature, and rustls is configured for TLS 1.3 only with key logging and early data disabled. Evidence: Red `f7d10d5`, Green `fabfce6`, Refactor `b6cf3ad`; policy coverage `9dbd95f`. `cargo test -p kmipkit-transport --all-targets --offline` passed; feature graph assertions pass; `cargo llvm-cov -p kmipkit-transport --all-features --offline` reports `tls_policy.rs` 100% line coverage. A local TLS 1.3 handshake succeeds.

---

## Phase 2: Foundational Worker, Resolver, Deadlines, and Errors

**Purpose**: Stabilize the shared synchronous-worker and I/O contracts required by all production transports.

- [x] T006 **Red**: Add worker lifecycle tests in `crates/kmipkit-transport/tests/worker.rs` for lazy network startup, one in-flight plus one queued exchange, sync calls inside a Tokio runtime, worker-start failure, queued-deadline expiry, cancellation/close races, and bounded shutdown; prove an exchange returning `NotSent` cannot be dispatched later and record expected failures in a Red commit. Evidence: Red commit `ee86d71`; `cargo test -p kmipkit-transport --test worker --offline` failed as expected before `worker.rs` existed (`E0583`).
- [x] T007 **Green**: Implement the private per-client current-thread Tokio worker, bounded command/result channel, and atomic `ExchangeControl` cancellation/dispatch state in `crates/kmipkit-transport/src/worker.rs`; prove T006 passes in a separate Green commit. Evidence: Green commit `22cf965`; `cargo test -p kmipkit-transport --lib worker::tests --offline` passed (22 tests); `cargo fmt --all --check` and `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` passed.
- [x] T008 **Refactor**: Simplify worker ownership, shutdown, and panic/error containment in `crates/kmipkit-transport/src/worker.rs`; rerun T006 and record a distinct Refactor commit. Evidence: Refactor commit `c9d3fc9`; `cargo test -p kmipkit-transport --all-targets --all-features --offline` passed (37 tests); `cargo fmt --all --check` and `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` passed; `cargo llvm-cov -p kmipkit-transport --all-features --offline` passed with 97.28% worker line coverage and 97.74% transport-crate line coverage.
- [x] T009 **Red**: Add `DeadlineIo` and real-Hyper-driver tests in `crates/kmipkit-transport/tests/timeout_delivery.rs` for independent blocked read/write/flush deadlines, progress reset, total absolute deadline, phase-vs-total precedence, blocked `SendRequest::ready()` returning `NotSent` before dispatch commit, all delivery states, cancellation before request dispatch, a writer failure immediately after dispatch commit, HTTPS headers sent without a TTLV body, partial body write, response headers before body, vectored-write behavior, and the race between first response-byte observation and timeout finalization; record expected failures in a Red commit. Evidence: `cargo test -p kmipkit-transport --test timeout_delivery --offline` failed as expected at the absent `src/timeout.rs` module required by T010; runtime assertions are deferred until Green.
- [ ] T010 **Green**: Implement the private async I/O wrapper with read/write/vectored-write/flush deadline coverage, monotonic deadline state, `SendRequest::ready()` before the atomic request-dispatch commit, and a shared finalization gate containing delivery state plus `Finalized`; record a decrypted positive read in that gate before returning bytes to Hyper, cancel the request/connection driver before timeout snapshot if finalization wins, and do not infer the HTTP body boundary from serialized I/O bytes; prove T009 passes in a separate Green commit.
- [ ] T011 **Refactor**: Refactor deadline state transitions and payload-free error categories in `crates/kmipkit-transport/src/timeout.rs`; rerun T009 and record a distinct Refactor commit.
- [ ] T012 **Red**: Add resolver tests in `crates/kmipkit-transport/tests/resolver.rs` for system resolver config on Linux/Windows/macOS, `hosts`/search/split-DNS behavior and documented limitations, a maximum of one retry (two attempts total) per query, at most two concurrent nameserver requests per query, the 32-active-request bound on each multiplexed upstream connection (not an aggregate per-client cap), a per-client cache capped at 128 responses, at most 16 returned A/AAAA candidates, timeout/cancellation, NXDOMAIN, ordered candidates, and no request I/O before resolution; record expected failures in a Red commit.
- [ ] T013 **Green**: Implement the per-client Hickory resolver initialization and bounded lookup in `crates/kmipkit-transport/src/resolver.rs`; prove T012 passes in a separate Green commit.
- [ ] T014 **Refactor**: Simplify resolver ownership; prove that an expired lookup is canceled from the caller's exchange, cannot dispatch later, and does not hold worker shutdown on a blocking OS resolver call in `crates/kmipkit-transport/src/resolver.rs`; rerun T012 and record a distinct Refactor commit.

---

## Phase 3: User Story 1 — Configure a Verified KMIP Connection (P1)

**Story goal**: Validate transport input and TLS identity/trust before network activity, with safe secret ownership and redaction.

**Independent test**: Construct valid in-memory and file-backed configurations; invalid endpoints, keys, certificates, trust, and CRLs fail before DNS or socket activity.

- [ ] T015 [US1] **Red**: Add transport-config tests in `crates/kmipkit-transport/tests/tls_config.rs` for raw/HTTPS endpoint rules, origin-form request target, explicit CA/platform trust, required mTLS identity, PEM/DER selection, positive `max_request_bytes` default/override, timeout defaults, and per-exchange options for both typed and direct-adapter paths; record expected failures in a Red commit.
- [ ] T016 [US1] **Green**: Implement validated public configuration and builders in `crates/kmipkit-transport/src/config.rs`; prove T015 passes in a separate Green commit.
- [ ] T017 [US1] **Refactor**: Refine config types, error categories, and constructor invariants in `crates/kmipkit-transport/src/config.rs`; rerun T015 and record a distinct Refactor commit.
- [ ] T018 [US1] **Red**: Add tests in `crates/kmipkit-transport/tests/tls_policy.rs` for certificate/key match, caller CA roots and `rustls-native-certs` platform roots/errors, an isolated-process `SSL_CERT_FILE` override and its precedence, chain/date/hostname, optional TLS server name, valid/expired/non-applicable/revoking caller CRLs, TLS 1.3 only, mTLS, disabled keylog/0-RTT, session resumption across two new connections using the same client configuration, maximum 16 entries, cache isolation across identities, deterministic one-hour local ticket expiry with an injectable monotonic clock, verification-call evidence that a resumed session inherits the original full-handshake trust/CRL snapshot, and a rebuilt client with changed trust inputs proving an empty cache and a new full handshake; record expected failures in a Red commit.
- [ ] T019 [US1] **Green**: Build the per-client rustls config with AWS-LC, TLS 1.3 only, explicit CA or native-root/WebPKI trust, client identity, caller CRLs, and a maximum-16-entry client-owned resumption store that expires tickets after one hour of local monotonic time in `crates/kmipkit-transport/src/tls.rs`; prove T018 passes in a separate Green commit.
- [ ] T020 [US1] **Refactor**: Refactor certificate/key parsing, provider selection, and trust-root ownership in `crates/kmipkit-transport/src/tls.rs`; rerun T018 and record a distinct Refactor commit.
- [ ] T021 [US1] **Red**: Add sentinel tests in `crates/kmipkit-transport/tests/secret_redaction.rs` for Debug/Display/error/log redaction, read-once file sources, encrypted/ambiguous key rejection, initialized key-buffer zeroization, and exclusion of endpoint path/query, credential paths, DNS errors, and dependency text; record expected failures in a Red commit.
- [ ] T022 [US1] **Green**: Implement explicit zeroizing key/source owners, single-read file inputs, and redacted config/error formatting in `crates/kmipkit-transport/src/secret.rs` and `crates/kmipkit-transport/src/config.rs`; prove T021 passes in a separate Green commit.
- [ ] T023 [US1] **Refactor**: Simplify secret ownership and cleanup paths in `crates/kmipkit-transport/src/secret.rs`; rerun T021 and record a distinct Refactor commit.
- [ ] T024 [US1] Document validated constructor methods, direct `exchange_with_options`, TLS policy, platform-root and `SSL_CERT_FILE` behavior, defaults, request/response caps, file behavior, redacted diagnostics, and external-copy limits with Rustdoc tests in `crates/kmipkit-transport/src/lib.rs`.

---

## Phase 4: User Story 2 — Exchange Raw TTLV over TLS (P1)

**Story goal**: Exchange one caller-supplied frame unchanged over verified TLS and return one bounded zeroizing response.

**Independent test**: Use the local ephemeral TLS peer to verify exact request bytes, bounded response framing, error delivery state, and no retry.

- [ ] T025 [US2] **Red**: Add raw TLS success and mTLS failure tests in `crates/kmipkit-transport/tests/raw_tls.rs` for exact input bytes, TLS 1.3, required client identity, unknown CA, invalid validity, hostname mismatch, caller CRL revocation, raw connection close after one response, and cleanup/zeroization of KMIPKit-owned staged request bytes after success and failure; record expected failures in a Red commit.
- [ ] T026 [US2] **Green**: Implement raw TLS connect/handshake, caller-byte exchange, and concrete adapter `exchange_with_options` entry point in `crates/kmipkit-transport/src/raw_tls.rs`; prove T025 passes in a separate Green commit.
- [ ] T027 [US2] **Refactor**: Refactor raw connection ownership and error mapping in `crates/kmipkit-transport/src/raw_tls.rs`; rerun T025 and record a distinct Refactor commit.
- [ ] T028 [US2] **Red**: Add malformed-frame and allocation-boundary tests in `crates/kmipkit-transport/tests/raw_tls.rs` for partial header, bad root tag/type, unaligned/overflowing length, exact cap, one byte over, truncated body, incomplete EOF, surplus/coalesced second frame discarded by connection close, and cleanup of initialized bytes; verify the next distinct call reconnects; record expected failures in a Red commit.
- [ ] T029 [US2] **Green**: Implement fixed-header validation, checked size arithmetic, pre-allocation response limits, single bounded response allocation, exact body/padding reads, and zeroizing cleanup in `crates/kmipkit-transport/src/raw_tls.rs`; prove T028 passes in a separate Green commit.
- [ ] T030 [US2] **Refactor**: Simplify the frame state machine and ensure errors invalidate the connection without retaining request/response bytes in `crates/kmipkit-transport/src/raw_tls.rs`; rerun T028 and record a distinct Refactor commit.
- [ ] T031 [US2] **Red**: Add raw TLS timeout/reconnect tests in `crates/kmipkit-transport/tests/timeout_delivery.rs` for connect/write/read/total timeout boundaries, dispatch/cancel linearization, request-buffer cleanup on partial write/timeout/cancellation, delivery transitions, connection invalidation, later reconnect, and no replay; record expected failures in a Red commit.
- [ ] T032 [US2] **Green**: Integrate `DeadlineIo`, resolver, raw TLS connection, and `DeliveryState` in `crates/kmipkit-transport/src/raw_tls.rs`; prove T031 passes in a separate Green commit.
- [ ] T033 [US2] **Refactor**: Refactor raw exchange orchestration while preserving the no-retry and exact-byte contracts in `crates/kmipkit-transport/src/raw_tls.rs`; rerun T031 and record a distinct Refactor commit.

---

## Phase 5: User Story 3 — Exchange TTLV over HTTPS/HTTP 1.1 (P1)

**Story goal**: POST exact TTLV bytes with the required headers and accept only a bounded, correctly framed HTTP response.

**Independent test**: Use the local ephemeral HTTPS/HTTP 1.1 peer to capture exact method/target/headers/body and exercise invalid statuses, framing, encoding, limits, and deadlines.

- [ ] T034 [US3] **Red**: Add HTTPS request-capture/request-building tests in `crates/kmipkit-transport/tests/https.rs` for HTTPS-only HTTP/1.1, POST, origin-form target/default `/kmip`, rejection of absolute/different-authority targets, exact body, exactly one `Host` serialized from the endpoint authority for a DNS hostname, bracketed IPv6 literal, and non-default port (unaffected by a different TLS verification name), one `Content-Type`, exact `Content-Length`, and `Cache-Control: no-cache`; verify the bounded request body owner zeroizes its initialized bytes on success and error; record expected failures in a Red commit.
- [ ] T035 [US3] **Green**: Implement the Hyper HTTP/1 request path over `DeadlineIo` and concrete adapter `exchange_with_options` entry point in `crates/kmipkit-transport/src/https.rs`; prove T034 passes in a separate Green commit.
- [ ] T036 [US3] **Refactor**: Refactor request construction so the authority/TLS name cannot be changed by the origin-form target and use one bounded zeroizing body owner (for example `bytes::Bytes::from_owner` with a zeroizing owner) in `crates/kmipkit-transport/src/https.rs`; rerun T034 and record a distinct Refactor commit.
- [ ] T037 [US3] **Red**: Add HTTPS response tests in `crates/kmipkit-transport/tests/https.rs` for non-200 status, duplicate/conflicting/missing headers, invalid media type/length, transfer/content encoding, parser errors, truncation, unsolicited extra responses on a reused connection, 64-header/64-KiB parser boundaries, oversized/incomplete headers, exact response cap, over-cap rejection before KMIPKit body allocation, and partial-buffer zeroization; prove an unsolicited response cannot be attributed to the next request; record expected failures in a Red commit.
- [ ] T038 [US3] **Green**: Implement strict status/header/content-length validation and bounded streamed response ownership in `crates/kmipkit-transport/src/https.rs`; prove T037 passes in a separate Green commit.
- [ ] T039 [US3] **Refactor**: Refine Hyper parser/error mapping and connection invalidation for malformed responses in `crates/kmipkit-transport/src/https.rs`; rerun T037 and record a distinct Refactor commit.
- [ ] T040 [US3] **Red**: Add policy tests in `crates/kmipkit-transport/tests/https.rs` for redirect response, proxy environment variables, cookies, compression, HTTP/2 negotiation, automatic retry, and server-supplied endpoint failover; prove cancellation before dispatch is `NotSent`, while a failure after dispatch commit—including headers sent without a TTLV body—and a partial body write are `PossiblySent`; record expected failures in a Red commit.
- [ ] T041 [US3] **Green**: Configure only the direct endpoint and HTTP/1 path with redirects/proxy/cookies/compression/retry/failover unavailable in `crates/kmipkit-transport/src/https.rs`; prove T040 passes in a separate Green commit.
- [ ] T042 [US3] **Refactor**: Consolidate HTTPS policy validation and document parser-owned versus KMIPKit-owned checks in `crates/kmipkit-transport/src/https.rs`; rerun T040 and record a distinct Refactor commit.
- [ ] T043 [US3] **Red**: Add HTTPS timeout/reconnect tests in `crates/kmipkit-transport/tests/timeout_delivery.rs` for connect/DNS/TLS, blocked `SendRequest::ready()`, blocked writes/flushes, blocked reads, total deadline, deadline expiry while queued, cancellation/request-dispatch race, first-response-byte versus timeout-finalization race, delivery-state transitions, invalidation, later reconnect, and no replay; record expected failures in a Red commit.
- [ ] T044 [US3] **Green**: Integrate the `DeadlineIo` state machine with Hyper's request/connection driver and bounded resolver in `crates/kmipkit-transport/src/https.rs`; prove T043 passes in a separate Green commit.
- [ ] T045 [US3] **Refactor**: Simplify request/response phase transitions and ensure all timeout errors remain redacted in `crates/kmipkit-transport/src/https.rs`; rerun T043 and record a distinct Refactor commit.

---

## Phase 6: User Story 4 — Execute the Existing Typed Client (P2)

**Story goal**: Construct the existing closed typed client from validated KMIPKit configuration and preserve encode/limit/decode boundaries.

**Independent test**: Execute every currently supported typed request through each production adapter
and return only a typed decoded result; verify cross-client extension-provenance rejection and
same-client extension preservation through the public Rust API.

- [ ] T046 [US4] **Red**: Add public API integration tests in `crates/kmipkit-client/tests/production_client.rs` for validated raw TLS/HTTPS construction, no arbitrary transport injection, sync execution inside an existing runtime, options-bearing variants for every current typed operation method, direct-adapter byte exchange overrides, timeout override precedence on both paths, and exact request/response delivery; record expected failures in a Red commit.
- [ ] T046a [US4] **Red**: Add a focused execution-boundary unit test in
  `crates/kmipkit-client/src/execute.rs` proving `ClientRequestMessageExtension` registry
  provenance is checked before outgoing `RequestMessage` construction, codec invocation, and adapter
  handoff. Add public Rust integration tests in
  `crates/kmipkit-client/tests/production_client.rs`:
  `production_client_rejects_foreign_client_request_message_extension_as_invalid_input_not_sent`
  constructs a `ClientRequestMessageExtension` from `ClientConfiguration` A, attaches it to a
  request for a production client retaining configuration B, and asserts sanitized
  `InvalidInput`/`NotSent` with no exchange observed at an ephemeral peer;
  `production_client_accepts_client_request_message_extension_from_its_own_configuration` executes a
  `ClientRequestMessageExtension` validated by the production client's retained configuration and
  asserts a typed response plus the expected unchanged Message Extension wire representation.
  Assert error formatting does not reveal registry identity or extension payload data. Record
  expected failures in a separate Red commit before T047.
- [ ] T047 [US4] **Green**: Add shared public `RequestOptions`, the production constructor, concrete
  direct-adapter `exchange_with_options` methods, adapter selection, and options-bearing variants
  corresponding to every current public typed operation method in `crates/kmipkit-client/src/lib.rs`
  and `crates/kmipkit-transport/src/`. Retain the immutable KMIPKIT-0012 `ClientConfiguration`
  separately from transport configuration; before constructing an outgoing `RequestMessage`,
  encoding, or adapter invocation, reject attached `ClientRequestMessageExtension` values whose
  private registry provenance does not match that client's registry as sanitized
  `InvalidInput`/`NotSent`, while preserving same-client execution and wire behavior. Prove T046 and
  T046a pass in a separate Green commit.
- [ ] T048 [US4] **Refactor**: Refactor constructor ownership and ensure calls serialize through the
  private worker without exposing raw transport types in `crates/kmipkit-client/src/lib.rs`; preserve
  separate configuration ownership and both `ClientRequestMessageExtension` provenance outcomes,
  rerun T046 and T046a, and record a distinct Refactor commit.
- [ ] T049 [US4] **Red**: Add tests in `crates/kmipkit-client/tests/production_client.rs` for direct `max_request_bytes` rejection before connect, typed encoded-request rejection at `CodecLimits::max_message_bytes()`, exact response cap, pre-decode returned-length check, typed response validation, and absence of raw response exposure; record expected failures in a Red commit.
- [ ] T050 [US4] **Green**: Connect existing encode/execute/decode flow to the production adapters and exact limits in `crates/kmipkit-client/src/lib.rs`; prove T049 passes in a separate Green commit.
- [ ] T051 [US4] **Refactor**: Refactor error conversion and typed decode ownership while preserving `TransportResponse` cleanup in `crates/kmipkit-client/src/lib.rs`; rerun T049 and record a distinct Refactor commit.

---

## Phase 7: Polish, Traceability, Reviews, and PR

**Purpose**: Close documentation, traceability, security, coverage, reproducibility, and review gates.

- [ ] T052 Record final implementation evidence in accepted ADR-0015 and update `docs/security/threat-model.md` with the worker, resolver limits, `SSL_CERT_FILE` trust override, parser limits, delivery/timeout races, raw-vs-HTTPS connection lifecycle, one-hour session-resumption trust snapshot, memory-copy boundaries, and new controls. The canonical connection-model and TLS-policy amendments are made at T001 before code starts.
- [ ] T053 Add complete stable requirement-to-source/spec/code/test links for every KMIPKIT-0013
  functional requirement and applicable OASIS requirement in
  `specification/compliance/requirements/KMIPKIT-0013.csv`, including FR-018's KMIPKIT-0012/ADR-0013
  source links and the T046a provenance tests.
- [ ] T054 Document Rust API behavior, trust configuration, timeout states, limitations, and executable examples in `docs/user-guide/en/` and `docs/user-guide/es/`; test examples in Rust.
- [ ] T055 Run catalog/traceability checks and verify all eight selected §5.3.1 requirements have behavior/test links without an unsupported profile claim; record evidence in `specs/013-production-transport/tasks.md`.
- [ ] T056 Run `cargo fmt --all --check`, strict workspace Clippy, focused tests, full workspace tests, docs tests, and repository dependency/security automation; fix every failure and record exact command results.
- [ ] T057 Run Linux, Windows, and macOS CI and coverage gates; verify at least 85% transport coverage, at least 95% changed-code coverage, 90% workspace coverage, and 100% normative traceability.
- [ ] T058 Measure configuration/startup, TLS handshake, steady-state exchange, and per-client worker overhead at representative concurrency in `crates/kmipkit-transport/benches/` and record the reproducible baseline and environment in `docs/development/transport-performance.md`.
- [ ] T059 Request independent QA review of every acceptance criterion and platform requirement; fix findings in this branch and record final evidence in `specs/013-production-transport/tasks.md`.
- [ ] T060 Request independent security review of parser, TLS, DNS, secrets, dependencies, worker lifecycle, and FFI-adjacent effects; fix all blockers and record the reviewer report and applicable security checks.
- [ ] T061 Update the feature branch from the active release branch, resolve conflicts, and rerun all required checks before opening review.
- [ ] T062 Prepare a draft PR to `release/1.0.0` with scope, rationale, dependency record, Red/Green/Refactor commit IDs, verification evidence, generated-output statement, risks, and limitations; verify its state in GitHub and attach the PR artifact.

## Dependencies and Execution Order

- T001 gates every code and manifest task.
- T002–T005 establish dependencies and shared test fixtures before the worker.
- T006–T014 stabilize the worker, resolver, and deadline contract before transport stories.
- User Story 1 (T015–T024) establishes validated config/TLS before raw TLS or HTTPS.
- User Stories 2 and 3 depend on the worker/deadline foundation and User Story 1; implement sequentially under one active implementer because both share TLS and I/O state.
- User Story 4 depends on both production adapters and preserves the existing client boundary.
- T052–T062 require all four stories complete; design ADR acceptance is T001 before implementation; review fixes precede final branch update and draft PR.

### Parallel Opportunities

There are no parallel code implementation tasks while the shared worker, TLS, resolver, and timeout interfaces are stabilizing. After T051, independent documentation proofreading and independent reviewer audits may run in parallel because they do not edit shared implementation files. Security and QA reviewers must report findings to the implementer; they do not edit, approve, or merge the PR.

## Requirement Coverage Map

| Requirement | Task coverage |
|---|---|
| FR-001, FR-016, FR-018: typed-client boundary, public transport surface, and KMIPKIT-0012 registry binding | T046, T046a, T047–T051 |
| FR-002, FR-003, FR-006: TLS, trust, mTLS, resumption, 0-RTT/keylog | T018–T020, T025–T027, T052–T055 |
| FR-004, FR-005: key inputs, zeroization, redaction | T018–T023, T054, T059 |
| FR-007: raw TTLV/TLS frame | T025–T033 |
| FR-008, FR-009, FR-010: HTTPS behavior and framing | T034–T042 |
| FR-011, FR-012: deadlines, queueing, request options, and delivery state | T006–T014, T031–T033, T040–T045, T046–T048 |
| FR-013: one endpoint, invalidation, no retry/failover | T006–T014, T031–T033, T040–T045 |
| FR-014, FR-015, FR-017: request/response limits and memory cleanup | T015–T025, T028–T031, T034, T037–T039, T049–T051, T054 |
| SC-001–SC-008 | T018–T062, with final evidence at T055–T060; SC-007 also includes T046a |
| Normative traceability | T053, T055 |

## Implementation Strategy

Deliver the validated configuration and tested worker/deadline foundation
first. Then complete raw TLS, HTTPS, and typed-client integration as separately
testable stories. Keep Red, Green, and Refactor commits distinct in every
story. Do not claim the transport is release-ready until all platform,
coverage, traceability, dependency, QA, and security gates pass.
