use std::process::Command;

#[test]
fn lists_only_the_providers_compiled_into_the_binary() {
    let output = Command::new(env!("CARGO_BIN_EXE_kmipkit-proxy"))
        .arg("--list-providers")
        .output()
        .expect("proxy binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        kmipkit_proxy::providers::compiled_provider_names().join("\n")
    );
}

#[cfg(not(feature = "aws"))]
#[test]
fn provider_not_compiled_into_binary_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_kmipkit-proxy"))
        .args(["--provider", "aws", "--no-tls"])
        .output()
        .expect("proxy binary should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not compiled into this binary"));
}

#[test]
fn unavailable_provider_does_not_fall_back_to_the_mock() {
    let output = Command::new(env!("CARGO_BIN_EXE_kmipkit-proxy"))
        .args(["--provider", "not-a-provider", "--no-tls"])
        .output()
        .expect("proxy binary should run");

    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("defaulting to in-memory mock"));
}
