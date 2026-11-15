# KMIPKIT-0057 raw TLS evidence

## Contract

JSON and XML responses are read incrementally under the configured local byte
cap. A lexical scanner locates one complete top-level document; the JSON/XML
codecs remain responsible for full grammar and KMIP semantic validation. Raw
TLS adds no private length prefix. The transport returns the first complete
document and closes that connection, discarding any surplus independently of
how the TLS stream fragments it. This matches the existing one-frame TTLV
connection lifecycle in ADR-0018 and is a KMIPKit decision, not an OASIS raw-TLS
framing mandate.

Temporary 4 KiB read chunks are zeroized on drop. Response buffers remain
bounded by actual received bytes, and replaced or dropped initialized response
allocations are zeroized. A format mismatch returns a redacted
`EncodingMismatch` transport cause with the established response delivery
state; the client maps it to its protocol `InvalidEncoding` error without
retry or fallback.

## Normative and architectural sources

- OASIS KMIP Profiles v2.1 §5.4 (XML message encoding) and §5.5 (JSON message
  encoding) define the selected text formats.
- OASIS KMIP Specification v2.1 §§10.1–10.2.3 define the encoded KMIP message
  structures consumed by the codecs. The framing scanner does not claim to
  validate those structures.
- KMIPKit ADR-0018 defines one complete document per raw-TLS connection, no
  proprietary prefix, and closure after one response. Its raw-TLS rule is an
  approved project interpretation rather than an OASIS profile requirement.

The suite exercises document boundaries and selected failure modes; it does
not claim complete JSON/XML profile conformance or replay all OASIS vectors.
Those vectors remain with the independent codec specifications, KMIPKIT-0055
and KMIPKIT-0056.

## Red, Green, Refactor

| Phase | Commit | Evidence |
|---|---|---|
| RED | `82045804b` | Fragmented JSON/XML, incomplete/malformed input, trailing input, local byte caps, and cross-encoding expectations were added before framing implementation. The initial focused run failed the JSON/XML fragmented response cases against TTLV-only framing. |
| Correction RED | `d45755bc8` | A second test showed that trailing text was handled differently depending on whether it arrived in the completing read or a later TLS fragment. The focused coalesced-surplus test failed before the fix. |
| GREEN | `8a4a42da1` | Added bounded incremental JSON/XML framing, first-document completion, immediate connection closure, response-buffer growth zeroization, mismatch detection, and fragmented TLS integration coverage. |
| REFACTOR | `d35c5d84a` | Extracted format dispatch and response readers into directly testable generic `AsyncRead` helpers, covered TTLV and text paths, and documented the final boundary policy. |

## Verification

- `cargo test -p kmipkit-transport --lib text_response -- --nocapture`: 11
  response-reader tests passed.
- `cargo test -p kmipkit-transport --test raw_tls -- --quiet`: 132 raw TLS
  integration tests passed, including delayed/coalesced surplus for JSON/XML,
  real TLS fragmentation, EOF, malformed input, limits, format mismatches, and
  connection lifecycle.
- `cargo llvm-cov -p kmipkit-transport --all-features --json --output-path target/0057-rawtls-coverage.json -- --quiet`: all transport tests passed. The changed executable lines in `raw_tls.rs` are 186/186 (100%); `wire_framing.rs` is 152/152 (100%). Full `raw_tls.rs` coverage is 725/790 lines (91.77%); `wire_framing.rs` is 152/152 (100%). The transport package remains above the repository's 85% transport threshold.
- `cargo clippy -p kmipkit-transport --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all --check` and `git diff --check`: passed.
- Independent read-only review found no remaining findings after the
  fragmentation correction and the response-reader refactor.

The local build reports existing AWS-LC probe and missing NASM warnings; Rust
tests and Clippy pass with those toolchain diagnostics.
