use super::*;

#[test]
fn get_usage_allocation_constructor_copies_identifier_and_preserves_signed_count() {
    let mut identifier_text = b"allocation-object".to_vec();
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: identifier_text.as_ptr(),
        text_length: u64::try_from(identifier_text.len()).expect("identifier length fits ABI"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_get_usage_allocation(
            &raw const identifier,
            i64::MAX,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    identifier_text.fill(b'X');

    let handle =
        reference_handle(item, Kind::BatchItem).expect("the returned batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item handle");
    };
    let ClientRequest::GetUsageAllocation(request) = batch_item.request() else {
        panic!("the batch item contains Get Usage Allocation");
    };
    assert_eq!(request.usage_limits_count(), i64::MAX);
    assert_eq!(
        request.unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "allocation-object".to_owned()
        ))
    );

    kmipkit_client_batch_item_release(item);
}

#[test]
fn get_usage_allocation_constructor_preserves_an_omitted_identifier() {
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_get_usage_allocation(ptr::null(), 12, &raw mut item),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the returned Get Usage Allocation batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item handle");
    };
    let ClientRequest::GetUsageAllocation(request) = batch_item.request() else {
        panic!("the batch item contains Get Usage Allocation");
    };
    assert_eq!(request.usage_limits_count(), 12);
    assert!(request.unique_identifier().is_none());

    kmipkit_client_batch_item_release(item);
}
