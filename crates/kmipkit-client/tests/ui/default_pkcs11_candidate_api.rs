use kmipkit_client::{ClientBatchOutcome, ClientRequest, Pkcs11Request};
use kmipkit_protocol::Pkcs11Response;

fn main() {
    let _ = std::mem::size_of::<ClientBatchOutcome>();
    let _ = std::mem::size_of::<ClientRequest>();
    let _ = std::mem::size_of::<Pkcs11Request>();
    let _ = std::mem::size_of::<Pkcs11Response>();
}
