//! Independent Rust consumer of installed KMIPKit release packages.
#![forbid(unsafe_code)]
mod adapters;
mod driver;
mod incoming;
mod models;
mod report;
mod runner;

use clap::{Parser, Subcommand};
use models::{Catalog, Connection, Result, load};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(about = "KMIP 2.0/2.1 installed-library interoperability consumer")]
struct Cli { #[command(subcommand)] command: Command }
#[derive(Subcommand)]
enum Command {
    /// Offline catalog planning; profiles never suppress probes.
    Plan { #[arg(long)] capabilities: Option<PathBuf>, #[arg(long)] connection: Option<PathBuf>, #[arg(long)] catalog: Option<PathBuf> },
    /// Validate local configuration, TLS identity and package metadata without sockets.
    Preflight { #[arg(long)] connection: PathBuf, #[arg(long)] capabilities: Option<PathBuf>, #[arg(long)] catalog: Option<PathBuf> },
    /// Perform real exchanges once per request in a dedicated test environment.
    Run { #[command(flatten)] listener: incoming::ListenerOptions, #[arg(long)] connection: PathBuf, #[arg(long)] capabilities: PathBuf, #[arg(long)] catalog: Option<PathBuf>, #[arg(long)] output: Option<PathBuf>, #[arg(long)] markdown: Option<PathBuf> },
}
fn catalog_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared/catalog/kmipkit-1.0-test-catalog.json"))
}
fn read(path: &Path) -> Result<Vec<u8>> { std::fs::read(path).map_err(|_| "input file unavailable") }
fn emit(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value).map_err(|_| "report serialization failed")?);
    Ok(())
}
fn dispatch(cli: Cli) -> Result<bool> {
    match cli.command {
        Command::Plan { capabilities, connection, catalog } => {
            if let Some(path) = capabilities { let _: Value = load(&path)?; }
            let connection: Option<Connection> = connection.as_deref().map(load).transpose()?;
            if let Some(connection) = &connection { connection.validate()?; }
            let catalog: Catalog = load(&catalog_path(catalog))?;
            emit(&json!({ "capability_policy": "probe_all_applicable", "network_attempted": false,
                "generic_ttlv_outbound": "unavailable", "cases": catalog.test_cases.iter().map(|case| json!({"case_id": case.case_id,
                "disposition": connection.as_ref().and_then(|c| runner::classify(case, c)).map_or("probe", |v| v.status)})).collect::<Vec<_>>() }))?;
            Ok(true)
        }
        Command::Preflight { connection, capabilities, catalog } => {
            let result = (|| {
                let connection: Connection = load(&connection)?;
                let _: Catalog = load(&catalog_path(catalog))?;
                if let Some(path) = capabilities { let profile: Value = load(&path)?; if profile["server"]["id"] != connection.server_id { return Err("connection/profile server identity mismatch"); } }
                let _ = driver::client(&connection)?;
                Ok(())
            })();
            emit(&json!({"ready": result.is_ok(), "network_attempted": false, "diagnostic": result.err(),
                "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"), "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
                "generic_ttlv_outbound": "unavailable"}))?;
            Ok(result.is_ok())
        }
        Command::Run { listener, connection, capabilities, catalog, output, markdown } => {
            let configuration = read(&connection)?;
            let connection: Connection = serde_json::from_slice(&configuration).map_err(|_| "invalid connection contract")?;
            let profile: Value = load(&capabilities)?;
            if profile["server"]["id"] != connection.server_id { return Err("connection/profile server identity mismatch"); }
            let catalog = read(&catalog_path(catalog))?;
            let cases: Catalog = serde_json::from_slice(&catalog).map_err(|_| "invalid catalog contract")?;
            let mut client = driver::client(&connection)?;
            let mut listener = incoming::listener(&connection, &listener)?;
            let results = cases.test_cases.iter().map(|case| {
                if case.requires["workflow_type"] == "server_to_client" {
                    incoming::run(case, &connection, listener.as_mut())
                } else { runner::run_case(case, &connection, &mut client) }
            }).collect();
            let report = report::report(&connection, &profile, &configuration, &catalog, results)?;
            let success = report["summary"]["failed"] == 0;
            if let Some(path) = output { std::fs::write(path, serde_json::to_vec_pretty(&report).map_err(|_| "report serialization failed")?).map_err(|_| "report output unavailable")?; } else { emit(&report)?; }
            if let Some(path) = markdown { std::fs::write(path, report::markdown(&report)).map_err(|_| "markdown output unavailable")?; }
            Ok(success)
        }
    }
}
fn main() -> ExitCode {
    match dispatch(Cli::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => { eprintln!("{message}"); ExitCode::FAILURE }
    }
}
