use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use base64::Engine;
use kmipkit_client::{Client, ClientBatchResponse, ClientError, ClientErrorCategory};
use kmipkit_protocol::{
    AttributeSet, CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateSplitKeyRequest,
    DelegatedLoginRequest, DerivationMethod, DerivationParameters, DeriveKeyRequest, ExportRequest,
    GetRequest, GetUsageAllocationRequest, ImportRequest, LocateRequest, ObjectGroupMember,
    ObjectType, OperationData, SecretBytes, SplitKeyMethod, StorageStatusMask, UniqueIdentifier,
};
use kmipkit_test_support::EphemeralPki;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestDeliveryState,
    RequestOptions, TimeoutLimit, TimeoutPolicy, TransportConfig, TrustSource,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};
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

fn failed_cryptographic_response(operation: u32, unique_id: &[u8]) -> Vec<u8> {
    ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, operation),
                    ttlv_item(0x0042_0093, 0x08, unique_id),
                    ttlv_enumeration(0x0042_007f, 1),
                    ttlv_enumeration(0x0042_007e, 6),
                ],
            ),
        ],
    )
}

fn failed_check_response(unique_id: &[u8]) -> Vec<u8> {
    let protocol_version = ttlv_structure(
        0x0042_0069,
        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
    );
    let header = ttlv_structure(
        0x0042_007a,
        [
            protocol_version,
            ttlv_datetime(0x0042_0092, 1),
            ttlv_integer(0x0042_000d, 1),
        ],
    );
    let payload = ttlv_structure(
        0x0042_007c,
        [
            ttlv_item(0x0042_0094, 0x07, b"failure-identifier"),
            ttlv_item(0x0042_0096, 0x03, &(-9_i64).to_be_bytes()),
            ttlv_item(0x0042_002c, 0x02, &(-1_i32).to_be_bytes()),
            ttlv_item(0x0042_0049, 0x0a, &0_u32.to_be_bytes()),
            ttlv_item(0x0054_1234, 0x02, &(-7_i32).to_be_bytes()),
        ],
    );
    ttlv_structure(
        0x0042_007b,
        [
            header,
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 9),
                    ttlv_item(0x0042_0093, 0x08, unique_id),
                    ttlv_enumeration(0x0042_007f, 1),
                    ttlv_enumeration(0x0042_007e, 0x29),
                    payload,
                ],
            ),
        ],
    )
}

fn successful_check_response() -> Vec<u8> {
    let protocol_version = ttlv_structure(
        0x0042_0069,
        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
    );
    let header = ttlv_structure(
        0x0042_007a,
        [
            protocol_version,
            ttlv_datetime(0x0042_0092, 1),
            ttlv_integer(0x0042_000d, 1),
        ],
    );
    let payload = ttlv_structure(
        0x0042_007c,
        [
            ttlv_item(0x0042_0094, 0x07, b"checked-object"),
            ttlv_item(0x0042_0096, 0x03, &3_i64.to_be_bytes()),
            ttlv_item(0x0042_002c, 0x02, &2_i32.to_be_bytes()),
            ttlv_item(0x0042_0049, 0x0a, &60_u32.to_be_bytes()),
            ttlv_item(0x0054_1234, 0x02, &(-7_i32).to_be_bytes()),
        ],
    );
    ttlv_structure(
        0x0042_007b,
        [
            header,
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 9),
                    ttlv_item(0x0042_0093, 0x08, b"check-success"),
                    ttlv_enumeration(0x0042_007f, 0),
                    payload,
                ],
            ),
        ],
    )
}

fn failed_cryptographic_item(
    request: ClientRequest,
    operation: u32,
    unique_id: &[u8],
) -> *mut kmipkit_client_batch_item_response_t {
    let response = execute_single_abi_response(
        request,
        unique_id,
        failed_cryptographic_response(operation, unique_id),
    )
    .expect("the KMIP failure response remains a typed completed result");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);
    item
}

fn successful_cryptographic_response(
    operation: u32,
    unique_id: &[u8],
    response_fields: Vec<Vec<u8>>,
) -> Vec<u8> {
    let protocol_version = ttlv_structure(
        0x0042_0069,
        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
    );
    let header = ttlv_structure(
        0x0042_007a,
        [
            protocol_version,
            ttlv_datetime(0x0042_0092, 1),
            ttlv_integer(0x0042_000d, 1),
        ],
    );
    let response_payload = {
        let children = response_fields.into_iter().flatten().collect::<Vec<_>>();
        ttlv_item(0x0042_007c, 0x01, &children)
    };
    let response_item = ttlv_structure(
        0x0042_000f,
        [
            ttlv_enumeration(0x0042_005c, operation),
            ttlv_item(0x0042_0093, 0x08, unique_id),
            ttlv_enumeration(0x0042_007f, 0),
            response_payload,
        ],
    );
    ttlv_structure(0x0042_007b, [header, response_item])
}

fn successful_cryptographic_item(
    request: ClientRequest,
    operation: u32,
    unique_id: &[u8],
    response_fields: Vec<Vec<u8>>,
) -> *mut kmipkit_client_batch_item_response_t {
    let response = execute_single_abi_response(
        request,
        unique_id,
        successful_cryptographic_response(operation, unique_id, response_fields),
    )
    .expect("the successful cryptographic response matches its typed request");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);
    item
}

fn assert_response_span(span: kmipkit_byte_span_v1_t, expected: &[u8]) {
    assert_eq!(
        span.length,
        u64::try_from(expected.len()).expect("response span length fits u64")
    );
    assert!(!span.data.is_null());
    // SAFETY: the caller keeps the response-item handle alive while checking its borrowed span.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                span.data,
                usize::try_from(span.length).expect("response span fits this test target"),
            )
        },
        expected
    );
}

fn assert_response_identifier(identifier: &kmipkit_unique_identifier_v1_t, expected: &[u8]) {
    assert_eq!(
        identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(
        identifier.text_length,
        u64::try_from(expected.len()).expect("identifier length fits u64")
    );
    assert!(!identifier.text_data.is_null());
    // SAFETY: the caller keeps the response-item handle alive while checking its borrowed text.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                identifier.text_data,
                usize::try_from(identifier.text_length)
                    .expect("response identifier fits this test target"),
            )
        },
        expected
    );
}

#[test]
fn ffi_successful_hash_response_getter_returns_typed_fields() {
    let correlation = [0x41, 0x00, 0x42];
    let mut hash_parameters = Structure::new();
    hash_parameters
        .try_push(
            Item::new(
                RawTag::new(0x0042_0038)
                    .expect("Hashing Algorithm tag fits the KMIP tag field")
                    .try_checked()
                    .expect("Hashing Algorithm is allocated in KMIP 2.1"),
                Value::enumeration(6),
            )
            .expect("valid Hashing Algorithm test item builds"),
        )
        .expect("Hashing Algorithm item fits within the test structure limit");
    let hash = successful_cryptographic_item(
        ClientRequest::Hash(HashRequest::new(hash_parameters).with_data(
            OperationData::ByteString(SecretBytes::new(b"request-hash-data".to_vec())),
        )),
        0x27,
        b"hash-success",
        vec![
            ttlv_item(0x0042_00c2, 0x08, b"hash-response-data"),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
        ],
    );
    let mut present = 0;
    let mut data_present = 0;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 0;
    let mut correlation_value = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_hash(
            hash,
            &raw mut present,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!((present, data_present, correlation_present), (1, 1, 1));
    assert_response_span(data, b"hash-response-data");
    assert_response_span(correlation_value, &correlation);
    kmipkit_client_batch_item_response_release(hash);
}

#[test]
fn ffi_successful_mac_response_getter_returns_typed_fields() {
    let correlation = [0x41, 0x00, 0x42];
    let response_identifier = b"response-id";
    let mac = successful_cryptographic_item(
        ClientRequest::Mac(MacRequest::new().with_data(OperationData::ByteString(
            SecretBytes::new(b"request-mac-data".to_vec()),
        ))),
        0x23,
        b"mac-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, response_identifier),
            ttlv_item(0x0042_00c4, 0x08, b"mac-response-data"),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut mac_data_present = 0;
    let mut mac_data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 0;
    let mut correlation_value = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_mac(
            mac,
            &raw mut present,
            &raw mut identifier,
            &raw mut mac_data_present,
            &raw mut mac_data,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!((present, mac_data_present, correlation_present), (1, 1, 1));
    assert_response_identifier(&identifier, response_identifier);
    assert_response_span(mac_data, b"mac-response-data");
    assert_response_span(correlation_value, &correlation);
    kmipkit_client_batch_item_response_release(mac);
}

#[test]
fn ffi_successful_mac_verify_response_getter_returns_typed_fields() {
    let correlation = [0x41, 0x00, 0x42];
    let response_identifier = b"response-id";
    let mac_verify = successful_cryptographic_item(
        ClientRequest::MacVerify(
            MacVerifyRequest::new().with_mac_data(SecretBytes::new(b"request-mac".to_vec())),
        ),
        0x24,
        b"mac-verify-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, response_identifier),
            ttlv_enumeration(0x0042_0128, 0xfedc_ba98),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut validity_present = 0;
    let mut validity = 0;
    let mut correlation_present = 0;
    let mut correlation_value = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_mac_verify(
            mac_verify,
            &raw mut present,
            &raw mut identifier,
            &raw mut validity_present,
            &raw mut validity,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!(
        (present, validity_present, validity, correlation_present),
        (1, 1, 0xfedc_ba98, 1)
    );
    assert_response_identifier(&identifier, response_identifier);
    assert_response_span(correlation_value, &correlation);
    kmipkit_client_batch_item_response_release(mac_verify);
}

#[test]
fn ffi_successful_sign_response_getter_returns_typed_fields() {
    let correlation = [0x41, 0x00, 0x42];
    let response_identifier = b"response-id";
    let sign = successful_cryptographic_item(
        ClientRequest::Sign(SignRequest::new().with_data(OperationData::ByteString(
            SecretBytes::new(b"request-sign-data".to_vec()),
        ))),
        0x21,
        b"sign-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, response_identifier),
            ttlv_item(0x0042_00c7, 0x08, b"signature-response-data"),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut signature_data_present = 0;
    let mut signature_data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 0;
    let mut correlation_value = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_sign(
            sign,
            &raw mut present,
            &raw mut identifier,
            &raw mut signature_data_present,
            &raw mut signature_data,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!(
        (present, signature_data_present, correlation_present),
        (1, 1, 1)
    );
    assert_response_identifier(&identifier, response_identifier);
    assert_response_span(signature_data, b"signature-response-data");
    assert_response_span(correlation_value, &correlation);
    kmipkit_client_batch_item_response_release(sign);
}

#[test]
fn ffi_successful_signature_verify_response_getter_returns_typed_fields() {
    let correlation = [0x41, 0x00, 0x42];
    let response_identifier = b"response-id";
    let signature_verify = successful_cryptographic_item(
        ClientRequest::SignatureVerify(
            SignatureVerifyRequest::new()
                .with_signature_data(SecretBytes::new(b"request-signature".to_vec())),
        ),
        0x22,
        b"signature-verify-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, response_identifier),
            ttlv_enumeration(0x0042_0128, 0xfedc_ba98),
            ttlv_item(0x0042_00c2, 0x08, b"recovered-response-data"),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut validity_present = 0;
    let mut validity = 0;
    let mut recovered_present = 0;
    let mut recovered = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 0;
    let mut correlation_value = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_signature_verify(
            signature_verify,
            &raw mut present,
            &raw mut identifier,
            &raw mut validity_present,
            &raw mut validity,
            &raw mut recovered_present,
            &raw mut recovered,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!(
        (
            present,
            validity_present,
            validity,
            recovered_present,
            correlation_present
        ),
        (1, 1, 0xfedc_ba98, 1, 1)
    );
    assert_response_identifier(&identifier, response_identifier);
    assert_response_span(recovered, b"recovered-response-data");
    assert_response_span(correlation_value, &correlation);
    kmipkit_client_batch_item_response_release(signature_verify);
}

#[test]
fn ffi_hash_response_getters_do_not_report_failed_payloads_as_present() {
    let mut hash_parameters = Structure::new();
    hash_parameters
        .try_push(
            Item::new(
                RawTag::new(0x0042_0038)
                    .expect("Hashing Algorithm tag fits the KMIP tag field")
                    .try_checked()
                    .expect("Hashing Algorithm is allocated in KMIP 2.1"),
                Value::enumeration(6),
            )
            .expect("valid Hashing Algorithm test item builds"),
        )
        .expect("Hashing Algorithm item fits within the test structure limit");
    let hash = failed_cryptographic_item(
        ClientRequest::Hash(HashRequest::new(hash_parameters).with_data(
            OperationData::ByteString(SecretBytes::new(b"failure-test-data".to_vec())),
        )),
        0x27,
        b"hash-failure",
    );
    let mut present = 9;
    let mut data_present = 9;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 9;
    let mut correlation = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_hash(
            hash,
            &raw mut present,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!((present, data_present, correlation_present), (0, 0, 0));
    kmipkit_client_batch_item_response_release(hash);
}

#[test]
fn ffi_certify_response_getter_does_not_report_failure_as_a_generated_identifier() {
    let certify = CertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
        .with_certificate_request_type(0xfedc_ba98);
    let item = failed_cryptographic_item(ClientRequest::Certify(certify), 0x06, b"certify-failure");
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_certify(item, &raw mut present, &raw mut identifier)
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());
    assert_eq!(identifier.text_length, 0);
    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_certify_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_certify_response_getter_returns_successful_generated_identifier() {
    let response_bytes = ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 6),
                    ttlv_item(0x0042_0093, 0x08, b"certify-success"),
                    ttlv_enumeration(0x0042_007f, 0),
                    ttlv_structure(
                        0x0042_007c,
                        [ttlv_item(0x0042_0094, 0x07, b"issued-certificate")],
                    ),
                ],
            ),
        ],
    );
    let response = execute_single_abi_response(
        ClientRequest::Certify(
            CertifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
                .with_certificate_request_type(1),
        ),
        b"certify-success",
        response_bytes,
    )
    .expect("the successful Certify response matches its request");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item)
    );
    kmipkit_client_batch_response_release(response_handle);

    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_certify(item, &raw mut present, &raw mut identifier)
    );
    assert_eq!(present, 1);
    assert_eq!(identifier.kind, 1);
    assert_eq!(identifier.text_length, 18);
    assert!(!identifier.text_data.is_null());
    let identifier_length = usize::try_from(identifier.text_length)
        .expect("the response identifier length fits this test target");
    let actual = unsafe { std::slice::from_raw_parts(identifier.text_data, identifier_length) };
    assert_eq!(actual, b"issued-certificate");
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_check_constructor_validates_each_presence_discriminator_and_accepts_raw_values() {
    for (count_flag, mask_flag, lease_flag) in [(2, 0, 0), (0, 2, 0), (0, 0, 2)] {
        let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();
        assert_eq!(
            kmipkit_client_batch_item_check(
                ptr::null(),
                count_flag,
                i64::MIN,
                mask_flag,
                i32::MIN,
                lease_flag,
                u32::MAX,
                &mut item,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(item.is_null());
    }

    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_check(
            ptr::null(),
            1,
            i64::MIN,
            1,
            i32::MIN,
            1,
            u32::MAX,
            &mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);

    let unique_identifier_text = b"optional-check-id";
    let mut unique_identifier = empty_unique_identifier_output_v1();
    unique_identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    unique_identifier.text_data = unique_identifier_text.as_ptr();
    unique_identifier.text_length =
        u64::try_from(unique_identifier_text.len()).expect("test identifier fits ABI");
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_check(
            &raw const unique_identifier,
            0,
            0,
            0,
            0,
            0,
            0,
            &raw mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);
}

#[test]
fn ffi_delegated_login_constructor_validates_typed_fields_and_requires_rights() {
    let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();
    assert_eq!(
        kmipkit_client_batch_item_delegated_login(
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());

    let rights = make_handle(
        Kind::Structure,
        HandleValue::Structure(kmipkit_ttlv::Structure::new()),
    );
    for (lease_kind, lease_value, count_present, count) in [
        (3, 0, 0, 0),
        (0, 1, 0, 0),
        (1, -1, 0, 0),
        (1, i64::from(u32::MAX) + 1, 0, 0),
        (0, 0, 2, 0),
        (0, 0, 0, 1),
    ] {
        item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();
        assert_eq!(
            kmipkit_client_batch_item_delegated_login(
                rights,
                lease_kind,
                lease_value,
                count_present,
                count,
                ptr::null_mut(),
                &mut item,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(item.is_null());
    }

    item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_delegated_login(
            rights,
            2,
            i64::MIN,
            1,
            i32::MIN,
            ptr::null_mut(),
            &mut item,
        ),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);

    let mut interval_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_delegated_login(
            rights,
            1,
            i64::from(u32::MAX),
            0,
            0,
            ptr::null_mut(),
            &mut interval_item,
        ),
        SUCCESS
    );
    assert!(!interval_item.is_null());
    kmipkit_client_batch_item_release(interval_item);

    let mut wrong_usage_limits = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_discover_versions(&mut wrong_usage_limits),
        SUCCESS
    );
    let mut invalid_item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();
    assert_eq!(
        kmipkit_client_batch_item_delegated_login(
            rights,
            0,
            0,
            0,
            0,
            wrong_usage_limits.cast(),
            &mut invalid_item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(invalid_item.is_null());
    kmipkit_client_batch_item_release(wrong_usage_limits);
    release(rights, Kind::Structure);
}

#[test]
fn ffi_delegated_login_response_getters_clear_outputs_for_invalid_item() {
    let mut present = 9;
    let mut ticket_present = 9;
    let mut ticket_type = u32::MAX;
    let mut ticket_value = byte_span(b"secret sentinel");
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login(
            ptr::null_mut(),
            &mut present,
            &mut ticket_present,
            &mut ticket_type,
            &mut ticket_value,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!((present, ticket_present, ticket_type), (0, 0, 0));
    assert!(ticket_value.data.is_null());
    assert_eq!(ticket_value.length, 0);

    let mut ttlv_present = 9;
    let mut ticket_ttlv = ptr::dangling_mut::<kmipkit_ttlv_structure_t>();
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login_ttlv(
            ptr::null_mut(),
            &mut ttlv_present,
            &mut ticket_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(ttlv_present, 0);
    assert!(ticket_ttlv.is_null());

    let malformed_value = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::Structure(Structure::new()),
    );
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login(
            malformed_value,
            &mut present,
            &mut ticket_present,
            &mut ticket_type,
            &mut ticket_value,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!((present, ticket_present, ticket_type), (0, 0, 0));
    assert!(ticket_value.data.is_null());
    assert_eq!(ticket_value.length, 0);
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login_ttlv(
            malformed_value,
            &mut ttlv_present,
            &mut ticket_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(ttlv_present, 0);
    assert!(ticket_ttlv.is_null());
    release(malformed_value, Kind::ClientBatchItemResponse);
}

#[test]
fn ffi_delegated_login_response_getters_return_ticket_fields_and_owned_ttlv() {
    let secret = b"delegated-login-abi-ticket";
    let item = successful_cryptographic_item(
        ClientRequest::DelegatedLogin(DelegatedLoginRequest::new(Structure::new())),
        0x2f,
        b"delegated-login-abi",
        vec![ttlv_structure(
            0x0042_0149,
            [
                ttlv_enumeration(0x0042_014a, 0xfeed_beef),
                ttlv_item(0x0042_014b, 0x08, secret),
            ],
        )],
    );

    let mut present = 0;
    let mut ticket_present = 0;
    let mut ticket_type = 0;
    let mut ticket_value = byte_span(&[]);
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login(
            item,
            &mut present,
            &mut ticket_present,
            &mut ticket_type,
            &mut ticket_value,
        ),
        SUCCESS
    );
    assert_eq!((present, ticket_present, ticket_type), (1, 1, 0xfeed_beef));
    assert_response_span(ticket_value, secret);

    let mut ttlv_present = 0;
    let mut ticket_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login_ttlv(
            item,
            &mut ttlv_present,
            &mut ticket_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(ttlv_present, 1);
    assert!(!ticket_ttlv.is_null());
    kmipkit_ttlv_structure_release(ticket_ttlv);

    let response_item = reference_handle(item, Kind::ClientBatchItemResponse)
        .expect("successful delegated-login item remains registered");
    let HandleValue::ClientBatchItemResponse { response, .. } = &response_item.value else {
        panic!("the response item constructor uses the expected handle payload");
    };
    let out_of_range_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(response),
            index: usize::MAX,
        },
    );
    let mut out_of_range_present = 9;
    let mut out_of_range_ticket_present = 9;
    let mut out_of_range_ticket_type = u32::MAX;
    let mut out_of_range_ticket_value = byte_span(b"secret sentinel");
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login(
            out_of_range_item,
            &mut out_of_range_present,
            &mut out_of_range_ticket_present,
            &mut out_of_range_ticket_type,
            &mut out_of_range_ticket_value,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(
        (
            out_of_range_present,
            out_of_range_ticket_present,
            out_of_range_ticket_type,
        ),
        (0, 0, 0)
    );
    assert!(out_of_range_ticket_value.data.is_null());
    assert_eq!(out_of_range_ticket_value.length, 0);
    let mut out_of_range_ttlv_present = 9;
    let mut out_of_range_ticket_ttlv = ptr::dangling_mut::<kmipkit_ttlv_structure_t>();
    assert_eq!(
        kmipkit_client_batch_item_response_delegated_login_ttlv(
            out_of_range_item,
            &mut out_of_range_ttlv_present,
            &mut out_of_range_ticket_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(out_of_range_ttlv_present, 0);
    assert!(out_of_range_ticket_ttlv.is_null());
    release(out_of_range_item, Kind::ClientBatchItemResponse);
    drop(response_item);
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_failed_check_keeps_policy_fields_and_complete_ttlv_available_to_c() {
    let response_bytes = failed_check_response(b"check-failure");
    let decoded = kmipkit_ttlv::codec::decode(&response_bytes)
        .expect("the Check response fixture is valid TTLV");
    let response_tree = decoded
        .with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(structure) => Some(
                structure
                    .try_clone()
                    .expect("the Check response fixture is within the model depth limit"),
            ),
            _ => None,
        })
        .expect("the Check response fixture is a Structure");
    let validated = kmipkit_protocol::ResponseMessage::try_from_ttlv(response_tree)
        .unwrap_or_else(|error| panic!("Check response fixture fails KMIP validation: {error:?}"));
    let response_item = validated
        .batch_items()
        .next()
        .expect("the Check response fixture contains one batch item");
    kmipkit_protocol::CheckResponse::try_from_response_item(response_item)
        .unwrap_or_else(|error| panic!("Check response fixture fails typed mapping: {error:?}"));
    let item = execute_single_abi_response(
        ClientRequest::Check(
            CheckRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("check-request".to_owned())),
        ),
        b"check-failure",
        response_bytes,
    )
    .expect("the policy-denial Check result remains typed");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(item)),
    );
    let mut response_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &mut response_item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);

    // SAFETY: the compiled C consumer owns the response-item handle and releases it after copying TTLV.
    let result =
        unsafe { kmipkit_client_abi_c_consumer_validate_failed_check_response(response_item) };
    assert_eq!(result, 1);
}

fn successful_check_client_response() -> Arc<ClientBatchResponse> {
    let response_bytes = successful_check_response();
    let decoded = kmipkit_ttlv::codec::decode(&response_bytes)
        .expect("the successful Check response fixture is valid TTLV");
    let response_tree = decoded
        .with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(structure) => Some(
                structure
                    .try_clone()
                    .expect("the Check fixture is within the model depth limit"),
            ),
            _ => None,
        })
        .expect("the Check fixture is a Structure");
    let validated = kmipkit_protocol::ResponseMessage::try_from_ttlv(response_tree)
        .unwrap_or_else(|error| panic!("Check fixture fails KMIP validation: {error:?}"));
    let response_item = validated
        .batch_items()
        .next()
        .expect("the Check fixture contains one batch item");
    kmipkit_protocol::CheckResponse::try_from_response_item(response_item)
        .unwrap_or_else(|error| panic!("Check fixture fails typed mapping: {error:?}"));

    Arc::new(
        execute_single_abi_response(
            ClientRequest::Check(
                CheckRequest::new()
                    .with_unique_identifier(UniqueIdentifier::TextString(
                        "checked-object".to_owned(),
                    ))
                    .with_usage_limits_count(3)
                    .with_cryptographic_usage_mask(2)
                    .with_lease_time(60),
            ),
            b"check-success",
            response_bytes,
        )
        .expect("the successful Check response maps its identifier and policy fields"),
    )
}

fn successful_check_response_item(
    response: &Arc<ClientBatchResponse>,
) -> *mut kmipkit_client_batch_item_response_t {
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::clone(response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);
    item
}

fn assert_check_response_getter_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 0;
    let mut identifier_present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut count_present = 0;
    let mut count = 0;
    let mut mask_present = 0;
    let mut mask = 0;
    let mut lease_present = 0;
    let mut lease = 0;
    assert_eq!(
        kmipkit_client_batch_item_response_check(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
            &raw mut count_present,
            &raw mut count,
            &raw mut mask_present,
            &raw mut mask,
            &raw mut lease_present,
            &raw mut lease,
        ),
        expected_status
    );
}

fn assert_check_response_ttlv_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 9;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_check_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        expected_status
    );
    if expected_status != SUCCESS {
        assert_eq!(present, 0);
        assert!(response_ttlv.is_null());
    }
}

#[test]
fn ffi_successful_check_getters_return_fields_and_ordered_ttlv() {
    let response = successful_check_client_response();
    let item = successful_check_response_item(&response);

    let mut present = 9;
    let mut identifier_present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut count_present = 9;
    let mut count = -1;
    let mut mask_present = 9;
    let mut mask = -1;
    let mut lease_present = 9;
    let mut lease = 0;
    assert_eq!(
        kmipkit_client_batch_item_response_check(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
            &raw mut count_present,
            &raw mut count,
            &raw mut mask_present,
            &raw mut mask,
            &raw mut lease_present,
            &raw mut lease,
        ),
        SUCCESS
    );
    assert_eq!(
        (
            present,
            identifier_present,
            count_present,
            count,
            mask_present,
            mask,
            lease_present,
            lease
        ),
        (1, 1, 1, 3, 1, 2, 1, 60)
    );
    assert_response_identifier(&identifier, b"checked-object");

    identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_check(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
            &raw mut count_present,
            &raw mut count,
            &raw mut mask_present,
            &raw mut mask,
            &raw mut lease_present,
            &raw mut lease,
        ),
        ERROR_INVALID_INPUT
    );

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_check_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_check_getters_reject_inconsistent_response_item_handles() {
    let response = successful_check_client_response();
    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    assert_check_response_getter_status(mismatched_item, ERROR_INVALID_INPUT);
    assert_check_response_ttlv_status(mismatched_item, ERROR_INVALID_INPUT);
    kmipkit_client_batch_item_response_release(mismatched_item);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(&response),
            index: usize::MAX,
        },
    );
    assert_check_response_getter_status(invalid_index_item, ERROR_INTERNAL);
    assert_check_response_ttlv_status(invalid_index_item, ERROR_INTERNAL);
    kmipkit_client_batch_item_response_release(invalid_index_item);
}

#[test]
fn ffi_mac_response_getters_do_not_report_failed_payloads_as_present() {
    let mac = failed_cryptographic_item(
        ClientRequest::Mac(MacRequest::new().with_data(OperationData::ByteString(
            SecretBytes::new(b"failure-test-data".to_vec()),
        ))),
        0x23,
        b"mac-failure",
    );
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut data_present = 9;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 9;
    let mut correlation = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_mac(
            mac,
            &raw mut present,
            &raw mut identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!((present, data_present, correlation_present), (0, 0, 0));
    kmipkit_client_batch_item_response_release(mac);
}

#[test]
fn ffi_mac_verify_response_getters_do_not_report_failed_payloads_as_present() {
    let mac_verify = failed_cryptographic_item(
        ClientRequest::MacVerify(
            MacVerifyRequest::new().with_mac_data(SecretBytes::new(b"failure-test-mac".to_vec())),
        ),
        0x24,
        b"mac-verify-failure",
    );
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut validity_present = 9;
    let mut validity = 9;
    let mut correlation_present = 9;
    let mut correlation = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_mac_verify(
            mac_verify,
            &raw mut present,
            &raw mut identifier,
            &raw mut validity_present,
            &raw mut validity,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!(
        (present, validity_present, validity, correlation_present),
        (0, 0, 0, 0)
    );
    kmipkit_client_batch_item_response_release(mac_verify);
}

#[test]
fn ffi_sign_response_getters_do_not_report_failed_payloads_as_present() {
    let sign = failed_cryptographic_item(
        ClientRequest::Sign(SignRequest::new().with_data(OperationData::ByteString(
            SecretBytes::new(b"failure-test-data".to_vec()),
        ))),
        0x21,
        b"sign-failure",
    );
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut data_present = 9;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 9;
    let mut correlation = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_sign(
            sign,
            &raw mut present,
            &raw mut identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!((present, data_present, correlation_present), (0, 0, 0));
    kmipkit_client_batch_item_response_release(sign);
}

#[test]
fn ffi_signature_verify_response_getters_do_not_report_failed_payloads_as_present() {
    let signature_verify = failed_cryptographic_item(
        ClientRequest::SignatureVerify(
            SignatureVerifyRequest::new()
                .with_signature_data(SecretBytes::new(b"failure-test-signature".to_vec())),
        ),
        0x22,
        b"signature-verify-failure",
    );
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut validity_present = 9;
    let mut validity = 9;
    let mut recovered_present = 9;
    let mut recovered = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    let mut correlation_present = 9;
    let mut correlation = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_signature_verify(
            signature_verify,
            &raw mut present,
            &raw mut identifier,
            &raw mut validity_present,
            &raw mut validity,
            &raw mut recovered_present,
            &raw mut recovered,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!(
        (
            present,
            validity_present,
            validity,
            recovered_present,
            correlation_present
        ),
        (0, 0, 0, 0, 0)
    );
    kmipkit_client_batch_item_response_release(signature_verify);
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

fn create_key_pair_success_response() -> Result<ClientBatchResponse, ClientError> {
    let response = ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 2),
                    ttlv_item(0x0042_0093, 0x08, b"key-pair"),
                    ttlv_enumeration(0x0042_007f, 0),
                    ttlv_structure(
                        0x0042_007c,
                        [
                            ttlv_integer(0x0042_0066, -17),
                            ttlv_item(0x0042_006f, 0x07, b"public-key-abi-test"),
                        ],
                    ),
                ],
            ),
        ],
    );
    execute_single_abi_response(
        ClientRequest::CreateKeyPair(CreateKeyPairRequest::new()),
        b"key-pair",
        response,
    )
}

fn create_split_key_success_response() -> Result<ClientBatchResponse, ClientError> {
    let response = ttlv_structure(
        0x0042_007b,
        [
            ttlv_structure(
                0x0042_007a,
                [
                    ttlv_structure(
                        0x0042_0069,
                        [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
                    ),
                    ttlv_datetime(0x0042_0092, 1),
                    ttlv_integer(0x0042_000d, 1),
                ],
            ),
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 40),
                    ttlv_item(0x0042_0093, 0x08, b"split-key"),
                    ttlv_enumeration(0x0042_007f, 0),
                    ttlv_structure(
                        0x0042_007c,
                        [
                            ttlv_integer(0x0042_0094, 101),
                            ttlv_item(0x0042_0094, 0x07, b"split-part-two"),
                        ],
                    ),
                ],
            ),
        ],
    );
    execute_single_abi_response(
        ClientRequest::CreateSplitKey(CreateSplitKeyRequest::new(
            ObjectType::from_raw(7),
            2,
            1,
            SplitKeyMethod::XOR,
            AttributeSet::new(),
        )),
        b"split-key",
        response,
    )
}

fn execute_single_abi_response(
    request: ClientRequest,
    batch_item_id: &[u8],
    response: Vec<u8>,
) -> Result<ClientBatchResponse, ClientError> {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI generation succeeds");
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback binds");
    let port = listener
        .local_addr()
        .expect("loopback has an address")
        .port();
    listener
        .set_nonblocking(true)
        .expect("test listener accepts with a bounded wait");
    let server_config = tls13_mtls_server_config(&pki);
    let peer = thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let (stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) => panic!("accepting the client connection: {error}"),
            }
        };
        stream
            .set_nonblocking(false)
            .expect("accepted test stream uses blocking reads");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("peer read timeout is set");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("peer write timeout is set");
        let connection = ServerConnection::new(server_config).expect("TLS 1.3 connection builds");
        let mut tls = StreamOwned::new(connection, stream);
        receive_ttlv_frame(&mut tls).expect("the typed request frame is bounded");
        tls.write_all(&response)
            .expect("Create Key Pair response is written");
        tls.flush().expect("the response is flushed");
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
    let mut client = Client::new(configuration, transport).expect("client construction succeeds");
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(request).with_unique_batch_item_id(batch_item_id.to_vec())
    ]);
    let result = client.execute(batch, &CodecLimits::defaults());
    peer.join().expect("controlled local peer completes");
    result
}

fn client_with_one_response(response: Vec<u8>) -> (Client, thread::JoinHandle<()>) {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI generation succeeds");
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("loopback binds");
    let port = listener
        .local_addr()
        .expect("loopback has an address")
        .port();
    listener
        .set_nonblocking(true)
        .expect("test listener accepts with a bounded wait");
    let server_config = tls13_mtls_server_config(&pki);
    let peer = thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let (stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => return,
                Err(error) => panic!("accepting the client connection: {error}"),
            }
        };
        stream
            .set_nonblocking(false)
            .expect("accepted test stream uses blocking reads");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("peer read timeout is set");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("peer write timeout is set");
        let connection = ServerConnection::new(server_config).expect("TLS 1.3 connection builds");
        let mut tls = StreamOwned::new(connection, stream);
        receive_ttlv_frame(&mut tls).expect("the typed request frame is bounded");
        tls.write_all(&response)
            .expect("the scripted response is written");
        tls.flush().expect("the response is flushed");
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
    let client = Client::new(configuration, transport).expect("client construction succeeds");
    (client, peer)
}

fn empty_unique_identifier_output_v1() -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("unique identifier size fits ABI field"),
        abi_version: 1,
        kind: 0,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: 0,
    }
}

#[test]
fn ffi_unique_identifier_writer_covers_valid_variants_and_rejects_bad_output_records() {
    assert_eq!(
        Err(ERROR_INVALID_INPUT),
        write_unique_identifier_v1(ptr::null_mut(), None)
    );

    let mut wrong_size = empty_unique_identifier_output_v1();
    wrong_size.struct_size -= 1;
    assert_eq!(
        Err(ERROR_INVALID_INPUT),
        write_unique_identifier_v1(&raw mut wrong_size, None)
    );

    let mut wrong_version = empty_unique_identifier_output_v1();
    wrong_version.abi_version += 1;
    assert_eq!(
        Err(ERROR_INVALID_INPUT),
        write_unique_identifier_v1(&raw mut wrong_version, None)
    );

    let text_identifier = UniqueIdentifier::TextString(String::from("response-identifier"));
    let UniqueIdentifier::TextString(text) = &text_identifier else {
        panic!("test identifier uses the text-string variant");
    };
    let mut text_output = empty_unique_identifier_output_v1();
    assert_eq!(
        Ok(()),
        write_unique_identifier_v1(&raw mut text_output, Some(&text_identifier))
    );
    assert_eq!(
        text_output.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(text_output.text_data, text.as_ptr());
    assert_eq!(
        text_output.text_length,
        u64::try_from(text.len()).expect("test identifier length fits the ABI")
    );

    let mut enumeration_output = empty_unique_identifier_output_v1();
    assert_eq!(
        Ok(()),
        write_unique_identifier_v1(
            &raw mut enumeration_output,
            Some(&UniqueIdentifier::Enumeration(u32::MAX))
        )
    );
    assert_eq!(
        enumeration_output.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION
    );
    assert_eq!(enumeration_output.enumeration_value, u32::MAX);

    let mut integer_output = empty_unique_identifier_output_v1();
    assert_eq!(
        Ok(()),
        write_unique_identifier_v1(
            &raw mut integer_output,
            Some(&UniqueIdentifier::Integer(i32::MIN))
        )
    );
    assert_eq!(
        integer_output.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER
    );
    assert_eq!(integer_output.integer_value, i32::MIN);
}

#[test]
fn ffi_borrowed_span_writer_preserves_absence_and_empty_presence() {
    let mut present = 9;
    let mut output = byte_span(b"stale");
    assert_eq!(Ok(()), set_borrowed_span(&mut present, &mut output, None));
    assert_eq!(present, 0);
    assert!(output.data.is_null());
    assert_eq!(output.length, 0);

    assert_eq!(
        Ok(()),
        set_borrowed_span(&mut present, &mut output, Some(&[]))
    );
    assert_eq!(present, 1);
    assert!(output.data.is_null());
    assert_eq!(output.length, 0);

    let bytes = [0x41, 0x00, 0x42];
    assert_eq!(
        Ok(()),
        set_borrowed_span(&mut present, &mut output, Some(&bytes))
    );
    assert_eq!(present, 1);
    assert_eq!(output.data, bytes.as_ptr());
    assert_eq!(
        output.length,
        u64::try_from(bytes.len()).expect("test span length fits the ABI")
    );
}

#[test]
fn ffi_create_key_pair_getter_copies_successful_typed_identifiers() {
    let response = create_key_pair_success_response()
        .expect("the valid successful typed response is associated with its request");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );

    let mut key_pair_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &mut key_pair_item),
        SUCCESS
    );
    let mut present = 0;
    let mut private_identifier = empty_unique_identifier_output_v1();
    let mut public_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_key_pair(
            key_pair_item,
            &mut present,
            &mut private_identifier,
            &mut public_identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        private_identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER
    );
    assert_eq!(private_identifier.integer_value, -17);
    assert_eq!(
        public_identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(
        public_identifier.text_length,
        u64::try_from(b"public-key-abi-test".len()).expect("test identifier length fits u64")
    );
    // SAFETY: the live response-item handle retains the Arc-backed response containing this text.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                public_identifier.text_data,
                usize::try_from(public_identifier.text_length)
                    .expect("response identifier length fits this test target"),
            )
        },
        b"public-key-abi-test"
    );

    kmipkit_client_batch_item_response_release(key_pair_item);
    kmipkit_client_batch_response_release(response_handle);
}

#[test]
fn ffi_get_usage_allocation_getter_rejects_a_missing_output_record() {
    let mut present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation(
            ptr::null_mut(),
            &mut present,
            ptr::null_mut(),
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 9);
}

fn derive_key_response_request() -> ClientRequest {
    let length = Item::new(
        RawTag::new(0x0042_002a)
            .expect("Cryptographic Length fits a KMIP tag")
            .try_checked()
            .expect("Cryptographic Length is allocated"),
        Value::integer(256),
    )
    .expect("Cryptographic Length is a valid TTLV item");
    let attributes = AttributeSet::try_new([length])
        .expect("the Derive Key output length uses a catalogued attribute");
    ClientRequest::DeriveKey(DeriveKeyRequest::new(
        ObjectType::from_raw(4),
        vec![
            UniqueIdentifier::TextString("source-one".to_owned()),
            UniqueIdentifier::TextString("source-one".to_owned()),
        ],
        DerivationMethod::from_raw(0xfeed_beef),
        DerivationParameters::new(),
        attributes,
    ))
}

#[test]
fn ffi_derive_key_response_getter_returns_success_identifier_and_clears_failure() {
    let output_identifier = b"derived-key";
    let successful_item = successful_cryptographic_item(
        derive_key_response_request(),
        5,
        b"derive-key-success",
        vec![ttlv_item(0x0042_0094, 0x07, output_identifier)],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut undersized_identifier = empty_unique_identifier_output_v1();
    undersized_identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key(
            successful_item,
            &mut present,
            &mut undersized_identifier,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key(
            successful_item,
            &mut present,
            &mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_response_identifier(&identifier, output_identifier);
    kmipkit_client_batch_item_response_release(successful_item);

    let failed_item =
        failed_cryptographic_item(derive_key_response_request(), 5, b"derive-key-failure");
    present = 9;
    identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = -17;
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key(failed_item, &mut present, &mut identifier,),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert_eq!(identifier.integer_value, 0);
    kmipkit_client_batch_item_response_release(failed_item);
}

#[test]
fn ffi_derive_key_response_ttlv_returns_owned_success_copy_only() {
    let successful_item = successful_cryptographic_item(
        derive_key_response_request(),
        5,
        b"derive-key-ttlv-success",
        vec![ttlv_item(0x0042_0094, 0x07, b"derived-key")],
    );
    let mut present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key_ttlv(
            successful_item,
            &mut present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(successful_item);
    let mut response_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(response_ttlv, &mut response_view),
        SUCCESS
    );
    let mut count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(response_view, &mut count),
        SUCCESS
    );
    assert_eq!(count, 4);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);

    let failed_item =
        failed_cryptographic_item(derive_key_response_request(), 5, b"derive-key-ttlv-failure");
    present = 9;
    response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key_ttlv(
            failed_item,
            &mut present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(failed_item);
}

#[test]
fn ffi_export_response_preserves_success_fields_and_owned_ttlv() {
    let attributes = ttlv_structure(0x0042_0125, [ttlv_enumeration(0x0042_001d, 0xf123_4567)]);
    let any_object = ttlv_structure(
        0x0042_008f,
        [ttlv_structure(
            0x0042_0040,
            [ttlv_item(0x0042_0045, 0x08, b"export-secret-material")],
        )],
    );
    let successful_item = successful_cryptographic_item(
        ClientRequest::export(ExportRequest::new()),
        0x2b,
        b"export-success",
        vec![
            ttlv_enumeration(0x0042_0057, 0xfedc_ba98),
            ttlv_item(0x0042_0094, 0x07, b"export-secret-id"),
            attributes,
            any_object,
        ],
    );
    let mut present = 0;
    let mut payload_present = 0;
    let mut object_type = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut attributes = ptr::null_mut();
    let mut any_object = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export(
            successful_item,
            &mut present,
            &mut payload_present,
            &mut object_type,
            &mut identifier,
            &mut attributes,
            &mut any_object,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, object_type), (1, 1, 0xfedc_ba98));
    assert_response_identifier(&identifier, b"export-secret-id");
    assert!(!attributes.is_null());
    assert!(!any_object.is_null());
    kmipkit_client_batch_item_response_release(successful_item);

    let mut attributes_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(attributes, &mut attributes_view),
        SUCCESS
    );
    let mut attributes_count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(attributes_view, &mut attributes_count),
        SUCCESS
    );
    assert_eq!(attributes_count, 1);
    kmipkit_ttlv_structure_view_release(attributes_view);
    kmipkit_ttlv_structure_release(attributes);

    let mut any_object_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(any_object, &mut any_object_view),
        SUCCESS
    );
    let mut any_object_count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(any_object_view, &mut any_object_count),
        SUCCESS
    );
    assert_eq!(any_object_count, 1);
    kmipkit_ttlv_structure_view_release(any_object_view);
    kmipkit_ttlv_structure_release(any_object);
}

#[test]
fn ffi_export_failure_has_no_typed_payload_and_preserves_response_ttlv() {
    let failed_item = failed_cryptographic_item(
        ClientRequest::export(ExportRequest::new()),
        0x2b,
        b"export-failure",
    );
    let mut present = 9;
    let mut payload_present = 9;
    let mut object_type = u32::MAX;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut attributes = ptr::dangling_mut();
    let mut any_object = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export(
            failed_item,
            &mut present,
            &mut payload_present,
            &mut object_type,
            &mut identifier,
            &mut attributes,
            &mut any_object,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, object_type), (1, 0, 0));
    assert_eq!(identifier.kind, 0);
    assert!(attributes.is_null());
    assert!(any_object.is_null());

    let mut response_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export_ttlv(
            failed_item,
            &mut response_present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(response_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(failed_item);
    let mut response_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(response_ttlv, &mut response_view),
        SUCCESS
    );
    let mut response_count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(response_view, &mut response_count),
        SUCCESS
    );
    assert_eq!(response_count, 4);
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
}

#[test]
fn ffi_derive_key_constructor_rejects_absent_source_identifier() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let identifier = empty_unique_identifier_output_v1();
    let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();

    assert_eq!(
        kmipkit_client_batch_item_derive_key(
            7,
            &raw const identifier,
            1,
            1,
            parameters,
            attributes,
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    release(parameters, Kind::Structure);
    release(attributes, Kind::Structure);
}

#[test]
fn ffi_derive_key_constructor_requires_attributes() {
    let parameters = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION;
    identifier.enumeration_value = 7;
    let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();

    assert_eq!(
        kmipkit_client_batch_item_derive_key(
            7,
            &raw const identifier,
            1,
            1,
            parameters,
            ptr::null_mut(),
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    release(parameters, Kind::Structure);
}

#[test]
fn ffi_export_request_accepts_all_optional_selectors_omitted() {
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_export(ptr::null(), 0, 0, 0, 0, 0, 0, ptr::null_mut(), &mut item,),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);

    for (has_key_format, has_key_wrap, has_key_compression) in [(2, 0, 0), (0, 2, 0), (0, 0, 2)] {
        let mut rejected_item = ptr::dangling_mut();
        assert_eq!(
            kmipkit_client_batch_item_export(
                ptr::null(),
                has_key_format,
                0,
                has_key_wrap,
                0,
                has_key_compression,
                0,
                ptr::null_mut(),
                &mut rejected_item,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(rejected_item.is_null());
    }
}

#[test]
fn ffi_get_constructor_preserves_table_220_optional_values_and_omission() {
    // OASIS KMIP Specification v2.1 §6.1.19, Table 220.
    let identifier_bytes = b"get-key";
    let mut identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = identifier_bytes.as_ptr();
    identifier.text_length = u64::try_from(identifier_bytes.len()).expect("test id fits u64");
    let wrapping_specification =
        make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_get(
            &raw const identifier,
            1,
            0xf123_4567,
            1,
            0xf123_4568,
            1,
            0xf123_4569,
            wrapping_specification,
            &raw mut item,
        ),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("Get item handle is registered");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::Get(request) = batch_item.request() else {
        panic!("the constructor returns a typed Get request");
    };
    assert_eq!(
        request.unique_identifier(),
        Some(&UniqueIdentifier::TextString("get-key".into()))
    );
    assert_eq!(
        request
            .key_format_type()
            .map(kmipkit_protocol::KeyFormatType::raw),
        Some(0xf123_4567)
    );
    assert_eq!(
        request
            .key_wrap_type()
            .map(kmipkit_protocol::KeyWrapType::raw),
        Some(0xf123_4568)
    );
    assert_eq!(
        request
            .key_compression_type()
            .map(kmipkit_protocol::KeyCompressionType::raw),
        Some(0xf123_4569)
    );
    assert!(request.key_wrapping_specification().is_some());
    kmipkit_client_batch_item_release(item);
    release(wrapping_specification, Kind::Structure);

    item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_get(
            ptr::null(),
            0,
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            &raw mut item,
        ),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("omitted Get item is registered");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::Get(request) = batch_item.request() else {
        panic!("the constructor returns a typed Get request");
    };
    assert!(request.unique_identifier().is_none());
    assert!(request.key_format_type().is_none());
    assert!(request.key_wrap_type().is_none());
    assert!(request.key_compression_type().is_none());
    assert!(request.key_wrapping_specification().is_none());
    kmipkit_client_batch_item_release(item);
}

#[test]
fn ffi_get_constructor_rejects_non_boolean_presence_flags() {
    for (has_key_format, has_key_wrap, has_key_compression) in [(2, 0, 0), (0, 2, 0), (0, 0, 2)] {
        let mut item = ptr::dangling_mut();
        assert_eq!(
            kmipkit_client_batch_item_get(
                ptr::null(),
                has_key_format,
                0,
                has_key_wrap,
                0,
                has_key_compression,
                0,
                ptr::null_mut(),
                &raw mut item,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(item.is_null());
    }
}

#[test]
fn ffi_locate_constructor_preserves_empty_attributes_and_raw_table_247_values() {
    // OASIS KMIP Specification v2.1 §6.1.28, Table 247.
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_locate(
            1,
            -4,
            1,
            0,
            1,
            0x8000_0001,
            1,
            0xf123_4567,
            attributes,
            &raw mut item,
        ),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("Locate item handle is registered");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::Locate(request) = batch_item.request() else {
        panic!("the constructor returns a typed Locate request");
    };
    assert_eq!(request.maximum_items(), Some(-4));
    assert_eq!(request.offset_items(), Some(0));
    assert_eq!(
        request.storage_status_mask(),
        Some(StorageStatusMask::from_raw(0x8000_0001))
    );
    assert_eq!(
        request.object_group_member(),
        Some(ObjectGroupMember::from_raw(0xf123_4567))
    );
    assert!(request.attributes().is_empty());
    kmipkit_client_batch_item_release(item);
    release(attributes, Kind::Structure);

    let mut rejected = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_locate(
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            &raw mut rejected,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(rejected.is_null());
}

#[test]
fn ffi_locate_constructor_rejects_non_boolean_flags_and_preserves_omitted_options() {
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    for (has_maximum, has_offset, has_storage_status, has_group_member) in
        [(2, 0, 0, 0), (0, 2, 0, 0), (0, 0, 2, 0), (0, 0, 0, 2)]
    {
        let mut item = ptr::dangling_mut();
        assert_eq!(
            kmipkit_client_batch_item_locate(
                has_maximum,
                0,
                has_offset,
                0,
                has_storage_status,
                0,
                has_group_member,
                0,
                attributes,
                &raw mut item,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(item.is_null());
    }

    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_locate(0, 0, 0, 0, 0, 0, 0, 0, attributes, &raw mut item,),
        SUCCESS
    );
    let handle =
        reference_handle(item, Kind::BatchItem).expect("omitted Locate item is registered");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::Locate(request) = batch_item.request() else {
        panic!("the constructor returns a typed Locate request");
    };
    assert!(request.maximum_items().is_none());
    assert!(request.offset_items().is_none());
    assert!(request.storage_status_mask().is_none());
    assert!(request.object_group_member().is_none());
    assert!(request.attributes().is_empty());
    kmipkit_client_batch_item_release(item);
    release(attributes, Kind::Structure);
}

#[test]
fn ffi_get_response_copies_the_secret_any_object() {
    // OASIS KMIP Specification v2.1 §6.1.19, Tables 221–222.
    let any_object = ttlv_structure(
        0x0042_008f,
        [ttlv_structure(
            0x0042_0040,
            [ttlv_item(0x0042_0045, 0x08, b"get-secret-material")],
        )],
    );
    let item = successful_cryptographic_item(
        ClientRequest::Get(GetRequest::new()),
        0x0a,
        b"get-success",
        vec![
            ttlv_enumeration(0x0042_0057, 0xf123_4567),
            ttlv_item(0x0042_0094, 0x07, b"get-secret-id"),
            any_object,
        ],
    );
    let mut present = 0;
    let mut payload_present = 0;
    let mut object_type = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut object = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get(
            item,
            &raw mut present,
            &raw mut payload_present,
            &raw mut object_type,
            &raw mut identifier,
            &raw mut object,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, object_type), (1, 1, 0xf123_4567));
    assert_response_identifier(&identifier, b"get-secret-id");
    assert!(!object.is_null());

    kmipkit_client_batch_item_response_release(item);

    let mut object_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(object, &raw mut object_view),
        SUCCESS
    );
    let mut object_count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(object_view, &raw mut object_count),
        SUCCESS
    );
    assert_eq!(object_count, 1);
    kmipkit_ttlv_structure_view_release(object_view);

    kmipkit_ttlv_structure_release(object);
}

#[test]
fn ffi_locate_response_preserves_repeated_identifiers_and_empty_success_payloads() {
    // OASIS KMIP Specification v2.1 §6.1.28, Tables 248–249.
    let request =
        LocateRequest::new(AttributeSet::try_new(Vec::new()).expect("empty criteria are valid"));
    let item = successful_cryptographic_item(
        ClientRequest::Locate(request),
        0x08,
        b"locate-success",
        vec![
            ttlv_integer(0x0042_00d5, 3),
            ttlv_item(0x0042_0094, 0x07, b"first"),
            ttlv_item(0x0042_0094, 0x07, b"second"),
            ttlv_item(0x0042_0094, 0x07, b"first"),
        ],
    );
    let mut present = 0;
    let mut payload_present = 0;
    let mut located_items_present = 0;
    let mut located_items = i32::MIN;
    let mut count = u64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_locate(
            item,
            &raw mut present,
            &raw mut payload_present,
            &raw mut located_items_present,
            &raw mut located_items,
            &raw mut count,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, located_items_present), (1, 1, 1));
    assert_eq!((located_items, count), (3, 3));
    for (index, expected) in [b"first".as_slice(), b"second", b"first"]
        .into_iter()
        .enumerate()
    {
        let mut identifier = empty_unique_identifier_output_v1();
        assert_eq!(
            kmipkit_client_batch_item_response_locate_unique_identifier_at(
                item,
                u64::try_from(index).expect("test index fits u64"),
                &raw mut identifier,
            ),
            SUCCESS
        );
        assert_response_identifier(&identifier, expected);
    }
    let mut out_of_range = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_locate_unique_identifier_at(
            item,
            3,
            &raw mut out_of_range,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(out_of_range.kind, 0);
    assert!(out_of_range.text_data.is_null());
    kmipkit_client_batch_item_response_release(item);

    let empty_request =
        LocateRequest::new(AttributeSet::try_new(Vec::new()).expect("empty criteria are valid"));
    let empty_item = successful_cryptographic_item(
        ClientRequest::Locate(empty_request),
        0x08,
        b"locate-empty",
        Vec::new(),
    );
    present = 0;
    payload_present = 0;
    located_items_present = 0;
    located_items = i32::MIN;
    count = u64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_locate(
            empty_item,
            &raw mut present,
            &raw mut payload_present,
            &raw mut located_items_present,
            &raw mut located_items,
            &raw mut count,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, located_items_present), (1, 1, 0));
    assert_eq!((located_items, count), (0, 0));
    kmipkit_client_batch_item_response_release(empty_item);
}

#[test]
fn ffi_get_response_clears_payload_for_failure_and_wrong_operation() {
    let failed =
        failed_cryptographic_item(ClientRequest::Get(GetRequest::new()), 0x0a, b"get-failed");
    let mut present = u8::MAX;
    let mut payload_present = u8::MAX;
    let mut object_type = u32::MAX;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut object = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get(
            failed,
            &raw mut present,
            &raw mut payload_present,
            &raw mut object_type,
            &raw mut identifier,
            &raw mut object,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, object_type), (1, 0, 0));
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());
    assert!(object.is_null());
    kmipkit_client_batch_item_response_release(failed);

    let request = LocateRequest::new(AttributeSet::try_new(Vec::new()).expect("empty criteria"));
    let locate = successful_cryptographic_item(
        ClientRequest::Locate(request),
        0x08,
        b"locate-for-get",
        Vec::new(),
    );
    present = u8::MAX;
    payload_present = u8::MAX;
    object_type = u32::MAX;
    identifier = empty_unique_identifier_output_v1();
    object = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get(
            locate,
            &raw mut present,
            &raw mut payload_present,
            &raw mut object_type,
            &raw mut identifier,
            &raw mut object,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, object_type), (0, 0, 0));
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());
    assert!(object.is_null());
    kmipkit_client_batch_item_response_release(locate);
}

#[test]
fn ffi_locate_response_clears_payload_for_failure_and_wrong_operation() {
    let failed = failed_cryptographic_item(
        ClientRequest::Locate(LocateRequest::new(
            AttributeSet::try_new(Vec::new()).expect("empty criteria"),
        )),
        0x08,
        b"locate-failed",
    );
    let mut present = u8::MAX;
    let mut payload_present = u8::MAX;
    let mut located_items_present = u8::MAX;
    let mut located_items = i32::MIN;
    let mut identifier_count = u64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_locate(
            failed,
            &raw mut present,
            &raw mut payload_present,
            &raw mut located_items_present,
            &raw mut located_items,
            &raw mut identifier_count,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, located_items_present), (1, 0, 0));
    assert_eq!((located_items, identifier_count), (0, 0));
    kmipkit_client_batch_item_response_release(failed);

    let get = successful_cryptographic_item(
        ClientRequest::Get(GetRequest::new()),
        0x0a,
        b"get-for-locate",
        vec![
            ttlv_enumeration(0x0042_0057, 1),
            ttlv_item(0x0042_0094, 0x07, b"object-id"),
            ttlv_structure(0x0042_008f, []),
        ],
    );
    present = u8::MAX;
    payload_present = u8::MAX;
    located_items_present = u8::MAX;
    located_items = i32::MIN;
    identifier_count = u64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_locate(
            get,
            &raw mut present,
            &raw mut payload_present,
            &raw mut located_items_present,
            &raw mut located_items,
            &raw mut identifier_count,
        ),
        SUCCESS
    );
    assert_eq!((present, payload_present, located_items_present), (0, 0, 0));
    assert_eq!((located_items, identifier_count), (0, 0));
    let mut wrong_operation_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_locate_unique_identifier_at(
            get,
            0,
            &raw mut wrong_operation_identifier,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(wrong_operation_identifier.kind, 0);
    assert!(wrong_operation_identifier.text_data.is_null());
    kmipkit_client_batch_item_response_release(get);
}

#[test]
fn ffi_export_response_getters_reject_inconsistent_internal_handles() {
    let wrong_variant: *mut kmipkit_client_batch_item_response_t = try_make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::Structure(Structure::new()),
    )
    .expect("test response handle is registered");
    let mut present = 9;
    let mut payload_present = 9;
    let mut object_type = u32::MAX;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut attributes = ptr::dangling_mut();
    let mut any_object = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export(
            wrong_variant,
            &mut present,
            &mut payload_present,
            &mut object_type,
            &mut identifier,
            &mut attributes,
            &mut any_object,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!((present, payload_present, object_type), (0, 0, 0));
    assert!(attributes.is_null());
    assert!(any_object.is_null());

    present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export_ttlv(
            wrong_variant,
            &mut present,
            &mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(wrong_variant);

    let valid_item = failed_cryptographic_item(
        ClientRequest::export(ExportRequest::new()),
        0x2b,
        b"export-invalid-index",
    );
    let valid_item_handle =
        reference_handle(valid_item, Kind::ClientBatchItemResponse).expect("test response is live");
    let HandleValue::ClientBatchItemResponse { response, .. } = &valid_item_handle.value else {
        panic!("test response uses the client batch response variant");
    };
    let invalid_index: *mut kmipkit_client_batch_item_response_t = try_make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(response),
            index: usize::MAX,
        },
    )
    .expect("test response with an invalid index is registered");

    present = 9;
    payload_present = 9;
    object_type = u32::MAX;
    identifier = empty_unique_identifier_output_v1();
    attributes = ptr::dangling_mut();
    any_object = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export(
            invalid_index,
            &mut present,
            &mut payload_present,
            &mut object_type,
            &mut identifier,
            &mut attributes,
            &mut any_object,
        ),
        ERROR_INTERNAL
    );
    assert_eq!((present, payload_present, object_type), (0, 0, 0));
    assert!(attributes.is_null());
    assert!(any_object.is_null());

    present = 9;
    response_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_export_ttlv(
            invalid_index,
            &mut present,
            &mut response_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());

    kmipkit_client_batch_item_response_release(invalid_index);
    kmipkit_client_batch_item_response_release(valid_item);
}

#[test]
fn ffi_get_usage_allocation_getter_borrows_successful_identifier_and_clears_failure() {
    let response_identifier = b"allocation-object";
    let successful_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"usage-allocation",
        vec![ttlv_item(0x0042_0094, 0x07, response_identifier)],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut undersized_identifier = empty_unique_identifier_output_v1();
    undersized_identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation(
            successful_item,
            &mut present,
            &mut undersized_identifier,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation(
            successful_item,
            &mut present,
            &mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(
        identifier.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(
        identifier.text_length,
        u64::try_from(response_identifier.len()).expect("identifier length fits ABI")
    );
    // SAFETY: the open item handle retains the Arc-backed typed response containing this text.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                identifier.text_data,
                usize::try_from(identifier.text_length)
                    .expect("identifier length fits this test target"),
            )
        },
        response_identifier
    );
    kmipkit_client_batch_item_response_release(successful_item);

    let failed_item = failed_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"usage-allocation-failure",
    );
    present = 9;
    identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = -17;
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation(
            failed_item,
            &mut present,
            &mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert_eq!(identifier.integer_value, 0);
    kmipkit_client_batch_item_response_release(failed_item);
}

#[test]
fn ffi_get_usage_allocation_ttlv_copies_success_and_excludes_failure_responses() {
    let successful_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"usage-allocation",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    let mut present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            successful_item,
            &mut present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    let failed_item = failed_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"usage-allocation-failure",
    );
    present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            failed_item,
            &mut present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(failed_item);
}

fn import_test_request() -> ClientRequest {
    let object = Item::new(
        RawTag::new(0x0042_008f)
            .expect("Symmetric Key tag fits the KMIP tag field")
            .try_checked()
            .expect("Symmetric Key is allocated in KMIP 2.1"),
        Value::structure(Structure::new()),
    )
    .expect("valid generic Any Object test item builds");
    ClientRequest::import(ImportRequest::new(
        UniqueIdentifier::TextString("request-import-id".to_owned()),
        ObjectType::from_raw(u32::MAX),
        AttributeSet::new(),
        object,
    ))
}

#[test]
fn ffi_import_response_exposes_success_identifier_and_owned_ordered_ttlv_only() {
    let successful_item = successful_cryptographic_item(
        import_test_request(),
        0x2a,
        b"import-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"imported-id"),
            ttlv_enumeration(0x0054_0001, 0xf123_4567),
        ],
    );
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_import(successful_item, &mut present, &mut identifier,),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_response_identifier(&identifier, b"imported-id");

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_import_ttlv(
            successful_item,
            &mut ttlv_present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    let mut view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(response_ttlv, &mut view),
        SUCCESS
    );
    let mut count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(view, &mut count),
        SUCCESS
    );
    assert_eq!(
        count, 4,
        "the copied full response retains unknown trailing fields"
    );
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(successful_item);

    let failed_item = failed_cryptographic_item(import_test_request(), 0x2a, b"import-failure");
    present = 9;
    identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = -17;
    assert_eq!(
        kmipkit_client_batch_item_response_import(failed_item, &mut present, &mut identifier,),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert_eq!(identifier.integer_value, 0);
    ttlv_present = 9;
    response_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_import_ttlv(
            failed_item,
            &mut ttlv_present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(failed_item);
}

#[test]
fn ffi_get_constraints_ttlv_copies_success_and_excludes_failure_responses() {
    let constraints = ttlv_structure(
        0x0042_0168,
        [ttlv_item(0x0054_0001, 0x07, b"unknown-constraint")],
    );
    let success = execute_single_abi_response(
        ClientRequest::get_constraints(),
        b"get-constraints-success",
        successful_cryptographic_response(0x38, b"get-constraints-success", vec![constraints]),
    )
    .expect("the successful Constraints response is accepted with unknown members");
    let success_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(success)),
    );
    let mut successful_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(success_handle, 0, &raw mut successful_item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(success_handle);

    let mut present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_constraints_ttlv(
            successful_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(successful_item);

    let failed_item = failed_cryptographic_item(
        ClientRequest::get_constraints(),
        0x38,
        b"get-constraints-failure",
    );
    present = 9;
    response_ttlv = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_constraints_ttlv(
            failed_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(failed_item);
}

#[test]
fn ffi_get_constraints_ttlv_getter_rejects_inconsistent_internal_handles() {
    let response = Arc::new(
        create_key_pair_success_response()
            .expect("the successful key-pair response is associated with its request"),
    );
    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_constraints_ttlv(
            mismatched_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(mismatched_item);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response,
            index: usize::MAX,
        },
    );
    present = 9;
    response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_get_constraints_ttlv(
            invalid_index_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(invalid_index_item);
}

#[test]
fn ffi_usage_allocation_ttlv_getter_fails_closed_for_invalid_and_unrelated_items() {
    let response = Arc::new(
        create_key_pair_success_response()
            .expect("the successful key-pair response is associated with its request"),
    );
    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    let mut present = 9;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            mismatched_item,
            &mut present,
            &mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(mismatched_item);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(&response),
            index: usize::MAX,
        },
    );
    present = 9;
    response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            invalid_index_item,
            &mut present,
            &mut response_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(invalid_index_item);

    let unrelated_success_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse { response, index: 0 },
    );
    present = 9;
    response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            unrelated_success_item,
            &mut present,
            &mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(unrelated_success_item);
}

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_validate_failed_check_response(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_certify_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_usage_allocation_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
}

#[test]
fn ffi_certify_ttlv_is_a_complete_ordered_caller_owned_copy_from_c() {
    let unknown_value = [0xde, 0xad, 0x00, 0xff];
    let item = successful_cryptographic_item(
        ClientRequest::Certify(
            CertifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("public-key".to_owned()))
                .with_certificate_request_type(1),
        ),
        0x06,
        b"certify-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"issued-certificate"),
            ttlv_item(0x0054_0001, 0x08, &unknown_value),
        ],
    );

    let mut present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_certify_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    // SAFETY: the compiled C consumer takes ownership of this live response-item handle.
    let result = unsafe { kmipkit_client_abi_c_consumer_validate_certify_response_ttlv(item) };

    assert_eq!(
        result, 1,
        "C can inspect response order and unknown fields after the item handle is released"
    );
}

#[test]
fn ffi_usage_allocation_ttlv_is_a_complete_ordered_caller_owned_copy() {
    let unknown_value = [0xde, 0xad, 0x00, 0xff];
    let item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"usage-allocation",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"allocation-object"),
            ttlv_item(0x0054_0001, 0x08, &unknown_value),
        ],
    );

    let mut present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    // SAFETY: the C consumer receives the live response-item handle and takes ownership of it.
    let result =
        unsafe { kmipkit_client_abi_c_consumer_validate_usage_allocation_response_ttlv(item) };

    assert_eq!(
        result, 1,
        "the copied generic response must outlive its source item"
    );
}

#[test]
fn ffi_create_split_key_getters_copy_nonempty_identifiers_and_reject_out_of_range_index() {
    let response = create_split_key_success_response()
        .expect("the valid successful split-key response is associated with its request");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &mut item),
        SUCCESS
    );

    let mut present = 0;
    let mut count = 0;
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_count(item, &mut present, &mut count),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_eq!(count, 2);

    let mut first = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            item, 0, &mut first,
        ),
        SUCCESS
    );
    assert_eq!(first.kind, crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER);
    assert_eq!(first.integer_value, 101);

    let mut second = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            item,
            1,
            &mut second,
        ),
        SUCCESS
    );
    assert_eq!(
        second.kind,
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
    );
    assert_eq!(
        second.text_length,
        u64::try_from(b"split-part-two".len()).expect("test identifier length fits u64")
    );
    // SAFETY: the live response-item handle retains the Arc-backed response containing this text.
    assert_eq!(
        unsafe {
            std::slice::from_raw_parts(
                second.text_data,
                usize::try_from(second.text_length)
                    .expect("response identifier length fits this test target"),
            )
        },
        b"split-part-two"
    );

    let mut out_of_range = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            item,
            count,
            &mut out_of_range,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(out_of_range.kind, 0);
    assert!(out_of_range.text_data.is_null());
    assert_eq!(out_of_range.text_length, 0);

    kmipkit_client_batch_item_response_release(item);
    kmipkit_client_batch_response_release(response_handle);
}

fn assert_creation_result_getters_return_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut metadata = kmipkit_client_batch_item_metadata_v2_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_client_batch_item_metadata_v2_t>())
            .expect("metadata size fits ABI field"),
        abi_version: 2,
        operation: 0,
        outcome_kind: 0,
        result_status: 0,
        has_result_reason: 0,
        result_reason: 0,
        has_result_message: 0,
        reserved: 0,
        result_message_data: ptr::null(),
        result_message_length: 0,
    };
    assert_eq!(
        kmipkit_client_batch_item_response_metadata_v2(item, &mut metadata),
        expected_status
    );

    let mut present = 0;
    let mut object_type = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create(
            item,
            &mut present,
            &mut object_type,
            &mut identifier,
        ),
        expected_status
    );

    let mut certify_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_certify(item, &mut present, &mut certify_identifier),
        expected_status
    );

    let mut private_identifier = empty_unique_identifier_output_v1();
    let mut public_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_key_pair(
            item,
            &mut present,
            &mut private_identifier,
            &mut public_identifier,
        ),
        expected_status
    );

    let mut count = 0;
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_count(item, &mut present, &mut count),
        expected_status
    );

    let mut split_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            item,
            0,
            &mut split_identifier,
        ),
        expected_status
    );

    let mut versions_present = 0;
    let mut version_count = 0;
    assert_eq!(
        kmipkit_client_batch_item_response_discover_versions_supported_versions(
            item,
            &mut versions_present,
            ptr::null_mut(),
            0,
            &mut version_count,
        ),
        expected_status
    );
}

fn assert_crypto_result_getters_return_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 0;
    let mut first_present = 0;
    let mut first_span = byte_span(&[]);
    let mut correlation_present = 0;
    let mut correlation = byte_span(&[]);
    assert_eq!(
        kmipkit_client_batch_item_response_hash(
            item,
            &raw mut present,
            &raw mut first_present,
            &raw mut first_span,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        expected_status
    );

    let mut unique_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_mac(
            item,
            &raw mut present,
            &raw mut unique_identifier,
            &raw mut first_present,
            &raw mut first_span,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        expected_status
    );
    let mut validity_present = 0;
    let mut validity = 0;
    assert_eq!(
        kmipkit_client_batch_item_response_mac_verify(
            item,
            &raw mut present,
            &raw mut unique_identifier,
            &raw mut validity_present,
            &raw mut validity,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        expected_status
    );
    assert_eq!(
        kmipkit_client_batch_item_response_sign(
            item,
            &raw mut present,
            &raw mut unique_identifier,
            &raw mut first_present,
            &raw mut first_span,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        expected_status
    );
    assert_eq!(
        kmipkit_client_batch_item_response_signature_verify(
            item,
            &raw mut present,
            &raw mut unique_identifier,
            &raw mut validity_present,
            &raw mut validity,
            &raw mut first_present,
            &raw mut first_span,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        expected_status
    );

    let mut derive_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key(
            item,
            &raw mut present,
            &raw mut derive_identifier,
        ),
        expected_status
    );
    let mut derive_response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_derive_key_ttlv(
            item,
            &raw mut present,
            &raw mut derive_response_ttlv,
        ),
        expected_status
    );
}

#[test]
fn ffi_response_getters_reject_inconsistent_response_item_handles() {
    let response = Arc::new(
        create_key_pair_success_response()
            .expect("the valid successful typed response is associated with its request"),
    );

    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    assert_creation_result_getters_return_status(mismatched_item, ERROR_INVALID_INPUT);
    assert_crypto_result_getters_return_status(mismatched_item, ERROR_INVALID_INPUT);
    assert_common_response_getters_return_status(mismatched_item, ERROR_INVALID_INPUT);
    assert_get_and_locate_response_getters_return_status(mismatched_item, ERROR_INVALID_INPUT);
    kmipkit_client_batch_item_response_release(mismatched_item);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response,
            index: usize::MAX,
        },
    );
    assert_creation_result_getters_return_status(invalid_index_item, ERROR_INTERNAL);
    assert_crypto_result_getters_return_status(invalid_index_item, ERROR_INTERNAL);
    assert_common_response_getters_return_status(invalid_index_item, ERROR_INTERNAL);
    assert_get_and_locate_response_getters_return_status(invalid_index_item, ERROR_INTERNAL);
    kmipkit_client_batch_item_response_release(invalid_index_item);
}

fn assert_common_response_getters_return_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_certify_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        expected_status
    );
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_get_usage_allocation(
            item,
            &raw mut present,
            &raw mut identifier,
        ),
        expected_status
    );
    let mut span = byte_span(&[]);
    assert_eq!(
        kmipkit_client_batch_item_response_unique_batch_item_id(
            item,
            &raw mut present,
            &raw mut span,
        ),
        expected_status
    );
    assert_eq!(
        kmipkit_client_batch_item_response_pending_correlation_value(
            item,
            &raw mut present,
            &raw mut span,
        ),
        expected_status
    );
}

fn assert_get_and_locate_response_getters_return_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut get_present = 9;
    let mut get_payload_present = 9;
    let mut get_object_type = u32::MAX;
    let mut get_identifier = empty_unique_identifier_output_v1();
    let mut get_object = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_get(
            item,
            &raw mut get_present,
            &raw mut get_payload_present,
            &raw mut get_object_type,
            &raw mut get_identifier,
            &raw mut get_object,
        ),
        expected_status
    );
    assert_eq!(
        (get_present, get_payload_present, get_object_type),
        (0, 0, 0)
    );
    assert_eq!(get_identifier.kind, 0);
    assert!(get_identifier.text_data.is_null());
    assert!(get_object.is_null());

    let mut locate_present = 9;
    let mut locate_payload_present = 9;
    let mut located_items_present = 9;
    let mut located_items = i32::MIN;
    let mut identifier_count = u64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_locate(
            item,
            &raw mut locate_present,
            &raw mut locate_payload_present,
            &raw mut located_items_present,
            &raw mut located_items,
            &raw mut identifier_count,
        ),
        expected_status
    );
    assert_eq!(
        (
            locate_present,
            locate_payload_present,
            located_items_present,
            located_items,
            identifier_count,
        ),
        (0, 0, 0, 0, 0)
    );
    let mut locate_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_locate_unique_identifier_at(
            item,
            0,
            &raw mut locate_identifier,
        ),
        expected_status
    );
    assert_eq!(locate_identifier.kind, 0);
    assert!(locate_identifier.text_data.is_null());
}

#[test]
fn ffi_create_split_key_identifier_getter_rejects_out_of_range_identifier_index() {
    let response = Arc::new(
        create_split_key_success_response()
            .expect("the split-key response contains ordered generated identifiers"),
    );
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(response),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);

    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_create_split_key_unique_identifier_at(
            item,
            u64::MAX,
            &raw mut identifier,
        ),
        ERROR_INVALID_INPUT
    );
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_certify_getter_reports_absent_result_and_rejects_invalid_output_record() {
    let response = Arc::new(
        create_key_pair_success_response()
            .expect("the valid successful typed response is associated with its request"),
    );
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &mut item),
        SUCCESS
    );

    let mut present = 1;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_certify(item, &mut present, &mut identifier),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);

    identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_certify(item, &mut present, &mut identifier),
        ERROR_INVALID_INPUT
    );

    let mut first_present = 0;
    let mut first_span = byte_span(&[]);
    let mut correlation_present = 0;
    let mut correlation = byte_span(&[]);
    identifier = empty_unique_identifier_output_v1();
    identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_mac(
            item,
            &mut present,
            &mut identifier,
            &mut first_present,
            &mut first_span,
            &mut correlation_present,
            &mut correlation,
        ),
        ERROR_INVALID_INPUT
    );

    let mut validity_present = 0;
    let mut validity = 0;
    identifier = empty_unique_identifier_output_v1();
    identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_mac_verify(
            item,
            &mut present,
            &mut identifier,
            &mut validity_present,
            &mut validity,
            &mut correlation_present,
            &mut correlation,
        ),
        ERROR_INVALID_INPUT
    );

    identifier = empty_unique_identifier_output_v1();
    identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_sign(
            item,
            &mut present,
            &mut identifier,
            &mut first_present,
            &mut first_span,
            &mut correlation_present,
            &mut correlation,
        ),
        ERROR_INVALID_INPUT
    );

    let mut recovered_present = 0;
    let mut recovered = byte_span(&[]);
    identifier = empty_unique_identifier_output_v1();
    identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_signature_verify(
            item,
            &mut present,
            &mut identifier,
            &mut validity_present,
            &mut validity,
            &mut recovered_present,
            &mut recovered,
            &mut correlation_present,
            &mut correlation,
        ),
        ERROR_INVALID_INPUT
    );

    kmipkit_client_batch_item_response_release(item);
    kmipkit_client_batch_response_release(response_handle);
}

#[test]
fn ffi_creation_requests_reject_missing_attributes_and_malformed_structure_handles() {
    let mut create_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_create(1, ptr::null_mut(), ptr::null_mut(), &mut create_item,),
        ERROR_INVALID_INPUT
    );
    assert!(create_item.is_null());

    let mut split_key_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_create_split_key(
            1,
            ptr::null(),
            2,
            2,
            1,
            0,
            ptr::null(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut split_key_item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(split_key_item.is_null());

    assert!(matches!(
        copy_structure_handle(ptr::null_mut(), true),
        Err(ERROR_INVALID_INPUT)
    ));
    assert!(matches!(
        copy_attribute_set_handle(ptr::null_mut(), true),
        Err(ERROR_INVALID_INPUT)
    ));

    let malformed_structure = make_handle::<kmipkit_ttlv_structure_t>(
        Kind::Structure,
        HandleValue::Value(Value::integer(1)),
    );
    assert!(matches!(
        copy_structure_handle(malformed_structure, false),
        Err(ERROR_INVALID_INPUT)
    ));
    assert!(matches!(
        copy_attribute_set_handle(malformed_structure, true),
        Err(ERROR_INVALID_INPUT)
    ));
    release(malformed_structure, Kind::Structure);
}

#[test]
fn ffi_attribute_set_copy_preserves_nonempty_unknown_attributes() {
    let tag = RawTag::new(0x0054_0001)
        .expect("vendor attribute test tag fits the KMIP tag field")
        .try_checked()
        .expect("vendor attribute test tag is allocated");
    let item = Item::new(tag, Value::byte_string(b"opaque-attribute".to_vec()))
        .expect("opaque attribute is a valid TTLV item");
    let mut structure = Structure::new();
    structure
        .try_push(item)
        .expect("opaque attribute fits in the test structure");
    let handle = make_handle(Kind::Structure, HandleValue::Structure(structure));

    let copied = copy_attribute_set_handle(handle, true)
        .expect("valid nonempty attribute structure copies")
        .expect("required structure produces an attribute set");

    assert_eq!(copied.len(), 1);
    assert_eq!(copied.as_items()[0].tag().raw(), 0x0054_0001);
    release(handle, Kind::Structure);
}

#[test]
fn ffi_create_key_pair_preserves_each_supplied_attribute_set() {
    let common = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let private_key = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let public_key = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::null_mut();

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_create_key_pair(
            common,
            private_key,
            public_key,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        )
    );
    let stored = reference_handle(item, Kind::BatchItem).expect("Create Key Pair item is live");
    let HandleValue::BatchItem(stored) = &stored.value else {
        panic!("Create Key Pair item contains a request");
    };
    let ClientRequest::CreateKeyPair(request) = stored.request() else {
        panic!("Create Key Pair item contains the matching operation");
    };
    assert!(request.has_common_attributes());
    assert!(request.has_private_key_attributes());
    assert!(request.has_public_key_attributes());

    release(item, Kind::BatchItem);
    release(common, Kind::Structure);
    release(private_key, Kind::Structure);
    release(public_key, Kind::Structure);
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
fn ffi_unique_batch_item_id_is_absent_when_response_omits_the_optional_identifier() {
    let header = ttlv_structure(
        0x0042_007a,
        [
            ttlv_structure(
                0x0042_0069,
                [ttlv_integer(0x0042_006a, 2), ttlv_integer(0x0042_006b, 1)],
            ),
            ttlv_datetime(0x0042_0092, 1),
            ttlv_integer(0x0042_000d, 1),
        ],
    );
    let failed_item = ttlv_structure(
        0x0042_000f,
        [
            ttlv_enumeration(0x0042_005c, 30),
            ttlv_enumeration(0x0042_007f, 1),
            ttlv_enumeration(0x0042_007e, 6),
        ],
    );
    let (mut client, peer) =
        client_with_one_response(ttlv_structure(0x0042_007b, [header, failed_item]));
    let batch = ClientBatch::from_items([ClientBatchItem::discover_versions()]);
    let response = client.execute(batch, &CodecLimits::defaults());
    peer.join()
        .expect("the local TLS peer completes the KMIP exchange");
    let response = response.expect("a failed item remains a typed batch response");
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item_handle = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &mut item_handle),
        SUCCESS
    );

    let sentinel = [0x41];
    let mut present = 1;
    let mut span = byte_span(&sentinel);
    assert_eq!(
        kmipkit_client_batch_item_response_unique_batch_item_id(
            item_handle,
            &mut present,
            &mut span,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(span.data.is_null());
    assert_eq!(span.length, 0);

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
        .with_total(TimeoutLimit::Bounded(std::time::Duration::new(
            2,
            125_000_000,
        )));
    assert_eq!(parsed, expected);

    let effective = TimeoutPolicy::default().with_overrides(&parsed);
    assert_eq!(effective.connect(), TimeoutPolicy::default().connect());
    assert_eq!(
        effective.write(),
        TimeoutLimit::Bounded(std::time::Duration::ZERO)
    );
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
    assert_eq!(
        request_options_from_v1(&input).err(),
        Some(ERROR_INVALID_INPUT)
    );

    input.write_timeout = timeout(0, 0, 0);
    input.read_timeout = timeout(0, 0, 0);
    input.total_timeout = timeout(0, 0, 0);
    input.abi_version = 2;
    assert_eq!(
        request_options_from_v1(&input).err(),
        Some(ERROR_INVALID_INPUT)
    );
}

#[test]
fn execute_with_options_invalid_options_nulls_response_output() {
    let mut response = std::ptr::NonNull::<kmipkit_client_batch_response_t>::dangling().as_ptr();

    let status = kmipkit_client_execute_with_options_v1(
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null(),
        &raw mut response,
    );

    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(
        response.is_null(),
        "error output slots must be initialized to NULL"
    );
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
    assert_config_rejected(&config, ERROR_INVALID_INPUT);
    config.abi_version = 1;

    config.struct_size -= 1;
    assert_config_rejected(&config, ERROR_INVALID_INPUT);
    config.struct_size += 1;

    config.reserved = 1;
    assert_config_rejected(&config, ERROR_INVALID_INPUT);
    config.reserved = 0;

    config.flags = u32::MAX;
    assert_config_rejected(&config, ERROR_INVALID_INPUT);
    config.flags = 0;

    config.connect_timeout = timeout(TRANSPORT_TIMEOUT_BOUNDED, 0, 1_000_000_000);
    assert_config_rejected(&config, ERROR_INVALID_INPUT);
    config.connect_timeout = timeout(0, 0, 0);

    config.client_certificate_count = MAX_TRANSPORT_CONFIG_ITEMS + 1;
    assert_config_rejected(&config, ERROR_RESOURCE_LIMIT);
    config.client_certificate_count =
        u64::try_from(certificates.len()).expect("test certificate count fits the ABI");

    config.endpoint.data = ptr::null();
    config.endpoint.length = MAX_TRANSPORT_TEXT_BYTES + 1;
    assert_config_rejected(&config, ERROR_RESOURCE_LIMIT);
    config.endpoint = byte_span(b"127.0.0.1");

    config.max_request_bytes = 0;
    assert_config_rejected(&config, ERROR_INVALID_INPUT);

    config.max_request_bytes = 1_048_576;
    config.client_certificates = ptr::null();
    config.client_certificate_count = 0;
    let mut output = std::ptr::NonNull::<kmipkit_transport_config_t>::dangling().as_ptr();
    assert_eq!(
        kmipkit_transport_config_create_v1(&config, &mut output),
        ERROR_INVALID_INPUT
    );
    assert!(output.is_null());

    config.client_certificates = certificates.as_ptr();
    config.client_certificate_count =
        u64::try_from(certificates.len()).expect("test certificate count fits the ABI");
    let invalid_root = input(INPUT_MEMORY, 99, pki.authority_certificate_der());
    config.trust_certificates = &raw const invalid_root;
    let mut output = std::ptr::NonNull::<kmipkit_transport_config_t>::dangling().as_ptr();
    assert_eq!(
        kmipkit_transport_config_create_v1(&config, &mut output),
        ERROR_INVALID_INPUT
    );
    assert!(output.is_null());

    config.trust_certificates = ptr::null();
    config.trust_certificate_count = 0;
    let mut output = std::ptr::NonNull::<kmipkit_transport_config_t>::dangling().as_ptr();
    assert_eq!(
        kmipkit_transport_config_create_v1(&config, &mut output),
        ERROR_INVALID_INPUT
    );
    assert!(output.is_null());
}

fn assert_config_rejected(config: &kmipkit_transport_config_v1_t, expected: i32) {
    assert_eq!(transport_config_from_v1(config).err(), Some(expected));
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
        HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
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
fn ffi_execute_with_options_and_error_ttlv_clears_outputs_before_options_validation() {
    let mut response = ptr::dangling_mut::<kmipkit_client_batch_response_t>();
    let mut error_response = ptr::dangling_mut::<kmipkit_ttlv_structure_t>();
    assert_eq!(
        kmipkit_client_execute_with_options_and_error_ttlv_v1(
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            &mut response,
            &mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(response.is_null());
    assert!(error_response.is_null());
}

#[test]
fn ffi_execute_with_options_and_error_ttlv_does_not_retain_non_uid_error_response() {
    const SENTINEL: &[u8] = b"KMIPKit-response-sentinel";
    let version = ttlv_structure(
        0x0042_0069,
        [ttlv_integer(0x0042_006A, 2), ttlv_integer(0x0042_006B, 1)],
    );
    let header = ttlv_structure(
        0x0042_007A,
        [
            version,
            ttlv_datetime(0x0042_0092, 1),
            ttlv_integer(0x0042_000D, 1),
        ],
    );
    let payload = ttlv_structure(
        0x0042_007C,
        [
            ttlv_item(0x0042_0069, 0x08, SENTINEL),
            ttlv_item(0x0054_0001, 0x08, SENTINEL),
        ],
    );
    let response_item = ttlv_structure(
        0x0042_000F,
        [
            ttlv_enumeration(0x0042_005C, 30),
            ttlv_enumeration(0x0042_007F, 0),
            payload,
        ],
    );
    let response_message = ttlv_structure(0x0042_007B, [header, response_item]);
    let (client, peer) = client_with_one_response(response_message);
    let client = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(std::sync::Mutex::new(client))),
    );
    let limits = default_codec_limits_handle();
    let mut batch = ptr::null_mut();
    assert_eq!(kmipkit_client_batch_create(&mut batch), SUCCESS);
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_discover_versions(&mut item),
        SUCCESS
    );
    assert_eq!(kmipkit_client_batch_add_item(batch, &mut item), SUCCESS);
    let options = request_options(
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );
    let mut response = ptr::null_mut();
    let mut error_response = ptr::null_mut();

    assert_eq!(
        kmipkit_client_execute_with_options_and_error_ttlv_v1(
            client,
            &mut batch,
            limits,
            &options,
            &mut response,
            &mut error_response,
        ),
        ERROR_PROTOCOL
    );
    assert!(batch.is_null());
    assert!(response.is_null());
    assert!(error_response.is_null());
    kmipkit_client_batch_response_release(response);
    kmipkit_client_batch_release(batch);
    kmipkit_client_release(client);
    kmipkit_codec_limits_release(limits);
    peer.join().expect("the controlled TLS peer completes");
}

#[test]
fn batch_add_item_rejects_null_slots_and_released_items_without_consuming_the_batch() {
    let mut batch = ptr::null_mut();
    assert_eq!(kmipkit_client_batch_create(&mut batch), SUCCESS);
    assert!(!batch.is_null());

    assert_eq!(
        kmipkit_client_batch_add_item(batch, ptr::null_mut()),
        ERROR_INVALID_INPUT
    );
    let mut empty_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_add_item(batch, &mut empty_item),
        ERROR_INVALID_INPUT
    );
    assert!(empty_item.is_null());

    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_discover_versions(&mut item),
        SUCCESS
    );
    let released_item = item;
    kmipkit_client_batch_item_release(item);
    assert_eq!(
        kmipkit_client_batch_add_item(batch, &mut item),
        ERROR_INVALID_INPUT
    );
    assert_eq!(item, released_item);

    let mut valid_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_discover_versions(&mut valid_item),
        SUCCESS
    );
    assert_eq!(
        kmipkit_client_batch_add_item(batch, &mut valid_item),
        SUCCESS
    );
    assert!(valid_item.is_null());
    kmipkit_client_batch_release(batch);
}

#[test]
fn ffi_execute_fails_closed_for_poisoned_batch_and_client_locks() {
    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut poisoned_batch = empty_client_batch_handle();
    {
        let handle = reference_handle(poisoned_batch, Kind::ClientBatch)
            .expect("fresh client batch handle is registered");
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
        let handle =
            reference_handle(client, Kind::Client).expect("fresh client handle is registered");
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
