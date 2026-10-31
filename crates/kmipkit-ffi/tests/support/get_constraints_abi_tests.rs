use super::*;

#[test]
fn get_constraints_constructor_creates_the_empty_typed_request() {
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_get_constraints(&raw mut item),
        SUCCESS
    );
    assert!(!item.is_null());

    let handle =
        reference_handle(item, Kind::BatchItem).expect("the returned batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item handle");
    };
    assert!(matches!(
        batch_item.request(),
        ClientRequest::GetConstraints(_)
    ));

    kmipkit_client_batch_item_release(item);
}

#[test]
fn get_constraints_ttlv_getter_clears_outputs_for_invalid_item() {
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_constraints_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}
