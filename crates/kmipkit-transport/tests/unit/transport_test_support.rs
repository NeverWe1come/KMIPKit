use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

use crate::config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};
use crate::resolver::Resolver;

pub(crate) fn loopback_listener() -> (TcpListener, SocketAddr) {
    let listener = LoopbackTcpListener::bind().expect("the test TLS peer binds loopback");
    let address = listener.local_addr();
    (listener.into_inner(), address)
}

pub(crate) fn client_config(pki: &EphemeralPki, endpoint: Endpoint) -> TransportConfig {
    client_config_with_server_name(pki, endpoint, None)
}

pub(crate) fn client_config_with_server_name(
    pki: &EphemeralPki,
    endpoint: Endpoint,
    server_name: Option<&str>,
) -> TransportConfig {
    let client = pki.client_identity();
    let builder = TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(
                client
                    .certificate_chain_der()
                    .into_iter()
                    .map(<[u8]>::to_vec)
                    .collect(),
            ),
            PrivateKeyInput::from_der(client.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]));
    let builder = if let Some(server_name) = server_name {
        builder.tls_server_name(server_name)
    } else {
        builder
    };
    builder
        .build()
        .expect("the explicit test identity and trust root build a valid config")
}

pub(crate) fn fixed_resolver(address: SocketAddr) -> Resolver {
    Resolver::with_lookup_and_governor(
        move |_host, _port| Ok(vec![address]),
        Arc::new(tokio::sync::Semaphore::new(1)),
    )
}

pub(crate) fn server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the local test server uses TLS 1.3 only");
    let mut client_roots = RootCertStore::empty();
    client_roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the test certificate authority is valid");
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .expect("the local test server verifies client certificates");
    let server = pki.server_identity();
    let config = builder
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![CertificateDer::from(server.certificate_der().to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(server.private_key_der().to_vec())),
        )
        .expect("the ephemeral server identity is valid");
    Arc::new(config)
}

pub(crate) fn spawn_http_peer(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    requests: usize,
) -> JoinHandle<io::Result<Vec<Vec<u8>>>> {
    spawn_http_peer_with_response(
        listener,
        config,
        requests,
        b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: 8\r\n\r\nresponse",
    )
}

pub(crate) fn spawn_http_peer_with_response(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    requests: usize,
    response: &'static [u8],
) -> JoinHandle<io::Result<Vec<Vec<u8>>>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept()?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        let connection = ServerConnection::new(config)
            .map_err(|_| io::Error::other("the test TLS server initializes"))?;
        let mut tls = StreamOwned::new(connection, socket);
        let mut captured = Vec::with_capacity(requests);
        for _ in 0..requests {
            captured.push(read_http_request(&mut tls)?);
            tls.write_all(response)?;
            tls.flush()?;
        }
        let mut close_probe = [0_u8; 1];
        let _ = tls.read(&mut close_probe);
        Ok(captured)
    })
}

pub(crate) fn spawn_http_peer_and_close(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    response: &'static [u8],
) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept()?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        let connection = ServerConnection::new(config)
            .map_err(|_| io::Error::other("the test TLS server initializes"))?;
        let mut tls = StreamOwned::new(connection, socket);
        let request = read_http_request(&mut tls)?;
        tls.write_all(response)?;
        tls.flush()?;
        Ok(request)
    })
}

fn read_http_request(tls: &mut StreamOwned<ServerConnection, TcpStream>) -> io::Result<Vec<u8>> {
    let mut request = Vec::new();
    let header_end = loop {
        if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
        read_more(tls, &mut request)?;
    };
    let header = std::str::from_utf8(&request[..header_end])
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "test HTTP header is UTF-8"))?;
    let content_length = header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "test request has a length"))?;
    let request_end = header_end
        .checked_add(content_length)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "test request length fits"))?;
    while request.len() < request_end {
        read_more(tls, &mut request)?;
    }
    request.truncate(request_end);
    Ok(request)
}

fn read_more(
    tls: &mut StreamOwned<ServerConnection, TcpStream>,
    buffer: &mut Vec<u8>,
) -> io::Result<()> {
    let mut chunk = [0_u8; 1024];
    let read = tls.read(&mut chunk)?;
    if read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "the TLS client closed before the request completed",
        ));
    }
    buffer.extend_from_slice(&chunk[..read]);
    Ok(())
}

pub(crate) fn spawn_raw_tls_peer(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    request_len: usize,
    response: &'static [u8],
) -> JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let (socket, _) = listener.accept()?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        socket.set_write_timeout(Some(Duration::from_secs(5)))?;
        let connection = ServerConnection::new(config)
            .map_err(|_| io::Error::other("the test TLS server initializes"))?;
        let mut tls = StreamOwned::new(connection, socket);
        let mut request = vec![0_u8; request_len];
        tls.read_exact(&mut request)?;
        tls.write_all(response)?;
        tls.flush()?;
        Ok(request)
    })
}
