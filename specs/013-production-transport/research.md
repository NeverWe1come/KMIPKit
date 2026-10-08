# Research: Production TLS and HTTPS Transports

**Research date**: 2026-10-08
**Scope**: Determine a maintainable implementation that preserves KMIPKit's synchronous API and accepted security policy while enforcing independent connect, write, read, and total deadlines.

## Decisions

### D1. Use Hyper HTTP/1 over KMIPKit-owned Tokio and rustls I/O

**Decision**: Use `hyper`'s low-level HTTP/1 client connection API to parse and serialize HTTPS messages. Build the TLS stream using `tokio-rustls` and an explicit `rustls::ClientConfig` with the AWS-LC provider, TLS 1.3 only, caller-selected trust roots, and the validated client identity. Keep the raw TTLV adapter on the same owned Tokio I/O foundation. The Hyper dependency is built with only the client and HTTP/1 features; do not enable HTTP/2 or compression integrations.

**Rationale**:

- The blocking Reqwest builder currently exposes a shared connect/read/write timeout and a separate connect timeout. It does not expose independent blocking read and write timeouts, so it cannot fulfill FR-011 without weakening that contract. See [Reqwest blocking ClientBuilder](https://docs.rs/reqwest/0.13.5/reqwest/blocking/struct.ClientBuilder.html).
- Hyper exposes an HTTP/1 connection handshake over caller-supplied I/O. KMIPKit can therefore retain a maintained HTTP parser while wrapping the post-TLS I/O for read/write deadlines and delivery-state accounting. See [Hyper HTTP/1 client handshake](https://docs.rs/hyper/1.11.1/hyper/client/conn/http1/fn.handshake.html).
- Rustls supports explicit provider selection and an explicit protocol-version list; its client configuration supports root trust and client certificates. Select the AWS-LC provider and TLS 1.3 only on each configuration instead of changing process-global provider state. See [rustls ClientConfig](https://docs.rs/rustls/0.23.45/rustls/client/struct.ClientConfig.html) and [rustls ConfigBuilder](https://docs.rs/rustls/0.23.45/rustls/struct.ConfigBuilder.html).
- `tokio-rustls` wraps a Tokio stream with the supplied `rustls::ClientConfig`. See [tokio-rustls client API](https://docs.rs/tokio-rustls/0.26.6/tokio_rustls/client/index.html).

**Alternatives considered**:

| Alternative | Result | Reason |
|---|---|---|
| `reqwest::blocking` | Rejected for this contract | Supports a shared connect/read/write timeout and a separate connect timeout, but not distinct stable read and write settings. Repeated reads can also make a shared per-read timeout differ from the absolute total deadline. |
| `ureq` 3.x | Rejected | Public phase timeout controls are useful, but its stable TLS builder does not select TLS 1.3 only or inject the required AWS-LC provider. Those controls require unversioned connector/provider APIs. |
| `ureq` 2.12 | Rejected | It accepts a supplied rustls config and phase timeouts, but it is an older superseded major line; no reason to pin a stale line for a new production backend. |
| Hand-written HTTP/1 parser | Rejected | It would put HTTP framing and parser-hardening risk in KMIPKit. Hyper already supplies a protocol parser over an I/O object. |
| OpenSSL/curl | Rejected | It adds an alternate TLS implementation/provider and native unsafe dependency surface, contrary to the accepted rustls/AWS-LC policy. |

### D2. Keep public calls synchronous using a private worker per client

**Decision**: One private OS worker thread owns a Tokio current-thread runtime, the system resolver call, TLS state, and the lazily established HTTPS connection. Resolve with `std::net::ToSocketAddrs` inside `spawn_blocking`, after acquiring one of 32 shared permits for the loaded KMIPKit library instance. Admission is fail-fast; the permit stays with the closure until the native resolver returns. Each worker runtime sets `max_blocking_threads(1)`. Make at most one resolver call per endpoint connection-establishment attempt, retain at most 16 returned addresses in OS order, and do not cache them in KMIPKit. The operating system owns DNS family-query behavior, packet retries, upstream concurrency, resolver caching and invalidation, hosts/search rules, and split-DNS/VPN routing. A timed-out or canceled caller returns without waiting for an already-started native lookup; the call may continue and retain its permit. Discard its late result and check cancellation/deadline before every candidate connection and before KMIP dispatch. The exchange total deadline takes precedence. The public client submits one bounded exchange through a synchronous channel and waits. One exchange may run and at most one may wait. HTTPS may reuse one healthy connection; raw TLS closes after one response frame because raw streaming has no separate HTTP boundary to protect the next call from unsolicited surplus bytes. Candidate addresses are alternatives only for the same configured endpoint, are tried sequentially in OS order, and may be attempted before request-dispatch commit and within the connect and total deadlines.

**Rationale**: Calling `Runtime::block_on` directly from application code can panic when that code already runs inside a Tokio runtime. A dedicated worker retains a synchronous user contract, avoids nested-runtime use, and provides a natural owner for serialized connection state. Hickory's in-process resolver flattens configured name servers and cannot preserve per-interface split-DNS/VPN routing; the platform resolver is required for host policy. Rust documents that `ToSocketAddrs` can block. Tokio runs it on the blocking pool, where started jobs cannot be aborted and the queue has no backpressure, so the shared permit must be acquired before submission and retained by the closure. See [Tokio runtime bridging](https://docs.rs/tokio/1.53.2/tokio/runtime/index.html), [Tokio `spawn_blocking`](https://docs.rs/tokio/1.53.2/tokio/task/fn.spawn_blocking.html), [Tokio runtime builder](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Builder.html), [Tokio runtime shutdown](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Runtime.html), and [Rust `ToSocketAddrs`](https://doc.rust-lang.org/1.94.0/std/net/trait.ToSocketAddrs.html).

**Costs and safeguards**:

- A constructed client owns one additional worker thread. It opens no network connection until the first request, and construction still performs configuration and credential validation only.
- The command queue is bounded to one pending exchange; calls through one client are serialized. The total deadline includes queue wait. Shared exchange-control state linearizes cancellation against handing the request to the HTTP/TLS writer: if a caller returns `NotSent`, the worker can never transmit that exchange later. No global mutable runtime or connection pool is introduced.
- Worker shutdown, request cancellation, panic containment, thread creation failure, and cleanup are public error/test cases. The worker never retries the in-flight KMIP request.
- Tokio's blocking pool has no queue backpressure, `max_blocking_threads` applies per runtime, and started blocking tasks cannot be interrupted. A shared governor is therefore required to bound KMIPKit-submitted active and queued lookups across clients. An abort-on-drop guard may prevent a queued task from starting, but cannot stop a started `ToSocketAddrs` call. `shutdown_timeout(Duration::ZERO)` bounds the worker's wait; the native task may outlive worker shutdown. The permit remains in the closure until it returns. Late-result isolation is tested with an injected blocking resolver, including a check that cancellation prevents TCP/TLS and KMIP dispatch. See [Tokio `JoinHandle`](https://docs.rs/tokio/1.53.2/tokio/task/struct.JoinHandle.html) and [Tokio runtime shutdown](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Runtime.html).
- OS-specific DNS traffic, retries, packet concurrency, cache contents/TTL, hosts-file behavior, and split-DNS routing cannot be given portable numeric assertions by KMIPKit. Cross-platform smoke tests verify that the platform resolver API is used and that returned addresses are retained in order and capped; controlled lifecycle tests inject the synchronous lookup function. Resolver packets already in progress may outlive a KMIPKit timeout.

### D3. Define timeout and delivery semantics at the I/O boundary

**Decision**:

- **Connect (10 seconds)**: One deadline covers system name resolution, TCP establishment, and TLS handshake. If it expires before the TLS channel is ready, return `NotSent`; no KMIP request byte has been written.
- **Write (30 seconds)**: Maximum time any one blocked application-data write/flush operation or Hyper sender-readiness wait may remain without progress. Successful write/flush progress resets the inactivity timer. Hyper `SendRequest::ready()` completes before dispatch commit; a readiness timeout is still `NotSent`. The worker then atomically commits request dispatch immediately before handing the request to Hyper or the raw TLS writer; after commit, failures are conservatively `PossiblySent` even if no request byte was accepted. This deliberately avoids trying to infer an HTTP header/body boundary from bytes observed below Hyper. `DeadlineIo` must bound `poll_write` and `poll_flush`, implement vectored-write deadlines if supported, or explicitly report that vectored writes are unsupported.
- **Read (30 seconds)**: Maximum time any one blocked application-data read may remain without progress. Successful progress resets the inactivity timer. It applies to the complete response header and body path.
- **Total (60 seconds)**: One monotonic absolute deadline starts at public exchange entry and applies across channel handoff, resolution, connect, TLS, write, and response read. The stricter of a phase deadline and the remaining total deadline governs each I/O wait.
- Explicit unbounded is represented distinctly from `Duration::ZERO`; a zero duration is an immediate deadline.
- A total or phase timeout invalidates the active connection. Exchange control uses one linearization gate for cancellation, dispatch, response observation, and finalization. The gate includes a `Finalized` bit with delivery evidence (a packed compare/exchange state or lock), not a separate monotonic byte flag. `DeadlineIo` commits the first decrypted positive read synchronously before returning bytes to Hyper. If that observation wins before timeout finalization, the returned state is `ResponseStarted`; if timeout finalization wins first, it cancels the request future/connection driver and invalidates the connection before snapshotting the error, so no later bytes are consumed for that exchange.

**Rationale**: The underlying architecture already defines separate connect/write/read/total defaults. This decision makes the commonly available socket-style write/read inactivity semantics precise while preserving a cumulative whole-exchange cap. Hyper does not expose these four policies as one configuration switch; KMIPKit must implement and test them around the I/O object. Tokio timeouts cancel async futures when they expire, and its `timeout_at` accepts an absolute deadline. See [Tokio time utilities](https://docs.rs/tokio/1.53.2/tokio/time/index.html).

### D4. Preserve HTTP and TLS behavior in KMIPKit configuration

**Decision**: Do not enable or add redirect, proxy discovery, compression/decompression, retry, cookie, HTTP/2, TLS keylog, or 0-RTT behavior. Connect directly to the one configured endpoint. Use a single Hyper HTTP/1 connection per client with explicit maximums of 64 response headers and 64 KiB of parser input buffer. Validate status, duplicate/ambiguous headers, media type, `Content-Length`, transfer/content encodings, response cap, and bounded body before returning `TransportResponse`. If the HTTP driver reports an unsolicited/surplus response on a reused connection, invalidate that connection and never associate the surplus response with a later KMIP call.

**Rationale**: These are existing accepted security and transport boundaries. Hyper's low-level one-connection API does not supply a high-level retry or redirect policy; the adapter keeps only the needed protocol machinery and implements the rest of KMIPKit's narrow contract.

### D5. Exact dependency versions are frozen at implementation time

**Decision**: The corrected direct runtime stack pins Hyper 1.12.0 (MSRV 1.63, MIT), Tokio 1.53.2 (MSRV 1.71, MIT), rustls 0.23.45 (MSRV 1.71, Apache-2.0/ISC/MIT), tokio-rustls 0.26.6 (MSRV 1.71, Apache-2.0/MIT), rustls-native-certs 0.8.4 (MSRV 1.71, Apache-2.0/MIT/ISC), and bytes 1.12.1 (MSRV 1.57, MIT). After T002a, these six remain exact-pinned in the workspace manifests and Hickory is absent from the production graph and lockfile. Select only Hyper `client` + `http1`, rustls `aws_lc_rs` + `std`, tokio-rustls `aws_lc_rs`, Tokio `io-util` + `macros` + `net` + `rt` + `sync` + `time`, and bytes `std`; disable defaults that would enable HTTP/2, TLS 1.2, early data, or unrelated runtime features. Use Rust's standard `ToSocketAddrs` without another resolver dependency. The direct crates fit KMIPKit's Rust 1.94 MSRV and Apache-2.0 distribution. The full transitive license report, advisories, native build tools, feature tree, and Linux/Windows/macOS targets must still pass KMIPKIT-0011 dependency review after Hickory removal. AWS-LC native build impact is included in cross-platform evidence. No git dependency is permitted.

**Rationale**: The repository's dependency policy in KMIPKIT-0011 requires a recorded capability rationale and reproducible lockfile. The exact pins and feature selection were checked against official crate metadata; dependency review gates production implementation. See [Hyper 1.12.0](https://docs.rs/crate/hyper/1.12.0), [Tokio 1.53.2](https://docs.rs/crate/tokio/1.53.2), [rustls 0.23.45](https://docs.rs/crate/rustls/0.23.45), [tokio-rustls 0.26.6](https://docs.rs/crate/tokio-rustls/0.26.6), [rustls-native-certs 0.8.4](https://docs.rs/crate/rustls-native-certs/0.8.4), and [bytes 1.12.1](https://docs.rs/crate/bytes/1.12.1).

### D6. Provide explicitly selected platform roots without OS revocation fetches

**Decision**: Use `rustls-native-certs` to load native roots only when the caller explicitly selects platform trust, then use rustls/WebPKI chain and hostname validation with the selected AWS-LC provider. Treat any root-loader error as trust-source failure. When set, `SSL_CERT_FILE` takes precedence for this mode and supplies the selected root bundle; callers selecting platform trust are responsible for this process environment. Test the override in an isolated process. Do not use a system verifier that could add opaque platform revocation/network behavior; revocation evidence for this feature is caller-supplied CRLs only.

**Tradeoff**: This loads platform root certificates but does not preserve every platform-specific distrust or revocation decision. The API and docs must state this precisely; callers needing other distrust/revocation semantics can select caller-provided trust and CRLs. Cross-platform loader, environment override, errors, and tests are dependency-policy gates. `rustls-native-certs` documents Windows, macOS, and Linux support plus `SSL_CERT_FILE` behavior; see [rustls-native-certs 0.8.4](https://docs.rs/crate/rustls-native-certs/0.8.4).

### D7. Bound the authentication snapshot used by TLS session resumption

**Decision**: Keep TLS 1.3 session tickets only in a custom per-client rustls
`ClientSessionStore`, capped at 16 entries and one hour from local monotonic
insertion time (or the shorter server-provided ticket lifetime). A resumed
session inherits the identity, hostname, certificate-time, root, and CRL
decision from its full handshake; it does not repeat certificate or CRL
validation. This bounds how long changed or expired trust evidence can remain
in effect through resumption. Rebuilding the client to apply updated roots,
identity, or CRLs creates a new empty store. Disable 0-RTT and key logging.

**Rationale**: Rustls exposes the session-store interface that controls ticket
insertion and retrieval, allowing KMIPKit to enforce a local expiry in addition
to rustls' server-ticket lifetime. Session data remains configuration-scoped
and is never persisted. See [rustls `ClientSessionStore`](https://docs.rs/rustls/0.23.45/rustls/client/trait.ClientSessionStore.html) and [TLS 1.3 client session value](https://docs.rs/rustls/0.23.45/rustls/client/struct.Tls13ClientSessionValue.html).

## ADR consequence

The backend choice in accepted [ADR-0005](../../../docs/adr/0005-transport-and-tls.md) must change because the accepted `reqwest::blocking` API cannot meet the timeout contract. This feature prepares a narrowly scoped ADR amendment: raw TTLV remains rustls; HTTPS changes to Hyper HTTP/1 over the private Tokio/rustls worker; TLS policy, public sync API, no-retry rule, and transport security boundary remain unchanged. The amendment and its dependency rationale are part of this feature's review package. Production code is gated on the accepted ADR and reviewed specification.
