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

macro_rules! assert_lifecycle_request_identifier {
    ($call:ident, $variant:ident, $identifier:expr, $expected:expr $(,)?) => {{
        let mut item = ptr::null_mut();
        let identifier = $identifier;
        assert_eq!(SUCCESS, $call(&raw const identifier, &raw mut item));
        let handle = reference_handle(item, Kind::BatchItem)
            .expect("lifecycle constructor returns a valid item handle");
        let HandleValue::BatchItem(batch_item) = &handle.value else {
            panic!("the constructor returns a batch item");
        };
        let ClientRequest::$variant(request) = batch_item.request() else {
            panic!("the batch item preserves its lifecycle operation");
        };
        assert_eq!(request.unique_identifier(), Some(&$expected));
        kmipkit_client_batch_item_release(item);
    }};
}

#[test]
fn lifecycle_constructors_preserve_supplied_identifier_kind_and_value() {
    let mut activate_text = b"activate-target".to_vec();
    let activate_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: activate_text.as_ptr(),
        text_length: u64::try_from(activate_text.len()).expect("text length fits ABI"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let expected_activate = UniqueIdentifier::TextString("activate-target".to_owned());
    let archive_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0x8fed_cba9,
        integer_value: 0,
    };
    let expected_archive = UniqueIdentifier::Enumeration(0x8fed_cba9);
    let destroy_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: -723,
    };
    let expected_destroy = UniqueIdentifier::Integer(-723);
    let mut recover_text = b"recover-target".to_vec();
    let recover_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: recover_text.as_ptr(),
        text_length: u64::try_from(recover_text.len()).expect("text length fits ABI"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let expected_recover = UniqueIdentifier::TextString("recover-target".to_owned());

    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_activate,
        Activate,
        activate_identifier,
        expected_activate
    );
    activate_text.fill(b'x');
    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_archive,
        Archive,
        archive_identifier,
        expected_archive
    );
    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_destroy,
        Destroy,
        destroy_identifier,
        expected_destroy
    );
    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_recover,
        Recover,
        recover_identifier,
        expected_recover
    );
    recover_text.fill(b'x');
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
