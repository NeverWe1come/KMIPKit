# Public Rust Contract: KMIP 2.1 Query and Ping

## Typed request models

- `PingRequest` has an empty operation payload.
- `QueryRequest` requires at least one ordered Query Function value and supports optional Object Groups containing zero or more ordered Object Group attributes with Text String values (§6.1.40 Table 282; §7.23 Table 375; §4.35 Tables 99–100).
- All 14 standard Query Function values are named; valid extension and unknown values remain available through the shared open-enumeration policy.
- Query and Ping are variants of the existing client request and ordered batch models.

## Client entry points and outcomes

- `Client::ping` and `Client::query` each make one call through the shared `Client::execute` exchange path.
- `QueryResponse` represents either the empty operation payload described in §6.1.40 or the structured Table 283 form, including its required Protection Storage Masks member (which may contain an empty list). It preserves unknown values/items and does not resolve the source conflict tracked as `KMIPKIT-DISC-047`.
- Ping success means the server returned a successful Ping response. It is not a general health or readiness guarantee.
- Failures preserve the common KMIP result, reason, permitted message, and delivery state.
- No API retries, polls, or issues follow-up calls automatically.

## Compatibility and diagnostics

- This is additive pre-1.0 Rust API. C, Java, Python, and generated-binding parity are separate later API work.
- Unknown values and structurally valid extension Items are preserved.
- Errors and Debug output do not include raw KMIP bodies.
- Existing transport, TLS, batch, decoder-limit, and redaction contracts remain authoritative.