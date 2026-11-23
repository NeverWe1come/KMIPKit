//! The read-only typed Query exchange, joined to a shared campaign obligation.

use crate::driver;
use crate::models::{self, Connection, Result};
use clap::Args;
use kmipkit::protocol::{QueryFunction, QueryRequest};
use kmipkit::ttlv::codec::CodecLimits;
use kmipkit::{ClientBatchOutcome, ClientRequest, ResultReason};
use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const CONSTRUCTOR: &str = "kmipkit_protocol::QueryRequest::new";
const EXECUTOR: &str = "kmipkit_client::Client::execute";

#[derive(Args)]
pub(crate) struct QueryArgs {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    matrix_sha256: String,
    #[arg(long, value_parser = ["live"])]
    scope: String,
    #[arg(long)]
    evidence: PathBuf,
    #[arg(long)]
    profile: Option<PathBuf>,
    #[arg(long)]
    connection: PathBuf,
}

fn select_obligation(plan: &Path, connection: &Connection) -> Result<String> {
    let input = File::open(plan).map_err(|_| "campaign plan is unavailable")?;
    let expected = [
        ("language", "rust"),
        ("protocol_version", connection.protocol_version.as_str()),
        ("encoding", connection.encoding.as_str()),
        ("transport", connection.transport.as_str()),
        ("case_id", "test-op-query"),
        ("target_id", "KMIPKIT-ELEM-OP-C2S-QUERY"),
        ("public_api_symbol", CONSTRUCTOR),
        ("execution_api_symbol", EXECUTOR),
        ("action", "send"),
        ("execution_status", "required"),
    ];
    let mut obligation = None;
    for line in BufReader::new(input).lines() {
        let row: Value = serde_json::from_str(&line.map_err(|_| "invalid campaign plan")?)
            .map_err(|_| "invalid campaign plan")?;
        if expected.iter().all(|(key, value)| row[key] == *value) {
            if obligation.is_some() {
                return Err("duplicate typed Query obligations");
            }
            obligation = row["obligation_id"].as_str().map(str::to_owned);
        }
    }
    obligation.ok_or("no matching typed Query send obligation")
}

fn exchange(
    connection: &Connection,
) -> Result<(&'static str, &'static str, Option<u32>, Option<u32>)> {
    let mut client = driver::client(connection)?;
    let request = ClientRequest::query(QueryRequest::new([QueryFunction::OPERATIONS]));
    let batch = driver::authenticated_batch(connection, request)?;
    let Ok(response) = client.execute(batch, &CodecLimits::defaults()) else {
        return Ok(("failed", "unknown", None, None));
    };
    if response.len() != 1 {
        return Ok(("failed", "unknown", None, None));
    }
    let Some(ClientBatchOutcome::Query(query)) = response
        .get(0)
        .map(kmipkit::ClientBatchItemResponse::outcome)
    else {
        return Ok(("failed", "unknown", None, None));
    };
    let status = query.result().status().raw();
    let reason = query.result().reason().map(ResultReason::raw);
    let (check, compatibility) = match (status, reason) {
        (0, _) if query.operations().next().is_some() => ("passed", "supported"),
        (1, Some(5 | 8)) => ("failed", "unsupported"),
        (1, Some(3 | 12)) => ("blocked", "policy_restricted"),
        _ => ("failed", "unknown"),
    };
    Ok((check, compatibility, Some(status), reason))
}

pub(crate) fn run(args: &QueryArgs) -> Result<bool> {
    let connection: Connection = models::load(&args.connection)?;
    if connection.api_level != "typed-protocol" {
        return Err("the Query campaign requires the typed protocol API");
    }
    let configuration: Value = models::load(&args.connection)?;
    let metadata = |field: &str| {
        configuration[field].as_str().filter(|value| !value.is_empty())
            .ok_or("live evidence requires an explicit server product, version and configuration fingerprint")
    };
    let server = json!({
        "id": connection.server_id,
        "product": metadata("server_product")?,
        "version": metadata("server_version")?,
        "configuration_fingerprint": metadata("server_configuration_fingerprint")?,
    });
    let obligation = select_obligation(&args.plan, &connection)?;
    let (status, compatibility, code, reason) = exchange(&connection)?;
    let mut event = json!({
        "obligation_id": obligation, "public_api_symbol": CONSTRUCTOR, "execution_api_symbol": EXECUTOR,
        "status": status, "server_compatibility": compatibility,
        "encoding": connection.encoding, "transport": connection.transport,
        "assertion": "query_operations_and_typed_response",
    });
    if let Some(code) = code {
        event["result_status_code"] = json!(code);
    }
    if let Some(reason) = reason {
        event["result_reason_code"] = json!(reason);
    }
    let evidence = json!({
        "schema_version": "1.0", "run_id": uuid::Uuid::new_v4().to_string(), "language": "rust",
        "scope": "live", "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
        "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
        "matrix_sha256": args.matrix_sha256, "server": server, "events": [event],
    });
    let output = File::options()
        .write(true)
        .create_new(true)
        .open(&args.evidence)
        .map_err(|_| "campaign evidence output unavailable")?;
    serde_json::to_writer(output, &evidence)
        .map_err(|_| "failed to serialize campaign evidence")?;
    Ok(status != "failed")
}
