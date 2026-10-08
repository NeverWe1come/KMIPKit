# T038 Green Report — KMIPKIT-0013

## Scope and implementation

T038 implements the approved HTTPS response contract in
`specs/013-production-transport/spec.md` FR-010, FR-013, FR-014, and FR-015
and `specs/013-production-transport/contracts/https-ttlv.md`. T037 remains the
Red specification and its approved tests were not weakened.

The HTTPS adapter now:

- Requires status 200, exactly one `Content-Type` whose media type is
  `application/octet-stream`, and exactly one unambiguous decimal
  `Content-Length`; it rejects `Transfer-Encoding`, `Content-Encoding`, and
  invalid or duplicate framing/response headers.
- Configures Hyper HTTP/1 with `max_headers(64)`, `max_header_size(64 * 1024)`,
  and `max_buf_size(64 * 1024)`. The two byte limits are both necessary:
  `max_header_size` caps the complete status line and header section, while
  `max_buf_size` bounds the connection buffer.
- Checks the declared response length against the caller's cap before creating
  or growing the KMIPKit response buffer, reserves only that validated length,
  and streams body data through the bounded owner. `ResponseBuffer` zeroizes
  initialized bytes on every error path.
- Reuses a healthy HTTP/1 TLS session after a successful exchange. The idle
  driver rejects unsolicited bytes and invalidates that session; response,
  framing, I/O, or cancellation failures drop and invalidate it. Each exchange
  installs its own deadlines and delivery control, and the request is never
  automatically replayed.
- Preserves driver cancellation and join cleanup registration through
  `ExchangeControl`; cleanup completes before the worker publishes a finalized
  result or admits later work, as covered by the existing cleanup tests.

The production-only Green change is commit `ca8a320`:

```text
ca8a320 feat(transport): enforce bounded reusable HTTPS responses
```

Strict Clippy exposed four lints in the newly added T037 surplus-response test
harness. The minimal lint-only reference/closure cleanup is isolated in commit
`98c2ace` and preserves its assertions and behavior:

```text
98c2ace test(transport): clean up HTTPS surplus test lints
```

No task ledger entry was changed; T038 remains unchecked for independent QA.

## Verification

The first focused Green run passed 65 tests and failed only the 65,537-byte
parser-boundary case: `max_buf_size` alone allowed Hyper to parse the larger
header incrementally. Adding `max_header_size(64 * 1024)` fixed that case while
retaining `max_buf_size(64 * 1024)`; the final focused run below passes 66/66.

| Command | Result |
| --- | --- |
| `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1` | Passed: 66 tests |
| `cargo test -p kmipkit-transport --test timeout_delivery --offline` | Passed: 56 tests |
| `cargo test -p kmipkit-transport --lib worker::tests --offline` | Passed: 24 tests |
| `cargo test -p kmipkit-transport --all-targets --all-features --offline -- --test-threads=1` | Passed: 334 tests across all package targets |
| `cargo clippy -p kmipkit-transport --all-targets --all-features --offline -- -D warnings` | Passed with no warnings |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

The all-targets package run passed 56 library tests, 6 delivery-state tests,
1 dependency-policy test, 66 HTTPS tests, 1 public-contract test, 81 raw-TLS
tests, 13 resolver tests, 2 runtime-policy tests, 22 secret-redaction tests,
9 current secret-redaction tests, 56 timeout-delivery tests, 8 TLS-config
tests, and 13 TLS-policy tests.

An earlier all-targets run without serial test execution,
`cargo test -p kmipkit-transport --all-targets --all-features --offline`, had
two concurrency-sensitive failures: the shared resolver governor observed 31
active jobs where the test expected 32, and a raw-TLS handshake fixture failed
during parallel setup. The serial replay with the same package/features and
test matrix, recorded in the table as
`cargo test -p kmipkit-transport --all-targets --all-features --offline -- --test-threads=1`,
passed all 334 tests. The deterministic serial result is the package
acceptance evidence; the earlier parallel-run contention is recorded here for
completeness.

The focused HTTPS run confirms that the complete 65,536-byte parser boundary
is accepted and 65,537 bytes are rejected, the declared over-limit body is
rejected before body arrival or KMIPKit buffer allocation, valid HTTPS
exchanges reuse a connection, and a queued surplus response cannot satisfy the
next exchange. All 66 focused tests pass, including cancellation, cleanup,
delivery-state, and partial-response zeroization coverage.

## T037 report correction

The Hyper parser explanation in `task-37-report.md` was corrected to distinguish
`max_buf_size` from `max_header_size`. The Green configuration and passing
boundary result are recorded there; the original T037 Red evidence remains
intact.
