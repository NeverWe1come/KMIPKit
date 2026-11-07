//! TCP and TLS listener runner.

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio_rustls::TlsAcceptor;

use super::handler::handle_connection;
use crate::ops::Dispatcher;

pub async fn run_listener(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    dispatcher: Arc<Dispatcher>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), String> {
    loop {
        tokio::select! {
            accept_res = listener.accept() => {
                let (stream, _peer_addr) = accept_res.map_err(|e| format!("accept error: {e}"))?;
                let acceptor = acceptor.clone();
                let dispatcher = Arc::clone(&dispatcher);

                tokio::spawn(async move {
                    match acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            if let Err(_err) = handle_connection(tls_stream, dispatcher).await {
                                // Connection completed or terminated
                            }
                        }
                        Err(_err) => {
                            // TLS handshake failed (e.g. invalid client cert or probing)
                        }
                    }
                });
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
            }
        }
    }

    Ok(())
}

pub fn create_tls_acceptor(server_config: Arc<rustls::ServerConfig>) -> TlsAcceptor {
    TlsAcceptor::from(server_config)
}
