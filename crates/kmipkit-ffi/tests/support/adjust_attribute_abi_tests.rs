use super::*;
use kmipkit_client::ClientRequest;
use kmipkit_ttlv::ValueView;

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
// SAFETY: this declaration matches the compiled C ABI consumer helper.
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_build_adjust_attribute(
        out_request: *mut *mut kmipkit_client_batch_item_t,
    ) -> i32;
}

#[test]
fn compiled_c_consumer_copies_adjust_attribute_value_and_preserves_field_order() {
    let mut request_handle = ptr::null_mut();
    assert_eq!(
        // SAFETY: the helper initializes the valid writable output slot before returning.
        unsafe { kmipkit_client_abi_c_consumer_build_adjust_attribute(&raw mut request_handle) },
        1,
        "the C consumer builds Adjust Attribute from its versioned reference and value"
    );
    assert!(!request_handle.is_null());

    let handle = reference_handle(request_handle, Kind::BatchItem)
        .expect("the C consumer returns an Adjust Attribute batch item");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the returned C handle contains a batch item");
    };
    let ClientRequest::AdjustAttribute(request) = batch_item.request() else {
        panic!("the C consumer constructs Adjust Attribute");
    };
    let payload = request
        .to_ttlv_payload()
        .expect("the C-built Adjust Attribute payload is valid");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].tag().raw(), 0x0042_013b);
    assert!(fields[0].with_value(|value| matches!(value, ValueView::Enumeration(0x0042_000a))));
    assert_eq!(fields[1].tag().raw(), 0x0042_0158);
    assert!(fields[1].with_value(|value| matches!(value, ValueView::Enumeration(1))));
    assert_eq!(fields[2].tag().raw(), 0x0042_0162);
    assert!(fields[2].with_value(|value| matches!(value, ValueView::Integer(5))));

    kmipkit_client_batch_item_release(request_handle);
}
