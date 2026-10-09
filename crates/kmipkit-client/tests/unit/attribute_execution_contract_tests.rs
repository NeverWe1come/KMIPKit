#![cfg(test)]

//! Cross-operation execution contracts for OASIS KMIP v2.1 §6.1.2, §6.1.3,
//! §6.1.13, §6.1.20, §6.1.21, §6.1.34, and §6.1.51. These derived fake-
//! transport tests verify KMIPKIT-0016 FR-012 and SC-007; they are not official
//! conformance vectors.

use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest, GetAttributesRequest,
    ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const ADD_ATTRIBUTE: u32 = 0x0000_000D;
const ADJUST_ATTRIBUTE: u32 = 0x0000_0030;
const DELETE_ATTRIBUTE: u32 = 0x0000_000F;
const GET_ATTRIBUTES: u32 = 0x0000_000B;
const GET_ATTRIBUTE_LIST: u32 = 0x0000_000C;
const MODIFY_ATTRIBUTE: u32 = 0x0000_000E;
const SET_ATTRIBUTE: u32 = 0x0000_0031;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007F;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

const UNIQUE_IDENTIFIER: &str = "attribute-contract-object";
const BATCH_ID: &[u8] = b"attribute-contract-batch-id";
const CORRELATION: &[u8] = b"ATTRIBUTE_CONTRACT_CORRELATION_SENTINEL";
const COMMENT: u32 = 0x0042_00FD;
const ACTIVATION_DATE: u32 = 0x0042_0001;

struct Case {
    operation: u32,
    client_operation: ClientOperation,
    request: ClientRequest,
}

fn assigned_item(tag: u32, value: Value) -> Item {
    let checked = RawTag::new(tag)
        .expect("fixture tag fits the KMIP tag field")
        .try_checked()
        .expect("fixture tag is assigned by KMIP 2.1");
    Item::new(checked, value).expect("fixture Item is structurally valid")
}

fn cases() -> [Case; 7] {
    let comment = || assigned_item(COMMENT, Value::text_string("caller comment".to_owned()));
    [
        Case {
            operation: ADD_ATTRIBUTE,
            client_operation: ClientOperation::AddAttribute,
            request: ClientRequest::add_attribute(AddAttributeRequest::new(
                Some(UNIQUE_IDENTIFIER.to_owned()),
                NewAttribute::new(comment()),
            )),
        },
        Case {
            operation: ADJUST_ATTRIBUTE,
            client_operation: ClientOperation::AdjustAttribute,
            request: ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(UNIQUE_IDENTIFIER.to_owned()),
                AttributeReference::tag(ACTIVATION_DATE),
                AdjustmentType::INCREMENT,
                None,
            )),
        },
        Case {
            operation: DELETE_ATTRIBUTE,
            client_operation: ClientOperation::DeleteAttribute,
            request: ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(UNIQUE_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(comment())),
                None,
            )),
        },
        Case {
            operation: GET_ATTRIBUTES,
            client_operation: ClientOperation::GetAttributes,
            request: ClientRequest::get_attributes(
                GetAttributesRequest::try_new(None, [])
                    .expect("an empty Attribute Reference list is valid"),
            ),
        },
        Case {
            operation: GET_ATTRIBUTE_LIST,
            client_operation: ClientOperation::GetAttributeList,
            request: ClientRequest::get_attribute_list(GetAttributeListRequest::new(None)),
        },
        Case {
            operation: MODIFY_ATTRIBUTE,
            client_operation: ClientOperation::ModifyAttribute,
            request: ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(UNIQUE_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(comment())),
                NewAttribute::new(comment()),
            )),
        },
        Case {
            operation: SET_ATTRIBUTE,
            client_operation: ClientOperation::SetAttribute,
            request: ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(UNIQUE_IDENTIFIER.to_owned()),
                NewAttribute::new(comment()),
            )),
        },
    ]
}

fn response_bytes(operation: u32) -> Vec<u8> {
    let response_item = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(BATCH_ID.to_vec())),
        test_item(RESULT_STATUS, Value::enumeration(2)),
        test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(CORRELATION.to_vec()),
        ),
        test_item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);

    encode_response_item(response_item)
}

fn malformed_success_response_bytes(operation: u32) -> Vec<u8> {
    let response_item = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(BATCH_ID.to_vec())),
        test_item(RESULT_STATUS, Value::enumeration(0)),
        test_item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
    ]);
    encode_response_item(response_item)
}

fn encode_response_item(response_item: Structure) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(response_item)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the Pending response obeys the common KMIP 2.1 message shape")
}

fn request_batch(request: ClientRequest) -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(request).with_unique_batch_item_id(BATCH_ID.to_vec()))
        .with_asynchronous_indicator(1)
}

fn response_matches(operation: ClientOperation, outcome: &ClientBatchOutcome) -> bool {
    let response = outcome.response();
    match operation {
        ClientOperation::AddAttribute => response.add_attribute().is_some(),
        ClientOperation::AdjustAttribute => response.adjust_attribute().is_some(),
        ClientOperation::DeleteAttribute => response.delete_attribute().is_some(),
        ClientOperation::GetAttributes => response.get_attributes().is_some(),
        ClientOperation::GetAttributeList => response.get_attribute_list().is_some(),
        ClientOperation::ModifyAttribute => response.modify_attribute().is_some(),
        ClientOperation::SetAttribute => response.set_attribute().is_some(),
        _ => false,
    }
}

#[test]
fn all_seven_attribute_operations_preserve_pending_results_and_do_not_retry() {
    for case in cases() {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(case.operation),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(request_batch(case.request), &CodecLimits::defaults())
            .expect("the client surfaces the server's Pending result");
        let item = result
            .get(0)
            .expect("the response is associated to one request");

        assert_eq!(
            fake.borrow().exchange_count(),
            1,
            "{:?}",
            case.client_operation
        );
        assert_eq!(item.unique_batch_item_id(), Some(BATCH_ID));
        let ClientBatchOutcome::Pending(pending) = item.outcome() else {
            panic!("the server Pending result remains pending");
        };
        assert_eq!(pending.operation(), case.client_operation);
        assert_eq!(pending.result().status().raw(), 2);
        assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
        assert!(response_matches(case.client_operation, item.outcome()));
        assert!(!format!("{pending:?}").contains("ATTRIBUTE_CONTRACT_CORRELATION_SENTINEL"));
    }
}

#[test]
fn all_seven_attribute_operations_report_partial_write_delivery_without_retry() {
    for case in cases() {
        let (mut client, fake, _) =
            client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
        let error = client
            .execute(request_batch(case.request), &CodecLimits::defaults())
            .expect_err("a partial request write is reported with delivery evidence");

        assert_eq!(error.category(), ClientErrorCategory::Transport);
        assert_eq!(
            error.delivery_state(),
            Some(RequestDeliveryState::PossiblySent),
            "{:?}",
            case.client_operation
        );
        assert_eq!(fake.borrow().exchange_count(), 1, "no operation is retried");
    }
}

#[test]
fn all_seven_attribute_operations_report_malformed_responses_after_one_exchange() {
    for case in cases() {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: malformed_success_response_bytes(case.operation),
            request_write_chunks: Vec::new(),
        });
        let error = client
            .execute(request_batch(case.request), &CodecLimits::defaults())
            .expect_err("an empty success payload violates each operation response table");

        assert_eq!(error.category(), ClientErrorCategory::Protocol);
        assert_eq!(
            error.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted),
            "{:?}",
            case.client_operation
        );
        assert_eq!(
            fake.borrow().exchange_count(),
            1,
            "malformed responses are not retried"
        );
    }
}
