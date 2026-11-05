# Feature Specification: Multi-Encoding Client and Transport Integration

**Feature Branch**: `feature/KMIPKIT-0057-multi-encoding-client-integration`
**Status**: Draft — requires accepted and implemented KMIPKIT-0055 and -0056.
**Owning identifier**: KMIPKIT-0057

Integrate TTLV, JSON, and XML codecs into the single typed-client execution
path and raw TLS/HTTPS transports. Add `WireEncoding { Ttlv, Json, Xml }`,
default TTLV selection in immutable client configuration, and a per-exchange
override. Resolve selection as override, then configured default, then TTLV.
The request and its response use the same selected encoding. A response format
mismatch returns a redacted protocol error with delivery evidence and never
triggers fallback or retry.

**Normative sources**: OASIS KMIP Specification v2.1 §§10.1–10.2.3; OASIS
KMIP Profiles v2.1 §§5.3.1–5.3.2, 5.4, and 5.5. Apply the contextual HTTPS
interpretation recorded in ADR-0017 and KMIPKIT-0054.

**Acceptance criteria**: Integrate all codecs on both transports; raw TLS
incrementally reads one complete text document per connection without a
proprietary prefix and closes after one response; HTTPS sends matching
Content-Type and body; keep local byte caps separate from TTLV-defined
Maximum Response Size; update public-api.json and its generated outputs in one
change; provide equivalent Rust/C/Java/Python config and call override; test
fragmentation, limits, content types, mismatches, delivery state, redaction,
zeroization, and absence of retries. Profile claims require applicable
official evidence and must not overstate support.

**Dependencies and exclusions**: Requires accepted/implemented 0054, 0055,
and 0056. This is the only feature among 0054–0057 that changes
`specification/api/public-api.json`, public client dispatch, generated API
outputs, or foreign-language public surfaces. Does not change operation
semantics, protocol version, TLS policy, operation direction, or retry policy.
