#![cfg(test)]

//! ID Placeholder operation classification derived from KMIP Specification v2.1 §6.1 and operation clauses.

use kmipkit_protocol::{
    AttributeSet, CertifyRequest, CreateRequest, CreateSplitKeyRequest, DerivationMethod,
    DerivationParameters, DeriveKeyRequest, ExportRequest, ImportRequest, JoinSplitKeyRequest,
    ObjectType, RecertifyRequest, SecretBytes, SplitKeyMethod, SplitKeyReference, UniqueIdentifier,
};
use kmipkit_ttlv::Value;

use super::ClientRequest;
use crate::execute_test_support::{test_item, test_structure};

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

fn explicit_id_placeholder_consumers() -> [(&'static str, ClientRequest); 4] {
    [
        (
            "Certify Unique Identifier with Certificate Request Value",
            ClientRequest::Certify(
                CertifyRequest::new()
                    .with_unique_identifier(UniqueIdentifier::Enumeration(1))
                    .with_certificate_request_type(1)
                    .with_certificate_request_value(SecretBytes::new(vec![1])),
            ),
        ),
        (
            "Import Unique Identifier",
            ClientRequest::Import(ImportRequest::new(
                UniqueIdentifier::Enumeration(1),
                ObjectType::from_raw(2),
                AttributeSet::new(),
                test_item(0x0042_0040, Value::structure(test_structure([]))),
            )),
        ),
        (
            "Join Split Key Unique Identifier",
            ClientRequest::JoinSplitKey(JoinSplitKeyRequest::new(
                ObjectType::from_raw(7),
                vec![
                    SplitKeyReference::new(UniqueIdentifier::Enumeration(1))
                        .with_split_key_threshold(1),
                ],
            )),
        ),
        (
            "Re-certify Certificate Request Unique Identifier",
            ClientRequest::Recertify(
                RecertifyRequest::new()
                    .with_unique_identifier(UniqueIdentifier::TextString(
                        "existing-certificate".to_owned(),
                    ))
                    .with_certificate_request_unique_identifier(UniqueIdentifier::Enumeration(1)),
            ),
        ),
    ]
}

#[test]
fn explicit_id_placeholder_in_certify_import_join_split_key_and_recertify_is_a_consumer() {
    for (field, request) in explicit_id_placeholder_consumers() {
        assert!(
            request.uses_id_placeholder(),
            "{field} selects Enumeration(1)"
        );
        let batch = super::ClientBatch::new(super::ClientBatchItem::new(request));
        assert_eq!(
            super::validate_id_placeholder_eligibility(&batch),
            Err(super::BatchValidationError::IneligibleIdPlaceholder),
            "{field} cannot consume a missing preceding placeholder"
        );
    }
}

#[test]
fn explicit_id_placeholder_consumers_including_certify_reject_false_batch_order_before_send() {
    for (field, request) in explicit_id_placeholder_consumers() {
        let batch = super::ClientBatch::from_items([
            super::ClientBatchItem::new(ClientRequest::Create(CreateRequest::new(
                ObjectType::from_raw(2),
                AttributeSet::new(),
            ))),
            super::ClientBatchItem::new(request),
        ])
        .with_batch_order_option(false);

        assert_eq!(
            super::validate_id_placeholder_eligibility(&batch),
            Err(super::BatchValidationError::IneligibleIdPlaceholder),
            "{field} requires Batch Order Option true or absent"
        );
    }
}
