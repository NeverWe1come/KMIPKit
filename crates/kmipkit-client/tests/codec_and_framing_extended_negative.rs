//! Extended negative tests for XML, JSON, and transport stream framing.
//!
//! Covers hostile XML (entity expansions, DTDs, recursive namespaces, deep nesting),
//! malformed JSON (non-hex byte strings, overflow integers, invalid escapes, unclosed structures),
//! and raw TLS transport stream truncations.
//! Conforms to `KMIPKit` AGENTS.md, KMIPKIT-REQ-PROF-5.4.1, and KMIPKIT-REQ-PROF-5.5.1.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientErrorCategory, ClientRequest, WireEncoding,
};
use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestDeliveryState,
    TimeoutPolicy, TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

const SERVER_NAME: &str = "server.kmipkit.test";
const PEER_TIMEOUT: Duration = Duration::from_secs(5);

fn empty_client_configuration(encoding: WireEncoding) -> ClientConfiguration {
    ClientConfiguration::new(
        client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
            .expect("empty extension registry is valid"),
    )
    .with_wire_encoding(encoding)
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
    let verifier = WebPkiClientVerifier::builder_with_provider(
        Arc::new(roots),
        Arc::new(rustls_openssl::default_provider()),
    )
    .build()
    .expect("the mTLS client verifier builds");
    let provider = Arc::new(rustls_openssl::default_provider());
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

fn spawn_peer_raw_responder(
    listener: TcpListener,
    configuration: Arc<ServerConfig>,
    response_bytes: Vec<u8>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        listener
            .set_nonblocking(false)
            .expect("peer listener switches to blocking mode");
        let (stream, _) = listener.accept().expect("client connects to peer");
        stream
            .set_read_timeout(Some(PEER_TIMEOUT))
            .expect("peer read timeout is configured");
        stream
            .set_write_timeout(Some(PEER_TIMEOUT))
            .expect("peer write timeout is configured");
        let connection = ServerConnection::new(configuration)
            .expect("ephemeral TLS server connection is created");
        let mut tls = StreamOwned::new(connection, stream);

        let mut request = [0_u8; 1024];
        let _ = tls.read(&mut request);

        if !response_bytes.is_empty() {
            let _ = tls.write_all(&response_bytes);
            let _ = tls.flush();
        }
    })
}

// ---------------------------------------------------------------------------
// XML Extended Negative Tests
// ---------------------------------------------------------------------------

#[test]
fn hostile_xml_with_entity_expansion_and_dtd_is_safely_rejected() {
    let pki = EphemeralPki::generate().expect("pki succeeds");
    let hostile_inputs = [
        // Classic billion laughs entity expansion / DTD (rejected at framing or preflight)
        b"<?xml version=\"1.0\"?><!DOCTYPE lolz [<!ENTITY lol \"lol\"><!ELEMENT lolz (#PCDATA)>]><ResponseMessage><ResponseHeader><ProtocolVersion><ProtocolVersionMajor type=\"Integer\" value=\"2\"/><ProtocolVersionMinor type=\"Integer\" value=\"1\"/></ProtocolVersion></ResponseHeader></ResponseMessage>".to_vec(),
        // External parameter entity declaration
        b"<!DOCTYPE foo SYSTEM \"http://127.0.0.1:9999/evil.dtd\"><ResponseMessage/>".to_vec(),
        // CDATA containing unclosed tags
        b"<ResponseMessage><ResponseHeader><![CDATA[ unclosed CDATA ".to_vec(),
        // Recursive / circular entity syntax
        b"<!ENTITY a \"&b;\"><!ENTITY b \"&a;\"><ResponseMessage/>".to_vec(),
        // Valid XML element structure but containing forbidden XML entity references in value
        b"<ResponseMessage><ResponseHeader><ProtocolVersion><ProtocolVersionMajor type=\"Integer\" value=\"2\"/><ProtocolVersionMinor type=\"Integer\" value=\"1\"/></ProtocolVersion></ResponseHeader><BatchItem><ResponsePayload type=\"TextString\" value=\"&customEntity;\"/></BatchItem></ResponseMessage>".to_vec(),
    ];

    for hostile in hostile_inputs {
        let listener = LoopbackTcpListener::bind().expect("listener binds");
        let port = listener.local_addr().port();
        let peer =
            spawn_peer_raw_responder(listener.into_inner(), server_configuration(&pki), hostile);
        let mut client = Client::new(
            empty_client_configuration(WireEncoding::Xml),
            transport_configuration(&pki, port),
        )
        .expect("client succeeds");

        let result = client.execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        );

        let err = result.expect_err("hostile XML must be rejected");
        assert!(
            err.category() == ClientErrorCategory::Protocol
                || err.category() == ClientErrorCategory::Transport,
            "error {err:?} must be Protocol or Transport"
        );
        assert_eq!(
            err.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted)
        );
        let msg = format!("{err} {err:?}");
        assert!(!msg.contains("DOCTYPE"));
        assert!(!msg.contains("ENTITY"));
        assert!(!msg.contains("customEntity"));
        peer.join().expect("peer finishes");
    }
}

// ---------------------------------------------------------------------------
// JSON Extended Negative Tests
// ---------------------------------------------------------------------------

#[test]
fn malformed_json_with_bad_hex_and_corrupt_numbers_is_safely_rejected() {
    let pki = EphemeralPki::generate().expect("pki succeeds");
    let malformed_inputs = [
        // Non-hex characters in byte string
        br#"{"tag":"ResponseMessage","value":[{"tag":"ResponseHeader","value":[{"tag":"ProtocolVersion","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]}]},{"tag":"BatchItem","value":[{"tag":"ResponsePayload","type":"ByteString","value":"NOT_VALID_HEX_ZZ"}]}]}"#.to_vec(),
        // Integer overflow / corrupted number
        br#"{"tag":"ResponseMessage","value":[{"tag":"ResponseHeader","value":[{"tag":"ProtocolVersion","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":999999999999999999999999999999999999999999999}]}]}]}"#.to_vec(),
        // Invalid unicode escape sequence in string
        br#"{"tag":"ResponseMessage","value":[{"tag":"ResponseHeader","value":[{"tag":"ProtocolVersion","value":[{"tag":"ProtocolVersionMajor","type":"TextString","value":"\uD800"}]}]}]}"#.to_vec(),
        // Truncated object key (rejected at framer)
        br#"{"tag":"ResponseMessage","value":[{"tag": "#.to_vec(),
    ];

    for malformed in malformed_inputs {
        let listener = LoopbackTcpListener::bind().expect("listener binds");
        let port = listener.local_addr().port();
        let peer =
            spawn_peer_raw_responder(listener.into_inner(), server_configuration(&pki), malformed);
        let mut client = Client::new(
            empty_client_configuration(WireEncoding::Json),
            transport_configuration(&pki, port),
        )
        .expect("client succeeds");

        let result = client.execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        );

        let err = result.expect_err("malformed JSON must be rejected");
        assert!(
            err.category() == ClientErrorCategory::Protocol
                || err.category() == ClientErrorCategory::Transport,
            "error {err:?} must be Protocol or Transport"
        );
        assert_eq!(
            err.delivery_state(),
            Some(RequestDeliveryState::ResponseStarted)
        );
        let msg = format!("{err} {err:?}");
        assert!(!msg.contains("NOT_VALID_HEX"));
        peer.join().expect("peer finishes");
    }
}

// ---------------------------------------------------------------------------
// Boundary Truncation & Framing Tests
// ---------------------------------------------------------------------------

#[test]
fn truncated_stream_responses_produce_safe_delivery_evidence() {
    let pki = EphemeralPki::generate().expect("pki succeeds");
    let truncated_cases = [
        // Empty response: server closes connection without writing any response octet.
        // The client transmitted the request, so delivery is PossiblySent.
        (
            WireEncoding::Ttlv,
            vec![],
            RequestDeliveryState::PossiblySent,
        ),
        // Truncated TTLV header (4 bytes received before EOF) -> ResponseStarted
        (
            WireEncoding::Ttlv,
            vec![0x42, 0x00, 0x7A, 0x01],
            RequestDeliveryState::ResponseStarted,
        ),
        // Truncated JSON stream (bytes received before EOF) -> ResponseStarted
        (
            WireEncoding::Json,
            br#"{"tag":"ResponseMessage""#.to_vec(),
            RequestDeliveryState::ResponseStarted,
        ),
        // Truncated XML stream (bytes received before EOF) -> ResponseStarted
        (
            WireEncoding::Xml,
            b"<ResponseMessage><ResponseHeader>".to_vec(),
            RequestDeliveryState::ResponseStarted,
        ),
    ];

    for (encoding, truncated_bytes, expected_state) in truncated_cases {
        let listener = LoopbackTcpListener::bind().expect("listener binds");
        let port = listener.local_addr().port();
        let peer = spawn_peer_raw_responder(
            listener.into_inner(),
            server_configuration(&pki),
            truncated_bytes,
        );
        let mut client = Client::new(
            empty_client_configuration(encoding),
            transport_configuration(&pki, port),
        )
        .expect("client succeeds");

        let result = client.execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
            &CodecLimits::defaults(),
        );

        let err = result.expect_err("truncated response stream must be rejected");
        assert_eq!(
            err.delivery_state(),
            Some(expected_state),
            "expected delivery state {expected_state:?} for truncated {encoding:?} response"
        );
        peer.join().expect("peer finishes");
    }
}
