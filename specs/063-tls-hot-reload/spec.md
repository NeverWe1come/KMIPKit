# Feature Specification: File-backed TLS Hot-Reload and Resumption Invalidation

**Feature Branch**: `feature/KMIPKIT-0063-tls-hot-reload`
**Created**: 2026-11-06
**Status**: Approved (Feature KMIPKIT-0063)
**Input**: Approved architectural plan for automatic hot-reload of file-backed client identity, CA trust, and CRLs, coordinated with TLS 1.3 session resumption and HTTPS Keep-Alive invalidation in `kmipkit-transport`.

## User Scenarios & Testing

### User Story 1 - Automatic hot-reload of rotated client mTLS credentials (Priority: P1)

As a KMIP application operator running in an automated certificate rotation environment (e.g. cert-manager, HashiCorp Vault Agent, SPIFFE), when client certificate and private key files on disk are overwritten with a freshly renewed pair, `KMIPKit` automatically uses the new credentials on the next exchange without restarting the application or recreating the `Client`.

**Why this priority**: Production mTLS certificates in cloud-native environments rotate on short lifetimes (24 hours to 7 days). Applications must not lose connectivity or require process restarts when files update.

**Independent Test**: Configure a transport with PEM certificate and private key files on disk. Execute a successful exchange. Overwrite the files with a renewed certificate/key pair. Execute a subsequent exchange and verify that the server receives the new client certificate and that the exchange succeeds.

**Acceptance Scenarios**:
1. **Given** valid file-backed client certificate and key inputs, **When** the files are rotated on disk with a valid matching pair, **Then** the next exchange atomically loads the new pair, zeroizes old key buffers, and completes the handshake under the new identity.
2. **Given** a rotation in progress where the certificate file is updated before the key file (mismatched pair), **When** an exchange is attempted, **Then** `kmipkit-transport` fails closed as `RequestDeliveryState::NotSent` (0 bytes transmitted on the wire), does not corrupt in-memory state, and redacts all error text.
3. **Given** a temporary credential file deleted after client construction, **When** subsequent exchanges occur, **Then** `kmipkit-transport` preserves the valid in-memory material (*Last-Known-Good*) without error.

### User Story 2 - Automatic hot-reload of updated CRLs and server revocation (Priority: P1)

As a security auditor, when an updated CRL file is written to disk containing the serial number of a revoked KMIP server certificate (or renewing an expiring CRL), `KMIPKit` immediately honors the new revocation list and terminates existing Keep-Alive connections.

**Why this priority**: CRLs have strict `nextUpdate` expiration enforced by `kmipkit-transport`'s `.enforce_revocation_expiration()`, and compromised servers must be blocked immediately upon CRL publication.

**Independent Test**: Configure a client with a CRL file where the server is initially valid. Execute an exchange (establishing an active HTTP/1.1 Keep-Alive connection or TLS 1.3 resumption ticket). Overwrite the CRL file with one that revokes the server. Execute a second exchange and assert that the connection fails closed as `NotSent` and any existing Keep-Alive connection is discarded.

**Acceptance Scenarios**:
1. **Given** an updated CRL on disk revoking the server certificate, **When** the next exchange runs, **Then** `kmipkit-transport` purges existing TLS 1.3 resumption tickets, invalidates active Keep-Alive connections, rebuilds the WebPKI verifier, and rejects the server handshake with `NotSent`.
2. **Given** an active CRL approaching its `nextUpdate`, **When** a renewed CRL is written to disk, **Then** the verifier is reloaded without expiring.

### User Story 3 - Atomic invalidation of TLS 1.3 session tickets and HTTPS Keep-Alive (Priority: P1)

As a transport security developer, when any file-backed TLS material rotates on disk, all cached TLS 1.3 session resumption tickets in `TlsSessionStore` and active HTTP/1.1 Keep-Alive connections in `HttpsTransport` are atomically invalidated before dispatch.

**Why this priority**: A client must never resume a TLS session or reuse a persistent TCP socket established under an expired or revoked identity.

**Independent Test**: Measure cached ticket count and connection generation before and after file rotation. Verify that rotation clears tickets and closes the pooled socket.

**Acceptance Scenarios**:
1. **Given** cached TLS 1.3 tickets in `TlsSessionStore`, **When** file-backed credentials rotate, **Then** `TlsSessionStore::clear()` purges all tickets.
2. **Given** an active `HttpsConnection`, **When** the generation counter advances due to credential rotation, **Then** `HttpsTransport` closes the previous socket and performs a fresh TLS 1.3 handshake.

## Scope and Boundaries

- **In-Scope**: `crates/kmipkit-transport`: `config.rs`, `tls.rs`, `raw_tls.rs`, `https.rs`, unit tests, and integration tests.
- **Out-of-Scope**: Zero changes to `specification/api/public-api.json`. Public API, C ABI headers, and foreign-language bindings remain completely untouched.

## Normative & Architecture References

- [ADR-0005: Synchronous transports and TLS policy](../../docs/adr/0005-transport-and-tls.md)
- [ADR-0015: Tokio worker for synchronous TLS and HTTPS transports](../../docs/adr/0015-asynchronous-transport-worker.md)
- [ADR-0019: File-backed TLS hot-reload](../../docs/adr/0019-file-backed-tls-hot-reload.md)
- `specs/013-production-transport/spec.md` (updated regarding file re-reading)
