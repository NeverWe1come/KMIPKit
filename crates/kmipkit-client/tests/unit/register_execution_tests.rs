#![cfg(test)]

//! Fake-transport Register contracts from OASIS KMIP v2.1 §6.1.43,
//! Tables 291–294. Traceability: all `KMIPKIT-REQ-SPEC-6.1.43-*` rows.
//! These are derived execution tests, not official fixture passes.

use kmipkit_protocol::{
    AttributeSet, CheckRequest, ObjectType, OpaqueObjectContext, RegisterRequest,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const REGISTER_OPERATION: u32 = 0x0000_0003;
const CHECK_OPERATION: u32 = 0x0000_0009;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OPAQUE_OBJECT_TYPE: u32 = 8;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const INVALID_OBJECT_TYPE: u32 = 0x0000_0038;
const PENDING_CORRELATION: &[u8] = b"REGISTER_PENDING_CORRELATION\xFF";

fn register_request() -> RegisterRequest {
    RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([test_item(CERTIFICATE_LENGTH, Value::integer(512))])
            .expect("Table 293 certificate attribute is correctly typed"),
        test_item(
            CERTIFICATE,
            Value::structure(test_structure([test_item(
                UNKNOWN_VENDOR_TAG,
                Value::byte_string(b"certificate-value".to_vec()),
            )])),
        ),
    )
}

fn managed_opaque_register_request(attributes: AttributeSet) -> RegisterRequest {
    RegisterRequest::new(
        ObjectType::from_raw(OPAQUE_OBJECT_TYPE),
        attributes,
        test_item(
            OPAQUE_OBJECT,
            Value::structure(test_structure([
                test_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001)),
                test_item(
                    OPAQUE_DATA_VALUE,
                    Value::byte_string(b"opaque-register-data".to_vec()),
                ),
            ])),
        ),
    )
    .with_opaque_object_context(OpaqueObjectContext::ManagedCryptographicObject)
}

fn request_fields(bytes: &[u8], item_index: usize) -> (Option<u32>, Vec<u32>, Vec<Vec<u32>>) {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(item_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(*raw),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("request contains a Request Payload");
            let (tags, nested) = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let tags = payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect();
                let nested = payload
                    .children()
                    .iter()
                    .filter_map(|field| {
                        field.with_value(|value| match value {
                            ValueView::Structure(structure) => Some(
                                structure
                                    .children()
                                    .iter()
                                    .map(|child| child.tag().raw())
                                    .collect(),
                            ),
                            _ => None,
                        })
                    })
                    .collect();
                (tags, nested)
            });
            (operation, tags, nested)
        })
    })
}

#[derive(Debug, Eq, PartialEq)]
enum RegisterAttributeValue {
    Integer(i32),
    Enumeration(u32),
    Other,
}

fn register_attribute_values(
    bytes: &[u8],
    item_index: usize,
) -> Vec<(u32, RegisterAttributeValue)> {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(item_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("request contains a Request Payload");
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let attributes = payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == ATTRIBUTES)
                    .expect("Register Request Payload contains Attributes");
                attributes.with_value(|value| {
                    let ValueView::Structure(attributes) = value else {
                        panic!("Attributes is a Structure");
                    };
                    attributes
                        .children()
                        .iter()
                        .map(|field| {
                            let value = field.with_value(|value| match value {
                                ValueView::Integer(value) => {
                                    RegisterAttributeValue::Integer(*value)
                                }
                                ValueView::Enumeration(value) => {
                                    RegisterAttributeValue::Enumeration(*value)
                                }
                                _ => RegisterAttributeValue::Other,
                            });
                            (field.tag().raw(), value)
                        })
                        .collect()
                })
            })
        })
    })
}

#[test]
fn register_sends_table_291_and_exposes_table_292_identifier_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::register(register_request()),
        "Register",
    );
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-certificate".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .register(register_request(), &CodecLimits::defaults())
        .expect("valid Table 291 request receives typed Table 292 success");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Register,
        "Register(",
        "Register(",
    );
    let typed = outcome
        .response()
        .register()
        .expect("result retains its typed Register response");
    assert_eq!(outcome.operation(), ClientOperation::Register);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "registered-certificate".to_owned()
        ))
    );
    let request = captured.borrow();
    let request = request.as_deref().expect("fake transport captured request");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(
        operation,
        Some(REGISTER_OPERATION),
        "§11.36 assigns Register Operation 0x00000003"
    );
    assert_eq!(fields, [OBJECT_TYPE, ATTRIBUTES, CERTIFICATE]);
    assert_eq!(nested, [vec![CERTIFICATE_LENGTH], vec![UNKNOWN_VENDOR_TAG]]);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_produces_id_placeholder_for_a_later_identifier_omitting_operation() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Register(register_request()))
            .with_unique_batch_item_id(b"register-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(CheckRequest::new()))
            .with_unique_batch_item_id(b"check-item".to_vec()),
    ]);

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("fake transport stops before any request bytes are sent");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let request = captured.borrow();
    let request = request.as_deref().expect("request reaches fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(CHECK_OPERATION), Vec::new(), Vec::new()),
        "§4.1.2 permits Check to consume Register's returned ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_failure_preserves_table_294_reason_without_retry() {
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_OBJECT_TYPE),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let item = client
        .register(register_request(), &CodecLimits::defaults())
        .expect("Table 294 Operation Failed is preserved as a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(INVALID_OBJECT_TYPE)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Register(
        register_request(),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Register Pending");
    let item = response.get(0).expect("one Register result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Register,
        "Register(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("server Operation Pending remains Pending");
    };
    assert_eq!(pending.operation(), ClientOperation::Register);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_transport_failure_preserves_partial_write_delivery_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .register(register_request(), &CodecLimits::defaults())
        .expect_err("fake transport fails after partial request write");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

// OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Tables 291 and 293; §11.34, Table 468.
#[test]
fn managed_opaque_register_missing_direct_table_293_attribute_is_not_sent() {
    let attributes = AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("the supplied direct attributes are individually well-formed");
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(
            managed_opaque_register_request(attributes),
            &CodecLimits::defaults(),
        )
        .expect_err("managed Opaque Object requires direct Cryptographic Usage Mask");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(captured.borrow().is_none(), "no request bytes are captured");
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

// OASIS KMIP v2.1 §2.3 and §6.1.43, Table 291; local context never changes wire eligibility.
#[test]
fn selected_opaque_context_for_certificate_register_is_not_sent() {
    let request = register_request().with_opaque_object_context(OpaqueObjectContext::Generic);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(request, &CodecLimits::defaults())
        .expect_err("an Opaque Object context cannot be selected for a Certificate");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(captured.borrow().is_none(), "no request bytes are captured");
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

// OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Table 291; generic context has no Table 293 override.
#[test]
fn generic_opaque_register_sends_once_without_a_context_wire_marker() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(OPAQUE_OBJECT_TYPE),
        AttributeSet::try_new([]).expect("an empty generic attribute set is valid"),
        test_item(
            OPAQUE_OBJECT,
            Value::structure(test_structure([
                test_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001)),
                test_item(
                    OPAQUE_DATA_VALUE,
                    Value::byte_string(b"generic-opaque-data".to_vec()),
                ),
            ])),
        ),
    );
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-generic-opaque-object".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    client
        .register(request, &CodecLimits::defaults())
        .expect("default Generic Opaque Object Register is eligible for one exchange");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the fake transport captures the encoded Register request");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(operation, Some(REGISTER_OPERATION));
    assert_eq!(fields, [OBJECT_TYPE, ATTRIBUTES, OPAQUE_OBJECT]);
    assert_eq!(
        nested,
        [vec![], vec![OPAQUE_DATA_TYPE, OPAQUE_DATA_VALUE]],
        "the local context is absent and Table 4 children stay ordered"
    );
    assert!(register_attribute_values(request, 0).is_empty());
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(fake.borrow().write_call_count(), 1);
}

// OASIS KMIP v2.1 §2.3, Table 4 requires both Opaque Data Type and Opaque Data Value.
#[test]
fn malformed_table_4_opaque_object_is_not_sent() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(OPAQUE_OBJECT_TYPE),
        AttributeSet::try_new([]).expect("an empty generic attribute set is valid"),
        test_item(
            OPAQUE_OBJECT,
            Value::structure(test_structure([test_item(
                OPAQUE_DATA_TYPE,
                Value::enumeration(0x8000_0001),
            )])),
        ),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(request, &CodecLimits::defaults())
        .expect_err("Table 4 requires Opaque Data Value as well as Opaque Data Type");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(
        captured.borrow().is_none(),
        "malformed input is not encoded"
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

// OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Tables 291 and 293; §11.34, Table 468.
#[test]
fn managed_opaque_register_preserves_standard_table_291_wire_shape_and_attribute_order() {
    let attributes = AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("all direct Table 293 attributes are well-formed");
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-opaque-object".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 7],
    });

    client
        .register(
            managed_opaque_register_request(attributes),
            &CodecLimits::defaults(),
        )
        .expect("managed Opaque Object with direct Table 293 attributes succeeds");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the successful request is captured by the fake transport");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(operation, Some(REGISTER_OPERATION));
    assert_eq!(
        fields,
        [OBJECT_TYPE, ATTRIBUTES, OPAQUE_OBJECT],
        "Table 291 remains unchanged and carries no local context marker"
    );
    assert_eq!(
        nested,
        [
            vec![
                CRYPTOGRAPHIC_USAGE_MASK,
                CRYPTOGRAPHIC_ALGORITHM,
                CRYPTOGRAPHIC_LENGTH,
            ],
            vec![OPAQUE_DATA_TYPE, OPAQUE_DATA_VALUE],
        ],
        "direct attributes retain caller order and the object remains Table 4"
    );
    assert_eq!(
        register_attribute_values(request, 0),
        [
            (CRYPTOGRAPHIC_USAGE_MASK, RegisterAttributeValue::Integer(3),),
            (
                CRYPTOGRAPHIC_ALGORITHM,
                RegisterAttributeValue::Enumeration(3),
            ),
            (CRYPTOGRAPHIC_LENGTH, RegisterAttributeValue::Integer(256),),
        ],
        "the Table 293 direct attribute values and TTLV types are retained"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
