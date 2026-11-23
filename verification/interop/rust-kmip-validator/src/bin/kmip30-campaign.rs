//! Collect local evidence from the installed KMIP 3.0 draft public API.
#![forbid(unsafe_code)]

use clap::{Parser, ValueEnum};
use kmipkit::ClientBatchItem;
use kmipkit::ttlv::Structure;
use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Clone, Copy, ValueEnum)]
enum Scope {
    Local,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    matrix_sha256: String,
    #[arg(long)]
    scope: Scope,
    #[arg(long)]
    evidence: PathBuf,
    #[arg(long)]
    profile: Option<PathBuf>,
}

fn check(symbol: &str) -> Option<bool> {
    let (make, expected): (fn(Structure) -> ClientBatchItem, u32) =
        match symbol.strip_prefix("kmipkit_client::ClientBatchItem::")? {
            "create_group_draft" => (ClientBatchItem::create_group_draft, 0x3c),
            "obliterate_draft" => (ClientBatchItem::obliterate_draft, 0x3d),
            "create_user_draft" => (ClientBatchItem::create_user_draft, 0x3e),
            "create_credential_draft" => (ClientBatchItem::create_credential_draft, 0x3f),
            "deactivate_draft" => (ClientBatchItem::deactivate_draft, 0x40),
            "encapsulate_draft" => (ClientBatchItem::encapsulate_draft, 0x41),
            "decapsulate_draft" => (ClientBatchItem::decapsulate_draft, 0x42),
            _ => return None,
        };
    Some(make(Structure::new()).operation() == expected)
}

fn run(args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    let mut events = Vec::new();
    let mut all_passed = true;
    for line in BufReader::new(File::open(args.plan)?).lines() {
        let row: Value = serde_json::from_str(&line?)?;
        if row["language"] != "rust"
            || row["protocol_version"] != "3.0"
            || row["kind"] != "public_route"
            || row["action"] != "construct"
            || row["execution_status"] != "required"
        {
            continue;
        }
        let symbol = row["public_api_symbol"]
            .as_str()
            .ok_or("missing API symbol")?;
        let Some(passed) = check(symbol) else {
            continue;
        };
        let obligation = row["obligation_id"]
            .as_str()
            .ok_or("missing obligation ID")?;
        all_passed &= passed;
        events.push(json!({
            "obligation_id": obligation,
            "public_api_symbol": symbol,
            "status": if passed { "passed" } else { "failed" },
            "assertion": "draft_operation_code",
        }));
    }

    if events.is_empty() {
        return Err("no supported Rust draft constructor obligations in the plan".into());
    }
    let mut output = BufWriter::new(File::create(args.evidence)?);
    serde_json::to_writer(
        &mut output,
        &json!({
            "schema_version": "1.0",
            "run_id": uuid::Uuid::new_v4().to_string(),
            "scope": "local",
            "language": "rust",
            "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
            "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
            "matrix_sha256": args.matrix_sha256,
            "events": events,
        }),
    )?;
    output.flush()?;
    Ok(all_passed)
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(_) => {
            eprintln!("Rust draft campaign failed. Check the plan and evidence paths.");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::check;

    #[test]
    fn checks_each_draft_constructor_through_the_installed_crate() {
        for operation in [
            "create_group",
            "obliterate",
            "create_user",
            "create_credential",
            "deactivate",
            "encapsulate",
            "decapsulate",
        ] {
            let symbol = format!("kmipkit_client::ClientBatchItem::{operation}_draft");
            assert_eq!(check(&symbol), Some(true), "{symbol}");
        }
        assert_eq!(check("kmipkit_client::ClientBatchItem::unknown"), None);
    }
}
