use super::*;
use crate::{kmipkit_server_listener_config_v1_t, kmipkit_transport_input_v1_t};
use kmipkit_test_support::EphemeralPki;

const TEST_INPUT_MEMORY: u32 = 1;
const TEST_ENCODING_DER: u32 = 2;

fn test_input(source_kind: u32, encoding: u32, bytes: &[u8]) -> kmipkit_transport_input_v1_t {
    kmipkit_transport_input_v1_t {
        source_kind,
        encoding,
        data: kmipkit_byte_span_v1_t {
            data: if bytes.is_empty() {
                ptr::null()
            } else {
                bytes.as_ptr()
            },
            length: bytes.len() as u64,
        },
    }
}

#[test]
fn listener_helpers_validate_timeouts_text_and_object_tags() {
    assert_eq!(timeout(0, 0), Ok(None));
    assert_eq!(timeout(2, 7), Ok(Some(Duration::new(2, 7))));
    assert_eq!(timeout(0, 1_000_000_000), Err(ERROR_INVALID_INPUT));

    assert_eq!(
        optional_text_input(kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0
        }),
        Ok(None)
    );
    let text = b"vendor";
    assert_eq!(
        optional_text_input(kmipkit_byte_span_v1_t {
            data: text.as_ptr(),
            length: text.len() as u64
        }),
        Ok(Some("vendor".to_owned()))
    );
    let invalid_utf8 = [0xff];
    assert_eq!(
        optional_text_input(kmipkit_byte_span_v1_t {
            data: invalid_utf8.as_ptr(),
            length: 1
        }),
        Err(ERROR_INVALID_INPUT)
    );
    assert_eq!(
        optional_text_input(kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: u64::from(u32::try_from(MAX_TEXT_BYTES).expect("test text limit fits u32"),)
                + 1,
        }),
        Err(ERROR_RESOURCE_LIMIT)
    );

    for (tag, expected) in [
        (0x0042_0013, 1),
        (0x0042_008F, 2),
        (0x0042_006D, 3),
        (0x0042_0064, 4),
        (0x0042_0089, 5),
        (0x0042_0085, 7),
        (0x0042_005B, 8),
        (0x0042_00BD, 9),
        (0x0042_0018, 10),
        (0x0042_01A1, 13),
        (0x0042_01A6, 14),
        (0x0042_01A7, 15),
        (0x0042_01AF, 16),
    ] {
        assert_eq!(managed_object_type_for_tag(tag).raw(), expected);
    }
    assert_eq!(managed_object_type_for_tag(0x0054_0001).raw(), 0);
}

#[test]
fn listener_config_rejects_invalid_version_reserved_and_encoding_fields() {
    assert!(matches!(
        listener_config_from_v1(ptr::null()),
        Err(ERROR_INVALID_INPUT)
    ));

    let mut invalid = kmipkit_server_listener_config_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_server_listener_config_v1_t>())
            .expect("listener config fits ABI size field"),
        abi_version: 1,
        transport_kind: SERVER_LISTENER_HTTPS,
        wire_encoding: 0,
        reserved: 0,
        bind_host: kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        },
        bind_port: 0,
        reserved2: 0,
        listener_certificates: ptr::null(),
        listener_certificate_count: 0,
        listener_private_key: test_input(TEST_INPUT_MEMORY, TEST_ENCODING_DER, &[]),
        trust_certificates: ptr::null(),
        trust_certificate_count: 0,
        revocation_lists: ptr::null(),
        revocation_list_count: 0,
        target_path: kmipkit_byte_span_v1_t {
            data: ptr::null(),
            length: 0,
        },
        max_request_bytes: 0,
    };
    invalid.abi_version = 2;
    assert!(matches!(
        listener_config_from_v1(&raw const invalid),
        Err(ERROR_INVALID_INPUT)
    ));
    invalid.abi_version = 1;
    invalid.struct_size -= 1;
    assert!(matches!(
        listener_config_from_v1(&raw const invalid),
        Err(ERROR_INVALID_INPUT)
    ));
    invalid.struct_size += 1;
    invalid.reserved = 1;
    assert!(matches!(
        listener_config_from_v1(&raw const invalid),
        Err(ERROR_INVALID_INPUT)
    ));
    invalid.reserved = 0;
    invalid.reserved2 = 1;
    assert!(matches!(
        listener_config_from_v1(&raw const invalid),
        Err(ERROR_INVALID_INPUT)
    ));
    invalid.reserved2 = 0;
    invalid.transport_kind = 99;
    assert!(matches!(
        listener_config_from_v1(&raw const invalid),
        Err(ERROR_INVALID_INPUT)
    ));
}

#[test]
fn https_listener_binds_for_all_supported_wire_encoding_modes() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI generation succeeds");
    let identity = pki.server_identity();
    let chain = identity.certificate_chain_der();
    let certificates: Vec<kmipkit_transport_input_v1_t> = chain
        .iter()
        .map(|certificate| test_input(TEST_INPUT_MEMORY, TEST_ENCODING_DER, certificate))
        .collect();
    let roots = [test_input(
        TEST_INPUT_MEMORY,
        TEST_ENCODING_DER,
        pki.authority_certificate_der(),
    )];
    let host = b"127.0.0.1";

    for wire_encoding in 0..=SERVER_LISTENER_ENCODING_XML {
        let key = test_input(
            TEST_INPUT_MEMORY,
            TEST_ENCODING_DER,
            identity.private_key_der(),
        );
        let path = b"/kmip";
        let mut config = kmipkit_server_listener_config_v1_t {
            struct_size: u32::try_from(std::mem::size_of::<kmipkit_server_listener_config_v1_t>())
                .expect("listener config fits ABI size field"),
            abi_version: 1,
            transport_kind: SERVER_LISTENER_HTTPS,
            wire_encoding,
            reserved: 0,
            bind_host: kmipkit_byte_span_v1_t {
                data: host.as_ptr(),
                length: host.len() as u64,
            },
            bind_port: 0,
            reserved2: 0,
            listener_certificates: certificates.as_ptr(),
            listener_certificate_count: certificates.len() as u64,
            listener_private_key: key,
            trust_certificates: roots.as_ptr(),
            trust_certificate_count: roots.len() as u64,
            revocation_lists: ptr::null(),
            revocation_list_count: 0,
            target_path: kmipkit_byte_span_v1_t {
                data: path.as_ptr(),
                length: path.len() as u64,
            },
            max_request_bytes: 1_048_576,
        };
        assert!(listener_config_from_v1(&raw const config).is_ok());
        config.wire_encoding = 99;
        assert!(matches!(
            listener_config_from_v1(&raw const config),
            Err(ERROR_INVALID_INPUT)
        ));
        config.wire_encoding = wire_encoding;
        if wire_encoding == 0 {
            config.transport_kind = SERVER_LISTENER_RAW_TLS;
            assert!(matches!(
                listener_config_from_v1(&raw const config),
                Err(ERROR_INVALID_INPUT)
            ));
            config.transport_kind = SERVER_LISTENER_HTTPS;
        }
        let mut configuration = ptr::null_mut();
        assert_eq!(
            kmipkit_server_listener_config_create_v1(&raw const config, &raw mut configuration),
            SUCCESS
        );
        let mut listener = ptr::null_mut();
        assert_eq!(
            kmipkit_server_listener_bind_https(configuration, &raw mut listener),
            SUCCESS
        );
        let mut port = 0;
        assert_eq!(
            kmipkit_server_listener_port(listener, &raw mut port),
            SUCCESS
        );
        assert_ne!(port, 0);
        kmipkit_server_listener_release(listener);
        kmipkit_server_listener_config_release(configuration);
    }
}

#[test]
fn listener_ffi_initializes_outputs_and_rejects_missing_handles() {
    let mut listener = ptr::null_mut();
    assert_eq!(
        kmipkit_server_listener_bind_raw_tls(ptr::null_mut(), &raw mut listener),
        ERROR_INVALID_INPUT
    );
    assert!(listener.is_null());
    assert_eq!(
        kmipkit_server_listener_bind_https(ptr::null_mut(), &raw mut listener),
        ERROR_INVALID_INPUT
    );
    assert!(listener.is_null());

    let mut port = u16::MAX;
    assert_eq!(
        kmipkit_server_listener_port(ptr::null_mut(), &raw mut port),
        ERROR_INVALID_INPUT
    );
    assert_eq!(port, 0);
    let mut connection = ptr::null_mut();
    assert_eq!(
        kmipkit_server_listener_accept(ptr::null_mut(), 0, 0, &raw mut connection),
        ERROR_INVALID_INPUT
    );
    assert!(connection.is_null());
    assert_eq!(
        kmipkit_server_listener_accept(ptr::null_mut(), 0, 1_000_000_000, &raw mut connection),
        ERROR_INVALID_INPUT
    );
    assert!(connection.is_null());
    let mut batch = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_recv_batch(
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            0,
            &raw mut batch
        ),
        ERROR_INVALID_INPUT
    );
    assert!(batch.is_null());

    let mut count = u64::MAX;
    assert_eq!(
        kmipkit_server_incoming_batch_count(ptr::null_mut(), &raw mut count),
        ERROR_INVALID_INPUT
    );
    assert_eq!(count, 0);
    let mut version = crate::kmipkit_protocol_version_v1_t { major: 9, minor: 9 };
    assert_eq!(
        kmipkit_server_incoming_batch_protocol_version(ptr::null_mut(), &raw mut version),
        ERROR_INVALID_INPUT
    );
    assert_eq!((version.major, version.minor), (0, 0));
    let mut request = ptr::null_mut();
    assert_eq!(
        kmipkit_server_incoming_batch_item_at(ptr::null_mut(), 0, &raw mut request),
        ERROR_INVALID_INPUT
    );
    assert!(request.is_null());
}

#[test]
fn request_item_ffi_rejects_missing_handles() {
    let mut operation = u32::MAX;
    assert_eq!(
        kmipkit_server_request_item_operation(ptr::null_mut(), &raw mut operation),
        ERROR_INVALID_INPUT
    );
    assert_eq!(operation, 0);
    let mut put_object = ptr::null_mut();
    assert_eq!(
        kmipkit_server_request_item_put_object(ptr::null_mut(), &raw mut put_object),
        ERROR_INVALID_INPUT
    );
    assert!(put_object.is_null());
    assert_eq!(
        kmipkit_server_put_object_type(ptr::null_mut(), &raw mut operation),
        ERROR_INVALID_INPUT
    );
    assert_eq!(operation, 0);
    assert_eq!(
        kmipkit_server_put_object_tag(ptr::null_mut(), &raw mut operation),
        ERROR_INVALID_INPUT
    );
    assert_eq!(operation, 0);
    let mut identifier = kmipkit_byte_span_v1_t {
        data: std::ptr::dangling(),
        length: 99,
    };
    assert_eq!(
        kmipkit_server_request_item_unique_identifier(ptr::null_mut(), &raw mut identifier),
        ERROR_INVALID_INPUT
    );
    assert!(identifier.data.is_null());
    assert_eq!(identifier.length, 0);
}

#[test]
fn notify_request_ffi_rejects_missing_handles() {
    let mut operation = u32::MAX;
    let mut count = u64::MAX;
    let mut present = u8::MAX;
    let mut attributes = ptr::null_mut();
    // SAFETY: both outputs are writable stack slots and the request is null.
    assert_eq!(
        unsafe {
            kmipkit_server_request_item_notify_attributes(
                ptr::null_mut(),
                &raw mut present,
                &raw mut attributes,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(attributes.is_null());
    assert_eq!(
        kmipkit_server_request_item_notify_deleted_attribute_reference_count(
            ptr::null_mut(),
            &raw mut count
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(count, 0);
    let mut deleted_reference = kmipkit_attribute_reference_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_attribute_reference_v1_t>())
            .expect("reference record fits ABI"),
        abi_version: 1,
        reserved: 0,
        reserved2: 0,
        kind: 0,
        tag_value: 0,
        vendor_identification_data: ptr::null(),
        vendor_identification_length: 0,
        attribute_name_data: ptr::null(),
        attribute_name_length: 0,
    };
    // SAFETY: the output is a valid v1 writable record; the request is null.
    assert_eq!(
        unsafe {
            kmipkit_server_request_item_notify_deleted_attribute_reference_at(
                ptr::null_mut(),
                0,
                &raw mut deleted_reference,
            )
        },
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_request_item_put_function(ptr::null_mut(), &raw mut operation),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_request_item_endpoint_role(ptr::null_mut(), &raw mut operation),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_request_item_discover_versions_count(ptr::null_mut(), &raw mut count),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_request_item_discover_version_at(ptr::null_mut(), 0, ptr::null_mut()),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_request_item_query_functions_count(ptr::null_mut(), &raw mut count),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_request_item_query_function_at(ptr::null_mut(), 0, &raw mut operation),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn response_builder_ffi_rejects_missing_handles() {
    let mut builder = ptr::null_mut();
    assert_eq!(
        kmipkit_server_response_builder_create(ptr::null_mut(), &raw mut builder),
        ERROR_INVALID_INPUT
    );
    assert!(builder.is_null());
    let mut response_item = ptr::null_mut();
    assert_eq!(
        kmipkit_server_notify_response_create(ptr::null_mut(), &raw mut response_item),
        ERROR_INVALID_INPUT
    );
    assert!(response_item.is_null());
    assert_eq!(
        kmipkit_server_put_response_create(ptr::null_mut(), &raw mut response_item),
        ERROR_INVALID_INPUT
    );
    assert!(response_item.is_null());
    assert_eq!(
        kmipkit_server_discover_versions_response_create(
            ptr::null_mut(),
            ptr::null(),
            0,
            &raw mut response_item
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_query_response_create(
            ptr::null_mut(),
            ptr::null(),
            0,
            ptr::null(),
            0,
            ptr::null(),
            0,
            kmipkit_byte_span_v1_t {
                data: ptr::null(),
                length: 0
            },
            &raw mut response_item
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_set_endpoint_role_response_create(
            ptr::null_mut(),
            1,
            kmipkit_byte_span_v1_t {
                data: ptr::null(),
                length: 0
            },
            &raw mut response_item
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_response_builder_add_success(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut()
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_server_response_builder_add_failure(
            ptr::null_mut(),
            ptr::null_mut(),
            1,
            0,
            0,
            kmipkit_byte_span_v1_t {
                data: ptr::null(),
                length: 0
            }
        ),
        ERROR_INVALID_INPUT
    );
    let mut response = ptr::null_mut();
    assert_eq!(
        kmipkit_server_response_builder_build(ptr::null_mut(), &raw mut response),
        ERROR_INVALID_INPUT
    );
    assert!(response.is_null());
    assert_eq!(
        kmipkit_server_connection_send_response(ptr::null_mut(), ptr::null_mut(), 0, 0),
        ERROR_INVALID_INPUT
    );

    kmipkit_server_listener_config_release(ptr::null_mut());
    kmipkit_server_listener_release(ptr::null_mut());
    kmipkit_server_connection_release(ptr::null_mut());
    kmipkit_server_incoming_batch_release(ptr::null_mut());
    kmipkit_server_request_item_release(ptr::null_mut());
    kmipkit_server_put_object_release(ptr::null_mut());
    kmipkit_server_response_builder_release(ptr::null_mut());
    kmipkit_server_response_item_release(ptr::null_mut());
    kmipkit_server_response_release(ptr::null_mut());
}
