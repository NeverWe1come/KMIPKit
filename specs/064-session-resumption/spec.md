# Specification: TLS 1.3 Session Resumption and HTTPS Keep-Alive Reconnect (KMIPKIT-0064)

## 1. Overview and Purpose

This specification governs client-side connection performance and resilience hardening in `kmipkit-transport` without altering the public API surface.

High-frequency KMIP client workflows (e.g. batch cryptographic operations, rapid key retrieval) require optimal connection handshake performance and resilience against idle connection drops:
1. Under Raw TLS (`RawTlsTransport`), successive exchanges directed to the same server must perform TLS 1.3 PSK Session Resumption (1-RTT) rather than full 2-RTT handshakes.
2. In all cases, 0-RTT early data must remain strictly disabled (`enable_early_data = false`) per `AGENTS.md` §8.
3. Under HTTPS (`HttpsTransport`), when reusing an idle Keep-Alive connection, if the underlying TCP/TLS socket was closed by the remote peer while idle (before dispatch commit), the transport must transparently reconnect fresh and dispatch the staged request, avoiding spurious failures.
4. If request dispatch has already been committed (`PossiblySent`), no automatic retry may occur under any circumstances per `AGENTS.md` §8.

## 2. Normative References

- **OASIS KMIP 2.1 Profile §5.1**: Raw TTLV over TLS transport.
- **OASIS KMIP 2.1 Profile §5.3**: HTTPS/HTTP 1.1 transport.
- **RFC 8446**: The Transport Layer Security (TLS) Protocol Version 1.3 (§2.2 Resumption and Pre-Shared Key, §4.6.1 New Session Ticket).
- **RFC 9112**: HTTP/1.1 (§9.3 Connection Management and Keep-Alive).
- **ADR-0020**: TLS 1.3 session resumption without 0-RTT and HTTPS idle reconnect.
- **KMIPKit `AGENTS.md` §8**: Security invariants (no 0-RTT, no automatic retry of dispatched KMIP requests).

## 3. Scope and Public API Impact

- **Public API change**: Exactly 0%. `specification/api/public-api.json` is completely unchanged.
- **Crate**: `kmipkit-transport`.

## 4. Acceptance Criteria

1. **`REQ-064-001` (Raw TLS Resumption)**:
   When `RawTlsTransport::exchange()` or `exchange_with_options()` is executed multiple times against an endpoint that provides `NewSessionTicket`, subsequent handshakes MUST use the cached session ticket and negotiate `HandshakeKind::Resumed`.

2. **`REQ-064-002` (Zero 0-RTT Early Data)**:
   `ClientConfig.enable_early_data` MUST remain `false`. Under no circumstances may client application data be transmitted before handshake completion.

3. **`REQ-064-003` (Session Ticket Storage Limits)**:
   The session ticket store MUST enforce a maximum of 16 cached tickets per server name and an expiration of 1 hour from receipt.

4. **`REQ-064-004` (HTTPS Keep-Alive Reconnect on Pre-Dispatch Idle Closure)**:
   When `HttpsTransport` attempts to reuse an idle connection from its pool, if the socket is found closed during sender readiness check before `commit_dispatch()`:
   - The transport MUST discard the dead connection.
   - The transport MUST establish a fresh TLS connection to the endpoint.
   - The transport MUST dispatch the staged request and return the valid response.
   - The entire recovery MUST complete seamlessly without failing or leaking resources.

5. **`REQ-064-005` (No Retries Once Dispatched)**:
   If a network, TLS, or HTTP error occurs after `commit_dispatch()` has succeeded, the transport MUST NOT attempt a reconnection or retry, failing closed with `PossiblySent` or `ResponseStarted`.
