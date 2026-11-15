# ADR-0020: TLS 1.3 session resumption without 0-RTT and HTTPS idle reconnect

Status: Accepted
Date: 2026-11-06

## Context

High-throughput KMIP clients perform multiple operations (e.g. key creation,
wrapping, activation) in sequence. Under raw TLS (`RawTlsTransport`), performing
a full TLS 1.3 handshake with mutual TLS for every exchange requires repeated
certificate chain validation and cryptographic signature computation, incurring
significant latency and server load. Under HTTPS (`HttpsTransport`), HTTP/1.1
connections are reused via Keep-Alive, but upstream servers and intermediate
proxies routinely terminate idle connections after a timeout.

KMIPKit operates under strict security and reliability rules defined in
`AGENTS.md` §8:
1. "Disable TLS 0-RTT, redirects, HTTP compression, proxies, and TLS key logging."
2. "Never retry a KMIP request automatically. Report whether a failed request
   was not sent, possibly sent, or had begun receiving a response."

We need to guarantee optimal connection performance through TLS 1.3 session
resumption and robust HTTP/1.1 Keep-Alive without violating either invariant.

## Decision

1. **TLS 1.3 Session Resumption in Raw TLS**:
   - `RawTlsTransport` retains an active `TlsSessionStore` inside its
     `TlsClientConfig`, caching up to 16 post-handshake TLS 1.3 resumption
     tickets per server name with a 1-hour maximum lifetime.
   - Early data (0-RTT) remains strictly disabled (`enable_early_data = false`)
     to eliminate replay attacks.
   - When a server issues a `NewSessionTicket`, subsequent exchanges reuse the
     ticket for an abbreviated 1-RTT PSK handshake.

2. **Transparent Reconnect on Stale Idle HTTPS Keep-Alive Sockets**:
   - When `HttpsTransport` reuses a cached idle connection, the socket may have
     been closed by the peer while idle in the pool.
   - If Hyper's `SendRequest::poll_ready` fails before dispatch commit (while
     `delivery_state == NotSent` and the staged request bytes remain intact):
     the dead connection is dropped, a fresh TLS connection is opened, and the
     request is dispatched on the new connection.
   - Once `commit_dispatch()` succeeds, no reconnection or retry is permitted;
     any subsequent transport or HTTP failure fails closed with the appropriate
     delivery state (`PossiblySent` or `ResponseStarted`).

3. **Public API Impact**:
   - 0% change to `specification/api/public-api.json`. The behavior is purely
     internal to `kmipkit-transport`.

## Consequences

- Consecutive raw TLS operations achieve ~5x to 10x faster connection setup
  via 1-RTT resumption while keeping 0-RTT strictly disabled.
- HTTPS applications avoid intermittent connection-reset errors when servers
  close idle Keep-Alive connections.
- Compliance with `AGENTS.md` §8 is strictly preserved: no KMIP request that has
  begun sending is ever retried.
