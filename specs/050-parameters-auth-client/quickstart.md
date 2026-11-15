# KMIPKIT-0050 Rust Quickstart

```rust,no_run
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest};
use kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm;
use kmipkit_protocol::{
    Authentication, CryptographicParametersBuilder, EncryptRequest,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::codec::CodecLimits;

fn encrypt_with_authentication(
    client: &mut Client,
    limits: &CodecLimits,
    request_options: &RequestOptions,
    authentication: Authentication,
) -> Result<(), ClientError> {
    let parameters = CryptographicParametersBuilder::new()
        .with_random_iv(true)
        .with_cryptographic_algorithm(cryptographic_algorithm::AES)
        .build()?;

    let request = EncryptRequest::new(None, None).with_typed_cryptographic_parameters(parameters);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(request)))
        .with_authentication(authentication);
    let _response = client.execute_with_options(batch, limits, request_options)?;
    Ok(())
}
```

The example uses the exact public constructors. Authentication is explicitly
owned by this one batch; omitting `with_authentication` omits the Request
Header field. KMIPKit does not persist it in `Client` configuration. The AES
value is chosen explicitly by the caller, and this example does not assert
that a remote server supports or accepts the operation.

For one operation, `Client::encrypt_with_options` and the corresponding direct
method for every client-initiated operation send one typed request through the
same execution path as `ClientBatch`; they do not poll or retry. The final
56-operation method and option inventory is maintained in the
[gap matrix](gap-matrix.md#client-operation-to-direct-method-inventory).
