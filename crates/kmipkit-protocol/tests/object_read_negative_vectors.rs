//! Negative Get and Locate vectors derived from OASIS KMIP v2.1.
//!
//! Get and Locate payload shapes follow OASIS KMIP v2.1 §6.1.19 Table 221
//! and §6.1.28 Table 248. Structure fields follow their described order under
//! §10.1.2. Field presence and ordering apply KMIPKIT-0017-FR-002; sanitized Get
//! errors apply FR-004 and FR-012. Decoder bounds and payload-free diagnostics
//! apply FR-012 and the shared KMIPKIT-0005-FR-006–FR-009 decoder contract.
//! These are project-derived negative vectors, not official OASIS test cases.

use std::{error::Error, fmt::Write as _};

use kmipkit_protocol::{
    GetError, GetResponse, LocateError, LocateResponse, ResponseBatchItemView, ResponseMessage,
};
use kmipkit_ttlv::codec::{CodecLimits, DecodeErrorKind, decode_with_limits};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const GET_OPERATION: u32 = 0x0000_000A;
const LOCATE_OPERATION: u32 = 0x0000_0008;
const SUCCESS: u32 = 0;

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

const OBJECT_TYPE: u32 = 0x0042_0057;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_MATERIAL: u32 = 0x0042_0043;
const LOCATED_ITEMS: u32 = 0x0042_00D5;
const VENDOR_TEXT: u32 = 0x0054_1235;
const VENDOR_BYTES: u32 = 0x0054_1236;

const RAW_TEXT_SENTINEL: &str = "KMIPKIT_RAW_RESPONSE_TEXT_3da90f";
const SECRET_BYTES_SENTINEL: &[u8] = b"KMIPKIT_SECRET_BYTES_7c218e";
const TRUNCATED_TAIL: &[u8] = b"TTLVEND!";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture Tag fits the KMIP Tag width")
        .try_checked()
        .expect("fixture Tag is assigned or in the allocated vendor range")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("fixture value forms a TTLV Item")
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

fn response_message(operation: u32, payload: Option<Structure>) -> ResponseMessage {
    let protocol_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let response_header = structure([
        item(PROTOCOL_VERSION, Value::structure(protocol_version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut batch_fields = vec![
        item(OPERATION, Value::enumeration(operation)),
        item(RESULT_STATUS, Value::enumeration(SUCCESS)),
    ];
    if let Some(payload) = payload {
        batch_fields.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }

    ResponseMessage::try_from_ttlv(structure([
        item(RESPONSE_HEADER, Value::structure(response_header)),
        item(BATCH_ITEM, Value::structure(structure(batch_fields))),
    ]))
    .expect("fixture is a structurally valid KMIP 2.1 response message")
}

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn get_response(payload: Option<Structure>) -> Result<GetResponse, GetError> {
    let message = response_message(GET_OPERATION, payload);
    GetResponse::try_from_response_item(response_item(&message))
}

fn locate_response(payload: Option<Structure>) -> Result<LocateResponse, LocateError> {
    let message = response_message(LOCATE_OPERATION, payload);
    LocateResponse::try_from_response_item(response_item(&message))
}

fn unique_identifier(value: Value) -> Item {
    item(UNIQUE_IDENTIFIER, value)
}

fn secret_object() -> Item {
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(
            KEY_BLOCK,
            Value::structure(structure([item(
                KEY_MATERIAL,
                Value::structure(structure([
                    item(
                        VENDOR_TEXT,
                        Value::text_string(RAW_TEXT_SENTINEL.to_owned()),
                    ),
                    item(
                        VENDOR_BYTES,
                        Value::byte_string(SECRET_BYTES_SENTINEL.to_vec()),
                    ),
                ])),
            )])),
        )])),
    )
}

fn diagnostics(error: &dyn Error) -> String {
    let mut output = String::new();
    let mut cause = Some(error);
    while let Some(current) = cause {
        let _ = write!(output, "debug={current:?}; display={current}; ");
        cause = current.source();
    }
    output
}

fn assert_sanitized(error: &dyn Error) {
    let diagnostic = diagnostics(error);
    let byte_text = String::from_utf8_lossy(SECRET_BYTES_SENTINEL);
    let mut byte_hex = String::with_capacity(SECRET_BYTES_SENTINEL.len() * 2);
    for byte in SECRET_BYTES_SENTINEL {
        let _ = write!(byte_hex, "{byte:02x}");
    }
    let byte_decimals = SECRET_BYTES_SENTINEL
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    assert!(!diagnostic.contains(RAW_TEXT_SENTINEL));
    assert!(!diagnostic.contains(byte_text.as_ref()));
    assert!(!diagnostic.contains(&byte_hex));
    assert!(!diagnostic.contains(&byte_decimals));
}

#[test]
fn malformed_get_fields_reject_with_payload_free_diagnostics() {
    // Existing Get vectors check these shapes with `is_err`; these vectors
    // strengthen the same Table 221 cases by checking every exposed diagnostic.
    let valid_type = || item(OBJECT_TYPE, Value::enumeration(2));
    let payloads = [
        structure([
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
            secret_object(),
        ]),
        structure([
            valid_type(),
            item(OBJECT_TYPE, Value::enumeration(3)),
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
            secret_object(),
        ]),
        structure([
            valid_type(),
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
            unique_identifier(Value::text_string("duplicate-id".to_owned())),
            secret_object(),
        ]),
        structure([
            item(
                OBJECT_TYPE,
                Value::text_string(RAW_TEXT_SENTINEL.to_owned()),
            ),
            unique_identifier(Value::text_string("typed-id".to_owned())),
            secret_object(),
        ]),
        structure([
            valid_type(),
            unique_identifier(Value::boolean(true)),
            secret_object(),
        ]),
        structure([
            valid_type(),
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
            item(
                SYMMETRIC_KEY,
                Value::byte_string(SECRET_BYTES_SENTINEL.to_vec()),
            ),
        ]),
    ];

    for payload in payloads {
        let error = get_response(Some(payload))
            .expect_err("malformed or incomplete Table 221 fields are rejected");
        assert_eq!(error, GetError::MalformedSuccessPayload);
        assert_sanitized(&error);
    }
}

#[test]
fn get_success_payload_rejects_table_221_fields_out_of_order() {
    // OASIS KMIP v2.1 §10.1.2 requires Structure fields in description order;
    // §6.1.19 Table 221 orders Object Type, Unique Identifier, then Any Object.
    let payload = structure([
        item(OBJECT_TYPE, Value::enumeration(2)),
        secret_object(),
        unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
    ]);
    let error = get_response(Some(payload))
        .expect_err("Get response fields must follow the Table 221 order");

    assert_eq!(error, GetError::MalformedSuccessPayload);
    assert_sanitized(&error);
}

#[test]
fn malformed_locate_fields_reject_with_payload_free_diagnostics() {
    // Table 248 permits an omitted Located Items field and repeated Unique
    // Identifiers. The malformed cases here target only the optional count's
    // duplicate/type/order constraints and the supported identifier forms.
    let payloads = [
        structure([
            item(LOCATED_ITEMS, Value::integer(2)),
            item(LOCATED_ITEMS, Value::integer(3)),
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
        ]),
        structure([
            item(
                LOCATED_ITEMS,
                Value::text_string(RAW_TEXT_SENTINEL.to_owned()),
            ),
            unique_identifier(Value::text_string("located-id".to_owned())),
        ]),
        structure([
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
            item(LOCATED_ITEMS, Value::integer(2)),
        ]),
        structure([unique_identifier(Value::boolean(true))]),
        structure([
            unique_identifier(Value::text_string(RAW_TEXT_SENTINEL.to_owned())),
            item(
                VENDOR_BYTES,
                Value::byte_string(SECRET_BYTES_SENTINEL.to_vec()),
            ),
        ]),
    ];

    for payload in payloads {
        let error =
            locate_response(Some(payload)).expect_err("malformed Table 248 fields are rejected");
        assert_eq!(error, LocateError::MalformedSuccessPayload);
        assert_sanitized(&error);
    }
}

fn wire_item(raw_tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut wire = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    wire.extend_from_slice(&raw_tag.to_be_bytes()[1..]);
    wire.push(item_type);
    wire.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test TTLV item value fits the wire length")
            .to_be_bytes(),
    );
    wire.extend_from_slice(value);
    let padding = (8 - value.len() % 8) % 8;
    wire.resize(wire.len() + padding, 0);
    wire
}

fn wire_structure(raw_tag: u32, children: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
    let value = children.into_iter().flatten().collect::<Vec<_>>();
    wire_item(raw_tag, 0x01, &value)
}

fn wire_integer(raw_tag: u32, value: i32) -> Vec<u8> {
    wire_item(raw_tag, 0x02, &value.to_be_bytes())
}

fn wire_enumeration(raw_tag: u32, value: u32) -> Vec<u8> {
    wire_item(raw_tag, 0x05, &value.to_be_bytes())
}

fn wire_text(raw_tag: u32, value: &[u8]) -> Vec<u8> {
    wire_item(raw_tag, 0x07, value)
}

fn wire_bytes(raw_tag: u32, value: &[u8]) -> Vec<u8> {
    wire_item(raw_tag, 0x08, value)
}

fn wire_date_time(raw_tag: u32, value: i64) -> Vec<u8> {
    wire_item(raw_tag, 0x09, &value.to_be_bytes())
}

fn response_wire(operation: u32, include_truncated_tail: bool) -> Vec<u8> {
    let protocol_version = wire_structure(
        PROTOCOL_VERSION,
        [
            wire_integer(PROTOCOL_VERSION_MAJOR, 2),
            wire_integer(PROTOCOL_VERSION_MINOR, 1),
        ],
    );
    let response_header = wire_structure(
        RESPONSE_HEADER,
        [
            protocol_version,
            wire_date_time(TIME_STAMP, 1),
            wire_integer(BATCH_COUNT, 1),
        ],
    );
    let mut payload_fields = if operation == GET_OPERATION {
        let key_block =
            wire_structure(KEY_BLOCK, [wire_bytes(KEY_MATERIAL, SECRET_BYTES_SENTINEL)]);
        let object = wire_structure(SYMMETRIC_KEY, [key_block]);
        vec![
            wire_enumeration(OBJECT_TYPE, 2),
            wire_text(UNIQUE_IDENTIFIER, RAW_TEXT_SENTINEL.as_bytes()),
            object,
        ]
    } else {
        vec![
            wire_integer(LOCATED_ITEMS, 1),
            wire_text(UNIQUE_IDENTIFIER, RAW_TEXT_SENTINEL.as_bytes()),
            wire_bytes(VENDOR_BYTES, SECRET_BYTES_SENTINEL),
        ]
    };
    if include_truncated_tail {
        payload_fields.push(wire_text(VENDOR_TEXT, TRUNCATED_TAIL));
    }
    let payload = wire_structure(RESPONSE_PAYLOAD, payload_fields);
    let batch_item = wire_structure(
        BATCH_ITEM,
        [
            wire_enumeration(OPERATION, operation),
            wire_enumeration(RESULT_STATUS, SUCCESS),
            payload,
        ],
    );

    wire_structure(0x0042_0078, [response_header, batch_item])
}

fn assert_wire_error(wire: &[u8], limits: &CodecLimits, expected: DecodeErrorKind) {
    let error = decode_with_limits(wire, limits).expect_err("negative TTLV vector is rejected");
    assert_eq!(error.kind(), expected);
    assert_sanitized(&error);
}

#[test]
fn truncated_and_malformed_get_and_locate_wire_vectors_are_sanitized() {
    // Generic TTLV structure and length rules come from the shared decoder
    // contract, KMIPKIT-0005-FR-006 and FR-008; they are not operation-specific.
    for operation in [GET_OPERATION, LOCATE_OPERATION] {
        let complete = response_wire(operation, true);
        let default_limits = CodecLimits::defaults();
        let mut truncated = complete.clone();
        truncated.truncate(truncated.len() - 1);
        assert_wire_error(&truncated, &default_limits, DecodeErrorKind::TruncatedValue);

        let mut malformed = response_wire(operation, false);
        let byte_tag = if operation == GET_OPERATION {
            KEY_MATERIAL
        } else {
            VENDOR_BYTES
        };
        let tag_bytes = byte_tag.to_be_bytes()[1..].to_vec();
        let item_start = malformed
            .windows(tag_bytes.len())
            .position(|window| window == tag_bytes)
            .expect("response wire includes the secret-bearing extension item");
        malformed[item_start + 3] = 0xFF;
        assert_wire_error(
            &malformed,
            &default_limits,
            DecodeErrorKind::UnsupportedItemType,
        );
    }
}

#[test]
fn configured_response_wire_byte_limit_rejects_get_and_locate_before_parsing() {
    // KMIPKIT-0005-FR-007 defines the shared configurable message-size limit;
    // KMIPKIT-0017-FR-012 applies it to object-reading response payloads.
    for operation in [GET_OPERATION, LOCATE_OPERATION] {
        let wire = response_wire(operation, false);
        let limits = CodecLimits::new(wire.len() - 1, 64, 100_000)
            .expect("lowered per-call byte limit is valid");
        assert_wire_error(&wire, &limits, DecodeErrorKind::MessageTooLarge);
    }
}

#[test]
fn configured_response_wire_depth_limit_rejects_get_and_locate() {
    for operation in [GET_OPERATION, LOCATE_OPERATION] {
        let wire = response_wire(operation, false);
        let limits = CodecLimits::new(wire.len(), 1, 100_000)
            .expect("lowered per-call Structure depth limit is valid");
        assert_wire_error(&wire, &limits, DecodeErrorKind::StructureDepthExceeded);
    }
}

#[test]
fn configured_response_wire_element_limit_rejects_get_and_locate() {
    for operation in [GET_OPERATION, LOCATE_OPERATION] {
        let wire = response_wire(operation, false);
        let limits =
            CodecLimits::new(wire.len(), 64, 1).expect("lowered per-call Item limit is valid");
        assert_wire_error(&wire, &limits, DecodeErrorKind::ElementLimitExceeded);
    }
}
