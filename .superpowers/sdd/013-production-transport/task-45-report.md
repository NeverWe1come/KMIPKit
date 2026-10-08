# T045 Red Report — KMIPKIT-0013

## Scope

Add a production HTTPS regression for a read-inactivity timeout after response
headers and part of the response body have arrived. The test checks
`ResponseStarted`, the `Timeout` cause, redacted `Display` and `Debug`, and
connection invalidation. It uses a local TLS 1.3 mutual-authentication peer,
an in-memory ephemeral PKI, and distinct request/response sentinel bodies.

## Red Evidence

The test was run before changing production code:

```text
cargo test -p kmipkit-transport --test timeout_delivery https_body_read_timeout_preserves_timeout_cause_and_redacts_payloads --offline -- --exact --nocapture
```

It failed at the expected cause assertion: actual `Http`, expected `Timeout`.
The peer had sent HTTP headers and a partial body, then held the response until
the configured 500 ms read-inactivity deadline. The test therefore isolates
the classification defect in Hyper's response-body error path; it does not
fail due to setup, TLS, or a total deadline.

No production files were changed for this Red commit.

## Green Evidence

`read_response_body` now passes the Hyper body error through the existing
transport error classifier. Hyper parse and incomplete-message errors remain
HTTP failures; a nested `io::ErrorKind::TimedOut` is recognized as a timeout.
`TransportError` continues to discard dependency error sources before public
formatting, so the classifier can preserve the cause without exposing request
or partial-response bytes.

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test timeout_delivery https_body_read_timeout_preserves_timeout_cause_and_redacts_payloads --offline -- --exact --nocapture` | Passed: 1/1; timeout category, `ResponseStarted`, invalidation, and both error-format redaction assertions passed. |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | Passed: 80/80. |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed. |
| `cargo fmt --all --check` | Passed after formatting the changed source. |
| `git diff --check` | Passed. |

The targeted test first failed with the expected `Http` category before the
source change and passed with the corrected `Timeout` category afterward.

## Refactor Evidence

The body-frame error conversion now lives in `response_body_error`, making the
HTTP response-body phase explicit instead of wrapping its Hyper error through
the generic I/O mapper at the call site. The shared Hyper classifier detects
nested `TimedOut` and `UnexpectedEof` I/O causes, while parse and incomplete
message errors remain HTTP failures. `TransportError` still removes the
source before formatting.

The first complete HTTPS run after Green caught a regression in the existing
truncated-body case: a nested `UnexpectedEof` was classified as `Io`. The
classifier was refined to keep truncated HTTP bodies in category `Http` while
retaining `Timeout` for read-inactivity expiry.

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https https_rejects_http_parser_errors_and_truncated_response_bodies --offline -- --exact --nocapture` | Passed: 1/1 after preserving `UnexpectedEof` as HTTP. |
| `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1` | Passed: 78/78. |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | Passed: 80/80. |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed. |
| `cargo fmt --all --check` | Passed. |
| `git diff --check` | Passed. |

## Independent Review

QA approved commits `cca130c..0aff8d2` without findings. It verified the new
timeout/redaction assertions, the existing parser/truncated-body behavior,
both complete HTTPS and timeout-delivery targets, strict Clippy, formatting,
and diff checks.

Security review found no findings. It confirmed `TransportError::new` drops
the Hyper/I/O source before public formatting, no logging was added, and the
request and partial-response sentinels are checked against both `Display` and
`Debug`. The review was manual; no separate automated security scan was
completed for this small diff.
