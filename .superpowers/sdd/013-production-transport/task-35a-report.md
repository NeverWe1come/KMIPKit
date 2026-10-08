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

T035a Green must move driver ownership into a cancellation-safe abort-on-drop
guard and make this test pass. T037 remains paused until Green and independent
read-only re-review.
