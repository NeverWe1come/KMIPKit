#![cfg(test)]

//! Derived malformed-response and bounded-decoding tests for the seven
//! attribute operations in OASIS KMIP Specification v2.1: Add Attribute
//! §6.1.2 Tables 167–169; Adjust Attribute §6.1.3 Tables 170–172; Delete
//! Attribute §6.1.13 Tables 202–204; Get Attributes §6.1.20 Tables 223–225;
//! Get Attribute List §6.1.21 Tables 226–228; Modify Attribute §6.1.34
//! Tables 265–267; and Set Attribute §6.1.51 Tables 322–324.
//!
//! Traceability: KMIPKIT-0016-FR-013 and SC-002. These derived tests do not
//! claim that an official OASIS conformance case passed.

use crate::{
    AddAttributeResponse, AdjustAttributeResponse, DeleteAttributeResponse,
    GetAttributeListResponse, GetAttributesResponse, ModifyAttributeResponse,
    ResponseBatchItemView, ResponseMessage, SetAttributeResponse,
};
use kmipkit_ttlv::codec::{CodecLimits, DecodeError, DecodeErrorKind, decode_with_limits};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const ADD_ATTRIBUTE: u32 = 0x0000_000D;
const ADJUST_ATTRIBUTE: u32 = 0x0000_0030;
const DELETE_ATTRIBUTE: u32 = 0x0000_000F;
const GET_ATTRIBUTES: u32 = 0x0000_000B;
const GET_ATTRIBUTE_LIST: u32 = 0x0000_000C;
const MODIFY_ATTRIBUTE: u32 = 0x0000_000E;
const SET_ATTRIBUTE: u32 = 0x0000_0031;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_MESSAGE: u32 = 0x0042_007B;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const ATTRIBUTES: u32 = 0x0042_0125;
const ATTRIBUTE_REFERENCE: u32 = 0x0042_013B;
const ATTRIBUTE_VALUE: u32 = 0x0042_000B;
const RESERVED_TAG: u32 = 0x0042_0009;
const SECRET_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_SECRET_SENTINEL";
const OBJECT_IDENTIFIER: &str = "attribute-object-17";

const ATTRIBUTE_OPERATIONS: [(&str, u32); 7] = [
    ("Add Attribute", ADD_ATTRIBUTE),
    ("Adjust Attribute", ADJUST_ATTRIBUTE),
    ("Delete Attribute", DELETE_ATTRIBUTE),
    ("Get Attributes", GET_ATTRIBUTES),
    ("Get Attribute List", GET_ATTRIBUTE_LIST),
    ("Modify Attribute", MODIFY_ATTRIBUTE),
    ("Set Attribute", SET_ATTRIBUTE),
];

fn checked_tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits the 24-bit field")
        .try_checked()
        .expect("fixture uses an allocated KMIP 2.1 tag")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(checked_tag(raw_tag), value).expect("fixture item satisfies model constraints")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture stays inside the model depth limit");
    }
    structure
}

fn typed_response_error(
    operation: u32,
    item: ResponseBatchItemView<'_>,
) -> Result<(), (String, String)> {
    match operation {
        ADD_ATTRIBUTE => safe_conversion(AddAttributeResponse::try_from_response_item(item)),
        ADJUST_ATTRIBUTE => safe_conversion(AdjustAttributeResponse::try_from_response_item(item)),
        DELETE_ATTRIBUTE => safe_conversion(DeleteAttributeResponse::try_from_response_item(item)),
        GET_ATTRIBUTES => safe_conversion(GetAttributesResponse::try_from_response_item(item)),
        GET_ATTRIBUTE_LIST => {
            safe_conversion(GetAttributeListResponse::try_from_response_item(item))
        }
        MODIFY_ATTRIBUTE => safe_conversion(ModifyAttributeResponse::try_from_response_item(item)),
        SET_ATTRIBUTE => safe_conversion(SetAttributeResponse::try_from_response_item(item)),
        _ => Err((
            "fixture names an unsupported operation".to_owned(),
            String::new(),
        )),
    }
}

fn safe_conversion<T, E: std::fmt::Debug + std::fmt::Display>(
    result: Result<T, E>,
) -> Result<(), (String, String)> {
    result
        .map(|_| ())
        .map_err(|error| (error.to_string(), format!("{error:?}")))
}

fn response_message(operation: u32, payload: Structure) -> ResponseMessage {
    let protocol_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(protocol_version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(0)),
        item(RESPONSE_PAYLOAD, Value::structure(payload)),
    ]);
    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]))
    .expect("fixture is a valid KMIP 2.1 Response Message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one Response Batch Item")
}

fn raw_item(raw_tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let value_length = u32::try_from(value.len()).expect("fixture value fits the TTLV length");
    let tag_bytes = raw_tag.to_be_bytes();
    let mut encoded = Vec::with_capacity(8 + value.len() + 7);
    encoded.extend_from_slice(&tag_bytes[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(&value_length.to_be_bytes());
    encoded.extend_from_slice(value);
    let padding_length = (8 - (value.len() % 8)) % 8;
    encoded.resize(encoded.len() + padding_length, 0);
    encoded
}

fn raw_structure(raw_tag: u32, children: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
    let value = children.into_iter().flatten().collect::<Vec<_>>();
    raw_item(raw_tag, 0x01, &value)
}

fn raw_integer(raw_tag: u32, value: u32) -> Vec<u8> {
    raw_item(raw_tag, 0x02, &value.to_be_bytes())
}

fn raw_enumeration(raw_tag: u32, value: u32) -> Vec<u8> {
    raw_item(raw_tag, 0x05, &value.to_be_bytes())
}

fn raw_text(raw_tag: u32, value: &[u8]) -> Vec<u8> {
    raw_item(raw_tag, 0x07, value)
}

fn raw_response_wire(
    operation: u32,
    include_reserved_tag: bool,
    result_message: Option<&[u8]>,
) -> Vec<u8> {
    let protocol_version = raw_structure(
        PROTOCOL_VERSION,
        [
            raw_integer(PROTOCOL_VERSION_MAJOR, 2),
            raw_integer(PROTOCOL_VERSION_MINOR, 1),
        ],
    );
    let header = raw_structure(
        RESPONSE_HEADER,
        [
            protocol_version,
            raw_item(TIME_STAMP, 0x09, &1_i64.to_be_bytes()),
            raw_integer(BATCH_COUNT, 1),
        ],
    );

    let mut batch_fields = vec![
        raw_enumeration(OPERATION, operation),
        raw_enumeration(RESULT_STATUS, u32::from(result_message.is_some())),
    ];
    if let Some(result_message) = result_message {
        batch_fields.push(raw_enumeration(RESULT_REASON, 0x0000_0007));
        batch_fields.push(raw_text(RESULT_MESSAGE, result_message));
    } else {
        let mut payload_fields = vec![raw_text(UNIQUE_IDENTIFIER, OBJECT_IDENTIFIER.as_bytes())];
        match operation {
            GET_ATTRIBUTES => payload_fields.push(raw_structure(ATTRIBUTES, [])),
            GET_ATTRIBUTE_LIST => {
                payload_fields.push(raw_enumeration(ATTRIBUTE_REFERENCE, 0x0042_002F));
            }
            _ => {}
        }
        if include_reserved_tag {
            payload_fields.push(raw_item(RESERVED_TAG, 0x08, SECRET_SENTINEL.as_bytes()));
        }
        batch_fields.push(raw_structure(RESPONSE_PAYLOAD, payload_fields));
    }
    let batch_item = raw_structure(BATCH_ITEM, batch_fields);
    raw_structure(RESPONSE_MESSAGE, [header, batch_item])
}

fn assert_payload_free_decode_error(
    error: DecodeError,
    expected_kind: DecodeErrorKind,
    case_name: &str,
) {
    assert_eq!(error.kind(), expected_kind, "{case_name}");
    let diagnostics = format!("{error} {error:?}");
    assert!(
        !diagnostics.contains(SECRET_SENTINEL),
        "decoder errors must not expose peer payloads for {case_name}"
    );
}

#[test]
fn malformed_typed_response_payloads_return_only_safe_errors_for_all_operations() {
    // Tables 168, 171, 203, 224, 227, 266, and 323 define successful response
    // payloads. Each bad payload has a wrong-typed Unique Identifier and a
    // secret-bearing unexpected Attribute Value; conversion must return Err,
    // never a partial operation-specific response.
    for (name, operation) in ATTRIBUTE_OPERATIONS {
        let payload = structure([
            item(UNIQUE_IDENTIFIER, Value::integer(17)),
            item(
                ATTRIBUTE_VALUE,
                Value::byte_string(SECRET_SENTINEL.as_bytes().to_vec()),
            ),
        ]);
        let message = response_message(operation, payload);
        let error = typed_response_error(operation, response_item(&message))
            .expect_err("malformed success payload must not produce a typed response");
        let diagnostics = format!("{} {}", error.0, error.1);
        assert!(
            !diagnostics.contains(SECRET_SENTINEL),
            "typed conversion errors must not include attribute values for {name}"
        );
    }
}

#[test]
fn truncated_ttlv_response_frames_are_rejected_payload_free_for_all_operations() {
    for (name, operation) in ATTRIBUTE_OPERATIONS {
        let mut wire = raw_response_wire(operation, false, None);
        wire.pop()
            .expect("the encoded Response Message is nonempty");
        let error = decode_with_limits(&wire, &CodecLimits::defaults())
            .expect_err("a response whose final byte is missing is malformed");
        assert_payload_free_decode_error(error, DecodeErrorKind::TruncatedValue, name);
    }
}

#[test]
fn configured_ttlv_limits_reject_all_seven_response_shapes_before_returning_a_tree() {
    // KMIPKit's accepted decoder defaults are 16 MiB, depth 64, and 100,000
    // Items (AGENTS.md §8; KMIPKIT-0016-FR-013). Lowered per-call limits are
    // checked while decoding, before an Item tree can be returned.
    let oversized_result_message = SECRET_SENTINEL.repeat(64);

    for (name, operation) in ATTRIBUTE_OPERATIONS {
        let oversized_wire =
            raw_response_wire(operation, false, Some(oversized_result_message.as_bytes()));
        let byte_limit = CodecLimits::new(256, 64, 100_000)
            .expect("the response test uses a valid restrictive byte limit");
        let error = decode_with_limits(&oversized_wire, &byte_limit)
            .expect_err("the configured byte limit is checked before tree construction");
        assert_payload_free_decode_error(error, DecodeErrorKind::MessageTooLarge, name);

        let nested_wire = raw_response_wire(operation, false, Some(b"bounded result"));
        let depth_limit = CodecLimits::new(nested_wire.len(), 2, 100_000)
            .expect("the lowered Structure depth is supported");
        let error = decode_with_limits(&nested_wire, &depth_limit)
            .expect_err("the configured Structure depth rejects the nested Response Header");
        assert_payload_free_decode_error(error, DecodeErrorKind::StructureDepthExceeded, name);

        let element_limit = CodecLimits::new(nested_wire.len(), 64, 1)
            .expect("the lowered Item count is supported");
        let error = decode_with_limits(&nested_wire, &element_limit).expect_err(
            "the configured Item count rejects the first child before returning a tree",
        );
        assert_payload_free_decode_error(error, DecodeErrorKind::ElementLimitExceeded, name);
    }
}

#[test]
fn received_reserved_tags_are_rejected_before_item_construction_for_all_operations() {
    // OASIS KMIP Specification v2.1 Chapter 11 introduction and §11.56,
    // Table 487, classify Tag 0x420009 as Reserved and prohibit implementation
    // use. Rejection on receipt is the accepted KMIPKit policy in ADR-0011;
    // this test does not attribute decoder rejection to OASIS.
    for (name, operation) in ATTRIBUTE_OPERATIONS {
        let wire = raw_response_wire(operation, true, None);
        let error = decode_with_limits(&wire, &CodecLimits::defaults())
            .expect_err("ADR-0011 rejects the peer's Reserved Tag before returning an Item tree");
        assert_payload_free_decode_error(error, DecodeErrorKind::ReservedTag, name);
    }
}
