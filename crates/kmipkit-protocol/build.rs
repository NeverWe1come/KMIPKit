use std::env;
use std::ffi::OsStr;
use std::path::{Component, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rustc-check-cfg=cfg(kmipkit_pkcs11_candidate_test)");
    let candidate_test_build = env::var_os("KMIPKIT_PKCS11_CANDIDATE_TEST_BUILD")
        .is_some_and(|value| value == OsStr::new("1"));
    reject_unrequested_candidate_cfg(candidate_test_build);
    if candidate_test_build {
        ensure_isolated_debug_target();
        println!("cargo:rustc-cfg=kmipkit_pkcs11_candidate_test");
    }
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
