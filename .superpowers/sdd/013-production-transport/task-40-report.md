# T040 Red Report — KMIPKIT-0013

## Scope

Add HTTPS integration coverage for the direct-endpoint and HTTP/1 client
policies, confirm there is no automatic replay after dispatch, and verify the
pre-dispatch cancellation delivery state. This is a test-only change.

## Red evidence and baseline result

The tests were added before any T041 production change. Unlike earlier Red
tasks, the new tests did not expose a current behavior gap: the implementation
already uses Hyper's low-level HTTP/1 connection API over the configured
endpoint, with no redirect, proxy, cookie jar, compression, HTTP/2, failover,
or retry middleware. Therefore the new assertions pass against the baseline.
No failure was manufactured solely to create a Red result; the report records
the actual outcome for the next implementation/refactor tasks.

New HTTPS tests cover:

- rejection of `Location` without connecting to a server-supplied endpoint;
- ignoring proxy environment variables in an isolated child process;
- no cookie persistence, HTTP/2 negotiation, or request compression headers;
- no retry after an HTTP error or after a dispatched request loses its response;
- cancellation while the injected resolver is held before TCP dispatch returns
  `NotSent` and opens no peer connection.

Existing deterministic delivery-state tests in
`crates/kmipkit-transport/tests/timeout_delivery.rs` cover cancellation before
dispatch, Hyper headers emitted with zero TTLV body bytes, and a partial TTLV
body write. They assert `NotSent` or `PossiblySent` as applicable. Existing
HTTPS response tests separately assert `ResponseStarted` when the server has
returned response bytes.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https --offline https_does_not_retry_after_a_dispatched_request_loses_its_response -- --exact --nocapture` | Passed: 1/1 |
| `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1` | Passed: 77/77 |
| `cargo fmt --all` | Passed |
| `git diff --check` | Passed before commit |

## Limitation

The new policy tests pass before T041 because the production path is already
structurally constrained. T041 should make any remaining policy explicit in
the HTTPS configuration/documentation; it should not add redundant behavior
or broaden the API.
