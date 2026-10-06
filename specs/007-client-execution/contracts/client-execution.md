# Client Execution Contract (Draft)

## Caller boundary

`Client::execute` accepts a closed typed KMIP request/batch. The initial operation is Discover Versions. The API has no argument for generic TTLV items, pre-encoded bytes, a serialization callback, or a downstream-implemented request conversion trait. It is synchronous and performs one exchange per call.

## Request preparation

1. Validate typed operation payloads, message cardinality, IDs, protocol version, and common option values.
2. Preserve an optional caller-supplied request Time Stamp Date-Time exactly when present; omit it when absent. Do not include it in diagnostics, error formatting, or exposed error-source chains, and do not log it. Do not synthesize one or expose a countdown-timer source; countdown-derived outgoing values are outside this feature's scope under OD-004.
3. Omit Server Correlation Value from client-initiated requests. Client Correlation Value, if present, is never a per-item match key.
4. Omit peer-visible Maximum Response Size for Discover Versions, which is not classified as likely-large.
5. Borrow the caller-provided `CodecLimits`; use that exact same reference for bounded encoding and response decoding, without cloning or reconstructing it.
6. Mint the private execute-owned permit and call the one private production writer site.
7. Keep the writer's zeroizing owner alive until the transport exchange returns.

## Transport

- One exchange can have partial writes and incremental reads.
- Before any write, delivery is `NotSent`.
- After any byte may have been written, delivery is `PossiblySent`.
- After any response byte begins, delivery is `ResponseStarted`.
- The transport receives the configured response-byte cap and must stop reads before allocating or retaining bytes beyond it. Dropping `TransportResponse` zeroizes the initialized byte range in its current owned allocation; on failure, initialized bytes in every current KMIPKit-owned partial/temporary response allocation are zeroized before release. Spare/uninitialized capacity, earlier allocations released by reallocation unless cleared before release, caller copies, and external TLS/transport-library copies are outside that guarantee. Each concrete adapter specification must test either no response-buffer reallocation after bytes are stored or cleanup of every previous/temporary allocation, on success and error paths. The client checks the borrowed response length before decode and then drops the wrapper.
- The public low-level `kmipkit-transport::Transport::exchange` accepts caller-supplied request bytes and returns a raw-response wrapper under ADR-0014, as a narrow exception to ADR-0012 Decision 4. Direct callers own the request buffer and successful response wrapper; partial response allocations are cleared as described above before payload-free errors. The API provides no KMIP validation or typed-client guarantee. Transport implementations must not log or retain caller request bytes beyond exchange and must zeroize each KMIPKit-owned temporary request copy before release, including any prior allocation released as that copy grows. Concrete adapter specifications must require applicable request nonlogging, nonretention, and temporary-copy cleanup tests and document external-library-copy limitations.
- The 0007 fake is tested directly through this low-level API with a unique request sentinel: captured logs and fake-retained state must not contain it after success or error, and a test-only observer must confirm every initialized byte in a KMIPKit-owned temporary request copy is zero before release on both paths. This verifies the fake contract only; each concrete adapter specification owns equivalent applicable tests for its implementation.
- Errors do not automatically retry, resubmit, fail over, poll, or wait.

## Response validation

- The complete message must be structurally valid under the 0005 decoder and 0006 model.
- Protocol Version is exactly 2.1 under ADR-0002.
- Discover Versions response versions are preserved in received order, including repeated equal offered versions. §6.1.16 Table 212 marks the Protocol Version response field repeatable and states no uniqueness rule; reject only malformed entries or versions outside the offered intersection.
- Response operation and item identities must match the request. Multi-item response order may vary and IDs drive association. A supplied ID is echoed; single-item requests may omit the ID.
- Pending is accepted only if the request option permits it under known semantics and the response contains the required Asynchronous Correlation Value from the accepted 0006 model. Preserve its opaque bytes unchanged through the `Client::execute` result for later explicit operations. Unknown extension-range indicator values do not grant Pending permission without an explicit registry policy. Under `KMIPKIT-DEC-002`, outbound Batch Error Continuation accepts assigned values only for this slice; generic decoding preserves all raw Enumeration values. This project policy does not claim that Table 435's extension allocation is invalid under OASIS. The separate `KMIPKIT-DISC-001` remains open for Continue/Undo execution effects.
- Treat the Asynchronous Correlation Value as capability-like sensitive metadata: expose it only through an explicit borrowed accessor, redact it from formatted output, errors, and logs, and retain KMIPKit-owned bytes in zeroizing storage until drop without an ordinary unzeroized duplicate.
- Unrecognized critical Message Extensions fail the complete response as required by OASIS §9.13. Unknown non-critical extension TTLV is preserved as KMIPKit policy; OASIS permits processing it as absent.
- Result and payload errors redact all request/response bytes and secret material.

## Source metadata

Expose only safe typed cause categories and delivery state through `ClientError`/`TransportError`. If an error-source chain is exposed, every link must be a safe typed KMIPKit error; do not retain or display arbitrary original transport/library error text, which may carry untrusted or sensitive payloads.

`TransportResponse` is the only direct low-level raw-response return type.
External transport implementers construct it with the documented public
`TransportResponse::new(Vec<u8>)`, which moves the vector into zeroizing-owned
storage; `as_bytes(&self) -> &[u8]` provides a borrowed view. Its `Debug`
output is redacted, and it zeroizes the initialized byte range in its current
owned allocation on drop. The high-level client borrows the response bytes
only to decode the message model and never returns the raw body.

## Boundaries

This contract is an internal execution foundation and has no production constructor. It does not define live TCP/TLS/HTTPS behavior, certificate configuration, any operation other than Discover Versions, C ABI, Java/Python bindings, server-initiated messages, or asynchronous follow-up execution. ADR-0014 defines `kmipkit-transport::Transport::exchange` as a documented public low-level API; it is absent from the top-level facade and is not accepted by a public client constructor. A separately approved TLS/HTTPS feature in `kmipkit-client` owns a public constructor from validated transport configuration. Those capabilities require their own accepted specifications and tests.
