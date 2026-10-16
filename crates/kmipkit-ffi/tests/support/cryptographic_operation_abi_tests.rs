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

#[test]
fn copied_byte_string_operation_data_survives_source_value_handle_release() {
    let source_bytes = b"operation-data-copy";
    let source = make_handle(
        Kind::Value,
        HandleValue::Value(Value::byte_string(source_bytes.to_vec())),
    );

    let copied = copy_operation_data(source)
        .expect("valid Byte String operation data copies")
        .expect("non-null input creates operation data");
    release(source, Kind::Value);

    let OperationData::ByteString(copied_bytes) = copied else {
        panic!("Byte String input remains a Byte String");
    };
    copied_bytes.with_bytes(|bytes| assert_eq!(bytes, source_bytes));
}

#[test]
fn operation_data_copies_integer_and_rejects_wrong_payload_variants() {
    let integer = make_handle(Kind::Value, HandleValue::Value(Value::integer(-17)));
    let copied = copy_operation_data(integer)
        .expect("valid Integer operation data copies")
        .expect("non-null input creates operation data");
    assert!(matches!(copied, OperationData::Integer(-17)));
    release(integer, Kind::Value);

    let boolean = make_handle(Kind::Value, HandleValue::Value(Value::boolean(true)));
    assert!(matches!(
        copy_operation_data(boolean),
        Err(ERROR_INVALID_INPUT)
    ));
    release(boolean, Kind::Value);

    let mismatched_payload = make_handle(Kind::Value, HandleValue::Structure(Structure::new()));
    assert!(matches!(
        copy_operation_data(mismatched_payload),
        Err(ERROR_INVALID_INPUT)
    ));
    release(mismatched_payload, Kind::Value);
}

#[test]
fn mac_constructor_preserves_present_cryptographic_parameters() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_mac(
            ptr::null(),
            parameters,
            ptr::null_mut(),
            ptr::null(),
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
    assert!(format!("{:?}", stored.request()).contains("has_cryptographic_parameters: true"));
    release(item, Kind::BatchItem);
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

#[test]
fn cryptographic_batch_item_constructor_rejects_wrong_kind_operation_data_handle() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let wrong_kind: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::null_mut();

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_hash(
            parameters,
            wrong_kind.cast(),
            ptr::null(),
            0,
            0,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    release(wrong_kind, Kind::Structure);
    release(parameters, Kind::Structure);
}

#[test]
fn hash_batch_item_constructor_requires_cryptographic_parameters() {
    let mut item = ptr::null_mut();
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_hash(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            0,
            0,
            &raw mut item,
        )
    );
    assert!(item.is_null());
}

#[test]
fn certify_batch_item_constructor_preserves_open_request_type_and_redacts_request_value() {
    let mut secret = *b"certify-request-sentinel";
    let request_value = kmipkit_byte_span_v1_t {
        data: secret.as_ptr(),
        length: u64::try_from(secret.len()).expect("test request value fits the ABI"),
    };
    let mut item = ptr::null_mut();

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_certify(
            ptr::null(),
            1,
            0xfedc_ba98,
            &raw const request_value,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        )
    );
    assert_operation(item, |request| matches!(request, ClientRequest::Certify(_)));
    let stored = reference_handle(item, Kind::BatchItem).expect("Certify item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Certify item contains a request");
    };
    let formatted = format!("{:?}", stored.request());
    assert!(formatted.contains("certificate_request_type: Some(4275878552)"));
    assert!(formatted.contains("has_certificate_request_value: true"));
    assert!(!formatted.contains("certify-request-sentinel"));

    secret.fill(0);
    release(item, Kind::BatchItem);
}

#[test]
fn certify_batch_item_constructor_preserves_absent_request_type() {
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_certify(
            ptr::null(),
            0,
            0xfedc_ba98,
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        )
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("Certify item is live");
    let HandleValue::BatchItem(stored) = &handle.value else {
        panic!("Certify item contains a request");
    };
    let ClientRequest::Certify(request) = stored.request() else {
        panic!("Certify handle contains a Certify request");
    };
    assert!(format!("{request:?}").contains("certificate_request_type: None"));
    release(item, Kind::BatchItem);
}

#[test]
fn certify_batch_item_constructor_rejects_invalid_presence_discriminators() {
    let mut item = ptr::null_mut();
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_certify(
            ptr::null(),
            2,
            0,
            ptr::null(),
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        )
    );
    assert!(item.is_null());
}
