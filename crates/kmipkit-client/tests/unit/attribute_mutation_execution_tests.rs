#![cfg(test)]

//! Fake-transport tests for client-initiated attribute mutations in OASIS KMIP
//! v2.1 §6.1.2 Tables 167–169 (Add), §6.1.3 Tables 170–172 (Adjust),
//! §6.1.13 Tables 202–204 (Delete), §6.1.34 Tables 265–267 (Modify), and
//! §6.1.51 Tables 322–324 (Set). Direct Current/New Attribute Items follow
//! §§5.6–5.7 Tables 162–163; Attribute Reference forms follow §5.5 Table 161;
//! Vendor Attribute follows §4.60 Table 150. Standard attribute mutation
//! policy is sourced from §4.1–§4.63 attribute tables and the generated
//! catalog policy (KMIPKIT-0016 FR-003–FR-005, FR-008–FR-010, FR-012, FR-015;
//! SC-003, SC-007, SC-008). These are derived project tests, not official
//! OASIS conformance-case claims.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    CurrentAttribute, DeleteAttributeRequest, ModifyAttributeRequest, NewAttribute, ResultReason,
    SetAttributeRequest,
};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchResponse, ClientErrorCategory, ClientRequest,
};

// Consume the same generated source policy used by KMIPKit-0016 T040. Keeping
// this test view tied to the generator output makes the rejection matrix
// exhaustive for its unconditional entries without copying policy metadata.
#[path = "../../../kmipkit-protocol/src/generated/attribute_policy.rs"]
mod generated_attribute_policy;

use generated_attribute_policy::{ATTRIBUTE_POLICIES, VENDOR_ATTRIBUTE_POLICY};

const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_0001;
const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;

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
const CURRENT_ATTRIBUTE: u32 = 0x0042_013C;
const NEW_ATTRIBUTE: u32 = 0x0042_013D;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const ADJUSTMENT_TYPE: u32 = 0x0042_0158;
const ADJUSTMENT_VALUE: u32 = 0x0042_0162;
const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const ATTRIBUTE_NAME: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE: u32 = 0x0042_000B;
const COMMENT: u32 = 0x0042_00FD;
const ACTIVATION_DATE: u32 = 0x0042_0001;
const USAGE_LIMITS: u32 = 0x0042_0095;
const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
const USAGE_LIMITS_TOTAL: u32 = 0x0042_0097;
const USAGE_LIMITS_UNIT: u32 = 0x0042_0098;
const ALLOCATION_VALID_UNKNOWN_TAG: u32 = 0x0054_0001;

const OBJECT_IDENTIFIER: &str = "object-id-17";
const VALUE_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_VALUE_SENTINEL";
const VENDOR_VALUE_SENTINEL: &[u8] = b"KMIPKIT_VENDOR_ATTRIBUTE_VALUE_SENTINEL";
const RESULT_MESSAGE_SENTINEL: &str = "attribute mutation rejected by server";
const VENDOR_CREATED_IDENTIFIER: &str = "y";
const OTHER_VENDOR_IDENTIFIER: &str = "KMIPKit_TestVendor";
const ATTRIBUTE_NAME_SENTINEL: &str = "Opaque.ExecutionAttribute";
const PERMISSION_DENIED: u32 = 0x0000_000C;

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits the 24-bit KMIP field")
        .try_checked()
        .expect("fixture tag is assigned or accepted by the KMIPKit allocation gate")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture Item uses an allocation-valid tag")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture Structure stays within the model depth limit");
    }
    structure
}

fn failure_response_bytes(operation: u32) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(RESULT_STATUS, Value::enumeration(1)),
        test_item(RESULT_REASON, Value::enumeration(PERMISSION_DENIED)),
        test_item(
            RESULT_MESSAGE,
            Value::text_string(RESULT_MESSAGE_SENTINEL.to_owned()),
        ),
    ]);

    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(batch)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("the server failure fixture is a valid KMIP 2.1 Response Message")
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
            .expect("the request contains one Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("the request Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|child| child.tag().raw() == REQUEST_PAYLOAD)
                .expect("the Batch Item contains a Request Payload");
            payload.with_value(|value| match value {
                ValueView::Structure(payload) => callback(payload),
                _ => panic!("the Request Payload is a Structure"),
            })
        })
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

fn wrapped_direct_item(field: &Item) -> Option<(u32, ItemType, Option<String>)> {
    field.with_value(|value| {
        let ValueView::Structure(wrapper) = value else {
            return None;
        };
        let [attribute] = wrapper.children() else {
            return None;
        };
        Some((
            attribute.tag().raw(),
            attribute.item_type(),
            text_value(attribute),
        ))
    })
}

fn request_field_tags(payload: &StructureView<'_>) -> Vec<(u32, ItemType)> {
    payload
        .children()
        .iter()
        .map(|field| (field.tag().raw(), field.item_type()))
        .collect()
}

fn run_once(
    request: ClientRequest,
    operation: u32,
) -> (
    ClientBatchResponse,
    Rc<RefCell<ScriptedTransport>>,
    Rc<RefCell<Option<zeroize::Zeroizing<Vec<u8>>>>>,
) {
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response: failure_response_bytes(operation),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &CodecLimits::defaults(),
        )
        .expect("the client preserves an authoritative server operation failure");
    (response, fake, captured_request)
}

fn assert_server_result(response: &ClientBatchResponse, expected_reason: u32, context: &str) {
    let item = response
        .get(0)
        .expect("the single request has one response");
    let response_view = item.outcome().response();
    let result = response_view.result();
    assert_eq!(result.status().raw(), 1, "{context}: Result Status");
    assert_eq!(
        result.reason().map(ResultReason::raw),
        Some(expected_reason),
        "{context}: Result Reason"
    );
    assert_eq!(
        result
            .message()
            .map(kmipkit_protocol::ResultMessage::as_str),
        Some(RESULT_MESSAGE_SENTINEL),
        "{context}: Result Message"
    );
}

fn assert_one_exchange(
    fake: &Rc<RefCell<ScriptedTransport>>,
    captured_request: &Rc<RefCell<Option<zeroize::Zeroizing<Vec<u8>>>>>,
    context: &str,
) {
    assert_eq!(fake.borrow().exchange_count(), 1, "{context}");
    assert!(fake.borrow().write_call_count() > 0, "{context}");
    assert!(
        captured_request.borrow().is_some(),
        "the allowed request reached the fake transport: {context}"
    );
}

fn assert_payload_free_not_sent(request: ClientRequest, operation: u32, context: &str) {
    let (mut client, fake, captured_request) = client_for(ExchangeScript::Success {
        response: failure_response_bytes(operation),
        request_write_chunks: Vec::new(),
    });
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &CodecLimits::defaults(),
        )
        .expect_err(context);

    assert_eq!(
        error.category(),
        ClientErrorCategory::Validation,
        "{context}"
    );
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::NotSent),
        "{context}"
    );
    assert_eq!(fake.borrow().exchange_count(), 0, "{context}");
    assert_eq!(fake.borrow().write_call_count(), 0, "{context}");
    assert!(captured_request.borrow().is_none(), "{context}");
    assert!(!error.to_string().contains(VALUE_SENTINEL), "{context}");
    assert!(!format!("{error:?}").contains(VALUE_SENTINEL), "{context}");
    assert!(
        !error
            .to_string()
            .contains("KMIPKIT_VENDOR_ATTRIBUTE_VALUE_SENTINEL"),
        "{context}"
    );
    assert!(
        !format!("{error:?}").contains("KMIPKIT_VENDOR_ATTRIBUTE_VALUE_SENTINEL"),
        "{context}"
    );
}

fn vendor_attribute(vendor: &str) -> Item {
    item(
        VENDOR_ATTRIBUTE,
        Value::structure(structure([
            item(VENDOR_IDENTIFICATION, Value::text_string(vendor.to_owned())),
            item(
                ATTRIBUTE_NAME,
                Value::text_string(ATTRIBUTE_NAME_SENTINEL.to_owned()),
            ),
            item(
                ATTRIBUTE_VALUE,
                Value::byte_string(VENDOR_VALUE_SENTINEL.to_vec()),
            ),
        ])),
    )
}

fn usage_limits_value() -> Item {
    // OASIS KMIP v2.1 §7.40 Table 392 requires Usage Limits Count in this
    // structure. Total and Unit are included as source-defined members too.
    item(
        USAGE_LIMITS,
        Value::structure(structure([
            item(USAGE_LIMITS_COUNT, Value::integer(3)),
            item(USAGE_LIMITS_TOTAL, Value::integer(9)),
            item(USAGE_LIMITS_UNIT, Value::enumeration(1)),
        ])),
    )
}

fn direct_comment() -> Item {
    item(COMMENT, Value::text_string(VALUE_SENTINEL.to_owned()))
}

fn assert_current_new_wrapper(
    field: &Item,
    expected_wrapper_tag: u32,
    expected_attribute_tag: u32,
    expected_text: &str,
) {
    assert_eq!(field.tag().raw(), expected_wrapper_tag);
    assert_eq!(field.item_type(), ItemType::Structure);
    assert_eq!(
        wrapped_direct_item(field),
        Some((
            expected_attribute_tag,
            ItemType::TextString,
            Some(expected_text.to_owned()),
        )),
        "the wrapper contains exactly one unchanged direct attribute Item"
    );
}

#[test]
fn add_and_set_send_the_new_attribute_as_one_unchanged_direct_item() {
    for (operation, request) in [
        (
            ADD_ATTRIBUTE_OPERATION,
            ClientRequest::add_attribute(AddAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(direct_comment()),
            )),
        ),
        (
            SET_ATTRIBUTE_OPERATION,
            ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(direct_comment()),
            )),
        ),
    ] {
        let (response, fake, captured_request) = run_once(request, operation);
        assert_one_exchange(
            &fake,
            &captured_request,
            "one explicit mutation is one exchange",
        );
        assert_server_result(&response, PERMISSION_DENIED, "server remains authoritative");

        let captured_request = captured_request.borrow();
        let wire = captured_request
            .as_ref()
            .expect("the fake transport captured the outbound request");
        with_request_payload(wire, |payload| {
            assert_eq!(
                request_field_tags(&payload),
                [
                    (UNIQUE_IDENTIFIER, ItemType::TextString),
                    (NEW_ATTRIBUTE, ItemType::Structure),
                ],
                "the operation contains only its ordered Table 167/322 fields"
            );
            assert_current_new_wrapper(
                &payload.children()[1],
                NEW_ATTRIBUTE,
                COMMENT,
                VALUE_SENTINEL,
            );
        });
    }
}

#[test]
fn modify_sends_both_current_and_new_values_as_distinct_direct_items() {
    let request = ClientRequest::modify_attribute(ModifyAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        Some(CurrentAttribute::new(item(
            COMMENT,
            Value::text_string("current caller value".to_owned()),
        ))),
        NewAttribute::new(item(
            COMMENT,
            Value::text_string("new caller value".to_owned()),
        )),
    ));
    let (response, fake, captured_request) = run_once(request, MODIFY_ATTRIBUTE_OPERATION);

    assert_one_exchange(
        &fake,
        &captured_request,
        "Modify Attribute sends one exchange",
    );
    assert_server_result(
        &response,
        PERMISSION_DENIED,
        "Modify server result is preserved",
    );
    let captured_request = captured_request.borrow();
    let wire = captured_request
        .as_ref()
        .expect("the fake transport captured the outbound request");
    with_request_payload(wire, |payload| {
        assert_eq!(
            request_field_tags(&payload),
            [
                (UNIQUE_IDENTIFIER, ItemType::TextString),
                (CURRENT_ATTRIBUTE, ItemType::Structure),
                (NEW_ATTRIBUTE, ItemType::Structure),
            ],
            "Table 265 keeps Current Attribute and New Attribute distinct and ordered"
        );
        assert_current_new_wrapper(
            &payload.children()[1],
            CURRENT_ATTRIBUTE,
            COMMENT,
            "current caller value",
        );
        assert_current_new_wrapper(
            &payload.children()[2],
            NEW_ATTRIBUTE,
            COMMENT,
            "new caller value",
        );
    });
}

fn attribute_reference_request_form(field: &Item) -> Option<Vec<(u32, ItemType)>> {
    field.with_value(|value| match value {
        ValueView::Structure(reference) => Some(
            reference
                .children()
                .iter()
                .map(|member| (member.tag().raw(), member.item_type()))
                .collect(),
        ),
        _ => None,
    })
}

fn attribute_reference_texts(field: &Item) -> Option<(String, String)> {
    field.with_value(|value| {
        let ValueView::Structure(reference) = value else {
            return None;
        };
        let [vendor_identification, attribute_name] = reference.children() else {
            return None;
        };
        Some((
            text_value(vendor_identification)?,
            text_value(attribute_name)?,
        ))
    })
}

#[test]
fn adjust_sends_both_tag_and_name_attribute_reference_forms_once() {
    let requests = [
        (
            AttributeReference::tag(COMMENT),
            ItemType::Enumeration,
            None,
        ),
        (
            AttributeReference::name(OTHER_VENDOR_IDENTIFIER, "Opaque.FutureCounter"),
            ItemType::Structure,
            Some(vec![
                (VENDOR_IDENTIFICATION, ItemType::TextString),
                (ATTRIBUTE_NAME, ItemType::TextString),
            ]),
        ),
    ];

    for (reference, reference_type, expected_name_form) in requests {
        let request = ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
            Some(OBJECT_IDENTIFIER.to_owned()),
            reference,
            AdjustmentType::from_raw(1),
            Some(Value::integer(2)),
        ));
        let (response, fake, captured_request) = run_once(request, ADJUST_ATTRIBUTE_OPERATION);
        assert_one_exchange(
            &fake,
            &captured_request,
            "Adjust Attribute makes one exchange",
        );
        assert_server_result(
            &response,
            PERMISSION_DENIED,
            "Adjust server result is preserved",
        );

        let captured_request = captured_request.borrow();
        let wire = captured_request
            .as_ref()
            .expect("the fake transport captured the outbound request");
        with_request_payload(wire, |payload| {
            assert_eq!(
                request_field_tags(&payload),
                [
                    (UNIQUE_IDENTIFIER, ItemType::TextString),
                    (ATTRIBUTE_REFERENCE, reference_type),
                    (ADJUSTMENT_TYPE, ItemType::Enumeration),
                    (ADJUSTMENT_VALUE, ItemType::Integer),
                ],
                "Table 170 preserves the Attribute Reference and parameter fields"
            );
            assert_eq!(
                attribute_reference_request_form(&payload.children()[1]),
                expected_name_form,
                "Table 161 retains the selected reference form"
            );
            if reference_type == ItemType::Structure {
                assert_eq!(
                    attribute_reference_texts(&payload.children()[1]),
                    Some((
                        OTHER_VENDOR_IDENTIFIER.to_owned(),
                        "Opaque.FutureCounter".to_owned(),
                    )),
                    "Table 161 preserves both name-form reference strings"
                );
            }
            if reference_type == ItemType::Enumeration {
                assert_eq!(enumeration_value(&payload.children()[1]), Some(COMMENT));
            }
        });
    }
}

#[test]
fn delete_sends_both_reference_forms_and_the_supplied_current_attribute() {
    let requests = [
        (
            Some(AttributeReference::tag(COMMENT)),
            ItemType::Enumeration,
            None,
        ),
        (
            Some(AttributeReference::name(
                OTHER_VENDOR_IDENTIFIER,
                "Opaque.FutureAttribute",
            )),
            ItemType::Structure,
            Some(vec![
                (VENDOR_IDENTIFICATION, ItemType::TextString),
                (ATTRIBUTE_NAME, ItemType::TextString),
            ]),
        ),
    ];

    for (reference, reference_type, expected_name_form) in requests {
        let request = ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(OBJECT_IDENTIFIER.to_owned()),
            Some(CurrentAttribute::new(direct_comment())),
            reference,
        ));
        let (response, fake, captured_request) = run_once(request, DELETE_ATTRIBUTE_OPERATION);
        assert_one_exchange(
            &fake,
            &captured_request,
            "Delete Attribute makes one exchange",
        );
        assert_server_result(
            &response,
            PERMISSION_DENIED,
            "Delete server result is preserved",
        );

        let captured_request = captured_request.borrow();
        let wire = captured_request
            .as_ref()
            .expect("the fake transport captured the outbound request");
        with_request_payload(wire, |payload| {
            assert_eq!(
                request_field_tags(&payload),
                [
                    (UNIQUE_IDENTIFIER, ItemType::TextString),
                    (CURRENT_ATTRIBUTE, ItemType::Structure),
                    (ATTRIBUTE_REFERENCE, reference_type),
                ],
                "Table 202 keeps both optional selectors and their order"
            );
            assert_current_new_wrapper(
                &payload.children()[1],
                CURRENT_ATTRIBUTE,
                COMMENT,
                VALUE_SENTINEL,
            );
            assert_eq!(
                attribute_reference_request_form(&payload.children()[2]),
                expected_name_form,
                "Table 161 retains the selected reference form"
            );
            if reference_type == ItemType::Structure {
                assert_eq!(
                    attribute_reference_texts(&payload.children()[2]),
                    Some((
                        OTHER_VENDOR_IDENTIFIER.to_owned(),
                        "Opaque.FutureAttribute".to_owned(),
                    )),
                    "Table 161 preserves both name-form reference strings"
                );
            }
            if reference_type == ItemType::Enumeration {
                assert_eq!(enumeration_value(&payload.children()[2]), Some(COMMENT));
            }
        });
    }
}

fn policy_value(policy_name: &str) -> Value {
    match policy_name {
        "Activation Date"
        | "Archive Date"
        | "Compromise Date"
        | "Compromise Occurrence Date"
        | "Deactivation Date"
        | "Destroy Date"
        | "Initial Date"
        | "Last Change Date"
        | "Original Creation Date"
        | "Process Start Date"
        | "Protect Stop Date" => Value::date_time(1_700_000_000),
        "Always Sensitive" | "Extractable" | "Fresh" | "Key Value Present"
        | "Never Extractable" | "Quantum Safe" | "Rotate Automatic" | "Rotate Latest"
        | "Sensitive" => Value::boolean(true),
        "Certificate Length"
        | "Cryptographic Length"
        | "Cryptographic Usage Mask"
        | "Protection Storage Mask"
        | "Rotate Generation" => Value::integer(1),
        "Certificate Type"
        | "Cryptographic Algorithm"
        | "Digital Signature Algorithm"
        | "Key Format Type"
        | "NIST Key Type"
        | "Object Type"
        | "Opaque Data Type"
        | "Protection Level"
        | "State" => Value::enumeration(1),
        "Lease Time" | "Protection Period" | "Rotate Interval" | "Rotate Offset" => {
            Value::interval(1)
        }
        "Rotate Date" => Value::date_time_extended(1_700_000_000_123_456),
        "Comment"
        | "Contact Information"
        | "Description"
        | "Object Group"
        | "PKCS#12 Friendly Name"
        | "Unique Identifier" => Value::text_string(VALUE_SENTINEL.to_owned()),
        "Short Unique Identifier" => Value::byte_string(VALUE_SENTINEL.as_bytes().to_vec()),
        "Alternative Name"
        | "Application Specific Information"
        | "Cryptographic Domain Parameters"
        | "Cryptographic Parameters"
        | "Digest"
        | "Key Value Location"
        | "Link"
        | "Name"
        | "Random Number Generator"
        | "Revocation Reason"
        | "Rotate Name"
        | "X.509 Certificate Identifier"
        | "X.509 Certificate Issuer"
        | "X.509 Certificate Subject" => Value::structure(Structure::new()),
        other => panic!("generated policy has an unmapped test value encoding: {other}"),
    }
}

#[derive(Clone, Copy)]
enum MutationKind {
    Add,
    Adjust,
    Modify,
    Set,
}

impl MutationKind {
    const fn operation(self) -> u32 {
        match self {
            Self::Add => ADD_ATTRIBUTE_OPERATION,
            Self::Adjust => ADJUST_ATTRIBUTE_OPERATION,
            Self::Modify => MODIFY_ATTRIBUTE_OPERATION,
            Self::Set => SET_ATTRIBUTE_OPERATION,
        }
    }

    fn request(self, policy: &generated_attribute_policy::AttributePolicy) -> ClientRequest {
        let current_or_new_item = || item(policy.tag, policy_value(policy.name));
        match self {
            Self::Add => ClientRequest::add_attribute(AddAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(current_or_new_item()),
            )),
            Self::Adjust => ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                AttributeReference::tag(policy.tag),
                AdjustmentType::from_raw(1),
                None,
            )),
            Self::Modify => ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(current_or_new_item())),
                NewAttribute::new(current_or_new_item()),
            )),
            Self::Set => ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(current_or_new_item()),
            )),
        }
    }
}

fn policy_context(policy: &generated_attribute_policy::AttributePolicy) -> String {
    let references = policy
        .source_refs
        .iter()
        .map(|source| format!("{} §{}", source.source_id, source.section))
        .collect::<Vec<_>>()
        .join(", ");
    let operation_rules = policy
        .source_operation_restrictions
        .iter()
        .map(|rule| {
            let citations = rule
                .source_refs
                .iter()
                .map(|source| format!("{} §{}", source.source_id, source.section))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{} ({citations})", rule.source_text)
        })
        .collect::<Vec<_>>()
        .join("; ");
    let conditional_rule_count = policy.source_conditional_rules.len();
    format!(
        "{} [{}; {}; {}], initially set by {}; {} conditional source rules; {}",
        policy.name,
        policy.element_id,
        policy.source_policy_table,
        references,
        policy.source_initially_set_by,
        conditional_rule_count,
        operation_rules
    )
}

#[test]
fn every_generated_unconditional_standard_policy_is_rejected_before_transport() {
    let non_modifiable = ATTRIBUTE_POLICIES
        .iter()
        .filter(|policy| policy.source_modifiable_by_client == "No")
        .collect::<Vec<_>>();
    assert!(
        !non_modifiable.is_empty(),
        "the generated policy matrix is present"
    );

    for policy in non_modifiable {
        let context = policy_context(policy);
        for mutation in [
            MutationKind::Add,
            MutationKind::Adjust,
            MutationKind::Modify,
            MutationKind::Set,
        ] {
            assert_payload_free_not_sent(
                mutation.request(policy),
                mutation.operation(),
                &format!("source-defined non-modifiable attribute: {context}"),
            );
        }
    }

    let non_deletable = ATTRIBUTE_POLICIES
        .iter()
        .filter(|policy| policy.source_deletable_by_client == "No")
        .collect::<Vec<_>>();
    assert!(
        !non_deletable.is_empty(),
        "the generated delete policy matrix is present"
    );
    for policy in non_deletable {
        let request = ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(OBJECT_IDENTIFIER.to_owned()),
            None,
            Some(AttributeReference::tag(policy.tag)),
        ));
        assert_payload_free_not_sent(
            request,
            DELETE_ATTRIBUTE_OPERATION,
            &format!(
                "source-defined non-deletable attribute: {}",
                policy_context(policy)
            ),
        );
    }
}

#[test]
fn explicit_source_operation_prohibitions_are_payload_free_not_sent() {
    // OASIS KMIP v2.1 §4.28 states that Key Value Present SHALL NOT be
    // modified by a client or server; §4.57 explicitly forbids changing State
    // with Modify Attribute. §4.30/Table 72 states Lease Time is read-only
    // and its generated operation restrictions cite §6.1.2, §6.1.3, and
    // §6.1.51 for Add, Adjust, and Set.
    let key_value_present = ATTRIBUTE_POLICIES
        .iter()
        .find(|policy| policy.name == "Key Value Present")
        .expect("Key Value Present policy comes from §4.28 Table 68");
    let state = ATTRIBUTE_POLICIES
        .iter()
        .find(|policy| policy.name == "State")
        .expect("State policy comes from §4.57 Table 140");
    let lease_time = ATTRIBUTE_POLICIES
        .iter()
        .find(|policy| policy.name == "Lease Time")
        .expect("Lease Time policy comes from §4.30 Table 72");

    assert_payload_free_not_sent(
        MutationKind::Modify.request(key_value_present),
        MODIFY_ATTRIBUTE_OPERATION,
        "§4.28 prohibits client modification of Key Value Present",
    );
    assert_payload_free_not_sent(
        MutationKind::Modify.request(state),
        MODIFY_ATTRIBUTE_OPERATION,
        "§4.57 prohibits changing State through Modify Attribute",
    );
    for mutation in [MutationKind::Add, MutationKind::Adjust, MutationKind::Set] {
        assert_payload_free_not_sent(
            mutation.request(lease_time),
            mutation.operation(),
            "§4.30 and its cited operation rules prohibit client changes to Lease Time",
        );
    }
}

#[test]
fn usage_limits_new_attribute_is_rejected_only_for_add_and_modify() {
    // §4.59 prohibits setting or modifying Usage Limits Count through Add and
    // Modify; §7.40 Table 392 requires Count in a Usage Limits value. The
    // Attribute remains server-authoritative for other operations.
    for operation in [ADD_ATTRIBUTE_OPERATION, MODIFY_ATTRIBUTE_OPERATION] {
        let new_attribute = NewAttribute::new(usage_limits_value());
        let request = if operation == ADD_ATTRIBUTE_OPERATION {
            ClientRequest::add_attribute(AddAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                new_attribute,
            ))
        } else {
            ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                None,
                new_attribute,
            ))
        };
        assert_payload_free_not_sent(
            request,
            operation,
            "§4.59/§7.40 Usage Limits Count Add/Modify prohibition",
        );
    }
}

#[test]
fn every_inspectable_vendor_attribute_y_value_and_name_reference_is_rejected() {
    assert!(VENDOR_ATTRIBUTE_POLICY.matches_vendor_identification("y"));
    assert_eq!(VENDOR_ATTRIBUTE_POLICY.equals, VENDOR_CREATED_IDENTIFIER);
    assert_eq!(VENDOR_ATTRIBUTE_POLICY.structure_table, "Table 150");
    assert_eq!(VENDOR_ATTRIBUTE_POLICY.source_refs[0].section, "4.60");
    assert_eq!(
        VENDOR_ATTRIBUTE_POLICY.member_element_id,
        "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-60-VENDOR-IDENTIFICATION"
    );
    assert_eq!(
        VENDOR_ATTRIBUTE_POLICY.source_indicates_origin,
        "server_created"
    );
    assert!(
        VENDOR_ATTRIBUTE_POLICY
            .source_text
            .contains("Vendor Identification")
    );
    assert_eq!(
        VENDOR_ATTRIBUTE_POLICY.prohibited_client_operations,
        [
            "created (provided during object creation)",
            "Set Attribute",
            "Add Attribute",
            "Adjust Attribute",
            "Modify Attribute",
            "Delete Attribute",
        ]
    );

    let vendor_new = || NewAttribute::new(vendor_attribute(VENDOR_CREATED_IDENTIFIER));
    let vendor_current = || CurrentAttribute::new(vendor_attribute(VENDOR_CREATED_IDENTIFIER));
    let cases = [
        (
            ClientRequest::add_attribute(AddAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                vendor_new(),
            )),
            ADD_ATTRIBUTE_OPERATION,
            "§4.60 Add Attribute supplied Vendor Attribute value with Vendor Identification y",
        ),
        (
            ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                vendor_new(),
            )),
            SET_ATTRIBUTE_OPERATION,
            "§4.60 Set Attribute supplied Vendor Attribute value with Vendor Identification y",
        ),
        (
            ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(vendor_current()),
                NewAttribute::new(direct_comment()),
            )),
            MODIFY_ATTRIBUTE_OPERATION,
            "§4.60 Modify Attribute Current Attribute with Vendor Identification y",
        ),
        (
            ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(direct_comment())),
                vendor_new(),
            )),
            MODIFY_ATTRIBUTE_OPERATION,
            "§4.60 Modify Attribute New Attribute with Vendor Identification y",
        ),
        (
            ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(vendor_current()),
                None,
            )),
            DELETE_ATTRIBUTE_OPERATION,
            "§4.60 Delete Attribute Current Attribute with Vendor Identification y",
        ),
        (
            ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                AttributeReference::name(VENDOR_CREATED_IDENTIFIER, ATTRIBUTE_NAME_SENTINEL),
                AdjustmentType::from_raw(1),
                None,
            )),
            ADJUST_ATTRIBUTE_OPERATION,
            "§4.60 Adjust Attribute name-form reference with Vendor Identification y",
        ),
        (
            ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                None,
                Some(AttributeReference::name(
                    VENDOR_CREATED_IDENTIFIER,
                    ATTRIBUTE_NAME_SENTINEL,
                )),
            )),
            DELETE_ATTRIBUTE_OPERATION,
            "§4.60 reference-only Delete Attribute name-form reference with Vendor Identification y",
        ),
    ];

    for (request, operation, context) in cases {
        assert_payload_free_not_sent(request, operation, context);
    }
}

#[test]
fn tag_references_unknown_names_and_non_y_vendor_attributes_are_server_authoritative() {
    // Table 161 tag form carries no Vendor Identification. The unknown tag is
    // in the §11.56 Table 487 Extensions range 0x540000–0x54FFFF and is
    // allocation-valid under KMIPKit-0004/ADR-0010, without a policy entry.
    let cases = [
        (
            ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                AttributeReference::tag(VENDOR_ATTRIBUTE),
                AdjustmentType::from_raw(1),
                None,
            )),
            ADJUST_ATTRIBUTE_OPERATION,
            "tag-form Adjust reference has no inspectable Vendor Identification",
        ),
        (
            ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                None,
                Some(AttributeReference::tag(VENDOR_ATTRIBUTE)),
            )),
            DELETE_ATTRIBUTE_OPERATION,
            "tag-form Delete reference has no inspectable Vendor Identification",
        ),
        (
            ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                AttributeReference::tag(ALLOCATION_VALID_UNKNOWN_TAG),
                AdjustmentType::from_raw(1),
                None,
            )),
            ADJUST_ATTRIBUTE_OPERATION,
            "an allocation-valid unknown tag is not locally prohibited",
        ),
        (
            ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(item(
                    ALLOCATION_VALID_UNKNOWN_TAG,
                    Value::text_string(VALUE_SENTINEL.to_owned()),
                )),
            )),
            SET_ATTRIBUTE_OPERATION,
            "an allocation-valid unknown attribute tag is not locally prohibited",
        ),
        (
            ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                AttributeReference::name("KMIPKit_FutureVendor", "Unknown.Future.Attribute"),
                AdjustmentType::from_raw(1),
                None,
            )),
            ADJUST_ATTRIBUTE_OPERATION,
            "unknown name-form attribute with vendor identifier other than y is server-authoritative",
        ),
        (
            ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                None,
                Some(AttributeReference::name(
                    "KMIPKit_FutureVendor",
                    "Unknown.Future.Attribute",
                )),
            )),
            DELETE_ATTRIBUTE_OPERATION,
            "unknown name-form Delete reference with vendor identifier other than y is server-authoritative",
        ),
        (
            ClientRequest::add_attribute(AddAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(vendor_attribute(OTHER_VENDOR_IDENTIFIER)),
            )),
            ADD_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit a supplied Vendor Attribute value with identifier x",
        ),
        (
            ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                NewAttribute::new(vendor_attribute(OTHER_VENDOR_IDENTIFIER)),
            )),
            SET_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit Set Attribute value with identifier x",
        ),
        (
            ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(vendor_attribute(
                    OTHER_VENDOR_IDENTIFIER,
                ))),
                NewAttribute::new(direct_comment()),
            )),
            MODIFY_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit Modify Current Attribute value with identifier x",
        ),
        (
            ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(direct_comment())),
                NewAttribute::new(vendor_attribute(OTHER_VENDOR_IDENTIFIER)),
            )),
            MODIFY_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit Modify New Attribute value with identifier x",
        ),
        (
            ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                Some(CurrentAttribute::new(vendor_attribute(
                    OTHER_VENDOR_IDENTIFIER,
                ))),
                None,
            )),
            DELETE_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit Delete Current Attribute value with identifier x",
        ),
        (
            ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                AttributeReference::name(OTHER_VENDOR_IDENTIFIER, "Opaque.FutureCounter"),
                AdjustmentType::from_raw(1),
                None,
            )),
            ADJUST_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit name-form Adjust reference with identifier x",
        ),
        (
            ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                Some(OBJECT_IDENTIFIER.to_owned()),
                None,
                Some(AttributeReference::name(
                    OTHER_VENDOR_IDENTIFIER,
                    "Opaque.FutureAttribute",
                )),
            )),
            DELETE_ATTRIBUTE_OPERATION,
            "§4.60 does not prohibit name-form Delete reference with identifier x",
        ),
    ];

    for (request, operation, context) in cases {
        let (response, fake, captured_request) = run_once(request, operation);
        assert_one_exchange(&fake, &captured_request, context);
        assert_server_result(&response, PERMISSION_DENIED, context);
    }
}

#[test]
fn remote_state_qualified_rules_are_not_preflighted() {
    // §4.1/Table 30 permits client changes to Activation Date only while the
    // object is Pre-Active. §4.59/Table 149 conditions Usage Limits mutation
    // on whether Get Usage Allocation has occurred. Both conditions require
    // server state; the client sends once and preserves the server response.
    let activation_policy = ATTRIBUTE_POLICIES
        .iter()
        .find(|policy| policy.tag == ACTIVATION_DATE)
        .expect("Activation Date policy is generated from §4.1 Table 30");
    assert_ne!(activation_policy.source_modifiable_by_client, "No");
    let activation = ClientRequest::modify_attribute(ModifyAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        None,
        NewAttribute::new(item(ACTIVATION_DATE, Value::date_time(1_700_000_000))),
    ));
    let (response, fake, captured_request) = run_once(activation, MODIFY_ATTRIBUTE_OPERATION);
    assert_one_exchange(
        &fake,
        &captured_request,
        "the Pre-Active condition is left to the server",
    );
    assert_server_result(
        &response,
        PERMISSION_DENIED,
        "Activation Date server result is preserved",
    );

    let usage_limits = ClientRequest::delete_attribute(DeleteAttributeRequest::new(
        Some(OBJECT_IDENTIFIER.to_owned()),
        None,
        Some(AttributeReference::tag(USAGE_LIMITS)),
    ));
    let (response, fake, captured_request) = run_once(usage_limits, DELETE_ATTRIBUTE_OPERATION);
    assert_one_exchange(
        &fake,
        &captured_request,
        "the Get Usage Allocation condition is left to the server",
    );
    assert_server_result(
        &response,
        PERMISSION_DENIED,
        "Usage Limits state-dependent server result is preserved",
    );
}

#[derive(Clone)]
struct RepeatingFailureTransport {
    response: Vec<u8>,
    exchange_count: Rc<Cell<usize>>,
    requests: Rc<RefCell<Vec<Vec<u8>>>>,
}

impl Transport for RepeatingFailureTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchange_count
            .set(self.exchange_count.get().saturating_add(1));
        self.requests.borrow_mut().push(request.to_vec());
        Ok(TransportResponse::new(self.response.clone()))
    }
}

#[test]
fn server_failures_do_not_trigger_retry_or_client_side_attribute_state_changes() {
    let cases: [(u32, Box<dyn Fn() -> ClientRequest>); 5] = [
        (
            ADD_ATTRIBUTE_OPERATION,
            Box::new(|| {
                ClientRequest::add_attribute(AddAttributeRequest::new(
                    Some(OBJECT_IDENTIFIER.to_owned()),
                    NewAttribute::new(direct_comment()),
                ))
            }),
        ),
        (
            ADJUST_ATTRIBUTE_OPERATION,
            Box::new(|| {
                ClientRequest::adjust_attribute(AdjustAttributeRequest::new(
                    Some(OBJECT_IDENTIFIER.to_owned()),
                    AttributeReference::tag(ACTIVATION_DATE),
                    AdjustmentType::from_raw(1),
                    Some(Value::date_time(1_700_000_000)),
                ))
            }),
        ),
        (
            DELETE_ATTRIBUTE_OPERATION,
            Box::new(|| {
                ClientRequest::delete_attribute(DeleteAttributeRequest::new(
                    Some(OBJECT_IDENTIFIER.to_owned()),
                    Some(CurrentAttribute::new(direct_comment())),
                    Some(AttributeReference::tag(COMMENT)),
                ))
            }),
        ),
        (
            MODIFY_ATTRIBUTE_OPERATION,
            Box::new(|| {
                ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                    Some(OBJECT_IDENTIFIER.to_owned()),
                    Some(CurrentAttribute::new(direct_comment())),
                    NewAttribute::new(item(
                        COMMENT,
                        Value::text_string("second caller value".to_owned()),
                    )),
                ))
            }),
        ),
        (
            SET_ATTRIBUTE_OPERATION,
            Box::new(|| {
                ClientRequest::set_attribute(SetAttributeRequest::new(
                    Some(OBJECT_IDENTIFIER.to_owned()),
                    NewAttribute::new(direct_comment()),
                ))
            }),
        ),
    ];

    for (operation, make_request) in cases {
        let exchange_count = Rc::new(Cell::new(0));
        let requests = Rc::new(RefCell::new(Vec::new()));
        let transport = RepeatingFailureTransport {
            response: failure_response_bytes(operation),
            exchange_count: Rc::clone(&exchange_count),
            requests: Rc::clone(&requests),
        };
        let mut client = crate::execute::Client::for_test(transport);

        for _ in 0..2 {
            let result = client
                .execute(
                    ClientBatch::new(ClientBatchItem::new(make_request()))
                        .with_request_time_stamp(1_700_000_000),
                    &CodecLimits::defaults(),
                )
                .expect("an explicit server failure is not retried or applied locally");
            assert_server_result(
                &result,
                PERMISSION_DENIED,
                "server error remains authoritative",
            );
        }

        assert_eq!(
            exchange_count.get(),
            2,
            "operation {operation:#x}: one exchange for each explicit call"
        );
        let requests = requests.borrow();
        assert_eq!(requests.len(), 2, "operation {operation:#x}");
        assert_eq!(
            requests[0], requests[1],
            "operation {operation:#x}: a server failure does not change the next caller-supplied attribute request"
        );
    }
}
