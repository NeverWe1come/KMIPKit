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

### Java

`org.kmipkit.server` provides an `AutoCloseable` facade. Configure the listener
identity and trusted peer certificates, bind with `ServerListener.bindRawTls`
or `bindHttps`, accept with an explicit `Duration`, and receive with
`CodecLimits`. `ServerRequestBatch.items()` returns operation-specific request
types. Build success responses with each response type's `forRequest` factory,
or use `ServerReply.failure` for an explicit KMIP failure; the
`ServerResponseBuilder` rejects a response paired with a different request.
`NotifyRequest.attributes()` returns an independently owned `AttributeSet` for
the optional changed attributes; close it when finished. Send the built
response explicitly on the accepted `ServerConnection`.

```java
try (ServerListenerConfiguration config = ServerListenerConfiguration.builder()
        .bindHost("127.0.0.1").bindPort(5696)
        .listenerCertificateChain(List.of(serverCertificatePem))
        .listenerPrivateKey(serverPrivateKeyPem)
        .trustCertificates(List.of(clientCaPem)).build();
     ServerListener listener = ServerListener.bindRawTls(config);
     ServerConnection connection = listener.accept(Duration.ofSeconds(5));
     ServerRequestBatch batch = connection.receive(CodecLimits.defaults(), Duration.ofSeconds(5));
     ServerResponseMessage response = batch.respond(request -> {
         if (request instanceof DiscoverVersionsRequest discover) {
             return ServerReply.success(DiscoverVersionsResponse.forRequest(
                     discover, List.of(new ProtocolVersion(2, 1))));
         }
         return ServerReply.failure(ServerResultStatus.OPERATION_FAILED,
                 ServerResultReason.OPERATION_NOT_SUPPORTED, "unsupported operation");
     })) {
    connection.send(response, Duration.ofSeconds(5));
}
```

### Python

`kmipkit.server` exposes the same explicit receive/respond/send flow through
context managers. The listener configuration accepts certificate material as
bytes, and the private-key staging copy is overwritten after native
configuration. `NotifyRequest.attributes` returns a validated `AttributeSet`
copy that can be closed or used as a context manager. Python runtime copies
cannot be reliably zeroized.

```python
with ServerListenerConfiguration(
    bind_host="127.0.0.1",
    bind_port=5696,
    listener_certificate_chain=(server_certificate_pem,),
    listener_private_key=server_private_key_pem,
    trust_certificates=(client_ca_pem,),
) as config, ServerListener.bind(config) as listener:
    with listener.accept(timeout=5) as connection:
        connection.serve_once(handler, ttlv.CodecLimits.defaults(), timeout=5)
```

The C binding remains the low-level ABI route with explicit opaque handles and
fixed-width status codes. All four bindings receive and answer typed server
requests; none adds a client-side `send()` operation to these request types.
See each language's reference for ownership and release rules.

In C, respond to the received Set Endpoint Role request first, then call
`kmipkit_server_connection_into_role_reversed_session`. The connection handle
must be released after the conversion attempt. An active batch prevents
conversion and leaves the connection usable; once conversion starts, the
underlying connection is consumed even if the transport is HTTPS and cannot
be converted. Use the role-reversed session's `recv_batch` and
`send_response` functions for the subsequent exchange.

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
