#![cfg(test)]

//! Fake-transport execution checks for OASIS KMIP v2.1 §6.1.20, Tables
//! 223–225, and §6.1.21, Tables 226–228. Attribute Reference encoding follows
//! §5.5, Table 161. Traceability: KMIPKIT-0016-FR-006/FR-007/FR-010/FR-012,
//! and SC-007. These are derived structural tests, not official conformance
//! cases.

use kmipkit_protocol::{
    AttributeReference, GetAttributeListRequest, GetAttributeListResponse, GetAttributesRequest,
    GetAttributesResponse, ResultReason,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, ItemType, Structure, StructureView, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest};

const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const GET_ATTRIBUTE_LIST_OPERATION: u32 = 0x0000_000C;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const ATTRIBUTES: u32 = 0x0042_0125;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;

const OBJECT_IDENTIFIER: &str = "object-id-17";
const VENDOR: &str = "KMIPKit_TestVendor";
const ATTRIBUTE_NAME_SENTINEL: &str = "Opaque.ExecutionAttribute";
const RESULT_MESSAGE_SENTINEL: &str = "attribute read rejected by server";
const REQUEST_BATCH_ID: &[u8] = b"attribute-read-batch-id";
const ASYNC_CORRELATION_SENTINEL: &[u8] = &[0xA5, 0x00, 0x5A, 0xFF];

// OASIS KMIP v2.1 §6.1.20/Table 225 lists Invalid Attribute plus all nine
// Table 228 reasons. Each operation table is exercised through its own client
// dispatch path so status, reason, message, and one-exchange behavior are
// verified for both typed response models.
const TABLE_225_REASONS: [u32; 10] = [
    0x2C, // Invalid Attribute
    0x37, // Object Not Found
    0x15, // Attestation Failed
    0x14, // Attestation Required
    0x08, // Feature Not Supported
    0x07, // Invalid Field
    0x04, // Invalid Message
    0x05, // Operation Not Supported
    0x0C, // Permission Denied
    0x02, // Response Too Large
];
const TABLE_228_REASONS: [u32; 9] = [
    0x37, // Object Not Found
    0x15, // Attestation Failed
    0x14, // Attestation Required
    0x08, // Feature Not Supported
    0x07, // Invalid Field
    0x04, // Invalid Message
    0x05, // Operation Not Supported
    0x0C, // Permission Denied
    0x02, // Response Too Large
];

fn attributes_response_payload() -> Structure {
    test_structure([
        test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        ),
        test_item(
            ATTRIBUTES,
            Value::structure(test_structure([test_item(
                0x0042_002F,
                Value::date_time(1_700_000_000),
            )])),
        ),
    ])
}

fn attribute_list_response_payload() -> Structure {
    test_structure([
        test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string(OBJECT_IDENTIFIER.to_owned()),
        ),
        test_item(
            ATTRIBUTE_REFERENCE,
            Value::structure(test_structure([
                test_item(VENDOR_IDENTIFICATION, Value::text_string(VENDOR.to_owned())),
                test_item(
                    ATTRIBUTE_NAME,
                    Value::text_string(ATTRIBUTE_NAME_SENTINEL.to_owned()),
                ),
            ])),
        ),
    ])
}

fn response_bytes(
    operation: u32,
    result_status: u32,
    result_reason: Option<u32>,
    result_message: Option<&str>,
    response_payload: Option<Structure>,
) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch = vec![
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(RESULT_STATUS, Value::enumeration(result_status)),
    ];
    if let Some(reason) = result_reason {
        batch.push(test_item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = result_message {
        batch.push(test_item(
            RESULT_MESSAGE,
            Value::text_string(message.to_owned()),
        ));
    }
    if let Some(payload) = response_payload {
        batch.push(test_item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }

    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(test_structure(batch))),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the response fixture has valid KMIP 2.1 TTLV framing")
}

fn pending_response_bytes(operation: u32, response_payload: Structure) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(
            UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(REQUEST_BATCH_ID.to_vec()),
        ),
        test_item(RESULT_STATUS, Value::enumeration(2)),
        test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(ASYNC_CORRELATION_SENTINEL.to_vec()),
        ),
        test_item(RESPONSE_PAYLOAD, Value::structure(response_payload)),
    ]);

    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(batch_item)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the Pending response fixture has valid KMIP 2.1 TTLV framing")
}

fn with_request_payload<R>(bytes: &[u8], callback: impl FnOnce(StructureView<'_>) -> R) -> R {
    let message = decode(bytes).expect("the captured request is valid TTLV");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("the request root is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|child| child.tag().raw() == BATCH_ITEM)
            .expect("the request has one Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the request Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|child| child.tag().raw() == REQUEST_PAYLOAD)
                .expect("the request has one Request Payload");
            payload.with_value(|value| match value {
                ValueView::Structure(payload) => callback(payload),
                _ => panic!("the Request Payload is a Structure"),
            })
        })
    })
}

fn item_fields(item: &Item) -> Option<Vec<(u32, ItemType)>> {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => Some(
            structure
                .children()
                .iter()
                .map(|child| (child.tag().raw(), child.item_type()))
                .collect(),
        ),
        _ => None,
    })
}

fn text_value(item: &Item) -> Option<String> {
    item.with_value(|value| match value {
        ValueView::TextString(value) => Some(value.to_owned()),
        _ => None,
    })
}

fn enumeration_value(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

fn completed_get_attributes(outcome: &ClientBatchOutcome) -> &GetAttributesResponse {
    assert_completed_read_outcome(outcome, ClientOperation::GetAttributes);
    let ClientBatchOutcome::GetAttributes(response) = outcome else {
        panic!("Get Attributes execution returns its typed response");
    };
    response
}

fn completed_get_attribute_list(outcome: &ClientBatchOutcome) -> &GetAttributeListResponse {
    assert_completed_read_outcome(outcome, ClientOperation::GetAttributeList);
    let ClientBatchOutcome::GetAttributeList(response) = outcome else {
        panic!("Get Attribute List execution returns its typed response");
    };
    response
}

fn assert_completed_read_outcome(outcome: &ClientBatchOutcome, operation: ClientOperation) {
    assert_eq!(outcome.operation(), operation);
    assert_eq!(outcome.asynchronous_correlation_value(), None);

    let response = outcome.response();
    assert_eq!(response.result(), outcome.result());
    assert!(response.supported_versions().is_none());
    assert!(response.discover_versions().is_none());
    assert!(response.create().is_none());
    assert!(response.create_key_pair().is_none());
    assert!(response.create_split_key().is_none());
    assert_eq!(
        response.add_attribute().is_some(),
        operation == ClientOperation::AddAttribute
    );
    assert_eq!(
        response.adjust_attribute().is_some(),
        operation == ClientOperation::AdjustAttribute
    );
    assert_eq!(
        response.delete_attribute().is_some(),
        operation == ClientOperation::DeleteAttribute
    );
    assert_eq!(
        response.modify_attribute().is_some(),
        operation == ClientOperation::ModifyAttribute
    );
    assert_eq!(
        response.set_attribute().is_some(),
        operation == ClientOperation::SetAttribute
    );
    assert_eq!(
        response.get_attributes().is_some(),
        operation == ClientOperation::GetAttributes
    );
    assert_eq!(
        response.get_attribute_list().is_some(),
        operation == ClientOperation::GetAttributeList
    );

    let outcome_debug = format!("{outcome:?}");
    let outcome_display = outcome.to_string();
    let response_debug = format!("{response:?}");
    for rendered in [&outcome_debug, &outcome_display, &response_debug] {
        assert!(rendered.contains(&format!("{operation:?}")));
    }
}

#[test]
fn get_attributes_encodes_selected_references_and_exchanges_once() {
    let request = GetAttributesRequest::try_new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        [
            AttributeReference::tag(0x0042_002F),
            AttributeReference::name(VENDOR, ATTRIBUTE_NAME_SENTINEL),
        ],
    )
    .expect("the request contains distinct Attribute References");
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response: response_bytes(
            GET_ATTRIBUTES_OPERATION,
            0,
            None,
            None,
            Some(attributes_response_payload()),
        ),
        request_write_chunks: Vec::new(),
    });

    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::get_attributes(request))),
            &CodecLimits::defaults(),
        )
        .expect("Table 224 success returns a typed Get Attributes response");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(result.len(), 1);
    let captured_request = captured_request.borrow();
    let wire = captured_request
        .as_ref()
        .expect("the fake transport captured the outbound request");
    with_request_payload(wire, |payload| {
        let fields = payload.children();
        assert_eq!(
            fields
                .iter()
                .map(|field| (field.tag().raw(), field.item_type()))
                .collect::<Vec<_>>(),
            [
                (UNIQUE_IDENTIFIER, ItemType::TextString),
                (ATTRIBUTE_REFERENCE, ItemType::Enumeration),
                (ATTRIBUTE_REFERENCE, ItemType::Structure),
            ],
            "Table 223 preserves the UID and both distinct references in order"
        );
        assert_eq!(text_value(&fields[0]).as_deref(), Some(OBJECT_IDENTIFIER));
        assert_eq!(enumeration_value(&fields[1]), Some(0x0042_002F));
        assert_eq!(
            item_fields(&fields[2]),
            Some(vec![
                (VENDOR_IDENTIFICATION, ItemType::TextString),
                (ATTRIBUTE_NAME, ItemType::TextString),
            ]),
            "Table 161 name form retains both fields"
        );
        fields[2].with_value(|value| match value {
            ValueView::Structure(name) => {
                assert_eq!(text_value(&name.children()[0]).as_deref(), Some(VENDOR));
                assert_eq!(
                    text_value(&name.children()[1]).as_deref(),
                    Some(ATTRIBUTE_NAME_SENTINEL)
                );
            }
            _ => panic!("name-form Attribute Reference is a Structure"),
        });
    });

    let response = completed_get_attributes(
        result
            .get(0)
            .expect("the response is associated with the single request")
            .outcome(),
    );
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
    assert_eq!(
        response
            .attributes()
            .expect("Table 224 success contains Attributes")
            .as_items()
            .len(),
        1
    );
}

#[test]
fn get_attribute_list_sends_only_uid_and_preserves_typed_response_once() {
    let request = GetAttributeListRequest::new(Some(OBJECT_IDENTIFIER.to_owned()));
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response: response_bytes(
            GET_ATTRIBUTE_LIST_OPERATION,
            0,
            None,
            None,
            Some(attribute_list_response_payload()),
        ),
        request_write_chunks: Vec::new(),
    });

    let result = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::get_attribute_list(
                request,
            ))),
            &CodecLimits::defaults(),
        )
        .expect("Table 227 success returns a typed Get Attribute List response");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(result.len(), 1);
    let captured_request = captured_request.borrow();
    let wire = captured_request
        .as_ref()
        .expect("the fake transport captured the outbound request");
    with_request_payload(wire, |payload| {
        assert_eq!(
            payload
                .children()
                .iter()
                .map(|field| (field.tag().raw(), field.item_type()))
                .collect::<Vec<_>>(),
            [(UNIQUE_IDENTIFIER, ItemType::TextString)],
            "Table 226 has no Attribute Reference request selector"
        );
        assert_eq!(
            text_value(&payload.children()[0]).as_deref(),
            Some(OBJECT_IDENTIFIER)
        );
    });

    let response = completed_get_attribute_list(
        result
            .get(0)
            .expect("the response is associated with the single request")
            .outcome(),
    );
    assert_eq!(response.result().status().raw(), 0);
    assert_eq!(response.unique_identifier(), Some(OBJECT_IDENTIFIER));
    assert_eq!(
        response.attribute_references(),
        Some(&[AttributeReference::name(VENDOR, ATTRIBUTE_NAME_SENTINEL)][..]),
        "Table 227 returns the full attribute-name list"
    );
}

#[test]
fn get_attributes_preserves_table_225_failures_and_does_not_retry() {
    for reason in TABLE_225_REASONS {
        let request = GetAttributesRequest::try_new(None, [])
            .expect("Table 223 permits a request for all attributes");
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(
                GET_ATTRIBUTES_OPERATION,
                1,
                Some(reason),
                Some(RESULT_MESSAGE_SENTINEL),
                None,
            ),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(ClientRequest::get_attributes(request))),
                &CodecLimits::defaults(),
            )
            .expect("Table 225 operation failures remain typed responses");
        let response = completed_get_attributes(
            result
                .get(0)
                .expect("one operation result is returned")
                .outcome(),
        );

        assert_eq!(fake.borrow().exchange_count(), 1, "reason {reason:#x}");
        assert_eq!(response.result().status().raw(), 1, "reason {reason:#x}");
        assert_eq!(
            response.result().reason().map(ResultReason::raw),
            Some(reason),
            "Table 225 reason {reason:#x} remains unchanged"
        );
        assert_eq!(
            response
                .result()
                .message()
                .map(kmipkit_protocol::ResultMessage::as_str),
            Some(RESULT_MESSAGE_SENTINEL),
            "Result Message remains unchanged for reason {reason:#x}"
        );
        assert!(response.attributes().is_none());
    }
}

#[test]
fn get_attribute_list_preserves_every_table_228_failure_result_without_retry() {
    for reason in TABLE_228_REASONS {
        let request = GetAttributeListRequest::new(None);
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(
                GET_ATTRIBUTE_LIST_OPERATION,
                1,
                Some(reason),
                Some(RESULT_MESSAGE_SENTINEL),
                None,
            ),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(ClientRequest::get_attribute_list(
                    request,
                ))),
                &CodecLimits::defaults(),
            )
            .expect("Table 228 failures remain typed responses");
        let response = completed_get_attribute_list(
            result
                .get(0)
                .expect("one operation result is returned")
                .outcome(),
        );

        assert_eq!(fake.borrow().exchange_count(), 1, "reason {reason:#x}");
        assert_eq!(response.result().status().raw(), 1, "reason {reason:#x}");
        assert_eq!(
            response.result().reason().map(ResultReason::raw),
            Some(reason),
            "Table 228 reason {reason:#x} remains unchanged"
        );
        assert_eq!(
            response
                .result()
                .message()
                .map(kmipkit_protocol::ResultMessage::as_str),
            Some(RESULT_MESSAGE_SENTINEL),
            "Result Message remains unchanged for reason {reason:#x}"
        );
        assert!(response.attribute_references().is_none());
    }
}

#[test]
fn pending_attribute_reads_preserve_typed_response_and_correlation_without_retry() {
    for (operation, request) in [
        (
            GET_ATTRIBUTES_OPERATION,
            ClientRequest::get_attributes(
                GetAttributesRequest::try_new(None, [])
                    .expect("the empty Attribute Reference list is valid"),
            ),
        ),
        (
            GET_ATTRIBUTE_LIST_OPERATION,
            ClientRequest::get_attribute_list(GetAttributeListRequest::new(None)),
        ),
    ] {
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: pending_response_bytes(
                operation,
                if operation == GET_ATTRIBUTES_OPERATION {
                    attributes_response_payload()
                } else {
                    attribute_list_response_payload()
                },
            ),
            request_write_chunks: Vec::new(),
        });
        let result = client
            .execute(
                ClientBatch::new(
                    ClientBatchItem::new(request)
                        .with_unique_batch_item_id(REQUEST_BATCH_ID.to_vec()),
                )
                .with_asynchronous_indicator(1),
                &CodecLimits::defaults(),
            )
            .expect("a permitted Pending result remains an explicit outcome");

        assert_eq!(
            fake.borrow().exchange_count(),
            1,
            "operation {operation:#x}"
        );
        let item = result.get(0).expect("one response is associated");
        assert_eq!(
            item.unique_batch_item_id(),
            Some(REQUEST_BATCH_ID),
            "the response remains associated with its request ID"
        );
        assert_eq!(
            item.outcome().asynchronous_correlation_value(),
            Some(ASYNC_CORRELATION_SENTINEL),
            "the opaque correlation value is preserved byte-for-byte"
        );
        let ClientBatchOutcome::Pending(pending) = item.outcome() else {
            panic!("an accepted Pending result must remain a Pending outcome");
        };
        assert_eq!(
            pending.asynchronous_correlation_value(),
            ASYNC_CORRELATION_SENTINEL,
            "the opaque correlation value is preserved byte-for-byte"
        );
        let response = item.outcome().response();
        assert_eq!(response.result().status().raw(), 2);
        if operation == GET_ATTRIBUTES_OPERATION {
            assert!(response.get_attributes().is_some());
            assert!(response.get_attribute_list().is_none());
        } else {
            assert!(response.get_attributes().is_none());
            assert!(response.get_attribute_list().is_some());
        }

        let rendered = format!("{:?}", item.outcome());
        let visible_correlation = format!("{ASYNC_CORRELATION_SENTINEL:?}");
        assert!(!rendered.contains("A5005AFF"));
        assert!(!rendered.contains(&visible_correlation));
        assert!(!format!("{}", item.outcome()).contains("A5005AFF"));
        assert!(!format!("{}", item.outcome()).contains(&visible_correlation));
    }
}
