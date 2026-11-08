//! KMIPKit Cloud Proxy CLI entrypoint.
#![forbid(unsafe_code)]
#![allow(
    unknown_lints,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::format_push_string,
    clippy::semicolon_if_nothing_returned,
    clippy::unreadable_literal,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::collapsible_if,
    clippy::items_after_statements,
    clippy::manual_is_multiple_of,
    clippy::return_self_not_must_use,
    clippy::unnecessary_wraps,
    clippy::vec_init_then_push,
    clippy::bool_to_int_with_if,
    clippy::manual_let_else,
    clippy::single_match_else,
    clippy::manual_range_contains
)]

use clap::Parser;
use kmipkit_proxy::config::{CliArgs, ProxyConfig};
use kmipkit_proxy::providers::{CloudKmsProvider, InMemoryMockProvider};
use kmipkit_proxy::server::ProxyServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    let config = if let Some(config_path) = &args.config {
        let loaded = ProxyConfig::from_file(config_path)?;
        args.merge_into(loaded)
    } else {
        let default_config = ProxyConfig::default();
        args.merge_into(default_config)
    };

    let provider: Arc<dyn CloudKmsProvider> = match config.backend.provider.as_str() {
        "mock" => Arc::new(InMemoryMockProvider::default()),
        #[cfg(feature = "aws")]
        "aws" => {
            let master_kek = config.backend.master_kek_hex.as_deref().and_then(|hex| {
                kmipkit_proxy::codec::datetime::hex_to_octets(hex)
                    .ok()
                    .and_then(|b| b.try_into().ok())
            });
            Arc::new(
                kmipkit_proxy::providers::aws::AwsKmsProvider::from_config(
                    &config.backend.aws,
                    master_kek,
                )
                .await?,
            )
        }
        #[cfg(feature = "azure")]
        "azure" => Arc::new(kmipkit_proxy::providers::azure::AzureKeyVaultProvider::new(
            &config.backend.azure,
        )?),
        #[cfg(feature = "gcp")]
        "gcp" => Arc::new(kmipkit_proxy::providers::gcp::GcpKmsProvider::new(
            &config.backend.gcp,
        )?),
        #[cfg(feature = "oci")]
        "oci" => Arc::new(kmipkit_proxy::providers::oci::OciVaultProvider::new(
            &config.backend.oci,
        )?),
        other => {
            eprintln!(
                "Unknown or unconfigured cloud provider: {other}, defaulting to in-memory mock"
            );
            Arc::new(InMemoryMockProvider::default())
        }
    };

    println!(
        "Starting KMIPKit Proxy on {} (provider: {}, tls: {})",
        config.listener.bind_addr, config.backend.provider, config.listener.tls_enabled,
    );

    let handle = ProxyServer::start(config, provider)
        .await
        .map_err(|e| format!("Proxy server startup failed: {e}"))?;

    println!("KMIPKit Proxy listening on {}", handle.local_addr());

    tokio::signal::ctrl_c().await?;
    println!("Shutting down KMIPKit Proxy...");
    handle
        .shutdown()
        .await
        .map_err(|e| format!("Shutdown error: {e}"))?;
    println!("Shutdown complete.");

    Ok(())
}
