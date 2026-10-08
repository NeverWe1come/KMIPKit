# Transport and security architecture

This document describes the intended 1.0 transport profile and its accepted
implementation boundary. KMIPKIT-0013 implements production TLS/HTTPS
adapters and a synchronous typed-client constructor from validated
configuration. Remaining operation coverage, platform, documentation, and
independent-review gates are tracked in that specification. The constructor
does not expose arbitrary transport injection. See the
[client execution guide](../user-guide/en/client-execution.md) for the
implemented boundary.

## Target 1.0 transports

### Raw TLS

- Tokio TCP and `tokio-rustls` on the private per-client worker.
- Read the 8-byte TTLV header first.
- Validate root tag, type, declared length, configured maximum, and arithmetic
  before allocation.
- Read exactly the value and padding, then close the connection after that one
  response frame. Any surplus stream bytes are discarded with the connection.
- Close the connection on invalid framing, incomplete EOF, or limit violation.

### HTTPS

- Hyper's HTTP/1 client parser over the verified rustls stream.
- HTTPS only, HTTP/1.1 only.
- POST to a configurable path; default `/kmip`.
- Send exactly one `Host` header derived from the endpoint authority; the
  target path and TLS verification-name override do not alter it.
- `Content-Type: application/octet-stream`.
- No redirect, proxy, or compression behavior.
- Strict KMIP HTTPS profile status, header, and body checks.

## TLS policy

- TLS 1.3 is the minimum and maximum.
- rustls with aws-lc-rs.
- Mutual TLS.
- Certificate and private key input from PEM or DER files and in-memory
  buffers.
- Explicit CA roots; optional platform trust is an explicit choice.
- Mandatory chain, validity, and hostname/SAN verification on each full
  handshake.
- Optional separate TLS server name for IP-based connections.
- Explicit caller-provided CRLs; no network CRL or OCSP lookup.
- TLS key logging and 0-RTT disabled.
- Support TLS 1.3 session resumption with at most 16 in-memory tickets per
  client configuration and a one-hour local expiry. A resumed session
  inherits the peer identity and trust/CRL decision from its verified full
  handshake; rebuilding the client applies changed trust inputs and starts
  with an empty cache.
- No persistence or sharing across different identities/configurations.

There is no production switch that accepts arbitrary certificates. Tests use
an ephemeral generated PKI.

The normal build makes no FIPS claim. A future FIPS-specific artifact must pin
the validated provider and environment and verify the full rustls
configuration at runtime.

## Connection model

One client serializes its calls through one worker. HTTPS may reuse a healthy
HTTP/1 connection; raw TLS closes its connection after each response frame.
A network or protocol framing error invalidates the current connection. A
later operation may reconnect. The failed operation is never retried
automatically.

One endpoint is configured per client. Alternative endpoints returned by KMIP
are exposed to the application but never selected automatically.

The worker calls `std::net::ToSocketAddrs` through `spawn_blocking` to preserve
the operating system's resolver policy. It acquires one of 32 shared permits
per loaded KMIPKit library instance before submission and retains at most the
first 16 returned addresses in OS order. A started native lookup may continue
after timeout or cancellation and retain its permit; KMIPKit discards its late
result before it can open a candidate connection or dispatch KMIP data. The
OS owns DNS packet retries, upstream concurrency, resolver caches, hosts and
search rules, and split-DNS/VPN routing. KMIPKit makes no numeric claims about
those OS-controlled behaviors. See
[ADR-0016](../adr/0016-native-system-name-resolution.md).

## Timeouts

Defaults:

- Connect: 10 seconds.
- Write: 30 seconds.
- Read: 30 seconds.
- Total request deadline: 60 seconds.

All are configurable per client and overridable per request. A deliberate
unbounded deadline is possible. Timeout errors state whether data was not sent,
possibly sent, or a response had begun. A timed-out connection is invalidated.

## Decoder limits

Defaults:

- Complete message: 16 MiB.
- Structure nesting: 64.
- Element count: 100,000.
- Individual value: bounded by the complete message maximum.

Values are configurable. Checks occur before allocation and use overflow-safe
arithmetic. Version 1.0 buffers one complete message; streaming values are
future work.

## Credentials

Transport identity (TLS client identity) and KMIP message Authentication are
separate. KMIPKIT-0008 models ordered Authentication and raw-preserving
Credentials in memory. It does not add Authentication or Credential data to a
request payload; request integration is outside this increment.

The typed Hashed Password model requires caller-provided Timestamp and hashed
bytes. Hashing Algorithm is optional: when absent, the model reports the
effective SHA-256 value while preserving that the field was absent. KMIPKit
does not calculate the hash. A typed Device value contains at least one of
Device Serial Number, Network Identifier, Machine Identifier, or Media
Identifier. The caller must supply one or a combination that is actually
unique; KMIPKit does not infer a uniqueness comparison scope or check actual
uniqueness. Generic TTLV can retain a Device tree without typed validation.

The synchronous and asynchronous request builders emit
`Attestation Capable Indicator = True` because the public API can construct
an Attestation Credential. This advertises
construction capability only; it neither sends Authentication or Credential
payloads nor generates/verifies evidence or predicts server acceptance. There
is no request-level override.

KMIPKit does not log Credential contents; text and byte wrappers also redact
Debug, Display, and validation diagnostics. KMIPKit zeroizes initialized bytes
in the owned current allocation when its owner is dropped, including after
ownership moves into a TTLV value. Spare or uninitialized capacity is not
covered unless initialized and cleanup is verified. This cannot clear
caller-made copies, old allocations from earlier buffer growth, callback-view copies,
temporary stack/register copies, or copies retained by foreign runtimes, TLS,
the operating system, or dependencies. The guarantee is about KMIPKit-owned
storage and does not claim process-wide erasure.

## Logging

Rust uses structured tracing. Adapters bridge redacted events to SLF4J and
Python logging. Logging is disabled until configured. Allowed data includes
operation, sanitized endpoint, correlation IDs, count, duration, and outcome.
Credentials, key material, raw bodies, and secret parameters are forbidden.

Foreign callbacks execute outside client locks and may not reenter the same
client.
