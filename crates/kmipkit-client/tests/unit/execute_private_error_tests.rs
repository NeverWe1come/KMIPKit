//! Derived private-boundary tests for the KMIPKIT-0007 result path. These are
//! not official OASIS test vectors. The represented Protocol Version and
//! response mapping checks are grounded in OASIS KMIP v2.1 §9.16, Table 421,
//! and §§8.3 and 9.21, Tables 396 and 426. Generic value copying preserves the
//! KMIP 2.1 TTLV model defined by §6.1.1.
//!
//! Traceability: `KMIPKIT-0007-FR-004`, `-FR-006`, `-FR-008`, `-FR-013`,
//! `-FR-016`; `KMIPKIT-0007-SC-002`, `-SC-005`, `-SC-009`.

use std::error::Error;

use kmipkit_protocol::ProtocolErrorKind;
use kmipkit_ttlv::{Item, RawTag, Value};

use super::{
    BatchIdentity, BatchValidationError, OutcomeValidationError, ResponseAssociationError,
    ResponseLimitExceeded, ResponseValidationFailure, associate_batch_items, copy_value,
    decode_response_message, encode_message_for_test, private_wire_writer,
};

#[test]
fn private_validation_errors_have_safe_display_text() {
    let errors = [
        BatchValidationError::EmptyBatch,
        BatchValidationError::MissingBatchItemId,
        BatchValidationError::DuplicateBatchItemId,
        BatchValidationError::InvalidAsynchronousIndicator,
        BatchValidationError::RepeatedBatchErrorContinuation,
        BatchValidationError::SingleItemBatchErrorContinuation,
        BatchValidationError::InvalidBatchErrorContinuation,
        BatchValidationError::ExtensionRegistryMismatch,
    ];
    for error in errors {
        assert_ne!(error.to_string(), "");
        assert!(Error::source(&error).is_none());
    }

    assert_eq!(
        ResponseValidationFailure.to_string(),
        "response validation failed"
    );
    assert_eq!(
        ResponseLimitExceeded.to_string(),
        "response exceeds the configured byte limit"
    );
    assert_eq!(
        BatchValidationError::ExtensionRegistryMismatch.to_string(),
        "request extension was validated for a different client registry"
    );
}

#[test]
fn protocol_error_mappings_cover_association_and_outcome_failures() {
    let error = kmipkit_protocol::ProtocolError::from(ResponseAssociationError::DuplicateId);
    assert_eq!(error.kind(), ProtocolErrorKind::InvalidValue);

    for (failure, expected) in [
        (
            OutcomeValidationError::PendingNotPermitted,
            ProtocolErrorKind::InvalidValue,
        ),
        (
            OutcomeValidationError::PendingCorrelationMissing,
            ProtocolErrorKind::InvalidValue,
        ),
        (
            OutcomeValidationError::UnknownCriticalExtension,
            ProtocolErrorKind::UnsupportedValue,
        ),
    ] {
        assert_eq!(
            kmipkit_protocol::ProtocolError::from(failure).kind(),
            expected
        );
    }
}

#[test]
fn response_association_rejects_duplicate_request_id_matches() {
    let requests = [
        BatchIdentity {
            operation: 0x0000_001E,
            unique_batch_item_id: Some(b"duplicate-request-id".to_vec()),
            response_context: None,
        },
        BatchIdentity {
            operation: 0x0000_001E,
            unique_batch_item_id: Some(b"duplicate-request-id".to_vec()),
            response_context: None,
        },
    ];
    let responses = [
        BatchIdentity {
            operation: 0x0000_001E,
            unique_batch_item_id: Some(b"duplicate-request-id".to_vec()),
            response_context: None,
        },
        BatchIdentity {
            operation: 0x0000_001E,
            unique_batch_item_id: Some(b"different-response-id".to_vec()),
            response_context: None,
        },
    ];

    assert_eq!(
        associate_batch_items(&requests, &responses),
        Err(ResponseAssociationError::DuplicateId)
    );
}

#[test]
fn response_association_rejects_an_unmatched_batch_item_identifier() {
    let requests = [BatchIdentity {
        operation: 0x0000_001E,
        unique_batch_item_id: Some(b"requested-id".to_vec()),
        response_context: None,
    }];
    let responses = [BatchIdentity {
        operation: 0x0000_001E,
        unique_batch_item_id: Some(b"returned-id".to_vec()),
        response_context: None,
    }];

    assert_eq!(
        associate_batch_items(&requests, &responses),
        Err(ResponseAssociationError::UnknownId)
    );
}

#[test]
fn response_decoder_requires_a_structure_root_and_valid_response_message() {
    let limits = super::CodecLimits::defaults();
    let scalar_tag = RawTag::new(0x0042_0173)
        .expect("fixture tag fits the KMIP width")
        .try_checked()
        .expect("fixture tag is allocated");
    let scalar = Item::new(scalar_tag, Value::integer(1)).expect("scalar Item is valid");
    let encoded_scalar = private_wire_writer::encode_item_for_test(&scalar, &limits)
        .expect("scalar TTLV is encodable");
    let scalar_error = decode_response_message(encoded_scalar.as_bytes(), &limits)
        .expect_err("the response decoder requires a Structure root");
    assert_eq!(scalar_error.kind(), ProtocolErrorKind::MalformedMessage);

    let encoded_empty_message = encode_message_for_test(kmipkit_ttlv::Structure::new(), &limits)
        .expect("an empty generic Message Structure is encodable");
    let message_error = decode_response_message(&encoded_empty_message, &limits)
        .expect_err("a generic Structure without response fields is not a ResponseMessage");
    assert_eq!(message_error.kind(), ProtocolErrorKind::MalformedMessage);
}

#[test]
fn generic_extension_value_copy_preserves_all_wide_scalar_types() {
    let tag = RawTag::new(0x0042_0173)
        .expect("fixture tag fits the KMIP width")
        .try_checked()
        .expect("fixture tag is allocated");
    let values = [
        Value::long_integer(-7),
        Value::big_integer(vec![0xA5; 8]),
        Value::interval(u32::MAX),
        Value::date_time_extended(i64::MIN),
    ];

    for value in values {
        let item = Item::new(tag, value).expect("fixture value fits a generic Item");
        let copied = item
            .with_value(copy_value)
            .expect("wide values are preserved");
        let copied_item = Item::new(tag, copied).expect("copied value remains a generic Item");
        assert_eq!(copied_item.item_type(), item.item_type());
    }
}
