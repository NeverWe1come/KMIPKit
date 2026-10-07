# Contract: Raw TTLV over TLS

## Request path

The direct low-level API borrows caller bytes for one synchronous exchange,
rejects a request above the configured `max_request_bytes` before DNS or
connection activity, performs no TTLV encoding or KMIP-schema validation,
does not log or retain the bytes, and reports write progress. The caller is
responsible for supplying a single valid TTLV message. The typed client
supplies bytes produced by its existing validated encode path. No connection
retries a failed request.

## Response framing

1. Read the 8-byte TTLV header into fixed-size stack storage.
2. Validate root tag/type, declared length alignment, checked arithmetic, and
   the effective `max_response_bytes` limit.
3. Only after those checks, allocate one response buffer with fallible bounded
   reservation. Never grow it after response bytes are initialized.
4. Read exactly the declared value and padding for one frame. EOF before
   completion, invalid headers, I/O failure, or limit violation invalidates
   the connection. Zeroize all initialized response bytes before returning an
   error.
5. Return a successful frame in `TransportResponse`, whose initialized bytes
   are zeroized when the owner is dropped.

The one-frame exchange is KMIPKit project behavior. OASIS specifies TTLV
encoding, but does not define separate raw-TLS stream framing. The adapter
does not claim full KMIP operation validity for direct low-level callers.

## Reuse and delivery

Calls through one client are serialized. Raw TTLV connections close after
exactly one response frame, whether that exchange succeeds or fails; the next
distinct call establishes a new connection. This prevents unsolicited or
surplus frames from being mistaken for a later response. Coalesced bytes
after the first frame are discarded with the closed connection and never
returned as a second response. Delivery transitions follow [timeout and
delivery semantics](timeout-and-delivery.md).
