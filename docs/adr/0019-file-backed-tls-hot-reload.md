# ADR-0019: File-backed TLS Hot-Reload and Session Resumption Invalidation

Status: Proposed (Feature KMIPKIT-0063)
Date: 2026-11-06

## Context

Production deployments of `KMIPKit` in cloud and container environments
(e.g., Kubernetes with `cert-manager`, HashiCorp Vault Agent, or SPIFFE/SPIRE)
routinely rotate client mTLS certificates, private keys, and Certificate
Revocation Lists (CRLs) by writing new `.pem` or `.der` files to the filesystem.
Under `specs/013-production-transport/spec.md` User Story 1 Scenario 3, file
sources were originally read once during construction and discarded.
Consequently, when a client certificate or CRL rotates on disk (or an active
CRL reaches its `nextUpdate` expiration as enforced by rustls's
`.enforce_revocation_expiration()`), long-lived `KMIPKit` client instances fail
unless the application process is restarted.

Additionally, `kmipkit-transport` maintains a bounded TLS 1.3 session ticket
cache (`TlsSessionStore`) for raw TLS and a reusable HTTP/1.1 persistent
connection (`HttpsConnection`) for HTTPS. If TLS credentials rotate on disk, any
cached session tickets and active HTTP/1.1 connections must be invalidated
atomically to prevent resuming connections under stale or revoked identities.

## Decision

1. **Retain and Redact File Source Descriptors**: When constructed via
   `CertificateInput::from_pem_file`/`from_der_file`, `PrivateKeyInput::from_pem_file`/`from_der_file`,
   or `RevocationListInput::from_pem_file`/`from_der_file`, `kmipkit-transport`
   retains private, redacted descriptors containing the filesystem path, the
   explicit encoding, and the SHA-256 digest of the successfully loaded bytes.
   Paths remain strictly redacted in `Debug`, `Display`, and all error text.
2. **On-Demand Atomic Refresh**: Before each exchange in `RawTlsTransport` and
   `HttpsTransport`, the transport checks whether any configured file sources
   have changed on disk. If a source file has been modified (detected by
   comparing its SHA-256 digest), the complete set of TLS credentials (client
   identity, root trust, and CRLs) is re-read and validated atomically.
3. **Fail-Closed on Desynchronized / Corrupt Files**: If an external process is in
   the middle of rotating files (e.g., new certificate written but private key
   not yet updated, or partial write), the refresh fails closed as
   `RequestDeliveryState::NotSent` before network I/O, without modifying the
   valid memory state or leaking secret/path information.
4. **Last-Known-Good for Deleted Temporaries**: If a file source was deleted after
   initial construction (e.g., ephemeral temporary files in test harnesses),
   `KMIPKit` preserves the valid in-memory material, ensuring full backward
   compatibility with existing file-source tests.
5. **Atomic Invalidation of Resumption and Connections**: A successful reload
   increments an atomic `tls_generation` counter. Any generation advance
   automatically purges all cached TLS 1.3 tickets from `TlsSessionStore` and
   closes/invalidates any active `HttpsConnection` Keep-Alive socket before
   dispatching the request, ensuring fresh TLS 1.3 handshakes under the new
   credentials.
6. **Zero Public API Changes**: This feature is entirely internal to
   `crates/kmipkit-transport`. `specification/api/public-api.json` and all C,
   Java, and Python binding declarations remain untouched.

## Consequences

- Long-running client instances seamlessly survive certificate, key, CA, and
  CRL rotations without application restart.
- Stale TLS 1.3 resumption tickets and HTTP/1.1 connections cannot outlive
  credential rotation.
- Memory and zeroization invariants are strictly preserved: displaced private
  key material is zeroized on reload.

## Alternatives considered

- **Require application-level client re-creation**: Rejected because in high-concurrency
  services, recreating the entire `Client` disrupts in-flight batches, extension
  registries, and configuration caches.
- **Background filesystem watcher thread**: Rejected because spawning background
  threads violates `KMIPKit`'s strictly bounded worker model and introduces
  platform-specific file watcher dependencies (e.g., inotify/kqueue/FSEvents).
  On-demand check before exchange is lightweight, deterministic, and has zero
  background overhead.
