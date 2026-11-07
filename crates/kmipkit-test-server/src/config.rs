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

/// Options parsed from command-line arguments.
#[derive(Clone, Debug)]
pub struct CliOptions {
    pub config: ServerConfig,
    pub print_env: bool,
    pub json_output: bool,
}

pub fn help_message() -> &'static str {
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
}

pub fn parse_cli_args(args: &[String]) -> Result<Option<CliOptions>, String> {
    let mut listen_addr: SocketAddr = "127.0.0.1:5696"
        .parse()
        .map_err(|e| format!("parse default addr: {e}"))?;
    let mut data_dir = PathBuf::from("./kmip-data");
    let mut export_pki = Some(PathBuf::from("./kmip-pki"));
    let mut clean = false;
    let mut require_client_cert = true;
    let mut print_env = false;
    let mut json_output = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => return Ok(None),
            "--listen" => {
                i += 1;
                if i >= args.len() {
                    return Err("--listen requires an argument".to_owned());
                }
                listen_addr = args[i]
                    .parse()
                    .map_err(|e| format!("invalid address '{}': {}", args[i], e))?;
            }
            "--data-dir" => {
                i += 1;
                if i >= args.len() {
                    return Err("--data-dir requires an argument".to_owned());
                }
                data_dir = PathBuf::from(&args[i]);
            }
            "--export-pki" => {
                i += 1;
                if i >= args.len() {
                    return Err("--export-pki requires an argument".to_owned());
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
                return Err(format!(
                    "unknown option '{other}'. Run with --help for usage."
                ));
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

    Ok(Some(CliOptions {
        config,
        print_env,
        json_output,
    }))
}

/// Runs the server CLI loop, optionally terminating when `shutdown_rx` receives a signal.
pub async fn run_cli_with_shutdown(
    args: &[String],
    mut shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>,
) -> Result<(), String> {
    let Some(options) = parse_cli_args(args)? else {
        print!("{}", help_message());
        return Ok(());
    };

    let export_pki = options.config.export_pki.clone();
    let data_dir = options.config.data_dir.clone();
    let print_env = options.print_env;
    let json_output = options.json_output;

    let handle = crate::server::TestServer::start(options.config)
        .await
        .map_err(|e| format!("Fatal error starting kmipkit-test-server: {e}"))?;

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

    if let Some(rx) = shutdown_rx.take() {
        let _ = rx.await;
    } else {
        println!("\nPress Enter or send EOF to terminate server...");
        let _ = std::thread::spawn(|| {
            let mut buf = [0u8; 1];
            let _ = std::io::Read::read(&mut std::io::stdin(), &mut buf);
        })
        .join();
    }

    println!("Shutting down kmipkit-test-server...");
    handle.shutdown().await?;
    println!("Shutdown complete.");
    Ok(())
}
