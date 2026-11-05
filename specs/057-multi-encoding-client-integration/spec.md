# Feature Specification: Multi-Encoding Client and Transport Integration

**Feature Branch**: `feature/KMIPKIT-0057-multi-encoding-client-integration`
**Status**: Approved for implementation by direct maintainer instruction on 2026-11-05; requires KMIPKIT-0054 and implemented KMIPKIT-0055 and -0056.
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
interpretation recorded in ADR-0018 and KMIPKIT-0054.

**Acceptance criteria**: Integrate all codecs on both transports; raw TLS
incrementally reads one complete text document per connection without a
proprietary prefix and closes after one response; HTTPS sends matching
Content-Type and body; keep local byte caps separate from TTLV-defined
Maximum Response Size; update public-api.json and its generated outputs in one
change; provide equivalent Rust/C/Java/Python config and call override; test
fragmentation, limits, content types, mismatches, delivery state, redaction,
zeroization, and absence of retries. Profile claims require applicable
official evidence and must not overstate support.

## Requirements

| ID | Requirement | Normative source / project decision | Verification |
|---|---|---|---|
| KMIPKIT-0057-FR-001 | The typed client SHALL support TTLV, JSON, and XML selection. The immutable client configuration SHALL default to TTLV, and an exchange override SHALL take precedence over that default. | KMIPKit ADR-0018; KMIPKIT-0054 common codec contract | Rust client selection tests and four-language API parity tests |
| KMIPKIT-0057-FR-002 | The selected encoding SHALL be used for both request encoding and the corresponding response decoding while preserving one shared operation, validation, and correlation path. | OASIS KMIP Specification v2.1 §§10.1–10.2.3; KMIPKit ADR-0018 | Equivalent-semantic vectors and fake-transport client tests for each encoding |
| KMIPKIT-0057-FR-003 | HTTPS exchanges SHALL use the selected encoding in their body and media type. Raw TLS text exchanges SHALL read one complete JSON or XML document incrementally within the local byte limit, without adding a private length prefix, then close the connection after the response. | OASIS KMIP Profiles v2.1 §§5.3.1–5.3.2, 5.4, and 5.5; HTTPS interpretation in ADR-0018; raw-TLS framing decision in ADR-0018 | HTTPS media-type/body tests, fragmented raw-TLS tests, response-limit tests, and connection-lifecycle tests |
| KMIPKIT-0057-FR-004 | A response in another encoding SHALL produce a redacted protocol encoding error with the established delivery state. The client SHALL NOT retry or fall back to another encoding. | KMIPKit ADR-0018 and KMIPKIT-0054 error contract | Mismatch, redaction, delivery-state, and single-attempt tests |
| KMIPKIT-0057-FR-005 | Rust, C, Java, and Python SHALL provide equivalent configuration-default and per-exchange override capabilities. Existing FFI entry points SHALL remain compatible and use TTLV/configuration defaults. | KMIPKit 1.0 language-parity boundary and additive C ABI decision | Manifest validation, generated parity outputs, C ABI tests, Java tests, and Python tests |

**Dependencies and exclusions**: Requires accepted/implemented 0054, 0055,
and 0056. This is the only feature among 0054–0057 that changes
`specification/api/public-api.json`, public client dispatch, generated API
outputs, or foreign-language public surfaces. Does not change operation
semantics, protocol version, TLS policy, operation direction, or retry policy.
