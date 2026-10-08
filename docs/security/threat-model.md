# KMIPKit threat model

Status: implementation snapshot and design baseline
Scope: KMIPKit 1.0 architecture, including KMIPKIT-0005 TTLV, KMIPKIT-0007 client execution, and KMIPKIT-0013 production transport
Last reviewed: 2026-10-08

This document models risks for the KMIPKit 1.0 architecture and distinguishes
current executable controls from planned components. The repository includes
the kmipkit-ttlv in-memory value model and public bounded decoder, typed KMIP
message execution, the public low-level exchange contract, production raw TLS
and HTTPS adapters, and a synchronous typed client constructor that owns
validated production adapters. The typed client currently exposes only its
implemented request set; this does not establish full KMIP 2.1 operation or
profile coverage. C ABI, Java, Python, additional operation families, and
publication remain design scope unless their source and tests establish
otherwise. Scenarios below remain hypotheses and security requirements unless
a control is explicitly tied to executable evidence. The model must be
revised as each executable boundary is introduced.
Independent architecture mapping and a source-level security diff review were
completed for KMIPKIT-0005 on 2026-10-05. A qualified human security review
remains a release requirement before 1.0.

## 1. Overview

KMIPKit is designed as a client library for Rust, C, Java, and Python. Its
target architecture converts typed KMIP 2.1 operations into TTLV and sends
them to a configured server through raw TLS or HTTPS. One Rust core owns the
wire model and protocol behavior; the C ABI and language adapters are planned
to translate types and lifecycle conventions without reimplementing KMIP
([architecture](../architecture/overview.md#system-shape)). The current
implemented surface is distinguished from that target in the evidence table
below.

```mermaid
flowchart LR
    subgraph Caller[Caller trust zone]
        App[Application]
        Config[Endpoint, trust roots, client identity,
        KMIP credentials and limits]
    end
    subgraph Native[KMIPKit process boundary]
        Binding[Rust / C / JNI / CFFI API]
        Core[Protocol model and client]
        Parser[Bounded TTLV codec]
        TLS[TLS 1.3 transport]
        Ext[Extension definitions]
    end
    subgraph Remote[Remote trust zone]
        Server[KMIP server]
    end
    subgraph Supply[Build and release trust zone]
        Source[Source and generated artifacts]
        CI[Protected CI and signing]
        Package[Native and language packages]
    end

    Config --> Binding
    App --> Binding --> Core --> Parser --> TLS <--> Server
    Ext --> Core
    Source --> CI --> Package --> App
```

### Components and evidence

The generic TTLV value model, bounded public decoder, typed client execution,
low-level transport contract, and production raw TLS and HTTPS adapters are
implemented. KMIPKIT-0013 connects the currently supported typed operations to
those adapters through a private worker. Other product component rows
describe target design unless marked implemented; CI validation is implemented
while package publication remains planned.
| Component | Responsibility | Security relevance | Evidence |
|---|---|---|---|
| Generic TTLV value model (implemented) | Construct and inspect typed in-memory values; preserve ordered Structures; check tag allocation and depth | Payload redaction and zeroization, bounded nesting, tag allocation; this model does not establish wire validity | `crates/kmipkit-ttlv/src/lib.rs:1-11`; `docs/architecture/public-api.md:34-49` |
| Public TTLV decoder (implemented) | Decode one bounded untrusted item into the public generic tree | Memory and CPU bounds, malformed-input rejection, unknown value preservation, no raw-byte retention | `crates/kmipkit-ttlv/src/codec/decoder.rs:29-162`; `crates/kmipkit-ttlv/src/codec/mod.rs:18-24,181-209` |
| Private client outbound writer (implemented) | Encodes only the closed typed request set through `Client::execute`; no general-purpose or public encoder | Execute-owned permit, one writer callsite, request owner through synchronous exchange, zeroization on drop, redacted diagnostics; request payload coverage remains limited to implemented operations | `crates/kmipkit-client/src/wire_encoder.rs`; `crates/kmipkit-client/src/execute.rs`; ADR-0012; specs/007-client-execution |
| Typed protocol client (implemented for the current request set) | Validates requests, correlates responses, and invokes one production adapter exchange | Extension-registry provenance, typed response validation, request-delivery evidence, response-size check before decoding, no retry | `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-client/tests/production_client.rs`; specs/013-production-transport |
| Raw TLS and HTTPS transports (implemented) | Authenticate peers and carry bounded messages | Server/client identity, confidentiality, framing, deadlines, delivery state, response cleanup; HTTPS may reuse a healthy connection and raw TLS closes after one frame | `crates/kmipkit-transport/src/raw_tls.rs`; `crates/kmipkit-transport/src/https.rs`; `crates/kmipkit-transport/src/timeout.rs`; `docs/architecture/transport-security.md` |
| Private client worker and system resolver (implemented) | Keep the public API synchronous while owning async I/O and system name resolution | Bounded queue, deadline/cancellation ordering, shared resolver admission, late-result isolation; OS resolver behavior remains outside KMIPKit's numeric guarantees | `crates/kmipkit-transport/src/worker.rs`; `crates/kmipkit-transport/src/resolver.rs`; ADR-0015; ADR-0016 |
| C ABI | Expose native functionality to foreign runtimes | Pointer validity, ownership, panic containment, stable layouts | `docs/architecture/ffi-and-bindings.md:3-33` |
| Java and Python adapters | Offer idiomatic APIs and load native code | Native package integrity, secret copies, lifecycle and concurrency | `docs/architecture/ffi-and-bindings.md:46-81` |
| Extension registry | Interpret optional vendor data | Untrusted schemas, semantic ambiguity, denial of service | `docs/architecture/extensions.md:18-59` |
| CI validation and release publication | Run repository checks; build and publish packages | Generated-source integrity, dependency and artifact substitution, signing authority | CI checks: `.github/workflows/ci.yml`; publication design: `docs/development/git-and-releases.md:44-64` |

### Effective resources and capabilities

| Deployment or workflow | Resource or capability | Configuration and precedence | Safe effective value or location | Readers, writers, or recipients | Enforcing control | Evidence or unknowns |
|---|---|---|---|---|---|---|
| KMIP connection | Server endpoint and HTTPS path | Immutable validated transport configuration; request cannot silently replace it | One explicit endpoint; `/kmip` is only the default HTTPS path | Caller, typed client, transport, configured server | URL validation, HTTPS-only policy, no redirects, no automatic alternative endpoint; one active and at most one queued exchange | Production typed construction is available for the current request set. Evidence: `crates/kmipkit-client/tests/production_client.rs`; `crates/kmipkit-transport/src/config.rs`; `crates/kmipkit-transport/src/worker.rs` |
| TLS authentication | Trust roots, server name, client certificate, private key, CRLs, resumption tickets | Caller explicitly selects caller-provided roots or platform trust; configuration owns a per-client ticket cache | TLS 1.3, mandatory chain/validity/name checks on a full handshake, mutual TLS; a resumed session inherits the full-handshake trust/CRL decision for at most one hour locally | Caller and rustls transport | No arbitrary-certificate switch, 0-RTT, key logging, cross-client ticket sharing, or online OCSP/CRL lookup. Platform trust uses `rustls-native-certs`; `SSL_CERT_FILE`, when set, overrides roots for that mode and is controlled by the caller's process environment | `crates/kmipkit-transport/src/config.rs`; `crates/kmipkit-transport/src/tls.rs`; `crates/kmipkit-transport/tests/tls_policy.rs`; ADR-0005 |
| Name resolution and worker admission | Configured endpoint host/port; DNS activity; bounded work slots | System `ToSocketAddrs`; one shared 32-permit governor per loaded KMIPKit library instance; fail-fast when no permit is available | At most one native resolver call per connection-establishment attempt; KMIPKit retains/tries at most 16 returned addresses in OS order | Caller thread, private worker, OS resolver, configured endpoint | Permit acquired before `spawn_blocking` and held until a started lookup returns; canceled late results cannot start a connection or dispatch KMIP bytes; OS packet retries/cache/routing/search/split-DNS remain OS-owned | `crates/kmipkit-transport/src/resolver.rs`; `crates/kmipkit-transport/tests/resolver.rs`; `crates/kmipkit-transport/tests/worker.rs`; ADR-0016 |
| HTTP parser and KMIP response body | Untrusted HTTP response headers and body | Hyper HTTP/1 parser limits are independent from caller KMIP codec limits | At most 64 response headers and 64 KiB parser input; typed response body is capped by `CodecLimits::max_message_bytes()` | Remote peer, Hyper, transport, bounded decoder | Status/header/content-length validation; reject unsupported encodings; invalidate malformed/reused connections; check body length before allocation and again before typed decode | `crates/kmipkit-transport/src/https.rs`; `crates/kmipkit-transport/tests/https.rs`; `crates/kmipkit-client/tests/unit/response_boundary_tests.rs` |
| Request and response memory | Encoded requests, partial network responses, and decoded typed values | KMIPKit owns bounded secret/request/response owners; dependencies and callers may make separate copies | Initialized bytes in current KMIPKit-owned buffers are zeroized before release; this is not a process-wide erasure guarantee | Caller, client, Hyper, rustls, OS, foreign runtimes | Typed client returns decoded results rather than raw response bytes; direct transport exposes a redacted `TransportResponse`; document copies in Hyper/rustls/OS/callers as outside the guarantee | `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-transport/src/secret.rs`; `crates/kmipkit-transport/src/response.rs`; `docs/architecture/transport-security.md` |
| Exchange deadlines and delivery | Connect, write, read, total deadline, cancellation, and worker startup | Total deadline starts at public exchange entry and is monotonic; finite phase values must be representable by `Instant` | Defaults are 10s connect, 30s write, 30s read, 60s total; explicit unbounded is distinct from zero-duration immediate expiry | Caller thread, private worker, adapter, remote peer | Lazy worker startup and queueing are bounded by total deadline; invalid unrepresentable finite durations return `InvalidInput`/`NotSent`; atomic dispatch/response/finalization gate preserves truthful delivery state | `crates/kmipkit-transport/src/timeout.rs`; `crates/kmipkit-transport/src/worker.rs`; `crates/kmipkit-transport/tests/timeout_delivery.rs`; `crates/kmipkit-transport/tests/worker.rs` |
| KMIP authentication | Message credentials, OTPs, and tickets | Credentials are modeled but are not added to the current typed operation request payloads | Current typed request paths do not send KMIP credentials; TLS client identity is a separate transport concern | Caller and a configured KMIP peer when a credential-bearing operation is implemented | Redaction and initialized-byte zeroization are implemented for typed execution; later secret-bearing operations require owner-through-transport tests | `crates/kmipkit-protocol`; `crates/kmipkit-client/src/execute.rs`; foreign-runtime copies and external transport copies remain outside current evidence |
| Native bindings | Rust-owned handles and result buffers | ABI version and structure size are checked at entry | Opaque handles; matching KMIPKit release functions own deallocation | C, JNI, CFFI callers | Fixed-width ABI, explicit lengths, panic containment, dedicated free functions | Designed in `docs/architecture/ffi-and-bindings.md:3-24`; implementation and sanitizer evidence pending |
| Native package loading | Platform binary bundled with Java or Python package | Bundled binary first; explicit administrator path may override | Package-adjacent binary or private atomic extraction directory | Language runtime and current user | Package hash verification and ABI check; no runtime download | Designed in `docs/architecture/ffi-and-bindings.md:73-81`; exact extraction permissions and anti-swap procedure remain to be specified |
| Vendor extensions | Data-only definitions in an immutable client registry | Explicit registration during client construction | Parsed, size-bounded in-memory schema | Caller, core validator, KMIP peer | Duplicate rejection, deterministic registration, core invariants cannot be disabled | Designed in `docs/architecture/extensions.md:18-59`; manifest format and complexity limits remain to be specified |
| Release publication | Package registry and signing authority | Protected tag and CI workflow only | Short-lived publishing identities, signed artifacts, hashes, SBOM and provenance | Maintainers, CI, package registries, users | Protected review, pinned actions, trusted publishing | Designed in `docs/development/git-and-releases.md:44-64`; concrete workflows do not exist yet |

## 2. Threat model, trust boundaries, and assumptions

### Protected assets

- Private and symmetric key material transported in KMIP objects.
- TLS client private keys and KMIP message credentials.
- Server authenticity and the binding between a response and its request.
- Integrity and confidentiality of KMIP messages in transit.
- Availability of the caller process when parsing hostile network input.
- Native memory safety and allocator ownership across the C ABI.
- Integrity of generated protocol definitions and distributed native binaries.
- Accurate conformance, security, FIPS, and interoperability claims.

### Actors and realistic capabilities

| Actor | Starting capabilities | Capabilities not assumed |
|---|---|---|
| Network attacker | Observe, delay, drop, replay, fragment, or replace traffic; present an untrusted certificate | A trusted CA, the configured client private key, or control of the caller process |
| Malicious or compromised KMIP server | Send arbitrary bytes after a successful authenticated connection; return inconsistent batches, extensions, pending states, and large messages | Direct native memory access or authority to change local configuration |
| Untrusted application input | Influence KMIP operation fields or generic TTLV passed by an application | Permission to bypass the application's own authorization policy |
| Faulty or hostile foreign caller | Pass invalid pointers, lengths, handles, call order, or concurrent calls into the C ABI | A guarantee that arbitrary invalid addresses can be safely dereferenced; C caller memory safety remains a shared boundary |
| Malicious extension author | Supply complex or conflicting declarative definitions when the application registers them | Code execution through a declarative manifest or access to TLS internals |
| Supply-chain attacker | Publish a dependency or package with a confusing name, tamper with an unprotected workflow, or substitute a native artifact | Protected repository administration or release credentials by default |
| Local same-user attacker | Race or replace files in locations writable by the same OS identity | Privilege isolation from the application when both run as the same user |

### Intended trust boundaries and invariants

These invariants apply to both implemented and planned components. The
Components and evidence table distinguishes executable controls from future
work.

1. **Application to public API.** All sizes, enum values, identifiers, paths,
   endpoints, and generic TTLV are validated before use. High-level APIs do not
   silently choose cryptographic policy.
2. **Foreign runtime to C ABI.** Every entry validates handles, pointer/length
   pairs, structure versions, and lifecycle state before touching memory. No
   panic crosses the boundary and no allocator ownership is ambiguous
   (`docs/architecture/ffi-and-bindings.md:3-24`).
3. **Network to decoder.** Framing and resource limits are checked before
   allocation with overflow-safe arithmetic. HTTP parser limits (64 response
   headers and 64 KiB input) are separate from the KMIP body limit. Malformed
   frames invalidate the connection (`docs/architecture/transport-security.md:5-12,67-78`).
4. **Transport to remote identity.** The server certificate chain, validity,
   and name are mandatory; mTLS credentials identify the client. Redirects,
   proxies, 0-RTT, and automatic endpoint selection cannot change the peer
   (`docs/architecture/transport-security.md:14-43,45-52`). A resumed TLS
   session inherits the trust and caller-CRL decision from its verified full
   handshake; rebuilding the client is required to apply changed trust input.
5. **Response to request.** Protocol version, correlation, batch count,
   operation, and pending state are validated before delivering a result
   (`docs/architecture/overview.md:119-122`). Dispatch, first response-byte
   observation, and timeout finalization share a delivery-state gate. `NotSent`
   guarantees a canceled or expired request cannot be dispatched later;
   failures after dispatch are `PossiblySent` until a response byte is observed.
6. **Secret to diagnostics.** Keys, credentials, OTPs, tickets, private keys,
   raw bodies, and secret parameters never enter logs, errors, snapshots, or
   fixtures (`docs/architecture/transport-security.md:80-92`). KMIPKit-owned
   initialized request, partial-response, and response allocations are
   zeroized on release. Hyper, rustls, the OS, the caller, and foreign runtimes
   may retain external copies outside that guarantee.
7. **Name resolution to connection.** KMIPKit bounds its own admission and
   retained candidates, but the operating system owns DNS retries, routing,
   cache behavior, hosts/search rules, and split-DNS/VPN policy. A started
   system lookup may continue after caller timeout and retain its governor
   permit; its late result cannot start TCP/TLS or dispatch KMIP bytes.
8. **Deadline to dispatch.** The total monotonic deadline begins at public
   exchange entry and includes lazy worker startup, queueing, resolution,
   connection, TLS, request write, and response read. A finite timeout that
   cannot be represented by `Instant` is rejected as `InvalidInput`/`NotSent`,
   never reinterpreted as unbounded.
9. **Connection to next request.** HTTPS may reuse only a healthy HTTP/1
   connection; raw TLS closes after exactly one TTLV response frame. Framing or
   I/O failures invalidate the connection. A later distinct call may reconnect,
   but the failed KMIP request is never retried automatically.
10. **Extension to core.** Extension interpretation cannot disable framing,
   limits, criticality handling, redaction, or transport policy
   (`docs/architecture/extensions.md:52-59`).
11. **Source to release.** Generated artifacts are reproducible from reviewed
   inputs, third-party sources and licenses are controlled, and publication
   occurs only from protected CI (`docs/development/git-and-releases.md:44-64`).

### Assumptions and caller obligations

- The application authorizes who may request KMIP operations. KMIPKit does not
  implement business authorization above KMIP.
- The caller protects certificate, private-key, credential, and output files
  with appropriate operating-system permissions.
- A process with the same privileges as the application can generally inspect
  its memory; zeroization reduces residual copies but does not create process
  isolation.
- Java and Python may create runtime-managed secret copies that native
  zeroization cannot guarantee to erase.
- The configured CA and server name are trustworthy. An explicitly selected
  platform trust store accepts its normal trust policy.
- When platform trust is selected, the caller is responsible for the process
  environment because `SSL_CERT_FILE`, when set, overrides the roots loaded by
  `rustls-native-certs`. KMIPKit does not import all platform-specific distrust
  or revocation decisions; caller-provided CRLs are the only configured
  revocation evidence.
- DNS resolution follows the host operating system. A canceled call can return
  while an already-started resolver operation and its OS-managed DNS activity
  continue in the background; KMIPKit holds the shared permit until that native
  call exits.
- TLS resumption intentionally retains the full-handshake peer and trust/CRL
  decision for a ticket's local lifetime, up to one hour. It does not recheck
  certificate validity or reevaluate CRLs during resumption.
- No FIPS, formal certification, or independent audit claim exists without
  release-specific evidence (`SECURITY.md:49-53`).
- Dynamic executable extension plugins, local cryptographic algorithms,
  server-initiated operations, and automatic failover are outside the 1.0
  runtime boundary.

### Open design questions for later increments and release

- Exact declarative extension manifest grammar, byte/depth/count limits, and
  validation complexity budget.
- Java native extraction directory permissions, file locking, atomic publish,
  cleanup, and resistance to same-user replacement.
- The exact status-code taxonomy for invalid foreign pointers versus invalid
  handles without attempting unsafe pointer recovery.
- Release signing technology, protected-environment rules, artifact retention,
  and key-compromise recovery.

## 3. Attack surface, mitigations, and attacker stories

Each row describes a possible attack path. A listed control is treated as
implemented only where its evidence points to current code and executable
tests.

| Priority | Scenario and capability gain | Prerequisites | Impact | Existing designed controls | Required mitigation and verification | Evidence |
|---|---|---|---|---|---|---|
| High | A malicious server declares extreme or overflowing TTLV lengths to obtain excessive allocation, CPU exhaustion, or memory corruption | Authenticated or network-reachable server can send a response | Caller process crash, denial of service, or potentially native code execution if unsafe parsing is introduced | Header-first framing; HTTP parser capped at 64 headers/64 KiB; 16 MiB/depth 64/100,000 element defaults; overflow-safe checks; safe Rust codec | Keep codec safe Rust; limit before allocation; property tests, malformed vectors, fuzzing, Miri and sanitizers at native boundaries | `crates/kmipkit-transport/tests/https.rs`; `crates/kmipkit-client/tests/unit/response_boundary_tests.rs`; `docs/architecture/transport-security.md:5-12,67-78` |
| High | A substituted CA, disabled name check, redirect, proxy, or unexpected platform trust bundle sends secrets to an attacker-controlled server | Caller selects unsafe trust input or the process environment changes `SSL_CERT_FILE` | Disclosure of keys and credentials; unauthorized KMIP operations | No insecure switch; mandatory chain, validity, and hostname checks; no redirects/proxies; explicit platform trust; environment override documented | Make unsafe states unrepresentable; test unknown CA, mismatch, redirects, proxy variables, and isolated `SSL_CERT_FILE` override; redact configuration diagnostics | `crates/kmipkit-transport/tests/tls_policy.rs`; `crates/kmipkit-transport/tests/https.rs`; ADR-0005 |
| High | Invalid FFI lengths, stale handles, double free, or panic corrupts memory or unwinds into Java/Python/C | Foreign caller invokes ABI incorrectly or races lifecycle calls | Process compromise or crash | Opaque handles, explicit lengths, Rust-owned frees, panic containment, isolated unsafe crate | Define handle validation and concurrency state machine; compile C consumer; negative ABI tests; sanitizers; review every unsafe block | `docs/architecture/ffi-and-bindings.md:3-24`; `docs/development/testing.md:48-55` |
| High | A forged or mismatched KMIP response is returned to the wrong operation or batch item | Compromised server, stale connection bytes, or client correlation defect | Application uses the wrong key/object or misreports operation success | Version, correlation, batch, and operation checks; response validation before returning typed results; invalid connections discarded | Typed state-machine tests for duplicates, reordering, missing items, stale responses, pending operations, and response/timeout races | `crates/kmipkit-client/tests/production_client.rs`; `docs/architecture/overview.md:88-94,119-122`; `crates/kmipkit-transport/tests/timeout_delivery.rs` |
| High | A compromised build action, dependency, registry account, or generated catalog inserts malicious native code | Weak CI protections or publishing credentials | All downstream applications execute attacker code | Pinned actions, protected tags, CI-only publication, hashes, signatures, SBOM, provenance | Least-privilege jobs; trusted publishing; review generated diffs; dependency policy; reproducible release check; incident procedure | `docs/development/git-and-releases.md:44-64` |
| Medium | Request timeout after partial delivery causes an application to repeat a non-idempotent operation | Network interruption and caller retries without delivery context | Duplicate keys, state changes, or destructive operations | No automatic retry; atomic delivery evidence distinguishes not sent, possibly sent, and response started; failures invalidate the connection | Preserve delivery state through every binding; document reconciliation; test partial writes, queue expiry, dispatch races, response-byte races, and timeouts | `crates/kmipkit-transport/tests/timeout_delivery.rs`; `crates/kmipkit-transport/tests/worker.rs`; `docs/architecture/transport-security.md:45-65` |
| Medium | A very large finite timeout is misinterpreted as unbounded, or lazy worker startup escapes the caller's total deadline | A timeout value exceeds the platform's representable monotonic clock range, or worker startup/queueing is delayed | A call unexpectedly outlives its configured bound or reports an unsafe delivery state | Total deadline begins at public exchange entry and covers lazy startup and queueing; an unrepresentable finite phase is rejected as `InvalidInput`/`NotSent`; zero duration remains an immediate deadline | Test `Instant::checked_add` failure, worker readiness after expiry, and proof that `NotSent` work cannot dispatch later | `crates/kmipkit-transport/tests/timeout_delivery.rs`; `crates/kmipkit-transport/tests/worker.rs`; ADR-0015 |
| Medium | Repeated calls to attacker-controlled or slow hosts exhaust resolver threads or stall all client calls | Application creates many clients or abandons lookups while native system resolution is blocked | Connection denial of service or resource starvation | Shared 32-permit governor per loaded library instance; fail-fast admission; one blocking thread per worker; at most 16 retained candidates; late results cannot connect or dispatch | Keep permit held for the native call lifetime; do not claim to cancel OS DNS traffic; test saturation, cancellation, late-result isolation, and shutdown | `crates/kmipkit-transport/tests/resolver.rs`; `crates/kmipkit-transport/tests/worker.rs`; `crates/kmipkit-transport/tests/timeout_delivery.rs`; ADR-0016 |
| Medium | A resumed TLS session continues using an outdated peer/trust decision after CA or CRL inputs change | Caller changes trust configuration but continues using the same client/session cache | A peer accepted under a prior full handshake remains trusted for the ticket lifetime | Per-configuration cache with at most 16 tickets and one-hour local age; resumed sessions inherit the verified full-handshake identity and trust/CRL decision; rebuilding starts an empty cache | Document the trust snapshot; require rebuilding the client after trust changes; test resumption verifier behavior, ticket expiry, and cache isolation | `crates/kmipkit-transport/tests/tls_policy.rs`; `docs/architecture/transport-security.md`; ADR-0005 |
| Medium | Secrets appear in errors, debug output, tracing, exceptions, test snapshots, or language runtime representations | Error path or convenience formatting handles a secret-bearing value | Credential or key disclosure to logs and telemetry | Specialized secret types, redacted typed errors, zeroization of KMIPKit-owned initialized request/response buffers, logging disabled by default | Keep raw response bytes out of typed results; redaction tests through every language; review panic and allocator diagnostics; document that Hyper/rustls/OS/caller/runtime copies cannot be zeroized by KMIPKit | `crates/kmipkit-client/tests/production_client.rs`; `crates/kmipkit-transport/tests/secret_redaction.rs`; `docs/architecture/transport-security.md:80-92`; `AGENTS.md:154-172` |
| Medium | A complex declarative extension consumes excessive CPU/memory or changes validation outcomes by registration order | Application loads an untrusted manifest | Denial of service or inconsistent protocol behavior | Immutable per-client registry, duplicate rejection, deterministic registration, core limits remain active | Define manifest limits and deterministic grammar; reject recursion/cycles; property test registration order; fuzz manifest parser | `docs/architecture/extensions.md:18-59` |
| Medium | A Java native binary is replaced between verification and loading | Attacker can write to extraction path under the same user or exploit non-atomic extraction | Native code execution in the application | Bundled hash verification, private location, atomic extraction, `System.load` | Specify restrictive permissions, collision-resistant path, create-new semantics, file handle strategy, and post-open identity check | `docs/architecture/ffi-and-bindings.md:73-81` |
| Medium | Unbounded or attacker-controlled file paths read unexpected certificates or keys | Caller passes a path derived from untrusted input | Credential disclosure or unintended identity use | Explicit configuration and no automatic discovery | Document caller authorization; reject non-files where needed; specify symlink policy; avoid including paths or contents in errors | `docs/architecture/transport-security.md:23-39` |
| Low | Detailed errors or timings expose object existence, server policy, or endpoint metadata | Application exposes library diagnostics to an untrusted party | Information disclosure that assists later attacks | Redacted structured events and caller-controlled logging | Classify public versus diagnostic errors; sanitize endpoint/query content; document that applications control audience | `docs/architecture/transport-security.md:87-92` |

## 4. Severity calibration

Severity describes the capability gained under realistic prerequisites. Lack of
implementation evidence lowers confidence, not impact.

### Critical

- Pre-authentication memory corruption in ordinary parser use that permits
  native code execution across supported clients.
- Release-signing or protected-publishing compromise that distributes malicious
  official packages broadly.
- Systematic unauthenticated extraction of private or symmetric key material
  from correctly configured clients.

A crash caused only by a caller already executing arbitrary native code in its
own process is not Critical because it grants no meaningful new authority.

### High

- Remote code execution or cross-boundary memory corruption requiring a
  successfully authenticated malicious KMIP server.
- TLS verification bypass that sends credentials or key material to an
  attacker-selected endpoint under normal secure configuration.
- Response-confusion flaws that cause an application to use the wrong secret or
  accept a failed sensitive operation as successful.

If exploitation requires the operator to install an attacker CA intentionally,
severity is reduced because that actor already grants server trust.

### Medium

- Reliable remote denial of service within documented message limits.
- Secret leakage into logs that requires access to application diagnostics.
- Native library substitution requiring write access to an otherwise protected
  same-user extraction directory.
- Duplicate non-idempotent operations caused by misleading delivery-state
  reporting.

### Low

- Bounded metadata exposure with no key or credential content.
- Local misuse that produces a clear error and affects only the invoking
  process.
- Documentation or hardening gaps without a demonstrated path to a protected
  asset.

The following alone are outside the security boundary: a fully privileged
application deliberately exporting its own secrets, an administrator
explicitly trusting a malicious CA, or a same-process native caller already
capable of arbitrary memory access. They may still justify safe defaults and
documentation but do not represent a new privilege gain by KMIPKit.
