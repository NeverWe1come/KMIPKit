# Transport and security architecture

This document describes the intended 1.0 transport profile, not current
backend availability. KMIPKIT-0007 provides the public low-level exchange
contract and an internal test fake only; no production TLS/HTTPS adapter or
usable network-client constructor is available yet. A separately approved
transport feature will provide construction from validated configuration and
must not expose arbitrary transport injection. See the
[client execution guide](../user-guide/en/client-execution.md) for the
implemented boundary.

## Target 1.0 transports

### Raw TLS

- `std::net::TcpStream` and rustls.
- Read the 8-byte TTLV header first.
- Validate root tag, type, declared length, configured maximum, and arithmetic
  before allocation.
- Read exactly the value and padding; following bytes belong to the next frame.
- Close the connection on invalid framing, incomplete EOF, or limit violation.

### HTTPS

- `reqwest::blocking` with only the rustls backend.
- HTTPS only, HTTP/1.1 only.
- POST to a configurable path; default `/kmip`.
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
- Mandatory chain, validity, and hostname/SAN verification.
- Optional separate TLS server name for IP-based connections.
- Explicit caller-provided CRLs; no network CRL or OCSP lookup.
- TLS key logging and 0-RTT disabled.
- Session resumption allowed in bounded per-client memory only.
- No persistence or sharing across different identities/configurations.

There is no production switch that accepts arbitrary certificates. Tests use
an ephemeral generated PKI.

The normal build makes no FIPS claim. A future FIPS-specific artifact must pin
the validated provider and environment and verify the full rustls
configuration at runtime.

## Connection model

One client owns one reusable connection and serializes its calls. A network or
protocol framing error invalidates the connection. A later operation may
reconnect. The failed operation is never retried automatically.

One endpoint is configured per client. Alternative endpoints returned by KMIP
are exposed to the application but never selected automatically.

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
