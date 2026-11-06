//! Configuration options for the KMIP test server.

use std::net::SocketAddr;
use std::path::PathBuf;

/// Configuration builder and options for `TestServer`.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Address and port to bind to. Default: `127.0.0.1:0` (ephemeral port) or `127.0.0.1:5696`.
    pub listen_addr: SocketAddr,
    /// Root directory for JSON file storage of objects.
    pub data_dir: PathBuf,
    /// Optional directory to export generated CA and client certificates and keys.
    pub export_pki: Option<PathBuf>,
    /// Whether mutual TLS is required (verifying client certificate against server CA).
    pub require_client_cert: bool,
    /// Whether to wipe the data directory on startup.
    pub clean: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            data_dir: std::env::temp_dir().join("kmipkit-test-server-data"),
            export_pki: None,
            require_client_cert: true,
            clean: false,
        }
    }
}

impl ServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_listen_addr(mut self, addr: SocketAddr) -> Self {
        self.listen_addr = addr;
        self
    }

    pub fn with_data_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.data_dir = dir.into();
        self
    }

    pub fn with_export_pki(mut self, dir: impl Into<PathBuf>) -> Self {
        self.export_pki = Some(dir.into());
        self
    }

    pub fn with_require_client_cert(mut self, require: bool) -> Self {
        self.require_client_cert = require;
        self
    }

    pub fn with_clean(mut self, clean: bool) -> Self {
        self.clean = clean;
        self
    }
}
