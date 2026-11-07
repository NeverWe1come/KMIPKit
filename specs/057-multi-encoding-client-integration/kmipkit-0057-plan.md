# KMIPKIT-0057 Implementation Plan

## Goal

Integrate the private TTLV, JSON, and XML codecs in the existing single typed
client execution path, expose immutable default selection and per-exchange
override through the Rust, C, Java, and Python surfaces, and carry the chosen
wire format through HTTPS and raw TLS without changing KMIP operation
semantics.

## Dependencies and constraints

- Start from the approved KMIPKIT-0054 head and include reviewed 0055 and 0056
  codec commits before implementing dispatch.
- Use `spec.md`, ADR-0018, and
  `specs/054-multi-encoding-governance/contracts/common-wire-codec.md` as the
  binding design sources.
- Preserve typed validation, correlation, profile checks, delivery evidence,
  request and response zeroization, no retry, and one request/response exchange.
- Keep `Maximum Response Size` in its OASIS logical TTLV meaning; enforce
  `CodecLimits::max_message_bytes()` against actual encoded or received bytes.
- HTTPS carries the selected encoding's body and Content-Type. Raw TLS carries
  one complete document per connection, detects text-document completion
  incrementally under byte limits, and closes after one response.
- Update `specification/api/public-api.json` only in this feature, regenerate
  every declared output with the pinned generator, and prove Rust/C/Java/
  Python parity.
- Do not alter the pinned upstream OASIS source copies.

## Implementation sequence

1. Integrate the reviewed private codecs and common error contract. Add tests
   proving a semantic `Item` request takes each codec path and responses are
   decoded with the same selected codec.
2. Add `WireEncoding` and immutable client default/override resolution using
   the precedence override, configured default, TTLV. Preserve the existing
   public behavior when neither setting is supplied.
3. Carry the resolved encoding into transport exchange. Set and validate HTTP
   content types as required; add bounded incremental JSON and XML document
   framing for raw TLS and close each raw TLS connection after one exchange.
4. Add manifest mappings and generated Rust FFI, C, Java, and Python
   declarations for the encoding selector and configuration/override
   capabilities. Keep old calls source-compatible and document the new calls.
5. Complete focused, vector, fake-transport, framing, mismatch, bounds,
   zeroization, parity, traceability, and generated-output checks. Update
   English and Spanish user documentation and implementation evidence.

## Test discipline

For each implementation slice, keep RED, GREEN, and REFACTOR evidence in
separate signed development commits. New behavior starts with a failing test.
Use OASIS `MSGENC-JSON-M-1-21` and `MSGENC-XML-M-1-21` only for the claims
they actually exercise; do not treat a partial fixture replay as full profile
conformance. Run focused tests while iterating and the required workspace,
binding, generation, traceability, coverage, and interoperability checks
before review.

## Risks to resolve during implementation

- Raw TLS text framing must distinguish document completion from incomplete,
  malformed, and trailing data without allocating beyond the local byte cap.
- JSON/XML codecs must not retain parser errors or raw payloads, and temporary
  wire buffers must follow the existing zeroizing ownership contract.
- API manifest support spans C ABI, generated declarations, Java and Python
  wrappers, and parity evidence; additions must use the repository generator.
- The approved contextual HTTPS rule is a KMIPKit interpretation recorded in
  ADR-0018, not an OASIS clarification.
