# Implementation Plan: Production TLS and HTTPS Transports

**Branch**: `feature/KMIPKIT-0013-production-transport`
**Date**: 2026-10-07
**Spec**: [spec.md](spec.md)

**Input**: KMIPKit roadmap and [production transport specification](spec.md).

## Summary

Add a validated production transport configuration, mutually authenticated TLS 1.3 transports for raw TTLV and HTTPS/HTTP 1.1, and a production constructor for the existing synchronous typed client. The transport uses Hyper's HTTP/1 parser, Tokio I/O/timers, Hickory's asynchronous resolver initialized from system configuration, and a rustls stream. This keeps HTTP parsing in a maintained protocol implementation while exposing the I/O boundary needed for independent phase timeouts and delivery-state tracking. A worker owned by each client avoids calling Tokio `Runtime::block_on` on the application thread, works when the caller already runs inside Tokio, and serializes requests. HTTPS may reuse one healthy connection; raw TLS closes after each response frame to prevent surplus frames from becoming a later response.

The transport stack differs from accepted ADR-0005's `reqwest::blocking` choice. Research shows that the blocking builder provides a shared connect/read/write timeout plus a separate connect timeout, but not independent read, write, and total deadlines. The public facade will remain synchronous and keep all current KMIPKit TLS and no-retry boundaries. This plan includes a narrowly scoped ADR update documenting the runtime and I/O instrumentation costs before implementation begins.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94.
**Primary Dependencies**: Exact-pinned `tokio` for owned asynchronous I/O and timers; `tokio-rustls` and `rustls` with AWS-LC for TLS; a client-owned `rustls::client::ClientSessionStore` to expire TLS 1.3 tickets after one hour; `hyper` with HTTP/1 only for HTTPS parsing; `bytes` for a request body owner backed by a zeroizing allocation; `rustls-native-certs` for explicitly selected platform root loading; `hickory-resolver` with Tokio and system configuration for bounded asynchronous hostname lookup; `zeroize` for KMIPKit-owned secret buffers. `kmipkit-test-support` uses exact-pinned `rcgen` only to make fresh local test credentials, with default features disabled and AWS-LC selected; no production crate depends on it, and `ring` must remain absent. T003 reviews exact versions, feature flags, transitive licenses/advisories, native requirements, MSRV, and supported targets under KMIPKIT-0011.
**Storage**: None. One client owns validated configuration, TLS state, a worker, and at most one connection at a time; only HTTPS reuses it across exchanges.
**Testing**: Rust unit, fake-I/O, ephemeral-PKI, raw TLS and HTTP/1 integration tests; existing cross-platform CI and coverage gates.
**Target Platform**: Linux, Windows, and macOS on Rust 1.94 or later.
**Project Type**: Synchronous Rust library with internal transport worker and C/Java/Python consumers in later 1.0 work.
**Performance Goals**: Keep one HTTPS connection per client reusable, close raw-TLS connections after one frame to prevent response confusion, avoid a worker or runtime per exchange, and measure construction, TLS handshake, steady-state exchange, and per-client worker overhead at representative client concurrency before 1.0.
**Constraints**: TLS 1.3 only, mTLS, explicit trust, HTTP/1.1 only, no proxy/redirect/compression/retry, bounded reads before allocation, redacted errors, and delivery-aware outcomes. No unsafe code outside the existing FFI crate.
**Scale/Scope**: One endpoint and one serialized worker/connection per client; synchronous typed request set is limited to operations already present in `kmipkit-client`.

## Constitution Check

| Gate | Result | Evidence or condition |
|---|---|---|
| Product and protocol boundaries | Pass | KMIP 2.1, TTLV only, client initiated, synchronous public API, TLS 1.3/mTLS; no server, JSON, or XML behavior. |
| Approved specification and exact OASIS traceability | Pass | `spec.md` cites the pinned Core and Profiles clauses. The design package reviews and ADR-0015 acceptance are recorded by T001; implementation remains gated by dependency review and Red/Green/Refactor evidence. |
| Strict Red/Green/Refactor | Required | Tasks will assign separate tests, implementation, and refactor commits for each independent behavior. |
| Secret handling and bounded parsing | Pass for design | Explicit secret ownership, redaction, limits-before-allocation, and cleanup contracts are defined in `spec.md` and `contracts/`. |
| Dependency and platform policy | Conditional | Exact versions, feature sets, transitive licenses, MSRV, native build cost, and supported targets must pass KMIPKIT-0011 checks in implementation. |
| Synchronous public API | Pass | Tokio exists only behind the owned worker; public constructors and `exchange`/`execute` remain synchronous. |
| Accepted architecture | Pass | ADR-0015 accepts the worker/Hyper backend, partially supersedes ADR-0005's implementation choices, and records the per-transport connection and TLS-session policies. The public API remains synchronous and TLS policy remains TLS 1.3/mTLS. |
| Coverage and verification | Required | Transport and changed-code coverage, all three CI operating systems, security checks, and 100% traceability are release gates. |

## Project Structure

### Documentation (this feature)

```text
specs/013-production-transport/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── tls-configuration.md
│   ├── raw-ttlv-tls.md
│   ├── https-ttlv.md
│   └── timeout-and-delivery.md
├── checklists/
└── tasks.md
```

### Source Code (repository root)

```text
crates/kmipkit-transport/src/
├── config.rs
├── secret.rs
├── tls.rs
├── timeout.rs
├── resolver.rs
├── worker.rs
├── raw_tls.rs
└── https.rs
crates/kmipkit-client/src/
└── lib.rs                 # production constructor using validated KMIPKit config
crates/kmipkit-test-support/src/
├── pki.rs                 # ephemeral test certificate helpers, if shared
├── dns.rs                 # deterministic loopback DNS fixture
└── local_transport.rs     # loopback-only listener helper
crates/kmipkit-transport/src/
└── tls_policy.rs          # private TLS 1.3/AWS-LC/key-log/early-data policy builder
crates/kmipkit-transport/tests/
├── raw_tls.rs
├── https.rs
├── tls_policy.rs
└── timeout_delivery.rs
docs/architecture/transport-security.md
docs/adr/0005-transport-and-tls.md
```

The implementation must adapt this module split to existing crate conventions during task generation; modules stay internal unless their documented contract is explicitly public.

**Structure Decision**: Extend the existing `kmipkit-transport` and `kmipkit-client` crates in the
Cargo workspace. Keep public transport configuration and documented low-level exchange in
`kmipkit-transport`; keep TLS parser/runtime helpers private; construct the existing typed client
only from validated project-owned adapters. The production client retains the immutable
KMIPKIT-0012 `ClientConfiguration` separately from transport configuration. The extension registry
remains owned by that client configuration and is not folded into `TransportConfig`; before
building or encoding an outgoing KMIP request, the client checks each attached extension's private
registry provenance against its retained configuration.

## Phase 0: Research Decisions

All research questions are resolved in [research.md](research.md). Key decisions:

1. Use Hyper's HTTP/1 connection parser with a custom post-TLS I/O wrapper instead of `reqwest::blocking`, because strict write, read, and total deadlines must remain independent.
2. Own a current-thread Tokio runtime on one private worker thread per client. Calls cross a bounded channel, remain synchronous to callers, and are serialized. The worker lazily connects; it reuses HTTPS connections but closes each raw-TLS connection after its one response frame.
3. Apply connect as a single deadline across system name resolution, TCP connect, and TLS handshake; apply read/write timeout to each blocked I/O operation and reset after forward progress; apply the total timeout as a monotonic absolute exchange deadline.
4. Preserve the accepted certificate, TLS, no proxy, no redirect, no compression, no retry, and delivery-state policies. HTTP/1 protocol parsing remains delegated to Hyper; the project validates KMIP-specific HTTP status, headers, and bounded body. A configured parser header-count/input-buffer bound is distinct from the KMIP response-body limit.
5. Accept ADR-0015 under the maintainer's delegated authorization, partially superseding ADR-0005's backend choices and recording the sync-worker/Hyper tradeoff. T001 amends both canonical connection/TLS-policy documents before implementation.

## Phase 1: Design and Contracts

- [Data model](data-model.md): validated configuration, timeout policy, secret inputs, TLS identity/trust, worker/connection lifecycle, and delivery state.
- [TLS configuration contract](contracts/tls-configuration.md): accepted inputs, file-read timing, certificate verification, CRLs, and redaction.
- [Raw TTLV/TLS contract](contracts/raw-ttlv-tls.md): one bounded frame, pre-allocation checks, partial-I/O behavior, and connection invalidation.
- [HTTPS TTLV contract](contracts/https-ttlv.md): HTTP/1.1 message, required headers, response validation, parser behavior, and no automatic client behaviors.
- [Timeout and delivery contract](contracts/timeout-and-delivery.md): connect/write/read/total semantics, deadline precedence, error delivery state, and cancellation.
- [Quickstart](quickstart.md): run the public synchronous API against a local ephemeral TLS peer and interpret expected pass/failure results.

## Constitution Check After Design

The design remains within product scope. Its accepted-architecture changes are the internal transport worker/backend, per-transport connection lifecycle, and bounded TLS session trust snapshot, recorded in ADR-0015 and the canonical documents. The public API remains synchronous, single-endpoint, mTLS-only, and no-retry. No implementation code may begin until dependency-policy review and the T001 design reviews pass; exact dependency checks remain required in T003.

## Complexity Tracking

| Added complexity | Why needed | Simpler alternative rejected because |
|---|---|---|
| Tokio worker thread and bounded command channel per client | Prevent nested-runtime panics while preserving a synchronous API and reusable HTTPS connection; keep async timers cancellable. | Calling `block_on` on the caller thread can panic inside an existing runtime; creating a runtime per exchange loses HTTPS reuse and increases cost. |
| Hyper HTTP/1 connection driver and post-TLS `DeadlineIo` | Preserve a maintained HTTP parser while applying independent I/O deadlines and delivery tracking. | `reqwest::blocking` has no independent read/write/total controls; manual HTTP framing adds parser risk. |
| Private transport state machine and worker shutdown protocol | Ensure timeouts, reconnects, and client drop do not replay requests or leave a live connection/thread. | Global runtime or mutable singleton would violate per-client state isolation and the no-global-state policy. |
