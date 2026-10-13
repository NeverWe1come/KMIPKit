#![allow(unsafe_code)]
#![cfg(feature = "coverage-c-consumer")]

use std::ffi::{CString, c_char};
use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use kmipkit_ffi::kmipkit_codec_limits_t;
use kmipkit_test_support::EphemeralPki;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

#[link(name = "kmipkit_extension_registry_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_extension_registry_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
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
    let certificate = CString::new(paths.0[0].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let authority = CString::new(paths.0[1].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let private_key = CString::new(paths.0[2].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let trust_root = CString::new(paths.0[3].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let server_config = mtls_server_config(&pki);
    let peer = thread::spawn(move || {
        let (stream, _) = listener
            .accept()
            .expect("the C client connects to the local peer");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls);
        assert!(request.starts_with(&[0x42, 0x00, 0x78]));
        tls.write_all(&discover_versions_response())
            .expect("the typed KMIP response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
        let (second_stream, _) = listener
            .accept()
            .expect("the explicit second call reaches the controlled peer");
        let _ = second_stream.shutdown(Shutdown::Both);
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

struct TemporaryCredentialFiles([PathBuf; 4]);

impl TemporaryCredentialFiles {
    fn create(contents: [&[u8]; 4]) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let directory = std::env::temp_dir();
        let paths = std::array::from_fn(|index| {
            directory.join(format!(
                "kmipkit-ffi-{}-{nonce}-{index}.der",
                std::process::id()
            ))
        });
        for (path, bytes) in paths.iter().zip(contents) {
            fs::write(path, bytes).expect("ephemeral credential fixture is written");
        }
        Self(paths)
    }
}

impl Drop for TemporaryCredentialFiles {
    fn drop(&mut self) {
        for path in &self.0 {
            let _ = fs::remove_file(path);
        }
    }
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

fn read_ttlv_frame(stream: &mut impl Read) -> Vec<u8> {
    let mut header = [0_u8; 8];
    stream
        .read_exact(&mut header)
        .expect("the request header arrives");
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let padded_length = length.div_ceil(8) * 8;
    let mut body = vec![0_u8; padded_length];
    stream
        .read_exact(&mut body)
        .expect("the request body arrives");
    let mut frame = Vec::with_capacity(8 + padded_length);
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&body);
    frame
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
