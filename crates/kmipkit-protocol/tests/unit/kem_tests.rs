//! Unit tests for KMIP 3.0 KEM (Encapsulate/Decapsulate) and Deactivate models (KMIPKIT-0071).

use crate::{
    DeactivateRequest, DeactivateResponse, DecapsulateRequest, DecapsulateResponse,
    EncapsulateRequest, EncapsulateResponse,
};

#[test]
fn test_deactivate_request_and_response_roundtrip() {
    let req = DeactivateRequest::new("test-key-uuid-123");
    assert_eq!(req.unique_identifier(), "test-key-uuid-123");

    let payload = req.into_ttlv_payload().expect("valid payload");
    assert_eq!(payload.view().children().len(), 1);

    let resp = DeactivateResponse::new("test-key-uuid-123");
    assert_eq!(resp.unique_identifier(), "test-key-uuid-123");
}

#[test]
fn test_encapsulate_request_and_response() {
    let req = EncapsulateRequest::new("public-key-uuid-456");
    assert_eq!(req.unique_identifier(), "public-key-uuid-456");

    let payload = req.into_ttlv_payload().expect("valid payload");
    assert_eq!(payload.view().children().len(), 1);

    let ciphertext = vec![0x01, 0x02, 0x03, 0x04];
    let shared_secret = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let resp = EncapsulateResponse::new(
        "public-key-uuid-456",
        ciphertext.clone(),
        shared_secret.clone(),
    );
    assert_eq!(resp.unique_identifier(), "public-key-uuid-456");
    assert_eq!(resp.ciphertext(), &ciphertext);
    assert_eq!(resp.shared_secret(), &shared_secret);
}

#[test]
fn test_decapsulate_request_and_response() {
    let ciphertext = vec![0x01, 0x02, 0x03, 0x04];
    let req = DecapsulateRequest::new("private-key-uuid-789", ciphertext.clone());
    assert_eq!(req.unique_identifier(), "private-key-uuid-789");
    assert_eq!(req.ciphertext(), &ciphertext);

    let shared_secret = vec![0xAA, 0xBB, 0xCC, 0xDD];
    let resp = DecapsulateResponse::new("private-key-uuid-789", shared_secret.clone());
    assert_eq!(resp.unique_identifier(), "private-key-uuid-789");
    assert_eq!(resp.shared_secret(), &shared_secret);
}
