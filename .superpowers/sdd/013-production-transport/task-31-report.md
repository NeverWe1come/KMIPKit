# T031 Red Report — KMIPKIT-0013

## Scope and baseline

T031 Red source/test commit: `0d20f20c349494c4d6f97d0f7ef75f73a6510adf`.
Candidate-order assertion correction: `028fa66c765ee17b5b432f0d8001889eb2e321f2`.
Resolver-peer harness correction: `7ee433aa98627c43ad220714149d5f6bc5f9e526`.
The worktree started at `cf2085c23c68accbef1c0e66cc5b65cbd5a5f965`, after
T028's QA result was recorded.

Baseline commands passed before adding T031 coverage:

| Command | Baseline result |
| --- | --- |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline` | 55 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls --offline` | 70 passed, 0 failed |
| `cargo test -p kmipkit-transport --lib worker:: --offline` | 24 passed, 0 failed |
| `cargo test -p kmipkit-transport --lib resolver:: --offline` | 13 passed, 0 failed |

The adapter-level cases live in the existing `tests/raw_tls.rs` source-including
target so they call the real `RawTlsTransport` `Transport::exchange` path while
retaining access to deterministic private test seams. The only change to
`src/raw_tls.rs` is behind `#[cfg(test)]`: per-adapter resolver injection and a
worker-spawner gate for the lazy-readiness case. No non-test behavior or public
API changed. Request-byte assertions use fixed messages and do not print test
sentinels on failure.

## Coverage matrix

| Requirement | Test/evidence | Result on current Green |
| --- | --- | --- |
| Connect timeout includes TLS handshake and prevents dispatch | `raw_tls_connect_deadline_covers_tls_handshake_before_dispatch` | Pass; `NotSent`, peer sees no KMIP bytes |
| Write timeout after partial request write | `raw_tls_write_timeout_zeroizes_a_partially_written_request` | Pass; `PossiblySent`, only a prefix reaches peer, complete staged owner is zeroized |
| Read timeout cleans staged request | `raw_tls_read_timeout_zeroizes_the_staged_request` | Pass; `PossiblySent`, owner is zeroized |
| Absolute total deadline while waiting for a response | `raw_tls_total_deadline_remains_absolute_while_waiting_for_response` | Pass; `PossiblySent`, owner is zeroized |
| Dispatch/cancel linearization and delivery-state transitions | Existing `cancellation_and_dispatch_race_has_one_delivery_state_winner`, `cancellation_before_dispatch_makes_a_later_commit_impossible`, `delivery_states_advance_only_at_dispatch_and_first_response_byte`, and worker equivalents | Pass |
| Invalidation, later reconnect, and no replay | `raw_tls_reconnects_after_a_failed_response_without_replaying_the_first_request` | Pass; each connection receives only its own request |
| Deadline cancellation during pending DNS; late-result isolation and owner cleanup | `raw_tls_connect_deadline_cancels_pending_dns_and_zeroizes_the_staged_request` | Pass; the caller receives timeout/`NotSent` while the resolver gate is still closed, proving worker cancellation; the staged owner is zeroized. After gate release, a completion channel confirms the resolver closure returned, then a bounded listener probe sees no late TCP connection |
| No TCP connection while an uncanceled lookup is pending | `raw_tls_has_no_tcp_side_effect_while_an_uncanceled_lookup_is_pending` | Pass; the test thread is the only accept owner during a 100 ms resolver gate probe; it transfers the listener to the TLS peer only after releasing DNS |
| Ordered candidates and TLS-before-request behavior | `raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake` | Red: the first server records a TLS handshake failure for its untrusted certificate; the test does not claim that peer observed application bytes. The expected next-candidate accept assertion fails; following assertions require the trusted peer to complete TLS and receive the exact request |
| Total deadline includes lazy worker readiness from public entry | `raw_tls_total_deadline_covers_lazy_worker_readiness` | Red: public `exchange` remains blocked beyond the 120 ms total duration while readiness is gated |
| Reject finite read/write phase durations outside `Instant` range | Adapter cases `raw_tls_rejects_an_unrepresentable_{read,write}_phase_before_dispatch`; lower-level `an_unrepresentable_finite_read_phase_is_rejected_as_invalid_input` | Red: adapter reports `PossiblySent` instead of `NotSent`; lower-level test reaches its 100 ms outer timeout instead of `InvalidInput` |

The adapter-level DNS tests use injected lookup gates rather than OS DNS. The
late-result case observes a result from the resolver closure through a bounded
channel after releasing its gate; only then does it probe the listener for a
bounded interval. The caller result is sampled before gate release, and the
request observer proves the worker cancellation path dropped the staged owner.
The uncanceled case gives the listener one accept owner during its probe, then
transfers it to the TLS peer after resolver release. The readiness test releases
its worker gate on every path and bounds peer/worker waits.

## Red verification

Final focused candidate command:

```text
cargo test -p kmipkit-transport --test raw_tls raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake --offline -- --exact
```

It compiled and failed at the intended assertion:

```text
the next resolver candidate is attempted after TLS rejection
```

The peer reports an actual handshake failure for the first untrusted
certificate before this assertion. The test contains subsequent assertions for
the trusted peer's completed handshake and exact request bytes, which remain
unreached in Red because the adapter does not attempt that candidate.

Focused DNS/candidate reruns after the harness correction:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_connect_deadline_cancels_pending_dns_and_zeroizes_the_staged_request --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_has_no_tcp_side_effect_while_an_uncanceled_lookup_is_pending --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake --offline -- --exact` | Expected Red: first handshake-failure assertion passes; next-candidate accept assertion fails |

Final serial target commands:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls --offline -- --test-threads=1` | 80 tests: 76 passed; 4 expected Red failures listed above |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | 56 tests: 55 passed; only the expected finite-phase `InvalidInput` failure |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |

The raw-TLS failures are confined to the T032 gaps: non-representable read and
write phases are treated as unbounded, worker readiness is not bounded by the
public total deadline, and TLS failure on the first connected address prevents
trying the next resolver candidate. The remaining raw-TLS, resolver, timeout,
and worker tests passed, including the shared-governor tests in serial runs.

One earlier default-parallel raw-TLS run reported a shared-governor stress-test
failure and then did not complete promptly. The final serial target runs passed
both shared-governor cases; the source of the parallel-only result is
undetermined, so no claim is made that it is a product defect or resolved
flakiness.

No Green implementation was made in this task checkpoint. T032 owns the
behavior changes for the three remaining behavior gaps: phase-duration
validation, readiness bounded by the total deadline, and continuing to the
next resolver candidate after a TLS handshake rejection.
