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

fn derive_parameter_item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("test tag is a valid raw KMIP tag")
            .try_checked()
            .expect("test tag is allocated"),
        value,
    )
    .expect("test value forms a valid TTLV item")
}

fn derive_parameter_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("test structure stays within model limits");
    }
    structure
}

fn derive_parameter_handle(items: impl IntoIterator<Item = Item>) -> *mut kmipkit_ttlv_structure_t {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("test structure stays within model limits");
    }
    make_handle(Kind::Structure, HandleValue::Structure(structure))
}

fn derive_key_parameter_values_are_preserved(item: &Item) -> bool {
    item.with_value(|value| {
        let ValueView::Structure(parameters) = value else {
            return false;
        };
        let items = parameters.children();
        if items.iter().map(|item| item.tag().raw()).collect::<Vec<_>>()
            != [
                0x0042_002b,
                0x0042_003a,
                0x0042_0030,
                0x0042_0030,
                0x0042_0084,
                0x0042_003c,
                0x0054_0001,
            ]
        {
            return false;
        }
        let cryptographic_parameters_preserved = items[0].with_value(|value| {
            let ValueView::Structure(parameters) = value else {
                return false;
            };
            let children = parameters.children();
            children.len() == 1
                && children[0].tag().raw() == 0x0042_0028
                && children[0].with_value(
                    |value| matches!(value, ValueView::Enumeration(value) if *value == 0xfeed_beef),
                )
        });
        cryptographic_parameters_preserved
            && items[1].with_value(
                |value| matches!(value, ValueView::ByteString(value) if value == b"iv-secret"),
            )
            && items[2].with_value(
                |value| matches!(value, ValueView::ByteString(value) if value == b"first-secret"),
            )
            && items[3].with_value(
                |value| matches!(value, ValueView::ByteString(value) if value == b"second-secret"),
            )
            && items[4].with_value(
                |value| matches!(value, ValueView::ByteString(value) if value == b"salt-secret"),
            )
            && items[5].with_value(|value| matches!(value, ValueView::Integer(37)))
            && items[6].with_value(
                |value| matches!(value, ValueView::TextString(value) if value == "vendor-extension"),
            )
    })
}

fn derive_key_output_attributes_are_preserved(item: &Item) -> bool {
    item.with_value(|value| {
        let ValueView::Structure(attributes) = value else {
            return false;
        };
        let items = attributes.children();
        items.len() == 2
            && items[0].tag().raw() == 0x0042_0028
            && items[0]
                .with_value(|value| matches!(value, ValueView::Enumeration(value) if *value == 3))
            && items[1].tag().raw() == 0x0042_002a
            && items[1].with_value(|value| matches!(value, ValueView::Integer(128)))
    })
}

#[test]
fn derive_key_parameter_parser_preserves_typed_fields_repeated_data_and_extensions() {
    let structure = derive_parameter_handle([
        derive_parameter_item(0x0042_002b, Value::structure(Structure::new())),
        derive_parameter_item(0x0042_003a, Value::byte_string(b"iv".to_vec())),
        derive_parameter_item(0x0042_0030, Value::byte_string(b"first".to_vec())),
        derive_parameter_item(0x0042_0030, Value::byte_string(b"second".to_vec())),
        derive_parameter_item(
            0x0042_0084,
            Value::byte_string(b"salt-secret-sentinel".to_vec()),
        ),
        derive_parameter_item(0x0042_003c, Value::integer(37)),
        derive_parameter_item(0x0054_0001, Value::text_string("vendor".to_owned())),
    ]);

    let parameters = derive_key_parameters_from_handle(structure)
        .expect("valid derivation parameters retain known and extension fields");
    let formatted = format!("{parameters:?}");
    assert!(formatted.contains("has_cryptographic_parameters: true"));
    assert!(formatted.contains("has_initialization_vector: true"));
    assert!(formatted.contains("derivation_data_count: 2"));
    assert!(formatted.contains("has_salt: true"));
    assert!(formatted.contains("has_iteration_count: true"));
    assert!(formatted.contains("extension_count: 1"));
    assert!(!formatted.contains("first"));
    assert!(!formatted.contains("second"));
    assert!(!formatted.contains("salt-secret-sentinel"));
    release(structure, Kind::Structure);
}

#[test]
fn derive_key_constructor_preserves_exact_ttlv_parameter_values_and_extensions() {
    let structure = derive_parameter_handle([
        derive_parameter_item(
            0x0042_002b,
            Value::structure(derive_parameter_structure([derive_parameter_item(
                0x0042_0028,
                Value::enumeration(0xfeed_beef),
            )])),
        ),
        derive_parameter_item(0x0042_003a, Value::byte_string(b"iv-secret".to_vec())),
        derive_parameter_item(0x0042_0030, Value::byte_string(b"first-secret".to_vec())),
        derive_parameter_item(0x0042_0030, Value::byte_string(b"second-secret".to_vec())),
        derive_parameter_item(0x0042_0084, Value::byte_string(b"salt-secret".to_vec())),
        derive_parameter_item(0x0042_003c, Value::integer(37)),
        derive_parameter_item(
            0x0054_0001,
            Value::text_string("vendor-extension".to_owned()),
        ),
    ]);
    let attributes = derive_parameter_handle([
        derive_parameter_item(0x0042_0028, Value::enumeration(3)),
        derive_parameter_item(0x0042_002a, Value::integer(128)),
    ]);
    let identifier_bytes = b"source-key";
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("test ABI record size fits u32"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: identifier_bytes.as_ptr(),
        text_length: u64::try_from(identifier_bytes.len())
            .expect("identifier byte length fits u64"),
        enumeration_value: 0,
        integer_value: 0,
    };

    let request = derive_key_request_from_handles(
        2,
        &raw const identifier,
        1,
        0xfeed_beef,
        structure,
        attributes,
    )
    .expect("the FFI mapper preserves valid Derive Key inputs");
    let payload = request
        .into_ttlv_payload()
        .expect("the mapped request serializes as a valid Derive Key payload");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|item| item.tag().raw())
            .collect::<Vec<_>>(),
        [
            0x0042_0057,
            0x0042_0094,
            0x0042_0031,
            0x0042_0032,
            0x0042_0125
        ]
    );
    assert!(fields[1].with_value(
        |value| matches!(value, ValueView::TextString(value) if value.as_bytes() == identifier_bytes)
    ));

    assert!(
        derive_key_parameter_values_are_preserved(&fields[3]),
        "typed parameter values and extension survive FFI mapping"
    );
    assert!(
        derive_key_output_attributes_are_preserved(&fields[4]),
        "output attributes survive FFI mapping"
    );

    release(attributes, Kind::Structure);
    release(structure, Kind::Structure);
}

#[test]
fn derive_key_parameter_parser_rejects_invalid_handles_types_and_duplicate_singletons() {
    assert!(matches!(
        derive_key_parameters_from_handle(ptr::null_mut()),
        Err(ERROR_INVALID_INPUT)
    ));

    let wrong_kind: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Value, HandleValue::Value(Value::integer(1)));
    assert!(matches!(
        derive_key_parameters_from_handle(wrong_kind),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_kind, Kind::Value);

    let mismatched_payload: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Structure, HandleValue::Value(Value::integer(1)));
    assert!(matches!(
        derive_key_parameters_from_handle(mismatched_payload),
        Err(ERROR_INVALID_INPUT)
    ));
    release(mismatched_payload, Kind::Structure);

    let wrong_cryptographic_parameters =
        derive_parameter_handle([derive_parameter_item(0x0042_002b, Value::integer(1))]);
    assert!(matches!(
        derive_key_parameters_from_handle(wrong_cryptographic_parameters),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_cryptographic_parameters, Kind::Structure);

    let wrong_initialization_vector =
        derive_parameter_handle([derive_parameter_item(0x0042_003a, Value::integer(4))]);
    assert!(matches!(
        derive_key_parameters_from_handle(wrong_initialization_vector),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_initialization_vector, Kind::Structure);

    let wrong_derivation_data =
        derive_parameter_handle([derive_parameter_item(0x0042_0030, Value::integer(5))]);
    assert!(matches!(
        derive_key_parameters_from_handle(wrong_derivation_data),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_derivation_data, Kind::Structure);

    let wrong_salt =
        derive_parameter_handle([derive_parameter_item(0x0042_0084, Value::integer(6))]);
    assert!(matches!(
        derive_key_parameters_from_handle(wrong_salt),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_salt, Kind::Structure);

    let wrong_iteration_count = derive_parameter_handle([derive_parameter_item(
        0x0042_003c,
        Value::byte_string(vec![7]),
    )]);
    assert!(matches!(
        derive_key_parameters_from_handle(wrong_iteration_count),
        Err(ERROR_INVALID_INPUT)
    ));
    release(wrong_iteration_count, Kind::Structure);

    let singleton_pairs = [
        (
            0x0042_002b,
            Value::structure(Structure::new()),
            Value::structure(Structure::new()),
        ),
        (
            0x0042_003a,
            Value::byte_string(vec![1]),
            Value::byte_string(vec![2]),
        ),
        (
            0x0042_0084,
            Value::byte_string(vec![3]),
            Value::byte_string(vec![4]),
        ),
        (0x0042_003c, Value::integer(1), Value::integer(2)),
    ];
    for (tag, first, second) in singleton_pairs {
        let structure = derive_parameter_handle([
            derive_parameter_item(tag, first),
            derive_parameter_item(tag, second),
        ]);
        assert!(matches!(
            derive_key_parameters_from_handle(structure),
            Err(ERROR_INVALID_INPUT)
        ));
        release(structure, Kind::Structure);
    }
}

#[test]
fn derive_key_constructor_accepts_mixed_identifier_kinds_and_redacts_text() {
    let text_identifier = b"derive-source-secret";
    let struct_size = u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
        .expect("test ABI record size fits u32");
    let identifiers = [
        kmipkit_unique_identifier_v1_t {
            struct_size,
            abi_version: 1,
            kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
            reserved: 0,
            text_data: text_identifier.as_ptr(),
            text_length: u64::try_from(text_identifier.len()).expect("identifier length fits u64"),
            enumeration_value: 0,
            integer_value: 0,
        },
        kmipkit_unique_identifier_v1_t {
            struct_size,
            abi_version: 1,
            kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION,
            reserved: 0,
            text_data: ptr::null(),
            text_length: 0,
            enumeration_value: 0xfedc_ba98,
            integer_value: 0,
        },
        kmipkit_unique_identifier_v1_t {
            struct_size,
            abi_version: 1,
            kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
            reserved: 0,
            text_data: ptr::null(),
            text_length: 0,
            enumeration_value: 0,
            integer_value: -17,
        },
    ];
    let parameters = derive_parameter_handle([]);
    let attributes = derive_parameter_handle([]);
    let mut item = ptr::null_mut();

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_derive_key(
            0xfedc_ba98,
            identifiers.as_ptr(),
            u64::try_from(identifiers.len()).expect("identifier count fits u64"),
            0x8000_0001,
            parameters,
            attributes,
            &raw mut item,
        )
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("Derive Key item is live");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("Derive Key handle contains a batch item");
    };
    let ClientRequest::DeriveKey(request) = batch_item.request() else {
        panic!("batch item contains the requested operation");
    };
    let formatted = format!("{request:?}");
    assert!(formatted.contains("object_type: 4275878552"));
    assert!(formatted.contains("unique_identifier_count: 3"));
    assert!(formatted.contains("derivation_method: 2147483649"));
    assert!(!formatted.contains("derive-source-secret"));

    release(item, Kind::BatchItem);
    release(attributes, Kind::Structure);
    release(parameters, Kind::Structure);
}

#[test]
fn derive_key_constructor_rejects_null_required_parameter_and_attribute_handles() {
    let struct_size = u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
        .expect("test ABI record size fits u32");
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size,
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: -17,
    };
    let parameters = derive_parameter_handle([]);
    let attributes = derive_parameter_handle([]);
    let mut item = ptr::null_mut();

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            &raw const identifier,
            1,
            1,
            ptr::null_mut(),
            attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            &raw const identifier,
            1,
            1,
            parameters,
            ptr::null_mut(),
            &raw mut item,
        )
    );
    assert!(item.is_null());

    release(attributes, Kind::Structure);
    release(parameters, Kind::Structure);
}

#[test]
fn derive_key_constructor_rejects_invalid_counts_identifiers_and_required_handles() {
    let mut item = ptr::null_mut();
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            ptr::null(),
            0,
            1,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        )
    );
    assert!(item.is_null());

    assert_eq!(
        ERROR_RESOURCE_LIMIT,
        kmipkit_client_batch_item_derive_key(
            4,
            ptr::null(),
            MAX_DERIVE_KEY_SOURCE_IDENTIFIERS + 1,
            1,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        )
    );
    assert!(item.is_null());

    let parameters = derive_parameter_handle([]);
    let attributes = derive_parameter_handle([]);
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            ptr::null(),
            1,
            1,
            parameters,
            attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    let invalid_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: 0,
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 1,
    };
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            &raw const invalid_identifier,
            1,
            1,
            parameters,
            attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    let oversized_identifier = vec![b'x'; usize::try_from(MAX_FFI_INPUT_BYTES).unwrap() + 1];
    let oversized = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("test ABI record size fits u32"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: oversized_identifier.as_ptr(),
        text_length: u64::try_from(oversized_identifier.len()).expect("test input length fits u64"),
        enumeration_value: 0,
        integer_value: 0,
    };
    assert_eq!(
        ERROR_RESOURCE_LIMIT,
        kmipkit_client_batch_item_derive_key(
            4,
            &raw const oversized,
            1,
            1,
            parameters,
            attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    release(attributes, Kind::Structure);
    release(parameters, Kind::Structure);
}

#[test]
fn derive_key_constructor_rejects_aggregate_text_limit_and_malformed_parameter_handles() {
    let struct_size = u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
        .expect("test ABI record size fits u32");
    let text = vec![b'x'; 9 * 1024 * 1024];
    let identifiers =
        [text.as_slice(), text.as_slice()].map(|bytes| kmipkit_unique_identifier_v1_t {
            struct_size,
            abi_version: 1,
            kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
            reserved: 0,
            text_data: bytes.as_ptr(),
            text_length: u64::try_from(bytes.len()).expect("identifier length fits u64"),
            enumeration_value: 0,
            integer_value: 0,
        });
    let parameters = derive_parameter_handle([]);
    let attributes = derive_parameter_handle([]);
    let mut item = ptr::null_mut();

    assert_eq!(
        ERROR_RESOURCE_LIMIT,
        kmipkit_client_batch_item_derive_key(
            4,
            identifiers.as_ptr(),
            u64::try_from(identifiers.len()).expect("identifier count fits u64"),
            1,
            parameters,
            attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    let malformed_parameters: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Structure, HandleValue::Value(Value::integer(1)));
    let valid_identifier = kmipkit_unique_identifier_v1_t {
        struct_size,
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 7,
    };
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            &raw const valid_identifier,
            1,
            1,
            malformed_parameters,
            attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    let malformed_attributes: *mut kmipkit_ttlv_structure_t =
        make_handle(Kind::Structure, HandleValue::Value(Value::integer(1)));
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_derive_key(
            4,
            &raw const valid_identifier,
            1,
            1,
            parameters,
            malformed_attributes,
            &raw mut item,
        )
    );
    assert!(item.is_null());

    release(malformed_attributes, Kind::Structure);
    release(malformed_parameters, Kind::Structure);
    release(attributes, Kind::Structure);
    release(parameters, Kind::Structure);
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
fn encrypt_and_decrypt_batch_item_constructors_preserve_operation_and_redact_inputs() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let data = make_handle(
        Kind::Value,
        HandleValue::Value(Value::byte_string(b"operation-data-secret".to_vec())),
    );
    let identifier_bytes = b"key-id-secret";
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("test ABI record size fits u32"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: identifier_bytes.as_ptr(),
        text_length: u64::try_from(identifier_bytes.len()).expect("identifier length fits u64"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let iv_bytes = b"iv-secret";
    let iv = kmipkit_byte_span_v1_t {
        data: iv_bytes.as_ptr(),
        length: u64::try_from(iv_bytes.len()).expect("IV length fits u64"),
    };
    let correlation_bytes = b"correlation-secret";
    let correlation = kmipkit_byte_span_v1_t {
        data: correlation_bytes.as_ptr(),
        length: u64::try_from(correlation_bytes.len()).expect("correlation length fits u64"),
    };
    let additional_data_bytes = b"aad-secret";
    let additional_data = kmipkit_byte_span_v1_t {
        data: additional_data_bytes.as_ptr(),
        length: u64::try_from(additional_data_bytes.len()).expect("AAD length fits u64"),
    };
    let tag_bytes = b"tag-secret";
    let tag = kmipkit_byte_span_v1_t {
        data: tag_bytes.as_ptr(),
        length: u64::try_from(tag_bytes.len()).expect("tag length fits u64"),
    };

    let mut encrypt_item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_encrypt(
            &raw const identifier,
            parameters,
            data,
            &raw const iv,
            &raw const correlation,
            1,
            2,
            &raw const additional_data,
            &raw mut encrypt_item,
        )
    );
    assert_operation(encrypt_item, |request| {
        matches!(request, ClientRequest::Encrypt(_))
    });
    let stored = reference_handle(encrypt_item, Kind::BatchItem).expect("Encrypt item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Encrypt item contains a request");
    };
    let formatted = format!("{:?}", stored.request());
    assert_eq!(formatted, "Encrypt([REDACTED])");
    assert!(!formatted.contains("operation-data-secret"));
    assert!(!formatted.contains("iv-secret"));
    assert!(!formatted.contains("correlation-secret"));
    assert!(!formatted.contains("aad-secret"));
    assert!(!formatted.contains("key-id-secret"));

    let mut decrypt_item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_decrypt(
            &raw const identifier,
            parameters,
            data,
            &raw const iv,
            &raw const tag,
            &raw const correlation,
            1,
            2,
            &raw const additional_data,
            &raw mut decrypt_item,
        )
    );
    assert_operation(decrypt_item, |request| {
        matches!(request, ClientRequest::Decrypt(_))
    });
    let stored = reference_handle(decrypt_item, Kind::BatchItem).expect("Decrypt item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Decrypt item contains a request");
    };
    let formatted = format!("{:?}", stored.request());
    assert_eq!(formatted, "Decrypt([REDACTED])");
    assert!(!formatted.contains("operation-data-secret"));
    assert!(!formatted.contains("tag-secret"));
    assert!(!formatted.contains("aad-secret"));
    assert!(!formatted.contains("key-id-secret"));

    release(decrypt_item, Kind::BatchItem);
    release(encrypt_item, Kind::BatchItem);
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
