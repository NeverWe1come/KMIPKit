#![forbid(unsafe_code)]

use std::error::Error;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kmipkit_test_support::{EphemeralPki, LoopbackTcpListener, fixtures};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, HttpsTransport, PrivateKeyInput, Transport,
    TransportConfig, TrustSource,
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection};

const DEFAULT_ITERATIONS: usize = 40;
const WARMUP_ITERATIONS: usize = 5;
const CONCURRENCY_LEVELS: [usize; 3] = [1, 4, 16];
const RESPONSE_LIMIT: usize = 1024;
const SERVER_NAME: &str = "server.kmipkit.test";
const RESPONSE_BODY: &[u8] = b"KMIPv2.1";

fn main() -> Result<(), Box<dyn Error>> {
    let iterations = parse_iterations()?;
    let pki = EphemeralPki::generate()?;
    let (client_tls, server_tls) = tls_configs(&pki)?;
    let peer = HttpsPeer::start(Arc::clone(&server_tls))?;

    println!("KMIPKit production transport benchmark");
    println!(
        "iterations={iterations} warmups={WARMUP_ITERATIONS} concurrency={CONCURRENCY_LEVELS:?}"
    );
    println!("server=loopback HTTPS/1.1 TLS1.3 mTLS; payload=request sentinel / 8-byte response");

    report("TransportConfig::build", iterations, || {
        let started = Instant::now();
        let _config = transport_config(&pki, "https://127.0.0.1:443")?;
        Ok(started.elapsed())
    })?;

    report(
        "HttpsTransport::new (config prepared before timing)",
        iterations,
        || {
            let config = transport_config(&pki, "https://127.0.0.1:443")?;
            let started = Instant::now();
            let _transport = HttpsTransport::new(config)?;
            Ok(started.elapsed())
        },
    )?;

    report(
        "TLS1.3 mTLS handshake (TCP connected before timing)",
        iterations,
        || tls_handshake_sample(peer.address, Arc::clone(&client_tls)),
    )?;

    report(
        "HTTPS cold first exchange (lazy worker + TCP + TLS + HTTP)",
        iterations,
        || {
            let mut transport = new_https_transport(&pki, peer.address)?;
            let started = Instant::now();
            exchange(&mut transport)?;
            Ok(started.elapsed())
        },
    )?;

    let mut reused = new_https_transport(&pki, peer.address)?;
    for _ in 0..WARMUP_ITERATIONS {
        exchange(&mut reused)?;
    }
    report("HTTPS reused steady-state exchange", iterations, || {
        let started = Instant::now();
        exchange(&mut reused)?;
        Ok(started.elapsed())
    })?;

    for concurrency in CONCURRENCY_LEVELS {
        report(
            &format!("concurrent cold first-exchange batch (n={concurrency})"),
            iterations,
            || {
                let mut transports = make_transports(&pki, peer.address, concurrency)?;
                let started = Instant::now();
                exchange_all(&mut transports)?;
                Ok(started.elapsed())
            },
        )?;

        let mut transports = make_transports(&pki, peer.address, concurrency)?;
        for _ in 0..WARMUP_ITERATIONS {
            exchange_all(&mut transports)?;
        }
        report(
            &format!("concurrent reused steady-state batch (n={concurrency})"),
            iterations,
            || {
                let started = Instant::now();
                exchange_all(&mut transports)?;
                Ok(started.elapsed())
            },
        )?;
    }

    drop(reused);
    drop(peer);
    Ok(())
}

fn parse_iterations() -> Result<usize, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut iterations = DEFAULT_ITERATIONS;
    while let Some(argument) = args.next() {
        if argument == "--bench" {
            // Cargo appends this marker when running a harness-free bench target.
        } else if argument == "--iterations" {
            let value = args
                .next()
                .ok_or("--iterations requires a positive integer")?;
            iterations = value.parse()?;
            if iterations == 0 {
                return Err("--iterations must be positive".into());
            }
        } else {
            return Err(format!("unknown argument: {argument}").into());
        }
    }
    Ok(iterations)
}

fn report<F>(name: &str, iterations: usize, mut sample: F) -> Result<(), Box<dyn Error>>
where
    F: FnMut() -> Result<Duration, Box<dyn Error>>,
{
    for _ in 0..WARMUP_ITERATIONS {
        let _ = sample()?;
    }
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        durations.push(sample()?);
    }
    durations.sort_unstable();
    let p50 = percentile(&durations, 50, 100);
    let p95 = percentile(&durations, 95, 100);
    println!(
        "{name}: p50={:.2} us p95={:.2} us",
        p50.as_secs_f64() * 1_000_000.0,
        p95.as_secs_f64() * 1_000_000.0
    );
    Ok(())
}

fn percentile(sorted: &[Duration], numerator: usize, denominator: usize) -> Duration {
    let rank = sorted
        .len()
        .saturating_mul(numerator)
        .saturating_add(denominator.saturating_sub(1))
        / denominator;
    let index = rank.saturating_sub(1).min(sorted.len().saturating_sub(1));
    sorted[index]
}

fn transport_config(pki: &EphemeralPki, endpoint: &str) -> Result<TransportConfig, Box<dyn Error>> {
    let client = pki.client_identity();
    let chain = client
        .certificate_chain_der()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();
    Ok(TransportConfig::builder(Endpoint::https(endpoint))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_der(chain),
            PrivateKeyInput::from_der(client.private_key_der().to_vec()),
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_der(vec![pki.authority_certificate_der().to_vec()]),
        ]))
        .tls_server_name(SERVER_NAME)
        .build()?)
}

fn new_https_transport(
    pki: &EphemeralPki,
    address: SocketAddr,
) -> Result<HttpsTransport, Box<dyn Error>> {
    let endpoint = format!("https://127.0.0.1:{}", address.port());
    Ok(HttpsTransport::new(transport_config(pki, &endpoint)?)?)
}

fn exchange(transport: &mut HttpsTransport) -> Result<(), Box<dyn Error>> {
    let response = transport
        .exchange(fixtures::REQUEST_SENTINEL, RESPONSE_LIMIT)
        .map_err(|error| {
            format!(
                "HTTPS exchange failed before completion: {:?}/{:?}",
                error.cause_category(),
                error.delivery_state()
            )
        })?;
    if response.as_bytes() != RESPONSE_BODY {
        return Err("HTTPS peer returned an unexpected response body".into());
    }
    Ok(())
}

fn make_transports(
    pki: &EphemeralPki,
    address: SocketAddr,
    count: usize,
) -> Result<Vec<HttpsTransport>, Box<dyn Error>> {
    (0..count)
        .map(|_| new_https_transport(pki, address))
        .collect()
}

fn exchange_all(transports: &mut [HttpsTransport]) -> Result<(), Box<dyn Error>> {
    thread::scope(|scope| {
        let handles: Vec<_> = transports
            .iter_mut()
            .map(|transport| scope.spawn(move || exchange(transport).map_err(|_| ())))
            .collect();
        for handle in handles {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(())) | Err(_) => return Err("concurrent HTTPS exchange failed".into()),
            }
        }
        Ok(())
    })
}

fn tls_configs(
    pki: &EphemeralPki,
) -> Result<(Arc<ClientConfig>, Arc<ServerConfig>), Box<dyn Error>> {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(
        pki.authority_certificate_der().to_vec(),
    ))?;
    let client_identity = pki.client_identity();
    let client_chain = client_identity
        .certificate_chain_der()
        .into_iter()
        .map(|certificate| CertificateDer::from(certificate.to_vec()))
        .collect();
    let client_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        client_identity.private_key_der().to_vec(),
    ));
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut client = ClientConfig::builder_with_provider(Arc::clone(&provider))
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_root_certificates(roots)
        .with_client_auth_cert(client_chain, client_key)?;
    client.resumption = rustls::client::Resumption::disabled();

    let mut client_roots = RootCertStore::empty();
    client_roots.add(CertificateDer::from(
        pki.authority_certificate_der().to_vec(),
    ))?;
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots)).build()?;
    let server_identity = pki.server_identity();
    let server = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![CertificateDer::from(
                server_identity.certificate_der().to_vec(),
            )],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                server_identity.private_key_der().to_vec(),
            )),
        )?;
    Ok((Arc::new(client), Arc::new(server)))
}

fn tls_handshake_sample(
    address: SocketAddr,
    client_config: Arc<ClientConfig>,
) -> Result<Duration, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address)?;
    stream.set_nodelay(true)?;
    let server_name = ServerName::try_from(SERVER_NAME.to_owned())?;
    let mut connection = ClientConnection::new(client_config, server_name)?;
    let started = Instant::now();
    while connection.is_handshaking() {
        connection.complete_io(&mut stream)?;
    }
    Ok(started.elapsed())
}

struct HttpsPeer {
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    accept_thread: Option<JoinHandle<()>>,
    connection_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl HttpsPeer {
    fn start(configuration: Arc<ServerConfig>) -> io::Result<Self> {
        let listener = LoopbackTcpListener::bind()?.into_inner();
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let connection_threads = Arc::new(Mutex::new(Vec::new()));
        let worker_connections = Arc::clone(&connection_threads);
        let accept_thread = thread::spawn(move || {
            while !worker_stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if stream.set_nonblocking(false).is_err() {
                            continue;
                        }
                        let config = Arc::clone(&configuration);
                        let handle = thread::spawn(move || serve_http_connection(stream, config));
                        if let Ok(mut threads) = worker_connections.lock() {
                            threads.push(handle);
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            address,
            stop,
            accept_thread: Some(accept_thread),
            connection_threads,
        })
    }
}

impl Drop for HttpsPeer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.accept_thread.take() {
            let _ = thread.join();
        }
        if let Ok(mut threads) = self.connection_threads.lock() {
            for thread in threads.drain(..) {
                let _ = thread.join();
            }
        }
    }
}

fn serve_http_connection(tcp: TcpStream, configuration: Arc<ServerConfig>) {
    let _ = tcp.set_read_timeout(Some(Duration::from_secs(60)));
    let _ = tcp.set_write_timeout(Some(Duration::from_secs(2)));
    let Ok(connection) = ServerConnection::new(configuration) else {
        return;
    };
    let stream = rustls::StreamOwned::new(connection, tcp);
    let mut reader = BufReader::new(stream);
    loop {
        if read_http_request(&mut reader).is_err() {
            return;
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
            RESPONSE_BODY.len()
        );
        let stream = reader.get_mut();
        if stream
            .write_all(response.as_bytes())
            .and_then(|()| stream.write_all(RESPONSE_BODY))
            .and_then(|()| stream.flush())
            .is_err()
        {
            return;
        }
    }
}

fn read_http_request(
    reader: &mut BufReader<rustls::StreamOwned<ServerConnection, TcpStream>>,
) -> io::Result<()> {
    let mut header_bytes = 0_usize;
    let mut content_length = 0_usize;
    loop {
        let mut line = Vec::new();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "HTTP request ended",
            ));
        }
        header_bytes = header_bytes.saturating_add(read);
        if header_bytes > 64 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "HTTP headers too large",
            ));
        }
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        if let Some(separator) = line.iter().position(|byte| *byte == b':')
            && line[..separator].eq_ignore_ascii_case(b"content-length")
        {
            let value = line.get(separator + 1..).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid content length")
            })?;
            let value = std::str::from_utf8(value).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid content length")
            })?;
            content_length = value.trim().parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid content length")
            })?;
            if content_length > 1024 * 1024 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "HTTP body too large",
                ));
            }
        }
    }
    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body)
}
