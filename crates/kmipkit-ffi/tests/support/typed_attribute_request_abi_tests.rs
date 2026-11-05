use super::*;
use kmipkit_protocol::{AdjustAttributeRequest, AdjustmentType, AttributeReference};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn standalone_adjust_attribute_model_serializes_and_enters_a_batch() {
    let identifier = b"object-7";
    let identifier_record = text_identifier_record(identifier);
    let attribute_reference = name_attribute_reference_record();
    let adjustment_value = make_handle(Kind::Value, HandleValue::Value(Value::integer(5)));
    let mut request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_adjust_attribute_request_create(
            &raw const identifier_record,
            &raw const attribute_reference,
            1,
            adjustment_value,
            &raw mut request,
        ),
        SUCCESS
    );

    let mut payload_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_adjust_attribute_request_to_ttlv_payload(
            request,
            &raw mut payload_handle_ptr,
        ),
        SUCCESS
    );
    let payload_handle = reference_handle(payload_handle_ptr, Kind::Structure)
        .expect("the standalone Adjust Attribute model serializes independently");
    let HandleValue::Structure(payload) = &payload_handle.value else {
        panic!("the serialized payload is a generic TTLV Structure");
    };
    assert_eq!(payload.view().children().len(), 4);

    let mut batch_item_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_adjust_attribute_from_request(
            request,
            &raw mut batch_item_handle_ptr,
        ),
        SUCCESS
    );
    let batch_handle = reference_handle(batch_item_handle_ptr, Kind::BatchItem)
        .expect("the typed Adjust Attribute model converts to a batch item");
    let HandleValue::BatchItem(batch_item) = &batch_handle.value else {
        panic!("the conversion returns a batch-item handle");
    };
    assert!(matches!(
        batch_item.request(),
        ClientRequest::AdjustAttribute(_)
    ));

    kmipkit_client_batch_item_release(batch_item_handle_ptr);
    kmipkit_ttlv_structure_release(payload_handle_ptr);
    kmipkit_adjust_attribute_request_release(request);
    kmipkit_ttlv_value_release(adjustment_value);
}

#[test]
fn standalone_delete_attribute_model_preserves_both_optional_selectors() {
    let identifier = b"object-7";
    let identifier_record = text_identifier_record(identifier);
    let attribute_reference = name_attribute_reference_record();
    let current_attribute = make_handle(Kind::Item, HandleValue::Item(attribute_item()));
    let mut request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_create(
            &raw const identifier_record,
            current_attribute,
            &raw const attribute_reference,
            &raw mut request,
        ),
        SUCCESS
    );

    let mut current_present = 0;
    let mut current_copy = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_current_attribute(
            request,
            &raw mut current_present,
            &raw mut current_copy,
        ),
        SUCCESS
    );
    assert_eq!(current_present, 1);
    assert!(!current_copy.is_null());
    kmipkit_ttlv_item_release(current_copy);

    let mut reference_present = 0;
    let mut reference_copy = name_attribute_reference_record();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_attribute_reference(
            request,
            &raw mut reference_present,
            &raw mut reference_copy,
        ),
        SUCCESS
    );
    assert_eq!(reference_present, 1);
    assert_eq!(
        reference_copy.kind,
        crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME
    );

    let mut payload_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_to_ttlv_payload(
            request,
            &raw mut payload_handle_ptr,
        ),
        SUCCESS
    );
    let payload_handle = reference_handle(payload_handle_ptr, Kind::Structure)
        .expect("the standalone Delete Attribute model serializes independently");
    let HandleValue::Structure(payload) = &payload_handle.value else {
        panic!("the serialized payload is a generic TTLV Structure");
    };
    assert_eq!(payload.view().children().len(), 3);

    let mut batch_item_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_delete_attribute_from_request(
            request,
            &raw mut batch_item_handle_ptr,
        ),
        SUCCESS
    );
    let batch_handle = reference_handle(batch_item_handle_ptr, Kind::BatchItem)
        .expect("the typed Delete Attribute model converts to a batch item");
    let HandleValue::BatchItem(batch_item) = &batch_handle.value else {
        panic!("the conversion returns a batch-item handle");
    };
    assert!(matches!(
        batch_item.request(),
        ClientRequest::DeleteAttribute(_)
    ));

    kmipkit_client_batch_item_release(batch_item_handle_ptr);
    kmipkit_ttlv_structure_release(payload_handle_ptr);
    kmipkit_delete_attribute_request_release(request);
    kmipkit_ttlv_item_release(current_attribute);
}

#[test]
fn standalone_modify_attribute_model_keeps_current_and_new_items_distinct() {
    let identifier = b"object-7";
    let identifier_record = text_identifier_record(identifier);
    let current_attribute = make_handle(Kind::Item, HandleValue::Item(attribute_item()));
    let new_attribute = make_handle(Kind::Item, HandleValue::Item(attribute_item()));
    let mut request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_create(
            &raw const identifier_record,
            current_attribute,
            new_attribute,
            &raw mut request,
        ),
        SUCCESS
    );

    let mut current_present = 0;
    let mut current_copy = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_current_attribute(
            request,
            &raw mut current_present,
            &raw mut current_copy,
        ),
        SUCCESS
    );
    assert_eq!(current_present, 1);
    assert!(!current_copy.is_null());
    kmipkit_ttlv_item_release(current_copy);

    let mut new_copy = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_new_attribute(request, &raw mut new_copy),
        SUCCESS
    );
    assert!(!new_copy.is_null());
    kmipkit_ttlv_item_release(new_copy);

    let mut payload_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_to_ttlv_payload(
            request,
            &raw mut payload_handle_ptr,
        ),
        SUCCESS
    );
    let payload_handle = reference_handle(payload_handle_ptr, Kind::Structure)
        .expect("the standalone Modify Attribute model serializes independently");
    let HandleValue::Structure(payload) = &payload_handle.value else {
        panic!("the serialized payload is a generic TTLV Structure");
    };
    assert_eq!(payload.view().children().len(), 3);

    let mut batch_item_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_modify_attribute_from_request(
            request,
            &raw mut batch_item_handle_ptr,
        ),
        SUCCESS
    );
    let batch_handle = reference_handle(batch_item_handle_ptr, Kind::BatchItem)
        .expect("the typed Modify Attribute model converts to a batch item");
    let HandleValue::BatchItem(batch_item) = &batch_handle.value else {
        panic!("the conversion returns a batch-item handle");
    };
    assert!(matches!(
        batch_item.request(),
        ClientRequest::ModifyAttribute(_)
    ));

    kmipkit_client_batch_item_release(batch_item_handle_ptr);
    kmipkit_ttlv_structure_release(payload_handle_ptr);
    kmipkit_modify_attribute_request_release(request);
    kmipkit_ttlv_item_release(new_attribute);
    kmipkit_ttlv_item_release(current_attribute);
}

#[test]
fn standalone_set_attribute_model_serializes_and_enters_a_batch() {
    let identifier = b"object-7";
    let identifier_record = text_identifier_record(identifier);
    let new_attribute = make_handle(Kind::Item, HandleValue::Item(attribute_item()));
    let mut request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_set_attribute_request_create(
            &raw const identifier_record,
            new_attribute,
            &raw mut request,
        ),
        SUCCESS
    );

    let mut payload_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_set_attribute_request_to_ttlv_payload(
            request,
            &raw mut payload_handle_ptr,
        ),
        SUCCESS
    );
    let payload_handle = reference_handle(payload_handle_ptr, Kind::Structure)
        .expect("the standalone Set Attribute model serializes independently");
    let HandleValue::Structure(payload) = &payload_handle.value else {
        panic!("the serialized payload is a generic TTLV Structure");
    };
    assert_eq!(payload.view().children().len(), 2);

    let mut batch_item_handle_ptr = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_set_attribute_from_request(
            request,
            &raw mut batch_item_handle_ptr,
        ),
        SUCCESS
    );
    let batch_handle = reference_handle(batch_item_handle_ptr, Kind::BatchItem)
        .expect("the typed Set Attribute model converts to a batch item");
    let HandleValue::BatchItem(batch_item) = &batch_handle.value else {
        panic!("the conversion returns a batch-item handle");
    };
    assert!(matches!(
        batch_item.request(),
        ClientRequest::SetAttribute(_)
    ));

    kmipkit_client_batch_item_release(batch_item_handle_ptr);
    kmipkit_ttlv_structure_release(payload_handle_ptr);
    kmipkit_set_attribute_request_release(request);
    kmipkit_ttlv_item_release(new_attribute);
}

#[test]
fn optional_attribute_request_getters_preserve_absence_and_reject_missing_outputs() {
    let mut delete_request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_create(
            ptr::null(),
            ptr::null_mut(),
            ptr::null(),
            &raw mut delete_request,
        ),
        SUCCESS
    );

    let mut current_present = u8::MAX;
    let mut current_copy = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_current_attribute(
            delete_request,
            &raw mut current_present,
            &raw mut current_copy,
        ),
        SUCCESS
    );
    assert_eq!(current_present, 0);
    assert!(current_copy.is_null());

    let mut reference_present = u8::MAX;
    let mut reference = name_attribute_reference_record();
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_attribute_reference(
            delete_request,
            &raw mut reference_present,
            &raw mut reference,
        ),
        SUCCESS
    );
    assert_eq!(reference_present, 0);
    assert_eq!(reference.kind, 0);
    assert_eq!(
        client_abi::kmipkit_delete_attribute_request_attribute_reference(
            delete_request,
            ptr::null_mut(),
            &raw mut reference,
        ),
        ERROR_INVALID_INPUT
    );

    let new_attribute = make_handle(Kind::Item, HandleValue::Item(attribute_item()));
    let mut modify_request = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_create(
            ptr::null(),
            ptr::null_mut(),
            new_attribute,
            &raw mut modify_request,
        ),
        SUCCESS
    );
    current_present = u8::MAX;
    current_copy = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_current_attribute(
            modify_request,
            &raw mut current_present,
            &raw mut current_copy,
        ),
        SUCCESS
    );
    assert_eq!(current_present, 0);
    assert!(current_copy.is_null());
    assert_eq!(
        client_abi::kmipkit_modify_attribute_request_current_attribute(
            modify_request,
            &raw mut current_present,
            ptr::null_mut(),
        ),
        ERROR_INVALID_INPUT
    );

    kmipkit_modify_attribute_request_release(modify_request);
    kmipkit_ttlv_item_release(new_attribute);
    kmipkit_delete_attribute_request_release(delete_request);
}

#[test]
fn typed_attribute_mutation_response_accessors_hide_other_operation_fields() {
    let request = ClientRequest::AdjustAttribute(AdjustAttributeRequest::new(
        Some(RESPONSE_FIXTURE_UID.to_owned()),
        AttributeReference::tag(0x0042_000a),
        AdjustmentType::from_raw(1),
        None,
    ));
    let success = successful_cryptographic_item(
        request,
        0x30,
        b"attribute-mutation-dispatch",
        vec![ttlv_item(0x0042_0094, 0x07, b"adjusted-object-1")],
    );

    let mut present = 0;
    let mut identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier record size fits ABI"),
        abi_version: 1,
        kind: 0,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 0,
    };
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_adjust_attribute(
            success,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );

    let other_operation_accessors: [extern "C" fn(
        *mut kmipkit_client_batch_item_response_t,
        *mut u8,
        *mut kmipkit_unique_identifier_v1_t,
    ) -> i32; 4] = [
        client_abi::kmipkit_client_batch_item_response_add_attribute,
        client_abi::kmipkit_client_batch_item_response_modify_attribute,
        client_abi::kmipkit_client_batch_item_response_set_attribute,
        client_abi::kmipkit_client_batch_item_response_delete_attribute,
    ];
    for accessor in other_operation_accessors {
        present = u8::MAX;
        identifier.kind = u32::MAX;
        identifier.text_data = std::ptr::dangling();
        identifier.text_length = u64::MAX;
        assert_eq!(
            accessor(success, &raw mut present, &raw mut identifier),
            SUCCESS
        );
        assert_eq!(present, 0);
        assert_eq!(identifier.kind, 0);
        assert!(identifier.text_data.is_null());
        assert_eq!(identifier.text_length, 0);
    }

    kmipkit_client_batch_item_response_release(success);
}

fn text_identifier_record(identifier: &'static [u8]) -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier record size fits the ABI"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: identifier.as_ptr(),
        text_length: u64::try_from(identifier.len()).expect("fixture length fits the ABI"),
        enumeration_value: 0,
        integer_value: 0,
    }
}

fn name_attribute_reference_record() -> crate::kmipkit_attribute_reference_v1_t {
    static VENDOR: &[u8] = b"KMIPKit";
    static NAME: &[u8] = b"Counter";
    crate::kmipkit_attribute_reference_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<crate::kmipkit_attribute_reference_v1_t>())
            .expect("attribute-reference record size fits the ABI"),
        abi_version: 1,
        kind: crate::KMIPKIT_ATTRIBUTE_REFERENCE_KIND_NAME,
        reserved: 0,
        tag_value: 0,
        reserved2: 0,
        vendor_identification_data: VENDOR.as_ptr(),
        vendor_identification_length: u64::try_from(VENDOR.len())
            .expect("fixture length fits the ABI"),
        attribute_name_data: NAME.as_ptr(),
        attribute_name_length: u64::try_from(NAME.len()).expect("fixture length fits the ABI"),
    }
}

fn attribute_item() -> Item {
    let tag = RawTag::new(0x0042_000a)
        .expect("fixture tag fits KMIP")
        .try_checked()
        .expect("fixture tag is allocated");
    Item::new(tag, Value::structure(Structure::new())).expect("fixture Attribute Item is valid")
}
