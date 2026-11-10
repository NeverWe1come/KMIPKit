//! Bind installed-package provenance to the executable at build time.
use std::error::Error;
use std::path::Path;
use std::process::Command;
fn main() -> Result<(), Box<dyn Error>> {
    let manifest = Path::new("../.local/rust-live/provenance.json");
    for path in [
        manifest,
        Path::new("../.local/rust-live/kmipkit-rust-1.0.0.tar.gz"),
        Path::new("../.local/rust-live/install"),
        Path::new("tools/verify_artifact.py"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    if !Command::new("python3")
        .args(["tools/verify_artifact.py", "--quiet"])
        .status()?
        .success()
    {
        return Err("installed artifact integrity check failed".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(manifest)?)?;
    for (field, variable, length) in [
        ("library_commit", "KMIPKIT_LIBRARY_COMMIT", 40),
        ("artifact_sha256", "KMIPKIT_ARTIFACT_SHA256", 64),
    ] {
        let text = value[field].as_str().ok_or("provenance value is missing")?;
        if text.len() != length || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid provenance value".into());
        }
        println!("cargo:rustc-env={variable}={text}");
    }
    Ok(())
}
