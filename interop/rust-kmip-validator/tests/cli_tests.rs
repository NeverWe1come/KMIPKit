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
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    let catalog_content =
        std::fs::read_to_string("../shared/catalog/kmipkit-1.0-test-catalog.json").unwrap();
    let catalog: serde_json::Value = serde_json::from_str(&catalog_content).unwrap();
    let catalog_size = catalog["test_cases"].as_array().unwrap().len() as u64;
    let summary = &parsed["summary"];
    assert_eq!(summary["total_cases"].as_u64(), Some(catalog_size));
    assert_eq!(
        summary["passed"].as_u64().unwrap()
            + summary["failed"].as_u64().unwrap()
            + summary["unsupported_by_server"].as_u64().unwrap(),
        catalog_size
    );

    let md_content = std::fs::read_to_string(output_md.path()).unwrap();
    assert!(md_content.contains("KMIP 2.1 Conformance Matrix"));
    assert!(md_content.contains("✅"));
    assert!(md_content.contains("➖"));

    let html_content = std::fs::read_to_string(output_html.path()).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("✅ Passed"));
    assert!(html_content.contains("➖ Unsupported"));
}
