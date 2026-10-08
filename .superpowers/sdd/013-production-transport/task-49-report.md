# T049 boundary-test evidence

## Added contracts

- Raw-TLS and HTTPS direct adapters reject request bytes above `max_request_bytes` before a
  loopback listener accepts a connection.
- The typed client rejects a request whose encoded form exceeds `CodecLimits::max_message_bytes()`
  before connecting.
- A valid typed response exactly at the configured codec byte limit is accepted.
- A malformed production response produces a `ResponseStarted` typed protocol error and does not
  expose a sentinel contained in the response bytes.
- A trybuild contract proves `ClientBatchResponse` has no raw `as_bytes` accessor.
- Existing `client_rejects_oversized_transport_response_before_decoder_entry` uses a deliberately
  non-compliant fake transport to prove returned-length checking happens before decoder entry.

## Baseline and TDD disposition

- Test-only commit: `bf232e7` (`test(client): add production transport boundary contracts`).
- The production flow was already implemented in `82eac36`. Each new behavior test passed on its
  first run against that implementation, so no artificial red failure or product-code change was
  created. T050 records this overlap explicitly.

## Verification

- `cargo test -p kmipkit-client --all-targets --all-features --offline --quiet`: passed; 203 library
  tests and all integration targets, including 14 production-client tests and both trybuild cases.
- `cargo clippy -p kmipkit-client --all-targets --all-features --offline -- -D warnings`: passed.
- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
