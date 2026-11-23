use std::process::Command;
use tempfile::NamedTempFile;

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rust-kmip-validator"))
        .args(args)
        .output()
        .expect("execute validator")
}

// KMIPKIT-0084-FR-001: planning must expose probes omitted by static profiles.
#[test]
fn plan_is_machine_readable_and_does_not_skip_profile_omissions() {
    let output = cli(&[
        "plan",
        "--capabilities",
        "../shared/profiles/cosmian-kms.json",
    ]);
    assert!(output.status.success());
    let plan: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("plan must be JSON");
    assert_eq!(plan["capability_policy"], "probe_all_applicable");
    assert_eq!(plan["cases"].as_array().expect("catalog rows").len(), 218);
}

// KMIPKIT-0084-FR-001/004: incomplete TLS files must fail offline preflight.
#[test]
fn preflight_reports_redacted_local_failure_without_a_network_exchange() {
    let output = cli(&[
        "preflight",
        "--connection",
        "../shared/profiles/connection.example.json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("preflight must return JSON");
    assert_eq!(report["network_attempted"], false);
    assert_eq!(report["ready"], false);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("/path/to/"));
}

// KMIPKIT-0084-FR-004/005: no fake successes are permitted with missing TLS.
#[test]
fn run_cannot_report_success_without_establishing_a_real_connection() {
    let output_file = NamedTempFile::new().expect("output file");
    let output = cli(&[
        "run",
        "--connection",
        "../shared/profiles/connection.example.json",
        "--capabilities",
        "../shared/profiles/cosmian-kms.json",
        "--output",
        output_file.path().to_str().expect("UTF-8 path"),
    ]);
    assert!(
        !output.status.success(),
        "missing TLS must not produce a successful mock run"
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("/path/to/"));
}

#[test]
fn draft_campaign_records_only_the_executed_local_constructor() {
    use std::io::Write;

    let mut plan = NamedTempFile::new().expect("plan file");
    let evidence = NamedTempFile::new().expect("evidence file");
    let symbol = "kmipkit_client::ClientBatchItem::create_group_draft";
    let obligation = "a".repeat(64);
    let matrix_digest = "b".repeat(64);
    for (language, api) in [("c", symbol), ("rust", "unknown"), ("rust", symbol)] {
        writeln!(
            plan,
            "{}",
            serde_json::json!({
                "language": language, "protocol_version": "3.0", "kind": "public_route",
                "action": "construct", "execution_status": "required",
                "obligation_id": obligation, "public_api_symbol": api
            })
        )
        .expect("write plan");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_kmip30-campaign"))
        .args(["--plan"])
        .arg(plan.path())
        .args([
            "--matrix-sha256",
            &matrix_digest,
            "--scope",
            "local",
            "--evidence",
        ])
        .arg(evidence.path())
        .output()
        .expect("execute campaign");
    assert!(output.status.success());
    let actual: serde_json::Value =
        serde_json::from_reader(evidence.as_file()).expect("valid campaign evidence");
    assert_eq!(actual["matrix_sha256"], matrix_digest);
    assert_eq!(
        actual["events"],
        serde_json::json!([{
            "obligation_id": obligation, "public_api_symbol": symbol, "status": "passed",
            "assertion": "draft_operation_code"
        }])
    );
}
