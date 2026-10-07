use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_COVERAGE_C_CONSUMER");
    let feature_enabled = env::var_os("CARGO_FEATURE_COVERAGE_C_CONSUMER").is_some();
    let is_linux = env::var("CARGO_CFG_TARGET_OS").is_ok_and(|target| target == "linux");
    if !feature_enabled || !is_linux {
        return;
    }

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo provides the package manifest directory"),
    );
    let workspace_root = manifest_dir.join("../..");
    let consumer_source = workspace_root.join("bindings/c/tests/extension_registry.c");
    let consumer_header = workspace_root.join("bindings/c/include");
    let target_deps = PathBuf::from(
        env::var_os("OUT_DIR").expect("Cargo provides the build-script output directory"),
    )
    .join("../../../deps");

    println!("cargo:rerun-if-changed={}", consumer_source.display());
    println!("cargo:rerun-if-changed={}", consumer_header.display());
    println!("cargo:rustc-link-search=native={}", target_deps.display());
    cc::Build::new()
        .file(consumer_source)
        .include(consumer_header)
        .define(
            "main",
            Some("kmipkit_extension_registry_c_consumer_main"),
        )
        .flag_if_supported("-std=c11")
        .compile("kmipkit_extension_registry_c_consumer");
}
