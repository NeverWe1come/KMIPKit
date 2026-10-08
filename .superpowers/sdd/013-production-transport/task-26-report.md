# T026 Green Report — KMIPKIT-0013

## Scope and design

Green source commit: `9e7c05755e63d1a8faf78ddd43e6c3cb680d1919`.
T025 Red test commits: `3b1579840010d42181e8fb4fad9449fd7d4ff919` and
`0560662951a4e4df15938e5a6bc84b700ea9eaf3`; the initial missing-module Red
and test expectations are recorded in `task-25-report.md`.

Added the public `RawTlsTransport`, re-exported from `kmipkit-transport`.
`RawTlsTransport::new(TransportConfig)` accepts a validated raw-TLS config,
and `exchange_with_options(&mut self, request, max_response_bytes, &RequestOptions)`
adds per-call timeout overrides. The existing `Transport::exchange` method
uses the configured timeout defaults. `max_response_bytes` caps the complete
response frame, including its eight-byte TTLV header.

The adapter builds the existing rustls client policy at construction and
defers the per-client worker, DNS, and socket activity until exchange. The
synchronous entry point submits asynchronous connect, TLS handshake, and I/O
to `ClientWorker`; it does not perform blocking network operations on the
calling thread. It reuses `Resolver` and `DeadlineIo`. One absolute deadline
is captured at public entry and carried through queueing, resolution, connect,
handshake, write, and response reads, alongside the configured per-phase
deadlines. The dispatch gate is committed immediately before the request
write. Failures keep the observed delivery state and use sanitized cause
categories; endpoint and dependency error text is not exposed.

The adapter sends the caller's bytes unchanged, makes no KMIP encoding or
automatic retry, and opens a fresh TLS connection per exchange. It uses the
existing TLS 1.3-only, mutual-authentication, trust-root, hostname, validity,
CRL, key-log, and early-data policy. TLS policy failures occur before request
dispatch. It reads one bounded TTLV ResponseMessage frame and drops the raw
TLS connection after that response. Header tag/type, alignment, checked size
arithmetic, and the configured cap are validated before the response body
allocation. Expanded malformed-frame, truncation, surplus-frame, and boundary
tests remain assigned to T028/T029.

The exact request bytes are copied into the existing `SecretBuffer` owner
using fallible allocation. The owner and T025's observer test seam are on the
same production request path; the focused contract target observes wiping
after both success and post-write peer closure. KMIPKit zeroizes its owned
request and temporary response allocations. Caller-owned bytes and copies
inside rustls, AWS-LC, the OS, or other dependencies are outside that
guarantee.

## Verification

All commands ran offline on the Green source tree before the evidence commit:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls --offline` | 56 passed, 0 failed |
| `cargo test -p kmipkit-transport --test secret_redaction_current --offline` | 9 passed, 0 failed |
| `cargo test -p kmipkit-transport --test secret_redaction --offline` | 22 passed, 0 failed |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline` | 239 passed, 0 failed across 12 targets |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `git diff --check` | passed |

The package run included 54 library tests and integration-target counts of
6, 1, 1, 56, 12, 2, 22, 9, 55, 8, and 13. The raw TLS target includes the
T025 contract scenarios and source-included worker, resolver, secret, and
deadline tests. T021's two requested targets also passed independently.

## Files changed

- `crates/kmipkit-transport/src/raw_tls.rs` (new adapter and bounded response
  frame handling)
- `crates/kmipkit-transport/src/lib.rs` (module, re-export, and crate-doc
  update)
- `crates/kmipkit-transport/src/config.rs` (validated raw endpoint accessor)
- `crates/kmipkit-transport/src/resolver.rs` (optional absolute-deadline
  resolution path)
- `crates/kmipkit-transport/src/secret.rs` (fallible request-owner copy)
- `crates/kmipkit-transport/src/timeout.rs` (Tokio I/O forwarding through the
  existing deadline state)
- `crates/kmipkit-transport/src/timeout_tests.rs` and
  `crates/kmipkit-transport/tests/timeout_delivery.rs` (disambiguate existing
  Hyper trait calls after adding Tokio I/O traits)
- `crates/kmipkit-transport/src/tls.rs` (private rustls config handle accessor)
- `crates/kmipkit-transport/src/worker.rs` (crate-visible cancellation
  subscription)
- `crates/kmipkit-transport/tests/raw_tls.rs` (strict-Clippy-compatible
  assertions and fixture observations; T025 scenarios unchanged)

No dependency or KMIP wire-boundary change was made. T026 review is pending.
