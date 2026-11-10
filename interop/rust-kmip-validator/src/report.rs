//! Allowlisted report data: never serialize a library response or arbitrary error.
use crate::models::{CaseResult, Connection, Result, hash};
use serde_json::{Value, json};

pub(crate) fn report(connection: &Connection, profile: &Value, configuration: &[u8], catalog: &[u8], cases: Vec<CaseResult>) -> Result<Value> {
    if profile["server"]["id"].as_str() != Some(&connection.server_id) { return Err("connection/profile server identity mismatch"); }
    let server = &profile["server"];
    let product = server["product"].as_str().ok_or("server product metadata required")?;
    let version = server["version"].as_str().filter(|v| !v.is_empty()).unwrap_or("unspecified");
    let count = |status: &str| cases.iter().filter(|c| c.status == status).count();
    Ok(json!({
        "schema_version": "2.0", "run_id": format!("rust-{}", uuid::Uuid::new_v4()),
        "tested_at": chrono::Utc::now().to_rfc3339(), "language": "rust",
        "server": { "id": connection.server_id, "product": product, "version": version,
            "configuration_fingerprint": hash(configuration) },
        "protocol_version": connection.protocol_version, "catalog_sha256": hash(catalog),
        "execution": { "transport": connection.transport, "encoding": connection.encoding,
            "api_level": connection.api_level, "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
            "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256") },
        "summary": { "total": cases.len(), "passed": count("passed"), "failed": count("failed"),
            "unsupported": count("server_unsupported"), "policy_restricted": count("policy_restricted"), "not_run": count("not_run") },
        "cases": cases
    }))
}

pub(crate) fn markdown(report: &Value) -> String {
    let mut output = String::from("# KMIPKit interoperability evidence\n\nControlled observations do not establish vendor compatibility.\n\n| Case | Outcome | Library | Server | Parity |\n|---|---|---|---|---|\n");
    if let Some(cases) = report["cases"].as_array() {
        for case in cases {
            let field = |name: &str| case[name].as_str().unwrap_or("unknown").replace(['|', '\n', '\r', '<', '>'], "_");
            use std::fmt::Write;
            let _ = writeln!(output, "| {} | {} | {} | {} | {} |", field("case_id"), field("status"), field("library_correctness"), field("server_compatibility"), field("parity"));
        }
    }
    output
}
