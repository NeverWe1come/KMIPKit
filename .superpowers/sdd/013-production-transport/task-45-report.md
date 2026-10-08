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
