//! Derived API contract test for direct Rust users of `kmipkit-transport`.
//!
//! Traceability: KMIPKIT-0007-FR-005/-FR-009/-FR-013 and ADR-0014. This is not
//! an official OASIS conformance test.

use kmipkit_transport::{Transport, TransportError, TransportResponse};

struct ExternalTransport;

impl Transport for ExternalTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        assert_eq!(request, b"external request");
        assert_eq!(max_response_bytes, 32);
        Ok(TransportResponse::new(b"external response".to_vec()))
    }
}

#[test]
fn external_crate_can_implement_transport_and_read_wrapped_response() {
    let mut transport = ExternalTransport;
    let response = transport
        .exchange(b"external request", 32)
        .expect("the external transport returns its fixture response");

    assert_eq!(response.as_bytes(), b"external response");
}
