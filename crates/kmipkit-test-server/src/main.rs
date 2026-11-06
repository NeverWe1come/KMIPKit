//! Standalone KMIP 1.0-2.1 test server binary.
#![forbid(unsafe_code)]
#![allow(
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
    clippy::bool_to_int_with_if
)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;

use kmipkit_test_server::{ServerConfig, TestServer};

fn print_help() {
    println!(
        r"kmipkit-test-server - Lightweight standalone KMIP 1.0-2.1 test server

USAGE:
    kmipkit-test-server [OPTIONS]

OPTIONS:
    --listen <IP:PORT>     Address and port to bind to (default: 127.0.0.1:5696)
    --data-dir <PATH>      Directory for JSON object storage (default: ./kmip-data)
    --export-pki <PATH>    Directory to export generated CA and client certificates (default: ./kmip-pki)
    --clean                Delete existing storage files before starting
    --no-client-auth       Disable mutual TLS requirement (accept any TLS connection)
    --print-env            Print shell export statements for client environment variables
    --json                 Print server metadata in JSON format on startup
    -h, --help             Print this help message
"
    );
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut listen_addr: SocketAddr = "127.0.0.1:5696".parse().unwrap();
    let mut data_dir = PathBuf::from("./kmip-data");
    let mut export_pki = Some(PathBuf::from("./kmip-pki"));
    let mut clean = false;
    let mut require_client_cert = true;
    let mut print_env = false;
    let mut json_output = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "--listen" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --listen requires an argument");
                    process::exit(1);
                }
                listen_addr = args[i].parse().unwrap_or_else(|e| {
                    eprintln!("error: invalid address '{}': {}", args[i], e);
                    process::exit(1);
                });
            }
            "--data-dir" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --data-dir requires an argument");
                    process::exit(1);
                }
                data_dir = PathBuf::from(&args[i]);
            }
            "--export-pki" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --export-pki requires an argument");
                    process::exit(1);
                }
                export_pki = Some(PathBuf::from(&args[i]));
            }
            "--clean" => {
                clean = true;
            }
            "--no-client-auth" => {
                require_client_cert = false;
            }
            "--print-env" => {
                print_env = true;
            }
            "--json" => {
                json_output = true;
            }
            other => {
                eprintln!("error: unknown option '{other}'. Run with --help for usage.");
                process::exit(1);
            }
        }
        i += 1;
    }

    let mut config = ServerConfig::new()
        .with_listen_addr(listen_addr)
        .with_data_dir(&data_dir)
        .with_clean(clean)
        .with_require_client_cert(require_client_cert);

    if let Some(ref pki_path) = export_pki {
        config = config.with_export_pki(pki_path);
    }

    let handle = match TestServer::start(config).await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Fatal error starting kmipkit-test-server: {e}");
            process::exit(1);
        }
    };

    let bound_addr = handle.local_addr();

    if json_output {
        let pki_dir_str = export_pki
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        println!(
            "{{\"status\":\"running\",\"address\":\"{}\",\"port\":{},\"data_dir\":\"{}\",\"pki_dir\":\"{}\"}}",
            bound_addr,
            bound_addr.port(),
            data_dir.display(),
            pki_dir_str
        );
    } else {
        println!("kmipkit-test-server running at https://{bound_addr}");
        println!("  Storage data directory: {}", data_dir.display());
        if let Some(ref pki_dir) = export_pki {
            println!("  PKI material exported to: {}", pki_dir.display());
            println!("    CA certificate:     {}/ca.crt", pki_dir.display());
            println!("    Server certificate: {}/server.crt", pki_dir.display());
            println!("    Client certificate: {}/client.crt", pki_dir.display());
            println!("    Client private key: {}/client.key", pki_dir.display());
        }
    }

    if print_env {
        if let Some(ref pki_dir) = export_pki {
            let p = pki_dir.clone();
            let abs_pki =
                tokio::task::spawn_blocking(move || std::fs::canonicalize(&p).unwrap_or(p))
                    .await
                    .unwrap_or_else(|_| pki_dir.clone());
            println!("\n# KMIP Client Environment Setup:");
            println!("export KMIP_SERVER_ADDR=\"{bound_addr}\"");
            println!("export KMIP_CA_CERT=\"{}/ca.crt\"", abs_pki.display());
            println!(
                "export KMIP_CLIENT_CERT=\"{}/client.crt\"",
                abs_pki.display()
            );
            println!(
                "export KMIP_CLIENT_KEY=\"{}/client.key\"",
                abs_pki.display()
            );
        }
    }

    println!("\nPress Enter or send EOF to terminate server...");

    let _ = std::thread::spawn(|| {
        let mut buf = [0u8; 1];
        let _ = std::io::Read::read(&mut std::io::stdin(), &mut buf);
    })
    .join();

    println!("Shutting down kmipkit-test-server...");
    let _ = handle.shutdown().await;
    println!("Shutdown complete.");
}
