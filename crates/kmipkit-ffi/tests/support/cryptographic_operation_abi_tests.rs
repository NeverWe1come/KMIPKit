use std::ptr;

use super::*;
use crate::kmipkit_byte_span_v1_t;

fn assert_operation(item: *mut kmipkit_client_batch_item_t, expected: fn(&ClientRequest) -> bool) {
    let handle = reference_handle(item, Kind::BatchItem)
        .expect("constructor returned a live batch-item handle");
    let HandleValue::BatchItem(item) = &handle.value else {
        panic!("batch-item handle stores the expected payload");
    };
    assert!(expected(item.request()));
}

#[test]
fn cryptographic_batch_item_constructors_preserve_operation_kind_and_redact_inputs() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let data = make_handle(
        Kind::Value,
        HandleValue::Value(Value::enumeration(0xfedc_ba98)),
    );
    let correlation_bytes = [0x71, 0x00, 0x72];
    let correlation = kmipkit_byte_span_v1_t {
        data: correlation_bytes.as_ptr(),
        length: u64::try_from(correlation_bytes.len()).expect("test value fits ABI length"),
    };
    let mac_bytes = [0x4d, 0x41, 0x43];
    let mac_data = kmipkit_byte_span_v1_t {
        data: mac_bytes.as_ptr(),
        length: u64::try_from(mac_bytes.len()).expect("test value fits ABI length"),
    };
    let digest_bytes = [0x44, 0x49, 0x47];
    let digest = kmipkit_byte_span_v1_t {
        data: digest_bytes.as_ptr(),
        length: u64::try_from(digest_bytes.len()).expect("test value fits ABI length"),
    };
    let signature_bytes = [0x53, 0x49, 0x47];
    let signature = kmipkit_byte_span_v1_t {
        data: signature_bytes.as_ptr(),
        length: u64::try_from(signature_bytes.len()).expect("test value fits ABI length"),
    };

    assert_hash_constructor(parameters, data, &raw const correlation);
    assert_mac_constructor(data, &raw const correlation, "fedcba98", "710072");
    assert_mac_verify_constructor(parameters, data, &raw const mac_data, "4d4143");
    assert_sign_constructor(parameters, data, &raw const digest, "444947");
    assert_signature_verify_constructor(parameters, data, &raw const digest, &raw const signature);

    release(data, Kind::Value);
    release(parameters, Kind::Structure);
}

fn assert_hash_constructor(
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    correlation: *const kmipkit_byte_span_v1_t,
) {
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_hash(parameters, data, correlation, 1, 2, &raw mut item)
    );
    assert_operation(item, |request| matches!(request, ClientRequest::Hash(_)));
    let stored = reference_handle(item, Kind::BatchItem).expect("Hash item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Hash item contains a request");
    };
    let formatted = format!("{:?}", stored.request());
    assert!(formatted.contains("HashRequest"));
    assert!(!formatted.contains("fedcba98"));
    assert!(!formatted.contains("710072"));
    release(item, Kind::BatchItem);
}

fn assert_mac_constructor(
    data: *mut kmipkit_ttlv_value_t,
    correlation: *const kmipkit_byte_span_v1_t,
    data_sentinel: &str,
    correlation_sentinel: &str,
) {
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_mac(
            ptr::null(),
            ptr::null_mut(),
            data,
            correlation,
            0,
            0,
            &raw mut item,
        )
    );
    assert_operation(item, |request| matches!(request, ClientRequest::Mac(_)));
    let stored = reference_handle(item, Kind::BatchItem).expect("MAC item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("MAC item contains a request");
    };
    let formatted = format!("{:?}", stored.request());
    assert!(!formatted.contains(data_sentinel));
    assert!(!formatted.contains(correlation_sentinel));
    release(item, Kind::BatchItem);
}

fn assert_mac_verify_constructor(
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    mac_data: *const kmipkit_byte_span_v1_t,
    mac_sentinel: &str,
) {
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_mac_verify(
            ptr::null(),
            parameters,
            data,
            mac_data,
            ptr::null(),
            2,
            1,
            &raw mut item,
        )
    );
    assert_operation(item, |request| {
        matches!(request, ClientRequest::MacVerify(_))
    });
    let stored = reference_handle(item, Kind::BatchItem).expect("MAC Verify item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("MAC Verify item contains a request");
    };
    assert!(!format!("{:?}", stored.request()).contains(mac_sentinel));
    release(item, Kind::BatchItem);
}

fn assert_sign_constructor(
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    digested_data: *const kmipkit_byte_span_v1_t,
    digest_sentinel: &str,
) {
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_sign(
            ptr::null(),
            parameters,
            data,
            digested_data,
            ptr::null(),
            0,
            0,
            &raw mut item,
        )
    );
    assert_operation(item, |request| matches!(request, ClientRequest::Sign(_)));
    let stored = reference_handle(item, Kind::BatchItem).expect("Sign item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Sign item contains a request");
    };
    assert!(!format!("{:?}", stored.request()).contains(digest_sentinel));
    release(item, Kind::BatchItem);
}

fn assert_signature_verify_constructor(
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    digested_data: *const kmipkit_byte_span_v1_t,
    signature_data: *const kmipkit_byte_span_v1_t,
) {
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_signature_verify(
            ptr::null(),
            parameters,
            data,
            digested_data,
            signature_data,
            ptr::null(),
            0,
            0,
            &raw mut item,
        )
    );
    assert_operation(item, |request| {
        matches!(request, ClientRequest::SignatureVerify(_))
    });
    let stored = reference_handle(item, Kind::BatchItem).expect("Signature Verify item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Signature Verify item contains a request");
    };
    let formatted = format!("{:?}", stored.request());
    assert!(!formatted.contains("444947"));
    assert!(!formatted.contains("534947"));
    release(item, Kind::BatchItem);
}

#[test]
fn cryptographic_batch_item_constructors_reject_invalid_indicator_codes() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::null_mut();

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_hash(
            parameters,
            ptr::null_mut(),
            ptr::null(),
            3,
            0,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    release(parameters, Kind::Structure);
}
