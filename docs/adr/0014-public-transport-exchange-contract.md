# ADR-0014: Public Low-Level Transport Exchange Contract

**Status**: Accepted under delegated maintainer authorization on 2026-10-06.
**Date**: 2026-10-06
**Decision owner**: KMIPKit maintainer
**Related ADRs**: [ADR-0003](0003-layered-cargo-workspace.md), [ADR-0005](0005-transport-and-tls.md), [ADR-0012](0012-caller-requested-wire-encoding-policy.md)

## Context

The public `kmipkit-client` crate must call a synchronous exchange interface
owned by the separately public `kmipkit-transport` crate. A cross-crate Rust
trait or method is technically public even if it is hidden from generated
documentation. The client API must still reject arbitrary transport injection
and raw request input at `Client::execute`.

## Decision

1. `kmipkit-transport` exposes its bounded synchronous `Transport::exchange`
   contract as a documented low-level Rust API. Its contract accepts caller
   request bytes, a response limit, and reports delivery-aware transport
   failures. A successful response is returned as a public `TransportResponse`
   wrapper, not a plain `Vec<u8>`. The wrapper exposes a borrowed byte view,
   redacts `Debug`, and zeroizes the initialized byte range in its current
   owned allocation on drop. The API is part of the
   public crate's compatibility surface; it is not marked `doc(hidden)` or
   described as inaccessible.
2. The top-level `kmipkit` facade does not re-export the low-level transport
   trait. `kmipkit-client` exposes no constructor that accepts an arbitrary
   caller-implemented transport. Production clients are constructed only
   from KMIPKit's approved TLS/HTTPS configuration and adapters.
3. `Client::execute` accepts only the closed set of typed KMIP requests and
   creates its private encoding permit itself. The transport API does not
   provide an encoder and does not weaken the requirements on this typed
   client path in ADR-0012. Its separate caller-byte exception and limits are
   stated in decision 5 below.
4. The client passes exactly `CodecLimits::max_message_bytes()` to exchange
   and checks the returned buffer length before decode. Each production
   adapter must enforce the same limit during reads before growing response
   storage. Once response bytes have been stored, an adapter must either avoid
   reallocation of that response buffer or zeroize each previous/temporary
   allocation before release. The adapter's own specification must define the
   chosen strategy and include executable success- and error-path tests that
   prove it, including pre-allocation boundary tests.
5. The low-level raw exchange is an explicit exception to the typed
   `Client::execute` boundary. It performs no KMIP encoding, schema validation,
   or KMIPKit-owned request-buffer construction. A direct caller owns the
   supplied request buffer and is responsible for its validity and lifecycle;
   this API does not provide the `Client::execute` zeroizing-owner guarantee.
   Transport implementations must never log or format request bytes, must not
   retain them beyond the synchronous exchange, and must zeroize every
   KMIPKit-owned temporary request copy before release. Cleanup covers the
   initialized byte range in each current allocation; it does not cover spare
   or uninitialized capacity, an earlier allocation released by buffer growth
   unless that allocation was zeroized before release, caller-created copies,
   or copies owned by TLS, operating-system, or third-party transport
   libraries. Concrete adapter specifications must require tests for request
   nonlogging, nonretention, and cleanup of KMIPKit-owned temporary copies as
   applicable, and document external-copy limitations. If a KMIPKit-owned
   request copy grows after bytes have been stored, the adapter must prevent
   reallocation or zeroize each previous allocation before release. This
   exception does not allow the top-level facade or `Client::execute` to
   accept raw request data.
6. `TransportResponse` is the explicit low-level inbound raw-byte exception to
   ADR-0012 Decision 4. On success, ownership of the zeroizing wrapper passes
   to the direct caller; its byte view is borrowed, and dropping the wrapper
   zeroizes the initialized byte range in its current owned allocation before
   release. Spare or otherwise uninitialized capacity is outside this
   guarantee. It does not cover an earlier allocation released by `Vec`
   reallocation unless the adapter zeroized that allocation before release,
   caller-created copies, or copies owned by TLS, operating-system, or
   third-party transport libraries. On any error, every KMIPKit-owned partial
   or temporary response allocation must have its initialized byte range
   zeroized before release and before returning a payload-free, redacted
   `TransportError`; delivery state is preserved. The same limits apply to
   error cleanup: spare/uninitialized capacity and external copies are not
   covered. The high-level `Client::execute` path must decode the borrowed
   bytes into the validated TTLV/message model and drop the wrapper; it must
   not expose raw response bodies. Each concrete adapter specification must
   test successful wrapper cleanup, partial-read error cleanup/redaction, cap
   enforcement, and its chosen no-reallocation or prior/temporary-allocation
   cleanup strategy on both success and error paths. Tests must inspect only
   initialized ranges and must not claim cleanup of uninitialized capacity or
   external-library copies.

## Rationale and consequences

This honors ADR-0003's public reusable crate boundary without pretending
`pub` can be restricted to workspace friends. Direct Rust users may use the
low-level transport API to exchange bytes they supply themselves; it carries
no KMIP operation validation or public encoding capability. This is a
deliberate lower-level escape hatch, outside the typed client's validation and
request-owner/zeroization guarantee; callers retain buffer ownership and
responsibility. The returned response wrapper is zeroizing, while caller-made
copies remain caller-owned. Wrapper cleanup zeroizes the initialized bytes in
its current allocation; partial/error cleanup applies to initialized bytes in
each current KMIPKit-owned allocation. Spare capacity, prior allocations not
cleared before reallocation, and TLS/transport-library copies are outside that
guarantee. Concrete adapter specifications and tests must account for response
buffer growth explicitly. This is also a narrow exception to ADR-0012's inbound raw-body rule for direct
transport users; the higher-level client always decodes the wrapper and never
returns a raw response body. The higher-level client remains a typed API and
does not accept arbitrary transport implementations. The adapter must not
log request/response bytes or retain extra copies after the exchange returns.
On success, the response allocation transfers to the caller in
`TransportResponse`; its initialized bytes in the current allocation are
zeroized on drop under the limits above. Any other KMIPKit-owned temporary
copies must be zeroized before release.

Treating exchange as supported public API creates SemVer obligations for its
types and signatures. That cost is accepted to keep crate ownership and
dependency direction clear. API and documentation tests must ensure the
transport trait is visible from its crate, absent from the top-level facade,
and not accepted by public client constructors. Tests also verify response
wrapper redaction/zeroization and partial-buffer cleanup in concrete adapters.

## Authorization

The maintainer's direct instruction to execute the complete KMIPKit plan
autonomously and avoid further approval requests delegates authorization for
this boundary decision. The exact KMIPKIT-0007 and ADR artifact revisions,
independent reviews, and limitations are recorded in
`specs/007-client-execution/approval-record.md`. This does not waive human PR
review/merge or the qualified independent security review required before 1.0.
