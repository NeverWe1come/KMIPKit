# Feature Specification: Production TLS and HTTPS Transports

**Feature Branch**: `feature/KMIPKIT-0013-production-transport`
**Created**: 2026-10-07
**Status**: Accepted
**Input**: User-approved KMIPKit roadmap: add production TLS 1.3 mutual-authenticated raw TTLV and HTTPS/HTTP 1.1 adapters, construct a synchronous typed client from validated configuration, and preserve the existing public low-level exchange contract.

## User Scenarios & Testing

### User Story 1 - Configure a verified KMIP connection (Priority: P1)

As a KMIP application developer, I can supply one endpoint, explicit server trust, and a client identity so KMIPKit can prepare a secure transport without contacting a server or sending a hidden KMIP operation.

**Why this priority**: Both wire transports depend on validated configuration and safe ownership of TLS credentials.

**Independent Test**: Construct configurations from in-memory and file-based certificates and keys, then assert accepted configuration is redacted and invalid input fails before any network activity.

**Acceptance Scenarios**:

1. **Given** a valid endpoint, explicit CA trust, and matching client certificate/key, **When** a caller constructs a transport-backed client, **Then** configuration succeeds without opening a socket or issuing a KMIP request.
2. **Given** an invalid URL, unsupported key encoding, absent client identity, missing trust source, or malformed certificate/CRL, **When** the caller constructs a client, **Then** construction fails with a safe category and no credential bytes or configured secret values in diagnostics.
3. **Given** certificate/key material supplied in memory or by an explicitly selected PEM/DER file source, **When** construction succeeds, **Then** KMIPKit reads file sources once, keeps no path for later rereading, and owns secret input through a zeroizing type.

### User Story 2 - Exchange a KMIP message over raw TTLV/TLS (Priority: P1)

As a KMIP application developer, I can exchange one complete TTLV message through raw TLS with a verified, mutually authenticated server.

**Why this priority**: Raw TTLV over TLS is one of the two 1.0 transports and is the simplest end-to-end use of the existing exchange contract.

**Independent Test**: Use an ephemeral TLS 1.3 server and generated PKI to exchange bounded messages, observe exact request bytes, and exercise framing, timeout, one-frame connection closure, session resumption, and delivery-state failures.

**Acceptance Scenarios**:

1. **Given** a server presenting a trusted, valid certificate and requiring the configured client identity, **When** a bounded request is exchanged, **Then** the peer receives the exact caller bytes and the caller receives one complete response wrapped in the zeroizing response owner.
2. **Given** an unknown CA, expired/not-yet-valid certificate, hostname/SAN mismatch, revoked server certificate covered by a supplied CRL, or absent/mismatched client identity, **When** the connection is attempted, **Then** the handshake fails closed and no KMIP request bytes are reported as sent.
3. **Given** an invalid TTLV response header, declared length over the effective response limit, arithmetic overflow, truncated frame, or incomplete EOF, **When** the transport reads the response, **Then** it closes/invalidates that connection, zeroizes initialized partial response storage, and returns a redacted error with the strongest delivery state.

### User Story 3 - Exchange a KMIP message over HTTPS (Priority: P1)

As a KMIP application developer, I can send TTLV as an HTTPS request to a configurable target URI and receive a bounded TTLV response under the same TLS trust and client-identity rules.

**Why this priority**: HTTPS/HTTP 1.1 is the other selected 1.0 transport and has request/response framing rules beyond raw TLS.

**Independent Test**: Use an ephemeral HTTPS/HTTP 1.1 server to inspect method, target URI, headers, body bytes, TLS identity, and response validation for success and malformed/error cases.

**Acceptance Scenarios**:

1. **Given** a valid HTTPS endpoint and client identity, **When** a request is exchanged, **Then** KMIPKit sends one HTTP/1.1 POST with the exact TTLV body, `Content-Type: application/octet-stream`, the exact `Content-Length`, and `Cache-Control: no-cache`.
2. **Given** no custom target path, **When** a request is sent, **Then** its target URI is `/kmip`; **And given** a configured valid target URI, **Then** that URI is used without changing the endpoint host or TLS name.
3. **Given** an HTTPS response that is not HTTP 200, has a missing/ambiguous or unexpected content type or content length, uses unsupported content encoding, exceeds the response limit, or has malformed/truncated HTTP framing, **When** KMIPKit receives it, **Then** it fails closed, invalidates the connection, clears KMIPKit-owned partial bytes, and returns a redacted error marked `ResponseStarted` when response bytes were observed.
4. **Given** an HTTP redirect, a configured/system proxy, compression feature, or TLS version below 1.3, **When** an HTTPS request is attempted, **Then** KMIPKit does not follow, use, negotiate, or enable that behavior.

### User Story 4 - Execute a typed KMIP request using production configuration (Priority: P2)

As a KMIP application developer, I can construct the existing synchronous typed client from its
immutable KMIPKIT-0012 client configuration and validated transport configuration, then execute
its existing typed request API without supplying a custom transport or raw KMIP bytes.

**Why this priority**: A production transport is useful only when it connects to the existing typed client without weakening its validated request boundary.

**Independent Test**: Construct the client over each transport using an ephemeral peer, execute the
currently supported typed request through the public Rust API, and inspect the exact wire exchange
and typed result. Also validate an extension with one immutable `ClientConfiguration`, attach it to a
request for a production client retaining a different configuration, and verify rejection before
request construction, encoding, or exchange; repeat with the same configuration and verify the
existing valid wire behavior.

**Acceptance Scenarios**:

1. **Given** an existing immutable KMIPKIT-0012 `ClientConfiguration` and valid transport
   configuration, **When** an application constructs the typed client, **Then** the client retains
   the two configurations separately, owns the selected production adapter, and does not accept a
   caller-implemented `Transport`.
2. **Given** the current closed typed request set, **When** an application executes a request, **Then** the client encodes and validates it using its configured `CodecLimits`, passes exactly the response-byte limit to the adapter, decodes the bounded response, and returns a typed result without exposing raw response bytes.
3. **Given** a failed exchange, **When** the next distinct operation is executed, **Then** the client may establish a new connection but never retries the failed KMIP request or automatically selects an alternate endpoint.
4. **Given** a production client retaining immutable `ClientConfiguration` A separately from its
   transport configuration, **When** a request carries a `ClientRequestMessageExtension` whose private
   registry provenance belongs to configuration B, **Then** execution returns sanitized
   `InvalidInput` with `NotSent` before constructing the outgoing KMIP request, encoding it, or
   invoking the adapter.
5. **Given** a request carrying a `ClientRequestMessageExtension` validated by the same immutable
   `ClientConfiguration` retained by the production client, **When** the request is executed,
   **Then** it follows the existing typed encoding and exchange path and preserves its valid Message
   Extension wire representation.

## Scope and Exclusions

- This feature adds production raw TTLV over TLS 1.3 and TTLV over HTTPS/HTTP 1.1, both with mutual TLS.
- Shared integration-test PKI fixtures use an exact-pinned, test-support-only certificate generator configured with AWS-LC and no default features; this dependency is excluded from production dependency paths and must not enable `ring`.
- The client is synchronous and owns one configured endpoint and its transport. Calls on one client are serialized.
- The production client retains the immutable KMIPKIT-0012 `ClientConfiguration` separately from
  transport configuration. Transport configuration does not contain or replace the extension
  registry.
- The typed client may expose only the request variants already implemented in the approved client API. This feature does not add KMIP operation schemas or server-initiated behavior.
- The low-level `kmipkit-transport::Transport::exchange` remains a direct caller-byte API and performs no KMIP encoding or schema validation. The typed client and top-level facade do not accept raw request bytes or arbitrary transport injection.
- This feature does not claim conformance to the complete HTTPS Client KMIP 2.1 Profile or any other profile. It implements selected HTTPS Client transport clauses needed for KMIPKit's HTTP/1.1 + TTLV scope; profile claims remain governed by KMIPKIT-0010 and complete profile evidence.
- HTTP/1.0, HTTP/2, JSON/XML, server-initiated operations, automatic retries/failover, proxies, redirects, compression, online OCSP/CRL retrieval, TLS 0-RTT, process-wide TLS session sharing, and an insecure certificate-acceptance switch are out of scope.
- KMIPKit does not provide local cryptographic algorithms or manage private-key lifecycle outside client TLS authentication.

## Requirements

### Functional Requirements

- **FR-001**: A production client MUST be constructible only from a validated KMIPKit transport configuration. Construction MUST NOT accept an arbitrary caller-implemented `Transport`, raw request bytes, or a generic TTLV encoder as the typed execution input.
- **FR-002**: Every production connection MUST use TLS 1.3 only, rustls with the `aws-lc-rs` cryptographic provider, and mutual TLS. Every full handshake MUST verify the server certificate chain, validity period, and hostname/SAN. A resumed TLS 1.3 handshake MUST be limited to FR-006's per-configuration cache and inherits the peer identity and validation decision from that verified full handshake. There MUST be no public production option that bypasses certificate or hostname verification. [ADR-0005]
- **FR-003**: The caller MUST explicitly select server trust roots: caller-supplied CA certificates or platform trust. No implicit trust-root source may be selected. Platform trust means loading the platform's native root certificates with `rustls-native-certs` and validating them through rustls/WebPKI; it does not import platform-specific distrust flags or perform platform revocation checks. Caller-supplied CRLs MAY be configured and MUST be enforced when selected; KMIPKit MUST NOT perform network OCSP/CRL lookup. A platform trust-store loading error MUST fail closed with a redacted category.
- **FR-004**: The client MUST accept certificate chains and private keys from in-memory material or caller-specified files with an explicit PEM or DER encoding. Private keys MUST be unencrypted PKCS#8, PKCS#1, or SEC1 material supported by the selected provider. Encrypted-key formats and passphrase callbacks are unsupported and MUST be rejected without logging or formatting key material. File paths MUST be read once during client construction using ordinary operating-system path resolution (including caller-selected symlinks); file ownership, permissions, and path trust remain caller responsibilities. [Threat model §2]
- **FR-005**: KMIPKit-owned private-key input and temporary key copies MUST use explicit secret ownership and zeroize their initialized bytes before release. Configuration and transport `Debug`, `Display`, errors, logs, and tracing MUST redact secret bytes and credential-bearing inputs. Documentation MUST state that KMIPKit cannot zeroize caller copies, foreign-runtime copies, or storage retained by Hyper, rustls, the OS, or other third-party libraries, including HTTP request/response buffers. [ADR-0014]
- **FR-006**: TLS 0-RTT and TLS key logging MUST be disabled. The client MUST support TLS 1.3 session resumption using only in-memory state owned by one client configuration, with a maximum of 16 entries and a local maximum ticket age of one hour; sessions MUST NOT be persisted or shared across different identities/configurations. A resumed session inherits the peer identity, certificate-time, hostname, trust-root, and caller-CRL decisions from its full handshake; KMIPKit does not revalidate the server certificate or re-evaluate CRLs during resumption. Rebuilding the client is required to apply changed trust inputs, and its new session cache MUST be empty. Tests MUST prove same-configuration resumption, the documented trust-snapshot behavior and one-hour local age limit, a new full handshake with changed trust inputs and an empty cache, cross-identity isolation, bounded cache behavior, and no early application data. [ADR-0005; transport-security architecture]
- **FR-007**: The raw TLS adapter MUST exchange the caller's exact request bytes unchanged as one caller-supplied frame and MUST NOT encode, normalize, retain, log, or KMIP-schema-validate those bytes. Direct users are responsible for supplying one valid TTLV message; the typed client supplies the encoded message. The adapter MUST read the response's 8-byte TTLV header first, validate root tag/type/declared length and overflow-safe effective size limits before allocating its response body, then read exactly the declared value and padding for that frame. The one-frame raw-TLS stream behavior is KMIPKit's project contract; OASIS defines TTLV messaging and item encoding but does not define a separate raw-TLS stream-framing rule. [ADR-0014; KMIP Specification 2.1 §§10.1.1–10.1.5, 10.2, 10.4]
- **FR-008**: The HTTPS adapter MUST use the configured absolute HTTPS endpoint and HTTP/1.1 only. It MUST send a POST to a configurable origin-form request target (absolute path with optional query, default `/kmip`) and MUST NOT allow that target to change the endpoint authority or TLS name. Each request MUST contain exactly one `Host` header derived from the configured endpoint authority, preserving its hostname or bracketed IP literal and any explicitly configured port; the request target and optional TLS verification-name override MUST NOT change this header. Tests MUST verify hostname, bracketed IPv6 literal, and non-default port serialization. The caller's TTLV bytes MUST appear unchanged in the binary request body. Each request MUST specify `Content-Type: application/octet-stream`, the exact request `Content-Length`, and `Cache-Control: no-cache`. [RFC 9112 §3.2; OASIS KMIP Profiles 2.1 §5.3.1; catalog requirements KMIPKIT-REQ-PROF-5.3.1-001/002/003/004/005/008/009/010]
- **FR-009**: The HTTPS endpoint MUST be an absolute `https` URI without user information or fragments. The configurable request target MUST be origin-form and MUST be rejected if it contains an authority, scheme, or fragment. The adapter MUST reject redirects, disable proxy discovery and proxy use, disable all request/response compression and automatic decompression, and disable automatic retries. A configured TLS name MUST affect certificate-name validation only as explicitly configured and MUST NOT redirect the request to another endpoint.
- **FR-010**: A successful HTTPS exchange MUST require HTTP status 200, exactly one valid `Content-Type` whose media type is `application/octet-stream`, an unambiguous `Content-Length`, and a parsed response body exactly matching that length. The content length MUST be checked against the effective response limit before KMIPKit allocates or grows its response-body owner. Hyper's parser MUST be configured with a maximum of 64 response headers and a 64 KiB input-buffer limit; these parser limits are separate from the KMIP response-body limit. Other statuses, ambiguous headers, unsupported transfer/content encoding, parser-reported framing errors or truncation, and size violations MUST fail closed and invalidate that connection. If an unsolicited or surplus HTTP response is observed on a reused connection, the adapter MUST invalidate the connection and MUST NOT associate that response with a later KMIP exchange. These are KMIPKit client acceptance rules; they do not assert server-profile conformance.
- **FR-011**: Default timeout limits MUST be connect 10 seconds, write 30 seconds, read 30 seconds, and total request deadline 60 seconds. Each value MUST be configurable per client and overridable per exchange. Every public typed operation method MUST have a documented options-bearing variant, and each direct production-adapter byte exchange MUST accept the same per-exchange timeout overrides; an exchange without overrides inherits the client policy. Unbounded MUST be represented separately from a zero-duration immediate deadline. The connect deadline MUST cover system name resolution, TCP connection, and TLS handshake. Write and read deadlines MUST bound each blocked application-data I/O operation and restart only after successful progress; writer-readiness waits such as Hyper `SendRequest::ready()` MUST also be bounded by the write and total deadlines. The total deadline MUST use a monotonic absolute deadline created at public exchange entry and bound command-channel queueing/handoff, resolution, connection, TLS, writer readiness, request write, and response read. If it expires while an exchange waits for the worker, connection, or writer readiness before dispatch commit, that exchange MUST return `NotSent` without writing request bytes. Atomic exchange control MUST arbitrate expiry/cancellation against the first application-level request-write dispatch so a returned `NotSent` exchange can never be transmitted later. At every wait, the stricter phase deadline or remaining total deadline MUST win. A timeout before dispatch commit MUST report `NotSent`; a timed-out connection MUST be invalidated.
- **FR-012**: Before the worker atomically commits an exchange to its first application-level request-write dispatch, configuration, endpoint, request-size, DNS, connect, TLS, queue, and writer-readiness failures MUST report `NotSent`; cancellation winning before this commit MUST make later transmission impossible. The dispatch commit occurs before request bytes are handed to the TLS/HTTP writer. From that commit through the first observed response byte, failures MUST report `PossiblySent`, even if the writer later reports that zero TTLV body bytes entered TLS. For HTTPS this includes a failure after HTTP headers are emitted without a TTLV body; the state is deliberately conservative and does not claim that the peer received the KMIP request. Response-byte observation and timeout/cancellation finalization MUST share one atomic or locked linearization gate. The TLS wrapper MUST commit the first decrypted response-byte observation synchronously before returning those bytes to Hyper. If that observation wins before finalization, errors MUST report `ResponseStarted`; if finalization wins first, it MUST cancel the Hyper request/connection driver and invalidate the connection before snapshotting delivery state, so no later byte can be consumed for that exchange. A zero-byte read MUST NOT advance delivery state. Errors MUST retain only safe cause categories and delivery evidence; endpoint/path/query values, raw request/response data, credential paths, and underlying untrusted error text MUST NOT appear in errors or logs. [ADR-0014]
- **FR-013**: One client MUST use one configured endpoint and serialize exchange calls. HTTPS MAY reuse a healthy connection; raw TLS MUST close its connection after exactly one response frame so unsolicited or surplus stream bytes can never become the next exchange's response. A network, TLS, HTTP-framing, or raw-TTLV-framing failure MUST invalidate the current connection. A later, distinct operation MAY reconnect; the failed KMIP request MUST NEVER be retried automatically, and server-supplied alternative endpoints MUST NOT be selected automatically. The resolver MUST use at most one retry after the initial query (two attempts total), two concurrent nameserver requests per query, and at most 32 active DNS requests per multiplexed upstream connection; it MUST use a per-client response cache of at most 128 entries and return at most 16 A/AAAA candidates per lookup. The 32-request bound is per upstream connection, not an aggregate per-client cap. Candidates are connect alternatives for the same configured hostname and MAY be tried only before request-dispatch commit and within the connect deadline; no KMIP request bytes may be handed to TLS before a candidate succeeds. [ADR-0005; transport-security architecture]
- **FR-014**: The adapter MUST enforce the `max_response_bytes` argument during response reads before buffer growth and MUST return no response larger than that value. The typed client MUST pass exactly `CodecLimits::max_message_bytes()` to exchange and MUST check the returned length before decoding. KMIPKit's typed decode MUST retain the existing configurable defaults of 16 MiB per message, depth 64, and 100,000 elements and validate them before nested allocations. [ADR-0014; transport-security architecture]
- **FR-015**: Every KMIPKit-owned request copy MUST be bounded to the configured request limit, not retained past synchronous exchange, not logged, and zeroized before release. Every successful response MUST be returned in `TransportResponse`; each initialized partial response allocation MUST be zeroized on errors. Response accumulation MUST not reallocate after storing response bytes unless every prior/temporary allocation is zeroized before release. Bytes retained by Hyper, rustls, the OS, or another dependency are external copies and MUST be covered by the documented third-party-copy limitation. [ADR-0014]
- **FR-016**: The raw-TLS and HTTPS adapters MUST be available to the synchronous typed client and to direct Rust users only through documented, bounded APIs. The top-level `kmipkit` facade MUST NOT re-export the low-level `Transport` trait or raw-byte exchange entry point.
- **FR-017**: Each production adapter MUST reject a caller request larger than its configured `max_request_bytes` before DNS, connection, or request transmission. The default MUST be 16 MiB; callers MAY configure a different positive limit. The typed client MUST reject encoded requests larger than the effective `CodecLimits::max_message_bytes()` and MUST NOT submit an oversized request to a transport.
- **FR-018**: The production client MUST retain the immutable KMIPKIT-0012 `ClientConfiguration`
  separately from the validated transport configuration; the transport configuration MUST NOT
  contain or replace that client's extension registry. Before constructing an outgoing KMIP
  `RequestMessage`, encoding it, or invoking the adapter, `Client::execute` MUST compare the private
  registry provenance of every attached `ClientRequestMessageExtension` with the registry owned by the
  retained `ClientConfiguration`. A mismatch MUST return sanitized `InvalidInput` with
  `DeliveryState::NotSent`, without constructing or encoding the outgoing KMIP request and without
  invoking the adapter. An extension validated by the same retained configuration MUST continue
  through the existing typed request, encoding, and exchange path unchanged.
  [KMIPKIT-0012-FR-001; ADR-0013; KMIPKIT-0012 verification record]

### Normative Requirement Traceability

| Requirement ID | Normative source | Scope and verification |
|---|---|---|
| KMIPKIT-REQ-SPEC-10.4-001-001 | OASIS KMIP Specification v2.1 §10.4 | TLS integration evidence demonstrates confidentiality for exchanged KMIP bytes. |
| KMIPKIT-REQ-SPEC-10.4-001-002 | OASIS KMIP Specification v2.1 §10.4 | TLS integrity tests reject tampered/invalid TLS records and the peer cannot alter accepted response bytes. |
| KMIPKIT-REQ-SPEC-10.4-001-003 | OASIS KMIP Specification v2.1 §10.4 | Server certificate chain, validity, hostname/SAN, and selected-trust tests demonstrate authenticated peer identity; client mTLS identity tests verify KMIPKit product policy separately. |
| KMIPKIT-REQ-PROF-5.3.1-001 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 1 | HTTPS adapter uses HTTP/1.1 over TLS; TLS policy tests cover TLS 1.3. This selected overlap does not establish full profile support. |
| KMIPKIT-REQ-PROF-5.3.1-002 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 2 | HTTPS request-capture test asserts POST. |
| KMIPKIT-REQ-PROF-5.3.1-003 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 3 (`SHOULD`) | Default-target test asserts `/kmip`; deviation is not permitted absent approved record. |
| KMIPKIT-REQ-PROF-5.3.1-004 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 4 | Public HTTPS configuration test asserts caller-selected target URI. |
| KMIPKIT-REQ-PROF-5.3.1-005 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 5 | HTTPS request-capture test asserts TTLV media type. |
| KMIPKIT-REQ-PROF-5.3.1-008 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 8 | HTTPS request-capture test asserts exact Content-Length. |
| KMIPKIT-REQ-PROF-5.3.1-009 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 9 | HTTPS request-capture test asserts `Cache-Control: no-cache`. |
| KMIPKIT-REQ-PROF-5.3.1-010 | OASIS KMIP Profiles v2.1 OS §5.3.1 item 10 | HTTPS request-capture test compares binary body byte-for-byte. |

The exact upstream source is `specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html`; the catalog entries are generated from the pinned source and MUST not be edited by hand. Raw TTLV tag, type, length, value, and padding semantics refer to `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` §§10.1.1–10.1.5; TTLV as the mandatory message protocol is §10.2; channel confidentiality, integrity, and authenticity are §10.4. The catalog IDs for §10.4 are `KMIPKIT-REQ-SPEC-10.4-001-001`, `-002`, and `-003`. The HTTPS Mandatory Test Case `MSGENC-HTTPS-M-1-21` (§5.3.3.1; catalog ID `KMIPKIT-TEST-PROF-5-3-3-1`) exercises Query and oversized response behavior; it remains blocked from end-to-end execution until the Query operation is implemented. Transport-level size/HTTP behavior is covered here, and profile evidence remains assigned to KMIPKIT-0010. Profile-conditional Basic and HTTPS Authentication Suite requirements and their cross-references remain out of scope while no formal profile is claimed; the client SHALL NOT be assigned server-only duties from §§3.1.1–3.1.4 or §5.3.2. KMIPKit's mTLS requirement is an explicit product boundary, not a claim that OASIS assigns a client-side mutual-TLS MUST.

## Edge Cases

- A caller supplies an HTTPS URL containing user information, a fragment, an unsupported scheme, an invalid port, or malformed target URI.
- A caller supplies empty, malformed, encrypted, multiple, mismatched, or unsupported PEM/DER key material; a file is absent, unreadable, or changes after client construction.
- The selected server trust store is empty or malformed; a CRL is malformed, stale according to its validity interval, or does not cover the server certificate issuer.
- DNS resolves to an IP endpoint while certificate verification uses the configured TLS name; a certificate has no matching DNS/IP SAN.
- TLS negotiation attempts TLS 1.2, has no common TLS 1.3 suite, fails mTLS, or fails certificate revocation checks.
- The raw-TLS peer sends only part of the 8-byte header, a wrong root type/tag, an unaligned/overflowing length, a message larger than the cap, a partial body, or more bytes after one complete frame.
- The HTTPS peer redirects, returns non-200, duplicates/conflicts critical headers, omits Content-Length, returns a non-TTLV media type, sets any Content-Encoding, truncates or overstates its body, or sends a body over the per-exchange cap.
- Connect, write, read, or total deadline expires before transmission, during partial transmission, after full transmission, or after response bytes begin.
- An HTTPS connection is reused successfully, becomes stale while idle, or is invalidated following a protocol/network error; a later exchange reconnects without replaying the failed request. Raw TLS closes after each response frame.
- TLS/file configuration, transport errors, tracing, panic containment, or diagnostics encounter secret-bearing values and must remain redacted.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 8 applicable HTTPS Client §5.3.1 inventory requirements listed above map to a public configuration/request behavior and at least one executable test; the traceability audit reports 8/8 mapped and zero unsupported full-profile claims.
- **SC-002**: Ephemeral-PKI tests prove TLS 1.3 mutual authentication succeeds and TLS 1.2, unknown CA, invalid validity, hostname/SAN mismatch, missing client certificate, expired/non-applicable/revoking configured CRLs, and platform-root loading errors fail closed on all CI operating systems; an isolated-process test proves the documented `SSL_CERT_FILE` precedence.
- **SC-003**: Raw-TLS framing tests cover header truncation, invalid root header, arithmetic overflow, exact boundary size, one byte over limit, partial body, and exact one-frame reads without allocating above the configured cap.
- **SC-004**: HTTPS capture tests prove the exact method, target URI, required headers, and unchanged body; redirect, proxy, compression, non-200, malformed header/body, unsolicited response on a reused connection, and response-cap cases each fail without retry or cross-exchange response attribution.
- **SC-005**: Tests demonstrate the 10s/30s/30s/60s defaults, per-client and per-exchange overrides for typed and direct production-adapter calls, unbounded configuration, queue-time deadline expiry, no post-`NotSent` dispatch, `PossiblySent` after dispatch commit even when only HTTP headers are emitted, partial-body `PossiblySent`, monotonic first-response-byte `ResponseStarted`, and timeout/response-observation race ordering.
- **SC-006**: Tests with secret sentinels find no private-key bytes, request bytes, response bytes, credentials, or underlying untrusted error text in `Debug`, `Display`, logs, or public error chains; tests inspect initialized KMIPKit-owned allocations on success and every error path.
- **SC-007**: The public typed-client integration suite executes every currently supported typed
  request through both production adapters, verifies exact bounded transport calls and typed
  decoded results, and proves no request is automatically retried. It rejects an extension carrying
  another client's registry provenance as sanitized `InvalidInput`/`NotSent` before request
  construction/encoding/exchange and proves a same-client extension retains its valid wire
  representation.
- **SC-008**: Formatting, Clippy, unit, negative, integration, documentation, dependency-policy, and platform CI checks required by the repository pass; transport/changed-code/workspace line coverage reaches at least 85%/95%/90% respectively, and requirement traceability reaches 100%.

## Clarification Record

The following details were checked against the accepted project architecture, threat model, pinned KMIP 2.1 sources, and official dependency documentation. They are recorded here so implementation does not invent transport behavior.

| Topic | Resolution | Basis / implementation gate |
|---|---|---|
| Transport formats and direction | KMIP 2.1 TTLV only; client-initiated requests; synchronous public Rust API. No server-initiated behavior or JSON/XML is added here. | Product boundary; server-initiated operations remain in 1.1. |
| TLS and trust | TLS 1.3 only, rustls with an explicitly selected AWS-LC provider, mTLS, explicit trust, mandatory chain/date/hostname checks, no key logging or 0-RTT. | Accepted ADR-0005 and transport-security architecture. |
| Session resumption trust | A resumed session inherits the peer identity and trust/CRL decision made during its full handshake; certificate and CRL checks are not repeated. The per-configuration cache expires tickets after one hour of local monotonic time, and rebuilding the client applies changed trust inputs with a fresh cache. | Accepted ADR-0015; implement and test an expiring per-client session store and the documented trust snapshot. |
| Platform trust semantics | Only when explicitly selected, load native root certificates with `rustls-native-certs` and validate with rustls/WebPKI; `SSL_CERT_FILE`, when set, takes precedence and supplies that mode's root bundle. Do not claim platform distrust or revocation integration. Fail closed on loader errors. | `rustls-native-certs` 0.8.4; isolated-process override test, dependency review, and cross-platform behavior tests gate implementation. |
| HTTPS profile | Implement only selected HTTPS Client §5.3.1 transport clauses for HTTP/1.1 + TTLV. This feature does not claim an HTTPS Client KMIP Profile or a Baseline Client profile. | Pinned Profiles v2.1 and KMIPKIT-0010; the complete mandatory test sets are not in this scope. |
| Direct low-level exchange | Direct Rust callers supply one request frame and own its input bytes. The adapter preserves those bytes but does not encode, normalize, or KMIP-schema-validate them. The typed client continues to accept only closed typed requests. | Accepted ADR-0014. |
| Private-key source | Explicit PEM or DER; unencrypted PKCS#8, PKCS#1, or SEC1 only when supported by the selected provider. Encrypted keys and passphrase callbacks are rejected. Files are read once at construction; normal path/symlink resolution applies. | Threat model §2 and transport-security architecture. |
| Timeout meaning | Connect spans DNS/TCP/TLS setup; read/write are inactivity limits for one blocked I/O operation reset on progress; total is one monotonic deadline from public exchange entry, including time queued for the worker. Per-request overrides are available through typed `execute_with_options` variants and direct production-adapter `exchange_with_options`. | Accepted ADR-0015; dependency, thread lifecycle, and implementation tests remain required. |
| Timeout cancellation and delivery | Shared exchange control arbitrates cancellation against handing the request to the HTTP/TLS writer; `NotSent` guarantees no later send. Once dispatch commits, including HTTPS headers emitted without a TTLV body, errors are conservatively `PossiblySent`. The design does not infer a header/body boundary below Hyper. | Accepted ADR-0015; tests exercise the real Hyper driver and deterministic dispatch/response races. |
| HTTPS implementation and sync bridge | Hyper owns HTTP/1 parsing; a private per-client Tokio worker owns resolver, TLS state, and one serialized connection. The facade stays synchronous. | Accepted ADR-0015; dependency and thread-lifecycle checks remain implementation gates. |
| HTTPS request/parser bounds | Request target is origin-form path/query only; Hyper allows at most 64 response headers and a 64 KiB input buffer, separately from the KMIP response-body cap. | Explicit KMIPKit acceptance policy; test exact and over-boundary cases. |
| HTTPS Host authority | Emit exactly one `Host` header serialized from the configured endpoint authority, including any explicitly configured port; request-target and TLS-name overrides cannot alter it. | HTTP/1.1 request contract; capture a non-default endpoint port with a distinct TLS verification name. |
| DNS behavior | Use Hickory's asynchronous resolver initialized from system DNS configuration, with one retry/two total attempts per query, at most two concurrent nameserver requests per query, at most 32 active requests on each multiplexed upstream connection (not a per-client aggregate), 128 cached responses per client, and 16 A/AAAA candidates per lookup. Validate system configuration behavior on Linux, Windows, and macOS. | Official Hickory 0.26.3 docs/release notes; exact dependency review and platform tests required before merge. |
| Request-size boundary | Direct raw transport requests default to a 16 MiB configured cap; typed requests remain bounded by `CodecLimits::max_message_bytes()`. | Required to bound request ownership/copies before DNS or connection activity. |
| Raw connection lifecycle | Close raw-TLS after exactly one response frame; HTTPS may reuse a healthy HTTP/1 connection. This narrows the accepted general reusable-connection statement, now recorded in both canonical architecture documents. | Prevents a surplus raw frame from becoming a later exchange's response; accepted ADR-0015 and completed T001. |
| HTTP response framing | Require one unambiguous `Content-Length`, reject `Transfer-Encoding`/content encoding, and accept only the exact body framing Hyper parses. Treat parser errors/truncation as failure and invalidate the connection. If Hyper observes an unsolicited response on a reused connection, invalidate it and never associate that response with a later operation. | HTTP/1 parser contract, with a real reused-connection test. No claim that unframed bytes after a complete response body form another KMIP response. |

## Assumptions

- The governing boundaries remain those in the constitution and accepted ADRs: KMIP 2.1, TTLV only, client initiated operations, synchronous typed API, TLS 1.3 only, mutual TLS, rustls with aws-lc-rs, and Rust/C/Java/Python parity later in the 1.0 program.
- Existing defaults and ownership constraints in `docs/architecture/transport-security.md`, `docs/adr/0005-transport-and-tls.md`, and `docs/adr/0014-public-transport-exchange-contract.md` are binding inputs; any conflict must be recorded and resolved through a reviewed ADR before implementation.
- The caller chooses endpoint, trust roots, identity, CRLs, and target URI. KMIPKit does not silently discover identities, weaken verification, or choose another endpoint.
- Encrypted private keys and passphrases are excluded to avoid long-lived passphrase callbacks or implicit decryption policy. A later need requires a separate security-reviewed specification.
- Ordinary OS path resolution follows configured symlinks; the file source is read exactly once when the client is constructed. The caller is responsible for protecting the named files and their parent directories.
- A successful HTTPS transport exchange validates HTTP framing and returns bounded bytes; the typed client then performs TTLV and KMIP-message validation. Transport success alone does not assert a well-formed KMIP response or server operation support.
- OASIS profile clauses are tracked as normative requirements, but full HTTPS Client Profile support requires the complete baseline/profile requirements and official tests and is not claimed by this feature.
