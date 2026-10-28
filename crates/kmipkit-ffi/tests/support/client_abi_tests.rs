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
    ActivateRequest, ArchiveRequest, AttributeSet, CertifyRequest, CheckRequest,
    CreateKeyPairRequest, CreateSplitKeyRequest, DelegatedLoginRequest, DerivationMethod,
    DerivationParameters, DeriveKeyRequest, DestroyRequest, ExportRequest, GetRequest,
    GetUsageAllocationRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest, LocateRequest,
    LogRequest, LoginRequest, LogoutRequest, ObjectGroupMember, ObjectType, ObtainLeaseRequest,
    OperationData, Pkcs11Request, ReProvisionRequest, RecertifyRequest, RecoverRequest,
    RegisterRequest, RekeyKeyPairRequest, RekeyRequest, RevocationReason, RevokeRequest,
    RngRetrieveRequest, RngSeedRequest, SecretBytes, SecretText, SplitKeyMethod, SplitKeyReference,
    StorageStatusMask, Ticket, TicketType, UniqueIdentifier, ValidateRequest,
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

#[path = "lifecycle_operation_abi_tests.rs"]
mod lifecycle_operation_abi_tests;
#[path = "query_ping_abi_tests.rs"]
mod query_ping_abi_tests;
#[path = "set_constraints_abi_tests.rs"]
mod set_constraints_abi_tests;
#[path = "set_defaults_abi_tests.rs"]
mod set_defaults_abi_tests;
#[path = "get_attribute_list_abi_tests.rs"]
mod get_attribute_list_abi_tests;

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
    pending_response_for_operation(8, unique_id, correlation)
}

fn pending_response_for_operation(operation: u32, unique_id: &[u8], correlation: &[u8]) -> Vec<u8> {
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
            ttlv_enumeration(0x0042_005C, operation),
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

fn successful_validate_response() -> Vec<u8> {
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
    let payload = ttlv_structure(0x0042_007c, [ttlv_enumeration(0x0042_009b, 0xf123_4567)]);
    ttlv_structure(
        0x0042_007b,
        [
            header,
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 0x17),
                    ttlv_item(0x0042_0093, 0x08, b"validate-success"),
                    ttlv_enumeration(0x0042_007f, 0),
                    payload,
                ],
            ),
        ],
    )
}

fn failed_validate_response() -> Vec<u8> {
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
    ttlv_structure(
        0x0042_007b,
        [
            header,
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 0x17),
                    ttlv_item(0x0042_0093, 0x08, b"validate-failure"),
                    ttlv_enumeration(0x0042_007f, 1),
                    ttlv_enumeration(0x0042_007e, 6),
                ],
            ),
        ],
    )
}

fn successful_obtain_lease_response() -> Vec<u8> {
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
            ttlv_item(0x0042_0094, 0x07, b"leased-object"),
            ttlv_item(0x0042_0049, 0x0a, &0_u32.to_be_bytes()),
            ttlv_datetime(0x0042_0048, -7),
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
                    ttlv_enumeration(0x0042_005c, 0x10),
                    ttlv_item(0x0042_0093, 0x08, b"obtain-lease-success"),
                    ttlv_enumeration(0x0042_007f, 0),
                    payload,
                ],
            ),
        ],
    )
}

fn failed_obtain_lease_response() -> Vec<u8> {
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
    ttlv_structure(
        0x0042_007b,
        [
            header,
            ttlv_structure(
                0x0042_000f,
                [
                    ttlv_enumeration(0x0042_005c, 0x10),
                    ttlv_item(0x0042_0093, 0x08, b"obtain-lease-failure"),
                    ttlv_enumeration(0x0042_007f, 1),
                    ttlv_enumeration(0x0042_007e, 0x37),
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

fn successful_response_without_batch_item_id(
    operation: u32,
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
    let children = response_fields.into_iter().flatten().collect::<Vec<_>>();
    let response_payload = ttlv_item(0x0042_007c, 0x01, &children);
    let response_item = ttlv_structure(
        0x0042_000f,
        [
            ttlv_enumeration(0x0042_005c, operation),
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

fn client_batch_item_response_from(
    response: ClientBatchResponse,
) -> *mut kmipkit_client_batch_item_response_t {
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
    item
}

#[test]
fn ffi_rng_request_builders_preserve_signed_length_and_required_seed_presence() {
    let mut retrieve = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_rng_retrieve(-17, &raw mut retrieve)
    );
    assert!(!retrieve.is_null());
    kmipkit_client_batch_item_release(retrieve);

    let seed = [0x00, 0x81, 0xff];
    let mut seed_span = byte_span(&seed);
    let mut seed_item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_rng_seed(&raw const seed_span, &raw mut seed_item)
    );
    assert!(!seed_item.is_null());
    kmipkit_client_batch_item_release(seed_item);

    seed_span = byte_span(&[]);
    let mut empty_seed_item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_rng_seed(&raw const seed_span, &raw mut empty_seed_item)
    );
    assert!(!empty_seed_item.is_null());
    kmipkit_client_batch_item_release(empty_seed_item);

    let mut omitted_seed_item = ptr::dangling_mut();
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_rng_seed(ptr::null(), &raw mut omitted_seed_item)
    );
    assert!(omitted_seed_item.is_null());
}

#[test]
fn ffi_rng_retrieve_response_exposes_success_data_and_generic_ttlv() {
    let secret = b"KMIP_RNG_RETRIEVE_SECRET_SENTINEL";
    let item = successful_cryptographic_item(
        ClientRequest::RngRetrieve(RngRetrieveRequest::new(-17)),
        0x25,
        b"rng-retrieve-success",
        vec![ttlv_item(0x0042_00c2, 0x08, secret)],
    );
    let mut present = 0;
    let mut data_present = 0;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rng_retrieve(
            item,
            &raw mut present,
            &raw mut data_present,
            &raw mut data,
        )
    );
    assert_eq!((present, data_present), (1, 1));
    assert_response_span(data, secret);

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rng_retrieve_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);
    assert_eq!(
        1,
        // SAFETY: the C consumer reads the live response and the known test bytes only.
        unsafe {
            kmipkit_client_abi_c_consumer_validate_rng_retrieve_response(
                item,
                secret.as_ptr(),
                u64::try_from(secret.len()).expect("test secret length fits u64"),
            )
        }
    );
}

#[test]
fn ffi_rng_seed_response_preserves_zero_data_length_and_generic_ttlv() {
    let item = successful_cryptographic_item(
        ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(
            b"rng-seed-request".to_vec(),
        ))),
        0x26,
        b"rng-seed-success",
        vec![ttlv_item(0x0042_00d5, 0x02, &0_i32.to_be_bytes())],
    );
    let mut present = 0;
    let mut data_length_present = 0;
    let mut data_length = -1;
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rng_seed(
            item,
            &raw mut present,
            &raw mut data_length_present,
            &raw mut data_length,
        )
    );
    assert_eq!((present, data_length_present, data_length), (1, 1, 0));

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rng_seed_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);
    assert_eq!(
        1,
        // SAFETY: the C consumer reads the live response and preserves the explicit zero value.
        unsafe { kmipkit_client_abi_c_consumer_validate_rng_seed_response(item, 0) }
    );
}

#[test]
fn ffi_rng_response_getters_fail_closed_for_mismatched_handles_and_operations() {
    let wrong_variant = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::Structure(Structure::new()),
    );
    let retrieve_item = successful_cryptographic_item(
        ClientRequest::RngRetrieve(RngRetrieveRequest::new(-17)),
        0x25,
        b"rng-retrieve-getter-mismatch",
        vec![ttlv_item(0x0042_00c2, 0x08, b"retrieve")],
    );
    let retrieve_owner = reference_handle(retrieve_item, Kind::ClientBatchItemResponse)
        .expect("successful RNG Retrieve response remains registered");
    let HandleValue::ClientBatchItemResponse { response, .. } = &retrieve_owner.value else {
        panic!("the RNG Retrieve handle stores a batch response and item index");
    };
    let invalid_index = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(response),
            index: usize::MAX,
        },
    );

    for (item, expected_status) in [
        (wrong_variant, ERROR_INVALID_INPUT),
        (invalid_index, ERROR_INTERNAL),
    ] {
        let mut present = 9;
        let mut data_present = 9;
        let mut data = byte_span(b"sentinel");
        assert_eq!(
            expected_status,
            kmipkit_client_batch_item_response_rng_retrieve(
                item,
                &raw mut present,
                &raw mut data_present,
                &raw mut data,
            )
        );
        assert_eq!((present, data_present), (0, 0));
        assert!(data.data.is_null());
        assert_eq!(data.length, 0);

        let mut ttlv_present = 9;
        let mut response_ttlv = ptr::dangling_mut();
        assert_eq!(
            expected_status,
            kmipkit_client_batch_item_response_rng_retrieve_ttlv(
                item,
                &raw mut ttlv_present,
                &raw mut response_ttlv,
            )
        );
        assert_eq!(ttlv_present, 0);
        assert!(response_ttlv.is_null());

        let mut seed_present = 9;
        let mut data_length_present = 9;
        let mut data_length = -1;
        assert_eq!(
            expected_status,
            kmipkit_client_batch_item_response_rng_seed(
                item,
                &raw mut seed_present,
                &raw mut data_length_present,
                &raw mut data_length,
            )
        );
        assert_eq!((seed_present, data_length_present, data_length), (0, 0, 0));

        let mut seed_ttlv_present = 9;
        let mut seed_ttlv = ptr::dangling_mut();
        assert_eq!(
            expected_status,
            kmipkit_client_batch_item_response_rng_seed_ttlv(
                item,
                &raw mut seed_ttlv_present,
                &raw mut seed_ttlv,
            )
        );
        assert_eq!(seed_ttlv_present, 0);
        assert!(seed_ttlv.is_null());
        kmipkit_client_batch_item_response_release(item);
    }
}

#[test]
fn ffi_rng_response_getters_hide_fields_from_the_other_operation() {
    let retrieve_item = successful_cryptographic_item(
        ClientRequest::RngRetrieve(RngRetrieveRequest::new(-17)),
        0x25,
        b"rng-retrieve-getter-cross-operation",
        vec![ttlv_item(0x0042_00c2, 0x08, b"retrieve")],
    );
    let seed_item = successful_cryptographic_item(
        ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(b"seed".to_vec()))),
        0x26,
        b"rng-seed-getter-cross-operation",
        vec![ttlv_item(0x0042_00d5, 0x02, &0_i32.to_be_bytes())],
    );
    for (item, expect_retrieve) in [(retrieve_item, true), (seed_item, false)] {
        let mut present = 9;
        let mut data_present = 9;
        let mut data = byte_span(b"sentinel");
        assert_eq!(
            SUCCESS,
            kmipkit_client_batch_item_response_rng_retrieve(
                item,
                &raw mut present,
                &raw mut data_present,
                &raw mut data,
            )
        );
        assert_eq!(present, u8::from(expect_retrieve));
        assert_eq!(data_present, u8::from(expect_retrieve));
        if expect_retrieve {
            assert_response_span(data, b"retrieve");
        } else {
            assert!(data.data.is_null());
            assert_eq!(data.length, 0);
        }

        let mut ttlv_present = 9;
        let mut response_ttlv = ptr::null_mut();
        assert_eq!(
            SUCCESS,
            kmipkit_client_batch_item_response_rng_retrieve_ttlv(
                item,
                &raw mut ttlv_present,
                &raw mut response_ttlv,
            )
        );
        assert_eq!(ttlv_present, u8::from(expect_retrieve));
        if response_ttlv.is_null() {
            assert!(!expect_retrieve);
        } else {
            assert!(expect_retrieve);
            kmipkit_ttlv_structure_release(response_ttlv);
        }

        let mut seed_present = 9;
        let mut data_length_present = 9;
        let mut data_length = -1;
        assert_eq!(
            SUCCESS,
            kmipkit_client_batch_item_response_rng_seed(
                item,
                &raw mut seed_present,
                &raw mut data_length_present,
                &raw mut data_length,
            )
        );
        assert_eq!(seed_present, u8::from(!expect_retrieve));
        assert_eq!(data_length_present, u8::from(!expect_retrieve));
        assert_eq!(data_length, 0);

        let mut seed_ttlv_present = 9;
        let mut seed_ttlv = ptr::null_mut();
        assert_eq!(
            SUCCESS,
            kmipkit_client_batch_item_response_rng_seed_ttlv(
                item,
                &raw mut seed_ttlv_present,
                &raw mut seed_ttlv,
            )
        );
        assert_eq!(seed_ttlv_present, u8::from(!expect_retrieve));
        if seed_ttlv.is_null() {
            assert!(expect_retrieve);
        } else {
            assert!(!expect_retrieve);
            kmipkit_ttlv_structure_release(seed_ttlv);
        }
        kmipkit_client_batch_item_response_release(item);
    }
}

#[test]
fn ffi_rng_response_c_consumer_rejects_overlapping_output_regions() {
    let retrieve_item = successful_cryptographic_item(
        ClientRequest::RngRetrieve(RngRetrieveRequest::new(-17)),
        0x25,
        b"rng-retrieve-overlapping-output-consumer",
        vec![ttlv_item(0x0042_00c2, 0x08, b"seedless")],
    );
    let seed_item = successful_cryptographic_item(
        ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(b"seed".to_vec()))),
        0x26,
        b"rng-seed-overlapping-output-consumer",
        vec![ttlv_item(0x0042_00d5, 0x02, &0_i32.to_be_bytes())],
    );

    // SAFETY: the compiled C consumer receives live, independently owned handles and only uses
    // valid object addresses plus deliberately aliased output regions.
    assert_eq!(1, unsafe {
        kmipkit_client_abi_c_consumer_rejects_rng_output_aliases(retrieve_item)
    });
    // SAFETY: the compiled C consumer receives live, independently owned handles and only uses
    // valid object addresses plus deliberately aliased output regions.
    assert_eq!(1, unsafe {
        kmipkit_client_abi_c_consumer_rejects_rng_output_aliases(seed_item)
    });
    kmipkit_client_batch_item_response_release(retrieve_item);
    kmipkit_client_batch_item_response_release(seed_item);
}

#[test]
fn ffi_rng_response_c_consumer_hides_failure_and_pending_success_fields() {
    let failure_retrieve = failed_cryptographic_item(
        ClientRequest::RngRetrieve(RngRetrieveRequest::new(-17)),
        0x25,
        b"rng-retrieve-failure",
    );
    let failure_seed = failed_cryptographic_item(
        ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(b"seed".to_vec()))),
        0x26,
        b"rng-seed-failure",
    );
    let retrieve_correlation = b"retrieve-pending-correlation";
    let seed_correlation = b"seed-pending-correlation";
    let pending_retrieve_response = execute_single_abi_response_with_async(
        ClientRequest::RngRetrieve(RngRetrieveRequest::new(-17)),
        b"rng-retrieve-pending",
        pending_response_for_operation(0x25, b"rng-retrieve-pending", retrieve_correlation),
        Some(1),
    )
    .expect("the asynchronous RNG Retrieve batch permits Pending");
    let pending_seed_response = execute_single_abi_response_with_async(
        ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(b"seed".to_vec()))),
        b"rng-seed-pending",
        pending_response_for_operation(0x26, b"rng-seed-pending", seed_correlation),
        Some(1),
    )
    .expect("the asynchronous RNG Seed batch permits Pending");
    let pending_retrieve = client_batch_item_response_from(pending_retrieve_response);
    let pending_seed = client_batch_item_response_from(pending_seed_response);

    for item in [
        failure_retrieve,
        pending_retrieve,
        failure_seed,
        pending_seed,
    ] {
        // SAFETY: the compiled C consumer receives a live response handle and only checks that
        // failure/Pending calls return empty success-only output records.
        assert_eq!(1, unsafe {
            kmipkit_client_abi_c_consumer_validates_absent_rng_success_fields(item)
        });
        kmipkit_client_batch_item_response_release(item);
    }
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
fn ffi_successful_encrypt_response_getter_returns_typed_fields() {
    let correlation = [0x51, 0x00, 0x52];
    let identifier = b"encrypt-response-id";
    let encrypt = successful_cryptographic_item(
        ClientRequest::Encrypt(EncryptRequest::new(
            Some(UniqueIdentifier::TextString(
                "encrypt-request-key".to_owned(),
            )),
            Some(OperationData::ByteString(SecretBytes::new(
                b"request-plaintext".to_vec(),
            ))),
        )),
        0x1f,
        b"encrypt-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, identifier),
            ttlv_item(0x0042_00c2, 0x08, b"ciphertext"),
            ttlv_item(0x0042_003d, 0x08, b"response-iv"),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
            ttlv_item(0x0042_00ff, 0x08, b"authentication-tag"),
        ],
    );
    let mut present = 0;
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 0;
    let mut data = byte_span(&[]);
    let mut iv_present = 0;
    let mut iv = byte_span(&[]);
    let mut correlation_present = 0;
    let mut correlation_value = byte_span(&[]);
    let mut tag_present = 0;
    let mut tag = byte_span(&[]);

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_encrypt(
            encrypt,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut iv_present,
            &raw mut iv,
            &raw mut correlation_present,
            &raw mut correlation_value,
            &raw mut tag_present,
            &raw mut tag,
        )
    );
    assert_eq!(
        (
            present,
            data_present,
            iv_present,
            correlation_present,
            tag_present
        ),
        (1, 1, 1, 1, 1)
    );
    assert_response_identifier(&output_identifier, identifier);
    assert_response_span(data, b"ciphertext");
    assert_response_span(iv, b"response-iv");
    assert_response_span(correlation_value, &correlation);
    assert_response_span(tag, b"authentication-tag");
    kmipkit_client_batch_item_response_release(encrypt);
}

#[test]
fn ffi_encrypt_response_getter_clears_fields_for_failed_result() {
    let encrypt = failed_cryptographic_item(
        ClientRequest::Encrypt(EncryptRequest::new(
            Some(UniqueIdentifier::TextString(
                "encrypt-request-key".to_owned(),
            )),
            Some(OperationData::ByteString(SecretBytes::new(
                b"request-plaintext".to_vec(),
            ))),
        )),
        0x1f,
        b"encrypt-failure",
    );
    let sentinel = b"stale-output";
    let mut present = 1;
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 1;
    let mut data = byte_span(sentinel);
    let mut iv_present = 1;
    let mut iv = byte_span(sentinel);
    let mut correlation_present = 1;
    let mut correlation_value = byte_span(sentinel);
    let mut tag_present = 1;
    let mut tag = byte_span(sentinel);

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_encrypt(
            encrypt,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut iv_present,
            &raw mut iv,
            &raw mut correlation_present,
            &raw mut correlation_value,
            &raw mut tag_present,
            &raw mut tag,
        )
    );
    assert_eq!(
        (
            present,
            data_present,
            iv_present,
            correlation_present,
            tag_present
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(output_identifier.kind, 0);
    for span in [data, iv, correlation_value, tag] {
        assert!(span.data.is_null());
        assert_eq!(span.length, 0);
    }
    kmipkit_client_batch_item_response_release(encrypt);
}

#[test]
fn ffi_encrypt_response_getter_rejects_invalid_identifier_output_record() {
    let encrypt = failed_cryptographic_item(
        ClientRequest::Encrypt(EncryptRequest::new(
            Some(UniqueIdentifier::TextString(
                "encrypt-request-key".to_owned(),
            )),
            Some(OperationData::ByteString(SecretBytes::new(
                b"request-plaintext".to_vec(),
            ))),
        )),
        0x1f,
        b"encrypt-failure",
    );
    let mut present = 1;
    let mut output_identifier = empty_unique_identifier_output_v1();
    output_identifier.struct_size = 0;
    let mut data_present = 1;
    let mut data = byte_span(b"stale-data");
    let mut iv_present = 1;
    let mut iv = byte_span(b"stale-iv");
    let mut correlation_present = 1;
    let mut correlation_value = byte_span(b"stale-correlation");
    let mut tag_present = 1;
    let mut tag = byte_span(b"stale-tag");

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_encrypt(
            encrypt,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut iv_present,
            &raw mut iv,
            &raw mut correlation_present,
            &raw mut correlation_value,
            &raw mut tag_present,
            &raw mut tag,
        )
    );
    assert_eq!(
        (
            present,
            data_present,
            iv_present,
            correlation_present,
            tag_present
        ),
        (0, 0, 0, 0, 0)
    );
    for span in [data, iv, correlation_value, tag] {
        assert!(span.data.is_null());
        assert_eq!(span.length, 0);
    }
    kmipkit_client_batch_item_response_release(encrypt);
}

#[test]
fn ffi_successful_decrypt_response_getter_returns_typed_fields() {
    let correlation = [0x61, 0x00, 0x62];
    let identifier = b"decrypt-response-id";
    let decrypt = successful_cryptographic_item(
        ClientRequest::Decrypt(DecryptRequest::new(
            Some(UniqueIdentifier::TextString(
                "decrypt-request-key".to_owned(),
            )),
            Some(OperationData::ByteString(SecretBytes::new(
                b"ciphertext".to_vec(),
            ))),
        )),
        0x20,
        b"decrypt-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, identifier),
            ttlv_item(0x0042_00c2, 0x08, b"plaintext"),
            ttlv_item(0x0042_00d6, 0x08, &correlation),
        ],
    );
    let mut present = 0;
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 0;
    let mut data = byte_span(&[]);
    let mut correlation_present = 0;
    let mut correlation_value = byte_span(&[]);

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_decrypt(
            decrypt,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!((present, data_present, correlation_present), (1, 1, 1));
    assert_response_identifier(&output_identifier, identifier);
    assert_response_span(data, b"plaintext");
    assert_response_span(correlation_value, &correlation);
    kmipkit_client_batch_item_response_release(decrypt);
}

#[test]
fn ffi_decrypt_response_getter_clears_fields_for_failed_result() {
    let decrypt = failed_cryptographic_item(
        ClientRequest::Decrypt(DecryptRequest::new(
            Some(UniqueIdentifier::TextString(
                "decrypt-request-key".to_owned(),
            )),
            Some(OperationData::ByteString(SecretBytes::new(
                b"ciphertext".to_vec(),
            ))),
        )),
        0x20,
        b"decrypt-failure",
    );
    let mut present = 1;
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 1;
    let mut data = byte_span(b"stale-data");
    let mut correlation_present = 1;
    let mut correlation_value = byte_span(b"stale-correlation");

    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_decrypt(
            decrypt,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation_value,
        )
    );
    assert_eq!((present, data_present, correlation_present), (0, 0, 0));
    assert_eq!(output_identifier.kind, 0);
    for span in [data, correlation_value] {
        assert!(span.data.is_null());
        assert_eq!(span.length, 0);
    }
    kmipkit_client_batch_item_response_release(decrypt);
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
fn ffi_login_constructor_validates_optional_table_253_fields() {
    for (lease_kind, lease_value, count_present, count) in [
        (3, 0, 0, 0),
        (0, 1, 0, 0),
        (1, -1, 0, 0),
        (1, i64::from(u32::MAX) + 1, 0, 0),
        (0, 0, 2, 0),
        (0, 0, 0, 1),
    ] {
        let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();
        assert_eq!(
            kmipkit_client_batch_item_login(
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

    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_login(2, i64::MIN, 1, i32::MIN, ptr::null_mut(), &mut item),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);

    let mut interval_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_login(
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

    let wrong_usage_limits = make_handle::<kmipkit_ttlv_structure_t>(
        Kind::Structure,
        HandleValue::Value(Value::integer(1)),
    );
    let mut invalid_usage_limits_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_login(
            0,
            0,
            0,
            0,
            wrong_usage_limits,
            &mut invalid_usage_limits_item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(invalid_usage_limits_item.is_null());
    release(wrong_usage_limits, Kind::Structure);
}

#[test]
fn ffi_logout_constructor_requires_and_validates_ticket_table_391() {
    let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();
    assert_eq!(
        kmipkit_client_batch_item_logout(ptr::null_mut(), &mut item),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());

    let invalid_ticket = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    assert_eq!(
        kmipkit_client_batch_item_logout(invalid_ticket, &mut item),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    release(invalid_ticket, Kind::Structure);

    let mut ticket = Structure::new();
    ticket
        .try_push(
            Item::new(
                RawTag::new(0x0042_014a)
                    .expect("Ticket Type tag fits the KMIP tag field")
                    .try_checked()
                    .expect("Ticket Type is allocated in KMIP 2.1"),
                Value::enumeration(TicketType::LOGIN.raw()),
            )
            .expect("valid Ticket Type test item builds"),
        )
        .expect("Ticket Type item fits within the test structure limit");
    ticket
        .try_push(
            Item::new(
                RawTag::new(0x0042_014b)
                    .expect("Ticket Value tag fits the KMIP tag field")
                    .try_checked()
                    .expect("Ticket Value is allocated in KMIP 2.1"),
                Value::byte_string(b"logout-abi-ticket".to_vec()),
            )
            .expect("valid Ticket Value test item builds"),
        )
        .expect("Ticket Value item fits within the test structure limit");
    let ticket = make_handle(Kind::Structure, HandleValue::Structure(ticket));

    assert_eq!(kmipkit_client_batch_item_logout(ticket, &mut item), SUCCESS);
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);
    release(ticket, Kind::Structure);
}

#[test]
fn ffi_logout_response_presence_maps_success_and_failure() {
    let success = successful_cryptographic_item(
        ClientRequest::logout(LogoutRequest::new(
            Ticket::new(
                TicketType::LOGIN,
                SecretBytes::new(b"logout-success-ticket".to_vec()),
            )
            .expect("valid Logout request Ticket builds"),
        )),
        0x2e,
        b"logout-success",
        Vec::new(),
    );
    let failure = failed_cryptographic_item(
        ClientRequest::logout(LogoutRequest::new(
            Ticket::new(
                TicketType::LOGIN,
                SecretBytes::new(b"logout-failure-ticket".to_vec()),
            )
            .expect("valid Logout request Ticket builds"),
        )),
        0x2e,
        b"logout-failure",
    );

    let mut present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_logout(success, &mut present),
        SUCCESS
    );
    assert_eq!(present, 1);

    present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_logout(failure, &mut present),
        SUCCESS
    );
    assert_eq!(present, 1);

    kmipkit_client_batch_item_response_release(failure);
    kmipkit_client_batch_item_response_release(success);
}

#[test]
fn ffi_login_response_getters_return_raw_ticket_fields_and_owned_ttlv() {
    let secret = b"login-abi-ticket";
    let item = successful_cryptographic_item(
        ClientRequest::Login(LoginRequest::new()),
        0x2d,
        b"login-abi",
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
        kmipkit_client_batch_item_response_login(
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
        kmipkit_client_batch_item_response_login_ttlv(item, &mut ttlv_present, &mut ticket_ttlv),
        SUCCESS
    );
    assert_eq!(ttlv_present, 1);
    assert!(!ticket_ttlv.is_null());
    kmipkit_ttlv_structure_release(ticket_ttlv);
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_login_response_getters_clear_outputs_for_inconsistent_handle() {
    let wrong_variant: *mut kmipkit_client_batch_item_response_t = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::Structure(Structure::new()),
    );
    let mut present = 9;
    let mut ticket_present = 9;
    let mut ticket_type = u32::MAX;
    let mut ticket_value = byte_span(b"secret sentinel");
    assert_eq!(
        kmipkit_client_batch_item_response_login(
            wrong_variant,
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

    present = 9;
    let mut ticket_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_login_ttlv(
            wrong_variant,
            &mut present,
            &mut ticket_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(ticket_ttlv.is_null());
    release(wrong_variant, Kind::ClientBatchItemResponse);
}

#[test]
fn ffi_login_response_getters_fail_closed_for_absent_ticket_and_invalid_index() {
    // KMIP 2.1 §6.1.30, Table 255 defines Login failure responses; the empty
    // successful payload is invalid because Table 254 requires a Ticket.
    let item = failed_cryptographic_item(
        ClientRequest::Login(LoginRequest::new()),
        0x2d,
        b"login-no-ticket",
    );
    let mut present = 9;
    let mut ticket_present = 9;
    let mut ticket_type = u32::MAX;
    let mut ticket_value = byte_span(b"secret sentinel");
    let mut ticket_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_login(
            item,
            &mut present,
            &mut ticket_present,
            &mut ticket_type,
            &mut ticket_value,
        ),
        SUCCESS
    );
    assert_eq!((present, ticket_present, ticket_type), (1, 0, 0));
    assert!(ticket_value.data.is_null());
    assert_eq!(ticket_value.length, 0);

    present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_login_ttlv(item, &mut present, &mut ticket_ttlv),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(ticket_ttlv.is_null());

    let response_item = reference_handle(item, Kind::ClientBatchItemResponse)
        .expect("successful Login item remains registered");
    let HandleValue::ClientBatchItemResponse { response, .. } = &response_item.value else {
        panic!("the Login response handle stores its batch response and index");
    };
    let invalid_index = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(response),
            index: usize::MAX,
        },
    );
    present = 9;
    ticket_present = 9;
    ticket_type = u32::MAX;
    ticket_value = byte_span(b"secret sentinel");
    assert_eq!(
        kmipkit_client_batch_item_response_login(
            invalid_index,
            &mut present,
            &mut ticket_present,
            &mut ticket_type,
            &mut ticket_value,
        ),
        ERROR_INTERNAL
    );
    assert_eq!((present, ticket_present, ticket_type), (0, 0, 0));
    assert!(ticket_value.data.is_null());
    assert_eq!(ticket_value.length, 0);

    present = 9;
    ticket_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_login_ttlv(
            invalid_index,
            &mut present,
            &mut ticket_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert!(ticket_ttlv.is_null());
    release(invalid_index, Kind::ClientBatchItemResponse);
    drop(response_item);
    kmipkit_client_batch_item_response_release(item);
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

#[test]
fn ffi_validate_response_accessors_are_available_to_c() {
    let success = execute_single_abi_response(
        ClientRequest::Validate(ValidateRequest::new().with_unique_identifier(
            UniqueIdentifier::TextString("validate-success-object".to_owned()),
        )),
        b"validate-success",
        successful_validate_response(),
    )
    .expect("successful Validate response maps to a typed result");
    let success_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(success)),
    );
    let mut success_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(success_handle, 0, &mut success_item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(success_handle);
    // SAFETY: the C consumer owns and releases the response-item handle after checking both accessors.
    assert_eq!(
        unsafe {
            kmipkit_client_abi_c_consumer_validate_validate_response(success_item, 1, 0xf123_4567)
        },
        1
    );

    let failure = execute_single_abi_response(
        ClientRequest::Validate(ValidateRequest::new().with_unique_identifier(
            UniqueIdentifier::TextString("validate-failure-object".to_owned()),
        )),
        b"validate-failure",
        failed_validate_response(),
    )
    .expect("failed Validate response remains available as a typed batch result");
    let failure_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(failure)),
    );
    let mut failure_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(failure_handle, 0, &mut failure_item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(failure_handle);
    // SAFETY: the C consumer owns and releases the response-item handle after checking absent outputs.
    assert_eq!(
        unsafe { kmipkit_client_abi_c_consumer_validate_validate_response(failure_item, 0, 0) },
        1
    );
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

fn ffi_obtain_lease_response_item(
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

#[test]
fn ffi_obtain_lease_constructor_preserves_optional_identifier() {
    // OASIS KMIP Specification v2.1 §6.1.35, Table 268 makes Unique Identifier optional.
    let mut omitted_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_obtain_lease(ptr::null(), &raw mut omitted_item),
        SUCCESS
    );
    assert!(!omitted_item.is_null());
    let omitted_handle = reference_handle(omitted_item, Kind::BatchItem)
        .expect("the omitted-identifier builder returns an item handle");
    let HandleValue::BatchItem(omitted_batch_item) = &omitted_handle.value else {
        panic!("Obtain Lease creates a typed batch item");
    };
    let ClientRequest::ObtainLease(omitted_request) = omitted_batch_item.request() else {
        panic!("the batch item contains an Obtain Lease request");
    };
    assert_eq!(omitted_request.unique_identifier(), None);
    kmipkit_client_batch_item_release(omitted_item);

    let mut text = b"lease-object".to_vec();
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: text.as_ptr(),
        text_length: u64::try_from(text.len()).expect("identifier length fits ABI"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let mut explicit_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_obtain_lease(&raw const identifier, &raw mut explicit_item),
        SUCCESS
    );
    text.fill(b'X');
    let explicit_handle = reference_handle(explicit_item, Kind::BatchItem)
        .expect("the explicit-identifier builder returns an item handle");
    let HandleValue::BatchItem(explicit_batch_item) = &explicit_handle.value else {
        panic!("Obtain Lease creates a typed batch item");
    };
    let ClientRequest::ObtainLease(explicit_request) = explicit_batch_item.request() else {
        panic!("the batch item contains an Obtain Lease request");
    };
    assert_eq!(
        explicit_request.unique_identifier(),
        Some(&UniqueIdentifier::TextString("lease-object".to_owned()))
    );
    kmipkit_client_batch_item_release(explicit_item);
}

#[test]
fn ffi_obtain_lease_response_preserves_required_values_and_ordered_success_ttlv() {
    // OASIS KMIP Specification v2.1 §6.1.35, Table 269 requires all three success fields.
    let response =
        Arc::new(
            execute_single_abi_response(
                ClientRequest::ObtainLease(ObtainLeaseRequest::new().with_unique_identifier(
                    UniqueIdentifier::TextString("leased-object".to_owned()),
                )),
                b"obtain-lease-success",
                successful_obtain_lease_response(),
            )
            .expect("the Table 269 success maps through the C ABI client"),
        );
    let item = ffi_obtain_lease_response_item(&response);
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut lease_time = u32::MAX;
    let mut last_change_date = i64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_obtain_lease(
            item,
            &raw mut present,
            &raw mut identifier,
            &raw mut lease_time,
            &raw mut last_change_date,
        ),
        SUCCESS
    );
    assert_eq!((present, lease_time, last_change_date), (1, 0, -7));
    assert_response_identifier(&identifier, b"leased-object");

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_obtain_lease_ttlv(
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
fn ffi_obtain_lease_failure_has_no_typed_values_or_raw_response_ttlv() {
    // Table 270 failures retain common result metadata without success payload access.
    let response =
        Arc::new(
            execute_single_abi_response(
                ClientRequest::ObtainLease(ObtainLeaseRequest::new().with_unique_identifier(
                    UniqueIdentifier::TextString("leased-object".to_owned()),
                )),
                b"obtain-lease-failure",
                failed_obtain_lease_response(),
            )
            .expect("the Table 270 result remains a KMIP outcome"),
        );
    let item = ffi_obtain_lease_response_item(&response);
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    let mut lease_time = u32::MAX;
    let mut last_change_date = i64::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_obtain_lease(
            item,
            &raw mut present,
            &raw mut identifier,
            &raw mut lease_time,
            &raw mut last_change_date,
        ),
        SUCCESS
    );
    assert_eq!((present, lease_time, last_change_date), (0, 0, 0));
    assert_eq!(identifier.kind, 0);
    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut::<kmipkit_ttlv_structure_t>();
    assert_eq!(
        kmipkit_client_batch_item_response_obtain_lease_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(item);
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
    execute_single_abi_response_with_async(request, batch_item_id, response, None)
}

fn execute_single_abi_response_with_async(
    request: ClientRequest,
    batch_item_id: &[u8],
    response: Vec<u8>,
    asynchronous_indicator: Option<u32>,
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
    let mut batch = ClientBatch::from_items([
        ClientBatchItem::new(request).with_unique_batch_item_id(batch_item_id.to_vec())
    ]);
    if let Some(indicator) = asynchronous_indicator {
        batch = batch.with_asynchronous_indicator(indicator);
    }
    let result = client.execute(batch, &CodecLimits::defaults());
    peer.join().expect("controlled local peer completes");
    result
}

fn client_with_one_response(response: Vec<u8>) -> (Client, thread::JoinHandle<()>) {
    let (client, peer, _) = client_with_responses(vec![response]);
    (client, peer)
}

fn client_with_responses(
    responses: Vec<Vec<u8>>,
) -> (Client, thread::JoinHandle<()>, Arc<AtomicU64>) {
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
    let exchange_count = Arc::new(AtomicU64::new(0));
    let peer_exchange_count = Arc::clone(&exchange_count);
    let peer = thread::spawn(move || {
        for response in responses {
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
            let connection =
                ServerConnection::new(server_config.clone()).expect("TLS 1.3 connection builds");
            let mut tls = StreamOwned::new(connection, stream);
            match receive_ttlv_frame(&mut tls) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return,
                Err(error) => panic!("the typed request frame is bounded: {error}"),
            }
            peer_exchange_count.fetch_add(1, Ordering::Relaxed);
            tls.write_all(&response)
                .expect("the scripted response is written");
            tls.flush().expect("the response is flushed");
        }
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
    (client, peer, exchange_count)
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
            ptr::null(),
            0,
            1,
            parameters,
            attributes,
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    assert_eq!(
        kmipkit_client_batch_item_derive_key(
            7,
            ptr::null(),
            1,
            1,
            parameters,
            attributes,
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());

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
fn ffi_rekey_constructor_preserves_optional_fields_and_rejects_invalid_offset_flag() {
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let masks = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut rejected = ptr::dangling_mut::<kmipkit_client_batch_item_t>();

    assert_eq!(
        kmipkit_client_batch_item_rekey(ptr::null(), 2, 0, attributes, masks, &mut rejected,),
        ERROR_INVALID_INPUT
    );
    assert!(rejected.is_null());

    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_rekey(ptr::null(), 1, u32::MAX, attributes, masks, &mut item,),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("Re-key item handle is registered");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("the constructor returns a batch item");
    };
    let ClientRequest::Rekey(request) = batch_item.request() else {
        panic!("the constructor returns a Re-key request");
    };
    assert!(format!("{request:?}").contains("has_attributes: true"));
    assert!(format!("{request:?}").contains("has_protection_storage_masks: true"));

    kmipkit_client_batch_item_release(item);
    release(masks, Kind::Structure);
    release(attributes, Kind::Structure);
}

fn import_test_object() -> Item {
    Item::new(
        RawTag::new(0x0042_008f)
            .expect("Symmetric Key tag fits the KMIP tag field")
            .try_checked()
            .expect("Symmetric Key is allocated in KMIP 2.1"),
        Value::structure(Structure::new()),
    )
    .expect("valid generic Any Object test item builds")
}

fn import_test_identifier(bytes: &[u8]) -> kmipkit_unique_identifier_v1_t {
    kmipkit_unique_identifier_v1_t {
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        text_data: bytes.as_ptr(),
        text_length: u64::try_from(bytes.len()).expect("test identifier length fits u64"),
        ..empty_unique_identifier_output_v1()
    }
}

#[test]
fn ffi_import_constructor_rejects_missing_identifier_and_invalid_optional_flags() {
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut object_structure = Structure::new();
    object_structure
        .try_push(import_test_object())
        .expect("Any Object test item fits the structure");
    let object = make_handle(Kind::Structure, HandleValue::Structure(object_structure));
    let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();

    assert_eq!(
        kmipkit_client_batch_item_import(ptr::null(), 2, 0, 0, 0, 0, attributes, object, &mut item,),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());

    let identifier_bytes = b"import-test-id";
    let identifier = import_test_identifier(identifier_bytes);
    for (has_replace, replace, has_wrap, wrap) in
        [(2, 0, 0, 0), (0, 1, 0, 0), (0, 0, 2, 0), (0, 0, 0, 1)]
    {
        assert_eq!(
            kmipkit_client_batch_item_import(
                &raw const identifier,
                2,
                has_replace,
                replace,
                has_wrap,
                wrap,
                attributes,
                object,
                &mut item,
            ),
            ERROR_INVALID_INPUT
        );
        assert!(item.is_null());
    }

    release(object, Kind::Structure);
    release(attributes, Kind::Structure);
}

#[test]
fn ffi_import_constructor_requires_attributes_and_one_any_object_item() {
    let identifier_bytes = b"import-test-id";
    let identifier = import_test_identifier(identifier_bytes);
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut one_item = Structure::new();
    one_item
        .try_push(import_test_object())
        .expect("Any Object test item fits the structure");
    let object = make_handle(Kind::Structure, HandleValue::Structure(one_item));
    let empty_object = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::dangling_mut::<kmipkit_client_batch_item_t>();

    assert_eq!(
        kmipkit_client_batch_item_import(
            &raw const identifier,
            2,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
            object,
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    assert_eq!(
        kmipkit_client_batch_item_import(
            &raw const identifier,
            2,
            0,
            0,
            0,
            0,
            attributes,
            ptr::null_mut(),
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    assert_eq!(
        kmipkit_client_batch_item_import(
            &raw const identifier,
            2,
            0,
            0,
            0,
            0,
            attributes,
            empty_object,
            &mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());

    release(empty_object, Kind::Structure);
    release(object, Kind::Structure);
    release(attributes, Kind::Structure);
}

#[test]
fn ffi_import_constructor_preserves_explicit_raw_selector_values() {
    let identifier_bytes = b"import-test-id";
    let identifier = import_test_identifier(identifier_bytes);
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut object_structure = Structure::new();
    object_structure
        .try_push(import_test_object())
        .expect("Any Object test item fits the structure");
    let object = make_handle(Kind::Structure, HandleValue::Structure(object_structure));
    let mut item = ptr::null_mut();

    assert_eq!(
        kmipkit_client_batch_item_import(
            &raw const identifier,
            0xf123_4567,
            1,
            1,
            1,
            0xf123_4568,
            attributes,
            object,
            &mut item,
        ),
        SUCCESS
    );
    let handle = reference_handle(item, Kind::BatchItem).expect("Import item handle is live");
    let HandleValue::BatchItem(batch_item) = &handle.value else {
        panic!("Import handle contains a typed batch item");
    };
    let ClientRequest::Import(request) = batch_item.request() else {
        panic!("batch item contains the requested Import operation");
    };
    assert_eq!(
        request.unique_identifier(),
        &UniqueIdentifier::TextString("import-test-id".into())
    );
    assert_eq!(request.object_type().raw(), 0xf123_4567);
    assert_eq!(request.replace_existing(), Some(true));
    assert_eq!(
        request
            .key_wrap_type()
            .map(kmipkit_protocol::KeyWrapType::raw),
        Some(0xf123_4568)
    );

    release(item, Kind::BatchItem);
    release(object, Kind::Structure);
    release(attributes, Kind::Structure);
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

    let mut missing_attributes = ptr::dangling_mut();
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
            &raw mut missing_attributes,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(missing_attributes.is_null());

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

    assert_register_getters_reject_handle(wrong_variant, ERROR_INVALID_INPUT);
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

    assert_register_getters_reject_handle(invalid_index, ERROR_INTERNAL);

    kmipkit_client_batch_item_response_release(invalid_index);
    kmipkit_client_batch_item_response_release(valid_item);
}

fn assert_register_getters_reject_handle(
    handle: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_register(handle, &mut present, &mut identifier,),
        expected_status
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);

    present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_register_ttlv(handle, &mut present, &mut response_ttlv,),
        expected_status
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
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

#[test]
fn ffi_usage_allocation_context_copies_successful_response_metadata() {
    let response_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(12)),
        0x11,
        b"usage-context",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    let mut context = ptr::null_mut();
    assert_eq!(
        kmipkit_usage_allocation_context_create(response_item, 12, u32::MAX, &raw mut context,),
        SUCCESS
    );
    assert!(!context.is_null());
    kmipkit_client_batch_item_response_release(response_item);

    let mut unit = 0;
    let mut allocated = 0;
    let mut available = 0;
    let mut reserved = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_usage_allocation_context_usage_limits_unit(context, &raw mut unit),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_allocated_units(context, &raw mut allocated),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_available_units(context, &raw mut available),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_reserved_units(context, &raw mut reserved),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_unique_identifier(context, &raw mut identifier),
        SUCCESS
    );
    assert_eq!(unit, u32::MAX);
    assert_eq!(allocated, 12);
    assert_eq!(available, 12);
    assert_eq!(reserved, 0);
    assert_response_identifier(&identifier, b"allocation-object");
    kmipkit_usage_allocation_context_release(context);
}

#[test]
fn ffi_usage_allocation_context_rejects_non_success_and_nonpositive_counts() {
    let failed_item = failed_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(12)),
        0x11,
        b"usage-context-failure",
    );
    let mut context = ptr::dangling_mut();
    assert_eq!(
        kmipkit_usage_allocation_context_create(failed_item, 12, u32::MAX, &raw mut context,),
        ERROR_INVALID_INPUT
    );
    assert!(context.is_null());
    kmipkit_client_batch_item_response_release(failed_item);

    let successful_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(12)),
        0x11,
        b"usage-context-nonpositive",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    assert_eq!(
        kmipkit_usage_allocation_context_create(successful_item, 0, u32::MAX, &raw mut context,),
        ERROR_INVALID_INPUT
    );
    assert!(context.is_null());
    kmipkit_client_batch_item_response_release(successful_item);
}

#[test]
fn ffi_usage_allocation_context_rejects_a_successful_non_allocation_response() {
    let identifier = "not-an-allocation-response";
    let item = successful_cryptographic_item(
        ClientRequest::Encrypt(EncryptRequest::new(
            Some(UniqueIdentifier::TextString(identifier.to_owned())),
            Some(OperationData::ByteString(SecretBytes::new(
                b"context-source-test".to_vec(),
            ))),
        )),
        0x1f,
        identifier.as_bytes(),
        vec![
            ttlv_item(0x0042_0094, 0x07, identifier.as_bytes()),
            ttlv_item(0x0042_00c2, 0x08, b"ciphertext"),
        ],
    );
    let mut context = ptr::dangling_mut();

    assert_eq!(
        kmipkit_usage_allocation_context_create(item, 12, u32::MAX, &raw mut context),
        ERROR_INVALID_INPUT
    );
    assert!(context.is_null());
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_usage_allocation_context_getters_clear_outputs_for_a_wrong_handle_kind() {
    let client = unconnected_client_handle();
    let mut usage_limits_unit = u32::MAX;
    let mut allocated_units = -1;
    let mut available_units = -2;
    let mut reserved_units = -3;

    assert_eq!(
        kmipkit_usage_allocation_context_usage_limits_unit(
            client.cast(),
            &raw mut usage_limits_unit,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(usage_limits_unit, 0);
    assert_eq!(
        kmipkit_usage_allocation_context_allocated_units(client.cast(), &raw mut allocated_units,),
        ERROR_INVALID_INPUT
    );
    assert_eq!(allocated_units, 0);
    assert_eq!(
        kmipkit_usage_allocation_context_available_units(client.cast(), &raw mut available_units,),
        ERROR_INVALID_INPUT
    );
    assert_eq!(available_units, 0);
    assert_eq!(
        kmipkit_usage_allocation_context_reserved_units(client.cast(), &raw mut reserved_units,),
        ERROR_INVALID_INPUT
    );
    assert_eq!(reserved_units, 0);

    let mut identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING;
    identifier.text_data = ptr::dangling();
    identifier.text_length = 19;
    identifier.enumeration_value = u32::MAX;
    identifier.integer_value = i32::MIN;
    assert_eq!(
        kmipkit_usage_allocation_context_unique_identifier(client.cast(), &raw mut identifier,),
        ERROR_INVALID_INPUT
    );
    assert_eq!(identifier.kind, 0);
    assert_eq!(identifier.reserved, 0);
    assert!(identifier.text_data.is_null());
    assert_eq!(identifier.text_length, 0);
    assert_eq!(identifier.enumeration_value, 0);
    assert_eq!(identifier.integer_value, 0);
    kmipkit_client_release(client);
}

#[test]
fn ffi_usage_allocation_identifier_getter_rejects_an_invalid_output_record() {
    let mut identifier = empty_unique_identifier_output_v1();
    identifier.struct_size = 0;

    assert_eq!(
        kmipkit_usage_allocation_context_unique_identifier(ptr::null_mut(), &raw mut identifier,),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn ffi_guarded_execution_rejects_empty_slots_and_validation_without_transport() {
    let allocation_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"guarded-validation-allocation",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    let mut context = ptr::null_mut();
    assert_eq!(
        kmipkit_usage_allocation_context_create(allocation_item, 5, u32::MAX, &raw mut context),
        SUCCESS
    );
    kmipkit_client_batch_item_response_release(allocation_item);

    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut response = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            context,
            ptr::null_mut(),
            1,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(response.is_null());

    let mut item = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            context,
            &raw mut item,
            1,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    assert!(response.is_null());

    item = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::Encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString("allocation-object".to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(
                    b"invalid-cost-plaintext".to_vec(),
                ))),
            ),
        )))),
    );
    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            context,
            &raw mut item,
            0,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null(), "the ABI consumes an item before validation");
    assert!(response.is_null());

    let mut reserved_units = 0;
    let mut available_units = 0;
    assert_eq!(
        kmipkit_usage_allocation_context_reserved_units(context, &raw mut reserved_units),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_available_units(context, &raw mut available_units),
        SUCCESS
    );
    assert_eq!(reserved_units, 0);
    assert_eq!(available_units, 5);

    kmipkit_client_release(client);
    kmipkit_usage_allocation_context_release(context);
    release(limits, Kind::CodecLimits);
}

#[test]
fn ffi_guarded_execution_rejects_wrong_handle_kinds_without_consuming_the_item() {
    let allocation_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"guarded-handle-kind-allocation",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    let mut context = ptr::null_mut();
    assert_eq!(
        kmipkit_usage_allocation_context_create(allocation_item, 5, u32::MAX, &raw mut context),
        SUCCESS
    );
    kmipkit_client_batch_item_response_release(allocation_item);

    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let wrong_client = default_codec_limits_handle();
    let mut item = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::Encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString("allocation-object".to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(
                    b"handle-kind-plaintext".to_vec(),
                ))),
            ),
        )))),
    );
    let original_item = item;
    let mut response = ptr::null_mut();

    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            wrong_client.cast(),
            context,
            &raw mut item,
            1,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(item, original_item);
    assert!(response.is_null());

    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            client.cast(),
            &raw mut item,
            1,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(item, original_item);
    assert!(response.is_null());

    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            context,
            &raw mut item,
            1,
            context.cast(),
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(item, original_item);
    assert!(response.is_null());

    let wrong_item = failed_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"not-a-batch-request-item",
    );
    let mut wrong_item_slot = wrong_item.cast::<kmipkit_client_batch_item_t>();
    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            context,
            &raw mut wrong_item_slot,
            1,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(wrong_item_slot, wrong_item.cast());
    assert!(response.is_null());

    kmipkit_client_batch_item_release(item);
    kmipkit_client_batch_item_response_release(wrong_item);
    release(wrong_client, Kind::CodecLimits);
    kmipkit_client_release(client);
    kmipkit_usage_allocation_context_release(context);
    release(limits, Kind::CodecLimits);
}

#[test]
fn ffi_guarded_execution_rejects_a_retained_item_owner_before_transport() {
    let allocation_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"guarded-owner-allocation",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    let mut context = ptr::null_mut();
    assert_eq!(
        kmipkit_usage_allocation_context_create(allocation_item, 5, u32::MAX, &raw mut context),
        SUCCESS
    );
    kmipkit_client_batch_item_response_release(allocation_item);

    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut item = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::Encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString("allocation-object".to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(
                    b"retained-owner-plaintext".to_vec(),
                ))),
            ),
        )))),
    );
    let retained_owner =
        clone_handle_owner(item, Kind::BatchItem).expect("test retains a live item owner");
    let mut response = ptr::null_mut();

    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item(
            client,
            context,
            &raw mut item,
            1,
            limits,
            &raw mut response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(
        item.is_null(),
        "the in/out slot is consumed before owner check"
    );
    assert!(response.is_null());

    let mut reserved_units = 0;
    let mut available_units = 0;
    assert_eq!(
        kmipkit_usage_allocation_context_reserved_units(context, &raw mut reserved_units),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_available_units(context, &raw mut available_units),
        SUCCESS
    );
    assert_eq!(reserved_units, 0);
    assert_eq!(available_units, 5);

    drop(retained_owner);
    kmipkit_client_release(client);
    kmipkit_usage_allocation_context_release(context);
    release(limits, Kind::CodecLimits);
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
fn ffi_interop_ttlv_getter_copies_success_failure_and_pending_responses() {
    let request = || {
        ClientRequest::interop(InteropRequest::new(
            0xf123_4567,
            SecretText::new("interop-identifier-sentinel".to_owned()),
        ))
    };
    let success = successful_cryptographic_item(request(), 0x34, b"interop-success", Vec::new());
    let failure = failed_cryptographic_item(request(), 0x34, b"interop-failure");
    let pending_correlation = [0, 0xff, 0x80, 0x7f];
    let pending_response = execute_single_abi_response_with_async(
        request(),
        b"interop-pending",
        pending_response_for_operation(0x34, b"interop-pending", &pending_correlation),
        Some(1),
    )
    .expect("the async request permits an Interop Pending response");
    let pending_response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(pending_response)),
    );
    let mut pending = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(pending_response_handle, 0, &raw mut pending),
        SUCCESS
    );
    kmipkit_client_batch_response_release(pending_response_handle);

    for (item, expected_count) in [(success, 4), (failure, 4), (pending, 5)] {
        let mut present = 0;
        let mut response_ttlv = ptr::null_mut();
        assert_eq!(
            client_abi::kmipkit_client_batch_item_response_interop_ttlv(
                item,
                &raw mut present,
                &raw mut response_ttlv,
            ),
            SUCCESS
        );
        assert_eq!(present, 1);
        assert!(!response_ttlv.is_null());

        let mut view = ptr::null_mut();
        assert_eq!(
            kmipkit_ttlv_structure_view(response_ttlv, &raw mut view),
            SUCCESS
        );
        let mut count = 0;
        assert_eq!(
            kmipkit_ttlv_structure_view_item_count(view, &raw mut count),
            SUCCESS
        );
        assert_eq!(count, expected_count);
        kmipkit_ttlv_structure_view_release(view);
        kmipkit_ttlv_structure_release(response_ttlv);
        kmipkit_client_batch_item_response_release(item);
    }

    let unrelated_item = successful_check_response_item(&successful_check_client_response());
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_interop_ttlv(
            unrelated_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(unrelated_item);
}

#[test]
fn ffi_interop_and_log_ttlv_getters_reject_inconsistent_response_item_handles() {
    let response = successful_check_client_response();
    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    for getter in [
        client_abi::kmipkit_client_batch_item_response_interop_ttlv,
        client_abi::kmipkit_client_batch_item_response_log_ttlv,
    ] {
        let mut present = 9;
        let mut response_ttlv = ptr::dangling_mut();
        assert_eq!(
            getter(mismatched_item, &raw mut present, &raw mut response_ttlv),
            ERROR_INVALID_INPUT
        );
        assert_eq!(present, 0);
        assert!(response_ttlv.is_null());
    }
    kmipkit_client_batch_item_response_release(mismatched_item);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response,
            index: usize::MAX,
        },
    );
    for getter in [
        client_abi::kmipkit_client_batch_item_response_interop_ttlv,
        client_abi::kmipkit_client_batch_item_response_log_ttlv,
    ] {
        let mut present = 9;
        let mut response_ttlv = ptr::dangling_mut();
        assert_eq!(
            getter(invalid_index_item, &raw mut present, &raw mut response_ttlv),
            ERROR_INTERNAL
        );
        assert_eq!(present, 0);
        assert!(response_ttlv.is_null());
    }
    kmipkit_client_batch_item_response_release(invalid_index_item);
}

#[test]
fn ffi_log_ttlv_getter_copies_success_failure_and_pending_responses() {
    let request = || {
        ClientRequest::log(LogRequest::new(SecretText::new(
            "log-message-private-sentinel".to_owned(),
        )))
    };
    let success = successful_cryptographic_item(request(), 0x2c, b"log-success", Vec::new());
    let failure = failed_cryptographic_item(request(), 0x2c, b"log-failure");
    let pending_correlation = [0, 0xff, 0x80, 0x7f];
    let pending_response = execute_single_abi_response_with_async(
        request(),
        b"log-pending",
        pending_response_for_operation(0x2c, b"log-pending", &pending_correlation),
        Some(1),
    )
    .expect("the async request permits a Log Pending response");
    let pending_response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(pending_response)),
    );
    let mut pending = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(pending_response_handle, 0, &raw mut pending),
        SUCCESS
    );
    kmipkit_client_batch_response_release(pending_response_handle);

    for (item, expected_count) in [(success, 4), (failure, 4), (pending, 5)] {
        let mut present = 0;
        let mut response_ttlv = ptr::null_mut();
        assert_eq!(
            client_abi::kmipkit_client_batch_item_response_log_ttlv(
                item,
                &raw mut present,
                &raw mut response_ttlv,
            ),
            SUCCESS
        );
        assert_eq!(present, 1);
        assert!(!response_ttlv.is_null());

        let mut view = ptr::null_mut();
        assert_eq!(
            kmipkit_ttlv_structure_view(response_ttlv, &raw mut view),
            SUCCESS
        );
        let mut count = 0;
        assert_eq!(
            kmipkit_ttlv_structure_view_item_count(view, &raw mut count),
            SUCCESS
        );
        assert_eq!(count, expected_count);
        kmipkit_ttlv_structure_view_release(view);
        kmipkit_ttlv_structure_release(response_ttlv);
        kmipkit_client_batch_item_response_release(item);
    }

    let unrelated_item = successful_check_response_item(&successful_check_client_response());
    let mut present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_log_ttlv(
            unrelated_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(unrelated_item);
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
    fn kmipkit_client_abi_c_consumer_validate_rng_retrieve_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_data: *const u8,
        expected_length: u64,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_rng_seed_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_data_length: i32,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_rejects_rng_output_aliases(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validates_absent_rng_success_fields(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_failed_check_response(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_validate_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_present: u8,
        expected_indicator: u32,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_certify_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_recertify_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_rekey_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_rekey_key_pair_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_reprovision_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expect_success: u8,
        expected_identifier_present: u8,
        expected_identifier: *const u8,
        expected_identifier_length: u64,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_register_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_identifier: *const u8,
        expected_identifier_length: u64,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_build_revoke_request() -> i32;
    fn kmipkit_client_abi_c_consumer_validate_revoke_response(
        item: *mut kmipkit_client_batch_item_response_t,
        expected_identifier: *const u8,
        expected_identifier_length: u64,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_absent_revoke_response(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_rejects_register_output_aliases(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_usage_allocation_response_ttlv(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_usage_allocation_context(
        item: *mut kmipkit_client_batch_item_response_t,
    ) -> i32;
    fn shared_usage_allocation_guard_matches_fixture(
        client: *mut kmipkit_client_t,
        identifier_data: *const u8,
        identifier_length: u64,
        allocation_count: i64,
        usage_limits_unit: u32,
        successful_cost: i64,
        over_budget_cost: i64,
        expected_initial_allocated: i64,
        expected_initial_reserved: i64,
        expected_initial_available: i64,
        expected_allocated: i64,
        expected_reserved: i64,
        expected_available: i64,
        item: *mut *mut kmipkit_client_batch_item_t,
        limits: *mut kmipkit_codec_limits_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_validate_join_split_key_response_ttlv(
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

// KMIP Specification v2.1 §6.1.45, Tables 301–302.
#[test]
fn ffi_recertify_ttlv_is_a_complete_ordered_caller_owned_copy_from_c() {
    let unknown_value = [0xde, 0xad, 0x00, 0xff];
    let item = successful_cryptographic_item(
        ClientRequest::recertify(
            RecertifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("old-certificate".to_owned())),
        ),
        0x07,
        b"recertify-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"replacement-certificate"),
            ttlv_item(0x0054_0001, 0x08, &unknown_value),
        ],
    );

    let mut identifier_present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_recertify(
            item,
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    assert_eq!(identifier_present, 1);
    assert_response_identifier(&identifier, b"replacement-certificate");

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_recertify_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    // SAFETY: the compiled C consumer receives this live response-item handle and takes ownership.
    let result = unsafe { kmipkit_client_abi_c_consumer_validate_recertify_response_ttlv(item) };

    assert_eq!(
        result, 1,
        "C reads the replacement identifier and complete ordered TTLV after releasing the source item"
    );
}

// OASIS KMIP Specification v2.1 §6.1.43, Tables 292 and 294.
#[test]
fn ffi_register_success_exposes_identifier_and_owned_ordered_ttlv_to_c() {
    let identifier_value = b"registered-opaque";
    let unknown_value = [0xde, 0xad, 0x00, 0xff];
    let mut opaque_fields = Structure::new();
    opaque_fields
        .try_push(
            Item::new(
                RawTag::new(0x0042_0059)
                    .expect("Opaque Data Type tag fits")
                    .try_checked()
                    .expect("Opaque Data Type is allocated"),
                Value::enumeration(0x8000_0001),
            )
            .expect("Opaque Data Type item is valid"),
        )
        .expect("Opaque Data Type fits the structure");
    opaque_fields
        .try_push(
            Item::new(
                RawTag::new(0x0042_005a)
                    .expect("Opaque Data Value tag fits")
                    .try_checked()
                    .expect("Opaque Data Value is allocated"),
                Value::byte_string(b"register-request-secret".to_vec()),
            )
            .expect("Opaque Data Value item is valid"),
        )
        .expect("Opaque Data Value fits the structure");
    let request = ClientRequest::Register(RegisterRequest::new(
        ObjectType::from_raw(8),
        AttributeSet::try_new([]).expect("generic Opaque Object has no mandatory attributes"),
        Item::new(
            RawTag::new(0x0042_005b)
                .expect("Opaque Object tag fits")
                .try_checked()
                .expect("Opaque Object is allocated"),
            Value::structure(opaque_fields),
        )
        .expect("Opaque Object item is valid"),
    ));
    let item = successful_cryptographic_item(
        request,
        0x03,
        b"register-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, identifier_value),
            ttlv_item(0x0054_1234, 0x08, &unknown_value),
        ],
    );

    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_register(item, &raw mut present, &raw mut identifier,)
    );
    assert_eq!(present, 1);
    assert_response_identifier(&identifier, identifier_value);

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_register_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    let mut malformed_present = 9;
    let mut malformed_identifier = empty_unique_identifier_output_v1();
    malformed_identifier.struct_size = 0;
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_register(
            item,
            &raw mut malformed_present,
            &raw mut malformed_identifier,
        )
    );
    assert_eq!(malformed_present, 9);
    assert_eq!(malformed_identifier.struct_size, 0);
    assert_eq!(malformed_identifier.kind, 0);

    // SAFETY: the compiled C consumer takes ownership of this live response item.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_validate_register_response(
            item,
            identifier_value.as_ptr(),
            u64::try_from(identifier_value.len()).expect("test identifier fits the ABI"),
        )
    };
    assert_eq!(
        result, 1,
        "C copies the successful identifier and ordered TTLV"
    );
}

#[test]
fn ffi_register_c_consumer_rejects_overlapping_outputs_without_writes() {
    let item = successful_cryptographic_item(
        register_request_fixture(),
        0x03,
        b"register-overlap-check",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"registered-overlap"),
            ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad]),
        ],
    );

    // SAFETY: the compiled C consumer receives this live Register response item.
    let result = unsafe { kmipkit_client_abi_c_consumer_rejects_register_output_aliases(item) };

    assert_eq!(result, 1, "overlap rejection preserves each output region");
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_revoke_request_and_response_are_available_to_compiled_c_consumer() {
    // SAFETY: the compiled public-header consumer creates, transfers, and releases its own handles.
    assert_eq!(1, unsafe {
        kmipkit_client_abi_c_consumer_build_revoke_request()
    });

    let identifier = b"c-revoke-success";
    let item = successful_cryptographic_item(
        ClientRequest::Revoke(
            RevokeRequest::new(RevocationReason::from_raw(2)).with_unique_identifier(
                UniqueIdentifier::TextString("revoke-request-id".to_owned()),
            ),
        ),
        0x13,
        b"revoke-response-item",
        vec![
            ttlv_item(0x0042_0094, 0x07, identifier),
            ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad]),
        ],
    );
    // SAFETY: the C consumer borrows this valid success result, copies its TTLV, then releases it.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_validate_revoke_response(
            item,
            identifier.as_ptr(),
            u64::try_from(identifier.len()).expect("identifier length fits u64"),
        )
    };
    assert_eq!(
        1, result,
        "the compiled C consumer observes typed Revoke success"
    );
}

#[test]
fn ffi_revoke_c_consumer_clears_outputs_for_failure_and_other_operations() {
    let failed = failed_cryptographic_item(
        ClientRequest::Revoke(
            RevokeRequest::new(RevocationReason::from_raw(2))
                .with_unique_identifier(UniqueIdentifier::TextString("failed-revoke".to_owned())),
        ),
        0x13,
        b"failed-revoke-item",
    );
    // SAFETY: the C consumer borrows this live failed Revoke response item.
    let result = unsafe { kmipkit_client_abi_c_consumer_validate_absent_revoke_response(failed) };
    assert_eq!(1, result, "a failed Revoke exposes no success fields");
    kmipkit_client_batch_item_response_release(failed);

    let other_operation = successful_cryptographic_item(
        register_request_fixture(),
        0x03,
        b"register-for-revoke-absence",
        vec![ttlv_item(0x0042_0094, 0x07, b"registered-object")],
    );
    // SAFETY: the C consumer borrows this live successful Register response item.
    let result =
        unsafe { kmipkit_client_abi_c_consumer_validate_absent_revoke_response(other_operation) };
    assert_eq!(1, result, "a different operation has no Revoke response");
    kmipkit_client_batch_item_response_release(other_operation);
}

#[test]
fn ffi_revoke_request_preserves_raw_reason_nested_message_and_optional_fields() {
    let message = b"revoke audit marker";
    let mut reason = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revocation_reason_create(
            u32::MAX,
            1,
            message.as_ptr(),
            u64::try_from(message.len()).expect("message length fits u64"),
            &raw mut reason,
        )
    );
    let mut request = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revoke_request_create(ptr::null(), reason, 0, 0, &raw mut request)
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_revoke(request, &raw mut item)
    );

    let handle = reference_handle(item, Kind::BatchItem)
        .expect("the typed Revoke request returns a batch-item handle");
    let HandleValue::BatchItem(item_value) = &handle.value else {
        panic!("the C ABI returns a typed batch item");
    };
    let ClientRequest::Revoke(request_value) = item_value.request() else {
        panic!("the batch item contains the Revoke request");
    };
    assert_eq!(request_value.revocation_reason().raw(), u32::MAX);
    assert_eq!(
        request_value.revocation_message(),
        Some("revoke audit marker")
    );
    assert_eq!(request_value.unique_identifier(), None);
    assert_eq!(request_value.compromise_occurrence_date(), None);
    kmipkit_client_batch_item_release(item);

    let mut compromise_reason = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revocation_reason_create(2, 0, ptr::null(), 0, &raw mut compromise_reason)
    );
    let identifier = import_test_identifier(b"key-to-revoke");
    let mut dated_request = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revoke_request_create(
            &raw const identifier,
            compromise_reason,
            1,
            -1,
            &raw mut dated_request,
        )
    );
    let mut dated_item = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_revoke(dated_request, &raw mut dated_item)
    );
    let dated_handle = reference_handle(dated_item, Kind::BatchItem)
        .expect("the key-compromise request accepts the optional date");
    let HandleValue::BatchItem(dated_value) = &dated_handle.value else {
        panic!("the dated request remains a typed batch item");
    };
    let ClientRequest::Revoke(dated_request) = dated_value.request() else {
        panic!("the dated batch item contains Revoke");
    };
    assert_eq!(
        dated_request.unique_identifier(),
        Some(&UniqueIdentifier::TextString("key-to-revoke".to_owned()))
    );
    assert_eq!(dated_request.compromise_occurrence_date(), Some(-1));
    kmipkit_client_batch_item_release(dated_item);

    let mut invalid_reason = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revocation_reason_create(4, 0, ptr::null(), 0, &raw mut invalid_reason)
    );
    let mut invalid_request = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revoke_request_create(ptr::null(), invalid_reason, 1, 1, &raw mut invalid_request)
    );
    let mut invalid_item = ptr::null_mut();
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_revoke(invalid_request, &raw mut invalid_item)
    );
    assert!(invalid_item.is_null());
}

#[test]
fn ffi_revoke_request_consumes_reason_when_later_argument_is_invalid() {
    let mut reason = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_revocation_reason_create(2, 0, ptr::null(), 0, &raw mut reason)
    );

    let mut request = ptr::dangling_mut();
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_revoke_request_create(ptr::null(), reason, 2, 0, &raw mut request)
    );
    assert!(request.is_null());
    assert!(reference_handle(reason, Kind::RevocationReason).is_err());
}

fn register_request_fixture() -> ClientRequest {
    let mut opaque_fields = Structure::new();
    opaque_fields
        .try_push(
            Item::new(
                RawTag::new(0x0042_0059)
                    .expect("Opaque Data Type tag fits")
                    .try_checked()
                    .expect("Opaque Data Type is allocated"),
                Value::enumeration(0x8000_0001),
            )
            .expect("Opaque Data Type item is valid"),
        )
        .expect("Opaque Data Type fits the structure");
    opaque_fields
        .try_push(
            Item::new(
                RawTag::new(0x0042_005a)
                    .expect("Opaque Data Value tag fits")
                    .try_checked()
                    .expect("Opaque Data Value is allocated"),
                Value::byte_string(b"register-request-secret".to_vec()),
            )
            .expect("Opaque Data Value item is valid"),
        )
        .expect("Opaque Data Value fits the structure");
    ClientRequest::Register(RegisterRequest::new(
        ObjectType::from_raw(8),
        AttributeSet::try_new([]).expect("generic Opaque Object needs no attributes"),
        Item::new(
            RawTag::new(0x0042_005b)
                .expect("Opaque Object tag fits")
                .try_checked()
                .expect("Opaque Object is allocated"),
            Value::structure(opaque_fields),
        )
        .expect("Opaque Object item is valid"),
    ))
}

fn assert_register_success_fields_absent(item: *mut kmipkit_client_batch_item_response_t) {
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_register(item, &raw mut present, &raw mut identifier,)
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());
    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_register_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
}

#[test]
fn ffi_register_failure_exposes_common_metadata_without_success_fields() {
    let failure = failed_cryptographic_item(register_request_fixture(), 0x03, b"register-failure");
    let mut failure_metadata = kmipkit_client_batch_item_metadata_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_client_batch_item_metadata_v1_t>())
            .expect("metadata size fits ABI field"),
        abi_version: 1,
        outcome_kind: 0,
        result_status: 0,
        has_result_reason: 0,
        result_reason: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_metadata_v1(failure, &raw mut failure_metadata)
    );
    assert_eq!(failure_metadata.outcome_kind, BATCH_OUTCOME_COMPLETED);
    assert_eq!(failure_metadata.result_status, 1);
    assert_eq!(failure_metadata.has_result_reason, 1);
    assert_eq!(failure_metadata.result_reason, 6);
    assert_register_success_fields_absent(failure);
    kmipkit_client_batch_item_response_release(failure);
}

#[test]
fn ffi_register_pending_exposes_correlation_without_success_fields() {
    let correlation = b"register-pending-correlation";
    let pending_id = b"register-pending";
    let pending_response = execute_single_abi_response_with_async(
        register_request_fixture(),
        pending_id,
        pending_response_for_operation(0x03, pending_id, correlation),
        Some(1),
    )
    .expect("the explicitly asynchronous Register batch permits Pending");
    let pending = client_batch_item_response_from(pending_response);
    assert_pending_abi_item_metadata(pending, correlation);
    assert_register_success_fields_absent(pending);
    kmipkit_client_batch_item_response_release(pending);
}

#[test]
fn ffi_recertify_accessors_leave_failure_results_absent_and_validate_outputs() {
    let item = failed_cryptographic_item(
        ClientRequest::recertify(
            RecertifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("old-certificate".to_owned())),
        ),
        0x07,
        b"recertify-failure",
    );

    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_recertify(item, &raw mut present, &raw mut identifier,)
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());

    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_recertify_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());

    let mut invalid_identifier = empty_unique_identifier_output_v1();
    invalid_identifier.struct_size -= 1;
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_recertify(
            item,
            &raw mut present,
            &raw mut invalid_identifier,
        )
    );

    kmipkit_client_batch_item_response_release(item);
}

// KMIP Specification v2.1 §6.1.46, Tables 305–307.
#[test]
fn ffi_rekey_ttlv_is_a_complete_ordered_caller_owned_copy_from_c() {
    let unknown_value = [0xde, 0xad, 0x00, 0xff];
    let item = successful_cryptographic_item(
        ClientRequest::Rekey(
            RekeyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("old-key".to_owned()))
                .with_offset(u32::MAX),
        ),
        0x04,
        b"rekey-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"replacement-key"),
            ttlv_item(0x0054_0001, 0x08, &unknown_value),
        ],
    );

    let mut invalid_identifier = empty_unique_identifier_output_v1();
    invalid_identifier.struct_size -= 1;
    let mut invalid_present = 9;
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_rekey(
            item,
            &raw mut invalid_present,
            &raw mut invalid_identifier,
        )
    );
    assert_eq!(invalid_present, 9);

    let mut identifier_present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey(
            item,
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    assert_eq!(identifier_present, 1);
    assert_response_identifier(&identifier, b"replacement-key");

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    // SAFETY: the compiled C consumer takes ownership of this live response-item handle.
    let result = unsafe { kmipkit_client_abi_c_consumer_validate_rekey_response_ttlv(item) };

    assert_eq!(
        result, 1,
        "C preserves the typed identifier and complete ordered response TTLV"
    );
}

#[test]
fn ffi_rekey_key_pair_returns_both_identifiers_and_compiled_c_owned_ttlv() {
    let unknown = [0xde, 0xad, 0x00, 0xff];
    let item = successful_cryptographic_item(
        ClientRequest::RekeyKeyPair(
            RekeyKeyPairRequest::new().with_private_key_unique_identifier(
                UniqueIdentifier::TextString("existing-private-key".to_owned()),
            ),
        ),
        0x1d,
        b"rekey-pair",
        vec![
            ttlv_item(0x0042_0066, 0x07, b"replacement-private-key"),
            ttlv_item(0x0042_006f, 0x07, b"replacement-public-key"),
            ttlv_item(0x0054_0001, 0x08, &unknown),
        ],
    );
    let mut present = 0;
    let mut private_identifier = empty_unique_identifier_output_v1();
    let mut public_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey_key_pair(
            item,
            &raw mut present,
            &raw mut private_identifier,
            &raw mut public_identifier
        )
    );
    assert_eq!(present, 1);
    assert_response_identifier(&private_identifier, b"replacement-private-key");
    assert_response_identifier(&public_identifier, b"replacement-public-key");
    // SAFETY: the compiled C consumer takes ownership of this live response item.
    assert_eq!(1, unsafe {
        kmipkit_client_abi_c_consumer_validate_rekey_key_pair_response_ttlv(item)
    });
}

#[test]
fn ffi_rekey_key_pair_failure_hides_identifiers_and_response_ttlv() {
    let item = failed_cryptographic_item(
        ClientRequest::RekeyKeyPair(
            RekeyKeyPairRequest::new().with_private_key_unique_identifier(
                UniqueIdentifier::TextString("existing-private-key".to_owned()),
            ),
        ),
        0x1d,
        b"rekey-pair-fail",
    );
    let mut present = 9;
    let mut private_identifier = empty_unique_identifier_output_v1();
    let mut public_identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey_key_pair(
            item,
            &raw mut present,
            &raw mut private_identifier,
            &raw mut public_identifier
        )
    );
    assert_eq!(present, 0);
    assert_eq!(private_identifier.kind, 0);
    assert_eq!(public_identifier.kind, 0);
    assert!(private_identifier.text_data.is_null());
    assert!(public_identifier.text_data.is_null());
    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey_key_pair_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_rekey_accessors_leave_failure_results_absent() {
    let item = failed_cryptographic_item(
        ClientRequest::Rekey(
            RekeyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString("existing-key".to_owned())),
        ),
        0x04,
        b"rekey-failure",
    );

    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey(item, &raw mut present, &raw mut identifier,)
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());

    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_rekey_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    kmipkit_client_batch_item_response_release(item);
}

// OASIS KMIP Specification v2.1 §6.1.48, Tables 313–315.
#[test]
fn ffi_reprovision_preserves_optional_success_identifier_and_owned_ttlv() {
    let unknown = [0xde, 0xad, 0x00, 0xff];
    let item = successful_cryptographic_item(
        ClientRequest::ReProvision(ReProvisionRequest::new()),
        0x35,
        b"reprovision-success",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"replacement-certificate"),
            ttlv_item(0x0054_1234, 0x08, &unknown),
        ],
    );

    let mut invalid_present = 9;
    let mut invalid_identifier_present = 9;
    let mut invalid_identifier = empty_unique_identifier_output_v1();
    invalid_identifier.struct_size -= 1;
    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_reprovision(
            item,
            &raw mut invalid_present,
            &raw mut invalid_identifier_present,
            &raw mut invalid_identifier,
        )
    );
    assert_eq!((invalid_present, invalid_identifier_present), (0, 0));

    let mut present = 0;
    let mut identifier_present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    assert_eq!(present, 1);
    assert_eq!(identifier_present, 1);
    assert_response_identifier(&identifier, b"replacement-certificate");

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    let mut response_view = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_ttlv_structure_view(response_ttlv, &raw mut response_view)
    );
    let mut item_count = 0;
    assert_eq!(
        SUCCESS,
        kmipkit_ttlv_structure_view_item_count(response_view, &raw mut item_count)
    );
    assert_eq!(item_count, 4, "the copy includes common response metadata");
    kmipkit_ttlv_structure_view_release(response_view);
    kmipkit_ttlv_structure_release(response_ttlv);
    // SAFETY: the compiled C caller receives this live item and consumes it after copying TTLV.
    let c_result = unsafe {
        kmipkit_client_abi_c_consumer_validate_reprovision_response(
            item,
            1,
            1,
            b"replacement-certificate".as_ptr(),
            b"replacement-certificate".len() as u64,
        )
    };
    assert_eq!(
        c_result, 1,
        "compiled C can copy and release the successful result"
    );
}

#[test]
fn ffi_reprovision_keeps_success_present_when_optional_identifier_is_absent() {
    let item = successful_cryptographic_item(
        ClientRequest::ReProvision(ReProvisionRequest::new()),
        0x35,
        b"reprovision-without-identifier",
        vec![ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad, 0x00, 0xff])],
    );

    let mut present = 0;
    let mut identifier_present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    assert_eq!(present, 1);
    assert_eq!(identifier_present, 0);
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());

    let mut ttlv_present = 0;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);
    // SAFETY: the compiled C caller receives this live item and consumes it after copying TTLV.
    let c_result = unsafe {
        kmipkit_client_abi_c_consumer_validate_reprovision_response(item, 1, 0, ptr::null(), 0)
    };
    assert_eq!(
        c_result, 1,
        "compiled C preserves successful absence of the identifier"
    );
}

#[test]
fn ffi_reprovision_rejects_overlapping_scalar_outputs() {
    let item = successful_cryptographic_item(
        ClientRequest::ReProvision(ReProvisionRequest::new()),
        0x35,
        b"reprovision-overlapping-outputs",
        vec![ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad, 0x00, 0xff])],
    );
    let mut aliased_output = 9;
    let mut identifier = empty_unique_identifier_output_v1();

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_reprovision(
            item,
            &raw mut aliased_output,
            &raw mut aliased_output,
            &raw mut identifier,
        )
    );
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_reprovision_rejects_scalar_output_overlapping_identifier_struct() {
    let item = successful_cryptographic_item(
        ClientRequest::ReProvision(ReProvisionRequest::new()),
        0x35,
        b"reprovision-overlapping-identifier-output",
        vec![ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad, 0x00, 0xff])],
    );
    let mut identifier = empty_unique_identifier_output_v1();
    let mut identifier_present = 9;

    assert_eq!(
        ERROR_INVALID_INPUT,
        kmipkit_client_batch_item_response_reprovision(
            item,
            ptr::addr_of_mut!(identifier.kind).cast::<u8>(),
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_reprovision_failure_exposes_neither_success_identifier_nor_ttlv() {
    let item = failed_cryptographic_item(
        ClientRequest::ReProvision(ReProvisionRequest::new()),
        0x35,
        b"reprovision-failure",
    );

    let mut present = 9;
    let mut identifier_present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    assert_eq!((present, identifier_present), (0, 0));
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());

    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    // SAFETY: the compiled C caller receives this live Failure item and releases it.
    let c_result = unsafe {
        kmipkit_client_abi_c_consumer_validate_reprovision_response(item, 0, 0, ptr::null(), 0)
    };
    assert_eq!(
        c_result, 1,
        "compiled C exposes neither typed data nor TTLV on Failure"
    );
}

#[test]
fn ffi_reprovision_pending_exposes_neither_success_identifier_nor_ttlv() {
    let batch_item_id = b"reprovision-pending";
    let correlation = [0x00, 0xff, 0x80, 0x7f];
    let response = execute_single_abi_response_with_async(
        ClientRequest::ReProvision(ReProvisionRequest::new()),
        batch_item_id,
        pending_response_for_operation(0x35, batch_item_id, &correlation),
        Some(1),
    )
    .expect("the async Re-Provision request permits a Pending response");
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
    let mut identifier_present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision(
            item,
            &raw mut present,
            &raw mut identifier_present,
            &raw mut identifier,
        )
    );
    assert_eq!((present, identifier_present), (0, 0));
    assert_eq!(identifier.kind, 0);
    assert!(identifier.text_data.is_null());

    let mut ttlv_present = 9;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_reprovision_ttlv(
            item,
            &raw mut ttlv_present,
            &raw mut response_ttlv,
        )
    );
    assert_eq!(ttlv_present, 0);
    assert!(response_ttlv.is_null());
    // SAFETY: the compiled C caller receives this live Pending item and releases it.
    let c_result = unsafe {
        kmipkit_client_abi_c_consumer_validate_reprovision_response(item, 0, 0, ptr::null(), 0)
    };
    assert_eq!(
        c_result, 1,
        "compiled C exposes neither typed data nor TTLV on Pending"
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
fn ffi_usage_allocation_context_survives_source_item_release_in_c() {
    let item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(12)),
        0x11,
        b"usage-context-c-consumer",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );

    // SAFETY: the compiled C consumer takes ownership of this live response-item handle.
    let result = unsafe { kmipkit_client_abi_c_consumer_validate_usage_allocation_context(item) };

    assert_eq!(
        result, 1,
        "C creates a context, releases its source item, then reads the copied metadata"
    );
}

#[test]
// Keep fixture parsing, mTLS setup, the FFI call, and handle cleanup together
// so the shared test inputs and native resource lifetimes stay explicit.
#[allow(clippy::too_many_lines)]
fn ffi_c_consumer_executes_guarded_encrypt_over_mtls_and_reserves_cost() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("shared API parity fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("shared fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "usage-allocation.guard-accounting.v1")
        .expect("shared Usage Allocation fixture case exists");
    let input = &case["input"];
    let expected = &case["expectedOutput"];
    let allocation_identifier = input["allocationIdentifier"]
        .as_str()
        .expect("allocation identifier is text");
    let allocation_count = input["allocationCount"]
        .as_i64()
        .expect("allocation count is signed");
    let usage_limits_unit = u32::try_from(
        input["usageLimitsUnit"]
            .as_u64()
            .expect("usage limits unit is unsigned"),
    )
    .expect("usage limits unit fits U32");
    let successful_cost = input["successfulCost"]
        .as_i64()
        .expect("successful cost is signed");
    let over_budget_cost = input["overBudgetCost"]
        .as_i64()
        .expect("over-budget cost is signed");
    let operation = match input["operation"]
        .as_str()
        .expect("protection operation is text")
    {
        "Encrypt" => 0x1F,
        other => panic!("unsupported protection operation {other}"),
    };

    let allocation_response = successful_response_without_batch_item_id(
        0x11,
        vec![ttlv_item(
            0x0042_0094,
            0x07,
            allocation_identifier.as_bytes(),
        )],
    );
    let response = successful_cryptographic_response(
        operation,
        allocation_identifier.as_bytes(),
        vec![
            ttlv_item(0x0042_0094, 0x07, allocation_identifier.as_bytes()),
            ttlv_item(0x0042_00c2, 0x08, b"ciphertext"),
        ],
    );
    let (client, peer, exchange_count) = client_with_responses(vec![allocation_response, response]);
    let client: *mut kmipkit_client_t = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(std::sync::Mutex::new(client))),
    );
    let request = ClientRequest::Encrypt(EncryptRequest::new(
        Some(UniqueIdentifier::TextString(
            allocation_identifier.to_owned(),
        )),
        Some(OperationData::ByteString(SecretBytes::new(
            b"plaintext".to_vec(),
        ))),
    ));
    let mut item = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(Box::new(
            ClientBatchItem::new(request)
                .with_unique_batch_item_id(allocation_identifier.as_bytes().to_vec()),
        )),
    );
    let limits = default_codec_limits_handle();

    let initial = &expected["initial"];
    let after_success = &expected["afterSuccessfulProtection"];
    // SAFETY: the fixture-backed identifier and scalar values remain live for
    // this synchronous call; C executes both exchanges and releases its context.
    let result = unsafe {
        shared_usage_allocation_guard_matches_fixture(
            client.cast(),
            allocation_identifier.as_bytes().as_ptr(),
            u64::try_from(allocation_identifier.len()).expect("identifier length fits u64"),
            allocation_count,
            usage_limits_unit,
            successful_cost,
            over_budget_cost,
            initial["allocated"]
                .as_i64()
                .expect("fixture allocated units are signed"),
            initial["reserved"]
                .as_i64()
                .expect("fixture reserved units are signed"),
            initial["available"]
                .as_i64()
                .expect("fixture available units are signed"),
            after_success["allocated"]
                .as_i64()
                .expect("fixture allocated units are signed"),
            after_success["reserved"]
                .as_i64()
                .expect("fixture reserved units are signed"),
            after_success["available"]
                .as_i64()
                .expect("fixture available units are signed"),
            &raw mut item,
            limits,
        )
    };
    release(client, Kind::Client);
    release(limits, Kind::CodecLimits);
    peer.join()
        .expect("the mTLS peer completes the guarded exchange");

    assert_eq!(
        result,
        1,
        "C executes Encrypt and observes its reservation; peer exchanges={}",
        exchange_count.load(Ordering::Relaxed)
    );
    assert!(item.is_null(), "guarded execution consumes the item handle");
    assert_eq!(case["errorCategory"].as_str(), Some("invalid_input"));
    assert!(expected["localOnlyAccounting"].as_bool().unwrap());
    assert_eq!(expected["overBudgetAttempt"]["accepted"], false);
    assert_eq!(expected["overBudgetAttempt"]["stateUnchanged"], true);
    assert_eq!(
        exchange_count.load(Ordering::Relaxed),
        expected["transportExchangeCount"]
            .as_u64()
            .expect("fixture exchange count is unsigned")
    );
}

#[test]
fn ffi_guarded_encrypt_pending_retains_context_and_returns_pending_metadata() {
    let allocation_item = successful_cryptographic_item(
        ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(5)),
        0x11,
        b"guarded-pending-allocation",
        vec![ttlv_item(0x0042_0094, 0x07, b"allocation-object")],
    );
    let mut context = ptr::null_mut();
    assert_eq!(
        kmipkit_usage_allocation_context_create(allocation_item, 5, u32::MAX, &raw mut context,),
        SUCCESS
    );
    kmipkit_client_batch_item_response_release(allocation_item);

    let batch_id = b"guarded-pending-item";
    let correlation = [0x00, 0xFF, 0x80, 0x7F];
    let response = pending_response_for_operation(0x1f, batch_id, &correlation);
    let (client_value, peer) = client_with_one_response(response);
    let client = make_handle(
        Kind::Client,
        HandleValue::Client(Box::new(std::sync::Mutex::new(client_value))),
    );
    let request = ClientRequest::Encrypt(EncryptRequest::new(
        Some(UniqueIdentifier::TextString("allocation-object".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"guarded-plaintext".to_vec(),
        ))),
    ));
    let mut item = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(Box::new(
            ClientBatchItem::new(request).with_unique_batch_item_id(batch_id.to_vec()),
        )),
    );
    let limits = default_codec_limits_handle();
    let mut response = ptr::null_mut();

    assert_eq!(
        client_abi::kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator(
            client,
            context,
            &raw mut item,
            2,
            1,
            limits,
            &raw mut response,
        ),
        SUCCESS
    );
    peer.join()
        .expect("the mTLS peer completes the guarded exchange");
    assert!(
        item.is_null(),
        "guarded FFI execution consumes the item slot"
    );
    assert!(!response.is_null());

    let mut response_item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response, 0, &raw mut response_item),
        SUCCESS
    );
    assert_pending_abi_item_metadata(response_item, &correlation);

    let mut reserved_units = 0;
    let mut available_units = 0;
    assert_eq!(
        kmipkit_usage_allocation_context_reserved_units(context, &raw mut reserved_units),
        SUCCESS
    );
    assert_eq!(
        kmipkit_usage_allocation_context_available_units(context, &raw mut available_units),
        SUCCESS
    );
    assert_eq!(reserved_units, 2);
    assert_eq!(available_units, 3);

    kmipkit_client_batch_item_response_release(response_item);
    kmipkit_client_batch_response_release(response);
    kmipkit_client_release(client);
    kmipkit_usage_allocation_context_release(context);
    release(limits, Kind::CodecLimits);
}

fn assert_pending_abi_item_metadata(
    response_item: *mut kmipkit_client_batch_item_response_t,
    expected_correlation: &[u8],
) {
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
        kmipkit_client_batch_item_response_metadata_v1(response_item, &raw mut metadata),
        SUCCESS
    );
    assert_eq!(metadata.outcome_kind, BATCH_OUTCOME_PENDING);
    assert_eq!(metadata.result_status, 2);

    let mut present = 0;
    let mut correlation_span = byte_span(&[]);
    assert_eq!(
        kmipkit_client_batch_item_response_pending_correlation_value(
            response_item,
            &raw mut present,
            &raw mut correlation_span,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_response_span(correlation_span, expected_correlation);
}

// KMIP Specification v2.1 §6.1.27, Tables 244–246.
#[test]
fn ffi_join_split_key_ttlv_is_a_complete_ordered_caller_owned_copy_from_c() {
    let unknown_value = [0xde, 0xad, 0x00, 0xff];
    let request = JoinSplitKeyRequest::new(
        ObjectType::from_raw(7),
        vec![
            SplitKeyReference::new(UniqueIdentifier::TextString("repeated-id".to_owned()))
                .with_split_key_threshold(1),
            SplitKeyReference::new(UniqueIdentifier::TextString("repeated-id".to_owned()))
                .with_split_key_threshold(2),
        ],
    );
    let item = successful_cryptographic_item(
        ClientRequest::join_split_key(request),
        0x29,
        b"join-split-key",
        vec![
            ttlv_item(0x0042_0094, 0x07, b"joined-id"),
            ttlv_item(0x0054_0001, 0x08, &unknown_value),
        ],
    );

    // SAFETY: the compiled C consumer receives this live response-item handle and takes ownership.
    let result =
        unsafe { kmipkit_client_abi_c_consumer_validate_join_split_key_response_ttlv(item) };

    assert_eq!(
        result, 1,
        "C reads the returned identifier and complete ordered TTLV after releasing the source item"
    );
}

#[test]
fn ffi_join_split_key_bounds_aggregate_repeated_identifier_bytes() {
    let identifier_length = usize::try_from(MAX_FFI_INPUT_BYTES / 2 + 1)
        .expect("test identifier length fits the platform");
    let identifier_text = vec![b'x'; identifier_length];
    let identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier record fits the ABI size field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: identifier_text.as_ptr(),
        text_length: u64::try_from(identifier_text.len()).expect("test length fits u64"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let mut reference = ptr::null_mut();
    assert_eq!(
        kmipkit_split_key_reference_create(&identifier, 1, &mut reference),
        SUCCESS,
        "one individually valid large identifier creates a reference"
    );

    let mut single_reference = [reference];
    let mut item = ptr::null_mut();
    let single_status = kmipkit_client_batch_item_join_split_key(
        7,
        single_reference.as_mut_ptr(),
        1,
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut item,
    );
    kmipkit_client_batch_item_release(item);

    let mut repeated_references = [reference, reference];
    item = ptr::null_mut();
    let repeated_status = kmipkit_client_batch_item_join_split_key(
        7,
        repeated_references.as_mut_ptr(),
        2,
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut item,
    );
    kmipkit_client_batch_item_release(item);
    kmipkit_split_key_reference_release(reference);

    assert_eq!(
        single_status, SUCCESS,
        "the under-budget request remains valid"
    );
    assert_eq!(
        repeated_status, ERROR_RESOURCE_LIMIT,
        "repeated occurrences are rejected before aggregate cloning exceeds the FFI budget"
    );
    assert!(
        item.is_null(),
        "rejected requests do not create a batch item"
    );
}

#[test]
fn ffi_join_split_key_request_preserves_enumeration_and_integer_identifiers() {
    let mut enumeration_identifier = empty_unique_identifier_output_v1();
    enumeration_identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION;
    enumeration_identifier.enumeration_value = u32::MAX;
    let mut integer_identifier = empty_unique_identifier_output_v1();
    integer_identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    integer_identifier.integer_value = i32::MIN;

    let mut enumeration_reference = ptr::null_mut();
    assert_eq!(
        kmipkit_split_key_reference_create(
            &raw const enumeration_identifier,
            1,
            &raw mut enumeration_reference,
        ),
        SUCCESS
    );
    let mut integer_reference = ptr::null_mut();
    assert_eq!(
        kmipkit_split_key_reference_create(
            &raw const integer_identifier,
            2,
            &raw mut integer_reference,
        ),
        SUCCESS
    );

    let mut references = [enumeration_reference, integer_reference];
    let mut item = ptr::null_mut();
    let status = kmipkit_client_batch_item_join_split_key(
        7,
        references.as_mut_ptr(),
        2,
        0,
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        &raw mut item,
    );

    kmipkit_client_batch_item_release(item);
    kmipkit_split_key_reference_release(integer_reference);
    kmipkit_split_key_reference_release(enumeration_reference);

    assert_eq!(status, SUCCESS);
    assert!(!item.is_null());
}

#[test]
fn ffi_join_split_key_request_rejects_non_boolean_secret_data_presence() {
    let mut identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = 42;
    let mut reference = ptr::null_mut();
    assert_eq!(
        kmipkit_split_key_reference_create(&raw const identifier, 1, &raw mut reference),
        SUCCESS
    );

    let mut references = [reference];
    let mut item = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_join_split_key(
            7,
            references.as_mut_ptr(),
            1,
            2,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    kmipkit_split_key_reference_release(reference);
}

#[test]
fn ffi_split_key_reference_rejects_a_missing_identifier() {
    let mut reference = ptr::dangling_mut();

    assert_eq!(
        kmipkit_split_key_reference_create(ptr::null(), 1, &raw mut reference),
        ERROR_INVALID_INPUT
    );
    assert!(reference.is_null());
}

#[test]
fn ffi_join_split_key_rejects_missing_references_and_wrong_optional_handle_kinds() {
    let mut item = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_join_split_key(
            7,
            ptr::null_mut(),
            0,
            0,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());

    let mut identifier = empty_unique_identifier_output_v1();
    identifier.kind = crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER;
    identifier.integer_value = 42;
    let mut reference = ptr::null_mut();
    assert_eq!(
        kmipkit_split_key_reference_create(&raw const identifier, 1, &raw mut reference),
        SUCCESS
    );
    let mut references = [reference];
    let wrong_attributes = unconnected_client_handle();
    assert_eq!(
        kmipkit_client_batch_item_join_split_key(
            7,
            references.as_mut_ptr(),
            1,
            0,
            0,
            wrong_attributes.cast(),
            ptr::null_mut(),
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    kmipkit_client_release(wrong_attributes);

    let wrong_masks = unconnected_client_handle();
    assert_eq!(
        kmipkit_client_batch_item_join_split_key(
            7,
            references.as_mut_ptr(),
            1,
            0,
            0,
            ptr::null_mut(),
            wrong_masks.cast(),
            &raw mut item,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(item.is_null());
    kmipkit_client_release(wrong_masks);
    kmipkit_split_key_reference_release(reference);
}

#[test]
fn ffi_join_split_key_response_identifier_getter_fails_closed_for_invalid_and_failed_items() {
    let failed_item = failed_join_split_key_response_item();
    let response_owner = reference_handle(failed_item, Kind::ClientBatchItemResponse)
        .expect("the failed response item remains a live handle");
    let HandleValue::ClientBatchItemResponse { response, .. } = &response_owner.value else {
        panic!("the test helper returns a response-item handle");
    };
    let response = Arc::clone(response);

    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    let mut present = 9;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key(
            mismatched_item,
            &raw mut present,
            &raw mut identifier,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(&response),
            index: usize::MAX,
        },
    );
    present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key(
            invalid_index_item,
            &raw mut present,
            &raw mut identifier,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);

    let mut invalid_output = empty_unique_identifier_output_v1();
    invalid_output.abi_version = 2;
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key(
            failed_item,
            &raw mut present,
            &raw mut invalid_output,
        ),
        ERROR_INVALID_INPUT
    );

    present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key(
            failed_item,
            &raw mut present,
            &raw mut identifier,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert_eq!(identifier.kind, 0);

    kmipkit_client_batch_item_response_release(invalid_index_item);
    kmipkit_client_batch_item_response_release(mismatched_item);
    kmipkit_client_batch_item_response_release(failed_item);
}

#[test]
fn ffi_join_split_key_response_ttlv_getter_fails_closed_for_invalid_and_failed_items() {
    let failed_item = failed_join_split_key_response_item();
    let response_owner = reference_handle(failed_item, Kind::ClientBatchItemResponse)
        .expect("the failed response item remains a live handle");
    let HandleValue::ClientBatchItemResponse { response, .. } = &response_owner.value else {
        panic!("the test helper returns a response-item handle");
    };
    let response = Arc::clone(response);
    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(&response),
            index: usize::MAX,
        },
    );
    let mut response_ttlv = ptr::null_mut();
    let mut present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key_ttlv(
            mismatched_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());

    present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key_ttlv(
            invalid_index_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());

    present = 9;
    assert_eq!(
        kmipkit_client_batch_item_response_join_split_key_ttlv(
            failed_item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());

    kmipkit_client_batch_item_response_release(invalid_index_item);
    kmipkit_client_batch_item_response_release(mismatched_item);
    kmipkit_client_batch_item_response_release(failed_item);
}

fn failed_join_split_key_response_item() -> *mut kmipkit_client_batch_item_response_t {
    let request = ClientRequest::join_split_key(JoinSplitKeyRequest::new(
        ObjectType::from_raw(7),
        vec![
            SplitKeyReference::new(UniqueIdentifier::TextString("source-fragment".to_owned()))
                .with_split_key_threshold(1),
        ],
    ));
    failed_cryptographic_item(request, 0x29, b"join-split-failed")
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

    assert_recertify_result_getters_return_status(item, expected_status);
    assert_symmetric_crypto_result_getters_return_status(item, expected_status);
}

fn assert_recertify_result_getters_return_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 0;
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_recertify(item, &raw mut present, &raw mut identifier,),
        expected_status
    );
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_recertify_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        expected_status
    );
}

fn assert_symmetric_crypto_result_getters_return_status(
    item: *mut kmipkit_client_batch_item_response_t,
    expected_status: i32,
) {
    let mut present = 0;
    let mut correlation_present = 0;
    let mut correlation = byte_span(&[]);
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 0;
    let mut data = byte_span(&[]);
    let mut iv_present = 0;
    let mut iv = byte_span(&[]);
    let mut tag_present = 0;
    let mut tag = byte_span(&[]);
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_encrypt(
            item,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut iv_present,
            &raw mut iv,
            &raw mut correlation_present,
            &raw mut correlation,
            &raw mut tag_present,
            &raw mut tag,
        ),
        expected_status
    );

    output_identifier = empty_unique_identifier_output_v1();
    data_present = 0;
    data = byte_span(&[]);
    correlation_present = 0;
    correlation = byte_span(&[]);
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_decrypt(
            item,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
            &raw mut correlation_present,
            &raw mut correlation,
        ),
        expected_status
    );
}

fn pkcs11_function_structure() -> Structure {
    let mut request_function = Structure::new();
    request_function
        .try_push(
            Item::new(
                RawTag::new(0x0042_015A)
                    .expect("PKCS#11 Function tag fits the KMIP tag field")
                    .try_checked()
                    .expect("PKCS#11 Function is allocated in KMIP 2.1"),
                Value::enumeration(0xFFFF_1234),
            )
            .expect("the PKCS#11 Function item fits the test model"),
        )
        .expect("the PKCS#11 request Function fits the test structure");
    request_function
}

fn successful_pkcs11_response_item() -> *mut kmipkit_client_batch_item_response_t {
    let correlation = b"pkcs11-ffi-response-correlation";
    successful_cryptographic_item(
        ClientRequest::Pkcs11(Pkcs11Request::new(pkcs11_function_structure())),
        0x33,
        b"pkcs11-ffi-response",
        vec![
            ttlv_structure(
                0x0042_0159,
                [ttlv_item(0x0054_1234, 0x08, b"opaque-interface")],
            ),
            ttlv_structure(0x0042_015A, [ttlv_enumeration(0x0042_015A, 0xFFFF_1234)]),
            ttlv_item(0x0042_00D6, 0x08, correlation),
            ttlv_structure(
                0x0042_015C,
                [ttlv_item(0x0054_1235, 0x08, b"opaque-output")],
            ),
            ttlv_structure(0x0042_015D, [ttlv_enumeration(0x0042_015D, 0x8FFF_1234)]),
        ],
    )
}

fn assert_present_pkcs11_structure(
    getter: extern "C" fn(
        *mut crate::kmipkit_client_batch_item_response_t,
        *mut u8,
        *mut *mut crate::kmipkit_ttlv_structure_t,
    ) -> i32,
    item: *mut crate::kmipkit_client_batch_item_response_t,
) {
    let mut present = 0;
    let mut structure = ptr::null_mut();
    assert_eq!(getter(item, &raw mut present, &raw mut structure), SUCCESS);
    assert_eq!(present, 1);
    assert!(!structure.is_null());
    kmipkit_ttlv_structure_release(structure);
}

fn assert_absent_pkcs11_structure(
    getter: extern "C" fn(
        *mut crate::kmipkit_client_batch_item_response_t,
        *mut u8,
        *mut *mut crate::kmipkit_ttlv_structure_t,
    ) -> i32,
    item: *mut crate::kmipkit_client_batch_item_response_t,
) {
    let mut present = u8::MAX;
    let mut structure = ptr::dangling_mut();
    assert_eq!(getter(item, &raw mut present, &raw mut structure), SUCCESS);
    assert_eq!(present, 0);
    assert!(structure.is_null());
}

fn assert_pkcs11_structure_getters_absent(item: *mut crate::kmipkit_client_batch_item_response_t) {
    assert_absent_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_interface_ttlv,
        item,
    );
    assert_absent_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_function_ttlv,
        item,
    );
    assert_absent_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv,
        item,
    );
    assert_absent_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_return_code_ttlv,
        item,
    );
}

#[test]
fn ffi_pkcs11_response_getters_copy_optional_values() {
    let item = successful_pkcs11_response_item();
    let correlation = b"pkcs11-ffi-response-correlation";

    let mut present = u8::MAX;
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);

    assert_present_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_interface_ttlv,
        item,
    );
    assert_present_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_function_ttlv,
        item,
    );
    assert_present_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_output_parameters_ttlv,
        item,
    );
    assert_present_pkcs11_structure(
        client_abi::kmipkit_client_batch_item_response_pkcs11_return_code_ttlv,
        item,
    );

    let mut correlation_span = byte_span(&[]);
    present = 0;
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
            item,
            &raw mut present,
            &raw mut correlation_span,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert_response_span(correlation_span, correlation);

    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_pkcs11_response_ttlv_rejects_mismatched_handle() {
    let item = successful_pkcs11_response_item();
    let handle = reference_handle(item, Kind::ClientBatchItemResponse)
        .expect("the PKCS#11 item is registered");
    let HandleValue::ClientBatchItemResponse { response, .. } = &handle.value else {
        panic!("the response item handle stores its response");
    };
    let mismatched = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(response)),
    );
    let mut present = u8::MAX;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_ttlv(
            mismatched,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());

    let mut interface = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_interface_ttlv(
            mismatched,
            &raw mut present,
            &raw mut interface,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(interface.is_null());

    kmipkit_client_batch_item_response_release(mismatched);
    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_pkcs11_response_getters_return_absent_for_other_operations() {
    let response = Arc::new(
        create_key_pair_success_response()
            .expect("the successful Create Key Pair response matches its request"),
    );
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(response),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item,),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);

    let mut present = u8::MAX;
    let mut response_ttlv = ptr::dangling_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(response_ttlv.is_null());
    assert_pkcs11_structure_getters_absent(item);

    let mut correlation = byte_span(&[]);
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
            item,
            &raw mut present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(correlation.data.is_null());
    assert_eq!(correlation.length, 0);
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
            item,
            &raw mut present,
            ptr::null_mut(),
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);

    kmipkit_client_batch_item_response_release(item);
}

#[test]
fn ffi_pkcs11_response_getters_preserve_absence_for_failure_result() {
    let item = failed_cryptographic_item(
        ClientRequest::Pkcs11(Pkcs11Request::new(pkcs11_function_structure())),
        0x33,
        b"pkcs11-failed-response",
    );

    assert_pkcs11_structure_getters_absent(item);
    let mut present = u8::MAX;
    let mut correlation = byte_span(&[]);
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_correlation_value(
            item,
            &raw mut present,
            &raw mut correlation,
        ),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(correlation.data.is_null());
    assert_eq!(correlation.length, 0);

    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        client_abi::kmipkit_client_batch_item_response_pkcs11_ttlv(
            item,
            &raw mut present,
            &raw mut response_ttlv,
        ),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!response_ttlv.is_null());
    kmipkit_ttlv_structure_release(response_ttlv);
    kmipkit_client_batch_item_response_release(item);
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

#[test]
fn ffi_import_getters_reject_invalid_output_records_and_inconsistent_handles() {
    let mut present = 9;
    let mut invalid_identifier = empty_unique_identifier_output_v1();
    invalid_identifier.struct_size -= 1;
    assert_eq!(
        kmipkit_client_batch_item_response_import(
            ptr::null_mut(),
            &mut present,
            &mut invalid_identifier,
        ),
        ERROR_INVALID_INPUT
    );

    let response = Arc::new(
        create_key_pair_success_response()
            .expect("the valid successful typed response is associated with its request"),
    );
    let mismatched_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchResponse(Arc::clone(&response)),
    );
    let mut identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_import(mismatched_item, &mut present, &mut identifier),
        ERROR_INVALID_INPUT
    );
    let mut response_ttlv = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_import_ttlv(
            mismatched_item,
            &mut present,
            &mut response_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    kmipkit_client_batch_item_response_release(mismatched_item);

    let invalid_index_item = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response,
            index: usize::MAX,
        },
    );
    identifier = empty_unique_identifier_output_v1();
    assert_eq!(
        kmipkit_client_batch_item_response_import(
            invalid_index_item,
            &mut present,
            &mut identifier
        ),
        ERROR_INTERNAL
    );
    assert_eq!(
        kmipkit_client_batch_item_response_import_ttlv(
            invalid_index_item,
            &mut present,
            &mut response_ttlv,
        ),
        ERROR_INTERNAL
    );
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
fn ffi_create_request_accepts_protection_storage_masks() {
    let attributes = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let masks = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut item = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_batch_item_create(1, attributes, masks, &raw mut item),
        SUCCESS
    );
    assert!(!item.is_null());
    kmipkit_client_batch_item_release(item);
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
fn ffi_execute_with_options_and_error_ttlv_rejects_invalid_options_before_consuming_batch() {
    let client = unconnected_client_handle();
    let limits = default_codec_limits_handle();
    let mut batch = empty_client_batch_handle();
    let mut options = request_options(
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
        timeout(0, 0, 0),
    );
    options.abi_version = 2;
    let mut response = ptr::dangling_mut();
    let mut error_response = ptr::dangling_mut();

    assert_eq!(
        kmipkit_client_execute_with_options_and_error_ttlv_v1(
            client,
            &mut batch,
            limits,
            &raw const options,
            &raw mut response,
            &raw mut error_response,
        ),
        ERROR_INVALID_INPUT
    );
    assert!(!batch.is_null());
    assert!(response.is_null());
    assert!(error_response.is_null());
    kmipkit_client_batch_release(batch);
    kmipkit_codec_limits_release(limits);
    kmipkit_client_release(client);
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

#[test]
fn pkcs11_correlation_span_distinguishes_absence_from_present_empty() {
    let mut present = u8::MAX;
    let mut span = crate::kmipkit_byte_span_v1_t {
        data: ptr::dangling(),
        length: u64::MAX,
    };

    super::set_borrowed_span(&mut present, &mut span, None)
        .expect("an absent correlation value clears its span");
    assert_eq!(present, 0);
    assert!(span.data.is_null());
    assert_eq!(span.length, 0);

    super::set_borrowed_span(&mut present, &mut span, Some(&[]))
        .expect("a present empty correlation value has a valid span");
    assert_eq!(present, 1);
    assert!(span.data.is_null());
    assert_eq!(span.length, 0);
}
