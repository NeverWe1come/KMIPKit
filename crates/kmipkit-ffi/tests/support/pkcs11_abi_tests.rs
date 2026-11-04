use super::*;

#[test]
fn pkcs11_constructor_copies_handles_and_redacts_correlation_value() {
    const CORRELATION: &[u8] = b"pkcs11-correlation-private-sentinel";
    let function = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let interface = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let input = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let correlation = crate::kmipkit_byte_span_v1_t {
        data: CORRELATION.as_ptr(),
        length: u64::try_from(CORRELATION.len()).expect("test correlation length fits ABI"),
    };
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_pkcs11(
            function,
            interface,
            &raw const correlation,
            input,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the returned PKCS#11 batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item handle");
    };
    assert!(matches!(batch_item.request(), ClientRequest::Pkcs11(_)));
    assert!(!format!("{batch_item:?}").contains("pkcs11-correlation-private-sentinel"));

    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_structure_release(input);
    kmipkit_ttlv_structure_release(interface);
    kmipkit_ttlv_structure_release(function);
}

#[test]
fn pkcs11_constructor_requires_function_and_clears_output() {
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_pkcs11(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            ptr::null_mut(),
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
}

#[test]
fn pkcs11_constructor_copies_caller_ordered_payload_extensions() {
    let function = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let extension = Item::new(
        RawTag::new(0x0042_02fe)
            .expect("fixture tag fits the KMIP tag field")
            .try_checked()
            .expect("fixture tag is structurally valid"),
        Value::enumeration(7),
    )
    .expect("the opaque extension is a structurally valid TTLV item");
    let extension_handle = make_handle(Kind::Item, HandleValue::Item(extension));
    let mut extensions = [extension_handle];
    let mut item = ptr::null_mut();

    // SAFETY: both input handles and the output slot remain live and correctly aligned.
    assert_eq!(
        unsafe {
            kmipkit_client_batch_item_pkcs11_with_extensions_v1(
                function,
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                extensions.as_mut_ptr(),
                1,
                &raw mut item,
            )
        },
        SUCCESS
    );
    assert!(!item.is_null());
    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the returned PKCS#11 batch item handle is valid");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item handle");
    };
    assert!(matches!(batch_item.request(), ClientRequest::Pkcs11(_)));
    assert!(reference_handle(extension_handle, Kind::Item).is_ok());

    kmipkit_client_batch_item_release(item);
    kmipkit_ttlv_item_release(extension_handle);
    kmipkit_ttlv_structure_release(function);
}

#[test]
fn pkcs11_response_ttlv_getter_clears_outputs_for_invalid_item() {
    let mut present = u8::MAX;
    let mut response_ttlv = ptr::dangling_mut();

    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
}

#[test]
fn pkcs11_typed_response_getters_clear_all_outputs_for_invalid_item() {
    let mut present = u8::MAX;
    let mut structure = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_interface_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut structure,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(structure.is_null());

    present = u8::MAX;
    structure = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_function_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut structure,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(structure.is_null());

    let mut correlation = crate::kmipkit_byte_span_v1_t {
        data: ptr::dangling(),
        length: u64::MAX,
    };
    present = u8::MAX;
    assert_eq!(
        test_pointer_ffi_call!(
            client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
                ptr::null_mut(),
                &raw mut present,
                &raw mut correlation,
            )
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(correlation.data.is_null());
    assert_eq!(correlation.length, 0);

    present = u8::MAX;
    structure = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut structure,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(structure.is_null());

    present = u8::MAX;
    structure = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_return_code_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut structure,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(structure.is_null());
}
