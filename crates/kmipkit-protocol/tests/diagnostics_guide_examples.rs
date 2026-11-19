//! Executable examples from the English/Spanish diagnostics user guides.
//! Source: OASIS KMIP v2.1 §§6.1.26, 6.1.29, 6.1.57; Tables 241, 250, 340.

use kmipkit_protocol::{InteropRequest, LogRequest, SecretText, UniqueIdentifier, ValidateRequest};

#[test]
fn interop_example_keeps_explicit_required_fields() {
    let request = InteropRequest::new(1, SecretText::new("caller-selected-test-id".to_owned()));
    let payload = request.into_ttlv_payload().unwrap();
    assert_eq!(payload.view().children().len(), 2);
}

#[test]
fn log_example_encodes_the_required_message() {
    let request = LogRequest::new(SecretText::new("test completed".to_owned()));
    let payload = request.into_ttlv_payload().unwrap();
    assert_eq!(payload.view().children()[0].tag().raw(), 0x0042_0141);
}

#[test]
fn validate_example_preserves_both_identifiers_and_selected_date() {
    let request = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("leaf-certificate".to_owned()))
        .with_unique_identifier(UniqueIdentifier::TextString(
            "issuer-certificate".to_owned(),
        ))
        .with_validity_date(1_700_000_000);
    let payload = request.into_ttlv_payload().unwrap();
    assert_eq!(payload.view().children().len(), 3);
}
