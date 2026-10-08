# T037 Red Report — KMIPKIT-0013

## Scope

T037 adds HTTPS response regressions for the accepted client-side rules in
`specs/013-production-transport/spec.md` FR-010, FR-013, FR-014, and FR-015
and `specs/013-production-transport/contracts/https-ttlv.md`. These tests do
not claim server conformance to OASIS KMIP Profiles 2.1 §5.3.1. Hyper owns
HTTP/1 parsing; KMIPKit owns the stricter response policy and bounded body
ownership. The selected request contract references RFC 9112 §3.2 for `Host`.

All peers use ephemeral TLS 1.3 mutual authentication and exchange real
HTTP/1.1 bytes with the adapter. The production HTTPS behavior was not edited.
`https.rs` changes in this Red step are test-only seams under `#[cfg(test)]`:
they attach a per-exchange response-buffer observer and report allocation
requests and the initialized range observed after the existing `Drop` has
zeroized it. With test configuration disabled, `ResponseBuffer` remains the
original `ResponseBuffer(Vec<u8>)` and its append/drop behavior is unchanged.

## Regression coverage

- Invalid responses: non-200 status; absent `Content-Type` and
  `Content-Length`; duplicate content type/length; conflicting lengths;
  unsupported media type; malformed and negative lengths; transfer encoding,
  transfer encoding combined with content length, and content encoding. The
  missing-length case is a valid HTTP/1.1 close-delimited response with
  `Connection: close`; its peer flushes rustls `close_notify`, and the harness
  asserts that shutdown completed cleanly.
- Parser and framing errors: malformed status/header syntax, malformed chunk
  framing, incomplete headers, and a fixed-length body truncated at EOF. The
  error must retain `ResponseStarted` when response bytes were observed.
- Parser boundaries: 64 headers and a complete 64 KiB header block are accepted;
  the 65th header and a 65,537-byte block are rejected.
- Body bounds: a body exactly at the configured cap is returned unchanged.
  For a declared body one byte over the cap, the TLS peer gates all body bytes
  after sending the response headers. The adapter must reject during this
  250 ms observation window and must not request KMIPKit response-buffer
  capacity. The peer then releases the body so the test can finish cleanly if
  the current implementation did not reject from the header.
- Cleanup: after a truncated body yields initialized partial bytes, the test
  observes the same `ResponseBuffer` drop and verifies that its initialized
  range is zero.
- Connection reuse and surplus response: the peer returns the first and
  second responses, then waits for the test to complete both exchanges before
  writing an unsolicited third response. It records the TLS connection used
  for each actual HTTP request, requires the first two exchanges to share that
  connection, requires the surplus write to succeed on it, and checks that a
  third exchange never returns the unsolicited body.

The reuse regression intentionally does not skip or weaken its same-connection
assertion. The current adapter creates and closes a connection in each
`exchange_on_worker` call. It therefore cannot satisfy this setup: the first
two completed calls use different TLS sockets, and the test peer cannot queue
the surplus response on the closed second socket. Persistent HTTPS reuse and
surplus invalidation remain a Green implementation gap for T038/T039.

## Hyper/parser review

The locked dependency is Hyper 1.12.0. Its HTTP/1 connection `Builder` exposes
`max_headers` and `max_buf_size`; its documented defaults are 100 headers and
approximately 400 KiB, respectively. The configured methods can enforce the
contract's 64-header and 64 KiB limits. Hyper also exposes a separate
`max_header_size`, but the contract specifies the input-buffer limit, so the
boundary tests target `max_buf_size` semantics rather than inventing another
policy. The parser itself rejects the malformed status/header syntax, malformed
chunk body, truncation, malformed length, and conflicting duplicate length
cases exercised here. It accepts valid close-delimited responses without a
`Content-Length`; rejecting that missing header is KMIPKit response policy,
which the current adapter does not implement. Hyper accepts the exact parser
boundary inputs and, with current defaults, accepts the over-boundary inputs.

## Red verification

Command:

```text
cargo test -p kmipkit-transport --test https --offline -- --test-threads=1
```

Result: compilation succeeded without warnings; 61 passed and 5 failed. Each
failure is a product assertion, not a compile failure or test-harness timeout:

1. `https_rejects_invalid_status_response_headers_and_encodings` reports that
   the adapter accepts these nine responses: non-200 status, missing media
   type, duplicate content type, duplicate identical content length, invalid
   media type, transfer encoding, transfer encoding plus content length,
   content encoding, and duplicate content encoding. Hyper rejects malformed,
   negative, and conflicting content lengths.
2. `https_enforces_64_headers_and_64_kibibyte_parser_input_boundary` reports
   acceptance of 65 headers and 65,537 bytes. The exact 64-header and 65,536
   byte inputs are accepted, as required.
3. `https_never_attributes_a_surplus_response_to_a_later_exchange` completes
   two valid calls before releasing the peer's surplus-response gate, then
   fails because the requests used different TLS connection IDs and the
   surplus could not be written to the second connection. The third exchange
   still runs and checks that the body is not attributed to it.
4. `https_rejects_close_delimited_response_without_content_length` sends a
   valid `Connection: close` response without `Content-Length` and completes a
   clean TLS shutdown. The peer assertion confirms `close_notify` was flushed;
   the product assertion fails because the adapter returns the close-delimited
   body instead of rejecting the absent header.
5. `https_rejects_declared_oversize_before_body_arrives_or_response_buffer_grows`
   confirms the current code rejects the eventual over-cap body with
   `ResponseStarted` and requests zero response-buffer capacity. Its product
   failure is that it remains pending while the body is deliberately withheld
   and does not reject the declared size from `Content-Length` in the
   observation window.

Expected Red coverage passed in this run: parser syntax/framing and truncation,
partial-buffer zeroization, exact 64-header/64 KiB inputs, and exact response
cap. These passing cases are retained to specify behavior Hyper or the current
buffer already provides.

Focused QA correction command:

```text
cargo test -p kmipkit-transport --test https --offline https_rejects_close_delimited_response_without_content_length -- --exact --nocapture
```

Result: compilation succeeded; expected Red, 0 passed and 1 failed. The peer
assertions for the captured request and flushed TLS `close_notify` completed;
the sole failure is the product assertion that a valid close-delimited body
without `Content-Length` must be rejected. The adapter returned that body.

## Over-limit harness correction

The over-limit test now retains the result returned by its 250 ms pre-body
observation. After releasing the gated body, it uses that result if one was
already received and waits on the result channel only when the observation
timed out. This preserves the assertion that the declared size is rejected
before any body bytes arrive and avoids losing a valid early rejection by
waiting for a second send on a one-result channel.

Focused harness verification command, run with the coordinator's uncommitted
T038 Green work in `src/https.rs` and `src/timeout.rs` present:

```text
cargo test -p kmipkit-transport --test https --offline https_rejects_declared_oversize_before_body_arrives_or_response_buffer_grows -- --exact --nocapture
```

Result: compilation succeeded; 1 passed, 0 failed, 65 filtered out. The
declared over-limit response was rejected during the pre-body observation,
and the existing zero-allocation/capacity assertions passed. This run verifies
that the test harness preserves an early result under the Green WIP; it does
not replace the Red-baseline run, which the coordinator will verify separately.

Formatting and patch checks:

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | Passed |
| `git diff --check` | Passed |

## Independent review

The first QA review found that the missing-`Content-Length` case could fail on
an abrupt TLS EOF instead of the HTTP policy. Correction commit
`3f8a834cde9d0660e093bce3d4a7243877b90c42` added a peer that sends and flushes
`close_notify`; independent QA then approved T037 Red. The full focused suite
reproduced 61 passed and 5 expected product failures, including the clean
close-delimited response that the current adapter incorrectly accepts.

## Oversize test harness correction

The first T038 Green run exposed a test-harness issue in the over-cap early
rejection case: it consumed the exchange's only result when rejection arrived
inside the 250 ms observation window, then waited for a second result after
releasing the gated body. Commit `110c98b5f7310379eb15359198f03a3553ff0861`
retains that early result and waits for one only when the observation window
expires. It preserves the requirement to reject from `Content-Length` before
body bytes arrive.

With the Green source changes temporarily stashed, the corrected baseline
command `cargo test -p kmipkit-transport --test https --offline -- --test-threads=1`
compiled and reproduced 61 passed / 5 failed. All five failures are product
assertions: response policy, parser limits, reused connection/surplus handling,
missing `Content-Length`, and early over-cap rejection. The isolated over-cap
test passes under the T038 Green work in progress. `cargo fmt --all --check`
and `git diff --check` passed. Independent QA approved the corrected peer and
channel behavior; T037 Red is checked in the task ledger.
