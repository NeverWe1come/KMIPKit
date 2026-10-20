//! Default-build API visibility check for KMIP 2.1 §6.1.37 Tables 273–275.

use kmipkit_client::{Client, ClientRequest};
use kmipkit_protocol::Pkcs11Request;
use kmipkit_ttlv::Structure;

#[test]
fn pkcs11_request_and_client_methods_are_public_in_the_default_build() {
    let request = Pkcs11Request::new(Structure::new());
    let request = ClientRequest::pkcs11(request);

    assert!(matches!(request, ClientRequest::Pkcs11(_)));
    let _single_request_method = Client::pkcs11;
    let _single_request_options_method = Client::pkcs11_with_options;
}
