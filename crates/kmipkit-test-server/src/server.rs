//! TestServer instance and lifecycle handle.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::config::ServerConfig;
use crate::ops::Dispatcher;
use crate::pki::AutoPki;
use crate::store::JsonStore;
use crate::transport::listener::{create_tls_acceptor, run_listener};

/// An actively running test server handle.
pub struct TestServerHandle {
    local_addr: SocketAddr,
    pki: AutoPki,
    store: Arc<JsonStore>,
    shutdown_tx: watch::Sender<bool>,
    task_handle: JoinHandle<Result<(), String>>,
}

impl TestServerHandle {
    /// Returns the local socket address bound by the server.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Returns the TCP port bound by the server.
    pub fn port(&self) -> u16 {
        self.local_addr.port()
    }

    /// Accesses the generated PKI material.
    pub fn pki(&self) -> &AutoPki {
        &self.pki
    }

    /// Accesses the store.
    pub fn store(&self) -> &Arc<JsonStore> {
        &self.store
    }

    /// Cleanly shuts down the test server.
    pub async fn shutdown(self) -> Result<(), String> {
        let _ = self.shutdown_tx.send(true);
        self.task_handle
            .await
            .map_err(|e| format!("join error: {e}"))?
    }
}

pub struct TestServer;

impl TestServer {
    /// Starts the test server according to `config`.
    pub async fn start(config: ServerConfig) -> Result<TestServerHandle, String> {
        let pki = AutoPki::generate().map_err(|e| format!("PKI generation error: {e}"))?;

        if let Some(ref export_dir) = config.export_pki {
            pki.export_to_directory(export_dir)
                .map_err(|e| format!("PKI export error: {e}"))?;
        }

        let store = Arc::new(
            JsonStore::new(&config.data_dir, config.clean)
                .map_err(|e| format!("storage init error: {e}"))?,
        );

        let dispatcher = Arc::new(Dispatcher::new(Arc::clone(&store)));

        let mut root_store = rustls::RootCertStore::empty();
        root_store
            .add(rustls::pki_types::CertificateDer::from(
                pki.ca_cert_der.clone(),
            ))
            .map_err(|e| format!("CA cert error: {e}"))?;

        let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store))
            .build()
            .map_err(|e| format!("client verifier error: {e}"))?;

        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let builder = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| format!("TLS 1.3 config error: {e}"))?;

        let cert_chain = vec![rustls::pki_types::CertificateDer::from(
            pki.server_cert_der.clone(),
        )];
        let key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(pki.server_key_der.to_vec()),
        );

        let server_tls_config = if config.require_client_cert {
            builder
                .with_client_cert_verifier(verifier)
                .with_single_cert(cert_chain, key_der)
        } else {
            builder
                .with_no_client_auth()
                .with_single_cert(cert_chain, key_der)
        }
        .map_err(|e| format!("server TLS config error: {e}"))?;

        let acceptor = create_tls_acceptor(Arc::new(server_tls_config));

        let listener = TcpListener::bind(config.listen_addr)
            .await
            .map_err(|e| format!("bind error on {}: {e}", config.listen_addr))?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| format!("local_addr error: {e}"))?;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let task_handle =
            tokio::spawn(
                async move { run_listener(listener, acceptor, dispatcher, shutdown_rx).await },
            );

        Ok(TestServerHandle {
            local_addr,
            pki,
            store,
            shutdown_tx,
            task_handle,
        })
    }
}
