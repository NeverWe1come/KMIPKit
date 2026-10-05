# Client Execution Contract (Draft)

## Caller boundary

`Client::execute` accepts a closed typed KMIP request/batch. The initial operation is Discover Versions. The API has no argument for generic TTLV items, pre-encoded bytes, a serialization callback, or a downstream-implemented request conversion trait. It is synchronous and performs one exchange per call.

## Request preparation

1. Validate typed operation payloads, message cardinality, IDs, protocol version, and common option values.
2. Preserve an optional caller-supplied request Time Stamp Date-Time exactly when present; omit it when absent. Do not synthesize one or use a countdown-timer source while OD-004 remains unresolved.
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
- The transport receives the configured response-byte cap and must stop reads before allocating or retaining bytes beyond it. The client checks the returned response length again before decode.
- Errors do not automatically retry, resubmit, fail over, poll, or wait.

## Response validation

- The complete message must be structurally valid under the 0005 decoder and 0006 model.
- Protocol Version is exactly 2.1 under ADR-0002.
- Response operation and item identities must match the request. Multi-item response order may vary and IDs drive association. A supplied ID is echoed; single-item requests may omit the ID.
- Pending is accepted only if the request option permits it under known semantics and the response contains the required Asynchronous Correlation Value from the accepted 0006 model. Preserve its opaque bytes unchanged through the `Client::execute` result for later explicit operations. Unknown extension-range indicator values do not grant Pending permission without an explicit registry policy. Batch Error Continuation values remain blocked by the open §9.6/Table 435 range conflict until a source disposition is approved.
- Unrecognized critical Message Extensions fail the complete response as required by OASIS §9.13. Unknown non-critical extension TTLV is preserved as KMIPKit policy; OASIS permits processing it as absent.
- Result and payload errors redact all request/response bytes and secret material.

## Source metadata

Expose only safe typed cause categories and delivery state through `ClientError`/`TransportError`. If an error-source chain is exposed, every link must be a safe typed KMIPKit error; do not retain or display arbitrary original transport/library error text, which may carry untrusted or sensitive payloads.

## Boundaries

This contract is an internal execution foundation and has no production constructor. It does not define live TCP/TLS/HTTPS behavior, certificate configuration, any operation other than Discover Versions, C ABI, Java/Python bindings, server-initiated messages, or asynchronous follow-up execution. A separately approved TLS/HTTPS feature in `kmipkit-client` owns a public constructor from validated transport configuration without arbitrary transport injection. Those capabilities require their own accepted specifications and tests.
