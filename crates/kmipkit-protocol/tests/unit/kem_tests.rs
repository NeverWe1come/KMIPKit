//! Unit tests for KMIP 3.0 KEM (Encapsulate/Decapsulate) and Deactivate models (KMIPKIT-0071).

use crate::{
    DeactivateRequest, DeactivateResponse, DecapsulateRequest, DecapsulateResponse,
    EncapsulateRequest, EncapsulateResponse,
};

#[test]
fn test_deactivate_request_and_response_roundtrip() {
    let req = DeactivateRequest::new("test-key-uuid-123");
    assert_eq!(
        req.unique_identifier(),
        &crate::UniqueIdentifier::TextString("test-key-uuid-123".to_owned())
    );

    let payload = req.into_ttlv_payload().expect("valid payload");
    assert_eq!(payload.view().children().len(), 1);

    let resp = DeactivateResponse::new("test-key-uuid-123");
    assert_eq!(
        resp.unique_identifier(),
        Some(&crate::UniqueIdentifier::TextString(
            "test-key-uuid-123".to_owned()
        ))
    );
    let resp_payload = resp.into_ttlv_payload().expect("valid payload");
    assert_eq!(resp_payload.view().children().len(), 1);
}

#[test]
fn test_encapsulate_request_and_response() {
    let req = EncapsulateRequest::new("public-key-uuid-456");
    assert_eq!(
        req.unique_identifier(),
        &crate::UniqueIdentifier::TextString("public-key-uuid-456".to_owned())
    );

    let payload = req.into_ttlv_payload().expect("valid payload");
    assert_eq!(payload.view().children().len(), 1);

    let ciphertext = vec![0x01, 0x02, 0x03, 0x04];
    let resp = EncapsulateResponse::new("public-key-uuid-456", ciphertext.clone());
    assert_eq!(
        resp.unique_identifier(),
        Some(&crate::UniqueIdentifier::TextString(
            "public-key-uuid-456".to_owned()
        ))
    );
    assert_eq!(resp.data(), Some(ciphertext.as_slice()));
    let debug = format!("{resp:?}");
    let resp_payload = resp.into_ttlv_payload().expect("valid payload");
    // OASIS KMIP 3.0 §6.1.22 Table 318 requires Unique Identifier and Data only.
    let response_tags: Vec<u32> = resp_payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect();
    assert_eq!(response_tags, [0x0042_0094, 0x0042_00C2]);
    assert!(!debug.contains("shared_secret"));
    assert!(!debug.contains("[170, 187, 204, 221]"));
}

#[test]
fn test_decapsulate_request_and_response() {
    let ciphertext = vec![0x01, 0x02, 0x03, 0x04];
    let req = DecapsulateRequest::new("private-key-uuid-789", ciphertext.clone());
    assert_eq!(
        req.unique_identifier(),
        &crate::UniqueIdentifier::TextString("private-key-uuid-789".to_owned())
    );
    assert_eq!(req.ciphertext(), &ciphertext);
    let req_payload = req.into_ttlv_payload().expect("valid payload");
    assert_eq!(req_payload.view().children().len(), 2);

    let resp = DecapsulateResponse::new("private-key-uuid-789");
    assert_eq!(
        resp.unique_identifier(),
        Some(&crate::UniqueIdentifier::TextString(
            "private-key-uuid-789".to_owned()
        ))
    );
    let debug = format!("{resp:?}");
    let resp_payload = resp.into_ttlv_payload().expect("valid payload");
    // OASIS KMIP 3.0 §6.1.15 Table 297 requires only Unique Identifier.
    let response_tags: Vec<u32> = resp_payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect();
    assert_eq!(response_tags, [0x0042_0094]);
    assert!(!debug.contains("shared_secret"));
    assert!(!debug.contains("[170, 187, 204, 221]"));
}
