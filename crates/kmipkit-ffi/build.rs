use std::env;
use std::path::{Path, PathBuf};

fn compile_public_client_api_consumer(workspace_root: &Path, consumer_header: &Path) {
    let source = workspace_root.join("bindings/c/tests/public_client_api.c");
    println!("cargo:rerun-if-changed={}", source.display());
    cc::Build::new()
        .file(source)
        .include(consumer_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_public_client_api_consumer");
}

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_COVERAGE_C_CONSUMER");
    if env::var_os("CARGO_FEATURE_COVERAGE_C_CONSUMER").is_none() {
        return;
    }

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo provides the package manifest directory"),
    );
    let workspace_root = manifest_dir.join("../..");
    let public_header = workspace_root.join("bindings/c/include");
    let internal_header = workspace_root.join("tests/fixtures/api/generated");
    let target_deps = PathBuf::from(
        env::var_os("OUT_DIR").expect("Cargo provides the build-script output directory"),
    )
    .join("../../../deps");

    println!("cargo:rerun-if-changed={}", public_header.display());
    println!("cargo:rerun-if-changed={}", internal_header.display());
    println!("cargo:rustc-link-search=native={}", target_deps.display());

    compile_public_client_api_consumer(&workspace_root, &consumer_header);

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
    client_consumer.compile("kmipkit_client_abi_c_consumer");

    let message_model_consumer =
        workspace_root.join("bindings/c/tests/message_model_abi_consumer.c");
    println!(
        "cargo:rerun-if-changed={}",
        message_model_consumer.display()
    );
    cc::Build::new()
        .file(message_model_consumer)
        .include(&consumer_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_message_model_abi_c_consumer");

    let credential_type_consumer =
        workspace_root.join("bindings/c/tests/credential_type_abi_consumer.c");
    println!(
        "cargo:rerun-if-changed={}",
        credential_type_consumer.display()
    );
    cc::Build::new()
        .file(credential_type_consumer)
        .include(&consumer_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_credential_type_abi_c_consumer");

    let nonce_consumer = workspace_root.join("bindings/c/tests/nonce_abi_consumer.c");
    println!("cargo:rerun-if-changed={}", nonce_consumer.display());
    cc::Build::new()
        .file(nonce_consumer)
        .include(&consumer_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_nonce_abi_c_consumer");

    let credential_consumer = workspace_root.join("bindings/c/tests/credential_abi_consumer.c");
    println!("cargo:rerun-if-changed={}", credential_consumer.display());
    cc::Build::new()
        .file(credential_consumer)
        .include(&consumer_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_credential_abi_c_consumer");

    let credential_secret_consumer =
        workspace_root.join("bindings/c/tests/credential_secret_abi_consumer.c");
    println!(
        "cargo:rerun-if-changed={}",
        credential_secret_consumer.display()
    );
    cc::Build::new()
        .file(credential_secret_consumer)
        .include(&consumer_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11")
        .compile("kmipkit_credential_secret_abi_c_consumer");
}
