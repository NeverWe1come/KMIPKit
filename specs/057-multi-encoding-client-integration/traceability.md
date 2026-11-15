# KMIPKIT-0057 requirement traceability

This integration composes the private codec contracts from KMIPKIT-0055 and
KMIPKIT-0056 with one typed client path. OASIS source files and exact fixture
copies remain immutable. Client defaults, per-call override precedence,
no-retry behavior, and raw-TLS document framing are KMIPKit API or transport
policy decisions; they are not presented as additional OASIS requirements.

| Requirement | Normative source / project decision | Implementation | Verification evidence |
|---|---|---|---|
| KMIPKIT-0057-FR-001 | Project API decision in ADR-0018 and KMIPKIT-0054 common contract. OASIS Profiles v2.1 §§5.4 and 5.5 define the XML and JSON encodings; the selected default/override API is KMIPKit policy. | `WireEncoding::{Ttlv, Json, Xml}`, immutable `ClientConfiguration` default (TTLV), optional `RequestOptions` override; precedence override → configuration → TTLV. Four-language mapping is in `specification/api/public-api.json`. | `crates/kmipkit-client/tests/unit/wire_encoding_tests.rs`: `default_configuration_keeps_ttlv_and_exposes_resolved_selection_to_transport`, `configured_json_and_xml_defaults_encode_and_decode_the_same_selected_format`, `per_exchange_override_wins_without_mutating_client_or_caller_options`; generated API/parity suite and binding evidence in [task5-bindings-evidence.md](task5-bindings-evidence.md). |
| KMIPKIT-0057-FR-002 | KMIP Specification v2.1 §§10.1–10.2.3 for KMIP message structure; Profiles v2.1 §§5.4 and 5.5 for their wire representations. | One typed operation, validation, correlation and delivery path; selected encoder writes the request and selected decoder reads its corresponding response. | `configured_json_and_xml_defaults_encode_and_decode_the_same_selected_format`; codec semantic cases are recorded in [0055 traceability](../055-json-encoding/traceability.md) and [0056 implementation traceability](../056-xml-encoding/implementation.md). |
| KMIPKIT-0057-FR-003 | Profiles v2.1 §5.3.1 (client HTTPS request) and §5.3.2 (server HTTPS response); §§5.4 and 5.5 (XML and JSON encoding). The body interpretation under HTTPS §5.3.1–§5.3.2 follows ADR-0018 decision 4 because the source text is internally contradictory. Raw-TLS document framing and one-exchange closure are KMIPKit policy in ADR-0018. | HTTPS sends selected body and `Content-Type`; raw TLS preserves TTLV framing and incrementally reads a bounded complete JSON/XML document without a private prefix, then closes. | Exact HTTPS requirements, tests and byte-cap evidence are in [https-evidence.md](https-evidence.md). Fragmentation, malformed/incomplete documents, trailing data, limits and closure are in [raw-tls-evidence.md](raw-tls-evidence.md). |
| KMIPKIT-0057-FR-004 | ADR-0018 and the KMIPKIT-0054 redacted common error/delivery contract. No retry/fallback is an explicit project policy. | A response-format mismatch yields a redacted protocol encoding error with delivery state; no alternate codec or request retry is attempted. | `mismatched_response_formats_return_redacted_protocol_errors_without_retry_or_fallback`, `transport_encoding_mismatch_becomes_a_redacted_protocol_error`; transport mismatch cases in `https-evidence.md` and `raw-tls-evidence.md`. |
| KMIPKIT-0057-FR-005 | KMIPKit 1.0 Rust/C/Java/Python parity boundary and additive ABI decision. | Configuration default and per-exchange selection are mapped for Rust, C, Java and Python; older execute entry points keep their behavior and symbols. | Manifest validation, generated output and binding tests in [task5-bindings-evidence.md](task5-bindings-evidence.md). This verifies selection mappings, not parity of all KMIP operations. |

## Official vector scope

The JSON codec pins OASIS KMIP Profiles v2.1 §5.5.4.1,
`MSGENC-JSON-M-1-21`; provenance, source hashes and the exact replay boundary
are in `crates/kmipkit-client/tests/fixtures/json/README.md`. It checks the
three literal normative JSON wire examples and four descriptor messages with
deterministic `$NOW` binding. The XML codec pins §5.4.4.1,
`MSGENC-XML-M-1-21`, and replays all four message documents; provenance and
hash are in `crates/kmipkit-client/tests/fixtures/xml/README.md`. The codecs'
requirement-to-case mappings are in the linked 0055 and 0056 records.

These are focused codec vectors, not complete client-profile evidence. They do
not establish Baseline Client support under Profiles §§5.4.2 or 5.5.2, full
profile conformance, formal certification, or interoperability with two
independent KMIP implementations. The transport and public-client tests also
include synthetic local peers, which are implementation evidence rather than
additional official OASIS cases.

## Remaining release gates

`task5-bindings-evidence.md` records the current export-audit baseline: the
fresh Rust audit still reports 907 unrelated missing manifest paths that were
also present on the KMIPKIT-0054 base. Task 6 remains open until the required
workspace, coverage, platform, security and two-implementation interoperability
gates have current evidence and these broader API inventory blockers are
resolved or explicitly dispositioned by the maintainer.
