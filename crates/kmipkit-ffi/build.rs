use std::env;
use std::ffi::OsStr;
use std::path::{Component, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rustc-check-cfg=cfg(kmipkit_pkcs11_candidate_test)");
    let candidate_cfg_authorization = if env::var_os("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD")
        .is_some_and(|value| value == OsStr::new("1"))
    {
        ensure_isolated_debug_target();
        "isolated"
    } else {
        "blocked"
    };
    println!(
        "cargo:rustc-env=KMIPKIT_PKCS11_CANDIDATE_CFG_AUTHORIZATION={candidate_cfg_authorization}"
    );
    let candidate_test_build = env::var_os("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD")
        .is_some_and(|value| value == OsStr::new("1"));
    reject_unrequested_candidate_cfg(candidate_test_build);
    if candidate_test_build {
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
    let mut client_consumer = cc::Build::new();
    client_consumer
        .file(public_client_consumer)
        .include(&consumer_header)
        .define("main", Some("kmipkit_client_abi_c_consumer_main"))
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11");
    if candidate_test_build {
        client_consumer.define("KMIPKIT_PKCS11_CANDIDATE_TEST", None);
    }
    client_consumer.compile("kmipkit_client_abi_c_consumer");
}

fn reject_unrequested_candidate_cfg(candidate_test_build: bool) {
    if candidate_test_build {
        return;
    }

    let encoded_flags_request_candidate = env::var("CARGO_ENCODED_RUSTFLAGS")
        .is_ok_and(|flags| has_candidate_cfg_argument(flags.split('\u{1f}')));
    let raw_flags_request_candidate = env::var("RUSTFLAGS")
        .is_ok_and(|flags| has_candidate_cfg_argument(flags.split_whitespace()));
    if encoded_flags_request_candidate || raw_flags_request_candidate {
        eprintln!("candidate cfg must be enabled only through the isolated build opt-in");
        std::process::exit(1);
    }
}

fn has_candidate_cfg_argument<'a>(arguments: impl IntoIterator<Item = &'a str>) -> bool {
    let arguments: Vec<_> = arguments.into_iter().collect();
    arguments.iter().enumerate().any(|(index, argument)| {
        let value = if *argument == "--cfg" {
            arguments.get(index + 1).copied()
        } else {
            argument.strip_prefix("--cfg=")
        };
        value.is_some_and(|value| {
            value.trim_matches('\"').split('=').next() == Some("kmipkit_pkcs11_candidate_test")
        })
    })
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
