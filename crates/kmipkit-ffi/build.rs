use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD");
    println!("cargo:rustc-check-cfg=cfg(kmipkit_pkcs11_candidate_test)");
    if env::var_os("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD")
        .is_some_and(|value| value == OsStr::new("1"))
    {
        println!("cargo:rustc-cfg=kmipkit_pkcs11_candidate_test");
    }

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_COVERAGE_C_CONSUMER");
    let feature_enabled = env::var_os("CARGO_FEATURE_COVERAGE_C_CONSUMER").is_some();
    if !feature_enabled {
        return;
    }

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo provides the package manifest directory"),
    );
    let workspace_root = manifest_dir.join("../..");
    let consumer_source = workspace_root.join("bindings/c/tests/extension_registry.c");
    let consumer_header = workspace_root.join("bindings/c/include");
    let internal_header = workspace_root.join("tests/fixtures/api/generated");
    let target_deps = PathBuf::from(
        env::var_os("OUT_DIR").expect("Cargo provides the build-script output directory"),
    )
    .join("../../../deps");

    println!("cargo:rerun-if-changed={}", consumer_source.display());
    println!("cargo:rerun-if-changed={}", consumer_header.display());
    println!("cargo:rerun-if-changed={}", internal_header.display());
    println!("cargo:rustc-link-search=native={}", target_deps.display());
    cc::Build::new()
        .file(consumer_source)
        .include(&consumer_header)
        .include(internal_header)
        .define("main", Some("kmipkit_extension_registry_c_consumer_main"))
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_extension_registry_c_consumer");

    let public_client_consumer = workspace_root.join("bindings/c/tests/client_abi_consumer.c");
    println!(
        "cargo:rerun-if-changed={}",
        public_client_consumer.display()
    );
    cc::Build::new()
        .file(public_client_consumer)
        .include(&consumer_header)
        .define("main", Some("kmipkit_client_abi_c_consumer_main"))
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_client_abi_c_consumer");
}
