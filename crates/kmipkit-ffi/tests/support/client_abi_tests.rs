use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use base64::Engine;
use kmipkit_client::{ClientBatchResponse, ClientError, ClientErrorCategory};
use kmipkit_protocol::{AttributeSet, LocateRequest};
use kmipkit_test_support::EphemeralPki;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestDeliveryState,
    RequestOptions, TimeoutLimit, TimeoutPolicy, TransportConfig, TrustSource,
};
use rcgen::{
    BasicConstraints, CertificateParams, CertificateRevocationListParams, IsCa, Issuer,
    KeyIdMethod, KeyPair, KeyUsagePurpose, SerialNumber, date_time_ymd,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

use super::*;
use crate::kmipkit_byte_span_v1_t;

const INPUT_MEMORY: u32 = 1;
const INPUT_FILE: u32 = 2;
const ENCODING_PEM: u32 = 1;
const ENCODING_DER: u32 = 2;
const MAX_TEST_TTLV_BODY_BYTES: usize = 1_048_576;
static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn byte_span(bytes: &[u8]) -> kmipkit_byte_span_v1_t {
    kmipkit_byte_span_v1_t {
        data: if bytes.is_empty() {
            ptr::null()
        } else {
            bytes.as_ptr()
        },
        length: u64::try_from(bytes.len()).expect("test span length fits the ABI"),
    }
}

fn input(source_kind: u32, encoding: u32, bytes: &[u8]) -> kmipkit_transport_input_v1_t {
    kmipkit_transport_input_v1_t {
        source_kind,
        encoding,
        data: byte_span(bytes),
    }
}

fn timeout(kind: u32, seconds: u64, nanoseconds: u32) -> kmipkit_timeout_limit_v1_t {
    kmipkit_timeout_limit_v1_t {
        kind,
        reserved: 0,
        seconds,
        nanoseconds,
        reserved2: 0,
    }
}

fn request_options(
    connect: kmipkit_timeout_limit_v1_t,
    write: kmipkit_timeout_limit_v1_t,
    read: kmipkit_timeout_limit_v1_t,
    total: kmipkit_timeout_limit_v1_t,
) -> kmipkit_request_options_v1_t {
    kmipkit_request_options_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_request_options_v1_t>())
            .expect("request options fit the ABI size field"),
        abi_version: 1,
        connect_timeout: connect,
        write_timeout: write,
        read_timeout: read,
        total_timeout: total,
    }
}

#[allow(clippy::too_many_arguments)]
fn configuration(
    endpoint_kind: u32,
    trust_kind: u32,
    flags: u32,
    endpoint: &[u8],
    endpoint_port: u16,
    client_certificates: &[kmipkit_transport_input_v1_t],
    client_private_key: kmipkit_transport_input_v1_t,
    trust_certificates: &[kmipkit_transport_input_v1_t],
    revocation_lists: &[kmipkit_transport_input_v1_t],
    tls_server_name: &[u8],
    target_uri: &[u8],
    connect_timeout: kmipkit_timeout_limit_v1_t,
    write_timeout: kmipkit_timeout_limit_v1_t,
    read_timeout: kmipkit_timeout_limit_v1_t,
    total_timeout: kmipkit_timeout_limit_v1_t,
) -> kmipkit_transport_config_v1_t {
    kmipkit_transport_config_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_transport_config_v1_t>())
            .expect("test configuration fits the ABI size field"),
        abi_version: 1,
        endpoint_kind,
        trust_kind,
        flags,
        reserved: 0,
        endpoint: byte_span(endpoint),
        endpoint_port,
        reserved2: 0,
        client_certificates: if client_certificates.is_empty() {
            ptr::null()
        } else {
            client_certificates.as_ptr()
        },
        client_certificate_count: u64::try_from(client_certificates.len())
            .expect("test certificate count fits the ABI"),
        client_private_key,
        trust_certificates: if trust_certificates.is_empty() {
            ptr::null()
        } else {
            trust_certificates.as_ptr()
        },
        trust_certificate_count: u64::try_from(trust_certificates.len())
            .expect("test trust count fits the ABI"),
        revocation_lists: if revocation_lists.is_empty() {
            ptr::null()
        } else {
            revocation_lists.as_ptr()
        },
        revocation_list_count: u64::try_from(revocation_lists.len())
            .expect("test CRL count fits the ABI"),
        tls_server_name: byte_span(tls_server_name),
        target_uri: byte_span(target_uri),
        connect_timeout,
        write_timeout,
        read_timeout,
        total_timeout,
        max_request_bytes: 1_048_576,
    }
}

fn pem(label: &str, der: &[u8]) -> Vec<u8> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(der);
    let mut output = format!("-----BEGIN {label}-----\n");
    for line in encoded.as_bytes().chunks(64) {
        output.push_str(std::str::from_utf8(line).expect("base64 is ASCII"));
        output.push('\n');
    }
    output.push_str("-----END ");
    output.push_str(label);
    output.push_str("-----\n");
    output.into_bytes()
}

fn crl_der() -> Vec<u8> {
    let mut issuer_params = CertificateParams::new(vec!["crl.kmipkit.test".to_owned()])
        .expect("test CRL issuer name is valid");
    issuer_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    issuer_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    let issuer_key = KeyPair::generate().expect("test CRL issuer key generation succeeds");
    let issuer = Issuer::from_params(&issuer_params, &issuer_key);
    let crl = CertificateRevocationListParams {
        this_update: date_time_ymd(2020, 1, 1),
        next_update: date_time_ymd(2099, 1, 1),
        crl_number: SerialNumber::from(1_u64),
        issuing_distribution_point: None,
        revoked_certs: Vec::new(),
        key_identifier_method: KeyIdMethod::Sha256,
    }
    .signed_by(&issuer)
    .expect("test CRL signing succeeds");
    crl.der().as_ref().to_vec()
}

struct TemporaryFiles {
    directory: PathBuf,
    files: Vec<PathBuf>,
}

impl TemporaryFiles {
    fn new() -> Self {
        let directory =
            create_private_test_directory().expect("private test directory is created exclusively");
        Self {
            directory,
            files: Vec::new(),
        }
    }

    fn write(&mut self, label: &str, contents: &[u8]) -> PathBuf {
        let path = self.directory.join(format!("{label}.tmp"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("test-only input file is created exclusively");
        file.write_all(contents)
            .expect("test-only input file is written");
        self.files.push(path.clone());
        path
    }
}

impl Drop for TemporaryFiles {
    fn drop(&mut self) {
        for path in &self.files {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&self.directory);
    }
}

fn create_private_test_directory() -> std::io::Result<PathBuf> {
    #[cfg(windows)]
    let parent = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("LOCALAPPDATA is required for private test files"))?;
    #[cfg(not(windows))]
    let parent = std::env::temp_dir();

    for _ in 0..128 {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = parent.join(format!(
            "kmipkit-ffi-client-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder
        };
        #[cfg(not(unix))]
        let builder = fs::DirBuilder::new();
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not create a unique private test directory",
    ))
}

fn path_bytes(path: &Path) -> Vec<u8> {
    path.to_str()
        .expect("temporary path is UTF-8")
        .as_bytes()
        .to_vec()
}

fn pending_locate_response(unique_id: &[u8], correlation: &[u8]) -> Vec<u8> {
    let protocol_version = ttlv_structure(
        0x0042_0069,
        [ttlv_integer(0x0042_006A, 2), ttlv_integer(0x0042_006B, 1)],
    );
    let header = ttlv_structure(
        0x0042_007A,
        [
            protocol_version,
            ttlv_datetime(0x0042_0092, 1),
            ttlv_integer(0x0042_000D, 1),
        ],
    );
    let item = ttlv_structure(
        0x0042_000F,
        [
            ttlv_enumeration(0x0042_005C, 8),
            ttlv_item(0x0042_0093, 0x08, unique_id),
            ttlv_enumeration(0x0042_007F, 2),
            ttlv_item(0x0042_0006, 0x08, correlation),
            ttlv_structure(0x0042_007C, []),
        ],
    );
    ttlv_structure(0x0042_007B, [header, item])
}

fn ttlv_structure<const N: usize>(tag: u32, children: [Vec<u8>; N]) -> Vec<u8> {
    ttlv_item(
        tag,
        0x01,
        &children.into_iter().flatten().collect::<Vec<_>>(),
    )
}

fn ttlv_integer(tag: u32, value: i32) -> Vec<u8> {
    ttlv_item(tag, 0x02, &value.to_be_bytes())
}

fn ttlv_enumeration(tag: u32, value: u32) -> Vec<u8> {
    ttlv_item(tag, 0x05, &value.to_be_bytes())
}

fn ttlv_datetime(tag: u32, value: i64) -> Vec<u8> {
    ttlv_item(tag, 0x09, &value.to_be_bytes())
}

fn ttlv_item(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    encoded.extend_from_slice(&tag.to_be_bytes()[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test TTLV item length fits its wire field")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    encoded
}

fn tls13_mtls_server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the local test authority is valid");
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("the local server requires the client's test certificate");
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local peer only enables TLS 1.3");
    let identity = pki.server_identity();
    let certificates = identity
        .certificate_chain_der()
        .into_iter()
        .map(|certificate| CertificateDer::from(certificate.to_vec()))
        .collect();
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        identity.private_key_der().to_vec(),
    ));
    Arc::new(
        builder
            .with_client_cert_verifier(verifier)
            .with_single_cert(certificates, private_key)
            .expect("the server test identity is valid"),
    )
}

fn receive_ttlv_frame(stream: &mut impl Read) -> io::Result<()> {
    let mut header = [0_u8; 8];
    stream.read_exact(&mut header)?;
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if length > MAX_TEST_TTLV_BODY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test peer refused an oversized TTLV body",
        ));
    }
    let padded_length = length
        .checked_add(7)
        .map(|padded| padded / 8 * 8)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "TTLV padding overflow"))?;
    let mut body = vec![0; padded_length];
    stream.read_exact(&mut body)?;
    Ok(())
}

#[test]
fn test_peer_rejects_oversized_request_before_body_allocation() {
    let mut stream = std::io::Cursor::new([0_u8, 0, 0, 0, 0xff, 0xff, 0xff, 0xff]);
    let error = receive_ttlv_frame(&mut stream).expect_err("oversized TTLV body is rejected");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(stream.position(), 8, "the body was not read");
}

fn pending_locate_result(async_requested: bool) -> Result<ClientBatchResponse, ClientError> {
    // KMIP 2.1 §§6.1.28, 8.6, 9.1, 9.2 and 11.3: Locate permits the caller to
    // request asynchronous behavior, and Pending carries its opaque correlation value.
    let pki = EphemeralPki::generate().expect("ephemeral test PKI generation succeeds");
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback binds");
    let port = listener
        .local_addr()
        .expect("loopback has an address")
        .port();
    let expected_id = [0, 0x80, 0xff, 0x13];
    let expected_correlation = [0, 0xff, 0x80, 0x7f, 0];
    let response = pending_locate_response(&expected_id, &expected_correlation);
    let server_config = tls13_mtls_server_config(&pki);
    let peer = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client connects");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("peer read timeout is set");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("peer write timeout is set");
        let connection = ServerConnection::new(server_config).expect("TLS 1.3 connection builds");
        let mut tls = StreamOwned::new(connection, stream);
        receive_ttlv_frame(&mut tls).expect("the typed Locate request frame is bounded");
        tls.write_all(&response)
            .expect("Pending response is written");
        tls.flush().expect("Pending response is flushed");
    });

    let identity = pki.client_identity();
    let certificates = identity
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", port))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(certificates),
            PrivateKeyInput::from_der(identity.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .tls_server_name("server.kmipkit.test")
        .timeouts(TimeoutPolicy::default())
        .build()
        .expect("the validated local transport configuration builds");
    let registry = client_extension::client_extension_registry(Vec::new(), extension::defaults())
        .expect("the empty approved extension registry is valid");
    let configuration = client_extension::ClientConfiguration::new(registry);
    let mut client =
        Client::new(configuration, transport).expect("production Client construction succeeds");
    let mut batch = ClientBatch::from_items([ClientBatchItem::new(
        kmipkit_client::ClientRequest::locate(LocateRequest::new(AttributeSet::new())),
    )
    .with_unique_batch_item_id(expected_id.to_vec())]);
    if async_requested {
        batch = batch.with_asynchronous_indicator(2);
    }
    let response = client.execute(batch, &CodecLimits::defaults());
    peer.join().expect("controlled local peer completes");
    response
}

#[test]
fn ffi_pending_metadata_preserves_exact_binary_identifier_and_correlation() {
    // KMIP 2.1 §§6.1.28, 8.6, 9.1, 9.2 and 11.3: Locate permits the caller to
    // request asynchronous behavior, and Pending carries its opaque correlation value.
    let response = pending_locate_result(true)
        .expect("Locate Pending is accepted after the caller permits async results");
    let expected_id = [0, 0x80, 0xff, 0x13];
    let expected_correlation = [0, 0xff, 0x80, 0x7f, 0];
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item_handle = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &mut item_handle),
        SUCCESS
    );
    let mut metadata = kmipkit_client_batch_item_metadata_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_client_batch_item_metadata_v1_t>())
            .expect("metadata size fits ABI field"),
        abi_version: 1,
        outcome_kind: 0,
        result_status: 0,
        has_result_reason: 0,
        result_reason: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_metadata_v1(item_handle, &mut metadata),
        SUCCESS
    );
    assert_eq!(metadata.outcome_kind, BATCH_OUTCOME_PENDING);
    assert_eq!(metadata.result_status, 2);
    let mut present = 0;
    let mut span = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_unique_batch_item_id(
            item_handle,
            &mut present,
            &mut span,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    // SAFETY: the response-item handle owns the Arc-backed response and remains alive.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                span.data,
                usize::try_from(span.length).expect("response span fits this test target"),
            )
        },
        expected_id
    );
    assert_eq!(
        kmipkit_client_batch_item_response_pending_correlation_value(
            item_handle,
            &mut present,
            &mut span,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    // SAFETY: the response-item handle owns the Arc-backed response and remains alive.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                span.data,
                usize::try_from(span.length).expect("response span fits this test target"),
            )
        },
        expected_correlation
    );
    kmipkit_client_batch_item_response_release(item_handle);
    kmipkit_client_batch_response_release(response_handle);
}

#[test]
fn ffi_pending_without_async_permission_reports_protocol_and_response_started() {
    let error = pending_locate_result(false)
        .expect_err("Pending is rejected when the caller did not permit async results");
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(record_client_error(&error), ERROR_PROTOCOL);

    let mut status = 0;
    let mut category = 0;
    let mut cause = 0;
    let mut delivery = 0;
    assert_eq!(
        kmipkit_last_error_get(&mut status, &mut category, &mut cause, &mut delivery),
        SUCCESS
    );
    assert_eq!(status, ERROR_PROTOCOL);
    assert_eq!(category, ERROR_CATEGORY_PROTOCOL);
    assert_eq!(delivery, DELIVERY_STATE_RESPONSE_STARTED);
}

#[test]
fn ffi_last_error_snapshots_are_isolated_between_threads() {
    let worker = thread::spawn(|| {
        assert_eq!(
            kmipkit_client_batch_create(ptr::null_mut()),
            ERROR_INVALID_INPUT
        );
        let mut status = 0;
        let mut category = 0;
        let mut cause = 0;
        let mut delivery = 0;
        assert_eq!(
            kmipkit_last_error_get(&mut status, &mut category, &mut cause, &mut delivery),
            SUCCESS
        );
        assert_eq!(status, ERROR_INVALID_INPUT);
        assert_eq!(category, ERROR_CATEGORY_VALIDATION);
        assert_eq!(cause, ERROR_CAUSE_INVALID_INPUT);
        assert_eq!(delivery, DELIVERY_STATE_NONE);
    });
    worker.join().expect("FFI error snapshot worker completes");

    let mut status = 0;
    let mut category = 0;
    let mut cause = 0;
    let mut delivery = 0;
    assert_eq!(
        kmipkit_last_error_get(&mut status, &mut category, &mut cause, &mut delivery),
        SUCCESS
    );
    assert_eq!(status, SUCCESS);
    assert_eq!(category, ERROR_CATEGORY_NONE);
    assert_eq!(cause, ERROR_CAUSE_NONE);
    assert_eq!(delivery, DELIVERY_STATE_NONE);
}

#[test]
fn ffi_constructs_a_client_from_validated_https_configuration() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let identity = pki.client_identity();
    let chain = identity
        .certificate_chain_der()
        .into_iter()
        .map(|certificate| input(INPUT_MEMORY, ENCODING_DER, certificate))
        .collect::<Vec<_>>();
    let key = input(INPUT_MEMORY, ENCODING_DER, identity.private_key_der());
    let roots = [input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.authority_certificate_der(),
    )];
    let config = configuration(
        TRANSPORT_ENDPOINT_HTTPS,
        TRANSPORT_TRUST_CERTIFICATES,
        TRANSPORT_FLAG_TARGET_URI,
        b"https://server.kmipkit.test:8443",
        0,
        &chain,
        key,
        &roots,
        &[],
        &[],
        b"/kmip",
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );

    let mut registry_limits = kmipkit_extension_registry_limits_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_extension_registry_limits_v1_t>())
            .expect("limits struct size fits ABI field"),
        abi_version: 1,
        max_definitions: 0,
        max_schema_nodes: 0,
        max_child_rules_per_structure: 0,
        max_text_bytes_per_field: 0,
        max_registry_text_bytes: 0,
        max_discriminator_scalar_bytes: 0,
        max_total_discriminator_scalar_bytes: 0,
        max_constraint_members_per_rule: 0,
        max_total_constraint_members: 0,
        max_payload_index_records: 0,
        max_lookup_comparisons: 0,
        max_depth: 0,
    };
    assert_eq!(
        kmipkit_extension_registry_limits_v1_default_values(&mut registry_limits),
        SUCCESS
    );
    let mut registry = ptr::null_mut();
    assert_eq!(
        kmipkit_client_extension_registry_create_v1(
            ptr::null_mut(),
            0,
            &registry_limits,
            &mut registry,
        ),
        SUCCESS
    );
    let mut client_configuration = ptr::null_mut();
    assert_eq!(
        kmipkit_client_configuration_create(registry, &mut client_configuration),
        SUCCESS
    );
    let mut transport_configuration = ptr::null_mut();
    assert_eq!(
        kmipkit_transport_config_create_v1(&config, &mut transport_configuration),
        SUCCESS
    );
    let mut client = ptr::null_mut();
    assert_eq!(
        kmipkit_client_create(
            client_configuration,
            &mut transport_configuration,
            &mut client,
        ),
        SUCCESS
    );
    assert!(transport_configuration.is_null());
    assert!(!client.is_null());

    kmipkit_client_release(client);
    kmipkit_client_configuration_release(client_configuration);
}

#[test]
fn validated_raw_tls_config_accepts_memory_der_crl_and_every_timeout_kind() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let chain = pki.client_identity().certificate_chain_der();
    let certificates = [
        input(INPUT_MEMORY, ENCODING_DER, chain[0]),
        input(INPUT_MEMORY, ENCODING_DER, chain[1]),
    ];
    let key = input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.client_identity().private_key_der(),
    );
    let roots = [input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.authority_certificate_der(),
    )];
    let crl = crl_der();
    let crls = [input(INPUT_MEMORY, ENCODING_DER, &crl)];
    let endpoint = b"127.0.0.1";
    let server_name = b"server.kmipkit.test";
    let config = configuration(
        TRANSPORT_ENDPOINT_RAW_TLS,
        TRANSPORT_TRUST_CERTIFICATES,
        TRANSPORT_FLAG_TLS_SERVER_NAME,
        endpoint,
        5_694,
        &certificates,
        key,
        &roots,
        &crls,
        server_name,
        &[],
        timeout(TRANSPORT_TIMEOUT_BOUNDED, 3, 250_000_000),
        timeout(TRANSPORT_TIMEOUT_UNBOUNDED, 0, 0),
        timeout(0, 0, 0),
        timeout(TRANSPORT_TIMEOUT_BOUNDED, 10, 0),
    );

    let parsed = transport_config_from_v1(&config).expect("raw-TLS config validates");
    assert_eq!(parsed.max_request_bytes(), 1_048_576);
    assert_eq!(
        parsed.timeouts().connect(),
        TimeoutLimit::Bounded(std::time::Duration::new(3, 250_000_000))
    );
    assert_eq!(parsed.timeouts().write(), TimeoutLimit::Unbounded);
    assert_eq!(parsed.timeouts().read(), TimeoutPolicy::default().read());
    assert_eq!(
        parsed.timeouts().total(),
        TimeoutLimit::Bounded(std::time::Duration::from_secs(10))
    );
}

#[test]
fn request_options_v1_inherits_defaults_and_preserves_zero_and_unbounded_overrides() {
    let input = request_options(
        timeout(0, 0, 0),
        timeout(TRANSPORT_TIMEOUT_BOUNDED, 0, 0),
        timeout(TRANSPORT_TIMEOUT_UNBOUNDED, 0, 0),
        timeout(TRANSPORT_TIMEOUT_BOUNDED, 2, 125_000_000),
    );

    let parsed = request_options_from_v1(&input).expect("request options validate");
    let expected = RequestOptions::default()
        .with_write(TimeoutLimit::Bounded(std::time::Duration::ZERO))
        .with_read(TimeoutLimit::Unbounded)
        .with_total(TimeoutLimit::Bounded(std::time::Duration::new(2, 125_000_000)));
    assert_eq!(parsed, expected);

    let effective = TimeoutPolicy::default().with_overrides(&parsed);
    assert_eq!(effective.connect(), TimeoutPolicy::default().connect());
    assert_eq!(effective.write(), TimeoutLimit::Bounded(std::time::Duration::ZERO));
    assert_eq!(effective.read(), TimeoutLimit::Unbounded);
    assert_eq!(
        effective.total(),
        TimeoutLimit::Bounded(std::time::Duration::new(2, 125_000_000))
    );
}

#[test]
fn request_options_v1_rejects_bad_versions_and_timeout_encodings() {
    let mut input = request_options(
        timeout(0, 0, 0),
        timeout(0, 1, 0),
        timeout(TRANSPORT_TIMEOUT_BOUNDED, 0, 1_000_000_000),
        timeout(TRANSPORT_TIMEOUT_UNBOUNDED, 1, 0),
    );
    assert_eq!(request_options_from_v1(&input).err(), Some(ERROR_INVALID_INPUT));

    input.write_timeout = timeout(0, 0, 0);
    input.read_timeout = timeout(0, 0, 0);
    input.total_timeout = timeout(0, 0, 0);
    input.abi_version = 2;
    assert_eq!(request_options_from_v1(&input).err(), Some(ERROR_INVALID_INPUT));
}

#[test]
fn validated_https_config_accepts_memory_pem_inputs_and_origin_target() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let chain = pki.client_identity().certificate_chain_der();
    let mut chain_pem = pem("CERTIFICATE", chain[0]);
    chain_pem.extend_from_slice(&pem("CERTIFICATE", chain[1]));
    let certificates = [input(INPUT_MEMORY, ENCODING_PEM, &chain_pem)];
    let key_pem = pem("PRIVATE KEY", pki.client_identity().private_key_der());
    let key = input(INPUT_MEMORY, ENCODING_PEM, &key_pem);
    let root_pem = pem("CERTIFICATE", pki.authority_certificate_der());
    let roots = [input(INPUT_MEMORY, ENCODING_PEM, &root_pem)];
    let crl_pem = pem("X509 CRL", &crl_der());
    let crls = [input(INPUT_MEMORY, ENCODING_PEM, &crl_pem)];
    let mut config = configuration(
        TRANSPORT_ENDPOINT_HTTPS,
        TRANSPORT_TRUST_CERTIFICATES,
        TRANSPORT_FLAG_TARGET_URI,
        b"https://server.kmipkit.test:8443",
        0,
        &certificates,
        key,
        &roots,
        &crls,
        &[],
        b"/kms/kmip",
        timeout(0, 0, 0),
        timeout(TRANSPORT_TIMEOUT_BOUNDED, 0, 0),
        timeout(TRANSPORT_TIMEOUT_UNBOUNDED, 0, 0),
        timeout(0, 0, 0),
    );

    let parsed = transport_config_from_v1(&config).expect("HTTPS config validates");
    assert_eq!(parsed.target_uri(), Some("/kms/kmip"));
    assert_eq!(
        parsed.timeouts().write(),
        TimeoutLimit::Bounded(std::time::Duration::ZERO)
    );
    assert_eq!(parsed.timeouts().read(), TimeoutLimit::Unbounded);

    // The URI owns the HTTPS authority and explicit port. The separate C
    // endpoint_port field is reserved for RAW_TLS and must remain zero here.
    config.endpoint_port = 8_443;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
}

#[test]
fn validated_file_pem_config_and_platform_trust_are_supported() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let chain = pki.client_identity().certificate_chain_der();
    let mut chain_pem = pem("CERTIFICATE", chain[0]);
    chain_pem.extend_from_slice(&pem("CERTIFICATE", chain[1]));
    let key_pem = pem("PRIVATE KEY", pki.client_identity().private_key_der());
    let mut files = TemporaryFiles::new();
    let chain_path = files.write("chain", &chain_pem);
    let key_path = files.write("key", &key_pem);
    let chain_path = path_bytes(&chain_path);
    let key_path = path_bytes(&key_path);
    let certificates = [input(INPUT_FILE, ENCODING_PEM, &chain_path)];
    let key = input(INPUT_FILE, ENCODING_PEM, &key_path);
    let config = configuration(
        TRANSPORT_ENDPOINT_RAW_TLS,
        TRANSPORT_TRUST_PLATFORM,
        0,
        b"server.kmipkit.test",
        5_694,
        &certificates,
        key,
        &[],
        &[],
        &[],
        &[],
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );

    let parsed = transport_config_from_v1(&config)
        .expect("PEM file inputs with all DEFAULT timeout selectors validate");
    let defaults = TimeoutPolicy::default();
    assert_eq!(parsed.timeouts().connect(), defaults.connect());
    assert_eq!(parsed.timeouts().write(), defaults.write());
    assert_eq!(parsed.timeouts().read(), defaults.read());
    assert_eq!(parsed.timeouts().total(), defaults.total());

    let first_certificate_path = files.write("certificate-der-1", chain[0]);
    let second_certificate_path = files.write("certificate-der-2", chain[1]);
    let private_key_path = files.write("key-der", pki.client_identity().private_key_der());
    let first_certificate_path = path_bytes(&first_certificate_path);
    let second_certificate_path = path_bytes(&second_certificate_path);
    let private_key_path = path_bytes(&private_key_path);
    let der_certificates = [
        input(INPUT_FILE, ENCODING_DER, &first_certificate_path),
        input(INPUT_FILE, ENCODING_DER, &second_certificate_path),
    ];
    let der_key = input(INPUT_FILE, ENCODING_DER, &private_key_path);
    let der_config = configuration(
        TRANSPORT_ENDPOINT_RAW_TLS,
        TRANSPORT_TRUST_PLATFORM,
        0,
        b"server.kmipkit.test",
        5_694,
        &der_certificates,
        der_key,
        &[],
        &[],
        &[],
        &[],
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );
    assert!(
        transport_config_from_v1(&der_config).is_ok(),
        "file DER certificate and key inputs are supported"
    );
}

#[test]
fn file_based_revocation_lists_are_rejected_before_reading() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let chain = pki.client_identity().certificate_chain_der();
    let certificates = [
        input(INPUT_MEMORY, ENCODING_DER, chain[0]),
        input(INPUT_MEMORY, ENCODING_DER, chain[1]),
    ];
    let key = input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.client_identity().private_key_der(),
    );
    let roots = [input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.authority_certificate_der(),
    )];
    let revocation_lists = [input(INPUT_FILE, ENCODING_DER, b"unused")];
    let config = configuration(
        TRANSPORT_ENDPOINT_RAW_TLS,
        TRANSPORT_TRUST_CERTIFICATES,
        0,
        b"server.kmipkit.test",
        5_694,
        &certificates,
        key,
        &roots,
        &revocation_lists,
        &[],
        &[],
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );

    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
}

#[test]
fn versioned_transport_config_rejects_invalid_headers_limits_and_timeout_fields() {
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let chain = pki.client_identity().certificate_chain_der();
    let certificates = [
        input(INPUT_MEMORY, ENCODING_DER, chain[0]),
        input(INPUT_MEMORY, ENCODING_DER, chain[1]),
    ];
    let key = input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.client_identity().private_key_der(),
    );
    let roots = [input(
        INPUT_MEMORY,
        ENCODING_DER,
        pki.authority_certificate_der(),
    )];
    let mut config = configuration(
        TRANSPORT_ENDPOINT_RAW_TLS,
        TRANSPORT_TRUST_CERTIFICATES,
        0,
        b"127.0.0.1",
        5_694,
        &certificates,
        key,
        &roots,
        &[],
        &[],
        &[],
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );

    config.abi_version = 2;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
    config.abi_version = 1;

    config.struct_size -= 1;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
    config.struct_size += 1;

    config.reserved = 1;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
    config.reserved = 0;

    config.flags = u32::MAX;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
    config.flags = 0;

    config.connect_timeout = timeout(TRANSPORT_TIMEOUT_BOUNDED, 0, 1_000_000_000);
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
    config.connect_timeout = timeout(0, 0, 0);

    config.client_certificate_count = MAX_TRANSPORT_CONFIG_ITEMS + 1;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_RESOURCE_LIMIT)
    );
    config.client_certificate_count =
        u64::try_from(certificates.len()).expect("test certificate count fits the ABI");

    config.endpoint.data = ptr::null();
    config.endpoint.length = MAX_TRANSPORT_TEXT_BYTES + 1;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_RESOURCE_LIMIT)
    );
    config.endpoint = byte_span(b"127.0.0.1");

    config.max_request_bytes = 0;
    assert_eq!(
        transport_config_from_v1(&config).err(),
        Some(ERROR_INVALID_INPUT)
    );
}

#[test]
fn transport_material_accepts_exact_local_bound_and_rejects_over_bound() {
    let bound =
        usize::try_from(MAX_TRANSPORT_MATERIAL_BYTES).expect("the test material bound fits usize");
    let bytes = vec![0x5a; bound + 1];
    let mut memory_input = input(INPUT_MEMORY, ENCODING_DER, &bytes[..bound]);
    let exact = transport_input_bytes(&memory_input).expect("exact memory bound is accepted");
    assert_eq!(exact.len(), bound);
    assert_eq!(
        validate_transport_input_total(std::slice::from_ref(&memory_input)),
        Ok(())
    );
    memory_input.data.length += 1;
    assert_eq!(
        transport_input_bytes(&memory_input).err(),
        Some(ERROR_RESOURCE_LIMIT)
    );
    assert_eq!(
        validate_transport_input_total(std::slice::from_ref(&memory_input)),
        Err(ERROR_RESOURCE_LIMIT)
    );

    let mut files = TemporaryFiles::new();
    let path = files.write("material", &bytes[..bound]);
    let path_bytes = path_bytes(&path);
    let file_input = input(INPUT_FILE, ENCODING_DER, &path_bytes);
    let exact_file = certificate_input_bytes(&file_input).expect("exact file bound is accepted");
    assert_eq!(exact_file.len(), bound);
    fs::write(&path, &bytes).expect("over-bound test material is written");
    assert_eq!(
        certificate_input_bytes(&file_input).err(),
        Some(ERROR_RESOURCE_LIMIT)
    );

    let split = [
        input(INPUT_MEMORY, ENCODING_DER, &bytes[..bound / 2]),
        input(INPUT_MEMORY, ENCODING_DER, &bytes[bound / 2..bound + 1]),
    ];
    assert_eq!(
        validate_transport_input_total(&split),
        Err(ERROR_RESOURCE_LIMIT),
        "aggregate client, trust, and CRL inputs may not exceed the facade cap"
    );
}

#[test]
fn ffi_client_correlation_value_enforces_the_local_material_bound() {
    let maximum =
        usize::try_from(MAX_TRANSPORT_MATERIAL_BYTES).expect("the ABI cap fits this test target");
    let exact_value = vec![b'c'; maximum];
    let mut batch = ptr::null_mut();
    assert_eq!(kmipkit_client_batch_create(&mut batch), SUCCESS);
    assert_eq!(
        kmipkit_client_batch_set_client_correlation_value(
            batch,
            exact_value.as_ptr(),
            u64::try_from(exact_value.len()).expect("exact cap fits the ABI"),
        ),
        SUCCESS,
        "the exact local bound is accepted"
    );
    assert_eq!(
        kmipkit_client_batch_set_client_correlation_value(
            batch,
            ptr::null(),
            MAX_TRANSPORT_MATERIAL_BYTES + 1,
        ),
        ERROR_RESOURCE_LIMIT,
        "an over-bound length is rejected before reading the input pointer"
    );
    kmipkit_client_batch_release(batch);
}

#[test]
fn bounded_file_reader_rejects_growth_after_aggregate_preflight() {
    let bound = usize::try_from(MAX_TRANSPORT_MATERIAL_BYTES)
        .expect("the material cap fits this test target");
    let per_file = bound / 2;
    let original_bytes = vec![b'a'; per_file];
    let grown_bytes = vec![b'b'; per_file + 1];
    let mut files = TemporaryFiles::new();
    let first_path = files.write("aggregate-first", &original_bytes);
    let second_path = files.write("aggregate-second", &original_bytes);
    let first_path_bytes = path_bytes(&first_path);
    let second_path_bytes = path_bytes(&second_path);
    let inputs = [
        input(INPUT_FILE, ENCODING_DER, &first_path_bytes),
        input(INPUT_FILE, ENCODING_DER, &second_path_bytes),
    ];

    validate_transport_input_total(&inputs)
        .expect("the exact aggregate file size passes preflight");
    fs::write(&second_path, grown_bytes).expect("second file grows after preflight");

    let first_file = fs::File::open(&first_path).expect("first file opens");
    let first = read_transport_reader_with_budget(first_file, per_file as u64, per_file as u64)
        .expect("first file consumes half of the aggregate budget");
    assert_eq!(first.len(), per_file);
    let second_file = fs::File::open(&second_path).expect("grown second file opens");
    let error = read_transport_reader_with_budget(second_file, per_file as u64, per_file as u64)
        .expect_err("grown file must not exceed the remaining aggregate budget");
    assert_eq!(error, ERROR_RESOURCE_LIMIT);
}

#[test]
fn private_key_file_reader_zeroizes_partial_error_and_over_bound_bytes() {
    let exact_observer = SecretBufferObserver::new();
    let exact = SecretBuffer::read_from_with_observer_for_test(
        std::io::Cursor::new(b"1234"),
        4,
        exact_observer.clone(),
    )
    .expect("the exact key-input bound is accepted");
    drop(exact);
    assert!(exact_observer.initialized_bytes_were_zeroized());

    let partial_observer = SecretBufferObserver::new();
    let partial_error = SecretBuffer::read_from_with_observer_for_test(
        PartialThenErrorReader { emitted: false },
        64,
        partial_observer.clone(),
    )
    .err()
    .expect("the reader reports its scripted I/O failure");
    assert_eq!(partial_error, ERROR_INVALID_INPUT);
    assert!(partial_observer.initialized_bytes_were_zeroized());

    let oversized_observer = SecretBufferObserver::new();
    let oversized_error = SecretBuffer::read_from_with_observer_for_test(
        ChunkedReader {
            bytes: b"12345",
            position: 0,
            max_chunk: 4,
        },
        4,
        oversized_observer.clone(),
    )
    .err()
    .expect("one byte over the local cap is rejected");
    assert_eq!(oversized_error, ERROR_RESOURCE_LIMIT);
    assert!(oversized_observer.initialized_bytes_were_zeroized());
}

fn unconnected_client_handle() -> *mut kmipkit_client_t {
    let registry = client_extension::client_extension_registry(Vec::new(), extension::defaults())
        .expect("the empty extension registry is valid");
    let configuration = client_extension::ClientConfiguration::new(registry);
    let pki = EphemeralPki::generate().expect("ephemeral PKI generation succeeds");
    let identity = pki.client_identity();
    let certificates = identity
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(certificates),
            PrivateKeyInput::from_der(identity.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .tls_server_name("server.kmipkit.test")
        .build()
        .expect("the local TLS transport configuration is valid");
    let client = Client::new(configuration, transport).expect("client construction succeeds");
    make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(std::sync::Mutex::new(client))),
    )
}

fn client_configuration_handle() -> *mut kmipkit_client_configuration_t {
    make_handle(
        Kind::Configuration,
        HandleValue::Configuration(empty_client_configuration()),
    )
}

fn empty_client_configuration() -> client_extension::ClientConfiguration {
    let registry = client_extension::client_extension_registry(Vec::new(), extension::defaults())
        .expect("the empty extension registry is valid");
    client_extension::ClientConfiguration::new(registry)
}

fn empty_client_batch_handle() -> *mut kmipkit_client_batch_t {
    make_handle(
        Kind::ClientBatch,
        HandleValue::ClientBatch(std::sync::Mutex::new(ClientBatchBuilder::new())),
    )
}

fn default_codec_limits_handle() -> *mut kmipkit_codec_limits_t {
    make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(CodecLimits::defaults()),
    )
}

#[test]
fn ffi_execute_with_error_ttlv_clears_batch_slot_when_retained_owner_blocks_consumption() {
    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut batch = empty_client_batch_handle();
    let retained_owner = clone_handle_owner(batch, Kind::ClientBatch)
        .expect("the test retains one additional owner");
    let mut response = ptr::null_mut();
    let mut error_response = ptr::null_mut();

    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(batch.is_null(), "consumption clears the caller's slot");
    assert!(response.is_null());
    assert!(error_response.is_null());

    drop(retained_owner);
    release(client, Kind::Client);
    release(limits, Kind::CodecLimits);
}

#[test]
fn ffi_execute_with_error_ttlv_rejects_null_slots_and_wrong_handle_kinds() {
    let mut response = ptr::null_mut();
    let mut error_response = ptr::null_mut();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(error_response.is_null());

    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut response,
            ptr::null_mut(),
        ),
        ERROR_INVALID_INPUT
    );
    assert!(response.is_null());

    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(response.is_null());
    assert!(error_response.is_null());

    let mut batch = ptr::null_mut();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            ptr::null_mut(),
            &mut batch,
            ptr::null_mut(),
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(batch.is_null());
    assert!(response.is_null());
    assert!(error_response.is_null());

    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let wrong_kind = client_configuration_handle();
    let mut batch = empty_client_batch_handle();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            wrong_kind.cast(),
            &mut batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(
        !batch.is_null(),
        "validation before consumption retains the batch"
    );

    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut batch,
            wrong_kind.cast(),
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(!batch.is_null(), "wrong limits retain the batch");

    let mut wrong_batch = wrong_kind.cast();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut wrong_batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        wrong_batch.cast::<kmipkit_client_configuration_t>(),
        wrong_kind
    );
    assert!(response.is_null());
    assert!(error_response.is_null());

    release(batch, Kind::ClientBatch);
    release(wrong_kind, Kind::Configuration);
    release(client, Kind::Client);
    release(limits, Kind::CodecLimits);
}

#[test]
fn ffi_execute_fails_closed_for_poisoned_batch_and_client_locks() {
    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut poisoned_batch = empty_client_batch_handle();
    {
        // SAFETY: the handle was just allocated above and remains live while its mutex is poisoned.
        let handle = unsafe { &*poisoned_batch.cast::<Handle>() };
        let HandleValue::ClientBatch(batch) = &handle.value else {
            unreachable!("the constructor created a client batch handle");
        };
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = batch.lock().expect("batch mutex starts unpoisoned");
            panic!("poison the test batch mutex");
        }));
        assert!(panic.is_err());
    }

    let mut response = ptr::null_mut();
    let mut error_response = ptr::null_mut();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut poisoned_batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INTERNAL
    );
    assert!(poisoned_batch.is_null());
    assert!(response.is_null());
    assert!(error_response.is_null());

    {
        // SAFETY: the handle was just allocated above and remains live while its mutex is poisoned.
        let handle = unsafe { &*client.cast::<Handle>() };
        let HandleValue::Client(client) = &handle.value else {
            unreachable!("the constructor created a client handle");
        };
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = client.lock().expect("client mutex starts unpoisoned");
            panic!("poison the test client mutex");
        }));
        assert!(panic.is_err());
    }

    let mut healthy_batch = empty_client_batch_handle();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut healthy_batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INTERNAL
    );
    assert!(healthy_batch.is_null());
    assert!(response.is_null());
    assert!(error_response.is_null());

    release(client, Kind::Client);
    release(limits, Kind::CodecLimits);
}

#[test]
fn ffi_execute_rejects_inconsistent_client_abi_handle_envelopes() {
    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut response = ptr::null_mut();
    let mut error_response = ptr::null_mut();

    let wrong_client = make_handle(
        Kind::Client,
        HandleValue::Configuration(empty_client_configuration()),
    );
    let mut batch = empty_client_batch_handle();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            wrong_client,
            &mut batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(!batch.is_null());
    release(batch, Kind::ClientBatch);
    release(wrong_client, Kind::Client);

    let wrong_limits = make_handle(
        Kind::CodecLimits,
        HandleValue::Configuration(empty_client_configuration()),
    );
    let mut batch = empty_client_batch_handle();
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut batch,
            wrong_limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(!batch.is_null());
    release(batch, Kind::ClientBatch);
    release(wrong_limits, Kind::CodecLimits);

    let mut wrong_batch = make_handle(
        Kind::ClientBatch,
        HandleValue::Configuration(empty_client_configuration()),
    );
    assert_eq!(
        kmipkit_client_execute_with_error_ttlv_v1(
            client,
            &mut wrong_batch,
            limits,
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(wrong_batch.is_null());

    release(client, Kind::Client);
    release(limits, Kind::CodecLimits);
}

struct PartialThenErrorReader {
    emitted: bool,
}

struct ChunkedReader {
    bytes: &'static [u8],
    position: usize,
    max_chunk: usize,
}

impl Read for ChunkedReader {
    fn read(&mut self, destination: &mut [u8]) -> std::io::Result<usize> {
        let remaining = &self.bytes[self.position..];
        let count = remaining.len().min(destination.len()).min(self.max_chunk);
        destination[..count].copy_from_slice(&remaining[..count]);
        self.position += count;
        Ok(count)
    }
}

impl Read for PartialThenErrorReader {
    fn read(&mut self, destination: &mut [u8]) -> std::io::Result<usize> {
        if self.emitted {
            return Err(std::io::Error::other("scripted partial-read failure"));
        }
        self.emitted = true;
        let sentinel = b"PRIVATE-KEY-SENTINEL";
        let count = destination.len().min(sentinel.len());
        destination[..count].copy_from_slice(&sentinel[..count]);
        Ok(count)
    }
}
