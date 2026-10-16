#![allow(unsafe_code)]
#![cfg(feature = "coverage-c-consumer")]

use std::ffi::{CString, c_char};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use kmipkit_ffi::kmipkit_codec_limits_t;
use kmipkit_test_support::EphemeralPki;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

const MAX_TEST_TTLV_VALUE_BYTES: usize = 1_048_576;
static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[link(name = "kmipkit_extension_registry_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_extension_registry_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
}

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
    fn kmipkit_client_abi_c_consumer_execute_tls(
        argc: i32,
        argv: *mut *mut c_char,
        limits: *mut kmipkit_codec_limits_t,
    ) -> i32;
}

#[link(name = "kmipkit_ffi")]
unsafe extern "C" {
    fn kmipkit_codec_limits_defaults(out_limits: *mut *mut kmipkit_codec_limits_t) -> i32;
    fn kmipkit_codec_limits_release(limits: *mut kmipkit_codec_limits_t);
}

#[test]
fn existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/extensions/cases.json")
        .canonicalize()
        .expect("shared C consumer fixture exists");
    let executable = CString::new("kmipkit_extension_registry_consumer")
        .expect("test executable name has no NUL byte");
    let fixture = CString::new(fixture.to_str().expect("fixture path is UTF-8"))
        .expect("fixture path has no NUL byte");
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let server_config = mtls_server_config(&pki);
    let retry_server_config = Arc::clone(&server_config);
    let peer = thread::spawn(move || {
        controlled_mtls_peer(&listener, server_config, retry_server_config);
    });
    let mut arguments = [
        executable.as_ptr().cast_mut(),
        fixture.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    let argument_count =
        i32::try_from(arguments.len()).expect("the C consumer argument count fits in i32");
    let mut limits = std::ptr::null_mut();

    // SAFETY: `limits` is a valid writable output slot for the duration of this call.
    let status = unsafe { kmipkit_codec_limits_defaults(&raw mut limits) };
    assert_eq!(status, 0, "codec limits defaults must succeed");
    assert!(
        !limits.is_null(),
        "codec limits defaults must return an owned handle"
    );

    // SAFETY: a successful defaults call returned this live owned handle, which is released once.
    unsafe {
        kmipkit_codec_limits_release(limits);
    }

    // SAFETY: both C strings and the mutable argument array remain live for the entire call.
    let result = unsafe {
        kmipkit_extension_registry_c_consumer_main(argument_count, arguments.as_mut_ptr())
    };

    assert_eq!(result, 0, "the existing C ABI behavior suite must pass");
    peer.join().expect("the loopback TLS peer completes");
}

#[test]
fn generated_public_client_abi_consumer_compiles_and_runs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "api-levels.shared.v1")
        .expect("the API-level fixture case exists");
    let expected_output = &case["expectedOutput"];
    assert_eq!(case["capability"], "api_level_discovery");
    assert_eq!(expected_output["networkCalls"], 0);
    assert_eq!(
        expected_output["distinctFromKmipOperation"],
        "Discover Versions"
    );
    assert_eq!(
        expected_output["operationMappingCompleteness"],
        "not-asserted"
    );
    let expected_levels = expected_output["apiLevels"]
        .as_array()
        .expect("fixture API levels are an array")
        .iter()
        .map(|level| {
            CString::new(level.as_str().expect("API level IDs are strings"))
                .expect("API level IDs have no NUL byte")
        })
        .collect::<Vec<_>>();
    assert_eq!(expected_levels.len(), 3);
    let executable = CString::new("kmipkit_public_client_consumer")
        .expect("test executable name has no NUL byte");
    let mut arguments = vec![executable.as_ptr().cast_mut()];
    arguments.extend(
        expected_levels
            .iter()
            .map(|level| level.as_ptr().cast_mut()),
    );
    let argument_count = i32::try_from(arguments.len()).expect("argument count fits in i32");
    // SAFETY: the compiled consumer includes the generated public header, receives
    // one live argv string, and releases every created handle before returning.
    let result =
        unsafe { kmipkit_client_abi_c_consumer_main(argument_count, arguments.as_mut_ptr()) };

    assert_eq!(
        result, 0,
        "generated public C client ABI consumer must pass its implementation checks"
    );
}

#[test]
fn generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let server_config = mtls_server_config(&pki);
    let exchange_count = Arc::new(AtomicU64::new(0));
    let peer_exchange_count = Arc::clone(&exchange_count);
    let peer = thread::spawn(move || {
        let (stream, _) = listener
            .accept()
            .expect("the public C client connects to the local peer");
        peer_exchange_count.fetch_add(1, Ordering::Relaxed);
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls).expect("bounded request frame arrives");
        assert!(request.starts_with(&[0x42, 0x00, 0x78]));
        tls.write_all(&mixed_discover_versions_response())
            .expect("the typed KMIP response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    });

    let executable = CString::new("kmipkit_public_client_consumer")
        .expect("test executable name has no NUL byte");
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let mut arguments = [
        executable.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    let argument_count = i32::try_from(arguments.len()).expect("argument count fits in i32");
    let mut limits = std::ptr::null_mut();
    // SAFETY: `limits` is a writable output slot and the successful handle is
    // retained for the C call, then released by this Rust test harness.
    let status = unsafe { kmipkit_codec_limits_defaults(&raw mut limits) };
    assert_eq!(status, 0, "codec limits defaults must succeed");
    assert!(
        !limits.is_null(),
        "codec limits defaults must return a handle"
    );
    // SAFETY: all argv entries remain live and NUL terminated for the synchronous
    // C call; the consumer is compiled against the generated public header only.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_execute_tls(argument_count, arguments.as_mut_ptr(), limits)
    };
    // SAFETY: the defaults call returned this live handle and the C consumer only
    // borrowed it while synchronously executing the batch.
    unsafe {
        kmipkit_codec_limits_release(limits);
    }

    assert_eq!(
        result,
        0,
        "generated public C consumer must create a client and execute the typed batch; server exchanges: {}",
        exchange_count.load(Ordering::Relaxed)
    );
    peer.join().expect("the loopback TLS peer completes");
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
}

fn controlled_mtls_peer(
    listener: &TcpListener,
    server_config: Arc<ServerConfig>,
    retry_server_config: Arc<ServerConfig>,
) {
    let (stream, _) = listener
        .accept()
        .expect("the C client connects to the local peer");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("server read timeout is configured");
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("server write timeout is configured");
    let connection =
        ServerConnection::new(server_config).expect("the TLS 1.3 server connection is configured");
    let mut tls = StreamOwned::new(connection, stream);
    let request = read_ttlv_frame(&mut tls).expect("bounded request frame arrives");
    assert!(request.starts_with(&[0x42, 0x00, 0x78]));
    tls.write_all(&discover_versions_response())
        .expect("the typed KMIP response is written");
    tls.flush().expect("the TLS response is flushed");
    let _ = tls.get_ref().shutdown(Shutdown::Both);

    let (second_stream, _) = listener
        .accept()
        .expect("the response-shape-error call reaches the controlled peer");
    second_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("post-dispatch peer read timeout is configured");
    let connection = ServerConnection::new(Arc::clone(&retry_server_config))
        .expect("the TLS 1.3 server connection is configured");
    let mut tls = StreamOwned::new(connection, second_stream);
    let request = read_ttlv_frame(&mut tls).expect("response-shape-error request arrives fully");
    assert!(request.starts_with(&[0x42, 0x00, 0x78]));
    tls.write_all(&mismatched_discover_versions_response())
        .expect("mismatched response fixture is written");
    tls.flush()
        .expect("the malformed typed response is flushed");
    let _ = tls.get_ref().shutdown(Shutdown::Both);

    let (third_stream, _) = listener
        .accept()
        .expect("the explicit post-dispatch transport-failure call reaches the peer");
    third_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("transport-failure peer read timeout is configured");
    let connection = ServerConnection::new(retry_server_config)
        .expect("the TLS 1.3 server connection is configured");
    let mut tls = StreamOwned::new(connection, third_stream);
    let request = read_ttlv_frame(&mut tls).expect("post-dispatch request arrives fully");
    assert!(request.starts_with(&[0x42, 0x00, 0x78]));
    let _ = tls.get_ref().shutdown(Shutdown::Both);

    listener
        .set_nonblocking(true)
        .expect("peer can check for an automatic retry");
    let retry_deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    loop {
        match listener.accept() {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= retry_deadline {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok((stream, _)) => {
                let _ = stream.shutdown(Shutdown::Both);
                panic!("the client automatically retried a dispatched KMIP request");
            }
            Err(error) => panic!("peer retry check failed: {error}"),
        }
    }
}

struct TemporaryCredentialFiles {
    directory: PathBuf,
    paths: [PathBuf; 4],
}

impl TemporaryCredentialFiles {
    fn create(contents: [&[u8]; 4]) -> Self {
        let directory = create_private_test_directory("kmipkit-ffi-c-consumer")
            .expect("private test directory is created exclusively");
        let paths = std::array::from_fn(|index| directory.join(format!("credential-{index}.der")));
        let files = Self { directory, paths };
        for (path, bytes) in files.paths.iter().zip(contents) {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .expect("ephemeral credential file is created exclusively");
            file.write_all(bytes)
                .expect("ephemeral credential fixture is written");
        }
        files
    }
}

impl Drop for TemporaryCredentialFiles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn create_private_test_directory(prefix: &str) -> io::Result<PathBuf> {
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
            "{prefix}-{}-{nonce}-{sequence}",
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
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique private test directory",
    ))
}

fn mtls_server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the test authority is a valid root certificate");
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("the mTLS client verifier builds");
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the test peer enables TLS 1.3 only");
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
            .expect("the ephemeral server identity is valid"),
    )
}

fn read_ttlv_frame(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 8];
    stream.read_exact(&mut header)?;
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if length > MAX_TEST_TTLV_VALUE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test peer refused an oversized remote TTLV frame",
        ));
    }
    let padded_length = length
        .checked_add(7)
        .map(|value| value / 8 * 8)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "TTLV padding overflow"))?;
    let frame_length = 8_usize
        .checked_add(padded_length)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "TTLV frame overflow"))?;
    let mut body = vec![0_u8; padded_length];
    stream.read_exact(&mut body)?;
    let mut frame = Vec::with_capacity(frame_length);
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&body);
    Ok(frame)
}

#[test]
fn test_peer_rejects_remote_ttlv_length_before_allocating() {
    let mut stream = std::io::Cursor::new([0_u8, 0, 0, 0, 0xff, 0xff, 0xff, 0xff]);
    let error = read_ttlv_frame(&mut stream).expect_err("oversized frame is rejected");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

fn discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 2),
        ],
    );
    let response_payload = structure(0x0042_007C, [protocol_version]);
    let failed_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, &[0x00, 0x82, 0xfe, 0x02]),
            enumeration(0x0042_007F, 1),
            enumeration(0x0042_007E, 5),
            ttlv_item(0x0042_007D, 0x07, b"operation unsupported"),
        ],
    );
    let successful_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, &[0x00, 0x81, 0xff, 0x01]),
            enumeration(0x0042_007F, 0),
            response_payload,
        ],
    );
    /* The peer returns a reversed batch; item IDs restore caller-request order. */
    structure(0x0042_007B, [header, failed_item, successful_item])
}

fn mixed_discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 4),
        ],
    );
    let response_payload = structure(
        0x0042_007C,
        [protocol_version.clone(), protocol_version.clone()],
    );
    let successful_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, b"item-0"),
            enumeration(0x0042_007F, 0),
            response_payload,
        ],
    );
    let pending_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, b"item-1"),
            enumeration(0x0042_007F, 2),
            ttlv_item(0x0042_0006, 0x08, &[0x00, 0xa1, 0x7f]),
            structure(0x0042_007C, []),
        ],
    );
    let failed_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, b"item-2"),
            enumeration(0x0042_007F, 1),
            enumeration(0x0042_007E, 1),
        ],
    );
    let empty_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, b"item-3"),
            enumeration(0x0042_007F, 0),
            structure(0x0042_007C, []),
        ],
    );
    structure(
        0x0042_007B,
        [
            header,
            successful_item,
            pending_item,
            failed_item,
            empty_item,
        ],
    )
}

fn mismatched_discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 1),
        ],
    );
    let item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, &[0xbb]),
            enumeration(0x0042_007F, 0),
            structure(0x0042_007C, [protocol_version]),
        ],
    );
    structure(0x0042_007B, [header, item])
}

fn structure<const N: usize>(tag: u32, children: [Vec<u8>; N]) -> Vec<u8> {
    let value = children.into_iter().flatten().collect::<Vec<_>>();
    ttlv_item(tag, 0x01, &value)
}

fn integer(tag: u32, value: i32) -> Vec<u8> {
    ttlv_item(tag, 0x02, &value.to_be_bytes())
}

fn enumeration(tag: u32, value: u32) -> Vec<u8> {
    ttlv_item(tag, 0x05, &value.to_be_bytes())
}

fn date_time(tag: u32, value: i64) -> Vec<u8> {
    ttlv_item(tag, 0x09, &value.to_be_bytes())
}

fn ttlv_item(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    encoded.extend_from_slice(&tag.to_be_bytes()[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test response element length fits the TTLV field")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    encoded
}
