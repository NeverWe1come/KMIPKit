# T028 Coverage Report — KMIPKIT-0013

## Scope and convergence

T028 test coverage commit: `ed2413c1d8b45ba7f33d8b7f6eeca3105d2298ab`.
The worktree started from the independently approved T026a/b and T027 range,
with QA approval recorded at `52cb4bde1846e0a33dbf9f2da826f419d12e862d`.

The audit found that T026 already implements the approved framing, response
limit, exact-read, cleanup, and one-shot connection behavior. The new matrix
passed against that code without a production change. I did not create an
artificial failing Red test or change production behavior to manufacture one.
T029 is therefore recorded as existing T026 Green behavior verified by this
matrix. T030 was reviewed after the matrix; T027's consuming
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
| Aligned declared length over cap | `raw_tls_rejects_an_aligned_response_length_over_the_limit_before_body_read` | Pass; header is rejected before body read/allocation by the existing validation order |
| Exact response cap and one byte over | `raw_tls_accepts_a_response_exactly_at_the_configured_limit`, `raw_tls_rejects_a_response_one_byte_over_the_configured_limit` | Pass |
| Truncated response value | `raw_tls_rejects_a_truncated_response_body_and_zeroizes_its_owner`; `raw_tls::response_buffer_zeroizes_partially_read_body_before_release` drives an actual `ResponseBuffer` through a partial async read error and observes its drop | Pass; `ResponseStarted`, body bytes observed before zeroization, full initialized owner zero afterward |
| Checked total-size overflow | `raw_tls_rejects_response_length_that_overflows_checked_total_size` is compiled only for 32-bit targets | Not executed on this 64-bit Windows host; the aligned over-cap path is exercised |
| Coalesced surplus frame | `raw_tls_returns_only_the_first_of_two_coalesced_response_frames` | Pass; only the first frame is returned and the peer observes connection close |
| Reconnect after malformed response | `raw_tls_reconnects_after_a_failed_response_without_replaying_the_first_request` | Pass; second call opens a new connection and each peer observes only its own request |

The test peer half-closes its write side after writing response bytes. This
provides deterministic EOF to malformed/truncated-frame tests while preserving
the read side so each test can observe that the adapter closes its connection.
No private-key/request sentinel is included in failure messages or diagnostics.

## Verification

All commands ran offline on the test coverage tree before the evidence commit:

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test raw_tls --offline` | 69 passed, 0 failed |
| `cargo test -p kmipkit-transport --test secret_redaction_current --offline` | 9 passed, 0 failed |
| `cargo test -p kmipkit-transport --test secret_redaction --offline` | 22 passed, 0 failed |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline --quiet` | 255 passed, 0 failed across 12 targets |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `git diff --check` | passed |

No cross-platform run was performed. Checked-size overflow remains a 32-bit
platform verification gap.

## Files changed

- `crates/kmipkit-transport/tests/raw_tls.rs`: response fixtures, deterministic
  framing/cap/EOF/coalescing/reconnect coverage, and bounded peer behavior.
- `crates/kmipkit-transport/src/raw_tls.rs`: a `cfg(test)`-only drop observer
  and partial-read regression for the existing response owner; no production
  code path changed.
