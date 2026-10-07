use std::process::Command;

#[test]
fn transport_feature_graph_uses_http1_and_aws_lc_without_proxy_or_compression() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args([
            "tree",
            "--manifest-path",
            manifest
                .to_str()
                .expect("workspace manifest path is Unicode"),
            "--locked",
            "--offline",
            "--all-targets",
            "-p",
            "kmipkit-transport",
            "-e",
            "features",
            "--prefix",
            "none",
        ])
        .output()
        .expect("cargo tree can inspect the already-resolved offline graph");
    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let graph = String::from_utf8(output.stdout).expect("cargo tree output is UTF-8");
    assert!(graph.contains("hyper feature \"http1\""));
    assert!(graph.contains("hyper feature \"client\""));
    assert!(graph.contains("rustls feature \"aws_lc_rs\""));
    assert!(graph.contains("tokio-rustls feature \"aws_lc_rs\""));

    for forbidden in [
        "hyper feature \"http2\"",
        "feature \"client-proxy\"",
        "feature \"gzip\"",
        "feature \"brotli\"",
        "feature \"zstd\"",
        "feature \"compression\"",
        "rustls feature \"tls12\"",
        "rustls feature \"ring\"",
        "tokio-rustls feature \"ring\"",
        "reqwest v",
        "hyper-util v",
        "async-compression v",
    ] {
        assert!(
            !graph.contains(forbidden),
            "the transport dependency graph unexpectedly includes {forbidden}"
        );
    }
    assert!(
        !graph.lines().any(|line| line.starts_with("ring v")),
        "the transport graph must not include the ring TLS provider"
    );
}
