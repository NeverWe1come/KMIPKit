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
