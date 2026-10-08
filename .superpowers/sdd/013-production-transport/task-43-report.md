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
- `HttpsTransport::new` exercises the production system resolver with an IPv4
  loopback endpoint and the configured TLS server-name override. The mutual-TLS
  exchange succeeds and the peer captures one request.
- Two resolver candidates are attempted in returned order. The first endpoint
  accepts TCP, then holds its untrusted TLS handshake. While it is held, the
  second listener observes no TCP connection. Releasing the handshake lets
  certificate verification fail; only then does the trusted second endpoint
  complete TLS and receive the one request.
- A total deadline expires after the peer sends HTTP headers and only part of
  the declared body. The error retains `ResponseStarted`, the connection is
  absent from the adapter cache, and a later explicit call opens a second
  connection. The two connections carry distinct `first-call` and
  `second-call` bodies, proving the first call was not replayed.
- The resolver-deadline, stalled-handshake, and partial-body total-deadline
  errors each assert `TransportCauseCategory::Timeout` alongside delivery
  evidence.

Existing tests in this target continue to cover generic blocked
`SendRequest::ready()`, read/write/flush deadlines, queued deadlines, dispatch
and response-observation races, and delivery-state transitions.

## Red Evidence

All six HTTPS integration tests passed against the implementation present
before T044. This task found no failing baseline assertion, so no product
failure was manufactured. The tests add HTTPS/TLS integration evidence around
already implemented timeout, resolver, candidate-order, and retry-inhibition
behavior.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test timeout_delivery https_ --offline -- --test-threads=1` | Passed: 6/6 |
| `cargo test -p kmipkit-transport --test timeout_delivery https_ --offline` | Passed: 6/6 |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | Passed: 79/79 |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

No OS DNS packet cancellation behavior is asserted. The blocked resolver gates
only the injected native lookup result; deadlines and cancellation remain
responsible for preventing late candidates from reaching TCP/TLS.

## Independent QA Review

QA re-reviewed the correction commit `c230551` and approved T043 with no
blocking findings. It verified the first-candidate TLS handshake is held while
the second listener observes no TCP connection, and repeated that candidate
ordering test five times successfully. Deadline tests assert both delivery
state and `TransportCauseCategory::Timeout`. The production adapter test uses
`HttpsTransport::new` and therefore the default `Resolver::system()` wiring;
its endpoint is the literal IPv4 loopback address, so hostname resolution is
covered separately by the resolver's `localhost` test rather than by this
HTTPS adapter case. QA passed the six HTTPS cases, three `system_resolver`
cases, the candidate-order test five times, formatting, and diff checks.

## Independent Security Review

Security review found no issue that invalidates T043 evidence. It confirmed
that the HTTPS system-resolver case is loopback-only, uses an in-memory
ephemeral PKI, and does not contact external DNS; peer and gate waits are
bounded; key material is not printed or persisted; and request bodies prove
that a timed-out call is not replayed. The audit manually inspected the three
changed files after its automated inventory returned no rows. It reported zero
security findings.
