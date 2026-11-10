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
    assert_eq!(plan["cases"].as_array().expect("catalog rows").len(), 207);
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
