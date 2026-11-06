use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_cli_plan_and_run_cosmian_profile() {
    let output_json = NamedTempFile::new().unwrap();
    let output_md = NamedTempFile::new().unwrap();
    let output_html = NamedTempFile::new().unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_rust-kmip-validator"))
        .args([
            "run",
            "--connection",
            "../shared/profiles/connection.example.json",
            "--capabilities",
            "../shared/profiles/cosmian-kms.json",
            "--catalog",
            "../shared/catalog/kmipkit-1.0-test-catalog.json",
            "--output",
            output_json.path().to_str().unwrap(),
            "--markdown",
            output_md.path().to_str().unwrap(),
            "--html",
            output_html.path().to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute CLI");

    assert!(status.success());

    let json_content = std::fs::read_to_string(output_json.path()).unwrap();
    assert!(json_content.contains("\"passed\": 40"));
    assert!(json_content.contains("\"unsupported_by_server\": 83"));

    let md_content = std::fs::read_to_string(output_md.path()).unwrap();
    assert!(md_content.contains("KMIP 2.1 Conformance Matrix"));
    assert!(md_content.contains("✅"));
    assert!(md_content.contains("➖"));

    let html_content = std::fs::read_to_string(output_html.path()).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("✅ Passed"));
    assert!(html_content.contains("➖ Unsupported"));
}
