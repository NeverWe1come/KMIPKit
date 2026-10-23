#![cfg(test)]

//! ID Placeholder operation classification derived from KMIP Specification v2.1 §6.1 and operation clauses.

use kmipkit_protocol::{
    AttributeSet, CreateSplitKeyRequest, DerivationMethod, DerivationParameters, DeriveKeyRequest,
    ExportRequest, ObjectType, SecretBytes, SplitKeyMethod, UniqueIdentifier,
};

use super::ClientRequest;

#[test]
fn operation_clauses_add_create_split_key_derive_key_and_export_as_producers() {
    // KMIP Specification v2.1 §§6.1.10, 6.1.14, and 6.1.18 explicitly set the ID Placeholder.
    let create_split_key = CreateSplitKeyRequest::new(
        ObjectType::from_raw(7),
        3,
        2,
        SplitKeyMethod::XOR,
        AttributeSet::new(),
    )
    .with_unique_identifier(UniqueIdentifier::TextString(
        "split-source-object".to_owned(),
    ));
    let derive_key = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("derivation-source".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new().with_derivation_data(SecretBytes::new(vec![1])),
        AttributeSet::new(),
    );
    let export = ExportRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("export-source".to_owned()));

    for (operation, request) in [
        (
            "Create Split Key",
            ClientRequest::CreateSplitKey(create_split_key),
        ),
        ("Derive Key", ClientRequest::DeriveKey(derive_key)),
        ("Export", ClientRequest::Export(export)),
    ] {
        assert!(
            request.is_id_placeholder_producer(),
            "{operation} is a normative ID Placeholder producer"
        );
    }
}

#[test]
fn create_split_key_can_select_a_preceding_placeholder_before_producing_its_result() {
    let request = ClientRequest::CreateSplitKey(
        CreateSplitKeyRequest::new(
            ObjectType::from_raw(7),
            3,
            2,
            SplitKeyMethod::XOR,
            AttributeSet::new(),
        )
        .with_unique_identifier(UniqueIdentifier::Enumeration(1)),
    );

    assert!(request.uses_id_placeholder());
    assert!(request.is_id_placeholder_producer());

    let batch = super::ClientBatch::new(super::ClientBatchItem::new(request));
    assert_eq!(
        super::validate_id_placeholder_eligibility(&batch),
        Err(super::BatchValidationError::IneligibleIdPlaceholder),
        "the request cannot consume a placeholder produced only by itself"
    );
}
