# KMIPKIT-0057 Tasks

## Task 1 — integrate private codec branches [x]

- Bring in the reviewed 0055 and 0056 commits and ensure the common error
  contract composes with both private codecs.
- Keep codec modules private; do not update the API manifest before public
  selection and dispatch are ready.
- Verify both branches contain their independent OASIS fixtures, provenance,
  and focused tests.

## Task 2 — prove encoding selection and client behavior [x]

- Add failing tests for TTLV default compatibility, configured JSON/XML
  defaults, per-exchange override precedence, same-format response decode,
  mismatch errors with delivery state, and no retry/fallback.
- Implement `WireEncoding` and immutable configuration/override through the
  one existing typed execution path, preserving validation, correlation,
  profiles, and local response byte limits.
- Update the client Rust source-inventory tests and public docs.

## Task 3 — carry the selected encoding through HTTPS [x]

- Add failing tests for request Content-Type and response body under each
  selected encoding, preserving HTTPS request/response size limits and
  response cleanup.
- Implement format-aware request headers and response bytes while retaining
  TLS, timeout, delivery-state, redaction, and no-retry behavior.
- Distinguish a valid alternate KMIP media type from HTTP metadata errors and
  report the former as a redacted protocol `InvalidEncoding` error.

## Task 4 — frame raw TLS JSON/XML documents [x]

- Add failing tests for fragmented complete JSON/XML, incomplete/malformed
  documents, trailing bytes, byte-cap enforcement, and one-exchange closure.
- Implement bounded incremental framing; do not add a proprietary length
  prefix. Keep TTLV framing unchanged and close the raw TLS connection after
  every response.
- Evidence: [raw-tls-evidence.md](raw-tls-evidence.md). RED, correction RED,
  GREEN, and REFACTOR are separate signed commits; final independent review
  found no findings.

## Task 5 — expose equivalent four-language selection [x]

- Add the reviewed `WireEncoding`, configuration default, and per-exchange
  override to `specification/api/public-api.json` and its schema mappings.
- Regenerate Rust FFI, C, Java, Python, and parity outputs with the pinned API
  manifest generator. Never edit generated output by hand.
- Add ABI, Java, and Python tests for default and override selection; audit
  exports and error/delivery mapping.
- Evidence: [task5-bindings-evidence.md](task5-bindings-evidence.md).

## Task 6 — close requirements and release evidence [ ]

- [x] Add exact requirement traceability for the applicable KMIP Specification
  v2.1 and KMIP Profiles v2.1 clauses; cite the official message encoding
  vectors and distinguish partial fixture replay from full profile support.
- [x] Update the English and Spanish user guides and architecture/API docs.
- [x] Run local formatting, lint, focused/workspace tests, generated-output
  checks, traceability, security-policy, and supported binding checks; record
  Red, Green, Refactor evidence in separate signed commits. See
  [task6-evidence.md](task6-evidence.md).
- [ ] Obtain hosted multi-platform/multi-binding coverage, CI, and SonarCloud
  evidence for the draft PR.
- [ ] Complete two-independent-implementation KMIP interoperability before the
  1.0 release; OASIS vectors and synthetic peers do not satisfy this gate.
