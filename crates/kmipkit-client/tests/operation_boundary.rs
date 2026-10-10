//! Client-boundary contracts for KMIPKIT-0017 FR-001 and FR-011.
//!
//! The wire Operation Enumeration values follow OASIS KMIP Specification v2.1
//! §11.36, Table 470: Get is 0x0000000A and Locate is 0x00000008. Operation
//! payloads follow §§6.1.19, Tables 220–222, and 6.1.28, Tables 247–249. The
//! Pending request opts into mixed synchronous/asynchronous responses with
//! Asynchronous Indicator Optional (2), per §§9.2 and 11.3, Tables 431–432.
//! Its response carries the §9.1, Table 400 Asynchronous Correlation Value
//! within the §8.6, Table 399 Response Batch Item. The response status values
//! and failure reason use §§11.46–11.47, Tables 479–480.

use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use kmipkit_protocol::{AttributeSet, GetRequest, LocateRequest, extension};
use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutPolicy, TransportConfig,
    TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

const SERVER_NAME: &str = "server.kmipkit.test";
const PEER_TIMEOUT: Duration = Duration::from_secs(5);
const OPERATION: u32 = 0x0042_005C;
const GET_OPERATION: u32 = 0x0000_000A;
const LOCATE_OPERATION: u32 = 0x0000_0008;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

#[test]
fn get_and_locate_operations_keep_completed_and_pending_response_views() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI generation succeeds");
    let listener = LoopbackTcpListener::bind().expect("loopback listener binds");
    let port = listener.local_addr().port();
    let peer = spawn_peer(
        listener.into_inner(),
        server_configuration(&pki),
        response_bytes(),
    );
    let transport = transport_configuration(&pki, port);
    let mut client = Client::new(empty_client_configuration(), transport)
        .expect("validated raw-TLS client construction succeeds");

    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))
            .with_unique_batch_item_id(b"get-error".to_vec()),
        ClientBatchItem::new(ClientRequest::Locate(LocateRequest::new(
            AttributeSet::new(),
        )))
        .with_unique_batch_item_id(b"locate-complete".to_vec()),
        ClientBatchItem::new(ClientRequest::Locate(LocateRequest::new(
            AttributeSet::new(),
        )))
        .with_unique_batch_item_id(b"locate-pending".to_vec()),
    ])
    .with_asynchronous_indicator(2);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("Get and Locate responses retain the requested typed outcomes");
    let request = peer
        .join()
        .expect("the raw-TLS peer completes one exchange");

    assert_eq!(operation_count(&request, GET_OPERATION), 1);
    assert_eq!(operation_count(&request, LOCATE_OPERATION), 2);
    assert_eq!(response.len(), 3);

    let get_outcome = response
        .get(0)
        .expect("the Get result remains in request order")
        .outcome();
    assert_eq!(get_outcome.operation(), ClientOperation::Get);
    assert_eq!(get_outcome.result().status().raw(), 1);
    let get_response = get_outcome
        .get_response()
        .expect("the non-Pending Get result has a completed-only response");
    assert!(get_outcome.locate_response().is_none());
    assert!(get_response.object().is_none());
    assert!(std::ptr::eq(
        get_response,
        get_outcome
            .response()
            .get()
            .expect("the unified response view exposes Get")
    ));
    assert!(get_outcome.response().locate().is_none());

    let locate_outcome = response
        .get(1)
        .expect("the completed Locate result remains in request order")
        .outcome();
    assert_eq!(locate_outcome.operation(), ClientOperation::Locate);
    assert_eq!(locate_outcome.result().status().raw(), 0);
    assert!(locate_outcome.get_response().is_none());
    let locate_response = locate_outcome
        .locate_response()
        .expect("the completed Locate result has its direct response accessor");
    assert!(locate_response.unique_identifiers().is_empty());
    assert!(std::ptr::eq(
        locate_response,
        locate_outcome
            .response()
            .locate()
            .expect("the unified response view exposes Locate")
    ));
    assert!(locate_outcome.response().get().is_none());

    let pending_outcome = response
        .get(2)
        .expect("the Pending Locate result remains in request order")
        .outcome();
    let ClientBatchOutcome::Pending(pending) = pending_outcome else {
        panic!("a valid Locate Pending response remains a shared Pending outcome");
    };
    assert_eq!(pending.operation(), ClientOperation::Locate);
    assert_eq!(pending.result().status().raw(), 2);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        b"pending-locate-correlation"
    );
    assert!(pending_outcome.get_response().is_none());
    assert!(pending_outcome.locate_response().is_none());
    assert!(pending.response().get().is_none());
    assert_eq!(
        pending
            .response()
            .locate()
            .map(|value| value.result().status().raw()),
        Some(2),
        "the shared Pending representation remains accessible through ClientResponseView"
    );
}

fn empty_client_configuration() -> ClientConfiguration {
    let registry = client_extension_registry(Vec::new(), extension::defaults())
        .expect("an empty extension registry is valid");
    ClientConfiguration::new(registry)
}

fn transport_configuration(pki: &EphemeralPki, port: u16) -> TransportConfig {
    let identity = pki.client_identity();
    let certificates = identity
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", port))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(certificates),
            PrivateKeyInput::from_der(identity.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .tls_server_name(SERVER_NAME)
        .timeouts(TimeoutPolicy::default())
        .build()
        .expect("ephemeral certificate inputs produce a valid transport configuration")
}

fn server_configuration(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the ephemeral authority is accepted");
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("the mTLS client verifier builds");
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the TLS 1.3 server policy builds");
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

fn spawn_peer(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response: Vec<u8>,
) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("peer listener switches to nonblocking mode");
        let deadline = Instant::now() + PEER_TIMEOUT;
        let (stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("the client connection was not accepted: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(PEER_TIMEOUT))
            .expect("peer read timeout is configured");
        stream
            .set_write_timeout(Some(PEER_TIMEOUT))
            .expect("peer write timeout is configured");
        let connection = ServerConnection::new(configuration)
            .expect("the ephemeral TLS server connection is created");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls);
        tls.write_all(&response)
            .expect("the raw response bytes are written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
        request
    })
}

fn read_ttlv_frame(stream: &mut (impl Read + Write)) -> Vec<u8> {
    let mut header = [0_u8; 8];
    stream
        .read_exact(&mut header)
        .expect("the request TTLV header is received");
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let padded_length = length.div_ceil(8) * 8;
    let mut value = vec![0; padded_length];
    stream
        .read_exact(&mut value)
        .expect("the request TTLV value is received");
    let mut frame = header.to_vec();
    frame.extend(value);
    frame
}

fn operation_count(request: &[u8], operation: u32) -> usize {
    let mut field = Vec::with_capacity(12);
    field.extend_from_slice(&OPERATION.to_be_bytes()[1..]);
    field.push(0x05);
    field.extend_from_slice(&4_u32.to_be_bytes());
    field.extend_from_slice(&operation.to_be_bytes());
    field.extend_from_slice(&[0; 4]);
    request
        .windows(field.len())
        .filter(|window| *window == field.as_slice())
        .count()
}

fn response_bytes() -> Vec<u8> {
    let protocol_version = structure(
        PROTOCOL_VERSION,
        [
            integer(PROTOCOL_VERSION_MAJOR, 2),
            integer(PROTOCOL_VERSION_MINOR, 1),
        ],
    );
    let header = structure(
        RESPONSE_HEADER,
        [
            protocol_version,
            date_time(TIME_STAMP, 1),
            integer(BATCH_COUNT, 3),
        ],
    );
    let get_error = response_batch_item(GET_OPERATION, b"get-error", 1, Some(1), None, None);
    let locate_completed = response_batch_item(
        LOCATE_OPERATION,
        b"locate-complete",
        0,
        None,
        None,
        Some(structure(RESPONSE_PAYLOAD, [])),
    );
    let locate_pending = response_batch_item(
        LOCATE_OPERATION,
        b"locate-pending",
        2,
        None,
        Some(b"pending-locate-correlation"),
        None,
    );
    structure(
        0x0042_007B,
        [header, get_error, locate_completed, locate_pending],
    )
}

fn response_batch_item(
    operation: u32,
    id: &[u8],
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    payload: Option<Vec<u8>>,
) -> Vec<u8> {
    let mut fields = vec![
        enumeration(OPERATION, operation),
        byte_string(UNIQUE_BATCH_ITEM_ID, id),
        enumeration(RESULT_STATUS, status),
    ];
    if let Some(reason) = reason {
        fields.push(enumeration(RESULT_REASON, reason));
        fields.push(text_string(RESULT_MESSAGE, "fixture result"));
    }
    if let Some(correlation) = correlation {
        fields.push(byte_string(ASYNCHRONOUS_CORRELATION_VALUE, correlation));
    }
    if let Some(payload) = payload {
        fields.push(payload);
    }
    structure(BATCH_ITEM, fields)
}

fn structure(tag: u32, children: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
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

fn byte_string(tag: u32, value: &[u8]) -> Vec<u8> {
    ttlv_item(tag, 0x08, value)
}

fn text_string(tag: u32, value: &str) -> Vec<u8> {
    ttlv_item(tag, 0x07, value.as_bytes())
}

fn ttlv_item(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    encoded.extend_from_slice(&tag.to_be_bytes()[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("the fixture fits the TTLV item length")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    encoded
}
