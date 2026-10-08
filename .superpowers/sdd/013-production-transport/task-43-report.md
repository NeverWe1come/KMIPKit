# T043 Red Report — KMIPKIT-0013

## Scope

Add deterministic HTTPS integration coverage for timeout and reconnect behavior
in `crates/kmipkit-transport/tests/timeout_delivery.rs`. The tests source-include
the production HTTPS adapter and use a gated injected resolver plus local TLS
1.3 mutual-authentication peers. Production code was not changed for T043.

## Coverage Added

- A connect deadline expires while the injected resolver is blocked. The test
  then releases and waits for the late native result, and observes the listening
  socket to prove that no TCP candidate starts afterward. The error remains
  `NotSent`.
- An unresolved lookup produces no TCP connection while held. After release,
  the exchange completes and the HTTPS peer captures exactly one request with
  the original body.
- A peer accepts TCP and holds the TLS handshake after receiving ClientHello.
  The connect deadline returns `NotSent`; TLS never completes and no HTTP/KMIP
  request is dispatched.
- Two resolver candidates are recorded in returned order. The first endpoint's
  server certificate is untrusted and receives no HTTP request; the second
  endpoint completes verified TLS and receives the one request.
- A total deadline expires after the peer sends HTTP headers and only part of
  the declared body. The error retains `ResponseStarted`, the connection is
  absent from the adapter cache, and a later explicit call opens a second
  connection. The two connections carry distinct `first-call` and
  `second-call` bodies, proving the first call was not replayed.

Existing tests in this target continue to cover generic blocked
`SendRequest::ready()`, read/write/flush deadlines, queued deadlines, dispatch
and response-observation races, and delivery-state transitions.

## Red Evidence

All five new integration tests passed against the implementation present before
T044. This task found no failing baseline assertion, so no product failure was
manufactured. The tests add HTTPS/TLS integration evidence around already
implemented timeout and retry-inhibition behavior.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test timeout_delivery https_ --offline -- --test-threads=1` | Passed: 5/5 |
| `cargo test -p kmipkit-transport --test timeout_delivery https_ --offline` | Passed: 5/5 |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | Passed: 78/78 |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

No OS DNS packet cancellation behavior is asserted. The blocked resolver gates
only the injected native lookup result; deadlines and cancellation remain
responsible for preventing late candidates from reaching TCP/TLS.
