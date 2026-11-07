//! Configuration loading from TOML and command-line arguments.
#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::ProxyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KeyCustodyMode {
    #[default]
    Auto,
    NativeKms,
    AutoEnvelope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    pub bind_addr: SocketAddr,
    #[serde(default = "default_tls_enabled")]
    pub tls_enabled: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
    pub tls_client_ca_path: Option<PathBuf>,
    pub require_client_cert: bool,
}

fn default_tls_enabled() -> bool {
    true
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            bind_addr: listen_addr_default(),
            tls_enabled: true,
            tls_cert_path: None,
            tls_key_path: None,
            tls_client_ca_path: None,
            require_client_cert: true,
        }
    }
}

fn listen_addr_default() -> SocketAddr {
    "127.0.0.1:5696".parse().unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyBehaviorConfig {
    #[serde(default)]
    pub key_custody_mode: KeyCustodyMode,
    #[serde(default = "default_cache_ttl")]
    pub metadata_cache_ttl_secs: u64,
}

fn default_cache_ttl() -> u64 {
    5
}

impl Default for ProxyBehaviorConfig {
    fn default() -> Self {
        Self {
            key_custody_mode: KeyCustodyMode::default(),
            metadata_cache_ttl_secs: default_cache_ttl(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsConfig {
    pub region: Option<String>,
    pub envelope_master_kek_id: Option<String>,
    pub secret_name_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureConfig {
    pub vault_url: Option<String>,
    pub envelope_master_kek_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GcpConfig {
    pub project_id: Option<String>,
    pub location: Option<String>,
    pub key_ring: Option<String>,
    pub envelope_master_kek_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OciConfig {
    pub compartment_id: Option<String>,
    pub vault_id: Option<String>,
    pub crypto_endpoint: Option<String>,
    pub management_endpoint: Option<String>,
    pub secrets_endpoint: Option<String>,
    pub endpoint_url: Option<String>,
    pub secret_name_prefix: Option<String>,
    pub envelope_master_kek_id: Option<String>,
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    pub master_kek_hex: Option<String>,
    #[serde(default)]
    pub aws: AwsConfig,
    #[serde(default)]
    pub azure: AzureConfig,
    #[serde(default)]
    pub gcp: GcpConfig,
    #[serde(default)]
    pub oci: OciConfig,
}

fn default_provider() -> String {
    "mock".to_string()
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            master_kek_hex: None,
            aws: AwsConfig::default(),
            azure: AzureConfig::default(),
            gcp: GcpConfig::default(),
            oci: OciConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    #[serde(default)]
    pub listener: ListenerConfig,
    #[serde(default)]
    pub proxy_behavior: ProxyBehaviorConfig,
    #[serde(default)]
    pub backend: BackendConfig,
}

impl ProxyConfig {
    /// Loads configuration from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ProxyError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProxyError::ConfigError(format!("failed to read config file: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| ProxyError::ConfigError(format!("failed to parse TOML config: {e}")))
    }
}

#[derive(Parser, Debug)]
#[command(name = "kmipkit-proxy")]
#[command(
    about = "Zero-DB Stateless KMIP Proxy Bridge for AWS, Azure, Google Cloud, and Oracle OCI"
)]
pub struct CliArgs {
    /// Path to TOML configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Listening address (e.g. 127.0.0.1:5696)
    #[arg(short, long)]
    pub listen: Option<SocketAddr>,

    /// Backend cloud provider (aws, azure, gcp, oci, mock)
    #[arg(short, long)]
    pub provider: Option<String>,

    /// Hex-encoded 256-bit Master KEK for auto-envelope wrapping
    #[arg(long)]
    pub master_kek_hex: Option<String>,

    /// Disable client mTLS certificate verification (test mode only)
    #[arg(long)]
    pub no_client_auth: bool,

    /// Disable TLS (plain TCP mode)
    #[arg(long)]
    pub no_tls: bool,
}

impl CliArgs {
    /// Merges CLI flags into the loaded ProxyConfig.
    pub fn merge_into(self, mut config: ProxyConfig) -> ProxyConfig {
        if let Some(listen) = self.listen {
            config.listener.bind_addr = listen;
        }
        if let Some(provider) = self.provider {
            config.backend.provider = provider;
        }
        if let Some(kek) = self.master_kek_hex {
            config.backend.master_kek_hex = Some(kek);
        }
        if self.no_client_auth {
            config.listener.require_client_cert = false;
        }
        if self.no_tls {
            config.listener.tls_enabled = false;
        }
        config
    }
}
