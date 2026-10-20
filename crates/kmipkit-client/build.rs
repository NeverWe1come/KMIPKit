use std::env;
use std::ffi::OsStr;
use std::path::{Component, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    println!("cargo:rustc-check-cfg=cfg(kmipkit_pkcs11_candidate_test)");
    if env::var_os("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD")
        .is_some_and(|value| value == OsStr::new("1"))
    {
        ensure_isolated_debug_target();
        println!("cargo:rustc-cfg=kmipkit_pkcs11_candidate_test");
    }
}

fn ensure_isolated_debug_target() {
    if env::var_os("DEBUG").is_some_and(|value| value == OsStr::new("false")) {
        eprintln!("the PKCS#11 candidate test build is restricted to debug profiles");
        std::process::exit(1);
    }
    let Some(target_directory) = env::var_os("CARGO_TARGET_DIR") else {
        eprintln!("the PKCS#11 candidate test build requires an isolated CARGO_TARGET_DIR");
        std::process::exit(1);
    };
    let target_directory = PathBuf::from(target_directory);
    let has_candidate_component = target_directory.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        name == "pkcs11-candidate" || name.starts_with("pkcs11-candidate-")
    });
    let contains_parent_traversal = target_directory
        .components()
        .any(|component| matches!(component, Component::ParentDir));
    if !has_candidate_component || contains_parent_traversal {
        eprintln!("CARGO_TARGET_DIR must identify an isolated pkcs11-candidate directory");
        std::process::exit(1);
    }
}
