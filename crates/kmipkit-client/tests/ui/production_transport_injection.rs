use kmipkit_client::extension_registry::ClientConfiguration;
use kmipkit_client::Client;
use kmipkit_transport::{Transport, TransportError, TransportResponse};

struct CallerTransport;

impl Transport for CallerTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        panic!("compile-fail fixture must never execute")
    }
}

#[allow(dead_code)]
fn caller_cannot_inject_transport(
    configuration: ClientConfiguration,
    transport: CallerTransport,
) {
    let _ = Client::new(configuration, transport);
}

fn main() {}
