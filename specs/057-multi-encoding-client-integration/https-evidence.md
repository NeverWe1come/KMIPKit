# KMIPKIT-0057 HTTPS integration evidence

## Scope and requirement traceability

This slice implements Task 3 only. The transport accepts already encoded
bytes; typed message validation and same-format body decoding remain in the
single client execution path. Empty-container transport fixtures distinguish
wire formats without claiming valid typed requests or official profile cases.

| Stable requirement | Exact source | Implementation | Verification |
| --- | --- | --- | --- |
| KMIPKIT-0057-HTTPS-001 | OASIS KMIP Profiles v2.1 §5.3.1, list items 2, 5–9; ADR-0018 decision 4 | Selected-format request Content-Type, unchanged body, POST, Content-Length, no-cache | `selected_encoding_posts_exact_body_and_headers_and_accepts_the_same_format_at_the_byte_cap` |
| KMIPKIT-0057-HTTPS-002 | Profiles v2.1 §5.3.2, list items 2, 4–7; ADR-0018 decisions 1, 4–5 | Require one response Content-Type matching this exchange; retain status and length validation | `selected_encoding_rejects_mismatched_duplicate_missing_and_invalid_response_types_without_allocation`; `selected_encoding_changes_on_a_reused_https_connection` |
| KMIPKIT-0057-HTTPS-003 | ADR-0018 decision 6; approved 0057 acceptance criteria | Actual request/response byte caps, allocation guard, partial response zeroization | `selected_encoding_preserves_request_limits_before_connection`; `selected_encoding_preserves_response_byte_limits_and_partial_response_zeroization`; exact-cap success test |
| KMIPKIT-0057-HTTPS-004 | ADR-0018 decision 5; constitution IV; approved 0057 acceptance criteria | Redacted HTTP mismatch error, ResponseStarted evidence, invalidated session, no retry/fallback | `selected_encoding_mismatch_never_retries_or_falls_back`; response type rejection test |

The contextual body rule is KMIPKit's interpretation of the internal source
inconsistency recorded by ADR-0018. This evidence does not claim an OASIS
resolution, full HTTPS/JSON/XML profile conformance, or certification.

## RED

Command (Windows x86_64 MSVC):

```text
cargo test -p kmipkit-transport --test https encoding_tests -- --nocapture
```

Result: exit 1; 1 passed, 5 failed. Expected failures show JSON/XML outgoing
Content-Type remains octet-stream, JSON/XML responses are rejected, text
exchanges accept a TTLV response, and media-type rejection masks the selected
format's response-byte limit. The request-limit compatibility case passes.
No production code was changed before this run.

Cargo reports existing AWS-LC build diagnostics about the optional stdalign
compiler probe and missing NASM; the Rust target builds and runs the tests.
