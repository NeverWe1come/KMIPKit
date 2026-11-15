//! Default-build API visibility check for KMIP 2.1 §6.1.37 Tables 273–275.

use kmipkit_client::{Client, ClientBatchItemResponse, ClientError, ClientRequest, RequestOptions};
use kmipkit_protocol::Pkcs11Request;
use kmipkit_ttlv::{Structure, codec::CodecLimits};

#[test]
fn pkcs11_request_and_client_methods_are_public_in_the_default_build() {
    let request = Pkcs11Request::new(Structure::new());
    let request = ClientRequest::pkcs11(request);

    assert!(matches!(request, ClientRequest::Pkcs11(_)));
    let single_request_method: fn(
        &mut Client,
        Pkcs11Request,
        &CodecLimits,
    ) -> Result<ClientBatchItemResponse, ClientError> = Client::pkcs11;
    let single_request_options_method: fn(
        &mut Client,
        Pkcs11Request,
        &CodecLimits,
        &RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> = Client::pkcs11_with_options;

    assert_ne!(single_request_method as usize, 0);
    assert_ne!(single_request_options_method as usize, 0);
}
