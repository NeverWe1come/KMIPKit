# T031 Red Report — KMIPKIT-0013

## Scope and baseline

T031 Red source/test commit: `0d20f20c349494c4d6f97d0f7ef75f73a6510adf`.
Candidate-order assertion correction: `028fa66c765ee17b5b432f0d8001889eb2e321f2`.
Resolver-peer harness correction: `7ee433aa98627c43ad220714149d5f6bc5f9e526`.
Candidate-event assertion Red commit: `be4765fbce3b90dc1e3eadfd597ff1b559a4e367`.
Per-adapter observer seam support commit: `fbbb30ca59a15a8beb4ab78878c8b5ba61b00c95`.
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
retaining access to deterministic private test seams. At the original T031 Red
checkpoint, changes to `src/raw_tls.rs` were confined to `#[cfg(test)]` seams.
The T032 Green source commit below adds only the approved deadline and
sequential-candidate behavior. No public API changed. Request-byte assertions
use fixed messages and do not print test sentinels on failure.

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
| Ordered candidates and TLS-before-request behavior | `raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake` | Pass; observer sequence is failed untrusted candidate, successful trusted handshake, then exactly one dispatch for the trusted address; its peer receives the exact request. The untrusted peer is asserted only to have a failed handshake, not to have observed application bytes |
| Observer success/dispatch ordering on a verified candidate | `raw_tls_candidate_observer_records_success_before_request_dispatch` | Pass; per-adapter events are `HandshakeSucceeded(address)`, then `RequestDispatch(address)`; the peer receives the exact request |
| Total deadline includes lazy worker readiness from public entry | `raw_tls_total_deadline_covers_lazy_worker_readiness` | Pass; the exchange returns `NotSent` by the 120 ms absolute deadline while readiness is gated; the late-ready worker exits without running its runtime or resolver |
| Reject finite read/write phase durations outside `Instant` range | Adapter cases `raw_tls_rejects_an_unrepresentable_{read,write}_phase_before_dispatch`; lower-level `an_unrepresentable_finite_read_phase_is_rejected_as_invalid_input` | Pass; adapter rejects both read and write overflow as `Other`/`NotSent` before worker startup or dispatch; `PhaseDeadline` returns sanitized `InvalidInput` rather than silently dropping an overflowed finite deadline |

The adapter-level DNS tests use injected lookup gates rather than OS DNS. The
late-result case observes a result from the resolver closure through a bounded
channel after releasing its gate; only then does it probe the listener for a
bounded interval. The caller result is sampled before gate release, and the
request observer proves the worker cancellation path dropped the staged owner.
The uncanceled case gives the listener one accept owner during its probe, then
transfers it to the TLS peer after resolver release. The readiness test releases
its worker gate on every path and bounds peer/worker waits.

## Red verification (historical)

### Candidate event assertion and test-only observer seam

The assertion-first Red commit is `be4765fbce3b90dc1e3eadfd597ff1b559a4e367`.
Its focused command failed at compilation because the candidate observer type,
event enum, and test constructor did not yet exist; no production code changed
in that Red commit. The exact expected sequence is:

```text
HandshakeFailed(rejected_address)
HandshakeSucceeded(trusted_address)
RequestDispatch(trusted_address)
```

The separate support commit `fbbb30ca59a15a8beb4ab78878c8b5ba61b00c95` adds
only `#[cfg(test)]` state and hooks in `src/raw_tls.rs`. The observer is owned
by one adapter and uses its own `Arc<Mutex<...>>`; it stores only candidate
addresses and event kinds. It records a completed TLS result and records
`RequestDispatch` after `commit_dispatch()` succeeds, immediately before the
request writer call. There is no global observer and no non-test behavior
change. A passing success-path test exercises the handshake-success and
dispatch events through the real adapter.

The candidate regression now compiles and intentionally remains Red for T032.
It observes only `HandshakeFailed(rejected_address)`; the required success and
dispatch events for the trusted candidate are absent because the current
adapter stops at the first TLS failure. This directly proves that the rejected
candidate has no dispatch/write event while specifying one dispatch event for
the candidate whose handshake succeeds. The first test peer separately records
that its untrusted handshake failed; it is not described as observing
application bytes.

Final focused candidate command:

```text
cargo test -p kmipkit-transport --test raw_tls raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake --offline -- --exact
```

It compiled and failed at the intended event-sequence assertion. The actual
observer list contains only the failed first candidate; the trusted success
and dispatch events are missing:

```text
only a TLS-verified candidate reaches request dispatch
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
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_candidate_observer_records_success_before_request_dispatch --offline -- --exact` | 1 passed, 0 failed; records the real success-path handshake/dispatch sequence |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake --offline -- --exact` | Expected Red: first peer handshake-failure assertion passes; event list contains only `HandshakeFailed(rejected)` and lacks trusted success/dispatch |

Final serial target commands:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls --offline -- --test-threads=1` | 81 tests: 77 passed; 4 expected Red failures listed above |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | 56 tests: 55 passed; only the expected finite-phase `InvalidInput` failure |
| `cargo check -p kmipkit-transport --offline` | Passed; non-test build has no candidate observer code |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
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

## T032 Green verification

Source Green commit: `78c65fb68df09c9136383e25009328e746b9be41`.

Independent QA approved T032 with no blocking gap. The review caveat is that
`TransportCauseCategory` has no `InvalidInput` variant: the adapter represents
an unrepresentable finite phase as sanitized `Other`/`NotSent`, while the
lower-level `DeadlineIo` reports `io::ErrorKind::InvalidInput`.

`exchange_with_options` captures `exchange_started` at public entry and checks
the bounded connect, read, write, and total durations with
`Instant::checked_add` before lazy worker creation. The same absolute total
deadline bounds worker readiness. A worker that misses readiness is signaled
to shut down and detached without blocking the caller; if its startup gate is
released later, the closed ready receiver prevents it from entering
`run_runtime`. `PhaseDeadline` now preserves overflow as invalid input instead
of turning it into an absent/unbounded deadline, and the raw TLS adapter checks
phase representability again immediately before dispatch.

The resolver's ordered candidates are now attempted one at a time through TCP
connect and TLS handshake under the same connect/total deadline. A TLS failure
continues to the next candidate without dispatch. Dispatch still occurs once,
only after a verified handshake and a successful atomic dispatch commit; no
KMIP request is retried after dispatch. The per-adapter test observer confirms
`HandshakeFailed(rejected)`, `HandshakeSucceeded(trusted)`, and one
`RequestDispatch(trusted)` event, and the trusted peer receives the exact
caller bytes.

The existing `#[allow(clippy::too_many_arguments)]` on `connect_and_handshake`
is unconditional because the function signature gains a per-adapter observer
argument under `cfg(test)`. It is a lint-only annotation; it does not change
runtime behavior.

Focused Green commands:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_rejects_an_unrepresentable --offline` | 2 passed, 0 failed |
| `cargo test -p kmipkit-transport --test timeout_delivery an_unrepresentable_finite_read_phase_is_rejected_as_invalid_input --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_total_deadline_covers_lazy_worker_readiness --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --lib phase_deadline_helpers_cover_absent_overflow_and_expired_deadlines --offline` | 1 passed, 0 failed |

Final verification:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls --offline -- --test-threads=1` | 81 passed, 0 failed |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | 56 passed, 0 failed |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline --quiet -- --test-threads=1` | Exit 0; all 268 tests passed across 12 test binaries |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo check -p kmipkit-transport --offline` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |

The source commit does not change public API or TLS policy. The T031 cases
previously recorded as Red now pass against T032 Green. No cross-platform run
was performed for this checkpoint.

## T033 Refactor verification

Source Refactor commit: `977db24677c30b46857fc68f4a95fbc13e5a703b`.

Independent QA approved the Refactor source commit with no findings.

The refactor removes an unnecessary `Option::map` plus nested
`Option<Result<...>>` match from `exchange_with_options`. After lazy worker
startup succeeds, it extracts the worker with `ok_or_else`, retaining the same
sanitized impossible-`None`/`NotSent` fallback, then directly maps
`worker.exchange` errors through `worker_error`. The identical captured
`total_deadline` is passed through. No connection selection, handshake,
dispatch, request bytes, retry, or delivery-state behavior changed.

Baseline commands before editing:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_tries_tls_candidates_in_order_and_writes_only_after_a_valid_handshake --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_total_deadline_covers_lazy_worker_readiness --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_reconnects_after_a_failed_response_without_replaying_the_first_request --offline -- --exact` | 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls --offline -- --test-threads=1` | 81 passed, 0 failed |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline -- --test-threads=1` | 56 passed, 0 failed |

Post-refactor focused and T031 commands produced the same passing results:
each focused case passed 1/1, and the serial `raw_tls` and `timeout_delivery`
targets passed 81/81 and 56/56.

Final Refactor verification:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline --quiet -- --test-threads=1` | Exit 0; all 268 tests passed across 12 test binaries |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo check -p kmipkit-transport --offline` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |
