# Production TLS and HTTPS transports

`KMIPKit`'s synchronous Rust client can use raw TTLV over TLS or TTLV over
HTTPS/HTTP 1.1. The client sends only an operation the application requested;
construction does not resolve the endpoint, connect, or issue a hidden
Discover Versions request. The 1.0 protocol scope is KMIP 2.1 with TTLV.

## Choose an endpoint

Use `Endpoint::raw_tls(host, port)` when the KMIP server accepts a raw TTLV
frame over TLS. A raw TLS connection carries one request and response and then
closes. Use `Endpoint::https("https://host:port")` when the server exposes the
KMIP HTTPS binding. HTTPS uses HTTP/1.1, defaults to the `/kmip` request target,
and can reuse a healthy connection for later exchanges by the same client.
`target_uri` can select another origin-form path and query. Redirects are not
followed.

## Configure mutual TLS and trust

The caller must provide a client certificate and private key and explicitly
choose server trust. Trust can come from caller-supplied CA certificates or
from the platform certificate store. The example uses PEM files and a caller
CA; the same API accepts DER inputs and explicit platform trust.

```rust,no_run
use std::time::Duration;

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::extension;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = TransportConfig::builder(Endpoint::https("https://kmip.example:5696"))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file("client-chain.pem")?,
            PrivateKeyInput::from_pem_file("client-key.pem")?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file("server-ca.pem")?,
        ]))
        .timeouts(TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(
            Duration::from_secs(60),
        )))
        .build()?;

    let registry = client_extension_registry(Vec::new(), extension::defaults())?;
    let mut client = Client::new(ClientConfiguration::new(registry), transport)?;
    let request = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    let response = client.execute(request, &CodecLimits::defaults())?;
    assert_eq!(response.len(), 1);
    Ok(())
}
```

The filenames above are application-managed paths. `KMIPKit` reads each selected
file once while constructing the input and does not retain its path. Private
keys must be unencrypted and use a supported PEM or DER encoding. Keep key
files readable only by the service account. In-memory inputs are also
available for applications that obtain credentials from another secure
source.

TLS is fixed to TLS 1.3 with mutual authentication, rustls, and AWS-LC. The
server certificate chain, validity period, and endpoint name are verified.
There is no production switch to accept an arbitrary certificate. A TLS
server-name override is available for IP endpoints; it does not disable
certificate verification. Optional CRLs are supplied locally and are not
downloaded. Early data, redirects, proxies, HTTP compression, and TLS key
logging are disabled.

`TrustSource::platform()` selects native platform roots. In that mode,
`SSL_CERT_FILE`, when set, is honored by the platform loader as the selected
bundle. Platform trust does not import every operating-system distrust or
revocation decision. Use an explicit caller CA when the server uses a private
PKI or when the application's trust policy requires a pinned CA set.

TLS session tickets are scoped to a client configuration, limited to 16
tickets, and locally expire after one hour. A resumed connection inherits the
peer identity and trust/CRL decision from its verified full handshake. Rebuild
the client after changing trust material or identity to start with an empty
ticket cache.

## Deadlines and request delivery

The default deadlines are 10 seconds to connect, 30 seconds without write
progress, 30 seconds without read progress, and 60 seconds for the complete
exchange. `TimeoutPolicy` sets client defaults. `RequestOptions` can override
individual phases for one call; phases without an override keep the configured
default. A bounded zero duration means an immediate deadline. Use
`TimeoutLimit::Unbounded` only when the application intentionally wants no
deadline for that phase.

For a single slower operation, apply an override to that call. The returned
delivery state helps the application decide whether it needs to reconcile
server state; it is not a retry recommendation.

```rust,no_run
use std::time::Duration;

use kmipkit_client::{Client, ClientBatch, ClientError};
use kmipkit_transport::{RequestDeliveryState, RequestOptions, TimeoutLimit};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_with_deadline(
    client: &mut Client,
    request: ClientBatch,
) -> Result<(), ClientError> {
    let limits = CodecLimits::defaults();
    let options = RequestOptions::default()
        .with_total(TimeoutLimit::Bounded(Duration::from_secs(90)));

    match client.execute_with_options(request, &limits, &options) {
        Ok(_response) => Ok(()),
        Err(error) => {
            match error.delivery_state() {
                Some(RequestDeliveryState::NotSent) => {
                    // The transport observed no request bytes on the wire.
                }
                Some(
                    RequestDeliveryState::PossiblySent
                    | RequestDeliveryState::ResponseStarted,
                ) => {
                    // Reconcile using operation semantics before any new request.
                }
                None => {
                    // This error has no local transport-delivery state.
                }
                Some(_) => {
                    // Preserve future states as uncertain; do not retry them blindly.
                }
            }
            Err(error)
        }
    }
}
```

Every local failure reports the strongest available request-delivery state:

- `NotSent`: no request bytes were sent.
- `PossiblySent`: sending began, but no decrypted response byte was observed.
- `ResponseStarted`: at least one response byte was observed, but no complete
  valid response was accepted.

Do not treat `PossiblySent` or `ResponseStarted` as proof that an operation did
not take effect. `KMIPKit` never retries a request automatically. The caller
must use the operation's semantics and application state to decide how to
reconcile an uncertain result. A failed or cancelled connection is not reused.

The typed client uses `CodecLimits` for request encoding and response decoding;
the default complete-message cap is 16 MiB. The transport checks the response
cap before the TTLV decoder receives response bytes. The transport's direct
request limit also defaults to 16 MiB. These local limits do not claim that a
server will accept a request or honor a requested response limit.

## Errors, secrets, and interoperability

Configuration and transport errors expose fixed safe categories, not
credential bytes, private key text, dependency error messages, raw KMIP bodies,
or configured URL path/query values. Applications should log the safe error
category and delivery state, not credentials or message bodies. `KMIPKit`
zeroizes initialized bytes in allocations it owns, including owned private-key
inputs and transport response buffers. This cannot erase caller-created
copies or copies retained by TLS libraries, the operating system, or foreign
runtimes.

`KMIPKit` transports and manages cryptographic material; it does not perform
local cryptographic operations. Whether an operation succeeds depends on the
server's implemented capabilities, configuration, and policy. A server's
rejection is not by itself evidence of a client protocol defect. Consult the
[transport security architecture](../../architecture/transport-security.md)
and [client execution guide](client-execution.md) for the detailed API and
delivery contracts.

## Local Cosmian integration tests

The repository includes an opt-in Docker Compose deployment of Cosmian KMS
5.28.0 and ignored live tests that use the typed Rust client over mutually
authenticated TLS. Follow the [local Cosmian test guide](../../../tests/integration/cosmian/README.md)
for setup, commands, tested operations, and current interoperability results.

## A local, executable request example

Building a typed batch does not contact a server. This example is run as a
Rust documentation test:

```rust
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};

let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
assert_eq!(batch.items().len(), 1);
```
