//! Pin the release identity into the consumer; deployment credentials never enter build output.
use std::path::Path;
fn main() {
    let manifest = Path::new("../.local/rust-live/provenance.json");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(manifest).expect("install the pinned Rust package artifact first");
    let value: serde_json::Value = serde_json::from_str(&text).expect("artifact provenance JSON");
    println!("cargo:rustc-env=KMIPKIT_LIBRARY_COMMIT={}", value["library_commit"].as_str().expect("release commit"));
    println!("cargo:rustc-env=KMIPKIT_ARTIFACT_SHA256={}", value["artifact_sha256"].as_str().expect("artifact SHA-256"));
}
