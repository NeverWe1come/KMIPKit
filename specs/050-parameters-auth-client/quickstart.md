# KMIPKIT-0050 Rust Quickstart (Planned Contract)

```rust,no_run
use kmipkit_protocol::{CryptographicParametersBuilder, EncryptRequest};
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse};
use kmipkit_protocol::Authentication;

let parameters = CryptographicParametersBuilder::new()
    .with_cryptographic_algorithm(0x8000_0001)
    .with_random_iv(false)
    .build()?;

let request = EncryptRequest::new(/* caller-selected values */)
    .with_typed_cryptographic_parameters(parameters);
let batch = ClientBatch::new(ClientBatchItem::new(request.into()))
    .with_authentication(authentication);
let response: ClientBatchItemResponse = client.execute(batch, &limits)?.into_single()?;
```

The final example will use the exact public constructors available after
implementation and be compiled as a test. Authentication is explicitly owned
by this one batch; omitting `with_authentication` omits the Request Header
field. KMIPKit does not persist it in `Client` configuration.
