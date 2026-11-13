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
fn encode_test_ttlv(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let tag_bytes = tag.to_be_bytes();
    let mut encoded = vec![tag_bytes[1], tag_bytes[2], tag_bytes[3], item_type];
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test TTLV value length fits the protocol field")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    if item_type != 0x01 {
        encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    }
    encoded
}

fn test_integer(tag: u32, value: i32) -> Vec<u8> {
    encode_test_ttlv(tag, 0x02, &value.to_be_bytes())
}

fn test_enumeration(tag: u32, value: u32) -> Vec<u8> {
    encode_test_ttlv(tag, 0x05, &value.to_be_bytes())
}

fn test_structure(tag: u32, children: &[u8]) -> Vec<u8> {
    encode_test_ttlv(tag, 0x01, children)
}

fn discover_versions_request() -> Vec<u8> {
    let version = test_structure(
        0x0042_0069,
        &[test_integer(0x0042_006A, 2), test_integer(0x0042_006B, 1)].concat(),
    );
    let header = test_structure(
        0x0042_0077,
        &[
            version.clone(),
            test_integer(0x0042_004F, 65_536),
            test_integer(0x0042_000D, 1),
        ]
        .concat(),
    );
    let payload = test_structure(0x0042_0079, &version);
    let batch_item = test_structure(
        0x0042_000F,
        &[
            test_enumeration(0x0042_005C, 0x1E),
            encode_test_ttlv(0x0042_0093, 0x08, &[1]),
            payload,
        ]
        .concat(),
    );
    test_structure(0x0042_0078, &[header, batch_item].concat())
}

fn test_failure_response(
    batch: *mut crate::kmipkit_server_incoming_batch_t,
) -> (
    *mut crate::kmipkit_server_request_item_t,
    *mut crate::kmipkit_server_response_builder_t,
    *mut crate::kmipkit_server_response_t,
) {
    let mut request_item = ptr::null_mut();
    assert_eq!(
        kmipkit_server_incoming_batch_item_at(batch, 0, &raw mut request_item),
        SUCCESS
    );
    let mut builder = ptr::null_mut();
    assert_eq!(
        kmipkit_server_response_builder_create(batch, &raw mut builder),
        SUCCESS
    );
    assert_eq!(
        kmipkit_server_response_builder_add_failure(
            builder,
            request_item,
            1,
            1,
            0,
            kmipkit_byte_span_v1_t {
                data: ptr::null(),
                length: 0,
            },
        ),
        SUCCESS
    );
    let mut response = ptr::null_mut();
    assert_eq!(
        kmipkit_server_response_builder_build(builder, &raw mut response),
        SUCCESS
    );
    (request_item, builder, response)
}

fn test_listener_client_tls_config(pki: &EphemeralPki) -> Arc<rustls::ClientConfig> {
    use rustls::RootCertStore;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

    let identity = pki.client_identity();
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("test authority is a valid trust root");
    Arc::new(
        rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("test client enables TLS 1.3 only")
        .with_root_certificates(roots)
        .with_client_auth_cert(
            vec![CertificateDer::from(identity.certificate_der().to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                identity.private_key_der().to_vec(),
            )),
        )
        .expect("test client identity is valid"),
    )
}

fn connect_test_listener_client(
    port: u16,
    config: Arc<rustls::ClientConfig>,
) -> rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream> {
    let socket = std::net::TcpStream::connect(("127.0.0.1", port)).expect("client connects");
    let server_name = rustls::pki_types::ServerName::try_from("server.kmipkit.test")
        .expect("test server name is valid");
    let connection =
        rustls::ClientConnection::new(config, server_name).expect("TLS client is configured");
    rustls::StreamOwned::new(connection, socket)
}

fn read_test_ttlv_response(stream: &mut impl std::io::Read) -> Vec<u8> {
    let mut header = [0_u8; 8];
    stream
        .read_exact(&mut header)
        .expect("client receives the server response");
    assert_eq!(&header[..4], &[0x42, 0x00, 0x7B, 0x01]);
    let body_length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let padded_length = body_length.div_ceil(8) * 8;
    let mut body = vec![0_u8; padded_length];
    stream
        .read_exact(&mut body)
        .expect("client receives the complete response body");
    let mut response = header.to_vec();
    response.extend_from_slice(&body);
    response
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

fn role_reversal_test_listener(pki: &EphemeralPki) -> (*mut crate::kmipkit_server_listener_t, u16) {
    use kmipkit_transport::listener::ListenerTransportConfig;
    use kmipkit_transport::{CertificateInput, ClientIdentity, PrivateKeyInput, TrustSource};

    let server_identity = pki.server_identity();
    let config = ListenerTransportConfig::builder("127.0.0.1", 0)
        .listener_identity(ClientIdentity::new(
            CertificateInput::from_der(
                server_identity
                    .certificate_chain_der()
                    .iter()
                    .map(|certificate| certificate.to_vec())
                    .collect(),
            ),
            PrivateKeyInput::from_der(server_identity.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .build()
        .expect("raw-TLS listener configuration builds");
    let listener =
        kmipkit_client::ClientListener::bind_raw_tls(config).expect("raw-TLS listener binds");
    let port = listener.port().expect("listener has a local port");
    let listener = try_make_handle(
        Kind::ServerListener,
        HandleValue::ServerListener(Box::new(Mutex::new(listener))),
    )
    .expect("listener handle is created");
    (listener, port)
}

fn role_reversal_test_peer(
    port: u16,
    pki: &EphemeralPki,
) -> (std::thread::JoinHandle<()>, std::sync::mpsc::Receiver<()>) {
    use std::io::Write;
    use std::sync::mpsc;
    use std::thread;

    let client_config = test_listener_client_tls_config(pki);
    let request = discover_versions_request();
    let (response_received, response_received_rx) = mpsc::channel();
    let peer = thread::spawn(move || {
        let mut tls_one = connect_test_listener_client(port, Arc::clone(&client_config));
        let mut tls_two = connect_test_listener_client(port, client_config);
        tls_one
            .write_all(&request)
            .expect("first client writes the request");
        tls_one.flush().expect("first client flushes the request");
        tls_two
            .write_all(&request)
            .expect("second client writes the request");
        tls_two.flush().expect("second client flushes the request");

        for tls in [&mut tls_one, &mut tls_two] {
            let _response = read_test_ttlv_response(tls);
            response_received
                .send(())
                .expect("server test is still waiting for both responses");
        }
    });
    (peer, response_received_rx)
}

fn accept_role_reversal_test_batches(
    listener: *mut crate::kmipkit_server_listener_t,
) -> (
    *mut crate::kmipkit_server_connection_t,
    *mut crate::kmipkit_server_connection_t,
    *mut crate::kmipkit_codec_limits_t,
    *mut crate::kmipkit_server_incoming_batch_t,
    *mut crate::kmipkit_server_incoming_batch_t,
) {
    let mut connection = ptr::null_mut();
    assert_eq!(
        kmipkit_server_listener_accept(listener, 5, 0, &raw mut connection),
        SUCCESS
    );
    let mut other_connection = ptr::null_mut();
    assert_eq!(
        kmipkit_server_listener_accept(listener, 5, 0, &raw mut other_connection),
        SUCCESS
    );
    let limits = try_make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
    )
    .expect("codec limits handle is created");
    let mut batch = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_recv_batch(connection, limits, 5, 0, &raw mut batch),
        SUCCESS
    );
    let mut other_batch = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_recv_batch(other_connection, limits, 5, 0, &raw mut other_batch),
        SUCCESS
    );
    (connection, other_connection, limits, batch, other_batch)
}

fn assert_role_reversal_consumes_only_matching_responses(
    connection: *mut crate::kmipkit_server_connection_t,
    other_connection: *mut crate::kmipkit_server_connection_t,
    response: *mut crate::kmipkit_server_response_t,
    other_response: *mut crate::kmipkit_server_response_t,
) -> *mut crate::kmipkit_server_role_reversed_session_t {
    let mut session = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_send_response(connection, other_response, 5, 0),
        ERROR_INVALID_INPUT,
        "a response for one TLS connection cannot consume another connection's request batch"
    );
    assert_eq!(
        kmipkit_server_connection_send_response(connection, response, 5, 0),
        SUCCESS
    );
    assert_eq!(
        kmipkit_server_connection_send_response(connection, response, 5, 0),
        ERROR_INVALID_INPUT,
        "a response cannot consume a batch more than once"
    );
    assert_eq!(
        kmipkit_server_connection_into_role_reversed_session(connection, &raw mut session),
        SUCCESS
    );
    assert_eq!(
        kmipkit_server_connection_send_response(other_connection, other_response, 5, 0),
        SUCCESS
    );
    assert!(!session.is_null());

    let mut duplicate_session = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_into_role_reversed_session(
            connection,
            &raw mut duplicate_session,
        ),
        ERROR_INVALID_INPUT,
        "the connection is consumed by successful role reversal"
    );
    assert!(duplicate_session.is_null());
    session
}

fn assert_role_reversed_session_idle_timeout_and_poisoning(
    session: *mut crate::kmipkit_server_role_reversed_session_t,
    limits: *mut crate::kmipkit_codec_limits_t,
) {
    let mut next_batch = ptr::null_mut();
    assert_eq!(
        kmipkit_server_role_reversed_session_recv_batch(
            session,
            limits,
            0,
            10_000_000,
            &raw mut next_batch,
        ),
        ERROR_TRANSPORT,
        "an idle role-reversed session reports its bounded receive timeout"
    );
    assert!(next_batch.is_null());

    let session_handle = reference_handle(session, Kind::ServerRoleReversedSession)
        .expect("role-reversed session handle remains registered");
    let HandleValue::ServerRoleReversedSession(session_state) = &session_handle.value else {
        panic!("session handle stores the role-reversed session state");
    };
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = session_state
            .lock()
            .expect("session state is healthy before poisoning");
        panic!("poison session state for the FFI error-path test");
    }));
    assert!(panic.is_err());
    assert_eq!(
        kmipkit_server_role_reversed_session_recv_batch(
            session,
            limits,
            0,
            1_000_000,
            &raw mut next_batch,
        ),
        ERROR_INTERNAL
    );
    assert!(next_batch.is_null());
}

#[test]
fn ffi_role_reversal_rejects_active_batch_and_handles_idle_timeout() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI generation succeeds");
    let (listener, port) = role_reversal_test_listener(&pki);
    let (peer, response_received_rx) = role_reversal_test_peer(port, &pki);
    let (connection, other_connection, limits, batch, other_batch) =
        accept_role_reversal_test_batches(listener);

    let mut session = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_into_role_reversed_session(connection, &raw mut session),
        ERROR_INVALID_INPUT,
        "role reversal cannot consume a connection while its request batch is active"
    );
    assert!(session.is_null());

    let (request_item, response_builder, response) = test_failure_response(batch);
    let (other_request_item, other_response_builder, other_response) =
        test_failure_response(other_batch);
    session = assert_role_reversal_consumes_only_matching_responses(
        connection,
        other_connection,
        response,
        other_response,
    );

    for _ in 0..2 {
        response_received_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("the TLS peer received both responses");
    }
    assert_role_reversed_session_idle_timeout_and_poisoning(session, limits);

    kmipkit_server_role_reversed_session_release(session);
    kmipkit_server_response_release(response);
    kmipkit_server_response_release(other_response);
    kmipkit_server_response_builder_release(response_builder);
    kmipkit_server_response_builder_release(other_response_builder);
    kmipkit_server_request_item_release(request_item);
    kmipkit_server_request_item_release(other_request_item);
    kmipkit_server_incoming_batch_release(batch);
    kmipkit_server_incoming_batch_release(other_batch);
    kmipkit_server_connection_release(connection);
    kmipkit_server_connection_release(other_connection);
    kmipkit_codec_limits_release(limits);
    kmipkit_server_listener_release(listener);
    peer.join().expect("TLS peer finishes");
}

#[test]
fn ffi_role_reversal_reports_a_poisoned_connection_handle() {
    let state = Mutex::new(ServerConnectionState {
        connection: None,
        active_batch: None,
    });
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = state.lock().expect("test mutex is initially usable");
        panic!("poison connection state for the FFI error-path test");
    }));
    assert!(panic.is_err());

    let connection = try_make_handle(
        Kind::ServerConnection,
        HandleValue::ServerConnection(Box::new(state)),
    )
    .expect("poisoned connection handle is registered");
    let mut session = ptr::null_mut();
    assert_eq!(
        kmipkit_server_connection_into_role_reversed_session(connection, &raw mut session),
        ERROR_INTERNAL
    );
    assert!(session.is_null());
    kmipkit_server_connection_release(connection);
}
