# Transport and security architecture

## Supported transports

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

Transport identity and KMIP message authentication are separate. A client has
default KMIP credentials and a request or batch may replace them. All KMIP 2.1
credential structures in scope are represented. Short-lived OTPs and tickets
can be scoped to one request.

## Logging

Rust uses structured tracing. Adapters bridge redacted events to SLF4J and
Python logging. Logging is disabled until configured. Allowed data includes
operation, sanitized endpoint, correlation IDs, count, duration, and outcome.
Credentials, key material, raw bodies, and secret parameters are forbidden.

Foreign callbacks execute outside client locks and may not reenter the same
client.
