# T035a Red Report — KMIPKIT-0013

## Review finding and scope

Independent QA found that `exchange_on_worker` called `driver.abort()` only
after its async exchange body returned normally. If `ClientWorker::exchange`
canceled and dropped that future while sender readiness, request dispatch, or
response reading was pending, dropping Tokio's `JoinHandle` detached the
Hyper connection driver.

T035a starts a test-first correction. Red test and test-seam commit:
`e543c17c7cf486e9cde07f396d5a57d38c06dffd`
(`test(transport): cover canceled HTTPS driver cleanup (T035a Red)`). The
commit changes `tests/https.rs` and only `#[cfg(test)]` seams in `src/https.rs`;
it does not change the production code path.

## Regression design

`https_cancellation_aborts_the_hyper_connection_driver` starts a bounded
TLS 1.3/mTLS peer that captures the actual request and withholds its response.
After the peer signals request receipt, a per-adapter test-only channel
cancels the in-flight `ExchangeControl`. This forces the worker's cancellation
branch to drop the operation future. The adapter is returned to and retained
by the test thread, so worker/runtime shutdown cannot conceal driver cleanup.

A separate per-adapter observer reports only when the existing explicit
`driver.abort()` site executes. The test waits for that event with a bounded
channel receive; it does not infer cleanup from elapsed time or from the
Hyper task's possible natural completion. The test also asserts the request
was sent (`PossiblySent`) and that the peer actually captured it.

## Red verification

Focused command:

```text
cargo test -p kmipkit-transport --test https --offline -- --test-threads=1
```

Result: expected Red, 56 passed and 1 failed. The sole failure is
`https_cancellation_aborts_the_hyper_connection_driver`, at its fixed
assertion that the driver abort observer received no event after worker
cancellation. The request capture and `PossiblySent` assertions passed.

Formatting and patch checks:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

## Green correction

Green source commit:
`86dea2ea4526c4cf70e5f5fec781b2a1100ba2cf`
(`fix(transport): abort HTTPS driver on cancellation (T035a Green)`). It adds
`HyperDriverGuard`, which owns the Hyper driver's `JoinHandle` and invokes
`abort()` in `Drop`. Normal completion explicitly drops the guard before
returning; when worker cancellation drops `exchange_on_worker`, Rust drops
the guard and aborts the driver. The explicit abort observer is owned by this
guard in test builds, so the regression verifies the same cleanup path.

The test-only cancellation signal and abort observer remain scoped to each
adapter. The production path has no observer or additional runtime/task; it
uses the existing Hyper driver and worker.

## Green verification

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https --offline https_cancellation_aborts_the_hyper_connection_driver -- --exact --nocapture` | Passed, 1/1 |
| `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1` | Passed, 57/57 |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline` | Passed, 325 tests across 13 targets |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

The initial Green commands ran offline on the Windows host. The T021 symlink
fixture used its documented fallback because temporary symlink creation was
unavailable on this host; Windows symlink-following remains a platform
verification gap.

## Cleanup ordering correction — Red

Independent QA found that the abort-on-drop guard does not ensure the caller
waits for the Hyper driver to finish cleanup before the worker returns and
snapshots delivery state. `ExchangeCommand::run` can publish its cancellation
result before dropping the operation future, so the guard's `Drop` and driver
abort happen after the public exchange has returned. T035a Green is pending;
the previous Green commit is not sufficient for T010's cancellation
finalization ordering, and T037 remains blocked.

Red test/seam commit:
`44b8a96e65613b80f5d8d6a8c16f63929b40c5ae`
(`test(transport): gate HTTPS driver cleanup acknowledgement`). The new
`https_exchange_waits_for_driver_cleanup_acknowledgement_before_returning`
test cancels after the peer captures the request, gates the per-adapter
cleanup waiter after `JoinHandle::abort()` and before it joins the actual
Hyper task, then observes whether the public exchange returned while that
acknowledgment is held. The test seam is scoped to the source-included HTTPS
test adapter. The production cleanup behavior has not been corrected in this
Red commit.

QA rejected the initial Red assertion because it sampled the result channel
once immediately after cleanup started. Test correction commit
`deb4761a874bbd17a8ad3d296259ea8f992e4132` replaces that sample with an
ordered event stream from the real worker thread and the test-only Hyper
driver joiner. While the cleanup gate is closed, the test waits for an
exchange-return event or a bounded 250 ms observation interval; after
releasing the gate, it requires the join acknowledgment event to precede
public exchange return. This avoids treating an instantaneous empty channel
as proof of ordering. Independent QA approved the corrected test design.

Red verification:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https --offline https_exchange_waits_for_driver_cleanup_acknowledgement_before_returning -- --exact --nocapture` | Expected Red: 0/1 passed; failed at `the public exchange remains pending while driver cleanup acknowledgment is gated`. Compilation succeeded; the test observed the early return, released the gate, and joined the peer. |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

The next Green correction must make cancellation finalization await the
Hyper driver's cleanup acknowledgment/join before publishing or snapshotting
the delivery state, with a bounded cleanup wait. Do not start T037 until this
Green correction passes and independent QA approves it.
