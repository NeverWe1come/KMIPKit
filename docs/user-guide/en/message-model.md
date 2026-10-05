# Inspecting KMIP Messages

The KMIP protocol crate can validate and inspect a decoded Request or Response
Message before another layer handles it. The message model keeps the complete
ordered TTLV tree, so unknown values and vendor extensions remain available to
application code.

## Parse and inspect

Start with a generic TTLV `Structure` produced by the bounded decoder. Pass
ownership to `RequestMessage::try_from_ttlv` or
`ResponseMessage::try_from_ttlv`. The constructor reports a safe validation
category and numeric location if the message envelope, header, batch items, or
Message Extensions do not have the required shape.

The request and response expose typed views for common header fields and batch
items. Use their accessors to read the Protocol Version, Batch Count, raw
option values, timestamps, item IDs, result status, and correlation values.
Nested Authentication, Nonce, payload, and vendor data are lent through
callbacks; inspect or copy only the values your application needs. Errors and
default formatting do not include payload contents.

The public crate rustdoc contains a compiled example that parses a request,
checks its version and Batch Count, and returns the original tree with
`into_ttlv()`.

## Defaults and asynchronous results

Optional fields preserve their original presence. Accessors also expose the
KMIP effective default when the field is absent: Asynchronous Indicator is
Prohibited, Batch Error Continuation Option is Stop, Batch Order Option is
True, and Attestation Capable Indicator is False. Raw Enumeration values stay
unchanged, including values this library does not assign a name.

A response may contain completed and Pending items in one batch. A Pending
item must include its Asynchronous Correlation Value; the model preserves its
bytes for a later explicit operation. This layer does not Poll, Cancel, wait,
or retry automatically.

## Scope

Message validation covers common message and batch structure. It does not
validate operation-specific payloads, choose cryptographic parameters, encode
TTLV bytes, or contact a KMIP server. The client execution API and operation
models are documented by their own feature guides as they become available.

For the public API boundary, see the
[architecture reference](../../architecture/public-api.md#kmip-message-model).
