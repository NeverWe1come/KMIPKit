# Server-to-client operations and role changes

KMIPKit 1.0 includes the KMIP 2.1 server-to-client operations and endpoint role
handling defined by KMIPKIT-0066 and ADR-0021. This capability lets an
application accept an authenticated KMIP server connection and respond to
server-originated requests. It is separate from ordinary client-initiated
calls through `Client`.

## Listener flow

The Rust API exposes `ClientListener::bind_raw_tls` and
`ClientListener::bind_https`. Build a `ListenerTransportConfig` with the
listener identity, trusted peer roots, any configured revocation lists,
encoding and request-size limits, and timeouts. Both listener transports
require TLS 1.3 and mutual certificate authentication; peer certificates are
validated against the configured trust source.

After binding, call `accept` to obtain a `ClientListenerConnection`, then
`recv_batch` to read a bounded `IncomingServerBatch`. Inspect its protocol
version, batch count, item identifiers, and typed operation views. The
supported server-originated requests are Discover Versions, Notify, Put,
Query, and Set Endpoint Role. Construct a `ResponseMessage` for the received
batch and send it with `send_response`. Use the operation-specific models and
the KMIP 2.1 specification to apply application policy; receiving a valid
request does not mean the application must accept its effect.

The C, Java, and Python bindings expose corresponding listener, dispatcher,
and typed request access. Use the published API manifest and each language's
reference docs for exact ownership and cleanup rules.

## Endpoint role reversal

KMIP Set Endpoint Role is supported in both directions. After a successful
client-to-server Set Endpoint Role exchange over raw TLS, convert the
connection with `into_role_reversed` and use the resulting
`RoleReversedSession` to receive server requests and send client responses
over that established stream. An HTTPS listener accepts inbound HTTPS
connections; it cannot be converted to a raw-TLS role-reversed session.

Role changes affect which endpoint sends the next KMIP message. Keep that
transition tied to the successful protocol response and the owning connection.
Do not treat it as ordinary background polling or automatically reconnect and
replay requests.

## Security and evidence

Listener decoding applies the configured size and structural limits. Secret
key material received in server-originated Put requests is handled under the
same zeroization and redaction rules as other KMIP secrets. Applications must
still authorize and validate the requested operation according to their own
policy.

The implementation and release evidence are summarized in the
[project status](../../project-status.md). The server matrix records external
results separately: support for a listener API does not prove that a particular
KMIP server has exercised every operation, encoding, or role transition. See
the [KMIPKIT-0066 specification](../../../specs/066-server-to-client-operations/spec.md)
and the normative references it cites before implementing a handler.
