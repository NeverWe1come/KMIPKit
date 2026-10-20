use kmipkit_client::{Client, ClientBatchOutcome, ClientOperation, ClientRequest, Pkcs11Request};
use kmipkit_protocol::Pkcs11Response;

fn main() {
    let _ = std::mem::size_of::<ClientBatchOutcome>();
    let _ = std::mem::size_of::<ClientRequest>();
    let _ = std::mem::size_of::<Pkcs11Request>();
    let _ = std::mem::size_of::<Pkcs11Response>();
    let _ = Client::pkcs11;
    let _ = ClientRequest::Pkcs11;
    let _ = ClientOperation::Pkcs11;
    let _ = ClientBatchOutcome::Pkcs11;
}
