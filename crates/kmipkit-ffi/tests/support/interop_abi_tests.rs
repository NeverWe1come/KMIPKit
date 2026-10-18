use super::*;

#[test]
fn interop_constructor_preserves_open_function_and_redacts_identifier() {
    const IDENTIFIER: &[u8] = b"interop-private-sentinel";
    const FUNCTION: u32 = 0xf123_4567;
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_interop(
            FUNCTION,
            IDENTIFIER.as_ptr(),
            IDENTIFIER.len() as u64,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the returned Interop batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the Interop constructor returns a batch item handle");
    };
    let ClientRequest::Interop(request) = batch_item.request() else {
        panic!("the batch item contains an Interop request");
    };
    assert_eq!(request.interop_function(), FUNCTION);
    request.with_interop_identifier(|identifier| assert_eq!(identifier.as_bytes(), IDENTIFIER));
    assert!(!format!("{request:?}").contains("interop-private-sentinel"));

    kmipkit_client_batch_item_release(item);
}

#[test]
fn interop_constructor_rejects_invalid_utf8_and_clears_output() {
    let invalid_identifier = [0xff];
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_interop(
            0,
            invalid_identifier.as_ptr(),
            invalid_identifier.len() as u64,
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
}

#[test]
fn interop_response_ttlv_getter_clears_outputs_for_invalid_item() {
    let mut present = u8::MAX;
    let mut response_ttlv = ptr::dangling_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_interop_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}
