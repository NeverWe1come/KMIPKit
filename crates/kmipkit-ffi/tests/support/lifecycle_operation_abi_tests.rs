use super::*;

macro_rules! assert_lifecycle_request {
    ($call:ident, $variant:ident $(,)?) => {{
        let mut item = ptr::null_mut();
        assert_eq!(SUCCESS, $call(ptr::null(), &raw mut item));
        let handle = reference_handle(item, Kind::BatchItem)
            .expect("lifecycle constructor returns a valid item handle");
        let HandleValue::BatchItem(batch_item) = &handle.value else {
            panic!("the constructor returns a batch item");
        };
        assert!(matches!(batch_item.request(), ClientRequest::$variant(request)
            if request.unique_identifier().is_none()));
        kmipkit_client_batch_item_release(item);
    }};
}

#[test]
fn lifecycle_constructors_preserve_optional_identifier_omission() {
    assert_lifecycle_request!(kmipkit_client_batch_item_activate, Activate,);
    assert_lifecycle_request!(kmipkit_client_batch_item_archive, Archive,);
    assert_lifecycle_request!(kmipkit_client_batch_item_destroy, Destroy,);
    assert_lifecycle_request!(kmipkit_client_batch_item_recover, Recover,);
}

macro_rules! assert_successful_lifecycle_response {
    ($request:expr, $operation:expr, $getter:ident) => {{
        let identifier = b"lifecycle-response";
        let response_item = successful_cryptographic_item(
            $request,
            $operation,
            b"lifecycle-batch-item",
            vec![ttlv_item(0x0042_0094, 0x07, identifier)],
        );
        let mut present = 0;
        let mut output_identifier = empty_unique_identifier_output_v1();
        assert_eq!(
            SUCCESS,
            $getter(response_item, &raw mut present, &raw mut output_identifier)
        );
        assert_eq!(present, 1);
        assert_response_identifier(&output_identifier, identifier);
        kmipkit_client_batch_item_response_release(response_item);
    }};
}

#[test]
fn lifecycle_response_accessors_return_identifier_only_for_success() {
    assert_successful_lifecycle_response!(
        ClientRequest::Activate(ActivateRequest::new(None)),
        0x12,
        kmipkit_client_batch_item_response_activate
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Archive(ArchiveRequest::new(None)),
        0x15,
        kmipkit_client_batch_item_response_archive
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Destroy(DestroyRequest::new(None)),
        0x14,
        kmipkit_client_batch_item_response_destroy
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Recover(RecoverRequest::new(None)),
        0x16,
        kmipkit_client_batch_item_response_recover
    );
}

#[test]
fn lifecycle_response_accessors_clear_identifier_for_failure() {
    let response_item = failed_cryptographic_item(
        ClientRequest::Recover(RecoverRequest::new(None)),
        0x16,
        b"lifecycle-batch-item",
    );
    let mut present = 1;
    let mut output_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_recover(
            response_item,
            &raw mut present,
            &raw mut output_identifier,
        )
    );
    assert_eq!(present, 0);
    assert_eq!(output_identifier.kind, 0);
    kmipkit_client_batch_item_response_release(response_item);
}
