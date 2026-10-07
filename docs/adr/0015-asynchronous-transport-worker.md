# ADR-0015: Tokio worker for synchronous TLS and HTTPS transports

**Status**: Accepted
**Date**: 2026-10-07
**Decision owner**: KMIPKit maintainer
**Related ADRs**: [ADR-0005](0005-transport-and-tls.md), [ADR-0014](0014-public-transport-exchange-contract.md)
**Specification**: [KMIPKIT-0013](../../specs/013-production-transport/spec.md)

## Context

ADR-0005 selected `reqwest::blocking` for HTTPS and a fully synchronous
transport implementation. KMIPKit's accepted transport contract also requires
separate connect, write, read, and total deadlines. The stable blocking
Reqwest builder provides one connect timeout and one timeout shared by
connect/read/write; it cannot implement the three independent phase/total
limits without changing the accepted contract. A total deadline is also
different from restarting a shared timeout on each response-body read.

KMIPKit must preserve its public synchronous API, TLS 1.3-only rustls/AWS-LC
policy, mTLS, explicit trust, strict HTTP/1.1 behavior, endpoint selection,
response limits, delivery state, and no-retry rule. The accepted project
definition and transport architecture said that a client owned one reusable
connection. The accepted decision below narrows that reuse rule by transport:
HTTPS can safely reuse a healthy framed connection, while raw TTLV has no
separate HTTP message boundary and an unsolicited surplus frame could become
the next response. The raw-TLS close-after-one-frame behavior is therefore an
explicit architecture exception, recorded in both canonical documents under
T001. HTTP framing continues to use a maintained parser instead of
project-written HTTP parsing.

## Decision

This ADR supersedes ADR-0005's HTTPS backend and runtime clauses as follows:

1. Use Hyper's low-level HTTP/1 client connection parser over a caller-owned
   KMIPKit I/O stream. Enable only HTTP/1 client features.
2. Use Tokio and tokio-rustls internally for cancellable transport I/O and
   deadline timers. Use a private per-client worker thread with its own
   current-thread runtime so public constructors and exchange methods remain
   synchronous and safe to call from a thread already running a Tokio runtime.
3. Use one active and at most one queued exchange per client; resolve/connect
   lazily; reuse a healthy HTTPS connection; close raw-TLS connections after
   exactly one response frame to prevent unsolicited surplus data from being
   mistaken for a later response. This explicitly amends the general
   reusable-connection statement in `docs/design/project-definition.md` and
   `docs/architecture/transport-security.md`: only HTTPS reuses a connection;
   raw TLS opens a connection per exchange. T001 records the completed
   amendments. Close any failed connection and never retry a failed KMIP
   request.
4. Use rustls with an explicitly selected AWS-LC provider and TLS 1.3 only.
   Keep explicit trust, mTLS, certificate validation, caller CRLs, disabled
   0-RTT, and disabled key logging from ADR-0005. Use a per-configuration
   session store capped at 16 tickets and a one-hour local monotonic age; a
   resumed session inherits the peer identity and trust/CRL decision from its
   full handshake without revalidation. Rebuilding the client applies changed
   trust inputs and starts with an empty store.
5. Use Hickory's asynchronous resolver configured from system DNS settings.
   Bound lookup behavior to one retry after the initial query (two attempts
   total), two concurrent nameserver requests per query, at most 32 active
   requests on each multiplexed upstream connection (not an aggregate
   per-client cap), 128 cached responses per client, and at most 16 A/AAAA
   candidates per lookup. Validate resolver behavior, hosts/search/split-DNS compatibility,
   and address-candidate policy on Linux, Windows, and macOS. DNS candidates
   are for the configured endpoint only; another address is allowed only
   before request-dispatch commit and within the connect deadline. It must not
   delay the synchronous public call beyond the effective total deadline.
6. Keep redirects, proxy discovery, cookies, compression/decompression,
   HTTP/2, retries, and automatic failover disabled. KMIPKit validates
   application status/headers and streams the bounded response itself.
7. Load platform roots only when explicitly selected, through
   `rustls-native-certs`, then validate with rustls/WebPKI. Treat any loader
   error as trust-source failure. This imports root certificates, not all
   platform-specific distrust or revocation behavior; caller CRLs remain the
   only configured revocation evidence, with no online fetch.
8. Configure Hyper's HTTP/1 parser with at most 64 response headers and a
   64 KiB input-buffer maximum. These parser bounds are separate from the
   response-body cap. Use a bounded request-body owner that zeroizes its
   initialized bytes; document Hyper/rustls/OS-owned buffers as outside the
   KMIPKit zeroization guarantee.

Timeouts have this exact meaning: connect bounds hostname resolution, TCP
connect, and TLS handshake; read/write each bound one blocked application I/O
operation and reset only after progress; total is one monotonic absolute
deadline beginning at public exchange entry, including queue wait. The
stricter active phase or remaining total deadline controls each wait. For
HTTPS, Hyper sender readiness is bounded by the write and total deadlines and
must succeed before dispatch can commit. Atomic exchange control commits
immediately before handing the request to Hyper or the raw TLS writer. If
cancellation wins before this point, `NotSent` means the request can never be
sent later. After dispatch commit, including an HTTPS failure after headers
but before a TTLV body, delivery is conservatively `PossiblySent` until a
response byte arrives; no header/body boundary is inferred from I/O bytes
below Hyper. Response observation and timeout finalization share one gate
containing delivery evidence and a `Finalized` bit. The TLS I/O wrapper
records its first decrypted positive read in that gate before returning bytes
to Hyper. If timeout finalization wins, the worker cancels the request and
connection driver and invalidates the connection before capturing the result.
A timeout invalidates the connection.

## Alternatives considered

- **Keep `reqwest::blocking` and relax timeout semantics**: rejected because
  the independent timeout and absolute total-deadline requirements are part
  of accepted project architecture.
- **Use ureq 3.x**: rejected because its stable TLS API does not expose the
  required TLS version selection and AWS-LC provider controls.
- **Use ureq 2.x**: rejected because it would pin a superseded major release
  for a new production implementation.
- **Implement HTTP parsing locally**: rejected because it moves parser and
  request-smuggling risk into KMIPKit.
- **Use a public async client API**: rejected because KMIPKit's public 1.0
  client contract is synchronous and must stay language-binding friendly.

## Consequences

- Each constructed production client owns a worker thread and runtime. This
  cost, thread startup/shutdown, cancellation, and behavior under many clients
  must be documented and measured before 1.0.
- The transport implementation must maintain and test an async I/O wrapper
  that enforces read/write inactivity deadlines, the total deadline, and
  delivery-state observations without logging payloads.
- Request-body buffers owned by KMIPKit are bounded and zeroizing; payload
  copies held by Hyper, rustls, or the OS are documented as external copies.
- Hyper remains responsible for HTTP/1 framing; KMIPKit owns the narrower
  KMIP headers, status, size, no-proxy/no-redirect/no-compression/no-retry,
  and response-owner rules.
- Hyper, Tokio, tokio-rustls, rustls, bytes, rustls-native-certs, Hickory,
  zeroize, and AWS-LC become dependencies
  subject to the full KMIPKIT-0011 version, license, advisory, feature,
  toolchain, supply-chain, and supported-target checks. The lockfile and
  feature sets must be exact and reproducible.
- This does not change profile claims, public sync APIs, TLS policy, OASIS
  scope, or the rule that a later distinct operation may reconnect.

## Acceptance record

The maintainer's standing instruction in this conversation delegates
authorization to execute the bounded KMIPKit roadmap without further approval
requests. KMIPKIT-0013 and this ADR were accepted under that instruction after
independent QA/specification and security/design reviews. The exact review
findings and their resolutions are recorded in
`specs/013-production-transport/approval-record.md`. This authorization does
not waive human review and approval of the eventual draft PR, human-only
merging, or the qualified independent human security audit required before
1.0.0. No implementation or release readiness is claimed by accepting this
design.
