# T028 Coverage Report — KMIPKIT-0013

## Scope and convergence

T028 test coverage commit: `ed2413c1d8b45ba7f33d8b7f6eeca3105d2298ab`.
The worktree started from the independently approved T026a/b and T027 range,
with QA approval recorded at `52cb4bde1846e0a33dbf9f2da826f419d12e862d`.

QA then found that the over-cap test proved rejection but did not directly
assert the response owner was never constructed. The correction adds a
per-adapter, `cfg(test)` allocation-attempt observer. At the response owner
constructor boundary, the observer records an attempted allocation and returns
a test error before reservation; this makes a cap-order regression fail
without risking a large allocation. The existing production cap check remains
before that constructor call, and no production behavior or public API changed.

The audit found that T026 already implements the approved framing, response
limit, exact-read, cleanup, and one-shot connection behavior. The initial
matrix passed against that code without a production change, so no artificial
Red was introduced for those already implemented behaviors. The separate
allocation-order assertion did have a missing test seam and its intentional
Red is recorded below. T029 is therefore recorded as existing T026 Green
behavior verified by this matrix. T030 was reviewed after the matrix; T027's consuming
`RawTlsConnection` already keeps the frame state and one-shot stream lifetime
small, so no further source refactor was justified.

## Matrix

| Contract case | Test/evidence | Result |
| --- | --- | --- |
| EOF before any response header | `raw_tls_zeroizes_staged_request_after_peer_closes_without_response` asserts `PossiblySent` | Pass |
| Partial 8-byte response header | `raw_tls_rejects_a_partial_response_header` | Pass; `ResponseStarted` |
| EOF after a full header but before its value | `raw_tls_rejects_eof_after_a_complete_header_before_the_value` | Pass; `ResponseStarted` |
| Invalid root tag and root type | `raw_tls_rejects_a_response_with_an_invalid_root_tag`, `raw_tls_rejects_a_response_with_an_invalid_root_type` | Pass; `ResponseStarted` |
| Unaligned declared value length | `raw_tls_rejects_an_unaligned_response_length` | Pass; `ResponseStarted` |
| Aligned declared length over cap | `raw_tls_rejects_an_aligned_response_length_over_the_limit_before_body_read` | Pass; adapter-scoped observer reports zero response-buffer constructor attempts |
| Exact response cap and one byte over | `raw_tls_accepts_a_response_exactly_at_the_configured_limit`, `raw_tls_rejects_a_response_one_byte_over_the_configured_limit` | Pass |
| Truncated response value | `raw_tls_rejects_a_truncated_response_body_and_zeroizes_its_owner`; `raw_tls::response_buffer_zeroizes_partially_read_body_before_release` drives an actual `ResponseBuffer` through a partial async read error and observes its drop | Pass; `ResponseStarted`, body bytes observed before zeroization, full initialized owner zero afterward |
| Checked total-size overflow | `raw_tls_rejects_response_length_that_overflows_checked_total_size` is compiled only for 32-bit targets | Not executed on this 64-bit Windows host; the aligned over-cap path is exercised |
| Coalesced surplus frame | `raw_tls_returns_only_the_first_of_two_coalesced_response_frames` | Pass; only the first frame is returned and the peer observes connection close |
| Reconnect after malformed response | `raw_tls_reconnects_after_a_failed_response_without_replaying_the_first_request` | Pass; second call opens a new connection and each peer observes only its own request |
| Observer isolation and activation | `raw_tls_response_allocation_observer_is_scoped_to_its_adapter` | Pass; an observed valid-frame attempt is intercepted/counts once, while a separate adapter succeeds without changing the observer |

## Allocation-order correction — Red/Green

Red test commit: `0347fc321c8af329234b04fe5f9088a24e78d792`.
The strengthened over-cap regression constructs an observer for its adapter
and requires the attempted response-buffer allocation count to remain zero.
The focused target exited 1 at compilation, as expected, because the
test-only `ResponseAllocationObserver` and its adapter constructor were not
implemented yet:

```text
cargo test -p kmipkit-transport --test raw_tls raw_tls_rejects_an_aligned_response_length_over_the_limit_before_body_read --offline
exit status: 1 (expected missing test observer seam)
```

```text
error[E0433]: cannot find `ResponseAllocationObserver` in `raw_tls`
error[E0425]: cannot find function `new_for_test_with_response_allocation_observer` in module `raw_tls`
```

No production code changed in Red. Green commits are
`55ea8404b344f8ba43da5cb8518afcd2f160d19c` and the strict-Clippy follow-up
`64f78df301ecd8214d62940283107ed65a620b2b`. The observer is held by the test
adapter, carried only through that exchange, and counts at the bounded owner
constructor seam. If the adapter reaches the seam, the observer stops before
reservation. The over-cap test passes with count zero. A separate scoped
observer test verifies an observed valid-frame attempt increments the count,
returns before reservation, and does not affect an adapter without the
observer.

The first Green package run also exposed a test-only dead-code warning for
observer methods used only by the source-including integration target; a
narrow, reasoned allowance was added on that observer implementation. Strict
Clippy then flagged the observer passed by value; it now borrows the
per-adapter counter. Final strict Clippy passes.

Focused Red/Green verification:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_rejects_an_aligned_response_length_over_the_limit_before_body_read --offline` | Green: 1 passed, 0 failed |
| `cargo test -p kmipkit-transport --test raw_tls raw_tls_response_allocation_observer_is_scoped_to_its_adapter --offline` | 1 passed, 0 failed |

The test peer half-closes its write side after writing response bytes. This
provides deterministic EOF to malformed/truncated-frame tests while preserving
the read side so each test can observe that the adapter closes its connection.
No private-key/request sentinel is included in failure messages or diagnostics.

## Verification

All commands ran offline on the test coverage tree before the evidence commit:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls --offline` | 70 passed, 0 failed |
| `cargo test -p kmipkit-transport --test secret_redaction_current --offline` | 9 passed, 0 failed |
| `cargo test -p kmipkit-transport --test secret_redaction --offline` | 22 passed, 0 failed |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline --quiet` | 256 passed, 0 failed across 12 targets |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `git diff --check` | passed |

No cross-platform run was performed. Checked-size overflow remains a 32-bit
platform verification gap.

One earlier all-target run intermittently failed
`resolver::tests::system_resolver_instances_share_one_governor` with 22 rather
than 32 available permits while other source-included resolver tests were
active. Its focused rerun passed, and a subsequent full all-target run passed
all 256 tests. No resolver implementation or test was changed for this
observer correction.

## Files changed

- `crates/kmipkit-transport/tests/raw_tls.rs`: response fixtures, deterministic
  framing/cap/EOF/coalescing/reconnect coverage, and the allocation-order
  assertion.
- `crates/kmipkit-transport/src/raw_tls.rs`: `cfg(test)`-only response drop
  and allocation observers; no production code path changed.

## Independent QA review

QA re-reviewed the allocation-order correction range
`3691b3ca2da82c8cfc573154704509a4ebf6f656..64f78df301ecd8214d62940283107ed65a620b2b`
and approved it with no actionable findings. This covers the test-only Red,
per-adapter Green observer, and strict-Clippy follow-up.
