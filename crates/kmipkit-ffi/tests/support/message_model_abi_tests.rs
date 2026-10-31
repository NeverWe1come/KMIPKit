use super::*;
use crate::kmipkit_protocol_version_v1_t;
use kmipkit_protocol::MessageExtensionView;
use kmipkit_ttlv::{StructureView, ValueView};
use serde_json::Value as JsonValue;

const SHARED_PARITY_FIXTURE: &str =
    include_str!("../../../../tests/fixtures/api/shared-capability-parity.json");

#[link(name = "kmipkit_message_model_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_message_model_c_consumer_validate_request(
        structure: *mut crate::kmipkit_ttlv_structure_t,
    ) -> i32;
    fn kmipkit_message_model_c_consumer_validate_response(
        structure: *mut crate::kmipkit_ttlv_structure_t,
    ) -> i32;
    fn kmipkit_message_model_c_consumer_typed_parity_request(
        structure: *mut crate::kmipkit_ttlv_structure_t,
        expected_operation_0: u32,
        expected_operation_1: u32,
        expected_asynchronous_default: u32,
        expected_attestation_default: u8,
        expected_batch_error_default: u32,
        expected_batch_order_default: u8,
        expected_extension_item_index: u64,
        expected_id_0: *const u8,
        expected_id_0_length: u64,
        expected_id_1: *const u8,
        expected_id_1_length: u64,
        expected_bit_mask_0: u32,
        expected_bit_mask_1: u32,
        expected_extension_value: *const u8,
        expected_extension_value_length: u64,
        expected_extension_criticality: u8,
    ) -> i32;
    fn kmipkit_message_model_c_consumer_typed_parity_invalid_request(
        structure: *mut crate::kmipkit_ttlv_structure_t,
        expected_kind: u32,
        expected_structure_index: u64,
    ) -> i32;
    fn kmipkit_message_model_c_consumer_typed_parity_response(
        structure: *mut crate::kmipkit_ttlv_structure_t,
        expected_operation: u32,
        expected_status: u32,
        expected_bit_mask: u32,
        expected_time_stamp: i64,
        expected_extension_item_index: u64,
        expected_extension_value: *const u8,
        expected_extension_value_length: u64,
        expected_extension_criticality: u8,
    ) -> i32;
}

const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;
const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const ASYNC_CORRELATION_VALUE_TAG: u32 = 0x0042_0006;
const CRITICALITY_INDICATOR_TAG: u32 = 0x0042_0026;
const MESSAGE_EXTENSION_TAG: u32 = 0x0042_0051;
const VENDOR_EXTENSION_TAG: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;

fn message_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn message_item(raw: u32, value: Value) -> Item {
    Item::new(message_tag(raw), value).expect("the fixture item is valid")
}

fn message_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("the fixture stays within TTLV limits");
    }
    structure
}

#[test]
fn typed_message_parity_case_is_shared_with_rust_and_compiled_c_consumers() {
    let fixture: serde_json::Value =
        serde_json::from_str(SHARED_PARITY_FIXTURE).expect("the shared fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("the shared fixture has cases")
        .iter()
        .find(|case| case["id"] == "typed-message.ordered-open-values-and-lifetime.v1")
        .expect("the typed message parity case is shared with Rust and C");
    assert_eq!(
        case["requirementIds"],
        serde_json::json!([
            "KMIPKIT-0043-FR-010",
            "KMIPKIT-0043-FR-011",
            "KMIPKIT-0043-FR-013",
            "KMIPKIT-0043-FR-017",
            "KMIPKIT-0043-FR-018",
            "KMIPKIT-0043-FR-022"
        ])
    );
    assert_eq!(case["consumerTests"]["rust"]["status"], "available");
    assert_eq!(case["consumerTests"]["c"]["status"], "available");
    assert!(case["input"]["request"].is_object());
    assert!(case["input"]["response"].is_object());
    assert!(case["input"]["invalidRequest"].is_object());
    assert!(case["expectedOutput"]["request"].is_object());
    assert!(case["expectedOutput"]["response"].is_object());
    assert!(case["expectedOutput"]["validation"].is_object());
}

fn fixture_tag(value: &JsonValue) -> u32 {
    u32::from_str_radix(
        value
            .as_str()
            .expect("fixture tags are hexadecimal strings")
            .strip_prefix("0x")
            .expect("fixture tags include a hexadecimal prefix"),
        16,
    )
    .expect("fixture tag fits in a KMIP tag")
}

fn fixture_hex_bytes(value: &JsonValue) -> Vec<u8> {
    let value = value
        .as_str()
        .expect("fixture byte values are hexadecimal strings");
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .expect("fixture bytes use valid hexadecimal")
        })
        .collect()
}

fn fixture_u32(value: &JsonValue) -> u32 {
    u32::try_from(
        value
            .as_u64()
            .expect("fixture field is an unsigned integer"),
    )
    .expect("fixture integer fits u32")
}

fn fixture_usize(value: &JsonValue) -> usize {
    usize::try_from(value.as_u64().expect("fixture index is unsigned"))
        .expect("fixture index fits usize")
}

fn fixture_raw_hex_u32(value: &JsonValue) -> u32 {
    u32::from_str_radix(
        value
            .as_str()
            .expect("fixture raw values are hexadecimal strings")
            .strip_prefix("0x")
            .expect("fixture raw values include a hexadecimal prefix"),
        16,
    )
    .expect("fixture raw value fits the 32-bit field")
}

fn fixture_extension(extension: &JsonValue) -> Structure {
    let vendor_tag = fixture_tag(&extension["vendorTag"]);
    let vendor_value = fixture_hex_bytes(&extension["vendorValueHex"]);
    let vendor_extension =
        message_structure([message_item(vendor_tag, Value::byte_string(vendor_value))]);
    message_structure([
        message_item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string(
                extension["vendorIdentification"]
                    .as_str()
                    .expect("fixture vendor identification is text")
                    .to_owned(),
            ),
        ),
        message_item(
            CRITICALITY_INDICATOR_TAG,
            Value::boolean(
                extension["criticalityIndicator"]
                    .as_bool()
                    .expect("fixture criticality indicator is Boolean"),
            ),
        ),
        message_item(VENDOR_EXTENSION_TAG, Value::structure(vendor_extension)),
    ])
}

fn typed_request_tree(input: &JsonValue, invalid: Option<&JsonValue>) -> Structure {
    let items = input["batchItems"]
        .as_array()
        .expect("fixture request has batch items");
    let mut header_fields = Vec::new();
    for raw_tag in input["headerFieldTags"]
        .as_array()
        .expect("fixture request has header tags")
    {
        let tag = fixture_tag(raw_tag);
        match tag {
            PROTOCOL_VERSION_TAG => header_fields.push(message_item(
                tag,
                Value::structure(message_structure([
                    message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
                    message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
                ])),
            )),
            BATCH_COUNT_TAG => header_fields.push(message_item(
                tag,
                Value::integer(i32::try_from(items.len()).expect("fixture batch count fits i32")),
            )),
            _ => panic!("fixture contains an unexpected request header tag"),
        }
    }
    let extension = &input["extension"];
    let missing_id_index = invalid.map_or(usize::MAX, |invalid| {
        usize::try_from(
            invalid["missingBatchItemIdIndex"]
                .as_u64()
                .expect("fixture missing-ID index is unsigned"),
        )
        .expect("fixture missing-ID index fits usize")
    });
    let mut root_fields = vec![message_item(
        REQUEST_HEADER_TAG,
        Value::structure(message_structure(header_fields)),
    )];
    for (index, item) in items.iter().enumerate() {
        let mut item_fields = vec![message_item(
            OPERATION_TAG,
            Value::enumeration(
                u32::try_from(
                    item["operationRaw"]
                        .as_u64()
                        .expect("fixture operation is an unsigned raw value"),
                )
                .expect("fixture operation fits u32"),
            ),
        )];
        if index != missing_id_index {
            item_fields.push(message_item(
                0x0042_0093,
                Value::byte_string(fixture_hex_bytes(&item["uniqueBatchItemIdHex"])),
            ));
        }
        let mut payload_fields = vec![message_item(
            fixture_tag(&item["bitMaskTag"]),
            Value::integer(i32::from_ne_bytes(
                fixture_raw_hex_u32(&item["bitMaskRawHex"]).to_ne_bytes(),
            )),
        )];
        if index == missing_id_index {
            payload_fields.push(message_item(
                0x0054_0002,
                Value::text_string(
                    invalid.expect("invalid fixture input exists")["payloadMarker"]
                        .as_str()
                        .expect("fixture payload marker is text")
                        .to_owned(),
                ),
            ));
        }
        item_fields.push(message_item(
            REQUEST_PAYLOAD_TAG,
            Value::structure(message_structure(payload_fields)),
        ));
        if item["messageExtension"].as_bool() == Some(true) {
            item_fields.push(message_item(
                MESSAGE_EXTENSION_TAG,
                Value::structure(fixture_extension(extension)),
            ));
        }
        root_fields.push(message_item(
            BATCH_ITEM_TAG,
            Value::structure(message_structure(item_fields)),
        ));
    }
    message_structure(root_fields)
}

fn typed_response_tree(input: &JsonValue) -> Structure {
    let mut header_fields = Vec::new();
    for raw_tag in input["headerFieldTags"]
        .as_array()
        .expect("fixture response has header tags")
    {
        let tag = fixture_tag(raw_tag);
        match tag {
            PROTOCOL_VERSION_TAG => header_fields.push(message_item(
                tag,
                Value::structure(message_structure([
                    message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
                    message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
                ])),
            )),
            TIME_STAMP_TAG => header_fields.push(message_item(
                tag,
                Value::date_time(
                    input["timeStamp"]
                        .as_i64()
                        .expect("fixture time stamp is signed"),
                ),
            )),
            BATCH_COUNT_TAG => header_fields.push(message_item(tag, Value::integer(1))),
            _ => panic!("fixture contains an unexpected response header tag"),
        }
    }
    let payload = message_structure([message_item(
        fixture_tag(&input["responsePayloadBitMaskTag"]),
        Value::integer(i32::from_ne_bytes(
            fixture_raw_hex_u32(&input["responsePayloadBitMaskRawHex"]).to_ne_bytes(),
        )),
    )]);
    let mut item_fields = vec![
        message_item(
            OPERATION_TAG,
            Value::enumeration(
                u32::try_from(
                    input["operationRaw"]
                        .as_u64()
                        .expect("fixture operation is unsigned"),
                )
                .expect("fixture operation fits u32"),
            ),
        ),
        message_item(
            RESULT_STATUS_TAG,
            Value::enumeration(
                u32::try_from(
                    input["resultStatusRaw"]
                        .as_u64()
                        .expect("fixture result status is unsigned"),
                )
                .expect("fixture result status fits u32"),
            ),
        ),
        message_item(RESPONSE_PAYLOAD_TAG, Value::structure(payload)),
    ];
    if input["messageExtension"].as_bool() == Some(true) {
        item_fields.push(message_item(
            MESSAGE_EXTENSION_TAG,
            Value::structure(fixture_extension(&input["extension"])),
        ));
    }
    message_structure([
        message_item(
            RESPONSE_HEADER_TAG,
            Value::structure(message_structure(header_fields)),
        ),
        message_item(
            BATCH_ITEM_TAG,
            Value::structure(message_structure(item_fields)),
        ),
    ])
}

fn fixture_child_tags(view: &StructureView<'_>) -> Vec<u32> {
    view.children()
        .iter()
        .map(|item| item.tag().raw())
        .collect()
}

fn assert_extension_matches_fixture(extension: MessageExtensionView<'_>, expected: &JsonValue) {
    assert_eq!(
        extension.with_vendor_identification(str::to_owned),
        expected["extensionVendorIdentification"]
            .as_str()
            .map(str::to_owned)
    );
    assert_eq!(
        extension.criticality_indicator(),
        expected["extensionCriticalityIndicator"].as_bool()
    );
    extension
        .with_ttlv(|view| {
            assert_eq!(
                fixture_child_tags(&view),
                vec![
                    VENDOR_IDENTIFICATION_TAG,
                    CRITICALITY_INDICATOR_TAG,
                    VENDOR_EXTENSION_TAG
                ]
            );
        })
        .expect("fixture extension exposes its ordered TTLV");
    extension
        .with_vendor_extension(|view| {
            let item = &view.children()[0];
            assert_eq!(
                item.tag().raw(),
                fixture_tag(&expected["extensionVendorTag"])
            );
            item.with_value(|value| match value {
                ValueView::ByteString(bytes) => assert_eq!(
                    bytes,
                    fixture_hex_bytes(&expected["extensionVendorValueHex"]).as_slice()
                ),
                other => panic!("fixture vendor extension had unexpected TTLV value: {other:?}"),
            });
        })
        .expect("fixture extension exposes its Vendor Extension");
}

#[test]
fn typed_request_response_messages_match_shared_fixture() {
    let fixture: JsonValue =
        serde_json::from_str(SHARED_PARITY_FIXTURE).expect("shared parity fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("shared parity fixture has cases")
        .iter()
        .find(|case| case["id"] == "typed-message.ordered-open-values-and-lifetime.v1")
        .expect("typed message parity case exists");
    let input = &case["input"];
    let expected = &case["expectedOutput"];
    assert_rust_request_fixture(&input["request"], &expected["request"]);
    assert_rust_response_fixture(&input["response"], &expected["response"]);
    assert_rust_validation_fixture(input, expected);
    assert_c_request_fixture(input, expected);
    assert_c_invalid_and_response_fixtures(input, expected);
}

fn assert_rust_request_fixture(request_input: &JsonValue, expected: &JsonValue) {
    let request =
        kmipkit_protocol::RequestMessage::try_from_ttlv(typed_request_tree(request_input, None))
            .expect("fixture request is structurally valid");
    assert_eq!(
        request.batch_items().len(),
        fixture_usize(&expected["batchCount"])
    );
    assert_eq!(
        request.header().effective_asynchronous_indicator(),
        fixture_u32(&expected["effectiveDefaults"]["asynchronousIndicator"])
    );
    assert_eq!(
        request.header().effective_attestation_capable_indicator(),
        expected["effectiveDefaults"]["attestationCapableIndicator"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(
        request.header().effective_batch_error_continuation_option(),
        fixture_u32(&expected["effectiveDefaults"]["batchErrorContinuationOption"])
    );
    assert_eq!(
        request.header().effective_batch_order_option(),
        expected["effectiveDefaults"]["batchOrderOption"]
            .as_bool()
            .unwrap()
    );
    assert!(request.header().asynchronous_indicator().is_none());
    assert!(request.header().attestation_capable_indicator().is_none());
    assert!(request.header().batch_error_continuation_option().is_none());
    assert!(request.header().batch_order_option().is_none());
    request.with_ttlv(|root| {
        let request_header = root.children()[0].with_value(|value| match value {
            ValueView::Structure(view) => Some(fixture_child_tags(&view)),
            _ => None,
        });
        assert_eq!(
            request_header,
            Some(vec![PROTOCOL_VERSION_TAG, BATCH_COUNT_TAG])
        );
    });
    let request_expected_items = expected["batchItems"].as_array().unwrap();
    for (index, (item, item_expected)) in request
        .batch_items()
        .zip(request_expected_items)
        .enumerate()
    {
        assert_eq!(
            u64::from(item.operation().expect("fixture operation is present")),
            item_expected["operationRaw"].as_u64().unwrap()
        );
        assert_eq!(
            item.with_unique_batch_item_id(<[u8]>::to_vec),
            Some(fixture_hex_bytes(&item_expected["uniqueBatchItemIdHex"]))
        );
        let expected_tags: Vec<u32> = item_expected["fieldTags"]
            .as_array()
            .unwrap()
            .iter()
            .map(fixture_tag)
            .collect();
        let observed_tags = request.with_ttlv(|root| {
            root.children()[index + 1].with_value(|value| match value {
                ValueView::Structure(view) => Some(fixture_child_tags(&view)),
                _ => None,
            })
        });
        assert_eq!(observed_tags, Some(expected_tags));
        let raw_bit_mask = item.with_request_payload(|payload| {
            payload.children()[0].with_value(|value| match value {
                ValueView::Integer(raw) => Some(u32::from_ne_bytes(raw.to_ne_bytes())),
                _ => None,
            })
        });
        assert_eq!(
            raw_bit_mask,
            Some(Some(fixture_raw_hex_u32(&item_expected["bitMaskRawHex"])))
        );
        if index == fixture_usize(&expected["extensionBatchItemIndex"]) {
            let extension = item
                .message_extension(0)
                .expect("fixture extension is present");
            assert_extension_matches_fixture(extension, expected);
        } else {
            assert_eq!(item.message_extension_count(), 0);
        }
    }
}

fn assert_rust_response_fixture(response_input: &JsonValue, expected: &JsonValue) {
    let response =
        kmipkit_protocol::ResponseMessage::try_from_ttlv(typed_response_tree(response_input))
            .expect("fixture response is structurally valid");
    assert_eq!(response.batch_items().len(), 1);
    assert_eq!(
        response.header().time_stamp(),
        response_input["timeStamp"].as_i64().unwrap()
    );
    response.with_ttlv(|root| {
        let header = root.children()[0].with_value(|value| match value {
            ValueView::Structure(view) => Some(fixture_child_tags(&view)),
            _ => None,
        });
        assert_eq!(
            header,
            Some(vec![PROTOCOL_VERSION_TAG, TIME_STAMP_TAG, BATCH_COUNT_TAG])
        );
    });
    let response_item = response.batch_items().next().unwrap();
    assert_eq!(
        u64::from(response_item.operation().unwrap()),
        expected["operationRaw"].as_u64().unwrap()
    );
    assert_eq!(
        u64::from(response_item.result_status().unwrap().raw()),
        expected["resultStatusRaw"].as_u64().unwrap()
    );
    let expected_response_tags: Vec<u32> = expected["batchItemFieldTags"]
        .as_array()
        .unwrap()
        .iter()
        .map(fixture_tag)
        .collect();
    assert_eq!(
        response_item.with_ttlv(|view| fixture_child_tags(&view)),
        Some(expected_response_tags)
    );
    let response_extension = response_item.message_extension(0).unwrap();
    assert_extension_matches_fixture(response_extension, expected);
}

fn assert_rust_validation_fixture(input: &JsonValue, expected: &JsonValue) {
    let request_input = &input["request"];
    let invalid = &input["invalidRequest"];
    let error = kmipkit_protocol::RequestMessage::try_from_ttlv(typed_request_tree(
        request_input,
        Some(invalid),
    ))
    .expect_err("fixture request omits the required second item ID");
    assert_eq!(
        error.kind(),
        kmipkit_protocol::MessageValidationErrorKind::MissingRequiredField
    );
    assert_eq!(
        error
            .structure_index()
            .map(|index| u64::try_from(index).expect("fixture index fits u64")),
        expected["validation"]["structureIndex"].as_u64()
    );
    assert_eq!(error.field_index(), None);
    let marker = input["invalidRequest"]["payloadMarker"].as_str().unwrap();
    assert!(!error.to_string().contains(marker));
    assert_eq!(expected["validation"]["kindCode"].as_u64(), Some(1));
    assert_eq!(expected["validation"]["cDiagnosticSurface"], "none");
}

fn assert_c_request_fixture(input: &JsonValue, expected: &JsonValue) {
    let request_input = &input["request"];
    let request_expected = &expected["request"];
    let c_request = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(typed_request_tree(request_input, None)),
    )
    .expect("fixture request TTLV handle is allocated for compiled C");
    let first_operation = fixture_u32(&request_input["batchItems"][0]["operationRaw"]);
    let second_operation = fixture_u32(&request_input["batchItems"][1]["operationRaw"]);
    let defaults = &request_expected["effectiveDefaults"];
    let request_extension = &input["request"]["extension"];
    let request_extension_value = fixture_hex_bytes(&request_extension["vendorValueHex"]);
    let first_id = fixture_hex_bytes(&request_input["batchItems"][0]["uniqueBatchItemIdHex"]);
    let second_id = fixture_hex_bytes(&request_input["batchItems"][1]["uniqueBatchItemIdHex"]);
    let first_mask = fixture_raw_hex_u32(&request_input["batchItems"][0]["bitMaskRawHex"]);
    let second_mask = fixture_raw_hex_u32(&request_input["batchItems"][1]["bitMaskRawHex"]);
    // SAFETY: The live Structure handle is transferred to the compiled C consumer, which
    // constructs and releases the message and every retained view exactly once.
    assert_eq!(
        unsafe {
            kmipkit_message_model_c_consumer_typed_parity_request(
                c_request,
                first_operation,
                second_operation,
                fixture_u32(&defaults["asynchronousIndicator"]),
                u8::from(defaults["attestationCapableIndicator"].as_bool().unwrap()),
                fixture_u32(&defaults["batchErrorContinuationOption"]),
                u8::from(defaults["batchOrderOption"].as_bool().unwrap()),
                request_expected["extensionBatchItemIndex"]
                    .as_u64()
                    .unwrap(),
                first_id.as_ptr(),
                first_id.len() as u64,
                second_id.as_ptr(),
                second_id.len() as u64,
                first_mask,
                second_mask,
                request_extension_value.as_ptr(),
                request_extension_value.len() as u64,
                u8::from(
                    request_expected["extensionCriticalityIndicator"]
                        .as_bool()
                        .unwrap(),
                ),
            )
        },
        0
    );
}

fn assert_c_invalid_and_response_fixtures(input: &JsonValue, expected: &JsonValue) {
    let request_input = &input["request"];
    let invalid_request = &input["invalidRequest"];
    let invalid_structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(typed_request_tree(request_input, Some(invalid_request))),
    )
    .expect("fixture invalid-request TTLV handle is allocated for compiled C");
    // SAFETY: The live invalid Structure handle is consumed by the C consumer, which reads and
    // releases only the returned payload-free validation error handle.
    assert_eq!(
        unsafe {
            kmipkit_message_model_c_consumer_typed_parity_invalid_request(
                invalid_structure,
                fixture_u32(&expected["validation"]["kindCode"]),
                expected["validation"]["structureIndex"].as_u64().unwrap(),
            )
        },
        0
    );

    let response_input = &input["response"];
    let response_expected = &expected["response"];
    let response_extension_value =
        fixture_hex_bytes(&response_input["extension"]["vendorValueHex"]);
    let response_structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(typed_response_tree(response_input)),
    )
    .expect("fixture response TTLV handle is allocated for compiled C");
    // SAFETY: The live response Structure handle is transferred to the compiled C consumer,
    // which closes the message and item owners before reading its retained extension view.
    assert_eq!(
        unsafe {
            kmipkit_message_model_c_consumer_typed_parity_response(
                response_structure,
                fixture_u32(&response_expected["operationRaw"]),
                fixture_u32(&response_expected["resultStatusRaw"]),
                fixture_raw_hex_u32(&response_input["responsePayloadBitMaskRawHex"]),
                response_input["timeStamp"].as_i64().unwrap(),
                response_expected["extensionBatchItemIndex"]
                    .as_u64()
                    .unwrap(),
                response_extension_value.as_ptr(),
                response_extension_value.len() as u64,
                u8::from(
                    response_expected["extensionCriticalityIndicator"]
                        .as_bool()
                        .unwrap(),
                ),
            )
        },
        0
    );
}

fn assert_structure_tags(
    handle: *mut crate::kmipkit_ttlv_structure_t,
    expected_tags: &[u32],
) -> Arc<Handle> {
    let structure_handle =
        reference_handle(handle, Kind::Structure).expect("the accessor returns a Structure");
    let HandleValue::Structure(structure) = &structure_handle.value else {
        panic!("the accessor returns a Structure");
    };
    assert_eq!(
        structure
            .view()
            .children()
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        expected_tags
    );
    structure_handle
}

fn assert_response_extension_is_lossless(
    batch_item: *mut crate::kmipkit_response_batch_item_view_t,
) {
    let mut extension_count = 0_u64;
    assert_eq!(
        kmipkit_response_batch_item_extension_count(batch_item, &raw mut extension_count),
        SUCCESS
    );
    assert_eq!(extension_count, 1);
    let mut extension_view: *mut crate::kmipkit_message_extension_view_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_batch_item_extension_at(batch_item, 0, &raw mut extension_view),
        SUCCESS
    );
    let mut extension_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_message_extension_view_ttlv(extension_view, &raw mut extension_ttlv),
        SUCCESS
    );
    let extension_handle = assert_structure_tags(
        extension_ttlv,
        &[
            VENDOR_IDENTIFICATION_TAG,
            CRITICALITY_INDICATOR_TAG,
            VENDOR_EXTENSION_TAG,
        ],
    );
    let HandleValue::Structure(extension_tree) = &extension_handle.value else {
        panic!("the extension TTLV accessor returns a Structure");
    };
    let vendor_value = extension_tree.view().children()[2].with_value(|value| match value {
        ValueView::Structure(vendor_extension) => {
            vendor_extension.children()[0].with_value(|value| match value {
                ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                _ => None,
            })
        }
        _ => None,
    });
    assert_eq!(vendor_value, Some(vec![0x00, 0xFE, 0x80]));
    kmipkit_ttlv_structure_release(extension_ttlv);
    kmipkit_message_extension_view_release(extension_view);
}

fn valid_request_message_tree() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0xF123_4567)),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    message_structure([
        message_item(REQUEST_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(batch)),
    ])
}

fn request_message_with_two_items_and_omitted_defaults() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(BATCH_COUNT_TAG, Value::integer(2)),
    ]);
    let first = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0xF123_4567)),
        message_item(0x0042_0093, Value::byte_string(vec![0x00, 0xFF, 0x80])),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    let second = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0x0000_000A)),
        message_item(0x0042_0093, Value::byte_string(vec![0x02, 0x03, 0x04])),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    message_structure([
        message_item(REQUEST_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(first)),
        message_item(BATCH_ITEM_TAG, Value::structure(second)),
    ])
}

fn request_message_missing_multi_item_id() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(BATCH_COUNT_TAG, Value::integer(2)),
    ]);
    let first_batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(1)),
        message_item(0x0042_0093, Value::byte_string(vec![0xA1])),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    let second_batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(1)),
        message_item(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ]);
    message_structure([
        message_item(REQUEST_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(first_batch)),
        message_item(BATCH_ITEM_TAG, Value::structure(second_batch)),
    ])
}

fn valid_pending_response_message_tree() -> Structure {
    let version = message_structure([
        message_item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        message_item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let header = message_structure([
        message_item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        message_item(TIME_STAMP_TAG, Value::date_time(5678)),
        message_item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let extension = message_structure([
        message_item(
            VENDOR_IDENTIFICATION_TAG,
            Value::text_string("Acme".to_owned()),
        ),
        message_item(CRITICALITY_INDICATOR_TAG, Value::boolean(false)),
        message_item(
            VENDOR_EXTENSION_TAG,
            Value::structure(message_structure([message_item(
                0x0054_0001,
                Value::byte_string(vec![0x00, 0xFE, 0x80]),
            )])),
        ),
    ]);
    let batch = message_structure([
        message_item(OPERATION_TAG, Value::enumeration(0x0000_001A)),
        message_item(RESULT_STATUS_TAG, Value::enumeration(2)),
        message_item(
            ASYNC_CORRELATION_VALUE_TAG,
            Value::byte_string(vec![0x00, 0xFF, 0x80]),
        ),
        message_item(MESSAGE_EXTENSION_TAG, Value::structure(extension)),
    ]);
    message_structure([
        message_item(RESPONSE_HEADER_TAG, Value::structure(header)),
        message_item(BATCH_ITEM_TAG, Value::structure(batch)),
    ])
}

#[test]
fn request_message_handle_exposes_validated_header_and_open_operation() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_request_message_tree()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::null_mut();
    let mut validation_error = ptr::dangling_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        SUCCESS
    );
    assert!(!message.is_null());
    assert!(validation_error.is_null());

    let mut header = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_header(message, &raw mut header),
        SUCCESS
    );
    let mut version = kmipkit_protocol_version_v1_t { major: 0, minor: 0 };
    assert_eq!(
        kmipkit_request_header_view_protocol_version(header, &raw mut version),
        SUCCESS
    );
    assert_eq!(
        version,
        kmipkit_protocol_version_v1_t { major: 2, minor: 1 }
    );

    let mut header_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_request_header_view_ttlv(header, &raw mut header_ttlv),
        SUCCESS
    );
    let _header_tree = assert_structure_tags(header_ttlv, &[PROTOCOL_VERSION_TAG, BATCH_COUNT_TAG]);
    kmipkit_ttlv_structure_release(header_ttlv);

    let mut batch_count = 0_i32;
    assert_eq!(
        kmipkit_request_header_view_batch_count(header, &raw mut batch_count),
        SUCCESS
    );
    assert_eq!(batch_count, 1);

    let mut item_count = 0_u64;
    assert_eq!(
        kmipkit_request_message_batch_item_count(message, &raw mut item_count),
        SUCCESS
    );
    assert_eq!(item_count, 1);

    let mut batch_item = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_batch_item_at(message, 0, &raw mut batch_item),
        SUCCESS
    );
    let mut batch_item_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_request_batch_item_view_ttlv(batch_item, &raw mut batch_item_ttlv),
        SUCCESS
    );
    let _batch_tree = assert_structure_tags(batch_item_ttlv, &[OPERATION_TAG, REQUEST_PAYLOAD_TAG]);
    kmipkit_ttlv_structure_release(batch_item_ttlv);
    let mut has_operation = u8::MAX;
    let mut operation = 0;
    assert_eq!(
        kmipkit_request_batch_item_view_operation(
            batch_item,
            &raw mut has_operation,
            &raw mut operation,
        ),
        SUCCESS
    );
    assert_eq!(has_operation, 1);
    assert_eq!(operation, 0xF123_4567);

    let mut original_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_ttlv(message, &raw mut original_ttlv),
        SUCCESS
    );
    let original_ttlv_handle =
        assert_structure_tags(original_ttlv, &[REQUEST_HEADER_TAG, BATCH_ITEM_TAG]);
    let HandleValue::Structure(original_tree) = &original_ttlv_handle.value else {
        panic!("the message TTLV accessor returns a Structure");
    };
    let original_view = original_tree.view();
    let root_items = original_view.children();
    let raw_operation = root_items[1].with_value(|batch| match batch {
        ValueView::Structure(batch) => {
            batch.children()[0].with_value(|operation| match operation {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        }
        _ => None,
    });
    assert_eq!(raw_operation, Some(0xF123_4567));

    kmipkit_request_batch_item_release(batch_item);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
    kmipkit_ttlv_structure_release(original_ttlv);
}

#[test]
fn request_header_c_abi_exposes_effective_kmip_defaults_without_materializing_fields() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(request_message_with_two_items_and_omitted_defaults()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::null_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        SUCCESS
    );
    assert!(validation_error.is_null());
    let mut header = ptr::null_mut();
    assert_eq!(
        kmipkit_request_message_header(message, &raw mut header),
        SUCCESS
    );

    let mut asynchronous_indicator = u32::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_asynchronous_indicator(
            header,
            &raw mut asynchronous_indicator,
        ),
        SUCCESS
    );
    assert_eq!(asynchronous_indicator, 3);

    let mut attestation_capable = u8::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_attestation_capable_indicator(
            header,
            &raw mut attestation_capable,
        ),
        SUCCESS
    );
    assert_eq!(attestation_capable, 0);

    let mut batch_error_continuation = u32::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_batch_error_continuation_option(
            header,
            &raw mut batch_error_continuation,
        ),
        SUCCESS
    );
    assert_eq!(batch_error_continuation, 2);

    let mut batch_order = u8::MAX;
    assert_eq!(
        kmipkit_request_header_view_effective_batch_order_option(header, &raw mut batch_order,),
        SUCCESS
    );
    assert_eq!(batch_order, 1);

    let mut header_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_request_header_view_ttlv(header, &raw mut header_ttlv),
        SUCCESS
    );
    let _header_tree = assert_structure_tags(header_ttlv, &[PROTOCOL_VERSION_TAG, BATCH_COUNT_TAG]);
    kmipkit_ttlv_structure_release(header_ttlv);
    kmipkit_request_header_view_release(header);
    kmipkit_request_message_release(message);
}

#[test]
fn invalid_request_message_returns_only_payload_free_validation_metadata() {
    let structure = try_make_handle(Kind::Structure, HandleValue::Structure(Structure::new()))
        .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::dangling_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        ERROR_PROTOCOL
    );
    assert!(message.is_null());
    assert!(!validation_error.is_null());

    let mut kind = u32::MAX;
    assert_eq!(
        kmipkit_message_validation_error_kind(validation_error, &raw mut kind),
        SUCCESS
    );
    assert_eq!(kind, 1, "missing required field is the stable kind value 1");
    kmipkit_message_validation_error_release(validation_error);
}

#[test]
fn validation_error_exposes_optional_numeric_indexes_without_payload() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(request_message_missing_multi_item_id()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message = ptr::null_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        ERROR_PROTOCOL
    );
    assert!(message.is_null());
    assert!(!validation_error.is_null());

    let mut has_structure_index = u8::MAX;
    let mut structure_index = u64::MAX;
    assert_eq!(
        kmipkit_message_validation_error_structure_index(
            validation_error,
            &raw mut has_structure_index,
            &raw mut structure_index,
        ),
        SUCCESS
    );
    assert_eq!(has_structure_index, 1);
    assert_eq!(structure_index, 2);

    let mut has_field_index = u8::MAX;
    let mut field_index = u64::MAX;
    assert_eq!(
        kmipkit_message_validation_error_field_index(
            validation_error,
            &raw mut has_field_index,
            &raw mut field_index,
        ),
        SUCCESS
    );
    assert_eq!(has_field_index, 0);
    assert_eq!(field_index, 0);

    kmipkit_message_validation_error_release(validation_error);
}

#[test]
fn ffi_output_region_validator_rejects_overlapping_slots() {
    let mut output = 0_u64;
    let shared = (&raw mut output).cast::<u8>().cast_const();

    assert_eq!(
        validate_output_regions(&[
            (shared, std::mem::size_of::<u64>()),
            (shared, std::mem::size_of::<u32>()),
        ]),
        Err(ERROR_INVALID_INPUT)
    );
}

#[test]
fn request_message_constructor_rejects_aliased_output_slots_before_consuming_input() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_request_message_tree()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut output: *mut crate::kmipkit_request_message_t = ptr::null_mut();
    let output_error = (&raw mut output).cast();

    assert_eq!(
        kmipkit_request_message_from_ttlv(structure, &raw mut output, output_error),
        ERROR_INVALID_INPUT
    );
    assert!(output.is_null());
    assert!(reference_handle(structure, Kind::Structure).is_ok());
    kmipkit_ttlv_structure_release(structure);
}

#[test]
fn response_message_handle_exposes_typed_header_and_raw_result_status() {
    let structure = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_pending_response_message_tree()),
    )
    .expect("the generic TTLV input handle is allocated");
    let mut message: *mut crate::kmipkit_response_message_t = ptr::null_mut();
    let mut validation_error = ptr::null_mut();

    assert_eq!(
        kmipkit_response_message_from_ttlv(structure, &raw mut message, &raw mut validation_error,),
        SUCCESS
    );
    assert!(!message.is_null());
    assert!(validation_error.is_null());

    let mut header: *mut crate::kmipkit_response_header_view_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_message_header(message, &raw mut header),
        SUCCESS
    );
    let mut version = kmipkit_protocol_version_v1_t { major: 0, minor: 0 };
    assert_eq!(
        kmipkit_response_header_view_protocol_version(header, &raw mut version),
        SUCCESS
    );
    assert_eq!(
        version,
        kmipkit_protocol_version_v1_t { major: 2, minor: 1 }
    );
    let mut header_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_header_view_ttlv(header, &raw mut header_ttlv),
        SUCCESS
    );
    let _response_header_tree = assert_structure_tags(
        header_ttlv,
        &[PROTOCOL_VERSION_TAG, TIME_STAMP_TAG, BATCH_COUNT_TAG],
    );
    kmipkit_ttlv_structure_release(header_ttlv);

    let mut time_stamp = 0_i64;
    assert_eq!(
        kmipkit_response_header_view_time_stamp(header, &raw mut time_stamp),
        SUCCESS
    );
    assert_eq!(time_stamp, 5678);

    let mut item_count = 0_u64;
    assert_eq!(
        kmipkit_response_message_batch_item_count(message, &raw mut item_count),
        SUCCESS
    );
    assert_eq!(item_count, 1);

    let mut batch_item: *mut crate::kmipkit_response_batch_item_view_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_message_batch_item_at(message, 0, &raw mut batch_item),
        SUCCESS
    );
    let mut batch_item_ttlv: *mut crate::kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(
        kmipkit_response_batch_item_view_ttlv(batch_item, &raw mut batch_item_ttlv),
        SUCCESS
    );
    let _response_batch_tree = assert_structure_tags(
        batch_item_ttlv,
        &[
            OPERATION_TAG,
            RESULT_STATUS_TAG,
            ASYNC_CORRELATION_VALUE_TAG,
            MESSAGE_EXTENSION_TAG,
        ],
    );
    kmipkit_ttlv_structure_release(batch_item_ttlv);
    let mut has_status = u8::MAX;
    let mut raw_status = 0_u32;
    assert_eq!(
        kmipkit_response_batch_item_view_result_status(
            batch_item,
            &raw mut has_status,
            &raw mut raw_status,
        ),
        SUCCESS
    );
    assert_eq!(has_status, 1);
    assert_eq!(raw_status, 2);

    assert_response_extension_is_lossless(batch_item);
    kmipkit_response_batch_item_release(batch_item);
    kmipkit_response_header_view_release(header);
    kmipkit_response_message_release(message);
}

#[test]
fn compiled_c_consumer_uses_request_and_response_message_abi() {
    let request = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_request_message_tree()),
    )
    .expect("the request TTLV handle is allocated");
    let response = try_make_handle(
        Kind::Structure,
        HandleValue::Structure(valid_pending_response_message_tree()),
    )
    .expect("the response TTLV handle is allocated");

    // SAFETY: Both pointers refer to live Structure handles. The compiled C consumer passes each
    // to the corresponding constructor, which consumes the handle exactly once.
    assert_eq!(
        unsafe { kmipkit_message_model_c_consumer_validate_request(request,) },
        0
    );
    // SAFETY: The response handle is live and is consumed once by the response constructor.
    assert_eq!(
        unsafe { kmipkit_message_model_c_consumer_validate_response(response,) },
        0
    );
}
