//! `kmip-lsp` CLI and stdio Language Server entrypoint.
#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use kmip_lsp::lsp::server::run_stdio_server;
use kmip_lsp::ttlv::inspector::{DecodeFormat, DecodeOptions, EncodeNodeInput, decode_ttlv, encode_ttlv};

#[derive(Debug, Parser)]
#[command(
    name = "kmip-lsp",
    version,
    about = "KMIPKit IDE Language Server & TTLV Inspector"
)]
struct Cli {
    /// Start LSP + JSON-RPC server over stdio (standard IDE integration mode).
    #[arg(long)]
    stdio: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Decode a TTLV payload from hex, base64, or a file and print JSON.
    Decode {
        #[arg(long, conflicts_with_all = ["base64", "file"])]
        hex: Option<String>,
        #[arg(long, conflicts_with_all = ["hex", "file"])]
        base64: Option<String>,
        #[arg(long, conflicts_with_all = ["hex", "base64"])]
        file: Option<PathBuf>,
        /// Explicitly reveal sensitive fields (redacted by default).
        #[arg(long)]
        reveal_sensitive: bool,
    },
    /// Encode a JSON TTLV node tree into hexadecimal TTLV.
    Encode {
        #[arg(long)]
        file: PathBuf,
    },
    /// Look up an OASIS KMIP tag or name in the specification catalog.
    Lookup {
        query: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.stdio || cli.command.is_none() {
        let stdin = io::stdin();
        let stdout = io::stdout();
        if let Err(err) = run_stdio_server(stdin.lock(), stdout.lock()) {
            eprintln!("kmip-lsp stdio error: {err}");
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    match cli.command {
        Some(Commands::Decode {
            hex,
            base64,
            file,
            reveal_sensitive,
        }) => {
            let (raw_input, format) = if let Some(h) = hex {
                (h, DecodeFormat::Hex)
            } else if let Some(b) = base64 {
                (b, DecodeFormat::Base64)
            } else if let Some(path) = file {
                match fs::read(&path) {
                    Ok(bytes) => match String::from_utf8(bytes.clone()) {
                        Ok(text) => (text, DecodeFormat::Auto),
                        Err(_) => (::hex::encode(&bytes), DecodeFormat::Hex),
                    },
                    Err(err) => {
                        eprintln!("Failed to read file {}: {err}", path.display());
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                eprintln!("Provide one of --hex, --base64, or --file");
                return ExitCode::FAILURE;
            };

            let options = DecodeOptions {
                format,
                reveal_all: reveal_sensitive,
                reveal_offsets: Vec::new(),
            };
            match decode_ttlv(&raw_input, &options) {
                Ok(result) => match serde_json::to_string_pretty(&result) {
                    Ok(json) => {
                        println!("{json}");
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("Serialization error: {err}");
                        ExitCode::FAILURE
                    }
                },
                Err(err) => {
                    eprintln!("TTLV decode error: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Encode { file }) => {
            let content = match fs::read_to_string(&file) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("Failed to read {}: {err}", file.display());
                    return ExitCode::FAILURE;
                }
            };
            let node: EncodeNodeInput = match serde_json::from_str(&content) {
                Ok(n) => n,
                Err(err) => {
                    eprintln!("Invalid JSON node input: {err}");
                    return ExitCode::FAILURE;
                }
            };
            match encode_ttlv(&node) {
                Ok(encoded) => match serde_json::to_string_pretty(&encoded) {
                    Ok(json) => {
                        println!("{json}");
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("Serialization error: {err}");
                        ExitCode::FAILURE
                    }
                },
                Err(err) => {
                    eprintln!("TTLV encode error: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Lookup { query }) => {
            let matches = kmip_lsp::spec::catalog::lookup(&query);
            match serde_json::to_string_pretty(&matches) {
                Ok(json) => {
                    println!("{json}");
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("Serialization error: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        None => ExitCode::SUCCESS,
    }
}
