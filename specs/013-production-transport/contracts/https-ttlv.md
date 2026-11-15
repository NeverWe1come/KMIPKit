# Contract: HTTPS with TTLV

## Request

- HTTPS and HTTP/1.1 only. POST the exact request bytes to the configured
  origin-form target (absolute path with optional query); default is `/kmip`.
- Set exactly one `Content-Type: application/octet-stream`, exact
  `Content-Length`, and `Cache-Control: no-cache`.
- The endpoint is the absolute HTTPS URI that determines authority. The
  request target is origin-form only; reject a scheme, authority, fragment,
  malformed path, user information, or invalid endpoint port. A request target
  can never change the configured authority or TLS server name.
- Send exactly one `Host` header serialized from the configured endpoint
  authority, preserving its hostname or bracketed IP literal and any
  explicitly configured port. The request target and optional TLS verification
  name do not change `Host`; test a non-default endpoint port with a different
  TLS verification name. HTTP/1.1 requires a `Host` field in each request
  (RFC 9112 §3.2).
- Never consult proxy environment/system configuration. Do not follow
  redirects or enable cookies, compression/decompression, HTTP/2, automatic
  retry, or automatic failover.

## Response

- Accept status 200 only.
- Require one unambiguous `Content-Type` with media type
  `application/octet-stream` and one valid `Content-Length`.
- Configure Hyper's maximum response header count to 64 and its connection
  input buffer limit to 64 KiB. The parser's bounded input buffer is separate
  from KMIPKit's response-body cap.
- Reject `Transfer-Encoding`, `Content-Encoding`, conflicting/duplicate
  framing headers, malformed header syntax, parser errors, non-200 statuses,
  truncation, and a declared body length over the effective response cap.
- Check declared size before KMIPKit allocates/grows its response-body owner,
  then stream into one bounded allocation. Never accumulate without the cap.
  On error, zeroize initialized KMIPKit-owned body bytes and invalidate the
  connection. Hyper/rustls/OS-owned input buffers are third-party copies and
  are outside the zeroization guarantee.
- Return only the HTTP message body in `TransportResponse`; TTLV and KMIP
  response validation remains in the typed client.
- Permit at most one exchange in flight on a reusable connection. If Hyper
  observes an unsolicited or surplus response after the current response,
  invalidate that connection and discard the surplus rather than associating
  it with the next request. The next distinct exchange may reconnect. An
  integration test must queue an unsolicited second response before the next
  request and prove it cannot satisfy that later exchange.

Hyper owns HTTP/1 message parsing. KMIPKit owns the narrower KMIP status,
header, content-type, content-length, encoding, size, and connection policy.
An HTTP transport exchange does not establish full HTTPS Client Profile
conformance.
