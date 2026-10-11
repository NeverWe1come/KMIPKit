//! Shared key-transfer contracts derived from KMIP 2.1 §§6.1.14, 6.1.18,
//! 6.1.25, 6.1.27, and 7.12, Tables 205–207, 217–219, 238–240, 244–246,
//! and 364. Source-derived coverage is not an official profile claim.

use crate::{
    DeriveKeyError, DeriveKeyRequest, DeriveKeyResponse, ExportError, ExportRequest,
    ExportResponse, ImportError, ImportRequest, ImportResponse, JoinSplitKeyError,
    JoinSplitKeyRequest, JoinSplitKeyResponse, KmipOperationResult, OperationData, ResultMessage,
    ResultReason, ResultStatus, SecretBytes,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

const SECRET_SENTINEL: &[u8] = b"KMIPKIT_KEY_TRANSFER_SECRET_SENTINEL";

#[test]
fn all_four_operation_models_and_payload_free_errors_are_public() {
    let _ = [
        std::any::type_name::<DeriveKeyRequest>(),
        std::any::type_name::<DeriveKeyResponse>(),
        std::any::type_name::<DeriveKeyError>(),
        std::any::type_name::<ExportRequest>(),
        std::any::type_name::<ExportResponse>(),
        std::any::type_name::<ExportError>(),
        std::any::type_name::<ImportRequest>(),
        std::any::type_name::<ImportResponse>(),
        std::any::type_name::<ImportError>(),
        std::any::type_name::<JoinSplitKeyRequest>(),
        std::any::type_name::<JoinSplitKeyResponse>(),
        std::any::type_name::<JoinSplitKeyError>(),
    ];
}

#[test]
fn unknown_result_values_and_untrusted_messages_remain_inspectable() {
    let status = ResultStatus::from_raw(0x8fff_fffe);
    let reason = ResultReason::from_raw(0x8fff_fffd);
    let message = ResultMessage::new("untrusted result text".to_owned());
    let result = KmipOperationResult::new(status, Some(reason), Some(message))
        .expect("an unknown status retains an open reason contract");

    assert_eq!(result.status().raw(), 0x8fff_fffe);
    assert_eq!(result.reason().map(ResultReason::raw), Some(0x8fff_fffd));
    assert_eq!(
        result.message().map(ResultMessage::as_str),
        Some("untrusted result text")
    );
}

#[test]
fn secret_operation_data_redacts_its_owned_bytes() {
    let data = OperationData::ByteString(SecretBytes::new(SECRET_SENTINEL.to_vec()));

    assert!(!format!("{data:?}").contains("KMIPKIT_KEY_TRANSFER_SECRET_SENTINEL"));
    assert!(!format!("{data}").contains("KMIPKIT_KEY_TRANSFER_SECRET_SENTINEL"));
    let value = data.into_ttlv_value();
    assert!(!format!("{value:?}").contains("KMIPKIT_KEY_TRANSFER_SECRET_SENTINEL"));
}

#[test]
fn generic_ttlv_retains_repeated_unknown_children_in_order() {
    let first_tag = RawTag::new(0x0054_1101)
        .expect("the vendor tag is in range")
        .try_checked()
        .expect("the vendor tag is structurally valid");
    let second_tag = RawTag::new(0x0054_1102)
        .expect("the vendor tag is in range")
        .try_checked()
        .expect("the vendor tag is structurally valid");
    let mut children = Structure::new();
    children
        .try_push(Item::new(first_tag, Value::enumeration(0xf000_0001)).expect("valid item"))
        .expect("first child fits");
    children
        .try_push(Item::new(second_tag, Value::integer(-7)).expect("valid item"))
        .expect("second child fits");
    children
        .try_push(Item::new(first_tag, Value::enumeration(0xf000_0002)).expect("valid item"))
        .expect("repeated child fits");

    let view = children.view();
    let entries = view.children();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].tag().raw(), 0x0054_1101);
    assert_eq!(entries[1].tag().raw(), 0x0054_1102);
    assert_eq!(entries[2].tag().raw(), 0x0054_1101);
}
