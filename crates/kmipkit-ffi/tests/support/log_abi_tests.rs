use super::*;

#[test]
fn log_constructor_preserves_and_redacts_message() {
    const MESSAGE: &[u8] = b"log-message-private-sentinel";
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_log(MESSAGE.as_ptr(), MESSAGE.len() as u64, &raw mut item),
        SUCCESS
    );
    assert!(!item.is_null());

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the returned Log batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the Log constructor returns a batch item handle");
    };
    let ClientRequest::Log(request) = batch_item.request() else {
        panic!("the batch item contains a Log request");
    };
    request.with_log_message(|message| assert_eq!(message.as_bytes(), MESSAGE));
    assert!(!format!("{request:?}").contains("log-message-private-sentinel"));

    kmipkit_client_batch_item_release(item);
}

#[test]
fn log_constructor_rejects_invalid_utf8_and_clears_output() {
    let invalid_message = [0xff];
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_log(
            invalid_message.as_ptr(),
            invalid_message.len() as u64,
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
}

#[test]
fn log_response_ttlv_getter_clears_outputs_for_invalid_item() {
    let mut present = u8::MAX;
    let mut response_ttlv = ptr::dangling_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_log_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}
